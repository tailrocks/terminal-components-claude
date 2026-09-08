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
use std::time::{Duration, Instant};

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, poll,
    read,
};
use ratatui_crossterm::crossterm::execute;
use ratatui_crossterm::crossterm::terminal::{
    EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use super::{App, Runtime};
use crate::event::Input;
use crate::theme::Theme;

/// The terminal every application draws into.
pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Owns raw mode, the alternate screen, mouse capture and bracketed paste for
/// the duration of the session.
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
    /// screen, mouse capture and bracketed paste.
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
            EnableBracketedPaste
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

/// Run an application to completion: draw, wait for input or the earliest
/// requested repaint deadline, handle, repeat until `App::should_quit` or
/// `Cx::quit`.
///
/// `theme` is narrowed once, here, by [`Theme::for_terminal`] (§34.2). This is
/// the only place in the library that resolves colour capability: a theme is
/// built before the terminal is known, and this is the function that opens it.
/// [`Runtime::new`] deliberately does **not** do it — `Harness`, `Scene` and
/// the crate's front-page doc example all call it, so an environment read there
/// would make every render digest a function of the CI runner's `TERM`.
///
/// [`run_with_feedback_clock`] selects simulation feedback explicitly while
/// retaining the same terminal capability narrowing.
///
/// # Errors
/// Terminal I/O errors.
pub fn run<A: App>(app: A, theme: Theme) -> io::Result<()> {
    run_with_feedback_clock(app, theme, super::FeedbackClock::Elapsed)
}

/// Run with a feedback clock selected before initialization or first activation.
/// Domain adapters pass their already-seeked simulation epoch and synchronize
/// absolute simulation time through `Cx`; the driver still owns elapsed time.
/// Simulation feedback never adds a wall-clock expiry wake or a polling loop.
///
/// # Errors
/// Terminal I/O errors.
#[expect(
    clippy::needless_pass_by_value,
    reason = "session owns the supplied theme API and narrows capability once before runtime construction"
)]
pub fn run_with_feedback_clock<A: App>(
    app: A,
    theme: Theme,
    clock: super::FeedbackClock,
) -> io::Result<()> {
    let mut session = TerminalSession::enter()?;
    let origin = Instant::now();
    let mut rt = Runtime::new_with_feedback_clock(app, theme.for_terminal(), clock);
    let _ = rt.initialize();
    let mut pending = None;
    loop {
        // Exactly one clock advance per scheduler turn. A callback rearming
        // deadline <= now cannot recursively starve an already-read input.
        rt.advance_to(super::Moment::from_duration(origin.elapsed()))
            .map_err(io::Error::other)?;
        loop {
            if rt.needs_settle() {
                let _ = rt.settle();
            }
            if rt.needs_present() {
                let mut painted = None;
                session.terminal().draw(|f| {
                    painted = Some(rt.draw(f));
                })?;
                if let Some(frame) = painted {
                    frame.commit_presented();
                }
                continue;
            }
            if !rt.needs_settle() {
                break;
            }
        }
        if rt.app().should_quit() || rt.quit_requested() {
            break;
        }
        if let Some(input) = pending.take() {
            match rt.handle(input) {
                Ok(_) => {}
                Err(event) => pending = Some(event.into_input()),
            }
            continue;
        }
        let now = super::Moment::from_duration(origin.elapsed());
        let ready = match rt.next_deadline() {
            // Bound transport timeout representations even for Moment's
            // saturated maximum. This never synthesizes an application update.
            Some(at) => poll(
                at.saturating_duration_since(now)
                    .min(Duration::from_secs(60)),
            )?,
            None => true,
        };
        if ready {
            pending = Input::from_crossterm(read()?);
        }
        // A real idle timeout advances elapsed time on the next turn. It never
        // fabricates input ticks, and an immediate deadline still polls input.
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
