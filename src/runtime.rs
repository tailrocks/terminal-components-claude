//! Terminal runtime shared by every application built on the library:
//! one terminal-session guard that owns raw mode, the alternate screen,
//! mouse capture, bracketed paste, cursor visibility and line-wrap state
//! (restored on every exit path, including panics), an event loop that
//! coalesces input floods, and animation ticks on demand.

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use crate::core::event::{Input, Outcome};

pub trait Application {
    fn handle(&mut self, input: Input) -> Outcome;
    fn render(&mut self, frame: &mut Frame);
    fn should_quit(&self) -> bool;
    /// How often to deliver `Input::Tick`.
    fn tick_interval(&self) -> Duration;
}

/// Owns every piece of terminal state the application changes and puts it
/// back in reverse order when dropped: bracketed paste and mouse capture
/// off, cursor shown, line wrap re-enabled, alternate screen left, raw mode
/// off. A panic inside the event loop unwinds through the guard, so the
/// host shell is restored before the panic message is printed.
pub struct TerminalSession {
    terminal: ratatui::DefaultTerminal,
    active: bool,
}

/// DECAWM: automatic line wrap. Applications that draw to the last column
/// turn it off; the guard turns it back on for the host shell.
const ENABLE_WRAP: &str = "\x1b[?7h";

impl TerminalSession {
    /// Enter raw mode, the alternate screen, mouse capture and bracketed
    /// paste. Installs a panic hook that restores the terminal first.
    pub fn enter() -> std::io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(
            out,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )?;
        let backend = ratatui::backend::CrosstermBackend::new(out);
        let terminal = ratatui::Terminal::new(backend)?;
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous(info);
        }));
        Ok(Self {
            terminal,
            active: true,
        })
    }

    pub fn terminal(&mut self) -> &mut ratatui::DefaultTerminal {
        &mut self.terminal
    }

    /// Restore explicitly (idempotent); `Drop` does the same.
    pub fn leave(&mut self) {
        if self.active {
            self.active = false;
            restore_terminal();
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.leave();
    }
}

fn restore_terminal() {
    let mut out = stdout();
    let _ = execute!(out, DisableBracketedPaste, DisableMouseCapture, Show);
    let _ = out.write_all(ENABLE_WRAP.as_bytes());
    let _ = execute!(out, LeaveAlternateScreen);
    let _ = out.flush();
    let _ = disable_raw_mode();
}

/// Run an application until it asks to quit. The terminal is restored on
/// every exit path: normal quit, I/O error, or panic.
pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    let mut session = TerminalSession::enter()?;
    let result = event_loop(session.terminal(), app);
    session.leave();
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
        // a tick-driven transition may have asked to quit without any input
        if app.should_quit() {
            return Ok(());
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
            if app.should_quit() {
                // draw the final frame so a closing caption is seen before restore
                if dirty {
                    terminal.draw(|f| app.render(f))?;
                }
                return Ok(());
            }
        }
    }
}

/// Drain any input that is already queued (stale key presses from the
/// command that started the application) so it cannot skip an opening
/// sequence. Returns how many events were discarded.
pub fn drain_pending_input() -> std::io::Result<usize> {
    let mut n = 0;
    while event::poll(Duration::ZERO)? {
        let _ = event::read()?;
        n += 1;
    }
    Ok(n)
}
