use crate::{actions::*, backend::*, patch, theme};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::{ListState, TableState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

pub const SCREENS: [(&str, &str); 8] = [
    ("◆", "Inicio"),
    ("▣", "Biblioteca"),
    ("⤓", "Instalaciones"),
    ("★", "Tienda"),
    ("⚒", "Mods y parches"),
    ("⊕", "Complementos"),
    ("☻", "Social"),
    ("⚙", "Sistema"),
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
}

pub struct Job {
    pub title: String,
    pub args: Vec<String>,
    pub lines: Vec<(bool, String)>,
    pub done: bool,
    pub code: Option<i32>,
    pub result: Option<Value>,
    pub tag: Tag,
    pub progress: String,
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

pub enum Overlay {
    None,
    Menu(usize),
    Prompt(Prompt),
    Launch(Launch),
    Theme { cursor: usize, orig: usize },
    Help,
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
    pub count: Option<usize>,
    pub g_pending: bool,
    pub last_search: String,
    pub search_dir: i32,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let cfg: Cfg = std::fs::read_to_string(cfg_dir().join("config.toml")).ok().and_then(|s| toml::from_str(&s).ok()).unwrap_or_default();
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
            theme: theme::index_of(if let Some(name) = cfg.theme.as_deref() {
                if name == "Sistema" { "Sistema" } else { name }
            } else { "Sistema" }),
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
            count: None,
            g_pending: false,
            last_search: String::new(),
            search_dir: 1,
        };
        app.reload_library();
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

    // ---------- trabajos ----------
    fn new_job(&mut self, title: String, args: Vec<String>, tag: Tag) -> usize {
        let id = self.jobs.len();
        self.jobs.push(Job { title, args, lines: vec![], done: false, code: None, result: None, tag, progress: String::new() });
        if tag == Tag::Normal {
            self.cur = Some(id);
            self.show_job_out = true;
            self.scroll = 0;
        }
        id
    }

    pub fn start_job(&mut self, title: String, args: Vec<String>, tag: Tag) {
        let id = self.new_job(title, args.clone(), tag);
        spawn_job(id, args, self.tx.clone());
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

    pub fn pump(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Line { job, err, text } => {
                    if let Some(j) = self.jobs.get_mut(job) {
                        if !err {
                            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                                if v.get("event").and_then(|e| e.as_str()) == Some("progress") {
                                    j.progress = fmt_progress(&v);
                                }
                            }
                        }
                        if j.tag == Tag::Normal {
                            j.lines.push((err, text));
                        }
                    }
                }
                Msg::Done { job, code, stdout } => {
                    let Some(j) = self.jobs.get_mut(job) else { continue };
                    j.done = true;
                    j.code = code;
                    j.result = parse_result(&stdout);
                    let tag = j.tag;
                    let first = j.args.first().cloned().unwrap_or_default();
                    if code.is_none() {
                        self.aurelia_ok = false;
                        self.msg = "No se encontró `aurelia`. Instálalo con: yay -S aurelia".into();
                    }
                    match tag {
                        Tag::LibAll => {
                            if let Some(v) = &self.jobs[job].result {
                                let g = extract_games(v);
                                if !g.is_empty() {
                                    self.games = g;
                                }
                            }
                            if code != Some(0) && code.is_some() && self.games.is_empty() {
                                self.msg = "No pude leer la biblioteca: inicia sesión en Sistema → Iniciar sesión".into();
                            }
                        }
                        Tag::LibInst => {
                            if let Some(v) = &self.jobs[job].result {
                                let g = extract_games(v);
                                self.inst = g.iter().map(|x| x.id.clone()).collect();
                                if self.games.is_empty() {
                                    self.games = g;
                                }
                            }
                        }
                        Tag::Normal => {
                            let ok = code == Some(0);
                            self.msg = format!("{} {}", if ok { "✔" } else { "✘" }, self.jobs[job].title);
                            if ok && ["install", "uninstall", "update", "move", "verify"].contains(&first.as_str()) {
                                self.reload_installed();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn after_interactive(&mut self) {
        self.reload_library();
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
                self.overlay = Overlay::Launch(Launch::new(g.id.clone(), g.name.clone()));
            }
            return;
        }
        if let Some(label) = act.ask {
            let pre = match (&app, label.starts_with("AppID")) {
                (Some(id), true) => {
                    if label.trim() == "AppID" { id.clone() } else { format!("{id} ") }
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
            Mode::Job | Mode::Interactive => {
                let args = build_args(act.args, app.as_deref(), &input);
                self.run_args(args);
            }
            Mode::Launch => {}
            Mode::Patch => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(dst) => self.start_patch(input.trim().to_string(), dst),
                    None => self.prompt("Carpeta de instalación del juego", String::new(), PKind::PatchDst(input.trim().to_string())),
                }
            }
            Mode::Revert => {
                let Some(id) = app else { return };
                match self.game_dir(&id) {
                    Some(dst) => self.start_revert(dst),
                    None => self.prompt("Carpeta de instalación del juego", String::new(), PKind::RevertDst),
                }
            }
        }
    }

    fn start_patch(&mut self, src: String, dst: String) {
        let id = self.new_job(format!("parche {src} → {dst}"), vec![], Tag::Normal);
        spawn_local(id, self.tx.clone(), move |log| patch::apply(std::path::Path::new(&src), std::path::Path::new(&dst), log));
    }

    fn start_revert(&mut self, dst: String) {
        let id = self.new_job(format!("revertir parche en {dst}"), vec![], Tag::Normal);
        spawn_local(id, self.tx.clone(), move |log| patch::revert(std::path::Path::new(&dst), log));
    }

    // ---------- teclado ----------
pub fn on_mouse(&mut self, m: ratatui::crossterm::event::MouseEvent, size: (u16, u16)) {
    use ratatui::crossterm::event::MouseEventKind;
    let (_w, h) = size;
    match m.kind {
        MouseEventKind::ScrollDown => {
            self.move_by(3);
        }
        MouseEventKind::ScrollUp => {
            self.move_by(-3);
        }
        MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left) => {
            let is_menu = matches!(self.overlay, Overlay::Menu(_));
            let is_theme = matches!(self.overlay, Overlay::Theme { .. });
            let is_help = matches!(self.overlay, Overlay::Help);
            if is_help || is_menu || is_theme {
                // Click anywhere closes the overlay (use j/k + Enter to select)
                self.overlay = Overlay::None;
                return;
            }
            // Tab bar (row 3): switch section by column
            if m.row == 3 {
                let tab_width = _w / 8;
                if tab_width > 0 {
                    let idx = (m.column / tab_width).min(7) as usize;
                    self.goto(idx);
                }
                return;
            }
            // Library (screen 1): main content area
            if self.screen == 1 {
                let row = m.row as usize;
                if row >= 7 && row < h as usize - 2 {
                    let idx = row - 7;
                    let filtered = self.filtered();
                    if idx < filtered.len() {
                        self.cursor[1] = idx;
                    }
                }
                return;
            }
            // Action lists (screens 2-7): select by row
            if self.screen >= 2 {
                let row = m.row as usize;
                if row >= 7 && row < h as usize - 2 {
                    let idx = row - 7;
                    let acts = acts_for(self.screen);
                    if idx < acts.len() {
                        self.cursor[self.screen] = idx;
                    }
                }
            }
        }
        MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Right) => {
            self.overlay = Overlay::None;
        }
        _ => {}
    }
}
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
            KeyCode::Enter => {
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
            KeyCode::Char('r') | KeyCode::Enter => {
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
            KeyCode::Enter => {
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
        self.focus_out = s >= 2; // auto-focus content on menu screens
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
                KeyCode::Char('t') => {
                    let next = (self.sidebar_cursor + 1) % 8;
                    self.goto(next);
                    return;
                }
                KeyCode::Char('T') => {
                    self.goto((self.sidebar_cursor + 7) % 8);
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
                KeyCode::Char('q') => self.quit = true,
                _ => {}
            }
            return;
        }

        // ---- teclas globales ----
        match k.code {
            KeyCode::Char(' ') => { self.leader = true; return; }
            KeyCode::Char('T') => { self.overlay = Overlay::Theme { cursor: self.theme, orig: self.theme }; return; }
            KeyCode::Char('?') => { self.overlay = Overlay::Help; return; }
            KeyCode::Char(':') => { self.prompt("aurelia", String::new(), PKind::Palette); return; }
            KeyCode::Char('g') => { self.g_pending = true; return; }
            KeyCode::Char(c @ '1'..='8') => { self.goto(c as usize - '1' as usize); return; }
            KeyCode::Char('q') => { self.quit = true; return; }
            KeyCode::Char('v') => { self.raw = !self.raw; return; }
            KeyCode::Char('[') => { self.cycle_job(-1); return; }
            KeyCode::Char(']') => { self.cycle_job(1); return; }
            KeyCode::Tab => { self.goto((self.screen + 1) % 8); return; }
            KeyCode::BackTab => { self.goto((self.screen + 7) % 8); return; }
            _ => {}
        }

        // ---- barra lateral enfocada (hjkl para moverse) ----
        if !self.focus_out {
            match k.code {
                KeyCode::Char('j') | KeyCode::Down => self.sidebar_cursor = (self.sidebar_cursor + 1) % 8,
                KeyCode::Char('k') | KeyCode::Up => self.sidebar_cursor = (self.sidebar_cursor + 7) % 8,
                KeyCode::Char('g') => self.sidebar_cursor = 0,
                KeyCode::Char('G') => self.sidebar_cursor = 7,
                KeyCode::Enter => self.goto(self.sidebar_cursor),
                KeyCode::Char('l') | KeyCode::Right => {
                    self.goto(self.sidebar_cursor);
                    self.focus_out = true;
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
            KeyCode::Char('l') | KeyCode::Right => self.focus_out = self.cur.is_some() || self.screen == 0,
            KeyCode::Char('v') => self.raw = !self.raw,
            KeyCode::Char('n') => self.search_next(1),
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
            KeyCode::Char('p') if self.screen == 1 => self.trigger(&GAME_ACTS[0]),
            KeyCode::Char('o') if self.screen == 1 => self.trigger(&GAME_ACTS[1]),
            KeyCode::Char('I') if self.screen == 1 => self.trigger(&GAME_ACTS[2]),
            KeyCode::Char('u') if self.screen == 1 => self.trigger(&GAME_ACTS[3]),
            KeyCode::Char('K') if self.screen == 1 => self.trigger(&GAME_ACTS[7]),
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

fn fmt_progress(v: &Value) -> String {
    let mut s = String::new();
    if let Some(b) = v.get("bytes_downloaded").and_then(|x| x.as_f64()) {
        s += &format!("{:.1} MB", b / 1_048_576.0);
    }
    if let Some(b) = v.get("speed_bps").and_then(|x| x.as_f64()) {
        s += &format!(" · {:.1} MB/s", b / 1_048_576.0);
    }
    if let Some(e) = v.get("eta_seconds").and_then(|x| x.as_f64()) {
        s += &format!(" · ETA {}s", e as u64);
    }
    if s.is_empty() {
        s = v.to_string();
    }
    s
}
