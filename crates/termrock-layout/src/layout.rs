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

    pub fn layout_with_basis_points(
        &self,
        area: Rect,
        gap: u16,
        basis_points: u16,
    ) -> (Rect, Rect) {
        self.layout_with_ratio(area, gap, u32::from(basis_points.clamp(500, 9_500)), 10_000)
    }

    pub fn layout_with_first_extent(
        &self,
        area: Rect,
        gap: u16,
        first_extent: u16,
    ) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => return (area, self.collapsed(area, true)),
            Maximized::Second => return (self.collapsed(area, false), area),
            Maximized::None => {}
        }

        let usable = self.length(area).saturating_sub(gap);
        if usable < self.min_first.saturating_add(self.min_second) {
            return (area, self.collapsed(area, true));
        }
        let first = first_extent.clamp(self.min_first, usable.saturating_sub(self.min_second));
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

    pub fn required_extent_with_first_extent(
        self,
        gap: u16,
        first_extent: u16,
        first: u16,
        second: u16,
    ) -> u16 {
        if self.maximized == Maximized::First {
            return first;
        }
        if self.maximized == Maximized::Second {
            return second;
        }

        let first_extent = first_extent.max(self.min_first);
        let first = first.max(self.min_first);
        if first > first_extent {
            return u16::MAX;
        }
        first_extent
            .saturating_add(gap)
            .saturating_add(second.max(self.min_second))
    }

    pub fn layout_with_first_share(
        &self,
        area: Rect,
        gap: u16,
        percent: u8,
        min_first: u16,
        max_first: u16,
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
        let requested = (u32::from(extent)
            .saturating_mul(u32::from(percent.min(100)))
            .saturating_add(50)
            / 100)
            .min(u32::from(u16::MAX)) as u16;
        let first = requested
            .clamp(min_first.min(max_first), max_first.max(min_first))
            .min(usable.saturating_sub(self.min_second))
            .max(self.min_first);
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

    pub fn required_extent_with_first_share(
        self,
        gap: u16,
        percent: u8,
        min_first: u16,
        max_first: u16,
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
                self.layout_with_first_share(area, gap, percent, min_first, max_first);
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
