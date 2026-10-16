use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{
    Meter, MeterLevel, MeterTone, MeterVisual, ProgressStatus, StaleStyle, render_bar,
    render_indeterminate, render_spinner,
};

const ID: WidgetId = WidgetId::of("progress");

pub struct ProgressPage {
    build: f64,
    running: bool,
    restart: Button,
    pause: Button,
    paused: bool,
}

impl ProgressPage {
    pub fn new() -> Self {
        Self {
            build: 0.0,
            running: true,
            restart: Button::secondary(ID.sub("restart"), "Restart"),
            pause: Button::secondary(ID.sub("pause"), "Pause"),
            paused: false,
        }
    }
}

impl Page for ProgressPage {
    fn title(&self) -> &'static str {
        "Progress"
    }
    fn blurb(&self) -> &'static str {
        "Determinate, indeterminate, compact activity, terminal states"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[12, 1, 0]);
        let panel = Panel::card(Some("Live")).meta("ticks at 80 ms");
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let w = inner.width.min(70);
        let status = if self.build >= 1.0 {
            ProgressStatus::Done
        } else if self.paused {
            ProgressStatus::Paused
        } else {
            ProgressStatus::Active
        };
        render_bar(
            Rect::new(inner.x, inner.y, w, 1),
            buf,
            ctx,
            "Building  ",
            self.build,
            status,
            bg,
        );
        render_indeterminate(
            Rect::new(inner.x, inner.y + 2, w, 1),
            buf,
            ctx,
            "Resolving ",
            bg,
        );
        render_spinner(
            Rect::new(inner.x, inner.y + 4, w, 1),
            buf,
            ctx,
            "Waiting for the test runner",
            bg,
        );
        let spin = junie_tui::widgets::progress::spinner_frame(ctx.interaction.tick);
        buf.set_string(
            inner.x,
            inner.y + 5,
            format!("{spin} 3 of 12 files"),
            t.secondary().bg(bg),
        );
        buf.set_string(inner.x, inner.y + 5, spin, t.accent_fg().bg(bg));
        let rects = junie_tui::widgets::button::row_layout(
            Rect::new(inner.x, inner.y + 7, inner.width, 1),
            &[self.restart.width(), self.pause.width()],
            2,
        );
        self.restart.render(rects[0], buf, ctx, bg);
        self.pause.render(rects[1], buf, ctx, bg);

        let panel = Panel::card(Some("States")).meta("static");
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(rows[2].x, rows[2].y, rows[2].width, rows[2].height.min(22)),
            buf,
            t,
        );
        let w = inner.width.min(70);
        let samples = [
            ("Queued    ", 0.0, ProgressStatus::Active),
            ("Halfway   ", 0.5, ProgressStatus::Active),
            ("Completed ", 1.0, ProgressStatus::Done),
            ("Failed    ", 0.64, ProgressStatus::Error),
            ("Paused    ", 0.3, ProgressStatus::Paused),
        ];
        for (i, (label, r, s)) in samples.iter().enumerate() {
            let y = inner.y + i as u16;
            if y < inner.bottom() {
                render_bar(Rect::new(inner.x, y, w, 1), buf, ctx, label, *r, *s, bg);
            }
        }
        if inner.height > 6 {
            buf.set_string(
                inner.x,
                inner.y + 6,
                "Narrow bars keep the percentage and drop the label:",
                t.muted().bg(bg),
            );
            render_bar(
                Rect::new(inner.x, inner.y + 7, 14, 1),
                buf,
                ctx,
                "",
                0.42,
                ProgressStatus::Active,
                bg,
            );
        }
        if inner.height > 13 {
            buf.set_string(
                inner.x,
                inner.y + 9,
                junie_tui::ui::text::truncate(
                    "Capacity meters are never green: low ≤ 59 % blue, medium ≤ 84 % warning ▲, high error !. Stale keeps its level, dashed.",
                    inner.width as usize,
                ),
                t.muted().bg(bg),
            );
            // left: one meter per tone, line visual; right: the options
            // (block visual, owner thresholds, a fixed level, staleness on
            // a domain tone, the faint stale style). Red appears only as a
            // run or a glyph, never as a filled block (colour follow-ups).
            let meters: [(&str, Option<u8>, MeterTone); 9] = [
                ("Low       ", Some(38), MeterTone::Normal),
                ("Medium    ", Some(72), MeterTone::Normal),
                ("High      ", Some(91), MeterTone::Normal),
                ("Warning   ", Some(82), MeterTone::Warning),
                ("Exhausted ", Some(100), MeterTone::Exhausted),
                ("Stale     ", Some(76), MeterTone::Stale),
                ("Refreshing", Some(54), MeterTone::Refreshing),
                ("Error     ", None, MeterTone::Error),
                ("Unknown   ", None, MeterTone::Unknown),
            ];
            let disk = MeterTone::Level { warn: 80, crit: 95 };
            let options: [Meter; 6] = [
                Meter::new(Some(38))
                    .value("38% used · block")
                    .visual(MeterVisual::Block),
                Meter::new(Some(72))
                    .value("72% disk, warns at 80")
                    .tone(disk),
                Meter::new(Some(88))
                    .value("88% disk, critical at 95")
                    .tone(disk),
                Meter::new(Some(30))
                    .value("30% · fixed high")
                    .tone(MeterTone::Fixed(MeterLevel::High)),
                Meter::new(Some(82))
                    .value("82% · stale warning")
                    .tone(MeterTone::Warning)
                    .stale(true),
                Meter::new(Some(76))
                    .value("76% · faint stale style")
                    .tone(MeterTone::Stale)
                    .stale_style(StaleStyle::Faint),
            ];
            let col_w = inner.width.saturating_sub(12) / 2;
            for (i, (label, pct, tone)) in meters.iter().enumerate() {
                let y = inner.y + 10 + i as u16;
                if y >= inner.bottom() {
                    break;
                }
                buf.set_string(inner.x, y, label, t.primary().bg(bg));
                let value = match tone {
                    MeterTone::Error => "quota read failed".to_owned(),
                    MeterTone::Unknown => String::new(),
                    _ => format!("{}% used", pct.unwrap_or(0)),
                };
                Meter::new(*pct).value(value).tone(*tone).render(
                    Rect::new(inner.x + 11, y, col_w.min(34), 1),
                    buf,
                    ctx,
                    bg,
                );
                if let Some(m) = options.get(i) {
                    let x = inner.x + 11 + col_w + 1;
                    m.render(Rect::new(x, y, col_w.min(34), 1), buf, ctx, bg);
                }
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Tick => {
                if self.running && !self.paused && self.build < 1.0 {
                    self.build = (self.build + 0.006).min(1.0);
                    if self.build >= 1.0 {
                        cx.status("Build finished ✓");
                    }
                    return Outcome::Changed;
                }
                Outcome::Changed
            }
            PageEvent::Key(key) => {
                if cx.focus.is(self.restart.id) {
                    let (o, act) = self.restart.on_key(key);
                    if act {
                        self.build = 0.0;
                        self.running = true;
                    }
                    return o;
                }
                if cx.focus.is(self.pause.id) {
                    let (o, act) = self.pause.on_key(key);
                    if act {
                        self.paused = !self.paused;
                        self.pause.label = if self.paused {
                            "Resume".into()
                        } else {
                            "Pause".into()
                        };
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Click { id, .. } => {
                if *id == self.restart.id {
                    if self.restart.on_click() {
                        self.build = 0.0;
                        self.running = true;
                    }
                    return Outcome::Changed;
                }
                if *id == self.pause.id {
                    if self.pause.on_click() {
                        self.paused = !self.paused;
                        self.pause.label = if self.paused {
                            "Resume".into()
                        } else {
                            "Pause".into()
                        };
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn animating(&self) -> bool {
        true
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![("Enter", "Activate")]
    }
}
