//! `SplitPane` — the two-pane split and its draggable seam
//! (`COMPONENT_ARCHITECTURE.md` §10, §14.1, §18.2 `splitter`, §18.3 item 20,
//! Appendix A 4E).
//!
//! The legacy `Splitter` widget and the `ui::layout::Split` value type are
//! **one** component here: the seam cannot register a hit region without the
//! container rect, and the container rect cannot be known without laying the
//! split out, so keeping them apart forced every caller to cache a
//! `seam_container: Rect` between the two phases. `SplitPane::draw` owns that
//! rect and reports it, and `update` reads it back through `Cx::area`.

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layout::{Maximized, SplitAxis, SplitModel};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// The percent a [`SplitCmd::Reset`] returns the seam to.
const BALANCED: u8 = 50;

/// What a split reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitAction {
    /// The seam moved; carries the new percent of the first pane.
    Resized(u8),
}

/// The const-constructible commands of the split keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitCmd {
    /// Give the first pane one cell less.
    Shrink,
    /// Give the first pane one cell more.
    Grow,
    /// Balance the panes and clear any maximised pane.
    Reset,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: SplitCmd,
    label: &'static str,
) -> Binding<SplitCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: 40,
        visible: true,
    }
}

const HORIZONTAL: &[Binding<SplitCmd>] = &[
    b(
        "split.shrink",
        Chord::key(KeyCode::Left),
        SplitCmd::Shrink,
        "Narrower",
    ),
    b(
        "split.grow",
        Chord::key(KeyCode::Right),
        SplitCmd::Grow,
        "Wider",
    ),
    b(
        "split.reset",
        Chord::key(KeyCode::Home),
        SplitCmd::Reset,
        "Balance",
    ),
];

const VERTICAL: &[Binding<SplitCmd>] = &[
    b(
        "split.shrink",
        Chord::key(KeyCode::Up),
        SplitCmd::Shrink,
        "Shorter",
    ),
    b(
        "split.grow",
        Chord::key(KeyCode::Down),
        SplitCmd::Grow,
        "Taller",
    ),
    b(
        "split.reset",
        Chord::key(KeyCode::Home),
        SplitCmd::Reset,
        "Balance",
    ),
];

/// Durable state of a [`SplitPane`]: where the seam sits and which pane, if
/// any, is maximised.
///
/// The axis, the gap and the minima are **props**, not state: they are
/// configuration the caller writes once, and only the percent and the
/// maximise state are moved by interaction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SplitPaneState {
    percent: u8,
    maximized: Maximized,
}

impl Default for SplitPaneState {
    fn default() -> Self {
        SplitPaneState {
            percent: BALANCED,
            maximized: Maximized::None,
        }
    }
}

impl SplitPaneState {
    /// A split with `percent` of the usable length given to the first pane,
    /// clamped to `5..=95` by [`SplitModel`].
    pub const fn new(percent: u8) -> Self {
        SplitPaneState {
            percent: clamp_percent(percent),
            maximized: Maximized::None,
        }
    }

    /// The percent of the usable length given to the first pane.
    pub const fn percent(&self) -> u8 {
        self.percent
    }

    /// Move the seam. Clamped to `5..=95` and to the minima when the split
    /// is next laid out.
    pub const fn set_percent(&mut self, p: u8) {
        self.percent = clamp_percent(p);
    }

    /// Which pane, if any, fills the whole area.
    pub const fn maximized(&self) -> Maximized {
        self.maximized
    }

    /// Maximise `which`, or restore both panes when `which` is already
    /// maximised.
    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }
}

/// Two panes with a draggable seam between them.
///
/// ## Construction
/// `SplitPane::new(id, axis)` — the axis is required because it is not a
/// look but a different layout, and a `bool` for it would be exactly the
/// parameter soup §13 forbids.
///
/// ## Ownership
/// The caller owns a [`SplitPaneState`] (percent and maximise state) and
/// paints both panes in `draw`'s body closure. The runtime owns focus,
/// hover, press and the seam's pointer capture.
///
/// ## Configuration
/// `.gap(u16)` (`1`), `.min_first(u16)` (`1`), `.min_second(u16)` (`1`),
/// `.resizable(bool)` (`false`), `.patch`, `.patch_part`, `.slot`,
/// reference fixtures use [`Ui::reference`](crate::Ui::reference).
///
/// ## Variants
/// `Family::SPLIT`, `Variant::DEFAULT` only.
///
/// ## States
/// The seam wears `HOVERED`, `FOCUSED` and `PRESSED` from the runtime; a
/// live seam capture keeps `PRESSED` for the whole drag. No state is
/// props-derived.
///
/// ## Actions
/// `SplitAction::Resized(u8)` — the seam moved, carrying the new percent.
/// It carries no `ItemKey`: a split has no items. Maximising is not an
/// action because it is not an input the component owns: the caller calls
/// [`SplitPaneState::toggle_max`] and already knows.
///
/// ## Focus
/// One `Focusable` stop over the whole container, registered **only** when
/// `.resizable(true)`, because a seam nobody can move with the keyboard has
/// no business in the ring. It does not swallow typing, opens no scope and
/// traps nothing.
///
/// ## Keyboard
/// Only when `.resizable(true)`; the table is empty otherwise, so the hint
/// bar advertises nothing that does not work. Horizontal: `←` narrower, `→`
/// wider, `Home` balance. Vertical: `↑` shorter, `↓` taller, `Home`
/// balance.
///
/// ## Mouse
/// `PartRef::of(Part::SEAM)`: a press claims pointer capture, drags put the
/// seam under the pointer (clamped by the minima), release ends the
/// capture, and a double-click balances the panes. Nothing else in the
/// container is a hit target of this component's.
///
/// ## Layout
/// [`SplitModel`] does the arithmetic: `gap` cells between the panes, the
/// minima honoured, and **when both minima cannot fit the first pane wins
/// on both axes** — which is also the narrow-collapse mode, so a caller
/// that wants a single pane below a width sets `min_second` and lets the
/// model collapse. `measure` reports the minima plus the gap. `draw`
/// passes the two pane rects to its body; both are `Rect::ZERO`-safe and either may be
/// empty (maximised or collapsed). A degenerate `area` registers nothing
/// and passes two origin-anchored empty rects to the body (R5).
///
/// ## Parts
/// `SEAM` — the gap strip, and the only part this component paints. The
/// panes are the caller's.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `Part::SEAM`. `.slot` is honoured for
/// `Part::SEAM`, which is the whole of this component's painting.
///
/// ## Identity
/// One `Id`; no items and no `ItemKey`.
///
/// ## Testing
/// `SplitPaneCase` with `Caps::FOCUSABLE | Caps::CAPTURES` and a fixture
/// that sets `.resizable(true)`; `render::components::split_pane::*`.
///
/// ## Invariants
/// The seam is painted with the **quiet** rule glyph at rest, the **active**
/// rule glyph while pressed, and `GlyphRole::FocusBar` while focused, so all
/// three states differ by a symbol and survive `ColorLevel::Mono` — the
/// recipe's own `SEAM` rules are colour-only, and R-8 forbids a component
/// assembling a `Style` of its own. The container rect `update` reads back
/// through `Cx::area` is the one `draw` registered, so no caller stores it.
pub struct SplitPane<'a> {
    id: Id,
    axis: SplitAxis,
    gap: u16,
    min_first: u16,
    min_second: u16,
    resizable: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for SplitPane<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPane")
            .field("id", &self.id)
            .field("axis", &self.axis)
            .field("gap", &self.gap)
            .field("min_first", &self.min_first)
            .field("min_second", &self.min_second)
            .field("resizable", &self.resizable)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> SplitPane<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::SEAM];

    /// A split along `axis`.
    pub const fn new(id: Id, axis: SplitAxis) -> Self {
        SplitPane {
            id,
            axis,
            gap: 1,
            min_first: 1,
            min_second: 1,
            resizable: false,
            ov: PartStyle::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Cells between the panes. `0` removes the seam, and with it the drag
    /// affordance.
    #[must_use]
    pub const fn gap(mut self, g: u16) -> Self {
        self.gap = g;
        self
    }

    /// The first pane's minimum length along the axis.
    #[must_use]
    pub const fn min_first(mut self, n: u16) -> Self {
        self.min_first = n;
        self
    }

    /// The second pane's minimum length along the axis.
    #[must_use]
    pub const fn min_second(mut self, n: u16) -> Self {
        self.min_second = n;
        self
    }

    /// Whether the keyboard can move the seam. `false` leaves the split out
    /// of the focus ring and its binding table empty; the seam still drags.
    #[must_use]
    pub const fn resizable(mut self, yes: bool) -> Self {
        self.resizable = yes;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The layout model for `st` under this instance's configuration.
    const fn model(&self, st: SplitPaneState) -> SplitModel {
        let mut m = SplitModel::new(self.axis, st.percent, self.min_first, self.min_second);
        m.maximized = st.maximized;
        m
    }

    /// The two pane rects for `area`, without painting anything.
    pub(crate) fn panes(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        self.model(st).layout(area, self.gap)
    }

    /// The seam strip for `area`; empty when a pane is maximised, when the
    /// split collapsed, or when `gap` is `0`.
    pub(crate) fn seam(&self, st: SplitPaneState, area: Rect) -> Rect {
        self.model(st).handle(area, self.gap)
    }

    /// The binding table: empty unless the split is resizable, so the hint
    /// bar never advertises a chord `update` will not honour.
    const fn table(&self) -> &'static [Binding<SplitCmd>] {
        if !self.resizable {
            return &[];
        }
        match self.axis {
            SplitAxis::Horizontal => HORIZONTAL,
            SplitAxis::Vertical => VERTICAL,
        }
    }

    /// Apply `f` to the model and report the resulting percent.
    fn apply(
        &self,
        st: &mut SplitPaneState,
        f: impl FnOnce(&mut SplitModel),
        acc: &mut Acc<SplitAction>,
    ) {
        let before = *st;
        let mut m = self.model(*st);
        f(&mut m);
        st.percent = m.percent;
        st.maximized = m.maximized;
        if *st == before {
            acc.consumed();
        } else {
            acc.action(SplitAction::Resized(st.percent));
        }
    }

    /// The update phase: keyboard resize (when resizable) and the seam drag.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SplitPaneState) -> Response<SplitAction> {
        let area = cx.area(self.id).unwrap_or(Rect::ZERO);
        let gap = self.gap;
        let table = self.table();
        let mut acc = Acc::<SplitAction>::new();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => match Binding::command(table, action) {
                    Some(SplitCmd::Shrink) => {
                        self.apply(st, |m| m.nudge(area, gap, -1), &mut acc);
                    }
                    Some(SplitCmd::Grow) => self.apply(st, |m| m.nudge(area, gap, 1), &mut acc),
                    Some(SplitCmd::Reset) => self.apply(
                        st,
                        |m| {
                            m.percent = BALANCED;
                            m.maximized = Maximized::None;
                        },
                        &mut acc,
                    ),
                    None => {}
                },
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::SEAM, ..
                        },
                    pos,
                    ..
                } => self.pointer(cx, st, phase, pos, area, &mut acc),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    fn pointer(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SplitPaneState,
        phase: Phase,
        pos: Position,
        area: Rect,
        acc: &mut Acc<SplitAction>,
    ) {
        let gap = self.gap;
        match phase {
            Phase::Press | Phase::DragStart => {
                let _ = cx.capture(self.id, PartRef::of(Part::SEAM));
                acc.consumed();
            }
            Phase::Drag => {
                let capture_area = cx.capture_area().unwrap_or_else(|| self.seam(*st, area));
                let origin = cx.capture_origin().unwrap_or(pos);
                let target = drag_target(self.axis, pos, capture_area, origin);
                self.apply(
                    st,
                    |m| {
                        let _ = m.drag_to(area, gap, target);
                    },
                    acc,
                );
            }
            Phase::Release | Phase::DragEnd => {
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                acc.consumed();
            }
            Phase::DoubleClick => self.apply(
                st,
                |m| {
                    m.percent = BALANCED;
                    m.maximized = Maximized::None;
                },
                acc,
            ),
            Phase::Click | Phase::Secondary => acc.consumed(),
            Phase::Move => {}
        }
    }

    /// The draw phase: register and paint the seam, then run `body` exactly
    /// once with the logical first and second pane rects. All body painting is
    /// clipped to `area`; an empty input yields two empty rects anchored at the
    /// input origin.
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &SplitPaneState,
        body: impl FnOnce(&mut Ui<'_>, Rect, Rect) -> R,
    ) -> R {
        if area.is_empty() {
            let empty = Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            };
            return ui.with_area(empty, |ui| body(ui, empty, empty));
        }
        let (first, second) = self.panes(*st, area);
        let seam = self.seam(*st, area);
        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        if self.resizable {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        ui.publish_bindings(self.id, live, self.table());
        if seam.is_empty() {
            return ui.with_area(area, |ui| body(ui, first, second));
        }
        if let Some(f) = self.ov.slot_for(Part::SEAM) {
            f(ui, seam);
        } else {
            self.paint_seam(ui, seam, live);
        }
        ui.register_part(self.id, PartRef::of(Part::SEAM), seam);
        ui.with_area(area, |ui| body(ui, first, second))
    }

    /// Paint the gap strip.
    ///
    /// The `SEAM` recipe distinguishes hovered, focused and pressed by colour
    /// alone, and `ColorLevel::Mono` erases colour, so the **symbol** carries
    /// all three — the same shape §11.4 gives the mono `PRESSED` bracket and
    /// the mono focus bar, and for the same reason: a `StateRule` binds one
    /// glyph to one part, and only the component knows which of the theme's
    /// three glyphs this strip should be drawn from. R-8 rules out the
    /// alternative, which is assembling a `Style` here.
    ///
    /// A pressed seam beats a focused one: a seam being dragged is being
    /// dragged whatever the focus ring says.
    fn paint_seam(&self, ui: &mut Ui<'_>, seam: Rect, live: StateFlags) {
        let s = self.ov.style(
            ui,
            self.id,
            Family::SPLIT,
            Variant::DEFAULT,
            Part::SEAM,
            live,
        );
        let rule = if live.contains(StateFlags::PRESSED) {
            let set = ui.design().glyphs.rule_active();
            match self.axis {
                SplitAxis::Horizontal => set.vertical,
                SplitAxis::Vertical => set.horizontal,
            }
        } else if live.contains(StateFlags::FOCUSED) {
            ui.glyph_str(GlyphRole::FocusBar)
        } else {
            let set = ui.design().glyphs.rule_quiet();
            match self.axis {
                SplitAxis::Horizontal => set.vertical,
                SplitAxis::Vertical => set.horizontal,
            }
        };
        match s.glyph {
            Slot::Set(g) => {
                for cell in seam.positions() {
                    let one = Rect {
                        x: cell.x,
                        y: cell.y,
                        width: 1,
                        height: 1,
                    };
                    ui.glyph(one, g, s.style);
                }
            }
            Slot::Clear => ui.fill(seam, s.style),
            Slot::Inherit => {
                for cell in seam.positions() {
                    ui.paint_cell(cell, rule, s.style);
                }
            }
        }
    }

    /// The natural size: both minima plus the gap along the axis, and one
    /// cell across it.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let along = self
            .min_first
            .saturating_add(self.min_second)
            .saturating_add(self.gap);
        let min = match self.axis {
            SplitAxis::Horizontal => (along, 1),
            SplitAxis::Vertical => (1, along),
        };
        Size {
            min,
            preferred: c.max,
        }
        .fit(c)
    }
}

const fn clamp_percent(percent: u8) -> u8 {
    if percent < 5 {
        5
    } else if percent > 95 {
        95
    } else {
        percent
    }
}

const fn drag_target(
    axis: SplitAxis,
    pos: Position,
    capture_area: Rect,
    origin: Position,
) -> Position {
    match axis {
        SplitAxis::Horizontal => Position::new(
            pos.x
                .saturating_sub(origin.x.saturating_sub(capture_area.x)),
            pos.y,
        ),
        SplitAxis::Vertical => Position::new(
            pos.x,
            pos.y
                .saturating_sub(origin.y.saturating_sub(capture_area.y)),
        ),
    }
}

impl Bindings for SplitPane<'_> {
    type Cmd = SplitCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<SplitCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::{ColorLevel, Role, Theme};

    const ID: Id = Id::root("split.tests");

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 10,
    };

    #[test]
    fn body_runs_once_for_empty_input_with_anchored_rects() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let calls = Cell::new(0);
        let first = Cell::new(Rect::ZERO);
        let second = Cell::new(Rect::ZERO);
        let area = Rect::new(9, 7, 0, 0);
        let mut answer = 0;
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            answer = SplitPane::new(ID, SplitAxis::Horizontal).draw(
                ui,
                area,
                &SplitPaneState::default(),
                |_, a, b| {
                    calls.set(calls.get() + 1);
                    first.set(a);
                    second.set(b);
                    42
                },
            );
        });
        assert_eq!(answer, 42);
        assert_eq!(calls.get(), 1);
        assert_eq!(first.get(), area);
        assert_eq!(second.get(), area);
    }

    #[test]
    fn seam_is_painted_before_the_body_runs() {
        let painted = Cell::new(false);
        let observed = Cell::new(false);
        let marker = |_: &mut Ui<'_>, _: Rect| painted.set(true);
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            SplitPane::new(ID, SplitAxis::Horizontal)
                .slot(Part::SEAM, &marker)
                .draw(ui, AREA, &SplitPaneState::default(), |_, _, _| {
                    observed.set(painted.get());
                });
        });
        assert!(observed.get());
    }

    #[test]
    fn body_paint_is_clipped_to_the_split_area() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let area = Rect::new(4, 3, 12, 5);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            SplitPane::new(ID, SplitAxis::Horizontal).draw(
                ui,
                area,
                &SplitPaneState::default(),
                |ui, _, _| {
                    let style = ui.surface_style();
                    for row in SCREEN.rows() {
                        ui.paint_str(row, "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", style);
                    }
                },
            );
        });
        for pos in SCREEN.positions() {
            let is_z = buf.cell(pos).is_some_and(|cell| cell.symbol() == "Z");
            assert_eq!(is_z, area.contains(pos), "body clip mismatch at {pos:?}");
        }
    }

    #[test]
    fn state_percent_is_always_clamped() {
        let mut st = SplitPaneState::new(0);
        assert_eq!(st.percent(), 5);
        st.set_percent(100);
        assert_eq!(st.percent(), 95);
    }

    #[test]
    fn resetting_maximization_is_a_full_state_change() {
        let sp = SplitPane::new(ID, SplitAxis::Horizontal);
        let mut st = SplitPaneState::default();
        st.toggle_max(Maximized::First);
        let mut acc = Acc::new();
        sp.apply(&mut st, |model| model.maximized = Maximized::None, &mut acc);
        assert_eq!(acc.finish(ID).into_action(), Some(SplitAction::Resized(50)));
    }

    #[test]
    fn drag_preserves_the_pointer_offset_inside_a_wide_seam() {
        let seam = Rect::new(19, 2, 3, 8);
        assert_eq!(
            drag_target(
                SplitAxis::Horizontal,
                Position::new(25, 6),
                seam,
                Position::new(21, 6),
            ),
            Position::new(23, 6)
        );
        assert_eq!(
            drag_target(
                SplitAxis::Vertical,
                Position::new(6, 14),
                Rect::new(2, 8, 8, 3),
                Position::new(6, 10),
            ),
            Position::new(6, 12)
        );
    }

    /// The two panes and the seam tile the container exactly: no cell belongs
    /// to two of them and no cell belongs to none. This is what lets the
    /// caller delete its `seam_container: Rect` field (§18.2) — the component
    /// is the only thing that knows the partition, so the partition has to be
    /// total.
    #[test]
    fn the_panes_and_the_seam_tile_the_container() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            for percent in [5u8, 30, 50, 95] {
                for gap in [0u16, 1, 2] {
                    let sp = SplitPane::new(ID, axis).gap(gap);
                    let st = SplitPaneState::new(percent);
                    let (a, b) = sp.panes(st, AREA);
                    let seam = sp.seam(st, AREA);
                    let mut seen: Vec<u32> = vec![0; (AREA.width * AREA.height) as usize];
                    for r in [a, b, seam] {
                        for p in r.positions() {
                            let i = (p.y * AREA.width + p.x) as usize;
                            seen[i] += 1;
                        }
                    }
                    assert!(
                        seen.iter().all(|n| *n == 1),
                        "{axis:?} {percent}% gap {gap}: panes {a:?} {b:?} seam {seam:?} \
                         do not tile {AREA:?}"
                    );
                }
            }
        }
    }

    /// `.resizable(false)` — the default, and what the legacy mouse-only
    /// `Splitter` did — must leave the split out of the focus ring **and**
    /// out of the hint bar. A visible chord for a resize that cannot happen
    /// is exactly the drift §16.2 case 20 exists to catch, so the table is
    /// empty rather than merely unreachable.
    #[test]
    fn a_non_resizable_split_declares_no_bindings_and_no_focus_stop() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = SplitPaneState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            SplitPane::new(ID, SplitAxis::Horizontal).draw(ui, a, &st, |_, _, _| ());
        });
        assert!(!rt.ring().is_registered(ID));
        assert!(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .bindings(BindingState::default())
                .is_empty()
        );
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            SplitPane::new(ID, SplitAxis::Horizontal)
                .resizable(true)
                .draw(ui, a, &st, |_, _, _| ());
        });
        assert!(rt.ring().is_registered(ID), "a resizable split has no stop");
        assert_eq!(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .resizable(true)
                .bindings(BindingState::default())
                .len(),
            3
        );
    }

    /// The `SPLIT` recipe separates the seam's default, hovered and pressed
    /// looks by **colour alone**, and `ColorLevel::Mono` erases colour, so
    /// the component owes the symbol and the modifier (§11.4's `PRESSED`
    /// row, same mechanism as the mono bracket). Compares the `(symbol,
    /// modifier)` multiset of the seam cells, colour excluded — conformance
    /// case 9's own comparison, run here so the property is owned by the
    /// component rather than only asserted about it.
    #[test]
    fn the_seam_distinguishes_focus_and_press_without_colour() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let seam_cells = |state: crate::ReferenceState| -> BTreeMap<(String, u16), usize> {
            let mut rt = Runtime::new(Stub::default(), theme.clone());
            let mut buf = Buffer::empty(SCREEN);
            let st = SplitPaneState::default();
            let sp = SplitPane::new(ID, SplitAxis::Horizontal).resizable(true);
            let seam = sp.seam(st, AREA);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                ui.reference(Some(crate::ReferenceTarget::new(ID, state)), |ui| {
                    sp.draw(ui, AREA, &st, |_, _, _| ());
                });
            });
            let mut out = BTreeMap::new();
            for p in seam.positions() {
                if let Some(c) = buf.cell(Position::new(p.x, p.y)) {
                    *out.entry((c.symbol().to_owned(), c.modifier.bits()))
                        .or_insert(0) += 1;
                }
            }
            out
        };
        let base = seam_cells(crate::ReferenceState::default());
        let focused = seam_cells(crate::ReferenceState::FOCUSED);
        let pressed = seam_cells(crate::ReferenceState::PRESSED);
        assert!(!base.is_empty(), "the seam painted nothing");
        assert_ne!(base, focused, "focused is indistinguishable from default");
        assert_ne!(base, pressed, "pressed is indistinguishable from default");
        assert_ne!(
            focused, pressed,
            "pressed is indistinguishable from focused"
        );
    }

    /// The minima are enforced once, in [`SplitModel`], and the component
    /// inherits the documented rule: when both minima cannot fit, the first
    /// pane wins on **both** axes. A caller relies on this for
    /// narrow-collapse, so it is asserted through the component's own API and
    /// not only through the layout primitive.
    #[test]
    fn the_first_pane_wins_when_the_minima_do_not_fit() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let sp = SplitPane::new(ID, axis).min_first(30).min_second(30);
            let st = SplitPaneState::default();
            let (a, b) = sp.panes(st, AREA);
            assert_eq!(a, AREA, "{axis:?}: the first pane did not take the area");
            assert!(b.is_empty(), "{axis:?}: the second pane survived");
            assert!(sp.seam(st, AREA).is_empty(), "{axis:?}: a seam survived");
        }
    }

    /// A reference split is not a live drag target.
    #[test]
    fn a_reference_split_registers_nothing() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = SplitPaneState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            ui.reference(None, |ui| {
                SplitPane::new(ID, SplitAxis::Horizontal)
                    .resizable(true)
                    .draw(ui, a, &st, |_, _, _| ());
            });
        });
        assert!(rt.area_of(ID).is_none());
        assert!(!rt.ring().is_registered(ID));
    }

    /// §33's Invariant P: every declared part is one a drawn split actually
    /// resolves, proven by the property (a per-part patch changes the painted
    /// cells) rather than by reading the const back.
    #[test]
    fn every_declared_part_is_one_a_drawn_split_styles() {
        let st = SplitPaneState::default();
        let render = |patched: Option<Part>| {
            let ps: [(Part, StylePatch); 1] = [(
                patched.unwrap_or(Part::SEAM),
                StylePatch::new().set_fg(Role::Warning).set_bg(Role::Danger),
            )];
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut sp = SplitPane::new(ID, SplitAxis::Horizontal).resizable(true);
                if patched.is_some() {
                    sp = sp.patch_part(&ps);
                }
                sp.draw(ui, AREA, &st, |_, _, _| ());
            });
            buf
        };
        let plain = render(None);
        for part in SplitPane::PARTS {
            assert_ne!(
                render(Some(*part)),
                plain,
                "SplitPane declares {part:?} and paints nothing with it"
            );
        }
    }

    /// §45's Invariant R: `## Overrides` names `Part::SEAM` and nothing else,
    /// so a slot on the seam must replace the strip and a slot on anything
    /// else must be inert.
    #[test]
    fn the_slot_addressable_parts_are_exactly_the_documented_ones() {
        let st = SplitPaneState::default();
        let marker = |ui: &mut Ui<'_>, r: Rect| {
            let s = ui.surface_style();
            ui.paint_str(r, "ZZZZ", s);
        };
        let render = |slot: Option<Part>| {
            let mut rt = Runtime::new(Stub::default(), Theme::junie());
            let mut buf = Buffer::empty(SCREEN);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                let mut sp = SplitPane::new(ID, SplitAxis::Horizontal).resizable(true);
                if let Some(part) = slot {
                    sp = sp.slot(part, &marker);
                }
                sp.draw(ui, AREA, &st, |_, _, _| ());
            });
            buf
        };
        let plain = render(None);
        assert_ne!(
            render(Some(Part::SEAM)),
            plain,
            "`## Overrides` grants a slot on Part::SEAM and it is dropped"
        );
        for part in [Part::CONTAINER, Part::LABEL] {
            assert_eq!(
                render(Some(part)),
                plain,
                "a slot on {part:?} changes cells, and `## Overrides` says it does not"
            );
        }
    }
}
