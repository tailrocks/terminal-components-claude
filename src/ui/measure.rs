use ratatui::layout::Size;

use crate::ui::ctx::RenderCtx;

pub trait Measure {
    fn measure(&self, avail: Size, ctx: &RenderCtx) -> Size;

    fn min_width(&self, avail: Size, ctx: &RenderCtx) -> u16 {
        self.measure(avail, ctx).width
    }
}

pub fn at_least(size: Size, min: Size) -> Size {
    Size::new(size.width.max(min.width), size.height.max(min.height))
}

pub struct Tile<'a> {
    pub min_width: u16,
    pub height: &'a dyn Fn(u16) -> u16,

    pub grow: bool,
}

impl<'a> Tile<'a> {
    pub fn new(min_width: u16, height: &'a dyn Fn(u16) -> u16) -> Self {
        Self {
            min_width,
            height,
            grow: true,
        }
    }

    pub fn fixed(mut self) -> Self {
        self.grow = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiled {
    pub rects: Vec<ratatui::layout::Rect>,

    pub size: Size,
}

impl Tiled {
    pub fn at(&self, i: usize, origin: ratatui::layout::Rect) -> ratatui::layout::Rect {
        let r = self.rects.get(i).copied().unwrap_or_default();
        ratatui::layout::Rect::new(origin.x + r.x, origin.y + r.y, r.width, r.height)
    }
}

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
            let mut widths: Vec<u16> = row.iter().map(|t| t.min_width).collect();
            let mut spare = width - gaps - mins;
            let fixed: u16 = row.iter().filter(|t| !t.grow).map(|t| t.min_width).sum();
            let growing = row.iter().filter(|t| t.grow).count() as u16;
            if let Some(share) = (width - gaps - fixed).checked_div(growing) {
                for (w, t) in widths.iter_mut().zip(row.iter()) {
                    if t.grow {
                        let add = share.saturating_sub(*w).min(spare);
                        *w += add;
                        spare -= add;
                    }
                }

                if let Some(i) = row.iter().rposition(|t| t.grow) {
                    widths[i] += spare;
                }
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
                let w = if t.grow { width } else { t.min_width };
                let h = (t.height)(w);
                rects.push(Rect::new(0, y, w, h));
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

    #[test]
    fn tiles_share_a_row_or_stack() {
        let short = |_: u16| 3;

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

        let t = tiles(25, &rows, (2, 1));
        assert_eq!(t.rects[0], Rect::new(0, 0, 30, 3));
        assert_eq!(t.rects[1], Rect::new(0, 4, 30, 2));
        assert_eq!(t.rects[2], Rect::new(0, 7, 30, 3));

        assert_eq!(t.size, Size::new(30, 10));

        let rows: [&[Tile]; 1] = [&[Tile::new(40, &short), Tile::new(5, &short)]];
        let t = tiles(60, &rows, (2, 1));
        assert_eq!(t.rects[0].width, 40);
        assert_eq!(t.rects[1], Rect::new(42, 0, 18, 3));

        let rows: [&[Tile]; 1] = [&[Tile::new(20, &short).fixed(), Tile::new(5, &short)]];
        let t = tiles(60, &rows, (2, 1));
        assert_eq!(t.rects[0], Rect::new(0, 0, 20, 3));
        assert_eq!(t.rects[1], Rect::new(22, 0, 38, 3));
        let t = tiles(24, &rows, (2, 1));
        assert_eq!(t.rects[0], Rect::new(0, 0, 20, 3));
        assert_eq!(t.rects[1], Rect::new(0, 4, 24, 3));
    }

    #[test]
    fn at_least_takes_the_larger_side_on_each_axis() {
        assert_eq!(
            at_least(Size::new(10, 50), Size::new(30, 20)),
            Size::new(30, 50)
        );
    }
}
