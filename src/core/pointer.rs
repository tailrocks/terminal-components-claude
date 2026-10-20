use ratatui::layout::Position;

use ratatui::crossterm::event::KeyModifiers;

use super::event::{Mouse, MouseKind, Outcome};
use super::focus::{Focus, FocusRing};
use super::hit::{Axis, Bar, Hit, HitRegistry, ReleaseFrom};
use super::id::{ItemKey, Part, PartRef, WidgetId};
use super::intent::{Intent, Phase};
use super::response::Response;
use super::scroll::Scrollable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusOn {
    #[default]
    Press,

    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecondaryOutside {
    #[default]
    Reopen,

    Close,

    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryRoute {
    Nothing,

    Target(WidgetId),

    Outside {
        backdrop: WidgetId,
        reopen: Option<WidgetId>,
    },

    Swallowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerConfig {
    pub focus_on: FocusOn,

    pub double_click_ms: u64,

    pub double_click_slop: u16,

    pub wheel_lines: i32,

    pub hover_refresh_after_scroll: bool,

    pub flash_ms: u64,

    pub secondary_outside: SecondaryOutside,

    pub secondary_focuses: bool,

    pub end_edit_on_dead_space: bool,

    pub autoscroll: bool,

    pub autoscroll_ms: u64,

    pub track_repeat: bool,

    pub track_repeat_delay_ms: u64,

    pub track_repeat_ms: u64,

    pub pointer_shapes: bool,

    pub caret_shape: super::shape::CaretShape,

    pub click_skips_animation: bool,
}

impl PointerConfig {
    pub const DEFAULT: Self = Self {
        focus_on: FocusOn::Press,
        double_click_ms: 500,
        double_click_slop: 1,
        wheel_lines: 3,
        hover_refresh_after_scroll: true,
        flash_ms: 140,
        secondary_outside: SecondaryOutside::Reopen,
        secondary_focuses: true,
        end_edit_on_dead_space: true,
        autoscroll: true,
        autoscroll_ms: 50,
        track_repeat: true,
        track_repeat_delay_ms: 300,
        track_repeat_ms: 50,
        pointer_shapes: true,
        caret_shape: super::shape::CaretShape::Bar,
        click_skips_animation: true,
    };

    pub const fn focus_on(mut self, focus_on: FocusOn) -> Self {
        self.focus_on = focus_on;
        self
    }
    pub const fn double_click_ms(mut self, ms: u64) -> Self {
        self.double_click_ms = ms;
        self
    }
    pub const fn double_click_slop(mut self, cells: u16) -> Self {
        self.double_click_slop = cells;
        self
    }
    pub const fn wheel_lines(mut self, lines: i32) -> Self {
        self.wheel_lines = lines;
        self
    }
    pub const fn hover_refresh_after_scroll(mut self, on: bool) -> Self {
        self.hover_refresh_after_scroll = on;
        self
    }
    pub const fn flash_ms(mut self, ms: u64) -> Self {
        self.flash_ms = ms;
        self
    }
    pub const fn secondary_outside(mut self, policy: SecondaryOutside) -> Self {
        self.secondary_outside = policy;
        self
    }

    pub const fn secondary_focuses(mut self, on: bool) -> Self {
        self.secondary_focuses = on;
        self
    }
    pub const fn pointer_shapes(mut self, on: bool) -> Self {
        self.pointer_shapes = on;
        self
    }

    pub const fn caret_shape(mut self, shape: super::shape::CaretShape) -> Self {
        self.caret_shape = shape;
        self
    }

    pub const fn end_edit_on_dead_space(mut self, on: bool) -> Self {
        self.end_edit_on_dead_space = on;
        self
    }
    pub const fn click_skips_animation(mut self, on: bool) -> Self {
        self.click_skips_animation = on;
        self
    }
    pub const fn autoscroll(mut self, on: bool) -> Self {
        self.autoscroll = on;
        self
    }
    pub const fn autoscroll_ms(mut self, ms: u64) -> Self {
        self.autoscroll_ms = ms;
        self
    }

    pub const fn track_repeat(mut self, on: bool) -> Self {
        self.track_repeat = on;
        self
    }

    pub const fn track_repeat_ms(mut self, delay_ms: u64, every_ms: u64) -> Self {
        self.track_repeat_delay_ms = delay_ms;
        self.track_repeat_ms = every_ms;
        self
    }
}

impl Default for PointerConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    Click(WidgetId),

    Outside,

    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastClick {
    id: WidgetId,
    pos: Position,
    at: u64,
    count: u8,
}

#[derive(Debug, Clone, Default)]
pub struct PointerState {
    pub config: PointerConfig,

    pub hover: Option<WidgetId>,

    pub pressed: Option<WidgetId>,

    pub hover_suppressed: bool,

    pub pos: Option<Position>,

    pub pointer_focus: bool,
    flash: Option<(WidgetId, u64)>,

    grab: Option<super::hit::Bar>,

    edge: Option<ratatui::layout::Rect>,

    autoscroll_at: Option<u64>,

    track_at: Option<u64>,

    secondary_at: Option<Position>,

    accepted: Option<WidgetId>,

    motion_seen: bool,
    last_click: Option<LastClick>,
    hover_refresh: Option<Position>,

    hover_mark: std::cell::Cell<Option<(WidgetId, u64)>>,
}

impl PointerState {
    pub fn new(config: PointerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn suppress_hover(&mut self) {
        self.hover_suppressed = true;
        self.pointer_focus = false;
    }

    pub fn focused_by_click(&mut self) {
        self.pointer_focus = true;
    }

    pub fn on_move(&mut self, hits: &HitRegistry, pos: Position) -> Outcome {
        self.pos = Some(pos);
        self.motion_seen = true;
        let was = self.hover;
        let suppressed = self.hover_suppressed;
        self.hover_suppressed = false;
        self.hover = hits.hover_at(pos);
        if self.hover != was || suppressed {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_drag(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);

        self.hover = hits.hit(pos);
        self.pressed
    }

    pub fn on_down(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        let hit = hits.hit(pos);
        let grab = hit.and_then(|id| hits.bar(id));

        if grab.is_none() || grab != self.grab || hit != self.pressed {
            self.track_at = None;
        }
        self.pressed = hit;
        self.hover = hit;
        self.grab = grab;
        self.edge = hit.and_then(|_| hits.autoscroll_at(pos));
        self.autoscroll_at = None;
        hit
    }

    pub fn autoscrolling(&self) -> bool {
        self.config.autoscroll
            && self.pressed.is_some()
            && self.grabbed().is_none()
            && matches!((self.edge, self.pos), (Some(edge), Some(pos)) if !edge.contains(pos))
    }

    pub fn autoscroll_due(&mut self, now_ms: u64) -> Option<(WidgetId, Position)> {
        if !self.autoscrolling() {
            self.autoscroll_at = None;
            return None;
        }
        if let Some(at) = self.autoscroll_at
            && now_ms.saturating_sub(at) < self.config.autoscroll_ms
        {
            return None;
        }
        self.autoscroll_at = Some(now_ms);
        Some((self.pressed?, self.pos?))
    }

    pub fn track_repeating(&self) -> bool {
        self.config.track_repeat
            && self.pressed.is_some()
            && matches!((self.grab, self.pos), (Some(bar), Some(pos)) if bar.track.contains(pos))
    }

    pub fn track_repeat_due(&mut self, now_ms: u64) -> Option<Position> {
        if !self.track_repeating() {
            self.track_at = None;
            return None;
        }
        match self.track_at {
            None => {
                self.track_at = Some(now_ms + self.config.track_repeat_delay_ms);
                None
            }
            Some(at) if now_ms >= at => {
                self.track_at = Some(now_ms + self.config.track_repeat_ms);
                self.pos
            }
            Some(_) => None,
        }
    }

    pub fn surface_press(
        &self,
        hits: &HitRegistry,
        id: WidgetId,
        focus: &mut super::focus::Focus,
    ) -> bool {
        if !hits.is_surface(id) {
            return false;
        }
        self.dead_space(focus);
        true
    }

    pub fn dead_space(&self, focus: &mut super::focus::Focus) {
        if self.config.end_edit_on_dead_space {
            focus.end_edit();
        }
    }

    pub fn grabbed(&self) -> Option<super::hit::Bar> {
        self.grab.filter(|b| self.pressed == Some(b.id))
    }

    pub fn reports_motion(&self) -> bool {
        self.motion_seen
    }

    pub fn on_secondary(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        self.pressed = None;
        self.secondary_at = Some(pos);
        let hit = hits.hit(pos);
        self.hover = hit;
        hit
    }

    pub fn route_secondary(&mut self, hits: &HitRegistry, pos: Position) -> SecondaryRoute {
        let hit = self.on_secondary(hits, pos);
        let Some(id) = hit else {
            return SecondaryRoute::Nothing;
        };
        let Some((backdrop, beneath)) = hits.backdrop_at(pos) else {
            return SecondaryRoute::Target(id);
        };
        match self.config.secondary_outside {
            SecondaryOutside::Ignore => SecondaryRoute::Swallowed,
            SecondaryOutside::Close => SecondaryRoute::Outside {
                backdrop,
                reopen: None,
            },
            SecondaryOutside::Reopen => {
                self.hover = beneath;
                SecondaryRoute::Outside {
                    backdrop,
                    reopen: beneath,
                }
            }
        }
    }

    pub fn on_secondary_drag(&mut self, hits: &HitRegistry, pos: Position) -> Outcome {
        self.pos = Some(pos);
        let was = self.hover;
        self.hover_suppressed = false;
        self.hover = hits.hit(pos);
        if self.hover != was {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_secondary_up(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        let from = self.secondary_at.take()?;
        if from == pos {
            return None;
        }
        let hit = hits.hit(pos)?;
        hits.accepts_release(hit, ReleaseFrom::Secondary)
            .then_some(hit)
    }

    pub fn on_up(
        &mut self,
        hits: &HitRegistry,
        pos: Position,
    ) -> (Option<WidgetId>, Option<WidgetId>) {
        let hit = hits.hit(pos);
        self.accepted = match (hit, self.pressed) {
            (Some(h), Some(p)) if h != p && hits.accepts_release(h, ReleaseFrom::Press(p)) => {
                Some(h)
            }
            _ => None,
        };
        let beneath = hit
            .filter(|id| self.pressed == Some(*id))
            .and_then(|id| hits.pass_through_target(id, pos));
        (hit, beneath)
    }

    pub fn release(&mut self, hit: Option<WidgetId>) -> Release {
        let pressed = self.pressed.take();
        let accepted = self.accepted.take();
        self.grab = None;
        self.edge = None;
        self.track_at = None;
        match hit {
            None if pressed.is_none() => Release::Outside,
            None => Release::Cancelled,
            Some(id) if pressed == Some(id) || accepted == Some(id) => Release::Click(id),
            Some(_) => Release::Cancelled,
        }
    }

    pub fn press(&mut self, id: WidgetId) {
        self.pressed = Some(id);
    }

    pub fn count_click(&mut self, id: WidgetId, pos: Position, now_ms: u64) -> u8 {
        let count = self.press_count(id, pos, now_ms);
        self.last_click = Some(LastClick {
            id,
            pos,
            at: now_ms,
            count,
        });
        count
    }

    pub fn press_count(&self, id: WidgetId, pos: Position, now_ms: u64) -> u8 {
        let slop = self.config.double_click_slop;
        match self.last_click {
            Some(l)
                if l.id == id
                    && now_ms.saturating_sub(l.at) < self.config.double_click_ms
                    && l.pos.x.abs_diff(pos.x) <= slop
                    && l.pos.y.abs_diff(pos.y) <= slop =>
            {
                l.count.saturating_add(1)
            }
            _ => 1,
        }
    }

    pub fn hover_ms(&self, now_ms: u64) -> u64 {
        let Some(id) = self.hover.filter(|_| !self.hover_suppressed) else {
            self.hover_mark.set(None);
            return 0;
        };
        match self.hover_mark.get() {
            Some((h, since)) if h == id => now_ms.saturating_sub(since),
            _ => {
                self.hover_mark.set(Some((id, now_ms)));
                0
            }
        }
    }

    pub fn reset_clicks(&mut self) {
        self.last_click = None;
    }

    pub fn flash(&mut self, id: WidgetId, now_ms: u64) {
        self.flash = Some((id, now_ms.saturating_add(self.config.flash_ms)));
    }

    pub fn flashing(&self, now_ms: u64) -> Option<WidgetId> {
        self.flash
            .filter(|&(_, until)| now_ms < until)
            .map(|(id, _)| id)
    }

    pub fn has_flash(&self) -> bool {
        self.flash.is_some()
    }

    pub fn expire_flash(&mut self, now_ms: u64) -> bool {
        match self.flash {
            Some((_, until)) if now_ms >= until => {
                self.flash = None;
                true
            }
            _ => false,
        }
    }

    pub fn wheel_delta(&self, kind: MouseKind) -> i32 {
        match kind {
            MouseKind::WheelUp | MouseKind::WheelLeft => -self.config.wheel_lines,
            _ => self.config.wheel_lines,
        }
    }

    pub fn scrolled(&mut self, pos: Position, out: Outcome) {
        if out == Outcome::Changed && self.config.hover_refresh_after_scroll {
            self.hover_refresh = Some(pos);
        }
    }

    pub fn refresh_hover(&mut self, hits: &HitRegistry) {
        if let Some(pos) = self.hover_refresh.take()
            && !self.hover_suppressed
        {
            self.hover = hits.hover_at(pos);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerEvent {
    pub id: WidgetId,

    pub hit: Hit,

    pub intent: Intent<'static>,

    pub bar: Option<Bar>,
}

impl PointerEvent {
    pub fn new(
        hits: &HitRegistry,
        phase: Phase,
        id: WidgetId,
        pos: Position,
        count: u8,
        mods: KeyModifiers,
    ) -> Self {
        let hit = hit_of(hits, id);

        let pos = hits.local(id, pos);
        Self {
            id,
            hit,
            intent: Intent::Pointer {
                phase,
                part: hit.part,
                pos,
                count,
                mods,
            },
            bar: hits.bar(id),
        }
    }

    pub fn wheel_on(
        hits: &HitRegistry,
        id: WidgetId,
        axis: Axis,
        delta: i32,
        pos: Position,
        mods: KeyModifiers,
    ) -> Self {
        let hit = hit_of(hits, id);
        let pos = hits.local(id, pos);
        Self {
            id,
            hit,
            intent: Intent::Wheel {
                axis,
                delta: delta.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
                part: hit.part,
                pos,
                mods,
            },
            bar: None,
        }
    }

    pub fn owner(&self) -> WidgetId {
        self.hit.owner
    }

    pub fn on(&self, owner: WidgetId) -> bool {
        self.hit.owner == owner
    }

    pub fn part(&self) -> PartRef {
        self.hit.part
    }

    pub fn index_of(&self, part: Part) -> Option<usize> {
        if self.hit.part.part != part {
            return None;
        }
        self.hit.part.item.and_then(ItemKey::as_index)
    }

    pub fn phase(&self) -> Option<Phase> {
        match self.intent {
            Intent::Pointer { phase, .. } => Some(phase),
            _ => None,
        }
    }

    pub fn is(&self, phase: Phase) -> bool {
        self.phase() == Some(phase)
    }

    pub fn pos(&self) -> Position {
        self.intent.pos().unwrap_or_default()
    }

    pub fn count(&self) -> u8 {
        match self.intent {
            Intent::Pointer { count, .. } => count,
            _ => 0,
        }
    }

    pub fn mods(&self) -> KeyModifiers {
        match self.intent {
            Intent::Pointer { mods, .. } | Intent::Wheel { mods, .. } => mods,
            _ => KeyModifiers::NONE,
        }
    }

    pub fn wheel(&self) -> Option<(Axis, i32)> {
        match self.intent {
            Intent::Wheel { axis, delta, .. } => Some((axis, delta as i32)),
            _ => None,
        }
    }
}

fn hit_of(hits: &HitRegistry, id: WidgetId) -> Hit {
    hits.part_of(id).unwrap_or(Hit {
        owner: id,
        part: PartRef::of(Part::CONTAINER),
    })
}

pub fn scroll_input<S: Scrollable + ?Sized>(w: &mut S, ev: &PointerEvent) -> Option<Response<()>> {
    if let Some((axis, delta)) = ev.wheel() {
        return Some(w.wheel(axis, delta));
    }
    let bar = ev.bar.filter(|b| b.owner == ev.owner())?;
    match ev.phase()? {
        Phase::Press => Some(w.bar_input(&bar, ev.pos(), true)),
        Phase::Drag | Phase::DragStart => Some(w.bar_input(&bar, ev.pos(), false)),
        Phase::Click | Phase::Release | Phase::DragEnd => {
            w.pane().release();
            Some(Response::consumed())
        }
        _ => None,
    }
}

pub trait MouseTarget {
    type Action;

    fn mouse_id(&self) -> WidgetId;

    fn pointer(&mut self, ev: &PointerEvent) -> Response<Self::Action>;

    fn route(&mut self, ev: &PointerEvent) -> Option<Response<Self::Action>> {
        ev.on(self.mouse_id()).then(|| self.pointer(ev))
    }
}

pub struct PointerParts<'a> {
    pub router: &'a mut PointerRouter,
    pub hits: &'a HitRegistry,
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
}

pub trait PointerHost {
    fn pointer_parts(&mut self) -> PointerParts<'_>;

    fn layer_open(&self) -> bool;

    fn now_ms(&self) -> u64;

    fn deliver(&mut self, ev: &PointerEvent) -> Response<()>;

    fn outside(&mut self, _pos: Position) -> Response<()> {
        Response::changed()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PointerRouter {
    pub state: PointerState,

    dragged: bool,

    held: Option<(WidgetId, Hit)>,
}

impl std::ops::Deref for PointerRouter {
    type Target = PointerState;
    fn deref(&self) -> &PointerState {
        &self.state
    }
}

impl std::ops::DerefMut for PointerRouter {
    fn deref_mut(&mut self) -> &mut PointerState {
        &mut self.state
    }
}

impl PointerRouter {
    pub fn new(config: PointerConfig) -> Self {
        Self {
            state: PointerState::new(config),
            dragged: false,
            held: None,
        }
    }

    fn held_event(
        &self,
        hits: &HitRegistry,
        phase: Phase,
        id: WidgetId,
        pos: Position,
        mods: KeyModifiers,
    ) -> PointerEvent {
        let mut ev = PointerEvent::new(hits, phase, id, pos, 1, mods);
        if hits.part_of(id).is_none()
            && let Some((held, hit)) = self.held
            && held == id
        {
            ev.hit = hit;
            if let Intent::Pointer { part, .. } = &mut ev.intent {
                *part = hit.part;
            }
        }
        ev
    }

    pub fn route<H: PointerHost + ?Sized>(host: &mut H, m: Mouse) -> Outcome {
        let before = host.pointer_parts().focus.current();
        let out = Self::dispatch(host, m);

        let p = host.pointer_parts();
        let pressed = m.kind == MouseKind::Down && p.router.state.pressed.is_some();
        if pressed || (m.kind == MouseKind::Up && p.focus.current() != before) {
            p.router.state.focused_by_click();
        }
        out
    }

    pub fn tick<H: PointerHost + ?Sized>(host: &mut H) -> Outcome {
        let now = host.now_ms();
        let due = host.pointer_parts().router.state.autoscroll_due(now);
        let out = match due {
            Some((_, pos)) => Self::route(host, Mouse::new(MouseKind::Drag, pos)),
            None => Outcome::Ignored,
        };
        let due = host.pointer_parts().router.state.track_repeat_due(now);
        match due {
            Some(pos) => out.or(Self::route(host, Mouse::new(MouseKind::Down, pos))),
            None => out,
        }
    }

    fn dispatch<H: PointerHost + ?Sized>(host: &mut H, m: Mouse) -> Outcome {
        let layer = host.layer_open();

        let swallow = |out: Outcome| {
            if layer && out == Outcome::Ignored {
                Outcome::Consumed
            } else {
                out
            }
        };
        host.pointer_parts().router.state.pos = Some(m.pos);
        match m.kind {
            MouseKind::Move => {
                let p = host.pointer_parts();
                p.router.state.on_move(p.hits, m.pos)
            }
            MouseKind::Down => Self::down(host, m, layer),
            MouseKind::Drag => {
                let p = host.pointer_parts();
                let Some(pressed) = p.router.state.on_drag(p.hits, m.pos) else {
                    return Outcome::Ignored;
                };
                p.router.dragged = true;
                let bar = p.router.state.grabbed();
                let mut ev = p
                    .router
                    .held_event(p.hits, Phase::Drag, pressed, m.pos, m.mods);
                ev.bar = bar;
                swallow(host.deliver(&ev).outcome())
            }
            MouseKind::Up => {
                let now = host.now_ms();
                let p = host.pointer_parts();
                let pressed = p.router.state.pressed;
                let dragged = std::mem::take(&mut p.router.dragged);
                let (hit, beneath) = p.router.state.on_up(p.hits, m.pos);
                let mut out = Outcome::Ignored;
                if let Some(pressed) = pressed.filter(|_| dragged) {
                    let ev = p
                        .router
                        .held_event(p.hits, Phase::DragEnd, pressed, m.pos, m.mods);
                    out = host.deliver(&ev).outcome();
                }
                out = out.or(Self::release(host, m, hit, layer, now));

                if let Some(below) = beneath {
                    host.pointer_parts().router.state.press(below);
                    out = out.or(Self::release(host, m, Some(below), layer, now));
                }
                out
            }
            MouseKind::Secondary => Self::secondary(host, m, layer),
            MouseKind::SecondaryDrag => {
                let p = host.pointer_parts();
                p.router.state.on_secondary_drag(p.hits, m.pos)
            }

            MouseKind::SecondaryUp => {
                let now = host.now_ms();
                let p = host.pointer_parts();
                match p.router.state.on_secondary_up(p.hits, m.pos) {
                    Some(id) => {
                        p.router.state.press(id);
                        Self::release(host, m, Some(id), layer, now)
                    }
                    None => Outcome::Ignored,
                }
            }

            MouseKind::Middle => {
                let p = host.pointer_parts();
                let Some(id) = p.hits.hit(m.pos) else {
                    return swallow(Outcome::Ignored);
                };
                let ev = PointerEvent::new(p.hits, Phase::Middle, id, m.pos, 1, m.mods);
                swallow(host.deliver(&ev).outcome())
            }
            MouseKind::WheelUp
            | MouseKind::WheelDown
            | MouseKind::WheelLeft
            | MouseKind::WheelRight => swallow(Self::wheel(host, m)),
        }
    }

    fn down<H: PointerHost + ?Sized>(host: &mut H, m: Mouse, layer: bool) -> Outcome {
        let now = host.now_ms();
        let p = host.pointer_parts();
        p.router.dragged = false;
        p.router.held = None;
        let Some(id) = p.router.state.on_down(p.hits, m.pos) else {
            if layer {
                return Outcome::Consumed;
            }
            p.router.state.dead_space(p.focus);
            return Outcome::Ignored;
        };
        if layer {
            p.router.state.surface_press(p.hits, id, p.focus);
        } else if p.router.state.config.focus_on == FocusOn::Press && p.ring.contains(id) {
            p.focus.focus(id);
        }
        p.router.held = Some((id, hit_of(p.hits, id)));

        let count = p.router.state.press_count(id, m.pos, now);
        let ev = PointerEvent::new(p.hits, Phase::Press, id, m.pos, count, m.mods);
        host.deliver(&ev).outcome().or(Outcome::Changed)
    }

    fn secondary<H: PointerHost + ?Sized>(host: &mut H, m: Mouse, layer: bool) -> Outcome {
        let now = host.now_ms();
        let p = host.pointer_parts();
        let route = p.router.state.route_secondary(p.hits, m.pos);
        let at = |host: &mut H, id: WidgetId| {
            let p = host.pointer_parts();

            if !layer && p.router.state.config.secondary_focuses && p.ring.contains(id) {
                p.focus.focus(id);
            }
            let ev = PointerEvent::new(p.hits, Phase::Secondary, id, m.pos, 1, m.mods);
            host.deliver(&ev).outcome()
        };
        let out = match route {
            SecondaryRoute::Nothing => Outcome::Ignored,
            SecondaryRoute::Swallowed => Outcome::Consumed,
            SecondaryRoute::Target(id) => at(host, id),
            SecondaryRoute::Outside { backdrop, reopen } => {
                host.pointer_parts().router.state.press(backdrop);
                let out = Self::release(host, m, Some(backdrop), layer, now).or(Outcome::Changed);

                match reopen {
                    Some(id) => out.or(at(host, id)),
                    None => out,
                }
            }
        };
        if layer && out == Outcome::Ignored {
            Outcome::Consumed
        } else {
            out
        }
    }

    fn wheel<H: PointerHost + ?Sized>(host: &mut H, m: Mouse) -> Outcome {
        let axis = if matches!(m.kind, MouseKind::WheelLeft | MouseKind::WheelRight) {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };
        let p = host.pointer_parts();
        let delta = p.router.state.wheel_delta(m.kind);

        let events: Vec<PointerEvent> = p
            .hits
            .scroll_stack(m.pos)
            .into_iter()
            .map(|id| PointerEvent::wheel_on(p.hits, id, axis, delta, m.pos, m.mods))
            .collect();
        let mut out = Outcome::Ignored;
        let mut tried: Vec<WidgetId> = Vec::new();
        for ev in events {
            if tried.contains(&ev.owner()) {
                continue;
            }
            tried.push(ev.owner());
            out = host.deliver(&ev).outcome();
            if out != Outcome::Ignored {
                break;
            }
        }

        host.pointer_parts().router.state.scrolled(m.pos, out);
        out
    }

    fn release<H: PointerHost + ?Sized>(
        host: &mut H,
        m: Mouse,
        hit: Option<WidgetId>,
        layer: bool,
        now: u64,
    ) -> Outcome {
        let p = host.pointer_parts();
        let pressed = p.router.state.pressed;
        let id = match p.router.state.release(hit) {
            Release::Click(id) => id,
            Release::Outside => return host.outside(m.pos).outcome().or(Outcome::Changed),
            Release::Cancelled => {
                let Some(pressed) = pressed else {
                    return Outcome::Changed;
                };
                let ev = p
                    .router
                    .held_event(p.hits, Phase::Release, pressed, m.pos, m.mods);
                return host.deliver(&ev).outcome().or(Outcome::Changed);
            }
        };

        p.router.state.flash(id, now);
        let count = p.router.state.count_click(id, m.pos, now);
        let ev = PointerEvent::new(p.hits, Phase::Click, id, m.pos, count, m.mods);
        let before = p.focus.moves();
        let out = host.deliver(&ev).outcome();

        let p = host.pointer_parts();
        if !layer
            && p.router.state.config.focus_on == FocusOn::Release
            && p.focus.moves() == before
            && p.ring.contains(id)
        {
            p.focus.focus(id);
        }
        out.or(Outcome::Changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn a_backdrop_click_needs_press_and_release_on_the_backdrop() {
        let mut hits = HitRegistry::default();
        hits.register_surface(WidgetId::of("dlg"), Rect::new(10, 5, 20, 6));
        let inside = Position::new(12, 6);
        let outside = Position::new(1, 1);
        let mut p = PointerState::default();
        p.on_down(&hits, inside);
        assert_eq!(p.release(hits.hit(outside)), Release::Cancelled);
        p.on_down(&hits, outside);
        assert_eq!(p.release(hits.hit(inside)), Release::Cancelled);
        p.on_down(&hits, outside);
        assert_eq!(p.release(hits.hit(Position::new(2, 2))), Release::Outside);
    }

    fn hits() -> (HitRegistry, WidgetId, WidgetId) {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let b = WidgetId::of("b");
        h.register(a, Rect::new(0, 0, 10, 1));
        h.register(b, Rect::new(0, 1, 10, 1));
        (h, a, b)
    }

    #[test]
    fn a_release_completes_a_press_from_elsewhere_only_where_accepted() {
        let (mut h, a, b) = hits();
        let mut p = PointerState::default();
        p.on_down(&h, Position::new(1, 0));
        let (hit, _) = p.on_up(&h, Position::new(1, 1));
        assert_eq!(p.release(hit), Release::Cancelled, "not accepted");
        h.accept_release(b, ReleaseFrom::Press(a));
        p.on_down(&h, Position::new(1, 0));
        let (hit, _) = p.on_up(&h, Position::new(1, 1));
        assert_eq!(p.release(hit), Release::Click(b), "press-drag-release");

        p.on_down(&h, Position::new(1, 1));
        p.press(WidgetId::of("c"));
        let (hit, _) = p.on_up(&h, Position::new(1, 1));
        assert_eq!(p.release(hit), Release::Cancelled);
    }

    #[test]
    fn a_secondary_release_chooses_an_accepting_row_after_movement() {
        let (mut h, a, b) = hits();
        h.accept_release(b, ReleaseFrom::Secondary);
        let mut p = PointerState::default();

        p.on_secondary(&h, Position::new(1, 1));
        assert_eq!(p.on_secondary_up(&h, Position::new(1, 1)), None);

        p.on_secondary(&h, Position::new(1, 0));
        assert_eq!(
            p.on_secondary_drag(&h, Position::new(2, 1)),
            Outcome::Changed
        );
        assert_eq!(p.hover, Some(b));
        assert_eq!(p.on_secondary_up(&h, Position::new(2, 1)), Some(b));

        p.on_secondary(&h, Position::new(1, 1));
        assert_eq!(p.on_secondary_up(&h, Position::new(3, 0)), None);
        let _ = a;

        assert_eq!(p.on_secondary_up(&h, Position::new(2, 1)), None);
    }

    #[test]
    fn motion_is_reported_after_the_first_buttonless_move() {
        let (h, _, _) = hits();
        let mut p = PointerState::default();
        p.on_down(&h, Position::new(1, 0));
        p.on_drag(&h, Position::new(2, 0));
        assert!(
            !p.reports_motion(),
            "drags report motion with a button held"
        );
        p.on_move(&h, Position::new(3, 0));
        assert!(p.reports_motion());
    }

    #[test]
    fn click_counting_respects_time_slop_and_target() {
        let (_, a, b) = hits();
        let mut p = PointerState::default();
        let at = Position::new(2, 0);
        assert_eq!(p.count_click(a, at, 0), 1);
        assert_eq!(p.count_click(a, at, 300), 2);
        assert_eq!(p.count_click(a, at, 600), 3);

        assert_eq!(p.count_click(a, at, 1200), 1);

        assert_eq!(p.count_click(a, Position::new(8, 0), 1300), 1);

        assert_eq!(p.count_click(b, Position::new(8, 0), 1400), 1);
        let mut slow = PointerState::new(PointerConfig::DEFAULT.double_click_ms(1000));
        slow.count_click(a, at, 0);
        assert_eq!(slow.count_click(a, at, 800), 2);
    }

    #[test]
    fn down_replaces_a_stale_press_and_release_resolves() {
        let (h, a, b) = hits();
        let mut p = PointerState::default();
        p.on_down(&h, Position::new(1, 0));

        assert_eq!(p.on_down(&h, Position::new(1, 1)), Some(b));
        assert_eq!(p.release(Some(b)), Release::Click(b));
        p.on_down(&h, Position::new(1, 0));
        assert_eq!(p.release(Some(b)), Release::Cancelled);
        p.on_down(&h, Position::new(30, 30));
        assert_eq!(p.release(None), Release::Outside);
        p.on_down(&h, Position::new(1, 0));
        assert_eq!(p.release(None), Release::Cancelled);
        let _ = a;
    }

    #[test]
    fn a_press_on_a_scrollbar_grabs_it_until_release() {
        use super::super::hit::{Axis, Bar};
        let mut h = HitRegistry::default();
        let owner = WidgetId::of("list");
        let bar = Bar {
            id: owner.sub("scrollbar"),
            owner,
            axis: Axis::Vertical,
            track: Rect::new(9, 0, 1, 10),
        };
        h.register_bar(bar);
        let mut p = PointerState::default();
        p.on_down(&h, Position::new(9, 3));
        assert_eq!(p.grabbed(), Some(bar));
        assert_eq!(p.release(Some(bar.id)), Release::Click(bar.id));
        assert_eq!(p.grabbed(), None);
    }

    #[test]
    fn a_held_track_press_repeats_after_a_delay_while_over_the_track() {
        use super::super::hit::{Axis, Bar};
        let mut h = HitRegistry::default();
        let owner = WidgetId::of("list");
        let bar = Bar {
            id: owner.sub("scrollbar"),
            owner,
            axis: Axis::Vertical,
            track: Rect::new(9, 0, 1, 10),
        };
        h.register_bar(bar);
        let mut p = PointerState::default();
        let at = Position::new(9, 8);
        p.on_down(&h, at);
        assert!(p.track_repeating());
        assert_eq!(p.track_repeat_due(1000), None, "the clock starts");
        assert_eq!(p.track_repeat_due(1299), None, "not before the delay");
        assert_eq!(p.track_repeat_due(1300), Some(at));

        p.on_down(&h, at);
        assert_eq!(p.track_repeat_due(1340), None);
        assert_eq!(p.track_repeat_due(1350), Some(at));

        p.on_drag(&h, Position::new(3, 8));
        assert!(!p.track_repeating());
        assert_eq!(p.track_repeat_due(2000), None);
        p.on_drag(&h, at);
        assert_eq!(
            p.track_repeat_due(2000),
            None,
            "back on it: the delay again"
        );

        p.release(Some(bar.id));
        assert!(!p.track_repeating());

        let mut off = PointerState::new(PointerConfig::DEFAULT.track_repeat(false));
        off.on_down(&h, at);
        assert!(!off.track_repeating());
    }

    #[test]
    fn keyboard_suppresses_hover_until_the_pointer_moves() {
        let (h, a, _) = hits();
        let mut p = PointerState::default();
        assert_eq!(p.on_move(&h, Position::new(1, 0)), Outcome::Changed);
        assert_eq!(p.hover, Some(a));
        assert_eq!(p.on_move(&h, Position::new(2, 0)), Outcome::Ignored);
        p.suppress_hover();
        assert_eq!(p.on_move(&h, Position::new(2, 0)), Outcome::Changed);
        assert!(!p.hover_suppressed);
    }

    #[test]
    fn flash_runs_out_and_hover_refreshes_after_a_scroll() {
        let (h, _, b) = hits();
        let mut p = PointerState::default();
        let a = WidgetId::of("a");
        p.flash(a, 1000);
        assert_eq!(p.flashing(1100), Some(a));
        assert!(!p.expire_flash(1100));
        assert_eq!(p.flashing(1140), None);
        assert!(p.expire_flash(1140));
        p.scrolled(Position::new(1, 1), Outcome::Changed);
        p.refresh_hover(&h);
        assert_eq!(p.hover, Some(b));
        let mut off = PointerState::new(PointerConfig::DEFAULT.hover_refresh_after_scroll(false));
        off.scrolled(Position::new(1, 1), Outcome::Changed);
        off.refresh_hover(&h);
        assert_eq!(off.hover, None);
    }

    #[test]
    fn a_held_press_past_its_body_edge_autoscrolls_on_a_timer() {
        let mut h = HitRegistry::default();
        let cell = WidgetId::of("cell");
        h.register(cell, Rect::new(0, 0, 10, 5));
        h.register_autoscroll(Rect::new(0, 0, 10, 5));
        let mut p = PointerState::default();
        p.on_down(&h, Position::new(2, 2));

        p.on_drag(&h, Position::new(3, 4));
        assert!(!p.autoscrolling());
        assert_eq!(p.autoscroll_due(0), None);

        p.on_drag(&h, Position::new(3, 7));
        assert!(p.autoscrolling());
        assert_eq!(p.autoscroll_due(100), Some((cell, Position::new(3, 7))));
        assert_eq!(p.autoscroll_due(120), None);
        assert_eq!(p.autoscroll_due(150), Some((cell, Position::new(3, 7))));

        p.release(Some(cell));
        assert!(!p.autoscrolling());
        assert_eq!(p.autoscroll_due(500), None);

        let mut off = PointerState::new(PointerConfig::DEFAULT.autoscroll(false));
        off.on_down(&h, Position::new(2, 2));
        off.on_drag(&h, Position::new(3, 7));
        assert_eq!(off.autoscroll_due(100), None);
    }

    #[test]
    fn autoscroll_areas_under_a_modal_barrier_are_unreachable() {
        let mut h = HitRegistry::default();
        h.register_autoscroll(Rect::new(0, 0, 10, 5));
        assert!(h.autoscroll_at(Position::new(1, 1)).is_some());
        h.push_barrier();
        assert_eq!(h.autoscroll_at(Position::new(1, 1)), None);
    }
}

#[cfg(test)]
mod router_tests {
    use super::*;
    use ratatui::layout::Rect;

    struct Host {
        router: PointerRouter,
        hits: HitRegistry,
        focus: Focus,
        ring: FocusRing,
        layer: bool,
        now: u64,
        log: Vec<PointerEvent>,

        deaf: Vec<WidgetId>,
        outside: usize,
    }

    impl Host {
        fn new(hits: HitRegistry) -> Self {
            Self {
                router: PointerRouter::default(),
                hits,
                focus: Focus::default(),
                ring: FocusRing::default(),
                layer: false,
                now: 1_000,
                log: Vec::new(),
                deaf: Vec::new(),
                outside: 0,
            }
        }

        fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
            PointerRouter::route(self, Mouse::new(kind, Position::new(x, y)))
        }

        fn click(&mut self, x: u16, y: u16) {
            self.mouse(MouseKind::Down, x, y);
            self.mouse(MouseKind::Up, x, y);
        }

        fn phases(&self) -> Vec<(Phase, WidgetId)> {
            self.log
                .iter()
                .filter_map(|e| e.phase().map(|p| (p, e.owner())))
                .collect()
        }
    }

    impl PointerHost for Host {
        fn pointer_parts(&mut self) -> PointerParts<'_> {
            PointerParts {
                router: &mut self.router,
                hits: &self.hits,
                focus: &mut self.focus,
                ring: &self.ring,
            }
        }
        fn layer_open(&self) -> bool {
            self.layer
        }
        fn now_ms(&self) -> u64 {
            self.now
        }
        fn deliver(&mut self, ev: &PointerEvent) -> Response<()> {
            self.log.push(*ev);
            if self.deaf.contains(&ev.owner()) {
                Response::ignored()
            } else {
                Response::changed()
            }
        }
        fn outside(&mut self, _pos: Position) -> Response<()> {
            self.outside += 1;
            Response::changed()
        }
    }

    fn layout() -> (HitRegistry, WidgetId, WidgetId) {
        let mut h = HitRegistry::default();
        let body = WidgetId::of("body");
        let list = WidgetId::of("list");
        h.register(body, Rect::new(0, 0, 20, 10));
        h.register(list, Rect::new(0, 0, 10, 3));
        for i in 0..3 {
            h.register_part(
                list,
                PartRef::item(Part::ROW, ItemKey::index(i)),
                Rect::new(0, i as u16, 10, 1),
            );
        }
        (h, list, body)
    }

    #[test]
    fn a_click_resolves_to_the_owner_and_its_part() {
        let (h, list, _) = layout();
        let mut host = Host::new(h);
        host.click(2, 1);
        assert_eq!(
            host.phases(),
            vec![(Phase::Press, list), (Phase::Click, list)]
        );
        let click = host.log[1];
        assert_eq!(click.index_of(Part::ROW), Some(1));
        assert_eq!(click.count(), 1);
        assert_eq!(host.router.flashing(host.now), Some(click.id));
        host.now += 100;
        host.click(2, 1);
        assert_eq!(host.log[3].count(), 2, "a double click counts");
    }

    #[test]
    fn a_drag_ends_with_drag_end_and_a_release_away_from_the_press() {
        let (h, list, body) = layout();
        let mut host = Host::new(h);
        host.mouse(MouseKind::Down, 2, 0);
        host.mouse(MouseKind::Drag, 2, 2);
        host.mouse(MouseKind::Up, 15, 8);
        assert_eq!(
            host.phases(),
            vec![
                (Phase::Press, list),
                (Phase::Drag, list),
                (Phase::DragEnd, list),
                (Phase::Release, list),
            ]
        );
        assert_eq!(host.log[1].index_of(Part::ROW), Some(0), "the held row");
        let _ = body;
    }

    #[test]
    fn a_drag_keeps_its_owner_after_the_pressed_region_scrolls_away() {
        let (h, list, _) = layout();
        let mut host = Host::new(h);
        host.mouse(MouseKind::Down, 2, 0);

        let mut h = HitRegistry::default();
        h.register(WidgetId::of("body"), Rect::new(0, 0, 20, 10));
        h.register(list, Rect::new(0, 0, 10, 3));
        host.hits = h;
        host.mouse(MouseKind::Drag, 2, 5);
        host.mouse(MouseKind::Up, 2, 5);
        assert_eq!(
            host.phases()[1..],
            [
                (Phase::Drag, list),
                (Phase::DragEnd, list),
                (Phase::Release, list)
            ]
        );
        assert_eq!(host.log[1].index_of(Part::ROW), Some(0), "the held row");
    }

    #[test]
    fn the_wheel_moves_wheel_lines_on_both_axes_and_passes_to_the_next_owner() {
        let (h, list, body) = layout();
        let mut host = Host::new(h);
        host.mouse(MouseKind::WheelRight, 2, 1);
        assert_eq!(host.log.len(), 1, "the list took it");
        assert_eq!(host.log[0].wheel(), Some((Axis::Horizontal, 3)));
        assert_eq!(host.log[0].owner(), list);
        host.log.clear();
        host.deaf.push(list);
        host.mouse(MouseKind::WheelUp, 2, 1);
        let owners: Vec<WidgetId> = host.log.iter().map(PointerEvent::owner).collect();
        assert_eq!(owners, vec![list, body], "one offer per owner");
        assert_eq!(host.log[1].wheel(), Some((Axis::Vertical, -3)));
    }

    #[test]
    fn a_middle_click_reaches_the_control_and_a_layer_swallows_the_rest() {
        let (h, list, body) = layout();
        let mut host = Host::new(h);
        host.mouse(MouseKind::Middle, 2, 2);
        assert_eq!(host.phases(), vec![(Phase::Middle, list)]);
        host.layer = true;
        host.deaf.extend([list, body]);
        assert_eq!(host.mouse(MouseKind::Middle, 2, 2), Outcome::Consumed);
        assert_eq!(host.mouse(MouseKind::WheelDown, 2, 2), Outcome::Consumed);
        assert_eq!(host.mouse(MouseKind::Middle, 40, 40), Outcome::Consumed);
    }

    #[test]
    fn a_click_on_nothing_is_an_outside_click() {
        let (h, _, _) = layout();
        let mut host = Host::new(h);
        host.click(40, 40);
        assert_eq!(host.outside, 1);
        assert!(host.log.is_empty());
    }

    #[test]
    fn a_press_focuses_a_ring_member_unless_a_layer_is_open() {
        let (h, list, _) = layout();
        let mut host = Host::new(h);
        host.ring.register(list);
        host.layer = true;
        host.mouse(MouseKind::Down, 2, 15);
        host.mouse(MouseKind::Down, 1, 4);
        assert_eq!(host.focus.current(), None);
        host.layer = false;
        host.hits
            .register(WidgetId::of("x"), Rect::new(30, 0, 1, 1));
        let mut h2 = HitRegistry::default();
        h2.register(list, Rect::new(0, 0, 10, 3));
        host.hits = h2;
        host.mouse(MouseKind::Down, 2, 1);
        assert_eq!(host.focus.current(), Some(list));
        assert!(host.router.pointer_focus, "focused by a click");
    }

    #[test]
    fn a_secondary_release_on_an_accepting_row_is_a_click() {
        let (mut h, list, _) = layout();
        let row = list.part(PartRef::item(Part::ROW, ItemKey::index(2)));
        h.accept_release(row, ReleaseFrom::Secondary);
        let mut host = Host::new(h);
        host.mouse(MouseKind::Secondary, 2, 0);
        host.mouse(MouseKind::SecondaryDrag, 2, 2);
        host.mouse(MouseKind::SecondaryUp, 2, 2);
        assert_eq!(
            host.phases(),
            vec![(Phase::Secondary, list), (Phase::Click, list)]
        );
        assert_eq!(host.log[1].index_of(Part::ROW), Some(2));
    }

    #[test]
    fn a_press_on_a_bar_carries_it_and_a_held_track_repeats_on_the_tick() {
        let mut h = HitRegistry::default();
        let owner = WidgetId::of("pane");
        let bar = Bar {
            id: owner.sub("scrollbar"),
            owner,
            axis: Axis::Vertical,
            track: Rect::new(9, 0, 1, 10),
        };
        h.register(owner, Rect::new(0, 0, 9, 10));
        h.register_bar(bar);
        let mut host = Host::new(h);
        host.mouse(MouseKind::Down, 9, 8);
        assert_eq!(host.log[0].bar, Some(bar));
        assert_eq!(host.log[0].owner(), owner);
        assert_eq!(host.log[0].part(), PartRef::of(Part::TRACK));
        PointerRouter::tick(&mut host);
        host.now += 400;
        PointerRouter::tick(&mut host);
        assert_eq!(host.log.len(), 2, "the held press replays");
        assert!(host.log[1].is(Phase::Press));
    }

    #[test]
    fn a_backdrop_resolves_to_its_popup() {
        let mut h = HitRegistry::default();
        let owner = WidgetId::of("select");
        h.register(WidgetId::of("page"), Rect::new(0, 0, 20, 10));
        h.push_barrier();
        h.register_backdrop_of(
            owner.sub("popup.backdrop"),
            owner,
            Rect::new(0, 0, 20, 10),
            false,
        );
        let mut host = Host::new(h);
        host.click(5, 5);
        assert_eq!(host.log[1].owner(), owner);
        assert_eq!(host.log[1].part(), PartRef::of(Part::BACKDROP));
    }
}
