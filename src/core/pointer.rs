//! Shared pointer bookkeeping.
//!
//! Every application used to hand-roll the same state machine in its
//! `on_mouse`: which widget is hovered, which one holds the button, the
//! brief pressed flash after an activation, the "keyboard wins until the
//! pointer moves" suppression, double-click detection and the hover refresh
//! after a wheel scroll. [`PointerState`] owns all of it, configured by
//! [`PointerConfig`], so every app behaves the same and a behaviour change
//! lands once.
//!
//! The state is clock-agnostic: callers pass the current time in
//! milliseconds (`now_ms`), so applications with a simulated clock stay
//! deterministic under test.

use ratatui::layout::Position;

use super::event::{MouseKind, Outcome};
use super::hit::{HitRegistry, ReleaseFrom};
use super::id::WidgetId;

/// When a pointer press moves keyboard focus to the control under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusOn {
    /// On button down (macOS: a click focuses as soon as it lands).
    #[default]
    Press,
    /// On a completed click (button released over the pressed control).
    Release,
}

/// What a secondary (context) click on a light-dismiss popup's backdrop
/// does, i.e. a right-click outside an open menu, select or popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecondaryOutside {
    /// Close the popup, then open the context menu of what is under the
    /// pointer (macOS: the new menu replaces the old one at the new spot).
    #[default]
    Reopen,
    /// Close the popup only.
    Close,
    /// Do nothing: the popup stays open.
    Ignore,
}

/// Where a secondary click goes ([`PointerState::route_secondary`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryRoute {
    /// Nothing reachable under the pointer.
    Nothing,
    /// A control: open its context menu.
    Target(WidgetId),
    /// The backdrop of an open popup. Deliver a click on `backdrop` (it
    /// closes the popup through its own [`crate::ui::popup::Dismiss`]),
    /// then, when `reopen` is set, the secondary click on that control.
    Outside {
        backdrop: WidgetId,
        reopen: Option<WidgetId>,
    },
    /// The backdrop of an open popup under [`SecondaryOutside::Ignore`].
    Swallowed,
}

/// Pointer behaviour knobs. The defaults follow desktop (macOS)
/// conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerConfig {
    /// When a press focuses the control under it.
    pub focus_on: FocusOn,
    /// Longest gap between two clicks that still counts them as one
    /// multi-click (double, triple, ...).
    pub double_click_ms: u64,
    /// How far (in cells, on either axis) the second click of a
    /// multi-click may land from the first.
    pub double_click_slop: u16,
    /// Lines one wheel notch scrolls.
    pub wheel_lines: i32,
    /// After a wheel scroll changes the content, re-resolve hover at the
    /// pointer once the next frame has registered its hit regions.
    pub hover_refresh_after_scroll: bool,
    /// How long the pressed flash shows after an activation.
    pub flash_ms: u64,
    /// A secondary click outside an open popup. Default
    /// [`SecondaryOutside::Reopen`].
    pub secondary_outside: SecondaryOutside,
    /// A press on dead space (no control under the pointer) ends the
    /// focused control's edit in place ([`super::focus::Focus::end_edit`]),
    /// as clicking a macOS window background ends field editing.
    pub end_edit_on_dead_space: bool,
    /// While a press that started inside a drag-selecting body
    /// ([`HitRegistry::register_autoscroll`]) is held with the pointer
    /// past that body's edge, the app tick replays the drag so the content
    /// keeps scrolling and the selection keeps extending without further
    /// mouse motion (macOS `autoscroll:`; GTK and Windows do the same).
    pub autoscroll: bool,
    /// How often an edge autoscroll step repeats. Each step moves as many
    /// rows or columns as the pointer is past the edge, so the speed
    /// scales with the distance.
    pub autoscroll_ms: u64,
}

impl PointerConfig {
    /// The desktop defaults.
    pub const DEFAULT: Self = Self {
        focus_on: FocusOn::Press,
        double_click_ms: 500,
        double_click_slop: 1,
        wheel_lines: 3,
        hover_refresh_after_scroll: true,
        flash_ms: 140,
        secondary_outside: SecondaryOutside::Reopen,
        end_edit_on_dead_space: true,
        autoscroll: true,
        autoscroll_ms: 50,
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
    pub const fn end_edit_on_dead_space(mut self, on: bool) -> Self {
        self.end_edit_on_dead_space = on;
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
}

impl Default for PointerConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// What a button release resolved to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// Pressed and released over the same control: a click.
    Click(WidgetId),
    /// Pressed and released over no control at all: a click outside
    /// everything reachable (under a modal: on its inert backdrop).
    Outside,
    /// The press and the release landed on different targets: nothing
    /// happens (the user dragged off to cancel).
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LastClick {
    id: WidgetId,
    pos: Position,
    at: u64,
    count: u8,
}

/// Hover, press, flash and click-count state shared by every app.
#[derive(Debug, Clone, Default)]
pub struct PointerState {
    pub config: PointerConfig,
    /// The control under the pointer.
    pub hover: Option<WidgetId>,
    /// The control that took the last button down, until it is released.
    pub pressed: Option<WidgetId>,
    /// Hover feedback is hidden after keyboard use until the pointer moves.
    pub hover_suppressed: bool,
    /// Last known pointer position.
    pub pos: Option<Position>,
    /// Focus was last moved by a click (see
    /// [`crate::ui::ctx::FocusVisible`]); cleared by any key.
    pub pointer_focus: bool,
    flash: Option<(WidgetId, u64)>,
    /// The scrollbar the held press started on.
    grab: Option<super::hit::Bar>,
    /// The autoscroll body the held press started in.
    edge: Option<ratatui::layout::Rect>,
    /// When the last autoscroll step ran.
    autoscroll_at: Option<u64>,
    /// Where the held secondary press started (until its release).
    secondary_at: Option<Position>,
    /// The release resolved by [`Self::on_up`] completes the held press on
    /// another control that accepts it ([`HitRegistry::accept_release`]).
    accepted: Option<WidgetId>,
    /// The terminal reported pointer motion with no button held (any-motion
    /// tracking). Until then, hover-only behaviour is not relied on.
    motion_seen: bool,
    last_click: Option<LastClick>,
    hover_refresh: Option<Position>,
}

impl PointerState {
    pub fn new(config: PointerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Keyboard input: the keyboard wins until the pointer moves again.
    pub fn suppress_hover(&mut self) {
        self.hover_suppressed = true;
        self.pointer_focus = false;
    }

    /// The app moved focus because of a click.
    pub fn focused_by_click(&mut self) {
        self.pointer_focus = true;
    }

    /// Pointer moved: hover follows it and suppression ends.
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

    /// Drag with the button held: hover follows, the pressed control (if
    /// any) receives the drag.
    pub fn on_drag(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        // a held press tracks what it would release on, so a hover-only
        // region (a cut title) never cancels the pressed look
        self.hover = hits.hit(pos);
        self.pressed
    }

    /// Button down: whatever is under the pointer is pressed. Any stale
    /// press (a release lost outside the terminal) is replaced.
    pub fn on_down(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        let hit = hits.hit(pos);
        self.pressed = hit;
        self.hover = hit;
        self.grab = hit.and_then(|id| hits.bar(id));
        self.edge = hit.and_then(|_| hits.autoscroll_at(pos));
        self.autoscroll_at = None;
        hit
    }

    /// A held press is past the edge of the body it started in, and edge
    /// autoscroll is on ([`PointerConfig::autoscroll`]): the app ticks at
    /// [`PointerConfig::autoscroll_ms`] while this holds.
    pub fn autoscrolling(&self) -> bool {
        self.config.autoscroll
            && self.pressed.is_some()
            && self.grabbed().is_none()
            && matches!((self.edge, self.pos), (Some(edge), Some(pos)) if !edge.contains(pos))
    }

    /// On an app tick: when an autoscroll step is due, the pressed control
    /// and the pointer position to replay as a drag (the drag handler
    /// scrolls and extends the selection), at most once per
    /// [`PointerConfig::autoscroll_ms`].
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

    /// A press on `id` while a modal is open: when `id` is the modal's own
    /// surface ([`HitRegistry::is_surface`]) it is dead space inside the
    /// modal, and ends the focused field's edit in place like
    /// [`Self::dead_space`] (per [`PointerConfig::end_edit_on_dead_space`]).
    /// `true` when it was the surface.
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

    /// A press that hit no control: ends the focused control's edit in
    /// place, per [`PointerConfig::end_edit_on_dead_space`]. Applications
    /// call it when [`Self::on_down`] returns `None` outside a modal.
    pub fn dead_space(&self, focus: &mut super::focus::Focus) {
        if self.config.end_edit_on_dead_space {
            focus.end_edit();
        }
    }

    /// The scrollbar the held press started on, if any: drags go to its
    /// owner on its axis, through [`crate::widgets::scrollbar::apply`].
    pub fn grabbed(&self) -> Option<super::hit::Bar> {
        self.grab.filter(|b| self.pressed == Some(b.id))
    }

    /// Whether the terminal reports pointer motion with no button held
    /// (any-motion tracking, `?1003`). Some terminals, and tmux without
    /// passthrough, report motion only while a button is held: there hover
    /// never follows the pointer, so hover-driven behaviour (a menu-bar
    /// hover switch) must not be relied on, and click and the arrow keys
    /// do the job ([`crate::widgets::menu::MenuBar::hover_switch`]).
    /// `false` until the first buttonless move arrives.
    pub fn reports_motion(&self) -> bool {
        self.motion_seen
    }

    /// Secondary button: hover lands on the target, no press is held.
    pub fn on_secondary(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        self.pressed = None;
        self.secondary_at = Some(pos);
        let hit = hits.hit(pos);
        self.hover = hit;
        hit
    }

    /// Secondary button, resolved against open popups: on a light-dismiss
    /// backdrop it is an outside click (closing the popup) followed, per
    /// [`PointerConfig::secondary_outside`], by the same secondary click on
    /// the control beneath. Hover lands on the final target.
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

    /// Secondary button moved while held: hover follows the pointer (onto
    /// the rows of the context menu the press opened).
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

    /// Secondary button released: the control a press-drag-release chose,
    /// if any. That is the control under the pointer when it accepts a
    /// secondary release ([`HitRegistry::accept_release`] with
    /// [`ReleaseFrom::Secondary`], an open context menu's rows) and the
    /// pointer moved at least one cell from the press, so the release of a
    /// plain right-click never picks the row it opened under. Deliver it as
    /// a click.
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

    /// Button up: the control under the pointer, plus — when it is a
    /// pass-through backdrop the press started on — the control beneath it
    /// that the same click also reaches. A control that accepts a release
    /// from the held press ([`HitRegistry::accept_release`]) completes the
    /// click in [`Self::release`].
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

    /// Resolve a release over `hit`, consuming the held press.
    pub fn release(&mut self, hit: Option<WidgetId>) -> Release {
        let pressed = self.pressed.take();
        let accepted = self.accepted.take();
        self.grab = None;
        self.edge = None;
        match hit {
            None if pressed.is_none() => Release::Outside,
            None => Release::Cancelled,
            Some(id) if pressed == Some(id) || accepted == Some(id) => Release::Click(id),
            Some(_) => Release::Cancelled,
        }
    }

    /// Hold `id` as pressed (a pass-through click re-dispatched beneath a
    /// backdrop).
    pub fn press(&mut self, id: WidgetId) {
        self.pressed = Some(id);
    }

    /// Count a completed click on `id` at `pos`: 1 for a single click, 2
    /// for a double click, 3 for a triple, ... Clicks chain while they land
    /// on the same control, within [`PointerConfig::double_click_slop`]
    /// cells and [`PointerConfig::double_click_ms`] of the previous one.
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

    /// The count the click a press on `id` at `pos` starts would complete
    /// with ([`Self::count_click`] without recording it): 2 for the second
    /// press of a double click. A text field selects the word on that
    /// press and a drag from it extends by words.
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

    /// Forget the click chain (a modal opened, the content changed).
    pub fn reset_clicks(&mut self) {
        self.last_click = None;
    }

    /// Flash `id` as pressed (activation feedback) from `now_ms`.
    pub fn flash(&mut self, id: WidgetId, now_ms: u64) {
        self.flash = Some((id, now_ms.saturating_add(self.config.flash_ms)));
    }

    /// The flashing control, if its flash has not run out yet.
    pub fn flashing(&self, now_ms: u64) -> Option<WidgetId> {
        self.flash
            .filter(|&(_, until)| now_ms < until)
            .map(|(id, _)| id)
    }

    /// A flash is pending (shown or waiting to be cleared by a tick).
    pub fn has_flash(&self) -> bool {
        self.flash.is_some()
    }

    /// Clear a flash that has run out; `true` when something changed.
    pub fn expire_flash(&mut self, now_ms: u64) -> bool {
        match self.flash {
            Some((_, until)) if now_ms >= until => {
                self.flash = None;
                true
            }
            _ => false,
        }
    }

    /// Lines to scroll for a vertical wheel event (negative = up).
    pub fn wheel_delta(&self, kind: MouseKind) -> i32 {
        match kind {
            MouseKind::WheelUp | MouseKind::WheelLeft => -self.config.wheel_lines,
            _ => self.config.wheel_lines,
        }
    }

    /// A wheel scroll changed the content at `pos`: re-resolve hover once
    /// the next frame's hit regions exist (see [`Self::refresh_hover`]).
    pub fn scrolled(&mut self, pos: Position, out: Outcome) {
        if out == Outcome::Changed && self.config.hover_refresh_after_scroll {
            self.hover_refresh = Some(pos);
        }
    }

    /// Call after a frame has rebuilt `hits`: applies a pending hover
    /// refresh after a scroll.
    pub fn refresh_hover(&mut self, hits: &HitRegistry) {
        if let Some(pos) = self.hover_refresh.take()
            && !self.hover_suppressed
        {
            self.hover = hits.hover_at(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

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
        // a press that started on another control is not accepted
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
        // a plain right-click: press and release on the same cell
        p.on_secondary(&h, Position::new(1, 1));
        assert_eq!(p.on_secondary_up(&h, Position::new(1, 1)), None);
        // press, drag onto the row, release
        p.on_secondary(&h, Position::new(1, 0));
        assert_eq!(
            p.on_secondary_drag(&h, Position::new(2, 1)),
            Outcome::Changed
        );
        assert_eq!(p.hover, Some(b));
        assert_eq!(p.on_secondary_up(&h, Position::new(2, 1)), Some(b));
        // over a row that does not accept it
        p.on_secondary(&h, Position::new(1, 1));
        assert_eq!(p.on_secondary_up(&h, Position::new(3, 0)), None);
        let _ = a;
        // a release with no secondary press held
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
        // too slow
        assert_eq!(p.count_click(a, at, 1200), 1);
        // too far on a large widget
        assert_eq!(p.count_click(a, Position::new(8, 0), 1300), 1);
        // a different control
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
        // the release was lost; the next press replaces the stale one
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
        // inside the body: nothing to do
        p.on_drag(&h, Position::new(3, 4));
        assert!(!p.autoscrolling());
        assert_eq!(p.autoscroll_due(0), None);
        // past the bottom edge: a step now, the next after autoscroll_ms
        p.on_drag(&h, Position::new(3, 7));
        assert!(p.autoscrolling());
        assert_eq!(p.autoscroll_due(100), Some((cell, Position::new(3, 7))));
        assert_eq!(p.autoscroll_due(120), None);
        assert_eq!(p.autoscroll_due(150), Some((cell, Position::new(3, 7))));
        // the release ends it
        p.release(Some(cell));
        assert!(!p.autoscrolling());
        assert_eq!(p.autoscroll_due(500), None);
        // switched off
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
