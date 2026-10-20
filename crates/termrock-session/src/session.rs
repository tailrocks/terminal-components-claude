use std::io::{self, Stdout, Write, stdout};
use std::time::{Duration, Instant};

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::cursor::{Hide, SetCursorStyle};
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, poll,
    read,
};
use ratatui_crossterm::crossterm::execute;
use ratatui_crossterm::crossterm::style::Colored;
use ratatui_crossterm::crossterm::terminal::{
    EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use termrock_core::event::Input;
use termrock_runtime::{App, FeedbackClock, Moment, Runtime};
use termrock_theme::Theme;

pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Startup behavior shared by applications using the terminal session.
///
/// The default preserves the historical `JUNIE_TUI_START_DELAY_MS` behavior
/// and discards input typed before the application starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartupInput {
    delay: Option<Duration>,
    discard_pending: bool,
}

impl Default for StartupInput {
    fn default() -> Self {
        Self::from_environment()
    }
}

impl StartupInput {
    /// Read the historical startup delay and discard pre-start input.
    pub fn from_environment() -> Self {
        Self {
            delay: std::env::var("JUNIE_TUI_START_DELAY_MS")
                .ok()
                .and_then(|value| value.parse().ok())
                .map(Duration::from_millis),
            discard_pending: true,
        }
    }

    /// Set an optional startup delay. `None` disables the delay.
    #[must_use]
    pub const fn delay(mut self, delay: Option<Duration>) -> Self {
        self.delay = delay;
        self
    }

    /// Choose whether to discard input queued before startup.
    #[must_use]
    pub const fn discard_pending(mut self, discard: bool) -> Self {
        self.discard_pending = discard;
        self
    }
}

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

fn restore_modes() -> io::Result<()> {
    let mut out = stdout();
    restore_modes_with(&mut out, disable_raw_mode)
}

fn restore_modes_with(
    out: &mut impl Write,
    restore_raw: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    let mut first = Ok(());

    first = first.and(execute!(out, EnableLineWrap));
    first = first.and(execute!(out, DisableBracketedPaste));
    first = first.and(execute!(out, DisableMouseCapture));
    first = first.and(execute!(out, LeaveAlternateScreen));
    first = first.and(restore_raw());
    first.and(out.flush())
}

struct EntryCleanup<R: FnMut() -> io::Result<()>> {
    restore: R,
    armed: bool,
}

impl<R: FnMut() -> io::Result<()>> Drop for EntryCleanup<R> {
    fn drop(&mut self) {
        if self.armed {
            let _ = (self.restore)();
        }
    }
}

fn enter_guarded<T>(
    enter: impl FnOnce() -> io::Result<T>,
    restore: impl FnMut() -> io::Result<()>,
) -> io::Result<T> {
    let mut cleanup = EntryCleanup {
        restore,
        armed: true,
    };
    let terminal = enter()?;
    cleanup.armed = false;
    Ok(terminal)
}

fn leave_with(
    left: &mut bool,
    show_cursor: impl FnOnce() -> io::Result<()>,
    restore: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    if *left {
        return Ok(());
    }
    let result = show_cursor().and(restore());
    *left = result.is_ok();
    result
}

pub fn chain_panic_hook(restore: impl Fn() + Send + Sync + 'static) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        chain_panic_hook(|| {
            let _ = restore_modes();
        });
        let terminal = enter_guarded(
            || {
                enable_raw_mode()?;
                let mut out = stdout();
                execute!(
                    out,
                    EnterAlternateScreen,
                    EnableMouseCapture,
                    EnableBracketedPaste,
                    Hide
                )?;
                Terminal::new(CrosstermBackend::new(out))
            },
            restore_modes,
        )?;
        Ok(TerminalSession {
            terminal,
            left: false,
        })
    }

    pub const fn terminal(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }

    pub fn leave(&mut self) -> io::Result<()> {
        leave_with(
            &mut self.left,
            || self.terminal.show_cursor(),
            restore_modes,
        )
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

pub fn run<A: App>(app: A, theme: Theme) -> io::Result<()> {
    run_with_feedback_clock(app, theme, FeedbackClock::Elapsed)
}

pub fn run_with_feedback_clock<A: App>(
    app: A,
    theme: Theme,
    clock: FeedbackClock,
) -> io::Result<()> {
    run_with_feedback_clock_and_startup(app, theme, clock, StartupInput::default())
}

/// Run an application with explicit startup input behavior.
pub fn run_with_startup<A: App>(app: A, theme: Theme, startup: StartupInput) -> io::Result<()> {
    run_with_feedback_clock_and_startup(app, theme, FeedbackClock::Elapsed, startup)
}

/// Run an application with explicit feedback clock and startup input behavior.
#[expect(
    clippy::needless_pass_by_value,
    reason = "session owns the supplied theme API and narrows capability once before runtime construction"
)]
pub fn run_with_feedback_clock_and_startup<A: App>(
    app: A,
    theme: Theme,
    clock: FeedbackClock,
    startup: StartupInput,
) -> io::Result<()> {
    prepare_startup_input(startup, std::thread::sleep, drain_pending_input);
    let mut session = TerminalSession::enter()?;
    let origin = Instant::now();
    let mut rt = Runtime::new_with_feedback_clock(app, theme.for_terminal(), clock);
    let _ = rt.initialize();
    let suppress_colors = Colored::ansi_color_disabled_memoized();
    let mut pending = None;
    loop {
        rt.advance_to(Moment::from_duration(origin.elapsed()))
            .map_err(io::Error::other)?;
        loop {
            if rt.needs_settle() {
                let _ = rt.settle();
            }
            if rt.needs_present() {
                let mut painted = None;
                session.terminal().draw(|f| {
                    painted = Some(rt.draw(f));
                    if suppress_colors {
                        reset_frame_colors(f.buffer_mut());
                    }
                })?;
                let bar = if let Some(frame) = painted {
                    let bar = frame.cursor_bar();
                    frame.commit_presented();
                    bar
                } else {
                    false
                };
                if bar {
                    execute!(stdout(), SetCursorStyle::SteadyBar)?;
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
        let now = Moment::from_duration(origin.elapsed());
        let ready = match rt.next_deadline() {
            Some(at) => poll(
                at.saturating_duration_since(now)
                    .min(Duration::from_secs(60)),
            )?,
            None => true,
        };
        if ready {
            pending = Input::from_crossterm(read()?);
        }
    }
    session.leave()
}

fn prepare_startup_input(
    startup: StartupInput,
    sleep: impl FnOnce(Duration),
    drain: impl FnOnce() -> io::Result<usize>,
) {
    if let Some(delay) = startup.delay {
        sleep(delay);
    }
    if startup.discard_pending {
        let _ = drain();
    }
}

fn drain_pending_input() -> io::Result<usize> {
    use ratatui_crossterm::crossterm::event::{poll as poll_input, read as read_input};
    use ratatui_crossterm::crossterm::terminal::{
        disable_raw_mode as disable_input_raw_mode, enable_raw_mode as enable_input_raw_mode,
        is_raw_mode_enabled,
    };

    let was_raw = is_raw_mode_enabled()?;
    if !was_raw {
        enable_input_raw_mode()?;
    }
    let drained = (|| {
        let mut count = 0;
        while poll_input(Duration::ZERO)? {
            let _ = read_input()?;
            count += 1;
        }
        Ok(count)
    })();
    if was_raw {
        drained
    } else {
        disable_input_raw_mode()?;
        drained
    }
}

fn reset_frame_colors(buffer: &mut ratatui_core::buffer::Buffer) {
    for cell in &mut buffer.content {
        cell.fg = ratatui_core::style::Color::Reset;
        cell.bg = ratatui_core::style::Color::Reset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::sync::{Arc, Mutex};

    #[test]
    fn startup_delay_precedes_discard_and_drain_errors_are_ignored() {
        let calls = std::cell::RefCell::new(Vec::new());
        prepare_startup_input(
            StartupInput {
                delay: Some(Duration::from_millis(17)),
                discard_pending: true,
            },
            |delay| {
                calls
                    .borrow_mut()
                    .push(format!("sleep:{}", delay.as_millis()))
            },
            || {
                calls.borrow_mut().push("drain".to_owned());
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            },
        );
        assert_eq!(*calls.borrow(), vec!["sleep:17", "drain"]);
    }

    #[test]
    fn startup_options_can_disable_delay_and_pending_input_discard() {
        let options = StartupInput::default().delay(None).discard_pending(false);
        let calls = Cell::new(0);
        prepare_startup_input(
            options,
            |_| calls.set(calls.get() + 1),
            || {
                calls.set(calls.get() + 10);
                Ok(0)
            },
        );
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn partial_entry_errors_restore_once_and_preserve_the_entry_error() {
        for failed_stage in 0..4 {
            let reached = Cell::new(0);
            let restored = Cell::new(0);
            let result = enter_guarded(
                || {
                    for stage in 0..4 {
                        reached.set(stage);
                        if stage == failed_stage {
                            return Err::<(), _>(io::Error::from(io::ErrorKind::BrokenPipe));
                        }
                    }
                    Ok(())
                },
                || {
                    restored.set(restored.get() + 1);
                    Err(io::Error::from(io::ErrorKind::PermissionDenied))
                },
            );
            assert_eq!(
                result.err().map(|error| error.kind()),
                Some(io::ErrorKind::BrokenPipe)
            );
            assert_eq!(reached.get(), failed_stage);
            assert_eq!(restored.get(), 1);
        }
        let restored = Cell::new(false);
        assert!(
            enter_guarded(
                || Ok(()),
                || {
                    restored.set(true);
                    Ok(())
                }
            )
            .is_ok()
        );
        assert!(!restored.get());
    }

    #[derive(Default)]
    struct BrokenOutput {
        calls: usize,
        flushes: usize,
        bytes: Vec<u8>,
    }
    impl Write for BrokenOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    #[test]
    fn broken_output_still_restores_later_modes_raw_input_and_flushes() {
        let mut out = BrokenOutput::default();
        let raw = Cell::new(false);
        let result = restore_modes_with(&mut out, || {
            raw.set(true);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(raw.get());
        assert!(out.flushes >= 4);
        let mut final_command = Vec::new();
        assert!(execute!(final_command, LeaveAlternateScreen).is_ok());
        assert!(out.bytes.ends_with(&final_command));
    }

    #[test]
    fn raw_restore_failure_is_retained_without_skipping_final_flush() {
        let mut out = BrokenOutput {
            calls: 1,
            ..BrokenOutput::default()
        };
        let result = restore_modes_with(&mut out, || {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::PermissionDenied)
        );
        assert_eq!(out.flushes, 5);
    }

    #[test]
    fn failed_cursor_restore_still_restores_modes_and_allows_retry() {
        let mut left = false;
        let restored = Cell::new(0);
        let result = leave_with(
            &mut left,
            || Err(io::Error::from(io::ErrorKind::BrokenPipe)),
            || {
                restored.set(restored.get() + 1);
                Ok(())
            },
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(!left);
        assert!(
            leave_with(
                &mut left,
                || Ok(()),
                || {
                    restored.set(restored.get() + 1);
                    Ok(())
                }
            )
            .is_ok()
        );
        assert!(left);
        assert_eq!(restored.get(), 2);
        assert!(
            leave_with(
                &mut left,
                || Err(io::Error::from(io::ErrorKind::Other)),
                || Err(io::Error::from(io::ErrorKind::Other))
            )
            .is_ok()
        );
        assert!(left);
    }

    #[test]
    fn failed_raw_restore_is_reported_and_retried() {
        let mut left = false;
        let result = leave_with(
            &mut left,
            || Ok(()),
            || Err(io::Error::from(io::ErrorKind::PermissionDenied)),
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::PermissionDenied)
        );
        assert!(!left);
        assert!(leave_with(&mut left, || Ok(()), || Ok(())).is_ok());
        assert!(left);
    }

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
