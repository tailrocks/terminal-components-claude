use ratatui_core::layout::{Margin, Position, Rect};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Fixed(u16),
    Flex(u16),
    Auto,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowAlign {
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Insets {
    pub l: u16,
    pub t: u16,
    pub r: u16,
    pub b: u16,
}

impl Insets {
    pub const fn all(n: u16) -> Self {
        Insets {
            l: n,
            t: n,
            r: n,
            b: n,
        }
    }

    pub const fn symmetric(h: u16, v: u16) -> Self {
        Insets {
            l: h,
            t: v,
            r: h,
            b: v,
        }
    }
}

pub fn distribute(total: u16, tracks: &[Track], spacing: u16, natural: Option<&[u16]>) -> Vec<u16> {
    let n = tracks.len() as u16;
    let gaps = spacing.saturating_mul(n.saturating_sub(1));
    let mut avail = total.saturating_sub(gaps);
    let mut out = vec![0u16; tracks.len()];
    let has_flex = tracks.iter().any(|t| matches!(t, Track::Flex(_)));

    for (i, t) in tracks.iter().enumerate() {
        let want = match t {
            Track::Fixed(w) => *w,
            Track::Auto => match natural.and_then(|n| n.get(i)) {
                Some(w) => *w,
                None if has_flex => 1,
                None => 0,
            },
            Track::Flex(_) => 0,
        };
        let give = want.min(avail);
        avail = avail.saturating_sub(give);
        if let Some(slot) = out.get_mut(i) {
            *slot = give;
        }
    }

    let weight_of = |t: &Track| -> u32 {
        match t {
            Track::Flex(w) => u32::from(*w),
            Track::Auto if !has_flex && natural.is_none() => 1,
            _ => 0,
        }
    };
    let weights: u32 = tracks.iter().map(weight_of).sum();
    if weights == 0 {
        return out;
    }
    let mut remainder = avail;
    for (i, t) in tracks.iter().enumerate() {
        let w = weight_of(t);
        if w == 0 {
            continue;
        }
        let share = (u32::from(avail).saturating_mul(w))
            .checked_div(weights)
            .unwrap_or(0) as u16;
        if let Some(slot) = out.get_mut(i) {
            *slot = share;
        }
        remainder = remainder.saturating_sub(share);
    }
    for (i, t) in tracks.iter().enumerate() {
        if remainder == 0 {
            break;
        }
        if weight_of(t) == 0 {
            continue;
        }
        if let Some(slot) = out.get_mut(i) {
            *slot = slot.saturating_add(1);
        }
        remainder = remainder.saturating_sub(1);
    }
    out
}

pub fn distribute_into(total: u16, tracks: &[Track], spacing: u16, out: &mut [u16]) {
    let n = tracks.len().min(out.len());
    let tracks = tracks.get(..n).unwrap_or(&[]);
    let gaps = spacing.saturating_mul((n as u16).saturating_sub(1));
    let mut avail = total.saturating_sub(gaps);
    let has_flex = tracks.iter().any(|t| matches!(t, Track::Flex(_)));
    for (i, t) in tracks.iter().enumerate() {
        let want = match t {
            Track::Fixed(w) => *w,
            Track::Auto if has_flex => 1,
            Track::Auto | Track::Flex(_) => 0,
        };
        let give = want.min(avail);
        avail = avail.saturating_sub(give);
        if let Some(slot) = out.get_mut(i) {
            *slot = give;
        }
    }
    let weight_of = |t: &Track| -> u32 {
        match t {
            Track::Flex(w) => u32::from(*w),
            Track::Auto if !has_flex => 1,
            _ => 0,
        }
    };
    let weights: u32 = tracks.iter().map(weight_of).sum();
    if weights == 0 {
        return;
    }
    let mut remainder = avail;
    for (i, t) in tracks.iter().enumerate() {
        let w = weight_of(t);
        if w == 0 {
            continue;
        }
        let share = (u32::from(avail).saturating_mul(w))
            .checked_div(weights)
            .unwrap_or(0) as u16;
        if let Some(slot) = out.get_mut(i) {
            *slot = share;
        }
        remainder = remainder.saturating_sub(share);
    }
    for (i, t) in tracks.iter().enumerate() {
        if remainder == 0 {
            break;
        }
        if weight_of(t) == 0 {
            continue;
        }
        if let Some(slot) = out.get_mut(i) {
            *slot = slot.saturating_add(1);
        }
        remainder = remainder.saturating_sub(1);
    }
}

fn stack(area: Rect, sizes: &[u16], spacing: u16, vertical: bool) -> Vec<Rect> {
    let mut cursor = if vertical { area.y } else { area.x };
    let limit = if vertical {
        area.bottom()
    } else {
        area.right()
    };
    sizes
        .iter()
        .map(|&s| {
            let start = cursor.min(limit);
            let len = s.min(limit.saturating_sub(start));
            cursor = start.saturating_add(len).saturating_add(spacing);
            if vertical {
                Rect {
                    x: area.x,
                    y: start,
                    width: area.width,
                    height: len,
                }
            } else {
                Rect {
                    x: start,
                    y: area.y,
                    width: len,
                    height: area.height,
                }
            }
        })
        .collect()
}

pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect> {
    stack(area, &distribute(area.height, heights, 0, None), 0, true)
}

pub fn rows_measured(area: Rect, heights: &[Track], natural: &[u16]) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.height, heights, 0, Some(natural)),
        0,
        true,
    )
}

pub fn columns(area: Rect, widths: &[Track], spacing: u16) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.width, widths, spacing, None),
        spacing,
        false,
    )
}

pub fn columns_measured(area: Rect, widths: &[Track], spacing: u16, natural: &[u16]) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.width, widths, spacing, Some(natural)),
        spacing,
        false,
    )
}

pub fn responsive_columns(area: Rect, spec: &[Track], spacing: u16, stack_below: u16) -> Vec<Rect> {
    if area.width < stack_below {
        let tracks: Vec<Track> = spec.iter().map(|_| Track::Flex(1)).collect();
        rows(area, &tracks)
    } else {
        columns(area, spec, spacing)
    }
}

pub fn action_row(area: Rect, widths: &[u16], spacing: u16, align: RowAlign) -> Vec<Rect> {
    let total: u16 = widths.iter().fold(0u16, |acc, w| acc.saturating_add(*w));
    let gaps = spacing.saturating_mul((widths.len() as u16).saturating_sub(1));
    let used = total.saturating_add(gaps).min(area.width);
    let start = match align {
        RowAlign::Start => area.x,
        RowAlign::End => area.right().saturating_sub(used),
    };
    let base = Rect {
        x: start,
        y: area.y,
        width: used,
        height: area.height,
    };
    stack(base, widths, spacing, false)
}

pub fn inset(area: Rect, i: Insets) -> Rect {
    if i.l == i.r && i.t == i.b {
        let r = area.inner(Margin::new(i.l, i.t));

        if !r.is_empty() {
            return r;
        }
    }
    Rect {
        x: area.x.saturating_add(i.l).min(area.right()),
        y: area.y.saturating_add(i.t).min(area.bottom()),
        width: area.width.saturating_sub(i.l).saturating_sub(i.r),
        height: area.height.saturating_sub(i.t).saturating_sub(i.b),
    }
}

pub fn split_v(area: Rect, at: u16) -> (Rect, Rect) {
    let at = at.min(area.height);
    (
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: at,
        },
        Rect {
            x: area.x,
            y: area.y.saturating_add(at),
            width: area.width,
            height: area.height.saturating_sub(at),
        },
    )
}

pub fn split_h(area: Rect, at: u16) -> (Rect, Rect) {
    let at = at.min(area.width);
    (
        Rect {
            x: area.x,
            y: area.y,
            width: at,
            height: area.height,
        },
        Rect {
            x: area.x.saturating_add(at),
            y: area.y,
            width: area.width.saturating_sub(at),
            height: area.height,
        },
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Maximized {
    #[default]
    None,
    First,
    Second,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SplitModel {
    pub percent: u8,
    pub min_first: u16,
    pub min_second: u16,
    pub maximized: Maximized,
    pub axis: SplitAxis,
}

impl SplitModel {
    pub const fn new(axis: SplitAxis, percent: u8, min_first: u16, min_second: u16) -> Self {
        SplitModel {
            percent: clamp_percent(percent),
            min_first,
            min_second,
            maximized: Maximized::None,
            axis,
        }
    }

    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    pub fn grow(&mut self, delta: i8) {
        let p = i16::from(self.percent).saturating_add(i16::from(delta));
        self.percent = clamp_percent(p.clamp(0, 100) as u8);
    }

    const fn length(self, area: Rect) -> u16 {
        match self.axis {
            SplitAxis::Vertical => area.height,
            SplitAxis::Horizontal => area.width,
        }
    }

    const fn collapsed(self, area: Rect, second: bool) -> Rect {
        match (self.axis, second) {
            (SplitAxis::Vertical, false) => Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 0,
            },
            (SplitAxis::Vertical, true) => Rect {
                x: area.x,
                y: area.bottom(),
                width: area.width,
                height: 0,
            },
            (SplitAxis::Horizontal, false) => Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: area.height,
            },
            (SplitAxis::Horizontal, true) => Rect {
                x: area.right(),
                y: area.y,
                width: 0,
                height: area.height,
            },
        }
    }

    pub fn layout(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        self.layout_with_ratio(area, gap, u32::from(self.percent), 100)
    }

    /// Resolve this split using a precise first-pane ratio in basis points.
    ///
    /// The public `percent` field remains the coarse compatibility value. This
    /// adapter lets stateful controls retain a cell-accurate ratio while a
    /// split is resized, without adding a required field to `SplitModel`.
    pub fn layout_with_basis_points(
        &self,
        area: Rect,
        gap: u16,
        basis_points: u16,
    ) -> (Rect, Rect) {
        self.layout_with_ratio(area, gap, u32::from(basis_points.clamp(500, 9_500)), 10_000)
    }

    /// Resolve this split from a fixed percentage of the full extent for the
    /// second pane. The requested second extent is floored, then clamped to
    /// its configured bounds and the first pane's minimum. The ordinary
    /// basis-point resolver remains the default for resizable splits.
    pub fn layout_with_second_share(
        &self,
        area: Rect,
        gap: u16,
        percent: u8,
        min_second: u16,
        max_second: u16,
    ) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => return (area, self.collapsed(area, true)),
            Maximized::Second => return (self.collapsed(area, false), area),
            Maximized::None => {}
        }

        let extent = self.length(area);
        let usable = extent.saturating_sub(gap);
        if usable < self.min_first.saturating_add(self.min_second) {
            return (area, self.collapsed(area, true));
        }
        let requested = (u32::from(extent).saturating_mul(u32::from(percent.min(100))) / 100)
            .min(u32::from(u16::MAX)) as u16;
        let second = requested
            .clamp(min_second.min(max_second), max_second.max(min_second))
            .min(usable.saturating_sub(self.min_first))
            .max(self.min_second);
        let first = usable.saturating_sub(second);
        match self.axis {
            SplitAxis::Vertical => {
                let (first, rest) = split_v(area, first);
                let (_, second) = split_v(rest, gap.min(rest.height));
                (first, second)
            }
            SplitAxis::Horizontal => {
                let (first, rest) = split_h(area, first);
                let (_, second) = split_h(rest, gap.min(rest.width));
                (first, second)
            }
        }
    }

    /// Return the smallest extent whose resolved panes can hold both requested
    /// along-axis sizes at this precise ratio. This calls the same canonical
    /// split resolver as rendering, so rounding and pane minima stay aligned.
    pub fn required_extent_with_basis_points(
        self,
        gap: u16,
        basis_points: u16,
        first: u16,
        second: u16,
    ) -> u16 {
        if self.maximized == Maximized::First {
            return first;
        }
        if self.maximized == Maximized::Second {
            return second;
        }

        let usable_min = u32::from(first).saturating_add(u32::from(second));
        let mut low = usable_min
            .saturating_add(u32::from(gap))
            .min(u32::from(u16::MAX)) as u16;
        let fits = |extent: u16| {
            let area = match self.axis {
                SplitAxis::Vertical => Rect::new(0, 0, 1, extent),
                SplitAxis::Horizontal => Rect::new(0, 0, extent, 1),
            };
            let (first_area, second_area) = self.layout_with_basis_points(area, gap, basis_points);
            let (first_extent, second_extent) = match self.axis {
                SplitAxis::Vertical => (first_area.height, second_area.height),
                SplitAxis::Horizontal => (first_area.width, second_area.width),
            };
            first_extent >= first && second_extent >= second
        };

        if fits(low) {
            return low;
        }
        if !fits(u16::MAX) {
            return u16::MAX;
        }
        let mut high = u16::MAX;
        while low < high {
            let middle = low + (high - low) / 2;
            if fits(middle) {
                high = middle;
            } else {
                low = middle.saturating_add(1);
            }
        }
        low
    }

    /// Return the smallest extent whose fixed second-share layout can hold
    /// both requested pane extents. Uses `layout_with_second_share` as the
    /// canonical resolver so measurement and drawing share rounding rules.
    pub fn required_extent_with_second_share(
        self,
        gap: u16,
        percent: u8,
        min_second: u16,
        max_second: u16,
        first: u16,
        second: u16,
    ) -> u16 {
        if self.maximized == Maximized::First {
            return first;
        }
        if self.maximized == Maximized::Second {
            return second;
        }

        let mut low = u32::from(first)
            .saturating_add(u32::from(second))
            .saturating_add(u32::from(gap))
            .min(u32::from(u16::MAX)) as u16;
        let fits = |extent: u16| {
            let area = match self.axis {
                SplitAxis::Vertical => Rect::new(0, 0, 1, extent),
                SplitAxis::Horizontal => Rect::new(0, 0, extent, 1),
            };
            let (first_area, second_area) =
                self.layout_with_second_share(area, gap, percent, min_second, max_second);
            let (first_extent, second_extent) = match self.axis {
                SplitAxis::Vertical => (first_area.height, second_area.height),
                SplitAxis::Horizontal => (first_area.width, second_area.width),
            };
            first_extent >= first && second_extent >= second
        };

        if fits(low) {
            return low;
        }
        if !fits(u16::MAX) {
            return u16::MAX;
        }
        let mut high = u16::MAX;
        while low < high {
            let middle = low + (high - low) / 2;
            if fits(middle) {
                high = middle;
            } else {
                low = middle.saturating_add(1);
            }
        }
        low
    }

    fn layout_with_ratio(
        &self,
        area: Rect,
        gap: u16,
        numerator: u32,
        denominator: u32,
    ) -> (Rect, Rect) {
        let split = |at: u16| match self.axis {
            SplitAxis::Vertical => {
                let (a, rest) = split_v(area, at);
                let (_, b) = split_v(rest, gap.min(rest.height));
                (a, b)
            }
            SplitAxis::Horizontal => {
                let (a, rest) = split_h(area, at);
                let (_, b) = split_h(rest, gap.min(rest.width));
                (a, b)
            }
        };
        match self.maximized {
            Maximized::First => (area, self.collapsed(area, true)),
            Maximized::Second => (self.collapsed(area, false), area),
            Maximized::None => {
                let usable = self.length(area).saturating_sub(gap);
                if usable < self.min_first.saturating_add(self.min_second) {
                    return (area, self.collapsed(area, true));
                }
                let first = (u32::from(usable).saturating_mul(numerator) / denominator) as u16;
                let first = first.clamp(self.min_first, usable.saturating_sub(self.min_second));
                split(first)
            }
        }
    }

    pub fn handle(&self, area: Rect, gap: u16) -> Rect {
        let (a, b) = self.layout(area, gap);
        let thickness = if a.is_empty() || b.is_empty() { 0 } else { gap };
        match self.axis {
            SplitAxis::Vertical => Rect {
                x: area.x,
                y: a.bottom(),
                width: area.width,
                height: thickness,
            },
            SplitAxis::Horizontal => Rect {
                x: a.right(),
                y: area.y,
                width: thickness,
                height: area.height,
            },
        }
    }

    pub fn drag_to(&mut self, area: Rect, gap: u16, pos: Position) -> bool {
        let usable = self.length(area).saturating_sub(gap);
        let offset = match self.axis {
            SplitAxis::Vertical => pos.y.saturating_sub(area.y),
            SplitAxis::Horizontal => pos.x.saturating_sub(area.x),
        };
        if usable == 0 || usable < self.min_first.saturating_add(self.min_second) {
            return false;
        }
        let first = offset.clamp(self.min_first, usable.saturating_sub(self.min_second));
        let percent = (u32::from(first)
            .saturating_mul(100)
            .saturating_add(u32::from(usable) / 2))
        .checked_div(u32::from(usable))
        .unwrap_or(0) as u8;
        let percent = clamp_percent(percent);
        let changed = percent != self.percent;
        self.percent = percent;
        changed
    }

    /// Update the split using an exact cell position and retain its ratio in
    /// basis points for later resizes.
    pub fn drag_to_with_basis_points(
        &mut self,
        area: Rect,
        gap: u16,
        pos: Position,
        basis_points: &mut u16,
    ) -> bool {
        let usable = self.length(area).saturating_sub(gap);
        let offset = match self.axis {
            SplitAxis::Vertical => pos.y.saturating_sub(area.y),
            SplitAxis::Horizontal => pos.x.saturating_sub(area.x),
        };
        if usable == 0 || usable < self.min_first.saturating_add(self.min_second) {
            return false;
        }
        let first = offset.clamp(self.min_first, usable.saturating_sub(self.min_second));
        // Choose the smallest ratio whose floored layout reaches the pointer
        // cell. Rounding to nearest can leave the seam one cell short.
        let precise = (u32::from(first)
            .saturating_mul(10_000)
            .saturating_add(u32::from(usable).saturating_sub(1)))
        .checked_div(u32::from(usable))
        .unwrap_or(0)
        .clamp(500, 9_500) as u16;
        let percent = clamp_percent(((u32::from(precise) + 50) / 100) as u8);
        let changed = percent != self.percent || precise != *basis_points;
        self.percent = percent;
        *basis_points = precise;
        changed
    }

    /// Move the first pane by one cell while retaining its precise ratio.
    pub fn nudge_with_basis_points(
        &mut self,
        area: Rect,
        gap: u16,
        delta: i16,
        basis_points: &mut u16,
    ) -> bool {
        let (first, _) = self.layout_with_basis_points(area, gap, *basis_points);
        let cur = i32::from(self.length(first));
        let target = cur.saturating_add(i32::from(delta)).max(0) as u16;
        let pos = match self.axis {
            SplitAxis::Vertical => Position::new(area.x, area.y.saturating_add(target)),
            SplitAxis::Horizontal => Position::new(area.x.saturating_add(target), area.y),
        };
        self.drag_to_with_basis_points(area, gap, pos, basis_points)
    }

    pub fn nudge(&mut self, area: Rect, gap: u16, delta: i16) {
        let (first, _) = self.layout(area, gap);
        let cur = i32::from(self.length(first));
        let target = cur.saturating_add(i32::from(delta)).max(0) as u16;
        let pos = match self.axis {
            SplitAxis::Vertical => Position::new(area.x, area.y.saturating_add(target)),
            SplitAxis::Horizontal => Position::new(area.x.saturating_add(target), area.y),
        };
        self.drag_to(area, gap, pos);
    }
}

const fn clamp_percent(p: u8) -> u8 {
    if p < 5 {
        5
    } else if p > 95 {
        95
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_distributes_flex_after_fixed() {
        let r = rows(
            Rect::new(0, 0, 10, 20),
            &[Track::Fixed(3), Track::Flex(1), Track::Flex(2)],
        );
        assert_eq!(
            r,
            vec![
                Rect::new(0, 0, 10, 3),
                Rect::new(0, 3, 10, 6),
                Rect::new(0, 9, 10, 11)
            ]
        );

        let a = rows(
            Rect::new(0, 0, 10, 10),
            &[Track::Auto, Track::Fixed(1), Track::Flex(1)],
        );
        assert_eq!(
            a.iter().map(|r| r.height).collect::<Vec<_>>(),
            vec![1, 1, 8]
        );
        let a = rows(Rect::new(0, 0, 10, 10), &[Track::Auto, Track::Auto]);
        assert_eq!(a.iter().map(|r| r.height).collect::<Vec<_>>(), vec![5, 5]);
        let m = rows_measured(
            Rect::new(0, 0, 10, 10),
            &[Track::Auto, Track::Flex(1)],
            &[4],
        );
        assert_eq!(m.iter().map(|r| r.height).collect::<Vec<_>>(), vec![4, 6]);

        let c = rows(Rect::new(0, 0, 10, 2), &[Track::Fixed(5), Track::Flex(1)]);
        assert_eq!(c.iter().map(|r| r.height).collect::<Vec<_>>(), vec![2, 0]);
    }

    #[test]
    fn columns_respects_gap_and_rounds_deterministically() {
        let c = columns(
            Rect::new(0, 0, 20, 1),
            &[Track::Flex(1), Track::Flex(1), Track::Flex(1)],
            2,
        );

        assert_eq!(
            c.iter().map(|r| (r.x, r.width)).collect::<Vec<_>>(),
            vec![(0, 6), (8, 5), (15, 5)]
        );
        assert_eq!(
            columns(
                Rect::new(0, 0, 20, 1),
                &[Track::Flex(1), Track::Flex(1), Track::Flex(1)],
                2
            ),
            c
        );
    }

    #[test]
    fn responsive_columns_stack_below_the_threshold() {
        let wide = responsive_columns(
            Rect::new(0, 0, 80, 4),
            &[Track::Flex(1), Track::Flex(1)],
            2,
            60,
        );
        assert_eq!(wide.len(), 2);
        assert_eq!(wide.first().map(|r| r.height), Some(4));
        let narrow = responsive_columns(
            Rect::new(0, 0, 40, 4),
            &[Track::Flex(1), Track::Flex(1)],
            2,
            60,
        );
        assert_eq!(
            narrow
                .iter()
                .map(|r| (r.y, r.height, r.width))
                .collect::<Vec<_>>(),
            vec![(0, 2, 40), (2, 2, 40)]
        );
    }

    #[test]
    fn action_row_right_aligns_and_left_aligns() {
        let area = Rect::new(0, 0, 40, 1);
        let end = action_row(area, &[10, 12], 2, RowAlign::End);
        assert_eq!(end, vec![Rect::new(16, 0, 10, 1), Rect::new(28, 0, 12, 1)]);
        let start = action_row(area, &[10, 12], 2, RowAlign::Start);
        assert_eq!(start, vec![Rect::new(0, 0, 10, 1), Rect::new(12, 0, 12, 1)]);

        let tight = action_row(Rect::new(0, 0, 15, 1), &[10, 12], 2, RowAlign::End);
        assert!(tight.iter().all(|r| r.right() <= 15));
    }

    #[test]
    fn auto_takes_one_cell_beside_flex_and_an_equal_share_without_it() {
        let d = distribute(20, &[Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![1, 19]);
        let d = distribute(20, &[Track::Auto, Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![1, 1, 18]);

        let d = distribute(20, &[Track::Fixed(6), Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![6, 1, 13]);

        let d = distribute(20, &[Track::Auto, Track::Auto], 0, None);
        assert_eq!(d, vec![10, 10]);
        let d = distribute(20, &[Track::Fixed(5), Track::Auto, Track::Auto], 0, None);
        assert_eq!(d, vec![5, 8, 7]);
    }

    #[test]
    fn rows_measured_uses_the_natural_size() {
        let area = Rect::new(0, 0, 30, 12);
        let tracks = [Track::Auto, Track::Fixed(1), Track::Flex(1)];

        let plain = rows(area, &tracks);
        assert_eq!(plain[0].height, 1);

        let rows_m = rows_measured(area, &tracks, &[4, 0, 0]);
        assert_eq!(rows_m[0].height, 4);
        assert_eq!(rows_m[1].height, 1);
        assert_eq!(rows_m[2].height, 7);
        assert_eq!(rows_m.iter().map(|r| r.height).sum::<u16>(), 12);

        let big = rows_measured(area, &tracks, &[40, 0, 0]);
        assert_eq!(big[0].height, 12);
        assert_eq!(big[2].height, 0);

        let cols = columns_measured(area, &tracks, 1, &[4, 0, 0]);
        assert_eq!(cols[0].width, 4);
    }

    #[test]
    fn inset_saturates_on_tiny_rects() {
        assert_eq!(
            inset(Rect::new(0, 0, 2, 2), Insets::all(3)),
            Rect::new(2, 2, 0, 0)
        );
        let area = Rect::new(3, 2, 3, 4);
        let tight = inset(area, Insets::all(2));
        assert!(tight.is_empty(), "{tight:?} should be empty");
        assert_eq!(
            (tight.x, tight.y),
            (5, 4),
            "a saturating symmetric inset lost {area:?}'s origin: {tight:?}"
        );
        assert_eq!(
            inset(
                Rect::new(5, 5, 10, 4),
                Insets {
                    l: 2,
                    t: 1,
                    r: 1,
                    b: 0
                }
            ),
            Rect::new(7, 6, 7, 3)
        );
        assert_eq!(
            inset(
                Rect::new(0, 0, 1, 1),
                Insets {
                    l: 5,
                    t: 0,
                    r: 0,
                    b: 9
                }
            ),
            Rect::new(1, 0, 0, 0)
        );
        assert_eq!(
            split_v(Rect::new(0, 0, 3, 2), 9),
            (Rect::new(0, 0, 3, 2), Rect::new(0, 2, 3, 0))
        );
        assert_eq!(
            split_h(Rect::new(0, 0, 3, 2), 1),
            (Rect::new(0, 0, 1, 2), Rect::new(1, 0, 2, 2))
        );

        for w in 0u16..=8 {
            for h in 0u16..=8 {
                for n in 0u16..=4 {
                    let area = Rect {
                        x: 3,
                        y: 2,
                        width: w,
                        height: h,
                    };
                    for i in [
                        Insets::all(n),
                        Insets::symmetric(n, 1),
                        Insets {
                            l: n,
                            t: 1,
                            r: 0,
                            b: 2,
                        },
                        Insets {
                            l: 0,
                            t: n,
                            r: 2,
                            b: n,
                        },
                        Insets {
                            l: n,
                            t: 2,
                            r: n,
                            b: 1,
                        },
                        Insets {
                            l: n,
                            t: 0,
                            r: n,
                            b: 0,
                        },
                    ] {
                        let r = inset(area, i);
                        assert!(
                            r.x >= area.x
                                && r.y >= area.y
                                && r.right() <= area.right()
                                && r.bottom() <= area.bottom(),
                            "inset({area:?}, {i:?}) = {r:?} escapes its container"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn split_first_pane_wins_on_both_axes_when_minima_do_not_fit() {
        let v = SplitModel::new(SplitAxis::Vertical, 60, 5, 5);
        let (a, b) = v.layout(Rect::new(0, 0, 80, 8), 1);
        assert_eq!(a.height, 8);
        assert!(b.is_empty());
        let h = SplitModel::new(SplitAxis::Horizontal, 60, 50, 50);
        let (a, b) = h.layout(Rect::new(0, 0, 80, 8), 1);
        assert_eq!(a.width, 80);
        assert!(b.is_empty());

        let (a, b) = v.layout(Rect::new(0, 0, 80, 30), 1);
        assert_eq!((a.height, b.height), (17, 12));
        assert_eq!(
            v.handle(Rect::new(0, 0, 80, 30), 1),
            Rect::new(0, 17, 80, 1)
        );
        let mut m = v;
        m.toggle_max(Maximized::Second);
        let (a, b) = m.layout(Rect::new(0, 0, 80, 30), 1);
        assert!(a.is_empty() && b.height == 30);
    }

    #[test]
    fn fixed_second_share_uses_full_extent_and_shared_minimum_solver() {
        let model = SplitModel::new(SplitAxis::Horizontal, 50, 1, 1);
        let expected = [(107, 36), (108, 36), (109, 37), (120, 40), (160, 48)];
        for (width, detail_width) in expected {
            let (master, detail) =
                model.layout_with_second_share(Rect::new(0, 0, width, 8), 2, 34, 34, 48);
            assert_eq!(detail.width, detail_width, "width={width}");
            assert_eq!(master.width + 2 + detail.width, width);
            assert_eq!(detail.x, master.right() + 2);
        }

        let (master, detail) = model.layout_with_second_share(Rect::new(0, 0, 0, 0), 2, 34, 34, 48);
        assert!(master.is_empty() && detail.is_empty());

        let required = model.required_extent_with_second_share(2, 34, 34, 48, 70, 36);
        assert_eq!(required, 108);
        let (master, detail) =
            model.layout_with_second_share(Rect::new(0, 0, required, 1), 2, 34, 34, 48);
        assert!(master.width >= 70 && detail.width >= 36);
    }

    #[test]
    fn precise_split_drag_retains_cell_position_across_resize() {
        let mut model = SplitModel::new(SplitAxis::Vertical, 38, 4, 6);
        let area = Rect::new(2, 3, 18, 35);
        let mut basis_points = 3_800;
        let changed = model.drag_to_with_basis_points(
            area,
            1,
            Position::new(area.x, area.y + 13),
            &mut basis_points,
        );
        assert!(changed);
        assert_eq!(model.percent, 38);
        assert_eq!(basis_points, 3_824);
        assert_eq!(
            model
                .layout_with_basis_points(area, 1, basis_points)
                .0
                .height,
            13
        );

        let resized = Rect::new(2, 3, 18, 45);
        assert_eq!(
            model
                .layout_with_basis_points(resized, 1, basis_points)
                .0
                .height,
            16
        );
    }

    #[test]
    fn precise_split_drag_reaches_each_feasible_cell_on_both_axes() {
        for axis in [SplitAxis::Vertical, SplitAxis::Horizontal] {
            let area = match axis {
                SplitAxis::Vertical => Rect::new(7, 11, 20, 35),
                SplitAxis::Horizontal => Rect::new(7, 11, 35, 20),
            };
            let mut model = SplitModel::new(axis, 38, 4, 6);
            for target in 4..=28 {
                let pos = match axis {
                    SplitAxis::Vertical => Position::new(area.x, area.y + target),
                    SplitAxis::Horizontal => Position::new(area.x + target, area.y),
                };
                let mut basis_points = 3_800;
                model.drag_to_with_basis_points(area, 1, pos, &mut basis_points);
                let (first, _) = model.layout_with_basis_points(area, 1, basis_points);
                let actual = match axis {
                    SplitAxis::Vertical => first.height,
                    SplitAxis::Horizontal => first.width,
                };
                assert_eq!(
                    actual, target,
                    "axis={axis:?}, target={target}, ratio={basis_points}"
                );
            }
        }
    }

    #[test]
    fn required_split_extent_uses_the_same_ratio_and_rounding_as_layout() {
        let model = SplitModel::new(SplitAxis::Vertical, 38, 4, 6);
        for first in 1..=20 {
            for second in 1..=20 {
                let extent = model.required_extent_with_basis_points(1, 3_800, first, second);
                let area = Rect::new(0, 0, 1, extent);
                let (first_area, second_area) = model.layout_with_basis_points(area, 1, 3_800);
                assert!(first_area.height >= first, "first={first}, second={second}");
                assert!(
                    second_area.height >= second,
                    "first={first}, second={second}"
                );
                if extent > 0 {
                    let smaller = Rect::new(0, 0, 1, extent - 1);
                    let (first_area, second_area) =
                        model.layout_with_basis_points(smaller, 1, 3_800);
                    assert!(
                        first_area.height < first || second_area.height < second,
                        "extent {extent} was not minimal for {first}/{second}"
                    );
                }
            }
        }
        assert_eq!(model.required_extent_with_basis_points(1, 3_800, 10, 6), 28);

        let maximized = SplitModel {
            maximized: Maximized::First,
            ..model
        };
        assert_eq!(
            maximized.required_extent_with_basis_points(1, 3_800, 10, 6),
            10
        );
    }

    #[test]
    fn legacy_layout_preserves_raw_public_percent_extremes() {
        let area = Rect::new(0, 0, 1_001, 10);
        let mut model = SplitModel::new(SplitAxis::Horizontal, 50, 1, 1);
        model.percent = 0;
        assert_eq!(model.layout(area, 1).0.width, 1);
        model.percent = 100;
        assert_eq!(model.layout(area, 1).0.width, 999);
    }

    #[test]
    fn precise_split_solver_handles_empty_and_minimum_constrained_regions() {
        let mut model = SplitModel::new(SplitAxis::Horizontal, 38, 4, 6);
        let mut basis_points = 3_800;
        assert!(!model.drag_to_with_basis_points(
            Rect::ZERO,
            1,
            Position::new(0, 0),
            &mut basis_points,
        ));
        let area = Rect::new(0, 0, 10, 2);
        let _ = model.drag_to_with_basis_points(area, 1, Position::new(0, 0), &mut basis_points);
        let (first, second) = model.layout_with_basis_points(area, 1, basis_points);
        assert_eq!((first.width, second.width), (10, 0));
    }

    #[test]
    fn precise_split_solver_supports_both_axes_large_extents_and_nudges() {
        for axis in [SplitAxis::Vertical, SplitAxis::Horizontal] {
            let area = match axis {
                SplitAxis::Vertical => Rect::new(3, 5, 20, 35),
                SplitAxis::Horizontal => Rect::new(3, 5, 35, 20),
            };
            let mut model = SplitModel::new(axis, 38, 4, 6);
            let mut basis_points = 3_800;
            let pos = match axis {
                SplitAxis::Vertical => Position::new(area.x, area.y + 13),
                SplitAxis::Horizontal => Position::new(area.x + 13, area.y),
            };
            assert!(model.drag_to_with_basis_points(area, 1, pos, &mut basis_points));
            let first = model.layout_with_basis_points(area, 1, basis_points).0;
            let extent = match axis {
                SplitAxis::Vertical => first.height,
                SplitAxis::Horizontal => first.width,
            };
            assert_eq!(extent, 13, "{axis:?}");
            assert!(model.nudge_with_basis_points(area, 1, 1, &mut basis_points));
            let nudged = model.layout_with_basis_points(area, 1, basis_points).0;
            let nudged_extent = match axis {
                SplitAxis::Vertical => nudged.height,
                SplitAxis::Horizontal => nudged.width,
            };
            assert_eq!(nudged_extent, 14, "{axis:?}");

            let large = match axis {
                SplitAxis::Vertical => Rect::new(0, 0, 20, 134),
                SplitAxis::Horizontal => Rect::new(0, 0, 134, 20),
            };
            let mut large_model = SplitModel::new(axis, 38, 4, 6);
            let mut large_ratio = 3_800;
            let target = match axis {
                SplitAxis::Vertical => Position::new(0, 51),
                SplitAxis::Horizontal => Position::new(51, 0),
            };
            assert!(large_model.drag_to_with_basis_points(large, 1, target, &mut large_ratio,));
            let large_first = large_model
                .layout_with_basis_points(large, 1, large_ratio)
                .0;
            let large_extent = match axis {
                SplitAxis::Vertical => large_first.height,
                SplitAxis::Horizontal => large_first.width,
            };
            assert_eq!(large_extent, 51, "{axis:?}");
        }
    }

    #[test]
    fn split_panes_and_seam_stay_inside_the_area() {
        let area = Rect::new(3, 2, 10, 6);
        let v = SplitModel::new(SplitAxis::Vertical, 60, 2, 2);
        let h = SplitModel::new(SplitAxis::Horizontal, 60, 2, 2);

        let mut vmax = v;
        vmax.toggle_max(Maximized::First);
        assert_eq!(vmax.layout(area, 1).1, Rect::new(3, 8, 10, 0));
        vmax.toggle_max(Maximized::First);
        vmax.toggle_max(Maximized::Second);
        assert_eq!(vmax.layout(area, 1).0, Rect::new(3, 2, 10, 0));
        let mut hmax = h;
        hmax.toggle_max(Maximized::First);
        assert_eq!(hmax.layout(area, 1).1, Rect::new(13, 2, 0, 6));
        hmax.toggle_max(Maximized::First);
        hmax.toggle_max(Maximized::Second);
        assert_eq!(hmax.layout(area, 1).0, Rect::new(3, 2, 0, 6));

        let tight = SplitModel::new(SplitAxis::Vertical, 60, 50, 50);
        assert_eq!(tight.layout(area, 1).1, Rect::new(3, 8, 10, 0));
        assert!(tight.layout(area, 1).1.is_empty());

        let inside = |r: Rect, area: Rect, what: &str, model: &SplitModel| {
            assert!(
                r.x >= area.x
                    && r.y >= area.y
                    && r.right() <= area.right()
                    && r.bottom() <= area.bottom(),
                "{what} {r:?} escapes {area:?} for {model:?}"
            );
        };
        for axis in [SplitAxis::Vertical, SplitAxis::Horizontal] {
            for w in [0u16, 1, 3, 12, 80] {
                for hh in [0u16, 1, 3, 12, 40] {
                    let area = Rect {
                        x: 3,
                        y: 2,
                        width: w,
                        height: hh,
                    };
                    for percent in [5u8, 50, 95] {
                        for (min_first, min_second) in [(0u16, 0u16), (1, 1), (5, 5), (50, 50)] {
                            for gap in [0u16, 1, 4] {
                                for max in [Maximized::None, Maximized::First, Maximized::Second] {
                                    let mut m =
                                        SplitModel::new(axis, percent, min_first, min_second);
                                    m.maximized = max;
                                    let (a, b) = m.layout(area, gap);
                                    inside(a, area, "first pane", &m);
                                    inside(b, area, "second pane", &m);
                                    inside(m.handle(area, gap), area, "seam", &m);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn split_percent_is_clamped_to_5_95() {
        let mut s = SplitModel::new(SplitAxis::Horizontal, 200, 1, 1);
        assert_eq!(s.percent, 95);
        s.grow(-120);
        assert_eq!(s.percent, 5);
        let area = Rect::new(0, 0, 101, 20);
        assert!(s.drag_to(area, 1, Position::new(70, 3)));
        assert_eq!(s.layout(area, 1).0.width, 70);
        s.min_first = 10;
        s.drag_to(area, 1, Position::new(2, 3));
        assert_eq!(s.layout(area, 1).0.width, 10);
        s.nudge(area, 1, 5);
        assert_eq!(s.layout(area, 1).0.width, 15);
    }
}
