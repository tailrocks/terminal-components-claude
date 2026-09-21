//! The terminal session (`COMPONENT_ARCHITECTURE.md` §17.0 A1, §22.2 item 7).
//!
//! A faithful mirror of ratatui's `try_init` / `try_restore`
//! (`ratatui-0.30.2/src/init.rs:369-399`, unavailable through
//! `ratatui-core`) plus the two modes ratatui's `init` never enables: mouse
//! capture and bracketed paste. The chained panic hook is installed
//! **before** the first mode change (`init.rs:196-197`); every mode is a
//! typed crossterm command in one `execute!`, never a raw escape string;
//! restoration attempts every reverse-order step and `leave` is retryable
//! after failure, idempotent after success. This
//! is the only file that names raw-mode / alternate-screen commands.
//!
//! One shared cleanup operation restores cursor visibility and every mode on
//! every path — normal exit, error, partial startup, suspension and the panic
//! hook before it delegates to the previous hook. Colour capability is
//! narrowed once per launch by [`TerminalColorPolicy`]. On Unix the session
//! additionally owns job-control suspension through the process-lifetime
//! signal broker (`SIGNAL_BROKER`, ADJ-13): the only mutable core global,
//! holding nothing but two signal flags, the serialized session lease and the
//! two registration identities.

use std::io::{self, Stdout, Write, stdout};
#[cfg(all(unix, feature = "crossterm"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(unix, feature = "crossterm"))]
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::cursor::Show;
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, poll,
    read,
};
use ratatui_crossterm::crossterm::execute;
#[cfg(all(unix, feature = "crossterm"))]
use ratatui_crossterm::crossterm::terminal::size;
use ratatui_crossterm::crossterm::terminal::{
    EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use super::{App, Runtime};
use crate::event::Input;
use crate::theme::{ColorLevel, Theme};

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

/// Undo cursor hiding and every mode this session sets; safe to call more
/// than once. This is the single shared cleanup operation used by `leave`,
/// the entry guard, suspension and the panic hook before it delegates.
fn restore_terminal() -> io::Result<()> {
    let mut out = stdout();
    restore_terminal_with(&mut out, disable_raw_mode)
}

fn restore_terminal_with(
    out: &mut impl Write,
    restore_raw: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    // Cursor first, matching `leave` order; every step is attempted even when
    // an earlier step failed, and the first error is preserved so a failed
    // Show/output can never claim successful cleanup.
    let show = execute!(out, Show);
    let modes = restore_modes_with(out, restore_raw);
    show.and(modes)
}

fn restore_modes_with(
    out: &mut impl Write,
    restore_raw: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    let mut first = Ok(());
    // Evaluate each step even when an earlier write failed. In particular,
    // broken stdout must never prevent restoring the terminal's input mode.
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

fn leave_with(left: &mut bool, restore: impl FnOnce() -> io::Result<()>) -> io::Result<()> {
    if *left {
        return Ok(());
    }
    let result = restore();
    *left = result.is_ok();
    result
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

/// Which of the two process-lifetime SIGTSTP registrations completed.
/// A retry registers only the missing one; a success is never repeated.
#[cfg(all(unix, feature = "crossterm"))]
#[derive(Debug, Default)]
struct InstallPhase {
    /// `flag::register(SIGTSTP, pending)`: always records the stop.
    first: Option<signal_hook::SigId>,
    /// `flag::register_conditional_default(SIGTSTP, inactive)`: emulates the
    /// default stop while no session holds the lease, inert while one does.
    second: Option<signal_hook::SigId>,
}

#[cfg(all(unix, feature = "crossterm"))]
impl InstallPhase {
    #[cfg(test)]
    const fn complete(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }
}

/// Process-lifetime SIGTSTP broker state (ADJ-13). The handlers only flip the
/// two flags: no lock, no terminal I/O, no waiting and no stopping inside a
/// handler. The runner owns every mode change and the actual stop emulation.
#[cfg(all(unix, feature = "crossterm"))]
#[derive(Debug)]
struct SignalBroker {
    /// True while no session holds the lease; arms default-stop emulation.
    inactive: Arc<AtomicBool>,
    /// A stop arrived and is waiting for the runner to service it.
    pending: Arc<AtomicBool>,
    /// The serialized session lease; at most one session holds it.
    lease: bool,
    /// The two registration identities, completed at most once.
    install: InstallPhase,
}

#[cfg(all(unix, feature = "crossterm"))]
impl SignalBroker {
    /// Flag storage only; registration happens under the lock so a failure
    /// can be retried without losing the first success.
    fn new() -> Self {
        Self {
            inactive: Arc::new(AtomicBool::new(true)),
            pending: Arc::new(AtomicBool::new(false)),
            lease: false,
            install: InstallPhase::default(),
        }
    }

    /// Register both handlers with the production `signal-hook` entry points.
    fn complete_missing(&mut self) -> io::Result<()> {
        use signal_hook::consts::SIGTSTP;
        self.complete_missing_with(
            |pending| signal_hook::flag::register(SIGTSTP, Arc::clone(pending)),
            |inactive| {
                signal_hook::flag::register_conditional_default(SIGTSTP, Arc::clone(inactive))
            },
        )
    }

    /// Register only the missing handler, keeping the same flag identities
    /// across retries. A failed second registration keeps the first success.
    fn complete_missing_with(
        &mut self,
        mut register_pending: impl FnMut(&Arc<AtomicBool>) -> io::Result<signal_hook::SigId>,
        mut register_default: impl FnMut(&Arc<AtomicBool>) -> io::Result<signal_hook::SigId>,
    ) -> io::Result<()> {
        if self.install.first.is_none() {
            self.install.first = Some(register_pending(&self.pending)?);
        }
        if self.install.second.is_none() {
            self.install.second = Some(register_default(&self.inactive)?);
        }
        Ok(())
    }
}

/// The sole permitted mutable core global (ADJ-13): one private Unix
/// crossterm signal-broker cell. No other global state exists in this crate.
#[cfg(all(unix, feature = "crossterm"))]
static SIGNAL_BROKER: OnceLock<Mutex<SignalBroker>> = OnceLock::new();

/// Lock the broker cell, initializing flag storage exactly once.
/// Poison maps to an I/O error; the caller decides whether to retry.
#[cfg(all(unix, feature = "crossterm"))]
fn lock_cell(
    cell: &'static OnceLock<Mutex<SignalBroker>>,
) -> io::Result<MutexGuard<'static, SignalBroker>> {
    cell.get_or_init(|| Mutex::new(SignalBroker::new()))
        .lock()
        .map_err(|_| io::Error::other("terminal signal broker lock poisoned"))
}

/// Acquire the serialized session lease, completing both registrations.
/// Overlapping sessions are rejected before any terminal mutation.
#[cfg(all(unix, feature = "crossterm"))]
fn broker_activate() -> io::Result<()> {
    let mut broker = lock_cell(&SIGNAL_BROKER)?;
    if broker.lease {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "a terminal session already holds the process signal lease",
        ));
    }
    broker.complete_missing()?;
    broker.pending.store(false, Ordering::SeqCst);
    broker.inactive.store(false, Ordering::SeqCst);
    broker.lease = true;
    Ok(())
}

/// Release the lease and re-arm default-stop emulation. Call only after mode
/// cleanup was attempted; idempotent, and safe to call from `Drop`.
#[cfg(all(unix, feature = "crossterm"))]
fn broker_deactivate() {
    if let Ok(mut broker) = lock_cell(&SIGNAL_BROKER) {
        broker.pending.store(false, Ordering::SeqCst);
        broker.inactive.store(true, Ordering::SeqCst);
        broker.lease = false;
    }
}

/// Re-arm the active session after continuation, coalescing repeated flags.
#[cfg(all(unix, feature = "crossterm"))]
fn broker_reactivate() -> io::Result<()> {
    let broker = lock_cell(&SIGNAL_BROKER)?;
    if !broker.lease {
        return Err(io::Error::other(
            "terminal session lost its signal lease across suspension",
        ));
    }
    broker.pending.store(false, Ordering::SeqCst);
    broker.inactive.store(false, Ordering::SeqCst);
    Ok(())
}

/// Take a pending stop observed during the active wait. Repeated deliveries
/// coalesce into one service; a poisoned broker reports no stop.
#[cfg(all(unix, feature = "crossterm"))]
fn broker_take_pending() -> bool {
    SIGNAL_BROKER.get().is_some_and(|cell| {
        cell.lock()
            .is_ok_and(|broker| broker.pending.swap(false, Ordering::SeqCst))
    })
}

/// Emulate the default SIGTSTP disposition from the runner: the process stops
/// (through SIGSTOP internally) and resumes here after `fg`/SIGCONT, with the
/// mode cleanup already done and re-entry still ahead.
#[cfg(all(unix, feature = "crossterm"))]
fn broker_emulate_stop() -> io::Result<()> {
    signal_hook::low_level::emulate_default_handler(signal_hook::consts::SIGTSTP)
}

/// Record a stop delivery without raising a real signal, which would stop the
/// test runner itself. The flag path is identical to the handler's.
#[cfg(all(unix, feature = "crossterm"))]
#[cfg(test)]
fn broker_simulate_stop_delivery() {
    if let Ok(broker) = lock_cell(&SIGNAL_BROKER) {
        broker.pending.store(true, Ordering::SeqCst);
    }
}

/// Read `(lease_held, inactive, pending)` for lifecycle assertions.
#[cfg(all(unix, feature = "crossterm"))]
#[cfg(test)]
fn broker_snapshot_for_test() -> (bool, bool, bool) {
    lock_cell(&SIGNAL_BROKER).map_or((false, true, false), |broker| {
        (
            broker.lease,
            broker.inactive.load(Ordering::SeqCst),
            broker.pending.load(Ordering::SeqCst),
        )
    })
}

impl TerminalSession {
    /// Enter the session: hook first, then raw mode, then the alternate
    /// screen, mouse capture and bracketed paste.
    ///
    /// # Errors
    /// Any terminal command that fails, or an overlapping session that already
    /// holds the process signal lease. A construction guard attempts every
    /// restoration step on failure, including ordinary I/O errors. The original
    /// entry error is retained even when restoration also fails.
    pub fn enter() -> io::Result<Self> {
        #[cfg(all(unix, feature = "crossterm"))]
        broker_activate()?;
        chain_panic_hook(|| {
            let _ = restore_terminal();
        });
        let terminal = match enter_guarded(
            || {
                enable_raw_mode()?;
                let mut out = stdout();
                execute!(
                    out,
                    EnterAlternateScreen,
                    EnableMouseCapture,
                    EnableBracketedPaste
                )?;
                Terminal::new(CrosstermBackend::new(out))
            },
            restore_terminal,
        ) {
            Ok(terminal) => terminal,
            Err(error) => {
                #[cfg(all(unix, feature = "crossterm"))]
                broker_deactivate();
                return Err(error);
            }
        };
        Ok(TerminalSession {
            terminal,
            left: false,
        })
    }

    /// The terminal.
    pub const fn terminal(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }

    /// Leave the session; idempotent after success. Failed restoration remains
    /// retryable, including the final attempt made by `Drop`.
    ///
    /// # Errors
    /// A terminal command that fails.
    pub fn leave(&mut self) -> io::Result<()> {
        let result = leave_with(&mut self.left, restore_terminal);
        #[cfg(all(unix, feature = "crossterm"))]
        if result.is_ok() {
            broker_deactivate();
        }
        result
    }

    /// Restore modes for an imminent job-control stop without consuming the
    /// session. The broker goes inactive only after cleanup was attempted.
    ///
    /// # Errors
    /// A terminal command that fails; the stop is not serviced then.
    #[cfg(all(unix, feature = "crossterm"))]
    fn suspend_for_stop(&mut self) -> io::Result<()> {
        debug_assert!(!self.left, "only a live session can suspend for a stop");
        let result = restore_terminal();
        broker_deactivate();
        result
    }

    /// Re-enter every mode after continuation, clear the stale frame buffers
    /// and re-read geometry. The driver publishes the geometry as a `Resize`
    /// so the resumed session performs a full redraw.
    ///
    /// # Errors
    /// A terminal command that fails, or a lease lost across suspension. A
    /// failed re-entry attempts cleanup and goes inactive before reporting.
    #[cfg(all(unix, feature = "crossterm"))]
    fn resume_after_continue(&mut self) -> io::Result<(u16, u16)> {
        broker_reactivate()?;
        let reentered = (|| {
            enable_raw_mode()?;
            let mut out = stdout();
            execute!(
                out,
                EnterAlternateScreen,
                EnableMouseCapture,
                EnableBracketedPaste
            )?;
            self.terminal.clear()?;
            size()
        })();
        match reentered {
            Ok(geometry) => Ok(geometry),
            Err(error) => {
                let _ = restore_terminal();
                broker_deactivate();
                Err(error)
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.leave();
        #[cfg(all(unix, feature = "crossterm"))]
        broker_deactivate();
    }
}

/// How the session driver resolves terminal colour capability (§34.2/§74.1).
/// This is the only place in the library that resolves capability: a theme is
/// built before the terminal is known, and the driver is the function that
/// opens it. [`Runtime::new`] deliberately does **not** do it — `Harness`,
/// `Scene` and the crate's front-page doc example all call it, so an
/// environment read there would make every render digest a function of the CI
/// runner's `TERM`.
///
/// Neither policy widens an already-narrower authored theme, and neither
/// infers an override from the passed theme: only this explicit policy and,
/// for [`TerminalColorPolicy::Detect`], the ambient environment decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalColorPolicy {
    /// Narrow once from ambient detection via [`Theme::for_terminal`].
    Detect,
    /// Narrow once to exactly this level via [`Theme::for_level`], with no
    /// ambient capability detection.
    Requested(ColorLevel),
}

/// Narrow the launch theme once under the given policy.
fn resolve_policy(theme: &Theme, policy: TerminalColorPolicy) -> Theme {
    match policy {
        TerminalColorPolicy::Detect => theme.for_terminal(),
        TerminalColorPolicy::Requested(level) => theme.for_level(level),
    }
}

/// The terminal-boundary lifecycle service quantum. The driver never waits
/// past it, so an externally signalled stop is serviced within 100 ms without
/// requiring real input; `crossterm` retries `EINTR`, so a flag combined with
/// an unbounded read would not be sufficient. A lifecycle-only wake stages no
/// Tick, Settle, Bootstrap, simulation advancement, feedback expiry, repaint
/// or application callback.
const LIFECYCLE_QUANTUM: Duration = Duration::from_millis(100);

/// How long the driver waits: the time remaining to the earliest real
/// deadline, capped at the lifecycle quantum. Overdue deadlines wait zero and
/// are handled immediately; an absent deadline still services the lifecycle.
fn wait_bound(deadline: Option<super::Moment>, now: super::Moment) -> Duration {
    match deadline {
        Some(at) => at.saturating_duration_since(now).min(LIFECYCLE_QUANTUM),
        None => LIFECYCLE_QUANTUM,
    }
}

/// Run an application to completion: draw, wait for input or the earliest
/// requested repaint deadline, handle, repeat until `App::should_quit` or
/// `Cx::quit`.
///
/// `theme` is narrowed once, here, from ambient detection (§34.2).
/// [`run_with_feedback_clock_and_color`] with an explicit policy retains the
/// same single session driver.
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
/// Capability is detected, exactly as [`run`].
///
/// # Errors
/// Terminal I/O errors.
pub fn run_with_feedback_clock<A: App>(
    app: A,
    theme: Theme,
    clock: super::FeedbackClock,
) -> io::Result<()> {
    run_with_feedback_clock_and_color(app, theme, clock, TerminalColorPolicy::Detect)
}

/// Run with both the feedback clock and the colour policy selected before
/// initialization. `Requested` narrows via [`Theme::for_level`] without
/// ambient detection; `Detect` preserves the historical ambient behaviour.
///
/// # Errors
/// Terminal I/O errors.
#[expect(
    clippy::needless_pass_by_value,
    reason = "session owns the supplied theme API and narrows capability once before runtime construction"
)]
pub fn run_with_feedback_clock_and_color<A: App>(
    app: A,
    theme: Theme,
    clock: super::FeedbackClock,
    policy: TerminalColorPolicy,
) -> io::Result<()> {
    drive(app, resolve_policy(&theme, policy), clock)
}

/// The single session driver behind every launch API.
fn drive<A: App>(app: A, theme: Theme, clock: super::FeedbackClock) -> io::Result<()> {
    let mut session = TerminalSession::enter()?;
    let origin = Instant::now();
    let mut rt = Runtime::new_with_feedback_clock(app, theme, clock);
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
        #[cfg(all(unix, feature = "crossterm"))]
        if broker_take_pending() {
            session.suspend_for_stop()?;
            broker_emulate_stop()?;
            let (width, height) = session.resume_after_continue()?;
            pending = Some(Input::Resize(width, height));
            continue;
        }
        let now = super::Moment::from_duration(origin.elapsed());
        // Bound transport timeout representations even for Moment's saturated
        // maximum, and never sleep past the lifecycle quantum so a stop is
        // serviced without real input. This never synthesizes an update.
        let ready = poll(wait_bound(rt.next_deadline(), now))?;
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
    use std::cell::Cell;
    use std::sync::{Arc, Mutex};

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
        // The first write is the shared cursor Show; its failure must not
        // skip line wrap, paste, mouse, alternate screen, raw mode or flush.
        let result = restore_terminal_with(&mut out, || {
            raw.set(true);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(raw.get());
        assert_eq!(out.calls, 5);
        assert_eq!(out.flushes, 5);
        let mut final_command = Vec::new();
        assert!(execute!(final_command, LeaveAlternateScreen).is_ok());
        assert!(out.bytes.ends_with(&final_command));
    }

    #[test]
    fn shared_cleanup_shows_cursor_before_restoring_modes() {
        let mut out = BrokenOutput {
            calls: 1,
            ..BrokenOutput::default()
        };
        assert!(
            restore_terminal_with(&mut out, || Err(io::Error::from(
                io::ErrorKind::PermissionDenied
            )))
            .err()
            .is_some_and(|error| error.kind() == io::ErrorKind::PermissionDenied)
        );
        let mut show_command = Vec::new();
        assert!(execute!(show_command, Show).is_ok());
        assert!(out.bytes.starts_with(&show_command));
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
    fn failed_restore_is_reported_and_retried_then_idempotent() {
        let mut left = false;
        let restored = Cell::new(0);
        let result = leave_with(&mut left, || {
            restored.set(restored.get() + 1);
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        });
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(!left);
        assert!(
            leave_with(&mut left, || {
                restored.set(restored.get() + 1);
                Ok(())
            })
            .is_ok()
        );
        assert!(left);
        assert_eq!(restored.get(), 2);
        assert!(leave_with(&mut left, || Err(io::Error::from(io::ErrorKind::Other))).is_ok());
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

    #[test]
    fn wait_bound_caps_at_the_lifecycle_quantum() {
        use super::super::Moment;
        let now = Moment::from_millis(100);
        assert_eq!(wait_bound(None, now), Duration::from_millis(100));
        assert_eq!(
            wait_bound(Some(Moment::from_millis(10_100)), now),
            Duration::from_millis(100)
        );
        assert_eq!(
            wait_bound(Some(Moment::from_millis(150)), now),
            Duration::from_millis(50)
        );
        assert_eq!(
            wait_bound(Some(Moment::from_millis(100)), now),
            Duration::ZERO
        );
        assert_eq!(
            wait_bound(Some(Moment::from_millis(99)), now),
            Duration::ZERO
        );
    }

    #[test]
    fn color_policy_detect_matches_terminal_and_requested_matches_level() {
        use crate::theme::ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};
        for authored in [
            Theme::junie(),
            Theme::paper(),
            Theme::junie().for_level(Mono),
        ] {
            assert_eq!(
                resolve_policy(&authored, TerminalColorPolicy::Detect),
                authored.for_terminal()
            );
            for requested in [TrueColor, Ansi256, Ansi16, Mono] {
                assert_eq!(
                    resolve_policy(&authored, TerminalColorPolicy::Requested(requested)),
                    authored.for_level(requested),
                    "Requested must call for_level without ambient detection"
                );
                // Requested never widens: an already-Mono theme stays Mono.
                if authored.capability.color == Mono {
                    assert_eq!(
                        resolve_policy(&authored, TerminalColorPolicy::Requested(requested))
                            .capability
                            .color,
                        Mono
                    );
                }
            }
        }
    }

    #[cfg(all(unix, feature = "crossterm"))]
    #[test]
    fn broker_registration_failures_retry_only_the_missing_one() {
        use std::sync::atomic::{AtomicBool as TestFlag, AtomicUsize};
        struct Script {
            fail_pending: TestFlag,
            fail_default: TestFlag,
            pending_calls: AtomicUsize,
            default_calls: AtomicUsize,
            pending_ptrs: Mutex<Vec<*const AtomicBool>>,
            default_ptrs: Mutex<Vec<*const AtomicBool>>,
        }
        impl Script {
            fn attempt(&self, broker: &mut SignalBroker) -> io::Result<()> {
                broker.complete_missing_with(
                    |flag| {
                        self.pending_calls.fetch_add(1, Ordering::SeqCst);
                        self.pending_ptrs
                            .lock()
                            .map_err(|_| io::Error::other("test script lock poisoned"))?
                            .push(Arc::as_ptr(flag));
                        if self.fail_pending.load(Ordering::SeqCst) {
                            return Err(io::Error::from(io::ErrorKind::Interrupted));
                        }
                        signal_hook::flag::register(signal_hook::consts::SIGTSTP, Arc::clone(flag))
                    },
                    |flag| {
                        self.default_calls.fetch_add(1, Ordering::SeqCst);
                        self.default_ptrs
                            .lock()
                            .map_err(|_| io::Error::other("test script lock poisoned"))?
                            .push(Arc::as_ptr(flag));
                        if self.fail_default.load(Ordering::SeqCst) {
                            return Err(io::Error::from(io::ErrorKind::Interrupted));
                        }
                        signal_hook::flag::register_conditional_default(
                            signal_hook::consts::SIGTSTP,
                            Arc::clone(flag),
                        )
                    },
                )
            }
        }
        let script = Script {
            fail_pending: TestFlag::new(true),
            fail_default: TestFlag::new(true),
            pending_calls: AtomicUsize::new(0),
            default_calls: AtomicUsize::new(0),
            pending_ptrs: Mutex::new(Vec::new()),
            default_ptrs: Mutex::new(Vec::new()),
        };
        let mut broker = SignalBroker::new();
        // First registration fails: nothing is installed.
        assert!(script.attempt(&mut broker).is_err());
        assert!(!broker.install.complete());
        assert_eq!(script.pending_calls.load(Ordering::SeqCst), 1);
        assert_eq!(script.default_calls.load(Ordering::SeqCst), 0);
        // First succeeds, second fails: the first success is retained.
        script.fail_pending.store(false, Ordering::SeqCst);
        assert!(script.attempt(&mut broker).is_err());
        assert!(broker.install.first.is_some());
        assert!(broker.install.second.is_none());
        // Retry registers only the missing second handler, with the same
        // flag identity; the first handler is never duplicated.
        script.fail_default.store(false, Ordering::SeqCst);
        assert!(script.attempt(&mut broker).is_ok());
        assert!(broker.install.complete());
        assert_eq!(script.pending_calls.load(Ordering::SeqCst), 2);
        assert_eq!(script.default_calls.load(Ordering::SeqCst), 2);
        let pending_ptrs = script.pending_ptrs.lock().unwrap();
        let default_ptrs = script.default_ptrs.lock().unwrap();
        assert_eq!(pending_ptrs.len(), 2);
        assert_eq!(pending_ptrs[0], pending_ptrs[1]);
        assert_eq!(default_ptrs.len(), 2);
        assert_eq!(default_ptrs[0], default_ptrs[1]);
        drop(pending_ptrs);
        drop(default_ptrs);
        // A completed install registers nothing more.
        assert!(script.attempt(&mut broker).is_ok());
        assert_eq!(script.pending_calls.load(Ordering::SeqCst), 2);
        assert_eq!(script.default_calls.load(Ordering::SeqCst), 2);
    }

    #[cfg(all(unix, feature = "crossterm"))]
    #[test]
    fn broker_lock_poison_maps_to_an_io_error() {
        let cell: &'static OnceLock<Mutex<SignalBroker>> = Box::leak(Box::new(OnceLock::new()));
        std::thread::scope(|scope| {
            let poisoned = scope.spawn(|| {
                let _held = lock_cell(cell).unwrap();
                panic!("poison the broker lock");
            });
            assert!(poisoned.join().is_err());
        });
        let error = lock_cell(cell).expect_err("poisoned lock must fail");
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(error.to_string().contains("poisoned"));
    }

    /// The single global-lease test: every phase runs sequentially so parallel
    /// test threads cannot interleave lease ownership.
    #[cfg(all(unix, feature = "crossterm"))]
    #[test]
    fn broker_singleton_serializes_entry_and_coalesces_stops() {
        broker_deactivate();
        assert_eq!(broker_snapshot_for_test(), (false, true, false));
        assert!(!broker_take_pending());
        // A stop before activation is recorded, then cleared by activation.
        broker_simulate_stop_delivery();
        assert_eq!(broker_snapshot_for_test(), (false, true, true));
        assert!(broker_activate().is_ok());
        assert_eq!(broker_snapshot_for_test(), (true, false, false));
        // An overlapping session is rejected while the lease is held.
        let overlap = broker_activate().expect_err("overlap must fail");
        assert_eq!(overlap.kind(), io::ErrorKind::AlreadyExists);
        // Repeated deliveries coalesce into one service.
        broker_simulate_stop_delivery();
        broker_simulate_stop_delivery();
        assert!(broker_take_pending());
        assert!(!broker_take_pending());
        assert_eq!(broker_snapshot_for_test(), (true, false, false));
        // Deactivation clears the flags and re-arms default-stop emulation.
        broker_deactivate();
        assert_eq!(broker_snapshot_for_test(), (false, true, false));
        // A new session reuses the completed install without registering again.
        let (first, second) = lock_cell(&SIGNAL_BROKER).map_or((None, None), |broker| {
            (broker.install.first, broker.install.second)
        });
        assert!(first.is_some() && second.is_some());
        assert!(broker_activate().is_ok());
        let (again_first, again_second) = lock_cell(&SIGNAL_BROKER)
            .map_or((None, None), |broker| {
                (broker.install.first, broker.install.second)
            });
        assert_eq!((first, second), (again_first, again_second));
        broker_deactivate();
        // Concurrent entry admits exactly one session.
        std::thread::scope(|scope| {
            let mut admitted = 0;
            let mut handles = Vec::new();
            for _ in 0..8 {
                handles.push(scope.spawn(broker_activate));
            }
            for handle in handles {
                match handle.join().expect("entry thread must finish") {
                    Ok(()) => admitted += 1,
                    Err(error) => assert_eq!(error.kind(), io::ErrorKind::AlreadyExists),
                }
            }
            assert_eq!(admitted, 1);
        });
        broker_deactivate();
        assert_eq!(broker_snapshot_for_test(), (false, true, false));
    }
}
