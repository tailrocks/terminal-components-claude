use crate::domain::plan::StepState;
use crate::sim::{
    plans::{ReviewBinding, ReviewedPlan},
    world::World,
};
use termrock::{
    Action, ActionKey, Cx, Dialog, DialogAction, DialogState, Id, Props, Rect, Response,
    TextViewport, Ui, ViewportLine, ViewportState,
};

pub(crate) const GATE: Id = Id::root("plan.gate");
const COMMANDS: Id = Id::root("plan.gate.commands");
const ACTIONS: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Run plan (simulated)"),
];

pub(crate) struct PlanGate {
    title: String,
    binding: ReviewBinding,
    phrase: String,
    input_label: String,
    facts: Vec<(String, String)>,
    commands: Vec<String>,
    dialog: DialogState,
    output: ViewportState,
}
impl PlanGate {
    pub(crate) fn new(review: &ReviewedPlan) -> Self {
        let plan = review.plan();
        Self {
            title: format!("Confirm: {}", plan.title()),
            binding: review.binding(),
            phrase: plan.phrase().into(),
            input_label: format!("Type {} to confirm", plan.phrase()),
            facts: vec![
                (
                    "Will happen".into(),
                    format!(
                        "{} of {} steps run on {}",
                        plan.included_count(),
                        plan.steps().len(),
                        plan.host()
                    ),
                ),
                ("Will change".into(), plan.will_change().into()),
                (
                    "Bound to".into(),
                    format!("host {} · phrase names the target", plan.host()),
                ),
                (
                    "Safety".into(),
                    "simulated · nothing on any real host executes".into(),
                ),
            ],
            commands: plan
                .steps()
                .iter()
                .filter(|step| {
                    !matches!(
                        step.state,
                        StepState::Excluded | StepState::PolicySkipped(_)
                    )
                })
                .map(|step| step.command.clone())
                .collect(),
            dialog: DialogState::default(),
            output: ViewportState::default(),
        }
    }
    fn props(&self) -> Dialog<'_> {
        Dialog::acknowledge(GATE, &self.title, &self.phrase)
            .input_label(&self.input_label)
            .actions(ACTIONS)
            .body_rows(
                self.facts
                    .len()
                    .saturating_add(1)
                    .saturating_add(self.commands.len().min(8))
                    .min(usize::from(u16::MAX)) as u16,
            )
            .width(66)
    }
    pub(crate) fn is_editing(&self) -> bool {
        self.dialog.is_editing()
    }

    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        let dialog = self.props();
        cx.open_layer(GATE, dialog.layer(cx));
        cx.focus(dialog.input_id());
    }
    pub(crate) fn poll(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<DialogAction>) {
        let title = &self.title;
        let phrase = &self.phrase;
        let dialog = Dialog::acknowledge(GATE, title, phrase)
            .input_label(&self.input_label)
            .actions(ACTIONS)
            .body_rows(
                self.facts
                    .len()
                    .saturating_add(1)
                    .saturating_add(self.commands.len().min(8))
                    .min(usize::from(u16::MAX)) as u16,
            )
            .width(66);
        let mut response = dialog.update(cx, &mut self.dialog);
        let lines: Vec<_> = self
            .commands
            .iter()
            .map(|command| ViewportLine::Plain(command))
            .collect();
        let output = commands_viewport().update(cx, &mut self.output, &lines);
        let action = response.take_action();
        (response.erase() | output.erase(), action)
    }
    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        review: &mut ReviewedPlan,
        world: &mut World,
    ) -> (Response<()>, Option<String>) {
        let (response, action) = self.poll(cx);
        let result = match action {
            Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                let result = if review.matches_binding(&self.binding) {
                    match review
                        .approve(&self.phrase, world)
                        .and_then(|approval| approval.execute(world))
                    {
                        Ok(_) => format!("Finished: {} · simulated", review.plan().summary()),
                        Err(error) => error.to_string(),
                    }
                } else {
                    "Plan selection changed · review it again".into()
                };
                self.dialog.zeroize();
                cx.close_layer(GATE, Some(ActionKey::CONFIRM));
                Some(result)
            }
            Some(DialogAction::Action(_) | DialogAction::Dismissed(_)) => {
                self.dialog.zeroize();
                cx.close_layer(GATE, Some(ActionKey::CANCEL));
                Some("Plan cancelled · nothing ran".into())
            }
            None => None,
        };
        (response, result)
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(GATE, |ui, area| {
            self.props().draw(ui, area, &self.dialog, |ui, body| {
                let facts: Vec<_> = self
                    .facts
                    .iter()
                    .map(|(label, value)| (label.as_str(), value.as_str()))
                    .collect();
                let height = self.facts.len().min(usize::from(u16::MAX)) as u16;
                Props::new(&facts).draw(
                    ui,
                    Rect::new(body.x, body.y, body.width, body.height.min(height)),
                );
                let lines: Vec<_> = self
                    .commands
                    .iter()
                    .map(|command| ViewportLine::Plain(command))
                    .collect();
                commands_viewport().draw(
                    ui,
                    Rect::new(
                        body.x,
                        body.y.saturating_add(height.saturating_add(1)),
                        body.width,
                        body.height.saturating_sub(height.saturating_add(1)),
                    ),
                    &self.output,
                    &lines,
                );
            });
        });
    }
}
fn commands_viewport() -> TextViewport<'static> {
    TextViewport::new(COMMANDS).wrap(false)
}
