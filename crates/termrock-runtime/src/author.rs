//! Component-author API and shared helpers.

use core::fmt;
use ratatui_core::layout::Rect;
use termrock_core::id::{Id, Part, PartRef};
use termrock_core::response::{Invalidate, Response, StateFlags};
use termrock_theme::{Family, GlyphRole, PaintStyle, Resolved, StylePatch, Surface, Variant};

use crate::ui::Ui;

/// Borrowed replacement painter for one component part.
pub type PartPainter<'a> = dyn Fn(&mut Ui<'_>, Rect) + 'a;

/// A replaced part: the component keeps layout, hit registration, focus and
/// state; the closure paints the part's rect.
pub type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

/// Borrowed per-instance styling and painting overrides.
#[derive(Clone, Copy)]
pub struct PartStyle<'a> {
    pub patch: Option<&'a StylePatch>,
    pub parts: &'a [(Part, StylePatch)],
    pub slot: Option<(Part, &'a PartPainter<'a>)>,
}

impl fmt::Debug for PartStyle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PartStyle")
            .field("patch", &self.patch)
            .field("parts", &self.parts.len())
            .field("slot", &self.slot.map(|(part, _)| part))
            .finish()
    }
}

impl<'a> PartStyle<'a> {
    /// An empty override set.
    pub const fn new() -> Self {
        Self {
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    /// Apply `patch` to every part this instance resolves.
    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Apply the matching patches to their named parts in declaration order.
    #[must_use]
    pub const fn part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self
    }

    /// Replace the painter for `part` while preserving the component's geometry
    /// and interaction registrations.
    #[must_use]
    pub const fn slot(mut self, part: Part, painter: &'a PartPainter<'a>) -> Self {
        self.slot = Some((part, painter));
        self
    }

    /// Combine runtime-owned and component-derived state flags.
    #[must_use]
    pub const fn flags(runtime: StateFlags, derived: StateFlags) -> StateFlags {
        runtime.union(derived)
    }

    /// Return the merged patch for `part`, if any patch applies.
    pub fn part_patch(&self, part: Part) -> Option<StylePatch> {
        let mut merged = self.patch.copied();
        for (named, patch) in self.parts {
            if *named == part {
                merged = Some(match merged {
                    Some(base) => base.merge(*patch),
                    None => *patch,
                });
            }
        }
        merged
    }

    /// Return the replacement painter for `part`, if one was configured.
    pub fn slot_for(&self, part: Part) -> Option<&'a PartPainter<'a>> {
        match self.slot {
            Some((named, painter)) if named == part => Some(painter),
            _ => None,
        }
    }

    /// Resolve one part through theme resolution and this instance's patches.
    pub fn style(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        let resolved = match self.part_patch(part) {
            Some(patch) => ui.style_patched(family, variant, part, flags, &patch),
            None => ui.style(family, variant, part, flags),
        };
        self.note(ui, owner, family, variant, part, resolved);
        resolved
    }

    /// Record a resolved part in testing builds; this is a no-op otherwise.
    pub fn note(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        resolved: Resolved,
    ) {
        #[cfg(feature = "testing")]
        ui.note_styled(owner, family, variant, part, resolved);
        #[cfg(not(feature = "testing"))]
        let _ = (ui, owner, family, variant, part, resolved);
    }
}

impl Default for PartStyle<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Paint the shared chrome of a layer-backed surface and run its content.
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
    invalidate: Invalidate,
    action: Option<A>,
}

impl<A> Acc<A> {
    pub const fn new() -> Self {
        Acc {
            consumed: false,
            invalidate: Invalidate::None,
            action: None,
        }
    }

    pub fn consumed(&mut self) {
        self.consumed = true;
    }

    pub fn changed(&mut self) {
        self.consumed = true;
        self.invalidate = self.invalidate.max(Invalidate::Paint);
    }

    /// Request a repaint **without** consuming: a notification the component
    /// drains and reacts to, but which must not swallow the input that is
    /// still being dispatched.
    pub fn repaint(&mut self) {
        self.invalidate = self.invalidate.max(Invalidate::Paint);
    }

    pub fn action(&mut self, a: A) {
        self.changed();
        self.action = Some(a);
    }

    pub fn fold(&mut self, r: &Response<()>) {
        self.consumed |= r.is_consumed();
        self.invalidate = self.invalidate.max(r.invalidate());
    }

    pub fn finish(self, id: Id) -> Response<A> {
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

impl<A> Default for Acc<A> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
