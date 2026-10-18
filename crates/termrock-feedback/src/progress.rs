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

pub(crate) const PCT_COLUMNS: u16 = 4;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pct {
    buf: [u8; 4],
    len: usize,
}

impl Pct {
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

    pub(crate) fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

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
    pub const PARTS: &'static [Part] = &[
        Part::LABEL,
        Part::TRACK,
        Part::THUMB,
        Part::META,
        Part::ICON,
    ];

    const MIN_TRACK: u16 = 6;

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

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub const fn done(mut self, yes: bool) -> Self {
        self.done = yes;
        self
    }

    #[must_use]
    pub const fn icon(mut self, g: GlyphRole) -> Self {
        self.icon = Some(g);
        self
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

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

    fn label_columns(&self, ui: &Ui<'_>, w: u16) -> u16 {
        let lw = width(self.label);
        let gap = ui.design().space.gap.max(1);
        if lw == 0 || w <= lw.saturating_add(gap).saturating_add(Self::MIN_TRACK) {
            0
        } else {
            lw.saturating_add(gap)
        }
    }

    fn tail_columns(&self) -> u16 {
        match self.ratio {
            Some(_) => PCT_COLUMNS.saturating_add(3),
            None => 2,
        }
    }

    fn pct_width(&self) -> u16 {
        self.ratio
            .map_or(0, |r| width(Pct::of(percent(r)).as_str()))
    }

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

fn percent(r: f64) -> u16 {
    (r.clamp(0.0, 1.0) * 100.0).round() as u16
}

pub struct Spinner<'a> {
    id: Id,
    label: &'a str,
    gap: Option<u16>,
    variant: Variant,
    frame: usize,
    ov: PartStyle<'a>,
}

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
    pub const PARTS: &'static [Part] = &[Part::ICON, Part::LABEL];

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

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn label(mut self, s: &'a str) -> Self {
        self.label = s;
        self
    }

    #[must_use]
    pub const fn gap(mut self, cells: u16) -> Self {
        self.gap = Some(cells);
        self
    }

    fn label_gap(&self, ui: &Ui<'_>) -> u16 {
        self.gap.unwrap_or_else(|| ui.design().space.gap.max(1))
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

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
