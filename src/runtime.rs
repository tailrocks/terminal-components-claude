//! Terminal runtime shared by every application built on the library:
//! one terminal-session guard that owns raw mode, the alternate screen,
//! mouse capture, bracketed paste, the kitty keyboard protocol
//! ([`crate::core::keyboard`]), cursor visibility and line-wrap state
//! (restored on every exit path, including panics), an event loop that
//! drains unchanged input while rendering state changes before the next queued
//! event, and animation ticks on demand.
//!
//! Bursts of the same continuous input (wheel notches, pointer moves, a
//! held navigation key, a window being resized) are coalesced: every event is handled, but the
//! frame is drawn once for the burst instead of once per event (see
//! [`Coalesce`]). Any other input still waits for the frame of the change
//! before it, so an activation always sees the controls it activates.
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

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::core::event::{Input, InputConfig, Key, Mouse, MouseKind, Outcome};
use crate::core::keyboard::{self, KeyboardProtocol};

pub trait Application {
    fn handle(&mut self, input: Input) -> Outcome;
    fn render(&mut self, frame: &mut Frame);
    fn should_quit(&self) -> bool;
    /// How often to deliver `Input::Tick`.
    fn tick_interval(&self) -> Duration;
    /// Which queued input bursts draw one frame instead of one per event.
    /// Default [`Coalesce::default`]; [`Coalesce::OFF`] draws every change.
    fn coalesce(&self) -> Coalesce {
        Coalesce::default()
    }
    /// The pointer and caret shapes to show after the last frame (see
    /// [`crate::core::shape`]; [`crate::core::shape::shapes_for`] reads
    /// them off an app's pointer state and hit regions). Default: none,
    /// the terminal's own.
    fn shapes(&self) -> crate::core::shape::Shapes {
        crate::core::shape::Shapes::default()
    }
}

/// Input coalescing policy of the event loop.
///
/// A terminal delivers a wheel gesture as many notches, a pointer sweep as
/// one move per cell and a held key as auto-repeat, all queued faster than
/// a frame can be drawn and written. Drawing after each of them makes the
/// screen trail the input (it keeps scrolling after the wheel stops). With
/// coalescing, consecutive events of one enabled kind are all handled and
/// then drawn once. Every other input, and a different kind, still waits
/// for the frame of the change before it.
///
/// Wheel notches in a burst are routed with the hit regions of the frame
/// the burst started on, so the burst stays with the container under the
/// pointer when it began (scroll latching, as on macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coalesce {
    /// Wheel notches with the same modifiers, on either axis. Default on.
    pub wheel: bool,
    /// Pointer moves (hover). The burst's last move is handled again after
    /// its frame, against the new hit regions, so hover always settles on
    /// what is drawn. Default on.
    pub pointer_moves: bool,
    /// Repeats of one navigation key with the same modifiers: arrows,
    /// PageUp/PageDown and Home/End, plain or with Shift/Ctrl. Default on.
    /// Other keys (Enter, Tab, characters) are never coalesced.
    pub key_repeat: bool,
    /// Terminal resizes (a window being dragged): every size is handled,
    /// the frame is drawn at the last one. Default on.
    pub resize: bool,
    /// Most events folded into one frame; a longer burst draws an
    /// intermediate frame every this many events. Default 64.
    pub max_batch: usize,
    /// Longest time spent handling one burst before a frame is drawn, so a
    /// flood never freezes the screen. Default 16 ms (one 60 Hz frame).
    pub max_latency: Duration,
}

impl Coalesce {
    /// Draw after every state change (the behaviour before coalescing).
    pub const OFF: Self = Self {
        wheel: false,
        pointer_moves: false,
        key_repeat: false,
        resize: false,
        max_batch: 1,
        max_latency: Duration::ZERO,
    };

    /// Whether `next` continues a burst whose last event was `last`.
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

/// Owns every piece of terminal state the application changes and puts it
/// back in reverse order when dropped: the kitty keyboard flags popped,
/// bracketed paste and mouse capture off, cursor shown, line wrap re-enabled, alternate screen left, raw mode
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
            // start crossterm's input reader now: it registers its SIGWINCH
            // handler only when first polled, and a resize that lands
            // before that (right after the first frame) would be ignored
            // by the default disposition, so the app never repaints
            prime_input()?;
            let mut out = stdout();
            execute!(
                out,
                EnterAlternateScreen,
                EnableMouseCapture,
                EnableBracketedPaste
            )?;
            // progressive enhancement: Cmd (super) chords reach the app
            // where the terminal speaks the kitty keyboard protocol
            if let Some(push) = keyboard::session::enter(KeyboardProtocol::current(), || {
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
        if let Some(push) = keyboard::session::resume()
            && let Err(e) = out.write_all(push.as_bytes()).and_then(|()| out.flush())
        {
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
    // the arrow pointer and the terminal's own caret, if they were set
    let _ = out.write_all(crate::core::shape::reset_sequence().as_bytes());
    if let Some(pop) = keyboard::session::leave() {
        let _ = out.write_all(pop.as_bytes());
    }
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

/// Start the terminal input source without reading anything from it.
/// crossterm builds its reader, and with it the `SIGWINCH` handler that
/// turns a resize into `Input::Resize`, on the first poll; priming it as
/// the session starts means no resize after that point is lost. (The
/// keyboard-protocol probe used to do this by accident, only under
/// `KeyboardProtocol::Auto`.)
fn prime_input() -> std::io::Result<()> {
    poll_uninterrupted(Duration::ZERO).map(|_| ())
}

/// Test instrumentation: `JUNIE_TUI_FIRST_POLL_DELAY_MS` holds the event
/// loop for that long after the first frame, before it first reads input,
/// so a PTY test can resize the terminal inside that window
/// deterministically (`tests/visual_baseline/startup_resize.rs`). Unset in
/// normal use.
fn first_poll_delay() -> Option<Duration> {
    std::env::var("JUNIE_TUI_FIRST_POLL_DELAY_MS")
        .ok()?
        .parse()
        .ok()
        .map(Duration::from_millis)
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
    let mut pump = InputPump::new(app.coalesce());
    // hover content held back by a frame (a delayed reveal or tooltip):
    // that frame and when it may show
    let mut shapes =
        crate::core::shape::ShapeSync::new(crate::core::shape::ShapeSupport::current());
    let mut hover_due: Option<(u64, Instant)> = None;
    let mut first_poll_delay = first_poll_delay();
    let mut sizes = SizeWatch::new(terminal_size(&session.terminal));
    loop {
        #[cfg(unix)]
        if job_control::take_request() {
            let (w, h) = session.suspend()?;
            // the shapes were reset on the way out: send them again
            shapes.forget();
            // geometry may have changed while stopped: rebuild, then a full draw
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
        // the pointer and caret shapes of what the last frame drew
        if shapes.apply(app.shapes(), terminal.backend_mut())? {
            terminal.backend_mut().flush()?;
        }
        // a tick-driven transition may have asked to quit without any input
        if app.should_quit() {
            return Ok(());
        }
        let interval = app.tick_interval();
        let mut wait = interval.saturating_sub(last_tick.elapsed());
        // hover content waiting for the pointer to rest (a delayed reveal
        // or tooltip) shows on time, not at the next tick
        let now = Instant::now();
        hover_due = match (crate::ui::reveal::hover_wait(), hover_due) {
            (None, _) => None,
            (Some((frame, _)), Some((seen, due))) if frame == seen => Some((seen, due)),
            (Some((frame, ms)), _) => Some((frame, now + Duration::from_millis(ms))),
        };
        if let Some((_, due)) = hover_due {
            wait = wait.min(due.saturating_duration_since(now));
        }
        // an input held back for the previous frame is ready without polling
        if pump.has_pending() || poll_uninterrupted(wait)? {
            pump.set_policy(app.coalesce());
            let changed = pump.drain(app, || {
                let input = next_ready_input()?;
                if let Some(input) = &input {
                    sizes.observe(input);
                }
                Ok(input)
            })?;
            if app.should_quit() {
                return Ok(());
            }
            if changed {
                // Rendering also rebuilds hit regions and reconciles focus.
                // The next queued key must see the newly active controls.
                terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                if pump.settle(app) {
                    terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                }
            }
        }
        if hover_due.is_some_and(|(_, due)| due <= now + wait) {
            // the pointer has rested long enough: draw the held-back content
            dirty = true;
        }
        // a resize crossterm never reported still reaches the app
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
                // draw the final frame so a closing caption is seen before restore
                if dirty {
                    terminal.draw(|f| render_frame(app, f, suppress_colors))?;
                }
                return Ok(());
            }
        }
    }
}

/// The terminal's size now, if the backend can tell.
fn terminal_size(terminal: &ratatui::DefaultTerminal) -> Option<(u16, u16)> {
    terminal.size().ok().map(|s| (s.width, s.height))
}

/// The terminal size the application was last told, so a resize whose
/// `SIGWINCH` crossterm never reported still reaches it.
///
/// crossterm 0.29's reader returns the first key a poll parses and drops
/// the rest of that poll's readiness; the `SIGWINCH` pipe is
/// edge-triggered, so a resize that arrives while a key is waiting is never
/// reported and the app keeps drawing at the old size (the
/// `showcase/resize/overview_shrunk` flake: a resize, then Ctrl+L at once).
/// The event loop compares the real size once per pass and delivers the
/// missed [`Input::Resize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeWatch {
    known: Option<(u16, u16)>,
}

impl SizeWatch {
    /// Start from the size the first frame is drawn at.
    pub fn new(size: Option<(u16, u16)>) -> Self {
        Self { known: size }
    }

    /// An input on its way to the application: a resize is what it now
    /// knows.
    pub fn observe(&mut self, input: &Input) {
        if let Input::Resize(w, h) = input {
            self.known = Some((*w, *h));
        }
    }

    /// The terminal reports `actual`: the resize the application missed,
    /// if any (once; it is then what the application knows).
    pub fn check(&mut self, actual: (u16, u16)) -> Option<Input> {
        if self.known == Some(actual) {
            return None;
        }
        self.known = Some(actual);
        Some(Input::Resize(actual.0, actual.1))
    }
}

fn next_ready_input() -> std::io::Result<Option<Input>> {
    while event::poll(Duration::ZERO)? {
        let config = InputConfig {
            // the protocol reports Ctrl+\\ and friends as themselves
            legacy_ctrl_punctuation: !keyboard::active(),
            ..InputConfig::default()
        };
        if let Some(input) = Input::from_crossterm_with(event::read()?, config) {
            keyboard::observe(&input);
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
/// been handled; no coalescing ([`Coalesce::OFF`]: the input after a change
/// is not read, so nothing is held back). Returns whether state changed.
/// The runtime itself uses [`InputPump`].
///
/// # Errors
/// Propagates the input source's error.
pub fn drain_ready_inputs(
    app: &mut impl Application,
    next: impl FnMut() -> std::io::Result<Option<Input>>,
) -> std::io::Result<bool> {
    InputPump::new(Coalesce::OFF).drain(app, next)
}

/// The event loop's input dispatcher: drains queued input, folds bursts per
/// its [`Coalesce`] policy, and holds back the first input that must wait
/// for a frame. Public so an application's own tests can pump the same
/// batched-versus-separated sequences the runtime does: call
/// [`Self::drain`], draw when it returns `true`, then [`Self::settle`] (and
/// draw again when that returns `true`).
#[derive(Debug, Clone, Default)]
pub struct InputPump {
    policy: Coalesce,
    /// Read from the queue but due only after the next frame.
    pending: Option<Input>,
    /// The last pointer move of a coalesced burst, handled again after the
    /// burst's frame so hover resolves against what was drawn.
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

    /// Change the policy for the next drain (an app may switch it at
    /// runtime); a held-back input stays queued.
    pub fn set_policy(&mut self, policy: Coalesce) {
        self.policy = policy;
    }

    /// An input was held back for the next frame.
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Dispatch queued input until a state change is complete, the
    /// application asks to quit, the queue is empty, or `DRAIN_BUDGET`
    /// unchanged events have been handled. A change is complete when the
    /// next input does not continue its burst (that input is held back
    /// until after the frame), or when the burst reaches `max_batch` events
    /// or `max_latency`. Returns whether state changed, that is whether a
    /// frame is due.
    ///
    /// # Errors
    /// Propagates the input source's error.
    pub fn drain(
        &mut self,
        app: &mut impl Application,
        mut next: impl FnMut() -> std::io::Result<Option<Input>>,
    ) -> std::io::Result<bool> {
        let started = Instant::now();
        let mut unchanged = 0;
        // the burst in progress: its last input and how many it folded
        let mut burst: Option<(Input, usize)> = None;
        self.settle = None;
        loop {
            // a burst that cannot grow ends before the next input is read,
            // so nothing is held back without need
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
            // the wheel over a pointer-only reveal tooltip scrolls it; no
            // widget owns that tooltip, so the app never sees the notch.
            // Esc over a shown tooltip dismisses it (WCAG 1.4.13) and goes
            // no further.
            let changed = crate::ui::reveal::on_pointer_wheel(&input)
                .or_else(|| crate::ui::tooltip::on_input(&input))
                .unwrap_or_else(|| app.handle(input.clone()))
                == Outcome::Changed;
            if app.should_quit() {
                return Ok(changed || burst.is_some());
            }
            match &mut burst {
                // inside a burst an unchanged event (a notch past the end)
                // still belongs to it
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

    /// After the frame of a coalesced pointer-move burst: handle its last
    /// move again so hover resolves against the regions just drawn. Returns
    /// whether that changed state (another frame is due).
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

    /// A resize the input reader never reported is delivered once from the
    /// size check; a reported one is not delivered twice.
    #[test]
    fn the_size_watch_delivers_a_missed_resize_once() {
        let mut w = SizeWatch::new(Some((120, 40)));
        assert_eq!(w.check((120, 40)), None, "nothing changed");
        // the SIGWINCH was dropped: only the size check sees it
        assert_eq!(w.check((100, 30)), Some(Input::Resize(100, 30)));
        assert_eq!(w.check((100, 30)), None, "delivered once");
        // a reported resize is what the app knows: no duplicate
        w.observe(&Input::Resize(72, 20));
        assert_eq!(w.check((72, 20)), None);
        // other input changes nothing
        w.observe(&Input::Tick);
        assert_eq!(w.check((72, 20)), None);
        // no size at start: the first known size is delivered
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

    /// Records every handled input; wheel, arrows, Enter and pointer moves
    /// to a new cell change state.
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

    /// Run `queue` through the pump the way the event loop does; returns
    /// the number of frames drawn.
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

    /// A policy whose latency bound never ends a burst, so counts are
    /// deterministic on a slow machine.
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

        /// A cut cell under a resting pointer whose full text is longer
        /// than its 4-row tooltip; every wheel the app sees is counted.
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
        // outside the tooltip the app scrolls as before
        assert!(send(&mut app, wheel(3, 10)));
        assert_eq!(app.wheels, 1);
        // a frame without the tooltip leaves every notch to the app
        app.reveal = app.reveal.hover(false);
        draw(&mut app);
        send(&mut app, wheel(3, 1));
        assert_eq!(app.wheels, 2);
        // with scrolling off the tooltip stays put and the app scrolls
        app.reveal = Reveal::default().max_rows(4).scroll(false);
        draw(&mut app);
        send(&mut app, wheel(3, 1));
        assert_eq!(app.wheels, 3);
    }

    /// WCAG 1.4.13: Esc over a shown tooltip hides it and goes no further
    /// (a dialog under it stays open); with no tooltip the app gets Esc.
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
        // the policy is the knob: off draws every change
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
        // the runtime's own pump, with its default coalescing
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
