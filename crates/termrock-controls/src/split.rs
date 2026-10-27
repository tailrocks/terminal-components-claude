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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SplitFallbackPolicy {
    Weighted,
    SecondOnly,
}

/// Temporary responsive layout when the configured pane minima do not fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitFallback {
    first_weight: u16,
    second_weight: u16,
    gap: u16,
    policy: SplitFallbackPolicy,
}

impl SplitFallback {
    /// Construct a weighted fallback, normalizing each zero weight to one and using a one-cell gap.
    pub const fn new(first_weight: u16, second_weight: u16) -> Self {
        SplitFallback {
            first_weight: if first_weight == 0 { 1 } else { first_weight },
            second_weight: if second_weight == 0 { 1 } else { second_weight },
            gap: 1,
            policy: SplitFallbackPolicy::Weighted,
        }
    }

    /// Construct a fallback that gives the entire area to the second pane and keeps the configured axis.
    pub const fn second_only() -> Self {
        SplitFallback {
            first_weight: 1,
            second_weight: 1,
            gap: 0,
            policy: SplitFallbackPolicy::SecondOnly,
        }
    }

    /// Set the weighted-fallback gap; the second-only policy always uses no gap.
    #[must_use]
    pub const fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    const fn is_second_only(self) -> bool {
        matches!(self.policy, SplitFallbackPolicy::SecondOnly)
    }
}

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

    /// Precise ratio retained after a manual resize.
    ///
    /// An opted-in initial-cell split uses its configured cell extent until
    /// manual input changes this value; query the resolved pane for that width.
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

    fn clear_grab_offset(&mut self) -> bool {
        self.grab_offset.take().is_some()
    }
}

#[derive(Clone, Copy, Debug)]
enum SeamAlign {
    Start,
    Center,
    End,
    Offset(u16),
}

#[derive(Clone)]
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
    undersized_fallback: Option<SplitFallback>,
    resolved_fallback: Option<SplitFallback>,
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
        debug
            .field("undersized_fallback", &self.undersized_fallback)
            .field("resolved_fallback", &self.resolved_fallback);
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
            undersized_fallback: None,
            resolved_fallback: None,
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

    /// Return the configured axis, or the effective axis on a resolved pane.
    pub const fn axis(&self) -> SplitAxis {
        self.axis
    }

    /// Return the active pane selection represented by this configuration and state.
    ///
    /// An unresolved pane reports only durable state. Call `resolved_for` first
    /// when querying a responsive policy; a resolved second-only pane reports
    /// `Second` unless durable state explicitly maximizes another pane.
    pub const fn effective_maximized(&self, state: &SplitPaneState) -> Maximized {
        match state.maximized {
            Maximized::None => match self.resolved_fallback {
                Some(SplitFallback {
                    policy: SplitFallbackPolicy::SecondOnly,
                    ..
                }) => Maximized::Second,
                _ => Maximized::None,
            },
            maximized => maximized,
        }
    }

    /// Resolve the responsive layout for the supplied area.
    ///
    /// Weighted fallback switches to the opposite axis; second-only fallback
    /// keeps the configured axis and gives the full area to the second pane.
    /// The returned pane owns a fixed configuration. It does not alter state,
    /// publish geometry, or release capture. Query `effective_maximized` on the
    /// returned pane to include a resolved second-only policy.
    pub fn resolved_for(&self, state: &SplitPaneState, area: Rect) -> Self {
        let Some(fallback) = self.undersized_fallback else {
            return self.clone();
        };
        if state.maximized != Maximized::None {
            return self.clone();
        }
        let extent = match self.axis {
            SplitAxis::Horizontal => area.width,
            SplitAxis::Vertical => area.height,
        };
        let required = u32::from(self.min_first) + u32::from(self.gap) + u32::from(self.min_second);
        if u32::from(extent) >= required {
            return self.clone();
        }

        let mut resolved = self.clone();
        if !fallback.is_second_only() {
            resolved.axis = match self.axis {
                SplitAxis::Horizontal => SplitAxis::Vertical,
                SplitAxis::Vertical => SplitAxis::Horizontal,
            };
        }
        resolved.undersized_fallback = None;
        resolved.resolved_fallback = Some(fallback);
        resolved.gap = if fallback.is_second_only() {
            0
        } else {
            fallback.gap
        };
        resolved.min_first = 0;
        resolved.min_second = 0;
        resolved.seam = None;
        resolved.resizable = false;
        resolved.fixed_first_share = None;
        resolved.fixed_second_share = None;
        resolved.initial_first_cells = None;
        resolved.resize_bindings = None;
        resolved.fixed_sizing = true;
        resolved
    }

    /// Smallest extent at which the resolved panes can hold their requested
    /// along-axis sizes under the current sizing policy and configured minima.
    pub fn required_extent(&self, state: &SplitPaneState, first: u16, second: u16) -> u16 {
        if let Some(fallback) = self.resolved_fallback {
            if fallback.is_second_only() {
                return match self.effective_maximized(state) {
                    Maximized::First => first,
                    Maximized::Second | Maximized::None => second,
                };
            }
            return required_extent_with_weights(
                fallback,
                self.effective_maximized(state),
                first,
                second,
            );
        }
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

    /// Return the child constraints for the pane rectangles resolved from the
    /// supplied root constraints and current state. Consumers can measure pane
    /// content at its actual split width without reproducing split arithmetic.
    pub fn pane_constraints(
        &self,
        state: &SplitPaneState,
        constraints: Constraints,
    ) -> (Constraints, Constraints) {
        if self.undersized_fallback.is_some() {
            let root = Rect::new(0, 0, constraints.max.0, constraints.max.1);
            return self
                .resolved_for(state, root)
                .pane_constraints_resolved(state, constraints);
        }
        self.pane_constraints_resolved(state, constraints)
    }

    pub(crate) fn pane_constraints_resolved(
        &self,
        state: &SplitPaneState,
        constraints: Constraints,
    ) -> (Constraints, Constraints) {
        let root = Rect::new(0, 0, constraints.max.0, constraints.max.1);
        let (first, second) = self.panes_resolved(*state, root);
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

    /// Preserve the pointer's grab offset when dragging a narrow aligned seam.
    ///
    /// By default, narrow seam drags continue to use the pointer's raw
    /// coordinate as the first-pane edge. Enable this policy when the seam
    /// should move by the same number of cells as the pointer.
    #[must_use]
    pub const fn preserve_drag_offset(mut self, preserve: bool) -> Self {
        self.preserve_drag_offset = preserve;
        self
    }

    /// Set the number of terminal cells moved by each keyboard resize command.
    /// The default remains one cell.
    #[must_use]
    pub const fn resize_step_cells(mut self, cells: u16) -> Self {
        self.resize_step_cells = if cells == 0 { 1 } else { cells };
        self
    }

    pub(crate) const fn resize_bindings(mut self, table: &'static [Binding<SplitCmd>]) -> Self {
        self.resize_bindings = Some(table);
        self
    }

    /// Resolve a fixed first-pane share with nearest-cell rounding.
    #[must_use]
    pub(crate) const fn fixed_first_share(mut self, percent: u8, min: u16, max: u16) -> Self {
        let percent = if percent > 100 { 100 } else { percent };
        let low = if min < max { min } else { max };
        let high = if min > max { min } else { max };
        self.fixed_first_share = Some((percent, low, high));
        self.fixed_second_share = None;
        self
    }

    /// Apply the non-interactive sizing policy used by fixed MasterDetail modes.
    #[must_use]
    pub(crate) const fn fixed_sizing(mut self) -> Self {
        self.fixed_sizing = true;
        self.resizable = false;
        self.seam = Some((0, SeamAlign::Offset(0)));
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
        self.fixed_first_share = None;
        self.fixed_second_share = Some((percent, low, high));
        self
    }

    #[must_use]
    pub const fn reset_policy(mut self, policy: ResetPolicy) -> Self {
        self.reset_policy = policy;
        self
    }

    /// Use a temporary responsive fallback while configured minima do not fit.
    #[must_use]
    pub const fn undersized_fallback(mut self, fallback: SplitFallback) -> Self {
        self.undersized_fallback = Some(fallback);
        self
    }

    /// Use an exact first-pane extent until the split is manually resized.
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
        m.maximized = if self.fixed_sizing && self.resolved_fallback.is_none() {
            Maximized::None
        } else {
            self.effective_maximized(&st)
        };
        m
    }

    pub(crate) fn panes(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        let resolved = self.resolved_for(&st, area);
        resolved.panes_resolved(st, area)
    }

    fn panes_resolved(&self, st: SplitPaneState, area: Rect) -> (Rect, Rect) {
        if let Some(fallback) = self.resolved_fallback {
            let extent = match self.axis {
                SplitAxis::Horizontal => area.width,
                SplitAxis::Vertical => area.height,
            };
            let gap = self.gap;
            let usable = extent.saturating_sub(gap);
            let first = if fallback.is_second_only() {
                0
            } else {
                let total = u64::from(fallback.first_weight) + u64::from(fallback.second_weight);
                (u64::from(usable) * u64::from(fallback.first_weight)).div_ceil(total) as u16
            };
            return self.model(st).layout_with_first_extent(area, gap, first);
        }
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
        let resolved = self.resolved_for(&st, area);
        resolved.seam_resolved(st, area)
    }

    fn seam_resolved(&self, st: SplitPaneState, area: Rect) -> Rect {
        if self.resolved_fallback.is_some() {
            return Rect::new(area.x, area.y, 0, 0);
        }
        let (first, second) = self.panes_resolved(st, area);
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

    /// Move the first-pane boundary by a number of terminal cells.
    ///
    /// A positive `delta` moves the boundary right for a horizontal split or
    /// down for a vertical split. A negative value moves it left or up. The
    /// resize uses the same precise-ratio, initial-cell, and minimum-size
    /// handling as keyboard and pointer resizing. Requests that reach a
    /// configured minimum or the available layout limit are clamped there. A
    /// clamped request can update the retained precise ratio without moving
    /// the visible boundary; callers that require a full step must compare
    /// the resolved pane geometry before and after this call.
    ///
    /// A zero delta, empty or undersized area, disabled or fixed sizing/share
    /// policy, non-resizable split, or maximized state returns a consumed
    /// response owned by this split and does not change `state`. This method
    /// does not alter pointer capture or the stored pointer grab offset.
    pub fn resize_by_cells(
        &self,
        state: &mut SplitPaneState,
        area: Rect,
        delta: i16,
    ) -> Response<SplitAction> {
        let resolved = self.resolved_for(state, area);
        resolved.resize_by_cells_resolved(state, area, delta)
    }

    fn resize_by_cells_resolved(
        &self,
        state: &mut SplitPaneState,
        area: Rect,
        delta: i16,
    ) -> Response<SplitAction> {
        let extent = match self.axis {
            SplitAxis::Horizontal => area.width,
            SplitAxis::Vertical => area.height,
        };
        let usable = extent.saturating_sub(self.gap);
        if delta == 0
            || area.is_empty()
            || usable == 0
            || u32::from(usable) < u32::from(self.min_first) + u32::from(self.min_second)
            || self.disabled
            || self.fixed_sizing
            || self.fixed_first_share.is_some()
            || self.fixed_second_share.is_some()
            || !self.resizable
            || state.maximized != Maximized::None
        {
            return Response::consumed().for_id(self.id);
        }

        let mut acc = Acc::<SplitAction>::new();
        let bounded_delta = if delta > 0 {
            let (first, _) = self.panes_resolved(*state, area);
            let current_first = match self.axis {
                SplitAxis::Horizontal => first.width,
                SplitAxis::Vertical => first.height,
            };
            let remaining_usable = usable.saturating_sub(current_first);
            delta.min(i16::try_from(remaining_usable).unwrap_or(i16::MAX))
        } else {
            delta
        };
        if bounded_delta == 0 {
            return Response::consumed().for_id(self.id);
        }
        self.nudge(state, area, bounded_delta, &mut acc);
        acc.finish(self.id)
    }

    /// Clear this split's stored pointer grab offset.
    ///
    /// This changes only `state.grab_offset`. It does not release runtime
    /// pointer capture, clear a runtime press, cancel a gesture while this seam
    /// stays registered, change focus, or alter layout state. Callers that hide
    /// or remove this seam must also release capture owned by this split and
    /// publish the target's disappearance or ineligibility before accepting
    /// another input.
    pub fn clear_drag_state(&self, state: &mut SplitPaneState) -> Response<()> {
        if state.clear_grab_offset() {
            Response::changed().for_id(self.id)
        } else {
            Response::ignored().for_id(self.id)
        }
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
        if self.undersized_fallback.is_some() && cx.capture_owner() != Some(self.id) {
            let _ = st.clear_grab_offset();
        }
        let area = cx
            .layout(self.id)
            .and_then(|facts| facts.logical_area)
            .unwrap_or(Rect::ZERO);
        let resolved = self.resolved_for(st, area);
        resolved.update_for_owner_resolved(cx, st, owner, allow_pointer, area)
    }

    fn update_for_owner_resolved(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SplitPaneState,
        owner: Id,
        allow_pointer: bool,
        area: Rect,
    ) -> Response<SplitAction> {
        if self.fixed_sizing {
            if cx.capture_owner() == Some(self.id) {
                cx.release_capture();
            }
            let _ = st.clear_grab_offset();
            return Response::ignored();
        }
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
                    let _ = st.clear_grab_offset();
                }
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    fn nudge(&self, st: &mut SplitPaneState, area: Rect, delta: i16, acc: &mut Acc<SplitAction>) {
        if st.initial_cells_active && self.initial_first_cells.is_some() {
            let (first, _) = self.panes_resolved(*st, area);
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
                    let (first, _) = self.panes_resolved(*st, area);
                    let edge = match self.axis {
                        SplitAxis::Horizontal => first.right(),
                        SplitAxis::Vertical => first.bottom(),
                    };
                    let coordinate = match self.axis {
                        SplitAxis::Horizontal => pos.x,
                        SplitAxis::Vertical => pos.y,
                    };
                    // Preserve the click offset for the full-gap target,
                    // explicitly padded targets, and opt-in narrow seams.
                    // Narrow aligned seams retain their raw-coordinate drag
                    // contract by default.
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
                let _ = st.clear_grab_offset();
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
        let resolved = self.resolved_for(st, area);
        resolved.draw_resolved(ui, area, st, body)
    }

    fn draw_resolved<R>(
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
        let (first, second) = self.panes_resolved(*st, area);
        if self.resolved_fallback.is_some() {
            ui.register_decor(self.id, PartRef::of(Part::CONTAINER), area);
            return ui.with_area(area, |ui| body(ui, first, second));
        }
        let seam = self.seam_resolved(*st, area);
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
        let state = SplitPaneState::default();
        let area = Rect::new(0, 0, c.max.0, c.max.1);
        let resolved = self.resolved_for(&state, area);
        let along = if resolved.resolved_fallback.is_some() {
            resolved.required_extent(&state, 0, 0)
        } else {
            u32::from(resolved.min_first)
                .saturating_add(u32::from(resolved.min_second))
                .saturating_add(u32::from(resolved.gap))
                .min(u32::from(u16::MAX)) as u16
        };
        let min = match resolved.axis() {
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

fn required_extent_with_weights(
    fallback: SplitFallback,
    maximized: Maximized,
    first: u16,
    second: u16,
) -> u16 {
    match maximized {
        Maximized::First => return first,
        Maximized::Second => return second,
        Maximized::None => {}
    }

    let first_weight = u64::from(fallback.first_weight);
    let second_weight = u64::from(fallback.second_weight);
    let total = first_weight + second_weight;
    let first_usable = if first == 0 {
        0
    } else {
        (u64::from(first - 1) * total / first_weight) + 1
    };
    let second_product = u64::from(second) * total;
    let second_usable = second_product.div_ceil(second_weight);
    first_usable
        .max(second_usable)
        .saturating_add(u64::from(fallback.gap))
        .min(u64::from(u16::MAX)) as u16
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
    use termrock_core::response::{Invalidate, Response};
    use termrock_runtime::{App, Cx};

    const ID: Id = Id::root("split.tests");
    const EDITOR_ID: Id = Id::root("split.tests.editor");
    const OTHER_ID: Id = Id::root("split.tests.other");

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

    struct ResizeProbe {
        axis: SplitAxis,
        state: SplitPaneState,
        first: Cell<Rect>,
        second: Cell<Rect>,
    }

    impl ResizeProbe {
        fn new(axis: SplitAxis, state: SplitPaneState) -> Self {
            ResizeProbe {
                axis,
                state,
                first: Cell::new(Rect::ZERO),
                second: Cell::new(Rect::ZERO),
            }
        }

        fn split(axis: SplitAxis) -> SplitPane<'static> {
            SplitPane::new(ID, axis)
                .gap(1)
                .min_first(4)
                .min_second(6)
                .resizable(true)
                .focusability(Focusability::ClickOnly)
                .keyboard_navigation(false)
                .hit_padding(1)
        }

        fn boundary(area: Rect, axis: SplitAxis) -> u16 {
            match axis {
                SplitAxis::Horizontal => area.right(),
                SplitAxis::Vertical => area.bottom(),
            }
        }

        fn extent(area: Rect, axis: SplitAxis) -> u16 {
            match axis {
                SplitAxis::Horizontal => area.width,
                SplitAxis::Vertical => area.height,
            }
        }
    }

    impl App for ResizeProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Self::split(self.axis).update(cx, &mut self.state).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = ui.full();
            Self::split(self.axis).draw(ui, area, &self.state, |_, first, second| {
                self.first.set(first);
                self.second.set(second);
            });
        }
    }

    fn pointer_derived_resize_runtime(axis: SplitAxis) -> (Runtime<ResizeProbe>, Buffer, Rect) {
        let area = Rect::new(0, 0, 40, 35);
        let mut runtime = Runtime::new(
            ResizeProbe::new(axis, SplitPaneState::new(38)),
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let seam = ResizeProbe::split(axis).seam(runtime.app().state, area);
        let start = Position::new(seam.x, seam.y);
        let target = match axis {
            SplitAxis::Horizontal => Position::new(start.x.saturating_add(1), start.y),
            SplitAxis::Vertical => Position::new(start.x, start.y.saturating_add(1)),
        };
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Down, start.x, start.y),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Drag, target.x, target.y),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(runtime.app().state.grab_offset, Some(0));
        assert_ne!(
            runtime.app().state.basis_points() % BASIS_POINTS_PER_PERCENT,
            0
        );
        (runtime, buffer, area)
    }

    fn state_with_grab() -> SplitPaneState {
        let mut state = SplitPaneState::new_with_initial_cells(38);
        state.grab_offset = Some(-2);
        state
    }

    fn assert_clear_drag_state_changes_only_the_offset(
        split: SplitPane<'_>,
        mut state: SplitPaneState,
    ) {
        assert!(state.grab_offset.is_some());
        let mut expected = state;
        expected.grab_offset = None;

        let response = split.clear_drag_state(&mut state);

        assert_eq!(state, expected, "clear changed durable split state");
        assert_eq!(response.id(), Some(split.id()));
        assert!(response.is_consumed());
        assert!(response.is_changed());
        assert_eq!(response.invalidate(), Invalidate::Paint);
        assert!(response.action_ref().is_none());
    }

    fn assert_resize_noop(split: SplitPane<'_>, mut state: SplitPaneState, area: Rect, delta: i16) {
        let before = state;
        let response = split.resize_by_cells(&mut state, area, delta);
        assert_eq!(state, before, "a guarded resize changed complete state");
        assert_eq!(response.id(), Some(ID));
        assert!(response.is_consumed());
        assert_eq!(response.action_ref(), None);
    }

    #[test]
    fn cell_resize_moves_a_pointer_derived_precise_boundary_on_both_axes() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let (mut runtime, mut buffer, area) = pointer_derived_resize_runtime(axis);
            let baseline = runtime.app().state;
            for delta in [-1, 1] {
                runtime.app_mut().state = baseline;
                runtime.draw_buffer(area, &mut buffer).commit_presented();
                let before = runtime.app().first.get();
                let mut resized = baseline;
                let response = ResizeProbe::split(axis).resize_by_cells(&mut resized, area, delta);

                assert_eq!(response.id(), Some(ID));
                assert_eq!(
                    response.action_ref().copied(),
                    Some(SplitAction::Resized(resized.percent()))
                );
                assert_eq!(resized.grab_offset, baseline.grab_offset);

                runtime.app_mut().state = resized;
                runtime.draw_buffer(area, &mut buffer).commit_presented();
                let after = runtime.app().first.get();
                assert_eq!(
                    i32::from(ResizeProbe::boundary(after, axis))
                        - i32::from(ResizeProbe::boundary(before, axis)),
                    i32::from(delta),
                    "{axis:?} cell delta did not move the drawn first-pane boundary"
                );
                assert_eq!(runtime.capture_owner(), Some(ID));
            }
        }
    }

    #[test]
    fn cell_resize_noop_guards_preserve_all_state_and_return_the_split_owner() {
        let area = Rect::new(0, 0, 40, 35);
        assert_resize_noop(
            ResizeProbe::split(SplitAxis::Horizontal),
            state_with_grab(),
            area,
            0,
        );
        assert_resize_noop(
            ResizeProbe::split(SplitAxis::Horizontal),
            state_with_grab(),
            Rect::ZERO,
            1,
        );
        assert_resize_noop(
            ResizeProbe::split(SplitAxis::Horizontal),
            state_with_grab(),
            Rect::new(0, 0, 10, 35),
            1,
        );
        assert_resize_noop(
            SplitPane::new(ID, SplitAxis::Horizontal),
            state_with_grab(),
            area,
            1,
        );
        assert_resize_noop(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .resizable(true)
                .disabled(true),
            state_with_grab(),
            area,
            1,
        );
        assert_resize_noop(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .fixed_sizing()
                .resizable(true),
            state_with_grab(),
            area,
            1,
        );
        assert_resize_noop(
            SplitPane::new(ID, SplitAxis::Horizontal)
                .resizable(true)
                .fixed_second_share(40, 4, 6),
            state_with_grab(),
            area,
            1,
        );

        let mut maximized = state_with_grab();
        maximized.toggle_max(Maximized::First);
        assert_resize_noop(
            ResizeProbe::split(SplitAxis::Horizontal),
            maximized,
            area,
            1,
        );
    }

    #[test]
    fn guarded_cell_resize_preserves_live_capture_and_grab_offset() {
        let (mut runtime, _buffer, area) = pointer_derived_resize_runtime(SplitAxis::Horizontal);
        let before = runtime.app().state;
        let response = ResizeProbe::split(SplitAxis::Horizontal).resize_by_cells(
            &mut runtime.app_mut().state,
            area,
            0,
        );

        assert_eq!(runtime.app().state, before);
        assert_eq!(runtime.app().state.grab_offset, Some(0));
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(response.id(), Some(ID));
        assert_eq!(response.action_ref(), None);
    }

    #[test]
    fn cell_resize_clamps_to_minima_on_both_axes() {
        let area = Rect::new(0, 0, 40, 35);
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            for (percent, delta, clamp_first) in [(5, -100, true), (95, 100, false)] {
                let mut runtime = Runtime::new(
                    ResizeProbe::new(axis, SplitPaneState::new(percent)),
                    Theme::junie(),
                );
                let _ = runtime.initialize();
                let mut buffer = Buffer::empty(area);
                runtime.draw_buffer(area, &mut buffer).commit_presented();
                let mut resized = runtime.app().state;
                let response = ResizeProbe::split(axis).resize_by_cells(&mut resized, area, delta);
                assert_eq!(response.id(), Some(ID));
                assert_eq!(
                    response.action_ref().copied(),
                    Some(SplitAction::Resized(resized.percent()))
                );
                runtime.app_mut().state = resized;
                runtime.draw_buffer(area, &mut buffer).commit_presented();

                let first = runtime.app().first.get();
                let second = runtime.app().second.get();
                let first_extent = ResizeProbe::extent(first, axis);
                let second_extent = ResizeProbe::extent(second, axis);
                assert!(first_extent >= 4, "{axis:?} first pane fell below min");
                assert!(second_extent >= 6, "{axis:?} second pane fell below min");
                if clamp_first {
                    assert_eq!(first_extent, 4, "{axis:?} first pane did not clamp");
                } else {
                    assert_eq!(second_extent, 6, "{axis:?} second pane did not clamp");
                }
            }
        }
    }

    #[test]
    fn cell_resize_transitions_an_initial_cell_split_to_precise_layout() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(4)
            .min_second(6)
            .resizable(true)
            .initial_first_cells(12);
        let area = Rect::new(0, 0, 40, 35);
        let mut state = SplitPaneState::new_with_initial_cells(50);
        let before = split.panes(state, area).0.width;
        assert_eq!(before, 12);

        let response = split.resize_by_cells(&mut state, area, 2);

        assert_eq!(response.id(), Some(ID));
        assert_eq!(
            response.action_ref().copied(),
            Some(SplitAction::Resized(state.percent()))
        );
        assert!(!state.initial_cells_active);
        assert_eq!(split.panes(state, area).0.width, before + 2);
    }

    #[test]
    fn cell_resize_preserves_direction_for_signed_extremes_on_large_both_axes() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let area = match axis {
                SplitAxis::Horizontal => Rect::new(0, 0, u16::MAX, 1),
                SplitAxis::Vertical => Rect::new(0, 0, 1, u16::MAX),
            };
            let split = ResizeProbe::split(axis);
            for (percent, delta) in [(90, i16::MIN), (90, i16::MAX), (95, i16::MAX)] {
                let mut state = SplitPaneState::new(percent);
                let before_state = state;
                let (before_first, _) = split.panes(state, area);
                let before_extent = ResizeProbe::extent(before_first, axis);
                if percent == 95 && delta == i16::MAX {
                    assert!(
                        i32::from(before_extent) + i32::from(delta) > i32::from(u16::MAX),
                        "{axis:?} fixture must exceed the u16 target range"
                    );
                }

                let response = split.resize_by_cells(&mut state, area, delta);
                let (after_first, after_second) = split.panes(state, area);
                let after_extent = ResizeProbe::extent(after_first, axis);
                let after_second_extent = ResizeProbe::extent(after_second, axis);

                assert_eq!(response.id(), Some(ID));
                assert!(response.is_consumed());
                assert!(after_extent >= 4, "{axis:?} first pane fell below min");
                assert!(
                    after_second_extent >= 6,
                    "{axis:?} second pane fell below min"
                );
                if delta < 0 {
                    assert!(
                        i32::from(before_extent) - i32::from(after_extent) >= i32::from(i16::MAX),
                        "{axis:?} i16::MIN did not retain its full negative range"
                    );
                } else if percent == 95 {
                    assert!(
                        after_extent >= before_extent,
                        "{axis:?} positive maximum request reversed the boundary"
                    );
                } else {
                    assert!(
                        after_extent > before_extent,
                        "{axis:?} positive maximum request did not move forward"
                    );
                }

                if state != before_state {
                    assert_eq!(
                        response.action_ref().copied(),
                        Some(SplitAction::Resized(state.percent()))
                    );
                } else {
                    assert_eq!(response.action_ref(), None);
                }
            }
        }
    }

    #[test]
    fn cell_resize_rejects_overflowing_minima_without_mutation_on_both_axes() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let area = match axis {
                SplitAxis::Horizontal => Rect::new(0, 0, u16::MAX, 1),
                SplitAxis::Vertical => Rect::new(0, 0, 1, u16::MAX),
            };
            let split = SplitPane::new(ID, axis)
                .gap(0)
                .min_first(u16::MAX)
                .min_second(1)
                .resizable(true)
                .initial_first_cells(20);

            assert_resize_noop(split, state_with_grab(), area, 1);
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
        state.reset(ResetPolicy::Initial, false);
        assert_eq!(state.percent(), 38);
        assert_eq!(state.basis_points(), 3_800);

        state.reset(ResetPolicy::Percent(42), false);
        assert_eq!(state.percent(), 42);
        state.reset(ResetPolicy::Balanced, false);
        assert_eq!(state.percent(), 50);
    }

    #[test]
    fn rejected_initial_cell_nudges_preserve_the_policy_for_a_later_wide_layout() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(2)
            .min_first(28)
            .min_second(40)
            .initial_first_cells(39);
        let wide = Rect::new(0, 0, 120, 24);

        for (area, label) in [
            (Rect::ZERO, "empty area"),
            (Rect::new(0, 0, 69, 24), "area below pane minima"),
        ] {
            let mut state = SplitPaneState::new_with_initial_cells(32);
            let before = state;
            let mut acc = Acc::<SplitAction>::new();

            split.nudge(&mut state, area, 1, &mut acc);
            let _ = acc.finish(ID);

            assert_eq!(
                state, before,
                "{label}: rejected resize must not mutate state"
            );
            assert!(
                state.initial_cells_active,
                "{label}: initial-cell policy remains active"
            );
            assert_eq!(
                split.panes(state, wide).0.width,
                39,
                "{label}: later valid layout retains the configured cell extent"
            );
        }
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
            |model, _| {
                model.maximized = Maximized::None;
                true
            },
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

    #[test]
    fn clear_drag_state_preserves_pointer_derived_layout_and_runtime_owners() {
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

        let seam = runtime
            .area_of_part(ID, PartRef::of(Part::SEAM))
            .unwrap_or(Rect::ZERO);
        assert!(
            seam.width > 0 && seam.height > 1,
            "the padded seam was not registered"
        );
        let press = Position::new(seam.x.saturating_add(seam.width / 2), seam.y);
        assert!(
            seam.contains(press),
            "the top padded cell must be inside the registered seam"
        );
        let first_height_before = DragProbe::first_height(&runtime.app().state, area);
        let first_edge = area.y.saturating_add(first_height_before);
        assert_ne!(
            press.y, first_edge,
            "the press must have a nonzero grab offset"
        );
        let drag = Position::new(press.x, press.y.saturating_add(1));

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Down, press.x, press.y),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(MouseKind::Drag, drag.x, drag.y),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let before = runtime.app().state;
        assert_eq!(
            before.grab_offset,
            Some(i32::from(press.y) - i32::from(first_edge))
        );
        assert_eq!(
            DragProbe::first_height(&before, area),
            first_height_before.saturating_add(1)
        );
        assert_ne!(before.basis_points() % BASIS_POINTS_PER_PERCENT, 0);
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(runtime.focus(), Some(EDITOR_ID));

        let mut expected = before;
        expected.grab_offset = None;
        let response = DragProbe::split().clear_drag_state(&mut runtime.app_mut().state);

        assert_eq!(runtime.app().state, expected);
        assert_eq!(response.id(), Some(ID));
        assert!(response.is_consumed());
        assert!(response.is_changed());
        assert_eq!(response.invalidate(), Invalidate::Paint);
        assert!(response.action_ref().is_none());
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(runtime.focus(), Some(EDITOR_ID));

        let clean_before = runtime.app().state;
        let response = DragProbe::split().clear_drag_state(&mut runtime.app_mut().state);
        assert_eq!(runtime.app().state, clean_before);
        assert_eq!(response.id(), Some(ID));
        assert!(!response.is_consumed());
        assert!(!response.is_changed());
        assert_eq!(response.invalidate(), Invalidate::None);
        assert!(response.action_ref().is_none());
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(runtime.focus(), Some(EDITOR_ID));

        let mut other_state = state_with_grab();
        let mut other_expected = other_state;
        other_expected.grab_offset = None;
        let response =
            SplitPane::new(OTHER_ID, SplitAxis::Horizontal).clear_drag_state(&mut other_state);
        assert_eq!(other_state, other_expected);
        assert_eq!(response.id(), Some(OTHER_ID));
        assert!(response.is_consumed());
        assert!(response.is_changed());
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(runtime.focus(), Some(EDITOR_ID));
    }

    #[test]
    fn clear_drag_state_is_not_guarded_by_split_layout_policy() {
        let state = state_with_grab();
        let mut maximized = state;
        maximized.toggle_max(Maximized::Second);

        for (split, state) in [
            (
                SplitPane::new(ID, SplitAxis::Horizontal).resizable(true),
                state,
            ),
            (
                SplitPane::new(ID, SplitAxis::Vertical).resizable(true),
                state,
            ),
            (
                SplitPane::new(ID, SplitAxis::Horizontal).disabled(true),
                state,
            ),
            (
                SplitPane::new(ID, SplitAxis::Horizontal).fixed_sizing(),
                state,
            ),
            (
                SplitPane::new(ID, SplitAxis::Horizontal).fixed_second_share(40, 4, 6),
                state,
            ),
            (SplitPane::new(ID, SplitAxis::Horizontal), state),
            (
                SplitPane::new(ID, SplitAxis::Horizontal).resizable(true),
                maximized,
            ),
        ] {
            assert_clear_drag_state_changes_only_the_offset(split, state);
        }
    }

    #[test]
    fn clear_drag_state_on_an_already_clean_state_is_an_owner_tagged_noop() {
        let split = SplitPane::new(OTHER_ID, SplitAxis::Vertical);
        let mut state = SplitPaneState::new_with_initial_cells(41);
        state.toggle_max(Maximized::First);
        let before = state;

        let response = split.clear_drag_state(&mut state);

        assert_eq!(state, before);
        assert_eq!(response.id(), Some(OTHER_ID));
        assert!(!response.is_consumed());
        assert!(!response.is_changed());
        assert_eq!(response.invalidate(), Invalidate::None);
        assert!(response.action_ref().is_none());
    }

    #[test]
    fn undersized_fallback_uses_flex_weights_on_the_opposite_axis() {
        let state = SplitPaneState::new_with_initial_cells(37);
        let before = state;
        let fallback = SplitFallback::new(3, 2);
        let cases = [
            (
                SplitAxis::Horizontal,
                Rect::new(2, 3, 18, 10),
                SplitAxis::Vertical,
                Rect::new(2, 3, 18, 6),
                Rect::new(2, 10, 18, 3),
            ),
            (
                SplitAxis::Vertical,
                Rect::new(2, 3, 10, 18),
                SplitAxis::Horizontal,
                Rect::new(2, 3, 6, 18),
                Rect::new(9, 3, 3, 18),
            ),
        ];

        for (primary_axis, area, effective_axis, expected_first, expected_second) in cases {
            let split = SplitPane::new(ID, primary_axis)
                .gap(2)
                .min_first(8)
                .min_second(9)
                .fixed_first_share(40, 4, 8)
                .initial_first_cells(5)
                .undersized_fallback(fallback);
            let resolved = split.resolved_for(&state, area);
            let resolved_again = resolved.resolved_for(&state, area);

            assert_eq!(resolved.axis(), effective_axis);
            assert_eq!(resolved_again.axis(), effective_axis);
            assert_eq!(resolved.undersized_fallback, None);
            assert_eq!(resolved.min_first, 0);
            assert_eq!(resolved.min_second, 0);
            assert_eq!(resolved.gap, 1);
            assert!(resolved.seam.is_none());
            assert_eq!(resolved.fixed_first_share, None);
            assert_eq!(resolved.fixed_second_share, None);
            assert_eq!(resolved.initial_first_cells, None);
            assert!(resolved.fixed_sizing);
            assert!(resolved.table().is_empty());
            assert_eq!(
                resolved.panes(state, area),
                (expected_first, expected_second)
            );
            assert_eq!(
                resolved_again.panes(state, area),
                (expected_first, expected_second)
            );
            assert_eq!(state, before, "resolving changed durable split state");
        }
    }

    #[test]
    fn undersized_fallback_threshold_maximization_and_default_layout_are_preserved() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(2)
            .min_first(8)
            .min_second(9)
            .undersized_fallback(SplitFallback::new(3, 2));
        let state = SplitPaneState::default();
        let at_threshold = Rect::new(0, 0, 19, 12);
        let below_threshold = Rect::new(0, 0, 18, 12);

        assert_eq!(
            split.resolved_for(&state, at_threshold).axis(),
            SplitAxis::Horizontal
        );
        assert_eq!(
            split.resolved_for(&state, at_threshold).resolved_fallback,
            None
        );
        assert_eq!(
            split.resolved_for(&state, below_threshold).axis(),
            SplitAxis::Vertical
        );

        for maximized in [Maximized::First, Maximized::Second] {
            let mut state = SplitPaneState::default();
            state.toggle_max(maximized);
            let resolved = split.resolved_for(&state, Rect::new(0, 0, 1, 12));
            assert_eq!(resolved.axis(), SplitAxis::Horizontal);
            assert_eq!(resolved.resolved_fallback, None);
            let (first, second) = resolved.panes(state, Rect::new(0, 0, 1, 12));
            let visible = if maximized == Maximized::First {
                first
            } else {
                second
            };
            assert_eq!(visible, Rect::new(0, 0, 1, 12));
        }

        let ordinary = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(4)
            .min_second(4);
        let area = Rect::new(0, 0, 20, 8);
        let (first, second) = ordinary.panes(state, area);
        assert_eq!(first, Rect::new(0, 0, 9, 8));
        assert_eq!(second, Rect::new(10, 0, 10, 8));
    }

    #[test]
    fn fallback_threshold_adds_maximum_minima_and_gap_without_u16_overflow() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(u16::MAX)
            .min_first(u16::MAX)
            .min_second(u16::MAX)
            .undersized_fallback(SplitFallback::new(3, 2));
        let state = SplitPaneState::default();
        let area = Rect::new(0, 0, u16::MAX, 1);

        let resolved = split.resolved_for(&state, area);

        assert_eq!(resolved.axis(), SplitAxis::Vertical);
        assert_eq!(resolved.resolved_fallback, Some(SplitFallback::new(3, 2)));
    }

    #[test]
    fn fallback_rounding_gap_and_tiny_geometry_match_flex_allocation() {
        let state = SplitPaneState::default();
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(5)
            .min_second(5)
            .undersized_fallback(SplitFallback::new(0, 0).gap(3));
        let area = Rect::new(0, 0, 8, 10);
        let (first, second) = split.panes(state, area);
        assert_eq!(first, Rect::new(0, 0, 8, 4));
        assert_eq!(second, Rect::new(0, 7, 8, 3));

        let tiny = Rect::new(0, 0, 0, 2);
        let (first, second) = split.panes(state, tiny);
        assert!(first.is_empty());
        assert!(second.is_empty());

        let empty = Rect::ZERO;
        let (first, second) = split.panes(state, empty);
        assert!(first.is_empty());
        assert!(second.is_empty());
    }

    #[test]
    fn fallback_normalizes_each_zero_weight_independently() {
        let state = SplitPaneState::default();
        let area = Rect::new(4, 5, 1, 7);
        let cases = [
            (
                SplitFallback::new(0, 2),
                Rect::new(4, 5, 1, 2),
                Rect::new(4, 8, 1, 4),
            ),
            (
                SplitFallback::new(3, 0),
                Rect::new(4, 5, 1, 5),
                Rect::new(4, 11, 1, 1),
            ),
        ];

        for (fallback, expected_first, expected_second) in cases {
            let split = SplitPane::new(ID, SplitAxis::Horizontal)
                .gap(2)
                .min_first(3)
                .min_second(3)
                .undersized_fallback(fallback);

            assert_eq!(
                split.panes(state, area),
                (expected_first, expected_second),
                "fallback={fallback:?}"
            );
        }
    }

    #[test]
    fn fallback_required_extent_is_the_inverse_of_flex_rounding() {
        let state = SplitPaneState::default();
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(2)
            .min_first(20)
            .min_second(20)
            .undersized_fallback(SplitFallback::new(3, 2));
        let resolved = split.resolved_for(&state, Rect::new(0, 0, 10, 10));

        assert_eq!(resolved.required_extent(&state, 7, 4), 12);
        assert_eq!(resolved.required_extent(&state, 0, 4), 11);
        assert_eq!(resolved.required_extent(&state, 0, 0), 1);

        let large = SplitPane::new(ID, SplitAxis::Horizontal)
            .min_first(20)
            .min_second(20)
            .undersized_fallback(SplitFallback::new(1, 1));
        let resolved = large.resolved_for(&state, Rect::new(0, 0, 10, 10));
        assert_eq!(
            resolved.required_extent(&state, u16::MAX, u16::MAX),
            u16::MAX
        );
    }

    #[test]
    fn fallback_draw_is_inert_and_resize_preserves_live_capture_and_grab_offset() {
        let area = Rect::new(0, 0, 40, 35);
        let (mut runtime, _initial_buffer, _) =
            pointer_derived_resize_runtime(SplitAxis::Horizontal);
        let before = runtime.app().state;
        let fallback = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(22)
            .min_second(22)
            .resizable(true)
            .undersized_fallback(SplitFallback::new(3, 2));

        let response = fallback.resize_by_cells(&mut runtime.app_mut().state, area, 1);
        assert_eq!(runtime.app().state, before);
        assert_eq!(runtime.app().state.grab_offset, before.grab_offset);
        assert_eq!(runtime.capture_owner(), Some(ID));
        assert_eq!(response.id(), Some(ID));
        assert!(response.is_consumed());
        assert_eq!(response.action_ref(), None);

        let state = runtime.app().state;
        let resolved = fallback.resolved_for(&state, area);
        let mut draw_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut draw_buffer = Buffer::empty(area);
        let first = Cell::new(Rect::ZERO);
        let second = Cell::new(Rect::ZERO);
        draw_runtime
            .draw_scene(area, &mut draw_buffer, |ui, _| {
                resolved.draw(ui, area, &state, |_, first_area, second_area| {
                    first.set(first_area);
                    second.set(second_area);
                });
            })
            .commit_presented();

        assert_eq!(first.get().height, 21);
        assert_eq!(second.get().height, 13);
        assert_eq!(first.get().width, area.width);
        assert_eq!(second.get().width, area.width);
        assert!(
            draw_runtime
                .area_of_part(ID, PartRef::of(Part::SEAM))
                .is_none()
        );
        assert!(!draw_runtime.ring().is_registered(ID));
        assert_eq!(
            runtime.app().state,
            state,
            "draw changed durable split state"
        );
    }

    #[test]
    fn second_only_fallback_uses_full_second_on_both_axes_and_preserves_state() {
        let mut state = SplitPaneState::new_with_initial_cells(37);
        state.grab_offset = Some(-2);
        let before = state;
        let fallback = SplitFallback::second_only().gap(9);
        let weighted = SplitFallback::new(0, 0);
        assert!(!weighted.is_second_only());
        assert_eq!(weighted.first_weight, 1);
        assert_eq!(weighted.second_weight, 1);
        assert_eq!(weighted.gap, 1);

        let cases = [
            (
                SplitAxis::Horizontal,
                Rect::new(4, 5, 63, 11),
                Rect::new(4, 5, 62, 11),
                SplitAxis::Horizontal,
            ),
            (
                SplitAxis::Vertical,
                Rect::new(4, 5, 11, 63),
                Rect::new(4, 5, 11, 62),
                SplitAxis::Vertical,
            ),
        ];

        for (primary_axis, threshold_area, undersized_area, effective_axis) in cases {
            let split = SplitPane::new(ID, primary_axis)
                .gap(1)
                .min_first(30)
                .min_second(32)
                .resizable(true)
                .seam_center(2)
                .fixed_first_share(40, 4, 8)
                .initial_first_cells(5)
                .resize_bindings(HORIZONTAL)
                .undersized_fallback(fallback);

            let at_threshold = split.resolved_for(&state, threshold_area);
            assert_eq!(at_threshold.axis(), primary_axis);
            assert_eq!(at_threshold.resolved_fallback, None);
            assert_eq!(at_threshold.effective_maximized(&state), Maximized::None);

            let resolved = split.resolved_for(&state, undersized_area);
            let resolved_again = resolved.resolved_for(&state, undersized_area);
            assert_eq!(resolved.axis(), effective_axis);
            assert_eq!(resolved_again.axis(), effective_axis);
            assert_eq!(resolved.undersized_fallback, None);
            assert_eq!(resolved.resolved_fallback, Some(fallback));
            assert_eq!(resolved.gap, 0, "second-only ignores configured gaps");
            assert_eq!(resolved.min_first, 0);
            assert_eq!(resolved.min_second, 0);
            assert!(resolved.seam.is_none());
            assert!(!resolved.resizable);
            assert_eq!(resolved.fixed_first_share, None);
            assert_eq!(resolved.fixed_second_share, None);
            assert_eq!(resolved.initial_first_cells, None);
            assert_eq!(resolved.resize_bindings, None);
            assert!(resolved.fixed_sizing);
            assert!(resolved.table().is_empty());
            assert_eq!(resolved.effective_maximized(&state), Maximized::Second);

            let (first, second) = resolved.panes(state, undersized_area);
            let (first_again, second_again) = resolved_again.panes(state, undersized_area);
            assert!(first.is_empty());
            assert_eq!(second, undersized_area);
            assert_eq!((first_again, second_again), (first, second));
            assert_eq!(state, before, "resolving changed durable SplitPaneState");
        }
    }

    #[test]
    fn effective_maximized_respects_resolution_and_explicit_state_changes() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(30)
            .min_second(32)
            .undersized_fallback(SplitFallback::second_only());
        let state = SplitPaneState::new_with_initial_cells(37);
        let area = Rect::new(4, 5, 62, 11);
        let before = state;

        assert_eq!(split.effective_maximized(&state), Maximized::None);
        assert_eq!(
            split
                .resolved_for(&state, Rect::new(4, 5, 63, 11))
                .effective_maximized(&state),
            Maximized::None
        );
        let resolved = split.resolved_for(&state, area);
        assert_eq!(resolved.axis(), SplitAxis::Horizontal);
        assert_eq!(resolved.effective_maximized(&state), Maximized::Second);
        assert_eq!(state, before);

        let (first, second) = resolved.panes(state, area);
        assert!(first.is_empty());
        assert_eq!(second, area);

        for maximized in [Maximized::First, Maximized::Second] {
            let mut changed_state = state;
            changed_state.toggle_max(maximized);
            let changed_before = changed_state;

            assert_eq!(split.effective_maximized(&changed_state), maximized);
            let unresolved = split.resolved_for(&changed_state, area);
            assert_eq!(unresolved.axis(), SplitAxis::Horizontal);
            assert_eq!(unresolved.resolved_fallback, None);
            assert_eq!(unresolved.effective_maximized(&changed_state), maximized);

            assert_eq!(resolved.effective_maximized(&changed_state), maximized);
            let changed_snapshot = resolved.resolved_for(&changed_state, area);
            assert_eq!(changed_snapshot.axis(), SplitAxis::Horizontal);
            let (first, second) = changed_snapshot.panes(changed_state, area);
            match maximized {
                Maximized::First => {
                    assert_eq!(first, area);
                    assert!(second.is_empty());
                }
                Maximized::Second => {
                    assert!(first.is_empty());
                    assert_eq!(second, area);
                }
                Maximized::None => unreachable!(),
            }
            assert_eq!(changed_state, changed_before);
        }

        let overflow = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(u16::MAX)
            .min_first(u16::MAX)
            .min_second(u16::MAX)
            .undersized_fallback(SplitFallback::second_only());
        let overflow_resolved = overflow.resolved_for(&state, Rect::new(4, 5, u16::MAX, u16::MAX));
        assert_eq!(overflow_resolved.axis(), SplitAxis::Horizontal);
        assert_eq!(
            overflow_resolved.effective_maximized(&state),
            Maximized::Second
        );
        assert_eq!(state, before);
    }

    #[test]
    fn effective_maximized_respects_explicit_state_changes_on_resolved_weighted_snapshot() {
        let split = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(30)
            .min_second(32)
            .undersized_fallback(SplitFallback::new(3, 2));
        let state = SplitPaneState::new_with_initial_cells(37);
        let before = state;
        let area = Rect::new(4, 5, 62, 11);
        let resolved = split.resolved_for(&state, area);

        assert_eq!(resolved.axis(), SplitAxis::Vertical);
        assert_eq!(resolved.effective_maximized(&state), Maximized::None);
        for maximized in [Maximized::First, Maximized::Second] {
            let mut changed_state = state;
            changed_state.toggle_max(maximized);
            let changed_before = changed_state;
            assert_eq!(resolved.effective_maximized(&changed_state), maximized);

            let (first, second) = resolved.panes(changed_state, area);
            match maximized {
                Maximized::First => {
                    assert_eq!(first, area);
                    assert!(second.is_empty());
                    assert_eq!(resolved.required_extent(&changed_state, 4, 2), 4);
                }
                Maximized::Second => {
                    assert!(first.is_empty());
                    assert_eq!(second, area);
                    assert_eq!(resolved.required_extent(&changed_state, 4, 2), 2);
                }
                Maximized::None => unreachable!(),
            }
            assert_eq!(changed_state, changed_before);
        }
        assert_eq!(state, before);
    }

    #[test]
    fn second_only_preserves_ordinary_zero_cell_and_weighted_tiny_geometry() {
        let state = SplitPaneState::default();
        let ordinary = SplitPane::new(ID, SplitAxis::Horizontal)
            .gap(1)
            .min_first(1)
            .min_second(1);
        let ordinary_area = Rect::new(4, 5, 1, 7);
        let (first, second) = ordinary.panes(state, ordinary_area);
        assert_eq!(ordinary.effective_maximized(&state), Maximized::None);
        assert_eq!(first, ordinary_area);
        assert!(second.is_empty());
        assert_eq!(ordinary.required_extent(&state, 1, 1), 3);

        let weighted = SplitPane::new(ID, SplitAxis::Horizontal)
            .min_first(20)
            .min_second(20)
            .undersized_fallback(SplitFallback::new(3, 2));
        let area = Rect::new(4, 5, 1, 7);
        let (first, second) = weighted.panes(state, area);
        assert_eq!(first, Rect::new(4, 5, 1, 4));
        assert_eq!(second, Rect::new(4, 10, 1, 2));

        let weighted_zero_area = Rect::new(4, 5, 1, 1);
        let weighted_resolved = weighted.resolved_for(&state, weighted_zero_area);
        let (weighted_first, weighted_second) = weighted_resolved.panes(state, weighted_zero_area);
        assert_eq!(weighted_resolved.axis(), SplitAxis::Vertical);
        assert_eq!(
            weighted_resolved.effective_maximized(&state),
            Maximized::None
        );
        assert!(weighted_first.is_empty());
        assert!(weighted_second.is_empty());
        assert_eq!(weighted_resolved.required_extent(&state, 4, 2), 7);
        assert_eq!(weighted_resolved.required_extent(&state, 4, 0), 7);
        assert_eq!(weighted_resolved.required_extent(&state, 0, 2), 6);

        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            let tiny_area = Rect::new(4, 5, 1, 1);
            let tiny = SplitPane::new(ID, axis)
                .min_first(u16::MAX)
                .min_second(u16::MAX)
                .undersized_fallback(SplitFallback::second_only());
            let resolved = tiny.resolved_for(&state, tiny_area);
            let (first, second) = resolved.panes(state, tiny_area);
            assert_eq!(resolved.axis(), axis);
            assert!(first.is_empty());
            assert_eq!(second, tiny_area);
        }
    }
}
