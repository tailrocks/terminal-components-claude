use ratatui_core::layout::Rect;
use termrock_controls::Button;
use termrock_core::action::{Action, ActionKey};
use termrock_core::event::{Chord, KeyCode, KeyModifiers};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::{Dialog, DialogAction, DialogState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("dialog.primary.contract");
const OPEN: Id = Id::root("dialog.primary.open");
const DONE: ActionKey = ActionKey::custom("dialog.primary.done");
const ALT: ActionKey = ActionKey::custom("dialog.primary.alt");
const PRIMARY: ActionKey = ActionKey::custom("dialog.primary.submit");

const REORDERED: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative"),
    Action::new(PRIMARY, "Submit"),
];
const DISABLED_PRIMARY: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative"),
    Action::new(PRIMARY, "Submit").enabled(false),
];
const MISSING_PRIMARY: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative"),
];
const CANCEL_PRIMARY: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative"),
];
const CHORDED_ALTERNATIVE: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative").chord(Chord::with(KeyCode::Char('k'), KeyModifiers::CONTROL)),
    Action::new(PRIMARY, "Submit"),
];
const EMPTY: &[Action<'static>] = &[];
const TRUNCATED: &[Action<'static>] = &[
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ALT, "Alternative 1"),
    Action::new(ActionKey::custom("dialog.primary.2"), "Alternative 2"),
    Action::new(ActionKey::custom("dialog.primary.3"), "Alternative 3"),
    Action::new(ActionKey::custom("dialog.primary.4"), "Alternative 4"),
    Action::new(ActionKey::custom("dialog.primary.5"), "Alternative 5"),
    Action::new(ActionKey::custom("dialog.primary.6"), "Alternative 6"),
    Action::new(ActionKey::custom("dialog.primary.7"), "Alternative 7"),
    Action::new(PRIMARY, "Outside effective action range"),
];

struct Page {
    state: DialogState,
    actions: &'static [Action<'static>],
    primary: Option<ActionKey>,
    opened_once: bool,
    focus_alternative: bool,
    focus_requested: bool,
    selected: Vec<ActionKey>,
}

impl Page {
    fn new(actions: &'static [Action<'static>], primary: Option<ActionKey>) -> Self {
        Self {
            state: DialogState::default(),
            actions,
            primary,
            opened_once: false,
            focus_alternative: false,
            focus_requested: false,
            selected: Vec::new(),
        }
    }

    fn dialog(&self) -> Dialog<'static> {
        let dialog = Dialog::prompt(ID, "Choose action", "Value").actions(self.actions);
        if let Some(primary) = self.primary {
            dialog.primary(primary)
        } else {
            dialog
        }
    }

    fn open(&self, cx: &mut Cx<'_>) {
        let dialog = self.dialog();
        cx.open_layer(ID, dialog.layer(cx));
        cx.focus(dialog.input_id());
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.opened_once {
            self.opened_once = true;
            self.open(cx);
        }

        let mut response = Button::new(OPEN, "Open prompt")
            .update(cx)
            .on_activated(|| self.open(cx));

        let dialog = self.dialog();
        let mut dialog_response = dialog.update(cx, &mut self.state);
        if let Some(DialogAction::Action(key)) = dialog_response.take_action() {
            self.selected.push(key);
            cx.close_layer(ID, Some(DONE));
        }
        if self.focus_alternative && !self.focus_requested && cx.is_open(ID) {
            cx.focus(dialog.action_id(1));
            self.focus_requested = true;
        }
        response |= dialog_response.erase();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        Button::new(OPEN, "Open prompt").draw(ui, full);
        ui.layer(ID, |ui, area: Rect| {
            self.dialog().draw(ui, area, &self.state, |_, _| {});
        });
    }
}

fn page(actions: &'static [Action<'static>], primary: Option<ActionKey>) -> Harness<Page> {
    let harness = Harness::new(Page::new(actions, primary), Theme::junie(), 80, 24);
    assert!(
        harness.is_open(ID),
        "button did not open prompt; ring: {:?}",
        harness.ring().entries()
    );
    assert_eq!(harness.focus(), Some(harness.app().dialog().input_id()));
    harness
}

fn page_with_focused_alternative() -> Harness<Page> {
    page_with_focused_action(REORDERED, Some(PRIMARY))
}

fn page_with_focused_action(
    actions: &'static [Action<'static>],
    primary: Option<ActionKey>,
) -> Harness<Page> {
    let mut app = Page::new(actions, primary);
    app.focus_alternative = true;
    Harness::new(app, Theme::junie(), 80, 24)
}

#[test]
fn prompt_commit_uses_configured_primary_after_reordering_and_unicode_input() {
    let mut h = page(REORDERED, Some(PRIMARY));
    let _ = h.type_str("猫é");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().selected, vec![PRIMARY]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn disabled_missing_truncated_and_empty_primary_never_fall_back() {
    for (actions, primary) in [
        (DISABLED_PRIMARY, Some(PRIMARY)),
        (MISSING_PRIMARY, Some(PRIMARY)),
        // Replacing prompt actions without replacing the inherited CONFIRM
        // primary must also fail closed.
        (MISSING_PRIMARY, None),
        (TRUNCATED, Some(PRIMARY)),
        (EMPTY, Some(PRIMARY)),
    ] {
        let mut h = page(actions, primary);
        let _ = h.type_str("input");
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.app().selected.is_empty(),
            "unexpected fallback for {actions:?}"
        );
        assert!(h.is_open(ID), "prompt closed for {actions:?}");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn explicit_cancel_primary_is_honored_when_enabled() {
    let mut h = page(CANCEL_PRIMARY, Some(ActionKey::CANCEL));
    let _ = h.type_str("cancel");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().selected, vec![ActionKey::CANCEL]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn focused_alternative_and_pointer_activation_keep_their_own_keys() {
    let mut h = page_with_focused_alternative();
    let alternative = h.app().dialog().action_id(1);
    assert_eq!(h.focus(), Some(alternative));
    let _ = h.key(KeyCode::Char(' '));
    assert_eq!(h.app().selected, vec![ALT]);

    let mut h = page_with_focused_alternative();
    let alternative = h.app().dialog().action_id(1);
    assert_eq!(h.focus(), Some(alternative));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().selected, vec![ALT]);

    let mut h = page(REORDERED, Some(PRIMARY));
    let alternative = h.app().dialog().action_id(1);
    let alt_area = h
        .area_of(alternative)
        .expect("alternative button is rendered");
    let _ = h.click(
        alt_area.x.saturating_add(alt_area.width / 2),
        alt_area.y.saturating_add(alt_area.height / 2),
    );
    assert_eq!(h.app().selected, vec![ALT], "action area {alt_area:?}");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn custom_action_chord_remains_eligible_while_prompt_is_open() {
    let mut h = page_with_focused_action(CHORDED_ALTERNATIVE, Some(PRIMARY));
    let _ = h.ctrl('k');
    assert_eq!(h.app().selected, vec![ALT]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn pointer_primary_activation_is_not_preempted_by_prompt_blur() {
    let mut h = page(REORDERED, Some(PRIMARY));
    let primary = h.app().dialog().action_id(2);
    let area = h.area_of(primary).expect("primary button is rendered");
    let _ = h.click(
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    assert_eq!(h.app().selected, vec![PRIMARY]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn dismissal_precedes_prompt_commit_and_repeated_open_close_is_stable() {
    let mut h = page(REORDERED, Some(PRIMARY));
    let _ = h.type_str("draft");
    let _ = h.key(KeyCode::Esc);
    assert!(h.app().selected.is_empty(), "first Esc clears the edit");
    assert!(h.is_open(ID));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().selected, vec![ActionKey::CANCEL]);
    assert!(!h.is_open(ID));

    let _ = h.click_id(OPEN);
    let _ = h.type_str("second");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().selected, vec![ActionKey::CANCEL, PRIMARY]);
    assert!(!h.is_open(ID));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
