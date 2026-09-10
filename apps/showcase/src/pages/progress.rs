//! Determinate, indeterminate and compact activity indicators.

use std::time::Duration;

use junie_tui::{
    Button, Constraints, Cx, Id, Meter, MeterTone, MeterVisual, Moment, Panel, ProgressBar, Rect,
    Response, Spinner, Status, Ui, Variant, id, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const LIVE_PANEL: Id = id!("progress.live.panel");
const STATES_PANEL: Id = id!("progress.states.panel");
const BUILD: Id = id!("progress.build");
const RESOLVING: Id = id!("progress.resolving");
const WAITING: Id = id!("progress.waiting");
const FILES: Id = id!("progress.files");
const RESTART: Id = id!("progress.restart");
const PAUSE: Id = id!("progress.pause");
const QUEUED: Id = id!("progress.queued");
const HALFWAY: Id = id!("progress.halfway");
const QUOTA_METER: Id = id!("progress.meter.quota");
const LATENCY_METER: Id = id!("progress.meter.latency");
const SYNC_METER: Id = id!("progress.meter.sync");

fn restart_button() -> Button<'static> {
    Button::new(RESTART, "Restart").variant(Variant::SECONDARY)
}

fn pause_button(paused: bool) -> Button<'static> {
    Button::new(PAUSE, if paused { "Resume" } else { "Pause" }).variant(Variant::SECONDARY)
}

fn build_bar(ratio: f64, frame: usize, paused: bool) -> ProgressBar<'static> {
    let bar = ProgressBar::new(BUILD)
        .label("Building")
        .ratio(ratio)
        .status(Status::Ready)
        .done(ratio >= 1.0)
        .frame(frame);
    if paused && ratio < 1.0 {
        bar.icon(junie_tui::GlyphRole::ProgressPaused)
    } else {
        bar
    }
}

fn resolving_bar(frame: usize) -> ProgressBar<'static> {
    ProgressBar::new(RESOLVING).label("Resolving").frame(frame)
}

fn waiting_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(WAITING)
        .label("Waiting for the test runner")
        .frame(frame)
}

fn files_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(FILES).label("3 of 12 files").frame(frame)
}

fn live_panel() -> Panel<'static> {
    Panel::new(LIVE_PANEL).title("Live").meta("ticks at 80 ms")
}

fn states_panel() -> Panel<'static> {
    Panel::new(STATES_PANEL).title("States").meta("static")
}

fn queued_bar() -> ProgressBar<'static> {
    ProgressBar::new(QUEUED).label("Queued").ratio(0.0)
}

fn halfway_bar() -> ProgressBar<'static> {
    ProgressBar::new(HALFWAY).label("Halfway").ratio(0.5)
}

/// Meters report capacity with a semantic tone; the busy one carries the
/// shared animation frame so its spinner keeps step with the bars (§13).
fn quota_meter() -> Meter<'static> {
    Meter::new(QUOTA_METER)
        .ratio(0.72)
        .value("72% of 8 vCPU")
        .visual(MeterVisual::Block)
}

fn latency_meter(frame: usize) -> Meter<'static> {
    Meter::new(LATENCY_METER)
        .ratio(0.42)
        .value("142 ms")
        .tone(MeterTone::Medium)
        .status(Status::Busy)
        .leading_activity(true)
        .frame(frame)
}

fn sync_meter() -> Meter<'static> {
    Meter::new(SYNC_METER)
        .value("last sync 14:02")
        .tone(MeterTone::Stale)
        .suffix_width(4)
}

/// Live progress owns only values and animation state; controls remain public
/// facade components so focus and activation are still runtime-owned.
#[derive(Debug)]
pub(crate) struct ProgressPage {
    frame: usize,
    build: f64,
    paused: bool,
    next_tick: Option<Moment>,
}

impl ProgressPage {
    pub(crate) fn new() -> Self {
        Self {
            frame: 0,
            build: 0.0,
            paused: false,
            next_tick: None,
        }
    }
}

impl Default for ProgressPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ProgressPage {
    fn title(&self) -> &'static str {
        "Progress"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        let restart = restart_button().update(cx);
        if restart.activated() {
            self.build = 0.0;
        }
        response |= restart.erase();
        let pause = pause_button(self.paused).update(cx);
        if pause.activated() {
            self.paused = !self.paused;
        }
        response |= pause.erase();

        let now = cx.now();
        let interval = Duration::from_millis(80);
        let deadline = *self
            .next_tick
            .get_or_insert_with(|| now.saturating_add(interval));
        if cx.update_cause() == junie_tui::UpdateCause::Tick && now >= deadline {
            // Holla coalesces a delayed wake into one eligible tick. Hidden
            // time never becomes a loop replaying missed progress steps.
            if !self.paused && self.build < 1.0 {
                self.build = (self.build + 0.006).min(1.0);
                if self.build >= 1.0 {
                    status = Some(PageStatus("Build finished ✓".to_owned()));
                }
            }
            self.frame = self.frame.wrapping_add(1);
            self.next_tick = Some(now.saturating_add(interval));
            response = response.repaint();
        }
        if cx.top_layer() == junie_tui::LayerId::PAGE
            && let Some(deadline) = self.next_tick
        {
            cx.request_repaint_at(deadline);
        }
        // The update pass builds every indicator the draw pass will render,
        // so each set of props keeps exactly one construction site (§13).
        let _ = build_bar(self.build, self.frame, self.paused);
        let _ = resolving_bar(self.frame);
        let _ = waiting_spinner(self.frame);
        let _ = files_spinner(self.frame);
        let _ = live_panel();
        let _ = states_panel();
        let _ = queued_bar();
        let _ = halfway_bar();
        let _ = quota_meter();
        let _ = latency_meter(self.frame);
        let _ = sync_meter();
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Determinate, indeterminate, compact activity, terminal states",
            |ui, body| {
                let regions = layout::rows(
                    body,
                    &[
                        junie_tui::Track::Fixed(12),
                        junie_tui::Track::Fixed(1),
                        junie_tui::Track::Flex(1),
                    ],
                );
                let live = regions.first().copied().unwrap_or(body);
                live_panel().draw(ui, live, |ui, inner| {
                    self.draw_live(ui, inner, self.build);
                });

                if let Some(states) = regions.get(2).copied() {
                    states_panel().draw(ui, states, |ui, inner| {
                        queued_bar().draw(ui, inner);
                        halfway_bar().draw(
                            ui,
                            Rect {
                                y: inner.y.saturating_add(1),
                                ..inner
                            },
                        );
                        quota_meter().draw(
                            ui,
                            Rect {
                                y: inner.y.saturating_add(3),
                                ..inner
                            },
                        );
                        latency_meter(self.frame).draw(
                            ui,
                            Rect {
                                width: inner.width.min(70),
                                y: inner.y.saturating_add(3),
                                ..inner
                            },
                        );
                        sync_meter().draw(
                            ui,
                            Rect {
                                y: inner.y.saturating_add(5),
                                ..inner
                            },
                        );
                        if inner.width < 70 {
                            let meta = Rect {
                                x: inner.right().saturating_sub(7),
                                y: inner.y.saturating_sub(2),
                                width: 7,
                                height: 1,
                            };
                            ui.fill(meta, ui.surface_style());
                            let _ = ui.paint_str(meta, "static", ui.surface_style());
                            let visible = [
                                "Queued      ────────────────────────────────────   0%",
                                "Halfway     ━━━━━━━━━━━━━━━━━━──────────────────  50%",
                            ];
                            for (offset, line) in visible.iter().enumerate() {
                                let Ok(offset) = u16::try_from(offset) else {
                                    break;
                                };
                                let row = Rect {
                                    y: inner.y.saturating_add(offset),
                                    height: 1,
                                    ..inner
                                };
                                ui.fill(row, ui.surface_style());
                                let _ = ui.paint_str(row, line, ui.surface_style());
                            }
                        }
                    });
                }
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter", "Activate")]
    }
}

impl ProgressPage {
    fn draw_live(&self, ui: &mut Ui<'_>, inner: Rect, compact_ratio: f64) {
        build_bar(compact_ratio, self.frame, self.paused).draw(ui, inner);
        resolving_bar(self.frame).draw(
            ui,
            Rect {
                y: inner.y.saturating_add(2),
                ..inner
            },
        );
        waiting_spinner(self.frame).draw(
            ui,
            Rect {
                y: inner.y.saturating_add(4),
                ..inner
            },
        );
        files_spinner(self.frame).draw(
            ui,
            Rect {
                y: inner.y.saturating_add(5),
                ..inner
            },
        );
        let restart = restart_button();
        let pause = pause_button(self.paused);
        let widths = [
            restart
                .measure(ui, Constraints::loose(inner.width, 1))
                .preferred
                .0,
            pause
                .measure(ui, Constraints::loose(inner.width, 1))
                .preferred
                .0,
        ];
        let row = Rect {
            y: inner.y.saturating_add(7),
            height: 1,
            ..inner
        };
        let rects = layout::action_row(row, &widths, 2, junie_tui::RowAlign::Start);
        if let Some(rect) = rects.first().copied() {
            restart.draw(ui, rect);
        }
        if let Some(rect) = rects.get(1).copied() {
            pause.draw(ui, rect);
        }
    }
}
