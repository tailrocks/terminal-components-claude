use core::fmt;
use std::time::Duration;

use termrock::{
    Button, Cx, FrameRead, Id, Panel, PanelKind, Rect, Response, Spinner, StateFlags, Status,
    StepState, Steps, StepsState, TextArea, TextAreaState, Ui, Variant, layout,
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

/// The rows the page's footer keeps below the panes.
const FOOTER_ROWS: u16 = 2;

const STAGES: &[(&str, StepState, &str)] = &[
    ("01 Resolve workspace", StepState::Running, "0.8 s"),
    ("02 Pull base image", StepState::Queued, "2.1 s"),
    ("03 Build container", StepState::Queued, "3.2 s"),
    ("04 Mount sources", StepState::Queued, "cached"),
    ("05 Resolve credentials", StepState::Queued, "1.4 s"),
    ("06 Start agent", StepState::Queued, "1.1 s"),
    ("07 Ready", StepState::Queued, "0.1 s"),
];

#[derive(Clone, Copy, Debug)]
struct TerminalStep {
    label: &'static str,
    state: StepState,
    duration: &'static str,
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
    Steps::navigable(RAIL)
        .step(&step_state)
        .metadata(&|step: &TerminalStep| Some(step.duration))
}

fn step_spinner() -> Spinner<'static> {
    Spinner::new(STEP_SPINNER).frame(0)
}

fn output_panel() -> Panel<'static> {
    Panel::new(OUTPUT_PANEL)
        .kind(PanelKind::Framed)
        .title("Viewport")
}

fn rail_panel(meta: &str) -> Panel<'_> {
    Panel::new(RAIL_PANEL).title("Step rail").meta(meta)
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
                .map(|(label, state, duration)| TerminalStep {
                    label,
                    state: *state,
                    duration,
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
            .map(|(index, (label, _, duration))| TerminalStep {
                label,
                duration,
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
                super::section_strip(ui, body);
                let content = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                // The footer keeps its own rows below the panes, so it never
                // lands on a panel border.
                let panes_area = Rect {
                    height: content.height.saturating_sub(FOOTER_ROWS),
                    ..content
                };
                let (left, right) = panes(panes_area);
                let wide = left.is_some();
                if let Some(left) = left {
                    output_panel().draw(ui, left, |ui, inner| {
                        output()
                            .value(&self.output)
                            .draw(ui, inner, &self.output_state);
                    });
                }

                let count = if self.running {
                    format!("{} of 7 · running", self.settled())
                } else {
                    format!("{} of 7", self.settled())
                };
                rail_panel(&count).draw(ui, right, |ui, inner| {
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
                            y: panes_area.bottom(),
                            height: FOOTER_ROWS,
                            ..content
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
            // The viewport is a read-only control: it scrolls and its caret
            // walks the log. It has no follow, copy or clear command.
            &[("↑ ↓", "Scroll"), ("Home End", "Oldest / live")]
        } else if ui.state(RAIL).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move")]
        } else {
            &[("Enter", "Activate")]
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
