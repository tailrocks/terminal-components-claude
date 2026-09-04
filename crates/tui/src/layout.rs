//! Layout primitives (`COMPONENT_ARCHITECTURE.md` §10).
//!
//! A hand-written, deterministic, allocation-light integer distribution —
//! never a constraint solver. The vocabulary reads against ratatui's
//! (`RowAlign::{Start, End}` ↔ `Flex`, `spacing` ↔ `Spacing`) while `Rect`
//! geometry is reused, never re-derived (§22 R‑12, R‑13).

use ratatui_core::layout::{Margin, Position, Rect};

/// One row height or column width.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    /// Exactly `n` cells (clipped to what is available).
    Fixed(u16),
    /// A weighted share of the space left after `Fixed` tracks.
    Flex(u16),
    /// Content-sized. Without a measurement the primitive gives it one cell
    /// when explicit `Flex` tracks exist, else an equal share of the
    /// remainder; use [`rows_measured`] / [`columns_measured`] to supply the
    /// natural size.
    Auto,
}

/// Where an action row packs its buttons; reads against `Flex::{Start, End}`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowAlign {
    /// Pack from the left.
    Start,
    /// Pack from the right.
    End,
}

/// Asymmetric insets; the symmetric case is `Rect::inner(Margin)` wherever
/// that answers a non-empty rect (see [`inset`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Insets {
    /// Left.
    pub l: u16,
    /// Top.
    pub t: u16,
    /// Right.
    pub r: u16,
    /// Bottom.
    pub b: u16,
}

impl Insets {
    /// The same inset on every side.
    pub const fn all(n: u16) -> Self {
        Insets {
            l: n,
            t: n,
            r: n,
            b: n,
        }
    }

    /// Horizontal `h`, vertical `v`.
    pub const fn symmetric(h: u16, v: u16) -> Self {
        Insets {
            l: h,
            t: v,
            r: h,
            b: v,
        }
    }
}

/// Distribute `total` cells over `tracks` with `spacing` between them.
/// Deterministic: leftover cells go to the earliest flexible tracks.
pub fn distribute(total: u16, tracks: &[Track], spacing: u16, natural: Option<&[u16]>) -> Vec<u16> {
    let n = tracks.len() as u16;
    let gaps = spacing.saturating_mul(n.saturating_sub(1));
    let mut avail = total.saturating_sub(gaps);
    let mut out = vec![0u16; tracks.len()];
    let has_flex = tracks.iter().any(|t| matches!(t, Track::Flex(_)));
    // pass 1: fixed and measured auto
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
    // pass 2: flex (or unmeasured auto when there is no flex)
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

/// [`distribute`] into a caller-supplied slice (no allocation); tracks
/// beyond `out.len()` are ignored.
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

/// Stack rows top to bottom.
pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect> {
    stack(area, &distribute(area.height, heights, 0, None), 0, true)
}

/// Stack rows with measured `Auto` heights.
pub fn rows_measured(area: Rect, heights: &[Track], natural: &[u16]) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.height, heights, 0, Some(natural)),
        0,
        true,
    )
}

/// Lay columns left to right with `spacing` between them.
pub fn columns(area: Rect, widths: &[Track], spacing: u16) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.width, widths, spacing, None),
        spacing,
        false,
    )
}

/// Lay columns with measured `Auto` widths.
pub fn columns_measured(area: Rect, widths: &[Track], spacing: u16, natural: &[u16]) -> Vec<Rect> {
    stack(
        area,
        &distribute(area.width, widths, spacing, Some(natural)),
        spacing,
        false,
    )
}

/// Columns that stack into rows when `area.width < stack_below`.
pub fn responsive_columns(area: Rect, spec: &[Track], spacing: u16, stack_below: u16) -> Vec<Rect> {
    if area.width < stack_below {
        let tracks: Vec<Track> = spec.iter().map(|_| Track::Flex(1)).collect();
        rows(area, &tracks)
    } else {
        columns(area, spec, spacing)
    }
}

/// An action row: fixed-width buttons packed from the start or the end —
/// the analogue of `Layout::horizontal(…).flex(Flex::End).spacing(Spacing::Space(spacing))`.
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

/// Shrink by asymmetric insets, saturating.
///
/// The result is always a subrect of `area`, **including when the insets do
/// not fit**: a saturating inset answers an empty rect anchored where the
/// content would have started (`area.x + l`, `area.y + t`, each clamped into
/// `area`), never `Rect::ZERO` at the screen corner. A caller that reads
/// `.x`/`.y` before checking `.is_empty()` therefore stays inside its own
/// container.
///
/// Symmetric insets go through `Rect::inner(Margin)` (§22 R‑12) wherever it
/// answers a non-empty rect, which is every case in which the two agree.
pub fn inset(area: Rect, i: Insets) -> Rect {
    if i.l == i.r && i.t == i.b {
        let r = area.inner(Margin::new(i.l, i.t));
        // `Rect::inner` answers `Rect::ZERO` when the margin does not fit, and
        // `Rect::ZERO`'s origin is the top-left of the *screen*, not `area`'s —
        // so its empty answer sits outside the container it was asked to inset.
        // `.is_empty()` cannot see that, because an empty rect at the wrong
        // origin is still empty. Fall through to the clamped arithmetic below,
        // which anchors the empty answer at the container. Where `Rect::inner`
        // returns a non-empty rect the two are identical by construction, so
        // the branch is an optimisation and not a second behaviour.
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

/// Split into a top pane of height `at` and the rest.
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

/// Split into a left pane of width `at` and the rest.
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

/// Which pane a split maximises.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Maximized {
    /// Neither.
    #[default]
    None,
    /// The first pane fills the area.
    First,
    /// The second pane fills the area.
    Second,
}

/// The split axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitAxis {
    /// First pane on top.
    Vertical,
    /// First pane on the left.
    Horizontal,
}

/// A two-pane split model: percent of the first pane, minima, maximise state
/// and axis. When both minima cannot fit, **the first pane wins on both axes**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SplitModel {
    /// Percent of the usable length given to the first pane, `5..=95`.
    pub percent: u8,
    /// Minimum length of the first pane.
    pub min_first: u16,
    /// Minimum length of the second pane.
    pub min_second: u16,
    /// Maximise state.
    pub maximized: Maximized,
    /// Axis.
    pub axis: SplitAxis,
}

impl SplitModel {
    /// A split with `percent` for the first pane.
    pub const fn new(axis: SplitAxis, percent: u8, min_first: u16, min_second: u16) -> Self {
        SplitModel {
            percent: clamp_percent(percent),
            min_first,
            min_second,
            maximized: Maximized::None,
            axis,
        }
    }

    /// Toggle maximising `which`.
    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    /// Grow the first pane by `delta` percent, clamped to `5..=95`.
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

    /// The empty pane of a degenerate split, anchored inside `area`: at the
    /// container's origin for the first pane, at its far edge along the split
    /// axis for the second.
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

    /// The two pane rects with `gap` cells between them.
    ///
    /// Both rects are always subrects of `area`, **including the empty one** a
    /// maximised or collapsed split produces: the empty pane is anchored where
    /// that pane would have started — `area`'s origin for the first pane,
    /// `area.bottom()` / `area.right()` for the second — never `Rect::ZERO`,
    /// whose origin is the top-left of the *screen* and not the container's.
    /// `.is_empty()` cannot see that difference, because an empty rect at the
    /// wrong origin is still empty, so a caller that reads `.x`/`.y` before
    /// checking `.is_empty()` would otherwise paint outside its container.
    /// This is the guarantee [`inset`] gives, on the same terms.
    pub fn layout(&self, area: Rect, gap: u16) -> (Rect, Rect) {
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
                    // not enough room for both: the first pane wins on both axes
                    return (area, self.collapsed(area, true));
                }
                let first =
                    (u32::from(usable).saturating_mul(u32::from(self.percent)) / 100) as u16;
                let first = first.clamp(self.min_first, usable.saturating_sub(self.min_second));
                split(first)
            }
        }
    }

    /// The seam strip between the panes; empty when maximised, when collapsed
    /// and when `gap == 0`.
    ///
    /// Like [`SplitModel::layout`], the empty answer stays inside `area`: the
    /// strip keeps the seam position (the far edge of the first pane, which is
    /// `area`'s own origin when the first pane is the collapsed one) and loses
    /// only its thickness.
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

    /// Put the seam under `pos`, clamped by the minima. Returns whether the
    /// percent changed.
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

    /// Resize the first pane by whole cells.
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
        // auto yields one row beside flex, and shares equally without flex
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
        // fixed beyond the area is clipped
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
        // 20 - 4 gaps = 16 → 5,5,5 + 1 leftover to the first
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
        // wider than the area: clipped, never out of bounds
        let tight = action_row(Rect::new(0, 0, 15, 1), &[10, 12], 2, RowAlign::End);
        assert!(tight.iter().all(|r| r.right() <= 15));
    }

    /// Adjudication 2.7: `Track::Auto` is content-sized, and without a
    /// measurement the primitive gives it **one cell** when explicit `Flex`
    /// tracks exist (so `Auto` never starves a `Flex`) and an **equal share
    /// of the remainder** when there are none.
    #[test]
    fn auto_takes_one_cell_beside_flex_and_an_equal_share_without_it() {
        // beside a Flex: exactly one cell
        let d = distribute(20, &[Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![1, 19]);
        let d = distribute(20, &[Track::Auto, Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![1, 1, 18]);
        // with a Fixed track and a Flex, the Fixed is honoured first
        let d = distribute(20, &[Track::Fixed(6), Track::Auto, Track::Flex(1)], 0, None);
        assert_eq!(d, vec![6, 1, 13]);
        // no Flex at all: an equal share of the remainder, leftovers to the
        // earliest tracks
        let d = distribute(20, &[Track::Auto, Track::Auto], 0, None);
        assert_eq!(d, vec![10, 10]);
        let d = distribute(20, &[Track::Fixed(5), Track::Auto, Track::Auto], 0, None);
        assert_eq!(d, vec![5, 8, 7]);
    }

    /// The `_measured` variants are the home for `Measure`-derived sizes: a
    /// supplied natural size replaces the one-cell placeholder.
    #[test]
    fn rows_measured_uses_the_natural_size() {
        let area = Rect::new(0, 0, 30, 12);
        let tracks = [Track::Auto, Track::Fixed(1), Track::Flex(1)];
        // unmeasured: `Auto` gets one cell beside the `Flex`
        let plain = rows(area, &tracks);
        assert_eq!(plain[0].height, 1);
        // measured: `Auto` gets exactly the natural size it reported, and the
        // `Flex` track absorbs the difference
        let rows_m = rows_measured(area, &tracks, &[4, 0, 0]);
        assert_eq!(rows_m[0].height, 4);
        assert_eq!(rows_m[1].height, 1);
        assert_eq!(rows_m[2].height, 7);
        assert_eq!(rows_m.iter().map(|r| r.height).sum::<u16>(), 12);
        // a natural size larger than the area degrades to the space left
        let big = rows_measured(area, &tracks, &[40, 0, 0]);
        assert_eq!(big[0].height, 12);
        assert_eq!(big[2].height, 0);
        // columns_measured behaves the same on the other axis
        let cols = columns_measured(area, &tracks, 1, &[4, 0, 0]);
        assert_eq!(cols[0].width, 4);
    }

    /// The symmetric branch of [`inset`] is `Rect::inner(Margin)` (§22 R‑12),
    /// which answers `Rect::ZERO` when the margin does not fit — and
    /// `Rect::ZERO`'s origin is the top-left of the **screen**, not the
    /// container's. Asserting only `.is_empty()` cannot see that, because an
    /// empty rect at the wrong origin is still empty; that blind spot is why
    /// §18.2's `panel` row was attributed to the component's own `x + 2`
    /// arithmetic when the shared helper had the same hole. So this asserts the
    /// **origin** of the saturating results, and then sweeps every tiny rect for
    /// the property the origin exists to protect: an inset is a subrect of its
    /// own container.
    #[test]
    fn inset_saturates_on_tiny_rects() {
        // the symmetric case is `Rect::inner`, which yields an empty rect when
        // the margin does not fit — anchored at the container, not at (0, 0)
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
        // no inset, symmetric or asymmetric, escapes the rect it was given
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
                        // the two shapes real callers pass: `Panel`'s
                        // card-with-a-title chrome, and `Dialog`'s
                        // horizontal-only padding
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
        // normal case and the seam
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

    /// Acceptance: no pane and no seam a `SplitModel` answers ever escapes the
    /// area it was given — the empty ones included. `Rect::ZERO` is anchored at
    /// the screen corner, so an empty pane carrying it reports an origin
    /// outside its own container and `.is_empty()` cannot reveal that.
    #[test]
    fn split_panes_and_seam_stay_inside_the_area() {
        let area = Rect::new(3, 2, 10, 6);
        let v = SplitModel::new(SplitAxis::Vertical, 60, 2, 2);
        let h = SplitModel::new(SplitAxis::Horizontal, 60, 2, 2);
        // the exact anchors of a maximised split
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
        // and the collapsed one, where the minima do not fit
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
