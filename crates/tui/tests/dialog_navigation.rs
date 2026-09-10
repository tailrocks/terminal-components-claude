//! Dialog action arrows are published beside ordinary button activation bindings.
use junie_tui::{
    Action, ActionKey, App, Cx, Dialog, DialogAction, DialogState, Id, KeyCode, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("dialog.navigation");
const ACTIONS: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Close"),
    Action::new(ActionKey::CONFIRM, "Open"),
];
fn dialog() -> Dialog<'static> {
    Dialog::new(ID).title("Snapshot").actions(ACTIONS)
}
#[derive(Default)]
struct Page {
    state: DialogState,
    opened: bool,
    actions: Vec<ActionKey>,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.opened {
            self.opened = true;
            cx.open_layer(ID, dialog().layer(cx));
            cx.focus(dialog().action_id(0));
        }
        let mut response = dialog().update(cx, &mut self.state);
        if let Some(DialogAction::Action(key)) = response.take_action() {
            self.actions.push(key);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            dialog().draw(ui, area, &self.state, |_, _| {});
        });
    }
}
#[test]
fn right_then_enter_activates_next_action_without_binding_collisions() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 80, 24);
    assert_eq!(h.focus(), Some(dialog().action_id(0)));
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.focus(), Some(dialog().action_id(1)));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().actions, vec![ActionKey::CONFIRM]);
    let _ = h.key(KeyCode::Left);
    assert_eq!(h.focus(), Some(dialog().action_id(0)));
    let _ = h.key(KeyCode::Char(' '));
    assert_eq!(h.app().actions, vec![ActionKey::CONFIRM, ActionKey::CANCEL]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn default_enter_and_pointer_still_activate_exactly_once() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 80, 24);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().actions, vec![ActionKey::CANCEL]);
    let _ = h.click_id(dialog().action_id(1));
    assert_eq!(h.app().actions, vec![ActionKey::CANCEL, ActionKey::CONFIRM]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
