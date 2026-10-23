use std::{cell::RefCell, rc::Rc};

use termrock_fields::{TextAction, TextInput, TextInputState};
use termrock_runtime::{App, Cx, Id, Rect, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const SCREEN_WIDTH: u16 = 24;
const SCREEN_HEIGHT: u16 = 5;
const LOCKED: Id = Id::root("disabled-input.locked");
const EDITABLE: Id = Id::root("disabled-input.editable");
const LOCKED_AREA: Rect = Rect::new(1, 1, 18, 1);
const EDITABLE_AREA: Rect = Rect::new(1, 3, 18, 1);

struct InputPage {
    locked_state: TextInputState,
    editable_state: TextInputState,
    locked_value: String,
    editable_value: String,
    locked_actions: Rc<RefCell<Vec<TextAction>>>,
    editable_actions: Rc<RefCell<Vec<TextAction>>>,
}

impl App for InputPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let locked_actions = Rc::clone(&self.locked_actions);
        let locked = TextInput::new(LOCKED)
            .disabled(true)
            .update(cx, &mut self.locked_state, &mut self.locked_value)
            .on_action(move |action| locked_actions.borrow_mut().push(action));

        let editable_actions = Rc::clone(&self.editable_actions);
        let editable = TextInput::new(EDITABLE)
            .update(cx, &mut self.editable_state, &mut self.editable_value)
            .on_action(move |action| editable_actions.borrow_mut().push(action));

        locked | editable
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(LOCKED)
            .disabled(true)
            .value(&self.locked_value)
            .draw(ui, LOCKED_AREA, &self.locked_state);
        TextInput::new(EDITABLE).value(&self.editable_value).draw(
            ui,
            EDITABLE_AREA,
            &self.editable_state,
        );
    }
}

#[test]
fn fully_disabled_input_stays_unchanged_and_does_not_take_keyboard_focus() {
    let mut page = Harness::new(
        InputPage {
            locked_state: TextInputState::default(),
            editable_state: TextInputState::default(),
            locked_value: "locked".to_owned(),
            editable_value: "active".to_owned(),
            locked_actions: Rc::new(RefCell::new(Vec::new())),
            editable_actions: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
    );

    assert_eq!(page.area_of(LOCKED), Some(LOCKED_AREA));
    assert_eq!(page.area_of(EDITABLE), Some(EDITABLE_AREA));
    let locked_state_before = page.app().locked_state.clone();

    let _ = page.click(EDITABLE_AREA.x + 2 + "active".len() as u16, EDITABLE_AREA.y);
    assert_eq!(page.focus(), Some(EDITABLE));
    assert!(page.app().editable_state.is_editing());

    let _ = page.type_str("x");
    assert_eq!(page.app().editable_state.draft_text(), Some("activex"));
    assert_eq!(page.app().editable_value, "active");
    assert_eq!(
        page.app().editable_actions.borrow().as_slice(),
        &[TextAction::Changed],
        "the enabled control is a real keyboard-binding control"
    );

    let _ = page.click(4, LOCKED_AREA.y);
    assert_eq!(
        page.focus(),
        Some(EDITABLE),
        "clicking the disabled field leaves the actual focus owner unchanged"
    );
    assert_eq!(page.app().locked_state, locked_state_before);
    assert!(!page.app().locked_state.is_editing());
    assert_eq!(page.app().locked_state.draft_text(), None);
    assert_eq!(page.app().locked_value, "locked");
    assert!(page.app().locked_actions.borrow().is_empty());

    let _ = page.type_str("y");
    assert_eq!(page.focus(), Some(EDITABLE));
    assert_eq!(page.app().editable_state.draft_text(), Some("activexy"));
    assert_eq!(page.app().editable_value, "active");
    assert_eq!(
        page.app().editable_actions.borrow().as_slice(),
        &[TextAction::Changed, TextAction::Changed]
    );
    assert_eq!(page.app().locked_state, locked_state_before);
    assert_eq!(page.app().locked_value, "locked");
    assert!(page.app().locked_actions.borrow().is_empty());
}
