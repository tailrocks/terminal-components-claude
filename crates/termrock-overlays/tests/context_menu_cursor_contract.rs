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

        let _ = harness.mouse(MouseKind::Move, 4, 1);
        let hovered = cell_by_symbol(&harness, "R").modifier;
        let _ = harness.mouse(MouseKind::Move, 0, 0);
        let hidden = cell_by_symbol(&harness, "R").modifier;
        assert_eq!(
            hidden,
            Modifier::empty(),
            "owner-focused hidden cursor at {level:?}"
        );
        assert_ne!(active, hidden, "pointer leave cue at {level:?}");

        let expected_active = match level {
            ColorLevel::Mono => Modifier::BOLD | Modifier::REVERSED,
            _ => Modifier::BOLD,
        };
        let expected_hover = match level {
            ColorLevel::TrueColor | ColorLevel::Ansi256 => Modifier::BOLD,
            ColorLevel::Ansi16 => Modifier::BOLD | Modifier::UNDERLINED,
            ColorLevel::Mono => Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED,
            _ => unreachable!("all supported menu profiles are enumerated"),
        };
        assert_eq!(active, expected_active, "active cue at {level:?}");
        assert_eq!(hovered, expected_hover, "hover cue at {level:?}");
        assert_ne!(active, hidden, "pointer leave cue at {level:?}");
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

const MATRIX_MENU: Id = Id::root("consumer.menu.cursor-contract.matrix");

struct MatrixPage {
    state: MenuState,
    item: MenuItem<'static>,
    underlay: bool,
}

impl MatrixPage {
    fn menu(&self) -> ContextMenu<'_> {
        let item = std::slice::from_ref(&self.item);
        let menu = ContextMenu::at(MATRIX_MENU, item, Position::new(0, 0))
            .skip_disabled(false)
            .clear_on_leave(true);
        if self.underlay {
            menu.keep_underlay()
        } else {
            menu
        }
    }
}

impl App for MatrixPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let items = [self.item];
        let menu = ContextMenu::at(MATRIX_MENU, &items, Position::new(0, 0))
            .skip_disabled(false)
            .clear_on_leave(true);
        let menu = if self.underlay {
            menu.keep_underlay()
        } else {
            menu
        };
        menu.update(cx, &mut self.state).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.menu().draw(ui, Rect::new(0, 0, 40, 8), &self.state);
    }
}

fn item(danger: bool, disabled: bool) -> MenuItem<'static> {
    let item = MenuItem::new(RUN, "Run").disabled(disabled);
    if danger { item.danger() } else { item }
}

fn frozen_modifier(level: ColorLevel, state: MenuVisualState, disabled: bool) -> Modifier {
    if disabled {
        return if matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono) {
            Modifier::DIM
        } else {
            Modifier::empty()
        };
    }
    match (level, state) {
        (ColorLevel::TrueColor | ColorLevel::Ansi256, _) => Modifier::BOLD,
        (ColorLevel::Ansi16, MenuVisualState::Initial) => Modifier::BOLD,
        (ColorLevel::Ansi16, MenuVisualState::Hover) => Modifier::BOLD | Modifier::UNDERLINED,
        (ColorLevel::Ansi16, MenuVisualState::Pressed) => Modifier::BOLD | Modifier::REVERSED,
        (ColorLevel::Mono, MenuVisualState::Initial) => Modifier::BOLD | Modifier::REVERSED,
        (ColorLevel::Mono, MenuVisualState::Hover) => {
            Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED
        }
        (ColorLevel::Mono, MenuVisualState::Pressed) => Modifier::BOLD | Modifier::UNDERLINED,
        _ => unreachable!("all supported menu profiles are enumerated"),
    }
}

#[derive(Clone, Copy)]
enum MenuVisualState {
    Initial,
    Hover,
    Pressed,
}

#[test]
fn owner_focused_hidden_cursor_clears_for_both_variants_and_underlay_modes() {
    for level in LEVELS {
        for danger in [false, true] {
            for underlay in [false, true] {
                let page = MatrixPage {
                    state: MenuState::default(),
                    item: item(danger, false),
                    underlay,
                };
                let mut harness = Harness::new(page, Theme::junie().for_level(level), 50, 10);
                let _ = harness.mouse(MouseKind::Move, 4, 1);
                let _ = harness.mouse(MouseKind::Move, 0, 0);
                assert_eq!(
                    cell_by_symbol(&harness, "R").modifier,
                    Modifier::empty(),
                    "owner-focused hidden {level:?} danger={danger} underlay={underlay}"
                );
            }
        }
    }
}

#[test]
fn actual_menu_modifier_matrix_matches_frozen_across_variants_and_underlay_modes() {
    for level in LEVELS {
        for danger in [false, true] {
            for disabled in [false, true] {
                for underlay in [false, true] {
                    let page = MatrixPage {
                        state: MenuState::default(),
                        item: item(danger, disabled),
                        underlay,
                    };
                    let mut harness = Harness::new(page, Theme::junie().for_level(level), 50, 10);
                    let expected = frozen_modifier(level, MenuVisualState::Initial, disabled);
                    assert_eq!(
                        cell_by_symbol(&harness, "R").modifier,
                        expected,
                        "initial {level:?} danger={danger} disabled={disabled} underlay={underlay}"
                    );

                    let _ = harness.mouse(MouseKind::Move, 4, 1);
                    let expected = frozen_modifier(level, MenuVisualState::Hover, disabled);
                    assert_eq!(
                        cell_by_symbol(&harness, "R").modifier,
                        expected,
                        "hover {level:?} danger={danger} disabled={disabled} underlay={underlay}"
                    );

                    let _ = harness.mouse(MouseKind::Down, 4, 1);
                    let expected = frozen_modifier(level, MenuVisualState::Pressed, disabled);
                    assert_eq!(
                        cell_by_symbol(&harness, "R").modifier,
                        expected,
                        "pressed {level:?} danger={danger} disabled={disabled} underlay={underlay}"
                    );
                }
            }
        }
    }
}

fn cell_by_symbol<'a, A: App>(
    harness: &'a Harness<A>,
    symbol: &str,
) -> &'a ratatui_core::buffer::Cell {
    harness
        .buffer()
        .content()
        .iter()
        .find(|cell| cell.symbol() == symbol)
        .expect("target label cell remains rendered")
}
