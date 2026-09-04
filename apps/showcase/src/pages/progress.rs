//! Determinate and animated progress indicators.

use std::time::Duration;

use tui_next::{Cx, Id, Panel, PanelKind, ProgressBar, Rect, Response, Spinner, Status, Ui, id, layout};

use super::{Page, frame, lines};

const BUILD: Id = id!("progress.build");
const PIPELINE: Id = id!("progress.pipeline");
const CHECKS: Id = id!("progress.checks");
const SPINNER: Id = id!("progress.spinner");

/// Progress has no input controls, but owns the animation frame and work
/// completion values used by every theme and colour level.
#[derive(Debug, Default)]
pub(crate) struct ProgressPage {
    frame: usize,
    checks: u16,
}

impl ProgressPage {
    pub(crate) fn new() -> Self { Self { frame: 0, checks: 72 } }
}

impl Page for ProgressPage {
    fn title(&self) -> &'static str { "Progress" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.frame = self.frame.wrapping_add(1);
        self.checks = 60 + u16::try_from(self.frame % 41).unwrap_or(0);
        cx.request_repaint_after(Duration::from_millis(120));
        Response::changed()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "determinate · spinner · colour downgrade", |ui, body| {
            let (meter, rest) = layout::split_v(body, 4);
            Panel::new(PIPELINE).kind(PanelKind::Card).title("Build pipeline").draw(ui, meter, |ui, inner| {
                ProgressBar::new(BUILD)
                    .label("Build")
                    .ratio(0.72)
                    .status(Status::Ready)
                    .frame(self.frame)
                    .draw(ui, inner);
            });
            let (check_area, spin_area) = layout::split_v(rest, 1);
            ProgressBar::new(CHECKS)
                .label("Checks")
                .ratio(f64::from(self.checks) / 100.0)
                .frame(self.frame)
                .draw(ui, check_area);
            Spinner::new(SPINNER).label("running tests").frame(self.frame).draw(ui, spin_area);
            lines(ui, Rect { y: spin_area.bottom().saturating_add(1), height: body.bottom().saturating_sub(spin_area.bottom().saturating_add(1)), ..body }, &["Every tick changes only durable progress state; drawing remains pure."]);
        });
    }
}
