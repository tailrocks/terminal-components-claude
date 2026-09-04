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
use ratatui_core::style::Modifier;

use super::{Acc, Overrides, SlotFn};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layout::{Maximized, SplitAxis, SplitModel};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, Slot, StylePatch, Variant};
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

const fn b(chord: Chord, cmd: SplitCmd, label: &'static str) -> Binding<SplitCmd> {
    Binding {
        chord,
        cmd,
        label,
        priority: 40,
        visible: true,
    }
}

const HORIZONTAL: &[Binding<SplitCmd>] = &[
    b(Chord::key(KeyCode::Left), SplitCmd::Shrink, "Narrower"),
    b(Chord::key(KeyCode::Right), SplitCmd::Grow, "Wider"),
    b(Chord::key(KeyCode::Home), SplitCmd::Reset, "Balance"),
];

const VERTICAL: &[Binding<SplitCmd>] = &[
    b(Chord::key(KeyCode::Up), SplitCmd::Shrink, "Shorter"),
    b(Chord::key(KeyCode::Down), SplitCmd::Grow, "Taller"),
    b(Chord::key(KeyCode::Home), SplitCmd::Reset, "Balance"),
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
            percent,
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
        self.percent = p;
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
/// paints both panes into the rects `draw` returns. The runtime owns focus,
/// hover, press and the seam's pointer capture.
///
/// ## Configuration
/// `.gap(u16)` (`1`), `.min_first(u16)` (`1`), `.min_second(u16)` (`1`),
/// `.resizable(bool)` (`false`), `.patch`, `.patch_part`, `.slot`,
/// `.state_override`.
///
/// ## Variants
/// `Family::SPLIT`, `Variant::DEFAULT` only.
///
/// ## States
/// The seam wears `HOVERED`, `FOCUSED` and `PRESSED` from the runtime; a
/// live seam capture keeps `PRESSED` for the whole drag. No state is
/// props-derived, so a forced state (A11) replaces the runtime half alone.
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
/// returns the two pane rects; both are `Rect::ZERO`-safe and either may be
/// empty (maximised or collapsed). A degenerate `area` registers nothing
/// and returns two empty rects (R5).
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
/// The seam is painted with the quiet rule glyph, the **active** rule glyph
/// while pressed, and the quiet glyph plus `Modifier::BOLD` while focused,
/// so all three states differ by a symbol or a modifier and survive
/// `ColorLevel::Mono` — the recipe's own `SEAM` rules are colour-only. The
/// container rect `update` reads back through `Cx::area` is the one `draw`
/// registered, so no caller stores it.
pub struct SplitPane<'a> {
    id: Id,
    axis: SplitAxis,
    gap: u16,
    min_first: u16,
    min_second: u16,
    resizable: bool,
    ov: Overrides<'a>,
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
            ov: Overrides::new(),
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
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// The layout model for `st` under this instance's configuration.
    const fn model(&self, st: &SplitPaneState) -> SplitModel {
        let mut m = SplitModel::new(self.axis, st.percent, self.min_first, self.min_second);
        m.maximized = st.maximized;
        m
    }

    /// The two pane rects for `area`, without painting anything.
    pub fn panes(&self, st: &SplitPaneState, area: Rect) -> (Rect, Rect) {
        self.model(st).layout(area, self.gap)
    }

    /// The seam strip for `area`; empty when a pane is maximised, when the
    /// split collapsed, or when `gap` is `0`.
    pub fn seam(&self, st: &SplitPaneState, area: Rect) -> Rect {
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
        let before = st.percent;
        let mut m = self.model(st);
        f(&mut m);
        st.percent = m.percent;
        st.maximized = m.maximized;
        if st.percent == before {
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
                Intent::Key(k) => match Binding::lookup(table, &k) {
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
                    part: PartRef {
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
            Phase::Drag => self.apply(st, |m| drop(m.drag_to(area, gap, pos)), acc),
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
        }
    }

    /// The draw phase: register the container and the seam, paint the seam,
    /// and return the two pane rects for the caller to fill.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SplitPaneState) -> (Rect, Rect) {
        if area.is_empty() {
            return (Rect::ZERO, Rect::ZERO);
        }
        let (first, second) = self.panes(st, area);
        let seam = self.seam(st, area);
        let forced = self.ov.is_forced();
        if !forced {
            if self.resizable {
                ui.register_control(self.id, area, Focusability::Focusable);
            }
            ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        }
        if seam.is_empty() {
            return (first, second);
        }
        let live = self.ov.flags(ui.state(self.id), StateFlags::empty());
        if let Some(f) = self.ov.slot_for(Part::SEAM) {
            f(ui, seam);
        } else {
            self.paint_seam(ui, seam, live);
        }
        if !forced {
            ui.register_part(self.id, PartRef::of(Part::SEAM), seam);
        }
        (first, second)
    }

    /// Paint the gap strip.
    ///
    /// The `SEAM` recipe distinguishes hovered and pressed by colour alone,
    /// which `ColorLevel::Mono` erases, so the symbol carries the press and
    /// a modifier carries the focus — the same shape §11.4 gives the mono
    /// `PRESSED` bracket, and for the same reason: a `StateRule` binds a
    /// style, and only the component knows which of two rule glyphs the
    /// strip should be drawn from.
    fn paint_seam(&self, ui: &mut Ui<'_>, seam: Rect, live: StateFlags) {
        let s = self.ov.style(
            ui,
            self.id,
            Family::SPLIT,
            Variant::DEFAULT,
            Part::SEAM,
            live,
        );
        let pressed = live.contains(StateFlags::PRESSED);
        let set = if pressed {
            ui.design().glyphs.rule_active()
        } else {
            ui.design().glyphs.rule_quiet()
        };
        let rule = match self.axis {
            SplitAxis::Horizontal => set.vertical,
            SplitAxis::Vertical => set.horizontal,
        };
        let style = if live.contains(StateFlags::FOCUSED) {
            s.style.add_modifier(Modifier::BOLD)
        } else {
            s.style
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
                    ui.glyph(one, g, style);
                }
            }
            Slot::Clear => ui.fill(seam, style),
            Slot::Inherit => {
                for cell in seam.positions() {
                    ui.paint_cell(cell, rule, style);
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

impl Bindings for SplitPane<'_> {
    type Cmd = SplitCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<SplitCmd>] {
        self.table()
    }
}
