use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Axis, Id, KeyCode, MouseKind, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const HELP: Id = Id::root("showcase_ui::app::help");

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
