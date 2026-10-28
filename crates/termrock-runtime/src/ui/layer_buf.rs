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

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CellUnderlayStyle {
    bg: Option<UnderlayChannel>,
    operations: Vec<UnderlayOp>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum UnderlayOp {
    Paint(PaintStyle),
    Patch(PaintStyle, StylePatch),
    Fade(UnderlayFade),
    ResetSourceInk(PaintStyle),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UnderlayFade {
    container: Color,
    keep: f32,
    outer: bool,
}

impl CellUnderlayStyle {
    fn apply_paint(&mut self, paint: PaintStyle) {
        self.operations.push(UnderlayOp::Paint(paint));
    }

    fn apply_patch(&mut self, paint: PaintStyle, explicit: StylePatch) {
        self.operations.push(UnderlayOp::Patch(paint, explicit));
    }

    pub(crate) fn defer_fade(&mut self, container: Color, keep: f32, outer: bool) {
        self.operations.push(UnderlayOp::Fade(UnderlayFade {
            container,
            keep,
            outer,
        }));
    }

    pub(crate) fn reset_source_ink(&mut self, reset: PaintStyle) {
        self.operations.push(UnderlayOp::ResetSourceInk(reset));
    }

    fn composite(
        &self,
        underlay: &ratatui_core::buffer::Cell,
        underlay_roles: CellRoles,
        output: &mut ratatui_core::buffer::Cell,
    ) -> CellRoles {
        output.fg = underlay.fg;
        output.bg = self.bg.map_or(underlay.bg, |channel| channel.color);
        output.underline_color = underlay.underline_color;
        output.modifier = underlay.modifier;
        let mut roles = underlay_roles;
        if let Some(bg) = self.bg {
            roles.bg = bg.role;
        }
        for operation in &self.operations {
            match *operation {
                UnderlayOp::Paint(paint) => apply_paint(paint, output, &mut roles),
                UnderlayOp::Patch(paint, explicit) => {
                    apply_patch(paint, explicit, output, &mut roles);
                }
                UnderlayOp::Fade(fade) => apply_fade(fade, output),
                UnderlayOp::ResetSourceInk(reset) => reset_source_ink(reset, output),
            }
        }
        roles
    }
}

fn apply_paint(paint: PaintStyle, output: &mut ratatui_core::buffer::Cell, roles: &mut CellRoles) {
    let style = paint.as_style();
    if let Some(color) = style.fg {
        output.fg = color;
        roles.fg = paint.fg_role.map(|(role, surface)| role.painted(surface));
    }
    if let Some(color) = style.bg {
        output.bg = color;
        roles.bg = paint.bg_role.map(|(role, surface)| role.painted(surface));
    }
    if let Some(color) = style.underline_color {
        output.underline_color = color;
    } else if paint.underline_cleared() {
        output.underline_color = Color::Reset;
    }

    output.modifier.insert(style.add_modifier);
    output.modifier.remove(style.sub_modifier);
}

fn apply_patch(
    paint: PaintStyle,
    explicit: StylePatch,
    output: &mut ratatui_core::buffer::Cell,
    roles: &mut CellRoles,
) {
    let style = paint.as_style();
    match explicit.fg {
        Slot::Inherit => {}
        Slot::Set(_) => {
            output.fg = style.fg.unwrap_or(Color::Reset);
            roles.fg = paint.fg_role.map(|(role, surface)| role.painted(surface));
        }
        Slot::Clear => {
            output.fg = Color::Reset;
            roles.fg = None;
        }
    }
    match explicit.bg {
        Slot::Inherit => {}
        Slot::Set(_) => {
            output.bg = style.bg.unwrap_or(Color::Reset);
            roles.bg = paint.bg_role.map(|(role, surface)| role.painted(surface));
        }
        Slot::Clear => {
            output.bg = Color::Reset;
            roles.bg = None;
        }
    }
    match explicit.underline {
        Slot::Inherit => {}
        Slot::Set(_) => output.underline_color = style.underline_color.unwrap_or(Color::Reset),
        Slot::Clear => output.underline_color = Color::Reset,
    }

    output.modifier.remove(explicit.remove);
    output.modifier.insert(explicit.add);
}

fn apply_fade(fade: UnderlayFade, output: &mut ratatui_core::buffer::Cell) {
    if output.bg != fade.container || output.modifier.contains(Modifier::REVERSED) {
        return;
    }
    match crate::theme::builder::fade_mix(output.fg, fade.container, fade.keep) {
        crate::theme::builder::FadeOutcome::Blended(color) => output.fg = color,
        crate::theme::builder::FadeOutcome::ApplyDim if fade.outer => {
            output.modifier.insert(Modifier::DIM);
        }
        crate::theme::builder::FadeOutcome::Unchanged
        | crate::theme::builder::FadeOutcome::ApplyDim => {}
    }
}

fn reset_source_ink(reset: PaintStyle, output: &mut ratatui_core::buffer::Cell) {
    let style = reset.as_style();
    output.underline_color = style.underline_color.unwrap_or(Color::Reset);
    output.modifier = style.add_modifier.difference(style.sub_modifier);
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
        let style = CellUnderlayStyle {
            bg: Some(UnderlayChannel {
                color: bg,
                role: None,
            }),
            ..CellUnderlayStyle::default()
        };
        if let Some(underlay) = &mut self.underlay {
            underlay.insert(i, style);
        }
        if let Some(roles) = self.roles.get_mut(i) {
            *roles = CellRoles::default();
        }
    }

    pub(crate) fn underlay_at(&self, pos: Position) -> Option<&CellUnderlayStyle> {
        self.index(pos)
            .and_then(|index| self.underlay.as_ref()?.get(&index))
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

    pub(crate) fn materialize_underlay(
        &mut self,
        pos: Position,
        page_cell: &ratatui_core::buffer::Cell,
        page_roles: CellRoles,
    ) {
        let Some(intent) = self.underlay_at(pos).cloned() else {
            return;
        };
        let Some(dst) = self.buf.cell_mut(pos) else {
            return;
        };
        let _roles = intent.composite(page_cell, page_roles, dst);
    }

    pub(crate) fn composite_onto(
        &self,
        page: &mut Buffer,
        roles: &mut [CellRoles],
        content_dim: bool,
    ) {
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

                    if content_dim {
                        dst.modifier.insert(ratatui_core::style::Modifier::DIM);
                    }
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
