use termrock_core::action::{Action, ActionKey};
use termrock_core::id::Id;
use termrock_overlays::Dialog;

const APPLY: ActionKey = ActionKey::application("consumer.apply");
const ACTIONS: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::application("consumer.preview"), "Preview"),
    Action::new(APPLY, "Apply"),
];

fn apply_prompt() -> Dialog<'static> {
    Dialog::prompt(Id::root("consumer.apply-prompt"), "Apply changes", "Name")
        .actions(ACTIONS)
        .primary(APPLY)
}

fn main() {
    let prompt = apply_prompt();
    assert_eq!(prompt.id(), Id::root("consumer.apply-prompt"));
    assert_eq!(
        prompt.action_id(2),
        Id::root("consumer.apply-prompt")
            .part(termrock_core::id::Part::ACTIONS)
            .index(2)
    );
}
