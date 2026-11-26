//! Vistas organizadas para la salida de los comandos.
//!
//! En vez de volcar el JSON o el texto crudo de `aurelia`, cada comando tiene
//! una vista pensada para leerse de un vistazo: tablas compactas, fichas,
//! progreso con porcentaje y errores explicados en español.

use crate::{backend::Game, theme::Theme};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use serde_json::Value;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MAX_ROWS: usize = 250;

#[derive(Clone, Default)]
pub struct Progress {
    pub pct: f64,
    pub done: f64,
    pub total: f64,
    pub speed: f64,
    pub eta: f64,
    pub state: String,
}

// ---------------------------------------------------------------- utilidades

fn st(fg: Color) -> Style {
    Style::new().fg(fg)
}

fn blank() -> Line<'static> {
    Line::raw("")
}

fn note(s: &str, t: &Theme) -> Line<'static> {
    Line::styled(s.to_string(), st(t.dim).add_modifier(Modifier::ITALIC))
}

fn section(s: &str, t: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("╭─ ", st(t.dim)),
        Span::styled(s.to_uppercase(), st(t.accent2).add_modifier(Modifier::BOLD)),
    ])
}

fn bullet(s: String, t: &Theme) -> Line<'static> {
    Line::from(vec![Span::styled("  • ", st(t.accent2)), Span::styled(s, st(t.fg))])
}

fn trunc(s: &str, n: usize) -> String {
    let s = s.replace(['\n', '\r', '\t'], " ");
    if n == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(s.as_str()) <= n {
        return s;
    }
    let mut out = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if w + cw > n.saturating_sub(1) {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let width = width.max(16);
    let mut out: Vec<String> = vec![];
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let wl = UnicodeWidthStr::width(word);
        let cl = UnicodeWidthStr::width(cur.as_str());
        if !cur.is_empty() && cl + 1 + wl > width {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        if wl > width {
            // palabra larguísima: se corta
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(trunc(word, width));
            continue;
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn hb(n: f64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut x = n.max(0.0);
    let mut i = 0usize;
    while x >= 1024.0 && i + 1 < units.len() {
        x /= 1024.0;
        i += 1;
    }
    let s = if i == 0 { format!("{x:.0}") } else { format!("{x:.1}") };
    format!("{} {}", s.replace('.', ","), units[i])
}

fn dur(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    if s >= 3600 {
        format!("{} h {} min", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{} min {} s", s / 60, s % 60)
    } else {
        format!("{s} s")
    }
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn date(secs: i64) -> String {
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{d:02}/{m:02}/{y}")
}

fn is_ts(k: &str) -> bool {
    matches!(
        k,
        "date" | "start" | "end" | "created" | "last_logoff" | "date_added" | "updated_at" | "last_played"
    )
}

fn is_bytes(k: &str) -> bool {
    k.ends_with("_bytes")
        || matches!(
            k,
            "size" | "size_on_disk" | "download_size" | "disk_size" | "reused_bytes" | "depot_bytes_downloaded" | "depot_total_bytes"
        )
}

/// Etiquetas en español para los campos que devuelve Aurelia.
pub fn label(k: &str) -> String {
    let s = match k {
        "app_id" | "appid" | "id" => "AppID",
        "name" => "Nombre",
        "title" => "Título",
        "is_installed" | "installed" => "Instalado",
        "is_owned" | "owned" => "En tu cuenta",
        "price" => "Precio",
        "original_price" => "Precio original",
        "price_cents" => "Precio (céntimos)",
        "discount_pct" => "Descuento",
        "discount_end_date" => "Fin del descuento",
        "release_date" => "Lanzamiento",
        "platforms" => "Plataformas",
        "platform" => "Plataforma",
        "reviews" => "Reseñas",
        "type" => "Tipo",
        "developers" => "Desarrolladores",
        "publishers" => "Distribuidora",
        "description" => "Descripción",
        "full_description" => "Descripción completa",
        "short_description" => "Descripción",
        "genres" => "Géneros",
        "tags" => "Etiquetas",
        "store_url" => "Tienda",
        "update_available" => "Actualización",
        "playtime_forever_minutes" => "Tiempo jugado",
        "install_path" | "install_dir" | "path" | "dir" => "Ruta",
        "size_on_disk" => "Tamaño",
        "active_branch" | "branch" => "Rama",
        "country" => "País",
        "free_bytes" => "Espacio libre",
        "total" | "total_bytes" => "Total",
        "unlocked" => "Desbloqueados",
        "achievements" => "Logros",
        "players" => "Jugadores",
        "persona_name" => "Nombre",
        "steam_id" => "SteamID",
        "account_name" => "Cuenta",
        "email" => "Correo",
        "email_validated" => "Correo verificado",
        "vac_bans" => "Baneos VAC",
        "vac_banned_apps" => "Juegos con VAC",
        "daemon_logon_id" => "ID de sesión",
        "authed_machines" => "Equipos autorizados",
        "flags" => "Flags",
        "visibility" => "Visibilidad",
        "is_public" => "Perfil público",
        "is_limited" => "Cuenta limitada",
        "created" => "Creado",
        "last_logoff" => "Última conexión",
        "real_name" => "Nombre real",
        "location" => "Ubicación",
        "headline" => "Titular",
        "summary" => "Resumen",
        "profile_url" => "Perfil",
        "avatar_url" => "Avatar",
        "persona_state" => "Estado",
        "game_app_id" => "AppID del juego",
        "game_name" => "Jugando ahora",
        "relationship" => "Relación",
        "author" => "Autor",
        "contents" => "Contenido",
        "feed" => "Fuente",
        "url" => "Enlace",
        "date" => "Fecha",
        "kind" => "Tipo",
        "gid" => "ID",
        "associated_id" => "AppID asociado",
        "associated_name" => "Juego asociado",
        "start" => "Inicio",
        "end" => "Fin",
        "regions" => "Regiones",
        "available" => "Disponible",
        "purchase_options" => "Opciones de compra",
        "libraries" => "Bibliotecas",
        "depots" => "Depots",
        "manifests" => "Manifiestos",
        "branches" => "Ramas",
        "results" => "Resultados",
        "items" => "Elementos",
        "running" => "En ejecución",
        "state" => "Estado",
        "progress" => "Progreso",
        "bytes_downloaded" => "Descargado",
        "speed_bps" => "Velocidad",
        "eta_seconds" => "Tiempo restante",
        "depot_percent" => "Progreso del depot",
        "depot_name" => "Depot",
        "depot_id" => "ID del depot",
        "manifest_id" => "ID del manifiesto",
        "file_count" => "Archivos",
        "config" => "Configuración",
        "launch_options" => "Opciones de lanzamiento",
        "launch_option" => "Opción de lanzamiento",
        "launch_options_list" => "Opciones de lanzamiento",
        "arguments" => "Argumentos",
        "executable" => "Ejecutable",
        "working_dir" => "Carpeta de trabajo",
        "oslist" => "Sistema",
        "osarch" => "Arquitectura",
        "targets" => "Destinos",
        "default" => "Predeterminado",
        "missing" => "Falta",
        "steam_mode" => "Modo Steam",
        "language" => "Idioma",
        "languages" => "Idiomas",
        "default_language" => "Idioma predeterminado",
        "installed_language" => "Idioma instalado",
        "depot_count" => "Depots",
        "download_size" => "Descarga",
        "disk_size" => "En disco",
        "is_free" => "Gratis",
        "coming_soon" => "Próximamente",
        "early_access" => "Acceso anticipado",
        "rating" => "Valoración",
        "franchises" => "Sagas",
        "links" => "Enlaces",
        "screenshots" => "Capturas",
        "trailers" => "Tráilers",
        "dlc" => "DLC",
        "achievement" => "Logro",
        "api_name" => "Nombre interno",
        "hidden" => "Oculto",
        "global_percent" => "Rareza global",
        "unlock_time" => "Desbloqueado",
        "achieved" => "Conseguido",
        "disabled" => "Deshabilitado",
        "image_url" => "Imagen",
        "image_fallback_url" => "Imagen alternativa",
        "pid" => "PID",
        "wallet" | "balance" => "Saldo",
        "collections" => "Colecciones",
        "entries" => "Elementos",
        "steam_runtime_runner" => "Runner del Steam runtime",
        "steam_prefix_mode" => "Modo del prefijo",
        "proton_version" => "Proton predeterminado",
        "steam_library_path" => "Biblioteca de Steam",
        "chat_presence" => "Presencia en el chat",
        "enable_cloud_sync" => "Sincronización en la nube",
        "use_shared_compat_data" => "Compatdata compartida",
        "windows_steam_discovery_enabled" => "Detectar Steam de Windows",
        "preferred_launch_options" => "Opciones de lanzamiento preferidas",
        "game_configs" => "Configuración por juego",
        "luxtorpeda_enabled" => "Luxtorpeda activo",
        "luxtorpeda_path" => "Ruta de Luxtorpeda",
        "umu_enabled" => "umu activo",
        "umu_path" => "Ruta de umu",
        "steam_runtime_policy" => "Política del Steam runtime",
        "install_resume" => "Reanudar instalaciones",
        "shader_precache" => "Precaché de shaders",
        "hide_uninstalled_launch_options" => "Ocultar opciones no instaladas",
        "anticheat_auto_install" => "Instalar anticheat automáticamente",
        "experimental" => "Experimental",
        "encrypt_session" => "Cifrar sesión",
        "persona_wait_secs" => "Espera de presencia",
        "download_limit_kbit" => "Límite de descarga",
        "forced_proton_version" => "Proton forzado",
        "launch_script" => "Script de lanzamiento",
        "platform_preference" => "Plataforma preferida",
        "prefix_dir" => "Prefijo",
        "runner" => "Runner",
        "wine_prefix" => "Prefijo de Wine",
        "cloud_sync" => "Nube",
        "cloud_sync_effective" => "Nube efectiva",
        _ => return k.replace('_', " "),
    };
    s.to_string()
}

fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    for c in s.chars() {
        match c {
            '<' => {
                tag = true;
                out.push(' ');
            }
            '>' => tag = false,
            _ if tag => {}
            _ => out.push(c),
        }
    }
    out.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn fmt_val(k: &str, v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::Bool(b) => {
            if *b {
                "sí".into()
            } else {
                "no".into()
            }
        }
        Value::String(s) => {
            if k == "date" && s.parse::<i64>().is_ok() {
                date(s.parse().unwrap_or(0))
            } else if k == "persona_state" {
                match s.as_str() {
                    "0" | "offline" => "desconectado".into(),
                    "1" | "online" => "en línea".into(),
                    "2" | "busy" => "ocupado".into(),
                    "3" | "away" => "ausente".into(),
                    "4" | "snooze" => "no molestar".into(),
                    _ => s.clone(),
                }
            } else {
                s.clone()
            }
        }
        Value::Number(n) => {
            if is_bytes(k) {
                hb(n.as_f64().unwrap_or(0.0))
            } else if k == "playtime_forever_minutes" {
                let m = n.as_u64().unwrap_or(0);
                format!("{} h {} min", m / 60, m % 60)
            } else if k == "discount_pct" {
                let d = n.as_u64().unwrap_or(0);
                if d == 0 {
                    "—".into()
                } else {
                    format!("-{d}%")
                }
            } else if k == "download_limit_kbit" {
                let v = n.as_u64().unwrap_or(0);
                if v == 0 {
                    "sin límite".into()
                } else {
                    format!("{v} kbit/s")
                }
            } else if is_ts(k) {
                date(n.as_i64().unwrap_or(0))
            } else {
                n.to_string()
            }
        }
        Value::Array(a) => a
            .iter()
            .filter_map(|x| match x {
                Value::String(s) if !s.is_empty() => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                Value::Object(o) => o
                    .get("name")
                    .or_else(|| o.get("title"))
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                _ => None,
            })
            .take(10)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(o) => {
            if let Some(d) = o.get("descriptors").and_then(|x| x.as_array()) {
                d.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else if let Some(n) = o.get("name").and_then(|x| x.as_str()) {
                n.to_string()
            } else {
                v.to_string()
            }
        }
    }
}

fn kv_line(k: &str, v: String, t: &Theme, width: u16) -> Line<'static> {
    let w = (width as usize).saturating_sub(22).max(8);
    Line::from(vec![
        Span::styled(format!(" {:<18}", trunc(&label(k), 18)), st(t.accent2)),
        Span::styled(trunc(&v, w), st(t.fg)),
    ])
}

fn card(title: &str, subs: &[String], t: &Theme, width: u16) -> Vec<Line<'static>> {
    let w = (width as usize).saturating_sub(4).max(8);
    let mut out = vec![Line::from(vec![
        Span::styled(" ▸ ", st(t.accent)),
        Span::styled(trunc(title, w), st(t.fg).add_modifier(Modifier::BOLD)),
    ])];
    for s in subs.iter().filter(|s| !s.is_empty()) {
        out.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(trunc(s, w), st(t.dim)),
        ]));
    }
    out
}

fn pct_short(reviews: &str) -> String {
    // "Muy positivas (80% positive, 2.829.556 reviews)" -> "Muy positivas (80%)"
    if let Some(open) = reviews.find('(') {
        let head = reviews[..open].trim();
        let inside = &reviews[open + 1..];
        if let Some(p) = inside.split('%').next() {
            return format!("{head} ({p}%)");
        }
    }
    reviews.to_string()
}

fn arr<'a>(v: &'a Value, keys: &[&str]) -> &'a [Value] {
    if let Some(a) = v.as_array() {
        return a;
    }
    for k in keys {
        if let Some(a) = v.get(*k).and_then(|x| x.as_array()) {
            return a;
        }
    }
    &[]
}

fn get<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if !x.is_null() {
                return Some(x);
            }
        }
    }
    None
}

fn sget(v: &Value, keys: &[&str]) -> Option<String> {
    get(v, keys).map(|x| fmt_val(keys[0], x)).filter(|s| !s.is_empty() && s != "—")
}

fn ival(v: &Value, keys: &[&str]) -> Option<i64> {
    get(v, keys).and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
}

fn fval(v: &Value, keys: &[&str]) -> Option<f64> {
    get(v, keys).and_then(|x| x.as_f64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
}

// ---------------------------------------------------------------- progreso

pub fn parse_progress(v: &Value) -> Option<Progress> {
    if v.get("event").and_then(|e| e.as_str()) != Some("progress") {
        return None;
    }
    let done = fval(v, &["bytes_downloaded"]).unwrap_or(0.0);
    let total = fval(v, &["total_bytes", "bytes_total"]).unwrap_or(0.0);
    let mut pct = fval(v, &["percent", "depot_percent"]).unwrap_or(-1.0);
    if pct < 0.0 {
        pct = if total > 0.0 { done / total * 100.0 } else { 0.0 };
    }
    Some(Progress {
        pct: pct.clamp(0.0, 100.0),
        done,
        total,
        speed: fval(v, &["speed_bps"]).unwrap_or(0.0),
        eta: fval(v, &["eta_seconds"]).unwrap_or(0.0),
        state: sget(v, &["state"]).unwrap_or_default(),
    })
}

fn state_es(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "downloading" | "download" => "Descargando",
        "pausing" => "Pausando",
        "stopping" => "Deteniendo",
        "verifying" => "Verificando archivos",
        "updating" => "Actualizando",
        "preallocating" => "Reservando espacio",
        "shaders" | "shader" => "Compilando shaders",
        "reusing" | "reuse" => "Reutilizando archivos",
        "complete" | "done" => "Completado",
        "" => "Trabajando",
        other => other,
    }
    .to_string()
}

pub fn progress_lines(p: &Progress, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let bw = (width as usize).saturating_sub(8).clamp(10, 44);
    let filled = ((p.pct / 100.0) * bw as f64).round() as usize;
    let bar = format!("{}{}", "█".repeat(filled.min(bw)), "░".repeat(bw - filled.min(bw)));
    let mut out = vec![blank()];
    out.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(state_es(&p.state), st(t.accent2).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {:.0}%", p.pct), st(t.ok).add_modifier(Modifier::BOLD)),
    ]));
    out.push(Line::from(vec![Span::raw("  "), Span::styled(bar, st(t.accent))]));
    let mut detail: Vec<String> = vec![];
    if p.total > 0.0 {
        detail.push(format!("{} de {}", hb(p.done), hb(p.total)));
    } else if p.done > 0.0 {
        detail.push(hb(p.done));
    }
    if p.speed > 0.0 {
        detail.push(format!("{} /s", hb(p.speed)));
    }
    if p.eta > 0.0 {
        detail.push(format!("faltan {}", dur(p.eta)));
    }
    if !detail.is_empty() {
        out.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(detail.join("  ·  "), st(t.fg)),
        ]));
    }
    out
}

// ---------------------------------------------------------------- errores

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            if it.peek() == Some(&'[') {
                it.next();
                while let Some(&n) = it.peek() {
                    it.next();
                    if n.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(Default)]
struct ApiError {
    text: String,
    kind: String,
    hint: String,
}

fn parse_errors(errors: &[String]) -> ApiError {
    let mut out = ApiError::default();
    for raw in errors {
        let clean = strip_ansi(raw).trim().to_string();
        if clean.is_empty() || clean.starts_with("WARN ") {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(clean.trim_start_matches("error: ")) {
            if let Some(e) = v.get("error").and_then(|x| x.as_str()) {
                out.text = e.to_string();
            }
            if let Some(k) = v.get("type").and_then(|x| x.as_str()) {
                out.kind = k.to_string();
            }
            if let Some(h) = v.get("hint").and_then(|x| x.as_str()) {
                out.hint = h.to_string();
            }
            continue;
        }
        if out.text.is_empty() {
            out.text = clean.trim_start_matches("Error: ").to_string();
        }
    }
    out
}

fn looks_like_auth(s: &str) -> bool {
    let l = s.to_lowercase();
    l.contains("auth_required")
        || l.contains("not logged")
        || l.contains("no session")
        || l.contains("login")
        || l.contains("inicia sesión")
}

pub fn is_auth_error(errors: &[String], code: Option<i32>) -> bool {
    code == Some(77) || errors.iter().any(|e| looks_like_auth(e))
}

fn explain(kind: &str, text: &str) -> (String, String) {
    let low = text.to_lowercase();
    let kind = if kind.is_empty() {
        if low.contains("error decoding response body") || low.contains("failed parsing") {
            "source_changed"
        } else if low.contains("not running") {
            "not_running"
        } else if low.contains("no dlc") {
            "no_dlc"
        } else if low.contains("unrecognized subcommand") {
            "bad_command"
        } else {
            ""
        }
    } else {
        kind
    };
    match kind {
        "auth_required" => (
            "No hay una sesión de Steam iniciada.".into(),
            "Inicia sesión en Sistema → Iniciar sesión (o pulsa X y elige «Iniciar sesión»).".into(),
        ),
        "source_changed" => (
            "Steam cambió el formato de una página y Aurelia no pudo leerla.".into(),
            "No es un fallo tuyo: es el backend. Actualiza Aurelia en Sistema → Actualizar Aurelia y reintenta.".into(),
        ),
        "rate_limited" => (
            "Steam está limitando las peticiones de este equipo.".into(),
            "Espera unos minutos y vuelve a intentarlo.".into(),
        ),
        "not_found" => (
            "Steam no tiene ese AppID o no está disponible en tu región.".into(),
            "Comprueba el número en la tienda de Steam y vuelve a intentarlo.".into(),
        ),
        "access_denied" | "privacy_restricted" => (
            "La cuenta o la lista es privada.".into(),
            "Haz pública la lista o el perfil en Steam y reintenta.".into(),
        ),
        "network_timeout" | "offline" => (
            "No hay conexión con Steam.".into(),
            "Comprueba tu conexión a internet y reintenta.".into(),
        ),
        "disk_full" => (
            "No hay espacio suficiente en el disco.".into(),
            "Libera espacio o instala en otra biblioteca (Instalaciones → Bibliotecas).".into(),
        ),
        "steam_config_mismatch" => (
            "Steam espera otro conjunto de depósitos (idioma o plataforma).".into(),
            "Cambia el idioma del juego o revisa el Proton en Complementos → Proton.".into(),
        ),
        "login_denied" | "login_expired" | "cancelled" | "timeout" => (
            "El inicio de sesión no se completó.".into(),
            "Vuelve a intentarlo desde Sistema → Iniciar sesión.".into(),
        ),
        "input_closed" => (
            "Aurelia necesitaba datos de entrada y no los recibió.".into(),
            "Ejecuta el comando desde la app, no en segundo plano.".into(),
        ),
        "not_running" => (
            "Ese juego no está en ejecución.".into(),
            "Ábrelo con Jugar y luego podrás detenerlo.".into(),
        ),
        "no_dlc" => ("Este juego no tiene DLC.".into(), String::new()),
        "bad_command" => (
            "Ese comando no existe en esta versión de Aurelia.".into(),
            "Actualiza Aurelia en Sistema → Actualizar Aurelia.".into(),
        ),
        _ => (
            if text.is_empty() {
                "El comando terminó con un error.".into()
            } else {
                text.to_string()
            },
            "Pulsa v para ver la salida completa del comando.".into(),
        ),
    }
}

pub fn error_lines(errors: &[String], code: Option<i32>, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let e = parse_errors(errors);
    let (cause, hint) = explain(&e.kind, &e.text);
    let w = (width as usize).saturating_sub(6).max(12);
    let mut out = vec![
        Line::from(vec![
            Span::styled(" ✗ ", st(t.err).add_modifier(Modifier::BOLD)),
            Span::styled("No se pudo completar", st(t.err).add_modifier(Modifier::BOLD)),
        ]),
        blank(),
    ];
    out.push(Line::from(vec![
        Span::styled("  Causa: ", st(t.accent2)),
        Span::styled(trunc(&cause, w), st(t.fg)),
    ]));
    if !hint.is_empty() {
        out.push(Line::from(vec![
            Span::styled("  Qué hacer: ", st(t.accent2)),
            Span::styled(trunc(&hint, w), st(t.fg)),
        ]));
    }
    if !e.hint.is_empty() && !hint.contains(&e.hint) {
        out.push(Line::from(vec![
            Span::styled("  Nota: ", st(t.dim)),
            Span::styled(trunc(&e.hint, w), st(t.dim)),
        ]));
    }
    if let Some(c) = code {
        if c != 0 && c != 1 {
            out.push(Line::from(vec![
                Span::styled("  Código: ", st(t.dim)),
                Span::styled(c.to_string(), st(t.dim)),
            ]));
        }
    }
    out
}

/// Una línea corta con la causa, para el pie de la app.
pub fn short_error(errors: &[String], code: Option<i32>) -> String {
    let e = parse_errors(errors);
    let (cause, _) = explain(&e.kind, &e.text);
    if cause.is_empty() {
        format!("Error (código {})", code.unwrap_or(-1))
    } else {
        cause
    }
}

// ---------------------------------------------------------------- títulos

/// Título humano y en español para un trabajo, p. ej. «Instalando · Undertale».
pub fn job_title(args: &[String], games: &[Game]) -> Option<String> {
    let first = args.first()?.as_str();
    let second = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let target = |id: &str| {
        games
            .iter()
            .find(|g| g.id == id)
            .map(|g| g.name.clone())
            .unwrap_or_else(|| format!("AppID {id}"))
    };
    let with = |verb: &str, id: &str| {
        if id.is_empty() || id.starts_with('-') {
            verb.to_string()
        } else {
            format!("{verb} · {}", trunc(&target(id), 34))
        }
    };
    let s = match first {
        "install" => match second {
            "list" => "Cola de instalaciones".into(),
            "pause" => with("Pausando instalación", args.get(2).map(|s| s.as_str()).unwrap_or("")),
            "resume" => "Reanudando instalaciones".into(),
            "stop" => with("Cancelando instalación", args.get(2).map(|s| s.as_str()).unwrap_or("")),
            "forget" => with("Olvidando instalación", args.get(2).map(|s| s.as_str()).unwrap_or("")),
            _ => with("Instalando", second),
        },
        "update" => with("Actualizando", second),
        "verify" => with("Verificando archivos", second),
        "uninstall" => with("Desinstalando", second),
        "play" => with("Jugando", second),
        "stop" => with("Deteniendo", second),
        "list" => "Biblioteca".into(),
        "info" => with("Información", second),
        "dlc" => with("DLC", second),
        "news" => with("Noticias", second),
        "achievements" => with("Logros", second),
        "reviews" => with("Reseñas", second),
        "similar" => with("Similares", second),
        "players" => with("Jugadores", second),
        "price" => with("Precios", second),
        "search" => {
            if second.is_empty() {
                "Buscar en la tienda".into()
            } else {
                format!("Buscando «{}»", trunc(second, 28))
            }
        }
        "deals" => "Ofertas".into(),
        "events" => "Eventos y rebajas".into(),
        "wishlist" => "Lista de deseos".into(),
        "friends" => match second {
            "search" => "Buscar usuario".into(),
            "add" => "Agregar amigo".into(),
            "remove" => "Quitar amigo".into(),
            _ => "Amigos".into(),
        },
        "config" => match second {
            "game" => with("Configurando", args.get(2).map(|s| s.as_str()).unwrap_or("")),
            "protons" => "Protones detectados".into(),
            "show" => "Configuración".into(),
            _ => format!("Configurando {second}"),
        },
        "proton" => match second {
            "list" => "Protones disponibles".into(),
            "install" => "Instalando Proton".into(),
            "uninstall" => "Desinstalando Proton".into(),
            "default" => "Proton predeterminado".into(),
            _ => format!("Proton {second}"),
        },
        "umu" => format!("umu {second}"),
        "luxtorpeda" => format!("Luxtorpeda {second}"),
        "steam-runtime" => format!("Steam runtime {second}"),
        "cloud" => format!("Nube {second}"),
        "workshop" => match second {
            "browse" => "Workshop".into(),
            "install" => "Instalando mod".into(),
            "subscribe" => "Suscribiendo mod".into(),
            "status" => "Estado del Workshop".into(),
            _ => format!("Workshop {second}"),
        },
        "depots" => match second {
            "files" => "Archivos del depot".into(),
            "download" => "Descargando archivo del depot".into(),
            _ => with("Depots", second),
        },
        "manifests" => with("Manifiestos", second),
        "branches" => with("Ramas", second),
        "set-branch" => with("Cambiando de rama", second),
        "set-language" => with("Cambiando idioma", second),
        "launch-options" => with("Opciones de lanzamiento", second),
        "launch-targets" => with("Destinos de lanzamiento", second),
        "move" => with("Moviendo", second),
        "available" => with("Comprobando instalación", second),
        "libraries" => "Bibliotecas".into(),
        "account" => "Cuenta".into(),
        "login" => "Iniciando sesión".into(),
        "logout" => "Cerrando sesión".into(),
        "capabilities" => "Capacidades de Aurelia".into(),
        "import" => with("Importando", second),
        "relink" => with("Revincular", second),
        "duplicates" => "Duplicados".into(),
        "pin" => with("Fijando versión", second),
        "unpin" => with("Soltando versión", second),
        "downgrade" => with("Degradando versión", second),
        "shaders" => with("Compilando shaders", second),
        "inventory" => "Inventario".into(),
        "wallet" => "Cartera".into(),
        "collections" => format!("Colecciones {second}"),
        "market" => format!("Mercado {second}"),
        "chat" => format!("Chat {second}"),
        "user" => "Perfil".into(),
        "drm" => with("DRM", second),
        "image" => with("Imágenes", second),
        "--version" => "Versión de Aurelia".into(),
        "--help" => "Ayuda de Aurelia".into(),
        _ => return None,
    };
    Some(s)
}

// ---------------------------------------------------------------- vistas

pub fn render(args: &[String], v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
    let sub = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match cmd {
        "list" => games_view(v, t, width),
        "search" | "similar" | "deals" | "wishlist" => store_view(v, t, width),
        "dlc" => dlc_view(v, t, width),
        "news" => news_view(v, t, width),
        "achievements" => achievements_view(v, t, width),
        "events" => events_view(v, t, width),
        "price" => price_view(v, t, width),
        "players" => players_view(v, t),
        "info" => info_view(v, t, width),
        "friends" => friends_view(v, t, width),
        "libraries" => libraries_view(v, t, width),
        "depots" => depots_view(v, t, width),
        "manifests" => manifests_view(v, t, width),
        "branches" => branches_view(v, t),
        "launch-options" => launch_options_view(v, t, width),
        "launch-targets" => launch_targets_view(v, t, width),
        "config" if sub == "show" || sub.is_empty() => config_view(v, t, width),
        "config" => config_game_view(v, t, width),
        "install" if sub == "list" || v.is_array() => installs_view(v, t, width),
        "install" => install_dry_view(v, t, width),
        "running" => running_view(v, t, width),
        "proton" => proton_view(v, t, width),
        "account" => account_view(v, t, width),
        "user" => user_view(v, t, width),
        "wallet" => wallet_view(v, t, width),
        "collections" => collections_view(v, t, width),
        "available" => available_view(v, t, width),
        _ => generic(v, t, width),
    }
}

fn games_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["games", "items", "library"]);
    if a.is_empty() {
        return vec![note("No hay juegos en la biblioteca. Inicia sesión o recarga con r.", t)];
    }
    let mut out = vec![];
    for g in a.iter().take(MAX_ROWS) {
        let name = sget(g, &["name", "title"]).unwrap_or_else(|| "Sin nombre".into());
        let id = sget(g, &["app_id", "appid", "id"]).unwrap_or_default();
        let installed = get(g, &["is_installed", "installed"]).and_then(|x| x.as_bool()).unwrap_or(false);
        let mut meta: Vec<String> = vec![];
        if let Some(p) = sget(g, &["platform"]) {
            meta.push(match p.as_str() {
                "windows" => "Windows".into(),
                "linux" => "Linux".into(),
                other => other.to_string(),
            });
        }
        if let Some(sz) = fval(g, &["size_on_disk"]) {
            if sz > 0.0 {
                meta.push(hb(sz));
            }
        }
        if let Some(m) = ival(g, &["playtime_forever_minutes"]) {
            if m > 0 {
                meta.push(format!("{} h {} min", m / 60, m % 60));
            }
        }
        if get(g, &["update_available"]).and_then(|x| x.as_bool()) == Some(true) {
            meta.push("actualización disponible".into());
        }
        let mut subs = vec![];
        if !meta.is_empty() {
            subs.push(meta.join(" · "));
        }
        if let Some(p) = sget(g, &["install_path", "install_dir"]) {
            subs.push(p);
        }
        let title = format!("{}  {}", if installed { "●" } else { "○" }, name);
        let mut lines = card(&title, &subs, t, width);
        if let Some(first) = lines.first_mut() {
            let badge = if installed { " instalado" } else { " no instalado" };
            let col = if installed { t.ok } else { t.dim };
            *first = Line::from(vec![
                Span::styled(" ▸ ", st(t.accent)),
                Span::styled(trunc(&title, (width as usize).saturating_sub(24).max(8)), st(t.fg).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  {badge}"), st(col)),
                Span::styled(format!("  #{id}"), st(t.dim)),
            ]);
        }
        out.extend(lines);
    }
    if a.len() > MAX_ROWS {
        out.push(note(&format!("… y {} juegos más", a.len() - MAX_ROWS), t));
    }
    out
}

fn store_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["results", "items", "games"]);
    if a.is_empty() {
        return vec![note("Sin resultados. Prueba con otro título o AppID.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let name = sget(x, &["name", "title"]).unwrap_or_else(|| "Sin nombre".into());
        let id = sget(x, &["app_id", "appid", "id"]).unwrap_or_default();
        let mut subs: Vec<String> = vec![];
        let price = sget(x, &["price"]).unwrap_or_else(|| "—".into());
        let disc = ival(x, &["discount_pct"]).unwrap_or(0);
        let mut money = price;
        if disc > 0 {
            if let Some(orig) = sget(x, &["original_price"]) {
                money = format!("{money}  (-{disc}%, antes {orig})");
            } else {
                money = format!("{money}  (-{disc}%)");
            }
        }
        subs.push(money);
        if let Some(r) = sget(x, &["reviews"]) {
            subs.push(pct_short(&r));
        }
        let mut meta = vec![];
        if let Some(pl) = sget(x, &["platforms"]) {
            meta.push(pl);
        }
        if let Some(d) = sget(x, &["release_date"]) {
            meta.push(d);
        }
        if let Some(ty) = sget(x, &["type"]) {
            meta.push(ty);
        }
        if !meta.is_empty() {
            subs.push(meta.join(" · "));
        }
        let title = format!("{name}  #{id}");
        out.extend(card(&title, &subs, t, width));
    }
    if a.len() > MAX_ROWS {
        out.push(note(&format!("… y {} más", a.len() - MAX_ROWS), t));
    }
    out
}

fn dlc_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["dlc"]);
    if a.is_empty() {
        return vec![note("Este juego no tiene DLC.", t)];
    }
    let mut out = vec![note(&format!("{} DLC", a.len()), t), blank()];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["app_id"]).unwrap_or_default();
        let name = sget(x, &["name"]).unwrap_or_else(|| "(nombre no disponible)".into());
        let owned = get(x, &["owned"]).and_then(|x| x.as_bool()).unwrap_or(false);
        let installed = get(x, &["installed"]).and_then(|x| x.as_bool()).unwrap_or(false);
        let price = sget(x, &["price"]).unwrap_or_else(|| "—".into());
        let mut meta = vec![
            if owned { "en tu cuenta" } else { "no lo tienes" }.to_string(),
            if installed { "instalado" } else { "no instalado" }.to_string(),
            price,
        ];
        if let Some(d) = ival(x, &["discount_pct"]) {
            if d > 0 {
                meta.push(format!("-{d}%"));
            }
        }
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(format!("#{id} "), st(t.dim)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(16).max(8)), st(t.fg)),
        ]));
        out.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(meta.join(" · "), st(t.dim)),
        ]));
    }
    if a.len() > MAX_ROWS {
        out.push(note(&format!("… y {} más", a.len() - MAX_ROWS), t));
    }
    out
}

fn news_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["news", "items"]);
    if a.is_empty() {
        return vec![note("Este juego no tiene noticias recientes.", t)];
    }
    let w = (width as usize).saturating_sub(4).max(20);
    let mut out = vec![];
    for x in a.iter().take(40) {
        let title = sget(x, &["title"]).unwrap_or_else(|| "Sin título".into());
        let date_s = ival(x, &["date"]).map(date).unwrap_or_default();
        let author = sget(x, &["author"]).unwrap_or_default();
        let feed = sget(x, &["feed"]).unwrap_or_default();
        let mut meta = vec![];
        if !date_s.is_empty() {
            meta.push(date_s);
        }
        if !author.is_empty() {
            meta.push(author);
        }
        if !feed.is_empty() {
            meta.push(feed);
        }
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&title, w), st(t.fg).add_modifier(Modifier::BOLD)),
        ]));
        if !meta.is_empty() {
            out.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(trunc(&meta.join(" · "), w), st(t.dim)),
            ]));
        }
        if let Some(c) = sget(x, &["contents"]) {
            let clean = strip_html(&c.replace("{STEAM_CLAN_IMAGE}", "").replace("{STEAM_CLAN_LOC_IMAGE}", ""));
            for l in wrap(&clean, w).into_iter().take(2) {
                out.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(l, st(t.fg)),
                ]));
            }
        }
        out.push(blank());
    }
    out
}

fn achievements_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["achievements"]);
    let total = ival(v, &["total"]).unwrap_or(a.len() as i64);
    let unlocked = ival(v, &["unlocked"]).unwrap_or(0);
    if a.is_empty() {
        return vec![note(
            "Sin logros disponibles para este juego (Steam no devolvió el esquema de logros).",
            t,
        )];
    }
    let pct = if total > 0 { unlocked as f64 / total as f64 * 100.0 } else { 0.0 };
    let mut out = vec![Line::from(vec![
        Span::styled(" Desbloqueados: ", st(t.accent2)),
        Span::styled(format!("{unlocked}/{total}"), st(t.ok).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ({pct:.0}%)"), st(t.dim)),
    ])];
    out.push(blank());
    let w = (width as usize).saturating_sub(6).max(10);
    for x in a.iter().take(MAX_ROWS) {
        let name = sget(x, &["name", "title", "description"]).unwrap_or_else(|| "Logro".into());
        let done = get(x, &["achieved", "unlocked"])
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let gp = fval(x, &["global_percent", "percent"]).unwrap_or(0.0);
        out.push(Line::from(vec![
            Span::styled(if done { " ✓ " } else { " ○ " }, st(if done { t.ok } else { t.dim })),
            Span::styled(trunc(&name, w), st(if done { t.fg } else { t.dim })),
            Span::styled(format!("  {gp:.1}%"), st(t.accent2)),
        ]));
        if let Some(d) = sget(x, &["description"]) {
            if d != name {
                out.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(trunc(&d, w), st(t.dim)),
                ]));
            }
        }
    }
    out
}

fn events_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["events"]);
    if a.is_empty() {
        return vec![note("No hay eventos ni rebajas activas ahora mismo.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let title = sget(x, &["title"]).unwrap_or_else(|| "Evento".into());
        let kind = sget(x, &["kind"]).unwrap_or_default();
        let start = ival(x, &["start"]).map(date).unwrap_or_default();
        let end = ival(x, &["end"]).map(date).unwrap_or_default();
        let assoc = sget(x, &["associated_name"]).filter(|s| !s.is_empty());
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&title, (width as usize).saturating_sub(4).max(10)), st(t.fg).add_modifier(Modifier::BOLD)),
        ]));
        let mut meta = vec![kind];
        if !start.is_empty() {
            meta.push(format!("{start} → {end}"));
        }
        if let Some(a) = assoc {
            meta.push(a);
        }
        out.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(meta.join(" · "), st(t.dim)),
        ]));
    }
    out
}

fn price_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(n) = sget(v, &["name"]) {
        out.push(Line::from(vec![
            Span::styled(" ", st(t.fg)),
            Span::styled(n, st(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  #{}", sget(v, &["app_id"]).unwrap_or_default()),
                st(t.dim),
            ),
        ]));
        out.push(blank());
    }
    let a = arr(v, &["regions", "prices"]);
    if a.is_empty() {
        out.push(note("Sin precios disponibles.", t));
        return out;
    }
    for x in a.iter().take(MAX_ROWS) {
        let country = sget(x, &["country"]).unwrap_or_default();
        let available = get(x, &["available"]).and_then(|x| x.as_bool()).unwrap_or(true);
        let price = sget(x, &["price"]).unwrap_or_else(|| "—".into());
        let disc = ival(x, &["discount_pct"]).unwrap_or(0);
        let mut meta = vec![];
        if !available {
            meta.push("no disponible".to_string());
        }
        if disc > 0 {
            meta.push(format!("-{disc}%"));
            if let Some(o) = sget(x, &["original_price"]) {
                meta.push(format!("antes {o}"));
            }
        }
        out.push(Line::from(vec![
            Span::styled(format!(" {:<4}", country), st(t.accent2).add_modifier(Modifier::BOLD)),
            Span::styled(trunc(&price, (width as usize).saturating_sub(28).max(8)), st(t.fg)),
            Span::styled(if meta.is_empty() { String::new() } else { format!("  {}", meta.join(" · ")) }, st(t.dim)),
        ]));
    }
    out
}

fn players_view(v: &Value, t: &Theme) -> Vec<Line<'static>> {
    let n = ival(v, &["players"]).unwrap_or(0);
    vec![
        blank(),
        Line::from(vec![
            Span::styled("  ", st(t.fg)),
            Span::styled(n.to_string(), st(t.ok).add_modifier(Modifier::BOLD)),
            Span::styled(" jugadores ahora mismo", st(t.fg)),
        ]),
        Line::from(vec![
            Span::styled("  AppID ", st(t.dim)),
            Span::styled(sget(v, &["app_id"]).unwrap_or_default(), st(t.dim)),
        ]),
    ]
}

fn info_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let w = (width as usize).saturating_sub(4).max(20);
    let mut out = vec![];
    let name = sget(v, &["name"]).unwrap_or_else(|| "Sin nombre".into());
    let id = sget(v, &["app_id"]).unwrap_or_default();
    out.push(Line::from(vec![
        Span::styled(" ", st(t.fg)),
        Span::styled(trunc(&name, w.saturating_sub(12)), st(t.accent).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  #{id}"), st(t.dim)),
    ]));
    let mut meta = vec![];
    if let Some(ty) = sget(v, &["type"]) {
        meta.push(ty);
    }
    if let Some(d) = sget(v, &["release_date"]) {
        meta.push(d);
    }
    if get(v, &["is_free"]).and_then(|x| x.as_bool()) == Some(true) {
        meta.push("gratis".into());
    }
    if get(v, &["coming_soon"]).and_then(|x| x.as_bool()) == Some(true) {
        meta.push("próximamente".into());
    }
    if get(v, &["early_access"]).and_then(|x| x.as_bool()) == Some(true) {
        meta.push("acceso anticipado".into());
    }
    if !meta.is_empty() {
        out.push(Line::from(vec![Span::raw("  "), Span::styled(meta.join(" · "), st(t.dim))]));
    }
    out.push(blank());
    let mut kv = |k: &str, val: Option<String>| {
        if let Some(val) = val.filter(|s| !s.is_empty() && s != "—") {
            out.push(kv_line(k, val, t, width));
        }
    };
    kv("developers", sget(v, &["developers"]));
    kv("publishers", sget(v, &["publishers"]));
    kv("platforms", sget(v, &["platforms"]));
    kv("franchises", sget(v, &["franchises"]));
    kv("price", sget(v, &["price"]));
    if let Some(d) = ival(v, &["discount_pct"]) {
        if d > 0 {
            kv("discount_pct", Some(format!("-{d}%")));
        }
    }
    kv("reviews", sget(v, &["reviews"]).map(|r| pct_short(&r)));
    kv("rating", sget(v, &["rating"]));
    if let Some(tags) = get(v, &["tags"]).and_then(|x| x.as_array()) {
        let names: Vec<String> = tags
            .iter()
            .take(10)
            .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect();
        if !names.is_empty() {
            kv("tags", Some(names.join(", ")));
        }
    }
    if let Some(dlc) = get(v, &["dlc"]).and_then(|x| x.as_array()) {
        if !dlc.is_empty() {
            kv("dlc", Some(format!("{} DLC", dlc.len())));
        }
    }
    if let Some(shots) = get(v, &["screenshots"]).and_then(|x| x.as_array()) {
        if !shots.is_empty() {
            kv("screenshots", Some(format!("{} capturas", shots.len())));
        }
    }
    if let Some(tr) = get(v, &["trailers"]).and_then(|x| x.as_array()) {
        if !tr.is_empty() {
            kv("trailers", Some(format!("{} tráilers", tr.len())));
        }
    }
    kv("store_url", sget(v, &["store_url"]));
    if let Some(desc) = sget(v, &["description", "short_description"]) {
        let desc = strip_html(&desc);
        if !desc.is_empty() {
            out.push(blank());
            out.push(section("Descripción", t));
            for l in wrap(&desc, w) {
                out.push(Line::from(vec![Span::raw("  "), Span::styled(l, st(t.fg))]));
            }
        }
    }
    out
}

fn friends_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["friends", "results", "items"]);
    if a.is_empty() {
        return vec![note("No hay amigos que mostrar.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let name = sget(x, &["persona_name", "name"]).unwrap_or_else(|| "Sin nombre".into());
        let state = ival(x, &["persona_state"]).unwrap_or(0);
        let st_s = match state {
            1 => ("en línea", t.ok),
            2 => ("ocupado", t.err),
            3 => ("ausente", t.warn),
            4 => ("no molestar", t.warn),
            _ => ("desconectado", t.dim),
        };
        let game = sget(x, &["game_name"]).filter(|s| !s.is_empty());
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(14).max(8)), st(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}", st_s.0), st(st_s.1)),
        ]));
        if let Some(g) = game {
            out.push(Line::from(vec![
                Span::raw("    jugando a "),
                Span::styled(trunc(&g, (width as usize).saturating_sub(16).max(8)), st(t.accent2)),
            ]));
        }
    }
    out
}

fn libraries_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["libraries"]);
    if a.is_empty() {
        return vec![note("No hay bibliotecas de Steam configuradas.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let path = sget(x, &["path"]).unwrap_or_default();
        let free = fval(x, &["free_bytes"]).unwrap_or(0.0);
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&path, (width as usize).saturating_sub(4).max(10)), st(t.fg)),
        ]));
        out.push(Line::from(vec![
            Span::raw("    libre: "),
            Span::styled(hb(free), st(t.ok)),
        ]));
    }
    out
}

fn depots_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["depots"]);
    if a.is_empty() {
        return vec![note("Este juego no expone depots.", t)];
    }
    let mut out = vec![note(&format!("{} depots", a.len()), t), blank()];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["id"]).unwrap_or_default();
        let name = sget(x, &["name"]).unwrap_or_default();
        let size = fval(x, &["size"]).unwrap_or(0.0);
        let files = ival(x, &["file_count"]).unwrap_or(0);
        let cfg = sget(x, &["config"]).unwrap_or_default();
        out.push(Line::from(vec![
            Span::styled(format!(" #{id:<8}"), st(t.accent2)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(22).max(8)), st(t.fg)),
            Span::styled(format!("  {}", hb(size)), st(t.dim)),
        ]));
        let mut meta = vec![];
        if files > 0 {
            meta.push(format!("{files} archivos"));
        }
        if !cfg.is_empty() {
            meta.push(cfg);
        }
        if !meta.is_empty() {
            out.push(Line::from(vec![
                Span::raw("          "),
                Span::styled(meta.join(" · "), st(t.dim)),
            ]));
        }
    }
    out
}

fn manifests_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["manifests"]);
    if a.is_empty() {
        return vec![note("Sin manifiestos para este juego.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let branch = sget(x, &["branch"]).unwrap_or_else(|| "public".into());
        let depot = sget(x, &["depot_id"]).unwrap_or_default();
        let mid = sget(x, &["manifest_id"]).unwrap_or_default();
        let size = fval(x, &["size"]).unwrap_or(0.0);
        out.push(Line::from(vec![
            Span::styled(format!(" {:<8}", branch), st(t.accent2)),
            Span::styled(format!("depot {depot}"), st(t.fg)),
            Span::styled(format!("  {}", hb(size)), st(t.dim)),
        ]));
        out.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(trunc(&mid, (width as usize).saturating_sub(4).max(10)), st(t.dim)),
        ]));
    }
    out
}

fn branches_view(v: &Value, t: &Theme) -> Vec<Line<'static>> {
    let a = arr(v, &["branches"]);
    if a.is_empty() {
        return vec![note("Este juego solo tiene la rama pública.", t)];
    }
    a.iter()
        .take(MAX_ROWS)
        .map(|x| bullet(x.as_str().unwrap_or("?").to_string(), t))
        .collect()
}

fn launch_options_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["launch_options", "options"]);
    if a.is_empty() {
        return vec![note("Sin opciones de lanzamiento.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["id"]).unwrap_or_default();
        let desc = sget(x, &["description"]).unwrap_or_else(|| "Lanzar".into());
        let os = sget(x, &["oslist"]).unwrap_or_default();
        let inst = get(x, &["installed"]).and_then(|x| x.as_bool()).unwrap_or(false);
        out.push(Line::from(vec![
            Span::styled(format!(" #{id:<3}"), st(t.accent2)),
            Span::styled(trunc(&desc, (width as usize).saturating_sub(14).max(8)), st(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}", if inst { "instalada" } else { "" }), st(t.ok)),
        ]));
        let mut meta = vec![];
        if !os.is_empty() {
            meta.push(os);
        }
        if let Some(exe) = sget(x, &["executable"]) {
            meta.push(exe);
        }
        if !meta.is_empty() {
            out.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(trunc(&meta.join(" · "), (width as usize).saturating_sub(6).max(10)), st(t.dim)),
            ]));
        }
        if let Some(args) = sget(x, &["arguments"]) {
            if !args.is_empty() {
                out.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(trunc(&args, (width as usize).saturating_sub(6).max(10)), st(t.dim)),
                ]));
            }
        }
    }
    out
}

fn launch_targets_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["targets"]);
    if a.is_empty() {
        return vec![note("Sin destinos de lanzamiento.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["id"]).unwrap_or_default();
        let available = get(x, &["available"]).and_then(|x| x.as_bool()).unwrap_or(false);
        let default = get(x, &["default"]).and_then(|x| x.as_bool()).unwrap_or(false);
        let missing = sget(x, &["missing"]).unwrap_or_default();
        let flags = sget(x, &["flags"]).unwrap_or_default();
        out.push(Line::from(vec![
            Span::styled(if available { " ✓ " } else { " ○ " }, st(if available { t.ok } else { t.dim })),
            Span::styled(trunc(&id, (width as usize).saturating_sub(12).max(8)), st(t.fg)),
            Span::styled(if default { "  (predeterminado)" } else { "" }, st(t.accent2)),
        ]));
        let mut meta = vec![];
        if !missing.is_empty() {
            meta.push(format!("falta: {missing}"));
        }
        if !flags.is_empty() {
            meta.push(flags);
        }
        if !meta.is_empty() {
            out.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(trunc(&meta.join(" · "), (width as usize).saturating_sub(6).max(10)), st(t.dim)),
            ]));
        }
    }
    out
}

fn config_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(m) = v.as_object() {
        for (k, x) in m {
            if x.is_object() || x.is_array() {
                continue;
            }
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
        for (k, x) in m {
            if let Some(arrv) = x.as_array() {
                if !arrv.is_empty() {
                    out.push(blank());
                    out.push(section(&label(k), t));
                    for item in arrv.iter().take(40) {
                        out.push(bullet(fmt_val("", item), t));
                    }
                }
            } else if let Some(o) = x.as_object() {
                if !o.is_empty() {
                    out.push(blank());
                    out.push(section(&label(k), t));
                    for (k2, x2) in o {
                        out.push(kv_line(k2, fmt_val(k2, x2), t, width));
                    }
                }
            }
        }
    } else {
        out.extend(generic(v, t, width));
    }
    out
}

fn config_game_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(m) = v.as_object() {
        for (k, x) in m {
            if x.is_object() || x.is_array() {
                continue;
            }
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
    }
    out
}

fn installs_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["installs", "items", "queue"]);
    if a.is_empty() {
        return vec![note("No hay instalaciones en curso ni en cola.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["app_id"]).unwrap_or_default();
        let name = sget(x, &["name"]).unwrap_or_else(|| format!("AppID {id}"));
        let state = sget(x, &["state"]).unwrap_or_default();
        let pct = fval(x, &["percent", "progress"]).unwrap_or(0.0);
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(14).max(8)), st(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {state}"), st(t.accent2)),
        ]));
        if pct > 0.0 {
            out.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{pct:.0}%"), st(t.ok)),
            ]));
        }
    }
    out
}

fn install_dry_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(m) = v.as_object() {
        for (k, x) in m {
            if x.is_object() || x.is_array() {
                continue;
            }
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
        if let Some(langs) = get(v, &["languages"]).and_then(|x| x.as_array()) {
            if !langs.is_empty() {
                out.push(kv_line("languages", fmt_val("languages", &Value::Array(langs.clone())), t, width));
            }
        }
    }
    out
}

fn running_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["running"]);
    if a.is_empty() {
        return vec![note("No hay juegos en ejecución.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let id = sget(x, &["app_id"]).unwrap_or_default();
        let name = sget(x, &["name"]).unwrap_or_else(|| format!("AppID {id}"));
        let pid = sget(x, &["pid"]).unwrap_or_default();
        out.push(Line::from(vec![
            Span::styled(" ● ", st(t.ok)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(12).max(8)), st(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  #{id}"), st(t.dim)),
            Span::styled(if pid.is_empty() { String::new() } else { format!("  PID {pid}") }, st(t.dim)),
        ]));
    }
    out.push(blank());
    out.push(note("X → detener · f fuerza el cierre", t));
    out
}

fn proton_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(d) = sget(v, &["default"]) {
        out.push(Line::from(vec![
            Span::styled(" Predeterminado: ", st(t.accent2)),
            Span::styled(d, st(t.fg).add_modifier(Modifier::BOLD)),
        ]));
        out.push(blank());
    }
    if let Some(a) = get(v, &["installed"]).and_then(|x| x.as_array()) {
        out.push(section("Instalados", t));
        for x in a.iter().take(60) {
            let name = sget(x, &["name"]).unwrap_or_default();
            let loc = sget(x, &["location"]).unwrap_or_default();
            out.push(Line::from(vec![
                Span::styled("  ✓ ", st(t.ok)),
                Span::styled(trunc(&name, (width as usize).saturating_sub(14).max(8)), st(t.fg)),
                Span::styled(if loc.is_empty() { String::new() } else { format!("  ({loc})") }, st(t.dim)),
            ]));
        }
        out.push(blank());
    }
    if let Some(a) = get(v, &["available"]).and_then(|x| x.as_array()) {
        out.push(section("Disponibles para instalar", t));
        for x in a.iter().take(60) {
            let name = sget(x, &["name"]).unwrap_or_default();
            let label = sget(x, &["label"]).unwrap_or_default();
            out.push(Line::from(vec![
                Span::styled("  · ", st(t.accent2)),
                Span::styled(trunc(&name, (width as usize).saturating_sub(14).max(8)), st(t.fg)),
                Span::styled(if label.is_empty() { String::new() } else { format!("  ({label})") }, st(t.dim)),
            ]));
        }
    }
    out
}

fn account_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    let name = sget(v, &["persona_name", "account_name"]).unwrap_or_default();
    out.push(Line::from(vec![
        Span::styled(" ", st(t.fg)),
        Span::styled(name, st(t.accent).add_modifier(Modifier::BOLD)),
    ]));
    out.push(blank());
    for k in [
        "steam_id",
        "account_name",
        "country",
        "email_validated",
        "vac_bans",
        "authed_machines",
    ] {
        if let Some(x) = v.get(k) {
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
    }
    out
}

fn user_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    let name = sget(v, &["persona_name"]).unwrap_or_default();
    out.push(Line::from(vec![
        Span::styled(" ", st(t.fg)),
        Span::styled(name, st(t.accent).add_modifier(Modifier::BOLD)),
    ]));
    if let Some(g) = sget(v, &["game_name"]) {
        out.push(Line::from(vec![
            Span::raw("  jugando a "),
            Span::styled(g, st(t.accent2)),
        ]));
    }
    out.push(blank());
    for k in [
        "steam_id",
        "location",
        "is_public",
        "is_limited",
        "created",
        "last_logoff",
        "profile_url",
    ] {
        if let Some(x) = v.get(k) {
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
    }
    if let Some(s) = sget(v, &["summary"]) {
        out.push(blank());
        for l in wrap(&s, (width as usize).saturating_sub(4).max(20)) {
            out.push(Line::from(vec![Span::raw("  "), Span::styled(l, st(t.fg))]));
        }
    }
    out
}

fn wallet_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    if let Some(m) = v.as_object() {
        for (k, x) in m {
            if x.is_object() || x.is_array() {
                continue;
            }
            out.push(kv_line(k, fmt_val(k, x), t, width));
        }
    }
    out
}

fn collections_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let a = arr(v, &["collections", "items"]);
    if a.is_empty() {
        return vec![note("No tienes colecciones.", t)];
    }
    let mut out = vec![];
    for x in a.iter().take(MAX_ROWS) {
        let name = sget(x, &["name"]).unwrap_or_else(|| "Colección".into());
        let count = ival(x, &["count", "entries"]).unwrap_or(0);
        out.push(Line::from(vec![
            Span::styled(" ▸ ", st(t.accent)),
            Span::styled(trunc(&name, (width as usize).saturating_sub(12).max(8)), st(t.fg)),
            Span::styled(format!("  {count} juegos"), st(t.dim)),
        ]));
    }
    out
}

fn available_view(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    let ok = get(v, &["available"]).and_then(|x| x.as_bool()).unwrap_or(false);
    out.push(Line::from(vec![
        Span::styled(" ", st(t.fg)),
        Span::styled(
            if ok { "✓ Está instalado en disco" } else { "○ No está instalado" },
            st(if ok { t.ok } else { t.warn }).add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(p) = sget(v, &["install_path"]) {
        out.push(kv_line("install_path", p, t, width));
    }
    if get(v, &["pinned"]).and_then(|x| x.as_bool()) == Some(true) {
        out.push(Line::from(vec![
            Span::styled("  Versión fijada", st(t.accent2)),
        ]));
    }
    out
}

fn generic(v: &Value, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut out = vec![];
    match v {
        Value::Array(a) => {
            if a.is_empty() {
                return vec![note("Sin resultados.", t)];
            }
            for x in a.iter().take(MAX_ROWS) {
                if let Some(o) = x.as_object() {
                    out.extend(obj_card(o, t, width));
                } else {
                    out.push(bullet(fmt_val("", x), t));
                }
            }
        }
        Value::Object(m) => {
            let mut scalars: Vec<(&String, &Value)> = vec![];
            let mut nested: Vec<(&String, &Value)> = vec![];
            for (k, x) in m {
                if x.is_object() || x.is_array() {
                    nested.push((k, x));
                } else {
                    scalars.push((k, x));
                }
            }
            for (k, x) in scalars {
                out.push(kv_line(k, fmt_val(k, x), t, width));
            }
            for (k, x) in nested {
                match x {
                    Value::Array(a) if !a.is_empty() => {
                        out.push(blank());
                        out.push(section(&label(k), t));
                        for item in a.iter().take(MAX_ROWS) {
                            if let Some(o) = item.as_object() {
                                out.extend(obj_card(o, t, width));
                            } else {
                                out.push(bullet(fmt_val("", item), t));
                            }
                        }
                    }
                    Value::Object(o) if !o.is_empty() => {
                        out.push(blank());
                        out.push(section(&label(k), t));
                        out.extend(generic(&Value::Object(o.clone()), t, width));
                    }
                    _ => {}
                }
            }
        }
        Value::Null => out.push(note("Sin datos.", t)),
        _ => out.push(note(&fmt_val("", v), t)),
    }
    out
}

fn obj_card(o: &serde_json::Map<String, Value>, t: &Theme, width: u16) -> Vec<Line<'static>> {
    let title_key = ["name", "title", "persona_name", "description", "app_id", "id"]
        .iter()
        .find(|k| o.get(**k).is_some())
        .copied();
    let title = title_key
        .and_then(|k| o.get(k))
        .map(|x| fmt_val(title_key.unwrap_or(""), x))
        .unwrap_or_else(|| "Elemento".into());
    let interesting = [
        "app_id", "price", "discount_pct", "reviews", "type", "platform", "country", "state", "installed",
        "owned", "size", "date", "end", "kind", "author",
    ];
    let mut subs: Vec<String> = vec![];
    let mut bits = vec![];
    for k in interesting {
        if Some(k) == title_key {
            continue;
        }
        if let Some(x) = o.get(k) {
            let val = fmt_val(k, x);
            if !val.is_empty() && val != "—" && val != "no" {
                bits.push(val);
            }
        }
    }
    if !bits.is_empty() {
        subs.push(bits.into_iter().take(4).collect::<Vec<_>>().join(" · "));
    }
    card(&title, &subs, t, width)
}
