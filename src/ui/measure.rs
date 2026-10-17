//! Natural size (plan §1.4 `Measure`, the part S15 builds on).
//!
//! A child that can size itself says how much room it needs to show all of
//! its content without squeezing: a container lays it out at that size and
//! scrolls when the size is larger than the room it has
//! ([`ScrollView`](crate::ui::scroll_view::ScrollView)). `avail` is the room
//! the container offers; a child that wraps reads its width from it, a
//! child that fills whatever it gets answers `avail`.

use ratatui::layout::Size;

use crate::ui::ctx::RenderCtx;

/// A child that knows its natural size.
pub trait Measure {
    /// The cells (width, height) the child needs to show all of its
    /// content when offered `avail`. It may be larger than `avail` on
    /// either axis: the container then scrolls instead of squeezing it.
    fn measure(&self, avail: Size, ctx: &RenderCtx) -> Size;
}

/// The larger of two sizes on each axis: content laid out at its natural
/// size, but never smaller than the room it fills.
pub fn at_least(size: Size, min: Size) -> Size {
    Size::new(size.width.max(min.width), size.height.max(min.height))
}

/// One tile of a [`tiles`] layout: the narrowest width it shows its
/// content whole at, and its height at a given width (a card of wrapping
/// facts grows taller as it narrows).
pub struct Tile<'a> {
    pub min_width: u16,
    pub height: &'a dyn Fn(u16) -> u16,
}

impl<'a> Tile<'a> {
    pub fn new(min_width: u16, height: &'a dyn Fn(u16) -> u16) -> Self {
        Self { min_width, height }
    }
}

/// Where [`tiles`] puts each tile, and the size of the whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiled {
    /// One rect per tile, in the order given, relative to the origin.
    pub rects: Vec<ratatui::layout::Rect>,
    /// The natural size of the whole: at least `width` wide.
    pub size: Size,
}

impl Tiled {
    /// Tile `i`'s rect placed at `origin` (the area the layout fills).
    pub fn at(&self, i: usize, origin: ratatui::layout::Rect) -> ratatui::layout::Rect {
        let r = self.rects.get(i).copied().unwrap_or_default();
        ratatui::layout::Rect::new(origin.x + r.x, origin.y + r.y, r.width, r.height)
    }
}

/// Rows of tiles laid out at their natural size in `width` columns: a row
/// whose tiles fit side by side (each at its `min_width`, `gap.0` between)
/// shares the width (equal shares, every tile at least its minimum), and
/// is as tall as its tallest tile; a row that does not fit stacks its
/// tiles, each the full width. Rows are `gap.1` apart. The result is never
/// narrower than its widest tile, so a container that lays it out at
/// [`Tiled::size`] (a [`ScrollView`](crate::ui::scroll_view::ScrollView))
/// scrolls instead of cutting a tile.
pub fn tiles(width: u16, rows: &[&[Tile]], gap: (u16, u16)) -> Tiled {
    use ratatui::layout::Rect;
    let width = rows
        .iter()
        .flat_map(|r| r.iter())
        .map(|t| t.min_width)
        .fold(width, u16::max);
    let mut rects = Vec::new();
    let mut y = 0u16;
    for row in rows.iter().filter(|r| !r.is_empty()) {
        if !rects.is_empty() {
            y = y.saturating_add(gap.1);
        }
        let n = row.len() as u16;
        let gaps = gap.0.saturating_mul(n - 1);
        let mins: u16 = row.iter().map(|t| t.min_width).sum();
        if mins.saturating_add(gaps) <= width {
            // side by side: equal shares, every tile at least its minimum
            let room = width - gaps;
            let mut widths: Vec<u16> = row.iter().map(|t| t.min_width).collect();
            let mut spare = room - mins;
            let share = room / n;
            for w in widths.iter_mut() {
                let add = share.saturating_sub(*w).min(spare);
                *w += add;
                spare -= add;
            }
            // what is left goes to the last tile, so the row fills
            if let Some(last) = widths.last_mut() {
                *last += spare;
            }
            let h = row
                .iter()
                .zip(&widths)
                .map(|(t, &w)| (t.height)(w))
                .max()
                .unwrap_or(0);
            let mut x = 0u16;
            for &w in &widths {
                rects.push(Rect::new(x, y, w, h));
                x += w + gap.0;
            }
            y = y.saturating_add(h);
        } else {
            for (k, t) in row.iter().enumerate() {
                if k > 0 {
                    y = y.saturating_add(gap.1);
                }
                let h = (t.height)(width);
                rects.push(Rect::new(0, y, width, h));
                y = y.saturating_add(h);
            }
        }
    }
    Tiled {
        rects,
        size: Size::new(width, y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    /// Tiles that fit share the row; a row that does not fit stacks at
    /// the full width; nothing is ever narrower than its minimum.
    #[test]
    fn tiles_share_a_row_or_stack() {
        let short = |_: u16| 3;
        // a wrapping tile: 40 cells of text
        let wraps = |w: u16| 40u16.div_ceil(w.max(1));
        let rows: [&[Tile]; 2] = [
            &[Tile::new(20, &short), Tile::new(10, &wraps)],
            &[Tile::new(30, &short)],
        ];
        let t = tiles(60, &rows, (2, 1));
        assert_eq!(t.rects[0], Rect::new(0, 0, 29, 3));
        assert_eq!(t.rects[1], Rect::new(31, 0, 29, 3));
        assert_eq!(t.rects[2], Rect::new(0, 4, 60, 3));
        assert_eq!(t.size, Size::new(60, 7));
        assert_eq!(t.at(2, Rect::new(5, 6, 60, 7)), Rect::new(5, 10, 60, 3));
        // too narrow for both: they stack, each the full width
        let t = tiles(25, &rows, (2, 1));
        assert_eq!(t.rects[0], Rect::new(0, 0, 30, 3));
        assert_eq!(t.rects[1], Rect::new(0, 4, 30, 2));
        assert_eq!(t.rects[2], Rect::new(0, 7, 30, 3));
        // never narrower than the widest tile
        assert_eq!(t.size, Size::new(30, 10));
        // a tile wider than an equal share keeps its minimum
        let rows: [&[Tile]; 1] = [&[Tile::new(40, &short), Tile::new(5, &short)]];
        let t = tiles(60, &rows, (2, 1));
        assert_eq!(t.rects[0].width, 40);
        assert_eq!(t.rects[1], Rect::new(42, 0, 18, 3));
    }

    #[test]
    fn at_least_takes_the_larger_side_on_each_axis() {
        assert_eq!(
            at_least(Size::new(10, 50), Size::new(30, 20)),
            Size::new(30, 50)
        );
    }
}
