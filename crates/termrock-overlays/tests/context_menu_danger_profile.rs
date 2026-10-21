use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use termrock_core::action::ActionKey;
use termrock_core::event::{KeyCode, MouseKind};
use termrock_core::id::Id;
use termrock_core::response::Response;
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{
    ColorLevel, Family, Overlay, OverlayRule, Part, Role, StateFlags, StylePatch, Surface, Theme,
    Variant,
};

const MENU: Id = Id::root("menu.danger.profile.integration");
const NORMAL: ActionKey = ActionKey::application("menu.normal");
const REMOVE: ActionKey = ActionKey::application("menu.remove");
const DISABLED_REMOVE: ActionKey = ActionKey::application("menu.disabled-remove");
const ITEMS: [MenuItem<'static>; 3] = [
    MenuItem::new(NORMAL, "Normal row"),
    MenuItem::new(REMOVE, "Remove record").danger(),
    MenuItem::new(DISABLED_REMOVE, "Disabled remove")
        .danger()
        .disabled(true),
];
const SAME_VALUE: Color = Color::Rgb(122, 42, 42);
const DIFFERENT_VALUE: Color = Color::Rgb(1, 2, 3);
const SAME_OVERLAY: [OverlayRule; 1] = [(
    Family::MENU,
    Variant::DANGER,
    Part::ROW,
    StateFlags::ACTIVE,
    StylePatch::new().set_bg(Role::Custom(SAME_VALUE)),
)];
const DIFFERENT_OVERLAY: [OverlayRule; 1] = [(
    Family::MENU,
    Variant::DANGER,
    Part::ROW,
    StateFlags::ACTIVE,
    StylePatch::new().set_bg(Role::Custom(DIFFERENT_VALUE)),
)];
const CLEAR_OVERLAY: [OverlayRule; 1] = [(
    Family::MENU,
    Variant::DANGER,
    Part::ROW,
    StateFlags::ACTIVE,
    StylePatch::new().clear_bg(),
)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatchCase {
    Same,
    Different,
    Clear,
}

impl PatchCase {
    const fn patch(self) -> StylePatch {
        match self {
            Self::Same => StylePatch::new().set_bg(Role::Custom(SAME_VALUE)),
            Self::Different => StylePatch::new().set_bg(Role::Custom(DIFFERENT_VALUE)),
            Self::Clear => StylePatch::new().clear_bg(),
        }
    }

    const fn overlay(self) -> &'static [OverlayRule] {
        match self {
            Self::Same => &SAME_OVERLAY,
            Self::Different => &DIFFERENT_OVERLAY,
            Self::Clear => &CLEAR_OVERLAY,
        }
    }
}

#[derive(Default)]
struct MenuPage {
    state: MenuState,
    chosen: Option<ActionKey>,
    scope_patch: Option<PatchCase>,
    instance_patch: Option<PatchCase>,
    local_patch: Option<PatchCase>,
}

impl App for MenuPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response =
            ContextMenu::at(MENU, &ITEMS, Position::new(0, 0)).update(cx, &mut self.state);
        if let Some(MenuAction::Chosen(action)) = response.action_ref() {
            self.chosen = Some(*action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let instance = self.instance_patch.map(PatchCase::patch);
        let local = self.local_patch.map(PatchCase::patch);
        let local_parts = local.map(|patch| [(Part::ROW, patch)]);
        let mut menu = ContextMenu::at(MENU, &ITEMS, Position::new(0, 0));
        if let Some(patch) = instance.as_ref() {
            menu = menu.patch(patch);
        }
        if let Some(patches) = local_parts.as_ref() {
            menu = menu.patch_part(patches);
        }
        let area = Rect::new(0, 0, 48, 8);
        if let Some(case) = self.scope_patch {
            let overlay = Overlay::new(case.overlay());
            ui.with_overlay(&overlay, |ui| {
                let _ = menu.draw(ui, area, &self.state);
            });
        } else {
            let _ = menu.draw(ui, area, &self.state);
        }
    }
}

fn focused_danger(level: ColorLevel) -> Harness<MenuPage> {
    focused_danger_with(Theme::junie(), MenuPage::default(), level)
}

fn focused_danger_with(theme: Theme, page: MenuPage, level: ColorLevel) -> Harness<MenuPage> {
    let mut harness = Harness::new(page, theme, 64, 12).with_color(level);
    assert!(harness.tab_to(MENU), "context menu receives keyboard focus");
    let _ = harness.key(KeyCode::Down);
    assert_eq!(
        harness.app().state.cursor(),
        1,
        "Down selects the danger row"
    );
    harness
}

#[test]
fn keyboard_active_danger_row_uses_frozen_background_ladder() {
    let cases = [
        (
            ColorLevel::TrueColor,
            [
                Color::Rgb(122, 42, 42),
                Color::Rgb(140, 48, 48),
                Color::Rgb(90, 31, 31),
            ],
            Modifier::BOLD,
            Modifier::BOLD,
            Modifier::BOLD,
            Color::Rgb(255, 255, 255),
        ),
        (
            ColorLevel::Ansi256,
            [Color::Indexed(88), Color::Indexed(124), Color::Indexed(52)],
            Modifier::BOLD,
            Modifier::BOLD,
            Modifier::BOLD,
            Color::Indexed(231),
        ),
        (
            ColorLevel::Ansi16,
            [Color::Red, Color::Red, Color::Red],
            Modifier::BOLD,
            Modifier::BOLD | Modifier::UNDERLINED,
            Modifier::BOLD | Modifier::REVERSED,
            Color::White,
        ),
        (
            ColorLevel::Mono,
            [Color::DarkGray, Color::DarkGray, Color::DarkGray],
            Modifier::BOLD | Modifier::REVERSED,
            Modifier::BOLD | Modifier::REVERSED | Modifier::UNDERLINED,
            Modifier::BOLD | Modifier::UNDERLINED,
            Color::White,
        ),
    ];

    for (level, expected, rest_modifiers, hover_modifiers, press_modifiers, expected_ink) in cases {
        let harness = focused_danger(level);
        let (x, y) = harness.find("Remove record").expect("danger label drawn");
        assert_eq!(harness.cell(x, y).bg, expected[0], "active/rest {level:?}");
        assert_eq!(harness.cell(x, y).fg, expected_ink, "active ink {level:?}");
        assert_eq!(
            harness.cell(x, y).modifier,
            rest_modifiers,
            "keyboard focus modifiers {level:?}"
        );

        let mut hover = focused_danger(level);
        let (x, y) = hover.find("Remove record").expect("danger label drawn");
        let _ = hover.mouse(MouseKind::Move, x, y);
        assert_eq!(
            hover.app().state.cursor(),
            1,
            "hover keeps danger row {level:?}"
        );
        assert_eq!(hover.cell(x, y).bg, expected[1], "active/hover {level:?}");
        assert_eq!(
            hover.cell(x, y).modifier,
            hover_modifiers,
            "hover modifiers {level:?}"
        );

        let mut pressed = focused_danger(level);
        let (x, y) = pressed.find("Remove record").expect("danger label drawn");
        let _ = pressed.mouse(MouseKind::Move, x, y);
        let _ = pressed.mouse(MouseKind::Down, x, y);
        assert_eq!(pressed.cell(x, y).bg, expected[2], "active/press {level:?}");
        assert_eq!(
            pressed.cell(x, y).fg,
            expected_ink,
            "pressed danger ink {level:?}"
        );
        assert_eq!(
            pressed.cell(x, y).modifier,
            press_modifiers,
            "press modifiers {level:?}"
        );
    }
}

#[test]
fn keyboard_activation_returns_the_danger_item_action() {
    let mut harness = focused_danger(ColorLevel::TrueColor);
    let _ = harness.key(KeyCode::Enter);
    assert_eq!(harness.app().chosen, Some(REMOVE));
}

#[test]
fn danger_profile_preserves_same_custom_different_custom_and_clear_overrides() {
    for scope in ["global", "scope", "instance", "local"] {
        for case in [PatchCase::Same, PatchCase::Different, PatchCase::Clear] {
            let mut page = MenuPage::default();
            let mut theme = Theme::junie();
            match scope {
                "global" => {
                    let patch = case.patch();
                    theme = theme.override_variant(Family::MENU, Variant::DANGER, |recipe| {
                        recipe.part(Part::ROW).when(StateFlags::ACTIVE, patch);
                    });
                }
                "scope" => page.scope_patch = Some(case),
                "instance" => page.instance_patch = Some(case),
                "local" => page.local_patch = Some(case),
                _ => unreachable!("all override scopes declared"),
            }
            let harness = focused_danger_with(theme, page, ColorLevel::TrueColor);
            let (x, y) = harness.find("Remove record").expect("danger label drawn");
            let (normal_x, normal_y) = harness.find("Normal row").expect("normal row drawn");
            let expected = match case {
                PatchCase::Same => SAME_VALUE,
                PatchCase::Different => DIFFERENT_VALUE,
                PatchCase::Clear => harness.cell(normal_x, normal_y).bg,
            };
            assert_eq!(harness.cell(x, y).bg, expected, "{scope} {case:?} override");
        }
    }
}

#[test]
fn theme_replacement_invalidates_cached_danger_profile_resolution() {
    let mut harness = focused_danger(ColorLevel::TrueColor);
    let (x, y) = harness.find("Remove record").expect("danger label drawn");
    assert_eq!(harness.cell(x, y).bg, Color::Rgb(122, 42, 42));

    let replacement = Theme::junie().override_variant(Family::MENU, Variant::DANGER, |recipe| {
        recipe.part(Part::ROW).when(
            StateFlags::ACTIVE,
            StylePatch::new().set_bg(Role::Custom(DIFFERENT_VALUE)),
        );
    });
    harness.runtime_mut().set_theme(replacement);
    harness.draw();
    let (x, y) = harness.find("Remove record").expect("danger label redrawn");
    assert_eq!(harness.cell(x, y).bg, DIFFERENT_VALUE);
}

#[test]
fn disabled_active_danger_profile_returns_to_popover_surface() {
    let cases = [
        (
            ColorLevel::TrueColor,
            Color::Rgb(77, 77, 77),
            Color::Rgb(63, 63, 70),
            Modifier::empty(),
        ),
        (
            ColorLevel::Ansi256,
            Color::Indexed(239),
            Color::Indexed(237),
            Modifier::empty(),
        ),
        (
            ColorLevel::Ansi16,
            Color::Gray,
            Color::DarkGray,
            Modifier::DIM,
        ),
        (
            ColorLevel::Mono,
            Color::Gray,
            Color::DarkGray,
            Modifier::DIM,
        ),
    ];
    for (level, expected_fg, expected_bg, expected_modifiers) in cases {
        let theme = Theme::junie().downgrade(level);
        let resolved = theme.resolve(
            Family::MENU,
            Variant::DANGER,
            Part::ROW,
            StateFlags::ACTIVE | StateFlags::DISABLED,
            Surface::Popover,
        );
        assert_eq!(
            resolved.style.as_style().bg,
            Some(expected_bg),
            "disabled row surface {level:?}"
        );
        assert_eq!(
            resolved.style.as_style().fg,
            Some(expected_fg),
            "disabled row ink {level:?}"
        );
        assert_eq!(
            resolved.style.as_style().add_modifier,
            expected_modifiers,
            "disabled row modifiers {level:?}"
        );

        // Disabled danger rows never respond visually to pointer hover or press.
        let disabled_only = theme.resolve(
            Family::MENU,
            Variant::DANGER,
            Part::ROW,
            StateFlags::DISABLED,
            Surface::Popover,
        );
        for ignored_pointer_state in [
            StateFlags::DISABLED | StateFlags::HOVERED,
            StateFlags::DISABLED | StateFlags::PRESSED,
            StateFlags::DISABLED | StateFlags::HOVERED | StateFlags::PRESSED,
            StateFlags::ACTIVE | StateFlags::DISABLED | StateFlags::HOVERED | StateFlags::PRESSED,
        ] {
            let pointer_state = theme.resolve(
                Family::MENU,
                Variant::DANGER,
                Part::ROW,
                ignored_pointer_state,
                Surface::Popover,
            );
            assert_eq!(
                pointer_state.style, disabled_only.style,
                "disabled row ignores pointer states {ignored_pointer_state:?} at {level:?}"
            );
            let label_state = theme.resolve(
                Family::MENU,
                Variant::DANGER,
                Part::LABEL,
                ignored_pointer_state,
                Surface::Popover,
            );
            assert_eq!(
                label_state.style, disabled_only.style,
                "disabled label ignores pointer states {ignored_pointer_state:?} at {level:?}"
            );
        }
    }
}

#[test]
fn disabled_danger_row_matches_frozen_cells_at_rest_hover_and_press() {
    let cases = [
        (
            ColorLevel::TrueColor,
            Color::Rgb(77, 77, 77),
            Color::Rgb(63, 63, 70),
            Modifier::empty(),
        ),
        (
            ColorLevel::Ansi256,
            Color::Indexed(239),
            Color::Indexed(237),
            Modifier::empty(),
        ),
        (
            ColorLevel::Ansi16,
            Color::Gray,
            Color::DarkGray,
            Modifier::DIM,
        ),
        (
            ColorLevel::Mono,
            Color::Gray,
            Color::DarkGray,
            Modifier::DIM,
        ),
    ];
    for (level, fg, bg, modifier) in cases {
        for pointer in [MouseKind::Move, MouseKind::Down] {
            let mut harness = focused_danger(level);
            let (x, y) = harness
                .find("Disabled remove")
                .expect("disabled danger label drawn");
            let _ = harness.mouse(pointer, x, y);
            assert_eq!(harness.app().state.cursor(), 1, "disabled row is skipped");
            for column in x - 2..x + "Disabled remove".len() as u16 {
                let cell = harness.cell(column, y);
                assert_eq!(cell.fg, fg, "disabled row ink {level:?} {pointer:?}");
                assert_eq!(cell.bg, bg, "disabled row surface {level:?} {pointer:?}");
                assert_eq!(
                    cell.modifier, modifier,
                    "disabled row modifiers {level:?} {pointer:?}"
                );
            }
        }
    }
}

#[test]
fn normal_and_disabled_danger_rows_keep_their_own_rendered_state() {
    let harness = focused_danger(ColorLevel::Ansi256);
    let (normal_x, normal_y) = harness.find("Normal row").expect("normal row drawn");
    let (danger_x, danger_y) = harness.find("Remove record").expect("danger row drawn");
    let (disabled_x, disabled_y) = harness.find("Disabled remove").expect("disabled row drawn");
    assert_ne!(
        harness.cell(normal_x, normal_y).bg,
        harness.cell(danger_x, danger_y).bg,
        "danger cursor fill does not leak into normal rows"
    );
    assert_ne!(
        harness.cell(disabled_x, disabled_y).bg,
        harness.cell(danger_x, danger_y).bg,
        "danger cursor fill does not leak into inactive disabled rows"
    );
}
