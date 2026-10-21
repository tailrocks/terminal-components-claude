//! Immutable, geometry-only plans for composing measured child regions.
//!
//! These plans deliberately do not know about widgets, input, or rendering. A
//! caller measures each child once, resolves a plan, then uses its logical
//! bounds and clipping bounds for drawing and interaction registration.

use ratatui_core::layout::Rect;

use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};

/// Main-axis alignment for children that do not consume all available space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MainAlign {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

/// Cross-axis alignment for children whose measured size is smaller than the
/// available cross-axis extent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrossAlign {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

/// Behavior when child minimum sizes exceed the available main-axis size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Compress children in source order, clipping any final overflow.
    #[default]
    Clip,
    /// Preserve each child's minimum size; visible bounds remain clipped.
    PreserveMinimum,
}

/// Shared configuration for row, column, and wrapping plans.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Composition {
    pub padding: Insets,
    pub gap: u16,
    pub main_align: MainAlign,
    pub cross_align: CrossAlign,
    pub overflow: OverflowPolicy,
}

/// Main-axis behavior used to measure a nested composition's intrinsic size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionKind {
    Row,
    Column,
    Wrap,
}

impl Composition {
    pub const fn new() -> Self {
        Self {
            padding: Insets::all(0),
            gap: 0,
            main_align: MainAlign::Start,
            cross_align: CrossAlign::Stretch,
            overflow: OverflowPolicy::Clip,
        }
    }

    pub const fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    pub const fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub const fn main_align(mut self, align: MainAlign) -> Self {
        self.main_align = align;
        self
    }

    pub const fn cross_align(mut self, align: CrossAlign) -> Self {
        self.cross_align = align;
        self
    }

    pub const fn overflow(mut self, overflow: OverflowPolicy) -> Self {
        self.overflow = overflow;
        self
    }
}

/// Per-child intrinsic size, bounds, and flexible-space weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChildPolicy {
    pub intrinsic: Size,
    pub min: (u16, u16),
    pub max: (Option<u16>, Option<u16>),
    pub flex: u16,
}

impl ChildPolicy {
    pub const fn new(intrinsic: Size) -> Self {
        Self {
            intrinsic,
            min: (0, 0),
            max: (None, None),
            flex: 0,
        }
    }

    pub const fn min(mut self, width: u16, height: u16) -> Self {
        self.min = (width, height);
        self
    }

    pub const fn max(mut self, width: Option<u16>, height: Option<u16>) -> Self {
        self.max = (width, height);
        self
    }

    pub const fn flex(mut self, weight: u16) -> Self {
        self.flex = weight;
        self
    }
}

/// A child value paired with its layout policy.
#[derive(Debug, PartialEq, Eq)]
pub struct Child<'a, T: ?Sized> {
    pub value: &'a T,
    pub policy: ChildPolicy,
}

impl<T: ?Sized> Copy for Child<'_, T> {}

impl<T: ?Sized> Clone for Child<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized> Child<'a, T> {
    pub const fn new(value: &'a T, policy: ChildPolicy) -> Self {
        Self { value, policy }
    }
}

/// A resolved child's requested bounds and its visible intersection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedChild {
    pub index: usize,
    pub logical: Rect,
    pub visible: Rect,
}

/// Immutable result shared by drawing, clipping, hit testing, and focus bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPlan {
    pub area: Rect,
    pub content: Rect,
    pub children: Vec<ResolvedChild>,
}

/// Measure a composition from child sizes without resolving input or drawing.
///
/// This is shared by nested composition groups so a group can participate as
/// one measured child in its parent. Padding and gaps contribute to the
/// intrinsic size; alignment and flex consume only space supplied by a parent.
pub fn intrinsic_size<T: ?Sized, F>(
    children: &[Child<'_, T>],
    spec: Composition,
    kind: CompositionKind,
    constraints: Constraints,
    measure: F,
) -> Size
where
    F: Fn(&T, Constraints) -> Size,
{
    let content_width = constraints
        .max
        .0
        .saturating_sub(spec.padding.l.saturating_add(spec.padding.r));
    let content_height = constraints
        .max
        .1
        .saturating_sub(spec.padding.t.saturating_add(spec.padding.b));
    let child_constraints = Constraints::loose(content_width, content_height);
    let sizes: Vec<_> = children
        .iter()
        .map(|child| normalized_measure(child, child_constraints, &measure))
        .collect();
    let gaps = u64::from(spec.gap).saturating_mul(children.len().saturating_sub(1) as u64);
    let sum_axis = |horizontal: bool, minimum: bool| {
        sizes
            .iter()
            .enumerate()
            .map(|(index, size)| {
                let child = &children[index];
                if minimum {
                    u64::from(if horizontal { size.min.0 } else { size.min.1 })
                } else {
                    u64::from(axis_preferred(child, *size, horizontal))
                }
            })
            .fold(0u64, u64::saturating_add)
    };
    let max_axis = |horizontal: bool, minimum: bool| {
        sizes
            .iter()
            .enumerate()
            .map(|(index, size)| {
                let child = &children[index];
                if minimum {
                    if horizontal { size.min.0 } else { size.min.1 }
                } else {
                    axis_preferred(child, *size, horizontal)
                }
            })
            .max()
            .unwrap_or(0)
    };
    let (min_w, min_h, pref_w, pref_h) = match kind {
        CompositionKind::Row => (
            sum_axis(true, true).saturating_add(gaps),
            u64::from(max_axis(false, true)),
            sum_axis(true, false).saturating_add(gaps),
            u64::from(max_axis(false, false)),
        ),
        CompositionKind::Column => (
            u64::from(max_axis(true, true)),
            sum_axis(false, true).saturating_add(gaps),
            u64::from(max_axis(true, false)),
            sum_axis(false, false).saturating_add(gaps),
        ),
        CompositionKind::Wrap => {
            let (min_w, min_h) =
                intrinsic_wrap_sizes(children, &sizes, spec.gap, true, content_width);
            let (pref_w, pref_h) =
                intrinsic_wrap_sizes(children, &sizes, spec.gap, false, content_width);
            (
                u64::from(min_w),
                u64::from(min_h),
                u64::from(pref_w),
                u64::from(pref_h),
            )
        }
    };
    let horizontal_padding = u64::from(spec.padding.l).saturating_add(u64::from(spec.padding.r));
    let vertical_padding = u64::from(spec.padding.t).saturating_add(u64::from(spec.padding.b));
    let min = (
        sat_extent(min_w.saturating_add(horizontal_padding)),
        sat_extent(min_h.saturating_add(vertical_padding)),
    );
    let preferred = (
        sat_extent(pref_w.saturating_add(horizontal_padding)).max(min.0),
        sat_extent(pref_h.saturating_add(vertical_padding)).max(min.1),
    );
    Size { min, preferred }.fit(constraints)
}

fn intrinsic_wrap_sizes<T: ?Sized>(
    children: &[Child<'_, T>],
    sizes: &[Size],
    gap: u16,
    minimum: bool,
    width: u16,
) -> (u16, u16) {
    let mut line_width = 0u64;
    let mut line_height = 0u64;
    let mut widest = 0u64;
    let mut total_height = 0u64;
    let mut line_items = 0usize;
    for (index, child) in children.iter().enumerate() {
        let size = sizes[index];
        let item_width = u64::from(if minimum {
            size.min.0
        } else {
            axis_preferred(child, size, true)
        });
        let item_height = u64::from(if minimum {
            size.min.1
        } else {
            axis_preferred(child, size, false)
        });
        let needed = line_width
            .saturating_add(if line_items == 0 { 0 } else { u64::from(gap) })
            .saturating_add(item_width);
        if line_items > 0 && needed > u64::from(width) {
            widest = widest.max(line_width);
            total_height = total_height
                .saturating_add(line_height)
                .saturating_add(u64::from(gap));
            line_width = 0;
            line_height = 0;
            line_items = 0;
        }
        line_width = line_width
            .saturating_add(if line_items == 0 { 0 } else { u64::from(gap) })
            .saturating_add(item_width);
        line_height = line_height.max(item_height);
        line_items += 1;
    }
    if line_items > 0 {
        widest = widest.max(line_width);
        total_height = total_height.saturating_add(line_height);
    }
    (sat_extent(widest), sat_extent(total_height))
}

fn sat_extent(value: u64) -> u16 {
    value.min(u64::from(u16::MAX)) as u16
}

/// Resolve a horizontal row from immutable child data and a pure measurement
/// callback. The callback receives loose constraints bounded by the padded
/// content area and must not perform input handling or domain effects.
pub fn row<T: ?Sized, F>(
    area: Rect,
    children: &[Child<'_, T>],
    spec: Composition,
    measure: F,
) -> ResolvedPlan
where
    F: Fn(&T, Constraints) -> Size,
{
    linear(area, children, spec, true, measure)
}

/// Resolve a vertical column from immutable child data and a pure measurement
/// callback.
pub fn column<T: ?Sized, F>(
    area: Rect,
    children: &[Child<'_, T>],
    spec: Composition,
    measure: F,
) -> ResolvedPlan
where
    F: Fn(&T, Constraints) -> Size,
{
    linear(area, children, spec, false, measure)
}

fn linear<T: ?Sized, F>(
    area: Rect,
    children: &[Child<'_, T>],
    spec: Composition,
    horizontal: bool,
    measure: F,
) -> ResolvedPlan
where
    F: Fn(&T, Constraints) -> Size,
{
    let content = inset(area, spec.padding);
    let constraints = Constraints::loose(content.width, content.height);
    let measured: Vec<Size> = children
        .iter()
        .map(|child| normalized_measure(child, constraints, &measure))
        .collect();
    let main_extent = if horizontal {
        content.width
    } else {
        content.height
    };
    let cross_extent = if horizontal {
        content.height
    } else {
        content.width
    };
    let main_sizes = allocate_main(
        children,
        &measured,
        main_extent,
        spec.gap,
        spec.overflow,
        horizontal,
    );
    let gaps = gap_positions(
        main_sizes.len(),
        spec.gap,
        main_sizes
            .iter()
            .map(|size| u64::from(*size))
            .fold(0u64, u64::saturating_add),
        main_extent,
        spec.main_align,
    );
    let main_used = main_sizes
        .iter()
        .map(|size| u64::from(*size))
        .fold(0u64, u64::saturating_add)
        .saturating_add(
            u64::from(spec.gap).saturating_mul(main_sizes.len().saturating_sub(1) as u64),
        );
    let mut cursor = main_offset(main_extent, main_used, spec.main_align);
    let mut resolved = Vec::with_capacity(children.len());

    for (index, (child, main_size)) in children.iter().zip(main_sizes).enumerate() {
        let measured_cross = if horizontal {
            measured[index].preferred.1
        } else {
            measured[index].preferred.0
        };
        let min_cross = if horizontal {
            measured[index].min.1
        } else {
            measured[index].min.0
        };
        let max_cross = if horizontal {
            child.policy.max.1
        } else {
            child.policy.max.0
        };
        let (cross_start, cross_size) = cross_position(
            cross_extent,
            measured_cross,
            min_cross,
            max_cross,
            spec.cross_align,
        );
        let (x, y, width, height) = if horizontal {
            (
                sat_add(content.x, cursor),
                sat_add(content.y, u64::from(cross_start)),
                main_size,
                cross_size,
            )
        } else {
            (
                sat_add(content.x, u64::from(cross_start)),
                sat_add(content.y, cursor),
                cross_size,
                main_size,
            )
        };
        let logical = Rect {
            x,
            y,
            width,
            height,
        };
        resolved.push(ResolvedChild {
            index,
            logical,
            visible: intersection(logical, content),
        });
        cursor = cursor
            .saturating_add(u64::from(main_size))
            .saturating_add(u64::from(spec.gap));
        if let Some(extra_gap) = gaps.get(index) {
            cursor = cursor.saturating_add(u64::from(*extra_gap));
        }
    }

    ResolvedPlan {
        area,
        content,
        children: resolved,
    }
}

/// Resolve a wrapping row. Children keep source order and wrap before the first
/// item whose preferred width would exceed the content width. Flex is resolved
/// within each line; line height is the largest measured child height.
pub fn wrap<T: ?Sized, F>(
    area: Rect,
    children: &[Child<'_, T>],
    spec: Composition,
    measure: F,
) -> ResolvedPlan
where
    F: Fn(&T, Constraints) -> Size,
{
    let content = inset(area, spec.padding);
    let constraints = Constraints::loose(content.width, content.height);
    let measured: Vec<Size> = children
        .iter()
        .map(|child| normalized_measure(child, constraints, &measure))
        .collect();
    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut line = Vec::new();
    let mut line_width = 0u64;
    for (index, child) in children.iter().enumerate() {
        let width = axis_preferred(child, measured[index], true);
        let needed = if line.is_empty() {
            u64::from(width)
        } else {
            line_width
                .saturating_add(u64::from(spec.gap))
                .saturating_add(u64::from(width))
        };
        if !line.is_empty() && needed > u64::from(content.width) {
            lines.push(std::mem::take(&mut line));
            line_width = 0;
        }
        if !line.is_empty() {
            line_width = line_width.saturating_add(u64::from(spec.gap));
        }
        line_width = line_width.saturating_add(u64::from(width));
        line.push(index);
    }
    if !line.is_empty() {
        lines.push(line);
    }

    let mut resolved = vec![
        ResolvedChild {
            index: 0,
            logical: Rect::default(),
            visible: Rect::default()
        };
        children.len()
    ];
    let mut y = 0u64;
    for indices in lines {
        let line_children: Vec<Child<'_, T>> = indices.iter().map(|&i| children[i]).collect();
        let line_sizes: Vec<Size> = indices.iter().map(|&i| measured[i]).collect();
        let widths = allocate_main(
            &line_children,
            &line_sizes,
            content.width,
            spec.gap,
            spec.overflow,
            true,
        );
        let used_children = widths
            .iter()
            .map(|width| u64::from(*width))
            .fold(0u64, u64::saturating_add);
        let used = used_children.saturating_add(
            u64::from(spec.gap).saturating_mul(widths.len().saturating_sub(1) as u64),
        );
        let mut x = main_offset(content.width, used, spec.main_align);
        let extra_gaps = gap_positions(
            indices.len(),
            spec.gap,
            used_children,
            content.width,
            spec.main_align,
        );
        let line_height = indices
            .iter()
            .map(|&i| axis_preferred(&children[i], measured[i], false))
            .max()
            .unwrap_or(0);
        for (local, (&index, width)) in indices.iter().zip(widths).enumerate() {
            let child = children[index];
            let height = cross_position(
                line_height,
                measured[index].preferred.1,
                measured[index].min.1,
                child.policy.max.1,
                spec.cross_align,
            );
            let logical = Rect {
                x: sat_add(content.x, x),
                y: sat_add(content.y, y.saturating_add(u64::from(height.0))),
                width,
                height: height.1,
            };
            resolved[index] = ResolvedChild {
                index,
                logical,
                visible: intersection(logical, content),
            };
            x = x
                .saturating_add(u64::from(width))
                .saturating_add(u64::from(spec.gap));
            if let Some(extra_gap) = extra_gaps.get(local) {
                x = x.saturating_add(u64::from(*extra_gap));
            }
        }
        y = y
            .saturating_add(u64::from(line_height))
            .saturating_add(u64::from(spec.gap));
    }
    ResolvedPlan {
        area,
        content,
        children: resolved,
    }
}

fn normalized_measure<T: ?Sized, F>(
    child: &Child<'_, T>,
    constraints: Constraints,
    measure: &F,
) -> Size
where
    F: Fn(&T, Constraints) -> Size,
{
    let measured = measure(child.value, constraints);
    let max_w = child.policy.max.0.unwrap_or(u16::MAX);
    let max_h = child.policy.max.1.unwrap_or(u16::MAX);
    let min_w = child
        .policy
        .min
        .0
        .max(child.policy.intrinsic.min.0)
        .max(measured.min.0)
        .min(max_w);
    let min_h = child
        .policy
        .min
        .1
        .max(child.policy.intrinsic.min.1)
        .max(measured.min.1)
        .min(max_h);
    let pref_w = measured
        .preferred
        .0
        .max(child.policy.intrinsic.preferred.0)
        .max(min_w)
        .min(max_w);
    let pref_h = measured
        .preferred
        .1
        .max(child.policy.intrinsic.preferred.1)
        .max(min_h)
        .min(max_h);
    Size {
        min: (min_w, min_h),
        preferred: (pref_w, pref_h),
    }
}

fn axis_preferred<T: ?Sized>(child: &Child<'_, T>, measured: Size, horizontal: bool) -> u16 {
    let preferred = if horizontal {
        measured.preferred.0
    } else {
        measured.preferred.1
    };
    let max = if horizontal {
        child.policy.max.0
    } else {
        child.policy.max.1
    };
    max.map_or(preferred, |limit| preferred.min(limit))
}

fn allocate_main<T: ?Sized>(
    children: &[Child<'_, T>],
    measured: &[Size],
    extent: u16,
    gap: u16,
    overflow: OverflowPolicy,
    horizontal: bool,
) -> Vec<u16> {
    let count = children.len();
    let gaps = u64::from(gap)
        .saturating_mul(count.saturating_sub(1) as u64)
        .min(u64::from(extent));
    let available = u64::from(extent).saturating_sub(gaps);
    let min = |i: usize| -> u64 {
        u64::from(if horizontal {
            measured[i].min.0
        } else {
            measured[i].min.1
        })
    };
    let pref = |i: usize| -> u64 {
        u64::from(axis_preferred(&children[i], measured[i], horizontal)).max(min(i))
    };
    let min_total = (0..count).map(min).fold(0u64, u64::saturating_add);
    let mut sizes: Vec<u64> = (0..count).map(pref).collect();
    let total = sizes.iter().copied().fold(0u64, u64::saturating_add);
    if total > available {
        let mut excess = total - available;
        for (i, size) in sizes.iter_mut().enumerate() {
            let reducible = size.saturating_sub(min(i));
            let reduce = reducible.min(excess);
            *size -= reduce;
            excess -= reduce;
            if excess == 0 {
                break;
            }
        }
    } else {
        distribute_flex(children, &mut sizes, horizontal, available - total);
    }
    if min_total > available && overflow == OverflowPolicy::Clip {
        let mut left = available;
        for (i, size) in sizes.iter_mut().enumerate() {
            let assigned = min(i).min(left);
            *size = assigned;
            left -= assigned;
        }
    }
    sizes
        .into_iter()
        .map(|size| size.min(u64::from(u16::MAX)) as u16)
        .collect()
}

fn distribute_flex<T: ?Sized>(
    children: &[Child<'_, T>],
    sizes: &mut [u64],
    horizontal: bool,
    mut remaining: u64,
) {
    while remaining > 0 {
        let mut total_weight = 0u64;
        for (index, child) in children.iter().enumerate() {
            let max = if horizontal {
                child.policy.max.0
            } else {
                child.policy.max.1
            };
            if child.policy.flex > 0 && !max.is_some_and(|limit| sizes[index] >= u64::from(limit)) {
                total_weight = total_weight.saturating_add(u64::from(child.policy.flex));
            }
        }
        if total_weight == 0 {
            break;
        }

        let round = remaining;
        let mut assigned = 0u64;
        for (index, child) in children.iter().enumerate() {
            let max = if horizontal {
                child.policy.max.0
            } else {
                child.policy.max.1
            };
            if child.policy.flex == 0 || max.is_some_and(|limit| sizes[index] >= u64::from(limit)) {
                continue;
            }
            let share = round.saturating_mul(u64::from(child.policy.flex)) / total_weight;
            let capped = max.map_or(share, |limit| {
                share.min(u64::from(limit).saturating_sub(sizes[index]))
            });
            sizes[index] = sizes[index].saturating_add(capped);
            assigned = assigned.saturating_add(capped);
        }
        remaining = remaining.saturating_sub(assigned);
        if assigned > 0 {
            continue;
        }

        // Floor division may leave fewer cells than active children. Hand
        // those cells out in source order, then recalculate weighted shares.
        let mut one_each = 0u64;
        for (index, child) in children.iter().enumerate() {
            if remaining == 0 {
                break;
            }
            let max = if horizontal {
                child.policy.max.0
            } else {
                child.policy.max.1
            };
            if child.policy.flex == 0 || max.is_some_and(|limit| sizes[index] >= u64::from(limit)) {
                continue;
            }
            sizes[index] = sizes[index].saturating_add(1);
            remaining -= 1;
            one_each += 1;
        }
        if one_each == 0 {
            break;
        }
    }
}

fn main_offset(extent: u16, used: u64, align: MainAlign) -> u64 {
    match align {
        MainAlign::Start | MainAlign::SpaceBetween => 0,
        MainAlign::Center => u64::from(extent).saturating_sub(used) / 2,
        MainAlign::End => u64::from(extent).saturating_sub(used),
    }
}

fn gap_positions(
    count: usize,
    gap: u16,
    used_children: u64,
    extent: u16,
    align: MainAlign,
) -> Vec<u16> {
    #[cfg(test)]
    GAP_POSITION_CALLS.with(|calls| calls.set(calls.get().saturating_add(1)));
    let gaps = count.saturating_sub(1);
    if gaps == 0 || align != MainAlign::SpaceBetween {
        return vec![0; count];
    }
    let gap_count = gaps as u64;
    let used = used_children.saturating_add(u64::from(gap).saturating_mul(gap_count));
    let extra = u64::from(extent).saturating_sub(used) / gap_count;
    let remainder = u64::from(extent).saturating_sub(used) % gap_count;
    (0..count)
        .map(|i| {
            if i < gaps {
                (extra + u64::from((i as u64) < remainder)).min(u64::from(u16::MAX)) as u16
            } else {
                0
            }
        })
        .collect()
}

#[cfg(test)]
std::thread_local! {
    static GAP_POSITION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn cross_position(
    extent: u16,
    preferred: u16,
    min: u16,
    max: Option<u16>,
    align: CrossAlign,
) -> (u16, u16) {
    let size = match align {
        CrossAlign::Stretch => max
            .map_or(extent, |limit| extent.min(limit))
            .max(min.min(extent)),
        _ => preferred.max(min).min(max.unwrap_or(u16::MAX)).min(extent),
    };
    let offset = match align {
        CrossAlign::Start | CrossAlign::Stretch => 0,
        CrossAlign::Center => extent.saturating_sub(size) / 2,
        CrossAlign::End => extent.saturating_sub(size),
    };
    (offset, size)
}

fn sat_add(origin: u16, offset: u64) -> u16 {
    u64::from(origin)
        .saturating_add(offset)
        .min(u64::from(u16::MAX)) as u16
}

fn intersection(a: Rect, b: Rect) -> Rect {
    let left = u32::from(a.x).max(u32::from(b.x));
    let top = u32::from(a.y).max(u32::from(b.y));
    let right = u32::from(a.x)
        .saturating_add(u32::from(a.width))
        .min(u32::from(b.x).saturating_add(u32::from(b.width)));
    let bottom = u32::from(a.y)
        .saturating_add(u32::from(a.height))
        .min(u32::from(b.y).saturating_add(u32::from(b.height)));
    let clip_right = u32::from(b.x).saturating_add(u32::from(b.width));
    let clip_bottom = u32::from(b.y).saturating_add(u32::from(b.height));
    let left = left.clamp(u32::from(b.x), clip_right);
    let top = top.clamp(u32::from(b.y), clip_bottom);
    Rect {
        x: left.min(u32::from(u16::MAX)) as u16,
        y: top.min(u32::from(u16::MAX)) as u16,
        width: right.saturating_sub(left).min(u32::from(u16::MAX)) as u16,
        height: bottom.saturating_sub(top).min(u32::from(u16::MAX)) as u16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::layout::Rect;

    fn item(value: &'static str, size: (u16, u16)) -> Child<'static, str> {
        Child::new(value, ChildPolicy::new(Size::exact(size.0, size.1)))
    }

    #[test]
    fn nested_intrinsic_wrap_handles_wide_narrow_empty_and_bounded_children() {
        let labels = [String::from("a"), String::from("b")];
        let children = [
            Child::new(
                labels[0].as_str(),
                ChildPolicy::default().min(2, 1).max(Some(4), None),
            ),
            Child::new(labels[1].as_str(), ChildPolicy::default().min(2, 1)),
        ];
        let measure = |name: &str, _: Constraints| {
            if name == "a" {
                Size {
                    min: (2, 1),
                    preferred: (3, 1),
                }
            } else {
                Size {
                    min: (2, 1),
                    preferred: (4, 2),
                }
            }
        };
        let narrow = intrinsic_size(
            &children,
            Composition::new().gap(1),
            CompositionKind::Wrap,
            Constraints::loose(4, 10),
            measure,
        );
        assert_eq!(narrow.min, (2, 3));
        assert_eq!(narrow.preferred, (4, 4));

        let wide = intrinsic_size(
            &children,
            Composition::new().gap(1),
            CompositionKind::Wrap,
            Constraints::loose(10, 10),
            measure,
        );
        assert_eq!(wide.preferred, (8, 2));

        let empty: [Child<'_, str>; 0] = [];
        let padded = intrinsic_size(
            &empty,
            Composition::new().padding(Insets::all(2)),
            CompositionKind::Column,
            Constraints::loose(8, 8),
            |_, _| Size::default(),
        );
        assert_eq!(padded.preferred, (4, 4));

        let zero = intrinsic_size(
            &children,
            Composition::new(),
            CompositionKind::Wrap,
            Constraints::loose(0, 0),
            measure,
        );
        assert_eq!(zero.preferred, (0, 0));
    }

    #[test]
    fn row_resolves_intrinsic_flex_padding_gap_alignment_and_clips() {
        let children = [
            item("label", (3, 1)),
            Child::new(
                "body",
                ChildPolicy::new(Size::exact(2, 1))
                    .min(2, 1)
                    .max(Some(8), Some(2))
                    .flex(1),
            ),
        ];
        let plan = row(
            Rect::new(4, 3, 12, 5),
            &children,
            Composition::new().padding(Insets::symmetric(1, 1)).gap(1),
            |value, _| {
                if value == "label" {
                    Size::exact(3, 1)
                } else {
                    Size::exact(2, 1)
                }
            },
        );
        assert_eq!(plan.content, Rect::new(5, 4, 10, 3));
        assert_eq!(plan.children[0].logical, Rect::new(5, 4, 3, 3));
        assert_eq!(plan.children[1].logical, Rect::new(9, 4, 6, 2));
        assert_eq!(plan.children[1].visible, plan.children[1].logical);
    }

    #[test]
    fn row_handles_shifted_origins_tiny_area_and_conflicting_bounds() {
        let children = [
            Child::new("a", ChildPolicy::new(Size::exact(4, 1)).min(5, 1)),
            Child::new("b", ChildPolicy::new(Size::exact(4, 1)).min(5, 1)),
        ];
        let area = Rect::new(17, 8, 6, 1);
        let clipped = row(area, &children, Composition::new().gap(1), |_, _| {
            Size::exact(4, 1)
        });
        assert_eq!(
            clipped
                .children
                .iter()
                .map(|child| child.logical.width)
                .collect::<Vec<_>>(),
            [5, 0]
        );
        assert!(
            clipped
                .children
                .iter()
                .all(|child| child.visible.x >= area.x)
        );
        let preserved = row(
            area,
            &children,
            Composition::new()
                .gap(1)
                .overflow(OverflowPolicy::PreserveMinimum),
            |_, _| Size::exact(4, 1),
        );
        assert_eq!(
            preserved
                .children
                .iter()
                .map(|child| child.logical.width)
                .collect::<Vec<_>>(),
            [5, 5]
        );
        assert_eq!(preserved.children[1].visible.width, 0);
        let min_larger_than_area = [Child::new(
            "wide",
            ChildPolicy::new(Size::default()).min(8, 1),
        )];
        let preserved = row(
            area,
            &min_larger_than_area,
            Composition::new().overflow(OverflowPolicy::PreserveMinimum),
            |_, _| Size::default(),
        );
        assert_eq!(preserved.children[0].logical.width, 8);
        assert_eq!(preserved.children[0].visible.width, area.width);
        let tiny = column(
            Rect::new(9, 7, 0, 0),
            &children,
            Composition::new().padding(Insets::all(u16::MAX)),
            |_, _| Size::exact(u16::MAX, u16::MAX),
        );
        assert!(tiny.children.iter().all(|child| child.visible.is_empty()));
    }

    #[test]
    fn intrinsic_minimum_participates_in_overflow_compression() {
        let children = [
            Child::new("first", ChildPolicy::new(Size::exact(8, 1))),
            Child::new("second", ChildPolicy::new(Size::exact(8, 1))),
        ];
        let plan = row(
            Rect::new(0, 0, 8, 1),
            &children,
            Composition::new(),
            |_, _| Size::default(),
        );
        assert_eq!(
            plan.children
                .iter()
                .map(|child| child.logical.width)
                .collect::<Vec<_>>(),
            [8, 0]
        );
    }

    #[test]
    fn main_alignment_and_space_between_are_deterministic() {
        let children = [item("a", (2, 1)), item("b", (2, 1)), item("c", (2, 1))];
        let centered = row(
            Rect::new(10, 0, 11, 2),
            &children,
            Composition::new().main_align(MainAlign::Center),
            |_, _| Size::exact(2, 1),
        );
        assert_eq!(centered.children[0].logical.x, 12);
        let spaced = row(
            Rect::new(0, 0, 11, 2),
            &children,
            Composition::new().main_align(MainAlign::SpaceBetween),
            |_, _| Size::exact(2, 1),
        );
        assert_eq!(
            spaced
                .children
                .iter()
                .map(|child| child.logical.x)
                .collect::<Vec<_>>(),
            [0, 5, 9]
        );
    }

    #[test]
    fn flex_respects_individual_maxima_and_reallocates_the_remainder() {
        let children = [
            Child::new(
                "bounded",
                ChildPolicy::new(Size::exact(1, 1))
                    .max(Some(3), None)
                    .flex(1),
            ),
            Child::new("open", ChildPolicy::new(Size::exact(1, 1)).flex(1)),
        ];
        let plan = row(
            Rect::new(0, 0, 10, 1),
            &children,
            Composition::new(),
            |_, _| Size::exact(1, 1),
        );
        assert_eq!(
            plan.children
                .iter()
                .map(|child| child.logical.width)
                .collect::<Vec<_>>(),
            [3, 7]
        );
    }

    #[test]
    fn wrap_preserves_order_and_uses_line_height() {
        let children = [item("a", (4, 1)), item("b", (4, 2)), item("c", (4, 1))];
        let plan = wrap(
            Rect::new(3, 4, 9, 6),
            &children,
            Composition::new().gap(1),
            |value, _| match value {
                "b" => Size::exact(4, 2),
                _ => Size::exact(4, 1),
            },
        );
        assert_eq!(
            plan.children
                .iter()
                .map(|child| child.logical.x)
                .collect::<Vec<_>>(),
            [3, 8, 3]
        );
        assert_eq!(
            plan.children
                .iter()
                .map(|child| child.logical.y)
                .collect::<Vec<_>>(),
            [4, 4, 7]
        );
        assert_eq!(plan.children[1].logical.height, 2);
    }

    #[test]
    fn wrap_space_between_distributes_only_the_remaining_line_width() {
        let children = [item("a", (4, 1)), item("b", (4, 1)), item("c", (4, 1))];
        let plan = wrap(
            Rect::new(3, 4, 11, 5),
            &children,
            Composition::new()
                .gap(1)
                .main_align(MainAlign::SpaceBetween),
            |_, _| Size::exact(4, 1),
        );
        assert_eq!(
            plan.children
                .iter()
                .map(|child| (child.logical.x, child.logical.y))
                .collect::<Vec<_>>(),
            [(3, 4), (10, 4), (3, 6)]
        );
    }

    #[test]
    fn measurement_callback_is_read_only_and_once_per_child() {
        use std::cell::Cell;

        let children = [item("one", (1, 1)), item("two", (1, 1))];
        let calls = Cell::new(0);
        let plan = column(
            Rect::new(0, 0, 10, 10),
            &children,
            Composition::new(),
            |value, constraints| {
                calls.set(calls.get() + 1);
                assert_eq!((constraints.max.0, constraints.max.1), (10, 10));
                if value == "one" {
                    Size::exact(2, 2)
                } else {
                    Size::exact(3, 3)
                }
            },
        );
        assert_eq!(plan.children.len(), 2);
        assert_eq!(plan.children[0].logical.height, 2);
        assert_eq!(plan.children[1].logical.height, 3);
        assert_eq!(calls.get(), children.len());
    }

    #[test]
    fn large_child_lists_resolve_without_overflow_or_out_of_bounds_visible_regions() {
        let labels = vec!["item"; 4_000];
        let children: Vec<_> = labels
            .iter()
            .map(|label| Child::new(label, ChildPolicy::new(Size::exact(1, 1)).flex(1)))
            .collect();
        let area = Rect::new(31, 9, 80, 4);
        let plan = row(area, &children, Composition::new().gap(1), |_, _| {
            Size::exact(1, 1)
        });
        assert_eq!(plan.children.len(), labels.len());
        assert!(plan.children.iter().all(|child| {
            child.visible.x >= area.x
                && child.visible.y >= area.y
                && u32::from(child.visible.x) + u32::from(child.visible.width)
                    <= u32::from(area.x) + u32::from(area.width)
        }));
    }

    #[test]
    fn very_large_preserved_minimum_sum_does_not_overflow() {
        let child = Child::new(
            "wide",
            ChildPolicy::new(Size::exact(u16::MAX, 1)).min(u16::MAX, 1),
        );
        let children = vec![child; 65_538];
        let plan = row(
            Rect::new(0, 0, u16::MAX, 1),
            &children,
            Composition::new().overflow(OverflowPolicy::PreserveMinimum),
            |_, _| Size::default(),
        );
        assert_eq!(plan.children.len(), children.len());
        assert_eq!(plan.children[0].logical.width, u16::MAX);
        assert_eq!(plan.children.last().unwrap().logical.width, u16::MAX);
    }

    #[test]
    fn wide_wrap_resolves_gap_positions_once_per_line() {
        let children = vec![Child::new("empty", ChildPolicy::new(Size::default())); 8_000];
        GAP_POSITION_CALLS.with(|calls| calls.set(0));
        let plan = wrap(
            Rect::new(1, 1, 1, 1),
            &children,
            Composition::new().main_align(MainAlign::SpaceBetween),
            |_, _| Size::default(),
        );
        let calls = GAP_POSITION_CALLS.with(std::cell::Cell::get);
        assert_eq!(calls, 1);
        assert_eq!(plan.children.len(), children.len());
        assert_eq!(plan.children[0].logical.x, 1);
        assert_eq!(plan.children[1].logical.x, 2);
        assert_eq!(plan.children.last().unwrap().logical.x, 2);
    }
}
