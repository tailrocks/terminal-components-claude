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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ResetPolicy {
    Initial,

    Percent(u8),

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SplitPaneState {
    percent: u8,
    initial_percent: u8,
    basis_points: u16,
    maximized: Maximized,
    grab_offset: Option<i32>,
    initial_cells_active: bool,
}

impl fmt::Debug for SplitPaneState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitPaneState")
            .field("percent", &self.percent)
            .field("initial_percent", &self.initial_percent)
            .field("basis_points", &self.basis_points)
            .field("maximized", &self.maximized)
            .field("grab_offset", &self.grab_offset)
            .finish()
    }
}

impl Default for SplitPaneState {
    fn default() -> Self {
        SplitPaneState {
            percent: BALANCED,
            initial_percent: BALANCED,
            basis_points: (BALANCED as u16) * BASIS_POINTS_PER_PERCENT,
            maximized: Maximized::None,
            grab_offset: None,
            initial_cells_active: false,
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
            initial_cells_active: false,
        }
    }

    pub const fn percent(&self) -> u8 {
        self.percent
    }

    pub const fn set_percent(&mut self, p: u8) {
        self.percent = clamp_percent(p);
        self.basis_points = (clamp_percent(p) as u16) * BASIS_POINTS_PER_PERCENT;
        self.initial_cells_active = false;
    }

    pub(crate) const fn new_with_initial_cells(percent: u8) -> Self {
        SplitPaneState {
            percent: clamp_percent(percent),
            initial_percent: clamp_percent(percent),
            basis_points: (clamp_percent(percent) as u16) * BASIS_POINTS_PER_PERCENT,
            maximized: Maximized::None,
            grab_offset: None,
            initial_cells_active: true,
        }
    }

    pub const fn maximized(&self) -> Maximized {
        self.maximized
    }

    pub const fn basis_points(&self) -> u16 {
        self.basis_points
    }

    fn reset(&mut self, policy: ResetPolicy, has_initial_cells: bool) {
        let percent = match policy {
            ResetPolicy::Initial => self.initial_percent,
            ResetPolicy::Percent(percent) => clamp_percent(percent),
            ResetPolicy::Balanced => BALANCED,
        };
        self.percent = percent;
        self.basis_points = (percent as u16) * BASIS_POINTS_PER_PERCENT;
        self.maximized = Maximized::None;
        self.grab_offset = None;
        self.initial_cells_active = has_initial_cells && policy == ResetPolicy::Initial;
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
    preserve_drag_offset: bool,
    fixed_first_share: Option<(u8, u16, u16)>,
    fixed_second_share: Option<(u8, u16, u16)>,
    initial_first_cells: Option<u16>,
    fixed_sizing: bool,
    resize_step_cells: u16,
    resize_bindings: Option<&'static [Binding<SplitCmd>]>,
    reset_policy: ResetPolicy,
    ov: PartStyle<'a>,
}

impl fmt::Debug for SplitPane<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("SplitPane");
        debug
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
            .field("preserve_drag_offset", &self.preserve_drag_offset)
            .field("fixed_first_share", &self.fixed_first_share)
            .field("fixed_second_share", &self.fixed_second_share);
        if self.initial_first_cells.is_some() {
            debug.field("initial_first_cells", &self.initial_first_cells);
        }
        debug
            .field("fixed_sizing", &self.fixed_sizing)
            .field("resize_step_cells", &self.resize_step_cells)
            .field("resize_bindings", &self.resize_bindings)
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
            preserve_drag_offset: false,
            fixed_first_share: None,
            fixed_second_share: None,
            initial_first_cells: None,
            fixed_sizing: false,
            resize_step_cells: 1,
            resize_bindings: None,
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

    pub fn required_extent(&self, state: &SplitPaneState, first: u16, second: u16) -> u16 {
        let model = self.model(*state);
        let first = first.max(model.min_first);
        let second = second.max(model.min_second);
        if let Some((percent, min, max)) = self.fixed_first_share {
            return model
                .required_extent_with_first_share(self.gap, percent, min, max, first, second);
        }
        if let Some((percent, min, max)) = self.fixed_second_share {
            return model
                .required_extent_with_second_share(self.gap, percent, min, max, first, second);
        }
        if state.initial_cells_active
            && let Some(initial_first_cells) = self.initial_first_cells
        {
            return model.required_extent_with_first_extent(
                self.gap,
                initial_first_cells,
                first,
                second,
            );
        }
        model.required_extent_with_basis_points(self.gap, state.basis_points, first, second)
    }

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

    #[must_use]
    pub const fn preserve_drag_offset(mut self, preserve: bool) -> Self {
        self.preserve_drag_offset = preserve;
        self
    }

    #[must_use]
    pub const fn resize_step_cells(mut self, cells: u16) -> Self {
        self.resize_step_cells = if cells == 0 { 1 } else { cells };
        self
    }

    pub(crate) const fn resize_bindings(mut self, table: &'static [Binding<SplitCmd>]) -> Self {
        self.resize_bindings = Some(table);
        self
    }

    #[must_use]
    pub(crate) const fn fixed_first_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.fixed_first_share = Some((percent, low, high));
        self.fixed_second_share = None;
        self
    }

    #[must_use]
    pub(crate) const fn fixed_sizing(mut self) -> Self {
        self.fixed_sizing = true;
        self.resizable = false;
        self.seam = Some((0, SeamAlign::Offset(0)));
        self
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn fixed_second_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.fixed_first_share = None;
        self.fixed_second_share = Some((percent, low, high));
        self
    }

    #[must_use]
    pub const fn reset_policy(mut self, policy: ResetPolicy) -> Self {
        self.reset_policy = policy;
        self
    }

    #[must_use]
    pub(crate) const fn initial_first_cells(mut self, cells: u16) -> Self {
        self.initial_first_cells = Some(cells);
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
        m.maximized = if self.fixed_sizing {
            Maximized::None
        } else {
            st.maximized
        };
        m
    }

    pub(crate) fn panes(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        let model = self.model(st);
        match (self.fixed_first_share, self.fixed_second_share) {
            (Some((percent, min, max)), _) => {
                model.layout_with_first_share(area, self.gap, percent, min, max)
            }
            (_, Some((percent, min, max))) => {
                model.layout_with_second_share(area, self.gap, percent, min, max)
            }
            (None, None)
                if st.initial_cells_active
                    && let Some(first) = self.initial_first_cells =>
            {
                model.layout_with_first_extent(area, self.gap, first)
            }
            (None, None) => model.layout_with_basis_points(area, self.gap, st.basis_points),
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
        if self.fixed_sizing || self.disabled || !self.resizable || !self.keyboard_navigation {
            return &[];
        }
        if let Some(table) = self.resize_bindings {
            return table;
        }
        match self.axis {
            SplitAxis::Horizontal => HORIZONTAL,
            SplitAxis::Vertical => VERTICAL,
        }
    }

    fn apply(
        &self,
        st: &mut SplitPaneState,
        f: impl FnOnce(&mut SplitModel, &mut u16) -> bool,
        acc: &mut Acc<SplitAction>,
    ) {
        let before = *st;
        let mut m = self.model(*st);
        let changed = f(&mut m, &mut st.basis_points);
        st.percent = m.percent;
        st.maximized = m.maximized;
        if changed {
            st.initial_cells_active = false;
        }
        if *st == before {
            acc.consumed();
        } else {
            acc.action(SplitAction::Resized(st.percent));
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SplitPaneState) -> Response<SplitAction> {
        self.update_for_owner(cx, st, self.id, true)
    }

    pub(crate) fn update_from_owner(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SplitPaneState,
        owner: Id,
    ) -> Response<SplitAction> {
        self.update_for_owner(cx, st, owner, false)
    }

    fn update_for_owner(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SplitPaneState,
        owner: Id,
        allow_pointer: bool,
    ) -> Response<SplitAction> {
        if self.fixed_sizing {
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            st.grab_offset = None;
            return Response::ignored();
        }
        let area = cx
            .layout(self.id)
            .and_then(|facts| facts.logical_area)
            .unwrap_or(Rect::ZERO);
        let table = self.table();
        let mut acc = Acc::<SplitAction>::new();
        let step = i16::try_from(self.resize_step_cells).unwrap_or(i16::MAX);
        for it in cx.intents(owner) {
            if self.disabled {
                continue;
            }
            match it {
                Intent::Binding(action) => match Binding::command(table, action) {
                    Some(SplitCmd::Shrink) => {
                        self.nudge(st, area, -step, &mut acc);
                    }
                    Some(SplitCmd::Grow) => {
                        self.nudge(st, area, step, &mut acc);
                    }
                    Some(SplitCmd::Reset) => {
                        let before = *st;
                        st.reset(self.reset_policy, self.initial_first_cells.is_some());
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
                } if allow_pointer => self.pointer(cx, st, phase, pos, area, &mut acc),
                Intent::Cancel if allow_pointer => {
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

    fn nudge(&self, st: &mut SplitPaneState, area: Rect, delta: i16, acc: &mut Acc<SplitAction>) {
        if st.initial_cells_active && self.initial_first_cells.is_some() {
            let (first, _) = self.panes(*st, area);
            let current = match self.axis {
                SplitAxis::Horizontal => first.width,
                SplitAxis::Vertical => first.height,
            };
            let target = i32::from(current)
                .saturating_add(i32::from(delta))
                .clamp(0, i32::from(u16::MAX)) as u16;
            let pos = match self.axis {
                SplitAxis::Horizontal => Position::new(area.x.saturating_add(target), area.y),
                SplitAxis::Vertical => Position::new(area.x, area.y.saturating_add(target)),
            };
            self.apply(
                st,
                |model, basis_points| {
                    model.drag_to_with_basis_points(area, self.gap, pos, basis_points)
                },
                acc,
            );
        } else {
            self.apply(
                st,
                |model, basis_points| {
                    model.nudge_with_basis_points(area, self.gap, delta, basis_points)
                },
                acc,
            );
        }
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

                    st.grab_offset = Some(
                        if self.hit_padding > 0 || self.seam.is_none() || self.preserve_drag_offset
                        {
                            i32::from(coordinate) - i32::from(edge)
                        } else {
                            0
                        },
                    );
                }
                acc.consumed();
            }
            Phase::Drag => {
                let target = drag_target(self.axis, pos, st.grab_offset.unwrap_or(0));
                self.apply(
                    st,
                    |m, basis_points| m.drag_to_with_basis_points(area, gap, target, basis_points),
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
        st.reset(self.reset_policy, self.initial_first_cells.is_some());
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
