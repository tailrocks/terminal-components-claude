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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterTone {
    #[default]
    Low,
    Medium,
    High,
    Stale,
    Unknown,
    Series(u8),
}

impl MeterTone {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub enum MeterVisual {
    #[default]
    Line,
    Block,
}

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
    pub const PARTS: &'static [Part] = &[Part::TRACK, Part::THUMB, Part::LABEL, Part::ICON];

    const MIN_TRACK: u16 = 6;

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

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub fn ratio(mut self, r: f64) -> Self {
        self.ratio = Some(r.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub const fn value(mut self, s: &'a str) -> Self {
        self.value = s;
        self
    }

    #[must_use]
    pub const fn tone(mut self, t: MeterTone) -> Self {
        self.tone = Some(t);
        self
    }

    #[must_use]
    pub const fn visual(mut self, v: MeterVisual) -> Self {
        self.visual = v;
        self
    }

    #[must_use]
    pub const fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub const fn suffix_width(mut self, cells: u16) -> Self {
        self.suffix_width = cells;
        self
    }

    #[must_use]
    pub const fn leading_activity(mut self, yes: bool) -> Self {
        self.leading_activity = yes;
        self
    }

    #[must_use]
    pub const fn frame(mut self, f: usize) -> Self {
        self.frame = f;
        self
    }

    #[must_use]
    pub const fn part_defaults(mut self, defaults: &'a [(Part, StylePatch)]) -> Self {
        self.part_defaults = defaults;
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

    fn paint_value(&self, ui: &mut Ui<'_>, cell: Rect, value: &str, style: PaintStyle) -> u16 {
        if let Some(f) = self.ov.slot_for(Part::LABEL) {
            f(ui, cell);
            return width(value).min(cell.width);
        }
        ui.paint_str(cell, value, style)
    }

    fn paint_icon(&self, ui: &mut Ui<'_>, cell: Rect, glyph: &str, style: PaintStyle) {
        if let Some(f) = self.ov.slot_for(Part::ICON) {
            f(ui, cell);
            return;
        }
        ui.paint_str(cell, glyph, style);
    }

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
