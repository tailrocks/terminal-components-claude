use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;
use termrock_core::action::ActionKey;
use termrock_core::event::MouseKind;
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::menu::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::stub::Stub;
use termrock_runtime::{App, Cx, Runtime, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const MENU: Id = Id::root("consumer.menu.cursor-contract");
const RUN: ActionKey = ActionKey::application("consumer.menu.cursor.run");
const UNAVAILABLE: ActionKey = ActionKey::application("consumer.menu.cursor.unavailable");
const ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(RUN, "Run"),
    MenuItem::new(UNAVAILABLE, "Unavailable").disabled(true),
];
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn menu() -> ContextMenu<'static> {
    ContextMenu::at(MENU, ITEMS, Position::new(0, 0))
        .skip_disabled(false)
        .clear_on_leave(true)
}

#[derive(Default)]
struct Page {
    state: MenuState,
    chosen: Option<ActionKey>,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(MenuAction::Chosen(action)) = menu().update(cx, &mut self.state).action_ref() {
            self.chosen = Some(*action);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        menu().draw(ui, Rect::new(0, 0, 40, 8), &self.state);
    }
}

#[test]
fn unowned_active_menu_cursor_is_bold_in_every_profile() {
    for level in LEVELS {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().for_level(level));
        let area = Rect::new(0, 0, 50, 10);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                menu().draw(ui, Rect::new(0, 0, 40, 8), &MenuState::default());
            })
            .commit_presented();
        let label = buffer
            .content()
            .iter()
            .find(|cell| cell.symbol() == "R")
            .expect("the active Run label is visible");
        assert!(
            label.modifier.contains(Modifier::BOLD),
            "the active cursor remains bold without owner focus at {level:?}: {label:?}"
        );
    }
}

#[test]
fn pointer_highlight_and_clear_on_leave_have_profile_specific_cues() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let mut harness = Harness::new(Page::default(), theme, 50, 10);
        let active = cell_by_symbol(&harness, "R").modifier;
        assert!(
            active.contains(Modifier::BOLD),
            "active cursor at {level:?}"
        );

        harness.blur();

        let _ = harness.mouse(MouseKind::Move, 4, 1);
        let hovered = cell_by_symbol(&harness, "R").modifier;
        let _ = harness.mouse(MouseKind::Move, 0, 0);
        let hidden = render_unowned_label(level, &harness.app().state);
        assert_eq!(hidden, Modifier::empty(), "hidden cursor at {level:?}");
        assert_ne!(active, hidden, "pointer leave cue at {level:?}");

        match level {
            ColorLevel::TrueColor | ColorLevel::Ansi256 => {
                assert!(hovered.contains(Modifier::BOLD), "hover cue at {level:?}");
                assert!(
                    !hovered.contains(Modifier::UNDERLINED),
                    "hover at {level:?}"
                );
                assert!(!active.contains(Modifier::REVERSED), "active at {level:?}");
            }
            ColorLevel::Ansi16 => {
                assert!(active.contains(Modifier::BOLD));
                assert!(hovered.contains(Modifier::BOLD | Modifier::UNDERLINED));
                assert!(!active.contains(Modifier::UNDERLINED));
                assert!(!active.contains(Modifier::REVERSED));
            }
            ColorLevel::Mono => {
                assert!(active.contains(Modifier::BOLD | Modifier::REVERSED));
                assert!(
                    hovered.contains(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED)
                );
            }
            _ => unreachable!("all supported menu profiles are enumerated"),
        }
    }
}

#[test]
fn disabled_active_menu_item_suppresses_cursor_modifiers_in_every_profile() {
    const DISABLED_ITEMS: &[MenuItem<'static>] =
        &[MenuItem::new(UNAVAILABLE, "Unavailable").disabled(true)];
    let disabled_menu = ContextMenu::at(
        Id::root("consumer.menu.cursor-contract.disabled"),
        DISABLED_ITEMS,
        Position::new(0, 0),
    )
    .skip_disabled(false);
    for level in LEVELS {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().for_level(level));
        let area = Rect::new(0, 0, 50, 10);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                disabled_menu.draw(ui, Rect::new(0, 0, 40, 8), &MenuState::default());
            })
            .commit_presented();
        let label = buffer
            .content()
            .iter()
            .find(|cell| cell.symbol() == "U")
            .expect("disabled label remains visible");
        assert!(
            !label
                .modifier
                .intersects(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED),
            "disabled active cursor must not add cursor modifiers at {level:?}: {label:?}"
        );
    }
}

fn render_unowned_label(level: ColorLevel, state: &MenuState) -> Modifier {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie().for_level(level));
    let area = Rect::new(0, 0, 50, 10);
    let mut buffer = Buffer::empty(area);
    runtime
        .draw_scene(area, &mut buffer, |ui, _| {
            menu().draw(ui, Rect::new(0, 0, 40, 8), state);
        })
        .commit_presented();
    buffer
        .content()
        .iter()
        .find(|cell| cell.symbol() == "R")
        .expect("target label cell remains rendered")
        .modifier
}

fn cell_by_symbol<'a>(harness: &'a Harness<Page>, symbol: &str) -> &'a ratatui_core::buffer::Cell {
    harness
        .buffer()
        .content()
        .iter()
        .find(|cell| cell.symbol() == symbol)
        .expect("target label cell remains rendered")
}
