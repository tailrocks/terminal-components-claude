//! Determinate and animated progress indicators.

use std::time::Duration;

use junie_tui::{
    Cx, Id, Meter, Panel, PanelKind, ProgressBar, Rect, Response, Spinner, Status, Ui, id, layout,
};

use super::{Page, frame, lines};

const BUILD: Id = id!("progress.build");
const PIPELINE: Id = id!("progress.pipeline");
const CHECKS: Id = id!("progress.checks");
const SPINNER: Id = id!("progress.spinner");
const METER: Id = id!("progress.meter");

fn pipeline_panel() -> Panel<'static> {
    Panel::new(PIPELINE)
        .kind(PanelKind::Card)
        .title("Build pipeline")
}

fn build_bar(frame: usize) -> ProgressBar<'static> {
    ProgressBar::new(BUILD)
        .label("Build")
        .ratio(0.72)
        .status(Status::Ready)
        .frame(frame)
}

fn checks_bar(checks: u16, frame: usize) -> ProgressBar<'static> {
    ProgressBar::new(CHECKS)
        .label("Checks")
        .ratio(f64::from(checks) / 100.0)
        .frame(frame)
}

fn test_spinner(frame: usize) -> Spinner<'static> {
    Spinner::new(SPINNER).label("running tests").frame(frame)
}

fn meter() -> Meter<'static> {
    Meter::new(METER).ratio(0.72).value("72%")
}

/// Progress has no input controls, but owns the animation frame and work
/// completion values used by every theme and colour level.
#[derive(Debug, Default)]
pub(crate) struct ProgressPage {
    frame: usize,
    checks: u16,
}

impl ProgressPage {
    pub(crate) fn new() -> Self {
        Self {
            frame: 0,
            checks: 72,
        }
    }
}

impl Page for ProgressPage {
    fn title(&self) -> &'static str {
        "Progress"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = pipeline_panel();
        let _ = build_bar(self.frame);
        let _ = checks_bar(self.checks, self.frame);
        let _ = test_spinner(self.frame);
        let _ = meter();
        self.frame = self.frame.wrapping_add(1);
        self.checks = 60_u16.saturating_add(u16::try_from(self.frame.rem_euclid(41)).unwrap_or(0));
        cx.request_repaint_after(Duration::from_millis(120));
        // Animation invalidation must not consume unrelated application keys.
        // `]`/`[` bubble to the shell navigation while this page is active.
        Response::ignored().repaint()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "determinate · spinner · colour downgrade",
            |ui, body| {
                let (meter_area, rest) = layout::split_v(body, 4);
                pipeline_panel().draw(ui, meter_area, |ui, inner| {
                    build_bar(self.frame).draw(ui, inner);
                });
                let (check_area, rest) = layout::split_v(rest, 1);
                let (meter_area, spin_area) = layout::split_v(rest, 1);
                checks_bar(self.checks, self.frame).draw(ui, check_area);
                meter().draw(ui, meter_area);
                test_spinner(self.frame).draw(ui, spin_area);
                lines(
                    ui,
                    Rect {
                        y: spin_area.bottom().saturating_add(1),
                        height: body
                            .bottom()
                            .saturating_sub(spin_area.bottom().saturating_add(1)),
                        ..body
                    },
                    &["Every tick changes only durable progress state; drawing remains pure."],
                );
            },
        );
    }
}
