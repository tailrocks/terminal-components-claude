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
use super::hit::HitRegistry;
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
        hit
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

    /// Secondary button: hover lands on the target, no press is held.
    pub fn on_secondary(&mut self, hits: &HitRegistry, pos: Position) -> Option<WidgetId> {
        self.pos = Some(pos);
        self.pressed = None;
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

    /// Button up: the control under the pointer, plus — when it is a
    /// pass-through backdrop the press started on — the control beneath it
    /// that the same click also reaches.
    pub fn on_up(&self, hits: &HitRegistry, pos: Position) -> (Option<WidgetId>, Option<WidgetId>) {
        let hit = hits.hit(pos);
        let beneath = hit
            .filter(|id| self.pressed == Some(*id))
            .and_then(|id| hits.pass_through_target(id, pos));
        (hit, beneath)
    }

    /// Resolve a release over `hit`, consuming the held press.
    pub fn release(&mut self, hit: Option<WidgetId>) -> Release {
        let pressed = self.pressed.take();
        self.grab = None;
        match hit {
            None if pressed.is_none() => Release::Outside,
            None => Release::Cancelled,
            Some(id) if pressed == Some(id) => Release::Click(id),
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
        let slop = self.config.double_click_slop;
        let count = match self.last_click {
            Some(l)
                if l.id == id
                    && now_ms.saturating_sub(l.at) < self.config.double_click_ms
                    && l.pos.x.abs_diff(pos.x) <= slop
                    && l.pos.y.abs_diff(pos.y) <= slop =>
            {
                l.count.saturating_add(1)
            }
            _ => 1,
        };
        self.last_click = Some(LastClick {
            id,
            pos,
            at: now_ms,
            count,
        });
        count
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
}
