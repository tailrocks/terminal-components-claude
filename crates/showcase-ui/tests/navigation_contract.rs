use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{
    Axis, ColorLevel, GlyphRole, Id, KeyCode, Modifier, MouseKind, Part, PartRef, Rect, Surface,
    Theme,
};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HELP: Id = Id::root("showcase_ui::app::help");
const INSPECTOR: Id = Id::root("showcase_ui::app::inspector");

const COLOR_LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn require_nav_row(h: &Harness<App>, label: &str, sidebar_width: usize) -> (u16, u16) {
    for y in 0..h.buffer().area().height {
        let row = h.row(y);
        let sidebar = row.chars().take(sidebar_width).collect::<String>();
        if let Some(byte) = sidebar.find(label) {
            let x = sidebar[..byte].chars().count();
            return (u16::try_from(x).expect("sidebar position fits u16"), y);
        }
    }
    panic!("navigation row {label:?} is not visible\n{}", h.text());
}

fn sidebar_has(h: &Harness<App>, label: &str, sidebar_width: usize) -> bool {
    (0..h.buffer().area().height).any(|y| {
        h.row(y)
            .chars()
            .take(sidebar_width)
            .collect::<String>()
            .contains(label)
    })
}

fn nav_key_state(app: &App) -> (String, String) {
    let debug = format!("{app:?}");
    let field = |name: &str| {
        debug
            .split_once(name)
            .and_then(|(_, value)| value.split(',').next())
            .unwrap_or_else(|| panic!("{name} missing from app debug state: {debug}"))
            .trim()
            .to_owned()
    };
    (field("cursor: "), field("chosen: "))
}

fn assert_no_diagnostics(h: &Harness<App>) {
    assert!(
        h.diagnostics().is_empty(),
        "TUI diagnostics: {:?}",
        h.diagnostics()
    );
}

#[test]
fn sidebar_uses_canvas_idle_selected_and_accent_focus_roles() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in COLOR_LEVELS {
            let resolved = theme.clone().downgrade(level);
            let canvas = resolved.bg(Surface::Canvas);
            let idle_ink = resolved.color.fg[termrock::FgStep::Secondary.index()];
            let primary_ink = resolved.color.fg[termrock::FgStep::Primary.index()];
            let accent_ink = resolved.color.accent;

            let idle = Harness::new(App::new(), theme.clone(), 120, 40).with_color(level);
            let (idle_x, idle_y) = require_nav_row(&idle, "Text areas", 25);
            let idle_cell = idle.cell(idle_x, idle_y);
            assert_eq!(idle_cell.fg, idle_ink, "{level:?} idle ink");
            assert_eq!(idle_cell.bg, canvas, "{level:?} idle canvas");
            assert!(
                !idle_cell.modifier.contains(Modifier::BOLD),
                "{level:?} idle weight"
            );

            let mut selected =
                Harness::new(App::for_page_name("Text areas"), theme.clone(), 120, 40)
                    .with_color(level);
            assert_eq!(selected.app().page(), PageId::TextAreas);
            assert_eq!(selected.focus(), Some(NAV));
            let (selected_x, selected_y) = require_nav_row(&selected, "Text areas", 25);
            let selected_cell = selected.cell(selected_x, selected_y);
            assert_eq!(selected_cell.fg, primary_ink, "{level:?} selected ink");
            assert_eq!(selected_cell.bg, canvas, "{level:?} selected canvas");
            assert!(
                selected_cell.modifier.contains(Modifier::BOLD),
                "{level:?} selected label remains bold"
            );

            let focus_bar = selected.cell(0, selected_y);
            assert_eq!(
                focus_bar.symbol(),
                resolved.design.glyphs.get(GlyphRole::FocusBar),
                "{level:?} focus marker glyph"
            );
            assert_eq!(focus_bar.fg, accent_ink, "{level:?} focus marker ink");
            let chosen = selected.cell(selected_x.saturating_sub(2), selected_y);
            assert_eq!(
                chosen.symbol(),
                resolved.design.glyphs.get(GlyphRole::Chosen),
                "{level:?} selected marker glyph"
            );
            assert_eq!(chosen.fg, accent_ink, "{level:?} selected marker ink");

            let _ = selected.key(KeyCode::Tab);
            assert_ne!(selected.focus(), Some(NAV));
            let (unfocused_x, unfocused_y) = require_nav_row(&selected, "Text areas", 25);
            let unfocused_cell = selected.cell(unfocused_x, unfocused_y);
            assert_eq!(
                unfocused_cell.fg, primary_ink,
                "{level:?} retained selected ink"
            );
            assert_eq!(
                unfocused_cell.bg, canvas,
                "{level:?} retained selected canvas"
            );
            assert!(
                unfocused_cell.modifier.contains(Modifier::BOLD),
                "{level:?} retained selected weight"
            );
            assert_eq!(selected.cell(0, unfocused_y).symbol(), " ");
        }
    }
}

#[test]
fn nav_region_matches_the_sidebar_at_every_width_threshold() {
    for (width, height, sidebar_width) in [
        (80, 24, 20),
        (100, 30, 20),
        (109, 30, 20),
        (110, 30, 25),
        (120, 40, 25),
    ] {
        let h = Harness::new(App::new(), Theme::junie(), width, height);
        let nav = h
            .area_of(NAV)
            .expect("Tree registers its full sidebar region");
        assert_eq!(
            nav,
            Rect::new(0, 2, sidebar_width, height.saturating_sub(4)),
            "terminal {width}x{height}"
        );
        let track = h
            .area_of_part(NAV, PartRef::of(Part::TRACK))
            .expect("expanded catalog overflows the sidebar viewport");
        let thumb = h
            .area_of_part(NAV, PartRef::of(Part::THUMB))
            .expect("overflowing viewport has a thumb");
        assert_eq!(track.x, nav.right().saturating_sub(1));
        assert_eq!(track.y, nav.y);
        assert_eq!(track.height, nav.height);
        assert_eq!(thumb.x, track.x);
        assert!(thumb.y >= track.y && thumb.bottom() <= track.bottom());
        assert_no_diagnostics(&h);
    }
}

#[test]
fn arrows_follow_catalog_neighbors_without_routing_group_rows() {
    let mut cells = Harness::new(App::for_page_name("Cell editors"), Theme::junie(), 120, 40);
    let _ = cells.key(KeyCode::Down);
    assert_eq!(cells.app().page(), PageId::CellEditors);
    assert!(cells.text().contains("Columns"));
    let _ = cells.key(KeyCode::Down);
    assert_eq!(cells.app().page(), PageId::CellEditors);
    assert!(cells.text().contains("Navigation"));
    let _ = cells.key(KeyCode::Down);
    assert_eq!(cells.app().page(), PageId::Lists);
    assert_no_diagnostics(&cells);

    let mut dialogs = Harness::new(App::for_page_name("Dialog kinds"), Theme::junie(), 120, 40);
    let _ = dialogs.key(KeyCode::Down);
    assert_eq!(dialogs.app().page(), PageId::DialogKinds);
    assert!(dialogs.text().contains("Popups & tooltips"));
    let _ = dialogs.key(KeyCode::Down);
    assert_eq!(dialogs.app().page(), PageId::DialogKinds);
    assert!(dialogs.text().contains("File browser"));
    let _ = dialogs.key(KeyCode::Down);
    assert_eq!(dialogs.app().page(), PageId::DialogKinds);
    assert!(dialogs.text().contains("Feedback"));
    let _ = dialogs.key(KeyCode::Down);
    assert_eq!(dialogs.app().page(), PageId::Progress);
    assert_no_diagnostics(&dialogs);

    let mut selection = Harness::new(App::for_page_name("Selection"), Theme::junie(), 120, 40);
    let _ = selection.key(KeyCode::Down);
    assert_eq!(selection.app().page(), PageId::CellEditors);
    assert_no_diagnostics(&selection);

    let mut diff = Harness::new(App::for_page_name("Diff"), Theme::junie(), 120, 40);
    assert_eq!(diff.app().page(), PageId::Overview);
    let _ = diff.key(KeyCode::Down);
    assert_eq!(diff.app().page(), PageId::Terminal);
    assert_no_diagnostics(&diff);
}

#[test]
fn external_placeholder_route_is_revealed_and_survives_responsive_resizes() {
    let mut h = Harness::new(
        App::for_page_name("Read-only report"),
        Theme::junie(),
        80,
        24,
    );
    let expected_index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Read-only report")
        .expect("placeholder identity");
    let expected_target = format!("nav_target: Placeholder({expected_index})");
    let expected_nav_keys = nav_key_state(h.app());
    assert_eq!(h.app().page(), PageId::Overview);
    assert!(h.text().contains("Read-only report"));

    for (width, height, expected_sidebar) in [
        (80, 24, 20),
        (100, 30, 20),
        (109, 30, 20),
        (110, 30, 25),
        (120, 40, 25),
    ] {
        let _ = h.resize(width, height);
        let area = h.area_of(NAV).expect("Tree registers the full sidebar");
        assert_eq!(area.width, expected_sidebar, "width {width}");
        assert!(format!("{:?}", h.app()).contains(&expected_target));
        assert_eq!(nav_key_state(h.app()), expected_nav_keys);
        assert!(h.text().contains("Read-only report"));
    }
    assert_no_diagnostics(&h);
}

#[test]
fn tree_group_disclosure_keeps_the_current_page_and_expansion() {
    let mut h = Harness::new(App::with_page(PageId::Chrome), Theme::junie(), 120, 40);
    let (group_x, group_y) = require_nav_row(&h, "App shell", 25);
    assert!(sidebar_has(&h, "Chrome", 25));

    let _ = h.click(group_x.saturating_add(1), group_y);
    assert_eq!(h.app().page(), PageId::Chrome);
    assert!(!sidebar_has(&h, "Chrome", 25));

    let _ = h.click(group_x.saturating_add(1), group_y);
    assert_eq!(h.app().page(), PageId::Chrome);
    assert!(sidebar_has(&h, "Chrome", 25));
    assert_no_diagnostics(&h);
}

#[test]
fn wheel_changes_the_registered_tree_viewport() {
    let mut h = Harness::new(
        App::for_page_name("Read-only report"),
        Theme::junie(),
        80,
        24,
    );
    let track = h
        .area_of_part(NAV, PartRef::of(Part::TRACK))
        .expect("catalog overflow registers its track");
    let before = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("catalog overflow registers its thumb");
    assert!(before.y > track.y, "external route reveals the tail entry");

    let _ = h.wheel(Axis::V, -8, track.x, track.y.saturating_add(1));
    let after = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("thumb remains registered after wheel scrolling");
    assert_ne!(after.y, before.y, "wheel input moves the real Tree thumb");
    assert_no_diagnostics(&h);
}

#[test]
fn thumb_drag_updates_the_nav_view_and_releases_outside_the_sidebar() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let nav = h.area_of(NAV).expect("full sidebar region");
    let track = h
        .area_of_part(NAV, PartRef::of(Part::TRACK))
        .expect("catalog overflow track");
    let thumb = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("catalog overflow thumb");
    let before_y = thumb.y;
    let grab_y = thumb.y.saturating_add(thumb.height / 2);
    let release_y = track.bottom().saturating_sub(1);

    let _ = h.drag((thumb.x, grab_y), (nav.right(), release_y));

    let after = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("thumb remains registered after outside release");
    assert_ne!(after.y, before_y, "drag moves the actual Tree viewport");
    assert!(after.y >= track.y && after.bottom() <= track.bottom());
    assert_eq!(h.app().page(), PageId::Overview, "scrolling is not routing");
    assert_no_diagnostics(&h);
}

#[test]
fn right_edge_track_blank_rows_and_neighboring_layers_keep_ownership() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    let nav = h.area_of(NAV).expect("full sidebar region");
    assert_eq!(nav, Rect::new(0, 2, 25, 36));

    let original = nav_key_state(h.app());
    for (x, y) in [
        (nav.right(), nav.y.saturating_add(1)),
        (nav.x, nav.y.saturating_sub(1)),
        (nav.x, nav.bottom()),
        (0, 0),
        (0, h.buffer().area().height.saturating_sub(1)),
    ] {
        let _ = h.click(x, y);
        assert_eq!(h.app().page(), PageId::Buttons, "outside click ({x},{y})");
        assert_eq!(nav_key_state(h.app()), original, "outside click ({x},{y})");
    }

    let (main_x, main_y) = h.find("Primary").expect("visible page action");
    assert!(main_x >= nav.right().saturating_add(1));
    let _ = h.click(main_x, main_y);
    assert_eq!(h.app().page(), PageId::Buttons, "main page owns its action");

    let _ = h.key(KeyCode::Char('i'));
    let inspector = h.area_of(INSPECTOR).expect("visible inspector dock");
    let _ = h.click(inspector.x.saturating_add(1), inspector.y.saturating_add(1));
    assert_eq!(h.app().page(), PageId::Buttons, "inspector owns its column");

    let (chrome_x, chrome_y) = require_nav_row(&h, "Chrome", 25);
    let _ = h.key(KeyCode::Char('?'));
    assert!(h.is_open(HELP));
    let modal_cursor = nav_key_state(h.app());
    let _ = h.click(chrome_x, chrome_y);
    assert!(!h.is_open(HELP), "outside click dismisses the help modal");
    assert_eq!(
        h.app().page(),
        PageId::Buttons,
        "modal owns the outside click"
    );
    assert_eq!(
        nav_key_state(h.app()),
        modal_cursor,
        "underlay stays unchanged"
    );
    assert_no_diagnostics(&h);

    let mut blank = Harness::new(App::new(), Theme::junie(), 120, 40);
    let mut sections = Vec::new();
    for entry in NAV_ENTRIES {
        if !entry.section.is_empty() && !sections.contains(&entry.section) {
            sections.push(entry.section);
        }
    }
    for section in sections {
        let (group_x, group_y) = require_nav_row(&blank, section, 25);
        let _ = blank.click(group_x.saturating_add(1), group_y);
        assert_eq!(blank.app().page(), PageId::Overview, "fold {section}");
    }
    let blank_nav = blank.area_of(NAV).expect("Tree region remains registered");
    let blank_y = blank_nav.bottom().saturating_sub(1);
    assert!(
        blank
            .row(blank_y)
            .chars()
            .take(25)
            .collect::<String>()
            .trim()
            .is_empty()
    );
    assert!(
        blank.area_of_part(NAV, PartRef::of(Part::TRACK)).is_none(),
        "a fitting collapsed tree has no scrollbar"
    );
    let _ = blank.click(blank_nav.x.saturating_add(10), blank_y);
    assert_eq!(blank.app().page(), PageId::Overview, "blank row is inert");
    assert_no_diagnostics(&blank);
}

#[test]
fn final_sidebar_column_is_scrollbar_owned_and_rows_remain_clickable() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let nav = h.area_of(NAV).expect("full sidebar region");
    let track = h
        .area_of_part(NAV, PartRef::of(Part::TRACK))
        .expect("overflow track owns the final sidebar column");
    assert_eq!(track.x, nav.right().saturating_sub(1));
    let thumb = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("overflow thumb");
    let before_y = thumb.y;
    let _ = h.click(track.x, track.bottom().saturating_sub(1));
    assert_eq!(h.app().page(), PageId::Overview, "track click scrolls only");
    let after = h
        .area_of_part(NAV, PartRef::of(Part::THUMB))
        .expect("track click preserves thumb");
    assert_ne!(after.y, before_y);

    let mut row = Harness::new(App::new(), Theme::junie(), 120, 40);
    let row_nav = row.area_of(NAV).expect("full sidebar region");
    let (_, chrome_y) = require_nav_row(&row, "Chrome", 25);
    let _ = row.click(row_nav.right().saturating_sub(2), chrome_y);
    assert_eq!(
        row.app().page(),
        PageId::Chrome,
        "adjacent row column remains clickable"
    );
    assert_no_diagnostics(&h);
    assert_no_diagnostics(&row);
}

#[test]
fn tab_enter_escape_and_modal_focus_keep_their_owners() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 120, 40);
    assert_eq!(h.focus(), Some(NAV));

    let _ = h.key(KeyCode::Tab);
    assert_ne!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(NAV));

    let _ = h.key(KeyCode::Enter);
    assert_ne!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(NAV));

    let _ = h.key(KeyCode::Char('?'));
    assert!(h.is_open(HELP));
    assert_ne!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Tab);
    assert!(h.is_open(HELP));
    assert_ne!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(HELP));
    assert_eq!(h.focus(), Some(NAV));
    assert_no_diagnostics(&h);
}

#[test]
fn row_activation_requires_a_matching_press_and_release() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    let (chrome_x, chrome_y) = require_nav_row(&h, "Chrome", 25);
    let (dialogs_x, dialogs_y) = require_nav_row(&h, "Dialogs", 25);

    let _ = h.mouse(MouseKind::Down, chrome_x, chrome_y);
    assert_eq!(h.app().page(), PageId::Overview);
    let _ = h.mouse(MouseKind::Up, dialogs_x, dialogs_y);
    assert_eq!(h.app().page(), PageId::Overview);

    let _ = h.click(chrome_x, chrome_y);
    assert_eq!(h.app().page(), PageId::Chrome);
    assert_no_diagnostics(&h);
}
