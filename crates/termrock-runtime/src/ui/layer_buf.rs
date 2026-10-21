use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use std::collections::HashMap;

use super::CellRoles;
use crate::id::Id;
use crate::layer::{LayerId, LayerSpec};
use crate::theme::{PaintStyle, Role, Slot, StylePatch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnderlayChannel {
    color: Color,
    role: Option<Role>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct CellUnderlayStyle {
    fg: Option<UnderlayChannel>,
    bg: Option<UnderlayChannel>,
    underline: Option<Color>,
    add: Modifier,
    remove: Modifier,
    fade: Option<UnderlayFade>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UnderlayFade {
    container: Color,
    keep: f32,
    outer: bool,
}

impl CellUnderlayStyle {
    fn apply_paint(&mut self, paint: PaintStyle) {
        let style = paint.as_style();
        if let Some(color) = style.fg {
            self.fg = Some(UnderlayChannel {
                color,
                role: paint.fg_role.map(|(role, surface)| role.painted(surface)),
            });
            self.fade = None;
        }
        if let Some(color) = style.bg {
            self.bg = Some(UnderlayChannel {
                color,
                role: paint.bg_role.map(|(role, surface)| role.painted(surface)),
            });
        }
        if let Some(color) = style.underline_color {
            self.underline = Some(color);
        } else if paint.underline_cleared() {
            self.underline = Some(Color::Reset);
        }
        self.apply_paint_modifiers(style.add_modifier, style.sub_modifier);
    }

    fn apply_patch(&mut self, paint: PaintStyle, explicit: StylePatch) {
        let style = paint.as_style();
        match explicit.fg {
            Slot::Inherit => {}
            Slot::Set(_) => {
                self.fg = Some(UnderlayChannel {
                    color: style.fg.unwrap_or(Color::Reset),
                    role: paint.fg_role.map(|(role, surface)| role.painted(surface)),
                });
                self.fade = None;
            }
            Slot::Clear => {
                self.fg = Some(UnderlayChannel {
                    color: Color::Reset,
                    role: None,
                });
                self.fade = None;
            }
        }
        match explicit.bg {
            Slot::Inherit => {}
            Slot::Set(_) => {
                self.bg = Some(UnderlayChannel {
                    color: style.bg.unwrap_or(Color::Reset),
                    role: paint.bg_role.map(|(role, surface)| role.painted(surface)),
                });
            }
            Slot::Clear => {
                self.bg = Some(UnderlayChannel {
                    color: Color::Reset,
                    role: None,
                });
            }
        }
        match explicit.underline {
            Slot::Inherit => {}
            Slot::Set(_) => self.underline = Some(style.underline_color.unwrap_or(Color::Reset)),
            Slot::Clear => self.underline = Some(Color::Reset),
        }
        self.apply_patch_modifiers(explicit.add, explicit.remove);
    }

    fn apply_paint_modifiers(&mut self, add: Modifier, remove: Modifier) {
        // Ratatui `Cell::set_style` adds first, then removes. If a modifier is
        // present in both fields, removal wins.
        self.apply_modifier_op(add.difference(remove), remove.difference(add));
    }

    fn apply_patch_modifiers(&mut self, add: Modifier, remove: Modifier) {
        // StylePatch applies removals first, then additions. If a modifier is
        // present in both fields, addition wins.
        self.apply_modifier_op(add, remove.difference(add));
    }

    fn apply_modifier_op(&mut self, add: Modifier, remove: Modifier) {
        let touched = add.union(remove);
        self.add = self.add.difference(touched).union(add);
        self.remove = self.remove.difference(touched).union(remove);
    }

    pub(crate) fn defer_fade(&mut self, container: Color, keep: f32, outer: bool) {
        self.fade = Some(UnderlayFade {
            container,
            keep,
            outer,
        });
    }

    pub(crate) fn reset_source_ink(&mut self, reset: PaintStyle) {
        let style = reset.as_style();
        self.underline = Some(style.underline_color.unwrap_or(Color::Reset));
        let modifiers = style.add_modifier.difference(style.sub_modifier);
        self.add = modifiers;
        self.remove = Modifier::all().difference(modifiers);
        self.fade = None;
    }

    fn composite(
        self,
        underlay: &ratatui_core::buffer::Cell,
        underlay_roles: CellRoles,
        output: &mut ratatui_core::buffer::Cell,
    ) -> CellRoles {
        output.fg = self.fg.map_or(underlay.fg, |channel| channel.color);
        output.bg = self.bg.map_or(underlay.bg, |channel| channel.color);
        output.underline_color = self.underline.unwrap_or(underlay.underline_color);
        output.modifier = underlay.modifier.difference(self.remove).union(self.add);
        if let Some(fade) = self.fade
            && output.bg == fade.container
            && !output.modifier.contains(Modifier::REVERSED)
        {
            match crate::theme::builder::fade_mix(output.fg, fade.container, fade.keep) {
                crate::theme::builder::FadeOutcome::Blended(color) => output.fg = color,
                crate::theme::builder::FadeOutcome::ApplyDim if fade.outer => {
                    output.modifier.insert(Modifier::DIM);
                }
                crate::theme::builder::FadeOutcome::Unchanged
                | crate::theme::builder::FadeOutcome::ApplyDim => {}
            }
        }
        CellRoles {
            fg: self.fg.map_or(underlay_roles.fg, |channel| channel.role),
            bg: self.bg.map_or(underlay_roles.bg, |channel| channel.role),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LayerDraw {
    pub(crate) id: Id,
    pub(crate) layer: LayerId,
    pub(crate) spec: LayerSpec,
    pub(crate) area: Rect,
    pub(crate) buf: Buffer,
    pub(crate) written: Vec<bool>,
    roles: Vec<CellRoles>,
    underlay: Option<HashMap<usize, CellUnderlayStyle>>,
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
            underlay: None,
            drawn: false,
        }
    }

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
        if let Some(underlay) = &mut self.underlay {
            underlay.clear();
        }
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

    pub(crate) fn mark(&mut self, pos: Position, style: Option<PaintStyle>) {
        if let Some(i) = self.index(pos)
            && let Some(w) = self.written.get_mut(i)
        {
            *w = true;
            if let Some(underlay) = &mut self.underlay {
                if let Some(style) = style {
                    if let Some(intent) = underlay.get_mut(&i) {
                        intent.apply_paint(style);
                    }
                } else {
                    underlay.remove(&i);
                }
            }
            if let Some(roles) = self.roles.get_mut(i) {
                *roles = roles.patch(style);
            }
        }
    }

    pub(crate) fn begin_underlay(&mut self, pos: Position, bg: Color) {
        let Some(i) = self.index(pos) else {
            return;
        };
        let Some(written) = self.written.get_mut(i) else {
            return;
        };
        *written = true;
        if self.underlay.is_none() {
            self.underlay = Some(HashMap::new());
        }
        let mut style = CellUnderlayStyle::default();
        style.apply_paint(PaintStyle::new().bg(bg));
        if let Some(underlay) = &mut self.underlay {
            underlay.insert(i, style);
        }
        if let Some(roles) = self.roles.get_mut(i) {
            *roles = CellRoles::default();
        }
    }

    pub(crate) fn underlay_at(&self, pos: Position) -> Option<CellUnderlayStyle> {
        self.index(pos)
            .and_then(|index| self.underlay.as_ref()?.get(&index))
            .copied()
    }

    pub(crate) fn has_underlay(&self, pos: Position) -> bool {
        self.index(pos).is_some_and(|index| {
            self.underlay
                .as_ref()
                .is_some_and(|underlay| underlay.contains_key(&index))
        })
    }

    pub(crate) fn set_underlay(&mut self, pos: Position, intent: Option<CellUnderlayStyle>) {
        if let Some(index) = self.index(pos) {
            if let Some(intent) = intent {
                if let Some(underlay) = &mut self.underlay {
                    underlay.insert(index, intent);
                }
            } else if let Some(underlay) = &mut self.underlay {
                underlay.remove(&index);
            }
        }
    }

    pub(crate) fn patch_underlay(
        &mut self,
        pos: Position,
        paint: PaintStyle,
        explicit: StylePatch,
    ) {
        if let Some(intent) = self
            .index(pos)
            .and_then(|index| self.underlay.as_mut()?.get_mut(&index))
        {
            intent.apply_patch(paint, explicit);
        }
    }

    pub(crate) fn reset_source_underlay(&mut self, pos: Position, reset: PaintStyle) {
        if let Some(intent) = self
            .index(pos)
            .and_then(|index| self.underlay.as_mut()?.get_mut(&index))
        {
            intent.reset_source_ink(reset);
        }
    }

    pub(crate) fn defer_fade(&mut self, pos: Position, container: Color, keep: f32, outer: bool) {
        if let Some(intent) = self
            .index(pos)
            .and_then(|index| self.underlay.as_mut()?.get_mut(&index))
        {
            intent.defer_fade(container, keep, outer);
        }
    }

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

    pub(crate) fn is_written(&self, pos: Position) -> bool {
        self.index(pos)
            .and_then(|i| self.written.get(i))
            .copied()
            .unwrap_or(false)
    }

    pub(crate) fn composite_onto(&self, page: &mut Buffer, roles: &mut [CellRoles]) {
        let screen = *page.area();
        let area = self.buf.area().intersection(*page.area());
        for pos in area.positions() {
            if !self.is_written(pos) {
                continue;
            }
            if let (Some(src), Some(dst)) = (self.buf.cell(pos), page.cell_mut(pos)) {
                let index = usize::from(pos.y.saturating_sub(screen.y))
                    .saturating_mul(usize::from(screen.width))
                    .saturating_add(usize::from(pos.x.saturating_sub(screen.x)));
                let underlay_roles = roles.get(index).copied().unwrap_or_default();
                if let Some(intent) = self.underlay_at(pos) {
                    let underlay = (*dst).clone();
                    *dst = src.clone();
                    let composite_roles = intent.composite(&underlay, underlay_roles, dst);
                    if let Some(target) = roles.get_mut(index) {
                        *target = composite_roles;
                    }
                } else {
                    *dst = src.clone();
                    if let Some(target) = roles.get_mut(index) {
                        *target = self.roles_at(pos);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LayerPool {
    pub(crate) draws: Vec<LayerDraw>,
    len: usize,
}

impl LayerPool {
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
