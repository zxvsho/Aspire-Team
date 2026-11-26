//! Herramientas locales: abrir rutas, tamaño en disco, guardados, prefijos de
//! Wine y extensiones del sistema (MangoHud, Goverlay, vkBasalt, GameMode…).

use anyhow::{bail, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn open_path(p: &str) -> Result<()> {
    let opener = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener)
        .arg(p)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn open_url(u: &str) -> Result<()> {
    open_path(u)
}

fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

pub fn which(bin: &str) -> Option<String> {
    capture("bash", &["-c", &format!("command -v {bin} 2>/dev/null")]).filter(|s| !s.is_empty())
}

/// Estado de las extensiones que un cliente de Steam debería poder configurar.
pub fn extensions_status(log: &dyn Fn(String)) -> Result<String> {
    let items: [(&str, &str); 7] = [
        ("mangohud", "MangoHud — FPS, temperaturas y métricas"),
        ("goverlay", "Goverlay — configurar MangoHud/vkBasalt con interfaz"),
        ("vkbasalt", "vkBasalt — filtros y efectos (ReShade)"),
        ("gamemoderun", "GameMode — prioridad y rendimiento"),
        ("gamescope", "Gamescope — compositor con escalado"),
        ("protonup-qt", "ProtonUp-Qt — descargar Proton-GE/Wine-GE"),
        ("protonplus", "ProtonPlus — descargar Proton/Wine"),
    ];
    log("Extensiones del sistema".into());
    for (bin, name) in items {
        match which(bin) {
            Some(p) => {
                log(format!("✓ {name}"));
                log(format!("    {p}"));
            }
            None => log(format!("○ {name} — no instalado")),
        }
    }
    log(String::new());
    log("Desde Complementos puedes instalar el paquete de extensiones y".into());
    log("activar MangoHud, GameMode o vkBasalt en el juego seleccionado.".into());
    Ok(String::new())
}

pub fn disk_info(dir: &str, log: &dyn Fn(String)) -> Result<String> {
    log(format!("Carpeta: {dir}"));
    if let Some(o) = capture("du", &["-sh", dir]) {
        let size = o.split_whitespace().next().unwrap_or("?");
        log(format!("Tamaño en disco: {size}"));
    }
    if let Some(o) = capture("df", &["-h", dir]) {
        log("Disco:".into());
        for l in o.lines() {
            log(format!("  {l}"));
        }
    }
    if let Some(o) = capture("stat", &["-f", "-c", "Sistema de archivos: %T", dir]) {
        log(o);
    }
    Ok(String::new())
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = vec![];
    if let Some(h) = dirs::home_dir() {
        roots.push(h.join(".local/share/Steam"));
        roots.push(h.join(".steam/steam"));
    }
    roots
}

pub fn saves_dir(app: &str) -> Option<PathBuf> {
    for root in steam_roots() {
        let userdata = root.join("userdata");
        let Ok(rd) = std::fs::read_dir(&userdata) else { continue };
        for e in rd.flatten() {
            let p = e.path().join(app).join("remote");
            if p.is_dir() {
                return Some(p);
            }
        }
    }
    None
}

pub fn prefix_dir(app: &str) -> Option<PathBuf> {
    for root in steam_roots() {
        let p = root.join("steamapps/compatdata").join(app).join("pfx");
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

fn launch_options(app: &str) -> Result<String> {
    let out = Command::new(crate::backend::bin())
        .args(["--json", "config", "game", app])
        .output()?;
    if !out.status.success() {
        bail!("No pude leer la configuración del juego {app}.");
    }
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    Ok(v.get("launch_options")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string())
}

/// Añade o quita un prefijo de Steam launch options (`mangohud`, `gamemoderun`,
/// `ENABLE_VKBASALT=1`…). Si se pasa `!prefijo`, lo elimina.
pub fn launch_prefix(app: &str, prefix: &str, log: &dyn Fn(String)) -> Result<String> {
    let remove = prefix.starts_with('!');
    let p = prefix.trim_start_matches('!');
    let cur = launch_options(app)?;
    let mut words: Vec<String> = cur.split_whitespace().map(|s| s.to_string()).collect();
    words.retain(|w| w != p);
    if !remove {
        if p.contains('=') {
            words.insert(0, p.to_string());
        } else if let Some(i) = words.iter().position(|w| w == "%command%") {
            words.insert(i, p.to_string());
        } else {
            words.push(p.to_string());
            words.push("%command%".to_string());
        }
    }
    let newopts = words.join(" ");
    if newopts == cur {
        log("Sin cambios: el juego ya estaba así.".into());
        return Ok(format!("Opciones actuales: {}", if cur.is_empty() { "(ninguna)" } else { &cur }));
    }
    let out = Command::new(crate::backend::bin())
        .args(["config", "game", app, "--launch-options", &newopts])
        .output()?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr);
        bail!(
            "Aurelia no pudo guardar las opciones de lanzamiento. {}",
            msg.lines().last().unwrap_or("").trim()
        );
    }
    log(format!(
        "✓ Opciones de lanzamiento: {}",
        if newopts.is_empty() { "(ninguna)" } else { &newopts }
    ));
    Ok(String::new())
}

pub const EXT_INSTALL: &str = r#"
echo "▸ Instalando extensiones (MangoHud, Goverlay, vkBasalt, GameMode, Gamescope)…"
if command -v pacman >/dev/null 2>&1; then
  sudo pacman -S --needed --noconfirm mangohud lib32-mangohud goverlay vkbasalt lib32-vkbasalt gamemode lib32-gamemode gamescope || true
elif command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update -qq || true
  sudo apt-get install -y mangohud goverlay vkbasalt gamemode gamescope || true
elif command -v dnf >/dev/null 2>&1; then
  sudo dnf install -y mangohud goverlay vkbasalt gamemode gamescope || true
elif command -v zypper >/dev/null 2>&1; then
  sudo zypper install -y mangohud goverlay vkbasalt gamemode gamescope || true
else
  echo "No reconozco el gestor de paquetes. Instala mangohud, goverlay, vkbasalt, gamemode y gamescope a mano."
fi
echo ""
echo "Hecho. Vuelve a Complementos → Extensiones: estado para comprobarlo."
"#;

pub const EXT_GOVERLAY: &str = r#"
if command -v goverlay >/dev/null 2>&1; then
  goverlay
else
  echo "Goverlay no está instalado."
  echo "Instálalo en Complementos → Extensiones: instalar paquete."
fi
"#;

pub const EXT_PROTONUP: &str = r#"
echo "▸ Instalando ProtonUp-Qt (para descargar Proton-GE/Wine-GE)…"
if command -v flatpak >/dev/null 2>&1; then
  flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo || true
  flatpak install -y flathub net.davidotek.pupgui2 || true
elif command -v pacman >/dev/null 2>&1; then
  sudo pacman -S --needed --noconfirm protonup-qt || yay -S --noconfirm protonup-qt || true
elif command -v apt-get >/dev/null 2>&1; then
  sudo apt-get install -y protonup-qt || true
fi
echo ""
echo "Hecho. Ábrelo desde Complementos → ProtonUp-Qt / ProtonPlus."
"#;

pub const EXT_PROTONMANAGER: &str = r#"
if command -v protonup-qt >/dev/null 2>&1; then
  protonup-qt
elif command -v protonplus >/dev/null 2>&1; then
  protonplus
elif command -v flatpak >/dev/null 2>&1 && flatpak info net.davidotek.pupgui2 >/dev/null 2>&1; then
  flatpak run net.davidotek.pupgui2
elif command -v flatpak >/dev/null 2>&1 && flatpak info com.vysp3r.ProtonPlus >/dev/null 2>&1; then
  flatpak run com.vysp3r.ProtonPlus
else
  echo "No hay ningún gestor de Proton instalado."
  echo "Instala ProtonUp-Qt en Complementos → ProtonUp-Qt: instalar."
fi
"#;

/// Borra un archivo o carpeta del juego (con confirmación previa en la app).
pub fn remove_path(p: &std::path::Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(p)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(p)?;
    } else {
        std::fs::remove_file(p)?;
    }
    Ok(())
}
