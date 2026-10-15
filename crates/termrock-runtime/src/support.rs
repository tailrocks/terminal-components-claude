//! Shared component-support helpers: slot slots, chrome overlay, geometry
//! cuts, and the per-intent response fold. Split out of the legacy
//! `components` module so every component crate owns one copy.

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;

use crate::id::{Id, Part, PartRef};
use crate::response::StateFlags;
use crate::theme::{Family, GlyphRole, Surface, Variant};
use crate::ui::Ui;

pub use crate::author::PartStyle;

/// A replaced part: the component keeps layout, hit registration, focus and
/// state; the closure paints the part's rect.
pub type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

/// Paint the shared chrome of a layer-backed surface and run its content.
///
/// Placement, z-order, pointer barriers and focus trapping remain owned by
/// [`Ui::layer`](crate::Ui::layer). This helper owns only the repeated
/// container fill, typed frame, decorative registrations and surface scope;
/// it never invents a barrier or a second placement algorithm. The content
/// runs exactly once, clipped to the frame interior — an empty clip when
/// there is no drawable area — and surface or clipping scopes cannot escape
/// the call.
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

        // The body keeps the frame interior as its clip even when the frame
        // could not draw (`Ui::frame` returns `Rect::ZERO`), so a degenerate
        // body area stays anchored inside `area` instead of the origin.
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

/// The first row of `area`, or an empty rect.
pub const fn first_row(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: if area.height == 0 { 0 } else { 1 },
    }
}

/// A one-cell-wide column of `area` at `x`, or an empty rect when `x` is
/// outside `area`.
///
/// The bound is enforced on **both** sides. Callers anchor a cell to the right
/// edge with `area.right().saturating_sub(n)`, which lands left of `area.x`
/// whenever `area.width < n` — at `area.width == 1`, `right() - 2` is
/// `area.x - 1`. Checking only the right edge made that a paintable cell one
/// column outside the component, so the helper enforces the whole extent its
/// name promises rather than leaving each caller to clamp.
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

/// Paint the mono pressed bracket into two cells reserved by the component.
pub fn paint_pressed_bracket(ui: &mut Ui<'_>, left: Rect, right: Rect, style: PaintStyle) {
    ui.glyph(left, GlyphRole::PressLeft, style);
    ui.glyph(right, GlyphRole::PressRight, style);
}

/// `area` shifted right by `by` columns, shrinking its width.
pub const fn shift(area: Rect, by: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(by),
        y: area.y,
        width: area.width.saturating_sub(by),
        height: area.height,
    }
}

/// Folds a component's per-intent outcomes into one `Response<A>`: an
/// action wins over a repaint, a repaint over a bare consume.
pub struct Acc<A> {
    consumed: bool,
    invalidate: crate::response::Invalidate,
    action: Option<A>,
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

    /// Request a repaint **without** consuming: a notification the component
    /// drains and reacts to, but which must not swallow the input that is
    /// still being dispatched.
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

    /// `cell_at` names a cell **of `area`**, so it must yield a zero-width
    /// rect for every `x` outside `area`'s horizontal extent — to the left of
    /// `area.x` exactly as much as at or past `area.x + area.width`. Only the
    /// right edge was enforced, so a caller computing a right-anchored column
    /// with `saturating_sub` (`Select`'s trailing indicator,
    /// `cell_at(area, area.right() - 2)` at `area.width == 1`) got a paintable
    /// cell one column outside the component.
    #[test]
    fn cell_at_is_empty_for_every_x_outside_the_area_on_either_side() {
        // left of `area.x`: empty, both one short and far short
        assert_eq!(cell_at(AREA, AREA.x - 1).width, 0);
        assert_eq!(cell_at(AREA, 0).width, 0);
        // the `Select` case verbatim: a 1-wide area's `right() - 2` is `x - 1`
        let narrow = Rect { width: 1, ..AREA };
        assert_eq!(cell_at(narrow, narrow.right().saturating_sub(2)).width, 0);
        // a zero-width area has no cells at all, including at its own `x`
        let empty = Rect { width: 0, ..AREA };
        assert_eq!(cell_at(empty, empty.x).width, 0);

        // unchanged: every in-range `x` is a one-cell column of `area`
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

        // unchanged: at or past the right edge is empty, and keeps `x`, `y`
        // and `height` as they were
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
