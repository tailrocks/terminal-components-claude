#![doc = r" `SplitPane` — the two-pane split and its draggable seam"]
#![doc = r" (`COMPONENT_ARCHITECTURE.md` §10, §14.1, §18.2 `splitter`, §18.3 item 20,"]
#![doc = r" Appendix A 4E)."]
#![doc = r""]
#![doc = r" The legacy `Splitter` widget and the `ui::layout::Split` value type are"]
#![doc = r" **one** component here: the seam cannot register a hit region without the"]
#![doc = r" container rect, and the container rect cannot be known without laying the"]
#![doc = r" split out, so keeping them apart forced every caller to cache a"]
#![doc = r" `seam_container: Rect` between the two phases. `SplitPane::draw` owns that"]
#![doc = r" rect and reports it, and `update` reads it back through `Cx::area`."]

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
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};

#[doc = r" The percent a [`SplitCmd::Reset`] returns the seam to."]
const BALANCED: u8 = 50;

#[doc = r" What a split reports."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitAction {
    #[doc = r" The seam moved; carries the new percent of the first pane."]
    Resized(u8),
}

#[doc = r" The const-constructible commands of the split keymap."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitCmd {
    #[doc = r" Give the first pane one cell less."]
    Shrink,
    #[doc = r" Give the first pane one cell more."]
    Grow,
    #[doc = r" Balance the panes and clear any maximised pane."]
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

#[doc = r" Durable state of a [`SplitPane`]: where the seam sits and which pane, if"]
#[doc = r" any, is maximised."]
#[doc = r""]
#[doc = r" The axis, the gap and the minima are **props**, not state: they are"]
#[doc = r" configuration the caller writes once, and only the percent and the"]
#[doc = r" maximise state are moved by interaction."]
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
    #[doc = r" A split with `percent` of the usable length given to the first pane,"]
    #[doc = r" clamped to `5..=95` by [`SplitModel`]."]
    pub const fn new(percent: u8) -> Self {
        SplitPaneState {
            percent: clamp_percent(percent),
            maximized: Maximized::None,
        }
    }

    #[doc = r" The percent of the usable length given to the first pane."]
    pub const fn percent(&self) -> u8 {
        self.percent
    }

    #[doc = r" Move the seam. Clamped to `5..=95` and to the minima when the split"]
    #[doc = r" is next laid out."]
    pub const fn set_percent(&mut self, p: u8) {
        self.percent = clamp_percent(p);
    }

    #[doc = r" Which pane, if any, fills the whole area."]
    pub const fn maximized(&self) -> Maximized {
        self.maximized
    }

    #[doc = r" Maximise `which`, or restore both panes when `which` is already"]
    #[doc = r" maximised."]
    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }
}

#[derive(Clone, Copy, Debug)]
enum SeamAlign {
    Start,
    Center,
    End,
}

#[doc = r" Two panes with a draggable seam between them."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `SplitPane::new(id, axis)` — the axis is required because it is not a"]
#[doc = r" look but a different layout, and a `bool` for it would be exactly the"]
#[doc = r" parameter soup §13 forbids."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" The caller owns a [`SplitPaneState`] (percent and maximise state) and"]
#[doc = r" paints both panes in `draw`'s body closure. The runtime owns focus,"]
#[doc = r" hover, press and the seam's pointer capture."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.gap(u16)` (`1`), `.min_first(u16)` (`1`), `.min_second(u16)` (`1`),"]
#[doc = r" `.resizable(bool)` (`false`), `.disabled(bool)` (`false`),"]
#[doc = r" `.seam_start/center/end(width)` (full gap"]
#[doc = r" by default), `.patch`, `.patch_part`, `.slot`,"]
#[doc = r" reference fixtures use [`Ui::reference`](crate::Ui::reference)."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::SPLIT`, `Variant::DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" The seam wears `HOVERED`, `FOCUSED` and `PRESSED` from the runtime; a"]
#[doc = r" live seam capture keeps `PRESSED` for the whole drag. `DISABLED` comes"]
#[doc = r" from the explicit disabled prop and revokes focus/input admission."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" `SplitAction::Resized(u8)` — the seam moved, carrying the new percent."]
#[doc = r" It carries no `ItemKey`: a split has no items. Maximising is not an"]
#[doc = r" action because it is not an input the component owns: the caller calls"]
#[doc = r" [`SplitPaneState::toggle_max`] and already knows."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" One `Focusable` stop over the whole container by default, or the visible"]
#[doc = r" seam when its width/alignment is configured, registered **only** when"]
#[doc = r" `.resizable(true)`, because a seam nobody can move with the keyboard has"]
#[doc = r" no business in the ring. Disabled instances register an unreachable stop"]
#[doc = r" for shared admission and capture cancellation. It does not swallow typing,"]
#[doc = r" opens no scope and traps nothing."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" Only when `.resizable(true)` and enabled; the table is empty otherwise."]
#[doc = r" The hint bar advertises nothing that does not work. Horizontal: `←` narrower, `→`"]
#[doc = r" wider, `Home` balance. Vertical: `↑` shorter, `↓` taller, `Home`"]
#[doc = r" balance."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" `PartRef::of(Part::SEAM)`: a press claims pointer capture, drags put the"]
#[doc = r" seam under the pointer (clamped by the minima), release ends the"]
#[doc = r" capture, and a double-click balances the panes. Disabled instances reject"]
#[doc = r" all of these actions. Nothing else in the container is a hit target of"]
#[doc = r" this component's."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" [`SplitModel`] does the arithmetic: `gap` cells between the panes, the"]
#[doc = r" minima honoured, and **when both minima cannot fit the first pane wins"]
#[doc = r" on both axes** — which is also the narrow-collapse mode, so a caller"]
#[doc = r" that wants a single pane below a width sets `min_second` and lets the"]
#[doc = r" model collapse. `measure` reports the minima plus the gap. `draw`"]
#[doc = r" passes the two pane rects to its body; both are `Rect::ZERO`-safe and either may be"]
#[doc = r" empty (maximised or collapsed). A degenerate `area` registers nothing"]
#[doc = r" and passes two origin-anchored empty rects to the body (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `SEAM` — the configured strip within the gap (the full gap by default),"]
#[doc = r" and the only part this component paints. The"]
#[doc = r" panes are the caller's."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach `Part::SEAM`. `.slot` is honoured for"]
#[doc = r" `Part::SEAM`, which is the whole of this component's painting."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id`; no items and no `ItemKey`."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `SplitPaneCase` with `Caps::FOCUSABLE | Caps::CAPTURES` and a fixture"]
#[doc = r" that sets `.resizable(true)`; `render::components::split_pane::*`."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" The seam is painted with the **quiet** rule glyph at rest, the **active**"]
#[doc = r" rule glyph while pressed, and `GlyphRole::FocusBar` while focused, so all"]
#[doc = r" three states differ by a symbol and survive `ColorLevel::Mono` — the"]
#[doc = r" recipe's own `SEAM` rules are colour-only, and R-8 forbids a component"]
#[doc = r" assembling a `Style` of its own. The container rect `update` reads back"]
#[doc = r" through published logical layout facts is the one `draw` used, even when"]
#[doc = r" ancestor clipping narrows the visible and hittable seam."]
pub struct SplitPane<'a> {
    id: Id,
    axis: SplitAxis,
    gap: u16,
    seam: Option<(u16, SeamAlign)>,
    min_first: u16,
    min_second: u16,
    resizable: bool,
    disabled: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for SplitPane<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPane")
            .field("id", &self.id)
            .field("axis", &self.axis)
            .field("gap", &self.gap)
            .field("seam", &self.seam)
            .field("min_first", &self.min_first)
            .field("min_second", &self.min_second)
            .field("resizable", &self.resizable)
            .field("disabled", &self.disabled)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> SplitPane<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::SEAM];

    #[doc = r" A split along `axis`."]
    pub const fn new(id: Id, axis: SplitAxis) -> Self {
        SplitPane {
            id,
            axis,
            gap: 1,
            seam: None,
            min_first: 1,
            min_second: 1,
            resizable: false,
            disabled: false,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Cells between the panes. `0` removes the seam, and with it the drag"]
    #[doc = r" affordance."]
    #[must_use]
    pub const fn gap(mut self, g: u16) -> Self {
        self.gap = g;
        self
    }

    #[doc = r" Paint and drag only `width` cells at the start of the gap."]
    #[doc = r" Width is clamped to the gap; zero leaves spacing but removes the seam."]
    #[must_use]
    pub const fn seam_start(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::Start));
        self
    }

    #[doc = r" Paint and drag only `width` cells centred in the gap."]
    #[doc = r" An odd spare cell stays at the end. Pane layout is unchanged."]
    #[must_use]
    pub const fn seam_center(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::Center));
        self
    }

    #[doc = r" Paint and drag only `width` cells at the end of the gap."]
    #[doc = r" For example, `.gap(2).seam_end(1)` leaves the leading gap cell blank."]
    #[must_use]
    pub const fn seam_end(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::End));
        self
    }

    #[doc = r" The first pane's minimum length along the axis."]
    #[must_use]
    pub const fn min_first(mut self, n: u16) -> Self {
        self.min_first = n;
        self
    }

    #[doc = r" The second pane's minimum length along the axis."]
    #[must_use]
    pub const fn min_second(mut self, n: u16) -> Self {
        self.min_second = n;
        self
    }

    #[doc = r" Whether the keyboard can move the seam. `false` leaves the split out"]
    #[doc = r" of the focus ring and its binding table empty; the seam still drags."]
    #[must_use]
    pub const fn resizable(mut self, yes: bool) -> Self {
        self.resizable = yes;
        self
    }

    #[doc = r" Disable keyboard and pointer resizing, preserving layout and painting."]
    #[doc = r" The next publication revokes any existing seam capture."]
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[doc = r" An instance patch over every part."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part instance patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The layout model for `st` under this instance's configuration."]
    const fn model(&self, st: SplitPaneState) -> SplitModel {
        let mut m = SplitModel::new(self.axis, st.percent, self.min_first, self.min_second);
        m.maximized = st.maximized;
        m
    }

    #[doc = r" The two pane rects for `area`, without painting anything."]
    pub(crate) fn panes(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        self.model(st).layout(area, self.gap)
    }

    #[doc = r" The seam strip for `area`; empty when a pane is maximised, when the"]
    #[doc = r" split collapsed, or when `gap` is `0`."]
    pub(crate) fn seam(&self, st: SplitPaneState, area: Rect) -> Rect {
        let mut seam = self.model(st).handle(area, self.gap);
        if let Some((width, align)) = self.seam {
            let extent = match self.axis {
                SplitAxis::Horizontal => seam.width,
                SplitAxis::Vertical => seam.height,
            };
            let width = width.min(extent);
            let spare = extent.saturating_sub(width);
            let offset = match align {
                SeamAlign::Start => 0,
                SeamAlign::Center => spare / 2,
                SeamAlign::End => spare,
            };
            match self.axis {
                SplitAxis::Horizontal => {
                    seam.x = seam.x.saturating_add(offset);
                    seam.width = width;
                }
                SplitAxis::Vertical => {
                    seam.y = seam.y.saturating_add(offset);
                    seam.height = width;
                }
            }
        }
        seam
    }

    #[doc = r" The binding table: empty unless the split is resizable, so the hint"]
    #[doc = r" bar never advertises a chord `update` will not honour."]
    const fn table(&self) -> &'static [Binding<SplitCmd>] {
        if self.disabled || !self.resizable {
            return &[];
        }
        match self.axis {
            SplitAxis::Horizontal => HORIZONTAL,
            SplitAxis::Vertical => VERTICAL,
        }
    }

    #[doc = r" Apply `f` to the model and report the resulting percent."]
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

    #[doc = r" The update phase: keyboard resize (when resizable) and the seam drag."]
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SplitPaneState) -> Response<SplitAction> {
        let area = cx
            .layout(self.id)
            .and_then(|facts| facts.logical_area)
            .unwrap_or(Rect::ZERO);
        let gap = self.gap;
        let table = self.table();
        let mut acc = Acc::<SplitAction>::new();
        for it in cx.intents(self.id) {
            if self.disabled {
                continue;
            }
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

            _ => {}
        }
    }

    #[doc = r" The draw phase: register and paint the seam, then run `body` exactly"]
    #[doc = r" once with the logical first and second pane rects. All body painting is"]
    #[doc = r" clipped to `area`; an empty input yields two empty rects anchored at the"]
    #[doc = r" input origin."]
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &SplitPaneState,
        body: impl FnOnce(&mut Ui<'_>, Rect, Rect) -> R,
    ) -> R {
        ui.report_layout(
            self.id,
            LayoutFacts::new(0, 0, area.height, area.width).with_logical_area(area),
        );
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
        let seam = if self.seam.is_some() {
            seam.intersection(ui.full())
        } else {
            seam
        };
        let own = if self.disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::empty()
        };
        let live = PartStyle::flags(ui.state(self.id), own);
        if self.resizable || self.disabled {
            ui.register_control(
                self.id,
                if self.seam.is_some() { seam } else { area },
                if self.disabled {
                    Focusability::Disabled
                } else {
                    Focusability::Focusable
                },
            );
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

    #[doc = r" Paint the gap strip."]
    #[doc = r""]
    #[doc = r" The `SEAM` recipe distinguishes hovered, focused and pressed by colour"]
    #[doc = r" alone, and `ColorLevel::Mono` erases colour, so the **symbol** carries"]
    #[doc = r" all three — the same shape §11.4 gives the mono `PRESSED` bracket and"]
    #[doc = r" the mono focus bar, and for the same reason: a `StateRule` binds one"]
    #[doc = r" glyph to one part, and only the component knows which of the theme's"]
    #[doc = r" three glyphs this strip should be drawn from. R-8 rules out the"]
    #[doc = r" alternative, which is assembling a `Style` here."]
    #[doc = r""]
    #[doc = r" A pressed seam beats a focused one: a seam being dragged is being"]
    #[doc = r" dragged whatever the focus ring says."]
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

    #[doc = r" The natural size: both minima plus the gap along the axis, and one"]
    #[doc = r" cell across it."]
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
        })
        .commit_presented();
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
        })
        .commit_presented();
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
        })
        .commit_presented();
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

    #[doc = r" The two panes and the seam tile the container exactly: no cell belongs"]
    #[doc = r" to two of them and no cell belongs to none. This is what lets the"]
    #[doc = r" caller delete its `seam_container: Rect` field (§18.2) — the component"]
    #[doc = r" is the only thing that knows the partition, so the partition has to be"]
    #[doc = r" total."]
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

    #[doc = r" `.resizable(false)` — the default, and what the legacy mouse-only"]
    #[doc = r" `Splitter` did — must leave the split out of the focus ring **and**"]
    #[doc = r" out of the hint bar. A visible chord for a resize that cannot happen"]
    #[doc = r" is exactly the drift §16.2 case 20 exists to catch, so the table is"]
    #[doc = r" empty rather than merely unreachable."]
    #[test]
    fn a_non_resizable_split_declares_no_bindings_and_no_focus_stop() {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        let st = SplitPaneState::default();
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            SplitPane::new(ID, SplitAxis::Horizontal).draw(ui, a, &st, |_, _, _| ());
        })
        .commit_presented();
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
        })
        .commit_presented();
        assert!(rt.ring().is_registered(ID), "a resizable split has no stop");
        assert_eq!(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .resizable(true)
                .bindings(BindingState::default())
                .len(),
            3
        );
    }

    #[doc = r" The `SPLIT` recipe separates the seam's default, hovered and pressed"]
    #[doc = r" looks by **colour alone**, and `ColorLevel::Mono` erases colour, so"]
    #[doc = r" the component owes the symbol and the modifier (§11.4's `PRESSED`"]
    #[doc = r" row, same mechanism as the mono bracket). Compares the `(symbol,"]
    #[doc = r" modifier)` multiset of the seam cells, colour excluded — conformance"]
    #[doc = r" case 9's own comparison, run here so the property is owned by the"]
    #[doc = r" component rather than only asserted about it."]
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
            })
            .commit_presented();
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

    #[doc = r" The minima are enforced once, in [`SplitModel`], and the component"]
    #[doc = r" inherits the documented rule: when both minima cannot fit, the first"]
    #[doc = r" pane wins on **both** axes. A caller relies on this for"]
    #[doc = r" narrow-collapse, so it is asserted through the component's own API and"]
    #[doc = r" not only through the layout primitive."]
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

    #[doc = r" A reference split is not a live drag target."]
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
        })
        .commit_presented();
        assert!(rt.area_of(ID).is_none());
        assert!(!rt.ring().is_registered(ID));
    }

    #[doc = r" §33's Invariant P: every declared part is one a drawn split actually"]
    #[doc = r" resolves, proven by the property (a per-part patch changes the painted"]
    #[doc = r" cells) rather than by reading the const back."]
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
            })
            .commit_presented();
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

    #[doc = r" §45's Invariant R: `## Overrides` names `Part::SEAM` and nothing else,"]
    #[doc = r" so a slot on the seam must replace the strip and a slot on anything"]
    #[doc = r" else must be inert."]
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
            })
            .commit_presented();
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
