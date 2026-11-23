use serde_json::Value;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::Sender,
};

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
        let spawned = Command::new(bin())
            .arg("--json")
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match spawned {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Msg::Line { job, err: true, text: format!("no se pudo ejecutar `{}`: {e}", bin()) });
                let _ = tx.send(Msg::Done { job, code: None, stdout: String::new() });
                return;
            }
        };
        let out = child.stdout.take().unwrap();
        let err = child.stderr.take().unwrap();
        let tx_out = tx.clone();
        let h_out = std::thread::spawn(move || {
            let mut buf = String::new();
            for l in BufReader::new(out).lines().map_while(Result::ok) {
                buf.push_str(&l);
                buf.push('\n');
                let _ = tx_out.send(Msg::Line { job, err: false, text: l });
            }
            buf
        });
        let tx_err = tx.clone();
        let h_err = std::thread::spawn(move || {
            for l in BufReader::new(err).lines().map_while(Result::ok) {
                let _ = tx_err.send(Msg::Line { job, err: true, text: l });
            }
        });
        let stdout = h_out.join().unwrap_or_default();
        let _ = h_err.join();
        let code = child.wait().ok().and_then(|s| s.code());
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
    if args.first().map(|s| s.as_str()) == Some("self-update") {
        // Actualizar Aspire-Team desde GitHub
        let script = r#"
            set -e
            echo "▸ Actualizando Aspire-Team..."
            cd ~/.local/share/aspire-team-src 2>/dev/null || {
                mkdir -p ~/.local/share
                git clone https://github.com/zxvsho/Aspire-Team.git ~/.local/share/aspire-team-src
                cd ~/.local/share/aspire-team-src
            }
            git pull origin main
            cargo build --release
            cargo install --path . --force
            echo ""
            echo "✔ Aspire-Team actualizado. Reinicia la app."
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
            set -e
            echo "▸ Actualizando Aurelia..."
            if command -v pacman >/dev/null 2>&1; then
                sudo pacman -Sy --noconfirm aurelia
            elif command -v apt-get >/dev/null 2>&1; then
                sudo apt-get update -qq && sudo apt-get install -y -qq --only-upgrade aurelia
            elif command -v dnf >/dev/null 2>&1; then
                sudo dnf upgrade -y aurelia
            else
                cargo install aurelia --force
            fi
            echo ""
            echo "✔ Aurelia actualizado. Versión: $(aurelia --version 2>&1 | head -1)"
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
        return Some(v);
    }
    let mut last = None;
    for l in t.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(l.trim()) {
            if v.get("event").and_then(|e| e.as_str()) == Some("result") {
                return Some(v);
            }
            last = Some(v);
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

fn show(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "—".into(),
        other => other.to_string(),
    }
}

/// Aplana un JSON a líneas (sangría, clave, valor) para mostrarlo legible.
pub fn flatten(v: &Value, indent: usize, out: &mut Vec<(usize, String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if indent == 0 && (k == "event" || k == "schema") {
                    continue;
                }
                match x {
                    Value::Object(_) | Value::Array(_) => {
                        out.push((indent, k.clone(), String::new()));
                        flatten(x, indent + 1, out);
                    }
                    _ => out.push((indent, k.clone(), show(x))),
                }
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                if i >= 300 {
                    out.push((indent, "…".into(), format!("{} más (usa la vista cruda con `v`)", a.len() - 300)));
                    break;
                }
                match x {
                    Value::Object(m) if m.values().all(|y| !y.is_object() && !y.is_array()) => {
                        let line = m.iter().map(|(k, y)| format!("{k}: {}", show(y))).collect::<Vec<_>>().join("  ·  ");
                        out.push((indent, "•".into(), line));
                    }
                    Value::Object(_) | Value::Array(_) => {
                        out.push((indent, "•".into(), String::new()));
                        flatten(x, indent + 1, out);
                    }
                    _ => out.push((indent, "•".into(), show(x))),
                }
            }
        }
        other => out.push((indent, String::new(), show(other))),
    }
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
