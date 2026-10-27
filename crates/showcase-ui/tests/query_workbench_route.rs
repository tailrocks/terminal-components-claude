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

fn assert_enter_activates_clear(h: &mut Harness<App>, context: &str) {
    assert_eq!(
        h.focus(),
        Some(TOOLBAR),
        "{context}: toolbar owns native focus"
    );
    let clear = h
        .area_of_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(3)))
        .expect("the native Clear toolbar row is published");
    assert_eq!(
        h.cell(clear.x, clear.y).symbol(),
        "▎",
        "{context}: the visible native toolbar cursor marks Clear"
    );
    assert!(
        h.text().contains("Enter Activate"),
        "{context}: the footer describes Enter activation\n{}",
        h.text()
    );
    assert!(
        !h.text().contains("Enter Run"),
        "{context}: the footer does not claim Enter always runs\n{}",
        h.text()
    );

    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Editor and saved query cleared."),
        "{context}: Enter activates the selected Clear command\n{}",
        h.text()
    );
    assert!(
        !h.text().contains("Running query ·"),
        "{context}: Enter does not start Run"
    );
    assert!(
        !h.text().contains("Explaining query ·"),
        "{context}: Enter does not start Explain"
    );
    assert!(
        !h.text().contains("SELECT id, customer"),
        "{context}: SQL was cleared"
    );
    assert!(
        h.diagnostics().is_empty(),
        "{context}: {:?}",
        h.diagnostics()
    );
}

#[test]
fn toolbar_enter_hint_matches_the_selected_command_after_plan_stop_and_clear() {
    let mut planned = harness();
    let _ = planned.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(1)));
    assert!(
        planned
            .text()
            .contains("Explaining query · 6 native ticks remaining")
    );
    for _ in 0..6 {
        let _ = planned.tick();
    }
    assert!(planned.text().contains("Query plan · simulated in 2 ms"));
    assert_eq!(planned.focus(), Some(TOOLBAR));
    let _ = planned.key(KeyCode::End);
    assert_enter_activates_clear(&mut planned, "after a plan");

    let mut stopped = harness();
    assert!(stopped.tab_to(EDITOR));
    press_ctrl_r(&mut stopped);
    let _ = stopped.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(2)));
    assert!(stopped.text().contains("Run stopped."));
    assert_eq!(stopped.focus(), Some(TOOLBAR));
    let _ = stopped.key(KeyCode::End);
    assert_enter_activates_clear(&mut stopped, "after stopping a run");

    let mut cleared = harness();
    assert!(cleared.tab_to(TOOLBAR));
    let _ = cleared.key(KeyCode::End);
    assert_enter_activates_clear(&mut cleared, "before the first clear");
    assert_enter_activates_clear(&mut cleared, "after the query is cleared");
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
    assert!(
        results.text().contains("25 rows from orders"),
        "the six-tick run completes before testing pointer focus\n{}",
        results.text()
    );
    assert!(
        results.ring().reachable().any(|entry| entry.id == RESULTS),
        "Results is reachable in the native focus ring"
    );
    let results_area = results
        .area_of(RESULTS)
        .expect("the Results area is published");
    let (x, y) = results
        .find("10480")
        .expect("the first orders row is visible in Results");
    assert!(
        x >= results_area.x
            && x < results_area.x.saturating_add(results_area.width)
            && y >= results_area.y
            && y < results_area.y.saturating_add(results_area.height),
        "first orders row at ({x}, {y}) lies inside Results area {results_area:?}"
    );
    let _ = termrock_test_support::harness_typed_input::click_cell(&mut results, x, y);
    assert_eq!(
        results.focus(),
        Some(RESULTS),
        "clicking the visible first results row focuses the native Results owner"
    );
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

#[test]
fn query_workbench_native_editor_exit_keys_preserve_visible_query_line_coordinates() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    let editor_text = h
        .area_of_part(EDITOR, PartRef::of(Part::TEXT))
        .expect("the query editor publishes its text part");
    assert!(
        editor_text.width >= 46,
        "the editor text part exposes the full 46-cell SELECT line: {}",
        editor_text.width
    );
    assert!(h.tab_to(EDITOR));
    assert_eq!(h.focus(), Some(EDITOR), "the editor receives native focus");
    let assert_no_diagnostics = |h: &Harness<App>, phase: &str| {
        let diagnostics = h.diagnostics();
        assert!(diagnostics.is_empty(), "{phase}: {diagnostics:?}");
    };
    assert_no_diagnostics(&h, "editor focus");

    let published_focus_order = h
        .ring()
        .reachable()
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    let toolbar_index = published_focus_order
        .iter()
        .position(|id| *id == TOOLBAR)
        .expect("the toolbar is reachable in the published native focus ring");
    let editor_index = published_focus_order
        .iter()
        .position(|id| *id == EDITOR)
        .expect("the editor is reachable in the published native focus ring");
    assert_eq!(
        published_focus_order.get(toolbar_index + 1),
        Some(&EDITOR),
        "the published Draw focus order places the editor after the toolbar: {published_focus_order:?}"
    );
    assert_eq!(
        published_focus_order.get(editor_index + 1),
        Some(&RESULTS),
        "the published Draw focus order places Results after the editor: {published_focus_order:?}"
    );

    let visible_query_lines = [
        "Pending orders, newest first",
        "SELECT id, customer, status, total, created_at",
        "FROM orders",
        "WHERE status = 'pending'",
        "ORDER BY created_at DESC",
        "LIMIT 25;",
    ]
    .map(|line| {
        (
            line,
            h.find(line)
                .unwrap_or_else(|| panic!("rendered query line is visible: {line}")),
        )
    });
    let assert_visible_query_unchanged = |h: &Harness<App>| {
        for &(line, position) in &visible_query_lines {
            assert_eq!(
                h.find(line),
                Some(position),
                "native focus movement preserves this rendered line and its coordinates: {line}"
            );
        }
    };
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::F(6));
    assert_eq!(
        h.focus(),
        Some(RESULTS),
        "F6 leaves editor mode and advances to the next published native owner"
    );
    assert_no_diagnostics(&h, "F6 from Editor to Results");
    assert_visible_query_unchanged(&h);

    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    assert_eq!(h.focus(), Some(RESULTS));
    assert_no_diagnostics(&h, "Ctrl+R start at Results");
    assert_visible_query_unchanged(&h);
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 5 native ticks remaining")
    );
    assert_eq!(h.focus(), Some(RESULTS));
    assert_no_diagnostics(&h, "query tick to 5");
    assert_visible_query_unchanged(&h);
    for remaining in [4, 3, 2, 1] {
        let _ = h.tick();
        assert!(
            h.text().contains(&format!(
                "Running query · {remaining} native ticks remaining"
            )),
            "{}",
            h.text()
        );
        assert_eq!(h.focus(), Some(RESULTS));
        assert_no_diagnostics(&h, "query countdown tick");
        assert_visible_query_unchanged(&h);
    }
    let _ = h.tick();
    assert!(h.text().contains("25 rows from orders"), "{}", h.text());
    assert!(!h.text().contains("Running query ·"));
    assert_eq!(h.focus(), Some(RESULTS));
    assert_no_diagnostics(&h, "query completion tick");
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::BackTab);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native BackTab from Results moves to the previous published owner"
    );
    assert_no_diagnostics(&h, "BackTab from Results to Editor");
    assert_visible_query_unchanged(&h);

    let _ = h.key_mod(KeyCode::F(6), KeyModifiers::SHIFT);
    assert_eq!(
        h.focus(),
        Some(TOOLBAR),
        "Shift+F6 from the focused editor uses its previous-owner binding"
    );
    assert_no_diagnostics(&h, "Shift+F6 from Editor to Toolbar");
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native Tab from Toolbar returns to Editor"
    );
    assert_no_diagnostics(&h, "Tab from Toolbar to Editor");
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::Esc);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "Escape commits editor input while retaining native focus"
    );
    assert_no_diagnostics(&h, "Escape commit in Editor");
    assert_visible_query_unchanged(&h);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(RESULTS),
        "native Tab after Escape moves from Editor to Results"
    );
    assert_no_diagnostics(&h, "Tab from committed Editor to Results");
    assert_visible_query_unchanged(&h);
}
