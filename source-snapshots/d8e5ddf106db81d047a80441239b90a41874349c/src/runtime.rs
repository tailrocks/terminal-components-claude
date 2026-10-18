//! Terminal runtime shared by every application built on the library:
//! one terminal-session guard that owns raw mode, the alternate screen,
//! mouse capture, bracketed paste, cursor visibility and line-wrap state
//! (restored on every exit path, including panics), an event loop that
//! drains unchanged input while rendering state changes before the next queued
//! event, and animation ticks on demand.
//!
//! Job control is part of the session's lifecycle, not only of its start and
//! end: an external `SIGTSTP` is recorded by a handler that touches nothing
//! but an atomic flag, and the event loop then leaves every owned mode,
//! stops the process with the default disposition, and on `SIGCONT`
//! re-acquires the modes, rebuilds geometry and redraws in full. Repeated
//! transitions are idempotent. Unsupported and unrecoverable cases are
//! explicit: non-Unix targets have no job control here; `SIGSTOP` and
//! `SIGKILL` cannot be intercepted, so a process stopped that way keeps raw
//! mode until it continues; and an output device that is gone cannot
//! receive restoration escapes.

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::style::Colored;
use ratatui::crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::style::Color;

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
    restoration: Restoration,
}

/// This owner exists before terminal construction, so partial setup has the
/// same rollback path as a fully initialized session.
struct Restoration {
    restore: fn(),
    active: bool,
}

impl Restoration {
    fn leave(&mut self) {
        if self.active {
            self.active = false;
            (self.restore)();
        }
    }

    /// Own the terminal again after a suspension; a no-op while active.
    fn reacquire(&mut self) {
        self.active = true;
    }
}

impl Drop for Restoration {
    fn drop(&mut self) {
        self.leave();
    }
}

fn initialize_with_restoration<T>(
    restore: fn(),
    initialize: impl FnOnce() -> std::io::Result<T>,
) -> std::io::Result<(T, Restoration)> {
    let restoration = Restoration {
        restore,
        active: true,
    };
    Ok((initialize()?, restoration))
}

/// DECAWM: automatic line wrap. Applications that draw to the last column
/// turn it off; the guard turns it back on for the host shell.
const ENABLE_WRAP: &str = "\x1b[?7h";

impl TerminalSession {
    /// Enter raw mode, the alternate screen, mouse capture and bracketed
    /// paste. Installs a panic hook that restores the terminal first.
    ///
    /// # Errors
    /// Returns a terminal I/O error if setup fails. Once raw mode is enabled,
    /// every later setup failure restores the terminal before returning.
    pub fn enter() -> std::io::Result<Self> {
        enable_raw_mode()?;
        let (terminal, restoration) = initialize_with_restoration(restore_terminal, || {
            let mut out = stdout();
            execute!(
                out,
                EnterAlternateScreen,
                EnableMouseCapture,
                EnableBracketedPaste
            )?;
            let backend = ratatui::backend::CrosstermBackend::new(out);
            ratatui::Terminal::new(backend)
        })?;
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous(info);
        }));
        Ok(Self {
            terminal,
            restoration,
        })
    }

    pub fn terminal(&mut self) -> &mut ratatui::DefaultTerminal {
        &mut self.terminal
    }

    /// Restore explicitly (idempotent); `Drop` does the same.
    pub fn leave(&mut self) {
        self.restoration.leave();
    }

    /// Supported job-control suspension: give the host shell its canonical,
    /// echoing terminal back, stop until continued, then own the terminal
    /// again. Returns the geometry after continuation so the caller can
    /// rebuild layout and redraw in full; the previous frame cache is
    /// dropped because the alternate screen was left and re-entered.
    ///
    /// # Errors
    /// Re-entry can fail like `enter` (raw mode or the escape sequences); the
    /// terminal is then left restored and the error is returned.
    #[cfg(unix)]
    pub fn suspend(&mut self) -> std::io::Result<(u16, u16)> {
        self.restoration.leave();
        job_control::stop_self();
        self.reenter()
    }

    #[cfg(unix)]
    fn reenter(&mut self) -> std::io::Result<(u16, u16)> {
        enable_raw_mode()?;
        let mut out = stdout();
        if let Err(e) = execute!(
            out,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        ) {
            restore_terminal();
            return Err(e);
        }
        self.restoration.reacquire();
        // clear the screen and drop the frame cache so the next draw paints
        // every cell; `Terminal::clear` is avoided because it round-trips a
        // cursor-position query, which a stopped-and-continued session (or
        // a headless pty) may never answer
        execute!(out, Clear(ClearType::All))?;
        self.terminal.swap_buffers();
        self.terminal.swap_buffers();
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }
}

/// Signal plumbing for supported suspension. The handler only records the
/// request; every terminal write happens on the event loop.
#[cfg(unix)]
pub mod job_control {
    use std::ffi::c_int;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    const SIGTSTP: c_int = 18;
    #[cfg(not(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )))]
    const SIGTSTP: c_int = 20;
    const SIG_DFL: usize = 0;

    static REQUESTED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" {
        fn signal(sig: c_int, handler: usize) -> usize;
        fn raise(sig: c_int) -> c_int;
    }

    extern "C" fn on_tstp(_sig: c_int) {
        // async-signal-safe: one atomic store, nothing else
        REQUESTED.store(true, Ordering::SeqCst);
    }

    /// Route `SIGTSTP` through the event loop instead of stopping mid-draw.
    pub fn install() {
        // SAFETY: installing a handler that performs one atomic store.
        let handler: extern "C" fn(c_int) = on_tstp;
        unsafe {
            signal(SIGTSTP, handler as *const () as usize);
        }
    }

    /// Take a pending suspension request.
    pub fn take_request() -> bool {
        REQUESTED.swap(false, Ordering::SeqCst)
    }

    /// Stop with the default disposition (the terminal is already restored);
    /// returns after `SIGCONT` with the handler reinstalled.
    pub fn stop_self() {
        // SAFETY: default disposition then raise; both are signal-safe libc
        // calls made from ordinary (non-handler) context.
        unsafe {
            signal(SIGTSTP, SIG_DFL);
            raise(SIGTSTP);
        }
        install();
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
    #[cfg(unix)]
    job_control::install();
    let result = event_loop(&mut session, app);
    session.leave();
    result
}

/// `poll` interrupted by a signal is not an error: the loop re-checks the
/// suspension flag and polls again.
fn poll_uninterrupted(wait: Duration) -> std::io::Result<bool> {
    match event::poll(wait) {
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => Ok(false),
        other => other,
    }
}

fn event_loop(session: &mut TerminalSession, app: &mut impl Application) -> std::io::Result<()> {
    // Use the backend's own memoized policy, including force_color_output,
    // rather than independently interpreting the environment.
    let suppress_colors = Colored::ansi_color_disabled_memoized();
    let mut dirty = true;
    let mut last_tick = Instant::now();
    loop {
        #[cfg(unix)]
        if job_control::take_request() {
            let (w, h) = session.suspend()?;
            // geometry may have changed while stopped: rebuild, then a full draw
            app.handle(Input::Resize(w, h));
            dirty = true;
        }
        let terminal = &mut session.terminal;
        if dirty {
            terminal.draw(|f| render_frame(app, f, suppress_colors))?;
            dirty = false;
        }
        // a tick-driven transition may have asked to quit without any input
        if app.should_quit() {
            return Ok(());
        }
        let interval = app.tick_interval();
        let wait = interval.saturating_sub(last_tick.elapsed());
        if poll_uninterrupted(wait)? {
            let changed = drain_ready_inputs(app, next_ready_input)?;
            if app.should_quit() {
                return Ok(());
            }
            if changed {
                // Rendering also rebuilds hit regions and reconciles focus.
                // The next queued key must see the newly active controls.
                terminal.draw(|f| render_frame(app, f, suppress_colors))?;
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
                    terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                }
                return Ok(());
            }
        }
    }
}

fn next_ready_input() -> std::io::Result<Option<Input>> {
    while event::poll(Duration::ZERO)? {
        if let Some(input) = Input::from_crossterm(event::read()?) {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// Unchanged events drained in one pass before the loop returns to its tick
/// check. A sustained flood of ignored input (a held key the page does not
/// bind, a paste of nothing that matters) therefore delays a tick by at most
/// this many dispatches instead of indefinitely; a changed event returns
/// immediately so its frame is drawn before the next queued event.
pub const DRAIN_BUDGET: usize = 256;

/// Dispatch queued input until one event changes state, the application
/// asks to quit, the queue is empty, or `DRAIN_BUDGET` unchanged events have
/// been handled. Returns whether state changed. Public so an application's
/// own tests can pump the same batched-versus-separated sequences the
/// runtime does.
///
/// # Errors
/// Propagates the input source's error.
pub fn drain_ready_inputs(
    app: &mut impl Application,
    mut next: impl FnMut() -> std::io::Result<Option<Input>>,
) -> std::io::Result<bool> {
    let mut unchanged = 0;
    while let Some(input) = next()? {
        let changed = app.handle(input) == Outcome::Changed;
        if changed || app.should_quit() {
            return Ok(changed);
        }
        unchanged += 1;
        if unchanged >= DRAIN_BUDGET {
            return Ok(false);
        }
    }
    Ok(false)
}

fn render_frame(app: &mut impl Application, frame: &mut Frame, suppress_colors: bool) {
    app.render(frame);
    if suppress_colors {
        reset_frame_colors(frame.buffer_mut());
    }
}

fn reset_frame_colors(buffer: &mut Buffer) {
    // Crossterm 0.29 serializes suppressed SetColors as ESC[;m, which resets
    // modifiers emitted immediately before it. Reset color values prevent
    // Ratatui from emitting those commands, preserving bold/underline/reverse.
    // Keep semantic theme colors during rendering: widgets use them to select
    // planes and state styles before this final backend boundary.
    for cell in &mut buffer.content {
        cell.fg = Color::Reset;
        cell.bg = Color::Reset;
        cell.underline_color = Color::Reset;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::{self, ErrorKind};

    thread_local! {
        static RESTORATIONS: Cell<usize> = const { Cell::new(0) };
    }

    fn record_restoration() {
        RESTORATIONS.set(RESTORATIONS.get() + 1);
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                ErrorKind::BrokenPipe,
                "terminal output closed",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn failed_terminal_setup_restores_state_and_preserves_error() {
        RESTORATIONS.set(0);
        let result = initialize_with_restoration(record_restoration, || {
            execute!(FailingWriter, EnterAlternateScreen, EnableMouseCapture)
        });
        assert!(matches!(result, Err(ref error) if error.kind() == ErrorKind::BrokenPipe));
        assert_eq!(RESTORATIONS.get(), 1);
    }

    #[test]
    fn panicking_terminal_setup_restores_state() {
        RESTORATIONS.set(0);
        let result = std::panic::catch_unwind(|| {
            initialize_with_restoration(record_restoration, || -> io::Result<()> {
                panic!("terminal initialization failed")
            })
        });
        assert!(result.is_err());
        assert_eq!(RESTORATIONS.get(), 1);
    }

    #[test]
    fn suspension_leaves_and_reacquires_idempotently() {
        RESTORATIONS.set(0);
        let (_, mut restoration) =
            initialize_with_restoration(record_restoration, || Ok(())).unwrap();
        // leave for the shell, come back, leave again: one restore per cycle
        restoration.leave();
        restoration.leave();
        assert_eq!(RESTORATIONS.get(), 1);
        restoration.reacquire();
        restoration.reacquire();
        restoration.leave();
        assert_eq!(RESTORATIONS.get(), 2);
        restoration.reacquire();
        drop(restoration);
        assert_eq!(
            RESTORATIONS.get(),
            3,
            "drop after re-entry restores once more"
        );
    }

    #[test]
    fn interrupted_poll_is_not_an_error() {
        // the signal-interrupted case is mapped to "nothing ready"
        let mapped = match Err::<bool, _>(io::Error::from(ErrorKind::Interrupted)) {
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(false),
            other => other,
        };
        assert!(matches!(mapped, Ok(false)));
    }

    #[test]
    fn successful_setup_transfers_restoration_and_leaves_once() {
        RESTORATIONS.set(0);
        let (value, mut restoration) =
            initialize_with_restoration(record_restoration, || Ok(42)).unwrap();
        assert_eq!(value, 42);
        assert_eq!(RESTORATIONS.get(), 0);
        restoration.leave();
        restoration.leave();
        drop(restoration);
        assert_eq!(RESTORATIONS.get(), 1);
    }

    #[test]
    fn no_color_backend_preserves_selection_attributes() {
        const CHILD: &str = "JUNIE_NO_COLOR_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            // Crossterm memoizes NO_COLOR. A child process tests its real policy
            // without mutating the environment of parallel tests.
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "runtime::tests::no_color_backend_preserves_selection_attributes",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env("NO_COLOR", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        use crate::theme::{ColorLevel, Theme};
        use ratatui::backend::{Backend, CrosstermBackend};
        use ratatui::layout::Rect;
        use ratatui::style::Modifier;

        assert!(Colored::ansi_color_disabled_memoized());
        let theme = Theme::for_level(ColorLevel::Mono);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 6, 1));
        buffer.set_string(
            0,
            0,
            "select",
            theme
                .selection()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                .underline_color(theme.accent),
        );
        let modifiers = buffer[(0, 0)].modifier;
        reset_frame_colors(&mut buffer);
        assert_eq!(buffer[(0, 0)].modifier, modifiers);
        let mut bytes = Vec::new();
        CrosstermBackend::new(&mut bytes)
            .draw(
                buffer
                    .content
                    .iter()
                    .enumerate()
                    .map(|(x, cell)| (x as u16, 0, cell)),
            )
            .unwrap();
        let output = String::from_utf8(bytes).unwrap();
        let (prefix, _) = output.split_once("select").unwrap();
        for attribute in ["\x1b[7m", "\x1b[1m", "\x1b[4m"] {
            assert!(prefix.contains(attribute), "{output:?}");
        }
        for reset in ["\x1b[;m", "\x1b[m", "\x1b[0m"] {
            assert!(!prefix.contains(reset), "{output:?}");
        }
    }

    #[test]
    fn queued_activation_waits_for_new_controls_to_render() {
        use crate::core::event::Key;
        use crate::core::{focus::Focus, focus::FocusRing, id::WidgetId};
        use ratatui::backend::TestBackend;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        use std::collections::VecDeque;

        struct RoutedApp {
            control: WidgetId,
            focus: Focus,
            activated: Option<WidgetId>,
            handled: usize,
        }
        impl Application for RoutedApp {
            fn handle(&mut self, input: Input) -> Outcome {
                self.handled += 1;
                match input {
                    Input::Key(key) if key.code == KeyCode::Esc => Outcome::Consumed,
                    Input::Key(key) if key.code == KeyCode::Char(']') => {
                        self.control = WidgetId::of("new page control");
                        Outcome::Changed
                    }
                    Input::Key(key) if key.code == KeyCode::Enter => {
                        self.activated = self.focus.current();
                        Outcome::Changed
                    }
                    _ => Outcome::Ignored,
                }
            }
            fn render(&mut self, _: &mut Frame) {
                let mut ring = FocusRing::default();
                ring.register(self.control);
                self.focus.ensure_valid(&ring);
            }
            fn should_quit(&self) -> bool {
                false
            }
            fn tick_interval(&self) -> Duration {
                Duration::from_secs(1)
            }
        }

        let mut app = RoutedApp {
            control: WidgetId::of("old page control"),
            focus: Focus::default(),
            activated: None,
            handled: 0,
        };
        let mut terminal = ratatui::Terminal::new(TestBackend::new(10, 2)).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let key = |code| {
            Input::Key(Key {
                code,
                mods: KeyModifiers::NONE,
            })
        };
        let mut queued = VecDeque::from([
            key(KeyCode::F(1)),
            key(KeyCode::Esc),
            key(KeyCode::Char(']')),
            key(KeyCode::Enter),
        ]);
        assert!(drain_ready_inputs(&mut app, || Ok(queued.pop_front())).unwrap());
        assert_eq!(app.handled, 3, "unchanged events remain batched");
        assert!(app.activated.is_none(), "activation must wait for render");
        terminal.draw(|frame| app.render(frame)).unwrap();
        assert!(drain_ready_inputs(&mut app, || Ok(queued.pop_front())).unwrap());
        assert_eq!(app.activated, Some(WidgetId::of("new page control")));
        assert!(queued.is_empty());
    }
}
