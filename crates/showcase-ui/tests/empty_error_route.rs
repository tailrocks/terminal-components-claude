use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

const NAV: Id = Id::root("showcase_ui::app::navigation");
const PAGE_TABS: Id = Id::root("showcase_ui::pages::empty_error.page_tabs");
const OPTION_KIND: Id = Id::root("showcase_ui::pages::empty_error.option.kind");
const OPTION_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.option.actions");
const OPTION_REASON: Id = Id::root("showcase_ui::pages::empty_error.option.reason");
const SAVED_LIST: Id = Id::root("showcase_ui::pages::empty_error.saved_queries");
const CONNECTION_PRIMARY: Id = Id::root("showcase_ui::pages::empty_error.connection.primary");
const CONNECTION_IMPORT: Id = Id::root("showcase_ui::pages::empty_error.connection.import");
const ORDERS_GRID: Id = Id::root("showcase_ui::pages::empty_error.orders.grid");
const ROW_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.orders.actions");
const ROW_MENU: Id = Id::root("showcase_ui::pages::empty_error.orders.menu");

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

fn click_tab(h: &mut Harness<App>, index: usize) {
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn choose(h: &mut Harness<App>, id: Id, current: usize, target: usize) {
    let _ = h.click_id(id);
    let key = if target < current {
        KeyCode::Up
    } else {
        KeyCode::Down
    };
    for _ in 0..current.abs_diff(target) {
        let _ = h.key(key);
    }
    let _ = h.key(KeyCode::Enter);
}

fn is_empty_target(h: &Harness<App>) -> bool {
    format!("{:?}", h.app()).contains("nav_target: Placeholder(31)")
}

#[test]
fn empty_alias_opens_the_private_placeholder_without_changing_catalog_identity() {
    let index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Empty & error")
        .expect("Empty & error remains in the navigation catalog");
    assert_eq!(index, 31);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(NAV_ENTRIES[index].id, PageId::Overview);
    assert_eq!(PageId::ALL.len(), 24);
    assert_eq!(PageId::from_name("empty"), None);
    assert_eq!(PageId::from_name("Empty & error"), None);

    for name in ["Empty & error", "empty"] {
        let mut h = Harness::new(App::for_page_name(name), Theme::junie(), 120, 40);
        assert_eq!(h.app().page(), PageId::Overview);
        assert!(is_empty_target(&h), "{name} keeps Placeholder(31)");
        assert_eq!(h.focus(), Some(NAV));
        assert!(
            h.area_of(PAGE_TABS).is_some(),
            "{name} mounts the live page"
        );
        assert!(
            h.area_of(SAVED_LIST).is_some(),
            "{name} mounts the empty List"
        );
        assert!(
            h.area_of(CONNECTION_PRIMARY).is_some(),
            "{name} mounts actions"
        );
        assert!(
            h.area_of(ROW_ACTIONS).is_some(),
            "{name} mounts row actions"
        );
        assert!(
            h.area_of(ORDERS_GRID).is_some(),
            "{name} mounts the read-only grid"
        );
        assert!(
            h.text().contains("No saved queries")
                && h.text().contains("Nothing matches “ordrs”")
                && h.text().contains("No connections yet"),
            "{name} renders the real three-state body\n{}",
            h.text()
        );

        let _ = h.key(KeyCode::Enter);
        assert_eq!(h.focus(), Some(PAGE_TABS), "Enter reaches the live body");
        let hints = footer(&h);
        assert!(
            hints.contains("Section"),
            "page-tab hints are active: {hints}"
        );
        assert!(
            hints.contains("Activate"),
            "page-tab hints are active: {hints}"
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn empty_page_options_real_actions_and_refusal_survive_route_away_and_back() {
    let mut h = Harness::new(App::for_page_name("empty"), Theme::junie(), 120, 40);
    click_tab(&mut h, 1);
    for id in [OPTION_KIND, OPTION_ACTIONS, OPTION_REASON] {
        assert!(
            h.area_of(id).is_some(),
            "the live option is present: {id:?}"
        );
    }
    choose(&mut h, OPTION_KIND, 0, 1);
    choose(&mut h, OPTION_ACTIONS, 2, 1);
    choose(&mut h, OPTION_REASON, 0, 1);

    click_tab(&mut h, 0);
    let text = h.text();
    assert!(text.contains("Could not load connections"), "{text}");
    assert!(text.contains("The config file is unreadable"), "{text}");
    assert!(text.contains("Retry"), "{text}");
    assert!(h.area_of(CONNECTION_PRIMARY).is_some());
    assert!(h.area_of(CONNECTION_IMPORT).is_none());

    assert!(h.tab_to(CONNECTION_PRIMARY));
    let _ = h.key(KeyCode::Enter);
    assert!(
        footer(&h).contains("Retry selected for the connection sample"),
        "the real Retry action reports its local effect: {}",
        footer(&h)
    );

    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("orders_view has no primary key, so rows cannot be told apart"),
        "the real grid refusal reaches the status frame\n{}",
        h.text()
    );
    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.is_open(ROW_MENU), "the real row menu opens from the page");
    assert!(h.text().contains("no primary key"), "{}", h.text());
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROW_MENU));

    for _ in 0..3 {
        let _ = h.key(KeyCode::Char(']'));
    }
    assert_eq!(h.app().page(), PageId::Panels);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Char('['));
    }
    assert!(is_empty_target(&h));
    assert_eq!(
        h.app().page(),
        PageId::Panels,
        "the private route preserves the last public PageId"
    );
    assert!(h.text().contains("Could not load connections"));
    assert!(h.area_of(CONNECTION_IMPORT).is_none());
    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("orders_view has no primary key, so rows cannot be told apart"),
        "the option state survives returning to the private route\n{}",
        h.text()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
