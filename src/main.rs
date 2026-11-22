mod actions;
mod app;
mod backend;
mod patch;
mod theme;
mod ui;

use anyhow::Result;
use ratatui::{
    crossterm::event::{self, Event, EnableMouseCapture, DisableMouseCapture},
    crossterm::execute,
    DefaultTerminal,
};
use std::time::Duration;

fn run(term: &mut DefaultTerminal, app: &mut app::App) -> Result<()> {
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    loop {
        app.pump();
        term.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(k) => app.on_key(k),
                Event::Mouse(m) => app.on_mouse(m, term.size().map(|s| (s.width, s.height)).unwrap_or((0, 0))),
                _ => {}
            }
        }
        if let Some(args) = app.pending.take() {
            let _ = execute!(std::io::stdout(), DisableMouseCapture);
            ratatui::restore();
            let _ = backend::run_interactive(&args);
            *term = ratatui::init();
            let _ = execute!(std::io::stdout(), EnableMouseCapture);
            term.clear()?;
            app.after_interactive();
        }
        if app.quit {
            return Ok(());
        }
        app.tick = app.tick.wrapping_add(1);
    }
}

fn main() -> Result<()> {
    let mut app = app::App::new();
    let mut term = ratatui::init();
    let res = run(&mut term, &mut app);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    res
}
