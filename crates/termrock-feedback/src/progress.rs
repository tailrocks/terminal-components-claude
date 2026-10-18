#![doc = r" `ProgressBar` and `Spinner` (`COMPONENT_ARCHITECTURE.md` §18.2,"]
#![doc = r" Appendix A 4G)."]
#![doc = r""]
#![doc = r" The legacy `render_bar` / `render_indeterminate` / `render_spinner` free"]
#![doc = r" functions and their five `bg: Color` parameters are gone: the plane comes"]
#![doc = r" from the surface the caller draws on, the spinner frames from"]
#![doc = r" `design.motion.spinner_frames` and the track glyphs from"]
#![doc = r" `GlyphRole::{RuleQuiet, RuleActive}` (A4, §11.5)."]

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{PartStyle, SlotFn, first_row, shift};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};
use termrock_runtime::Status;

#[doc = r#" Columns the percentage column occupies: `"100%"` is the widest value."#]
pub(crate) const PCT_COLUMNS: u16 = 4;

#[doc = r" A percentage rendered into a fixed stack buffer."]
#[doc = r""]
#[doc = r" A component may not allocate once per frame (§20.9-6, R5) and there is no"]
#[doc = r" `Ui::paint_fmt`, so the two digits are produced by division rather than by"]
#[doc = r" `format!`."]
#[doc = r""]
#[doc = r" Each branch of `of` writes a whole four-byte literal, percent sign"]
#[doc = r" included, so no column is ever addressed by a runtime index: the buffer's"]
#[doc = r" capacity is discharged by the array literal itself rather than by an"]
#[doc = r" in-bounds argument the reader has to reconstruct."]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pct {
    buf: [u8; 4],
    len: usize,
}

impl Pct {
    #[doc = r" `v` percent, clamped to `0..=100`."]
    pub(crate) const fn of(v: u16) -> Self {
        let v = if v > 100 { 100 } else { v };

        let (buf, len) = if v >= 100 {
            (*b"100%", 4)
        } else if v >= 10 {
            (
                [
                    b'0'.saturating_add((v / 10) as u8),
                    b'0'.saturating_add((v % 10) as u8),
                    b'%',
                    0,
                ],
                3,
            )
        } else {
            ([b'0'.saturating_add(v as u8), b'%', 0, 0], 2)
        };
        Pct { buf, len }
    }

    #[doc = r#" The rendered text, `"0%"` … `"100%"`."#]
    #[doc = r""]
    #[doc = r" `len` is one of `2`, `3` or `4` and every byte written is ASCII, so"]
    #[doc = r#" both fallible steps succeed; the `""` arm is a correct empty rendering"#]
    #[doc = r" rather than a panic if a future edit breaks that."]
    pub(crate) fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

#[doc = r" Paint a run of one glyph across `run`, one cell at a time (R-4: a single"]
#[doc = r" pass over columns, never a nested `for y … for x`)."]
pub(crate) fn run_of(
    ui: &mut Ui<'_>,
    run: Rect,
    glyph: GlyphRole,
    style: crate::theme::PaintStyle,
) {
    let sym = ui.glyph_str(glyph);
    for col in run.columns() {
        ui.paint_cell(Position::new(col.x, run.y), sym, style);
    }
}

#[doc = r" A one-row progress bar: `label ━━━━━━────── 64% ✓`."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `ProgressBar::new(id)`. `.ratio(f)` makes it determinate; without one it"]
#[doc = r" is indeterminate and paints a sweeping segment."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns the label, the ratio and the animation frame;"]
#[doc = r" the runtime owns nothing — a progress bar is never focused, hovered or"]
#[doc = r" pressed."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.label(&str)`"]
#[doc = r" (empty), `.ratio(f64)` (none — indeterminate; clamped to `0.0..=1.0`),"]
#[doc = r" `.status(Status)` (`Ready`), `.done(bool)` (`false`),"]
#[doc = r" `.icon(GlyphRole)` (none — the explicit override that expresses a paused"]
#[doc = r" bar as `GlyphRole::ProgressPaused`), `.frame(usize)` (`0`), `.patch`,"]
#[doc = r" `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::PROGRESS`; `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)` and `CHECKED`"]
#[doc = r" from `.done(true)`; wears no runtime state. The recipe keys the trailing"]
#[doc = r" glyph on exactly those: `ERROR` → `GlyphRole::Error`, `CHECKED` →"]
#[doc = r" `GlyphRole::ProgressDone`."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `ProgressBar` has no `update` phase."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; registers no ring entry and no region."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" `measure` returns `(label + gap + a six-cell minimum track + percentage +"]
#[doc = r" icon, 1)`. `draw` uses the first row of `area`; when the track would be"]
#[doc = r" narrower than six cells it paints the percentage alone, exactly as the"]
#[doc = r" legacy bar did. Returns the rect it painted; a degenerate rect paints"]
#[doc = r" nothing (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `LABEL` (the leading label), `TRACK` (the unfilled remainder), `THUMB`"]
#[doc = r" (the filled run), `META` (the percentage), `ICON` (the trailing state"]
#[doc = r" glyph or the spinner)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` on any part. `.slot` on exactly `LABEL`,"]
#[doc = r" `TRACK`, `META` and `ICON`. `THUMB` is **not** slot-addressable: a slot"]
#[doc = r" on `TRACK` replaces the whole track, filled run included, because the"]
#[doc = r" split between the two is the bar's own arithmetic. The `ICON` slot"]
#[doc = r" replaces the busy spinner as well as the state glyph (§45.4)."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `ProgressBarCase` in `crates/tui/tests/conformance.rs`, declaring"]
#[doc = r" `Caps::REPORTS_STATUS`. Its mono states retain the default, `ERROR` and"]
#[doc = r" `BUSY` renderings required by that readiness capability."]
#[doc = r""]
#[doc = r" The render matrix in `crates/tui/tests/render_components.rs` generates"]
#[doc = r" exactly eight cells per component, one per `St` variant, so there is no"]
#[doc = r" `render::components::progress_bar::busy` and no `::error` to cite."]
#[doc = r" Readiness reaches the bar through the matrix's `status_for` mapping"]
#[doc = r" instead: `::pressed` draws `Status::Busy`, `::editing` draws"]
#[doc = r" `Status::Loading`, `::disabled` draws `Status::Error`, and `::default`,"]
#[doc = r" `::focused`, `::hovered`, `::selected` and `::empty` draw"]
#[doc = r" `Status::Ready`. `draw` reads `self.status` rather than the resolved"]
#[doc = r" flags to pick the spinner, so the spinner really is painted — and pinned"]
#[doc = r" as a digest — by `::pressed` and `::editing`."]
#[doc = r""]
#[doc = r" The span and percentage arithmetic is unit-tested in this module by"]
#[doc = r" `a_determinate_span_starts_at_zero_and_rounds`,"]
#[doc = r" `the_indeterminate_sweep_crosses_the_track_and_stays_inside_it` and"]
#[doc = r" `percentages_format_without_allocating`."]
#[doc = r""]
#[doc = r" The recipe's `ERROR → GlyphRole::Error` rule fires from the controlled"]
#[doc = r" `Status::Error` prop (§39.2)."]
#[doc = r""]
#[doc = r" `a_done_bar_reaches_the_checked_recipe_rule` exercises the recipe's"]
#[doc = r" `CHECKED → GlyphRole::ProgressDone` rule through `.done(true)`. Nothing"]
#[doc = r" draws an *indeterminate* bar in"]
#[doc = r" the render matrix — every fixture calls `.ratio` — and nothing calls"]
#[doc = r" `.icon(…)`. The sub-`MIN_TRACK` fallback is reached only inside"]
#[doc = r" `progress_bar::survives_tiny_rects_0x0_to_3x3`, which asserts that the bar"]
#[doc = r" neither panics nor paints outside its rect, never what it paints."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" Discharges §11.4's readiness obligation: a `BUSY`/`LOADING` bar paints"]
#[doc = r" `Part::ICON` with `design.motion.spinner_frames`, which is a *symbol* and"]
#[doc = r" therefore survives `ColorLevel::Mono`. The frame is a prop, so two draws"]
#[doc = r" with the same props are byte-identical. Never allocates."]
pub struct ProgressBar<'a> {
    id: Id,
    label: &'a str,
    ratio: Option<f64>,
    variant: Variant,
    status: Status,
    done: bool,
    icon: Option<GlyphRole>,
    frame: usize,
    ov: PartStyle<'a>,
}

impl fmt::Debug for ProgressBar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressBar")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("ratio", &self.ratio)
            .field("status", &self.status)
            .field("done", &self.done)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> ProgressBar<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::LABEL,
        Part::TRACK,
        Part::THUMB,
        Part::META,
        Part::ICON,
    ];

    #[doc = r" The smallest track worth painting; below it the bar reports the"]
    #[doc = r" percentage alone."]
    const MIN_TRACK: u16 = 6;

    #[doc = r" An indeterminate bar."]
    pub const fn new(id: Id) -> Self {
        ProgressBar {
            id,
            label: "",
            ratio: None,
            variant: Variant::DEFAULT,
            status: Status::Ready,
            done: false,
            icon: None,
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

    #[doc = r" A leading label."]
    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    #[doc = r" Make the bar determinate at `r`, clamped to `0.0..=1.0`."]
    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    #[doc = r" Data readiness; `Busy`/`Loading` paint the spinner, `Error` the error"]
    #[doc = r" glyph."]
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[doc = r" Completion: the recipe's `CHECKED` glyph."]
    #[must_use]
    pub const fn done(mut self, yes: bool) -> Self {
        self.done = yes;
        self
    }

    #[doc = r" Override the trailing glyph. The explicit way to express a suspended"]
    #[doc = r" bar (`GlyphRole::ProgressPaused`) without a second lifecycle enum"]
    #[doc = r" beside `Status` (§13: no boolean parameter soup)."]
    #[must_use]
    pub const fn icon(mut self, g: GlyphRole) -> Self {
        self.icon = Some(g);
        self
    }

    #[doc = r" The animation frame the spinner and the indeterminate sweep read."]
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
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

    const fn busy(&self) -> bool {
        matches!(self.status, Status::Busy | Status::Loading)
    }

    fn flags(&self) -> StateFlags {
        let mut f = self.status.flags();
        if self.done {
            f |= StateFlags::CHECKED;
        }
        f
    }

    #[doc = r" Columns the label and its gap occupy at `w`, or `0` when the label"]
    #[doc = r" does not earn its place."]
    fn label_columns(&self, ui: &Ui<'_>, w: u16) -> u16 {
        let lw = width(self.label);
        let gap = ui.design().space.gap.max(1);
        if lw == 0 || w <= lw.saturating_add(gap).saturating_add(Self::MIN_TRACK) {
            0
        } else {
            lw.saturating_add(gap)
        }
    }

    #[doc = r" Columns the percentage and the trailing glyph occupy."]
    fn tail_columns(&self) -> u16 {
        match self.ratio {
            Some(_) => PCT_COLUMNS.saturating_add(3),
            None => 2,
        }
    }

    #[doc = r" Columns the percentage readout occupies; zero when indeterminate."]
    fn pct_width(&self) -> u16 {
        self.ratio
            .map_or(0, |r| width(Pct::of(percent(r)).as_str()))
    }

    #[doc = r" Paint the percentage readout into `cell`, or hand the cell to the"]
    #[doc = r" `Part::META` slot. An indeterminate bar has no percentage and paints"]
    #[doc = r" nothing, so there is no cell for a slot to replace."]
    fn paint_meta(&self, ui: &mut Ui<'_>, cell: Rect, live: StateFlags) {
        let Some(r) = self.ratio else { return };
        if let Some(f) = self.ov.slot_for(Part::META) {
            f(ui, cell);
            return;
        }
        let s = self.ov.style(
            ui,
            self.id,
            Family::PROGRESS,
            self.variant,
            Part::META,
            live,
        );
        ui.paint_str(cell, Pct::of(percent(r)).as_str(), s.style);
    }

    #[doc = r" The draw phase; returns the rect painted."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), self.flags());
        let ov = self.ov;
        let id = self.id;
        let style = |ui: &mut Ui<'_>, part: Part| {
            ov.style(ui, id, Family::PROGRESS, self.variant, part, live)
        };

        let label_w = self.label_columns(ui, area.width);
        if label_w > 0 {
            let cell = Rect {
                width: label_w,
                ..area
            };
            if let Some(f) = ov.slot_for(Part::LABEL) {
                f(ui, cell);
            } else {
                let s = style(ui, Part::LABEL);
                ui.paint_str(cell, self.label, s.style);
            }
        }
        let body = shift(area, label_w);
        let tail = self.tail_columns();
        let track_w = body.width.saturating_sub(tail);
        if track_w < Self::MIN_TRACK {
            self.paint_meta(ui, body, live);
            return area;
        }
        let track = Rect {
            width: track_w,
            ..body
        };
        if let Some(f) = ov.slot_for(Part::TRACK) {
            f(ui, track);
        } else {
            let rest = style(ui, Part::TRACK);
            let fill = style(ui, Part::THUMB);
            let (from, to) = self.filled_span(track_w);
            run_of(ui, track, GlyphRole::RuleQuiet, rest.style);
            let filled = Rect {
                x: track.x.saturating_add(from),
                width: to.saturating_sub(from),
                ..track
            };
            if !filled.is_empty() {
                run_of(ui, filled, GlyphRole::RuleActive, fill.style);
            }
        }
        let mut x = track.x.saturating_add(track_w).saturating_add(1);
        if self.ratio.is_some() {
            let pad = PCT_COLUMNS.saturating_sub(self.pct_width());
            let cell = Rect {
                x: x.saturating_add(pad),
                width: area.right().saturating_sub(x.saturating_add(pad)),
                ..area
            };
            self.paint_meta(ui, cell, live);
            x = x.saturating_add(PCT_COLUMNS).saturating_add(1);
        }
        let icon_cell = Rect {
            x,
            width: area.right().saturating_sub(x),
            ..area
        };
        if !icon_cell.is_empty() {
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, icon_cell);
            } else {
                let s = style(ui, Part::ICON);
                if self.busy() {
                    let frames = ui.design().motion.spinner_frames;
                    let frame = frames
                        .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                        .copied()
                        .unwrap_or("");
                    ui.paint_str(icon_cell, frame, s.style);
                } else {
                    let recipe_glyph = match s.glyph {
                        Slot::Set(g) => Some(g),
                        Slot::Inherit | Slot::Clear => None,
                    };
                    if let Some(g) = self.icon.or(recipe_glyph) {
                        ui.glyph(icon_cell, g, s.style);
                    }
                }
            }
        }
        area
    }

    #[doc = r" `(first, last)` filled column of a `w`-wide track."]
    fn filled_span(&self, w: u16) -> (u16, u16) {
        if let Some(r) = self.ratio {
            return (0, (f64::from(w) * r).round() as u16);
        }

        let seg = (w / 5).clamp(2, 8);
        let period = usize::from(w.saturating_add(seg)).max(1);
        let edge = self.frame.checked_rem(period).unwrap_or(0);
        let edge = u16::try_from(edge).unwrap_or(u16::MAX);
        (edge.saturating_sub(seg).min(w), edge.min(w))
    }

    #[doc = r" The natural size: one row wide enough for a usable track."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let gap = ui.design().space.gap.max(1);
        let lw = width(self.label);
        let lead = if lw == 0 { 0 } else { lw.saturating_add(gap) };
        let min = lead
            .saturating_add(Self::MIN_TRACK)
            .saturating_add(self.tail_columns());
        let preferred = lead
            .saturating_add(ui.design().size.meter_track.max(Self::MIN_TRACK))
            .saturating_add(self.tail_columns());
        Size {
            min: (min, 1),
            preferred: (preferred, 1),
        }
        .fit(c)
    }
}

#[doc = r" A ratio as whole percent."]
fn percent(r: f64) -> u16 {
    (r.clamp(0.0, 1.0) * 100.0).round() as u16
}

#[doc = r" The compact activity state: `⠋ label`."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `Spinner::new(id)`; `.label(&str)` adds the text beside it."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns the label and the animation frame; the runtime"]
#[doc = r" owns nothing."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.variant(Variant)` (default `Recipe.default_variant`), `.label(&str)`"]
#[doc = r" (empty), `.frame(usize)` (`0`), `.patch`, `.patch_part`, `.slot`,"]
#[doc = r" runtime state."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::PROGRESS`; `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" None. A spinner is always spinning: which frame it shows is a *prop*, not"]
#[doc = r" a state, so a digest is a pure function of the props."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `Spinner` has no `update` phase."]
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
#[doc = r" `measure` returns `(glyph + gap + label, 1)`. `draw` uses the first row of"]
#[doc = r" `area` and returns the rect it painted; a degenerate rect paints nothing"]
#[doc = r" (R5)."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `ICON` (the frame), `LABEL` (the text)."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` on both parts; `.slot` on exactly `ICON` and"]
#[doc = r" `LABEL`."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance; no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `SpinnerCase` in `crates/tui/tests/conformance.rs` with `Caps::empty()`,"]
#[doc = r" under the same twelve-of-twenty-one gating as `ProgressBar`, plus the"]
#[doc = r" eight `render::components::spinner::*` cells the matrix generates —"]
#[doc = r" `default`, `focused`, `hovered`, `pressed`, `disabled`, `selected`,"]
#[doc = r" `editing` and `empty`. A spinner takes no `Status`, so the matrix's"]
#[doc = r" `status_for` mapping never reaches it. Reference cells can supply only"]
#[doc = r" transient runtime state; `BUSY` remains derived by the spinner."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" Reads `design.motion.spinner_frames`, never a baked-in table (A4). Never"]
#[doc = r" allocates; never writes outside `area`."]
pub struct Spinner<'a> {
    id: Id,
    label: &'a str,
    gap: Option<u16>,
    variant: Variant,
    frame: usize,
    ov: PartStyle<'a>,
}

#[doc = r" Canonical frame selection for composed and standalone spinners."]
pub(super) fn spinner_glyph(frames: &'static [&'static str], frame: usize) -> Option<&'static str> {
    frames
        .get(frame.checked_rem(frames.len()).unwrap_or(0))
        .copied()
}

impl fmt::Debug for Spinner<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Spinner")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("frame", &self.frame)
            .finish_non_exhaustive()
    }
}

impl<'a> Spinner<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[Part::ICON, Part::LABEL];

    #[doc = r" A spinner with no label."]
    pub const fn new(id: Id) -> Self {
        Spinner {
            id,
            label: "",
            gap: None,
            variant: Variant::DEFAULT,
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

    #[doc = r" The text beside the frame."]
    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    #[doc = r" Space between the frame and a nonempty label. Omission uses the"]
    #[doc = r" design gap (at least one cell); an explicit value may be zero."]
    #[must_use]
    pub const fn gap(mut self, cells: u16) -> Self {
        self.gap = Some(cells);
        self
    }

    fn label_gap(&self, ui: &Ui<'_>) -> u16 {
        self.gap.unwrap_or_else(|| ui.design().space.gap.max(1))
    }

    #[doc = r" The animation frame."]
    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
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

    #[doc = r" The frame this spinner shows."]
    fn glyph(ui: &Ui<'_>, frame: usize) -> &'static str {
        spinner_glyph(ui.design().motion.spinner_frames, frame).unwrap_or("")
    }

    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        let g = width(Self::glyph(ui, self.frame));
        if self.label.is_empty() {
            return g;
        }
        g.saturating_add(self.label_gap(ui))
            .saturating_add(width(self.label))
    }

    #[doc = r" The draw phase; returns the rect painted."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = Rect {
            width: self.natural_width(ui).min(area.width),
            ..first_row(area)
        };
        if area.is_empty() {
            return area;
        }

        let live = PartStyle::flags(StateFlags::empty(), StateFlags::BUSY);
        let ov = self.ov;
        let frame = Self::glyph(ui, self.frame);
        let icon = Rect {
            width: width(frame).min(area.width),
            ..area
        };
        if let Some(f) = ov.slot_for(Part::ICON) {
            f(ui, icon);
        } else {
            let s = ov.style(
                ui,
                self.id,
                Family::PROGRESS,
                self.variant,
                Part::ICON,
                live,
            );
            ui.paint_str(icon, frame, s.style);
        }
        if !self.label.is_empty() {
            let gap = self.label_gap(ui);
            let rest = shift(area, icon.width.saturating_add(gap));
            if !rest.is_empty() {
                if let Some(f) = ov.slot_for(Part::LABEL) {
                    f(ui, rest);
                } else {
                    let s = ov.style(
                        ui,
                        self.id,
                        Family::PROGRESS,
                        self.variant,
                        Part::LABEL,
                        live,
                    );
                    ui.paint_str(rest, self.label, s.style);
                }
            }
        }
        area
    }

    #[doc = r" The natural size: one row."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;

    #[test]
    fn percentages_format_without_allocating() {
        assert_eq!(Pct::of(0).as_str(), "0%");
        assert_eq!(Pct::of(7).as_str(), "7%");
        assert_eq!(Pct::of(64).as_str(), "64%");
        assert_eq!(Pct::of(100).as_str(), "100%");
        assert_eq!(Pct::of(4_000).as_str(), "100%");
        assert_eq!(percent(0.644), 64);
        assert_eq!(percent(0.645), 65);
        assert_eq!(percent(2.0), 100);
    }

    #[test]
    fn the_indeterminate_sweep_crosses_the_track_and_stays_inside_it() {
        let bar = ProgressBar::new(Id::root("t"));
        let w = 20u16;
        let mut covered = false;
        for f in 0..64 {
            let (from, to) = ProgressBar { frame: f, ..bar }.filled_span(w);
            assert!(from <= to && to <= w, "frame {f}: {from}..{to}");
            if from > 0 && to == from + (w / 5).clamp(2, 8) {
                covered = true;
            }
        }
        assert!(covered, "the segment never sat wholly inside the track");
    }

    #[test]
    fn a_determinate_span_starts_at_zero_and_rounds() {
        let bar = ProgressBar::new(Id::root("t")).ratio(0.5);
        assert_eq!(bar.filled_span(20), (0, 10));
        assert_eq!(
            ProgressBar::new(Id::root("t")).ratio(1.5).filled_span(20),
            (0, 20)
        );
        assert_eq!(
            ProgressBar::new(Id::root("t")).ratio(-1.0).filled_span(20),
            (0, 0)
        );
    }

    #[test]
    fn a_done_bar_reaches_the_checked_recipe_rule() {
        const AREA: Rect = Rect::new(0, 0, 24, 1);
        const DONE: &str = "D";
        let mut theme = Theme::junie();
        theme.design.glyphs.set(GlyphRole::ProgressDone, DONE);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(AREA);

        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ProgressBar::new(Id::root("progress.done"))
                    .ratio(1.0)
                    .done(true)
                    .draw(ui, area);
            })
            .commit_presented();

        assert!(
            buffer.content().iter().any(|cell| cell.symbol() == DONE),
            "`.done(true)` did not paint the recipe's ProgressDone glyph"
        );
    }
}
