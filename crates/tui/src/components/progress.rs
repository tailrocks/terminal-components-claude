//! `ProgressBar` and `Spinner` (`COMPONENT_ARCHITECTURE.md` §18.2,
//! Appendix A 4G).
//!
//! The legacy `render_bar` / `render_indeterminate` / `render_spinner` free
//! functions and their five `bg: Color` parameters are gone: the plane comes
//! from the surface the caller draws on, the spinner frames from
//! `design.motion.spinner_frames` and the track glyphs from
//! `GlyphRole::{RuleQuiet, RuleActive}` (A4, §11.5).

use core::fmt;

use ratatui_core::layout::{Position, Rect};

use super::{Overrides, SlotFn, first_row, shift};
use crate::collection::Status;
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, GlyphRole, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

/// Columns the percentage column occupies: `"100%"` is the widest value.
pub(crate) const PCT_COLUMNS: u16 = 4;

/// A percentage rendered into a fixed stack buffer.
///
/// A component may not allocate once per frame (§20.9-6, R5) and there is no
/// `Ui::paint_fmt`, so the two digits are produced by division rather than by
/// `format!`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pct {
    buf: [u8; 4],
    len: usize,
}

impl Pct {
    /// `v` percent, clamped to `0..=100`.
    pub(crate) const fn of(v: u16) -> Self {
        let v = if v > 100 { 100 } else { v };
        let mut buf = [0u8; 4];
        let mut len;
        if v >= 100 {
            buf[0] = b'1';
            buf[1] = b'0';
            buf[2] = b'0';
            len = 3;
        } else if v >= 10 {
            buf[0] = b'0' + (v / 10) as u8;
            buf[1] = b'0' + (v % 10) as u8;
            len = 2;
        } else {
            buf[0] = b'0' + v as u8;
            len = 1;
        }
        buf[len] = b'%';
        len += 1;
        Pct { buf, len }
    }

    /// The rendered text, `"0%"` … `"100%"`.
    pub(crate) fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

/// Paint a run of one glyph across `run`, one cell at a time (R-4: a single
/// pass over columns, never a nested `for y … for x`).
pub(crate) fn run_of(ui: &mut Ui<'_>, run: Rect, glyph: GlyphRole, style: ratatui_core::style::Style) {
    let sym = ui.glyph_str(glyph);
    for col in run.columns() {
        ui.paint_cell(Position::new(col.x, run.y), sym, style);
    }
}

/// A one-row progress bar: `label ━━━━━━────── 64% ✓`.
///
/// ## Construction
/// `ProgressBar::new(id)`. `.ratio(f)` makes it determinate; without one it
/// is indeterminate and paints a sweeping segment.
///
/// ## Ownership
/// Stateless. The caller owns the label, the ratio and the animation frame;
/// the runtime owns nothing — a progress bar is never focused, hovered or
/// pressed.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.label(&str)`
/// (empty), `.ratio(f64)` (none — indeterminate; clamped to `0.0..=1.0`),
/// `.status(Status)` (`Ready`), `.done(bool)` (`false`),
/// `.icon(GlyphRole)` (none — the explicit override that expresses a paused
/// bar as `GlyphRole::ProgressPaused`), `.frame(usize)` (`0`), `.patch`,
/// `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::PROGRESS`; `DEFAULT` only.
///
/// ## States
/// Derives `BUSY`/`LOADING`/`ERROR` from `.status(Status)` and `CHECKED`
/// from `.done(true)`; wears no runtime state. The recipe keys the trailing
/// glyph on exactly those: `ERROR` → `GlyphRole::Error`, `CHECKED` →
/// `GlyphRole::ProgressDone`.
///
/// ## Actions
/// None; `ProgressBar` has no `update` phase.
///
/// ## Focus
/// Never a focus stop; registers no ring entry and no region.
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None.
///
/// ## Layout
/// `measure` returns `(label + gap + a six-cell minimum track + percentage +
/// icon, 1)`. `draw` uses the first row of `area`; when the track would be
/// narrower than six cells it paints the percentage alone, exactly as the
/// legacy bar did. Returns the rect it painted; a degenerate rect paints
/// nothing (R5).
///
/// ## Parts
/// `LABEL` (the leading label), `TRACK` (the unfilled remainder), `THUMB`
/// (the filled run), `META` (the percentage), `ICON` (the trailing state
/// glyph or the spinner).
///
/// ## Overrides
/// `.patch`, `.patch_part` and `.slot` on any part; `TRACK` and `THUMB`
/// cannot be replaced independently by a slot — a slot on `TRACK` replaces
/// the whole track, filled run included, because the split between them is
/// the bar's own arithmetic.
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `ProgressBarCase` with no capabilities;
/// `render::components::progress_bar::{default, busy, error, empty}`.
///
/// ## Invariants
/// Discharges §11.4's readiness obligation: a `BUSY`/`LOADING` bar paints
/// `Part::ICON` with `design.motion.spinner_frames`, which is a *symbol* and
/// therefore survives `ColorLevel::Mono`. The frame is a prop, so two draws
/// with the same props are byte-identical. Never allocates.
pub struct ProgressBar<'a> {
    id: Id,
    label: &'a str,
    ratio: Option<f64>,
    variant: Variant,
    status: Status,
    done: bool,
    icon: Option<GlyphRole>,
    frame: usize,
    ov: Overrides<'a>,
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
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::LABEL,
        Part::TRACK,
        Part::THUMB,
        Part::META,
        Part::ICON,
    ];

    /// The smallest track worth painting; below it the bar reports the
    /// percentage alone.
    const MIN_TRACK: u16 = 6;

    /// An indeterminate bar.
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

    /// A leading label.
    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    /// Make the bar determinate at `r`, clamped to `0.0..=1.0`.
    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    /// Data readiness; `Busy`/`Loading` paint the spinner, `Error` the error
    /// glyph.
    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// Completion: the recipe's `CHECKED` glyph.
    #[must_use]
    pub const fn done(mut self, yes: bool) -> Self {
        self.done = yes;
        self
    }

    /// Override the trailing glyph. The explicit way to express a suspended
    /// bar (`GlyphRole::ProgressPaused`) without a second lifecycle enum
    /// beside `Status` (§13: no boolean parameter soup).
    #[must_use]
    pub const fn icon(mut self, g: GlyphRole) -> Self {
        self.icon = Some(g);
        self
    }

    /// The animation frame the spinner and the indeterminate sweep read.
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

    /// Columns the label and its gap occupy at `w`, or `0` when the label
    /// does not earn its place.
    fn label_columns(&self, ui: &Ui<'_>, w: u16) -> u16 {
        let lw = width(self.label);
        let gap = ui.design().space.gap.max(1);
        if lw == 0 || w <= lw.saturating_add(gap).saturating_add(Self::MIN_TRACK) {
            0
        } else {
            lw.saturating_add(gap)
        }
    }

    /// Columns the percentage and the trailing glyph occupy.
    fn tail_columns(&self) -> u16 {
        // one space, `100%`, one space, the glyph
        match self.ratio {
            Some(_) => PCT_COLUMNS.saturating_add(3),
            None => 2,
        }
    }

    /// The draw phase; returns the rect painted.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over label, track, percentage and trailing glyph"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let live = self.ov.flags(self.flags());
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
            // too narrow for a meaningful bar: the percentage alone
            if let Some(r) = self.ratio {
                let s = style(ui, Part::META);
                let pct = Pct::of(percent(r));
                ui.paint_str(body, pct.as_str(), s.style);
            }
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
        if let Some(r) = self.ratio {
            let s = style(ui, Part::META);
            let pct = Pct::of(percent(r));
            let pad = PCT_COLUMNS.saturating_sub(width(pct.as_str()));
            let cell = Rect {
                x: x.saturating_add(pad),
                width: area.right().saturating_sub(x.saturating_add(pad)),
                ..area
            };
            ui.paint_str(cell, pct.as_str(), s.style);
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
                } else if let Some(g) = self.icon.or(s.glyph) {
                    ui.glyph(icon_cell, g, s.style);
                }
            }
        }
        area
    }

    /// `(first, last)` filled column of a `w`-wide track.
    fn filled_span(&self, w: u16) -> (u16, u16) {
        match self.ratio {
            Some(r) => (0, (f64::from(w) * r).round() as u16),
            None => {
                // the indeterminate sweep: a short segment crossing the track
                let seg = (w / 5).clamp(2, 8);
                let period = usize::from(w.saturating_add(seg)).max(1);
                let pos = self.frame.checked_rem(period).unwrap_or(0) as i32
                    - i32::from(seg);
                let from = pos.max(0) as u16;
                let to = (pos + i32::from(seg)).clamp(0, i32::from(w)) as u16;
                (from.min(w), to)
            }
        }
    }

    /// The natural size: one row wide enough for a usable track.
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

/// A ratio as whole percent.
fn percent(r: f64) -> u16 {
    (r.clamp(0.0, 1.0) * 100.0).round() as u16
}

/// The compact activity state: `⠋ label`.
///
/// ## Construction
/// `Spinner::new(id)`; `.label(&str)` adds the text beside it.
///
/// ## Ownership
/// Stateless. The caller owns the label and the animation frame; the runtime
/// owns nothing.
///
/// ## Configuration
/// `.variant(Variant)` (default `Recipe.default_variant`), `.label(&str)`
/// (empty), `.frame(usize)` (`0`), `.patch`, `.patch_part`, `.slot`,
/// `.state_override`.
///
/// ## Variants
/// `Family::PROGRESS`; `DEFAULT` only.
///
/// ## States
/// None. A spinner is always spinning: which frame it shows is a *prop*, not
/// a state, so a digest is a pure function of the props.
///
/// ## Actions
/// None; `Spinner` has no `update` phase.
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
/// `measure` returns `(glyph + gap + label, 1)`. `draw` uses the first row of
/// `area` and returns the rect it painted; a degenerate rect paints nothing
/// (R5).
///
/// ## Parts
/// `ICON` (the frame), `LABEL` (the text).
///
/// ## Overrides
/// `.patch`, `.patch_part` and `.slot` on both parts.
///
/// ## Identity
/// One `Id` per instance; no items.
///
/// ## Testing
/// `SpinnerCase` with no capabilities;
/// `render::components::spinner::{default, empty}`.
///
/// ## Invariants
/// Reads `design.motion.spinner_frames`, never a baked-in table (A4). Never
/// allocates; never writes outside `area`.
pub struct Spinner<'a> {
    id: Id,
    label: &'a str,
    variant: Variant,
    frame: usize,
    ov: Overrides<'a>,
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
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::ICON, Part::LABEL];

    /// A spinner with no label.
    pub const fn new(id: Id) -> Self {
        Spinner {
            id,
            label: "",
            variant: Variant::DEFAULT,
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

    /// The text beside the frame.
    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    /// The animation frame.
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

    /// The frame this spinner shows.
    fn glyph(ui: &Ui<'_>, frame: usize) -> &'static str {
        let frames = ui.design().motion.spinner_frames;
        frames
            .get(frame.checked_rem(frames.len()).unwrap_or(0))
            .copied()
            .unwrap_or("")
    }

    fn natural_width(&self, ui: &Ui<'_>) -> u16 {
        let g = width(Self::glyph(ui, self.frame));
        if self.label.is_empty() {
            return g;
        }
        g.saturating_add(ui.design().space.gap.max(1))
            .saturating_add(width(self.label))
    }

    /// The draw phase; returns the rect painted.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let area = Rect {
            width: self.natural_width(ui).min(area.width),
            ..first_row(area)
        };
        if area.is_empty() {
            return area;
        }
        let live = self.ov.flags(StateFlags::BUSY);
        let ov = self.ov;
        let frame = Self::glyph(ui, self.frame);
        let icon = Rect {
            width: width(frame).min(area.width),
            ..area
        };
        if let Some(f) = ov.slot_for(Part::ICON) {
            f(ui, icon);
        } else {
            let s = ov.style(ui, self.id, Family::PROGRESS, self.variant, Part::ICON, live);
            ui.paint_str(icon, frame, s.style);
        }
        if !self.label.is_empty() {
            let gap = ui.design().space.gap.max(1);
            let rest = shift(area, icon.width.saturating_add(gap));
            if !rest.is_empty() {
                if let Some(f) = ov.slot_for(Part::LABEL) {
                    f(ui, rest);
                } else {
                    let s =
                        ov.style(ui, self.id, Family::PROGRESS, self.variant, Part::LABEL, live);
                    ui.paint_str(rest, self.label, s.style);
                }
            }
        }
        area
    }

    /// The natural size: one row.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        Size::exact(self.natural_width(ui), 1).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ProgressBar::new(Id::root("t")).ratio(1.5).filled_span(20), (0, 20));
        assert_eq!(ProgressBar::new(Id::root("t")).ratio(-1.0).filled_span(20), (0, 0));
    }
}
