use crate::{
    actions::*,
    backend::*,
    patch, theme, tools,
    ui::{overlay_rect, tab_at},
    views,
};
use ratatui::{
    crossterm::event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    layout::Rect,
    widgets::{ListState, TableState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};

pub const SCREENS: [(&str, &str); 8] = [
    ("\u{f02dc}", "Inicio"),
    ("\u{f00bd}", "Biblioteca"),
    ("\u{f01da}", "Instalaciones"),
    ("\u{f07c7}", "Tienda"),
    ("\u{f08ea}", "Mods y parches"),
    ("\u{f01a6}", "Complementos"),
    ("\u{f0849}", "Social"),
    ("\u{f0493}", "Sistema"),
];

pub const LOGO: &str = r#" █████╗ ███████╗██████╗ ██╗██████╗ ███████╗
██╔══██╗██╔════╝██╔══██╗██║██╔══██╗██╔════╝
███████║███████╗██████╔╝██║██████╔╝█████╗  
██╔══██║╚════██║██╔═══╝ ██║██╔══██╗██╔══╝  
██║  ██║███████║██║     ██║██║  ██║███████╗
╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝"#;

#[derive(Clone, Copy, PartialEq)]
pub enum Tag {
    Normal,
    LibAll,
    LibInst,
    Protons,
    ProtonList,
    Auth,
    Running,
    Libraries,
}

pub struct Job {
    pub title: String,
    pub args: Vec<String>,
    pub lines: Vec<(bool, String)>,
    pub errors: Vec<String>,
    pub done: bool,
    pub code: Option<i32>,
    pub result: Option<Value>,
    pub tag: Tag,
    pub progress: Option<views::Progress>,
}

pub enum PKind {
    Act(&'static Act, Option<String>),
    Palette,
    Filter,
    PatchDst(String),
    RevertDst,
}

pub struct Prompt {
    pub label: String,
    pub input: String,
    pub kind: PKind,
}

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

pub enum Overlay {
    None,
    Menu(usize),
    Prompt(Prompt),
    Launch(Launch),
    Theme { cursor: usize, orig: usize },
    Help,
    ProtonPick { act: &'static Act, versions: Vec<String>, cursor: usize },
    Links { cursor: usize },
    Files { path: PathBuf, entries: Vec<FileEntry>, cursor: usize },
    Confirm { msg: String, path: PathBuf, dir: PathBuf },
    Login { cursor: usize },
    Running { cursor: usize },
}

#[derive(Serialize, Deserialize, Default)]
struct Cfg {
    theme: Option<String>,
}

fn cfg_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("aspire-team")
}

pub struct App {
    pub screen: usize,
    pub sidebar_cursor: usize,
    pub cursor: [usize; 8],
    pub focus_out: bool,
    pub scroll: u16,
    pub jobs: Vec<Job>,
    pub cur: Option<usize>,
    pub show_job_out: bool,
    pub raw: bool,
    pub show_jobs: bool,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    pub theme: usize,
    pub overlay: Overlay,
    pub leader: bool,
    pub games: Vec<Game>,
    pub inst: HashSet<String>,
    pub filter: String,
    pub only_inst: bool,
    pub lib_state: TableState,
    pub list_state: ListState,
    pub ov_state: ListState,
    pub msg: String,
    pub tick: usize,
    pub quit: bool,
    pub pending: Option<Vec<String>>,
    pub logo: Vec<String>,
    pub aurelia_ok: bool,
    pub proton_versions: Vec<String>,
    pub proton_list: Vec<String>,
    pub proton_installed: Vec<String>,
    pub urls: Vec<String>,
    pub count: Option<usize>,
    pub g_pending: bool,
    pub last_search: String,
    pub search_dir: i32,
    pub account: Option<Value>,
    pub running_games: Vec<(String, String)>,
    pub libraries: Vec<(String, f64)>,
    pub need_login: bool,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let cfg: Cfg = std::fs::read_to_string(cfg_dir().join("config.toml"))
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        let logo_txt = std::fs::read_to_string(cfg_dir().join("logo.txt")).unwrap_or_else(|_| LOGO.to_string());
        let mut app = App {
            screen: 0,
            sidebar_cursor: 0,
            cursor: [0; 8],
            focus_out: false,
            scroll: 0,
            jobs: vec![],
            cur: None,
            show_job_out: false,
            raw: false,
            show_jobs: false,
            tx,
            rx,
            theme: theme::index_of(cfg.theme.as_deref().unwrap_or("Sistema")),
            overlay: Overlay::None,
            leader: false,
            games: vec![],
            inst: HashSet::new(),
            filter: String::new(),
            only_inst: false,
            lib_state: TableState::default(),
            list_state: ListState::default(),
            ov_state: ListState::default(),
            msg: String::new(),
            tick: 0,
            quit: false,
            pending: None,
            logo: logo_txt.lines().map(|s| s.to_string()).collect(),
            aurelia_ok: true,
            proton_versions: vec![],
            proton_list: vec![],
            proton_installed: vec![],
            urls: vec![],
            count: None,
            g_pending: false,
            last_search: String::new(),
            search_dir: 1,
            account: None,
            running_games: vec![],
            libraries: vec![],
            need_login: false,
        };
        app.reload_library();
        app.reload_protons();
        app.reload_proton_list();
        app.reload_running();
        app.reload_libraries();
        app.start_job("account".into(), vec!["account".into()], Tag::Auth);
        app
    }

    fn save_cfg(&self) {
        let dir = cfg_dir();
        let _ = std::fs::create_dir_all(&dir);
        let cfg = Cfg { theme: Some(theme::THEMES[self.theme].name.to_string()) };
        if let Ok(s) = toml::to_string(&cfg) {
            let _ = std::fs::write(dir.join("config.toml"), s);
        }
    }

    // ---------- biblioteca ----------
    pub fn is_inst(&self, g: &Game) -> bool {
        g.installed || self.inst.contains(&g.id)
    }

    pub fn filtered(&self) -> Vec<usize> {
        let f = self.filter.to_lowercase();
        self.games
            .iter()
            .enumerate()
            .filter(|(_, g)| (!self.only_inst || self.is_inst(g)) && (f.is_empty() || g.name.to_lowercase().contains(&f) || g.id.contains(&f)))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn selected_game(&self) -> Option<&Game> {
        let idx = self.filtered();
        idx.get(self.cursor[1].min(idx.len().saturating_sub(1))).map(|&i| &self.games[i])
    }

    pub fn reload_library(&mut self) {
        self.start_job("list".into(), vec!["list".into()], Tag::LibAll);
        self.reload_installed();
    }

    pub fn reload_installed(&mut self) {
        self.start_job("list --installed".into(), vec!["list".into(), "--installed".into()], Tag::LibInst);
    }

    pub fn reload_protons(&mut self) {
        self.start_job("config protons".into(), vec!["config".into(), "protons".into()], Tag::Protons);
    }

    pub fn reload_proton_list(&mut self) {
        self.start_job(
            "proton list --json".into(),
            vec!["proton".into(), "list".into()],
            Tag::ProtonList,
        );
    }

    pub fn reload_running(&mut self) {
        self.start_job("running".into(), vec!["running".into()], Tag::Running);
    }

    pub fn reload_libraries(&mut self) {
        self.start_job("libraries".into(), vec!["libraries".into()], Tag::Libraries);
    }

    // ---------- trabajos ----------
    fn new_job(&mut self, title: String, args: Vec<String>, tag: Tag) -> usize {
        let id = self.jobs.len();
        self.jobs.push(Job {
            title,
            args,
            lines: vec![],
            errors: vec![],
            done: false,
            code: None,
            result: None,
            tag,
            progress: None,
        });
        if tag == Tag::Normal {
            self.cur = Some(id);
            self.show_job_out = true;
            self.scroll = 0;
        }
        id
    }

    pub fn start_job(&mut self, title: String, args: Vec<String>, tag: Tag) {
        let title = if tag == Tag::Normal {
            views::job_title(&args, &self.games).unwrap_or(title)
        } else {
            title
        };
        let id = self.new_job(title, args.clone(), tag);
        spawn_job(id, args, self.tx.clone());
    }

    fn start_shell(&mut self, title: String, command: String) {
        let id = self.new_job(title, vec![], Tag::Normal);
        spawn_shell(id, command, self.tx.clone());
    }

    fn run_args(&mut self, args: Vec<String>) {
        if args.is_empty() {
            return;
        }
        if is_interactive(&args) {
            self.pending = Some(args);
        } else {
            let title = format!("aurelia {}", args.join(" "));
            self.start_job(title, args, Tag::Normal);
        }
    }

    pub fn running(&self) -> usize {
        self.jobs.iter().filter(|j| !j.done).count()
    }

    fn push_urls(&mut self, text: &str) {
        for w in text.split_whitespace() {
            if !w.starts_with("http://") && !w.starts_with("https://") {
                continue;
            }
            let mut u = w;
            while let Some(last) = u.chars().last() {
                if "),.';\"`]>".contains(last) {
                    u = &u[..u.len() - last.len_utf8()];
                } else {
                    break;
                }
            }
            if u.len() > 8 && self.urls.len() < 60 && !self.urls.iter().any(|x| x == u) {
                self.urls.push(u.to_string());
            }
        }
    }

    pub fn pump(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Line { job, err, text } => {
                    if !err {
                        self.push_urls(&text);
                        if let Ok(v) = serde_json::from_str::<Value>(&text) {
                            if let Some(p) = views::parse_progress(&v) {
                                if let Some(j) = self.jobs.get_mut(job) {
                                    if j.tag == Tag::Normal {
                                        j.progress = Some(p);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(j) = self.jobs.get_mut(job) {
                        if err {
                            j.errors.push(text.clone());
                        }
                        if j.tag == Tag::Normal {
                            j.lines.push((err, text));
                        }
                    }
                }
                Msg::Done { job, code, stdout } => {
                    let (tag, first, errors, has_out) = {
                        let Some(j) = self.jobs.get_mut(job) else { continue };
                        j.done = true;
                        j.code = code;
                        j.result = parse_result(&stdout);
                        j.progress = None;
                        // Los errores de Aurelia vienen en stdout como JSON.
                        if code != Some(0) {
                            if let Some(v) = &j.result {
                                if v.get("error").is_some() {
                                    j.errors.push(v.to_string());
                                }
                            }
                        }
                        let has_out = j.lines.iter().any(|(_, l)| !l.trim().is_empty()) || j.result.is_some();
                        (j.tag, j.args.first().cloned().unwrap_or_default(), j.errors.clone(), has_out)
                    };
                    if code.is_none() {
                        self.aurelia_ok = false;
                        self.msg = "No se encontró `aurelia`. Instálalo con: yay -S aurelia".into();
                    }
                    match tag {
                        Tag::LibAll => {
                            let g = self.jobs[job].result.as_ref().map(extract_games).unwrap_or_default();
                            if !g.is_empty() {
                                self.games = g;
                                self.need_login = false;
                            } else if code != Some(0) && code.is_some() && self.games.is_empty() {
                                self.msg = "No pude leer la biblioteca: inicia sesión en Sistema → Iniciar sesión".into();
                            }
                        }
                        Tag::LibInst => {
                            let g = self.jobs[job].result.as_ref().map(extract_games).unwrap_or_default();
                            if !g.is_empty() {
                                self.inst = g.iter().map(|x| x.id.clone()).collect();
                                if self.games.is_empty() {
                                    self.games = g;
                                }
                            }
                        }
                        Tag::Protons => {
                            if let Some(v) = &self.jobs[job].result {
                                let mut versions = vec![];
                                for key in ["custom", "steam"] {
                                    if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
                                        for s in arr {
                                            if let Some(s) = s.as_str() {
                                                versions.push(s.to_string());
                                            }
                                        }
                                    }
                                }
                                if !versions.is_empty() {
                                    self.proton_versions = versions;
                                }
                            }
                        }
                        Tag::ProtonList => {
                            if let Some(v) = &self.jobs[job].result {
                                if let Some(avail) = v.get("available").and_then(|x| x.as_array()) {
                                    let names: Vec<String> = avail
                                        .iter()
                                        .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                                        .collect();
                                    if !names.is_empty() {
                                        self.proton_list = names;
                                    }
                                }
                                if let Some(inst) = v.get("installed").and_then(|x| x.as_array()) {
                                    let names: Vec<String> = inst
                                        .iter()
                                        .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                                        .collect();
                                    if !names.is_empty() {
                                        self.proton_installed = names;
                                    }
                                }
                            }
                        }
                        Tag::Running => {
                            if let Some(v) = &self.jobs[job].result {
                                if let Some(a) = v.get("running").and_then(|x| x.as_array()) {
                                    self.running_games = a
                                        .iter()
                                        .map(|g| {
                                            let id = g
                                                .get("app_id")
                                                .map(|x| {
                                                    x.as_u64()
                                                        .map(|n| n.to_string())
                                                        .or_else(|| x.as_str().map(|s| s.to_string()))
                                                        .unwrap_or_else(|| x.to_string())
                                                })
                                                .unwrap_or_default();
                                            let name = g
                                                .get("name")
                                                .and_then(|x| x.as_str())
                                                .map(|s| s.to_string())
                                                .unwrap_or_else(|| format!("AppID {id}"));
                                            (id, name)
                                        })
                                        .collect();
                                }
                            }
                        }
                        Tag::Libraries => {
                            if let Some(v) = &self.jobs[job].result {
                                if let Some(a) = v.get("libraries").and_then(|x| x.as_array()) {
                                    self.libraries = a
                                        .iter()
                                        .filter_map(|l| {
                                            Some((
                                                l.get("path")?.as_str()?.to_string(),
                                                l.get("free_bytes").and_then(|x| x.as_f64()).unwrap_or(0.0),
                                            ))
                                        })
                                        .collect();
                                }
                            }
                        }
                        Tag::Auth => {
                            if code == Some(0) {
                                self.account = self.jobs[job].result.clone();
                                self.need_login = false;
                            } else if views::is_auth_error(&errors, code) {
                                self.need_login = true;
                                self.screen = 7;
                                self.sidebar_cursor = 7;
                                self.overlay = Overlay::Login { cursor: 0 };
                                self.msg = "Inicia sesión en Steam para continuar".into();
                            }
                        }
                        Tag::Normal => {
                            let ok = code == Some(0);
                            let title = self.jobs[job].title.clone();
                            self.msg = if ok {
                                if has_out {
                                    format!("✓ {title}")
                                } else {
                                    format!("✓ {title} — sin resultados")
                                }
                            } else {
                                format!("✗ {}", views::short_error(&errors, code))
                            };
                            if ok && ["install", "uninstall", "update", "move", "verify"].contains(&first.as_str()) {
                                self.reload_installed();
                            }
                            if ["stop", "play"].contains(&first.as_str()) {
                                self.reload_running();
                            }
                        }
                    }
                    if views::is_auth_error(&errors, code) && tag != Tag::Auth {
                        self.msg = "Necesitas iniciar sesión en Steam (Sistema → Iniciar sesión)".into();
                        self.need_login = true;
                        if tag == Tag::Normal && matches!(self.overlay, Overlay::None) {
                            self.screen = 7;
                            self.sidebar_cursor = 7;
                            self.overlay = Overlay::Login { cursor: 0 };
                        }
                    }
                }
            }
        }
    }

    pub fn after_interactive(&mut self) {
        self.reload_library();
        self.reload_running();
        self.start_job("account".into(), vec!["account".into()], Tag::Auth);
        self.msg = "Volviste de aurelia".into();
    }

    // ---------- acciones ----------
    pub fn open_menu(&mut self) {
        if self.selected_game().is_some() {
            self.overlay = Overlay::Menu(0);
        } else {
            self.msg = "No hay juego seleccionado".into();
        }
    }

    fn prompt(&mut self, label: &str, input: String, kind: PKind) {
        self.overlay = Overlay::Prompt(Prompt { label: label.to_string(), input, kind });
    }

    pub fn trigger(&mut self, act: &'static Act) {
        let app = self.selected_game().map(|g| g.id.clone());
        if act.game && app.is_none() {
            self.msg = "Elige un juego en Biblioteca primero".into();
            return;
        }
        if act.mode == Mode::Launch {
            if let Some(g) = self.selected_game() {
                self.overlay = Overlay::Launch(Launch::new(g.id.clone(), g.name.clone(), &self.proton_versions));
            }
            return;
        }
        if act.mode == Mode::Pick {
            let mut versions = if act.args.starts_with("proton uninstall") {
                if !self.proton_installed.is_empty() {
                    self.proton_installed.clone()
                } else {
                    self.proton_versions.clone()
                }
            } else {
                self.proton_list.clone()
            };
            if versions.is_empty() {
                versions = self.proton_versions.clone();
            }
            self.overlay = Overlay::ProtonPick { act, versions, cursor: 0 };
            return;
        }
        if let Some(label) = act.ask {
            let pre = match (&app, label.starts_with("AppID")) {
                (Some(id), true) => {
                    if label.trim() == "AppID" {
                        id.clone()
                    } else {
                        format!("{id} ")
                    }
                }
                _ => String::new(),
            };
            self.prompt(label, pre, PKind::Act(act, app));
            return;
        }
        self.exec(act, app, String::new());
    }

    fn game_dir(&self, id: &str) -> Option<String> {
        self.games.iter().find(|g| g.id == id).and_then(install_dir)
    }

    fn exec(&mut self, act: &'static Act, app: Option<String>, input: String) {
        match act.mode {
            Mode::Job | Mode::Interactive | Mode::Pick => {
                let args = build_args(act.args, app.as_deref(), &input);
                self.run_args(args);
            }
            Mode::Launch => {}
            Mode::Patch => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(dst) => self.start_patch(input.trim().to_string(), dst),
                    None => self.prompt(
                        "Carpeta de instalación del juego",
                        String::new(),
                        PKind::PatchDst(input.trim().to_string()),
                    ),
                }
            }
            Mode::Revert => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(dst) => self.start_revert(dst),
                    None => self.prompt("Carpeta de instalación del juego", String::new(), PKind::RevertDst),
                }
            }
            Mode::OpenDir => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(d) => {
                        let _ = tools::open_path(&d);
                    }
                    None => self.msg = "Este juego no está instalado (no hay carpeta conocida).".into(),
                }
            }
            Mode::Browse => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(d) => self.open_files(PathBuf::from(d)),
                    None => self.msg = "Este juego no está instalado (no hay carpeta conocida).".into(),
                }
            }
            Mode::Saves => self.open_saves(),
            Mode::OpenPrefix => {
                let Some(id) = app else { return };
                match tools::prefix_dir(&id) {
                    Some(p) => {
                        let _ = tools::open_path(&p.to_string_lossy());
                    }
                    None => self.msg = "Este juego todavía no tiene prefijo de Wine.".into(),
                }
            }
            Mode::OpenUrl => {
                let Some(id) = app else { return };
                let url = self
                    .selected_game()
                    .and_then(|g| g.raw.get("store_url").and_then(|x| x.as_str()).map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("https://store.steampowered.com/app/{id}"));
                let _ = tools::open_url(&url);
            }
            Mode::DiskInfo => {
                let Some(id) = app else { return };
                self.start_disk_info(id);
            }
            Mode::LaunchPrefix => {
                let Some(id) = app else { return };
                self.start_launch_prefix(id, act.args);
            }
            Mode::Shell | Mode::ShellInteractive => {
                let id = app.unwrap_or_default();
                let dir = self.game_dir(&id).unwrap_or_default();
                let cmd = act.args.replace("{appdir}", &dir).replace("{app}", &id);
                if act.mode == Mode::Shell {
                    self.start_shell(act.label.to_string(), cmd);
                } else {
                    self.pending = Some(vec!["__shell".into(), cmd]);
                }
            }
            Mode::ExtStatus => {
                let job = self.new_job("Extensiones del sistema".into(), vec![], Tag::Normal);
                spawn_local(job, self.tx.clone(), move |log| tools::extensions_status(log));
            }
        }
    }

    fn start_patch(&mut self, src: String, dst: String) {
        let id = self.new_job(format!("parche {src} → {dst}"), vec![], Tag::Normal);
        spawn_local(id, self.tx.clone(), move |log| {
            patch::apply(std::path::Path::new(&src), std::path::Path::new(&dst), log)
        });
    }

    fn start_revert(&mut self, dst: String) {
        let id = self.new_job(format!("revertir parche en {dst}"), vec![], Tag::Normal);
        spawn_local(id, self.tx.clone(), move |log| patch::revert(std::path::Path::new(&dst), log));
    }

    fn start_disk_info(&mut self, id: String) {
        let Some(dir) = self.game_dir(&id) else {
            self.msg = "Este juego no está instalado (no hay carpeta conocida).".into();
            return;
        };
        let job = self.new_job(format!("Tamaño y disco · {dir}"), vec![], Tag::Normal);
        spawn_local(job, self.tx.clone(), move |log| tools::disk_info(&dir, log));
    }

    fn start_launch_prefix(&mut self, id: String, prefix: &'static str) {
        let job = self.new_job(format!("Opciones de lanzamiento · {prefix}"), vec![], Tag::Normal);
        spawn_local(job, self.tx.clone(), move |log| tools::launch_prefix(&id, prefix, log));
    }

    fn open_saves(&mut self) {
        let id = match self.selected_game() {
            Some(g) => g.id.clone(),
            None => {
                self.msg = "Elige un juego primero".into();
                return;
            }
        };
        match tools::saves_dir(&id) {
            Some(p) => {
                let _ = tools::open_path(&p.to_string_lossy());
            }
            None => self.msg = "Steam no tiene guardados para este juego.".into(),
        }
    }

    fn open_files(&mut self, path: PathBuf) {
        match load_entries(&path) {
            Ok(entries) => self.overlay = Overlay::Files { path, entries, cursor: 0 },
            Err(e) => self.msg = format!("No pude abrir la carpeta: {e}"),
        }
    }

    fn stop_game(&mut self, id: String, force: bool) {
        let mut args = vec!["stop".to_string(), id];
        if force {
            args.push("--force".into());
        }
        self.run_args(args);
        self.overlay = Overlay::None;
    }

    fn login_activate(&mut self, i: usize) {
        match i {
            0 => self.pending = Some(vec!["login".into()]),
            1 => self.pending = Some(vec!["login".into(), "--qr".into(), "--timeout".into(), "120".into()]),
            _ => self.quit = true,
        }
        self.overlay = Overlay::None;
    }

    // ---------- ratón ----------
    pub fn on_mouse(&mut self, m: MouseEvent, size: (u16, u16)) {
        let (w, h) = size;
        let area = Rect::new(0, 3, w, h.saturating_sub(5));
        match m.kind {
            MouseEventKind::ScrollDown => self.move_by(3),
            MouseEventKind::ScrollUp => self.move_by(-3),
            MouseEventKind::Down(MouseButton::Left) => {
                if !matches!(self.overlay, Overlay::None) {
                    let r = overlay_rect(&self.overlay, self, area);
                    let idx = (m.row as usize).saturating_sub(r.y as usize + 1);
                    match &self.overlay {
                        Overlay::Menu(c) => {
                            let c = *c;
                            if idx < GAME_ACTS.len() {
                                if idx == c {
                                    self.trigger(&GAME_ACTS[idx]);
                                } else {
                                    self.overlay = Overlay::Menu(idx);
                                }
                            } else {
                                self.overlay = Overlay::None;
                            }
                        }
                        Overlay::Theme { cursor, orig } => {
                            let (cur, orig) = (*cursor, *orig);
                            if idx < theme::THEMES.len() {
                                if idx == cur {
                                    self.theme = idx;
                                    self.save_cfg();
                                    self.overlay = Overlay::None;
                                } else {
                                    self.theme = idx;
                                    self.overlay = Overlay::Theme { cursor: idx, orig };
                                }
                            }
                        }
                        Overlay::Help | Overlay::Prompt(_) | Overlay::Launch(_) => {
                            self.overlay = Overlay::None;
                        }
                        Overlay::ProtonPick { act, versions, cursor } => {
                            let (act, cur, n) = (*act, *cursor, versions.len());
                            if idx < n {
                                if idx == cur {
                                    let v = versions[idx].clone();
                                    self.overlay = Overlay::None;
                                    self.exec(act, None, v);
                                } else {
                                    self.overlay = Overlay::ProtonPick { act, versions: versions.clone(), cursor: idx };
                                }
                            }
                        }
                        Overlay::Links { .. } => {
                            if idx < self.urls.len() {
                                let u = self.urls[idx].clone();
                                let _ = tools::open_url(&u);
                            }
                            self.overlay = Overlay::None;
                        }
                        Overlay::Files { path, entries, cursor } => {
                            let (path, entries, cur) = (path.clone(), entries.clone(), *cursor);
                            if idx < entries.len() {
                                if idx == cur {
                                    self.file_activate(&path, &entries[idx]);
                                } else {
                                    self.overlay = Overlay::Files { path, entries, cursor: idx };
                                }
                            }
                        }
                        Overlay::Confirm { dir, .. } => {
                            let dir = dir.clone();
                            self.open_files(dir);
                        }
                        Overlay::Login { cursor } => {
                            let cur = *cursor;
                            let idx = idx.saturating_sub(1);
                            if idx < 3 {
                                if idx == cur {
                                    self.login_activate(idx);
                                } else {
                                    self.overlay = Overlay::Login { cursor: idx };
                                }
                            }
                        }
                        Overlay::Running { cursor } => {
                            let cur = *cursor;
                            let n = self.running_games.len();
                            if idx < n {
                                if idx == cur {
                                    let id = self.running_games[idx].0.clone();
                                    self.stop_game(id, false);
                                } else {
                                    self.overlay = Overlay::Running { cursor: idx };
                                }
                            }
                        }
                        Overlay::None => {}
                    }
                    return;
                }
                // Pestañas (filas 0-2)
                if m.row < 3 {
                    if let Some(i) = tab_at(w, m.column) {
                        self.goto(i);
                    }
                    return;
                }
                // Biblioteca
                if self.screen == 1 {
                    let first = area.y as usize + 1;
                    let off = self.lib_state.offset();
                    let idx = off + (m.row as usize).saturating_sub(first);
                    let filtered = self.filtered();
                    if idx < filtered.len() {
                        self.cursor[1] = idx;
                        self.show_job_out = false;
                    }
                    return;
                }
                // Listas de acciones (pantallas 2-7)
                if self.screen >= 2 {
                    let first = area.y as usize + 1;
                    let off = self.list_state.offset();
                    let idx = off + (m.row as usize).saturating_sub(first);
                    let acts = acts_for(self.screen);
                    if idx < acts.len() {
                        self.cursor[self.screen] = idx;
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.overlay = Overlay::None;
            }
            _ => {}
        }
    }

    fn file_activate(&mut self, path: &Path, e: &FileEntry) {
        if e.is_dir {
            match load_entries(&e.path) {
                Ok(entries) => self.overlay = Overlay::Files { path: e.path.clone(), entries, cursor: 0 },
                Err(err) => self.msg = format!("No pude abrir la carpeta: {err}"),
            }
        } else {
            let _ = tools::open_path(&e.path.to_string_lossy());
        }
        let _ = path;
    }

    // ---------- teclado ----------
    pub fn on_key(&mut self, k: KeyEvent) {
        if k.kind == KeyEventKind::Release {
            return;
        }
        let ov = std::mem::replace(&mut self.overlay, Overlay::None);
        match ov {
            Overlay::None => self.key_normal(k),
            Overlay::Menu(c) => self.key_menu(c, k),
            Overlay::Prompt(p) => self.key_prompt(p, k),
            Overlay::Launch(l) => self.key_launch(l, k),
            Overlay::Theme { cursor, orig } => self.key_theme(cursor, orig, k),
            Overlay::Help => {}
            Overlay::ProtonPick { act, versions, cursor } => self.key_proton_pick(act, versions, cursor, k),
            Overlay::Links { cursor } => self.key_links(cursor, k),
            Overlay::Files { path, entries, cursor } => self.key_files(path, entries, cursor, k),
            Overlay::Confirm { msg, path, dir } => self.key_confirm(msg, path, dir, k),
            Overlay::Login { cursor } => self.key_login(cursor, k),
            Overlay::Running { cursor } => self.key_running(cursor, k),
        }
    }

    fn key_menu(&mut self, mut c: usize, k: KeyEvent) {
        let n = GAME_ACTS.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => c = (c + n - 1) % n,
            KeyCode::Char('g') => c = 0,
            KeyCode::Char('G') => c = n - 1,
            KeyCode::Enter | KeyCode::Char('n') => {
                self.trigger(&GAME_ACTS[c]);
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Menu(c);
    }

    fn key_prompt(&mut self, mut p: Prompt, k: KeyEvent) {
        match k.code {
            KeyCode::Esc => {
                if matches!(p.kind, PKind::Filter) {
                    self.filter.clear();
                }
                return;
            }
            KeyCode::Enter => {
                let input = p.input.clone();
                match p.kind {
                    PKind::Act(act, app) => self.exec(act, app, input),
                    PKind::Palette => {
                        let trimmed = input.trim().to_lowercase();
                        if trimmed == "q" || trimmed == "quit" || trimmed == "q!" || trimmed == "exit" {
                            self.quit = true;
                            return;
                        }
                        let mut args = split_args(&input);
                        if args.first().map(|s| s == "aurelia").unwrap_or(false) {
                            args.remove(0);
                        }
                        self.run_args(args);
                    }
                    PKind::Filter => {
                        self.last_search = input.clone();
                        self.search_dir = 1;
                        self.cursor[1] = 0;
                    }
                    PKind::PatchDst(src) => self.start_patch(src, input.trim().to_string()),
                    PKind::RevertDst => self.start_revert(input.trim().to_string()),
                }
                return;
            }
            KeyCode::Backspace => {
                p.input.pop();
            }
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => p.input.push(c),
            KeyCode::Char('u') => p.input.clear(),
            _ => {}
        }
        if matches!(p.kind, PKind::Filter) {
            self.filter = p.input.clone();
            self.cursor[1] = 0;
        }
        self.overlay = Overlay::Prompt(p);
    }

    fn key_launch(&mut self, mut l: Launch, k: KeyEvent) {
        let n = l.rows.len();
        if l.editing {
            let i = l.cursor;
            match k.code {
                KeyCode::Enter | KeyCode::Esc => l.editing = false,
                KeyCode::Backspace => {
                    l.rows[i].text.pop();
                }
                KeyCode::Char(c) => l.rows[i].text.push(c),
                _ => {}
            }
            self.overlay = Overlay::Launch(l);
            return;
        }
        let i = l.cursor;
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') => l.cursor = (i + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => l.cursor = (i + n - 1) % n,
            KeyCode::Left | KeyCode::Char('h') => {
                let r = &mut l.rows[i];
                if !r.opts.is_empty() {
                    r.idx = (r.idx + r.opts.len() - 1) % r.opts.len();
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') | KeyCode::Tab => {
                let r = &mut l.rows[i];
                if r.opts.is_empty() {
                    l.editing = true;
                } else {
                    r.idx = (r.idx + 1) % r.opts.len();
                }
            }
            KeyCode::Char('e') | KeyCode::Char('i') => {
                if l.rows[i].opts.is_empty() {
                    l.editing = true;
                }
            }
            KeyCode::Char('d') => l.rows[i].text.clear(),
            KeyCode::Char('s') => {
                match l.save_args() {
                    Some(a) => self.run_args(a),
                    None => self.msg = "Nada que guardar (Proton, parámetros Steam, umu o motor nativo)".into(),
                }
                self.overlay = Overlay::Launch(l);
                return;
            }
            KeyCode::Char('n') | KeyCode::Char('r') | KeyCode::Enter => {
                let args = l.args();
                self.run_args(args);
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Launch(l);
    }

    fn key_theme(&mut self, mut c: usize, orig: usize, k: KeyEvent) {
        let n = theme::THEMES.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.theme = orig;
                return;
            }
            KeyCode::Enter | KeyCode::Char('n') => {
                self.save_cfg();
                return;
            }
            KeyCode::Down | KeyCode::Char('j') => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => c = (c + n - 1) % n,
            _ => {}
        }
        self.theme = c;
        self.overlay = Overlay::Theme { cursor: c, orig };
    }

    fn key_links(&mut self, mut c: usize, k: KeyEvent) {
        let n = self.urls.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') if n > 0 => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') if n > 0 => c = (c + n - 1) % n,
            KeyCode::Enter | KeyCode::Char('n') if n > 0 => {
                if let Some(u) = self.urls.get(c) {
                    let _ = tools::open_url(u);
                }
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Links { cursor: c };
    }

    fn key_proton_pick(&mut self, act: &'static Act, versions: Vec<String>, mut c: usize, k: KeyEvent) {
        let n = versions.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') if n > 0 => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') if n > 0 => c = (c + n - 1) % n,
            KeyCode::Enter | KeyCode::Char('n') => {
                if let Some(v) = versions.get(c) {
                    let v = v.clone();
                    self.overlay = Overlay::None;
                    self.exec(act, None, v);
                }
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::ProtonPick { act, versions, cursor: c };
    }

    fn key_files(&mut self, path: PathBuf, entries: Vec<FileEntry>, mut c: usize, k: KeyEvent) {
        let n = entries.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') if n > 0 => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') if n > 0 => c = (c + n - 1) % n,
            KeyCode::Char('g') => c = 0,
            KeyCode::Char('G') => c = n.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char('n') | KeyCode::Char('l') => {
                if let Some(e) = entries.get(c) {
                    self.file_activate(&path, e);
                }
                return;
            }
            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                if let Some(p) = path.parent() {
                    let p = p.to_path_buf();
                    self.open_files(p);
                }
                return;
            }
            KeyCode::Char('o') => {
                let _ = tools::open_path(&path.to_string_lossy());
                self.overlay = Overlay::Files { path, entries, cursor: c };
                return;
            }
            KeyCode::Char('d') => {
                if let Some(e) = entries.get(c) {
                    self.overlay = Overlay::Confirm {
                        msg: format!("¿Borrar «{}»? No se puede deshacer.", e.name),
                        path: e.path.clone(),
                        dir: path.clone(),
                    };
                }
                return;
            }
            KeyCode::Char('r') => {
                self.open_files(path);
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Files { path, entries, cursor: c };
    }

    fn key_confirm(&mut self, msg: String, path: PathBuf, dir: PathBuf, k: KeyEvent) {
        match k.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => match tools::remove_path(&path) {
                Ok(()) => {
                    self.msg = "✓ Borrado".into();
                    self.open_files(dir);
                }
                Err(e) => {
                    self.msg = format!("No pude borrar: {e}");
                    self.open_files(dir);
                }
            },
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => self.open_files(dir),
            _ => self.overlay = Overlay::Confirm { msg, path, dir },
        }
    }

    fn key_login(&mut self, mut c: usize, k: KeyEvent) {
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') => c = (c + 1) % 3,
            KeyCode::Up | KeyCode::Char('k') => c = (c + 2) % 3,
            KeyCode::Enter | KeyCode::Char('n') => {
                self.login_activate(c);
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Login { cursor: c };
    }

    fn key_running(&mut self, mut c: usize, k: KeyEvent) {
        let n = self.running_games.len();
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => return,
            KeyCode::Down | KeyCode::Char('j') if n > 0 => c = (c + 1) % n,
            KeyCode::Up | KeyCode::Char('k') if n > 0 => c = (c + n - 1) % n,
            KeyCode::Char('r') => {
                self.reload_running();
                self.overlay = Overlay::Running { cursor: c };
                return;
            }
            KeyCode::Enter | KeyCode::Char('n') if n > 0 => {
                let id = self.running_games[c].0.clone();
                self.stop_game(id, false);
                return;
            }
            KeyCode::Char('f') | KeyCode::Char('F') if n > 0 => {
                let id = self.running_games[c].0.clone();
                self.stop_game(id, true);
                return;
            }
            _ => {}
        }
        self.overlay = Overlay::Running { cursor: c };
    }

    fn list_len(&self) -> usize {
        match self.screen {
            1 => self.filtered().len(),
            s => acts_for(s).len(),
        }
    }

    fn move_by(&mut self, d: i32) {
        if self.focus_out || self.screen == 0 {
            self.scroll = (self.scroll as i32 + d).max(0) as u16;
            return;
        }
        let n = self.list_len();
        if n == 0 {
            return;
        }
        let c = self.cursor[self.screen] as i32 + d;
        self.cursor[self.screen] = c.clamp(0, n as i32 - 1) as usize;
        if self.screen == 1 {
            self.show_job_out = false;
        }
    }

    fn goto(&mut self, s: usize) {
        self.screen = s.min(7);
        self.sidebar_cursor = self.screen;
        self.focus_out = false;
        self.scroll = 0;
        self.list_state = ListState::default();
    }

    fn key_normal(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && k.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }

        // ---- prefijo g (gg, gt, gT, g1..g8) ----
        if self.g_pending {
            self.g_pending = false;
            match k.code {
                KeyCode::Char('g') => {
                    if self.focus_out {
                        self.scroll = 0;
                    } else {
                        self.sidebar_cursor = 0;
                    }
                    return;
                }
                KeyCode::Char(c @ '1'..='8') => {
                    self.goto(c as usize - '1' as usize);
                    return;
                }
                _ => return,
            }
        }

        // ---- tecla líder (which-key) ----
        if self.leader {
            self.leader = false;
            match k.code {
                KeyCode::Char('h') => self.goto(0),
                KeyCode::Char('l') => self.goto(1),
                KeyCode::Char('i') => self.goto(2),
                KeyCode::Char('s') => self.goto(3),
                KeyCode::Char('m') => self.goto(4),
                KeyCode::Char('p') => self.goto(5),
                KeyCode::Char('f') => self.goto(6),
                KeyCode::Char('y') => self.goto(7),
                KeyCode::Char('t') => self.overlay = Overlay::Theme { cursor: self.theme, orig: self.theme },
                KeyCode::Char('j') => self.show_jobs = !self.show_jobs,
                KeyCode::Char('c') => self.prompt("aurelia", String::new(), PKind::Palette),
                KeyCode::Char('r') => self.reload_library(),
                KeyCode::Char('x') => {
                    self.reload_running();
                    self.overlay = Overlay::Running { cursor: 0 };
                }
                KeyCode::Char('q') => self.quit = true,
                _ => {}
            }
            return;
        }

        // ---- teclas globales ----
        match k.code {
            KeyCode::Char(' ') => {
                self.leader = true;
                return;
            }
            KeyCode::Char('T') => {
                self.overlay = Overlay::Theme { cursor: self.theme, orig: self.theme };
                return;
            }
            KeyCode::Char('U') => {
                if !self.urls.is_empty() {
                    self.overlay = Overlay::Links { cursor: 0 };
                }
                return;
            }
            KeyCode::Char('X') => {
                self.reload_running();
                self.overlay = Overlay::Running { cursor: 0 };
                return;
            }
            KeyCode::Char('L') if self.need_login => {
                self.overlay = Overlay::Login { cursor: 0 };
                return;
            }
            KeyCode::Char('?') => {
                self.overlay = Overlay::Help;
                return;
            }
            KeyCode::Char(':') => {
                self.prompt("aurelia", String::new(), PKind::Palette);
                return;
            }
            KeyCode::Char('g') => {
                self.g_pending = true;
                return;
            }
            KeyCode::Char(c @ '1'..='8') => {
                self.goto(c as usize - '1' as usize);
                return;
            }
            KeyCode::Char('n') if self.screen != 0 => {
                self.activate();
                return;
            }
            KeyCode::Char('v') => {
                self.raw = !self.raw;
                return;
            }
            KeyCode::Char('[') => {
                self.cycle_job(-1);
                return;
            }
            KeyCode::Char(']') => {
                self.cycle_job(1);
                return;
            }
            KeyCode::Tab => {
                self.goto((self.screen + 1) % 8);
                return;
            }
            KeyCode::BackTab => {
                self.goto((self.screen + 7) % 8);
                return;
            }
            _ => {}
        }

        // ---- barra de pestañas (solo en Inicio) ----
        if !self.focus_out && self.screen == 0 {
            match k.code {
                KeyCode::Char('j') | KeyCode::Down => self.sidebar_cursor = (self.sidebar_cursor + 1) % 8,
                KeyCode::Char('k') | KeyCode::Up => self.sidebar_cursor = (self.sidebar_cursor + 7) % 8,
                KeyCode::Char('g') => self.sidebar_cursor = 0,
                KeyCode::Char('G') => self.sidebar_cursor = 7,
                KeyCode::Enter | KeyCode::Char('n') => self.goto(self.sidebar_cursor),
                KeyCode::Char('l') | KeyCode::Right => {
                    self.goto(self.sidebar_cursor);
                }
                KeyCode::Char('h') | KeyCode::Left => {}
                _ => {}
            }
            return;
        }

        // ---- contenido enfocado ----
        match k.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_by(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_by(-1),
            KeyCode::Char('d') if ctrl => self.move_by(10),
            KeyCode::Char('u') if ctrl => self.move_by(-10),
            KeyCode::Char('f') if ctrl => self.move_by(20),
            KeyCode::Char('b') if ctrl => self.move_by(-20),
            KeyCode::PageDown => self.move_by(10),
            KeyCode::PageUp => self.move_by(-10),
            KeyCode::Char('G') => {
                if !self.focus_out && self.screen > 0 {
                    self.cursor[self.screen] = self.list_len().saturating_sub(1)
                }
            }
            KeyCode::Char('$') => {
                if !self.focus_out && self.screen > 0 {
                    self.cursor[self.screen] = self.list_len().saturating_sub(1)
                }
            }
            KeyCode::Char('H') => self.move_by(-(self.list_len() as i32)),
            KeyCode::Char('M') => {
                if self.screen > 0 {
                    self.cursor[self.screen] = self.list_len() / 2;
                }
            }
            KeyCode::Char('L') => {
                if self.screen > 0 {
                    self.cursor[self.screen] = self.list_len().saturating_sub(1);
                }
            }
            KeyCode::Char('h') | KeyCode::Left => self.focus_out = false,
            KeyCode::Char('l') | KeyCode::Right => self.focus_out = true,
            KeyCode::Char('v') => self.raw = !self.raw,
            KeyCode::Char('N') => self.search_next(-1),
            KeyCode::Char('r') => {
                if self.screen == 1 {
                    self.reload_library();
                    self.msg = "Recargando biblioteca…".into();
                }
            }
            KeyCode::Enter => self.activate(),
            KeyCode::Char('/') if self.screen == 1 => self.prompt("Filtrar", self.filter.clone(), PKind::Filter),
            KeyCode::Char('a') if self.screen == 1 => self.open_menu(),
            KeyCode::Char('p') if self.screen == 1 => {
                if let Some(a) = game_act("Jugar") {
                    self.trigger(a)
                }
            }
            KeyCode::Char('o') if self.screen == 1 => {
                if let Some(a) = game_act("Jugar con opciones…") {
                    self.trigger(a)
                }
            }
            KeyCode::Char('I') if self.screen == 1 => {
                if let Some(a) = game_act("Instalar") {
                    self.trigger(a)
                }
            }
            KeyCode::Char('u') if self.screen == 1 => {
                if let Some(a) = game_act("Actualizar") {
                    self.trigger(a)
                }
            }
            KeyCode::Char('K') if self.screen == 1 => {
                if let Some(a) = game_act("Información") {
                    self.trigger(a)
                }
            }
            KeyCode::Char('F') if self.screen == 1 => {
                let id = match self.selected_game() {
                    Some(g) => g.id.clone(),
                    None => {
                        self.msg = "Elige un juego primero".into();
                        return;
                    }
                };
                match self.game_dir(&id) {
                    Some(d) => self.open_files(PathBuf::from(d)),
                    None => self.msg = "Este juego no está instalado (no hay carpeta conocida).".into(),
                }
            }
            KeyCode::Char('D') if self.screen == 1 => {
                if let Some(g) = self.selected_game() {
                    let id = g.id.clone();
                    self.start_disk_info(id);
                }
            }
            KeyCode::Char('i') if self.screen == 1 => {
                self.only_inst = !self.only_inst;
                self.cursor[1] = 0;
            }
            _ => {}
        }
    }

    fn search_next(&mut self, dir: i32) {
        if self.screen != 1 {
            return;
        }
        let filtered = self.filtered();
        let n = filtered.len();
        if n == 0 || self.last_search.is_empty() {
            return;
        }
        let f = self.last_search.to_lowercase();
        let start = self.cursor[1] as i32;
        for step in 1..=n {
            let idx = ((start + dir * step as i32).rem_euclid(n as i32)) as usize;
            if idx < filtered.len() {
                let g = &self.games[filtered[idx]];
                if g.name.to_lowercase().contains(&f) || g.id.contains(&f) {
                    self.cursor[1] = idx;
                    return;
                }
            }
        }
    }

    fn cycle_job(&mut self, d: i32) {
        let ids: Vec<usize> = self.jobs.iter().enumerate().filter(|(_, j)| j.tag == Tag::Normal).map(|(i, _)| i).collect();
        if ids.is_empty() {
            return;
        }
        let pos = self.cur.and_then(|c| ids.iter().position(|&i| i == c)).unwrap_or(ids.len() - 1) as i32;
        self.cur = Some(ids[(pos + d).rem_euclid(ids.len() as i32) as usize]);
        self.show_job_out = true;
        self.scroll = 0;
    }

    fn activate(&mut self) {
        match self.screen {
            1 => self.open_menu(),
            0 => self.goto(1),
            s => {
                let acts = acts_for(s);
                if let Some(a) = acts.get(self.cursor[s]) {
                    self.trigger(a);
                }
            }
        }
    }
}

pub fn load_entries(path: &Path) -> std::io::Result<Vec<FileEntry>> {
    let mut v = vec![];
    for e in std::fs::read_dir(path)? {
        let Ok(e) = e else { continue };
        let p = e.path();
        let md = std::fs::symlink_metadata(&p).ok();
        let is_dir = md.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = md.as_ref().map(|m| m.len()).unwrap_or(0);
        let name = e.file_name().to_string_lossy().to_string();
        v.push(FileEntry { name, path: p, is_dir, size });
    }
    v.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(v)
}
