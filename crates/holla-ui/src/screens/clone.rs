use crate::{
    domain::{
        action::Action,
        github::{GhRepo, GhState},
    },
    screens::dialogs::CloneBinding,
};
use termrock::{Cx, Id, Item, ItemKey, Picker, PickerAction, PickerState, Response, Ui};

pub(crate) type CloneSelection = (GhRepo, String, Action, GhState, CloneBinding);

const PICKER: Id = Id::root("clone.picker");
fn picker<'s>(filtering: bool) -> Picker<'s, Item<'s>> {
    let picker = Picker::new(PICKER).title("Clone which repository?");
    if filtering {
        picker.placeholder("Filter repositories…")
    } else {
        picker
    }
}
pub(crate) struct ClonePicker {
    action: Action,
    github: GhState,
    binding: CloneBinding,
    login: String,
    repos: Vec<GhRepo>,
    labels: Vec<String>,
    state: PickerState,
}
impl ClonePicker {
    pub(crate) fn new(github: &GhState, action: Action, binding: CloneBinding) -> Self {
        Self {
            action,
            github: github.clone(),
            binding,
            login: github.login.clone(),
            repos: github.repos.clone(),
            labels: github.repos.iter().map(GhRepo::slug).collect(),
            state: PickerState::default(),
        }
    }
    fn items(&self) -> Vec<Item<'_>> {
        self.repos
            .iter()
            .zip(&self.labels)
            .map(|(repo, label)| {
                Item::new(ItemKey::text(label), label)
                    .group(&repo.owner)
                    .tag(if repo.private { "private" } else { "public" })
                    .detail(&repo.default_branch)
            })
            .collect()
    }
    pub(crate) fn open(&self, cx: &mut Cx<'_>) {
        cx.open_layer(PICKER, picker(false).layer(cx, &self.items()));
    }
    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
    ) -> (Response<()>, Option<CloneSelection>, bool) {
        let mut state = std::mem::take(&mut self.state);
        let mut response = picker(true).update(cx, &mut state, &self.items());
        self.state = state;
        let chosen = match response.take_action() {
            Some(PickerAction::Chosen(key)) => self
                .repos
                .iter()
                .zip(&self.labels)
                .find(|(_, label)| ItemKey::text(label) == key)
                .map(|(repo, _)| {
                    (
                        repo.clone(),
                        self.login.clone(),
                        self.action.clone(),
                        self.github.clone(),
                        self.binding.clone(),
                    )
                }),
            _ => None,
        };
        let closed = !cx.is_open(PICKER);
        if chosen.is_some() {
            cx.close_layer(PICKER, None);
        }
        (response.erase(), chosen, closed)
    }
    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(PICKER, |ui, area| {
            picker(true).draw(ui, area, &self.state, &self.items())
        });
    }
}
