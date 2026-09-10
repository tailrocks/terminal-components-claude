//! Acknowledgement Enter arms and moves to actions without executing.
use junie_tui::{
    Action, ActionKey, App, Cx, Dialog, DialogAction, DialogState, Id, KeyCode, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("ack.keyboard");
const TOKEN: &str = "REMOVE DATA ON devbox";
const ACTIONS: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Run"),
];
const DISABLED: &[Action<'static>] = &[
    Action::new(ActionKey::application("disabled"), "Disabled").enabled(false),
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Run"),
];
struct Page {
    state: DialogState,
    opened: bool,
    disabled: bool,
    label: Option<&'static str>,
    actions: Vec<ActionKey>,
}
impl Page {
    fn dialog(&self) -> Dialog<'static> {
        let dialog = Dialog::acknowledge(ID, "Review", TOKEN).actions(if self.disabled {
            DISABLED
        } else {
            ACTIONS
        });
        if let Some(label) = self.label {
            dialog.input_label(label)
        } else {
            dialog
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.opened {
            self.opened = true;
            cx.open_layer(ID, self.dialog().layer(cx));
            cx.focus(self.dialog().input_id());
        }
        let mut response = self.dialog().update(cx, &mut self.state);
        if let Some(DialogAction::Action(action)) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            self.dialog().draw(ui, area, &self.state, |_, _| {});
        });
    }
}
fn page(disabled: bool) -> Harness<Page> {
    Harness::new(
        Page {
            state: DialogState::default(),
            opened: false,
            disabled,
            label: None,
            actions: Vec::new(),
        },
        Theme::junie(),
        100,
        30,
    )
}
#[test]
fn correct_and_wrong_acknowledgements_move_to_cancel_without_executing() {
    for correct in [false, true] {
        let mut h = page(false);
        let _ = h.type_str(if correct { TOKEN } else { "wrong host" });
        let _ = h.key(KeyCode::Enter);
        assert!(h.app().actions.is_empty());
        assert_eq!(h.focus(), Some(h.app().dialog().action_id(0)));
        assert!(h.app().state.draft().is_empty());
        assert!(!format!("{:?}", h.app().state).contains(TOKEN));
        let _ = h.key(KeyCode::Right);
        assert_eq!(
            h.focus(),
            Some(h.app().dialog().action_id(usize::from(correct)))
        );
        let _ = h.key(KeyCode::Enter);
        assert_eq!(
            h.app().actions,
            vec![if correct {
                ActionKey::CONFIRM
            } else {
                ActionKey::CANCEL
            }]
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
#[test]
fn commit_skips_disabled_action_without_executing_enabled_one() {
    let mut h = page(true);
    let _ = h.type_str(TOKEN);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(h.app().dialog().action_id(1)));
    assert!(h.app().actions.is_empty());
}

#[test]
fn label_override_is_paint_only_and_default_buffers_are_unchanged() {
    let default = page(false);
    let mut explicit = page(false);
    explicit.app_mut().label = Some("Type the token to confirm");
    let _ = explicit.tick();
    assert_eq!(default.buffer(), explicit.buffer());
    let mut custom = page(false);
    custom.app_mut().label = Some("Type the target-bound phrase to confirm");
    let _ = custom.tick();
    assert!(
        custom
            .text()
            .contains("Type the target-bound phrase to confirm")
    );
    let _ = custom.type_str(TOKEN);
    assert!(!custom.text().contains(TOKEN));
    assert!(custom.app().state.draft().is_empty());
    assert!(!format!("{:?}", custom.app().state).contains(TOKEN));
    assert!(!format!("{:?}", custom.app().dialog()).contains(TOKEN));
    let _ = custom.key(KeyCode::Enter);
    assert!(custom.app().actions.is_empty());
    let _ = custom.key(KeyCode::Right);
    let _ = custom.key(KeyCode::Enter);
    assert_eq!(custom.app().actions, vec![ActionKey::CONFIRM]);
}
