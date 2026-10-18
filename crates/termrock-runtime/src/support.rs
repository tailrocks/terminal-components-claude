



use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;

use crate::id::{Id, Part, PartRef};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Surface, Variant};
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
        let container = ov.style(ui, id, family, Variant::DEFAULT, Part::CONTAINER, live);
        ui.fill(area, container.style);
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

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;

    use super::cell_at;

    const AREA: Rect = Rect {
        x: 4,
        y: 5,
        width: 3,
        height: 2,
    };








    #[test]
    fn cell_at_is_empty_for_every_x_outside_the_area_on_either_side() {

        assert_eq!(cell_at(AREA, AREA.x - 1).width, 0);
        assert_eq!(cell_at(AREA, 0).width, 0);

        let narrow = Rect { width: 1, ..AREA };
        assert_eq!(cell_at(narrow, narrow.right().saturating_sub(2)).width, 0);

        let empty = Rect { width: 0, ..AREA };
        assert_eq!(cell_at(empty, empty.x).width, 0);


        for x in AREA.x..AREA.right() {
            assert_eq!(
                cell_at(AREA, x),
                Rect {
                    x,
                    y: AREA.y,
                    width: 1,
                    height: AREA.height,
                }
            );
        }



        assert_eq!(
            cell_at(AREA, AREA.right()),
            Rect {
                x: AREA.right(),
                y: AREA.y,
                width: 0,
                height: AREA.height,
            }
        );
        assert_eq!(cell_at(AREA, AREA.right() + 1).width, 0);
    }
}
