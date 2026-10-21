use ratatui_core::layout::{Position, Rect};
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyCode, MouseKind};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, KeyMap, Ui};
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
    keymap: KeyMap,
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

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

#[test]
fn public_consumer_gets_reason_without_binding_or_activation() {
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
            keymap: KeyMap::default(),
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
fn hidden_highlight_cannot_activate_until_keyboard_navigation_restores_it() {
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
            keymap: KeyMap::default(),
        },
        Theme::junie(),
        40,
        7,
    );

    let _ = harness.mouse(MouseKind::Move, 4, 1);
    assert!(harness.text().contains("Run"));
    let selected_bg = harness.cell(4, 1).bg;
    let _ = harness.mouse(MouseKind::Move, 0, 0);
    assert_ne!(harness.cell(4, 1).bg, selected_bg);

    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.app().chosen,
        None,
        "Enter cannot activate a hidden row"
    );
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(
        harness.app().chosen,
        None,
        "Space cannot activate a hidden row"
    );

    let _ = harness.key(KeyCode::Home);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().chosen, Some(RUN));
}

#[test]
fn physical_enter_and_space_cannot_activate_remapped_item_bindings_when_hidden() {
    for (physical, activation_action, alternate) in [
        (
            KeyCode::Enter,
            ActionKey::custom("menu.context.activate.enter"),
            KeyCode::F(9),
        ),
        (
            KeyCode::Char(' '),
            ActionKey::custom("menu.context.activate.space"),
            KeyCode::F(8),
        ),
    ] {
        let mut keymap = KeyMap::default();
        keymap.remap_component(MENU_ID, RUN, Chord::key(physical));
        keymap.remap_component(MENU_ID, activation_action, Chord::key(alternate));
        let mut harness = Harness::new(
            Consumer {
                state: MenuState::default(),
                chosen: None,
                keymap,
            },
            Theme::junie(),
            40,
            7,
        );

        let _ = harness.mouse(MouseKind::Move, 4, 1);
        let selected_bg = harness.cell(4, 1).bg;
        let _ = harness.mouse(MouseKind::Move, 0, 0);
        assert_ne!(harness.cell(4, 1).bg, selected_bg);
        let _ = harness.key(physical);
        assert_eq!(
            harness.app().chosen,
            None,
            "physical {physical:?} must not activate the remapped item"
        );
        let _ = harness.key(alternate);
        assert_eq!(
            harness.app().chosen,
            Some(RUN),
            "the explicitly remapped activation command remains available"
        );
    }
}

#[test]
fn ordinary_item_letter_shortcut_can_activate_while_highlight_is_hidden() {
    let mut keymap = KeyMap::default();
    keymap.remap_component(MENU_ID, RUN, Chord::key(KeyCode::Char('x')));
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
            keymap,
        },
        Theme::junie(),
        40,
        7,
    );

    let _ = harness.mouse(MouseKind::Move, 4, 1);
    let selected_bg = harness.cell(4, 1).bg;
    let _ = harness.mouse(MouseKind::Move, 0, 0);
    assert_ne!(harness.cell(4, 1).bg, selected_bg);
    let _ = harness.key(KeyCode::Char('x'));
    assert_eq!(harness.app().chosen, Some(RUN));
}

#[test]
fn pointer_activation_after_leave_remains_available() {
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
            keymap: KeyMap::default(),
        },
        Theme::junie(),
        40,
        7,
    );

    let _ = harness.mouse(MouseKind::Move, 4, 1);
    let selected_bg = harness.cell(4, 1).bg;
    let _ = harness.mouse(MouseKind::Move, 0, 0);
    assert_ne!(harness.cell(4, 1).bg, selected_bg);
    let _ = harness.click(4, 1);
    assert_eq!(harness.app().chosen, Some(RUN));
}

#[test]
fn blocked_physical_activation_remapped_to_navigation_keeps_highlight_hidden() {
    for (physical, activate_action, alternate, next_activation) in [
        (
            KeyCode::Char(' '),
            ActionKey::custom("menu.context.activate.space"),
            KeyCode::F(8),
            KeyCode::Enter,
        ),
        (
            KeyCode::Enter,
            ActionKey::custom("menu.context.activate.enter"),
            KeyCode::F(9),
            KeyCode::Char(' '),
        ),
    ] {
        for navigation in [
            ActionKey::custom("menu.context.first"),
            ActionKey::custom("menu.context.last"),
            ActionKey::custom("menu.context.prev.up"),
            ActionKey::custom("menu.context.next.down"),
        ] {
            let mut keymap = KeyMap::default();
            keymap.remap_component(MENU_ID, activate_action, Chord::key(alternate));
            keymap.remap_component(MENU_ID, navigation, Chord::key(physical));
            let mut harness = Harness::new(
                Consumer {
                    state: MenuState::default(),
                    chosen: None,
                    keymap,
                },
                Theme::junie(),
                50,
                10,
            );

            let _ = harness.mouse(MouseKind::Move, 4, 1);
            let selected_bg = harness.cell(4, 1).bg;
            let _ = harness.mouse(MouseKind::Move, 0, 0);
            assert_ne!(harness.cell(4, 1).bg, selected_bg);
            let hidden = (1..39)
                .map(|x| harness.cell(x, 1).clone())
                .collect::<Vec<_>>();
            let _ = harness.key(physical);
            assert_eq!(
                (1..39)
                    .map(|x| harness.cell(x, 1).clone())
                    .collect::<Vec<_>>(),
                hidden,
                "blocked {physical:?} routed to {navigation:?} must not reveal the row"
            );
            let _ = harness.key(next_activation);
            assert_eq!(
                harness.app().chosen,
                None,
                "blocked {physical:?} must leave the stored cursor hidden"
            );
        }
    }
}

#[test]
fn blocked_space_navigation_does_not_enable_following_enter() {
    let mut keymap = KeyMap::default();
    keymap.remap_component(
        MENU_ID,
        ActionKey::custom("menu.context.activate.space"),
        Chord::key(KeyCode::F(8)),
    );
    keymap.remap_component(
        MENU_ID,
        ActionKey::custom("menu.context.first"),
        Chord::key(KeyCode::Char(' ')),
    );
    let mut harness = Harness::new(
        Consumer {
            state: MenuState::default(),
            chosen: None,
            keymap,
        },
        Theme::junie(),
        50,
        10,
    );

    let _ = harness.mouse(MouseKind::Move, 4, 1);
    let selected_bg = harness.cell(4, 1).bg;
    let _ = harness.mouse(MouseKind::Move, 0, 0);
    assert_ne!(harness.cell(4, 1).bg, selected_bg);
    let _ = harness.key(KeyCode::Char(' '));
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(
        harness.app().chosen,
        None,
        "blocked physical Space must not restore the hidden cursor"
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
                keymap: KeyMap::default(),
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
