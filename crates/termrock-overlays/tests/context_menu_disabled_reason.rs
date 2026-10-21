use ratatui_core::layout::{Position, Rect};
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyCode};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const MENU_ID: Id = Id::root("consumer.menu");
const RUN: ActionKey = ActionKey::application("consumer.run");
const COMMIT: ActionKey = ActionKey::application("consumer.commit");
const ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(RUN, "Run"),
    MenuItem::new(COMMIT, "Commit")
        .chord(Chord::key(KeyCode::Char('c')))
        .reason("Choose a target"),
];

struct Consumer {
    state: MenuState,
    chosen: Option<ActionKey>,
}

fn menu() -> ContextMenu<'static> {
    ContextMenu::at(MENU_ID, ITEMS, Position::new(0, 0))
        .skip_disabled(false)
        .clear_on_leave(true)
}

impl App for Consumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(MenuAction::Chosen(key)) = menu().update(cx, &mut self.state).action_ref() {
            self.chosen = Some(*key);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        menu().draw(ui, Rect::new(0, 0, 40, 6), &self.state);
    }
}

#[test]
fn public_consumer_gets_reason_without_binding_or_activation() {
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
        },
        Theme::junie(),
        40,
        6,
    );

    assert_eq!(harness.app().state.cursor(), 0);
    let _ = harness.mouse(termrock_core::event::MouseKind::Move, 4, 2);
    assert_eq!(harness.app().state.cursor(), 0);
    assert!(harness.text().contains(" Choose a target "));

    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.app().state.cursor(), 1);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Char('c'));
    assert_eq!(harness.app().chosen, None);
    assert!(harness.text().contains(" Choose a target "));

    let _ = harness.key(KeyCode::Home);
    assert!(!harness.text().contains("Choose a target"));
    let _ = harness.key(KeyCode::Char('c'));
    assert_eq!(
        harness.app().chosen,
        None,
        "display reason is never a binding"
    );
}
