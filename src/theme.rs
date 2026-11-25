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
    pub seltext: Color,
}

use Color::Rgb as C;

pub const THEMES: &[Theme] = &[
    // Retro82: paleta retro estilo viejo TUI
    Theme { name: "Retro82", bg: C(20, 15, 30), fg: C(255, 210, 150), dim: C(180, 150, 110), accent: C(255, 160, 100), accent2: C(255, 200, 150), ok: C(180, 220, 150), warn: C(255, 210, 80), err: C(255, 90, 70), sel: C(80, 50, 30), seltext: C(255, 160, 100), },
    // Terminal: usa la paleta ANSI de tu terminal (respeta tus colores)
    Theme { name: "Terminal", bg: Color::Reset, fg: Color::Reset, dim: Color::White, accent: Color::Cyan, accent2: Color::Magenta, ok: Color::Green, warn: Color::Yellow, err: Color::Red, sel: Color::DarkGray, seltext: Color::Cyan, },
    // Sistema: lee ~/.config/theme-engine/colors.css/.rasi y construye la paleta
    Theme { name: "Sistema", bg: C(30, 30, 35), fg: C(230, 230, 235), dim: C(185, 185, 195), accent: C(120, 190, 240), accent2: C(160, 210, 255), ok: C(120, 210, 150), warn: C(255, 190, 80), err: C(255, 100, 80), sel: C(80, 130, 200), seltext: C(120, 190, 240), },
    // Everforest: verde oscuro y cálido
    Theme { name: "Everforest", bg: C(45, 53, 59), fg: C(230, 218, 190), dim: C(180, 180, 160), accent: C(185, 206, 150), accent2: C(180, 216, 165), ok: C(185, 206, 150), warn: C(235, 198, 128), err: C(240, 133, 136), sel: C(70, 82, 89), seltext: C(185, 206, 150), },
    // Miasma: oscuro y apagado pero texto visible
    Theme { name: "Miasma", bg: C(34, 34, 34), fg: C(220, 220, 200), dim: C(170, 170, 150), accent: C(190, 190, 170), accent2: C(200, 200, 180), ok: C(170, 190, 160), warn: C(220, 190, 110), err: C(200, 130, 120), sel: C(45, 45, 45), seltext: C(190, 190, 170), },
    // Kanagawa: azul oscuro estilo pintura japonesa
    Theme { name: "Kanagawa", bg: C(31, 31, 40), fg: C(235, 232, 210), dim: C(170, 172, 155), accent: C(150, 184, 232), accent2: C(175, 155, 205), ok: C(180, 210, 140), warn: C(232, 204, 155), err: C(220, 100, 100), sel: C(50, 70, 95), seltext: C(150, 184, 232), },
    // Material: oscuro moderno
    Theme { name: "Material", bg: C(38, 50, 56), fg: C(248, 255, 255), dim: C(140, 155, 170), accent: C(150, 190, 255), accent2: C(205, 150, 240), ok: C(220, 248, 195), warn: C(255, 215, 130), err: C(248, 130, 145), sel: C(55, 75, 90), seltext: C(150, 190, 255), },
    // Synthwave84: neón años 80
    Theme { name: "Synthwave84", bg: C(20, 10, 40), fg: C(255, 230, 255), dim: C(170, 140, 210), accent: C(255, 80, 200), accent2: C(80, 220, 255), ok: C(120, 255, 190), warn: C(255, 220, 90), err: C(255, 90, 110), sel: C(55, 25, 90), seltext: C(255, 80, 200), },
    // Osaka Jade: jade oscuro
    Theme { name: "Osaka Jade", bg: C(17, 28, 24), fg: C(215, 220, 175), dim: C(155, 175, 150), accent: C(100, 165, 135), accent2: C(100, 215, 200), ok: C(100, 165, 135), warn: C(255, 195, 90), err: C(255, 110, 80), sel: C(30, 58, 47), seltext: C(100, 165, 135), },
    // Ambar: cálido estilo aurora
    Theme { name: "Ambar", bg: C(20, 11, 2), fg: C(255, 210, 120), dim: C(185, 145, 55), accent: C(255, 170, 40), accent2: C(255, 220, 140), ok: C(190, 230, 90), warn: C(255, 210, 60), err: C(255, 100, 70), sel: C(70, 38, 6), seltext: C(255, 170, 40), },
    // Gruvbox: oscuro con acento azul pastel
    Theme { name: "Gruvbox", bg: C(40, 40, 40), fg: C(238, 221, 178), dim: C(180, 160, 140), accent: C(131, 165, 152), accent2: C(142, 192, 124), ok: C(184, 187, 38), warn: C(250, 189, 47), err: C(251, 73, 52), sel: C(60, 56, 54), seltext: C(131, 165, 152), },
    // Nord: fresco y limpio
    Theme { name: "Nord", bg: C(46, 52, 64), fg: C(228, 232, 242), dim: C(170, 180, 190), accent: C(136, 192, 208), accent2: C(180, 142, 173), ok: C(163, 190, 140), warn: C(235, 203, 139), err: C(191, 97, 106), sel: C(67, 76, 94), seltext: C(136, 192, 208), },
    // Dracula: oscuro y con personalidad
    Theme { name: "Dracula", bg: C(40, 42, 54), fg: C(253, 253, 253), dim: C(160, 170, 180), accent: C(189, 147, 249), accent2: C(255, 121, 198), ok: C(248, 252, 200), warn: C(251, 230, 147), err: C(255, 107, 114), sel: C(70, 72, 93), seltext: C(189, 147, 249), },
    // TokyoNight: oscuro suave para ojos
    Theme { name: "TokyoNight", bg: C(26, 27, 38), fg: C(220, 222, 233), dim: C(155, 165, 185), accent: C(140, 160, 200), accent2: C(180, 190, 220), ok: C(120, 190, 210), warn: C(255, 180, 100), err: C(255, 140, 150), sel: C(80, 90, 120), seltext: C(140, 160, 200), },
    // Catppuccin Mocha: oscuro moderno
    Theme { name: "Catppuccin Mocha", bg: C(30, 30, 35), fg: C(225, 220, 205), dim: C(150, 150, 160), accent: C(180, 160, 225), accent2: C(210, 180, 245), ok: C(180, 210, 175), warn: C(240, 205, 140), err: C(255, 130, 140), sel: C(110, 95, 160), seltext: C(180, 160, 225), },
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
            if let Some(st) = colors.get("selectedtext") { theme.seltext = *st; }
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
            if let Some(st) = colors.get("selectedtext") { theme.seltext = *st; }
        }
    }
    theme
}