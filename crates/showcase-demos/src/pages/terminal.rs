//! Terminal-style output with a scrollable viewport and a seven-step rail.

use core::fmt;

use termrock::{
    Button, ByIndex, Cx, FrameRead, Id, Panel, PanelKind, Rect, Response, RowUi, Spinner,
    StateFlags, Status, StepState, Steps, StepsState, TextArea, TextAreaState, Ui, Variant, id,
    layout,
};

use showcase_data::log_lines;

use super::{Page, PageUpdate, frame, lines};

const OUTPUT: Id = id!("terminal.output");
const OUTPUT_PANEL: Id = id!("terminal.output.panel");
const RAIL: Id = id!("terminal.rail");
const RAIL_PANEL: Id = id!("terminal.rail.panel");
const STEP_SPINNER: Id = id!("terminal.step.spinner");
const RUN: Id = id!("terminal.run");
const FAIL: Id = id!("terminal.fail");

const STAGES: &[(&str, StepState)] = &[
    ("01 Resolve workspace", StepState::Running),
    ("02 Pull base image", StepState::Queued),
    ("03 Build container", StepState::Queued),
    ("04 Mount sources", StepState::Queued),
    ("05 Resolve credentials", StepState::Queued),
    ("06 Start agent", StepState::Queued),
    ("07 Ready", StepState::Queued),
];

#[derive(Clone, Copy, Debug)]
struct TerminalStep {
    label: &'static str,
    state: StepState,
}

impl fmt::Display for TerminalStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn step_state(step: &TerminalStep) -> StepState {
    step.state
}

fn step_row(step: &TerminalStep, row: &mut RowUi<'_>) {
    if step.state == StepState::Running {
        row.meta("0.7 s");
    }
    row.label(step.label);
}

fn step_rail() -> Steps<'static, TerminalStep, ByIndex, impl Fn(&TerminalStep, &mut RowUi<'_>)> {
    Steps::navigable(RAIL).step(&step_state).row(step_row)
}

fn step_spinner() -> Spinner<'static> {
    Spinner::new(STEP_SPINNER).frame(0)
}

fn output_panel() -> Panel<'static> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
}

fn rail_meta(running: bool) -> &'static str {
    if running {
        "0 of 7 · running "
    } else {
        "0 of 7 "
    }
}

fn rail_panel(running: bool) -> Panel<'static> {
    Panel::new(RAIL_PANEL)
        .title("Step rail")
        .meta(rail_meta(running))
}

fn output() -> TextArea<'static> {
    TextArea::new(OUTPUT, 12)
        .read_only(true)
        .status(Status::Ready)
}

fn run_button() -> Button<'static> {
    Button::new(RUN, "Run").variant(Variant::SECONDARY)
}

fn failure_button() -> Button<'static> {
    Button::new(FAIL, "Run with a failure").variant(Variant::SECONDARY)
}

fn panes(area: Rect) -> (Option<Rect>, Rect) {
    let usable = area.width.saturating_sub(2);
    if usable < 58 {
        return (None, area);
    }
    let left_width = area.width.saturating_mul(62).checked_div(100).unwrap_or(0);
    let right_x = area.x.saturating_add(left_width).saturating_add(2);
    (
        Some(Rect {
            width: left_width,
            ..area
        }),
        Rect {
            x: right_x,
            width: area.right().saturating_sub(right_x),
            ..area
        },
    )
}

/// The page keeps terminal text and lifecycle data in app state; public
/// controls retain ownership of scrolling, focus, and activation semantics.
#[derive(Debug)]
pub struct TerminalPage {
    output: String,
    output_state: TextAreaState,
    steps: Vec<TerminalStep>,
    steps_state: StepsState,
    running: bool,
}

impl TerminalPage {
    pub fn new() -> Self {
        Self {
            output: log_lines(120).join("\n"),
            output_state: TextAreaState::default(),
            steps: STAGES
                .iter()
                .map(|(label, state)| TerminalStep {
                    label,
                    state: *state,
                })
                .collect(),
            steps_state: StepsState::default(),
            running: true,
        }
    }

    fn reset(&mut self, failed: bool) {
        self.steps = STAGES
            .iter()
            .enumerate()
            .map(|(index, (label, _))| TerminalStep {
                label,
                state: if failed && index > 0 {
                    StepState::Blocked
                } else if failed && index == 0 {
                    StepState::Failed
                } else if index == 0 {
                    StepState::Running
                } else {
                    StepState::Queued
                },
            })
            .collect();
        self.steps_state = StepsState::default();
        self.running = !failed;
    }
}

impl Default for TerminalPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TerminalPage {
    fn title(&self) -> &'static str {
        "Terminal"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let output = output().update(cx, &mut self.output_state, &mut self.output);
        response |= output.erase();
        response |= step_rail()
            .update(cx, &mut self.steps_state, &self.steps)
            .erase();

        let run = run_button().update(cx);
        if run.activated() {
            self.reset(false);
        }
        response |= run.erase();
        let fail = failure_button().update(cx);
        if fail.activated() {
            self.reset(true);
        }
        response |= fail.erase();
        // Both phases build the same panels and spinner (§13); the update pass
        // only needs the construction to stay the single source of the props.
        let _ = output_panel();
        let _ = rail_panel(self.running);
        let _ = step_spinner();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Viewport with scrollback, selection and copy · drag the seam · step rail reports a job",
            |ui, body| {
                let (left, right) = panes(body);
                let wide = left.is_some();
                if let Some(left) = left {
                    output_panel().draw(ui, left, |ui, inner| {
                        output()
                            .value(&self.output)
                            .draw(ui, inner, &self.output_state);
                    });
                }

                rail_panel(self.running).draw(ui, right, |ui, inner| {
                    let rail_area = Rect {
                        height: inner.height.saturating_sub(3),
                        ..inner
                    };
                    step_rail().draw(ui, rail_area, &self.steps_state, &self.steps);
                    // The running row's icon cell carries the owned Spinner so
                    // the rail animates; elapsed time rides the row META
                    // channel (see step_row), not a preview overlay.
                    if self.running {
                        let _ = step_spinner().draw(
                            ui,
                            Rect {
                                x: inner.x.saturating_add(1),
                                y: inner.y,
                                width: 1,
                                height: 1,
                            },
                        );
                    }

                    let run = run_button();
                    let fail = failure_button();
                    let widths = [
                        run.measure(ui, termrock::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                        fail.measure(ui, termrock::Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                    ];
                    let row = Rect {
                        y: inner.bottom().saturating_sub(1),
                        height: 1,
                        ..inner
                    };
                    let rects = layout::action_row(row, &widths, 2, termrock::RowAlign::Start);
                    if let Some(rect) = rects.first().copied() {
                        run.draw(ui, rect);
                    }
                    if let Some(rect) = rects.get(1).copied() {
                        fail.draw(ui, rect);
                    }
                });

                if wide {
                    lines(
                        ui,
                        Rect {
                            y: body.bottom().saturating_sub(2),
                            height: 2,
                            ..body
                        },
                        &[
                            "Output remains borrowed by the public control.",
                            "status: ready",
                        ],
                    );
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(OUTPUT).contains(StateFlags::FOCUSED) {
            &[
                ("↑ ↓", "Scroll"),
                ("Home End", "Oldest / live"),
                ("f", "Follow"),
                ("drag", "Select"),
                ("y", "Copy"),
                ("Esc", "Clear"),
            ]
        } else if ui.state(RAIL).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("wheel", "Scroll")]
        } else {
            &[("Enter", "Activate"), ("drag ┃", "Resize")]
        }
    }
}

#[cfg(test)]
mod narrow_rail_tests {
    use super::*;
    use termrock::{App, KeyCode, Theme};
    use termrock_test_support::Harness;

    struct PageApp(TerminalPage);

    impl App for PageApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            self.0.draw(ui, full);
        }
    }

    /// At every standard size the rail inner is narrower than 60 cells, where
    /// the retired historical painter overwrote the rail with clipped static
    /// strings and a duplicate button row. The composed rail must come from
    /// Steps/Spinner/Button state instead: lifecycle words visible where they
    /// fit, one button row, and the failure/run actions repainting the rail
    /// through the same components.
    #[test]
    fn narrow_rail_composes_owned_steps_spinner_and_buttons() {
        let mut h = Harness::new(PageApp(TerminalPage::new()), Theme::junie(), 80, 24);
        h.draw();
        let boot = h.text();
        assert!(
            boot.contains("01 Reso"),
            "running step label renders (ellipsis-truncated by the rail)"
        );
        assert!(
            boot.contains("0.7 s"),
            "running step shows elapsed row meta"
        );
        assert!(
            boot.contains("queued"),
            "queued lifecycle word fits the narrow rail"
        );
        assert!(boot.contains("⠋"), "owned spinner animates the running row");
        assert_eq!(
            boot.matches("Run with a fail").count(),
            1,
            "exactly one failure button row (no historical duplicate)"
        );

        // The page-chrome annotation registers a decor region over the button
        // row, so pointer hits never reach these buttons (pre-existing page
        // issue, out of scope here); drive them through the real keyboard
        // path instead. Enter is harmless on the other stops: the output is
        // read-only, the rail action is discarded, and Run resets boot state.
        for _ in 0..8 {
            if h.text().contains("failed") {
                break;
            }
            let _ = h.key(KeyCode::Tab);
            let _ = h.key(KeyCode::Enter);
        }
        let failed = h.text();
        assert!(
            failed.contains("failed"),
            "failed lifecycle word after failure action"
        );
        assert!(
            failed.contains("blocked"),
            "later steps blocked after failure action"
        );
        assert!(
            !failed.contains("0.7 s"),
            "no running step, no elapsed meta"
        );
        assert!(
            !failed.contains("⠋") && !failed.contains("⠏"),
            "spinner rests when nothing runs"
        );

        for _ in 0..8 {
            let current = h.text();
            if current.contains("0.7 s") && !current.contains("failed") {
                break;
            }
            let _ = h.key(KeyCode::Tab);
            let _ = h.key(KeyCode::Enter);
        }
        let rerun = h.text();
        assert!(rerun.contains("0.7 s"), "run restores the running step");
        assert!(rerun.contains("queued"), "run re-queues later steps");
        assert!(!rerun.contains("failed"), "run clears the failure");
    }
}
