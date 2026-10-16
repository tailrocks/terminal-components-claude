//! The shell an application runs on (plan §1.8): the state every app keeps
//! (focus, ring, hit regions, pointer, status line, size) in [`Shell`], and
//! the [`App`] trait whose implementors get
//! [`Application`](crate::runtime::Application) from the library: the
//! render prologue and epilogue (background, too-small screen, overlays,
//! footer hits, hover refresh), focus upkeep and the blur fan-out.

use std::time::Duration;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::core::event::{Input, InputConfig, Outcome};
use crate::core::focus::{Focus, FocusChange, FocusFallback, FocusRing};
use crate::core::hit::HitRegistry;
use crate::core::id::WidgetId;
use crate::core::keyboard::KeyboardProtocol;
use crate::core::pointer::{PointerParts, PointerRouter};
use crate::motion::Motion;
use crate::runtime::{Application, Coalesce};
use crate::theme::Theme;
use crate::ui::ctx::{Interaction, RenderCtx, fill};
use crate::widgets::hintbar::HintKeys;
use crate::widgets::keyhint::Hint;
use crate::widgets::too_small::TooSmall;

use super::{Cx, Request, StatusLine};
use crate::ui::context::ContextConfig;

/// Which page of an app is mounted, for focus upkeep: when it changes,
/// focus that lost its control goes to the new page's
/// [`App::primary_focus`]; while it stays, to the nearest control
/// ([`FocusFallback::Nearest`], K8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PageKey(pub u64);

impl PageKey {
    /// The key of an app with one page.
    pub const ROOT: Self = Self(0);

    /// A key from anything hashable (a route, a tab index and its depth).
    pub fn of(value: impl std::hash::Hash) -> Self {
        use std::hash::Hasher;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut h);
        Self(h.finish())
    }
}

/// The footer hints drawn this frame and where each landed: the shell
/// registers them as click targets after the overlays, so the footer (which
/// names the topmost layer's keys) sits above every popup.
type FooterDrawn = (Vec<Hint>, Vec<(usize, Rect)>);

/// The state every app keeps for its shell. `P` is the status line's
/// presentation payload (toned parts, a status kind).
pub struct Shell<P = ()> {
    pub focus: Focus,
    /// The focus ring of the last frame.
    pub ring: FocusRing,
    /// The hit regions of the last frame.
    pub hits: HitRegistry,
    /// Hover, press, flash and click-count state.
    pub pointer: PointerRouter,
    /// The footer status ([`StatusLine`]).
    pub status: StatusLine<P>,
    /// The terminal size of the last frame or resize.
    pub size: (u16, u16),
    /// The last frame was the too-small screen.
    pub too_small: bool,
    /// The app asked to quit.
    pub quit: bool,
    /// Tick delivery and decorative motion.
    pub motion: Motion,
    /// Where focus goes when its control vanishes on the same page.
    /// Default [`FocusFallback::Nearest`] (K8).
    pub focus_fallback: FocusFallback,
    /// The keys of the footer hints drawn last frame: a click replays one.
    pub footer_keys: HintKeys,
    /// Which keys open a context menu, handed to every page's [`Cx`]
    /// ([`ContextConfig::DEFAULT`] unless the app sets one).
    pub context: ContextConfig,
    footer: Option<FooterDrawn>,
    page: Option<PageKey>,
}

impl<P> Default for Shell<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Shell<P> {
    pub fn new() -> Self {
        Self {
            focus: Focus::default(),
            ring: FocusRing::default(),
            hits: HitRegistry::default(),
            pointer: PointerRouter::default(),
            status: StatusLine::new(),
            size: (0, 0),
            too_small: false,
            quit: false,
            motion: Motion::Full,
            focus_fallback: FocusFallback::Nearest,
            footer_keys: HintKeys::default(),
            context: ContextConfig::DEFAULT,
            footer: None,
            page: None,
        }
    }

    /// The router's parts for a [`PointerHost`](crate::core::pointer::PointerHost).
    pub fn pointer_parts(&mut self) -> PointerParts<'_> {
        PointerParts {
            router: &mut self.pointer,
            hits: &self.hits,
            focus: &mut self.focus,
            ring: &self.ring,
        }
    }

    /// A fresh event context over this shell's focus and ring, with its
    /// context-menu keys.
    pub fn cx<R, W: ?Sized, G>(&mut self) -> Cx<'_, R, W, G> {
        Cx::new(&mut self.focus, &self.ring).context(self.context)
    }

    /// Hand an event to a page: run `f` in a fresh [`Cx`] and return its
    /// outcome with what the page asked for. A page that is not `shown`
    /// (a background tab, another route) keeps only its statuses (the
    /// background-request policy, [`super::background`]). The app applies
    /// the requests.
    pub fn deliver<R, W: ?Sized, G>(
        &mut self,
        shown: bool,
        f: impl FnOnce(&mut Cx<'_, R, W, G>) -> Outcome,
    ) -> (Outcome, Vec<Request<R, W, G>>) {
        let mut cx = self.cx();
        let o = f(&mut cx);
        let reqs = cx.take();
        if shown {
            (o, reqs)
        } else {
            (o, super::background(reqs).collect())
        }
    }

    /// Move focus to the next control of the ring.
    pub fn focus_next(&mut self) {
        self.focus.next(&self.ring);
    }

    /// Move focus to the previous control of the ring.
    pub fn focus_prev(&mut self) {
        self.focus.prev(&self.ring);
    }

    /// The interaction state a frame draws with. `now_ms` is the pointer
    /// clock, `flash_ms` the clock the press flash reads (a paused capture
    /// may stop it), `tick` the frame's animation tick.
    pub fn interaction(&self, now_ms: u64, flash_ms: u64, tick: u64) -> Interaction {
        let p = &self.pointer;
        Interaction {
            focus: self.focus.current(),
            hover: p.hover,
            pressed: p.pressed,
            flash: p.flashing(flash_ms),
            focus_hidden: false,
            hover_suppressed: p.hover_suppressed,
            pointer_focus: p.pointer_focus,
            motion_unreported: !p.reports_motion(),
            tick,
            // a paused capture has no clock: hover content shows at once
            hover_ms: (self.motion != Motion::Paused).then(|| p.hover_ms(now_ms)),
        }
    }

    /// The footer drew `hints`, each at its spot: the shell makes them
    /// click targets after the overlays (they name the top layer's keys,
    /// so they sit above every popup).
    pub fn footer_drawn(&mut self, hints: Vec<Hint>, spots: Vec<(usize, Rect)>) {
        self.footer = Some((hints, spots));
    }

    /// Keep focus on a control of the frame just drawn. Under a layer the
    /// layer keeps its own focus ([`Focus::ensure_valid`]). Otherwise,
    /// when the page changed, focus that lost its control goes to the
    /// page's `primary` control (else the first); on the same page it
    /// falls back per [`Self::focus_fallback`] (the nearest by default).
    pub fn upkeep(&mut self, layer_open: bool, page: PageKey, primary: Option<WidgetId>) {
        if layer_open {
            self.focus.ensure_valid(&self.ring);
            return;
        }
        if self.too_small {
            return;
        }
        let changed = self.page.replace(page) != Some(page);
        let valid = self.focus.current().is_some_and(|c| self.ring.contains(c));
        if changed && !valid {
            let ring = &self.ring;
            self.focus
                .set(primary.filter(|p| ring.contains(*p)).or(ring.first()));
        }
        self.focus
            .ensure_valid_with(&self.ring, self.focus_fallback);
    }
}

/// An application on the library shell. Implementing it gives the app
/// [`Application`]: the shell runs the frame (background, too-small screen,
/// [`Self::draw`], overlays, footer hits, focus upkeep) and the input path
/// (status seen, [`Self::on_input`], the blur fan-out).
pub trait App {
    /// The status line's presentation payload.
    type Status;

    fn shell(&self) -> &Shell<Self::Status>;
    fn shell_mut(&mut self) -> &mut Shell<Self::Status>;

    /// The theme the next frame draws with.
    fn theme(&self) -> Theme;

    /// The too-small screen (its brand and minimum size).
    fn too_small(&self) -> TooSmall<'_>;

    /// The pointer clock in milliseconds.
    fn now_ms(&self) -> u64;

    /// The clock the press flash reads. Default [`Self::now_ms`].
    fn flash_ms(&self) -> u64 {
        self.now_ms()
    }

    /// The animation tick of the next frame.
    fn tick(&self) -> u64;

    /// How often to deliver `Input::Tick`.
    fn tick_interval(&self) -> Duration;

    /// Draw the app on a cleared, big-enough screen.
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);

    /// App-wide defaults on the frame's context (track click, tooltips).
    fn configure(&self, _ctx: &mut RenderCtx) {}

    /// Route one input. The shell has already noted the status as seen
    /// and the size of a resize.
    fn on_input(&mut self, input: Input) -> Outcome;

    /// One focus change of the input just handled: fan it out to every
    /// open layer, then every mounted page, so fields end their edits.
    fn on_blur(&mut self, _change: FocusChange) -> Outcome {
        Outcome::Ignored
    }

    /// After the input and its blur fan-out (an input log).
    fn after_input(&mut self, _input: &Input, out: Outcome) -> Outcome {
        out
    }

    /// A modal layer holds the focus: upkeep keeps it inside the layer.
    fn layer_open(&self) -> bool {
        false
    }

    /// The page draws under a layer and shows no focus
    /// ([`RenderCtx::page_under_layer`]). Default [`Self::layer_open`].
    fn page_under_layer(&self) -> bool {
        self.layer_open()
    }

    /// The mounted page (see [`PageKey`]).
    fn page_key(&self) -> PageKey {
        PageKey::ROOT
    }

    /// Where focus lands on the mounted page when it has none.
    fn primary_focus(&self) -> Option<WidgetId> {
        None
    }

    /// See [`Application::coalesce`].
    fn coalesce(&self) -> Coalesce {
        Coalesce::default()
    }

    /// See [`Application::input_config`].
    fn input_config(&self) -> InputConfig {
        InputConfig::default()
    }

    /// See [`Application::keyboard_protocol`].
    fn keyboard_protocol(&self) -> KeyboardProtocol {
        KeyboardProtocol::current()
    }
}

/// Hand every focus change the last input made to [`App::on_blur`].
pub fn blur_fan_out<A: App + ?Sized>(app: &mut A) -> Outcome {
    let changes = {
        let s = app.shell_mut();
        s.focus.settle(&s.ring)
    };
    let mut out = Outcome::Ignored;
    for ch in changes {
        out = out.or(app.on_blur(ch));
    }
    out
}

/// Draw one frame of `app`: see [`App`].
pub fn render<A: App + ?Sized>(app: &mut A, frame: &mut Frame) {
    let area = frame.area();
    app.shell_mut().size = (area.width, area.height);
    let theme = app.theme();
    let mut cursor;
    // a second pass when the focused control vanished this frame: upkeep
    // picks the new focus from the ring the first pass built, and the
    // frame must show it (one focus indicator, never none)
    let mut pass = 0;
    loop {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = app
            .shell()
            .interaction(app.now_ms(), app.flash_ms(), app.tick());
        let small = app.too_small().applies(area);
        let mut footer_keys = HintKeys::default();
        {
            let buf = frame.buffer_mut();
            if pass > 0 {
                buf.reset();
            }
            fill(buf, area, theme.base());
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            if small {
                app.too_small().render(area, buf, &theme);
            } else {
                app.configure(&mut ctx);
                ctx.page_under_layer(app.page_under_layer());
                app.draw(area, buf, &mut ctx);
                ctx.flush_overlays(buf);
                if let Some((hints, spots)) = app.shell_mut().footer.take() {
                    footer_keys = HintKeys::register(&mut ctx, buf, &hints, spots);
                }
            }
            cursor = ctx.cursor;
        }
        let layer_open = app.layer_open();
        let page = app.page_key();
        let primary = app.primary_focus();
        let s = app.shell_mut();
        s.too_small = small;
        s.footer = None;
        s.footer_keys = footer_keys;
        s.hits = hits;
        s.ring = ring;
        s.pointer.refresh_hover(&s.hits);
        let before = s.focus.current();
        s.upkeep(layer_open, page, primary);
        pass += 1;
        if pass > 1 || before.is_none() || s.focus.current() == before {
            break;
        }
    }
    if let Some(pos) = cursor {
        frame.set_cursor_position(pos);
    }
}

/// Handle one input of `app`: see [`App`].
pub fn handle<A: App + ?Sized>(app: &mut A, input: Input) -> Outcome {
    // an error status has been seen once the user acts again
    let seen = app.shell_mut().status.on_input(&input);
    if let Input::Resize(w, h) = input {
        app.shell_mut().size = (w, h);
    }
    let logged = input.clone();
    let mut out = app.on_input(input);
    if seen {
        out = out.or(Outcome::Changed);
    }
    out = out.or(blur_fan_out(app));
    app.after_input(&logged, out)
}

impl<A: App> Application for A {
    fn handle(&mut self, input: Input) -> Outcome {
        handle(self, input)
    }
    fn render(&mut self, frame: &mut Frame) {
        render(self, frame)
    }
    fn should_quit(&self) -> bool {
        self.shell().quit
    }
    fn tick_interval(&self) -> Duration {
        App::tick_interval(self)
    }
    fn coalesce(&self) -> Coalesce {
        App::coalesce(self)
    }
    fn shapes(&self) -> crate::core::shape::Shapes {
        let s = self.shell();
        crate::core::shape::shapes_for(&s.pointer, &s.hits)
    }
    fn input_config(&self) -> InputConfig {
        App::input_config(self)
    }
    fn keyboard_protocol(&self) -> KeyboardProtocol {
        App::keyboard_protocol(self)
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
