use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Theme};
use termrock_test_support::Harness;

const TABS: Id = Id::root("showcase_ui::pages::query_workbench.tabs");
const TOOLBAR: Id = Id::root("showcase_ui::pages::query_workbench.toolbar");
const EDITOR: Id = Id::root("showcase_ui::pages::query_workbench.editor");
const RESULTS: Id = Id::root("showcase_ui::pages::query_workbench.results");
const NAV: Id = Id::root("showcase_ui::app::navigation");
const HELP: Id = Id::root("showcase_ui::app::help");
const HEADER_HELP: Id = Id::root("showcase_ui::app::header.help");
const HEADER_INSPECT: Id = Id::root("showcase_ui::app::header.inspect");
const INSPECTOR: Id = Id::root("showcase_ui::app::inspector");

fn orders_key() -> ItemKey {
    ItemKey::text("query.workbench.orders")
}

fn customers_key() -> ItemKey {
    ItemKey::text("query.workbench.customers")
}

fn harness() -> Harness<App> {
    Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        120,
        40,
    )
}

fn click_tab(h: &mut Harness<App>, key: ItemKey) {
    let part = PartRef::item(Part::TAB, key);
    assert!(
        h.area_of_part(TABS, part).is_some(),
        "tab {key:?} is published"
    );
    let _ = h.click_part(TABS, part);
}

fn click_close(h: &mut Harness<App>, key: ItemKey) {
    let part = PartRef::item(Part::CLOSE, key);
    assert!(
        h.area_of_part(TABS, part).is_some(),
        "close target for {key:?} is published"
    );
    let _ = h.click_part(TABS, part);
}

fn press_ctrl_r(h: &mut Harness<App>) {
    let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
}

#[test]
fn query_workbench_is_a_live_route_with_native_tabs_editor_toolbar_and_results() {
    let h = harness();
    let entry = NAV_ENTRIES
        .iter()
        .find(|entry| entry.label == "Query workbench")
        .expect("Query workbench remains in the navigation catalogue");

    assert_eq!(entry.id, PageId::QueryWorkbench);
    assert_eq!(PageId::ALL.len(), 26);
    assert_eq!(PageId::QueryWorkbench.index(), 25);
    assert_eq!(
        PageId::from_name("Query workbench"),
        Some(PageId::QueryWorkbench)
    );
    assert_eq!(h.app().page(), PageId::QueryWorkbench);
    assert!(h.area_of(TABS).is_some());
    assert!(h.area_of(TOOLBAR).is_some());
    assert!(h.area_of(EDITOR).is_some());
    assert!(h.text().contains("orders.sql"));
    assert!(h.text().contains("Pending orders, newest first"));
    assert!(h.text().contains("Results"));
    assert!(h.text().contains("sample data · no database connection"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_workbench_ctrl_r_completes_on_exactly_six_native_ticks_and_explain_shows_a_plan() {
    let mut h = harness();
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );

    for _ in 0..5 {
        let _ = h.tick();
    }
    assert!(
        h.text().contains("1 native ticks remaining"),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("25 rows from orders"));
    let _ = h.tick();
    assert!(h.text().contains("25 rows from orders"), "{}", h.text());

    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(1)));
    assert!(
        h.text()
            .contains("Explaining query · 6 native ticks remaining")
    );
    for _ in 0..5 {
        let _ = h.tick();
    }
    assert!(
        h.text()
            .contains("Explaining query · 1 native ticks remaining")
    );
    let _ = h.tick();
    assert!(h.text().contains("Query plan · simulated in 2 ms"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn ctrl_r_does_not_reset_a_run_that_is_already_counting_down() {
    let mut h = harness();
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    let _ = h.tick();
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );

    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    for _ in 0..4 {
        let _ = h.tick();
    }
    assert!(h.text().contains("25 rows from orders"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn ctrl_r_after_switching_tabs_keeps_the_existing_run_owner() {
    let mut h = harness();
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    let _ = h.tick();
    let _ = h.tick();
    click_tab(&mut h, customers_key());
    assert!(h.tab_to(TABS));
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    for _ in 0..4 {
        let _ = h.tick();
    }

    click_tab(&mut h, orders_key());
    assert!(h.text().contains("25 rows from orders"), "{}", h.text());
    click_tab(&mut h, customers_key());
    assert!(!h.text().contains("25 rows from customers"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_workbench_stop_and_clear_cancel_pending_work_and_clear_saved_sql() {
    let mut h = harness();
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(2)));
    assert!(h.text().contains("Run stopped."));
    for _ in 0..7 {
        let _ = h.tick();
    }
    assert!(h.text().contains("Run stopped."));
    assert!(!h.text().contains("25 rows from orders"));

    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(3)));
    assert!(h.text().contains("Editor and saved query cleared."));
    assert!(!h.text().contains("SELECT id, customer"));
    click_tab(&mut h, customers_key());
    assert!(h.text().contains("Recent customer accounts"));
    click_tab(&mut h, orders_key());
    assert!(!h.text().contains("SELECT id, customer"));
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Nothing to run · the SQL editor is empty.")
    );
    assert!(
        !h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_tabs_keep_their_editors_and_runs_follow_the_stable_owner_key() {
    let mut h = harness();
    click_tab(&mut h, customers_key());
    assert!(h.text().contains("Recent customer accounts"));
    click_tab(&mut h, orders_key());
    assert!(h.text().contains("Pending orders, newest first"));

    click_tab(&mut h, customers_key());
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    click_close(&mut h, orders_key());
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    for _ in 0..6 {
        let _ = h.tick();
    }
    assert!(h.text().contains("25 rows from customers"), "{}", h.text());
    assert!(!h.text().contains("orders.sql"));

    let mut closing_owner = harness();
    assert!(closing_owner.tab_to(EDITOR));
    press_ctrl_r(&mut closing_owner);
    click_close(&mut closing_owner, orders_key());
    assert!(
        closing_owner
            .text()
            .contains("Run cancelled · its query tab was closed.")
    );
    for _ in 0..7 {
        let _ = closing_owner.tick();
    }
    assert!(
        closing_owner
            .text()
            .contains("Run cancelled · its query tab was closed.")
    );
    assert!(!closing_owner.text().contains("25 rows from orders"));
    assert!(
        closing_owner.diagnostics().is_empty(),
        "{:?}",
        closing_owner.diagnostics()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn ctrl_r_capture_requires_a_focused_page_control_and_the_base_page_layer() {
    for owner in [TABS, TOOLBAR, EDITOR] {
        let mut h = harness();
        assert!(h.tab_to(owner), "page owner {owner:?} is focusable");
        press_ctrl_r(&mut h);
        assert!(
            h.text()
                .contains("Running query · 6 native ticks remaining"),
            "Ctrl+R remains available from native page owner {owner:?}\n{}",
            h.text()
        );
    }

    let mut results = harness();
    assert!(results.tab_to(EDITOR));
    press_ctrl_r(&mut results);
    results.ticks(6);
    assert!(results.tab_to(RESULTS), "results are a native focus owner");
    press_ctrl_r(&mut results);
    assert!(
        results
            .text()
            .contains("Running query · 6 native ticks remaining")
    );

    let mut navigation = harness();
    assert_eq!(navigation.focus(), Some(NAV));
    press_ctrl_r(&mut navigation);
    assert!(!navigation.text().contains("Running query ·"));

    let mut filter = harness();
    let _ = filter.key(KeyCode::Char('/'));
    let _ = filter.type_str("query");
    let before = filter.text();
    press_ctrl_r(&mut filter);
    assert!(!filter.text().contains("Running query ·"));
    assert!(
        filter.text().contains("query"),
        "navigation filter remains active"
    );
    assert!(before.contains("query"));

    let mut help = harness();
    assert!(help.tab_to(EDITOR));
    help = help.with_auto_draw(false);
    let help_area = help
        .area_of_part(HEADER_HELP, PartRef::of(Part::LABEL))
        .expect("Help header action is registered");
    let _ = help.click(help_area.x.saturating_add(1), help_area.y);
    assert!(help.is_open(HELP));
    press_ctrl_r(&mut help);
    help.draw();
    assert!(!help.text().contains("Running query ·"));
    assert!(help.is_open(HELP), "Ctrl+R stays within the Help layer");
    let _ = help.key(KeyCode::Esc);
    help.draw();
    assert!(
        !help.is_open(HELP),
        "Help closes after the modal command check"
    );

    let mut inspector = Harness::new(App::for_page_name("queryworkbench"), Theme::junie(), 80, 24);
    assert!(inspector.tab_to(EDITOR));
    inspector = inspector.with_auto_draw(false);
    let inspector_area = inspector
        .area_of_part(HEADER_INSPECT, PartRef::of(Part::LABEL))
        .expect("Inspector header action is registered");
    let _ = inspector.click(inspector_area.x.saturating_add(1), inspector_area.y);
    assert!(inspector.is_open(INSPECTOR));
    press_ctrl_r(&mut inspector);
    inspector.draw();
    assert!(!inspector.text().contains("Running query ·"));
    assert!(inspector.is_open(INSPECTOR));
    let _ = inspector.key(KeyCode::Esc);
    inspector.draw();
    assert!(
        !inspector.is_open(INSPECTOR),
        "Inspector closes after the modal command check"
    );

    let mut other_route = Harness::new(App::for_page_name("buttons"), Theme::junie(), 120, 40);
    press_ctrl_r(&mut other_route);
    assert!(!other_route.text().contains("Running query ·"));
    for h in [&navigation, &filter, &help, &inspector, &other_route] {
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
