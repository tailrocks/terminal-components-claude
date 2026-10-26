use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::id::{Id, Part, PartRef};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Slot, Surface, Variant};
use crate::ui::Ui;

pub use crate::author::PartStyle;

pub type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

#[expect(
    clippy::too_many_arguments,
    reason = "the shared chrome contract keeps each authored style and state channel explicit"
)]
pub fn overlay_chrome<R>(
    ui: &mut Ui<'_>,
    id: Id,
    area: Rect,
    family: Family,
    surface: Surface,
    ov: PartStyle<'_>,
    live: StateFlags,
    border_live: StateFlags,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> R {
    ui.with_surface(surface, |ui| {
        if area.is_empty() {
            return ui.with_area(area, |ui| body(ui, area));
        }
        let backdrop_chrome = matches!(family, Family::PICKER | Family::DIALOG);
        let container = (!backdrop_chrome)
            .then(|| ov.style(ui, id, family, Variant::DEFAULT, Part::CONTAINER, live));
        if backdrop_chrome {
            let container_patch = ov.part_patch(Part::CONTAINER);
            let detailed = ui.style_defaults_with_explicit(
                family,
                Variant::DEFAULT,
                Part::CONTAINER,
                live,
                crate::theme::StyleDefaults::default(),
                container_patch.as_ref(),
            );
            ov.note(
                ui,
                id,
                family,
                Variant::DEFAULT,
                Part::CONTAINER,
                detailed.resolved,
            );
            let resolved = detailed.resolved.style.into_style();
            let explicit = detailed.explicit;
            let card_bg = match explicit.bg {
                Slot::Inherit => resolved.bg.unwrap_or(Color::Reset),
                Slot::Set(_) => resolved.bg.unwrap_or(Color::Reset),
                Slot::Clear => Color::Reset,
            };
            for pos in area.positions() {
                let (src_fg, src_bg) = ui
                    .page_mut()
                    .cell(pos)
                    .map(|cell| (cell.fg, cell.bg))
                    .unwrap_or((
                        ratatui_core::style::Color::Reset,
                        ratatui_core::style::Color::Reset,
                    ));
                let (backdrop_fg, _, _) = ui.theme_ref().backdrop_cell(src_fg, src_bg);
                let dim = ui.theme_ref().backdrop_dim_level();
                if let Some(cell) = ui.buffer().cell_mut(pos) {
                    cell.set_symbol(" ");
                    cell.fg = backdrop_fg;
                    cell.bg = card_bg;
                    cell.modifier = if dim {
                        Modifier::DIM
                    } else {
                        Modifier::empty()
                    };
                }
            }
            ui.patch_style(area, detailed.resolved.style, explicit);
            ui.mark_area(area, None);
        } else if let Some(container) = container {
            ui.fill(area, container.style);
        }
        ui.register_decor(id, PartRef::of(Part::CONTAINER), area);

        let border = ov.style(ui, id, family, Variant::DEFAULT, Part::BORDER, border_live);
        let inner = ui.frame(area, border.style);
        if let Some(slot) = ov.slot_for(Part::BORDER) {
            slot(ui, area);
        }
        ui.register_decor(id, PartRef::of(Part::BORDER), area);

        let body_area = if inner.is_empty() {
            Rect {
                x: area.x,
                y: area.y,
                width: 0,
                height: 0,
            }
        } else {
            inner
        };
        ui.with_area(body_area, |ui| body(ui, body_area))
    })
}

pub const fn first_row(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: if area.height == 0 { 0 } else { 1 },
    }
}

pub const fn cell_at(area: Rect, x: u16) -> Rect {
    Rect {
        x,
        y: area.y,
        width: if x >= area.x && x < area.x.saturating_add(area.width) {
            1
        } else {
            0
        },
        height: area.height,
    }
}

pub fn paint_pressed_bracket(ui: &mut Ui<'_>, left: Rect, right: Rect, style: PaintStyle) {
    ui.glyph(left, GlyphRole::PressLeft, style);
    ui.glyph(right, GlyphRole::PressRight, style);
}

pub const fn shift(area: Rect, by: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(by),
        y: area.y,
        width: area.width.saturating_sub(by),
        height: area.height,
    }
}

pub struct Acc<A> {
    consumed: bool,
    invalidate: crate::response::Invalidate,
    action: Option<A>,
}

impl<A: Default> Default for Acc<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> Acc<A> {
    pub const fn new() -> Self {
        Acc {
            consumed: false,
            invalidate: crate::response::Invalidate::None,
            action: None,
        }
    }

    pub fn consumed(&mut self) {
        self.consumed = true;
    }

    pub fn changed(&mut self) {
        self.consumed = true;
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub fn repaint(&mut self) {
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub fn action(&mut self, a: A) {
        self.changed();
        self.action = Some(a);
    }

    pub fn fold(&mut self, r: &crate::response::Response<()>) {
        self.consumed |= r.is_consumed();
        self.invalidate = self.invalidate.max(r.invalidate());
    }

    pub fn finish(self, id: Id) -> crate::response::Response<A> {
        use crate::response::{Invalidate, Response};
        let r = match self.action {
            Some(a) => Response::action(a),
            None if self.consumed => Response::consumed(),
            None if self.invalidate == Invalidate::None => return Response::ignored(),
            None => Response::ignored(),
        };
        let r = match self.invalidate {
            Invalidate::None => r,
            Invalidate::Paint => r.repaint(),
            Invalidate::Layout => r.relayout(),
        };
        r.for_id(id)
    }
}
