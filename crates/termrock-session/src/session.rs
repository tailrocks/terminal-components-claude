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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionCursorShape {
    #[default]
    Auto,

    Terminal,

    Block,

    Underline,

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
    pub fn from_environment() -> Self {
        Self {
            delay: std::env::var("JUNIE_TUI_START_DELAY_MS")
                .ok()
                .and_then(|value| value.parse().ok())
                .map(Duration::from_millis),
            discard_pending: true,
        }
    }

    #[must_use]
    pub const fn delay(mut self, delay: Option<Duration>) -> Self {
        self.delay = delay;
        self
    }

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

    pub const fn keyboard(&self) -> &KeyboardNegotiation {
        &self.keyboard
    }

    pub fn suspend_keyboard(&mut self) -> io::Result<bool> {
        keyboard_leave_with(&mut self.keyboard, &mut stdout(), &self.keyboard_pushed)
    }

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

pub fn run_with_startup<A: App>(app: A, theme: Theme, startup: StartupInput) -> io::Result<()> {
    run_with_feedback_clock_and_startup(app, theme, FeedbackClock::Elapsed, startup)
}

pub fn run_with_feedback_clock_and_startup<A: App>(
    app: A,
    theme: Theme,
    clock: FeedbackClock,
    startup: StartupInput,
) -> io::Result<()> {
    run_with_cursor_shape_startup(app, theme, clock, startup, SessionCursorShape::default())
}

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
    let bridged = crate::esc_bridge::install().is_ok();
    prepare_startup_input(startup, std::thread::sleep, || {
        if crate::esc_bridge::installed() {
            crate::esc_bridge::drain_real()
        } else {
            drain_pending_input()
        }
    });

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
                ratatui_crossterm::crossterm::terminal::supports_keyboard_enhancement()
                    .unwrap_or(false)
            }
        },
    )?;

    prime_input()?;

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

        let actual_size = session.terminal().size()?;
        if let Some(resize) = size_watch.check((actual_size.width, actual_size.height)) {
            pending = Some(resize.into());
            continue;
        }

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
            let event = read()?;
            if bridged {
                crate::esc_bridge::acknowledge_split_prefix_consumed(&event);
            }
            pending = decode_session_input(event);
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
