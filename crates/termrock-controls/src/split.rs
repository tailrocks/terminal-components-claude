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
const BASIS_POINTS_PER_PERCENT: u16 = 100;

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

/// Reset destination for keyboard and pointer reset gestures.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ResetPolicy {
    /// Return to the constructor position held by `SplitPaneState`.
    Initial,
    /// Return to the configured percentage.
    Percent(u8),
    /// Return to a balanced split.
    #[default]
    Balanced,
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
    initial_percent: u8,
    basis_points: u16,
    maximized: Maximized,
    grab_offset: Option<i32>,
}

impl Default for SplitPaneState {
    fn default() -> Self {
        SplitPaneState {
            percent: BALANCED,
            initial_percent: BALANCED,
            basis_points: (BALANCED as u16) * BASIS_POINTS_PER_PERCENT,
            maximized: Maximized::None,
            grab_offset: None,
        }
    }
}

impl SplitPaneState {
    pub const fn new(percent: u8) -> Self {
        SplitPaneState {
            percent: clamp_percent(percent),
            initial_percent: clamp_percent(percent),
            basis_points: (clamp_percent(percent) as u16) * BASIS_POINTS_PER_PERCENT,
            maximized: Maximized::None,
            grab_offset: None,
        }
    }

    pub const fn percent(&self) -> u8 {
        self.percent
    }

    pub const fn set_percent(&mut self, p: u8) {
        self.percent = clamp_percent(p);
        self.basis_points = (clamp_percent(p) as u16) * BASIS_POINTS_PER_PERCENT;
    }

    pub const fn maximized(&self) -> Maximized {
        self.maximized
    }

    pub const fn basis_points(&self) -> u16 {
        self.basis_points
    }

    fn reset(&mut self, policy: ResetPolicy) {
        let percent = match policy {
            ResetPolicy::Initial => self.initial_percent,
            ResetPolicy::Percent(percent) => clamp_percent(percent),
            ResetPolicy::Balanced => BALANCED,
        };
        self.percent = percent;
        self.basis_points = (percent as u16) * BASIS_POINTS_PER_PERCENT;
        self.maximized = Maximized::None;
        self.grab_offset = None;
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
    focusability: Focusability,
    keyboard_navigation: bool,
    hit_padding: u16,
    fixed_second_share: Option<(u8, u16, u16)>,
    reset_policy: ResetPolicy,
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
            .field("focusability", &self.focusability)
            .field("keyboard_navigation", &self.keyboard_navigation)
            .field("hit_padding", &self.hit_padding)
            .field("fixed_second_share", &self.fixed_second_share)
            .field("reset_policy", &self.reset_policy)
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
            focusability: Focusability::Focusable,
            keyboard_navigation: true,
            hit_padding: 0,
            fixed_second_share: None,
            reset_policy: ResetPolicy::Balanced,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn axis(&self) -> SplitAxis {
        self.axis
    }

    /// Smallest extent at which the resolved panes can hold their requested
    /// along-axis sizes. The current state ratio and the pane's configured
    /// minima are included by the shared split model.
    pub fn required_extent(&self, state: &SplitPaneState, first: u16, second: u16) -> u16 {
        let model = self.model(*state);
        let first = first.max(model.min_first);
        let second = second.max(model.min_second);
        if let Some((percent, min, max)) = self.fixed_second_share {
            return model
                .required_extent_with_second_share(self.gap, percent, min, max, first, second);
        }
        model.required_extent_with_basis_points(self.gap, state.basis_points, first, second)
    }

    /// Return the child constraints for the pane rectangles resolved from the
    /// supplied root constraints and current state. Consumers can measure pane
    /// content at its actual split width without reproducing split arithmetic.
    pub fn pane_constraints(
        &self,
        state: &SplitPaneState,
        constraints: Constraints,
    ) -> (Constraints, Constraints) {
        let root = Rect::new(0, 0, constraints.max.0, constraints.max.1);
        let (first, second) = self.panes(*state, root);
        (
            Constraints::loose(first.width, first.height),
            Constraints::loose(second.width, second.height),
        )
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
    pub const fn focusability(mut self, focusability: Focusability) -> Self {
        self.focusability = focusability;
        self
    }

    #[must_use]
    pub const fn keyboard_navigation(mut self, enabled: bool) -> Self {
        self.keyboard_navigation = enabled;
        self
    }

    #[must_use]
    pub const fn hit_padding(mut self, padding: u16) -> Self {
        self.hit_padding = padding;
        self
    }

    /// Set a fixed share for the second pane. Intended for reusable blocks
    /// with a documented responsive policy; interactive splits keep using
    /// their stateful ratio by default.
    #[doc(hidden)]
    #[must_use]
    pub const fn fixed_second_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.fixed_second_share = Some((percent, low, high));
        self
    }

    #[must_use]
    pub const fn reset_policy(mut self, policy: ResetPolicy) -> Self {
        self.reset_policy = policy;
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
        let model = self.model(st);
        match self.fixed_second_share {
            Some((percent, min, max)) => {
                model.layout_with_second_share(area, self.gap, percent, min, max)
            }
            None => model.layout_with_basis_points(area, self.gap, st.basis_points),
        }
    }

    pub(crate) fn seam(&self, st: SplitPaneState, area: Rect) -> Rect {
        let (first, second) = self.panes(st, area);
        let thickness = if first.is_empty() || second.is_empty() {
            0
        } else {
            self.gap
        };
        let mut seam = match self.axis {
            SplitAxis::Horizontal => Rect {
                x: first.right(),
                y: area.y,
                width: thickness,
                height: area.height,
            },
            SplitAxis::Vertical => Rect {
                x: area.x,
                y: first.bottom(),
                width: area.width,
                height: thickness,
            },
        };
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
        if self.disabled || !self.resizable || !self.keyboard_navigation {
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
        f: impl FnOnce(&mut SplitModel, &mut u16),
        acc: &mut Acc<SplitAction>,
    ) {
        let before = *st;
        let mut m = self.model(*st);
        f(&mut m, &mut st.basis_points);
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
                        self.apply(
                            st,
                            |m, basis_points| {
                                m.nudge_with_basis_points(area, gap, -1, basis_points);
                            },
                            &mut acc,
                        );
                    }
                    Some(SplitCmd::Grow) => self.apply(
                        st,
                        |m, basis_points| {
                            m.nudge_with_basis_points(area, gap, 1, basis_points);
                        },
                        &mut acc,
                    ),
                    Some(SplitCmd::Reset) => {
                        let before = *st;
                        st.reset(self.reset_policy);
                        if *st == before {
                            acc.consumed();
                        } else {
                            acc.action(SplitAction::Resized(st.percent));
                        }
                    }
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
                Intent::Cancel => {
                    if cx.capture_owner() == Some(self.id) {
                        cx.release_capture();
                    }
                    st.grab_offset = None;
                }
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
                if st.grab_offset.is_none() {
                    let (first, _) = self.panes(*st, area);
                    let edge = match self.axis {
                        SplitAxis::Horizontal => first.right(),
                        SplitAxis::Vertical => first.bottom(),
                    };
                    let coordinate = match self.axis {
                        SplitAxis::Horizontal => pos.x,
                        SplitAxis::Vertical => pos.y,
                    };
                    // Preserve the click offset for the full-gap target and
                    // explicitly padded targets. Narrow aligned seam targets
                    // retain their raw-coordinate drag contract.
                    st.grab_offset = Some(if self.hit_padding > 0 || self.seam.is_none() {
                        i32::from(coordinate) - i32::from(edge)
                    } else {
                        0
                    });
                }
                acc.consumed();
            }
            Phase::Drag => {
                let target = drag_target(self.axis, pos, st.grab_offset.unwrap_or(0));
                self.apply(
                    st,
                    |m, basis_points| {
                        let _ = m.drag_to_with_basis_points(area, gap, target, basis_points);
                    },
                    acc,
                );
            }
            Phase::Release | Phase::DragEnd => {
                if cx.capture_owner() == Some(self.id) {
                    cx.release_capture();
                }
                st.grab_offset = None;
                acc.consumed();
            }
            Phase::DoubleClick => {
                if cx
                    .pointer_click_count(self.id, PartRef::of(Part::SEAM))
                    .is_some_and(|count| count != 2)
                {
                    acc.consumed();
                } else {
                    self.reset_for_double_click(st, acc);
                }
            }
            Phase::Click => {
                if cx
                    .pointer_click_count(self.id, PartRef::of(Part::SEAM))
                    .is_some_and(|count| count == 2)
                {
                    self.reset_for_double_click(st, acc);
                } else {
                    acc.consumed();
                }
            }
            Phase::Secondary => acc.consumed(),
            Phase::Move => {}

            _ => {}
        }
    }

    fn reset_for_double_click(&self, st: &mut SplitPaneState, acc: &mut Acc<SplitAction>) {
        let before = *st;
        st.reset(self.reset_policy);
        if *st == before {
            acc.consumed();
        } else {
            acc.action(SplitAction::Resized(st.percent));
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
        if seam.is_empty() {
            return ui.with_area(area, |ui| body(ui, first, second));
        }
        if self.resizable || self.disabled {
            ui.register_control(
                self.id,
                if self.seam.is_some() { seam } else { area },
                if self.disabled {
                    Focusability::Disabled
                } else {
                    self.focusability
                },
            );
        }
        ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
        ui.publish_bindings(self.id, live, self.table());
        if let Some(f) = self.ov.slot_for(Part::SEAM) {
            f(ui, seam);
        } else {
            self.paint_seam(ui, seam, live);
        }
        ui.register_part(
            self.id,
            PartRef::of(Part::SEAM),
            hit_area(seam, area, self.hit_padding),
        );
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
        let rule = if live.contains(StateFlags::PRESSED) || live.contains(StateFlags::HOVERED) {
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

fn drag_target(axis: SplitAxis, pos: Position, grab_offset: i32) -> Position {
    let shifted = |coordinate: u16| {
        (i32::from(coordinate) - grab_offset).clamp(0, i32::from(u16::MAX)) as u16
    };
    match axis {
        SplitAxis::Horizontal => Position::new(shifted(pos.x), pos.y),
        SplitAxis::Vertical => Position::new(pos.x, shifted(pos.y)),
    }
}

fn hit_area(seam: Rect, area: Rect, padding: u16) -> Rect {
    Rect {
        x: seam.x.saturating_sub(padding),
        y: seam.y.saturating_sub(padding),
        width: seam.width.saturating_add(padding.saturating_mul(2)),
        height: seam.height.saturating_add(padding.saturating_mul(2)),
    }
    .intersection(area)
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
    use crate::Button;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::{ColorLevel, Role, Theme};
    use termrock_core::event::MouseKind;
    use termrock_core::response::Response;
    use termrock_runtime::{App, Cx};

    const ID: Id = Id::root("split.tests");
    const EDITOR_ID: Id = Id::root("split.tests.editor");

    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 10,
    };

    struct DragProbe {
        state: SplitPaneState,
        actions: Vec<SplitAction>,
    }

    impl DragProbe {
        fn split() -> SplitPane<'static> {
            SplitPane::new(ID, SplitAxis::Vertical)
                .gap(1)
                .min_first(4)
                .min_second(6)
                .resizable(true)
                .focusability(Focusability::ClickOnly)
                .keyboard_navigation(false)
                .hit_padding(1)
                .reset_policy(ResetPolicy::Initial)
        }
    }

    impl App for DragProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = Self::split().update(cx, &mut self.state);
            if let Some(action) = response.action_ref().copied() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            Self::split().draw(ui, area, &self.state, |ui, first, _| {
                Button::new(EDITOR_ID, "Editor").draw(ui, first);
            });
        }
    }

    #[test]
    fn padded_seam_drag_keeps_precise_position_and_editor_focus() {
        let area = Rect::new(0, 0, 40, 35);
        let mut runtime = Runtime::new(
            DragProbe {
                state: SplitPaneState::new(38),
                actions: Vec::new(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        runtime.set_focus(Some(EDITOR_ID));
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.app().state.percent(), 38);
        assert_eq!(runtime.focus(), Some(EDITOR_ID));
        assert!(!runtime.ring().is_registered(ID));
        assert_eq!(
            runtime.area_of_part(ID, PartRef::of(Part::SEAM)),
            Some(Rect::new(0, 11, 40, 3))
        );

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Down, 8, 11),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Drag, 8, 12),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.app().state.percent(), 38);
        assert_eq!(runtime.app().state.basis_points(), 3_824);
        assert_eq!(DragProbe::first_height(&runtime.app().state, area), 13);
        assert_eq!(runtime.focus(), Some(EDITOR_ID));

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Up, 8, 12),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let mut reset_runtime = Runtime::new(
            DragProbe {
                state: SplitPaneState::new(38),
                actions: Vec::new(),
            },
            Theme::junie(),
        );
        let _ = reset_runtime.initialize();
        reset_runtime.app_mut().state.set_percent(45);
        let mut reset_buffer = Buffer::empty(area);
        reset_runtime
            .draw_buffer(area, &mut reset_buffer)
            .commit_presented();
        reset_runtime.set_focus(Some(EDITOR_ID));
        reset_runtime
            .draw_buffer(area, &mut reset_buffer)
            .commit_presented();
        let seam_y = DragProbe::first_height(&reset_runtime.app().state, area);
        for _ in 0..2 {
            let _ = crate::runtime::stub::deliver(
                &mut reset_runtime,
                crate::runtime::stub::mouse(MouseKind::Down, 8, seam_y),
            );
            let _ = crate::runtime::stub::deliver(
                &mut reset_runtime,
                crate::runtime::stub::mouse(MouseKind::Up, 8, seam_y),
            );
            reset_runtime
                .draw_buffer(area, &mut reset_buffer)
                .commit_presented();
        }
        assert_eq!(reset_runtime.app().state.percent(), 38);
        assert_eq!(reset_runtime.app().state.basis_points(), 3_800);
        assert!(
            reset_runtime
                .app()
                .actions
                .contains(&SplitAction::Resized(38))
        );
        assert_eq!(reset_runtime.focus(), Some(EDITOR_ID));

        reset_runtime.app_mut().state.set_percent(45);
        reset_runtime
            .draw_buffer(area, &mut reset_buffer)
            .commit_presented();
        let seam_y = DragProbe::first_height(&reset_runtime.app().state, area);
        for _ in 0..2 {
            let _ = crate::runtime::stub::deliver(
                &mut reset_runtime,
                crate::runtime::stub::mouse(MouseKind::Down, 8, seam_y),
            );
            let _ = crate::runtime::stub::deliver(
                &mut reset_runtime,
                crate::runtime::stub::mouse(MouseKind::Up, 8, seam_y),
            );
            reset_runtime
                .draw_buffer(area, &mut reset_buffer)
                .commit_presented();
        }
        assert_eq!(reset_runtime.app().state.percent(), 45);
    }

    impl DragProbe {
        fn first_height(state: &SplitPaneState, area: Rect) -> u16 {
            Self::split().panes(*state, area).0.height
        }
    }

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
    fn authored_split_seam_colors_reach_cells_at_each_capability_level() {
        let cases = [
            (
                ColorLevel::TrueColor,
                ratatui_core::style::Color::Rgb(77, 77, 77),
            ),
            (
                ColorLevel::Ansi256,
                ratatui_core::style::Color::Indexed(239),
            ),
            (ColorLevel::Ansi16, ratatui_core::style::Color::DarkGray),
            (ColorLevel::Mono, ratatui_core::style::Color::DarkGray),
        ];
        for (level, expected) in cases {
            let mut theme = Theme::junie();
            theme.capability.color = level;
            let mut rt = Runtime::new(Stub::default(), theme);
            let mut buf = Buffer::empty(SCREEN);
            let split = SplitPane::new(ID, SplitAxis::Horizontal).resizable(true);
            let seam = split.seam(SplitPaneState::default(), AREA);
            rt.draw_scene(SCREEN, &mut buf, |ui, _| {
                split.draw(ui, AREA, &SplitPaneState::default(), |_, _, _| ());
            })
            .commit_presented();

            assert_eq!(
                buf.cell((seam.x, seam.y)).unwrap().fg,
                expected,
                "{level:?}"
            );
        }
    }

    #[test]
    fn explicit_same_value_custom_split_seam_still_uses_generic_downgrade() {
        let theme = Theme::junie()
            .override_family(Family::SPLIT, |recipe| {
                recipe.part(Part::SEAM).base(
                    StylePatch::new().set_fg(Role::Custom(ratatui_core::style::Color::DarkGray)),
                );
            })
            .downgrade(ColorLevel::Mono);
        let mut rt = Runtime::new(Stub::default(), theme);
        let mut buf = Buffer::empty(SCREEN);
        let split = SplitPane::new(ID, SplitAxis::Horizontal).resizable(true);
        let seam = split.seam(SplitPaneState::default(), AREA);
        rt.draw_scene(SCREEN, &mut buf, |ui, _| {
            split.draw(ui, AREA, &SplitPaneState::default(), |_, _, _| ());
        })
        .commit_presented();

        assert_eq!(
            buf.cell((seam.x, seam.y)).unwrap().fg,
            ratatui_core::style::Color::Black
        );
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
    fn split_reset_policy_retains_the_initial_position() {
        let mut state = SplitPaneState::new(38);
        state.set_percent(63);
        state.reset(ResetPolicy::Initial);
        assert_eq!(state.percent(), 38);
        assert_eq!(state.basis_points(), 3_800);

        state.reset(ResetPolicy::Percent(42));
        assert_eq!(state.percent(), 42);
        state.reset(ResetPolicy::Balanced);
        assert_eq!(state.percent(), 50);
    }

    #[test]
    fn split_hit_padding_expands_pointer_area_without_changing_the_seam() {
        let seam = Rect::new(10, 4, 1, 8);
        assert_eq!(
            hit_area(seam, Rect::new(0, 0, 30, 20), 1),
            Rect::new(9, 3, 3, 10)
        );
        assert_eq!(hit_area(seam, Rect::new(10, 4, 1, 8), 1), seam);
    }

    #[test]
    fn resetting_maximization_is_a_full_state_change() {
        let sp = SplitPane::new(ID, SplitAxis::Horizontal);
        let mut st = SplitPaneState::default();
        st.toggle_max(Maximized::First);
        let mut acc = Acc::new();
        sp.apply(
            &mut st,
            |model, _| model.maximized = Maximized::None,
            &mut acc,
        );
        assert_eq!(acc.finish(ID).into_action(), Some(SplitAction::Resized(50)));
    }

    #[test]
    fn drag_preserves_the_pointer_offset_inside_a_wide_seam() {
        assert_eq!(
            drag_target(SplitAxis::Horizontal, Position::new(25, 6), 2,),
            Position::new(23, 6)
        );
        assert_eq!(
            drag_target(SplitAxis::Vertical, Position::new(6, 14), 2,),
            Position::new(6, 12)
        );
        assert_eq!(
            drag_target(SplitAxis::Horizontal, Position::new(18, 6), -1),
            Position::new(19, 6)
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
        let hovered = seam_cells(crate::ReferenceState::HOVERED);
        let focused = seam_cells(crate::ReferenceState::FOCUSED);
        let pressed = seam_cells(crate::ReferenceState::PRESSED);
        assert!(!base.is_empty(), "the seam painted nothing");
        assert_ne!(base, hovered, "hover is indistinguishable from default");
        assert_ne!(base, focused, "focused is indistinguishable from default");
        assert_ne!(base, pressed, "pressed is indistinguishable from default");
        assert_eq!(
            hovered.keys().map(|(glyph, _)| glyph).collect::<Vec<_>>(),
            pressed.keys().map(|(glyph, _)| glyph).collect::<Vec<_>>(),
            "hover and press should share the heavy seam glyph in mono"
        );
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
