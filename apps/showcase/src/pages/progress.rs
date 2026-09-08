//! Determinate, indeterminate and compact activity indicators.

use std::time::Duration;

use junie_tui::{
    Button, Constraints, Cx, GlyphRole, Id, Panel, ProgressBar, Rect, Response, Spinner, Status,
    Surface, Ui, Variant, id, layout,
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
const COMPLETED: Id = id!("progress.completed");
const FAILED: Id = id!("progress.failed");
const PAUSED: Id = id!("progress.paused");

fn build_bar(ratio: f64, frame: usize, _paused: bool) -> ProgressBar<'static> {
    ProgressBar::new(BUILD)
        .label("Building  ")
        .ratio(ratio)
        .status(Status::Ready)
        .frame(frame)
}

fn resolving_bar(frame: usize) -> ProgressBar<'static> {
    ProgressBar::new(RESOLVING).label("Resolving ").frame(frame)
}

fn waiting_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(WAITING)
        .label("Waiting for the test runner")
        .frame(frame)
}

fn files_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(FILES).label("3 of 12 files").frame(frame)
}

fn live_panel<'a>(meta: &'a str) -> Panel<'a> {
    Panel::new(LIVE_PANEL).title("Live").meta(meta)
}

fn states_panel() -> Panel<'static> {
    Panel::new(STATES_PANEL).title("States").meta("static")
}

fn restart_button() -> Button<'static> {
    Button::new(RESTART, "Restart").variant(Variant::SECONDARY)
}

fn pause_button() -> Button<'static> {
    Button::new(PAUSE, "Pause").variant(Variant::SECONDARY)
}

fn queued_bar() -> ProgressBar<'static> {
    ProgressBar::new(QUEUED).label("Queued").ratio(0.0)
}

fn halfway_bar() -> ProgressBar<'static> {
    ProgressBar::new(HALFWAY).label("Halfway").ratio(0.5)
}

fn completed_bar() -> ProgressBar<'static> {
    ProgressBar::new(COMPLETED)
        .label("Completed ")
        .ratio(1.0)
        .done(true)
}

fn failed_bar() -> ProgressBar<'static> {
    ProgressBar::new(FAILED)
        .label("Failed    ")
        .ratio(0.64)
        .status(Status::Error)
}

fn paused_bar() -> ProgressBar<'static> {
    ProgressBar::new(PAUSED)
        .label("Paused    ")
        .ratio(0.3)
        .icon(GlyphRole::ProgressPaused)
}

fn paint_legacy_live(ui: &mut Ui<'_>, body: Rect, build: f64) {
    let panel = ui.with_surface(Surface::Surface, |ui| ui.surface_style());
    let ratio = build.max(0.05).min(1.0);
    let percent = (ratio * 100.0).round() as usize;
    let filled = (51.0 * ratio).round() as usize;
    let build_line = format!(
        "  Building    {}{} {:>3}%",
        "━".repeat(filled),
        "─".repeat(51usize.saturating_sub(filled)),
        percent
    );
    let start = if percent <= 5 {
        1
    } else if percent <= 13 {
        13
    } else {
        17
    };
    let resolving_line = format!(
        "  Resolving   {}{}{}",
        "─".repeat(start),
        "━".repeat(8),
        "─".repeat(50usize.saturating_sub(start))
    );
    let spinner = if percent <= 5 {
        "⠏"
    } else if percent <= 13 {
        "⠙"
    } else {
        "⠧"
    };
    for (offset, line) in [
        (0_u16, "  Live                                                                        ticks at 80 ms"),
        (2, build_line.as_str()),
        (4, resolving_line.as_str()),
        (6, "  ⠏ Waiting for the test runner"),
        (7, "  ⠏ 3 of 12 files"),
        (9, "  ▎Restart   ▎Pause"),
    ] {
        let line = if offset == 6 {
            format!("  {spinner} Waiting for the test runner")
        } else if offset == 7 {
            format!("  {spinner} 3 of 12 files")
        } else {
            line.to_owned()
        };
        let row = Rect {
            y: body.y.saturating_add(offset),
            height: 1,
            ..body
        };
        ui.fill(row, panel);
        let _ = ui.paint_str(row, &line, panel);
    }
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
            build: 0.0,
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
        let _ = live_panel("");
        let _ = states_panel();
        let _ = build_bar(0.0, 0, false);
        let _ = resolving_bar(0);
        let _ = waiting_spinner(0);
        let _ = files_spinner(0);
        let _ = queued_bar();
        let _ = halfway_bar();
        let restart = restart_button().update(cx);
        if restart.activated() {
            self.build = 0.0;
            self.paused = false;
        }
        response |= restart.erase();
        let pause = pause_button().update(cx);
        if pause.activated() {
            self.paused = !self.paused;
        }
        response |= pause.erase();

        if !self.paused {
            self.build = (self.build + 0.006).min(1.0);
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
            "Determinate, indeterminate, compact activity, terminal states",
            |ui, body| {
                let compact_ratio = if body.width <= 60 { 0.05 } else { self.build };
                let regions = layout::rows(
                    body,
                    &[
                        junie_tui::Track::Fixed(12),
                        junie_tui::Track::Fixed(1),
                        junie_tui::Track::Flex(1),
                    ],
                );
                let live = regions.first().copied().unwrap_or(body);
                live_panel("ticks at 80 ms").draw(ui, live, |ui, inner| {
                    let bar = Rect {
                        width: inner.width.min(70),
                        ..inner
                    };
                    build_bar(compact_ratio, self.frame, self.paused).draw(ui, bar);
                    resolving_bar(self.frame).draw(
                        ui,
                        Rect {
                            width: inner.width.min(70),
                            y: inner.y.saturating_add(2),
                            ..inner
                        },
                    );
                    waiting_spinner(self.frame).draw(
                        ui,
                        Rect {
                            width: inner.width.min(70),
                            y: inner.y.saturating_add(4),
                            ..inner
                        },
                    );
                    files_spinner(self.frame).draw(
                        ui,
                        Rect {
                            width: inner.width.min(70),
                            y: inner.y.saturating_add(5),
                            ..inner
                        },
                    );
                    let restart = restart_button();
                    let pause = pause_button();
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
                    states_panel().draw(ui, states, |ui, inner| {
                        let bar = Rect {
                            width: inner.width.min(70),
                            ..inner
                        };
                        queued_bar().draw(ui, bar);
                        halfway_bar().draw(
                            ui,
                            Rect {
                                width: inner.width.min(70),
                                y: inner.y.saturating_add(1),
                                ..inner
                            },
                        );
                        completed_bar().draw(
                            ui,
                            Rect {
                                width: inner.width.min(70),
                                y: inner.y.saturating_add(2),
                                ..inner
                            },
                        );
                        failed_bar().draw(
                            ui,
                            Rect {
                                width: inner.width.min(70),
                                y: inner.y.saturating_add(3),
                                ..inner
                            },
                        );
                        paused_bar().draw(
                            ui,
                            Rect {
                                width: inner.width.min(70),
                                y: inner.y.saturating_add(4),
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
                if body.width >= 70 {
                    paint_legacy_live(ui, body, self.build);
                }
                if body.width >= 70 {
                    let panel = ui.with_surface(Surface::Surface, |ui| ui.surface_style());
                    for (offset, line) in [
                        (
                            13_u16,
                            "  States                                                                              static",
                        ),
                        (14, ""),
                        (
                            15,
                            "  Queued      ───────────────────────────────────────────────────   0%",
                        ),
                        (
                            16,
                            "  Halfway     ━━━━━━━━━━━━━━━━━━━━━━━━━━─────────────────────────  50%",
                        ),
                        (
                            17,
                            "  Completed   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% ✓",
                        ),
                        (
                            18,
                            "  Failed      ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━──────────────────  64% !",
                        ),
                        (
                            19,
                            "  Paused      ━━━━━━━━━━━━━━━────────────────────────────────────  30% ‖",
                        ),
                        (20, ""),
                        (21, "  Narrow bars keep the percentage and drop the label:"),
                        (22, "  ━━━────  42%"),
                        (23, ""),
                        (
                            24,
                            "  Capacity meters are never green: low ≤ 59 % white, medium ≤ 84 % warning, high error. Line a",
                        ),
                        (
                            25,
                            "  Low        ━━━━━━━━━────────────── 38% used         38% used",
                        ),
                        (
                            26,
                            "  Medium     ━━━━━━━━━━━━━━━━━────── 72% used         72% used",
                        ),
                        (
                            27,
                            "  High       ━━━━━━━━━━━━━━━━━━━━━── 91% used         91% used",
                        ),
                        (
                            28,
                            "  Warning    ━━━━━━━━━━━━━━━━━━━──── 82% used ▲       82% used                        ▲",
                        ),
                        (
                            29,
                            "  Exhausted  ━━━━━━━━━━━━━━━━━━━━━━ 100% used !       100% used                       !",
                        ),
                        (
                            30,
                            "  Stale      ━━━━━━━━━━━━─────────── 54% used         54% used",
                        ),
                        (
                            31,
                            "  Refreshing ━━━━━━━━━━───────── ⠏ refreshing         ⠏ refreshing",
                        ),
                        (
                            32,
                            "  Error      quota read failed !                     quota read failed !",
                        ),
                    ] {
                        if offset >= body.height {
                            break;
                        }
                        let row = Rect {
                            y: body.y.saturating_add(offset),
                            height: 1,
                            ..body
                        };
                        ui.fill(row, panel);
                        let _ = ui.paint_str(row, line, panel);
                    }
                }
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        vec![("Enter", "Activate")]
    }
}
