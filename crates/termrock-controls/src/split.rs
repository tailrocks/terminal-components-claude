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

const BALANCED: u8 = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitAction {
    Resized(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitCmd {
    Shrink,
    Grow,
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
    pub const fn new(percent: u8) -> Self {
        SplitPaneState {
            percent: clamp_percent(percent),
            maximized: Maximized::None,
        }
    }

    pub const fn percent(&self) -> u8 {
        self.percent
    }

    pub const fn set_percent(&mut self, p: u8) {
        self.percent = clamp_percent(p);
    }

    pub const fn maximized(&self) -> Maximized {
        self.maximized
    }

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
    Offset(u16),
}

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
    pub const PARTS: &'static [Part] = &[Part::SEAM];

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

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn gap(mut self, g: u16) -> Self {
        self.gap = g;
        self
    }

    #[must_use]
    pub const fn seam_start(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::Start));
        self
    }

    #[must_use]
    pub const fn seam_center(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::Center));
        self
    }

    #[must_use]
    pub const fn seam_end(mut self, width: u16) -> Self {
        self.seam = Some((width, SeamAlign::End));
        self
    }

    #[must_use]
    pub const fn seam_at_offset(mut self, width: u16, offset: u16) -> Self {
        self.seam = Some((width, SeamAlign::Offset(offset)));
        self
    }

    #[must_use]
    pub const fn min_first(mut self, n: u16) -> Self {
        self.min_first = n;
        self
    }

    #[must_use]
    pub const fn min_second(mut self, n: u16) -> Self {
        self.min_second = n;
        self
    }

    #[must_use]
    pub const fn resizable(mut self, yes: bool) -> Self {
        self.resizable = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn model(&self, st: SplitPaneState) -> SplitModel {
        let mut m = SplitModel::new(self.axis, st.percent, self.min_first, self.min_second);
        m.maximized = st.maximized;
        m
    }

    pub(crate) fn panes(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        self.model(st).layout(area, self.gap)
    }

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
                SeamAlign::Offset(offset) => offset.min(spare),
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

    const fn table(&self) -> &'static [Binding<SplitCmd>] {
        if self.disabled || !self.resizable {
            return &[];
        }
        match self.axis {
            SplitAxis::Horizontal => HORIZONTAL,
            SplitAxis::Vertical => VERTICAL,
        }
    }

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

    #[test]
    fn explicit_seam_offset_matches_master_detail_odd_and_even_gaps() {
        let area = Rect::new(5, 3, 120, 40);
        for (gap, expected_x) in [(2, 43), (3, 43), (4, 44), (5, 43)] {
            let split = SplitPane::new(ID, SplitAxis::Horizontal)
                .gap(gap)
                .seam_at_offset(1, gap / 2);
            assert_eq!(
                split.seam(SplitPaneState::new(32), area),
                Rect::new(expected_x, area.y, 1, area.height),
                "gap {gap}"
            );
        }
        let centered = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(2)
            .seam_center(1);
        assert_eq!(
            centered.seam(SplitPaneState::new(32), area),
            Rect::new(42, area.y, 1, area.height),
            "existing centered seam contract remains unchanged"
        );
    }

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
