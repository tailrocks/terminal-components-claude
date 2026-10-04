//! Pooled layer buffers with a written-cell bitset (`COMPONENT_ARCHITECTURE.md` §3.3 step 12).

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};

use super::CellRoles;
use crate::id::Id;
use crate::layer::{LayerId, LayerSpec};
use crate::theme::PaintStyle;

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
    roles: Vec<CellRoles>,
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
            roles: vec![CellRoles::default(); screen.area() as usize],
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
        self.roles.clear();
        self.roles
            .resize(screen.area() as usize, CellRoles::default());
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
    pub(crate) fn mark(&mut self, pos: Position, style: Option<PaintStyle>) {
        if let Some(i) = self.index(pos)
            && let Some(w) = self.written.get_mut(i)
        {
            *w = true;
            if let Some(roles) = self.roles.get_mut(i) {
                *roles = roles.patch(style);
            }
        }
    }

    /// Mark every cell of `area` written.
    pub(crate) fn mark_area(&mut self, area: Rect, style: Option<PaintStyle>) {
        let area = area.intersection(*self.buf.area());
        for pos in area.positions() {
            self.mark(pos, style);
        }
    }

    pub(crate) fn roles_at(&self, pos: Position) -> CellRoles {
        self.index(pos)
            .and_then(|i| self.roles.get(i))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn set_roles(&mut self, pos: Position, roles: CellRoles) {
        if let Some(i) = self.index(pos)
            && let Some(target) = self.roles.get_mut(i)
        {
            *target = roles;
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
    pub(crate) fn composite_onto(&self, page: &mut Buffer, roles: &mut [CellRoles]) {
        let screen = *page.area();
        let area = self.buf.area().intersection(*page.area());
        for pos in area.positions() {
            if !self.is_written(pos) {
                continue;
            }
            if let (Some(src), Some(dst)) = (self.buf.cell(pos), page.cell_mut(pos)) {
                *dst = src.clone();
                let index = usize::from(pos.y.saturating_sub(screen.y))
                    .saturating_mul(usize::from(screen.width))
                    .saturating_add(usize::from(pos.x.saturating_sub(screen.x)));
                if let Some(target) = roles.get_mut(index) {
                    *target = self.roles_at(pos);
                }
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
            d.mark(Position::new(0, 0), None);
            d.mark(Position::new(1, 0), None);
            d.mark(Position::new(9, 9), None);
        }
        let mut page = Buffer::empty(screen);
        page.set_stringn(0, 0, "xxxx", 4, Style::new());
        if let Some(d) = pool.active().first() {
            d.composite_onto(
                &mut page,
                &mut vec![CellRoles::default(); screen.area() as usize],
            );
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
    #[test]
    fn reused_layer_clears_origins_and_inherited_channels_stay_local() {
        use crate::theme::{FgStep, Role, StylePatch, Surface, Theme};
        let screen = Rect::new(3, 2, 4, 2);
        let id = Id::root("pooled.origins");
        let spec = LayerSpec::modal(id);
        let mut d = LayerDraw::new(id, LayerId(1), spec, screen, screen);
        let theme = Theme::junie();
        let semantic = crate::theme::resolve::bind(
            &theme,
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Ghost))
                .set_bg(Role::Surface(Surface::Overlay)),
            None,
            Surface::Canvas,
        )
        .style;
        let pos = Position::new(3, 2);
        d.buf
            .set_stringn(pos.x, pos.y, "X", 1, semantic.into_style());
        d.mark(pos, Some(semantic));
        let original = d.roles_at(pos);
        d.mark(
            pos,
            Some(
                Style::new()
                    .add_modifier(ratatui_core::style::Modifier::BOLD)
                    .into(),
            ),
        );
        assert_eq!(d.roles_at(pos), original);
        d.mark(
            pos,
            Some(Style::new().bg(ratatui_core::style::Color::Red).into()),
        );
        assert_eq!(
            d.roles_at(pos),
            CellRoles {
                fg: original.fg,
                bg: None
            }
        );
        for next in [screen, Rect::new(1, 1, 8, 3)] {
            d.reset(id, LayerId(1), spec, next, next);
            assert!(!d.is_written(pos));
            assert_eq!(d.roles_at(pos), CellRoles::default());
            d.mark(
                pos,
                Some(Style::new().fg(ratatui_core::style::Color::Red).into()),
            );
            let mut page = Buffer::empty(next);
            let mut roles = vec![original; next.area() as usize];
            d.composite_onto(&mut page, &mut roles);
            let index = usize::from(pos.y.saturating_sub(next.y))
                .saturating_mul(usize::from(next.width))
                .saturating_add(usize::from(pos.x.saturating_sub(next.x)));
            assert_eq!(roles.get(index), Some(&CellRoles::default()));
            assert!(
                roles
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != index)
                    .all(|(_, r)| *r == original)
            );
        }
    }
}
