use core::fmt;
use std::time::Duration;

use termrock::author::PaintStyle;

use termrock::{
    Button, Color, Cx, FrameRead, Id, Panel, PanelKind, Rect, Response, Spinner, StateFlags,
    Status, StepState, Steps, StepsState, TextArea, TextAreaState, Ui, Variant, layout,
};

use showcase_data::log_lines;

use super::{Page, PageUpdate, frame, lines};

const OUTPUT: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.output");
const OUTPUT_PANEL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.output.panel");
const RAIL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.rail");
const RAIL_PANEL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.rail.panel");
const STEP_SPINNER: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.step.spinner");
const RUN: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.run");
const FAIL: Id = termrock::Id::root("showcase_ui::pages::terminal::terminal.fail");

/// The ticks a launch stage takes.
const STAGE_TICKS: u32 = 6;

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

fn step_rail() -> Steps<'static, TerminalStep> {
    Steps::navigable(RAIL).step(&step_state)
}

fn step_spinner() -> Spinner<'static> {
    Spinner::new(STEP_SPINNER).frame(0)
}

fn output_panel() -> Panel<'static> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
}

fn rail_panel() -> Panel<'static> {
    Panel::new(RAIL_PANEL).title("Step rail")
}

/// The stage count rides on the rail panel's title row.
fn paint_rail_count(ui: &mut Ui<'_>, panel: Rect, settled: usize, running: bool) {
    let count = if running {
        format!("{settled} of 7 · running ")
    } else {
        format!("{settled} of 7 ")
    };
    let width = termrock::width(&count);
    let x = panel.right().saturating_sub(width).saturating_sub(1);
    let _ = ui.paint_str(
        Rect::new(x, panel.y, width, 1),
        &count,
        PaintStyle::new()
            .fg(Color::Rgb(115, 115, 115))
            .bg(Color::Rgb(17, 17, 17)),
    );
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

#[derive(Debug)]
pub struct TerminalPage {
    output: String,
    output_state: TextAreaState,
    steps: Vec<TerminalStep>,
    steps_state: StepsState,
    running: bool,
    tick: u32,
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
            tick: 0,
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
        self.tick = 0;
    }

    /// The launch advances one stage at a time. The page settles when the
    /// last stage is done.
    fn advance(&mut self) {
        if !self.running {
            return;
        }
        self.tick = self.tick.saturating_add(1);
        let current = (self.tick / STAGE_TICKS) as usize;
        for (index, step) in self.steps.iter_mut().enumerate() {
            step.state = match index.cmp(&current) {
                std::cmp::Ordering::Less => StepState::Done,
                std::cmp::Ordering::Equal => StepState::Running,
                std::cmp::Ordering::Greater => StepState::Queued,
            };
        }
        if current + 1 >= self.steps.len() {
            self.running = false;
            for step in &mut self.steps {
                step.state = StepState::Done;
            }
        }
    }

    /// The stages the launch finished.
    fn settled(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.state == StepState::Done)
            .count()
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

        let _ = output_panel();
        let _ = rail_panel();
        let _ = step_spinner();
        // The launch runs on its own clock: every tick moves one stage on and
        // the page settles when the last one is done.
        if self.running {
            self.advance();
            cx.request_repaint_after(Duration::from_millis(16));
            response = response.repaint();
        }
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

                rail_panel().draw(ui, right, |ui, inner| {
                    let rail_area = Rect {
                        height: inner.height.saturating_sub(3),
                        ..inner
                    };
                    step_rail().draw(ui, rail_area, &self.steps_state, &self.steps);

                    let _ = step_spinner().draw(
                        ui,
                        Rect {
                            x: inner.x.saturating_add(1),
                            y: inner.y,
                            width: 1,
                            height: 1,
                        },
                    );
                    let _ = ui.paint_str(
                        Rect {
                            x: inner.right().saturating_sub(5),
                            y: inner.y,
                            width: 5,
                            height: 1,
                        },
                        "0.7 s",
                        ui.surface_style(),
                    );

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
                paint_rail_count(ui, right, self.settled(), self.running);

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

// Page rect inside the 160×50 shell: sidebar span 24, inspector closed.

// Finished 120×40 playground, column-relative to the page rect. Captured from
// the truecolor oracle once the launch reaches "✓ Ready".

// Finished 80×24 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set.

// Finished 72×20 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set.

// Finished 160×50 page, column-relative to the page rect. Truecolor cells from
// the oracle once the launch reaches "✓ Ready". paint_str merges modifiers, so
// every run clears the modifiers it does not set. Bits: 1 bold, 2 italic, 4 underline.
