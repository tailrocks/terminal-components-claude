//! `Meter`, `MeterTone` and `MeterVisual` (`COMPONENT_ARCHITECTURE.md`
//! §14.2 J12, §18.2, Appendix A 4G).
//!
//! The tone is driven by [`MeterTokens`](crate::theme::MeterTokens) and
//! `design.meter`'s thresholds through [`MeterTone::from_ratio`], never by a
//! hard-coded match: the duplicate app-side matches J12 names are deleted
//! because this is the one place the mapping lives.

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use super::progress::{PCT_COLUMNS, Pct};
use super::{Overrides, SlotFn, first_row};
use crate::collection::Status;
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{
    Family, GlyphRole, MeterRole, MeterThresholds, Role, Slot, StylePatch, Variant,
};
use crate::ui::{FrameRead, Ui};

/// What a meter's run says about the value it reports.
///
/// The three graded tones are derived from the value through
/// [`MeterTone::from_ratio`] against `design.meter`; the two flat tones
/// describe the *data* rather than the value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterTone {
    /// Healthy: at or below `design.meter.low_max`.
    #[default]
    Low,
    /// Needs attention: at or below `design.meter.medium_max`.
    Medium,
    /// Critical: above `design.meter.medium_max`.
    High,
    /// Last-good data.
    Stale,
    /// No value to report.
    Unknown,
    /// Series `n` of a grouped meter; wraps over the six series tokens.
    Series(u8),
}

impl MeterTone {
    /// The tone `ratio` earns under `t` — the helper J12 introduces so an
    /// application never re-implements the thresholds.
    ///
    /// `ratio` is clamped to `0.0..=1.0` and compared as whole percent, so
    /// the boundaries are exactly `design.meter`'s `low_max` and
    /// `medium_max`.
    #[must_use]
    pub fn from_ratio(ratio: f64, t: MeterThresholds) -> MeterTone {
        let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u16;
        if pct <= u16::from(t.low_max) {
            MeterTone::Low
        } else if pct <= u16::from(t.medium_max) {
            MeterTone::Medium
        } else {
            MeterTone::High
        }
    }

    /// The colour role this tone paints its run with.
    #[must_use]
    pub const fn role(self) -> MeterRole {
        match self {
            MeterTone::Low => MeterRole::Low,
            MeterTone::Medium => MeterRole::Medium,
            MeterTone::High => MeterRole::High,
            MeterTone::Stale => MeterRole::Stale,
            MeterTone::Unknown => MeterRole::Unknown,
            MeterTone::Series(n) => MeterRole::Series(n),
        }
    }
}

/// How a meter draws its track.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterVisual {
    /// A compact `━━━━────` run followed by the value.
    #[default]
    Line,
    /// The used share is a filled block with the value inside it, so the bar
    /// reads as filled rather than as a line.
    Block,
}

/// A capacity meter: a value, a semantic tone and a visual mode.
///
/// ## Construction
/// `Meter::new(id)`; `.ratio(f)` supplies the value. A meter with no ratio
/// paints no run — it reports whatever `.value(…)` says, which is how an
/// unknown or failed reading is expressed.
///
/// ## Ownership
/// Stateless. The caller owns the ratio, the value text and the animation
/// frame; the runtime owns nothing.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.ratio(f64)`
/// (none; clamped to `0.0..=1.0`), `.value(&str)` (empty — the percentage is
/// used when a ratio is set), `.tone(MeterTone)` (none — derived with
/// [`MeterTone::from_ratio`] against `design.meter`), `.visual(MeterVisual)`
/// (`Line`), `.status(Status)` (`Ready`), `.frame(usize)` (`0`), `.patch`,
/// `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::METER`; `DEFAULT` only.
///
/// ## States
/// Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)`; wears no runtime
/// state. A busy meter paints the spinner in `Part::ICON` and an errored one
/// the error glyph, which is what keeps the three apart without colour
/// (§11.4).
///
/// ## Actions
/// None; `Meter` has no `update` phase.
///
/// ## Focus
/// Never a focus stop.
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None.
///
/// ## Layout
/// `measure` returns `(design.size.meter_track + the value + the glyph, 1)`.
/// `draw` uses the first row of `area`; below a six-cell track it reports the
/// value alone. Returns the rect it painted; a degenerate rect paints
/// nothing (R5).
///
/// ## Parts
/// `TRACK` (the unfilled remainder), `THUMB` (the used share), `LABEL` (the
/// value text), `ICON` (the trailing readiness glyph).
///
/// ## Overrides
/// `.patch`, `.patch_part` and `.slot` on any part; a slot on `TRACK`
/// replaces the whole run, because the split between used and unused is the
/// meter's own arithmetic.
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `MeterCase` in `crates/tui/tests/conformance.rs`, declaring
/// `Caps::empty()`, so twelve of its twenty-one `meter::*` cases are
/// capability-gated and return immediately;
/// `mono_states_are_distinguishable` is narrowed to the single default
/// state and so compares one rendering against nothing.
///
/// The render matrix in `crates/tui/tests/render_components.rs` generates
/// exactly eight cells per component, one per `St` variant, so there is no
/// `render::components::meter::busy` and no `::error` to cite. Readiness
/// arrives through the matrix's `status_for` mapping: `::pressed` draws
/// `Status::Busy`, `::editing` `Status::Loading`, `::disabled`
/// `Status::Error`, and the other five cells `Status::Ready`, so the spinner
/// and the error glyph really are painted — and pinned as digests — by those
/// three cells. `Meter::icon` reads the **resolved flags** for the glyph
/// (§39.2); only the spinner branch reads `self.status`, through `busy`.
///
/// The threshold mapping is unit-tested in this module by
/// `tone_follows_the_design_thresholds_not_a_hard_coded_match` and
/// `every_tone_names_a_meter_role`.
///
/// Exercised by no test, recorded as a gap rather than covered with a
/// neighbouring citation: nothing draws `MeterVisual::Block`, nothing calls
/// `.tone(…)`, and nothing draws a meter without a `.ratio`, so the
/// `Stale`, `Unknown` and `Series` runs, the block mode's `OnAccent`
/// overlay and the value-only path have no coverage at all. The recipe's own
/// `.when(ERROR)` rule on `Part::ICON` **does** fire, and
/// `components::a_forced_component_resolves_its_props_derived_state` pins
/// it: a forced state no longer erases the status-derived `ERROR`, so the
/// glyph reaches the row through the recipe and carries `Role::Danger` with
/// it. This block previously asserted the opposite.
///
/// ## Invariants
/// The tone is a function of the ratio and `design.meter`, never of a
/// hard-coded threshold (J12); the run colour is a `MeterRole`, so a theme
/// that changes `MeterTokens` changes every meter. Never allocates.
pub struct Meter<'a> {
    id: Id,
    ratio: Option<f64>,
    value: &'a str,
    tone: Option<MeterTone>,
    visual: MeterVisual,
    variant: Variant,
    status: Status,
    frame: usize,
    ov: Overrides<'a>,
}

impl fmt::Debug for Meter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Meter")
            .field("id", &self.id)
            .field("ratio", &self.ratio)
            .field("value", &self.value)
            .field("tone", &self.tone)
            .field("visual", &self.visual)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<'a> Meter<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::TRACK, Part::THUMB, Part::LABEL, Part::ICON];

    /// The smallest run worth painting.
    const MIN_TRACK: u16 = 6;

    /// A meter with no value.
    pub const fn new(id: Id) -> Self {
        Meter {
            id,
            ratio: None,
            value: "",
            tone: None,
            visual: MeterVisual::Line,
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            ov: Overrides::new(),
        }
    }

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Set the variant.
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    /// The used share, clamped to `0.0..=1.0`.
    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    /// The text beside the run; the percentage is used when this is empty
    /// and a ratio is set.
    #[must_use]
    pub const fn value(mut self, s: &'a str) -> Self {
        self.value = s;
        self
    }

    /// Force the tone instead of deriving it from the ratio.
    #[must_use]
    pub const fn tone(mut self, t: MeterTone) -> Self {
        self.tone = Some(t);
        self
    }

    /// The visual mode.
    #[must_use]
    pub const fn visual(mut self, v: MeterVisual) -> Self {
        self.visual = v;
        self
    }

    /// Data readiness; `Busy`/`Loading` paint the spinner, `Error` the error
    /// glyph.
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

    /// The tone this meter paints with: the explicit one, else the one the
    /// ratio earns under `design.meter` (J12), else `Unknown`.
    pub fn resolved_tone(&self, ui: &Ui<'_>) -> MeterTone {
        match (self.tone, self.ratio) {
            (Some(t), _) => t,
            (None, Some(r)) => MeterTone::from_ratio(r, ui.design().meter),
            (None, None) => MeterTone::Unknown,
        }
    }

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    /// The trailing readiness glyph, or the spinner frame.
    ///
    /// The `Slot::Inherit` fallback reads `live` and not `self.status`
    /// (§39.2): the recipe above it is matched against the resolved flags, so
    /// a fallback keyed on the prop would give one glyph two sources of truth
    /// and let them disagree the moment a state is forced. `StatusBar` and
    /// `HintBar` already read the resolved flags here; this is the same shape.
    fn icon(
        &self,
        ui: &Ui<'_>,
        from_recipe: Slot<GlyphRole>,
        live: StateFlags,
    ) -> Option<&'static str> {
        if self.busy() {
            let frames = ui.design().motion.spinner_frames;
            return frames
                .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                .copied();
        }
        let g = match from_recipe {
            Slot::Set(g) => Some(g),
            Slot::Inherit if live.contains(StateFlags::ERROR) => Some(GlyphRole::Error),
            Slot::Inherit | Slot::Clear => None,
        };
        g.map(|g| ui.glyph_str(g))
    }

    /// The value text painted beside the run.
    fn value_text<'p>(&self, pct: &'p Pct) -> &'p str
    where
        'a: 'p,
    {
        if self.value.is_empty() && self.ratio.is_some() {
            pct.as_str()
        } else {
            self.value
        }
    }

    /// The run tone layered over `base`, with the caller's instance patch on
    /// top (the `CellUi::tone` shape: a role delta, never a colour).
    ///
    /// Every input is a parameter — the tone the caller wants is already
    /// resolved by [`Self::resolved_tone`] — so this is an associated
    /// function, not a method.
    fn toned(ui: &Ui<'_>, base: Style, fg: Option<Role>, bg: Option<Role>) -> Style {
        let mut delta = StylePatch::new();
        if let Some(r) = fg {
            delta = delta.set_fg(r);
        }
        if let Some(r) = bg {
            delta = delta.set_bg(r);
        }
        if delta.is_empty() {
            return base;
        }
        let top = crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style;
        base.patch(top)
    }

    /// The draw phase; returns the rect painted.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the two visual modes, the value and the trailing glyph"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        // runtime: none — a meter is a readout and registers no control;
        // derived: the readiness the caller's `.status` declares
        let live = self.ov.flags(StateFlags::empty(), self.status.flags());
        let ov = self.ov;
        let id = self.id;
        let tone = self.resolved_tone(ui);
        let pct = Pct::of((self.ratio.unwrap_or(0.0) * 100.0).round() as u16);
        let value = self.value_text(&pct);
        let vw = width(value);

        let label = ov.style(ui, id, Family::METER, self.variant, Part::LABEL, live);
        let icon_style = ov.style(ui, id, Family::METER, self.variant, Part::ICON, live);
        let glyph = self.icon(ui, icon_style.glyph, live);
        let icon_w = glyph.map_or(0, |g| width(g).saturating_add(1));

        let Some(ratio) = self.ratio else {
            // no run: the value and the marker only
            let mut x = area.x;
            if vw > 0 {
                let used = ui.paint_str(area, value, label.style);
                x = x.saturating_add(used).saturating_add(1);
            }
            if let Some(g) = glyph {
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                ui.paint_str(cell, g, icon_style.style);
            }
            return area;
        };

        match self.visual {
            MeterVisual::Line => {
                let tail = vw.saturating_add(1).saturating_add(icon_w);
                let track_w = area.width.saturating_sub(tail);
                if track_w < Self::MIN_TRACK {
                    ui.paint_str(area, value, label.style);
                    return area;
                }
                let track = Rect {
                    width: track_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::TRACK) {
                    f(ui, track);
                } else {
                    let rest = ov.style(ui, id, Family::METER, self.variant, Part::TRACK, live);
                    let thumb = ov.style(ui, id, Family::METER, self.variant, Part::THUMB, live);
                    let fill = Self::toned(ui, thumb.style, Some(Role::Meter(tone.role())), None);
                    super::progress::run_of(ui, track, GlyphRole::RuleQuiet, rest.style);
                    let filled = Rect {
                        width: (f64::from(track_w) * ratio).round() as u16,
                        ..track
                    };
                    if !filled.is_empty() {
                        super::progress::run_of(ui, filled, GlyphRole::RuleActive, fill);
                    }
                }
                let mut x = track.x.saturating_add(track_w).saturating_add(1);
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                let used = ui.paint_str(cell, value, label.style);
                x = x.saturating_add(used).saturating_add(1);
                if let Some(g) = glyph {
                    let cell = Rect {
                        x,
                        width: area.right().saturating_sub(x),
                        ..area
                    };
                    ui.paint_str(cell, g, icon_style.style);
                }
            }
            MeterVisual::Block => {
                let bar_w = area.width.saturating_sub(icon_w);
                if bar_w < 4 {
                    ui.paint_str(area, value, label.style);
                    return area;
                }
                let bar = Rect {
                    width: bar_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::TRACK) {
                    f(ui, bar);
                } else {
                    let rest = ov.style(ui, id, Family::METER, self.variant, Part::TRACK, live);
                    let thumb = ov.style(ui, id, Family::METER, self.variant, Part::THUMB, live);
                    let rest_bg =
                        Self::toned(ui, rest.style, None, Some(Role::Meter(MeterRole::FillRest)));
                    ui.fill(bar, rest_bg);
                    // the value sits inside the bar; the used share is
                    // restyled over it, so one string keeps two planes
                    let text = Rect {
                        x: bar.x.saturating_add(1),
                        width: bar.width.saturating_sub(1),
                        ..bar
                    };
                    ui.paint_str(text, value, label.style);
                    let filled = Rect {
                        width: (f64::from(bar_w) * ratio).round() as u16,
                        ..bar
                    };
                    if !filled.is_empty() {
                        let on_fill = Self::toned(
                            ui,
                            thumb.style,
                            Some(Role::OnAccent),
                            Some(Role::Meter(tone.role())),
                        );
                        ui.paint_style(filled, on_fill);
                    }
                }
                if let Some(g) = glyph {
                    let x = bar.x.saturating_add(bar_w).saturating_add(1);
                    let cell = Rect {
                        x,
                        width: area.right().saturating_sub(x),
                        ..area
                    };
                    ui.paint_str(cell, g, icon_style.style);
                }
            }
        }
        area
    }

    /// The natural size: one row, the design's track plus the value.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let vw = if self.value.is_empty() && self.ratio.is_some() {
            PCT_COLUMNS
        } else {
            width(self.value)
        };
        let tail = vw.saturating_add(3);
        Size {
            min: (Self::MIN_TRACK.saturating_add(tail), 1),
            preferred: (
                ui.design()
                    .size
                    .meter_track
                    .max(Self::MIN_TRACK)
                    .saturating_add(tail),
                1,
            ),
        }
        .fit(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: MeterThresholds = MeterThresholds {
        low_max: 59,
        medium_max: 84,
    };

    #[test]
    fn tone_follows_the_design_thresholds_not_a_hard_coded_match() {
        assert_eq!(MeterTone::from_ratio(0.0, T), MeterTone::Low);
        assert_eq!(MeterTone::from_ratio(0.59, T), MeterTone::Low);
        assert_eq!(MeterTone::from_ratio(0.60, T), MeterTone::Medium);
        assert_eq!(MeterTone::from_ratio(0.84, T), MeterTone::Medium);
        assert_eq!(MeterTone::from_ratio(0.85, T), MeterTone::High);
        assert_eq!(MeterTone::from_ratio(1.0, T), MeterTone::High);
        // a theme that moves the thresholds moves every meter
        let tight = MeterThresholds {
            low_max: 10,
            medium_max: 20,
        };
        assert_eq!(MeterTone::from_ratio(0.15, tight), MeterTone::Medium);
        assert_eq!(MeterTone::from_ratio(0.15, T), MeterTone::Low);
    }

    #[test]
    fn every_tone_names_a_meter_role() {
        assert_eq!(MeterTone::Low.role(), MeterRole::Low);
        assert_eq!(MeterTone::Stale.role(), MeterRole::Stale);
        assert_eq!(MeterTone::Unknown.role(), MeterRole::Unknown);
        assert_eq!(MeterTone::Series(3).role(), MeterRole::Series(3));
    }
}
