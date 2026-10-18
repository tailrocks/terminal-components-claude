use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, Event, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::style::Colored;
use ratatui::crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::style::Color;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{Input, InputConfig, Key, Mouse, MouseKind, Outcome};
use crate::core::keyboard::{self, KeyboardProtocol};

pub trait Application {
    fn handle(&mut self, input: Input) -> Outcome;
    fn render(&mut self, frame: &mut Frame);
    fn should_quit(&self) -> bool;

    fn tick_interval(&self) -> Duration;

    fn coalesce(&self) -> Coalesce {
        Coalesce::default()
    }

    fn shapes(&self) -> crate::core::shape::Shapes {
        crate::core::shape::Shapes::default()
    }

    fn input_config(&self) -> InputConfig {
        InputConfig::default()
    }

    fn keyboard_protocol(&self) -> KeyboardProtocol {
        KeyboardProtocol::current()
    }

    fn take_output(&mut self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coalesce {
    pub wheel: bool,

    pub pointer_moves: bool,

    pub key_repeat: bool,

    pub resize: bool,

    pub max_batch: usize,

    pub max_latency: Duration,
}

impl Coalesce {
    pub const OFF: Self = Self {
        wheel: false,
        pointer_moves: false,
        key_repeat: false,
        resize: false,
        max_batch: 1,
        max_latency: Duration::ZERO,
    };

    pub fn folds(&self, last: &Input, next: &Input) -> bool {
        match (last, next) {
            (Input::Mouse(a), Input::Mouse(b)) => {
                (self.wheel && a.wheel().is_some() && b.wheel().is_some() && a.mods == b.mods)
                    || (self.pointer_moves
                        && a.kind == MouseKind::Move
                        && b.kind == MouseKind::Move)
            }
            (Input::Key(a), Input::Key(b)) => self.key_repeat && a == b && is_navigation(a),
            (Input::Resize(..), Input::Resize(..)) => self.resize,
            _ => false,
        }
    }
}

impl Default for Coalesce {
    fn default() -> Self {
        Self {
            wheel: true,
            pointer_moves: true,
            key_repeat: true,
            resize: true,
            max_batch: 64,
            max_latency: Duration::from_millis(16),
        }
    }
}

fn is_navigation(key: &Key) -> bool {
    matches!(
        key.code,
        KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End
    ) && key
        .mods
        .difference(KeyModifiers::SHIFT | KeyModifiers::CONTROL)
        .is_empty()
}

pub struct TerminalSession {
    terminal: ratatui::DefaultTerminal,
    restoration: Restoration,
}

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

const ENABLE_WRAP: &str = "\x1b[?7h";

impl TerminalSession {
    pub fn enter() -> std::io::Result<Self> {
        Self::enter_with(KeyboardProtocol::current())
    }

    pub fn enter_with(protocol: KeyboardProtocol) -> std::io::Result<Self> {
        enable_raw_mode()?;
        let (terminal, restoration) = initialize_with_restoration(restore_terminal, || {
            prime_input()?;
            let mut out = stdout();
            execute!(
                out,
                EnterAlternateScreen,
                EnableMouseCapture,
                EnableBracketedPaste,
                EnableFocusChange
            )?;

            if let Some(push) = keyboard::session::enter(protocol, || {
                ratatui::crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false)
            }) {
                out.write_all(push.as_bytes())?;
                out.flush()?;
            }
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

    pub fn leave(&mut self) {
        self.restoration.leave();
    }

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
            EnableBracketedPaste,
            EnableFocusChange
        ) {
            restore_terminal();
            return Err(e);
        }
        if let Some(push) = keyboard::session::resume()
            && let Err(e) = out.write_all(push.as_bytes()).and_then(|()| out.flush())
        {
            restore_terminal();
            return Err(e);
        }
        self.restoration.reacquire();

        execute!(out, Clear(ClearType::All))?;
        self.terminal.swap_buffers();
        self.terminal.swap_buffers();
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }
}

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
        REQUESTED.store(true, Ordering::SeqCst);
    }

    pub fn install() {
        let handler: extern "C" fn(c_int) = on_tstp;
        unsafe {
            signal(SIGTSTP, handler as *const () as usize);
        }
    }

    pub fn take_request() -> bool {
        REQUESTED.swap(false, Ordering::SeqCst)
    }

    pub fn stop_self() {
        unsafe {
            signal(SIGTSTP, SIG_DFL);
            raise(SIGTSTP);
        }
        install();
    }
}

fn restore_terminal() {
    let mut out = stdout();

    let _ = out.write_all(crate::core::shape::reset_sequence().as_bytes());
    if let Some(pop) = keyboard::session::leave() {
        let _ = out.write_all(pop.as_bytes());
    }
    let _ = execute!(
        out,
        DisableFocusChange,
        DisableBracketedPaste,
        DisableMouseCapture,
        Show
    );
    let _ = out.write_all(ENABLE_WRAP.as_bytes());
    let _ = execute!(out, LeaveAlternateScreen);
    let _ = out.flush();
    let _ = disable_raw_mode();
}

pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    let mut session = TerminalSession::enter_with(app.keyboard_protocol())?;
    #[cfg(unix)]
    job_control::install();
    let result = event_loop(&mut session, app);
    session.leave();
    result
}

fn prime_input() -> std::io::Result<()> {
    poll_uninterrupted(Duration::ZERO).map(|_| ())
}

fn first_poll_delay() -> Option<Duration> {
    std::env::var("JUNIE_TUI_FIRST_POLL_DELAY_MS")
        .ok()?
        .parse()
        .ok()
        .map(Duration::from_millis)
}

fn poll_uninterrupted(wait: Duration) -> std::io::Result<bool> {
    match poll_input(wait) {
        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => Ok(false),
        other => other,
    }
}

fn poll_input(wait: Duration) -> std::io::Result<bool> {
    if event::poll(Duration::ZERO)? {
        return Ok(true);
    }
    #[cfg(unix)]
    if tty_input::rearm() && event::poll(Duration::ZERO)? {
        return Ok(true);
    }
    if wait.is_zero() {
        return Ok(false);
    }
    event::poll(wait)
}

#[cfg(unix)]
mod tty_input {
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, RawFd};
    use std::sync::OnceLock;

    fn fd() -> Option<RawFd> {
        static TTY: OnceLock<Option<(RawFd, Option<std::fs::File>)>> = OnceLock::new();
        TTY.get_or_init(|| {
            if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
                return Some((libc::STDIN_FILENO, None));
            }
            let tty = std::fs::File::open("/dev/tty").ok()?;
            Some((tty.as_raw_fd(), Some(tty)))
        })
        .as_ref()
        .map(|(fd, _)| *fd)
    }

    fn queued(fd: RawFd) -> bool {
        let mut p = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        unsafe { libc::poll(&mut p, 1, 0) > 0 && p.revents & libc::POLLIN != 0 }
    }

    pub fn rearm() -> bool {
        let Some(fd) = fd() else {
            return false;
        };
        if !queued(fd) {
            return false;
        }

        unsafe {
            if libc::tcgetpgrp(fd) != libc::getpgrp() {
                return false;
            }
            let mut attrs = MaybeUninit::<libc::termios>::uninit();
            if libc::tcgetattr(fd, attrs.as_mut_ptr()) != 0 {
                return false;
            }
            libc::tcsetattr(fd, libc::TCSANOW, attrs.as_ptr()) == 0
        }
    }
}

fn event_loop(session: &mut TerminalSession, app: &mut impl Application) -> std::io::Result<()> {
    let suppress_colors = Colored::ansi_color_disabled_memoized();
    let mut dirty = true;
    let mut last_tick = Instant::now();
    let mut pump = InputPump::new(app.coalesce());

    let mut shapes =
        crate::core::shape::ShapeSync::new(crate::core::shape::ShapeSupport::current());
    let mut hover_due: Option<(u64, Instant)> = None;
    let mut first_poll_delay = first_poll_delay();
    let mut sizes = SizeWatch::new(terminal_size(&session.terminal));
    loop {
        #[cfg(unix)]
        if job_control::take_request() {
            let (w, h) = session.suspend()?;

            shapes.forget();

            let resize = Input::Resize(w, h);
            sizes.observe(&resize);
            app.handle(resize);
            dirty = true;
        }
        let terminal = &mut session.terminal;
        if dirty {
            terminal.draw(|f| render_frame(app, f, suppress_colors))?;
            dirty = false;
            if let Some(delay) = first_poll_delay.take() {
                std::thread::sleep(delay);
            }
        }

        if shapes.apply(app.shapes(), terminal.backend_mut())? {
            terminal.backend_mut().flush()?;
        }

        let output = app.take_output();
        if !output.is_empty() {
            for seq in output {
                terminal.backend_mut().write_all(seq.as_bytes())?;
            }
            terminal.backend_mut().flush()?;
        }

        if app.should_quit() {
            return Ok(());
        }
        let interval = app.tick_interval();
        let mut wait = interval.saturating_sub(last_tick.elapsed());

        let now = Instant::now();
        hover_due = match (crate::ui::reveal::hover_wait(), hover_due) {
            (None, _) => None,
            (Some((frame, _)), Some((seen, due))) if frame == seen => Some((seen, due)),
            (Some((frame, ms)), _) => Some((frame, now + Duration::from_millis(ms))),
        };
        if let Some((_, due)) = hover_due {
            wait = wait.min(due.saturating_duration_since(now));
        }

        if pump.has_pending() || poll_uninterrupted(wait)? {
            pump.set_policy(app.coalesce());
            let config = effective_input_config(app.input_config(), keyboard::active());
            let changed = pump.drain(app, || {
                let input = next_ready_input(config)?;
                if let Some(input) = &input {
                    sizes.observe(input);
                }
                Ok(input)
            })?;
            if app.should_quit() {
                return Ok(());
            }
            if changed {
                terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                if pump.settle(app) {
                    terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                }
            }
        }
        if hover_due.is_some_and(|(_, due)| due <= now + wait) {
            dirty = true;
        }

        if let Some(size) = terminal_size(terminal)
            && let Some(resize) = sizes.check(size)
            && app.handle(resize) == Outcome::Changed
        {
            dirty = true;
        }
        if last_tick.elapsed() >= interval {
            last_tick = Instant::now();
            if app.handle(Input::Tick) == Outcome::Changed {
                dirty = true;
            }
            if app.should_quit() {
                if dirty {
                    terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                }
                return Ok(());
            }
        }
    }
}

fn terminal_size(terminal: &ratatui::DefaultTerminal) -> Option<(u16, u16)> {
    terminal.size().ok().map(|s| (s.width, s.height))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeWatch {
    known: Option<(u16, u16)>,
}

impl SizeWatch {
    pub fn new(size: Option<(u16, u16)>) -> Self {
        Self { known: size }
    }

    pub fn observe(&mut self, input: &Input) {
        if let Input::Resize(w, h) = input {
            self.known = Some((*w, *h));
        }
    }

    pub fn check(&mut self, actual: (u16, u16)) -> Option<Input> {
        if self.known == Some(actual) {
            return None;
        }
        self.known = Some(actual);
        Some(Input::Resize(actual.0, actual.1))
    }
}

fn effective_input_config(app: InputConfig, protocol_active: bool) -> InputConfig {
    InputConfig {
        legacy_ctrl_punctuation: app.legacy_ctrl_punctuation && !protocol_active,
        ..app
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusClickGuard {
    pub window: Duration,
    gained_at: Option<Instant>,
    swallowing: bool,
}

impl FocusClickGuard {
    pub const DEFAULT_WINDOW: Duration = Duration::from_millis(300);

    pub const fn new(window: Duration) -> Self {
        Self {
            window,
            gained_at: None,
            swallowing: false,
        }
    }

    pub fn drops(&mut self, ev: &Event, now: Instant) -> bool {
        match ev {
            Event::FocusGained => {
                self.gained_at = Some(now);
                true
            }
            Event::FocusLost => {
                self.gained_at = None;
                true
            }
            Event::Mouse(m) => match m.kind {
                MouseEventKind::Down(_) => {
                    let fresh = self
                        .gained_at
                        .take()
                        .is_some_and(|at| now.saturating_duration_since(at) < self.window);
                    self.swallowing = fresh;
                    fresh
                }
                MouseEventKind::Drag(_) => self.swallowing,
                MouseEventKind::Up(_) => std::mem::take(&mut self.swallowing),
                _ => false,
            },
            _ => false,
        }
    }
}

impl Default for FocusClickGuard {
    fn default() -> Self {
        Self::new(Self::DEFAULT_WINDOW)
    }
}

thread_local! {
    static FOCUS_GUARD: std::cell::Cell<FocusClickGuard> =
        std::cell::Cell::new(FocusClickGuard::default());
}

pub fn set_focus_click_guard(window: Duration) {
    FOCUS_GUARD.with(|g| g.set(FocusClickGuard::new(window)));
}

fn next_ready_input(config: InputConfig) -> std::io::Result<Option<Input>> {
    while poll_input(Duration::ZERO)? {
        let ev = event::read()?;
        let dropped = FOCUS_GUARD.with(|g| {
            let mut guard = g.get();
            let d = guard.drops(&ev, Instant::now());
            g.set(guard);
            d
        });
        if dropped {
            continue;
        }
        if let Some(input) = Input::from_crossterm_with(ev, config) {
            keyboard::observe(&input);
            return Ok(Some(input));
        }
    }
    Ok(None)
}

pub const DRAIN_BUDGET: usize = 256;

pub fn drain_ready_inputs(
    app: &mut impl Application,
    next: impl FnMut() -> std::io::Result<Option<Input>>,
) -> std::io::Result<bool> {
    InputPump::new(Coalesce::OFF).drain(app, next)
}

#[derive(Debug, Clone, Default)]
pub struct InputPump {
    policy: Coalesce,

    pending: Option<Input>,

    settle: Option<Mouse>,
}

impl InputPump {
    pub fn new(policy: Coalesce) -> Self {
        Self {
            policy,
            pending: None,
            settle: None,
        }
    }

    pub fn set_policy(&mut self, policy: Coalesce) {
        self.policy = policy;
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn drain(
        &mut self,
        app: &mut impl Application,
        mut next: impl FnMut() -> std::io::Result<Option<Input>>,
    ) -> std::io::Result<bool> {
        let started = Instant::now();
        let mut unchanged = 0;

        let mut burst: Option<(Input, usize)> = None;
        self.settle = None;
        loop {
            if let Some((last, folded)) = &burst
                && (!self.policy.folds(last, last)
                    || *folded >= self.policy.max_batch
                    || started.elapsed() >= self.policy.max_latency)
            {
                break;
            }
            let input = match self.pending.take() {
                Some(input) => input,
                None => match next()? {
                    Some(input) => input,
                    None => break,
                },
            };
            if let Some((last, _)) = &burst
                && !self.policy.folds(last, &input)
            {
                self.pending = Some(input);
                break;
            }

            let changed = crate::ui::reveal::on_pointer_wheel(&input)
                .or_else(|| crate::ui::tooltip::on_input(&input))
                .unwrap_or_else(|| app.handle(input.clone()))
                == Outcome::Changed;
            if app.should_quit() {
                return Ok(changed || burst.is_some());
            }
            match &mut burst {
                Some((last, folded)) => {
                    *last = input;
                    *folded += 1;
                }
                None if changed => burst = Some((input, 1)),
                None => {
                    unchanged += 1;
                    if unchanged >= DRAIN_BUDGET {
                        return Ok(false);
                    }
                }
            }
        }
        if let Some((Input::Mouse(m), folded)) = &burst
            && m.kind == MouseKind::Move
            && *folded > 1
        {
            self.settle = Some(*m);
        }
        Ok(burst.is_some())
    }

    pub fn settle(&mut self, app: &mut impl Application) -> bool {
        match self.settle.take() {
            Some(m) => app.handle(Input::Mouse(m)) == Outcome::Changed,
            None => false,
        }
    }
}

fn render_frame(app: &mut impl Application, frame: &mut Frame, suppress_colors: bool) {
    app.render(frame);
    if suppress_colors {
        reset_frame_colors(frame.buffer_mut());
    }
}

fn reset_frame_colors(buffer: &mut Buffer) {
    for cell in &mut buffer.content {
        cell.fg = Color::Reset;
        cell.bg = Color::Reset;
        cell.underline_color = Color::Reset;
    }
}

pub fn drain_pending_input() -> std::io::Result<usize> {
    let was_raw = ratatui::crossterm::terminal::is_raw_mode_enabled()?;
    if !was_raw {
        enable_raw_mode()?;
    }
    let mut n = 0;
    let drained = loop {
        match poll_input(Duration::ZERO) {
            Ok(false) => break Ok(n),
            Ok(true) => match event::read() {
                Ok(_) => n += 1,
                Err(e) => break Err(e),
            },
            Err(e) => break Err(e),
        }
    };
    if !was_raw {
        disable_raw_mode()?;
    }
    drained
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::{self, ErrorKind};

    #[test]
    fn the_event_loop_reads_input_through_the_apps_input_config() {
        let app = InputConfig {
            shift_wheel_horizontal: false,
            ctrl_click_secondary: true,
            legacy_ctrl_punctuation: true,
        };
        assert_eq!(effective_input_config(app, false), app, "the app's own");
        assert_eq!(
            effective_input_config(app, true),
            InputConfig {
                legacy_ctrl_punctuation: false,
                ..app
            },
            "the protocol reports Ctrl punctuation as itself"
        );
        let off = InputConfig {
            legacy_ctrl_punctuation: false,
            ..app
        };
        assert_eq!(
            effective_input_config(off, false),
            off,
            "an app may opt out"
        );
    }

    #[test]
    fn the_window_activating_click_is_ignored() {
        use ratatui::crossterm::event::{MouseButton, MouseEvent};
        let mouse = |kind| {
            Event::Mouse(MouseEvent {
                kind,
                column: 3,
                row: 4,
                modifiers: KeyModifiers::NONE,
            })
        };
        let down = mouse(MouseEventKind::Down(MouseButton::Left));
        let drag = mouse(MouseEventKind::Drag(MouseButton::Left));
        let up = mouse(MouseEventKind::Up(MouseButton::Left));
        let t0 = Instant::now();
        let ms = |n| t0 + Duration::from_millis(n);
        let mut g = FocusClickGuard::default();
        assert!(!g.drops(&down, t0), "no focus report: the click lands");
        assert!(!g.drops(&up, t0));
        assert!(g.drops(&Event::FocusGained, ms(1000)));
        assert!(!g.drops(&mouse(MouseEventKind::Moved), ms(1010)));
        assert!(!g.drops(&mouse(MouseEventKind::ScrollDown), ms(1020)));
        assert!(g.drops(&down, ms(1100)), "the activating press");
        assert!(g.drops(&drag, ms(1120)));
        assert!(g.drops(&up, ms(1150)), "and its release");
        assert!(!g.drops(&down, ms(1200)), "the next click is the user's");
        assert!(!g.drops(&up, ms(1210)));

        assert!(g.drops(&Event::FocusGained, ms(2000)));
        assert!(!g.drops(&down, ms(2400)));
        assert!(!g.drops(&up, ms(2410)));
        let mut off = FocusClickGuard::new(Duration::ZERO);
        assert!(off.drops(&Event::FocusGained, t0));
        assert!(!off.drops(&down, t0), "zero turns the guard off");
    }

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
    fn the_size_watch_delivers_a_missed_resize_once() {
        let mut w = SizeWatch::new(Some((120, 40)));
        assert_eq!(w.check((120, 40)), None, "nothing changed");

        assert_eq!(w.check((100, 30)), Some(Input::Resize(100, 30)));
        assert_eq!(w.check((100, 30)), None, "delivered once");

        w.observe(&Input::Resize(72, 20));
        assert_eq!(w.check((72, 20)), None);

        w.observe(&Input::Tick);
        assert_eq!(w.check((72, 20)), None);

        let mut unknown = SizeWatch::new(None);
        assert_eq!(unknown.check((80, 24)), Some(Input::Resize(80, 24)));
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

    #[derive(Default)]
    struct Recorder {
        handled: Vec<Input>,
        hover: Option<ratatui::layout::Position>,
        quit_on: Option<usize>,
    }

    impl Application for Recorder {
        fn handle(&mut self, input: Input) -> Outcome {
            self.handled.push(input.clone());
            match input {
                Input::Mouse(m) if m.kind == MouseKind::Move => {
                    if self.hover.replace(m.pos) == Some(m.pos) {
                        Outcome::Consumed
                    } else {
                        Outcome::Changed
                    }
                }
                Input::Mouse(_) | Input::Key(_) => Outcome::Changed,
                _ => Outcome::Ignored,
            }
        }
        fn render(&mut self, _: &mut Frame) {}
        fn should_quit(&self) -> bool {
            self.quit_on.is_some_and(|n| self.handled.len() >= n)
        }
        fn tick_interval(&self) -> Duration {
            Duration::from_secs(1)
        }
    }

    fn wheel_at(x: u16) -> Input {
        Input::Mouse(Mouse::new(
            MouseKind::WheelDown,
            ratatui::layout::Position::new(x, 5),
        ))
    }

    fn move_to(x: u16) -> Input {
        Input::Mouse(Mouse::new(
            MouseKind::Move,
            ratatui::layout::Position::new(x, 5),
        ))
    }

    fn press(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    fn frames(app: &mut Recorder, policy: Coalesce, queue: Vec<Input>) -> usize {
        let mut queue = std::collections::VecDeque::from(queue);
        let mut pump = InputPump::new(policy);
        let mut frames = 0;
        while pump.has_pending() || !queue.is_empty() {
            if pump.drain(app, || Ok(queue.pop_front())).unwrap() {
                frames += 1;
                if pump.settle(app) {
                    frames += 1;
                }
            }
            if app.should_quit() {
                break;
            }
        }
        frames
    }

    fn unbounded() -> Coalesce {
        Coalesce {
            max_latency: Duration::MAX,
            ..Coalesce::default()
        }
    }

    #[test]
    fn the_wheel_over_a_pointer_only_tooltip_scrolls_it_and_not_the_app() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::theme::Theme;
        use crate::ui::ctx::{Interaction, RenderCtx};
        use crate::ui::reveal::Reveal;
        use ratatui::backend::TestBackend;
        use ratatui::layout::{Position, Rect};

        struct TipApp {
            text: String,
            reveal: Reveal,
            wheels: usize,
        }
        impl Application for TipApp {
            fn handle(&mut self, input: Input) -> Outcome {
                if matches!(input, Input::Mouse(m) if m.wheel().is_some()) {
                    self.wheels += 1;
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            fn render(&mut self, frame: &mut Frame) {
                let theme = Theme::junie();
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let cell_id = crate::core::id::WidgetId::of("tip cell");
                let hover = Interaction {
                    hover: Some(cell_id),
                    ..Default::default()
                };
                let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
                let cell = Rect::new(2, 1, 6, 1);
                let reveal = Some(self.reveal);
                crate::ui::reveal::truncated(&mut ctx, reveal, cell_id, cell, &self.text);
                ctx.flush_overlays(frame.buffer_mut());
            }
            fn should_quit(&self) -> bool {
                false
            }
            fn tick_interval(&self) -> Duration {
                Duration::from_secs(1)
            }
        }
        let text = (1..=60)
            .map(|i| format!("line{i:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut app = TipApp {
            text,
            reveal: Reveal::default().max_rows(4),
            wheels: 0,
        };
        let mut terminal = ratatui::Terminal::new(TestBackend::new(40, 12)).unwrap();
        let mut draw = |app: &mut TipApp| {
            let buf = terminal.draw(|f| app.render(f)).unwrap().buffer.clone();
            (0..12)
                .map(|y| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
        };
        let wheel = |x, y| Input::Mouse(Mouse::new(MouseKind::WheelDown, Position::new(x, y)));
        let mut pump = InputPump::new(Coalesce::default());
        let mut send = |app: &mut TipApp, input: Input| {
            let mut queue = std::collections::VecDeque::from([input]);
            pump.drain(app, || Ok(queue.pop_front())).unwrap()
        };
        let rows = draw(&mut app);
        assert!(rows[1].contains("line01"), "{rows:#?}");
        assert!(rows[4].contains("↓"), "{rows:#?}");
        assert!(send(&mut app, wheel(3, 1)), "a frame is due");
        assert_eq!(app.wheels, 0, "the tooltip took the notch");
        let rows = draw(&mut app);
        assert!(rows[1].contains("↑"), "scrolled: {rows:#?}");
        assert!(!rows.iter().any(|r| r.contains("line01 ")), "{rows:#?}");

        assert!(send(&mut app, wheel(3, 10)));
        assert_eq!(app.wheels, 1);

        app.reveal = app.reveal.hover(false);
        draw(&mut app);
        send(&mut app, wheel(3, 1));
        assert_eq!(app.wheels, 2);

        app.reveal = Reveal::default().max_rows(4).scroll(false);
        draw(&mut app);
        send(&mut app, wheel(3, 1));
        assert_eq!(app.wheels, 3);
    }

    #[test]
    fn esc_dismisses_a_shown_tooltip_before_the_app_sees_it() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::core::id::WidgetId;
        use crate::theme::Theme;
        use crate::ui::ctx::{Interaction, RenderCtx};
        use crate::ui::tooltip::{self, Tip};
        use ratatui::backend::TestBackend;
        use ratatui::layout::Rect;

        const OWNER: WidgetId = WidgetId::of("tip owner");
        #[derive(Default)]
        struct TipApp {
            escs: usize,
            hovering: bool,
        }
        impl Application for TipApp {
            fn handle(&mut self, input: Input) -> Outcome {
                if matches!(input, Input::Key(k) if k.code == KeyCode::Esc) {
                    self.escs += 1;
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            fn render(&mut self, frame: &mut Frame) {
                let theme = Theme::junie();
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let hover = Interaction {
                    hover: self.hovering.then_some(OWNER),
                    hover_ms: Some(1_000),
                    ..Default::default()
                };
                let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
                tooltip::show(
                    &mut ctx,
                    OWNER,
                    Rect::new(1, 1, 6, 1),
                    &Tip::new("What it does"),
                    None,
                );
                ctx.flush_overlays(frame.buffer_mut());
            }
            fn should_quit(&self) -> bool {
                false
            }
            fn tick_interval(&self) -> Duration {
                Duration::from_secs(1)
            }
        }
        let mut app = TipApp {
            hovering: true,
            ..Default::default()
        };
        let mut terminal = ratatui::Terminal::new(TestBackend::new(30, 6)).unwrap();
        let mut draw = |app: &mut TipApp| {
            let buf = terminal.draw(|f| app.render(f)).unwrap().buffer.clone();
            (0..6)
                .map(|y| (0..30).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let esc = || {
            Input::Key(crate::core::event::Key {
                code: KeyCode::Esc,
                mods: ratatui::crossterm::event::KeyModifiers::NONE,
            })
        };
        let mut pump = InputPump::new(Coalesce::default());
        let mut send = |app: &mut TipApp, input: Input| {
            let mut queue = std::collections::VecDeque::from([input]);
            pump.drain(app, || Ok(queue.pop_front())).unwrap()
        };
        assert!(draw(&mut app).contains("What it does"));
        assert!(send(&mut app, esc()), "a frame is due");
        assert_eq!(app.escs, 0, "the tooltip took Esc");
        assert!(!draw(&mut app).contains("What it does"), "dismissed");
        assert!(send(&mut app, esc()));
        assert_eq!(app.escs, 1, "nothing shown: the app gets Esc");
    }

    #[test]
    fn a_wheel_burst_draws_one_frame_and_handles_every_notch() {
        let mut app = Recorder::default();
        let n = frames(&mut app, unbounded(), vec![wheel_at(3); 60]);
        assert_eq!(n, 1, "60 notches, one frame");
        assert_eq!(app.handled.len(), 60, "no notch is dropped");

        let mut app = Recorder::default();
        assert_eq!(frames(&mut app, Coalesce::OFF, vec![wheel_at(3); 60]), 60);
    }

    #[test]
    fn a_window_drag_draws_once_at_the_last_size() {
        #[derive(Default)]
        struct Sized {
            sizes: Vec<(u16, u16)>,
        }
        impl Application for Sized {
            fn handle(&mut self, input: Input) -> Outcome {
                match input {
                    Input::Resize(w, h) => {
                        self.sizes.push((w, h));
                        Outcome::Changed
                    }
                    _ => Outcome::Ignored,
                }
            }
            fn render(&mut self, _: &mut Frame) {}
            fn should_quit(&self) -> bool {
                false
            }
            fn tick_interval(&self) -> Duration {
                Duration::from_secs(1)
            }
        }
        let drag: Vec<Input> = (0..30).map(|i| Input::Resize(80 + i, 24)).collect();
        for (policy, want) in [
            (unbounded(), 1),
            (
                Coalesce {
                    resize: false,
                    ..unbounded()
                },
                30,
            ),
        ] {
            let mut app = Sized::default();
            let mut queue = std::collections::VecDeque::from(drag.clone());
            let mut pump = InputPump::new(policy);
            let mut frames = 0;
            while pump.has_pending() || !queue.is_empty() {
                if pump.drain(&mut app, || Ok(queue.pop_front())).unwrap() {
                    frames += 1;
                }
            }
            assert_eq!(frames, want);
            assert_eq!(app.sizes.len(), 30, "every size is handled");
            assert_eq!(app.sizes.last(), Some(&(109, 24)));
        }
    }

    #[test]
    fn a_held_key_draws_once_per_batch() {
        let mut app = Recorder::default();
        let n = frames(&mut app, unbounded(), vec![press(KeyCode::Down); 120]);
        assert_eq!(n, 2, "max_batch 64 splits 120 repeats into two frames");
        assert_eq!(app.handled.len(), 120);
        let policy = Coalesce {
            key_repeat: false,
            ..unbounded()
        };
        let mut app = Recorder::default();
        assert_eq!(frames(&mut app, policy, vec![press(KeyCode::Down); 10]), 10);
    }

    #[test]
    fn a_different_input_waits_for_the_frame_of_the_burst() {
        let mut app = Recorder::default();
        let mut queue = std::collections::VecDeque::from(vec![
            press(KeyCode::Down),
            press(KeyCode::Down),
            press(KeyCode::Enter),
            press(KeyCode::Enter),
            wheel_at(1),
            wheel_at(2),
        ]);
        let mut pump = InputPump::new(unbounded());
        assert!(pump.drain(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(app.handled.len(), 2, "both Downs, then the frame");
        assert!(pump.has_pending(), "Enter is held for the next frame");
        assert!(pump.drain(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(
            app.handled.len(),
            3,
            "Enter never folds, not even into Enter"
        );
        assert!(pump.drain(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(app.handled.len(), 4);
        assert!(pump.drain(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(app.handled.len(), 6, "the wheel burst folds");
        assert!(!pump.has_pending() && queue.is_empty());
    }

    #[test]
    fn wheel_bursts_split_on_modifiers_and_keys_on_chords() {
        let c = Coalesce::default();
        let shifted = Input::Mouse(
            Mouse::new(MouseKind::WheelDown, ratatui::layout::Position::new(0, 0))
                .with_mods(KeyModifiers::SHIFT),
        );
        assert!(c.folds(&wheel_at(0), &wheel_at(9)), "position may move");
        assert!(
            !c.folds(&wheel_at(0), &shifted),
            "Shift+wheel is another axis"
        );
        let up = Input::Mouse(Mouse::new(
            MouseKind::WheelUp,
            ratatui::layout::Position::new(0, 0),
        ));
        assert!(c.folds(&wheel_at(0), &up), "a reversal stays one gesture");
        assert!(!c.folds(&press(KeyCode::Down), &press(KeyCode::Up)));
        assert!(!c.folds(&press(KeyCode::Char('j')), &press(KeyCode::Char('j'))));
        let alt_down = Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::ALT,
        });
        assert!(!c.folds(&alt_down, &alt_down), "Alt chords are commands");
        assert!(!c.folds(&wheel_at(0), &move_to(0)));
    }

    #[test]
    fn a_pointer_sweep_draws_once_then_settles_hover_on_the_new_frame() {
        let mut app = Recorder::default();
        let sweep: Vec<Input> = (0..40).map(move_to).collect();
        let n = frames(&mut app, unbounded(), sweep);
        assert_eq!(n, 1, "settling on the same cell changes nothing");
        assert_eq!(app.handled.len(), 41, "the last move is handled again");
        assert_eq!(app.handled.last(), Some(&move_to(39)));
    }

    #[test]
    fn quitting_inside_a_burst_stops_the_drain() {
        let mut app = Recorder {
            quit_on: Some(5),
            ..Default::default()
        };
        let mut queue = std::collections::VecDeque::from(vec![wheel_at(0); 30]);
        let mut pump = InputPump::new(unbounded());
        assert!(pump.drain(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(app.handled.len(), 5);
    }

    #[test]
    fn the_uncoalesced_drain_never_reads_past_a_change() {
        let mut app = Recorder::default();
        let mut queue = std::collections::VecDeque::from(vec![wheel_at(0); 3]);
        assert!(drain_ready_inputs(&mut app, || Ok(queue.pop_front())).unwrap());
        assert_eq!(queue.len(), 2, "the next notch stays queued, not dropped");
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

        let mut pump = InputPump::new(Coalesce::default());
        assert!(pump.drain(&mut app, || Ok(queued.pop_front())).unwrap());
        assert_eq!(app.handled, 3, "unchanged events remain batched");
        assert!(app.activated.is_none(), "activation must wait for render");
        terminal.draw(|frame| app.render(frame)).unwrap();
        assert!(pump.drain(&mut app, || Ok(queued.pop_front())).unwrap());
        assert_eq!(app.activated, Some(WidgetId::of("new page control")));
        assert!(queued.is_empty());
    }
}
