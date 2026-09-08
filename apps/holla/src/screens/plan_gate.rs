//! Gate two binds a shared exact-match acknowledgement to one reviewed plan.
use crate::domain::plan::StepState;
use crate::sim::{
    plans::{ReviewBinding, ReviewedPlan},
    world::World,
};
use junie_tui::{
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
                // Dialog's exact-match action is the acknowledgement proof. Its
                // secret draft remains private; domain validation receives the
                // immutable phrase whose matching armed this specific action.
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "Named deterministic reviewed plans must exist"
)]
mod tests {
    use super::*;
    use crate::{domain::fixtures, scenario::Scenario, sim::plans};
    use junie_tui::{App, KeyCode, Theme};
    use junie_tui_testing::Harness;
    struct Fixture {
        gate: PlanGate,
        review: ReviewedPlan,
        world: World,
        opened: bool,
        result: Option<String>,
    }
    impl App for Fixture {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if !self.opened {
                self.gate.open(cx);
                self.opened = true;
            }
            let (response, result) = self.gate.update(cx, &mut self.review, &mut self.world);
            if result.is_some() {
                self.result = result;
            }
            response
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            self.gate.draw(ui);
        }
    }
    fn fixture() -> Fixture {
        let mut world = fixtures::world_for(Scenario::DockerCleanup);
        world.seek(4_000);
        let review = plans::plan_for(&world, "docker.cleanup").unwrap();
        let gate = PlanGate::new(&review);
        Fixture {
            gate,
            review,
            world,
            opened: false,
            result: None,
        }
    }
    #[test]
    fn wrong_acknowledgement_cannot_approve_by_keyboard_or_pointer() {
        let mut harness = Harness::new(fixture(), Theme::junie(), 120, 40);
        let _ = harness.type_str("incomplete");
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.click_id(Dialog::new(GATE).action_id(1));
        assert!(!harness.app().review.plan().ran());
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(harness.app().result.is_none());
    }
    #[test]
    fn exact_acknowledgement_applies_once_and_stale_host_still_refuses() {
        for stale in [false, true] {
            let mut harness = Harness::new(fixture(), Theme::junie(), 120, 40);
            assert_eq!(harness.focus(), Some(Dialog::new(GATE).input_id()));
            let phrase = harness.app().review.plan().phrase().to_owned();
            let _ = harness.type_str(&phrase);
            if stale {
                harness.app_mut().world.host.name = "different-host".into();
            }
            let _ = harness.click_id(Dialog::new(GATE).action_id(1));
            assert_eq!(harness.app().review.plan().ran(), !stale);
            assert_eq!(harness.app().world.effect_revision, u64::from(!stale));
            assert!(harness.app().result.is_some());
        }
    }
    #[test]
    fn surrounding_whitespace_is_not_exact_acknowledgement() {
        let mut harness = Harness::new(fixture(), Theme::junie(), 120, 40);
        let phrase = format!(" {} ", harness.app().review.plan().phrase());
        let _ = harness.type_str(&phrase);
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.click_id(Dialog::new(GATE).action_id(1));
        assert_eq!(harness.app().world.effect_revision, 0);
    }
    #[test]
    fn changing_committed_acknowledgement_disarms_confirmation() {
        let mut harness = Harness::new(fixture(), Theme::junie(), 120, 40);
        let phrase = harness.app().review.plan().phrase().to_owned();
        let _ = harness.type_str(&phrase);
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.click_id(Dialog::new(GATE).input_id());
        assert!(harness.app().gate.is_editing());
        let _ = harness.type_str("x");
        let _ = harness.click_id(Dialog::new(GATE).action_id(1));
        assert_eq!(harness.app().world.effect_revision, 0);
    }
    #[test]
    fn changed_inclusion_after_gate_open_refuses_exact_acknowledgement() {
        let mut harness = Harness::new(fixture(), Theme::junie(), 120, 40);
        assert_eq!(harness.focus(), Some(Dialog::new(GATE).input_id()));
        let phrase = harness.app().review.plan().phrase().to_owned();
        let index = harness
            .app()
            .review
            .plan()
            .steps()
            .iter()
            .position(|step| step.optional)
            .unwrap();
        harness.app_mut().review.toggle(index).unwrap();
        let _ = harness.type_str(&phrase);
        let _ = harness.click_id(Dialog::new(GATE).action_id(1));
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(!harness.app().review.plan().ran());
        assert_eq!(
            harness.app().result.as_deref(),
            Some("Plan selection changed · review it again")
        );
    }
    #[test]
    fn independent_replaced_review_cannot_reuse_old_gate_acknowledgement() {
        let mut h = Harness::new(fixture(), Theme::junie(), 120, 40);
        let phrase = h.app().review.plan().phrase().to_owned();
        let _ = h.type_str(&phrase);
        let _ = h.key(KeyCode::Enter);
        h.app_mut().world.docker.as_mut().unwrap().cache_bytes += 12345;
        let replacement = plans::plan_for(&h.app().world, "docker.cleanup").unwrap();
        h.app_mut().review = replacement;
        let _ = h.click_id(Dialog::new(GATE).action_id(1));
        assert_eq!(
            h.app().world.effect_revision,
            0,
            "old gate authorized a replaced review"
        );
    }
    #[test]
    fn identical_regenerated_review_cannot_reuse_old_acknowledgement() {
        let mut h = Harness::new(fixture(), Theme::junie(), 120, 40);
        let phrase = h.app().review.plan().phrase().to_owned();
        let _ = h.type_str(&phrase);
        let _ = h.key(KeyCode::Enter);
        h.app_mut().review = plans::plan_for(&h.app().world, "docker.cleanup").unwrap();
        let _ = h.click_id(Dialog::new(GATE).action_id(1));
        assert_eq!(h.app().world.effect_revision, 0);
        assert!(!h.app().review.plan().ran());
        assert_eq!(
            h.app().result.as_deref(),
            Some("Plan selection changed · review it again")
        );
    }
}
