use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::diff.page_tabs");
const FILE_LIST: Id = Id::root("showcase_ui::pages::diff.files");
const LIVE_VIEW: Id = Id::root("showcase_ui::pages::diff.view");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const HELP: Id = Id::root("showcase_ui::app::help");

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

fn is_diff_target(h: &Harness<App>) -> bool {
    format!("{:?}", h.app()).contains("nav_target: Placeholder(40)")
}

#[test]
fn diff_placeholder_opens_real_body_and_keeps_its_catalog_neighbor() {
    let mut h = Harness::new(App::for_page_name("Diff"), Theme::junie(), 120, 40);

    let diff_index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Diff")
        .expect("Diff remains in the navigation catalog");
    assert_eq!(diff_index, 40);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(PageId::ALL.len(), 24);
    assert_eq!(
        h.app().page(),
        PageId::Overview,
        "public placeholder ID stays compatible"
    );
    assert!(is_diff_target(&h), "Diff retains Placeholder(40)");
    assert!(
        h.area_of(PAGE_TABS).is_some(),
        "the live Diff page tabs are mounted"
    );
    assert!(
        h.area_of(FILE_LIST).is_some(),
        "the live file chooser is mounted"
    );
    assert!(
        h.area_of(LIVE_VIEW).is_some(),
        "the shared DiffView is mounted"
    );
    assert!(h.text().contains("Diff viewer"), "{}", h.text());
    assert!(h.text().contains("config/service.toml"), "{}", h.text());

    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().page(), PageId::Terminal);
    assert!(
        h.text().contains("Terminal"),
        "Down still reaches the next catalog page"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn diff_file_choice_survives_route_away_and_back() {
    let mut h = Harness::new(App::for_page_name("Diff"), Theme::junie(), 120, 40);
    let added_file = PartRef::item(Part::ROW, ItemKey::index(1));
    assert!(h.area_of_part(FILE_LIST, added_file).is_some());
    let _ = h.click_part(FILE_LIST, added_file);
    assert!(h.text().contains("docs/retries.md"), "{}", h.text());
    assert!(h.text().contains("# Retries"), "{}", h.text());

    let _ = h.key(KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Terminal);
    let _ = h.key(KeyCode::Char('['));
    assert!(is_diff_target(&h));
    assert_eq!(
        h.app().page(),
        PageId::Terminal,
        "placeholder routing retains the previous public page ID"
    );
    assert!(
        h.text().contains("docs/retries.md"),
        "selected file survives route\n{}",
        h.text()
    );
    assert!(
        h.text().contains("# Retries"),
        "selected source survives route\n{}",
        h.text()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn filtered_navigation_can_activate_the_diff_placeholder() {
    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    assert_eq!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str("Diff");
    assert!(
        h.text().contains("/ Diff"),
        "filtered catalog row is visible\n{}",
        h.text()
    );
    let _ = h.key(KeyCode::Enter);

    assert!(is_diff_target(&h));
    assert!(h.area_of(PAGE_TABS).is_some());
    assert!(h.area_of(LIVE_VIEW).is_some());
    assert!(h.text().contains("config/service.toml"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn diff_focus_copy_route_and_modal_hints_belong_to_the_live_page() {
    let mut h = Harness::new(App::for_page_name("Diff"), Theme::junie(), 120, 40);
    assert_eq!(h.focus(), Some(NAV));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.focus(),
        Some(PAGE_TABS),
        "Enter opens the live placeholder body"
    );
    assert!(
        footer(&h).contains("Section"),
        "page tab hints are active\n{}",
        footer(&h)
    );

    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(NAV), "Escape returns focus to navigation");
    assert!(
        h.tab_to(LIVE_VIEW),
        "the actual shared DiffView is keyboard reachable"
    );
    let (start_x, start_y) = h.find("cafe").expect("the Unicode source row is visible");
    let (cup_x, cup_y) = h.find("☕").expect("the same source row contains the cup");
    assert_eq!(start_y, cup_y);
    let endpoint = cup_x.saturating_add(termrock::width("☕").max(1));
    let view = h.area_of(LIVE_VIEW).expect("active source has a view");
    assert!(
        endpoint < view.right(),
        "copy selection stays in the real viewport: {view:?}"
    );
    let _ = h.drag((start_x, start_y), (endpoint, start_y));
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    assert!(
        footer(&h).contains("Capture"),
        "selection hints are active\n{}",
        footer(&h)
    );
    let _ = h.key(KeyCode::Char('y'));
    assert!(h.text().contains("Selection captured"), "{}", h.text());
    assert!(
        footer(&h).contains("Captured selection: cafe\u{301} ☕"),
        "{}",
        footer(&h)
    );

    let help = h
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("shell Help control is visible");
    let _ = h.click(help.x.saturating_add(1), help.y);
    assert!(h.is_open(HELP));
    assert!(
        footer(&h).contains("Close"),
        "modal owns footer hints\n{}",
        footer(&h)
    );
    assert!(
        !footer(&h).contains("y Capture") && !footer(&h).contains("Esc Clear"),
        "page action hints are hidden by Help\n{}",
        footer(&h)
    );
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(HELP));
    assert_eq!(
        h.focus(),
        Some(LIVE_VIEW),
        "Help returns focus to the DiffView"
    );

    let _ = h.key(KeyCode::Char(']'));
    let _ = h.key(KeyCode::Char('['));
    assert!(is_diff_target(&h));
    assert!(
        h.text().contains("Selection captured"),
        "copy feedback survives route roundtrip\n{}",
        h.text()
    );
    let _ = h.resize(80, 24);
    let files = h.area_of(FILE_LIST).expect("file list survives resize");
    let view = h.area_of(LIVE_VIEW).expect("DiffView survives resize");
    assert_eq!(files.x, view.x, "narrow app shell stacks the page panes");
    assert!(view.y >= files.bottom());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
