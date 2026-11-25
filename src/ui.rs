use crate::{
    actions::*,
    app::*,
    backend::flatten,
    theme::{Theme, THEMES},
};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;

const SPIN: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

fn centered(a: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(a.width);
    let h = h.min(a.height);
    Rect { x: a.x + (a.width - w) / 2, y: a.y + (a.height - h) / 2, width: w, height: h }
}

fn fill_bg(f: &mut Frame, r: Rect, t: &Theme) {
    f.render_widget(Block::new().style(Style::new().bg(t.bg).fg(t.fg)), r);
}

fn blk(title: &str, t: &Theme, focus: bool) -> Block<'static> {
    let c = if focus { t.accent } else { t.dim };
    Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(c))
        .title(Line::from(vec![
            Span::styled("╡ ", Style::new().fg(c)),
            Span::styled(title.to_string(), Style::new().fg(t.accent2).bold()),
            Span::styled(" ╞", Style::new().fg(c)),
        ]))
        .style(Style::new().bg(t.bg).fg(t.fg))
}

fn sel_style(t: &Theme) -> Style {
    Style::new().bg(t.sel).fg(t.seltext).add_modifier(Modifier::BOLD)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let t = if app.theme < THEMES.len() && THEMES[app.theme].name == "Sistema" {
        crate::theme::load_system_theme()
    } else {
        THEMES[app.theme]
    };
    let area = f.area();
    f.render_widget(Block::new().style(Style::new().bg(t.bg).fg(t.fg)), area);
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(5), Constraint::Length(2)]).split(area);
    // Tabs only in the top row
    tabs(f, rows[0], app, &t);
    let main = if app.show_jobs {
        let v = Layout::vertical([Constraint::Percentage(62), Constraint::Percentage(38)]).split(rows[1]);
        jobs_panel(f, v[1], app, &t);
        v[0]
    } else {
        rows[1]
    };
    match app.screen {
        0 => home(f, main, app, &t),
        1 => library(f, main, app, &t),
        _ => menu_screen(f, main, app, &t),
    }
    footer(f, rows[2], app, &t);
    overlays(f, rows[1], app, &t);
}

fn tabs(f: &mut Frame, a: Rect, app: &App, t: &Theme) {
    let mut spans = vec![Span::raw(" ")];
    for (i, (ic, name)) in SCREENS.iter().enumerate() {
        let sel = i == app.sidebar_cursor;
        let active = i == app.screen;
        let style = if sel {
            Style::new().bg(t.accent).fg(Color::Black).bold()
        } else if active {
            Style::new().fg(t.accent).bold()
        } else {
            Style::new().fg(t.fg)
        };
        spans.push(Span::styled(format!(" {} {} ", ic, name), style));
        if i < SCREENS.len() - 1 {
            spans.push(Span::styled("│", Style::new().fg(t.fg)));
        }
    }
    f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center).block(blk("MENÚ", t, !app.focus_out)), a);
}

fn kv_lines(v: &Value, t: &Theme) -> Vec<Line<'static>> {
    let mut fl = vec![];
    flatten(v, 0, &mut fl);
    fl.into_iter()
        .map(|(ind, k, val)| {
            let mut sp = vec![Span::raw("  ".repeat(ind))];
            sp.push(Span::styled(k.clone(), Style::new().fg(if k == "•" { t.accent2 } else { t.accent })));
            if !val.is_empty() {
                sp.push(Span::styled(if k.is_empty() { val } else if k == "•" { format!(" {val}") } else { format!(": {val}") }, Style::new().fg(t.fg)));
            }
            Line::from(sp)
        })
        .collect()
}

fn draw_output(f: &mut Frame, a: Rect, app: &mut App, t: &Theme, focus: bool) {
    let mut lines: Vec<Line> = vec![];
    let mut title = "Resultado".to_string();
    match app.cur.and_then(|i| app.jobs.get(i)) {
        None => lines.push(Line::styled("Elige una acción y pulsa Enter. Aquí verás el resultado.", Style::new().fg(t.dim))),
        Some(j) => {
            let icon = if !j.done { SPIN[app.tick / 2 % 8] } else if j.code == Some(0) { "✓" } else { "✗" };
            title = format!("{icon} {}", j.title);
            if !j.done && !j.progress.is_empty() {
                lines.push(Line::styled(format!("▶ {}", j.progress), Style::new().fg(t.ok).bold()));
            }
            match (&j.result, app.raw) {
                (Some(v), false) if j.done => {
                    lines.extend(kv_lines(v, t));
                    for (e, l) in j.lines.iter().filter(|(e, _)| *e) {
                        let _ = e;
                        lines.push(Line::styled(l.clone(), Style::new().fg(t.err)));
                    }
                }
                _ => {
                    for (e, l) in j.lines.iter().rev().take(2000).collect::<Vec<_>>().into_iter().rev() {
                        lines.push(Line::styled(l.clone(), Style::new().fg(if *e { t.warn } else { t.fg })));
                    }
                }
            }
            if j.done && lines.is_empty() {
                lines.push(Line::styled(format!("(sin salida) código {:?}", j.code), Style::new().fg(t.dim)));
            }
        }
    }
    let h = a.height.saturating_sub(2);
    let max = lines.len().saturating_sub(h as usize) as u16;
    // los trabajos en curso siguen la cola
    let running = app.cur.and_then(|i| app.jobs.get(i)).map(|j| !j.done).unwrap_or(false);
    if running && app.scroll == 0 {
        app.scroll = max;
    }
    app.scroll = app.scroll.min(max);
    let p = Paragraph::new(lines).wrap(Wrap { trim: false }).scroll((app.scroll, 0)).block(blk(&title, t, focus));
    f.render_widget(p, a);
}

fn home(f: &mut Frame, a: Rect, app: &App, t: &Theme) {
    let mut lines: Vec<Line> = vec![Line::raw("")];
    for (idx, l) in app.logo.iter().enumerate() {
        let display = if idx == 0 {
            l.clone() // logo tal como está en el archivo
        } else {
            l.clone()
        };
        lines.push(Line::styled(display, Style::new().fg(t.accent).bold()));
    }
    lines.push(Line::styled("Aurelia, con una interfaz que no necesita manual".to_string(), Style::new().fg(t.accent2)));
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", app.games.len()), Style::new().fg(t.accent).bold()),
        Span::raw("juegos   "),
        Span::styled(format!("{} ", app.inst.len()), Style::new().fg(t.ok).bold()),
        Span::raw("instalados   "),
        Span::styled(format!("{} ", app.running()), Style::new().fg(t.warn).bold()),
        Span::raw("trabajos en curso"),
    ]));
    lines.push(Line::raw(""));
    for (k, d) in [
        ("Tab", "cambiar de sección"),
        ("j k", "navegar barra lateral · contenido: mover"),
        ("Enter o n", "seleccionar sección"),
        ("l", "entrar al contenido"),
        ("h", "volver a la barra lateral"),
        ("gg G", "inicio / fin de la barra"),
        ("g1..g8", "ir a la sección N"),
        ("Space", "tecla líder (which-key)"),
        (":", "ejecutar cualquier comando de aurelia"),
        ("T", "cambiar tema (incluye «Sistema»)"),
        ("?", "ayuda"),
    ] {
        lines.push(Line::from(vec![Span::styled(format!("{k:>11}  "), Style::new().fg(t.accent)), Span::styled(d, Style::new().fg(t.fg))]));
    }
    if !app.aurelia_ok {
        lines.push(Line::raw(""));
        lines.push(Line::styled("⚠ No encuentro `aurelia`. Instálalo: yay -S aurelia", Style::new().fg(t.err).bold()));
    }
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center).block(blk("INICIO", t, true)), a);
}

fn library(f: &mut Frame, a: Rect, app: &mut App, t: &Theme) {
    let cols = Layout::horizontal([Constraint::Percentage(55), Constraint::Min(20)]).split(a);
    let idx = app.filtered();
    let cur = app.cursor[1].min(idx.len().saturating_sub(1));
    app.cursor[1] = cur;
    let rows: Vec<Row> = idx
        .iter()
        .map(|&i| {
            let g = &app.games[i];
            let inst = app.is_inst(g);
            Row::new(vec![
                Cell::from(Span::styled(if inst { "●" } else { "○" }, Style::new().fg(if inst { t.ok } else { t.dim }))),
                Cell::from(g.name.clone()),
                Cell::from(Span::styled(g.id.clone(), Style::new().fg(t.dim))),
            ])
        })
        .collect();
    let mut title = format!("BIBLIOTECA {}/{}", idx.len(), app.games.len());
    if app.only_inst {
        title += " · instalados";
    }
    if !app.filter.is_empty() {
        title += &format!(" · /{}", app.filter);
    }
    app.lib_state.select(if idx.is_empty() { None } else { Some(cur) });
    let table = Table::new(rows, [Constraint::Length(2), Constraint::Min(10), Constraint::Length(9)])
        .block(blk(&title, t, !app.focus_out))
        .row_highlight_style(sel_style(t))
        .highlight_symbol("▸");
    f.render_stateful_widget(table, cols[0], &mut app.lib_state);
    if idx.is_empty() {
        let inner = cols[0].inner(Margin::new(2, 2));
        f.render_widget(Paragraph::new("Sin juegos.\nInicia sesión en Sistema → Iniciar sesión\no recarga con r.").style(Style::new().fg(t.dim)), inner);
    }

    if app.show_job_out && app.cur.is_some() {
        draw_output(f, cols[1], app, t, app.focus_out);
    } else if let Some(g) = app.selected_game() {
        let mut lines = vec![];
        lines.push(Line::from(vec![Span::styled(g.name.clone(), Style::new().fg(t.accent).bold()), Span::styled(format!("  {}", g.id), Style::new().fg(t.dim))]));
        lines.push(Line::raw(""));
        let raw = &g.raw;
        let kv = |key: &str, val: String| Line::from(vec![Span::styled(format!("{:<20}", key), Style::new().fg(t.accent2).bold()), Span::styled(val, Style::new().fg(t.fg))]);
        if let Some(v) = raw.get("is_installed").and_then(|x| x.as_bool()) {
            lines.push(kv("Instalado", if v { "● sí" } else { "○ no" }.to_string()));
        }
        if let Some(v) = raw.get("platform").and_then(|x| x.as_str()) {
            lines.push(kv("Plataforma", v.to_string()));
        }
        if let Some(v) = raw.get("playtime_forever_minutes").and_then(|x| x.as_u64()) {
            lines.push(kv("Tiempo de juego", format!("{} min", v)));
        }
        if let Some(v) = raw.get("update_available").and_then(|x| x.as_bool()) {
            lines.push(kv("Actualización", if v { "disponible" } else { "al día" }.to_string()));
        }
        if let Some(v) = raw.get("install_path").and_then(|x| x.as_str()) {
            if !v.is_empty() {
                lines.push(kv("Ruta", v.to_string()));
            }
        }
        if let Some(v) = raw.get("active_branch").and_then(|x| x.as_str()) {
            lines.push(kv("Rama", v.to_string()));
        }
        if let Some(v) = raw.get("assets").and_then(|x| x.get("capsule")).and_then(|x| x.as_str()) {
            lines.push(kv("Portada", v.to_string()));
        }
        lines.push(Line::raw(""));
        lines.push(Line::styled("Enter: acciones · p jugar · o opciones · I instalar · u actualizar · K info", Style::new().fg(t.dim)));
        let h = cols[1].height.saturating_sub(2);
        let max = lines.len().saturating_sub(h as usize) as u16;
        app.scroll = app.scroll.min(max);
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).scroll((app.scroll, 0)).block(blk("DETALLE", t, app.focus_out)), cols[1]);
    } else {
        f.render_widget(blk("DETALLE", t, false), cols[1]);
    }
}

fn menu_screen(f: &mut Frame, a: Rect, app: &mut App, t: &Theme) {
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Min(20)]).split(a);
    let acts = acts_for(app.screen);
    let items: Vec<ListItem> = acts
        .iter()
        .map(|x| {
            let mut s = x.label.to_string();
            if x.ask.is_some() {
                s.push('…');
            }
            if x.mode == Mode::Interactive {
                s.push_str(" ↗");
            }
            ListItem::new(s)
        })
        .collect();
    app.list_state.select(Some(app.cursor[app.screen].min(acts.len().saturating_sub(1))));
    let list = List::new(items).block(blk(SCREENS[app.screen].1, t, !app.focus_out)).highlight_style(sel_style(t)).highlight_symbol("▸");
    f.render_stateful_widget(list, cols[0], &mut app.list_state);
    draw_output(f, cols[1], app, t, app.focus_out);
}

fn jobs_panel(f: &mut Frame, a: Rect, app: &App, t: &Theme) {
    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, j)| j.tag == Tag::Normal)
        .map(|(i, j)| {
            let (icon, c) = if !j.done { (SPIN[app.tick / 2 % 8], t.warn) } else if j.code == Some(0) { ("✓", t.ok) } else { ("✗", t.err) };
            let mark = if app.cur == Some(i) { "▸" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(mark, Style::new().fg(t.accent)),
                Span::styled(format!("{icon} "), Style::new().fg(c)),
                Span::raw(j.title.clone()),
                Span::styled(if j.progress.is_empty() { String::new() } else { format!("   {}", j.progress) }, Style::new().fg(t.dim)),
            ]))
        })
        .collect();
    f.render_widget(List::new(items).block(blk("TRABAJOS  ([ ] para ver uno)", t, false)), a);
}

fn footer(f: &mut Frame, a: Rect, app: &App, t: &Theme) {
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(a);
    let (mode, mode_bg) = match &app.overlay {
        Overlay::Prompt(_) => ("INSERT", t.ok),
        Overlay::None if app.leader => ("LEADER", t.warn),
        Overlay::None if app.g_pending => ("g", t.accent2),
        Overlay::None => ("NORMAL", t.accent),
        _ => ("MENU", t.accent2),
    };
    let tip = if app.msg.is_empty() {
        match app.screen {
            1 => app.selected_game().map(|g| g.name.clone()).unwrap_or_default(),
            s => acts_for(s).get(app.cursor[s]).map(|x| x.tip.to_string()).unwrap_or_default(),
        }
    } else {
        app.msg.clone()
    };
    let pos = if app.screen == 1 {
        let n = app.filtered().len();
        if n == 0 { "0/0".to_string() } else { format!("{}/{}", app.cursor[1] + 1, n) }
    } else if app.screen == 0 {
        String::new()
    } else {
        let n = acts_for(app.screen).len();
        if n == 0 { "0/0".to_string() } else { format!("{}/{}", app.cursor[app.screen] + 1, n) }
    };
    let mut left = vec![
        Span::styled(format!(" {mode} "), Style::new().bg(mode_bg).fg(Color::Black).bold()),
        Span::styled(format!(" {} ", SCREENS[app.screen].1), Style::new().bg(t.sel).fg(t.seltext)),
    ];
    if let Some(c) = app.count {
        left.push(Span::styled(format!(" {c}"), Style::new().fg(t.warn).bold()));
    }
    if !pos.is_empty() {
        left.push(Span::styled(format!(" {pos} "), Style::new().fg(t.dim)));
    }
    left.push(Span::styled(format!(" {tip}"), Style::new().fg(t.fg)));
    let n = app.running();
    if n > 0 {
        left.push(Span::styled(format!("  {} {n} en curso", SPIN[app.tick / 2 % 8]), Style::new().fg(t.warn)));
    }
    f.render_widget(Paragraph::new(Line::from(left)).style(Style::new().bg(t.bg)), rows[0]);
    let hints = match app.screen {
        1 => "j/k mover  h/l foco  Enter o n seleccionar  Tab siguiente  / filtrar  p jugar  o opciones  ? ayuda",
        0 => "j/k sidebar  Enter o n seleccionar  l entrar  Space líder  T tema  Tab siguiente  ? ayuda  Ctrl-c salir",
        _ => "j/k mover  Enter o n ejecutar  Tab siguiente  h/l foco  v crudo  [ ] trabajos  : comando",
    };
    let bottom = Line::from(vec![
        Span::styled("ASPIRE-TEAM", Style::new().fg(t.dim)),
        Span::styled(format!("  ·  {}", t.name), Style::new().fg(t.dim)),
        Span::styled(format!("   {}", hints), Style::new().fg(t.dim)),
    ]);
    f.render_widget(Paragraph::new(bottom).style(Style::new().bg(t.bg)), rows[1]);
}

fn overlays(f: &mut Frame, area: Rect, app: &mut App, t: &Theme) {
    if app.leader {
        let items = [("h", "Inicio"), ("l", "Biblioteca"), ("i", "Instalaciones"), ("s", "Tienda"), ("m", "Mods y parches"), ("p", "Complementos"), ("f", "Social"), ("y", "Sistema"), ("t", "Tema"), ("j", "Trabajos"), ("c", "Comando"), ("r", "Recargar"), ("q", "Salir")];
        let h = items.len() as u16 + 2;
        let r = Rect { x: area.x + area.width.saturating_sub(28), y: area.y + area.height.saturating_sub(h + 2), width: 26.min(area.width), height: h.min(area.height) };
        f.render_widget(Clear, r);
        fill_bg(f, r, t);
        let lines: Vec<Line> = items.iter().map(|(k, d)| Line::from(vec![Span::styled(format!(" {k} "), Style::new().fg(t.accent).bold()), Span::raw(format!("→ {d}"))])).collect();
        f.render_widget(Paragraph::new(lines).block(blk("Space", t, true)), r);
        return;
    }
    if !matches!(app.overlay, Overlay::None) {
        f.render_widget(Clear, area);
        fill_bg(f, area, t);
    }
    match &app.overlay {
        Overlay::None => {}
        Overlay::Menu(c) => {
            let name = app.selected_game().map(|g| g.name.clone()).unwrap_or_default();
            let r = centered(area, 58, area.height.saturating_sub(1).min(GAME_ACTS.len() as u16 + 2));
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let items: Vec<ListItem> = GAME_ACTS.iter().map(|x| ListItem::new(format!("{}{}", x.label, if x.ask.is_some() { "…" } else { "" }))).collect();
            app.ov_state.select(Some(*c));
            let list = List::new(items).block(blk(&name, t, true)).highlight_style(sel_style(t)).highlight_symbol("▸");
            f.render_stateful_widget(list, r, &mut app.ov_state);
        }
        Overlay::Prompt(p) => {
            let r = centered(area, 70.min(area.width), 3);
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let line = Line::from(vec![Span::styled("❯ ", Style::new().fg(t.accent)), Span::raw(p.input.clone())]);
            f.render_widget(Paragraph::new(line).block(blk(&p.label, t, true)), r);
            f.set_cursor_position((r.x + 3 + p.input.chars().count() as u16, r.y + 1));
        }
        Overlay::Launch(l) => {
            let r = centered(area, 84.min(area.width), l.rows.len() as u16 + 6);
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let mut lines: Vec<Line> = vec![];
            for (i, row) in l.rows.iter().enumerate() {
                let sel = i == l.cursor;
                let val = if row.opts.is_empty() {
                    let cur = if sel && l.editing { "█" } else { "" };
                    let txt = if row.text.is_empty() && !(sel && l.editing) { "—".to_string() } else { row.text.clone() };
                    format!("{txt}{cur}")
                } else {
                    format!("◂ {} ▸", row.opts[row.idx])
                };
                let st = if sel { sel_style(t) } else { Style::new().fg(t.fg) };
                lines.push(Line::from(vec![Span::styled(format!("{} {:<20}", if sel { "▸" } else { " " }, row.label), st), Span::styled(val, st)]));
            }
            lines.push(Line::raw(""));
            lines.push(Line::styled(l.rows[l.cursor].hint, Style::new().fg(t.dim)));
            lines.push(Line::styled("Space/h/l cambia · Enter/r jugar · s guardar en el juego · d borrar · Esc cerrar", Style::new().fg(t.accent2)));
            f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(blk(&format!("Jugar · {}", l.name), t, true)), r);
        }
        Overlay::Theme { cursor, .. } => {
            let r = centered(area, 42, THEMES.len() as u16 + 4);
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let mut lines: Vec<Line> = THEMES
                .iter()
                .enumerate()
                .map(|(i, th)| {
                    let sel = i == *cursor;
                    Line::from(vec![
                        Span::styled(if sel { "▸" } else { " " }, Style::new().fg(t.accent)),
                        Span::styled(format!(" {:<17}", th.name), if sel { sel_style(t) } else { Style::new().fg(t.fg) }),
                        Span::styled("██", Style::new().fg(th.accent)),
                        Span::styled("██", Style::new().fg(th.accent2)),
                        Span::styled("██", Style::new().fg(th.ok)),
                        Span::styled("██", Style::new().fg(th.warn)),
                        Span::styled("██", Style::new().fg(th.err)),
                    ])
                })
                .collect();
            lines.push(Line::styled(" Enter guardar · Esc cancelar", Style::new().fg(t.dim)));
            f.render_widget(Paragraph::new(lines).block(blk("TEMA", t, true)), r);
        }
        Overlay::Help => {
            let rows = [
                ("j k", "Barra lateral: navegar secciones"),
                ("Enter o n", "Seleccionar sección"),
                ("l", "Entrar al contenido"),
                ("h", "Volver a la barra lateral"),
                ("gg G", "Inicio / fin de la barra"),
                ("g1..g8", "Ir a la sección N"),
                ("Tab / Shift-Tab", "Sección siguiente / anterior"),
                ("j k", "Contenido: mover arriba / abajo"),
                ("h l", "Contenido: enfocar lista / resultado"),
                ("gg G · 0 $", "Contenido: inicio / fin"),
                ("Ctrl-d u · f b", "Media página / página completa"),
                ("H M L", "Arriba / centro / abajo de la vista"),
                ("Enter o n", "Ejecutar acción (Biblioteca: menú)"),
                ("Space", "Líder (which-key)"),
                (":", "Comando libre de aurelia"),
                ("/ n N", "Biblioteca: filtrar · siguiente · anterior"),
                ("p o I u K", "Jugar · opciones · instalar · actualizar · info"),
                ("i", "Biblioteca: solo instalados"),
                ("v", "Resultado formateado / crudo"),
                ("[ ]  J", "Trabajos"),
                ("T", "Tema (Sistema lee theme-engine)"),
                ("Ctrl-c", "Salir"),
                (":q", "Salir (comando)"),
            ];
            let r = centered(area, 74.min(area.width), rows.len() as u16 + 2);
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let lines: Vec<Line> = rows.iter().map(|(k, d)| Line::from(vec![Span::styled(format!(" {k:<15}"), Style::new().fg(t.accent).bold()), Span::raw(*d)])).collect();
            f.render_widget(Paragraph::new(lines).block(blk("AYUDA · cualquier tecla cierra", t, true)), r);
        }
        Overlay::ProtonPick { act, versions, cursor } => {
            let n = versions.len();
            let r = centered(area, 64.min(area.width), (n + 3) as u16);
            f.render_widget(Clear, r);
            fill_bg(f, r, t);
            let mut lines: Vec<Line> = versions
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let sel = n > 0 && i == *cursor;
                    Line::from(vec![
                        Span::styled(if sel { "▸" } else { " " }, Style::new().fg(t.accent)),
                        Span::styled(format!(" {v}"), if sel { sel_style(t) } else { Style::new().fg(t.fg) }),
                    ])
                })
                .collect();
            lines.push(Line::styled(
                if n > 0 { "j/k mueve · Enter elige · Esc cierra" } else { "(no hay versiones disponibles)" },
                Style::new().fg(t.accent2),
            ));
            f.render_widget(Paragraph::new(lines).block(blk(act.label, t, true)), r);
        }
    }
}
