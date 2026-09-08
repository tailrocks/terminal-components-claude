//! Acknowledgement is exact current input, never a stale committed value.
use junie_tui::{
    ActionKey, App, Cx, Dialog, DialogAction, DialogState, Id, KeyCode, Response, Theme, Ui,
};
use junie_tui_testing::Harness;

const DIALOG: Id = Id::root("ack.regression");
const TOKEN: &str = "delete table";
fn dialog() -> Dialog<'static> {
    Dialog::acknowledge(DIALOG, "Delete", TOKEN)
}
#[derive(Default)]
struct Fixture {
    state: DialogState,
    opened: bool,
    confirmations: usize,
}
impl App for Fixture {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.opened {
            self.opened = true;
            cx.open_layer(DIALOG, dialog().layer(cx));
            cx.focus(dialog().input_id());
        }
        let r = dialog().update(cx, &mut self.state);
        if matches!(
            r.action_ref(),
            Some(DialogAction::Action(ActionKey::CONFIRM))
        ) {
            self.confirmations = self.confirmations.saturating_add(1);
        }
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DIALOG, |ui, area| {
            dialog().draw(ui, area, &self.state, |_, _| {});
        });
    }
}
#[test]
fn exact_live_acknowledgement_confirms_by_pointer_or_enter() {
    for enter in [false, true] {
        let mut h = Harness::new(Fixture::default(), Theme::junie(), 100, 30);
        let _ = h.type_str(TOKEN);
        if enter {
            let _ = h.key(KeyCode::Enter);
            assert_eq!(h.app().confirmations, 0, "editor Enter only commits");
            for _ in 0..3 {
                if h.focus() == Some(dialog().action_id(1)) {
                    break;
                }
                let _ = h.key(KeyCode::Tab);
            }
            assert_eq!(h.focus(), Some(dialog().action_id(1)));
            let _ = h.key(KeyCode::Enter);
        } else {
            let _ = h.click_id(dialog().action_id(1));
        }
        assert_eq!(h.app().confirmations, 1);
        assert!(h.app().state.draft().is_empty());
        let _ = h.click_id(dialog().action_id(1));
        assert_eq!(
            h.app().confirmations,
            1,
            "confirmation must wipe the acknowledgement"
        );
    }
}
#[test]
fn differing_live_acknowledgements_never_confirm() {
    for text in [
        "",
        "delete",
        "Delete table",
        " delete table",
        "delete table ",
        "delete tablex",
    ] {
        let mut h = Harness::new(Fixture::default(), Theme::junie(), 100, 30);
        let _ = h.type_str(text);
        let _ = h.click_id(dialog().action_id(1));
        assert_eq!(h.app().confirmations, 0);
        let _ = h.key(KeyCode::Enter);
        let _ = h.click_id(dialog().action_id(1));
        assert_eq!(h.app().confirmations, 0);
    }
}
#[test]
fn editing_armed_live_acknowledgement_disarms_pointer_confirmation() {
    let mut h = Harness::new(Fixture::default(), Theme::junie(), 100, 30);
    let _ = h.type_str(TOKEN);
    let _ = h.type_str("x");
    let _ = h.click_id(dialog().action_id(1));
    assert_eq!(h.app().confirmations, 0);
}

#[test]
fn changing_committed_acknowledgement_disarms_confirmation() {
    let mut h = Harness::new(Fixture::default(), Theme::junie(), 100, 30);
    let _ = h.type_str(TOKEN);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().confirmations, 0);
    let _ = h.key(KeyCode::Enter); // Re-enter editing the committed value.
    let _ = h.type_str("x");
    let _ = h.click_id(dialog().action_id(1));
    assert_eq!(h.app().confirmations, 0);
}
