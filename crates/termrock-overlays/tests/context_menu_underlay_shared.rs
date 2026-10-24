use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyCode};
use termrock_core::id::{Id, Part, PartRef};
use termrock_core::response::Response;
use termrock_overlays::menu::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, Role, StylePatch, Theme};

const MENU: Id = Id::root("consumer.shared-underlay-menu");
const RUN: ActionKey = ActionKey::application("consumer.shared-underlay.run");
const OPEN_CHILD: ActionKey = ActionKey::application("consumer.shared-underlay.open-child");
const DELETE: ActionKey = ActionKey::application("consumer.shared-underlay.delete");
const CHILD: &[MenuItem<'static>] = &[MenuItem::new(RUN, "Open child")];
const DISABLED_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(RUN, "Run")
        .chord(Chord::key(KeyCode::Char('x')))
        .separator(),
    MenuItem::new(OPEN_CHILD, "Open child").submenu(CHILD),
    MenuItem::new(DELETE, "Delete")
        .danger()
        .disabled(true)
        .reason("No target"),
];
const ENABLED_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(RUN, "Run")
        .chord(Chord::key(KeyCode::Char('x')))
        .separator(),
    MenuItem::new(OPEN_CHILD, "Open child").submenu(CHILD),
    MenuItem::new(DELETE, "Delete").danger(),
];
const BORDER_PATCH: [(Part, StylePatch); 1] = [(
    Part::BORDER,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const TITLE_PATCH: [(Part, StylePatch); 1] = [(
    Part::TITLE,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const ROW_PATCH: [(Part, StylePatch); 1] = [(
    Part::ROW,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const LABEL_PATCH: [(Part, StylePatch); 1] = [(
    Part::LABEL,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const KEY_PATCH: [(Part, StylePatch); 1] = [(
    Part::KEY,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const MARKER_PATCH: [(Part, StylePatch); 1] = [(
    Part::MARKER,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const RULE_PATCH: [(Part, StylePatch); 1] = [(
    Part::RULE,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const HELP_PATCH: [(Part, StylePatch); 1] = [(
    Part::HELP,
    StylePatch::new().set_fg(Role::Custom(Color::Magenta)),
)];
const CONTAINER_CLEAR_PATCH: [(Part, StylePatch); 1] =
    [(Part::CONTAINER, StylePatch::new().clear_bg())];
const GLOBAL_PATCH: StylePatch = StylePatch::new().set_fg(Role::Custom(Color::Magenta));

#[derive(Default)]
struct Page {
    state: MenuState,
    underlay: bool,
    enabled_delete: bool,
    inset: u16,
    slot_part: Option<Part>,
    patch_part: Option<Part>,
    patch_global: bool,
    clear_container_bg: bool,
    chosen: Option<ActionKey>,
}

impl Page {
    fn menu(&self) -> ContextMenu<'static> {
        let items = if self.enabled_delete {
            ENABLED_ITEMS
        } else {
            DISABLED_ITEMS
        };
        let mut menu = ContextMenu::at(MENU, items, Position::new(1, 1))
            .title("Actions")
            .separator_inset(self.inset)
            .skip_disabled(false);
        if self.underlay {
            menu = menu.keep_underlay();
        }
        if let Some(part) = self.patch_part {
            menu = menu.patch_part(patch_for(part));
        }
        if self.patch_global {
            menu = menu.patch(&GLOBAL_PATCH);
        }
        if self.clear_container_bg {
            menu = menu.patch_part(&CONTAINER_CLEAR_PATCH);
        }
        if let Some(part) = self.slot_part {
            menu = menu.slot(part, &slot_marker);
        }
        menu
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(MenuAction::Chosen(action)) =
            self.menu().update(cx, &mut self.state).action_ref()
        {
            self.chosen = Some(*action);
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let x = 1.min(full.width);
        let y = 1.min(full.height);
        let area = Rect::new(
            x,
            y,
            full.width.saturating_sub(x.saturating_add(1)).min(35),
            full.height.saturating_sub(y.saturating_add(1)).min(8),
        );
        let _ = self.menu().draw(ui, area, &self.state);
    }
}

fn slot_marker(ui: &mut Ui<'_>, area: Rect) {
    let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Danger));
    let _ = ui.paint_str(area, "§", style);
}

fn patch_for(part: Part) -> &'static [(Part, StylePatch)] {
    match part {
        Part::BORDER => &BORDER_PATCH,
        Part::CONTAINER => &CONTAINER_CLEAR_PATCH,
        Part::TITLE => &TITLE_PATCH,
        Part::ROW => &ROW_PATCH,
        Part::LABEL => &LABEL_PATCH,
        Part::KEY => &KEY_PATCH,
        Part::MARKER => &MARKER_PATCH,
        Part::RULE => &RULE_PATCH,
        Part::HELP => &HELP_PATCH,
        _ => &[],
    }
}

fn page(underlay: bool, enabled_delete: bool, color: ColorLevel) -> Harness<Page> {
    Harness::new(
        Page {
            underlay,
            enabled_delete,
            ..Page::default()
        },
        Theme::junie(),
        40,
        10,
    )
    .with_color(color)
}

#[test]
fn underlay_uses_shared_chord_and_disabled_state_in_each_color_profile() {
    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let underlay = page(true, false, color);
        let normal = page(false, true, color);
        assert!(
            underlay.text().contains('x'),
            "effective chord at {color:?}"
        );
        let marker = Theme::junie().design.glyphs.get(GlyphRole::Expanded);
        assert!(
            underlay.text().contains(marker),
            "submenu marker {marker:?} at {color:?}"
        );
        let (x, y) = underlay.find("Delete").expect("disabled row is rendered");
        let disabled = underlay.cell(x, y).clone();
        let enabled = normal.cell(x, y).clone();
        assert_ne!(disabled, enabled, "disabled style at {color:?}");
    }
}

#[test]
fn underlay_uses_shared_border_part_patches() {
    let mut harness = Harness::new(
        Page {
            underlay: true,
            patch_part: Some(Part::BORDER),
            ..Page::default()
        },
        Theme::junie(),
        40,
        10,
    );
    assert_eq!(harness.cell(1, 1).fg, Color::Magenta);

    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Down);
    assert!(harness.text().contains("No target"));
    assert_ne!(harness.app().chosen, Some(DELETE));
}

#[test]
fn part_and_global_style_overrides_apply_in_both_fill_modes() {
    for underlay in [false, true] {
        for part in [
            Part::BORDER,
            Part::TITLE,
            Part::ROW,
            Part::LABEL,
            Part::KEY,
            Part::MARKER,
            Part::RULE,
            Part::HELP,
        ] {
            let mut harness = Harness::new(
                Page {
                    underlay,
                    patch_part: Some(part),
                    ..Page::default()
                },
                Theme::junie(),
                40,
                10,
            );
            if part == Part::HELP {
                let _ = harness.key(KeyCode::Down);
                let _ = harness.key(KeyCode::Down);
            }
            let (x, y) = match part {
                Part::BORDER => (1, 1),
                Part::TITLE => harness.find("Actions").expect("title cell"),
                Part::ROW | Part::LABEL => harness.find("Run").expect("row label cell"),
                Part::KEY => harness.find("x").expect("shortcut cell"),
                Part::MARKER => harness
                    .find(Theme::junie().design.glyphs.get(GlyphRole::Expanded))
                    .expect("submenu marker cell"),
                Part::RULE => (2, 4),
                Part::HELP => harness.find("No target").expect("reason cell"),
                _ => unreachable!(),
            };
            assert_eq!(
                harness.cell(x, y).fg,
                Color::Magenta,
                "{part:?}, underlay={underlay}"
            );
        }

        let harness = Harness::new(
            Page {
                underlay,
                patch_global: true,
                ..Page::default()
            },
            Theme::junie(),
            40,
            10,
        );
        assert_eq!(harness.cell(1, 1).fg, Color::Magenta, "global patch");

        let clear = Harness::new(
            Page {
                underlay,
                clear_container_bg: true,
                ..Page::default()
            },
            Theme::junie(),
            40,
            10,
        );
        assert_eq!(clear.cell(3, 7).bg, Color::Reset, "container clear bg");
    }
}

#[test]
fn underlay_keyboard_shortcut_uses_the_shared_action_path() {
    let mut harness = page(true, false, ColorLevel::TrueColor);
    let _ = harness.key(KeyCode::Char('x'));
    assert_eq!(harness.app().chosen, Some(RUN));
}

#[test]
fn named_part_slots_are_used_in_default_and_underlay_modes() {
    for underlay in [false, true] {
        for part in [
            Part::BORDER,
            Part::TITLE,
            Part::LABEL,
            Part::KEY,
            Part::MARKER,
            Part::RULE,
        ] {
            let harness = Harness::new(
                Page {
                    underlay,
                    slot_part: Some(part),
                    ..Page::default()
                },
                Theme::junie(),
                40,
                10,
            );
            assert!(
                harness.text().contains('§'),
                "slot {part:?} must draw in underlay={underlay}"
            );
        }

        let mut help = Harness::new(
            Page {
                underlay,
                slot_part: Some(Part::HELP),
                ..Page::default()
            },
            Theme::junie(),
            40,
            10,
        );
        let _ = help.key(KeyCode::Down);
        let _ = help.key(KeyCode::Down);
        assert!(
            help.text().contains('§'),
            "HELP slot in underlay={underlay}"
        );
    }
}

#[test]
fn separator_inset_has_one_mode_independent_resolved_rectangle() {
    for color in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let normal = page(false, false, color);
        let underlay = page(true, false, color);
        let part = PartRef::of(Part::RULE);
        let default_rect = normal.area_of_part(MENU, part).expect("RULE slot bounds");
        assert_eq!(underlay.area_of_part(MENU, part), Some(default_rect));
        assert_eq!(default_rect.width, 33);

        let inset = Harness::new(
            Page {
                underlay: true,
                inset: 1,
                ..Page::default()
            },
            Theme::junie(),
            40,
            10,
        )
        .with_color(color);
        let inset_rect = inset.area_of_part(MENU, part).expect("inset RULE bounds");
        assert_eq!(inset_rect.x, default_rect.x + 1);
        assert_eq!(inset_rect.width, default_rect.width - 2);
    }
}

#[test]
fn underlay_layout_is_total_at_narrow_widths_and_after_resize() {
    let mut harness = Harness::new(
        Page {
            underlay: true,
            ..Page::default()
        },
        Theme::junie(),
        40,
        10,
    );
    for width in [0, 1, 2, 3, 4, 8, 12, 17, 40] {
        let _ = harness.resize(width, 10);
        assert!(
            harness.diagnostics().is_empty(),
            "underlay draw at width={width}: {:?}",
            harness.diagnostics()
        );
    }
}
