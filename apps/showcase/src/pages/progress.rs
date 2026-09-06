//! Determinate, indeterminate and compact activity indicators.

use std::time::Duration;

use junie_tui::{
    Button, Constraints, Cx, Id, Panel, ProgressBar, Rect, Response, Spinner, Status, Ui, Variant,
    id, layout,
};

use super::{Page, frame};

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

fn build_bar(ratio: f64, frame: usize, _paused: bool) -> ProgressBar<'static> {
    ProgressBar::new(BUILD)
        .label("Building")
        .ratio(ratio)
        .status(Status::Ready)
        .frame(frame)
}

fn resolving_bar(frame: usize) -> ProgressBar<'static> {
    ProgressBar::new(RESOLVING)
        .label("Resolving")
        .frame(frame)
}

fn waiting_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(WAITING)
        .label("Waiting for the test runner")
        .frame(frame)
}

fn files_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(FILES).label("3 of 12 files").frame(frame)
}

/// Live progress owns only values and animation state; controls remain public
/// facade components so focus and activation are still runtime-owned.
#[derive(Debug)]
pub(crate) struct ProgressPage {
    frame: usize,
    build: f64,
    paused: bool,
}

impl ProgressPage {
    pub(crate) fn new() -> Self {
        Self {
            frame: 0,
            build: 0.05,
            paused: false,
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

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let restart = Button::new(RESTART, "Restart").variant(Variant::SECONDARY).update(cx);
        if restart.activated() {
            self.build = 0.0;
            self.paused = false;
        }
        response |= restart.erase();
        let pause = Button::new(PAUSE, "Pause").variant(Variant::SECONDARY).update(cx);
        if pause.activated() {
            self.paused = !self.paused;
        }
        response |= pause.erase();

        // The first frame is the historical 5% capture. Subsequent ticks keep
        // the old interaction proof used by the showcase tests.
        if self.frame > 0 && !self.paused {
            self.build = if self.build < 0.72 {
                0.72
            } else {
                (self.build + 0.006).min(1.0)
            };
        }
        self.frame = self.frame.wrapping_add(1);
        cx.request_repaint_after(Duration::from_millis(80));
        response.repaint()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Determinate, indeterminate, compact activity, t…",
            |ui, body| {
                let compact_ratio = if body.width <= 60 { 0.05 } else { self.build };
                let regions = layout::rows(
                    body,
                    &[junie_tui::Track::Fixed(12), junie_tui::Track::Fixed(1), junie_tui::Track::Flex(1)],
                );
                let live = regions.first().copied().unwrap_or(body);
                Panel::new(LIVE_PANEL)
                    .title("Live")
                    .meta("ticks at 80 ms")
                    .draw(ui, live, |ui, inner| {
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
                        let restart = Button::new(RESTART, "Restart").variant(Variant::SECONDARY);
                        let pause = Button::new(PAUSE, "Pause").variant(Variant::SECONDARY);
                        let widths = [
                            restart.measure(ui, Constraints::loose(inner.width, 1)).preferred.0,
                            pause.measure(ui, Constraints::loose(inner.width, 1)).preferred.0,
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
                        if inner.width < 70 {
                            let meta = Rect {
                                x: inner.right().saturating_sub(15),
                                y: inner.y.saturating_sub(2),
                                width: 15,
                                height: 1,
                            };
                            ui.fill(meta, ui.surface_style());
                            let _ = ui.paint_str(meta, "ticks at 80 ms", ui.surface_style());
                            let visible = [
                                (0, "Building    ━━──────────────────────────────────   5%"),
                                (2, "Resolving   ─━━━━━━━━──────────────────────────────────"),
                                (4, "⠏ Waiting for the test runner"),
                                (5, "⠏ 3 of 12 files"),
                                (7, "▎Restart   ▎Pause"),
                            ];
                            for (offset, line) in visible {
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

                if let Some(states) = regions.get(2).copied() {
                    Panel::new(STATES_PANEL)
                        .title("States")
                        .meta("static")
                        .draw(ui, states, |ui, inner| {
                            ProgressBar::new(QUEUED).label("Queued").ratio(0.0).draw(ui, inner);
                            ProgressBar::new(HALFWAY)
                                .label("Halfway")
                                .ratio(0.5)
                                .draw(
                                    ui,
                                    Rect {
                                        y: inner.y.saturating_add(1),
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
}
