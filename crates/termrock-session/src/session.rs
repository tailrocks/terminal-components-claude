use std::io::{self, Stdout, Write, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::cursor::{Hide, SetCursorStyle, Show};
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event,
    poll, read,
};
use ratatui_crossterm::crossterm::style::Colored;
use ratatui_crossterm::crossterm::terminal::{
    EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui_crossterm::crossterm::{execute, queue};

use termrock_core::event::RoutedInput;
use termrock_core::keys::keyboard::KeyboardProtocol;
use termrock_runtime::{App, FeedbackClock, Moment, PreparedFrameError, Runtime, SizeWatch};
use termrock_terminal::{FLAGS, KeyboardNegotiation, POP};
use termrock_theme::Theme;

pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

const SIZE_CHECK_INTERVAL: Duration = Duration::from_millis(50);

/// Terminal caret shape used before the first rendered cursor request.
///
/// The default follows the frozen shell's terminal capability policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionCursorShape {
    /// Use the baseline bar shape only when the terminal advertises support.
    #[default]
    Auto,
    /// Use the terminal's configured default shape.
    Terminal,
    /// Use a steady block caret.
    Block,
    /// Use a steady underline caret.
    Underline,
    /// Use the baseline steady bar caret even when capability detection says unsupported.
    Bar,
}

impl SessionCursorShape {
    const fn command(self, caret_supported: bool) -> Option<SetCursorStyle> {
        match self {
            Self::Auto if caret_supported => Some(SetCursorStyle::SteadyBar),
            Self::Auto => None,
            Self::Terminal => Some(SetCursorStyle::DefaultUserShape),
            Self::Block => Some(SetCursorStyle::SteadyBlock),
            Self::Underline => Some(SetCursorStyle::SteadyUnderScore),
            Self::Bar => Some(SetCursorStyle::SteadyBar),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TerminalShapeSupport {
    caret: bool,
}

impl TerminalShapeSupport {
    fn current() -> Self {
        Self::from_env(|key| std::env::var(key).ok())
    }

    fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
        let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
        let program = get("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
        let modern = term.contains("kitty")
            || term.contains("ghostty")
            || term.starts_with("foot")
            || term.starts_with("wezterm")
            || program == "wezterm"
            || program == "ghostty"
            || get("KITTY_WINDOW_ID").is_some();
        let caret = modern
            || term.starts_with("xterm")
            || term.starts_with("tmux")
            || term.starts_with("screen")
            || term.starts_with("alacritty")
            || term.starts_with("rxvt-unicode")
            || term.starts_with("vte");
        Self { caret }
    }
}

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
    reset_cursor_shape: bool,
    keyboard: KeyboardNegotiation,
    keyboard_pushed: Arc<AtomicBool>,
}

impl core::fmt::Debug for TerminalSession {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TerminalSession")
            .field("left", &self.left)
            .finish_non_exhaustive()
    }
}

struct KeyboardWriteTracker<'a, W: Write> {
    out: &'a mut W,
    expected: &'a [u8],
    accepted: usize,
    pushed_after_write: bool,
    keyboard_pushed: &'a AtomicBool,
}

impl<'a, W: Write> KeyboardWriteTracker<'a, W> {
    fn new(
        out: &'a mut W,
        expected: &'a [u8],
        pushed_after_write: bool,
        keyboard_pushed: &'a AtomicBool,
    ) -> Self {
        Self {
            out,
            expected,
            accepted: 0,
            pushed_after_write,
            keyboard_pushed,
        }
    }
}

impl<W: Write> Write for KeyboardWriteTracker<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let remaining = &self.expected[self.accepted..];
        if !remaining.starts_with(bytes) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "keyboard writer received bytes outside the tracked protocol sequence",
            ));
        }
        let accepted = self.out.write(bytes)?;
        if accepted > bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "keyboard writer reported more accepted bytes than supplied",
            ));
        }
        self.accepted += accepted;
        if self.accepted == self.expected.len() {
            self.keyboard_pushed
                .store(self.pushed_after_write, Ordering::Relaxed);
        }
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

fn keyboard_enter_with(
    keyboard: &mut KeyboardNegotiation,
    out: &mut impl Write,
    keyboard_pushed: &AtomicBool,
    policy: KeyboardProtocol,
    detect: impl FnOnce() -> bool,
) -> io::Result<bool> {
    let sequence = termrock_core::keys::keyboard::push(FLAGS);
    let result = {
        let mut tracked =
            KeyboardWriteTracker::new(out, sequence.as_bytes(), true, keyboard_pushed);
        keyboard.enter(&mut tracked, policy, detect)
    };
    keyboard_pushed.store(keyboard.is_pushed(), Ordering::Relaxed);
    result
}

struct PreviouslyConsumedKeyboardPop;

impl Write for PreviouslyConsumedKeyboardPop {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A prior same-session cleanup accepted and recorded the POP already.
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn keyboard_leave_with(
    keyboard: &mut KeyboardNegotiation,
    out: &mut impl Write,
    keyboard_pushed: &AtomicBool,
) -> io::Result<bool> {
    if keyboard.is_pushed() && !keyboard_pushed.load(Ordering::Relaxed) {
        // The panic/entry cleanup hook can consume the owned frame without
        // access to this local value. Reconcile only that stale pushed bit;
        // the cleared mirror proves this session's POP was already accepted.
        let mut consumed = PreviouslyConsumedKeyboardPop;
        let _ = keyboard.leave(&mut consumed)?;
        return Ok(false);
    }

    let result = {
        let mut tracked = KeyboardWriteTracker::new(out, POP.as_bytes(), false, keyboard_pushed);
        keyboard.leave(&mut tracked)
    };
    keyboard_pushed.store(keyboard.is_pushed(), Ordering::Relaxed);
    result
}

fn keyboard_resume_with(
    keyboard: &mut KeyboardNegotiation,
    out: &mut impl Write,
    keyboard_pushed: &AtomicBool,
) -> io::Result<bool> {
    let sequence = termrock_core::keys::keyboard::push(FLAGS);
    let result = {
        let mut tracked =
            KeyboardWriteTracker::new(out, sequence.as_bytes(), true, keyboard_pushed);
        keyboard.resume(&mut tracked)
    };
    keyboard_pushed.store(keyboard.is_pushed(), Ordering::Relaxed);
    result
}

fn pop_owned_keyboard_with(out: &mut impl Write, keyboard_pushed: &AtomicBool) -> io::Result<()> {
    if !keyboard_pushed.load(Ordering::Relaxed) {
        return Ok(());
    }
    let mut tracked = KeyboardWriteTracker::new(out, POP.as_bytes(), false, keyboard_pushed);
    tracked.write_all(POP.as_bytes())?;
    tracked.flush()
}

fn restore_modes(
    reset_cursor_shape: bool,
    pop_keyboard: bool,
    keyboard_pushed: &AtomicBool,
) -> io::Result<()> {
    let mut out = stdout();
    restore_modes_with(
        &mut out,
        disable_raw_mode,
        reset_cursor_shape,
        pop_keyboard,
        keyboard_pushed,
    )
}

fn restore_modes_with(
    out: &mut impl Write,
    restore_raw: impl FnOnce() -> io::Result<()>,
    reset_cursor_shape: bool,
    pop_keyboard: bool,
    keyboard_pushed: &AtomicBool,
) -> io::Result<()> {
    let mut first = if pop_keyboard {
        // Reverse order of entry: consume only the flag set this session owns.
        pop_owned_keyboard_with(out, keyboard_pushed)
    } else {
        Ok(())
    };
    if reset_cursor_shape {
        first = first.and(execute!(out, SetCursorStyle::DefaultUserShape));
    }
    first = first.and(execute!(out, Show));
    first = first.and(execute!(out, EnableLineWrap));
    first = first.and(execute!(out, DisableBracketedPaste));
    first = first.and(execute!(out, DisableMouseCapture));
    first = first.and(execute!(out, LeaveAlternateScreen));
    first = first.and(restore_raw());
    first.and(out.flush())
}

fn enter_terminal_modes(
    out: &mut impl Write,
    cursor_shape: SessionCursorShape,
    caret_supported: bool,
    mark_cursor_shape_changed: impl FnOnce(),
) -> io::Result<()> {
    execute!(
        out,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    if let Some(command) = cursor_shape.command(caret_supported) {
        queue!(out, command)?;
        mark_cursor_shape_changed();
        out.flush()?;
    }
    execute!(out, Hide)
}

const fn rendered_cursor_command(
    cursor_bar: bool,
    caret_supported: bool,
) -> Option<SetCursorStyle> {
    if cursor_bar && caret_supported {
        Some(SetCursorStyle::SteadyBar)
    } else {
        None
    }
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
    leave_keyboard: impl FnOnce() -> io::Result<()>,
    show_cursor: impl FnOnce() -> io::Result<()>,
    restore: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    if *left {
        return Ok(());
    }
    let keyboard = leave_keyboard();
    let shown = show_cursor();
    let restored = restore();
    let result = keyboard.and(shown).and(restored);
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
        Self::enter_with_cursor_shape(SessionCursorShape::default())
    }

    pub fn enter_with_cursor_shape(cursor_shape: SessionCursorShape) -> io::Result<Self> {
        Self::enter_with_cursor_shape_support(cursor_shape, TerminalShapeSupport::current().caret)
    }

    /// Enter with kitty keyboard-protocol negotiation.
    ///
    /// Opt-in: the frozen apps never negotiate (the baseline captures
    /// contain no `CSI >` push), so [`KeyboardProtocol::Off`] — the
    /// behaviour of [`TerminalSession::enter`] — writes no protocol bytes.
    /// With `Auto` the `detect` probe decides; with
    /// [`KeyboardProtocol::On`] the flag set pushes unconditionally.
    pub fn enter_with_keyboard_protocol(
        protocol: KeyboardProtocol,
        detect: impl FnOnce() -> bool,
    ) -> io::Result<Self> {
        Self::enter_with_support(
            SessionCursorShape::default(),
            TerminalShapeSupport::current().caret,
            protocol,
            detect,
        )
    }

    fn enter_with_cursor_shape_support(
        cursor_shape: SessionCursorShape,
        caret_supported: bool,
    ) -> io::Result<Self> {
        Self::enter_with_support(cursor_shape, caret_supported, KeyboardProtocol::Off, || {
            false
        })
    }

    fn enter_with_support(
        cursor_shape: SessionCursorShape,
        caret_supported: bool,
        keyboard_protocol: KeyboardProtocol,
        keyboard_detect: impl FnOnce() -> bool,
    ) -> io::Result<Self> {
        let cursor_shape_changed = Arc::new(AtomicBool::new(false));
        let keyboard_pushed = Arc::new(AtomicBool::new(false));
        let panic_shape_changed = Arc::clone(&cursor_shape_changed);
        let panic_keyboard_pushed = Arc::clone(&keyboard_pushed);
        chain_panic_hook(move || {
            let _ = restore_modes(
                panic_shape_changed.load(Ordering::Relaxed),
                true,
                &panic_keyboard_pushed,
            );
        });
        let cleanup_shape_changed = Arc::clone(&cursor_shape_changed);
        let cleanup_keyboard_pushed = Arc::clone(&keyboard_pushed);
        let enter_shape_changed = Arc::clone(&cursor_shape_changed);
        let enter_keyboard_pushed = Arc::clone(&keyboard_pushed);
        let mut keyboard = KeyboardNegotiation::default();
        let terminal = enter_guarded(
            || {
                enable_raw_mode()?;
                let mut out = stdout();
                keyboard_enter_with(
                    &mut keyboard,
                    &mut out,
                    &enter_keyboard_pushed,
                    keyboard_protocol,
                    keyboard_detect,
                )?;
                enter_terminal_modes(&mut out, cursor_shape, caret_supported, || {
                    enter_shape_changed.store(true, Ordering::Relaxed);
                })?;
                Terminal::new(CrosstermBackend::new(out))
            },
            || {
                restore_modes(
                    cleanup_shape_changed.load(Ordering::Relaxed),
                    true,
                    &cleanup_keyboard_pushed,
                )
            },
        )?;
        Ok(TerminalSession {
            terminal,
            left: false,
            reset_cursor_shape: cursor_shape_changed.load(Ordering::Relaxed),
            keyboard,
            keyboard_pushed,
        })
    }

    pub const fn terminal(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }

    /// The session's keyboard-protocol negotiation state.
    pub const fn keyboard(&self) -> &KeyboardNegotiation {
        &self.keyboard
    }

    /// Suspend the protocol (pop) while an external program runs. A
    /// session that never pushed pops nothing and writes no bytes.
    pub fn suspend_keyboard(&mut self) -> io::Result<bool> {
        keyboard_leave_with(&mut self.keyboard, &mut stdout(), &self.keyboard_pushed)
    }

    /// Continue after [`TerminalSession::suspend_keyboard`] (push again).
    pub fn resume_keyboard(&mut self) -> io::Result<bool> {
        keyboard_resume_with(&mut self.keyboard, &mut stdout(), &self.keyboard_pushed)
    }

    pub fn leave(&mut self) -> io::Result<()> {
        let Self {
            left,
            terminal,
            reset_cursor_shape,
            keyboard,
            keyboard_pushed,
        } = self;
        leave_with(
            left,
            || keyboard_leave_with(keyboard, &mut stdout(), keyboard_pushed).map(|_| ()),
            || terminal.show_cursor(),
            || restore_modes(*reset_cursor_shape, false, keyboard_pushed),
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
pub fn run_with_feedback_clock_and_startup<A: App>(
    app: A,
    theme: Theme,
    clock: FeedbackClock,
    startup: StartupInput,
) -> io::Result<()> {
    run_with_cursor_shape_startup(app, theme, clock, startup, SessionCursorShape::default())
}

/// Run an application with an explicit initial terminal caret shape.
pub fn run_with_cursor_shape<A: App>(
    app: A,
    theme: Theme,
    cursor_shape: SessionCursorShape,
) -> io::Result<()> {
    run_with_cursor_shape_startup(
        app,
        theme,
        FeedbackClock::Elapsed,
        StartupInput::default(),
        cursor_shape,
    )
}

fn keyboard_protocol_for_app(app: &dyn App) -> KeyboardProtocol {
    app.keyboard_protocol()
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "session owns the supplied theme API and narrows capability once before runtime construction"
)]
fn run_with_cursor_shape_startup<A: App>(
    app: A,
    theme: Theme,
    clock: FeedbackClock,
    startup: StartupInput,
    cursor_shape: SessionCursorShape,
) -> io::Result<()> {
    // Install before any Crossterm poll/read. The process-wide reader binds
    // to stdin on first use and does not follow a later dup2.
    let bridged = crate::esc_bridge::install().is_ok();
    prepare_startup_input(startup, std::thread::sleep, || {
        if crate::esc_bridge::installed() {
            crate::esc_bridge::drain_real()
        } else {
            drain_pending_input()
        }
    });
    // Declared before `session`, so it drops after raw-mode restore and puts
    // the real tty's original termios back last.
    struct RestoreRealStdin;
    impl Drop for RestoreRealStdin {
        fn drop(&mut self) {
            crate::esc_bridge::restore_stdin();
        }
    }
    let _restore_real_stdin = RestoreRealStdin;
    let keyboard_protocol = keyboard_protocol_for_app(&app);
    let caret_supported = TerminalShapeSupport::current().caret;
    let mut session = TerminalSession::enter_with_support(
        cursor_shape,
        caret_supported,
        keyboard_protocol,
        || {
            if bridged {
                crate::esc_bridge::probe_keyboard_enhancement().unwrap_or(false)
            } else {
                // REVIEW: the unbridged path has no source-proven lossless raw-input handoff.
                // Keep the existing fallback until that ownership contract is admitted.
                ratatui_crossterm::crossterm::terminal::supports_keyboard_enhancement()
                    .unwrap_or(false)
            }
        },
    )?;
    // The bridge and configured startup-input drain are complete. Prime now,
    // while the terminal session owns the correct stdin binding, without reading.
    prime_input()?;
    // Propagate a failed size query; do not invent a 0×0 starting size.
    let initial_size = session.terminal().size()?;
    let mut size_watch = SizeWatch::new(Some((initial_size.width, initial_size.height)));
    let mut first_poll_delay = first_poll_delay_from_env();
    let origin = Instant::now();
    let mut rt = Runtime::new_with_feedback_clock(app, theme.for_terminal(), clock);
    let _ = rt.initialize();
    let suppress_colors = Colored::ansi_color_disabled_memoized();
    let mut pending = None;
    loop {
        let now = Moment::from_duration(origin.elapsed());
        advance_runtime_for_session(&mut rt, now, pending.is_some())?;
        loop {
            if rt.needs_settle() {
                let _ = rt.settle();
            }
            if rt.needs_present() {
                session.terminal().autoresize()?;
                let area = session.terminal().get_frame().area();
                let prepared = rt.prepare(area).map_err(io::Error::other)?;
                let mut bar = false;
                let mut area_changed = false;
                let result = session.terminal().try_draw(|frame| {
                    bar = rt
                        .draw_prepared(prepared, frame)
                        .map_err(|error| match error {
                            PreparedFrameError::AreaChanged { .. } => {
                                area_changed = true;
                                io::Error::other(error)
                            }
                            PreparedFrameError::Stale => io::Error::other(error),
                            PreparedFrameError::DidNotSettle => io::Error::other(error),
                        })?;
                    if suppress_colors {
                        reset_frame_colors(frame.buffer_mut());
                    }
                    Ok::<(), io::Error>(())
                });
                if let Err(error) = result {
                    if area_changed {
                        continue;
                    }
                    return Err(error);
                }
                rt.acknowledge_presented(prepared)
                    .map_err(io::Error::other)?;
                if let Some(command) = rendered_cursor_command(bar, caret_supported) {
                    execute!(stdout(), command)?;
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
            if let RoutedInput::Input(input) = &input {
                size_watch.observe(input);
            }
            match rt.handle_routed(input) {
                Ok(_) => {}
                Err(event) => pending = Some(event.into_routed_input()),
            }
            continue;
        }
        // Propagate query errors and preserve the previous known size.
        let actual_size = session.terminal().size()?;
        if let Some(resize) = size_watch.check((actual_size.width, actual_size.height)) {
            pending = Some(resize.into());
            continue;
        }
        // This one-shot delay follows a successful first presentation and the
        // initial size baseline, immediately before the first input wait.
        sleep_after_first_present(&mut first_poll_delay, std::thread::sleep);
        let now = Moment::from_duration(origin.elapsed());
        let ready = wait_for_session_input(&rt, now, |wait| {
            let ready = if bridged {
                crate::esc_bridge::wait_for_event(Some(wait))?
            } else {
                poll(wait)?
            };
            Ok(ready)
        })?;
        if ready {
            pending = decode_session_input(read()?);
        }
    }
    session.leave()
}

fn prime_input() -> io::Result<()> {
    prime_input_with(poll)
}

fn prime_input_with(poll_once: impl FnOnce(Duration) -> io::Result<bool>) -> io::Result<()> {
    match poll_once(Duration::ZERO) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(()),
        Err(error) => Err(error),
    }
}

fn first_poll_delay_from_env() -> Option<Duration> {
    std::env::var("JUNIE_TUI_FIRST_POLL_DELAY_MS")
        .ok()
        .and_then(|value| parse_first_poll_delay(Some(&value)))
}

fn parse_first_poll_delay(value: Option<&str>) -> Option<Duration> {
    value?.parse::<u64>().ok().map(Duration::from_millis)
}

fn sleep_after_first_present(delay: &mut Option<Duration>, sleep: impl FnOnce(Duration)) {
    if let Some(delay) = delay.take() {
        sleep(delay);
    }
}

fn input_wait_duration(deadline: Option<Duration>) -> Duration {
    deadline.map_or(SIZE_CHECK_INTERVAL, |remaining| {
        remaining.min(SIZE_CHECK_INTERVAL)
    })
}

fn wait_for_session_input<A: App>(
    runtime: &Runtime<A>,
    now: Moment,
    poll_once: impl FnOnce(Duration) -> io::Result<bool>,
) -> io::Result<bool> {
    let deadline = runtime
        .next_deadline()
        .map(|at| at.saturating_duration_since(now));
    poll_once(input_wait_duration(deadline))
}

fn advance_runtime_for_session<A: App>(
    runtime: &mut Runtime<A>,
    now: Moment,
    pending_input: bool,
) -> io::Result<()> {
    if pending_input
        || runtime
            .next_deadline()
            .is_some_and(|deadline| deadline <= now)
    {
        // Runtime::advance_to invalidates the presented frame when time changes.
        // A size-probe timeout with no input or due Runtime deadline is not an
        // application wakeup and must not cause a periodic redraw.
        runtime.advance_to(now).map_err(io::Error::other)?;
    }
    Ok(())
}

fn decode_session_input(event: Event) -> Option<RoutedInput> {
    RoutedInput::from_crossterm(event)
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui_core::layout::Rect;
    use std::cell::Cell;
    use std::sync::{Arc, Mutex};

    struct KeyboardPolicyProbe {
        policy: KeyboardProtocol,
        calls: std::cell::Cell<usize>,
    }

    impl App for KeyboardPolicyProbe {
        fn update(&mut self, _cx: &mut termrock_runtime::Cx<'_>) -> termrock_runtime::Response<()> {
            termrock_runtime::Response::ignored()
        }

        fn draw(&self, _ui: &mut termrock_runtime::Ui<'_>) {}

        fn keyboard_protocol(&self) -> KeyboardProtocol {
            self.calls.set(self.calls.get() + 1);
            self.policy
        }
    }

    #[test]
    fn app_policy_selection_uses_the_object_safe_hook_once() {
        for policy in [
            KeyboardProtocol::Auto,
            KeyboardProtocol::On,
            KeyboardProtocol::Off,
        ] {
            let app = KeyboardPolicyProbe {
                policy,
                calls: std::cell::Cell::new(0),
            };
            let erased_app: &dyn App = &app;

            assert_eq!(keyboard_protocol_for_app(erased_app), policy);
            assert_eq!(app.calls.get(), 1);
        }
    }

    struct SessionWaitProbe {
        repaint_after: Option<Duration>,
        updates: usize,
        ticks: usize,
        draws: Cell<usize>,
    }

    impl SessionWaitProbe {
        fn new(repaint_after: Option<Duration>) -> Self {
            Self {
                repaint_after,
                updates: 0,
                ticks: 0,
                draws: Cell::new(0),
            }
        }
    }

    impl App for SessionWaitProbe {
        fn update(&mut self, cx: &mut termrock_runtime::Cx<'_>) -> termrock_runtime::Response<()> {
            self.updates += 1;
            if cx.update_cause() == termrock_runtime::UpdateCause::Bootstrap {
                if let Some(delay) = self.repaint_after {
                    cx.request_repaint_after(delay);
                }
            } else if cx.update_cause() == termrock_runtime::UpdateCause::Tick {
                self.ticks += 1;
            }
            termrock_runtime::Response::ignored()
        }

        fn draw(&self, ui: &mut termrock_runtime::Ui<'_>) {
            self.draws.set(self.draws.get() + 1);
            ui.paint_str(ui.full(), "session wait probe", ui.surface_style());
        }
    }

    struct PresentedSessionProbe {
        runtime: Runtime<SessionWaitProbe>,
        terminal: Terminal<TestBackend>,
        area: Rect,
    }

    fn presented_session_probe(repaint_after: Option<Duration>) -> PresentedSessionProbe {
        let area = Rect::new(0, 0, 32, 4);
        let mut runtime = Runtime::new(SessionWaitProbe::new(repaint_after), Theme::junie());
        let _ = runtime.initialize();
        let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))
            .expect("test backend initializes");
        present_session_probe(&mut runtime, &mut terminal, area);
        assert!(
            runtime.app().draws.get() > 0,
            "App drew the initial test frame"
        );
        assert!(
            !runtime.needs_present(),
            "the initial TestBackend frame was acknowledged"
        );
        PresentedSessionProbe {
            runtime,
            terminal,
            area,
        }
    }

    fn present_session_probe(
        runtime: &mut Runtime<SessionWaitProbe>,
        terminal: &mut Terminal<TestBackend>,
        area: Rect,
    ) {
        for _ in 0..16 {
            if runtime.needs_settle() {
                let _ = runtime.settle();
            }
            if runtime.needs_present() {
                let prepared = runtime.prepare(area).expect("runtime prepares a frame");
                terminal
                    .draw(|frame| {
                        runtime
                            .draw_prepared(prepared, frame)
                            .expect("runtime draws the prepared frame");
                    })
                    .expect("test backend presents the frame");
                runtime
                    .acknowledge_presented(prepared)
                    .expect("test backend presentation is acknowledged");
            }
            if !runtime.needs_settle() && !runtime.needs_present() {
                return;
            }
        }
        panic!("session probe did not settle and present");
    }

    #[test]
    fn priming_polls_once_without_consuming_a_ready_event() {
        let queued = Cell::new(true);
        let calls = Cell::new(0);
        prime_input_with(|timeout| {
            assert_eq!(timeout, Duration::ZERO);
            calls.set(calls.get() + 1);
            Ok(queued.get())
        })
        .expect("zero-duration input prime");
        assert_eq!(calls.get(), 1);
        assert!(queued.get(), "priming must not read a queued event");
    }

    #[test]
    fn priming_tolerates_interrupts_and_preserves_other_errors() {
        assert!(prime_input_with(|_| Err(io::Error::from(io::ErrorKind::Interrupted))).is_ok());
        let error = prime_input_with(|_| Err(io::Error::from(io::ErrorKind::BrokenPipe)))
            .expect_err("non-interrupt poll error must propagate");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }

    #[test]
    fn first_poll_delay_parses_and_the_helper_takes_it_once() {
        assert_eq!(parse_first_poll_delay(None), None);
        assert_eq!(
            parse_first_poll_delay(Some("1500")),
            Some(Duration::from_millis(1500))
        );
        assert_eq!(parse_first_poll_delay(Some("invalid")), None);
        assert_eq!(parse_first_poll_delay(Some("-1")), None);

        let mut delay = Some(Duration::from_millis(17));
        let mut sleeps = Vec::new();
        sleep_after_first_present(&mut delay, |duration| sleeps.push(duration));
        sleep_after_first_present(&mut delay, |duration| sleeps.push(duration));
        assert_eq!(sleeps, vec![Duration::from_millis(17)]);
        assert_eq!(delay, None);
    }

    #[test]
    fn size_probe_wait_is_finite_without_deadlines_and_respects_real_deadlines() {
        assert_eq!(input_wait_duration(None), Duration::from_millis(50));
        assert_eq!(
            input_wait_duration(Some(Duration::from_millis(12))),
            Duration::from_millis(12)
        );
        assert_eq!(
            input_wait_duration(Some(Duration::from_secs(10))),
            Duration::from_millis(50)
        );
        assert_eq!(input_wait_duration(Some(Duration::ZERO)), Duration::ZERO);
    }

    #[test]
    fn unchanged_size_probe_timeouts_do_not_redraw_without_a_runtime_deadline() {
        let mut fixture = presented_session_probe(None);
        let initial_now = fixture.runtime.now();
        let initial_updates = fixture.runtime.app().updates;
        let initial_ticks = fixture.runtime.app().ticks;
        let initial_draws = fixture.runtime.app().draws.get();
        let mut elapsed = Duration::ZERO;
        let mut size_watch = SizeWatch::new(Some((32, 4)));

        assert_eq!(fixture.runtime.next_deadline(), None);
        for _ in 0..3 {
            let size = fixture.terminal.size().expect("test terminal size");
            assert_eq!(size_watch.check((size.width, size.height)), None);
            let now = Moment::from_duration(elapsed);
            let ready = wait_for_session_input(&fixture.runtime, now, |timeout| {
                assert_eq!(timeout, SIZE_CHECK_INTERVAL);
                elapsed = elapsed.saturating_add(timeout);
                Ok(false)
            })
            .expect("bounded session wait");
            assert!(!ready);
            advance_runtime_for_session(
                &mut fixture.runtime,
                Moment::from_duration(elapsed),
                false,
            )
            .expect("no due Runtime work");

            assert_eq!(
                fixture.runtime.now(),
                initial_now,
                "idle size probes do not advance Runtime"
            );
            assert_eq!(fixture.runtime.app().updates, initial_updates);
            assert_eq!(fixture.runtime.app().ticks, initial_ticks);
            assert_eq!(fixture.runtime.app().draws.get(), initial_draws);
            assert!(!fixture.runtime.needs_settle());
            assert!(!fixture.runtime.needs_present());
            assert_eq!(fixture.runtime.next_deadline(), None);
        }
    }

    #[test]
    fn session_wait_stops_at_and_dispatches_an_earlier_runtime_deadline() {
        let deadline = Duration::from_millis(12);
        let mut fixture = presented_session_probe(Some(deadline));
        let initial_updates = fixture.runtime.app().updates;
        let initial_draws = fixture.runtime.app().draws.get();
        let mut elapsed = Duration::ZERO;

        assert_eq!(
            fixture.runtime.next_deadline(),
            Some(Moment::from_millis(12))
        );
        let ready = wait_for_session_input(&fixture.runtime, Moment::ZERO, |timeout| {
            assert_eq!(timeout, deadline);
            elapsed = elapsed.saturating_add(timeout);
            Ok(false)
        })
        .expect("bounded wait reaches the real deadline");
        assert!(!ready);
        advance_runtime_for_session(&mut fixture.runtime, Moment::from_duration(elapsed), false)
            .expect("advance only because the deadline is due");

        assert_eq!(
            fixture.runtime.app().ticks,
            0,
            "Runtime records the due tick before settling it"
        );
        assert!(fixture.runtime.needs_settle());
        assert_eq!(
            fixture.runtime.next_deadline(),
            Some(Moment::from_millis(12))
        );
        present_session_probe(&mut fixture.runtime, &mut fixture.terminal, fixture.area);
        assert_eq!(fixture.runtime.app().ticks, 1);
        assert_eq!(fixture.runtime.app().updates, initial_updates + 1);
        assert!(fixture.runtime.app().draws.get() > initial_draws);
        assert_eq!(fixture.runtime.next_deadline(), None);
        assert!(!fixture.runtime.needs_settle());
        assert!(!fixture.runtime.needs_present());
    }

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
    fn converted_crossterm_keys_dispatch_with_only_reported_modifiers() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use ratatui::layout::Rect;
        use ratatui_crossterm::crossterm::event::{
            Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyEvent,
            KeyModifiers as CrosstermModifiers,
        };
        use std::sync::OnceLock;
        use termrock_core::action::ActionKey;
        use termrock_core::event::{Chord, KeyCode, KeyModifiers};
        use termrock_runtime::keymap::{KeyMap, KeyPhase};

        const X: ActionKey = ActionKey::custom("session.input.x");
        const UNICODE: ActionKey = ActionKey::custom("session.input.unicode");
        const UPPER: ActionKey = ActionKey::custom("session.input.upper");
        const ALT_X: ActionKey = ActionKey::custom("session.input.alt_x");

        struct DispatchProbe(Arc<Mutex<Vec<ActionKey>>>);

        impl App for DispatchProbe {
            fn update(
                &mut self,
                cx: &mut termrock_runtime::Cx<'_>,
            ) -> termrock_runtime::Response<()> {
                if let Some(command) = cx.command() {
                    self.0.lock().expect("command log").push(command);
                }
                termrock_runtime::Response::ignored()
            }

            fn draw(&self, _ui: &mut termrock_runtime::Ui<'_>) {}

            fn keymap(&self) -> &KeyMap {
                static KEYMAP: OnceLock<KeyMap> = OnceLock::new();
                KEYMAP.get_or_init(|| {
                    KeyMap::new()
                        .bind(KeyPhase::Capture, Chord::key(KeyCode::Char('x')), X)
                        .bind(KeyPhase::Capture, Chord::key(KeyCode::Char('é')), UNICODE)
                        .bind(KeyPhase::Capture, Chord::key(KeyCode::Char('X')), UPPER)
                        .bind(
                            KeyPhase::Capture,
                            Chord::with(KeyCode::Char('x'), KeyModifiers::ALT),
                            ALT_X,
                        )
                })
            }
        }

        fn present<A: App>(
            runtime: &mut Runtime<A>,
            terminal: &mut Terminal<TestBackend>,
            area: Rect,
        ) {
            for _ in 0..16 {
                if runtime.needs_settle() {
                    let _ = runtime.settle();
                }
                let token = runtime.prepare(area).expect("runtime prepares a frame");
                terminal
                    .draw(|frame| {
                        runtime
                            .draw_prepared(token, frame)
                            .expect("runtime draws prepared frame");
                    })
                    .expect("test backend presents frame");
                runtime
                    .acknowledge_presented(token)
                    .expect("test backend presentation is acknowledged");
                if !runtime.needs_settle() && !runtime.needs_present() {
                    return;
                }
            }
            panic!("runtime did not settle and present");
        }

        let commands = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = Runtime::new(DispatchProbe(Arc::clone(&commands)), Theme::junie());
        let _ = runtime.initialize();
        let area = Rect::new(0, 0, 40, 12);
        let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))
            .expect("test terminal initializes");
        present(&mut runtime, &mut terminal, area);

        let event = |code, modifiers| CrosstermEvent::Key(KeyEvent::new(code, modifiers));
        for key_event in [
            event(CrosstermKeyCode::Esc, CrosstermModifiers::NONE),
            event(CrosstermKeyCode::Char('x'), CrosstermModifiers::NONE),
            event(CrosstermKeyCode::Esc, CrosstermModifiers::NONE),
            event(CrosstermKeyCode::Char('é'), CrosstermModifiers::NONE),
            event(CrosstermKeyCode::Esc, CrosstermModifiers::NONE),
            event(CrosstermKeyCode::Char('X'), CrosstermModifiers::SHIFT),
            event(CrosstermKeyCode::Char('x'), CrosstermModifiers::ALT),
        ] {
            let input = super::decode_session_input(key_event).expect("key event converts");
            let _ = runtime
                .handle_routed(input)
                .expect("runtime dispatches converted input");
            present(&mut runtime, &mut terminal, area);
        }

        assert_eq!(
            *commands.lock().expect("command log"),
            vec![X, UNICODE, UPPER, ALT_X]
        );

        for event in [
            CrosstermEvent::Key(KeyEvent::new(
                CrosstermKeyCode::Esc,
                CrosstermModifiers::NONE,
            )),
            CrosstermEvent::Resize(40, 12),
            CrosstermEvent::Paste("typed paste".to_owned()),
        ] {
            let input = super::decode_session_input(event).expect("non-key event converts");
            let _ = runtime
                .handle_routed(input)
                .expect("runtime dispatches converted non-key input");
            present(&mut runtime, &mut terminal, area);
        }
        assert_eq!(
            *commands.lock().expect("command log"),
            vec![X, UNICODE, UPPER, ALT_X]
        );
    }

    #[test]
    fn private_session_decoder_keeps_middle_down_only() {
        use ratatui_core::layout::Position;
        use ratatui_crossterm::crossterm::event::{
            Event as CrosstermEvent, MouseButton, MouseEvent, MouseEventKind,
        };
        use termrock_core::event::{KeyModifiers, RoutedInput};

        let event = |kind| {
            CrosstermEvent::Mouse(MouseEvent {
                kind,
                column: 17,
                row: 9,
                modifiers: KeyModifiers::CONTROL.into(),
            })
        };
        assert_eq!(
            decode_session_input(event(MouseEventKind::Down(MouseButton::Middle))),
            Some(RoutedInput::MiddleClick {
                pos: Position::new(17, 9),
                mods: KeyModifiers::CONTROL,
            })
        );
        assert_eq!(
            decode_session_input(event(MouseEventKind::Drag(MouseButton::Middle))),
            None
        );
        assert_eq!(
            decode_session_input(event(MouseEventKind::Up(MouseButton::Middle))),
            None
        );
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

    struct FailOnSequence<'a> {
        sequence: &'a [u8],
        bytes: Vec<u8>,
    }

    impl Write for FailOnSequence<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes
                .windows(self.sequence.len())
                .any(|window| window == self.sequence)
            {
                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
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
        let result = restore_modes_with(
            &mut out,
            || {
                raw.set(true);
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            },
            false,
            false,
            &AtomicBool::new(false),
        );
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
        let result = restore_modes_with(
            &mut out,
            || Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            true,
            false,
            &AtomicBool::new(false),
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::PermissionDenied)
        );
        assert_eq!(out.flushes, 7);
    }

    struct FailShapeFlush {
        bytes: Vec<u8>,
        failed: bool,
    }

    impl Write for FailShapeFlush {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if !self.failed && self.bytes.windows(5).any(|window| window == b"\x1b[6 q") {
                self.failed = true;
                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
            }
            Ok(())
        }
    }

    #[test]
    fn entry_cleanup_resets_shape_after_emitted_command_flush_fails() {
        let shape_changed = Cell::new(false);
        let mut out = FailShapeFlush {
            bytes: Vec::new(),
            failed: false,
        };
        let mut cleanup = Vec::new();
        let result = enter_guarded(
            || {
                enter_terminal_modes(&mut out, SessionCursorShape::Auto, true, || {
                    shape_changed.set(true);
                })
            },
            || {
                restore_modes_with(
                    &mut cleanup,
                    || Ok(()),
                    shape_changed.get(),
                    false,
                    &AtomicBool::new(false),
                )
            },
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(out.bytes.windows(5).any(|window| window == b"\x1b[6 q"));
        assert!(shape_changed.get());
        let cleanup = String::from_utf8_lossy(&cleanup);
        assert!(
            cleanup.contains("\x1b[0 q"),
            "shape reset after flush failure"
        );
        assert!(cleanup.contains("\x1b[?25h"), "cursor shown after failure");
    }

    #[test]
    fn entry_cleanup_does_not_reset_shape_when_command_write_fails() {
        let shape_changed = Cell::new(false);
        let mut out = FailOnSequence {
            sequence: b"\x1b[6 q",
            bytes: Vec::new(),
        };
        let mut cleanup = Vec::new();
        let result = enter_guarded(
            || {
                enter_terminal_modes(&mut out, SessionCursorShape::Auto, true, || {
                    shape_changed.set(true);
                })
            },
            || {
                restore_modes_with(
                    &mut cleanup,
                    || Ok(()),
                    shape_changed.get(),
                    false,
                    &AtomicBool::new(false),
                )
            },
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::BrokenPipe)
        );
        assert!(!shape_changed.get());
        let cleanup = String::from_utf8_lossy(&cleanup);
        assert!(
            !cleanup.contains("\x1b[0 q"),
            "shape command was not emitted"
        );
        assert!(cleanup.contains("\x1b[?25h"), "cursor shown after failure");
    }

    #[test]
    fn partial_entry_resets_cursor_shape_only_after_shape_command_succeeds() {
        for (failed_sequence, expected_shape_change) in [
            (b"\x1b[?1049h".as_slice(), false),
            (b"\x1b[?25l".as_slice(), true),
        ] {
            let shape_changed = Cell::new(false);
            let mut out = FailOnSequence {
                sequence: failed_sequence,
                bytes: Vec::new(),
            };
            let result =
                enter_terminal_modes(&mut out, SessionCursorShape::default(), true, || {
                    shape_changed.set(true)
                });
            assert_eq!(
                result.err().map(|error| error.kind()),
                Some(io::ErrorKind::BrokenPipe)
            );
            assert_eq!(shape_changed.get(), expected_shape_change);

            let mut cleanup = Vec::new();
            assert!(
                restore_modes_with(
                    &mut cleanup,
                    || Ok(()),
                    shape_changed.get(),
                    false,
                    &AtomicBool::new(false),
                )
                .is_ok()
            );
            let cleanup = String::from_utf8_lossy(&cleanup);
            assert_eq!(cleanup.contains("\x1b[0 q"), expected_shape_change);
            assert!(cleanup.contains("\x1b[?25h"));
            assert!(cleanup.contains("\x1b[?7h"));
            assert!(cleanup.contains("\x1b[?1049l"));
        }
    }

    #[test]
    fn session_starts_with_bar_shape_even_while_cursor_is_hidden() {
        let mut out = Vec::new();
        let shape_changed = Cell::new(false);
        assert!(
            enter_terminal_modes(&mut out, SessionCursorShape::default(), true, || {
                shape_changed.set(true)
            },)
            .is_ok()
        );
        assert!(shape_changed.get());
        let bytes = String::from_utf8_lossy(&out);
        let steady_bar = bytes.find("\x1b[6 q").expect("steady bar sequence");
        let hide = bytes.find("\x1b[?25l").expect("hidden cursor sequence");
        assert!(
            steady_bar < hide,
            "shape is set before the cursor is hidden"
        );
    }

    #[test]
    fn session_cursor_shape_configuration_maps_to_terminal_shapes() {
        for (shape, sequence) in [
            (SessionCursorShape::Terminal, "\x1b[0 q"),
            (SessionCursorShape::Block, "\x1b[2 q"),
            (SessionCursorShape::Underline, "\x1b[4 q"),
            (SessionCursorShape::Bar, "\x1b[6 q"),
        ] {
            let mut out = Vec::new();
            let shape_changed = Cell::new(false);
            assert!(
                enter_terminal_modes(&mut out, shape, false, || shape_changed.set(true)).is_ok()
            );
            assert!(shape_changed.get());
            assert!(String::from_utf8_lossy(&out).contains(sequence));
        }
        assert_eq!(SessionCursorShape::default(), SessionCursorShape::Auto);
    }

    #[test]
    fn terminal_cleanup_restores_cursor_style_and_visibility() {
        let mut out = Vec::new();
        assert!(
            restore_modes_with(&mut out, || Ok(()), true, false, &AtomicBool::new(false),).is_ok()
        );
        let bytes = String::from_utf8_lossy(&out);
        assert!(bytes.contains("\x1b[0 q"), "cursor style reset");
        assert!(bytes.contains("\x1b[?25h"), "cursor shown");
    }

    #[test]
    fn automatic_shape_matches_frozen_terminal_capability_policy() {
        let env = |pairs: &[(&str, &str)]| {
            TerminalShapeSupport::from_env(|key| {
                pairs
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| (*value).to_owned())
            })
        };
        assert!(env(&[("TERM", "xterm-256color")]).caret);
        assert!(env(&[("TERM_PROGRAM", "WezTerm")]).caret);
        assert!(env(&[("TERM", "tmux-256color")]).caret);
        assert!(!env(&[("TERM", "linux")]).caret);
        assert!(!env(&[("TERM", "dumb")]).caret);
        assert!(!env(&[]).caret);
    }

    #[test]
    fn unsupported_automatic_shape_preserves_terminal_default_and_cleanup() {
        let mut enter = Vec::new();
        let shape_changed = Cell::new(false);
        assert!(
            enter_terminal_modes(&mut enter, SessionCursorShape::Auto, false, || {
                shape_changed.set(true);
            })
            .is_ok()
        );
        assert!(!shape_changed.get());
        assert!(!String::from_utf8_lossy(&enter).contains("\x1b[6 q"));

        let mut cleanup = Vec::new();
        assert!(
            restore_modes_with(
                &mut cleanup,
                || Ok(()),
                false,
                false,
                &AtomicBool::new(false),
            )
            .is_ok()
        );
        assert!(!String::from_utf8_lossy(&cleanup).contains("\x1b[0 q"));
        assert!(String::from_utf8_lossy(&cleanup).contains("\x1b[?25h"));
    }

    #[test]
    fn rendered_cursor_style_respects_terminal_capability() {
        assert_eq!(
            rendered_cursor_command(true, true),
            Some(SetCursorStyle::SteadyBar)
        );
        assert_eq!(rendered_cursor_command(true, false), None);
        assert_eq!(rendered_cursor_command(false, true), None);
    }

    #[test]
    fn failed_cursor_restore_still_restores_modes_and_allows_retry() {
        let mut left = false;
        let restored = Cell::new(0);
        let result = leave_with(
            &mut left,
            || Ok(()),
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
                || Ok(()),
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
            || Ok(()),
            || Err(io::Error::from(io::ErrorKind::PermissionDenied)),
        );
        assert_eq!(
            result.err().map(|error| error.kind()),
            Some(io::ErrorKind::PermissionDenied)
        );
        assert!(!left);
        assert!(leave_with(&mut left, || Ok(()), || Ok(()), || Ok(())).is_ok());
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

#[cfg(test)]
mod keyboard_tests {
    use super::*;
    use std::cell::Cell;

    /// A plain capturing writer.
    #[derive(Default)]
    struct Capture {
        bytes: Vec<u8>,
    }

    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn captured(out: &Capture) -> String {
        String::from_utf8(out.bytes.clone()).expect("plain CSI bytes")
    }

    #[test]
    fn the_default_restore_writes_no_protocol_bytes() {
        let mut out = Capture::default();
        restore_modes_with(&mut out, || Ok(()), false, false, &AtomicBool::new(false)).unwrap();
        assert!(
            !captured(&out).contains("\x1b[<u"),
            "the frozen cleanup pops nothing: {:?}",
            captured(&out)
        );
    }

    #[test]
    fn a_pushed_protocol_pops_first_on_restore() {
        let mut out = Capture::default();
        let keyboard_pushed = AtomicBool::new(true);
        restore_modes_with(&mut out, || Ok(()), false, true, &keyboard_pushed).unwrap();
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        let text = captured(&out);
        assert!(text.starts_with(termrock_terminal::POP), "{text:?}");
        assert!(
            text.contains("\x1b[?1049l"),
            "the alternate screen still leaves"
        );
    }

    #[test]
    fn enter_terminal_modes_is_byte_stable_without_a_push() {
        let mut out = Capture::default();
        enter_terminal_modes(&mut out, SessionCursorShape::default(), false, || {}).unwrap();
        let text = captured(&out);
        assert!(
            !text.contains("\x1b[>"),
            "no negotiation on the default path"
        );
        assert!(text.starts_with("\x1b[?1049h"), "{text:?}");
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ScriptedError(&'static str);

    impl std::fmt::Display for ScriptedError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl std::error::Error for ScriptedError {}

    #[derive(Default)]
    struct ScriptedOutput {
        bytes: Vec<u8>,
        fail_after: Option<usize>,
        flushes: usize,
        fail_flush_call: Option<usize>,
        panic_flush_call: Option<usize>,
    }

    impl ScriptedOutput {
        fn fail_after(accepted: usize) -> Self {
            Self {
                fail_after: Some(accepted),
                ..Self::default()
            }
        }

        fn fail_flush_call(call: usize) -> Self {
            Self {
                fail_flush_call: Some(call),
                ..Self::default()
            }
        }

        fn panic_flush_call(call: usize) -> Self {
            Self {
                panic_flush_call: Some(call),
                ..Self::default()
            }
        }
    }

    impl Write for ScriptedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Some(limit) = self.fail_after {
                if self.bytes.len() >= limit {
                    self.fail_after = None;
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        ScriptedError("scripted write failure"),
                    ));
                }
                let accepted = bytes.len().min(limit - self.bytes.len());
                self.bytes.extend_from_slice(&bytes[..accepted]);
                return Ok(accepted);
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.panic_flush_call == Some(self.flushes) {
                panic!("scripted flush panic");
            }
            if self.fail_flush_call == Some(self.flushes) {
                self.fail_flush_call = None;
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    ScriptedError("scripted flush failure"),
                ));
            }
            Ok(())
        }
    }

    fn assert_scripted_error(error: &io::Error, marker: &'static str) {
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(
            error
                .get_ref()
                .and_then(|source| source.downcast_ref::<ScriptedError>())
                .map(|source| source.0),
            Some(marker)
        );
    }

    fn count_sequence(bytes: &[u8], sequence: &[u8]) -> usize {
        bytes
            .windows(sequence.len())
            .filter(|window| *window == sequence)
            .count()
    }

    #[test]
    fn session_mirror_tracks_partial_suspend_and_resume_failures() {
        let mut keyboard = KeyboardNegotiation::default();
        let keyboard_pushed = AtomicBool::new(false);
        let mut enter = Capture::default();
        assert!(
            keyboard_enter_with(
                &mut keyboard,
                &mut enter,
                &keyboard_pushed,
                KeyboardProtocol::On,
                || false,
            )
            .unwrap()
        );
        assert!(keyboard.is_pushed());
        assert!(keyboard_pushed.load(Ordering::Relaxed));

        let pushed_state = keyboard;
        for accepted in 0..POP.len() {
            let mut partial_keyboard = pushed_state;
            let partial_mirror = AtomicBool::new(true);
            let mut partial_pop = ScriptedOutput::fail_after(accepted);
            let error =
                keyboard_leave_with(&mut partial_keyboard, &mut partial_pop, &partial_mirror)
                    .unwrap_err();
            assert_scripted_error(&error, "scripted write failure");
            assert_eq!(partial_pop.bytes.as_slice(), &POP.as_bytes()[..accepted]);
            assert!(partial_keyboard.is_pushed());
            assert!(partial_mirror.load(Ordering::Relaxed));
        }

        let mut complete_pop = Capture::default();
        assert!(keyboard_leave_with(&mut keyboard, &mut complete_pop, &keyboard_pushed,).unwrap());
        assert!(!keyboard.is_pushed());
        assert!(!keyboard_pushed.load(Ordering::Relaxed));

        let suspended_state = keyboard;
        let push = termrock_core::keys::keyboard::push(FLAGS);
        for accepted in 0..push.len() {
            let mut partial_keyboard = suspended_state;
            let partial_mirror = AtomicBool::new(false);
            let mut partial_push = ScriptedOutput::fail_after(accepted);
            let error =
                keyboard_resume_with(&mut partial_keyboard, &mut partial_push, &partial_mirror)
                    .unwrap_err();
            assert_scripted_error(&error, "scripted write failure");
            assert_eq!(partial_push.bytes.as_slice(), &push.as_bytes()[..accepted]);
            assert!(!partial_keyboard.is_pushed());
            assert!(!partial_mirror.load(Ordering::Relaxed));
        }

        let mut complete_push = Capture::default();
        assert!(
            keyboard_resume_with(&mut keyboard, &mut complete_push, &keyboard_pushed,).unwrap()
        );
        assert!(keyboard.is_pushed());
        assert!(keyboard_pushed.load(Ordering::Relaxed));
    }

    #[test]
    fn session_mirror_tracks_complete_suspend_and_resume_flush_errors() {
        let mut keyboard = KeyboardNegotiation::default();
        let keyboard_pushed = AtomicBool::new(false);
        let mut enter = Capture::default();
        assert!(
            keyboard_enter_with(
                &mut keyboard,
                &mut enter,
                &keyboard_pushed,
                KeyboardProtocol::On,
                || false,
            )
            .unwrap()
        );

        let mut pop = ScriptedOutput::fail_flush_call(1);
        let error = keyboard_leave_with(&mut keyboard, &mut pop, &keyboard_pushed).unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert_eq!(pop.bytes.as_slice(), POP.as_bytes());
        assert!(!keyboard.is_pushed());
        assert!(!keyboard_pushed.load(Ordering::Relaxed));

        let push = termrock_core::keys::keyboard::push(FLAGS);
        let mut resumed = ScriptedOutput::fail_flush_call(1);
        let error =
            keyboard_resume_with(&mut keyboard, &mut resumed, &keyboard_pushed).unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert_eq!(resumed.bytes, push.as_bytes());
        assert!(keyboard.is_pushed());
        assert!(keyboard_pushed.load(Ordering::Relaxed));
    }

    #[test]
    fn entry_cleanup_consumes_only_a_fully_written_push_and_keeps_entry_error() {
        let mut keyboard = KeyboardNegotiation::default();
        let keyboard_pushed = AtomicBool::new(false);
        let mut entry_output = ScriptedOutput::fail_flush_call(1);
        let mut cleanup_output = Capture::default();
        let ownership_seen_by_cleanup = Cell::new(false);
        let result = enter_guarded(
            || {
                keyboard_enter_with(
                    &mut keyboard,
                    &mut entry_output,
                    &keyboard_pushed,
                    KeyboardProtocol::On,
                    || false,
                )
                .map(|_| ())
            },
            || {
                ownership_seen_by_cleanup.set(keyboard_pushed.load(Ordering::Relaxed));
                restore_modes_with(
                    &mut cleanup_output,
                    || Ok(()),
                    false,
                    true,
                    &keyboard_pushed,
                )
            },
        );
        let error = result.unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert!(ownership_seen_by_cleanup.get());
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        let push = termrock_core::keys::keyboard::push(FLAGS);
        assert_eq!(count_sequence(&entry_output.bytes, push.as_bytes()), 1);
        assert_eq!(count_sequence(&cleanup_output.bytes, POP.as_bytes()), 1);
        restore_modes_with(
            &mut cleanup_output,
            || Ok(()),
            false,
            true,
            &keyboard_pushed,
        )
        .unwrap();
        assert_eq!(count_sequence(&cleanup_output.bytes, POP.as_bytes()), 1);
    }

    #[test]
    fn owned_pop_failure_preserves_first_error_and_attempts_every_restore() {
        let keyboard_pushed = AtomicBool::new(true);
        let raw_restores = Cell::new(0);
        let mut out = ScriptedOutput::fail_after(1);
        let result = restore_modes_with(
            &mut out,
            || {
                raw_restores.set(raw_restores.get() + 1);
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    ScriptedError("raw restore failure"),
                ))
            },
            false,
            true,
            &keyboard_pushed,
        );
        let error = result.unwrap_err();
        assert_scripted_error(&error, "scripted write failure");
        assert_eq!(raw_restores.get(), 1);
        assert!(keyboard_pushed.load(Ordering::Relaxed));
        assert_eq!(out.bytes.first(), Some(&POP.as_bytes()[0]));
        for sequence in [
            b"\x1b[?25h".as_slice(),
            b"\x1b[?7h".as_slice(),
            b"\x1b[?2004l".as_slice(),
            b"\x1b[?1006l".as_slice(),
            b"\x1b[?1003l".as_slice(),
            b"\x1b[?1049l".as_slice(),
        ] {
            assert!(
                out.bytes
                    .windows(sequence.len())
                    .any(|window| window == sequence)
            );
        }
        let mut retry = Capture::default();
        restore_modes_with(&mut retry, || Ok(()), false, true, &keyboard_pushed).unwrap();
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        assert_eq!(count_sequence(&retry.bytes, POP.as_bytes()), 1);
    }

    #[test]
    fn leave_orders_pop_before_cursor_and_restoration_and_does_not_pop_twice() {
        let mut keyboard = KeyboardNegotiation::default();
        let keyboard_pushed = AtomicBool::new(false);
        let mut enter = Capture::default();
        keyboard_enter_with(
            &mut keyboard,
            &mut enter,
            &keyboard_pushed,
            KeyboardProtocol::On,
            || false,
        )
        .unwrap();
        let mut pop_output = ScriptedOutput::fail_flush_call(1);
        let mut restore_output = Capture::default();
        let events = std::cell::RefCell::new(Vec::new());
        let mut left = false;
        let result = leave_with(
            &mut left,
            || {
                events.borrow_mut().push("keyboard");
                keyboard_leave_with(&mut keyboard, &mut pop_output, &keyboard_pushed).map(|_| ())
            },
            || {
                events.borrow_mut().push("cursor");
                Ok(())
            },
            || {
                events.borrow_mut().push("restore");
                restore_modes_with(
                    &mut restore_output,
                    || Ok(()),
                    false,
                    false,
                    &keyboard_pushed,
                )
            },
        );
        let error = result.unwrap_err();
        assert_scripted_error(&error, "scripted flush failure");
        assert_eq!(*events.borrow(), vec!["keyboard", "cursor", "restore"]);
        assert!(!left);
        assert!(!keyboard.is_pushed());
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        assert_eq!(count_sequence(&pop_output.bytes, POP.as_bytes()), 1);
        assert_eq!(count_sequence(&restore_output.bytes, POP.as_bytes()), 0);

        let mut retry_keyboard_output = Capture::default();
        let mut retry_restore_output = Capture::default();
        assert!(
            leave_with(
                &mut left,
                || {
                    keyboard_leave_with(&mut keyboard, &mut retry_keyboard_output, &keyboard_pushed)
                        .map(|_| ())
                },
                || Ok(()),
                || {
                    restore_modes_with(
                        &mut retry_restore_output,
                        || Ok(()),
                        false,
                        false,
                        &keyboard_pushed,
                    )
                },
            )
            .is_ok()
        );
        assert!(left);
        assert_eq!(
            count_sequence(&retry_keyboard_output.bytes, POP.as_bytes()),
            0
        );
        assert_eq!(
            count_sequence(&retry_restore_output.bytes, POP.as_bytes()),
            0
        );
    }

    #[test]
    fn panic_during_entry_flush_is_not_popped_again_by_session_leave() {
        let mut keyboard = KeyboardNegotiation::default();
        let keyboard_pushed = AtomicBool::new(false);
        let mut out = ScriptedOutput::panic_flush_call(1);
        let mut cleanup = Capture::default();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = enter_guarded(
                || {
                    keyboard_enter_with(
                        &mut keyboard,
                        &mut out,
                        &keyboard_pushed,
                        KeyboardProtocol::On,
                        || false,
                    )
                },
                || restore_modes_with(&mut cleanup, || Ok(()), false, true, &keyboard_pushed),
            );
        }));
        assert!(panic.is_err());
        assert!(
            keyboard.is_pushed(),
            "the hook cannot mutate the local copy"
        );
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        let push = termrock_core::keys::keyboard::push(FLAGS);
        assert_eq!(count_sequence(&out.bytes, push.as_bytes()), 1);
        assert_eq!(count_sequence(&cleanup.bytes, POP.as_bytes()), 1);

        let mut left = false;
        let mut leave_keyboard_output = Capture::default();
        let mut leave_restore_output = Capture::default();
        assert!(
            leave_with(
                &mut left,
                || {
                    keyboard_leave_with(&mut keyboard, &mut leave_keyboard_output, &keyboard_pushed)
                        .map(|_| ())
                },
                || Ok(()),
                || {
                    restore_modes_with(
                        &mut leave_restore_output,
                        || Ok(()),
                        false,
                        false,
                        &keyboard_pushed,
                    )
                },
            )
            .is_ok()
        );
        assert!(left);
        assert!(
            !keyboard.is_pushed(),
            "leave reconciles the local state after hook-owned POP"
        );
        assert!(!keyboard_pushed.load(Ordering::Relaxed));
        assert_eq!(
            count_sequence(&leave_keyboard_output.bytes, POP.as_bytes()),
            0
        );
        assert_eq!(
            count_sequence(&leave_restore_output.bytes, POP.as_bytes()),
            0
        );
        assert_eq!(count_sequence(&cleanup.bytes, POP.as_bytes()), 1);
    }

    #[test]
    fn owned_session_pop_preserves_a_caller_protocol_stack_frame() {
        let push = termrock_core::keys::keyboard::push(FLAGS);
        let mut out = Capture {
            bytes: push.as_bytes().to_vec(),
        };
        let keyboard_pushed = AtomicBool::new(false);
        let mut keyboard = KeyboardNegotiation::default();
        assert!(
            keyboard_enter_with(
                &mut keyboard,
                &mut out,
                &keyboard_pushed,
                KeyboardProtocol::On,
                || false,
            )
            .unwrap()
        );
        assert!(keyboard_leave_with(&mut keyboard, &mut out, &keyboard_pushed).unwrap());
        assert_eq!(
            out.bytes,
            [push.as_bytes(), push.as_bytes(), POP.as_bytes()].concat()
        );
        assert_eq!(count_sequence(&out.bytes, push.as_bytes()), 2);
        assert_eq!(count_sequence(&out.bytes, POP.as_bytes()), 1);
        assert!(!keyboard_pushed.load(Ordering::Relaxed));

        let mut no_owner_output = Capture {
            bytes: push.as_bytes().to_vec(),
        };
        pop_owned_keyboard_with(&mut no_owner_output, &keyboard_pushed).unwrap();
        assert_eq!(no_owner_output.bytes.as_slice(), push.as_bytes());
    }
}
