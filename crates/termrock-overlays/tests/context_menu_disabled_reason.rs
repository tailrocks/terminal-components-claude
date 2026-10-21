use ratatui_core::layout::{Position, Rect};
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyCode, MouseKind};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const MENU_ID: Id = Id::root("consumer.menu");
const RUN: ActionKey = ActionKey::application("consumer.run");
const COMMIT: ActionKey = ActionKey::application("consumer.commit");
const ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(RUN, "Run"),
    MenuItem::new(COMMIT, "Commit")
        .chord(Chord::key(KeyCode::Char('c')))
        .reason("Choose a target"),
    MenuItem::new(
        ActionKey::application("consumer.unavailable"),
        "Unavailable",
    )
    .disabled(true),
    MenuItem::new(
        ActionKey::application("consumer.empty-reason"),
        "Empty reason",
    )
    .reason(""),
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
        menu().draw(ui, Rect::new(0, 0, 40, 7), &self.state);
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
        7,
    );

    assert_eq!(harness.app().state.cursor(), 0);
    let _ = harness.mouse(termrock_core::event::MouseKind::Move, 4, 2);
    assert_eq!(harness.app().state.cursor(), 0);
    assert!(harness.text().contains(" Choose a target "));

    let _ = harness.key(KeyCode::Down);
    assert_eq!(harness.app().state.cursor(), 1);
    let _ = harness.mouse(MouseKind::Move, 4, 3);
    assert_eq!(harness.app().state.cursor(), 1);
    assert!(harness.text().contains(" Choose a target "));
    let _ = harness.mouse(MouseKind::Move, 4, 4);
    assert_eq!(harness.app().state.cursor(), 1);
    assert!(harness.text().contains(" Choose a target "));
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

#[test]
fn disabled_reason_row_does_not_change_while_pointer_is_held_or_dragged() {
    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut harness = Harness::new(
            Consumer {
                state: MenuState::default(),
                chosen: None,
            },
            Theme::junie().downgrade(color),
            40,
            7,
        );
        let rest = row_cells(&harness, 2);
        let _ = harness.mouse(MouseKind::Move, 4, 2);
        let hovered = row_cells(&harness, 2);
        assert_eq!(hovered, rest, "disabled hover at {color:?}");
        let _ = harness.mouse(MouseKind::Down, 4, 2);
        assert_eq!(
            row_cells(&harness, 2),
            hovered,
            "disabled press at {color:?}"
        );
        let _ = harness.mouse(MouseKind::Drag, 4, 1);
        assert_eq!(
            row_cells(&harness, 2),
            hovered,
            "disabled drag at {color:?}"
        );
        let _ = harness.mouse(MouseKind::Up, 4, 2);
        assert_eq!(
            row_cells(&harness, 2),
            hovered,
            "disabled release at {color:?}"
        );
        assert_eq!(harness.app().chosen, None, "disabled action at {color:?}");
    }
}

fn row_cells(harness: &Harness<Consumer>, y: u16) -> Vec<ratatui_core::buffer::Cell> {
    (0..harness.buffer().area.width)
        .map(|x| {
            harness
                .buffer()
                .cell((x, y))
                .expect("row remains in the menu buffer")
                .clone()
        })
        .collect()
}
