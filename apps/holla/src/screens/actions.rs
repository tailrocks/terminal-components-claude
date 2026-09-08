//! Structured alternatives for one retained action identity.
use super::context::Context;
use crate::{domain::action::Action, sim::world::World};
use junie_tui::{Cx, Id, Item, ItemKey, Picker, PickerAction, PickerState, Response, Ui};
const PICKER: Id = Id::root("action.picker");
#[derive(Clone, Copy)]
pub(crate) enum Choice {
    Run,
    Preview,
    Copy,
    Pin,
    Alias,
    Hide,
    Reset,
}
const CHOICES: [Choice; 7] = [
    Choice::Run,
    Choice::Preview,
    Choice::Copy,
    Choice::Pin,
    Choice::Alias,
    Choice::Hide,
    Choice::Reset,
];
pub(crate) struct Actions {
    action: Action,
    context: Context,
    state: PickerState,
    labels: Vec<String>,
    details: Vec<String>,
}
impl Actions {
    pub(crate) fn new(action: Action, world: &World) -> Self {
        let labels = [
            "Run",
            "Preview",
            "Copy command",
            if world.memory.pin_at(&world.cwd, &action.command) {
                "Unpin here"
            } else {
                "Pin here"
            },
            if world.memory.alias_for(&action.command).is_some() {
                "Change alias…"
            } else {
                "Set alias…"
            },
            if world.memory.hidden_at(&world.cwd, &action.command) {
                "Unhide here"
            } else {
                "Hide here"
            },
            "Reset ranking",
        ]
        .map(str::to_owned)
        .to_vec();
        let details = vec![
            action.command.clone(),
            "facts before any run".into(),
            action.command.clone(),
            format!("ranking memory · {}", world.cwd),
            "teach a short name · the query matches it".into(),
            "gone from this folder's list · reset restores".into(),
            "clears pin, alias and hide for this command".into(),
        ];
        Self {
            action,
            context: Context::of(world),
            state: PickerState::default(),
            labels,
            details,
        }
    }
    pub(crate) fn reviewed_action(&self, world: &World) -> Result<Action, &'static str> {
        if self.context != Context::of(world) {
            return Err("Context changed · reopen actions");
        }
        Ok(self.action.clone())
    }
    fn items(&self) -> Vec<Item<'_>> {
        self.labels
            .iter()
            .zip(&self.details)
            .enumerate()
            .map(|(index, (label, detail))| {
                Item::new(ItemKey::num(index as u64), label).detail(detail)
            })
            .collect()
    }
    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        cx.open_layer(
            PICKER,
            Picker::new(PICKER)
                .title(&self.action.title)
                .layer(cx, &self.items()),
        );
    }
    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<Choice>, bool) {
        let mut state = std::mem::take(&mut self.state);
        let mut response =
            Picker::new(PICKER)
                .title(&self.action.title)
                .update(cx, &mut state, &self.items());
        self.state = state;
        let choice = match response.take_action() {
            Some(PickerAction::Chosen(key)) => CHOICES
                .into_iter()
                .enumerate()
                .find(|(index, _)| ItemKey::num(*index as u64) == key)
                .map(|(_, choice)| choice),
            _ => None,
        };
        let closed = !cx.is_open(PICKER);
        if choice.is_some() {
            cx.close_layer(PICKER, None);
        }
        (response.erase(), choice, closed)
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(PICKER, |ui, area| {
            Picker::new(PICKER)
                .title(&self.action.title)
                .draw(ui, area, &self.state, &self.items())
        });
    }
}
