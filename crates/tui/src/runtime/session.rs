//! The terminal session (`COMPONENT_ARCHITECTURE.md` §17.0 A1, §22.2 item 7).
//!
//! A faithful mirror of ratatui's `try_init` / `try_restore`
//! (`ratatui-0.30.2/src/init.rs:369-399`, unavailable through
//! `ratatui-core`) plus the two modes ratatui's `init` never enables: mouse
//! capture and bracketed paste. The chained panic hook is installed
//! **before** the first mode change (`init.rs:196-197`); every mode is a
//! typed crossterm command in one `execute!`, never a raw escape string;
//! restore is one reverse-order `execute!` and `leave` is idempotent. This
//! is the only file that names raw-mode / alternate-screen commands.

use std::io::{self, Stdout, Write, stdout};
use std::time::Duration;

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, poll,
    read,
};
use ratatui_crossterm::crossterm::execute;
use ratatui_crossterm::crossterm::terminal::{
    DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};

use super::{App, Runtime};
use crate::event::Input;
use crate::theme::Theme;

/// The terminal every application draws into.
pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Owns raw mode, the alternate screen, mouse capture, bracketed paste and
/// line wrap for the duration of the session.
pub struct TerminalSession {
    terminal: DefaultTerminal,
    left: bool,
}

impl core::fmt::Debug for TerminalSession {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TerminalSession")
            .field("left", &self.left)
            .finish_non_exhaustive()
    }
}

/// Undo every mode this session sets; safe to call more than once.
fn restore_modes() -> io::Result<()> {
    let mut out = stdout();
    execute!(
        out,
        EnableLineWrap,
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    out.flush()
}

/// Install a panic hook that runs `restore` and then delegates to the
/// previous hook — in that order, mirroring `try_init`.
pub fn chain_panic_hook(restore: impl Fn() + Send + Sync + 'static) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

impl TerminalSession {
    /// Enter the session: hook first, then raw mode, then the alternate
    /// screen, mouse capture, bracketed paste and no line wrap.
    ///
    /// # Errors
    /// Any terminal command that fails; nothing is left half-set because
    /// the hook restores on the way out.
    pub fn enter() -> io::Result<Self> {
        chain_panic_hook(|| {
            let _ = restore_modes();
        });
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(
            out,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste,
            DisableLineWrap
        )?;
        let terminal = Terminal::new(CrosstermBackend::new(out))?;
        Ok(TerminalSession {
            terminal,
            left: false,
        })
    }

    /// The terminal.
    pub const fn terminal(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }

    /// Leave the session; idempotent.
    ///
    /// # Errors
    /// A terminal command that fails.
    pub fn leave(&mut self) -> io::Result<()> {
        if self.left {
            return Ok(());
        }
        self.left = true;
        self.terminal.show_cursor()?;
        restore_modes()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

/// Run an application to completion: draw, wait for input (at the tick
/// cadence when a repaint is pending, at the idle cadence otherwise),
/// handle, repeat until `App::should_quit` or `Cx::quit`.
///
/// # Errors
/// Terminal I/O errors.
pub fn run<A: App>(app: A, theme: Theme) -> io::Result<()> {
    let mut session = TerminalSession::enter()?;
    let mut rt = Runtime::new(app, theme);
    let tick = Duration::from_millis(rt.theme().design.motion.tick_ms);
    let idle = Duration::from_millis(rt.theme().design.motion.idle_tick_ms);
    loop {
        session.terminal().draw(|f| rt.draw(f))?;
        if rt.app().should_quit() || rt.quit_requested() {
            break;
        }
        let wait = if rt.wants_tick() { tick } else { idle };
        let mut got_input = false;
        while poll(Duration::ZERO)? {
            if let Some(input) = Input::from_crossterm(read()?) {
                let _ = rt.handle(input);
                got_input = true;
            }
        }
        if !got_input {
            if poll(wait)? {
                if let Some(input) = Input::from_crossterm(read()?) {
                    let _ = rt.handle(input);
                }
            } else {
                let _ = rt.handle(Input::Tick);
            }
        }
    }
    session.leave()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn panic_hook_restores_before_delegating() {
        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
        let outer = Arc::clone(&log);
        let original = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |_| {
            if let Ok(mut l) = outer.lock() {
                l.push("previous");
            }
        }));
        let inner = Arc::clone(&log);
        chain_panic_hook(move || {
            if let Ok(mut l) = inner.lock() {
                l.push("restore");
            }
        });
        let _ = std::panic::catch_unwind(|| {
            panic!("boom");
        });
        std::panic::set_hook(original);
        let seen = log.lock().map(|l| l.clone()).unwrap_or_default();
        assert_eq!(seen, vec!["restore", "previous"]);
    }
}
