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
