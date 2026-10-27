use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Rect, Theme};
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
const OPTIONS: Id = Id::root("showcase_ui::pages::query_workbench.options");
const OPTIONS_FORM: Id = OPTIONS.part(Part::BODY);
const ALLOW_NEW: Id = OPTIONS_FORM.part(Part::FIELD).index(0);
const CLOSE_REVEAL: Id = OPTIONS_FORM.part(Part::FIELD).index(1);
const READ_ONLY: Id = OPTIONS_FORM.part(Part::FIELD).index(2);
const RESULT_FIXTURE: Id = OPTIONS_FORM.part(Part::FIELD).index(3);
const ENVIRONMENT: Id = OPTIONS_FORM.part(Part::FIELD).index(4);
const CLOSE_OPTIONS: Id = OPTIONS_FORM.part(Part::ACTIONS).index(0);

const RESULTS_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.results_panel");
const ORDERS_SAMPLE_SUMMARY: &str = "25 rows from orders · simulated in 15 ms";

fn rect_is_inside(area: Rect, parent: Rect) -> bool {
    area.x >= parent.x
        && area.y >= parent.y
        && area.x.saturating_add(area.width) <= parent.x.saturating_add(parent.width)
        && area.y.saturating_add(area.height) <= parent.y.saturating_add(parent.height)
}

fn query_workbench_summary_area(h: &Harness<App>) -> Rect {
    let table = h
        .area_of(RESULTS)
        .expect("the published Results table owns its native area");
    let panel = h
        .area_of(RESULTS_PANEL)
        .expect("the published Results panel owns its native area");
    assert!(table.y > panel.y, "Results table is below the panel title");
    let summary = Rect::new(table.x, table.y - 1, table.width, 1);
    assert!(
        rect_is_inside(summary, panel),
        "summary area {summary:?} is inside Results panel {panel:?}"
    );
    assert_eq!(
        summary.x, table.x,
        "summary and table share the published x"
    );
    assert_eq!(
        summary.width, table.width,
        "summary and table share the published width"
    );
    assert_eq!(summary.y.saturating_add(1), table.y);
    summary
}

fn prefix_with_cell_width(text: &str, max_width: u16) -> String {
    let mut prefix = String::new();
    let mut used_width = 0u16;
    for (_, grapheme) in termrock::text::measure::graphemes(text) {
        let grapheme_width = termrock::text::measure::grapheme_width(grapheme);
        if used_width.saturating_add(grapheme_width) > max_width {
            break;
        }
        prefix.push_str(grapheme);
        used_width = used_width.saturating_add(grapheme_width);
    }
    prefix
}

fn buffer_line_prefix(h: &Harness<App>, area: Rect, max_width: u16) -> String {
    let mut visible = String::new();
    let mut used_width = 0u16;
    for x in area.x..area.x.saturating_add(area.width) {
        if used_width >= max_width {
            break;
        }
        let symbol = h.cell(x, area.y).symbol();
        if symbol.is_empty() {
            continue;
        }
        let symbol_width = termrock::text::width(symbol);
        if symbol_width == 0 {
            continue;
        }
        assert!(
            used_width.saturating_add(symbol_width) <= max_width,
            "Buffer cell at ({x}, {}) exceeds the measured prefix width {max_width}",
            area.y
        );
        visible.push_str(symbol);
        used_width = used_width.saturating_add(symbol_width);
    }
    assert_eq!(
        used_width, max_width,
        "Buffer rectangle {area:?} contains the requested {max_width}-cell prefix"
    );
    visible
}

fn find_ascii_in_rect(h: &Harness<App>, area: Rect, needle: &str) -> Option<(u16, u16)> {
    let needle_width = u16::try_from(needle.len()).ok()?;
    if needle.is_empty() || area.width < needle_width || area.height == 0 {
        return None;
    }
    let last_x = area.x.saturating_add(area.width - needle_width);
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..=last_x {
            let matches = needle.chars().enumerate().all(|(offset, expected)| {
                h.cell(
                    x.saturating_add(u16::try_from(offset).expect("needle fits u16")),
                    y,
                )
                .symbol()
                    == expected.to_string()
            });
            if matches {
                return Some((x, y));
            }
        }
    }
    None
}

fn assert_orders_summary_visible(h: &Harness<App>, context: &str) {
    let summary = query_workbench_summary_area(h);
    let expected_width = termrock::text::width(ORDERS_SAMPLE_SUMMARY).min(summary.width);
    let expected = prefix_with_cell_width(ORDERS_SAMPLE_SUMMARY, expected_width);
    let actual = buffer_line_prefix(h, summary, expected_width);
    assert_eq!(
        actual, expected,
        "{context}: Results summary shows the exact visible prefix of {ORDERS_SAMPLE_SUMMARY:?} in {summary:?}"
    );

    let table = h
        .area_of(RESULTS)
        .expect("Results table area remains published");
    assert!(
        table.height > 1,
        "Results table publishes its header and data rows"
    );
    let data = Rect::new(
        table.x,
        table.y.saturating_add(1),
        table.width,
        table.height.saturating_sub(1),
    );
    assert!(
        rect_is_inside(data, table),
        "table data area {data:?} is inside {table:?}"
    );
    let first_order_row = find_ascii_in_rect(h, data, "10480");
    assert!(
        first_order_row.is_some(),
        "{context}: complete first-row ID 10480 is visible inside table data area {data:?}"
    );
}

fn visible_line_text(h: &Harness<App>, area: Rect) -> String {
    let mut visible = String::new();
    for x in area.x..area.x.saturating_add(area.width) {
        visible.push_str(h.cell(x, area.y).symbol());
    }
    visible.trim_end().to_owned()
}

fn assert_plan_fixture_visible(h: &Harness<App>, context: &str) {
    let summary = query_workbench_summary_area(h);
    assert_eq!(
        visible_line_text(h, summary),
        "Query plan · simulated in 2 ms",
        "{context}: the Results summary identifies the Plan fixture in its published bounds"
    );

    let table = h
        .area_of(RESULTS)
        .expect("the Results table publishes its native bounds");
    assert!(
        find_ascii_in_rect(h, table, "Using index").is_some(),
        "{context}: the exact visible Plan cell appears inside Results table {table:?}"
    );
}

fn assert_orders_summary_fully_visible(h: &Harness<App>, context: &str) {
    let summary = query_workbench_summary_area(h);
    let full_width = termrock::text::width(ORDERS_SAMPLE_SUMMARY);
    assert!(
        summary.width >= full_width,
        "{context}: {summary:?} is wide enough for the full {full_width}-cell summary"
    );
    assert_orders_summary_visible(h, context);
}

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

fn click_new_tab(h: &mut Harness<App>) {
    let part = PartRef::of(Part::NEW);
    assert!(
        h.area_of_part(TABS, part).is_some(),
        "native New action is published"
    );
    let _ = h.click_part(TABS, part);
}

fn press_ctrl_r(h: &mut Harness<App>) {
    let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
}

fn click_toolbar_options(h: &mut Harness<App>) {
    let options = PartRef::item(Part::ROW, ItemKey::index(4));
    assert!(h.area_of_part(TOOLBAR, options).is_some());
    let _ = h.click_part(TOOLBAR, options);
    assert!(h.is_open(OPTIONS));
}

fn choose_select_index(h: &mut Harness<App>, id: Id, down: usize) {
    assert!(
        h.tab_to(id),
        "select {id:?} is reachable in the Options dialog"
    );
    let _ = h.key(KeyCode::Enter);
    for _ in 0..down {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
}

fn choose_select_last(h: &mut Harness<App>, id: Id) {
    assert!(
        h.tab_to(id),
        "select {id:?} is reachable in the Options dialog"
    );
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
}

fn assert_environment_status(h: &Harness<App>, label: &str, expected_fg: termrock::Color) {
    assert!(
        h.text().contains(label),
        "the selected label is fully visible"
    );
    let (x, y) = h.find(label).expect("selected environment is visible");
    let toolbar = h.area_of(TOOLBAR).expect("toolbar publishes its bounds");
    assert_eq!(
        y,
        toolbar.y.saturating_sub(3),
        "environment status occupies the page header row"
    );
    let header_left = toolbar
        .x
        .saturating_add(termrock::width("Query workbench"))
        .saturating_add(2);
    let header_right = toolbar.x.saturating_add(toolbar.width);
    assert!(
        x >= header_left && x.saturating_add(termrock::width(label)) <= header_right,
        "the complete environment label fits inside the header status bounds"
    );
    assert_eq!(
        h.cell(x, y).fg,
        expected_fg,
        "{label} has semantic status ink"
    );
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
    assert_orders_summary_visible(&h, "initial Orders sample");
    assert!(!h.text().contains("Running query ·"));
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
    assert_orders_summary_visible(&h, "Run after five native ticks");
    let _ = h.tick();
    assert!(h.text().contains("25 rows from orders"), "{}", h.text());
    assert!(!h.text().contains("Running query ·"));

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
    assert!(!h.text().contains("Running query ·"));
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
    assert_orders_summary_visible(&h, "stopped Run preserves the sample");

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
fn toolbar_enter_hint_matches_the_selected_command_after_plan_stop_and_clear() {
    let mut h = harness();
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
    assert_orders_summary_visible(&h, "initial sample while Explain counts down");
    let _ = h.tick();
    assert!(h.text().contains("Query plan · simulated in 2 ms"));
    assert!(!h.text().contains("Explaining query ·"));
    assert_eq!(h.focus(), Some(TOOLBAR));
    assert!(h.text().contains("Enter Activate"));
    assert!(!h.text().contains("Enter Run"));

    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(0)));
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    assert!(h.text().contains("Query plan · simulated in 2 ms"));
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 5 native ticks remaining")
    );
    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(2)));
    assert!(h.text().contains("Run stopped."));
    assert!(!h.text().contains("Running query ·"));
    assert!(h.text().contains("Query plan · simulated in 2 ms"));
    for _ in 0..7 {
        let _ = h.tick();
    }
    assert!(h.text().contains("Run stopped."));
    assert!(!h.text().contains("Running query ·"));
    assert!(
        h.text().contains("Query plan · simulated in 2 ms"),
        "stopping Run preserves the completed plan\n{}",
        h.text()
    );
    assert_eq!(h.focus(), Some(TOOLBAR));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let _ = h.key(KeyCode::End);
    let options = h
        .area_of_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(4)))
        .expect("Options is the last native toolbar row");
    assert_eq!(
        h.cell(options.x, options.y).symbol(),
        "▎",
        "End follows native toolbar order and focuses Options"
    );
    let _ = h.key(KeyCode::Left);
    assert_enter_activates_clear(&mut h, "after End then Left from Options");
    assert!(
        h.text().contains("Query plan · simulated in 2 ms"),
        "native Clear preserves the last completed plan\n{}",
        h.text()
    );
    assert!(h.text().contains("Enter Activate"));
}

#[test]
fn new_query_tabs_keep_unique_keys_and_isolated_unrun_state() {
    let mut h = harness();
    let first_new_key = ItemKey::num(2);
    let second_new_key = ItemKey::num(3);

    click_new_tab(&mut h);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, first_new_key))
            .is_some()
    );
    assert!(h.text().contains("Query 2"));
    assert!(h.text().contains("Not run yet · choose Run."));
    assert!(!h.text().contains("Recent customer accounts"));
    assert!(!h.text().contains("Pending orders, newest first"));

    click_tab(&mut h, orders_key());
    assert!(h.text().contains("Pending orders, newest first"));
    assert_orders_summary_visible(&h, "Orders sample after switching from a new tab");
    click_tab(&mut h, first_new_key);
    assert!(h.text().contains("Not run yet · choose Run."));
    assert!(!h.text().contains("Pending orders, newest first"));

    click_close(&mut h, first_new_key);
    click_new_tab(&mut h);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, second_new_key))
            .is_some()
    );
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, first_new_key))
            .is_none()
    );
    assert!(h.text().contains("Query 3"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn closing_the_last_query_keeps_tabs_focusable_and_new_recovers() {
    let mut h = harness();
    click_close(&mut h, orders_key());
    click_close(&mut h, customers_key());

    assert!(h.text().contains("No query is open."));
    assert!(h.area_of(EDITOR).is_none());
    assert!(h.area_of_part(TABS, PartRef::of(Part::NEW)).is_some());
    assert!(h.tab_to(TABS), "empty native Tabs remains focusable");

    click_new_tab(&mut h);
    assert!(h.text().contains("Query 2"));
    assert!(h.text().contains("Not run yet · choose Run."));
    assert!(h.area_of(EDITOR).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_tabs_keep_their_editors_and_runs_follow_the_stable_owner_key() {
    let mut h = harness();
    click_tab(&mut h, customers_key());
    assert!(h.text().contains("Recent customer accounts"));
    assert!(h.text().contains("Not run yet · choose Run."));
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
    assert!(
        results
            .text()
            .contains("Running query · 6 native ticks remaining")
    );
    results.ticks(5);
    assert!(
        results
            .text()
            .contains("Running query · 1 native ticks remaining")
    );
    let _ = results.tick();
    assert!(results.text().contains("25 rows from orders"));
    assert!(!results.text().contains("Running query ·"));
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
    assert_orders_summary_fully_visible(&h, "route-local 160x40 full-render proof");
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

#[test]
fn options_dialog_has_the_historical_defaults_and_restores_native_focus() {
    let mut h = harness();
    click_toolbar_options(&mut h);
    assert_eq!(h.focus(), Some(ALLOW_NEW));
    assert!(h.text().contains("Allow new query tabs"));
    assert!(h.text().contains("Read-only SQL input"));
    assert!(h.text().contains("always"));
    assert!(h.text().contains("run"));
    assert!(h.text().contains("production"));
    assert!(h.text().contains("Sample environments"));

    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(OPTIONS), "Enter never submits or closes the form");
    let _ = h.click_id(CLOSE_OPTIONS);
    assert!(!h.is_open(OPTIONS));
    assert_eq!(h.focus(), Some(TOOLBAR), "Close restores toolbar focus");

    let _ = h.key(KeyCode::Enter);
    assert!(
        h.is_open(OPTIONS),
        "keyboard Enter reopens the selected Options item"
    );
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(OPTIONS));
    assert_eq!(h.focus(), Some(TOOLBAR), "Escape restores toolbar focus");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn escape_closes_the_native_select_before_options_and_restores_focus() {
    let mut h = harness();
    click_toolbar_options(&mut h);
    assert!(h.tab_to(CLOSE_REVEAL));
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(CLOSE_REVEAL), "Enter opens the native Select");
    assert!(h.is_open(OPTIONS));

    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(CLOSE_REVEAL), "first Escape closes only Select");
    assert!(
        h.is_open(OPTIONS),
        "Options remains open after Select dismisses"
    );
    assert_eq!(h.focus(), Some(CLOSE_REVEAL), "focus returns to Select");

    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(OPTIONS), "second Escape closes Options");
    assert_eq!(h.focus(), Some(TOOLBAR), "focus restores to the toolbar");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_dialog_changes_each_functional_field_and_closes_without_rollback() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    click_toolbar_options(&mut h);

    assert!(h.tab_to(ALLOW_NEW));
    let _ = h.key(KeyCode::Char(' '));
    choose_select_index(&mut h, CLOSE_REVEAL, 1);
    assert!(h.text().contains("hover"));

    assert!(h.tab_to(READ_ONLY));
    let _ = h.key(KeyCode::Char(' '));
    choose_select_last(&mut h, RESULT_FIXTURE);
    assert!(h.text().contains("plan"));
    choose_select_index(&mut h, ENVIRONMENT, 2);
    assert!(h.text().contains("development"));

    let _ = h.click_id(CLOSE_OPTIONS);
    assert!(!h.is_open(OPTIONS));
    assert_plan_fixture_visible(&h, "Options-selected Plan fixture");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    click_toolbar_options(&mut h);
    assert!(h.text().contains("hover"));
    assert!(h.text().contains("development"));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(OPTIONS));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn result_fixture_choices_apply_once_and_run_selection_preserves_the_current_result() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    click_toolbar_options(&mut h);
    choose_select_index(&mut h, RESULT_FIXTURE, 1);
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("Enter SQL, then choose Run."));

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, RESULT_FIXTURE, 1);
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("relation \"ordrs\" does not exist"));

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, RESULT_FIXTURE, 1);
    let _ = h.key(KeyCode::Esc);
    assert_plan_fixture_visible(&h, "selected Plan fixture");

    click_toolbar_options(&mut h);
    assert!(h.tab_to(RESULT_FIXTURE));
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Esc);
    assert_plan_fixture_visible(&h, "selecting Run preserves the current Plan fixture");
    assert!(!h.text().contains("Running query ·"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_modal_keeps_run_owner_busy_and_ticks_while_fixtures_are_changed() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    let _ = h.tick();
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    click_tab(&mut h, customers_key());

    click_toolbar_options(&mut h);
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    choose_select_last(&mut h, RESULT_FIXTURE);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 3 native ticks remaining")
    );
    for _ in 0..3 {
        let _ = h.tick();
    }
    assert!(
        h.is_open(OPTIONS),
        "the modal stays open through completion"
    );
    assert!(!h.text().contains("Running query ·"));
    let _ = h.key(KeyCode::Esc);

    click_tab(&mut h, orders_key());
    assert!(h.text().contains("25 rows from orders"));
    click_tab(&mut h, customers_key());
    assert_plan_fixture_visible(&h, "non-owner Plan fixture survives the Orders completion");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn a_fixture_on_the_run_owner_loses_to_that_runs_completion() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    click_toolbar_options(&mut h);
    choose_select_last(&mut h, RESULT_FIXTURE);
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    h.ticks(6);
    assert!(h.is_open(OPTIONS));
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("25 rows from orders"));
    let table = h
        .area_of(RESULTS)
        .expect("the Results table publishes its native bounds");
    assert!(
        find_ascii_in_rect(&h, table, "Using index").is_none(),
        "the completed owner result replaced the temporary Plan fixture"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn read_only_blocks_editor_input_but_keeps_run_and_clear_available() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    click_toolbar_options(&mut h);
    assert!(h.tab_to(READ_ONLY));
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);

    let editor_text = h
        .area_of_part(EDITOR, PartRef::of(Part::TEXT))
        .expect("the native editor publishes its text bounds");
    let full_select_width = termrock::width("SELECT id, customer, status, total, created_at");
    assert!(
        editor_text.width >= full_select_width,
        "160x40 editor text width {} exposes the full {full_select_width}-cell SQL line",
        editor_text.width
    );

    assert!(h.tab_to(EDITOR));
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native Editor focus receives typing"
    );
    let _ = h.type_str(" DROP");
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native Editor focus receives paste"
    );
    let _ = h.paste(" UPDATE orders SET status = 'bad'");
    assert!(
        h.text()
            .contains("SELECT id, customer, status, total, created_at")
    );
    assert!(!h.text().contains("DROP"));
    assert!(!h.text().contains("UPDATE orders"));

    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native Editor focus receives Ctrl+R"
    );
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(2)));
    assert!(h.text().contains("Run stopped."));
    let _ = h.click_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(3)));
    assert!(h.text().contains("Editor and saved query cleared."));
    assert!(!h.text().contains("SELECT id, customer"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_remain_reachable_without_documents_and_allow_new_can_be_reenabled() {
    let mut h = harness();
    click_close(&mut h, orders_key());
    click_close(&mut h, customers_key());
    assert!(h.text().contains("No query is open."));

    click_toolbar_options(&mut h);
    assert!(h.tab_to(ALLOW_NEW));
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);
    assert!(h.area_of_part(TABS, PartRef::of(Part::NEW)).is_none());

    click_toolbar_options(&mut h);
    assert!(h.tab_to(ALLOW_NEW));
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);
    click_new_tab(&mut h);
    assert!(h.text().contains("Query 2"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn selected_environment_uses_semantic_status_ink_and_keeps_all_sample_labels() {
    let mut h = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        160,
        40,
    );
    click_toolbar_options(&mut h);
    for label in [
        "Production: sample data only",
        "Staging: sample data only",
        "Development: sample data only",
        "Local: sample data only",
    ] {
        assert!(h.text().contains(label), "the dialog explains {label}");
    }
    let theme = Theme::junie();
    assert!(h.text().contains("Sample labels only"));
    let environments = [
        ("Production: sample data only", theme.color.danger),
        ("Staging: sample data only", theme.color.warning),
        ("Development: sample data only", theme.color.info),
        (
            "Local: sample data only",
            theme.color.fg[termrock::FgStep::Muted.index()],
        ),
    ];
    for (index, (label, color)) in environments.into_iter().enumerate() {
        if index == 0 {
            let _ = h.key(KeyCode::Esc);
        } else {
            click_toolbar_options(&mut h);
            assert!(h.tab_to(ENVIRONMENT));
            let _ = h.key(KeyCode::Enter);
            let _ = h.key(KeyCode::Down);
            let _ = h.key(KeyCode::Enter);
            let _ = h.key(KeyCode::Esc);
        }
        assert_environment_status(&h, label, color);
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_modal_has_no_native_diagnostics_at_supported_viewport_sizes() {
    for (width, height) in [(120, 40), (80, 24), (72, 20)] {
        let mut h = Harness::new(
            App::for_page_name("queryworkbench"),
            Theme::junie(),
            width,
            height,
        );
        click_toolbar_options(&mut h);
        assert!(h.is_open(OPTIONS));
        let _ = h.key(KeyCode::Esc);
        assert!(
            h.diagnostics().is_empty(),
            "{width}x{height}: {:?}",
            h.diagnostics()
        );
    }
}
