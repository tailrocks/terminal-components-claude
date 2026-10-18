#![doc = r" `Meter`, `MeterTone` and `MeterVisual` (`COMPONENT_ARCHITECTURE.md`"]
#![doc = r" §14.2 J12, §18.2, Appendix A 4G)."]
#![doc = r""]
#![doc = r" The tone is driven by [`MeterTokens`](crate::theme::MeterTokens) and"]
#![doc = r" `design.meter`'s thresholds through [`MeterTone::from_ratio`], never by a"]
#![doc = r" hard-coded match: the duplicate app-side matches J12 names are deleted"]
#![doc = r" because this is the one place the mapping lives."]

use core::fmt;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn, first_row};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::progress::Pct;
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{
    Family, FgStep, GlyphRole, MeterRole, MeterThresholds, Role, Slot, StyleDefaults, StylePatch,
    Variant,
};
use crate::ui::{FrameRead, Ui};
use termrock_runtime::Status;

#[doc = r" What a meter's run says about the value it reports."]
#[doc = r""]
#[doc = r" The three graded tones are derived from the value through"]
#[doc = r" [`MeterTone::from_ratio`] against `design.meter`; the two flat tones"]
#[doc = r" describe the *data* rather than the value."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterTone {
    #[doc = r" Healthy: at or below `design.meter.low_max`."]
    #[default]
    Low,
    #[doc = r" Needs attention: at or below `design.meter.medium_max`."]
    Medium,
    #[doc = r" Critical: above `design.meter.medium_max`."]
    High,
    #[doc = r" Last-good data."]
    Stale,
    #[doc = r" No value to report."]
    Unknown,
    #[doc = r" Series `n` of a grouped meter; wraps over the six series tokens."]
    Series(u8),
}

impl MeterTone {
    #[doc = r" The tone `ratio` earns under `t` — the helper J12 introduces so an"]
    #[doc = r" application never re-implements the thresholds."]
    #[doc = r""]
    #[doc = r" `ratio` is clamped to `0.0..=1.0` and compared as whole percent, so"]
    #[doc = r" the boundaries are exactly `design.meter`'s `low_max` and"]
    #[doc = r" `medium_max`."]
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

    #[doc = r" The colour role this tone paints its run with."]
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

#[doc = r" How a meter draws its track."]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterVisual {
    #[doc = r" A compact `━━━━────` run followed by the value."]
    #[default]
    Line,
    #[doc = r" The used share is a filled block with the value inside it, so the bar"]
    #[doc = r" reads as filled rather than as a line."]
    Block,
}

#[doc = r" A capacity meter: a value, a semantic tone and a visual mode."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Meter::new(id)`; `.ratio(f)` supplies the value. A meter with no ratio"]
#[doc = r" paints no run — it reports whatever `.value(…)` says, which is how an"]
#[doc = r" unknown or failed reading is expressed."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns the ratio, the value text and the animation"]
#[doc = r" frame; the runtime owns nothing."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.ratio(f64)`"]
#[doc = r" (none; clamped to `0.0..=1.0`), `.value(&str)` (empty — the percentage is"]
#[doc = r" used when a ratio is set), `.tone(MeterTone)` (none — derived with"]
#[doc = r" [`MeterTone::from_ratio`] against `design.meter`), `.visual(MeterVisual)`"]
#[doc = r" (`Line`), `.status(Status)` (`Ready`), `.frame(usize)` (`0`),"]
#[doc = r" `.suffix_width(u16)` (minimum zero), `.leading_activity(bool)` (false), `.patch`,"]
#[doc = r" `.patch_part`, `.part_defaults`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::METER`; `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)`; wears no runtime"]
#[doc = r" state. A busy meter paints the spinner in `Part::ICON` and an errored one"]
#[doc = r" the error glyph, which is what keeps the three apart without colour"]
#[doc = r" (§11.4)."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `Meter` has no `update` phase."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` returns `(design.size.meter_track + the value + the glyph, 1)`."]
#[doc = r" `draw` uses the first row of `area`; below a six-cell track it reports the"]
#[doc = r" value alone. Returns the rect it painted; a degenerate rect paints"]
#[doc = r" nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `TRACK` (the unfilled remainder), `THUMB` (the used share), `LABEL` (the"]
#[doc = r" value text), `ICON` (the readiness glyph, optionally leading activity)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.part_defaults` supplies borrowed role-level defaults after intrinsic"]
#[doc = r" tone defaults and before recipe/state, Mono policy and explicit overrides."]
#[doc = r" `.patch` and `.patch_part` on any part. `.slot` on exactly `TRACK`,"]
#[doc = r" `LABEL` and `ICON`. `THUMB` is **not** slot-addressable: a slot on"]
#[doc = r" `TRACK` replaces the whole run, used share included, because the split"]
#[doc = r" between the two is the meter's own arithmetic. The `ICON` slot replaces"]
#[doc = r" the busy spinner as well as the error glyph (§45.4)."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `MeterCase` in `crates/tui/tests/conformance.rs`, declaring"]
#[doc = r" `Caps::REPORTS_STATUS`. Its mono states retain the default, `ERROR` and"]
#[doc = r" `BUSY` renderings required by that readiness capability."]
#[doc = r""]
#[doc = r" The render matrix in `crates/tui/tests/render_components.rs` generates"]
#[doc = r" exactly eight cells per component, one per `St` variant, so there is no"]
#[doc = r" `render::components::meter::busy` and no `::error` to cite. Readiness"]
#[doc = r" arrives through the matrix's `status_for` mapping: `::pressed` draws"]
#[doc = r" `Status::Busy`, `::editing` `Status::Loading`, `::disabled`"]
#[doc = r" `Status::Error`, and the other five cells `Status::Ready`, so the spinner"]
#[doc = r" and the error glyph really are painted — and pinned as digests — by those"]
#[doc = r" three cells. `Meter::icon` reads the **resolved flags** for the glyph"]
#[doc = r" (§39.2); only the spinner branch reads `self.status`, through `busy`."]
#[doc = r""]
#[doc = r" The threshold mapping is unit-tested in this module by"]
#[doc = r" `tone_follows_the_design_thresholds_not_a_hard_coded_match` and"]
#[doc = r" `every_tone_names_a_meter_role`."]
#[doc = r""]
#[doc = r" `tests/meter_defaults.rs` covers both visual modes and every tone, authored"]
#[doc = r" part defaults, theme/subtree/instance precedence, and unchanged default"]
#[doc = r" cells across both themes and all four color capabilities."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" The tone is a function of the ratio and `design.meter`, never of a"]
#[doc = r" hard-coded threshold (J12); the run colour is a `MeterRole`, so a theme"]
#[doc = r" that changes `MeterTokens` changes every meter. Never allocates."]
pub struct Meter<'a> {
    id: Id,
    ratio: Option<f64>,
    value: &'a str,
    part_defaults: &'a [(Part, StylePatch)],
    tone: Option<MeterTone>,
    visual: MeterVisual,
    suffix_width: u16,
    leading_activity: bool,
    variant: Variant,
    status: Status,
    frame: usize,
    ov: PartStyle<'a>,
}

#[derive(Clone, Copy)]
struct Readout {
    icon: crate::theme::Resolved,
    glyph: Option<&'static str>,
    leading: bool,
    width: u16,
    suffix: u16,
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
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::TRACK, Part::THUMB, Part::LABEL, Part::ICON];

    #[doc = r" The smallest run worth painting."]
    const MIN_TRACK: u16 = 6;

    #[doc = r" A meter with no value."]
    pub const fn new(id: Id) -> Self {
        Meter {
            id,
            ratio: None,
            value: "",
            part_defaults: &[],
            tone: None,
            visual: MeterVisual::Line,
            suffix_width: 0,
            leading_activity: false,
            variant: Variant::DEFAULT,
            status: Status::Ready,
            frame: 0,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Set the variant."]
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[doc = r" The used share, clamped to `0.0..=1.0`."]
    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    #[doc = r" The text beside the run; the percentage is used when this is empty"]
    #[doc = r" and a ratio is set."]
    #[must_use]
    pub const fn value(mut self, s: &'a str) -> Self {
        self.value = s;
        self
    }

    #[doc = r" Force the tone instead of deriving it from the ratio."]
    #[must_use]
    pub const fn tone(mut self, t: MeterTone) -> Self {
        self.tone = Some(t);
        self
    }

    #[doc = r" The visual mode."]
    #[must_use]
    pub const fn visual(mut self, v: MeterVisual) -> Self {
        self.visual = v;
        self
    }

    #[doc = r" Data readiness; `Busy`/`Loading` paint the spinner, `Error` the error"]
    #[doc = r" glyph."]
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[doc = r" Minimum trailing marker budget, including its leading gap. The"]
    #[doc = r" resolved glyph may enlarge it. Default zero retains dynamic width."]
    #[must_use]
    pub const fn suffix_width(mut self, cells: u16) -> Self {
        self.suffix_width = cells;
        self
    }

    #[doc = r" Put busy/loading activity before the value instead of after it."]
    #[doc = r" Default false. The shared animation frames and ICON slot still apply."]
    #[must_use]
    pub const fn leading_activity(mut self, yes: bool) -> Self {
        self.leading_activity = yes;
        self
    }

    #[doc = r" The animation frame the spinner reads."]
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[doc = r" Borrowed author defaults layered over intrinsic tone defaults, before"]
    #[doc = r" recipe/state, Mono policy, theme/scope and instance overrides."]
    #[must_use]
    pub const fn part_defaults(mut self, defaults: &'a [(Part, StylePatch)]) -> Self {
        self.part_defaults = defaults;
        self
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The tone this meter paints with: the explicit one, else the one the"]
    #[doc = r" ratio earns under `design.meter` (J12), else `Unknown`."]
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

    #[doc = r" The trailing readiness glyph, or the spinner frame."]
    #[doc = r""]
    #[doc = r" The `Slot::Inherit` fallback reads `live` and not `self.status`"]
    #[doc = r" (§39.2): the recipe above it is matched against the resolved flags, so"]
    #[doc = r" a fallback keyed on the prop would give one glyph two sources of truth"]
    #[doc = r" and let them disagree. `StatusBar` and"]
    #[doc = r" `HintBar` already read the resolved flags here; this is the same shape."]
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

    #[doc = r" Paint the value readout into `cell`, or hand the cell to the"]
    #[doc = r" `Part::LABEL` slot. Returns the columns used."]
    fn paint_value(&self, ui: &mut Ui<'_>, cell: Rect, value: &str, style: PaintStyle) -> u16 {
        if let Some(f) = self.ov.slot_for(Part::LABEL) {
            f(ui, cell);
            return width(value).min(cell.width);
        }
        ui.paint_str(cell, value, style)
    }

    #[doc = r" Paint the trailing readiness glyph into `cell`, or hand the cell to"]
    #[doc = r" the `Part::ICON` slot."]
    #[doc = r""]
    #[doc = r" The slot is consulted **before** `design.motion.spinner_frames`"]
    #[doc = r" (§45.4): a slot is substitution, not suppression, so one `Part` keeps"]
    #[doc = r" one answer whether the glyph came from the recipe or from the spinner."]
    fn paint_icon(&self, ui: &mut Ui<'_>, cell: Rect, glyph: &str, style: PaintStyle) {
        if let Some(f) = self.ov.slot_for(Part::ICON) {
            f(ui, cell);
            return;
        }
        ui.paint_str(cell, glyph, style);
    }

    #[doc = r" The value text painted beside the run."]
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

    fn resolve_part(
        &self,
        ui: &Ui<'_>,
        part: Part,
        live: StateFlags,
        base: StylePatch,
    ) -> crate::theme::Resolved {
        let mut defaults = base;
        for (named, patch) in self.part_defaults {
            if *named == part {
                defaults = defaults.merge(*patch);
            }
        }
        let local = self.ov.part_patch(part);
        ui.style_defaults(
            Family::METER,
            self.variant,
            part,
            live,
            StyleDefaults::new(defaults),
            local.as_ref(),
        )
    }

    fn part_style(
        &self,
        ui: &mut Ui<'_>,
        part: Part,
        live: StateFlags,
        base: StylePatch,
    ) -> crate::theme::Resolved {
        let resolved = self.resolve_part(ui, part, live, base);
        self.ov
            .note(ui, self.id, Family::METER, self.variant, part, resolved);
        resolved
    }

    fn readout(&self, ui: &Ui<'_>, value: &str, live: StateFlags) -> Readout {
        let icon = self.resolve_part(ui, Part::ICON, live, StylePatch::new().set_fg(Role::Accent));
        let glyph = self.icon(ui, icon.glyph, live);
        let leading = self.leading_activity && self.busy();
        let glyph_width = glyph.map_or(0, |g| width(g).saturating_add(1));
        Readout {
            icon,
            glyph,
            leading,
            width: width(value).saturating_add(if leading { glyph_width } else { 0 }),
            suffix: self.suffix_width.max(if leading { 0 } else { glyph_width }),
        }
    }

    fn paint_readout(
        &self,
        ui: &mut Ui<'_>,
        mut cell: Rect,
        value: &str,
        style: PaintStyle,
        readout: Readout,
    ) -> u16 {
        let mut lead = 0;
        if readout.leading
            && let Some(glyph) = readout.glyph
        {
            let icon = Rect {
                width: width(glyph).min(cell.width),
                ..cell
            };
            self.paint_icon(ui, icon, glyph, readout.icon.style);
            lead = width(glyph).saturating_add(1).min(cell.width);
            cell.x = cell.x.saturating_add(lead);
            cell.width = cell.width.saturating_sub(lead);
        }
        lead.saturating_add(self.paint_value(ui, cell, value, style))
    }

    #[doc = r" The draw phase; returns the rect painted."]
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the two visual modes, the value and the trailing glyph"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), self.status.flags());
        let ov = self.ov;
        let tone = self.resolved_tone(ui);
        let pct = Pct::of((self.ratio.unwrap_or(0.0) * 100.0).round() as u16);
        let value = self.value_text(&pct);
        let readout = self.readout(ui, value, live);
        let vw = readout.width;
        let label = self.part_style(
            ui,
            Part::LABEL,
            live,
            StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)),
        );
        let icon_style = readout.icon;
        ov.note(
            ui,
            self.id,
            Family::METER,
            self.variant,
            Part::ICON,
            icon_style,
        );
        let glyph = if readout.leading { None } else { readout.glyph };
        let icon_w = readout.suffix;

        let Some(ratio) = self.ratio else {
            let mut x = area.x;
            if vw > 0 {
                let text = Rect {
                    width: area.width.saturating_sub(icon_w),
                    ..area
                };
                let used = self.paint_readout(ui, text, value, label.style, readout);
                x = x.saturating_add(used).saturating_add(u16::from(used > 0));
            }
            if let Some(g) = glyph {
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                self.paint_icon(ui, cell, g, icon_style.style);
            }
            return area;
        };

        match self.visual {
            MeterVisual::Line => {
                let tail = vw.saturating_add(1).saturating_add(icon_w);
                let track_w = area.width.saturating_sub(tail);
                if track_w < Self::MIN_TRACK {
                    self.paint_readout(ui, area, value, label.style, readout);
                    return area;
                }
                let track = Rect {
                    width: track_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::TRACK) {
                    f(ui, track);
                } else {
                    let rest = self.part_style(
                        ui,
                        Part::TRACK,
                        live,
                        StylePatch::new().set_fg(Role::Meter(MeterRole::Track)),
                    );
                    let fill = self
                        .part_style(
                            ui,
                            Part::THUMB,
                            live,
                            StylePatch::new().set_fg(Role::Meter(tone.role())),
                        )
                        .style;
                    crate::progress::run_of(ui, track, GlyphRole::RuleQuiet, rest.style);
                    let filled = Rect {
                        width: (f64::from(track_w) * ratio).round() as u16,
                        ..track
                    };
                    if !filled.is_empty() {
                        crate::progress::run_of(ui, filled, GlyphRole::RuleActive, fill);
                    }
                }
                let mut x = track.x.saturating_add(track_w).saturating_add(1);
                let cell = Rect {
                    x,
                    width: area.right().saturating_sub(x),
                    ..area
                };
                let used = self.paint_readout(ui, cell, value, label.style, readout);
                x = x.saturating_add(used).saturating_add(1);
                if let Some(g) = glyph {
                    let cell = Rect {
                        x,
                        width: area.right().saturating_sub(x),
                        ..area
                    };
                    self.paint_icon(ui, cell, g, icon_style.style);
                }
            }
            MeterVisual::Block => {
                let bar_w = area.width.saturating_sub(icon_w);
                if bar_w < 4 {
                    self.paint_readout(ui, area, value, label.style, readout);
                    return area;
                }
                let bar = Rect {
                    width: bar_w,
                    ..area
                };
                if let Some(f) = ov.slot_for(Part::TRACK) {
                    f(ui, bar);
                } else {
                    let rest_bg = self
                        .part_style(
                            ui,
                            Part::TRACK,
                            live,
                            StylePatch::new()
                                .set_fg(Role::Meter(MeterRole::Track))
                                .set_bg(Role::Meter(MeterRole::FillRest)),
                        )
                        .style;
                    ui.fill(bar, rest_bg);

                    let text = Rect {
                        x: bar.x.saturating_add(1),
                        width: bar.width.saturating_sub(1),
                        ..bar
                    };
                    self.paint_readout(ui, text, value, label.style, readout);
                    let filled = Rect {
                        width: (f64::from(bar_w) * ratio).round() as u16,
                        ..bar
                    };
                    if !filled.is_empty() {
                        let on_fill = self
                            .part_style(
                                ui,
                                Part::THUMB,
                                live,
                                StylePatch::new()
                                    .set_fg(Role::OnAccent)
                                    .set_bg(Role::Meter(tone.role())),
                            )
                            .style;
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
                    self.paint_icon(ui, cell, g, icon_style.style);
                }
            }
        }
        area
    }

    #[doc = r" One-row size from the same readout/suffix geometry used by draw."]
    #[doc = r" Line mode adds the value beside the track; block mode holds it inside."]
    #[doc = r" A value-only meter has no minimum track budget."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let pct = Pct::of((self.ratio.unwrap_or(0.0) * 100.0).round() as u16);
        let value = self.value_text(&pct);
        let readout = self.readout(ui, value, self.status.flags());
        if self.ratio.is_none() {
            return Size::exact(readout.width.saturating_add(readout.suffix), 1).fit(c);
        }
        let track = ui.design().size.meter_track;
        let (minimum, preferred) = match self.visual {
            MeterVisual::Line => {
                let tail = readout
                    .width
                    .saturating_add(1)
                    .saturating_add(readout.suffix);
                (
                    Self::MIN_TRACK.saturating_add(tail),
                    track.max(Self::MIN_TRACK).saturating_add(tail),
                )
            }
            MeterVisual::Block => (
                4_u16.saturating_add(readout.suffix),
                track
                    .max(4)
                    .max(readout.width.saturating_add(2))
                    .saturating_add(readout.suffix),
            ),
        };
        Size {
            min: (minimum, 1),
            preferred: (preferred, 1),
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
