//! `Empty` — the standalone empty / loading / partial / error surface
//! (`COMPONENT_ARCHITECTURE.md` §12.2, §18.2, Appendix A 4G).

use core::fmt;

use ratatui_core::layout::Rect;

use super::{Overrides, SlotFn};
use crate::collection::EmptyState;
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::{width, wrapped_rows};
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::Ui;

/// A pane with nothing to show: a quiet title, an optional hint, never a big
/// glyph.
///
/// ## Construction
/// `Empty::new(id, state)` over the shared [`EmptyState`] vocabulary, so a
/// screen-level empty pane and a collection's own empty slot say the same
/// four things in the same words (§12.2).
///
/// ## Ownership
/// Stateless. The caller owns the [`EmptyState`] and the strings it borrows,
/// and supplies the animation frame; the runtime owns nothing.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.frame(usize)`
/// (`0` — the spinner frame, a prop so a digest is a pure function of the
/// props), `.patch`, `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::EMPTY`; `DEFAULT` only.
///
/// ## States
/// Derives `BUSY`/`LOADING`/`ERROR` from the [`EmptyState`] variant through
/// [`EmptyState::status`]; wears no runtime state, because it is never
/// focused, hovered, pressed or disabled.
///
/// ## Actions
/// None; `Empty` has no `update` phase. A retry affordance is a `Button` the
/// owning screen places beside it.
///
/// ## Focus
/// Never a focus stop; registers no ring entry.
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None; no `PartRef` is registered.
///
/// ## Layout
/// `measure` returns `(the widest line, 1 or 3)` — one row for the title,
/// plus a blank row and the wrapped detail when there is one. `draw`
/// centres the block in `area` and returns the rect it used; a degenerate
/// rect paints nothing (R5).
///
/// ## Parts
/// `EMPTY` (the whole slot; the part a container reserves for it, filled on
/// every non-degenerate frame), then the three parts [`EmptyState::draw`]
/// paints under `Family::EMPTY` — the one renderer every collection shares:
/// `TITLE` (the primary line, on every frame), `HELP` (the wrapped detail,
/// only when the state carries one and the block has three rows) and `ICON`
/// (the readiness glyph: the spinner frame for `Loading`/`Partial`, the error
/// glyph for `Error`, nothing for `Empty`). `EMPTY` is first because it is
/// the only one painted unconditionally, and `PARTS[0]` is what §16.2 case 10
/// patches when it asserts an instance patch reaches the surface.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `EMPTY`; `TITLE`, `HELP` and `ICON` are
/// resolved by the shared [`EmptyState::draw`] straight from the theme, so a
/// per-part patch aimed at them is not honoured. `.slot(Part::EMPTY, …)`
/// replaces the whole surface, which is the documented way to put an
/// illustration or an action row where the default text goes.
///
/// ## Identity
/// One `Id` per instance, used to attribute style resolution and overrides;
/// no items.
///
/// ## Testing
/// `EmptyCase` with no capabilities;
/// `collection::empty::tests::empty_state_covers_empty_loading_partial_error`;
/// the render matrix names states, not readiness, so the four surfaces are
/// `render::components::empty::{default, editing, disabled, empty}` — the
/// matrix maps `editing` onto `Loading` and `disabled` onto `Error`.
///
/// ## Invariants
/// The readiness glyph is a *symbol* — the spinner frame for
/// `Loading`/`Partial`, the error glyph for `Error` — so the four states stay
/// distinguishable with colour removed (§11.4, §16.2 case 9). Never writes
/// outside `area`.
pub struct Empty<'a> {
    id: Id,
    state: EmptyState<'a>,
    variant: Variant,
    frame: usize,
    ov: Overrides<'a>,
}

impl fmt::Debug for Empty<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("frame", &self.frame)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Empty<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::EMPTY, Part::TITLE, Part::HELP, Part::ICON];

    /// The surface for `state`.
    pub const fn new(id: Id, state: EmptyState<'a>) -> Self {
        Empty {
            id,
            state,
            variant: Variant::DEFAULT,
            frame: 0,
            ov: Overrides::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The data state being reported.
    pub const fn state(&self) -> EmptyState<'a> {
        self.state
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// The animation frame the spinner reads. A prop, never a clock read:
    /// two draws with the same props are byte-identical (§16.2 case 5).
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    /// An instance patch over every part (precedence 6).
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace the whole surface.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11): render a forced state.
    #[must_use]
    pub const fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    /// The draw phase; returns the rect the block occupies.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        let live = self.ov.flags(self.state.status().flags());
        let ov = self.ov;
        if let Some(f) = ov.slot_for(Part::EMPTY) {
            f(ui, area);
            return area;
        }
        let slot = ov.style(ui, self.id, Family::EMPTY, self.variant, Part::EMPTY, live);
        ui.fill(area, slot.style);
        let rows = self.rows(area.width);
        let top = area.y.saturating_add(area.height.saturating_sub(rows) / 2);
        let block = Rect {
            y: top,
            height: rows.min(area.height),
            ..area
        };
        let used = self.state.draw(ui, block, self.frame);
        Rect {
            height: used,
            ..block
        }
    }

    /// Rows the block needs at `w` columns.
    fn rows(&self, w: u16) -> u16 {
        match self.state.detail() {
            Some(d) => wrapped_rows(d, w.max(1)).saturating_add(2),
            None => 1,
        }
    }

    /// The natural size: the widest line, and one or three rows.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let title = width(self.state.title());
        let detail = self.state.detail().map_or(0, width);
        let w = title.max(detail);
        Size {
            min: (title.min(c.max.0), 1),
            preferred: (w, self.rows(c.max.0.max(1))),
        }
        .fit(c)
    }
}
