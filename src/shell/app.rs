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

use super::{Clipboard, Copied, Cx, Request, StatusLine};
use crate::clock::Moment;
use crate::theme::Strings;
use crate::ui::context::ContextConfig;
use crate::ui::leave::{LeaveGuard, Stakes};
use crate::widgets::dialog::Dialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PageKey(pub u64);

impl PageKey {
    pub const ROOT: Self = Self(0);

    pub fn of(value: impl std::hash::Hash) -> Self {
        use std::hash::Hasher;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut h);
        Self(h.finish())
    }
}

type FooterDrawn = (Vec<Hint>, Vec<(usize, Rect)>);

pub struct Shell<P = ()> {
    pub focus: Focus,

    pub ring: FocusRing,

    pub hits: HitRegistry,

    pub pointer: PointerRouter,

    pub status: StatusLine<P>,

    pub size: (u16, u16),

    pub too_small: bool,

    pub quit: bool,

    pub motion: Motion,

    pub clock_steps: bool,

    pub focus_fallback: FocusFallback,

    pub footer_keys: HintKeys,

    pub context: ContextConfig,

    pub clipboard: Clipboard,
    footer: Option<FooterDrawn>,
    page: Option<PageKey>,
}

impl<P> Default for Shell<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Default> Shell<P> {
    pub fn copy_at(&mut self, text: &str, what: Option<&str>, now: Moment) -> Copied {
        let r = self.clipboard.copy(text);
        let (msg, tone) = Clipboard::report(&r, text, what);
        self.status.set_at(msg, tone, now);
        r
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
            clock_steps: crate::clock::ClockStep::opted_in(),
            focus_fallback: FocusFallback::Nearest,
            footer_keys: HintKeys::default(),
            context: ContextConfig::DEFAULT,
            clipboard: Clipboard::new(),
            footer: None,
            page: None,
        }
    }

    pub fn pointer_parts(&mut self) -> PointerParts<'_> {
        PointerParts {
            router: &mut self.pointer,
            hits: &self.hits,
            focus: &mut self.focus,
            ring: &self.ring,
        }
    }

    pub fn cx<R, W: ?Sized, G>(&mut self) -> Cx<'_, R, W, G> {
        Cx::new(&mut self.focus, &self.ring).context(self.context)
    }

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

    pub fn quit_through<T: Clone>(
        &mut self,
        guard: &mut LeaveGuard<T>,
        stakes: &Stakes,
        strings: &Strings,
    ) -> Result<(), Box<Dialog>> {
        guard.request_quit(stakes, strings)?;
        self.quit = true;
        Ok(())
    }

    pub fn focus_next(&mut self) {
        self.focus.next(&self.ring);
    }

    pub fn focus_prev(&mut self) {
        self.focus.prev(&self.ring);
    }

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

            hover_ms: (self.motion != Motion::Paused).then(|| p.hover_ms(now_ms)),
        }
    }

    pub fn footer_drawn(&mut self, hints: Vec<Hint>, spots: Vec<(usize, Rect)>) {
        self.footer = Some((hints, spots));
    }

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

pub trait App {
    type Status;

    fn shell(&self) -> &Shell<Self::Status>;
    fn shell_mut(&mut self) -> &mut Shell<Self::Status>;

    fn theme(&self) -> Theme;

    fn too_small(&self) -> TooSmall<'_>;

    fn now_ms(&self) -> u64;

    fn flash_ms(&self) -> u64 {
        self.now_ms()
    }

    fn tick(&self) -> u64;

    fn tick_interval(&self) -> Duration;

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);

    fn configure(&self, _ctx: &mut RenderCtx) {}

    fn on_input(&mut self, input: Input) -> Outcome;

    fn on_blur(&mut self, _change: FocusChange) -> Outcome {
        Outcome::Ignored
    }

    fn after_input(&mut self, _input: &Input, out: Outcome) -> Outcome {
        out
    }

    fn layer_open(&self) -> bool {
        false
    }

    fn page_under_layer(&self) -> bool {
        self.layer_open()
    }

    fn page_key(&self) -> PageKey {
        PageKey::ROOT
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        None
    }

    fn advance_clock(&mut self, _ms: u64) -> Option<Outcome> {
        None
    }

    fn stakes(&mut self) -> Stakes {
        Stakes::Clean
    }

    fn coalesce(&self) -> Coalesce {
        Coalesce::default()
    }

    fn input_config(&self) -> InputConfig {
        InputConfig::default()
    }

    fn keyboard_protocol(&self) -> KeyboardProtocol {
        KeyboardProtocol::current()
    }
}

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

pub fn render<A: App + ?Sized>(app: &mut A, frame: &mut Frame) {
    let area = frame.area();
    app.shell_mut().size = (area.width, area.height);
    let theme = app.theme();
    let mut cursor;

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

pub fn handle<A: App + ?Sized>(app: &mut A, input: Input) -> Outcome {
    let seen = app.shell_mut().status.on_input(&input);
    if let Input::Resize(w, h) = input {
        app.shell_mut().size = (w, h);
    }
    let logged = input.clone();

    let step = match &input {
        Input::Paste(text) if app.shell().clock_steps && app.shell().motion == Motion::Paused => {
            crate::clock::ClockStep::parse(text)
        }
        _ => None,
    };
    let mut out = match step.and_then(|ms| app.advance_clock(ms)) {
        Some(out) => out,
        None => app.on_input(input),
    };
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
    fn take_output(&mut self) -> Vec<String> {
        super::clipboard::take_output()
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
