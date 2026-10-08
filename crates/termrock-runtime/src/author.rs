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
///
/// ## Override precedence
///
/// For a resolved part `P`, weakest to strongest:
///
/// 1. The theme chain (§11.3): family recipe, variant, state flags, live
///    overlays, surface binding.
/// 2. The global patch ([`PartStyle::global`]): one [`StylePatch`] applied
///    to every part this instance resolves.
/// 3. The per-part patches ([`PartStyle::part`]), applied in declaration
///    order: each entry matching `P` merges over the previous result, so a
///    later entry wins where it speaks ([`StylePatch::merge`]).
///
/// Layers 2 and 3 are the §11.3 instance layer (precedence 6); a part with
/// no matching override resolves exactly as if the instance carried none.
/// [`PartStyle::slot`] replaces painting for exactly one part and never
/// changes resolution precedence.
///
/// ## Declared parts (R8)
///
/// A component declares its contract once with [`PartStyle::declare`]
/// (its `PARTS` constant, before `part()`/`slot()`). A patch or slot
/// naming any other part is rejected: it fails a `debug_assert` at the
/// builder, and in testing builds it is recorded as an `UnknownPart`
/// diagnostic when the component resolves styles. Slot *eligibility* is
/// narrower than declaration — each component documents which declared
/// parts accept a slot — and an ineligible-but-declared slot keeps its
/// silently-ignored behavior.
#[derive(Clone, Copy)]
pub struct PartStyle<'a> {
    pub patch: Option<&'a StylePatch>,
    pub parts: &'a [(Part, StylePatch)],
    pub slot: Option<(Part, &'a PartPainter<'a>)>,
    declared: Option<&'a [Part]>,
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
            declared: None,
        }
    }

    /// Declare the component's `PARTS` contract: the only parts `part()`
    /// and `slot()` may name. Call this before either builder; a component
    /// that receives forwarded patch arrays (an embedded scrollbar, a
    /// viewport, an inner list) must stay undeclared, because the producer's
    /// parts are legitimately outside its own contract.
    #[must_use]
    pub const fn declare(mut self, declared: &'a [Part]) -> Self {
        let slot_ok = match self.slot {
            Some((part, _)) => is_declared(declared, part),
            None => true,
        };
        debug_assert!(
            patches_are_declared(declared, self.parts) && slot_ok,
            "PartStyle::declare: an override names a part outside the component's PARTS",
        );
        self.declared = Some(declared);
        self
    }

    /// Apply `patch` to every part this instance resolves.
    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Apply the matching patches to their named parts in declaration order.
    ///
    /// Every named part must be a member of the declared contract (see
    /// [`PartStyle::declare`]); an unknown part fails a `debug_assert`, and
    /// in testing builds it is recorded as an `UnknownPart` diagnostic at
    /// resolve time instead of being applied.
    #[must_use]
    pub const fn part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        debug_assert!(
            patches_are_known(self.declared, patches),
            "PartStyle::part: a patch names a part outside the component's PARTS",
        );
        self.parts = patches;
        self
    }

    /// Replace the painter for `part` while preserving the component's geometry
    /// and interaction registrations.
    ///
    /// `part` must be a member of the declared contract (see
    /// [`PartStyle::declare`]); an unknown part fails a `debug_assert`, and
    /// in testing builds it is recorded as an `UnknownPart` diagnostic at
    /// resolve time instead of being installed.
    #[must_use]
    pub const fn slot(mut self, part: Part, painter: &'a PartPainter<'a>) -> Self {
        debug_assert!(
            slot_is_known(self.declared, part),
            "PartStyle::slot: the slot names a part outside the component's PARTS",
        );
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
        self.note_rejected(ui, owner);
        resolved
    }

    /// Resolve one part with a component default under this instance's patches.
    ///
    /// The default voices only what the instance leaves unsaid: an explicit
    /// `.patch()`/`.patch_part()` wins every slot it speaks on, exactly as
    /// with [`PartStyle::style`]. Components use this when the historical
    /// widget paints a fixed tone the shared recipe cannot carry without
    /// breaking the recipe's other consumers (a slot edit would leak; a
    /// component default stays inside the component).
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors PartStyle::style's explicit resolve channels plus the default"
    )]
    pub fn style_with_default(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        default: &StylePatch,
    ) -> Resolved {
        let patch = match self.part_patch(part) {
            Some(instance) => default.merge(instance),
            None => *default,
        };
        let resolved = ui.style_patched(family, variant, part, flags, &patch);
        self.note(ui, owner, family, variant, part, resolved);
        self.note_rejected(ui, owner);
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

    /// Record every named-but-undeclared override as an `UnknownPart`
    /// diagnostic in testing builds; a no-op otherwise. `style()` calls
    /// this once per resolved part and `Ui` dedupes per frame, so one draw
    /// reports each unknown part exactly once however many parts the
    /// component resolves.
    fn note_rejected(&self, ui: &mut Ui<'_>, owner: Id) {
        #[cfg(feature = "testing")]
        {
            let Some(declared) = self.declared else {
                return;
            };
            for (part, _) in self.parts {
                if !declared.contains(part) {
                    ui.note_unknown_part(owner, *part);
                }
            }
            if let Some((part, _)) = self.slot
                && !declared.contains(&part)
            {
                ui.note_unknown_part(owner, part);
            }
        }
        #[cfg(not(feature = "testing"))]
        let _ = (ui, owner);
    }
}

/// Const membership: `PartialEq` is not const, so parts compare by number.
const fn is_declared(declared: &[Part], part: Part) -> bool {
    match declared {
        [] => false,
        [head, tail @ ..] => head.raw() == part.raw() || is_declared(tail, part),
    }
}

/// Const conjunction over a patch array.
const fn patches_are_declared(declared: &[Part], patches: &[(Part, StylePatch)]) -> bool {
    match patches {
        [] => true,
        [(part, _), tail @ ..] => {
            is_declared(declared, *part) && patches_are_declared(declared, tail)
        }
    }
}

/// An undeclared override set skips validation (see [`PartStyle::declare`]).
const fn patches_are_known(declared: Option<&[Part]>, patches: &[(Part, StylePatch)]) -> bool {
    match declared {
        None => true,
        Some(known) => patches_are_declared(known, patches),
    }
}

/// An undeclared override set skips validation (see [`PartStyle::declare`]).
const fn slot_is_known(declared: Option<&[Part]>, part: Part) -> bool {
    match declared {
        None => true,
        Some(known) => is_declared(known, part),
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
    use ratatui_core::buffer::Buffer;
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::diagnostics::Diagnostic;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const AREA: Rect = Rect {
        x: 4,
        y: 5,
        width: 3,
        height: 2,
    };

    const OWNER: Id = Id::root("author.tests.unknown-part");
    const DECLARED: &[Part] = &[Part::CONTAINER, Part::BORDER];

    /// Paint shared chrome under `ov` and return the buffer plus the frame's
    /// diagnostics. Chrome resolves exactly the declared parts.
    fn paint_chrome(ov: PartStyle<'_>) -> (Buffer, Vec<Diagnostic>) {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                overlay_chrome(
                    ui,
                    OWNER,
                    area,
                    Family::DIALOG,
                    Surface::Elevated,
                    ov,
                    StateFlags::empty(),
                    StateFlags::empty(),
                    |_, _| {},
                );
            })
            .commit_presented();
        (buffer, runtime.diagnostics().to_vec())
    }

    /// R8: a patch on an undeclared part is recorded once per frame and
    /// changes nothing on screen. The override set is built literally to
    /// bypass `part()`, whose `debug_assert` fires first by design.
    #[test]
    fn style_reports_an_undeclared_patch_part_once_and_ignores_it() {
        let (plain_buf, plain_diags) = paint_chrome(PartStyle::new().declare(DECLARED));
        assert!(plain_diags.is_empty());

        let bold = StylePatch::new().add(Modifier::BOLD);
        let patches = [(Part::TITLE, bold)];
        let rejected = PartStyle {
            patch: None,
            parts: &patches,
            slot: None,
            declared: Some(DECLARED),
        };
        let (bad_buf, bad_diags) = paint_chrome(rejected);
        assert_eq!(bad_buf, plain_buf, "an undeclared patch must not paint");
        assert_eq!(
            bad_diags,
            &[Diagnostic::UnknownPart {
                owner: OWNER,
                part: Part::TITLE
            }],
            "one frame reports each unknown part exactly once",
        );
    }

    /// R8: a slot on an undeclared part is recorded and never installed.
    #[test]
    fn style_reports_an_undeclared_slot_part_and_ignores_it() {
        let (plain_buf, _) = paint_chrome(PartStyle::new().declare(DECLARED));

        let paint = |_ui: &mut Ui<'_>, _area: Rect| {};
        let rejected = PartStyle {
            patch: None,
            parts: &[],
            slot: Some((Part::TITLE, &paint)),
            declared: Some(DECLARED),
        };
        let (bad_buf, bad_diags) = paint_chrome(rejected);
        assert_eq!(bad_buf, plain_buf, "an undeclared slot must not paint");
        assert_eq!(
            bad_diags,
            &[Diagnostic::UnknownPart {
                owner: OWNER,
                part: Part::TITLE
            }],
        );
    }

    /// R8: validation only runs against a declared contract; an undeclared
    /// override set (a forwarded patch array) keeps its silent behavior.
    #[test]
    fn undeclared_override_sets_skip_validation() {
        let bold = StylePatch::new().add(Modifier::BOLD);
        let patches = [(Part::TITLE, bold)];
        let paint = |_ui: &mut Ui<'_>, _area: Rect| {};
        let forwarded = PartStyle::new().part(&patches).slot(Part::TITLE, &paint);
        let (buf, diags) = paint_chrome(forwarded);
        let (plain_buf, _) = paint_chrome(PartStyle::new());
        assert_eq!(buf, plain_buf);
        assert!(diags.is_empty());
    }

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
