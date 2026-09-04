//! `HintBar` — the one key-hint surface a shell owns
//! (`COMPONENT_ARCHITECTURE.md` §13.1, §18.2, Appendix A 4G).
//!
//! Hints are **derived**, never hand-written: a component publishes a
//! `const` [`Binding`](crate::keymap::Binding) table, `HintLayer::from_bindings`
//! turns its visible entries into a layer ordered by priority, and
//! [`HintBar::resolve`] picks the topmost layer that exists. A screen
//! contributes product-level extras only.

use core::fmt;

use ratatui_core::layout::Rect;

use super::keyhint::KeyHint;
use super::{Overrides, SlotFn, first_row, shift};
use crate::collection::Status;
use crate::id::{Id, Part};
use crate::keymap::HintLayer;
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

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
/// `.state_override`. Centring is a property of the layer
/// (`HintLayer::centered`), not of the bar, because the layer is what a
/// screen or an overlay contributes.
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
/// `.patch`, `.patch_part` and `.slot` on any part; `.patch` and
/// `.patch_part` are forwarded to the nested `KeyHint`s, so patching `KEY`
/// on the bar restyles every chord it draws.
///
/// ## Identity
/// One `Id` per instance; the hints are positional and carry no `ItemKey`,
/// because nothing addresses an individual hint.
///
/// ## Testing
/// `HintBarCase` with no capabilities;
/// `render::components::hint_bar::{default, busy, error, overflow, empty}`.
///
/// ## Invariants
/// Overflow drops from the **right** and always leaves the marker, so the
/// operator can see there is more; the status message keeps the right edge
/// and wins the space it needs. Never allocates: chords are rendered into a
/// fixed stack buffer by `KeyHint`.
pub struct HintBar<'a> {
    id: Id,
    layer: &'a HintLayer,
    variant: Variant,
    status: Status,
    frame: usize,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: Overrides<'a>,
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
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            patch: None,
            parts: &[],
            ov: Overrides::new(),
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
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part patches, forwarded to the nested hints.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
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

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    /// One hint, wearing this bar's overrides.
    fn hint(&self, i: usize) -> Option<KeyHint<'a>> {
        let h = self.layer.hints.get(i)?;
        let mut k = KeyHint::from_hint(self.id, h)
            .variant(self.variant)
            .patch_part(self.parts)
            .inherit_forced(self.ov.forced_state());
        if let Some(p) = self.patch {
            k = k.patch(p);
        }
        Some(k)
    }

    /// Columns the badge occupies, padding included.
    fn badge_width(&self) -> u16 {
        self.layer
            .badge
            .filter(|b| !b.is_empty())
            .map_or(0, |b| width(b).saturating_add(2))
    }

    /// Columns the status message occupies, its glyph included.
    fn status_width(&self, ui: &Ui<'_>, live: StateFlags) -> u16 {
        let Some(s) = self.layer.status.as_deref().filter(|s| !s.is_empty()) else {
            return 0;
        };
        let glyph = self.status_glyph(ui, live);
        width(s).saturating_add(glyph.map_or(0, |g| width(g).saturating_add(1)))
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
            let g = ui
                .resolve(Family::HINTBAR, self.variant, Part::MARKER, live)
                .glyph
                .unwrap_or(GlyphRole::Error);
            return Some(ui.glyph_str(g));
        }
        if live.contains(StateFlags::WARNING) {
            let g = ui
                .resolve(Family::HINTBAR, self.variant, Part::MARKER, live)
                .glyph
                .unwrap_or(GlyphRole::Dirty);
            return Some(ui.glyph_str(g));
        }
        None
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
        let live = self.ov.flags(self.status.flags());
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
                let s = ov.style(
                    ui,
                    id,
                    Family::HINTBAR,
                    self.variant,
                    if self.busy() { Part::ICON } else { Part::MARKER },
                    live,
                );
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                let used = ui.paint_str(cell, g, s.style);
                x = x.saturating_add(used).saturating_add(1);
            }
            if let Some(text) = self.layer.status.as_deref() {
                let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::LABEL, live);
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                ui.paint_str(cell, text, s.style);
            }
        }

        // the badge leads
        let mut x = area.x.saturating_add(1);
        let badge_w = self.badge_width();
        if badge_w > 0 && x.saturating_add(badge_w) <= right_limit {
            if let Some(b) = self.layer.badge {
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
        if self.layer.centered {
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
            let s = ov.style(ui, id, Family::HINTBAR, self.variant, Part::OVERFLOW, live);
            let cell = Rect {
                x,
                width: right_limit.saturating_sub(x),
                ..area
            };
            ui.glyph(cell, s.glyph.unwrap_or(GlyphRole::Ellipsis), s.style);
        }
        area
    }

    /// The natural size: one row wide enough for every hint.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let live = self.ov.flags(self.status.flags());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Chord, KeyCode};
    use crate::keymap::Hint;

    fn layer(labels: &[(&'static str, KeyCode)]) -> HintLayer {
        HintLayer {
            hints: labels
                .iter()
                .map(|(l, c)| Hint {
                    chord: Chord::key(*c),
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
        assert!(core::ptr::eq(
            HintBar::resolve(&[Some(&modal), None, Some(&screen)]).unwrap_or(&screen),
            &modal
        ));
        assert!(core::ptr::eq(
            HintBar::resolve(&[None, None, Some(&screen)]).unwrap_or(&modal),
            &screen
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
}
