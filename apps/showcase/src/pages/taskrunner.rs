//! A cancellable task runner using the public lifecycle rail.

use std::time::Duration;

use tui_next::{Button, Cx, Dialog, DialogAction, DialogState, Id, ItemKey, Rect, Response, RowUi, StepState, Steps, StepsAction, StepsState, Ui, Variant, id, layout};

use super::{Page, frame, lines};

const RUN: Id = id!("taskrunner.run");
const CANCEL: Id = id!("taskrunner.cancel");
const STEPS: Id = id!("taskrunner.steps");
const CANCEL_DIALOG: Id = id!("taskrunner.cancel.dialog");

#[derive(Clone, Debug)]
struct RunStep { id: u8, name: &'static str, state: StepState }

const NAMES: &[&str] = &["compile started", "unit tests", "integration tests", "package artifact", "publish report"];

fn step_key(step: &RunStep) -> ItemKey { ItemKey::num(u64::from(step.id)) }
fn step_state(step: &RunStep) -> StepState { step.state }
fn step_row(step: &RunStep, row: &mut RowUi<'_>) { row.label(step.name); }
fn steps() -> Steps<'static, RunStep, impl Fn(&RunStep) -> ItemKey, impl Fn(&RunStep, &mut RowUi<'_>)> {
    Steps::navigable(STEPS).key(step_key).step(&step_state).row(step_row)
}
fn cancel_dialog() -> Dialog<'static> { Dialog::confirm(CANCEL_DIALOG, "Cancel pipeline?", "The running pipeline will be stopped safely.") }

/// The runner advances one lifecycle step per virtual tick and confirms
/// cancellation through a modal layer.
#[derive(Debug)]
pub(crate) struct TaskRunnerPage {
    steps: Vec<RunStep>,
    state: StepsState,
    frame: usize,
    running: bool,
    cancel_state: DialogState,
    message: &'static str,
}

impl TaskRunnerPage {
    pub(crate) fn new() -> Self {
        Self {
            steps: NAMES.iter().enumerate().map(|(i, name)| RunStep { id: u8::try_from(i + 1).unwrap_or(0), name, state: StepState::Queued }).collect(),
            state: StepsState::new(),
            frame: 0,
            running: false,
            cancel_state: DialogState::default(),
            message: "pipeline idle",
        }
    }

    fn start(&mut self) {
        self.running = true;
        self.frame = 0;
        self.message = "compile started";
        for (i, step) in self.steps.iter_mut().enumerate() { step.state = if i == 0 { StepState::Running } else { StepState::Queued }; }
    }

    fn advance(&mut self) {
        if !self.running { return; }
        self.frame = self.frame.saturating_add(1);
        let current = self.frame / 4;
        for (i, step) in self.steps.iter_mut().enumerate() {
            step.state = if i < current { StepState::Done } else if i == current { StepState::Running } else { StepState::Queued };
        }
        if current >= self.steps.len() { self.running = false; self.message = "pipeline complete"; }
    }
}

impl Default for TaskRunnerPage { fn default() -> Self { Self::new() } }

impl Page for TaskRunnerPage {
    fn title(&self) -> &'static str { "Task runner" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let run = Button::new(RUN, "Run pipeline").variant(Variant::PRIMARY).disabled(self.running).update(cx);
        if run.activated() { self.start(); }
        result |= run.erase();
        let cancel = Button::new(CANCEL, "Cancel pipeline").variant(Variant::DANGER).disabled(!self.running).update(cx);
        if cancel.activated() && !cx.is_open(CANCEL_DIALOG) { cx.open_layer(CANCEL_DIALOG, cancel_dialog().layer(cx)); }
        result |= cancel.erase();
        if self.running { self.advance(); cx.request_repaint_after(Duration::from_millis(120)); }
        let rail = steps().update(cx, &mut self.state, &self.steps);
        if rail.action_ref().is_some_and(|action| matches!(action, StepsAction::Activated(_))) { self.message = "step selected"; }
        result |= rail.erase();
        if cx.is_open(CANCEL_DIALOG) {
            let dialog = cancel_dialog().update(cx, &mut self.cancel_state);
            if let Some(action) = dialog.action_ref() {
                match action {
                    DialogAction::Action(key) if *key == tui_next::ActionKey::CONFIRM => {
                        self.running = false;
                        self.message = "pipeline cancelled";
                        for step in &mut self.steps { if step.state == StepState::Running { step.state = StepState::Skipped; } }
                    }
                    DialogAction::Action(_) | DialogAction::Dismissed(_) => { self.message = "cancel dismissed"; }
                }
                cx.close_layer(CANCEL_DIALOG, None);
            }
            result |= dialog.erase();
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "lifecycle steps · virtual ticks · cancellation", |ui, body| {
            let (rail_area, actions) = layout::split_v(body, body.height.saturating_sub(6));
            steps().draw(ui, rail_area, &self.state, &self.steps);
            let action_rows = super::rows(actions, 3);
            Button::new(RUN, "Run pipeline").variant(Variant::PRIMARY).disabled(self.running).draw(ui, action_rows.first().copied().unwrap_or(actions));
            Button::new(CANCEL, "Cancel pipeline").variant(Variant::DANGER).disabled(!self.running).draw(ui, action_rows.get(1).copied().unwrap_or(actions));
            let progress = format!("Pipeline · {} · frame={} · {}", if self.running { "running" } else { "idle" }, self.frame, self.message);
            let _ = ui.paint_str(action_rows.get(2).copied().unwrap_or(actions), &progress, ui.surface_style());
            lines(ui, Rect { y: actions.bottom().saturating_add(1), height: 1, ..body }, &["The rail derives BUSY/CHECKED/ERROR from each app-owned lifecycle state."]);
        });
        ui.layer(CANCEL_DIALOG, |ui, layer| { cancel_dialog().draw(ui, layer, &self.cancel_state, |ui, body| { let _ = ui.paint_str(body, "Enter confirms · Esc resumes", ui.surface_style()); }); });
    }
}
