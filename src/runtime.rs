//! Terminal runtime shared by every application built on the library:
//! raw mode + alternate screen + mouse capture + bracketed paste, an event
//! loop that coalesces input floods, and animation ticks on demand.

use std::io::stdout;
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;

use crate::core::event::{Input, Outcome};

pub trait Application {
    fn handle(&mut self, input: Input) -> Outcome;
    fn render(&mut self, frame: &mut Frame);
    fn should_quit(&self) -> bool;
    /// How often to deliver `Input::Tick`.
    fn tick_interval(&self) -> Duration;
}

/// Run an application until it asks to quit. Restores the terminal even on
/// error; panics are handled by ratatui's installed hook.
pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture, EnableBracketedPaste)?;
    let result = event_loop(&mut terminal, app);
    let _ = execute!(stdout(), DisableMouseCapture, DisableBracketedPaste);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut impl Application,
) -> std::io::Result<()> {
    let mut dirty = true;
    let mut last_tick = Instant::now();
    loop {
        if dirty {
            terminal.draw(|f| app.render(f))?;
            dirty = false;
        }
        let interval = app.tick_interval();
        let wait = interval.saturating_sub(last_tick.elapsed());
        if event::poll(wait)? {
            loop {
                let ev = event::read()?;
                if let Some(input) = Input::from_crossterm(ev)
                    && app.handle(input) == Outcome::Changed
                {
                    dirty = true;
                }
                if app.should_quit() {
                    return Ok(());
                }
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= interval {
            last_tick = Instant::now();
            if app.handle(Input::Tick) == Outcome::Changed {
                dirty = true;
            }
        }
    }
}
