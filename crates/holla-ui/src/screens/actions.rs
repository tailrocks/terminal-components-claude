use super::context::Context;
use crate::{
    domain::action::{Action, Availability},
    sim::world::World,
};
use termrock::{
    Chord, Cx, Hint, HintKey, HintLayer, Id, Item, ItemKey, KeyCode, Picker, PickerAction,
    PickerState, Response, Ui,
};
pub(crate) const PICKER: Id = Id::root("action.picker");
#[derive(Clone)]
pub(crate) enum Choice {
    Run,
    Preview,
    Copy,
    Insert,
    Alternative(String),
    Pin,
    Alias,
    Hide,
    Reset,
}
struct MenuRow {
    key: String,
    choice: Choice,
    label: String,
    detail: String,
    disabled: bool,
}

pub(crate) struct Actions {
    action: Action,
    context: Context,
    state: PickerState,
    rows: Vec<MenuRow>,
}
impl Actions {
    pub(crate) fn new(action: Action, world: &World) -> Self {
        let mut rows = vec![MenuRow {
            key: "run".into(),
            choice: Choice::Run,
            label: "Run".into(),
            detail: if action.command.is_empty() {
                action.reason.clone()
            } else {
                action.command.clone()
            },
            disabled: false,
        }];
        if !action.command.is_empty() {
            rows.push(MenuRow {
                key: "copy".into(),
                choice: Choice::Copy,
                label: "Copy command".into(),
                detail: action.command.clone(),
                disabled: false,
            });
            rows.push(MenuRow {
                key: "insert".into(),
                choice: Choice::Insert,
                label: "Insert into shell".into(),
                detail: action.command.clone(),
                disabled: false,
            });
        }
        let available_actions = crate::sim::catalogue::catalogue(world);
        for alternative in &action.alternatives {
            let (detail, disabled) = available_actions
                .iter()
                .find(|candidate| candidate.id == alternative.action_id)
                .map_or_else(
                    || ("Not in this preview".into(), true),
                    |candidate| match &candidate.availability {
                        Availability::Ready => (candidate.reason.clone(), false),
                        Availability::NeedsTrust(file) => {
                            (format!("Trust required · {file}"), false)
                        }
                        Availability::Blocked(reason) => (format!("Unavailable · {reason}"), true),
                    },
                );
            rows.push(MenuRow {
                key: format!("alternative:{}", alternative.key),
                choice: Choice::Alternative(alternative.key.clone()),
                label: alternative.label.clone(),
                detail,
                disabled,
            });
        }
        rows.push(MenuRow {
            choice: Choice::Pin,
            key: "pin".into(),
            label: if world.memory.pin_at(&world.cwd, &action.command) {
                "Unpin here".into()
            } else {
                "Pin here".into()
            },
            detail: format!("ranking memory · {}", world.cwd),
            disabled: false,
        });
        let alias = world.memory.alias_for(&action.command);
        rows.push(MenuRow {
            key: "alias".into(),
            choice: Choice::Alias,
            label: alias.map_or_else(
                || "Set alias…".into(),
                |alias| format!("Change alias ({alias})…"),
            ),
            detail: alias.map_or_else(
                || "teach a short name · the query matches it".into(),
                |alias| format!("current alias · {alias}"),
            ),
            disabled: false,
        });
        let hidden = world.memory.hidden_at(&world.cwd, &action.command);
        rows.push(MenuRow {
            key: "hide".into(),
            choice: Choice::Hide,
            label: if hidden {
                "Restore here".into()
            } else {
                "Hide here".into()
            },
            detail: if hidden {
                "restore this command here".into()
            } else {
                "remove from this folder's list · reset restores".into()
            },
            disabled: false,
        });
        rows.push(MenuRow {
            key: "reset".into(),
            choice: Choice::Reset,
            label: "Reset ranking".into(),
            detail: "clears pin, alias and hide for this command".into(),
            disabled: false,
        });
        rows.push(MenuRow {
            key: "why".into(),
            choice: Choice::Preview,
            label: "Why is this here?".into(),
            detail: format!("{} · {}", action.scope.label(), action.reason),
            disabled: false,
        });
        Self {
            action,
            context: Context::of(world),
            state: PickerState::default(),
            rows,
        }
    }
    pub(crate) fn reviewed_action(&self, world: &World) -> Result<Action, &'static str> {
        if self.context != Context::of(world) {
            return Err("Context changed · reopen actions");
        }
        Ok(self.action.clone())
    }
    fn items(&self) -> Vec<Item<'_>> {
        self.rows
            .iter()
            .map(|row| {
                Item::new(ItemKey::text(&row.key), &row.label)
                    .detail(&row.detail)
                    .disabled(row.disabled)
            })
            .collect()
    }
    pub(crate) fn hints() -> HintLayer {
        HintLayer {
            hints: vec![
                Hint {
                    key: HintKey::Label("↑↓"),
                    label: "Move",
                    priority: 50,
                },
                Hint {
                    key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                    label: "Choose",
                    priority: 50,
                },
                Hint {
                    key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                    label: "Cancel",
                    priority: 50,
                },
            ],
            ..HintLayer::empty()
        }
    }
    fn picker(&self) -> Picker<'_, Item<'_>> {
        Picker::new(PICKER)
            .title(&self.action.title)
            .width(64)
            .searchable(false)
    }
    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        cx.open_layer(PICKER, self.picker().layer(cx, &self.items()));
    }
    pub(crate) fn update(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<Choice>, bool) {
        let mut state = std::mem::take(&mut self.state);
        let mut response = self.picker().update(cx, &mut state, &self.items());
        self.state = state;
        let choice = match response.take_action() {
            Some(PickerAction::Chosen(key)) => self
                .rows
                .iter()
                .find(|row| ItemKey::text(&row.key) == key)
                .map(|row| row.choice.clone()),
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
            self.picker().draw(ui, area, &self.state, &self.items())
        });
    }
}
