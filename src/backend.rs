use serde_json::Value;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex, OnceLock,
    },
    time::Duration,
};

/// Procesos hijos vivos, para terminarlos cuando se cierra la app.
struct Proc {
    pid: u32,
    keep: bool,
    alive: Arc<AtomicBool>,
}

fn procs() -> &'static Mutex<Vec<Proc>> {
    static PROCS: OnceLock<Mutex<Vec<Proc>>> = OnceLock::new();
    PROCS.get_or_init(|| Mutex::new(vec![]))
}

fn register(pid: u32, keep: bool) -> Arc<AtomicBool> {
    let alive = Arc::new(AtomicBool::new(true));
    if let Ok(mut g) = procs().lock() {
        g.push(Proc { pid, keep, alive: alive.clone() });
    }
    alive
}

#[cfg(unix)]
fn terminate(pid: u32, force: bool) {
    let target = format!("-{pid}");
    let sig = if force { "-KILL" } else { "-TERM" };
    let _ = Command::new("kill")
        .args([sig, target.as_str()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(windows)]
fn terminate(pid: u32, _force: bool) {
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Alguna vez se lanzó un juego con `play`: el demonio debe seguir vivo para
/// que el juego no se cierre al salir de la interfaz.
static PLAY_LAUNCHED: AtomicBool = AtomicBool::new(false);

/// Termina los trabajos propios al salir (los juegos lanzados con `play` se
/// dejan abiertos) y apaga el demonio de sesión si no queda nada vivo.
pub fn shutdown() {
    let items: Vec<(u32, bool)> = {
        let Ok(mut g) = procs().lock() else { return };
        g.retain(|p| p.alive.load(Ordering::Relaxed));
        g.iter().map(|p| (p.pid, p.keep)).collect()
    };
    let victims: Vec<u32> = items.iter().filter(|(_, k)| !*k).map(|(p, _)| *p).collect();
    for pid in &victims {
        terminate(*pid, false);
    }
    if !victims.is_empty() {
        std::thread::sleep(Duration::from_millis(180));
        for pid in &victims {
            terminate(*pid, true);
        }
    }
    if !PLAY_LAUNCHED.load(Ordering::Relaxed) {
        let _ = Command::new(bin())
            .args(["daemon", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn stream(mut child: std::process::Child, job: usize, tx: Sender<Msg>) -> (Option<i32>, String) {
    let out = child.stdout.take();
    let err = child.stderr.take();
    let h_out = out.map(|out| {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut buf = String::new();
            for l in BufReader::new(out).lines().map_while(Result::ok) {
                buf.push_str(&l);
                buf.push('\n');
                let _ = tx.send(Msg::Line { job, err: false, text: l });
            }
            buf
        })
    });
    let h_err = err.map(|err| {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for l in BufReader::new(err).lines().map_while(Result::ok) {
                let _ = tx.send(Msg::Line { job, err: true, text: l });
            }
        })
    });
    let stdout = h_out.and_then(|h| h.join().ok()).unwrap_or_default();
    if let Some(h) = h_err {
        let _ = h.join();
    }
    let code = child.wait().ok().and_then(|s| s.code());
    (code, stdout)
}

pub fn bin() -> String {
    std::env::var("ASPIRE_AURELIA_BIN").unwrap_or_else(|_| "aurelia".into())
}

pub enum Msg {
    Line { job: usize, err: bool, text: String },
    Done { job: usize, code: Option<i32>, stdout: String },
}

/// Ejecuta `aurelia --json <args>` en segundo plano, enviando cada línea al canal.
pub fn spawn_job(job: usize, args: Vec<String>, tx: Sender<Msg>) {
    std::thread::spawn(move || {
        let first = args.first().cloned().unwrap_or_default();
        // `play` se deja abierto al cerrar la app y no se fuerza a JSON.
        let keep = first == "play";
        if keep {
            PLAY_LAUNCHED.store(true, Ordering::Relaxed);
        }
        let mut cmd = Command::new(bin());
        if !keep && !args.iter().any(|a| a == "--json" || a == "-j") {
            cmd.arg("--json");
        }
        cmd.args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Msg::Line { job, err: true, text: format!("no se pudo ejecutar `{}`: {e}", bin()) });
                let _ = tx.send(Msg::Done { job, code: None, stdout: String::new() });
                return;
            }
        };
        let alive = register(child.id(), keep);
        let (code, stdout) = stream(child, job, tx.clone());
        alive.store(false, Ordering::Relaxed);
        let _ = tx.send(Msg::Done { job, code, stdout });
    });
}

/// Ejecuta un comando de shell local (tamaño en disco, estado de extensiones…).
pub fn spawn_shell(job: usize, command: String, tx: Sender<Msg>) {
    std::thread::spawn(move || {
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(&command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Msg::Line { job, err: true, text: format!("no se pudo ejecutar el comando: {e}") });
                let _ = tx.send(Msg::Done { job, code: Some(1), stdout: String::new() });
                return;
            }
        };
        let alive = register(child.id(), false);
        let (code, stdout) = stream(child, job, tx.clone());
        alive.store(false, Ordering::Relaxed);
        let _ = tx.send(Msg::Done { job, code, stdout });
    });
}

/// Trabajo local (sin aurelia), p. ej. aplicar parches.
pub fn spawn_local<F>(job: usize, tx: Sender<Msg>, f: F)
where
    F: FnOnce(&dyn Fn(String)) -> anyhow::Result<String> + Send + 'static,
{
    std::thread::spawn(move || {
        let tx_log = tx.clone();
        let log = move |s: String| {
            let _ = tx_log.send(Msg::Line { job, err: false, text: s });
        };
        match f(&log) {
            Ok(s) => {
                let _ = tx.send(Msg::Line { job, err: false, text: s });
                let _ = tx.send(Msg::Done { job, code: Some(0), stdout: String::new() });
            }
            Err(e) => {
                let _ = tx.send(Msg::Line { job, err: true, text: format!("{e:#}") });
                let _ = tx.send(Msg::Done { job, code: Some(1), stdout: String::new() });
            }
        }
    });
}

/// Comandos que piden teclado (login, chat…): se ejecutan con la terminal completa.
pub fn run_interactive(args: &[String]) -> std::io::Result<()> {
    if args.first().map(|s| s.as_str()) == Some("__shell") {
        if let Some(cmd) = args.get(1) {
            let status = Command::new("bash").arg("-c").arg(cmd).status()?;
            println!("\n[terminó con código {}] — pulsa Enter para volver…", status.code().unwrap_or(-1));
            let mut s = String::new();
            std::io::stdin().read_line(&mut s)?;
        }
        return Ok(());
    }
    if args.first().map(|s| s.as_str()) == Some("self-update") {
        // Actualizar Aspire-Team desde GitHub
        let script = r#"
            set -e
            echo "▸ Actualizando Aspire-Team..."
            DIR="$HOME/.local/share/aspire-team-src"
            if [ ! -d "$DIR/.git" ]; then
                mkdir -p "$(dirname "$DIR")"
                git clone --depth 50 https://github.com/zxvsho/Aspire-Team.git "$DIR"
            fi
            cd "$DIR"
            git pull origin main 2>/dev/null || echo "(sin cambios)"
            cargo build --release 2>&1 | tail -1
            cargo install --path . --force 2>&1 | tail -1
            echo ""
            echo "✓ Aspire-Team actualizado. Reinicia la app."
        "#;
        let status = std::process::Command::new("bash")
            .arg("-c")
            .arg(script)
            .status()?;
        println!("\n[actualización terminó con código {}] — pulsa Enter para volver…", status.code().unwrap_or(-1));
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        return Ok(());
    }
    if args.first().map(|s| s.as_str()) == Some("update-aurelia") {
        let script = r#"
            echo "▸ Actualizando Aurelia..."
            if command -v pacman >/dev/null 2>&1; then
                sudo pacman -Sy --noconfirm aurelia 2>&1 || true
            elif command -v apt-get >/dev/null 2>&1; then
                sudo apt-get update -qq 2>&1 && sudo apt-get install -y -qq --only-upgrade aurelia 2>&1 || true
            elif command -v dnf >/dev/null 2>&1; then
                sudo dnf upgrade -y aurelia 2>&1 || true
            elif command -v zypper >/dev/null 2>&1; then
                sudo zypper install -y aurelia 2>&1 || true
            else
                echo "(distro no reconocido: usando cargo install)"
            fi
            if ! command -v aurelia >/dev/null 2>&1; then
                cargo install aurelia --force 2>&1 | tail -1
            fi
            echo ""
            echo "✓ Aurelia actualizado. Versión: $(aurelia --version 2>&1 | head -1)"
        "#;
        let status = std::process::Command::new("bash")
            .arg("-c")
            .arg(script)
            .status()?;
        println!("\n[actualización terminó con código {}] — pulsa Enter para volver…", status.code().unwrap_or(-1));
        let mut s = String::new();
        std::io::stdin().read_line(&mut s)?;
        return Ok(());
    }
    let status = Command::new(bin()).args(args).status()?;
    println!("\n[aurelia terminó con código {}] — pulsa Enter para volver…", status.code().unwrap_or(-1));
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(())
}

pub fn is_interactive(args: &[String]) -> bool {
    let a: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    matches!(
        a.as_slice(),
        ["login", ..] | ["chat", "open", ..] | ["steam-runtime", "install" | "login" | "repair", ..] | ["self-update"] | ["update-aurelia"]
    )
}

pub fn parse_result(out: &str) -> Option<Value> {
    let t = out.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<Value>(t) {
        if v.get("event").and_then(|e| e.as_str()) == Some("progress") {
            return None;
        }
        return Some(v);
    }
    let mut last = None;
    for l in t.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(l.trim()) {
            match v.get("event").and_then(|e| e.as_str()) {
                Some("result") => return Some(v),
                Some("progress") => continue,
                _ => last = Some(v),
            }
        }
    }
    last
}

pub struct Game {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub raw: Value,
}

fn find_array(v: &Value, depth: u8) -> Option<&Vec<Value>> {
    match v {
        Value::Array(a) if a.is_empty() || a[0].is_object() => Some(a),
        Value::Object(m) if depth > 0 => {
            for k in ["games", "library", "items", "results", "data", "apps"] {
                if let Some(a) = m.get(k).and_then(|x| find_array(x, depth - 1)) {
                    return Some(a);
                }
            }
            m.values().find_map(|x| find_array(x, depth - 1))
        }
        _ => None,
    }
}

fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub fn extract_games(v: &Value) -> Vec<Game> {
    let Some(arr) = find_array(v, 3) else { return vec![] };
    arr.iter()
        .filter_map(|g| {
            let id = ["app_id", "appid", "appId", "id"].iter().find_map(|k| g.get(k).and_then(scalar))?;
            let name = ["name", "title"].iter().find_map(|k| g.get(k).and_then(scalar)).unwrap_or_else(|| format!("app {id}"));
            let installed = g.get("is_installed")
                .and_then(|x| x.as_bool())
                .or_else(|| g.get("installed").and_then(|x| x.as_bool()))
                .unwrap_or(false);
            Some(Game { id, name, installed, raw: g.clone() })
        })
        .collect()
}

pub fn install_dir(g: &Game) -> Option<String> {
    ["install_dir", "install_path", "installdir", "path", "dir"]
        .iter()
        .find_map(|k| g.raw.get(k).and_then(|x| x.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
}

/// Divide como una shell sencilla (respeta comillas dobles/simples).
pub fn split_args(s: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    let mut q: Option<char> = None;
    let mut has = false;
    for c in s.chars() {
        match (q, c) {
            (Some(qc), c) if c == qc => q = None,
            (Some(_), c) => cur.push(c),
            (None, '"') | (None, '\'') => {
                q = Some(c);
                has = true;
            }
            (None, c) if c.is_whitespace() => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    out
}
