//! Pooled layer buffers with a written-cell bitset (`COMPONENT_ARCHITECTURE.md` §3.3 step 12).

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};

use crate::id::Id;
use crate::layer::{LayerId, LayerSpec};

/// One open layer's draw target for the frame.
#[derive(Debug, Clone)]
pub(crate) struct LayerDraw {
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
    pub(crate) spec: LayerSpec,
    /// The resolved area on screen.
    pub(crate) area: Rect,
    /// A screen-sized buffer; only `written` cells are composited.
    pub(crate) buf: Buffer,
    pub(crate) written: Vec<bool>,
    pub(crate) drawn: bool,
}

impl LayerDraw {
    pub(crate) fn new(id: Id, layer: LayerId, spec: LayerSpec, area: Rect, screen: Rect) -> Self {
        LayerDraw {
            id,
            layer,
            spec,
            area,
            buf: Buffer::empty(screen),
            written: vec![false; screen.area() as usize],
            drawn: false,
        }
    }

    /// Reuse for a new frame.
    pub(crate) fn reset(
        &mut self,
        id: Id,
        layer: LayerId,
        spec: LayerSpec,
        area: Rect,
        screen: Rect,
    ) {
        self.id = id;
        self.layer = layer;
        self.spec = spec;
        self.area = area;
        self.drawn = false;
        if *self.buf.area() == screen {
            self.buf.reset();
        } else {
            self.buf.resize(screen);
            self.buf.reset();
        }
        self.written.clear();
        self.written.resize(screen.area() as usize, false);
    }

    fn index(&self, pos: Position) -> Option<usize> {
        let a = *self.buf.area();
        if !a.contains(pos) {
            return None;
        }
        let row = usize::from(pos.y.saturating_sub(a.y));
        let col = usize::from(pos.x.saturating_sub(a.x));
        Some(row.saturating_mul(usize::from(a.width)).saturating_add(col))
    }

    /// Mark one cell written.
    pub(crate) fn mark(&mut self, pos: Position) {
        if let Some(i) = self.index(pos)
            && let Some(w) = self.written.get_mut(i)
        {
            *w = true;
        }
    }

    /// Mark every cell of `area` written.
    pub(crate) fn mark_area(&mut self, area: Rect) {
        let area = area.intersection(*self.buf.area());
        for pos in area.positions() {
            self.mark(pos);
        }
    }

    /// Whether a cell was written this frame.
    pub(crate) fn is_written(&self, pos: Position) -> bool {
        self.index(pos)
            .and_then(|i| self.written.get(i))
            .copied()
            .unwrap_or(false)
    }

    /// Copy written cells onto `page`.
    pub(crate) fn composite_onto(&self, page: &mut Buffer) {
        let area = self.buf.area().intersection(*page.area());
        for pos in area.positions() {
            if !self.is_written(pos) {
                continue;
            }
            if let (Some(src), Some(dst)) = (self.buf.cell(pos), page.cell_mut(pos)) {
                *dst = src.clone();
            }
        }
    }
}

/// The pool of layer draws, reused across frames.
#[derive(Debug, Default, Clone)]
pub(crate) struct LayerPool {
    pub(crate) draws: Vec<LayerDraw>,
    len: usize,
}

impl LayerPool {
    /// Start a frame with `n` layers; returns nothing, callers fill via `push`.
    pub(crate) fn begin(&mut self) {
        self.len = 0;
    }

    pub(crate) fn push(
        &mut self,
        id: Id,
        layer: LayerId,
        spec: LayerSpec,
        area: Rect,
        screen: Rect,
    ) {
        if let Some(d) = self.draws.get_mut(self.len) {
            d.reset(id, layer, spec, area, screen);
        } else {
            self.draws
                .push(LayerDraw::new(id, layer, spec, area, screen));
        }
        self.len = self.len.saturating_add(1);
    }

    pub(crate) fn active(&self) -> &[LayerDraw] {
        self.draws.get(..self.len).unwrap_or(&[])
    }

    pub(crate) fn active_mut(&mut self) -> &mut [LayerDraw] {
        let n = self.len;
        self.draws.get_mut(..n).unwrap_or(&mut [])
    }

    pub(crate) fn find(&self, id: Id) -> Option<usize> {
        self.active().iter().position(|d| d.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::style::Style;

    #[test]
    fn only_written_cells_composite() {
        let screen = Rect::new(0, 0, 4, 2);
        let mut pool = LayerPool::default();
        pool.begin();
        pool.push(
            Id::root("l"),
            LayerId(1),
            LayerSpec::modal(Id::root("l")),
            screen,
            screen,
        );
        let first = pool.active_mut().first_mut();
        assert!(first.is_some());
        if let Some(d) = first {
            d.buf.set_stringn(0, 0, "ab", 4, Style::new());
            d.mark(Position::new(0, 0));
            d.mark(Position::new(1, 0));
            d.mark(Position::new(9, 9));
        }
        let mut page = Buffer::empty(screen);
        page.set_stringn(0, 0, "xxxx", 4, Style::new());
        if let Some(d) = pool.active().first() {
            d.composite_onto(&mut page);
            assert!(d.is_written(Position::new(1, 0)));
            assert!(!d.is_written(Position::new(2, 0)));
        }
        let row: String = (0..4u16)
            .filter_map(|x| page.cell((x, 0)).map(|c| c.symbol().to_owned()))
            .collect();
        assert_eq!(row, "abxx");
        assert_eq!(pool.find(Id::root("l")), Some(0));
        pool.begin();
        assert!(pool.active().is_empty());
    }
}
