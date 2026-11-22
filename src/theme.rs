use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub accent2: Color,
    pub ok: Color,
    pub warn: Color,
    pub err: Color,
    pub sel: Color,
}

use Color::Rgb as C;

pub const THEMES: &[Theme] = &[
    // Retro82: paleta retro estilo viejo TUI
    Theme { name: "Retro82", bg: C(20, 15, 30), fg: C(220, 180, 130), dim: C(100, 80, 60), accent: C(255, 160, 100), accent2: C(255, 200, 150), ok: C(180, 220, 150), warn: C(255, 210, 80), err: C(255, 90, 70), sel: C(80, 50, 30) },
    // Terminal: usa la paleta ANSI de tu terminal (respeta tus colores)
    Theme { name: "Terminal", bg: Color::Reset, fg: Color::Reset, dim: Color::DarkGray, accent: Color::Cyan, accent2: Color::Magenta, ok: Color::Green, warn: Color::Yellow, err: Color::Red, sel: Color::DarkGray },
    // Sistema: lee ~/.config/theme-engine/colors.css/.rasi y construye la paleta
    Theme { name: "Sistema", bg: C(30, 30, 35), fg: C(200, 200, 200), dim: C(120, 120, 130), accent: C(100, 180, 220), accent2: C(140, 200, 240), ok: C(80, 200, 130), warn: C(255, 160, 50), err: C(255, 80, 50), sel: C(60, 120, 180) },
    // Everforest: verde oscuro y cálido
    Theme { name: "Everforest", bg: C(45, 53, 59), fg: C(211, 198, 170), dim: C(133, 146, 137), accent: C(167, 192, 128), accent2: C(131, 192, 146), ok: C(167, 192, 128), warn: C(219, 188, 127), err: C(230, 126, 128), sel: C(61, 72, 77) },
    // Miasma: oscuro y apagado
    Theme { name: "Miasma", bg: C(34, 34, 34), fg: C(194, 194, 176), dim: C(107, 107, 90), accent: C(153, 153, 133), accent2: C(176, 176, 144), ok: C(143, 170, 127), warn: C(201, 165, 84), err: C(187, 102, 85), sel: C(51, 51, 51) },
    // Kanagawa: azul oscuro estilo pintura japonesa
    Theme { name: "Kanagawa", bg: C(31, 31, 40), fg: C(220, 215, 186), dim: C(114, 113, 105), accent: C(126, 156, 216), accent2: C(149, 127, 184), ok: C(152, 187, 108), warn: C(230, 195, 132), err: C(195, 64, 67), sel: C(45, 79, 103) },
    // Material: oscuro moderno
    Theme { name: "Material", bg: C(38, 50, 56), fg: C(238, 255, 255), dim: C(84, 110, 122), accent: C(130, 170, 255), accent2: C(199, 146, 234), ok: C(195, 232, 141), warn: C(255, 203, 107), err: C(240, 113, 120), sel: C(55, 71, 79) },
    // Synthwave84: neón años 80
    Theme { name: "Synthwave84", bg: C(20, 10, 40), fg: C(235, 220, 255), dim: C(110, 90, 160), accent: C(255, 80, 200), accent2: C(80, 220, 255), ok: C(120, 255, 190), warn: C(255, 220, 90), err: C(255, 90, 110), sel: C(55, 25, 90) },
    // Osaka Jade: jade oscuro
    Theme { name: "Osaka Jade", bg: C(17, 28, 24), fg: C(193, 196, 151), dim: C(83, 104, 91), accent: C(80, 148, 117), accent2: C(45, 213, 183), ok: C(80, 148, 117), warn: C(230, 182, 85), err: C(255, 83, 69), sel: C(30, 58, 47) },
    // Ambar: cálido estilo aurora
    Theme { name: "Ambar", bg: C(20, 11, 2), fg: C(255, 190, 90), dim: C(130, 85, 30), accent: C(255, 170, 40), accent2: C(255, 220, 140), ok: C(190, 230, 90), warn: C(255, 210, 60), err: C(255, 100, 70), sel: C(70, 38, 6) },
    // Gruvbox: oscuro con acento azul pastel
    Theme { name: "Gruvbox", bg: C(40, 40, 40), fg: C(235, 219, 178), dim: C(146, 131, 116), accent: C(131, 165, 152), accent2: C(142, 192, 124), ok: C(184, 187, 38), warn: C(250, 189, 47), err: C(251, 73, 52), sel: C(60, 56, 54) },
    // Nord: fresco y limpio
    Theme { name: "Nord", bg: C(46, 52, 64), fg: C(216, 222, 233), dim: C(100, 112, 134), accent: C(136, 192, 208), accent2: C(180, 142, 173), ok: C(163, 190, 140), warn: C(235, 203, 139), err: C(191, 97, 106), sel: C(67, 76, 94) },
    // Dracula: oscuro y con personalidad
    Theme { name: "Dracula", bg: C(40, 42, 54), fg: C(248, 248, 242), dim: C(98, 114, 164), accent: C(189, 147, 249), accent2: C(255, 121, 198), ok: C(80, 250, 123), warn: C(241, 250, 140), err: C(255, 85, 85), sel: C(68, 71, 90) },
    // TokyoNight: oscuro suave para ojos
    Theme { name: "TokyoNight", bg: C(20, 18, 28), fg: C(180, 190, 210), dim: C(60, 70, 80), accent: C(140, 160, 200), accent2: C(180, 190, 220), ok: C(100, 180, 200), warn: C(255, 180, 100), err: C(255, 100, 80), sel: C(80, 100, 140) },
    // Catppuccin Mocha: oscuro moderno
    Theme { name: "Catppuccin Mocha", bg: C(30, 30, 35), fg: C(210, 205, 195), dim: C(80, 80, 90), accent: C(170, 150, 220), accent2: C(200, 180, 240), ok: C(130, 200, 160), warn: C(230, 180, 100), err: C(255, 100, 80), sel: C(100, 80, 160) },
];

pub fn index_of(name: &str) -> usize {
    THEMES.iter().position(|t| t.name.eq_ignore_ascii_case(name)).unwrap_or(1)
}

/// Carga el tema "Sistema" desde ~/.config/theme-engine/colors.css o colors.rasi
pub fn load_system_theme() -> Theme {
    let mut theme = THEMES.iter().find(|t| t.name == "Sistema").copied().unwrap_or(THEMES[0]);
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("~/.config")).join("theme-engine");
    let css_path = base.join("colors.css");
    let rasi_path = base.join("colors.rasi");
    
    fn parse_color_hex(s: &str) -> Option<Color> {
        let s = s.trim().trim_start_matches('#');
        if s.len() == 6 || s.len() == 8 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        } else {
            None
        }
    }

    // Try CSS first (from matugen/tinty via color-engine.sh)
    if css_path.exists() {
        if let Ok(content) = std::fs::read_to_string(css_path) {
            let mut colors = std::collections::HashMap::new();
            for line in content.lines() {
                if line.trim_start().starts_with("@define-color") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let name = parts[1];
                        let hex = parts[2].trim_end_matches(';');
                        if let Some(c) = parse_color_hex(hex) {
                            colors.insert(name, c);
                        }
                    }
                }
            }
            if let Some(bg) = colors.get("background") { theme.bg = *bg; }
            if let Some(fg) = colors.get("foreground") { theme.fg = *fg; }
            if let Some(acc) = colors.get("selected") { theme.accent = *acc; }
            if let Some(acc2) = colors.get("active") { theme.accent2 = *acc2; }
            if let Some(ok) = colors.get("color10") { theme.ok = *ok; }
            else if let Some(ok) = colors.get("color2") { theme.ok = *ok; }
            if let Some(warn) = colors.get("color11") { theme.warn = *warn; }
            else if let Some(warn) = colors.get("color3") { theme.warn = *warn; }
            if let Some(err) = colors.get("color9") { theme.err = *err; }
            else if let Some(err) = colors.get("color1") { theme.err = *err; }
            if let Some(dim) = colors.get("color8") { theme.dim = *dim; }
            if let Some(sel) = colors.get("selected") { theme.sel = *sel; }
        }
    }
    // Fallback to RASI
    else if rasi_path.exists() {
        if let Ok(content) = std::fs::read_to_string(rasi_path) {
            let mut colors = std::collections::HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("color") && line.contains(':') {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        let name = parts[0].trim();
                        let hex = parts[1].trim().trim_end_matches(';');
                        if let Some(c) = parse_color_hex(hex) {
                            colors.insert(name, c);
                        }
                    }
                } else if line.starts_with("background:") || line.starts_with("foreground:") || 
                          line.starts_with("selected:") || line.starts_with("active:") ||
                          line.starts_with("urgent:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim_end_matches(':');
                        let hex = parts[1].trim().trim_end_matches(';');
                        if let Some(c) = parse_color_hex(hex) {
                            colors.insert(name, c);
                        }
                    }
                }
            }
            if let Some(bg) = colors.get("background") { theme.bg = *bg; }
            if let Some(fg) = colors.get("foreground") { theme.fg = *fg; }
            if let Some(acc) = colors.get("selected") { theme.accent = *acc; }
            if let Some(acc2) = colors.get("active") { theme.accent2 = *acc2; }
            if let Some(ok) = colors.get("color10") { theme.ok = *ok; }
            else if let Some(ok) = colors.get("color2") { theme.ok = *ok; }
            if let Some(warn) = colors.get("color11") { theme.warn = *warn; }
            else if let Some(warn) = colors.get("color3") { theme.warn = *warn; }
            if let Some(err) = colors.get("color9") { theme.err = *err; }
            else if let Some(err) = colors.get("color1") { theme.err = *err; }
            if let Some(dim) = colors.get("color8") { theme.dim = *dim; }
            if let Some(sel) = colors.get("selected") { theme.sel = *sel; }
        }
    }
    theme
}