//! `HintBar` — the one key-hint surface a shell owns
//! (`COMPONENT_ARCHITECTURE.md` §13.1, §18.2, Appendix A 4G).
//!
//! Components derive chord hints from `const` [`Binding`](crate::keymap::Binding)
//! tables through `HintLayer::from_bindings`. Screens can contribute product
//! affordances, including descriptive keycaps such as `Type`, without creating
//! routing bindings. [`HintBar::resolve`] picks the topmost contributing layer.

use core::fmt;

use ratatui_core::layout::Rect;

use super::keyhint::KeyHint;
use super::{PartStyle, SlotFn, first_row, shift};
use crate::collection::Status;
use crate::id::{Id, Part};
use crate::keymap::HintLayer;
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{ColorLevel, Family, GlyphRole, Slot, StateRule, StylePatch, Theme, Variant};
use crate::ui::{FrameRead, Ui};

#[derive(Clone, Copy, Debug, Default)]
enum MetadataOverride<'a> {
    #[default]
    Inherit,
    Set(Option<&'a str>),
}

impl<'a> MetadataOverride<'a> {
    fn resolve(self, inherited: Option<&'a str>) -> Option<&'a str> {
        match self {
            Self::Inherit => inherited,
            Self::Set(value) => value,
        }
    }
}

/// The bottom row of hints: a badge, the key hints that fit, and a status
/// message pinned to the right edge.
///
/// ## Construction
/// `HintBar::new(id, layer)` over the [`HintLayer`] the screen resolved.
/// [`HintBar::resolve`] is the associated function that performs that
/// resolution: topmost layer wins.
///
/// ## Ownership
/// Stateless. The caller owns the [`HintLayer`] — which it typically caches
/// behind `(focus_id, StateFlags, top_layer)` in `Ui::cache`, so an
/// unchanged focus costs no allocation per frame (§13.1) — and the animation
/// frame. The runtime owns nothing.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.status(Status)`
/// (`Ready`), `.frame(usize)` (`0`), `.patch`, `.patch_part`, `.slot`,
/// Default centring comes from the selected layer (`HintLayer::centered`).
/// [`DerivedHintBar::centered`] can override it after context selection.
///
/// ## Variants
/// `Family::HINTBAR`; `DEFAULT` only. The nested key hints resolve under
/// `Family::KEYHINT`, so a theme restyles chords once for the bar and for
/// any one-off hint chip.
///
/// ## States
/// Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)`; wears no runtime
/// state, because the bar is chrome and never takes focus, hover or press.
/// An errored bar leads its status message with the error glyph and a busy
/// one with a spinner frame, which is what keeps the three apart once colour
/// is removed (§11.4).
///
/// ## Actions
/// None; `HintBar` has no `update` phase. A hint is a *label* for a chord
/// another component owns; making the label clickable would put a second
/// dispatch path beside the binding table §13.1 exists to be the only one.
///
/// ## Focus
/// Never a focus stop; registers no ring entry and no region.
///
/// ## Keyboard
/// None of its own — every chord it shows belongs to the component that
/// declared it.
///
/// ## Mouse
/// None.
///
/// ## Layout
/// `measure` returns `(badge + every hint + the status, 1)`. `draw` uses the
/// first row of `area`, drops hints **from the right** when they do not fit
/// and marks the cut with `GlyphRole::Ellipsis`, and returns the rect it
/// painted; a degenerate rect paints nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the row fill), `BADGE` (the leading mode badge), `KEY` and
/// `ACTION` (each hint, through `KeyHint`), `LABEL` (the status message),
/// `MARKER` (the status glyph), `ICON` (the readiness spinner),
/// `OVERFLOW` (the `…` that marks dropped hints).
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part, forwarded to the nested
/// [`KeyHint`]s, so patching `KEY` on the bar restyles every chord it draws.
/// `.slot` on exactly `BADGE`, `KEY`, `ACTION`, `LABEL`, `MARKER`, `ICON`
/// and `OVERFLOW`; `KEY` and `ACTION` are forwarded to the nested
/// [`KeyHint`]s on the same reasoning as the patches. `CONTAINER` is not
/// slot-addressable: its fill *is* the bar.
///
/// ## Identity
/// One `Id` per instance; the hints are positional and carry no `ItemKey`,
/// because nothing addresses an individual hint.
///
/// ## Testing
/// `HintBarCase` in `crates/tui/tests/conformance.rs`, declaring
/// `Caps::REPORTS_STATUS`. Its mono states retain the default, `ERROR` and
/// `BUSY` renderings required by that readiness capability.
///
/// The render matrix in `crates/tui/tests/render_components.rs` generates
/// exactly eight cells per component, one per `St` variant: there is no
/// `render::components::hint_bar::busy`, no `::error` and no `::overflow`.
/// Readiness arrives through the matrix's `status_for` mapping —
/// `::pressed` is `Status::Busy`, `::editing` `Status::Loading`,
/// `::disabled` `Status::Error`. `busy` reads `self.status`, so the spinner
/// really does lead the status message in the first two, and is pinned as a
/// digest there.
///
/// The layer resolution and the right-hand drop are unit-tested in this
/// module by `the_topmost_layer_wins_and_the_fallback_is_none` and
/// `narrow_rows_drop_hints_from_the_right`, which calls `fitting` directly.
///
/// Exercised by no test: the painted `…` — with the matrix's two-hint
/// fixture, `fitting` keeps both hints at 40 and at 120 columns whether or
/// not a readiness glyph takes its two cells, so no matrix cell reaches the
/// `Part::OVERFLOW` branch, and the unit test stops at `fitting`'s counts.
///
/// The error glyph **is** painted from the semantic [`Status::Error`] state.
///
/// ## Invariants
/// Overflow drops from the **right** and always leaves the marker, so the
/// operator can see there is more; the status message keeps the right edge
/// and wins the space it needs. Never allocates: chords are rendered into a
/// fixed stack buffer by `KeyHint`.
pub struct HintBar<'a> {
    id: Id,
    layer: &'a HintLayer,
    badge_override: MetadataOverride<'a>,
    status_text_override: MetadataOverride<'a>,
    centered_override: Option<bool>,
    variant: Variant,
    status: Status,
    // `Status` has no warning variant. Unit tests OR `WARNING` in so `draw`
    // paints that path; production leaves the set empty.
    #[cfg(test)]
    flags: StateFlags,
    frame: usize,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

fn marker_states(part: Option<&crate::theme::PartRecipe>) -> &[StateRule] {
    match part {
        Some(part) => part.states.as_slice(),
        None => &[],
    }
}

impl fmt::Debug for HintBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintBar")
            .field("id", &self.id)
            .field("hints", &self.layer.hints.len())
            .field("badge", &self.layer.badge)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> HintBar<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BADGE,
        Part::KEY,
        Part::ACTION,
        Part::LABEL,
        Part::MARKER,
        Part::ICON,
        Part::OVERFLOW,
    ];

    /// Cells between two hints.
    const HINT_GAP: u16 = 2;

    /// A bar showing `layer`.
    pub const fn new(id: Id, layer: &'a HintLayer) -> Self {
        HintBar {
            id,
            layer,
            badge_override: MetadataOverride::Inherit,
            status_text_override: MetadataOverride::Inherit,
            centered_override: None,
            variant: Variant::DEFAULT,
            status: Status::Ready,
            #[cfg(test)]
            flags: StateFlags::empty(),
            frame: 0,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    /// A bar that derives the focused component layer from the current frame.
    pub const fn derived(id: Id) -> DerivedHintBar<'a> {
        DerivedHintBar {
            id,
            top: None,
            mode: None,
            screen: None,
            global: None,
            badge: MetadataOverride::Inherit,
            status_text: MetadataOverride::Inherit,
            centered: None,
        }
    }

    /// The topmost layer that exists, ordered from the topmost context down
    /// to the global fallback: top layer ▸ temporary mode ▸ the focused
    /// component's visible bindings ▸ screen extras ▸ global fallback
    /// (§13.1).
    ///
    /// Borrowing rather than cloning is deliberate: resolution runs every
    /// frame, and the layer it selects is usually the one the caller already
    /// cached.
    #[must_use]
    pub fn resolve<'l>(layers: &[Option<&'l HintLayer>]) -> Option<&'l HintLayer> {
        layers.iter().flatten().copied().next()
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The layer being shown.
    pub const fn layer(&self) -> &'a HintLayer {
        self.layer
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// Data readiness of the surface the bar reports on.
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// The animation frame the spinner reads.
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    /// An instance patch over every part, the nested hints included.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part patches, forwarded to the nested hints.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    /// Readiness flags `draw` and `measure` share.
    fn derived_flags(&self) -> StateFlags {
        #[cfg(test)]
        {
            self.status.flags().union(self.flags)
        }
        #[cfg(not(test))]
        {
            self.status.flags()
        }
    }

    /// One hint, wearing this bar's overrides.
    ///
    /// `KEY` and `ACTION` are painted by the nested [`KeyHint`] and by
    /// nothing else, so the bar's slot on either is forwarded rather than
    /// dropped: `.slot(Part::KEY, …)` on the bar answers for every chord it
    /// draws, exactly as `.patch_part` on `KEY` already restyles them all
    /// (§45.3, Invariant R).
    fn hint(&self, i: usize) -> Option<KeyHint<'a>> {
        let h = self.layer.hints.get(i)?;
        let mut k = KeyHint::from_hint(self.id, h)
            .variant(self.variant)
            .patch_part(self.parts);
        if let Some(p) = self.patch {
            k = k.patch(p);
        }
        if let Some(f) = self.ov.slot_for(Part::KEY) {
            k = k.slot(Part::KEY, f);
        } else if let Some(f) = self.ov.slot_for(Part::ACTION) {
            k = k.slot(Part::ACTION, f);
        }
        Some(k)
    }

    /// Columns the badge occupies, padding included.
    fn badge_width(&self) -> u16 {
        self.badge_text()
            .filter(|b| !b.is_empty())
            .map_or(0, |b| width(b).saturating_add(2))
    }

    /// Columns the status message occupies, its glyph included.
    fn status_width(&self, ui: &Ui<'_>, live: StateFlags) -> u16 {
        let Some(s) = self.message().filter(|s| !s.is_empty()) else {
            return 0;
        };
        let glyph = self.status_glyph(ui, live);
        width(s).saturating_add(glyph.map_or(0, |g| width(g).saturating_add(1)))
    }

    fn badge_text(&self) -> Option<&str> {
        self.badge_override.resolve(self.layer.badge)
    }

    fn message(&self) -> Option<&str> {
        self.status_text_override
            .resolve(self.layer.status.as_deref())
    }

    /// The glyph slot `Part::MARKER` resolves under for **this instance**.
    ///
    /// [`Ui::resolve`] is §26 N2's `&self` measuring path: it stops at
    /// precedence 5, so an instance `.patch` or `.patch_part` reached this
    /// cell's *colour* — resolved through [`PartStyle::style`] — and could
    /// not reach its *glyph* (§45.5). Precedence 6 is applied here exactly
    /// as `theme::resolve::bind` applies it on the painting path, so the
    /// measuring and painting answers cannot diverge.
    fn marker_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Slot<GlyphRole> {
        let base = ui
            .resolve(Family::HINTBAR, self.variant, Part::MARKER, live)
            .glyph;
        self.ov
            .part_patch(Part::MARKER)
            .map_or(base, |p| p.glyph.over(base))
    }

    /// The glyph that leads the status message: the spinner while busy, the
    /// recipe's marker (or the error glyph) while in error.
    fn status_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if self.busy() {
            let frames = ui.design().motion.spinner_frames;
            return frames
                .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                .copied();
        }
        if live.contains(StateFlags::ERROR) {
            return match self.marker_glyph(ui, live) {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::Error)),
                Slot::Clear => None,
            };
        }
        if live.contains(StateFlags::WARNING) {
            return self.warning_glyph(ui, live);
        }
        None
    }

    /// The warning marker.
    ///
    /// The generic mono fallback rewrites every warning marker to the dirty
    /// bullet. A status warning keeps `▲` unless the theme or this instance
    /// sets or clears `Part::MARKER`. Busy still wins: callers have already
    /// returned the spinner.
    fn warning_glyph(&self, ui: &Ui<'_>, live: StateFlags) -> Option<&'static str> {
        if let Some(patch) = self.ov.part_patch(Part::MARKER) {
            match patch.glyph {
                Slot::Set(g) => return Some(ui.glyph_str(g)),
                Slot::Clear => return None,
                Slot::Inherit => {}
            }
        }
        let authored = self.theme_marker_slot(ui.theme_ref(), live);
        if authored.speaks() {
            return match authored {
                Slot::Set(g) => Some(ui.glyph_str(g)),
                Slot::Clear => None,
                Slot::Inherit => Some(ui.glyph_str(GlyphRole::WarningMark)),
            };
        }
        // No explicit glyph. `marker_glyph` still sees overlays, but a mono
        // `Dirty` here is the generic fallback, not an author Set.
        match self.marker_glyph(ui, live) {
            Slot::Set(GlyphRole::Dirty) if ui.theme_ref().capability.color == ColorLevel::Mono => {
                Some(ui.glyph_str(GlyphRole::WarningMark))
            }
            Slot::Set(g) => Some(ui.glyph_str(g)),
            Slot::Inherit => Some(ui.glyph_str(GlyphRole::WarningMark)),
            Slot::Clear => None,
        }
    }

    /// `Part::MARKER`'s glyph before the generic mono fallback.
    ///
    /// Family base, variant delta, state rules, then theme overrides. Mono
    /// rules are omitted on purpose: they are what rewrite `▲` to `•`.
    fn theme_marker_slot(&self, theme: &Theme, live: StateFlags) -> Slot<GlyphRole> {
        let recipes = &theme.recipes;
        let recipe = recipes.get_or_neutral(Family::HINTBAR);
        let variant = if self.variant == Variant::DEFAULT {
            recipe.default_variant
        } else {
            self.variant
        };
        let family = recipe.parts.get(Part::MARKER);
        let delta = recipe
            .variant(variant)
            .and_then(|parts| parts.get(Part::MARKER));
        let mut acc = StylePatch::new();
        if let Some(part) = family {
            acc = part.apply_base(acc);
        }
        if let Some(part) = delta {
            acc = part.apply_base(acc);
        }
        acc = crate::theme::recipe::merge_states(
            acc,
            marker_states(family),
            marker_states(delta),
            live,
        );
        for over in recipes.overrides() {
            if over.family != Family::HINTBAR {
                continue;
            }
            let applies = match over.variant {
                None => true,
                Some(candidate) => candidate == variant,
            };
            if applies && let Some(part) = over.parts.get(Part::MARKER) {
                acc = part.apply(acc, live);
            }
        }
        acc.glyph
    }

    /// How many hints fit in `budget` columns from `x`, and whether any were
    /// dropped. Two cells are reserved for the cut marker while more hints
    /// follow, so the marker never pushes a hint off the row it just fitted.
    fn fitting(&self, budget: u16) -> (usize, u16) {
        let n = self.layer.hints.len();
        let mut used = 0u16;
        let mut drawn = 0usize;
        for i in 0..n {
            let Some(w) = self.hint(i).map(|h| h.width()) else {
                break;
            };
            let reserve = if i.saturating_add(1) < n {
                Self::HINT_GAP
            } else {
                0
            };
            let need = used.saturating_add(w).saturating_add(reserve);
            if need > budget {
                break;
            }
            used = used.saturating_add(w).saturating_add(Self::HINT_GAP);
            drawn = drawn.saturating_add(1);
        }
        (drawn, used.saturating_sub(Self::HINT_GAP))
    }

    /// The draw phase; returns the rect painted.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the status, the badge, the hints and the cut marker"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        // runtime: none — the bar is chrome and registers no control of
        // its own; derived: the readiness the caller's `.status` declares
        let live = PartStyle::flags(StateFlags::empty(), self.derived_flags());
        let ov = self.ov;
        let id = self.id;
        let container = ov.style(ui, id, Family::HINTBAR, self.variant, Part::CONTAINER, live);
        ui.fill(area, container.style);

        // the status message keeps the right edge and wins the space
        let status_w = self.status_width(ui, live);
        let mut right_limit = area.right();
        if status_w > 0 && area.width > status_w.saturating_add(2) {
            let glyph = self.status_glyph(ui, live);
            let mut x = area.right().saturating_sub(status_w).saturating_sub(1);
            right_limit = x.saturating_sub(Self::HINT_GAP);
            if let Some(g) = glyph {
                // one cell, two parts: the spinner is `ICON` and the error or
                // warning marker is `MARKER`. The slot is consulted before
                // `spinner_frames` (§45.4) — a slot is substitution, not
                // suppression, so the cell it replaces keeps its columns.
                let part = if self.busy() {
                    Part::ICON
                } else {
                    Part::MARKER
                };
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                let used = if let Some(f) = ov.slot_for(part) {
                    f(ui, cell);
                    width(g).min(cell.width)
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, part, live);
                    ui.paint_str(cell, g, s.style)
                };
                let gap_x = x.saturating_add(used);
                x = gap_x.saturating_add(1);
            }
            if let Some(text) = self.message() {
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::LABEL) {
                    f(ui, cell);
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
                    ui.paint_str(cell, text, s.style);
                }
            }
        } else if status_w > 0 {
            // A status wider than the row still wins: tag `keyhint.rs`
            // truncates it to the content lane and right-aligns the cut,
            // which fills a narrow footer and leaves no room for hints.
            right_limit = self.paint_truncated_status(ui, area, live);
        }

        // the badge leads
        let mut x = area.x.saturating_add(1);
        let badge_w = self.badge_width();
        if badge_w > 0 && x.saturating_add(badge_w) <= right_limit {
            if let Some(b) = self.badge_text() {
                let cell = Rect {
                    x,
                    width: badge_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::BADGE) {
                    f(ui, cell);
                } else {
                    let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::BADGE, live);
                    ui.fill(cell, s.style);
                    ui.paint_str(shift(cell, 1), b, s.style);
                }
            }
            x = x.saturating_add(badge_w).saturating_add(Self::HINT_GAP);
        }

        let budget = right_limit.saturating_sub(x);
        let (drawn, used) = self.fitting(budget);
        if self.centered_override.unwrap_or(self.layer.centered) {
            // the block sits mid-row, never past the badge and never under
            // the status
            let free = area.width.saturating_sub(used);
            let mid = area.x.saturating_add(free / 2);
            x = mid.max(x).min(right_limit.saturating_sub(used).max(x));
        }
        for i in 0..drawn {
            let Some(h) = self.hint(i) else { break };
            let w = h.width();
            let cell = Rect {
                x,
                width: right_limit.saturating_sub(x).min(w),
                ..area
            };
            if cell.is_empty() {
                break;
            }
            h.draw(ui, cell);
            x = x.saturating_add(w).saturating_add(Self::HINT_GAP);
        }
        if drawn < self.layer.hints.len() && x < right_limit {
            let cell = Rect {
                x,
                width: right_limit.saturating_sub(x),
                ..area
            };
            if let Some(f) = ov.slot_for(Part::OVERFLOW) {
                f(ui, cell);
            } else {
                let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::OVERFLOW, live);
                match s.glyph {
                    Slot::Set(g) => {
                        ui.glyph(cell, g, s.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(cell, GlyphRole::Ellipsis, s.style);
                    }
                    Slot::Clear => {
                        ui.fill(cell, s.style);
                    }
                }
            }
        }
        area
    }

    /// Paint a status that does not fit in `area.width - 2`.
    ///
    /// The cut is right-aligned in the content lane. Nothing is written
    /// outside `area`. The returned limit is the right edge hints may use,
    /// so they drop instead of drawing under the status.
    fn paint_truncated_status(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) -> u16 {
        let glyph = self.status_glyph(ui, live);
        let run = match (glyph, self.message()) {
            (Some(g), Some(text)) => format!("{g} {text}"),
            (Some(g), None) => g.to_owned(),
            (None, Some(text)) => text.to_owned(),
            (None, None) => String::new(),
        };
        let text = crate::text::truncate(&run, area.width.saturating_sub(2));
        let w = width(&text);
        if w == 0 {
            return area.right();
        }
        let x = area.right().saturating_sub(w).saturating_sub(1);
        if x < area.x {
            return area.right();
        }
        let right_limit = x.saturating_sub(Self::HINT_GAP);
        let cell = Rect {
            x,
            width: area.right().saturating_sub(x),
            ..area
        };
        if cell.is_empty() {
            return area.right();
        }
        let ov = self.ov;
        let id = self.id;
        let label = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
        ui.paint_str(cell, &text, label.style);
        if let Some(g) = glyph.filter(|g| text.starts_with(g)) {
            let part = if self.busy() {
                Part::ICON
            } else {
                Part::MARKER
            };
            let mark = Rect {
                x,
                width: width(g).min(cell.width),
                ..area
            };
            if mark.is_empty() {
                return right_limit;
            }
            if let Some(f) = ov.slot_for(part) {
                f(ui, mark);
            } else {
                let marker = ov.style(ui, id, Family::HINTBAR, self.variant, part, live);
                ui.paint_str(mark, g, marker.style);
            }
        }
        right_limit
    }

    /// The natural size: one row wide enough for every hint.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let live = PartStyle::flags(StateFlags::empty(), self.derived_flags());
        let hints: u16 = (0..self.layer.hints.len())
            .filter_map(|i| self.hint(i))
            .fold(0u16, |acc, h| {
                acc.saturating_add(h.width()).saturating_add(Self::HINT_GAP)
            });
        let w = self
            .badge_width()
            .saturating_add(hints)
            .saturating_add(self.status_width(ui, live))
            .saturating_add(2);
        Size {
            min: (self.badge_width().saturating_add(2), 1),
            preferred: (w, 1),
        }
        .fit(c)
    }
}

/// Context layers around the runtime-derived focused-component hints.
#[derive(Clone, Copy, Debug)]
pub struct DerivedHintBar<'a> {
    id: Id,
    top: Option<&'a HintLayer>,
    mode: Option<&'a HintLayer>,
    screen: Option<&'a HintLayer>,
    global: Option<&'a HintLayer>,
    badge: MetadataOverride<'a>,
    status_text: MetadataOverride<'a>,
    centered: Option<bool>,
}

impl<'a> DerivedHintBar<'a> {
    /// Supply the active top-layer hints.
    #[must_use]
    pub const fn top(mut self, layer: &'a HintLayer) -> Self {
        self.top = Some(layer);
        self
    }

    /// Supply temporary mode hints.
    #[must_use]
    pub const fn mode(mut self, layer: &'a HintLayer) -> Self {
        self.mode = Some(layer);
        self
    }

    /// Supply screen-level extras.
    #[must_use]
    pub const fn screen(mut self, layer: &'a HintLayer) -> Self {
        self.screen = Some(layer);
        self
    }

    /// Supply the application-wide fallback.
    #[must_use]
    pub const fn global(mut self, layer: &'a HintLayer) -> Self {
        self.global = Some(layer);
        self
    }

    /// Override the selected context's status text with a borrowed message.
    /// `None` explicitly clears it; omitting this builder preserves it.
    /// Hint selection and the cached context layer remain unchanged.
    #[must_use]
    pub const fn status_text(mut self, text: Option<&'a str>) -> Self {
        self.status_text = MetadataOverride::Set(text);
        self
    }

    /// Override the selected context's badge with borrowed text.
    /// `None` explicitly clears it; omitting this builder preserves it.
    #[must_use]
    pub const fn badge(mut self, text: Option<&'a str>) -> Self {
        self.badge = MetadataOverride::Set(text);
        self
    }

    /// Override centering after selecting the active hint context.
    /// Omitting this builder preserves the selected layer's alignment.
    #[must_use]
    pub const fn centered(mut self, centered: bool) -> Self {
        self.centered = Some(centered);
        self
    }

    fn draw_layer(&self, ui: &mut Ui<'_>, area: Rect, layer: &HintLayer) -> Rect {
        let mut bar = HintBar::new(self.id, layer);
        bar.badge_override = self.badge;
        bar.status_text_override = self.status_text;
        bar.centered_override = self.centered;
        bar.draw(ui, area)
    }

    /// Draw the first nonempty layer in top, mode, focused, screen, global
    /// precedence order, then apply explicit badge, status text and alignment overrides.
    /// Nonempty explicit metadata also renders when no hint context exists.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if let Some(layer) = self.top.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(layer) = self.mode.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(rect) = ui
            .with_focused_hints(|ui, layer| {
                (!layer.is_empty()).then(|| self.draw_layer(ui, area, layer))
            })
            .flatten()
        {
            return rect;
        }
        if let Some(layer) = self.screen.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if let Some(layer) = self.global.filter(|layer| !layer.is_empty()) {
            return self.draw_layer(ui, area, layer);
        }
        if self
            .badge
            .resolve(None)
            .is_some_and(|text| !text.is_empty())
            || self
                .status_text
                .resolve(None)
                .is_some_and(|text| !text.is_empty())
        {
            return self.draw_layer(ui, area, &HintLayer::empty());
        }
        Rect {
            width: 0,
            height: 0,
            ..area
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Chord, KeyCode};
    use crate::keymap::Hint;
    use crate::theme::Theme;
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};
    use ratatui_core::buffer::Buffer;

    const AREA: Rect = Rect::new(0, 0, 40, 1);

    fn glyph(theme: &Theme, status: Status) -> Option<String> {
        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
        HintBar::new(Id::root("hintbar.warning"), &layer)
            .status(status)
            .status_glyph(&ui, status.flags())
            .map(str::to_owned)
    }

    fn layer(labels: &[(&'static str, KeyCode)]) -> HintLayer {
        HintLayer {
            hints: labels
                .iter()
                .map(|(l, c)| Hint {
                    key: crate::keymap::HintKey::Chord(Chord::key(*c)),
                    label: l,
                    priority: 50,
                })
                .collect(),
            ..HintLayer::empty()
        }
    }

    #[test]
    fn the_topmost_layer_wins_and_the_fallback_is_none() {
        let screen = layer(&[("Launch", KeyCode::Enter)]);
        let modal = layer(&[("Close", KeyCode::Esc)]);
        assert_eq!(
            HintBar::resolve(&[Some(&modal), None, Some(&screen)]).map(|l| l.hints.len()),
            Some(1)
        );
        // Identity, not equality, is the assertion. `resolve` returns one of the
        // caller's own borrows, so which slot it picked is provenance, not content:
        // `HintLayer: PartialEq`, and two layers carrying the same hints would be
        // `==` while coming from different slots, so `==` would pass even if the
        // wrong element were returned. Each case also falls back to the *other*
        // layer, so a `None` result fails rather than aliases the expected pointer.
        assert!(core::ptr::eq(
            HintBar::resolve(&[Some(&modal), None, Some(&screen)]).unwrap_or(&screen),
            &raw const modal
        ));
        assert!(core::ptr::eq(
            HintBar::resolve(&[None, None, Some(&screen)]).unwrap_or(&modal),
            &raw const screen
        ));
        assert!(HintBar::resolve(&[None, None]).is_none());
    }

    #[test]
    fn narrow_rows_drop_hints_from_the_right() {
        let l = layer(&[
            ("Open", KeyCode::Enter),
            ("Choose", KeyCode::Char(' ')),
            ("Next", KeyCode::Tab),
            ("Cancel", KeyCode::Esc),
        ]);
        let bar = HintBar::new(Id::root("t"), &l);
        let (all, _) = bar.fitting(200);
        assert_eq!(all, 4);
        let (few, used) = bar.fitting(24);
        assert!((1..4).contains(&few), "{few}");
        assert!(used <= 24, "{used}");
        assert_eq!(bar.fitting(0).0, 0);
    }

    #[test]
    fn inherited_warning_status_uses_warning_mark() {
        let theme = Theme::junie();
        assert_eq!(glyph(&theme, Status::Error).as_deref(), Some("!"));
        assert_eq!(
            glyph(&theme, Status::Ready).as_deref(),
            None,
            "ready status does not imply a glyph"
        );
        // No public Warning readiness status exists; test this path's semantic
        // flag directly through its private resolver below.
        let mut frame = FrameState::default();
        frame.reset(1, AREA);
        let mut page = Buffer::empty(AREA);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let layer = HintLayer::empty();
        let ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
        assert_eq!(
            HintBar::new(Id::root("hintbar.warning"), &layer)
                .status_glyph(&ui, StateFlags::WARNING)
                .map(str::to_owned)
                .as_deref(),
            Some("▲")
        );
    }

    fn with_warning(mut bar: HintBar<'_>) -> HintBar<'_> {
        bar.flags |= StateFlags::WARNING;
        bar
    }

    fn draw_page(theme: &Theme, screen: Rect, area: Rect, bar: &HintBar<'_>) -> Buffer {
        let mut frame = FrameState::default();
        frame.reset(1, screen);
        let mut page = Buffer::empty(screen);
        let untouched = page.clone();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
            bar.draw(&mut ui, area);
        }
        assert_painted_inside(&untouched, &page, screen, area);
        page
    }

    fn assert_painted_inside(before: &Buffer, after: &Buffer, screen: Rect, area: Rect) {
        for y in 0..screen.height {
            for x in 0..screen.width {
                let inside = area.height > 0
                    && area.width > 0
                    && y == area.y
                    && x >= area.x
                    && x < area.right();
                if inside {
                    continue;
                }
                let left = &before[(x, y)];
                let right = &after[(x, y)];
                assert_eq!(
                    (left.symbol(), left.fg, left.bg, left.modifier),
                    (right.symbol(), right.fg, right.bg, right.modifier),
                    "painted outside the area at ({x}, {y})"
                );
            }
        }
    }

    fn row_text(page: &Buffer, area: Rect) -> String {
        let mut out = String::new();
        if area.height == 0 || area.width == 0 {
            return out;
        }
        for x in area.x..area.right() {
            out.push_str(page[(x, area.y)].symbol());
        }
        out
    }

    fn theme_with_marker(glyph: Slot<GlyphRole>) -> Theme {
        Theme::junie().define_family(Family::HINTBAR, |edit| {
            edit.part(Part::MARKER).base(StylePatch {
                glyph,
                ..StylePatch::new()
            });
        })
    }

    fn choose_layer(status: &'static str) -> HintLayer {
        HintLayer {
            hints: vec![Hint {
                key: crate::keymap::HintKey::Chord(Chord::key(KeyCode::Enter)),
                label: "Choose",
                priority: 50,
            }],
            status: Some(status.into()),
            centered: true,
            ..HintLayer::empty()
        }
    }

    #[test]
    fn mono_warning_status_keeps_the_warning_mark() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let area = Rect::new(0, 0, 24, 1);
        let screen = Rect::new(0, 0, 24, 3);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let mut frame = FrameState::default();
        frame.reset(1, screen);
        let mut page = Buffer::empty(screen);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            assert_eq!(
                ui.resolve(
                    Family::HINTBAR,
                    Variant::DEFAULT,
                    Part::MARKER,
                    StateFlags::WARNING
                )
                .glyph,
                Slot::Set(GlyphRole::Dirty),
                "mono fallback still rewrites the resolved marker"
            );
        }
        let painted = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        let row = row_text(&painted, area);
        assert!(row.contains('▲'), "{row}");
        assert!(!row.contains('•'), "{row}");
    }

    #[test]
    fn a_status_wider_than_the_row_truncates_and_drops_hints() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 72, 1);
        let screen = Rect::new(0, 0, 72, 2);
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert_eq!(page[(1, 0)].symbol(), "▲");
        assert_eq!(page[(2, 0)].symbol(), " ");
        assert_eq!(page[(2, 0)].fg, theme.color.warning);
        assert_eq!(page[(70, 0)].symbol(), "…");
        assert_eq!(page[(27, 0)].fg, theme.color.warning);
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
    }

    #[test]
    fn narrow_widths_do_not_paint_outside_the_area() {
        let theme = Theme::junie();
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let screen = Rect::new(0, 0, 20, 6);
        for width in 0..=5 {
            let area = Rect::new(3, 2, width, 2);
            let page = draw_page(
                &theme,
                screen,
                area,
                &with_warning(HintBar::new(Id::root("hintbar.narrow"), &layer)),
            );
            let row = row_text(&page, area);
            assert!(!row.contains("Choose"), "width {width}: {row}");
        }
    }

    #[test]
    fn a_unicode_wide_status_truncates_inside_a_shifted_row() {
        let theme = Theme::junie();
        let status = "警告：无法确认运行实例";
        let area = Rect::new(4, 2, 11, 3);
        let screen = Rect::new(0, 0, 24, 7);
        let layer = choose_layer(status);
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.wide"), &layer)),
        );
        let expected = crate::text::truncate(&format!("▲ {status}"), area.width.saturating_sub(2));
        let w = width(&expected);
        assert!(w > 0, "{expected}");
        assert!(expected.contains('…'), "{expected}");
        let x0 = area.right().saturating_sub(w).saturating_sub(1);
        assert!(x0 >= area.x && x0 < area.right(), "{x0}");
        let mut col = x0;
        for (_, grapheme) in crate::text::measure::graphemes(&expected) {
            assert!(col < area.right(), "wide status spilled");
            assert_eq!(page[(col, area.y)].symbol(), grapheme, "at {col}");
            let columns = crate::text::measure::grapheme_width(grapheme);
            let mut step = 1u16;
            while step < columns {
                let next = col.saturating_add(step);
                assert!(next < area.right(), "wide glyph spilled at {next}");
                step = step.saturating_add(1);
            }
            col = col.saturating_add(columns);
        }
        assert!(col <= area.right());
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
        assert!(!row.contains('•'), "{row}");
    }

    #[test]
    fn truncated_status_respects_a_shifted_origin() {
        let theme = Theme::junie();
        let area = Rect::new(5, 3, 72, 2);
        let screen = Rect::new(0, 0, 90, 8);
        let layer = choose_layer(
            "Could not confirm running instances: instance index unreadable · entered without the ritual",
        );
        let page = draw_page(
            &theme,
            screen,
            area,
            &with_warning(HintBar::new(Id::root("hintbar.shift"), &layer)),
        );
        assert_eq!(page[(6, 3)].symbol(), "▲");
        assert_eq!(page[(7, 3)].fg, theme.color.warning);
        assert_eq!(page[(75, 3)].symbol(), "…");
        let row = row_text(&page, area);
        assert!(!row.contains("Choose"), "{row}");
    }

    #[test]
    fn custom_theme_marker_set_clear_and_inherit() {
        let screen = Rect::new(0, 0, 32, 4);
        let area = Rect::new(2, 1, 24, 1);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let paint = |theme: &Theme, bar: HintBar<'_>| -> String {
            row_text(&draw_page(theme, screen, area, &bar), area)
        };
        let set = theme_with_marker(Slot::Set(GlyphRole::Error)).downgrade(ColorLevel::Mono);
        let clear = theme_with_marker(Slot::Clear).downgrade(ColorLevel::Mono);
        let inherit = theme_with_marker(Slot::Inherit).downgrade(ColorLevel::Mono);
        let mono = Theme::junie().downgrade(ColorLevel::Mono);
        let chosen = [(Part::MARKER, StylePatch::new().set_glyph(GlyphRole::Chosen))];
        let cleared = [(
            Part::MARKER,
            StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        )];
        let inherited = [(Part::MARKER, StylePatch::new())];

        let row = paint(
            &set,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains('!'), "theme Set: {row}");
        assert!(!row.contains('▲'), "theme Set: {row}");
        assert!(!row.contains('•'), "theme Set: {row}");

        let row = paint(
            &clear,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains("Attention"), "theme Clear: {row}");
        assert!(!row.contains('▲'), "theme Clear: {row}");
        assert!(!row.contains('•'), "theme Clear: {row}");
        assert!(!row.contains('!'), "theme Clear: {row}");

        let row = paint(
            &inherit,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer)),
        );
        assert!(row.contains('▲'), "theme Inherit: {row}");
        assert!(!row.contains('•'), "theme Inherit: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&chosen)),
        );
        assert!(row.contains('›'), "instance Set: {row}");
        assert!(!row.contains('▲'), "instance Set: {row}");
        assert!(!row.contains('•'), "instance Set: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&cleared)),
        );
        assert!(row.contains("Attention"), "instance Clear: {row}");
        assert!(!row.contains('▲'), "instance Clear: {row}");
        assert!(!row.contains('•'), "instance Clear: {row}");
        assert!(!row.contains('›'), "instance Clear: {row}");

        let row = paint(
            &mono,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&inherited)),
        );
        assert!(row.contains('▲'), "instance Inherit: {row}");
        assert!(!row.contains('•'), "instance Inherit: {row}");

        let row = paint(
            &set,
            with_warning(HintBar::new(Id::root("hintbar.warning"), &layer).patch_part(&chosen)),
        );
        assert!(row.contains('›'), "instance Set beats theme Set: {row}");
        assert!(!row.contains('!'), "instance Set beats theme Set: {row}");
    }

    #[test]
    fn busy_status_keeps_the_spinner_ahead_of_a_warning() {
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let spinner = theme
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        let area = Rect::new(1, 1, 24, 1);
        let screen = Rect::new(0, 0, 28, 4);
        let layer = HintLayer {
            status: Some("Attention".into()),
            ..HintLayer::empty()
        };
        let mut bar = HintBar::new(Id::root("hintbar.warning"), &layer).status(Status::Busy);
        bar.flags |= StateFlags::WARNING;
        let page = draw_page(&theme, screen, area, &bar);
        let row = row_text(&page, area);
        assert!(row.contains(spinner), "{row}");
        assert!(!row.contains('▲'), "{row}");
        assert!(!row.contains('•'), "{row}");
    }
}
