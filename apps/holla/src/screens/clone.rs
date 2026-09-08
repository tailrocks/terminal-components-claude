//! Repository selection keeps stable repository identity through shared filtering.
use crate::domain::github::{GhRepo, GhState};
use junie_tui::{Cx, Id, Item, ItemKey, Picker, PickerAction, PickerState, Response, Ui};
const PICKER: Id = Id::root("clone.picker");
pub(crate) struct ClonePicker {
    login: String,
    repos: Vec<GhRepo>,
    labels: Vec<String>,
    state: PickerState,
}
impl ClonePicker {
    pub(crate) fn new(github: &GhState) -> Self {
        Self {
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
        cx.open_layer(
            PICKER,
            Picker::new(PICKER)
                .title("Clone repository")
                .layer(cx, &self.items()),
        );
    }
    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
    ) -> (Response<()>, Option<(GhRepo, String)>, bool) {
        let mut state = std::mem::take(&mut self.state);
        let mut response = Picker::new(PICKER)
            .title("Clone repository")
            .placeholder("Filter repositories…")
            .update(cx, &mut state, &self.items());
        self.state = state;
        let chosen = match response.take_action() {
            Some(PickerAction::Chosen(key)) => self
                .repos
                .iter()
                .zip(&self.labels)
                .find(|(_, label)| ItemKey::text(label) == key)
                .map(|(repo, _)| (repo.clone(), self.login.clone())),
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
            Picker::new(PICKER)
                .title("Clone repository")
                .placeholder("Filter repositories…")
                .draw(ui, area, &self.state, &self.items())
        });
    }
}
