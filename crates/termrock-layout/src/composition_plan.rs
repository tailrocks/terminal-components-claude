use ratatui_core::layout::Rect;

use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MainAlign {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrossAlign {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowPolicy {
    #[default]
    Clip,

    PreserveMinimum,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Composition {
    pub padding: Insets,
    pub gap: u16,
    pub main_align: MainAlign,
    pub cross_align: CrossAlign,
    pub overflow: OverflowPolicy,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedChild {
    pub index: usize,
    pub logical: Rect,
    pub visible: Rect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPlan {
    pub area: Rect,
    pub content: Rect,
    pub children: Vec<ResolvedChild>,
}

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
