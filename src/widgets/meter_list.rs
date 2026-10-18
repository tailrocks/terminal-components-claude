use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{fit, truncate, width};
use crate::widgets::progress::{Meter, MeterTone};

#[derive(Debug, Clone)]
pub struct MeterRow {
    pub label: String,
    pub meter: Meter,

    pub detail: Option<String>,
}

impl MeterRow {
    pub fn new(label: impl Into<String>, meter: Meter) -> Self {
        Self {
            label: label.into(),
            meter,
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct MeterList {
    pub rows: Vec<MeterRow>,

    pub label_width: Option<u16>,

    pub label_tone: Tone,

    pub label_gap: u16,

    pub pad_label: bool,

    pub meter_width: Option<u16>,

    pub reserve: u16,

    pub min_meter: u16,

    pub detail_gap: u16,

    pub detail_tone: Tone,
}

impl Default for MeterList {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl MeterList {
    pub fn new(rows: Vec<MeterRow>) -> Self {
        Self {
            rows,
            label_width: None,
            label_tone: Tone::Normal,
            label_gap: 1,
            pad_label: false,
            meter_width: None,
            reserve: 0,
            min_meter: 0,
            detail_gap: 2,
            detail_tone: Tone::Muted,
        }
    }

    pub fn label_width(mut self, w: u16) -> Self {
        self.label_width = Some(w);
        self
    }
    pub fn label_tone(mut self, tone: Tone) -> Self {
        self.label_tone = tone;
        self
    }

    pub fn pad_label(mut self, on: bool) -> Self {
        self.pad_label = on;
        self
    }
    pub fn label_gap(mut self, gap: u16) -> Self {
        self.label_gap = gap;
        self
    }

    pub fn meter_width(mut self, w: u16) -> Self {
        self.meter_width = Some(w);
        self
    }
    pub fn reserve(mut self, cells: u16) -> Self {
        self.reserve = cells;
        self
    }
    pub fn min_meter(mut self, w: u16) -> Self {
        self.min_meter = w;
        self
    }
    pub fn detail_gap(mut self, gap: u16) -> Self {
        self.detail_gap = gap;
        self
    }
    pub fn detail_tone(mut self, tone: Tone) -> Self {
        self.detail_tone = tone;
        self
    }

    pub fn height(&self) -> u16 {
        self.rows.len() as u16
    }

    pub fn label_column(&self, area: Rect) -> u16 {
        self.label_width
            .unwrap_or_else(|| {
                self.rows
                    .iter()
                    .map(|r| width(&r.label) as u16)
                    .max()
                    .unwrap_or(0)
            })
            .min(area.width)
    }

    pub fn meter_column(&self, area: Rect) -> (u16, u16) {
        let mx = area.x + self.label_column(area) + self.label_gap;
        let room = area.right().saturating_sub(mx).saturating_sub(self.reserve);
        let mw = self.meter_width.map_or(room, |w| w.min(room));
        if mw == 0 || mw < self.min_meter {
            (mx, 0)
        } else {
            (mx, mw)
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) -> u16 {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return 0;
        }
        let t = ctx.theme;
        let lw = self.label_column(area);
        let (mx, mw) = self.meter_column(area);
        let label_style = Style::new().fg(t.tone(self.label_tone)).bg(bg);
        let detail_style = Style::new().fg(t.tone(self.detail_tone)).bg(bg);
        let mut drawn = 0;
        for (i, row) in self.rows.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            let label = if self.pad_label {
                fit(&row.label, lw as usize)
            } else {
                truncate(&row.label, lw as usize)
            };
            buf.set_string(area.x, y, label, label_style);
            drawn += 1;
            if mw == 0 {
                continue;
            }
            row.meter.render(Rect::new(mx, y, mw, 1), buf, ctx, bg);
            if let Some(d) = &row.detail {
                let dx = mx + mw + self.detail_gap;
                if dx < area.right() {
                    buf.set_string(
                        dx,
                        y,
                        truncate(d, (area.right() - dx) as usize),
                        detail_style,
                    );
                }
            }
        }
        drawn
    }
}

impl crate::ui::measure::Measure for MeterList {
    fn measure(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        let label = self.label_column(Rect::new(0, 0, u16::MAX, 1));
        let meter = self.meter_width.unwrap_or_else(|| {
            self.rows
                .iter()
                .map(|r| r.meter.measure(avail, ctx).width)
                .max()
                .unwrap_or(0)
        });

        let detail = match self.meter_width {
            None => 0,
            Some(_) => self
                .rows
                .iter()
                .filter_map(|r| r.detail.as_deref())
                .map(|d| self.detail_gap + width(d) as u16)
                .max()
                .unwrap_or(0),
        };
        ratatui::layout::Size::new(
            label + self.label_gap + meter.max(self.min_meter) + detail.max(self.reserve),
            self.height(),
        )
    }
}

const SAMPLE_ID: WidgetId = WidgetId::of("meter-list-sample");

impl crate::ui::specimen::Specimen for MeterList {
    fn specimen_id(&self) -> WidgetId {
        SAMPLE_ID
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Loading | Error | Overflow | Narrow)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        self.height().max(1)
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut l = self.clone();
        for r in &mut l.rows {
            match state {
                Loading => r.meter.tone = MeterTone::Refreshing,
                Error => {
                    r.meter.tone = MeterTone::Error;
                    r.meter.value = "read failed".into();
                }
                Overflow => {
                    r.label = format!("{} and a longer name", r.label);
                    r.detail = Some("a detail longer than its column".into());
                }
                _ => {}
            }
        }
        l.render(area, buf, ctx, bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn text(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    #[test]
    fn a_list_at_its_measure_shows_everything() {
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let ctx = RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        let l = MeterList::new(vec![
            MeterRow::new("Session", Meter::new(Some(38)).value("38% used"))
                .detail("resets in 2 h"),
            MeterRow::new(
                "Quota",
                Meter::new(None)
                    .value("quota read failed")
                    .tone(MeterTone::Error),
            ),
            MeterRow::new(
                "Weekly",
                Meter::new(Some(91))
                    .value("91%")
                    .visual(crate::widgets::progress::MeterVisual::Block),
            )
            .detail("of 100 credits"),
        ])
        .meter_width(22)
        .min_meter(8)
        .reserve(2);
        let size = l.measure(ratatui::layout::Size::new(10, 1), &ctx);
        drop(ctx);
        assert_eq!(size.height, 3);
        let buf = draw(&l, size.width);
        let all: Vec<String> = (0..3).map(|y| text(&buf, y)).collect();
        for (y, parts) in [
            &["Session", "38% used", "resets in 2 h"][..],
            &["Quota", "quota read failed"],
            &["Weekly", "91%", "of 100 credits"],
        ]
        .iter()
        .enumerate()
        {
            for p in *parts {
                assert!(all[y].contains(p), "{p:?} in {:?}", all[y]);
            }
        }
    }

    fn list() -> MeterList {
        MeterList::new(vec![
            MeterRow::new("Session", Meter::new(Some(38))).detail("resets in 2 h"),
            MeterRow::new("Weekly", Meter::new(Some(91))).detail("of 100"),
        ])
    }

    fn draw(l: &MeterList, w: u16) -> Buffer {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 3));
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let mut ctx = RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        l.render(buf.area, &mut buf, &mut ctx, t.canvas);
        buf
    }

    #[test]
    fn labels_meters_and_details_line_up_in_columns() {
        let l = list().meter_width(20);
        let buf = draw(&l, 60);

        assert_eq!(l.meter_column(buf.area), (8, 20));
        for y in 0..2 {
            let row = text(&buf, y);

            assert_eq!(buf[(30, y)].fg, Theme::junie().text_muted, "{row}");
        }
        assert!(text(&buf, 0).starts_with("Session "));
        assert!(text(&buf, 1).starts_with("Weekly  "));
        assert!(text(&buf, 0).ends_with("resets in 2 h"));

        assert!(text(&buf, 1).contains('!'));
    }

    #[test]
    fn a_narrow_list_shrinks_the_meter_and_hides_it_below_the_minimum() {
        let l = list()
            .label_width(10)
            .meter_width(24)
            .reserve(4)
            .min_meter(8);
        let area = Rect::new(0, 0, 30, 2);

        assert_eq!(l.meter_column(area), (11, 15));
        let buf = draw(&l, 20);

        assert_eq!(text(&buf, 0), "Session");
    }

    #[test]
    fn a_padded_label_column_paints_the_cells_after_a_short_label() {
        let t = Theme::junie();
        let l = list()
            .label_width(10)
            .meter_width(10)
            .label_tone(Tone::Secondary);
        let buf = draw(&l, 40);

        assert_ne!(buf[(8, 1)].fg, t.text_secondary);
        let buf = draw(&l.pad_label(true), 40);
        assert_eq!(buf[(8, 1)].fg, t.text_secondary);
        assert_eq!(buf[(8, 1)].symbol(), " ");
    }

    #[test]
    fn a_detail_is_truncated_to_the_row() {
        let l = list().meter_width(10).label_width(8);
        let buf = draw(&l, 26);

        assert_eq!(text(&buf, 0).chars().skip(21).collect::<String>(), "rese…");
    }

    #[test]
    fn rows_past_the_area_are_not_drawn() {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let mut ctx = RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        let l = list().meter_width(10);
        let n = l.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
        assert_eq!(n, 1);
        assert_eq!(text(&buf, 1), "");
    }
}
