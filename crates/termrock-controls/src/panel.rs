use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{PartStyle, SlotFn, cell_at, first_row};
use crate::id::{Id, Part, PartRef};
use crate::layout::{Insets, inset};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{FrameRead, Ui, VerticalWindow};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PanelKind {
    #[default]
    Card,
    Framed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelWindowError {
    InvalidWindow,

    UnsupportedLegacySlot(Part),
}

impl fmt::Display for PanelWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindow => f.write_str("window does not match the panel logical area"),
            Self::UnsupportedLegacySlot(part) => {
                write!(
                    f,
                    "partial panel window does not support the {part:?} legacy slot"
                )
            }
        }
    }
}

impl core::error::Error for PanelWindowError {}

pub struct Panel<'a> {
    id: Id,
    kind: PanelKind,
    title: Option<&'a str>,
    meta: Option<&'a str>,
    badge: Option<&'a str>,
    focused: bool,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Panel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Panel")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("title", &self.title)
            .field("meta", &self.meta)
            .field("badge", &self.badge)
            .field("focused", &self.focused)
            .field("overrides", &self.ov)
            .finish()
    }
}

impl<'a> Panel<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::TITLE,
        Part::DETAIL,
        Part::BADGE,
        Part::BORDER,
    ];

    pub const fn new(id: Id) -> Self {
        Panel {
            id,
            kind: PanelKind::Card,
            title: None,
            meta: None,
            badge: None,
            focused: false,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(mut self, k: PanelKind) -> Self {
        self.kind = k;
        self
    }

    #[must_use]
    pub const fn title(mut self, t: &'a str) -> Self {
        self.title = Some(t);
        self
    }

    #[must_use]
    pub const fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }

    #[must_use]
    pub const fn badge(mut self, text: &'a str) -> Self {
        self.badge = if text.is_empty() { None } else { Some(text) };
        self
    }

    #[must_use]
    pub const fn focused(mut self, yes: bool) -> Self {
        self.focused = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn derived(&self) -> StateFlags {
        if self.focused {
            StateFlags::FOCUSED
        } else {
            StateFlags::empty()
        }
    }

    fn plane(&self, ui: &Ui<'_>) -> Surface {
        match self.kind {
            PanelKind::Card => ui.theme().raise(ui.surface()),
            PanelKind::Framed => ui.surface(),
        }
    }

    const fn top_inset(&self) -> u16 {
        match self.kind {
            PanelKind::Card if self.title.is_some() => 2,
            PanelKind::Card | PanelKind::Framed => 1,
        }
    }

    fn insets(&self, ui: &Ui<'_>) -> Insets {
        let side = match self.kind {
            PanelKind::Card => ui.design().space.card_inset,
            PanelKind::Framed => ui.design().space.frame_inset,
        };
        Insets {
            l: side,
            t: self.top_inset(),
            r: side,
            b: 1,
        }
    }

    pub fn inner(&self, ui: &Ui<'_>, area: Rect) -> Rect {
        let r = inset(area, self.insets(ui));
        if r.is_empty() {
            return Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            };
        }
        r
    }

    fn badge_lane_width(&self) -> Option<u16> {
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let badge_w = self.badge.map(crate::text::width)?;

        Some(
            title_w
                .saturating_add(meta_w)
                .saturating_add(badge_w)
                .saturating_add(7)
                .saturating_add(if self.kind == PanelKind::Framed {
                    1u16.saturating_add(2u16.saturating_mul(u16::from(self.meta.is_some())))
                } else {
                    0
                }),
        )
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let insets = self.insets(ui);
        let chrome_w = insets.l.saturating_add(insets.r);
        let chrome_h = insets.t.saturating_add(insets.b);
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let head = title_w
            .saturating_add(meta_w)
            .saturating_add(u16::from(meta_w != 0))
            .saturating_add(2);
        let head = self.badge_lane_width().map_or(head, |lane| lane.max(head));
        Size {
            min: (chrome_w.saturating_add(1), chrome_h.saturating_add(1)),
            preferred: (
                chrome_w.saturating_add(1).max(head),
                chrome_h.saturating_add(1),
            ),
        }
        .fit(c)
    }

    pub fn measure_with_body(
        &self,
        ui: &Ui<'_>,
        constraints: Constraints,
        measure_body: impl FnOnce(Constraints) -> Size,
    ) -> Size {
        let insets = self.insets(ui);
        let chrome_w = insets.l.saturating_add(insets.r);
        let chrome_h = insets.t.saturating_add(insets.b);
        let body_constraints = Constraints {
            max: (
                constraints.max.0.saturating_sub(chrome_w),
                constraints.max.1.saturating_sub(chrome_h),
            ),
            tight_w: constraints.tight_w,
            tight_h: constraints.tight_h,
        };
        let body = measure_body(body_constraints);
        let title_w = self.title.map_or(0, crate::text::width);
        let meta_w = self.meta.map_or(0, crate::text::width);
        let head = title_w
            .saturating_add(meta_w)
            .saturating_add(u16::from(meta_w != 0))
            .saturating_add(2);
        let head = self.badge_lane_width().map_or(head, |lane| lane.max(head));
        let one_body_cell = 1;
        Size {
            min: (
                chrome_w
                    .saturating_add(body.min.0)
                    .max(chrome_w.saturating_add(one_body_cell)),
                chrome_h
                    .saturating_add(body.min.1)
                    .max(chrome_h.saturating_add(one_body_cell)),
            ),
            preferred: (
                chrome_w
                    .saturating_add(body.preferred.0)
                    .max(chrome_w.saturating_add(one_body_cell))
                    .max(head),
                chrome_h
                    .saturating_add(body.preferred.1)
                    .max(chrome_h.saturating_add(one_body_cell)),
            ),
        }
        .fit(constraints)
    }

    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> R {
        let plane = self.plane(ui);
        ui.with_surface(plane, |ui| {
            let inner = self.inner(ui, area);
            self.chrome(ui, area);
            ui.with_area(inner, |ui| body(ui, inner))
        })
    }

    pub fn draw_window<R>(
        &self,
        ui: &mut Ui<'_>,
        logical_area: Rect,
        window: VerticalWindow,
        body: impl FnOnce(&mut Ui<'_>, Option<VerticalWindow>) -> R,
    ) -> Result<R, PanelWindowError> {
        if window == VerticalWindow::full(logical_area) {
            if !logical_area.is_empty()
                && VerticalWindow::new(logical_area, usize::from(logical_area.height), 0).is_err()
            {
                return Err(PanelWindowError::InvalidWindow);
            }
            return Ok(self.draw(ui, logical_area, |ui, inner| {
                let body_window = (!inner.is_empty()).then(|| VerticalWindow::full(inner));
                body(ui, body_window)
            }));
        }

        let visible = window.visible_body();
        let valid = window.logical_rows() == usize::from(logical_area.height)
            && visible.x == logical_area.x
            && visible.width == logical_area.width
            && !visible.is_empty()
            && VerticalWindow::new(
                visible,
                usize::from(logical_area.height),
                window.first_source_row(),
            )
            .is_ok();
        if !valid {
            return Err(PanelWindowError::InvalidWindow);
        }
        if let Some((part, _)) = self.ov.slot {
            return Err(PanelWindowError::UnsupportedLegacySlot(part));
        }

        let plane = self.plane(ui);
        Ok(ui.with_surface(plane, |ui| {
            ui.with_area(visible, |ui| {
                let inner = self.inner(ui, logical_area);
                let live = PartStyle::flags(StateFlags::empty(), self.derived());
                let container = self.ov.style(
                    ui,
                    self.id,
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    live,
                );
                ui.fill(visible, container.style);
                ui.register_decor(self.id, PartRef::of(Part::CONTAINER), visible);

                if self.kind == PanelKind::Framed {
                    let border = self.ov.style(
                        ui,
                        self.id,
                        Family::PANEL,
                        Variant::DEFAULT,
                        Part::BORDER,
                        live,
                    );
                    self.frame_window(ui, logical_area, window, border.style);
                    ui.register_decor(self.id, PartRef::of(Part::BORDER), visible);
                }

                if let Some(head_y) = window.physical_row_for_source(0) {
                    let head = Rect::new(logical_area.x, head_y, logical_area.width, 1);
                    self.head(ui, head, live, container.style);
                }

                let body_window = body_window(inner, logical_area, window);
                if let Some(body_window) = body_window {
                    ui.with_area(body_window.visible_body(), |ui| body(ui, Some(body_window)))
                } else {
                    let empty = Rect::new(inner.x, visible.y, inner.width, 0);
                    ui.with_area(empty, |ui| body(ui, None))
                }
            })
        }))
    }

    fn frame_window(
        &self,
        ui: &mut Ui<'_>,
        logical_area: Rect,
        window: VerticalWindow,
        style: crate::theme::PaintStyle,
    ) {
        if logical_area.width < 2 || logical_area.height < 2 {
            return;
        }

        let borders = ui.theme_ref().design.borders;
        let left = logical_area.left();
        let right = logical_area.right().saturating_sub(1);
        let bottom_source = usize::from(logical_area.height.saturating_sub(1));
        for source_row in window.visible_source_range() {
            let Some(y) = window.physical_row_for_source(source_row) else {
                continue;
            };
            let top = source_row == 0;
            let bottom = source_row == bottom_source;
            if top || bottom {
                let horizontal = if top {
                    borders.horizontal_top
                } else {
                    borders.horizontal_bottom
                };
                for col in logical_area.columns().map(|col| col.x) {
                    ui.paint_cell(Position::new(col, y), horizontal, style);
                }
                let (left_corner, right_corner) = if top {
                    (borders.top_left, borders.top_right)
                } else {
                    (borders.bottom_left, borders.bottom_right)
                };
                ui.paint_cell(Position::new(left, y), left_corner, style);
                ui.paint_cell(Position::new(right, y), right_corner, style);
            } else {
                ui.paint_cell(Position::new(left, y), borders.vertical_left, style);
                ui.paint_cell(Position::new(right, y), borders.vertical_right, style);
            }
        }
    }

    fn chrome(&self, ui: &mut Ui<'_>, area: Rect) {
        let ov = self.ov;
        let id = self.id;
        let live = PartStyle::flags(StateFlags::empty(), self.derived());
        let container = ov.style(
            ui,
            id,
            Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        ui.register_decor(id, PartRef::of(Part::CONTAINER), area);
        if self.kind == PanelKind::Framed {
            let border = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::BORDER, live);
            if let Some(f) = ov.slot_for(Part::BORDER) {
                f(ui, area);
            } else {
                ui.frame(area, border.style);
            }
            ui.register_decor(id, PartRef::of(Part::BORDER), area);
        }
        self.head(ui, area, live, container.style);
    }

    fn draw_badge(&self, ui: &mut Ui<'_>, rect: Rect, live: StateFlags) {
        let Some(text) = self.badge.filter(|_| !rect.is_empty()) else {
            return;
        };
        let style = self.ov.style(
            ui,
            self.id,
            Family::PANEL,
            Variant::DEFAULT,
            Part::BADGE,
            live,
        );
        ui.with_area(rect, |ui| {
            if let Some(paint) = self.ov.slot_for(Part::BADGE) {
                paint(ui, rect);
            } else {
                ui.fill(rect, style.over(ui.surface_style()));
                ui.paint_str(
                    Rect {
                        x: rect.x.saturating_add(1),
                        y: rect.y,
                        width: rect.width.saturating_sub(2),
                        height: 1,
                    },
                    text,
                    style.style,
                );
            }
        });
    }

    fn head(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags, fill: crate::theme::PaintStyle) {
        let head = first_row(area);
        if head.is_empty() || area.width < 3 {
            return;
        }
        let ov = self.ov;
        let id = self.id;

        let pad = u16::from(self.kind == PanelKind::Framed);
        let gutter_x = area.x.saturating_add(1);
        let gutter = cell_at(head, gutter_x);
        if let Some(f) = ov.slot_for(Part::GUTTER) {
            f(ui, gutter);
        } else {
            let g = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::GUTTER, live);
            match g.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter, glyph, g.style);
                }
                Slot::Inherit if live.contains(StateFlags::FOCUSED) => {
                    ui.glyph(gutter, GlyphRole::FocusBar, g.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(gutter, g.style),
            }
        }
        let text_x = area.x.saturating_add(2);

        let span_w = area.width.saturating_sub(3);

        if self
            .badge_lane_width()
            .is_none_or(|needed| area.width < needed)
        {
            self.head_without_badge(ui, area, live, fill);
            return;
        }

        let span_right = text_x.saturating_add(span_w);
        let mut right = span_right;

        let meta_block = self.meta.and_then(|m| {
            let width = crate::text::width(m).saturating_add(pad.saturating_mul(2));
            (width < span_w).then_some(width)
        });
        let meta_rect = meta_block.map(|width| {
            right = right.saturating_sub(width);
            Rect {
                x: right,
                y: head.y,
                width,
                height: 1,
            }
        });

        let badge_block = self.badge.and_then(|b| {
            let width = crate::text::width(b).saturating_add(2);
            let gap = u16::from(meta_rect.is_some());
            (width.saturating_add(gap) <= right.saturating_sub(text_x)).then_some((width, gap))
        });
        let badge_rect = badge_block.map(|(width, gap)| {
            right = right.saturating_sub(gap).saturating_sub(width);
            Rect {
                x: right,
                y: head.y,
                width,
                height: 1,
            }
        });

        if let Some(t) = self.title {
            let rect = Rect {
                x: text_x,
                y: head.y,
                width: right.saturating_sub(text_x),
                height: 1,
            };
            if let Some(f) = ov.slot_for(Part::TITLE) {
                f(ui, rect);
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::TITLE, live);
                paint_label(ui, rect, t, pad, s.style);
            }
        }

        if let Some(rect) = badge_rect {
            self.draw_badge(ui, rect, live);
        }

        if let (Some(m), Some(rect)) = (self.meta, meta_rect) {
            if let Some(f) = ov.slot_for(Part::DETAIL) {
                f(ui, rect);
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live);
                paint_label(ui, rect, m, pad, s.style);
            }
        }
    }

    fn head_without_badge(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        live: StateFlags,
        fill: crate::theme::PaintStyle,
    ) {
        let head = first_row(area);
        let text_x = area.x.saturating_add(2);
        let span_w = area.width.saturating_sub(3);
        let pad = u16::from(self.kind == PanelKind::Framed);
        let ov = self.ov;
        let id = self.id;
        let meta_block = self.meta.map_or(0, |m| {
            let want = crate::text::width(m).saturating_add(pad.saturating_mul(2));
            if want < span_w { want } else { 0 }
        });
        let title_room = span_w.saturating_sub(meta_block);
        if let Some(t) = self.title {
            let avail = title_room.saturating_sub(pad);
            let rect = Rect {
                x: text_x,
                y: head.y,
                width: avail,
                height: 1,
            };
            let used = if let Some(f) = ov.slot_for(Part::TITLE) {
                f(ui, rect);
                avail
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::TITLE, live);
                ui.paint_str(rect, t, s.style)
            };
            if pad == 1 && used > 0 {
                ui.fill(cell_at(head, text_x.saturating_add(used)), fill);
            }
        }
        if let (Some(m), true) = (self.meta, meta_block > 0) {
            let x = text_x.saturating_add(title_room).saturating_add(pad);
            let rect = Rect {
                x,
                y: head.y,
                width: meta_block.saturating_sub(pad.saturating_mul(2)),
                height: 1,
            };
            let used = if let Some(f) = ov.slot_for(Part::DETAIL) {
                f(ui, rect);
                rect.width
            } else {
                let s = ov.style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live);

                if pad == 1 {
                    ui.fill(cell_at(head, x.saturating_sub(1)), s.style);
                }
                ui.paint_str(rect, m, s.style)
            };
            if pad == 1 && used > 0 {
                let style = ov
                    .style(ui, id, Family::PANEL, Variant::DEFAULT, Part::DETAIL, live)
                    .style;
                ui.fill(cell_at(head, x.saturating_add(used)), style);
            }
        }
    }
}

fn body_window(
    inner: Rect,
    logical_area: Rect,
    panel_window: VerticalWindow,
) -> Option<VerticalWindow> {
    if inner.is_empty() {
        return None;
    }

    let body_start = usize::from(inner.y.checked_sub(logical_area.y)?);
    let body_end = body_start.saturating_add(usize::from(inner.height));
    let visible = panel_window.visible_source_range();
    let first = visible.start.max(body_start);
    let end = visible.end.min(body_end);
    if first >= end {
        return None;
    }

    let physical_y = panel_window.physical_row_for_source(first)?;
    let visible_height = u16::try_from(end.saturating_sub(first)).ok()?;
    let visible_body = Rect::new(inner.x, physical_y, inner.width, visible_height);
    VerticalWindow::new(visible_body, usize::from(inner.height), first - body_start).ok()
}

fn paint_label(
    ui: &mut Ui<'_>,
    block: Rect,
    text: &str,
    pad: u16,
    style: crate::theme::PaintStyle,
) {
    let inner = Rect {
        x: block.x.saturating_add(pad),
        width: block.width.saturating_sub(pad.saturating_mul(2)),
        ..block
    };
    let text_width = crate::text::width(text).min(inner.width);
    let label = Rect {
        width: text_width.saturating_add(pad.saturating_mul(2)),
        ..block
    };
    if pad > 0 {
        ui.fill(label, style);
    }
    ui.paint_str(inner, text, style);
}
