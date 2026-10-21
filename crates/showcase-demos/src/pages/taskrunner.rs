use std::{cmp::Ordering, time::Duration};
use termrock::author::PaintStyle;

use termrock::{
    ActionKey, Button, Cx, Dialog, DialogAction, DialogState, FrameRead, Id, ItemKey, Panel,
    PanelKind, ProgressBar, Rect, Response, RowUi, StateFlags, StepState, Steps, StepsAction,
    StepsState, Ui, Variant, layout, width,
};

use super::{Page, PageUpdate, frame};

const RUN: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.run");
const CANCEL: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.cancel");
const STEPS: Id = termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.steps");
const CANCEL_DIALOG: Id =
    termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.cancel.dialog");
const TARGETS_PANEL: Id =
    termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.targets.panel");
const PIPELINE_PANEL: Id =
    termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.pipeline.panel");
const PIPELINE_PROGRESS: Id =
    termrock::Id::root("showcase_ui::pages::taskrunner::taskrunner.pipeline.progress");
pub const RUN_COMMAND: ActionKey = ActionKey::application("showcase.taskrunner.run");

#[derive(Clone, Debug)]
struct RunStep {
    id: u8,
    name: &'static str,
    state: StepState,
}

/// The targets the pipeline runs against: the label, its depth and whether
/// it holds other targets.
const TARGET_ROWS: &[(&str, u16, bool)] = &[
    ("payments-gateway", 0, true),
    ("build", 1, true),
    ("compile", 2, false),
    ("lint", 2, false),
    ("typecheck", 2, false),
    ("test", 1, true),
    ("unit", 2, false),
    ("integration", 2, false),
    ("e2e", 2, false),
    ("deploy", 1, true),
    ("staging", 2, false),
    ("production", 2, false),
    ("shared-libs", 0, true),
    ("compile", 1, false),
    ("publish", 1, false),
];

const NAMES: &[&str] = &["compile", "lint", "typecheck", "unit", "integration", "e2e"];

fn step_key(step: &RunStep) -> ItemKey {
    ItemKey::num(u64::from(step.id))
}
fn step_state(step: &RunStep) -> StepState {
    step.state
}
fn step_row(step: &RunStep, row: &mut RowUi<'_>) {
    row.label(step.name);
}
fn steps()
-> Steps<'static, RunStep, impl Fn(&RunStep) -> ItemKey, impl Fn(&RunStep, &mut RowUi<'_>)> {
    Steps::navigable(STEPS)
        .key(step_key)
        .step(&step_state)
        .row(step_row)
}

/// The targets the pipeline runs against. The tree is a drawing: the
/// pipeline owns the page's focus.
fn paint_targets(ui: &mut Ui<'_>, inner: Rect) {
    let muted = ink(0x8a_8a8a, 0x11_11_11);
    let text = ink(0xff_ff_ff, 0x11_11_11);
    for (row, (label, depth, parent)) in TARGET_ROWS.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else { break };
        if row >= inner.height {
            break;
        }
        let area = Rect {
            y: inner.y.saturating_add(row),
            height: 1,
            ..inner
        };
        let indent = depth.saturating_mul(2);
        if *parent {
            let _ = ui.paint_str(
                Rect {
                    x: area.x.saturating_add(indent),
                    width: 2,
                    ..area
                },
                "▾ ",
                muted,
            );
            let _ = ui.paint_str(
                Rect {
                    x: area.x.saturating_add(indent).saturating_add(2),
                    ..area
                },
                label,
                text,
            );
        } else {
            let _ = ui.paint_str(
                Rect {
                    x: area.x.saturating_add(indent).saturating_add(2),
                    ..area
                },
                label,
                text,
            );
        }
    }
}

fn run_button(running: bool) -> Button<'static> {
    Button::new(RUN, "Run pipeline")
        .variant(Variant::PRIMARY)
        .disabled(running)
}

fn cancel_button(running: bool) -> Button<'static> {
    Button::new(CANCEL, "Cancel pipeline")
        .variant(Variant::DANGER)
        .disabled(!running)
}

fn cancel_dialog() -> Dialog<'static> {
    Dialog::confirm(
        CANCEL_DIALOG,
        "Cancel pipeline?",
        "The running pipeline will be stopped safely.",
    )
}

#[derive(Debug)]
pub struct TaskRunnerPage {
    steps: Vec<RunStep>,
    state: StepsState,
    frame: usize,
    running: bool,
    motion_paused: bool,
    cancel_state: DialogState,
    message: &'static str,
}

impl TaskRunnerPage {
    pub fn new() -> Self {
        Self {
            steps: NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| RunStep {
                    id: u8::try_from(i.checked_add(1).unwrap_or(0)).unwrap_or(0),
                    name,
                    state: StepState::Queued,
                })
                .collect(),
            state: StepsState::new(),
            frame: 0,
            running: false,
            motion_paused: false,
            cancel_state: DialogState::default(),
            message: "pipeline idle",
        }
    }

    fn start(&mut self) {
        self.running = true;
        self.frame = 0;
        self.message = "compile started";
        for (i, step) in self.steps.iter_mut().enumerate() {
            step.state = if i == 0 {
                StepState::Running
            } else {
                StepState::Queued
            };
        }
    }

    fn advance(&mut self) {
        if !self.running {
            return;
        }
        self.frame = self.frame.saturating_add(1);
        let current = self.frame / 4;
        for (i, step) in self.steps.iter_mut().enumerate() {
            step.state = match i.cmp(&current) {
                Ordering::Less => StepState::Done,
                Ordering::Equal => StepState::Running,
                Ordering::Greater => StepState::Queued,
            };
        }
        if current >= self.steps.len() {
            self.running = false;
            self.message = "pipeline complete";
        }
    }
}

impl Default for TaskRunnerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TaskRunnerPage {
    fn seek_paused(&mut self, frame: usize) {
        if frame > 0 {
            self.start();
            for _ in 0..frame.min(self.steps.len().saturating_mul(4).saturating_add(1)) {
                self.advance();
            }
        }
        self.motion_paused = true;
    }

    fn title(&self) -> &'static str {
        "Task runner"
    }

    fn command(&mut self, _cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action == RUN_COMMAND && !self.running {
            self.start();
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut result = Response::ignored();
        let run = run_button(self.running).update(cx);
        if run.activated() {
            self.start();
        }
        result |= run.erase();
        let cancel = cancel_button(self.running).update(cx);
        if cancel.activated() && !cx.is_open(CANCEL_DIALOG) {
            cx.open_layer(CANCEL_DIALOG, cancel_dialog().layer(cx));
        }
        result |= cancel.erase();
        if self.running && !self.motion_paused {
            self.advance();
            cx.request_repaint_after(Duration::from_millis(120));
        }
        let rail = steps().update(cx, &mut self.state, &self.steps);
        if rail
            .action_ref()
            .is_some_and(|action| matches!(action, StepsAction::Activated(_)))
        {
            self.message = "step selected";
        }
        result |= rail.erase();
        if cx.is_open(CANCEL_DIALOG) {
            let dialog = cancel_dialog().update(cx, &mut self.cancel_state);
            if let Some(action) = dialog.action_ref() {
                match action {
                    DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                        self.running = false;
                        self.message = "pipeline cancelled";
                        for step in &mut self.steps {
                            if step.state == StepState::Running {
                                step.state = StepState::Skipped;
                            }
                        }
                    }
                    DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                        self.message = "cancel dismissed";
                    }
                }
                cx.close_layer(CANCEL_DIALOG, None);
            }
            result |= dialog.erase();
        }
        result.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Composed: tree, live progress, following log, busy states",
            |ui, body| {
                super::section_strip(ui, body);
                let content = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                // The targets tree and the pipeline share the page when it is
                // wide enough and stack when it is not.
                let wide = content.width >= 64;
                let tree_width = if wide { 30 } else { content.width };
                let tree_height = if wide {
                    content.height
                } else {
                    content.height / 2
                };
                let tree_area = Rect {
                    height: tree_height,
                    ..content
                };
                let pipeline_area = if wide {
                    Rect {
                        x: content.x.saturating_add(tree_width),
                        width: content.width.saturating_sub(tree_width),
                        ..content
                    }
                } else {
                    Rect {
                        y: content.y.saturating_add(tree_height),
                        height: content.height.saturating_sub(tree_height),
                        ..content
                    }
                };
                Panel::new(TARGETS_PANEL)
                    .kind(PanelKind::Card)
                    .title("Targets")
                    .draw(ui, tree_area, |ui, inner| {
                        paint_targets(ui, inner);
                    });
                let done = self
                    .steps
                    .iter()
                    .filter(|step| step.state == StepState::Done)
                    .count();
                let meta = format!("{done} of {} done", self.steps.len());
                let title = if self.running {
                    "Pipeline · running"
                } else {
                    "Pipeline"
                };
                Panel::new(PIPELINE_PANEL)
                    .kind(PanelKind::Card)
                    .title(title)
                    .meta(&meta)
                    .draw(ui, pipeline_area, |ui, inner| {
                        let steps_area = Rect {
                            height: inner.height.saturating_sub(13),
                            ..inner
                        };
                        steps().draw(ui, steps_area, &self.state, &self.steps);
                        let actions = Rect {
                            y: inner.y.saturating_add(7),
                            height: 1,
                            ..inner
                        };
                        let run = run_button(self.running);
                        let cancel = cancel_button(self.running);
                        let widths = [
                            run.measure(ui, termrock::Constraints::loose(inner.width, 1))
                                .preferred
                                .0,
                            cancel
                                .measure(ui, termrock::Constraints::loose(inner.width, 1))
                                .preferred
                                .0,
                        ];
                        let rects =
                            layout::action_row(actions, &widths, 2, termrock::RowAlign::Start);
                        if let Some(rect) = rects.first().copied() {
                            run.draw(ui, rect);
                        }
                        if let Some(rect) = rects.get(1).copied() {
                            cancel.draw(ui, rect);
                        }
                        if self.running {
                            let progress_row = Rect {
                                y: inner.y.saturating_add(6),
                                height: 1,
                                ..inner
                            };
                            let ratio = self.frame as f64 / (self.steps.len() as f64 * 4.0);
                            ProgressBar::new(PIPELINE_PROGRESS)
                                .label("Overall")
                                .ratio(ratio)
                                .draw(ui, progress_row);
                        }
                        let log_row = Rect {
                            y: inner.y.saturating_add(10),
                            height: 1,
                            ..inner
                        };
                        paint_log(ui, log_row, self.message, self.running);
                    });
            },
        );
        ui.layer(CANCEL_DIALOG, |ui, layer| {
            cancel_dialog().draw(ui, layer, &self.cancel_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc resumes", ui.surface_style());
            });
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(STEPS).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Move"), ("← →", "Fold")]
        } else {
            &[("r", "Run pipeline"), ("Enter", "Activate")]
        }
    }
}

/// The following log: its title, its rule and the latest line.
fn paint_log(ui: &mut Ui<'_>, row: Rect, message: &str, running: bool) {
    let muted = ink(0x8a_8a8a, 0x00_00_00);
    let faint = ink(0x73_73_73, 0x00_00_00);
    let text = if running || message != "pipeline idle" {
        message
    } else {
        "Ready. Press r or Run to start the pipeline."
    };
    let _ = ui.paint_str(Rect { height: 1, ..row }, "Log", muted);
    let meta = Rect {
        x: row.right().saturating_sub(width("· following")),
        height: 1,
        ..row
    };
    let _ = ui.paint_str(meta, "· following", muted);
    let rule = Rect {
        y: row.y.saturating_add(1),
        height: 1,
        ..row
    };
    let _ = ui.paint_str(rule, &"─".repeat(usize::from(row.width)), faint);
    let line = Rect {
        y: row.y.saturating_add(2),
        height: 1,
        ..row
    };
    let _ = ui.paint_str(line, text, faint);
}

fn rgb(hex: u32) -> termrock::Color {
    termrock::Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}
