use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, ItemKey, KeyCode, KeyModifiers, MouseKind, Part, PartRef, Rect, Theme, layout};
use termrock_test_support::Harness;

const TABS: Id = Id::root("showcase_ui::pages::query_workbench.tabs");
const TOOLBAR: Id = Id::root("showcase_ui::pages::query_workbench.toolbar");
const EDITOR: Id = Id::root("showcase_ui::pages::query_workbench.editor");
const EDITOR_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.editor_panel");
const RESULTS: Id = Id::root("showcase_ui::pages::query_workbench.results");
const SPLIT: Id = Id::root("showcase_ui::pages::query_workbench.split");
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
const EDITOR_SHARE: Id = OPTIONS_FORM.part(Part::FIELD).index(6);
const SHOW_EXPLORER: Id = OPTIONS_FORM.part(Part::FIELD).index(7);
const EXPLORER_SHARE: Id = OPTIONS_FORM.part(Part::FIELD).index(8);
const EXPLORER_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.explorer_panel");
const EXPLORER_SPLIT: Id = Id::root("showcase_ui::pages::query_workbench.explorer_split");
const EXPLORER_FILTER: Id = Id::root("showcase_ui::pages::query_workbench.explorer_filter");
const EXPLORER_TREE: Id = Id::root("showcase_ui::pages::query_workbench.explorer_tree");
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

fn assert_vertical_editor_results(
    h: &Harness<App>,
    editor_share: u8,
    context: &str,
) -> (Rect, Rect, Rect) {
    let split = h
        .area_of(SPLIT)
        .expect("the native Editor and Results split publishes its root area");
    let editor = h
        .area_of(EDITOR_PANEL)
        .expect("the Editor panel publishes its native area");
    let results = h
        .area_of(RESULTS_PANEL)
        .expect("the Results panel publishes its native area");
    let seam = h
        .area_of_part(SPLIT, PartRef::of(Part::SEAM))
        .expect("the vertical split publishes its native seam");

    assert!(
        rect_is_inside(editor, split),
        "{context}: {editor:?} inside {split:?}"
    );
    assert!(
        rect_is_inside(results, split),
        "{context}: {results:?} inside {split:?}"
    );
    assert_eq!(
        editor.x, split.x,
        "{context}: Editor spans the full split width"
    );
    assert_eq!(
        results.x, split.x,
        "{context}: Results spans the full split width"
    );
    assert_eq!(
        editor.width, split.width,
        "{context}: Editor has full split width"
    );
    assert_eq!(
        results.width, split.width,
        "{context}: Results has full split width"
    );
    assert_eq!(seam.x, split.x, "{context}: seam spans the split width");
    assert_eq!(
        seam.width, split.width,
        "{context}: seam spans the split width"
    );
    assert_eq!(seam.height, 1, "{context}: native gap is one row");
    assert_eq!(
        editor.y.saturating_add(editor.height),
        seam.y,
        "{context}: Editor ends at the native seam"
    );
    assert_eq!(
        seam.y.saturating_add(seam.height),
        results.y,
        "{context}: Results starts below the native seam"
    );
    assert!(
        editor.height >= 5,
        "{context}: Editor meets its five-row minimum"
    );
    assert!(
        results.height >= 6,
        "{context}: Results meets its six-row minimum"
    );

    let available = split.height.saturating_sub(seam.height);
    let actual_scaled = u32::from(editor.height) * 100;
    let expected_scaled = u32::from(available) * u32::from(editor_share);
    assert!(
        actual_scaled.abs_diff(expected_scaled) <= 100,
        "{context}: Editor height {} follows the unclamped {editor_share}% preset over {available} usable rows",
        editor.height
    );
    (split, editor, results)
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
    if h.area_of_part(TOOLBAR, options).is_some() {
        let _ = h.click_part(TOOLBAR, options);
    } else {
        assert!(
            h.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
                .is_some(),
            "native Toolbar overflow exposes the hidden Options command"
        );
        assert!(h.tab_to(TOOLBAR));
        let _ = h.key(KeyCode::End);
        let _ = h.key(KeyCode::Enter);
        let menu = TOOLBAR.sub("more-menu");
        assert!(h.is_open(menu), "native More menu opens");
        let _ = h.key(KeyCode::End);
        let _ = h.key(KeyCode::Enter);
    }
    assert!(h.is_open(OPTIONS));
}

fn choose_select_index(h: &mut Harness<App>, id: Id, down: usize) {
    assert!(
        h.tab_to(id),
        "select {id:?} is reachable in the Options dialog"
    );
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Home);
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
    let tabs = h.area_of(TABS).expect("native Tabs publish their area");
    let toolbar = h
        .area_of(TOOLBAR)
        .expect("native Toolbar publishes its area");
    assert_eq!(tabs.height, 1, "Tabs keep their native header row");
    assert_eq!(toolbar.height, 1, "Toolbar keeps its native row");
    assert_eq!(
        toolbar.y,
        tabs.y.saturating_add(tabs.height),
        "Toolbar follows Tabs in the page body"
    );
    let header_bounds = if let Some(outer) = h.area_of(EXPLORER_SPLIT) {
        assert_eq!(outer.y, tabs.y, "visible work area starts at Tabs");
        outer
    } else {
        assert!(
            h.area_of_part(EXPLORER_SPLIT, PartRef::of(Part::SEAM))
                .is_none(),
            "hidden Explorer has no interactive seam"
        );
        tabs
    };
    let header_y = tabs
        .y
        .checked_sub(2)
        .expect("the page-frame header is two rows above body Tabs");
    let header_area = Rect::new(header_bounds.x, header_y, header_bounds.width, 1);
    let status_start = header_area
        .x
        .saturating_add(termrock::width("Query workbench"))
        .saturating_add(2);
    assert!(
        x >= status_start,
        "environment status starts after the Query workbench title prefix"
    );
    assert_eq!(
        y, header_area.y,
        "environment label occupies its native header status row"
    );
    assert!(
        rect_is_inside(Rect::new(x, y, termrock::width(label), 1), header_area),
        "the complete environment label fits inside page header bounds {header_area:?}"
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
    assert_vertical_editor_results(&h, 40, "initial 120x40 layout");
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
    for owner in [TABS, TOOLBAR, SPLIT, EDITOR] {
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
    const PAGE_OWNER_ORDER: [Id; 8] = [
        EXPLORER_SPLIT,
        EXPLORER_FILTER,
        EXPLORER_TREE,
        TABS,
        TOOLBAR,
        SPLIT,
        EDITOR,
        RESULTS,
    ];
    let page_owner_order = published_focus_order
        .iter()
        .copied()
        .filter(|owner| PAGE_OWNER_ORDER.contains(owner))
        .collect::<Vec<_>>();
    assert_eq!(
        page_owner_order, PAGE_OWNER_ORDER,
        "native Query Workbench owners keep Explorer-first, then complete-main order"
    );
    let navigation_index = published_focus_order
        .iter()
        .position(|owner| *owner == NAV)
        .expect("the showcase App keeps NAV registered in the whole native ring");
    let explorer_index = published_focus_order
        .iter()
        .position(|owner| *owner == EXPLORER_SPLIT)
        .expect("the page Explorer split is in the whole native ring");
    assert!(
        navigation_index < explorer_index,
        "the full ring retains App navigation before page owners: {published_focus_order:?}"
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
        Some(SPLIT),
        "Shift+F6 from Editor reaches the native split owner"
    );
    assert_no_diagnostics(&h, "Shift+F6 from Editor to Split");
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::BackTab);
    assert_eq!(
        h.focus(),
        Some(TOOLBAR),
        "native BackTab from the split reaches the previous published owner"
    );
    assert_no_diagnostics(&h, "BackTab from Split to Toolbar");
    assert_visible_query_unchanged(&h);

    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(SPLIT),
        "native Tab reaches the split from Toolbar"
    );
    assert_no_diagnostics(&h, "Tab from Toolbar to Split");
    assert_visible_query_unchanged(&h);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(EDITOR),
        "native Tab reaches Editor from the split"
    );
    assert_no_diagnostics(&h, "Tab from Split to Editor");
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
fn editor_share_presets_allocate_full_width_vertical_panes() {
    let mut h = harness();
    assert_vertical_editor_results(&h, 40, "historical 40% default");

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, EDITOR_SHARE, 1);
    let _ = h.key(KeyCode::Esc);
    assert_vertical_editor_results(&h, 25, "historical 25% preset");

    click_toolbar_options(&mut h);
    choose_select_last(&mut h, EDITOR_SHARE);
    let _ = h.key(KeyCode::Esc);
    assert_vertical_editor_results(&h, 60, "historical 60% preset");
    assert_orders_summary_visible(&h, "Results remain visible at the 60% editor preset");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn split_keyboard_and_pointer_resize_restore_the_selected_preset_and_keep_child_arrows() {
    let mut h = harness();
    click_toolbar_options(&mut h);
    choose_select_index(&mut h, EDITOR_SHARE, 1);
    let _ = h.key(KeyCode::Esc);
    let (_, initial_editor, _) = assert_vertical_editor_results(&h, 25, "selected 25% preset");

    assert!(h.tab_to(SPLIT), "the native seam is keyboard reachable");
    assert!(h.text().contains("Resize panes"));
    assert!(h.text().contains("Reset editor share"));
    let _ = h.key(KeyCode::Down);
    let (_, keyboard_editor, _) = {
        let split = h.area_of(SPLIT).expect("the split root remains published");
        let editor = h.area_of(EDITOR_PANEL).expect("Editor remains published");
        let results = h.area_of(RESULTS_PANEL).expect("Results remains published");
        (split, editor, results)
    };
    assert!(
        keyboard_editor.height > initial_editor.height,
        "Down grows the first pane while Split owns focus"
    );

    let seam = h
        .area_of_part(SPLIT, PartRef::of(Part::SEAM))
        .expect("the native seam remains published");
    let x = seam.x.saturating_add(seam.width / 2);
    let _ = h.drag((x, seam.y), (x, seam.y.saturating_add(3)));
    let after_drag = h
        .area_of(EDITOR_PANEL)
        .expect("Editor remains published after a pointer resize");
    assert!(
        after_drag.height > keyboard_editor.height,
        "dragging the native seam grows the first pane"
    );

    assert!(h.tab_to(SPLIT));
    let _ = h.key(KeyCode::Home);
    assert_vertical_editor_results(&h, 25, "Home restores the selected 25% preset");

    let editor_height = h
        .area_of(EDITOR_PANEL)
        .expect("Editor remains published")
        .height;
    assert!(h.tab_to(EDITOR));
    let _ = h.key(KeyCode::Down);
    assert_eq!(
        h.area_of(EDITOR_PANEL)
            .expect("Editor remains published")
            .height,
        editor_height,
        "Editor keeps its arrow-key behavior"
    );
    let results_area = h
        .area_of(RESULTS)
        .expect("Results publishes its table area");
    let (id_x, id_y) = find_ascii_in_rect(&h, results_area, "10480")
        .expect("the first visible result cell is inside the published Results geometry");
    assert!(rect_is_inside(Rect::new(id_x, id_y, 5, 1), results_area));
    let _ = termrock_test_support::harness_typed_input::click_cell(&mut h, id_x, id_y);
    assert_eq!(
        h.focus(),
        Some(RESULTS),
        "clicking the visible first Results cell moves focus out of Editor"
    );
    let _ = h.key(KeyCode::Down);
    assert_eq!(
        h.area_of(EDITOR_PANEL)
            .expect("Editor remains published")
            .height,
        editor_height,
        "Results keeps its row-navigation behavior"
    );
    assert_eq!(h.focus(), Some(RESULTS));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn editor_share_presets_change_only_split_geometry_and_unrelated_options_keep_manual_ratio() {
    let mut h = harness();
    click_toolbar_options(&mut h);
    choose_select_last(&mut h, RESULT_FIXTURE);
    let _ = h.key(KeyCode::Esc);
    assert_plan_fixture_visible(&h, "Plan fixture before resizing");

    assert!(h.tab_to(SPLIT));
    let _ = h.key(KeyCode::Down);
    let manual_height = h
        .area_of(EDITOR_PANEL)
        .expect("Editor publishes its resized area")
        .height;
    click_toolbar_options(&mut h);
    choose_select_index(&mut h, ENVIRONMENT, 1);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(
        h.area_of(EDITOR_PANEL)
            .expect("Editor remains published after Environment change")
            .height,
        manual_height,
        "an unrelated setting keeps the manual live split ratio"
    );
    assert_environment_status(
        &h,
        "Staging: sample data only",
        Theme::junie().color.warning,
    );
    assert_plan_fixture_visible(&h, "Environment change preserves the active Plan result");
    assert!(h.text().contains("orders.sql"));
    assert!(h.text().contains("Pending orders, newest first"));

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, EDITOR_SHARE, 1);
    let _ = h.key(KeyCode::Esc);
    assert_vertical_editor_results(&h, 25, "Editor % preset updates the native split state");
    assert_plan_fixture_visible(&h, "Editor % change preserves the active Plan result");
    assert!(h.text().contains("orders.sql"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn editor_share_and_close_stay_reachable_with_the_semantic_note_at_supported_sizes() {
    let semantic_note_lines = [
        "Sample environments",
        "Production: sample data only",
        "Staging: sample data only",
        "Development: sample data only",
        "Local: sample data only",
    ];
    for (width, height) in [(120, 40), (80, 24), (72, 20)] {
        let mut h = Harness::new(
            App::for_page_name("queryworkbench"),
            Theme::junie(),
            width,
            height,
        );
        click_toolbar_options(&mut h);
        assert!(
            h.tab_to(ENVIRONMENT),
            "{width}x{height}: Environment is reachable through native form scrolling"
        );
        let environment_view = h.text();
        assert!(
            h.tab_to(EDITOR_SHARE),
            "{width}x{height}: Editor % is reachable through native form scrolling"
        );
        assert!(h.text().contains("Editor %"), "{width}x{height}");
        let editor_share_view = h.text();
        assert!(
            h.tab_to(CLOSE_OPTIONS),
            "{width}x{height}: Close is reachable through native form scrolling"
        );
        assert_eq!(h.focus(), Some(CLOSE_OPTIONS), "{width}x{height}");
        let close_view = h.text();
        for line in semantic_note_lines {
            assert!(
                environment_view.contains(line)
                    || editor_share_view.contains(line)
                    || close_view.contains(line),
                "{width}x{height}: a focused, native-scrolled form view reveals semantic note line {line:?}"
            );
        }

        assert!(h.tab_to(EDITOR_SHARE));
        let _ = h.key(KeyCode::Enter);
        assert!(h.is_open(EDITOR_SHARE), "{width}x{height}: Select opens");
        let _ = h.key(KeyCode::Esc);
        assert!(
            h.is_open(OPTIONS),
            "{width}x{height}: first Escape closes Select only"
        );
        assert_eq!(h.focus(), Some(EDITOR_SHARE), "{width}x{height}");
        let _ = h.key(KeyCode::Esc);
        assert!(
            !h.is_open(OPTIONS),
            "{width}x{height}: second Escape closes Options"
        );
        assert_eq!(
            h.focus(),
            Some(TOOLBAR),
            "{width}x{height}: modal focus restores"
        );
        assert!(
            h.diagnostics().is_empty(),
            "{width}x{height}: {:?}",
            h.diagnostics()
        );
    }
}

#[test]
fn compact_split_releases_an_active_drag_and_restores_the_live_ratio() {
    let mut h = harness();
    assert!(h.tab_to(SPLIT));
    let _ = h.key(KeyCode::Down);
    let seam = h
        .area_of_part(SPLIT, PartRef::of(Part::SEAM))
        .expect("the 120x40 seam is published");
    let x = seam.x.saturating_add(seam.width / 2);
    let _ = h.mouse(MouseKind::Down, x, seam.y);
    assert_eq!(h.runtime().capture_owner(), Some(SPLIT));
    let _ = h.mouse(MouseKind::Drag, x, seam.y.saturating_add(2));
    assert_eq!(h.runtime().capture_owner(), Some(SPLIT));
    let live_height = h
        .area_of(EDITOR_PANEL)
        .expect("Editor publishes its active-drag area")
        .height;

    let _ = h.resize(72, 20);
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "fallback releases split capture"
    );
    assert!(h.text().contains("orders.sql"));
    assert!(h.text().contains("Results"));
    assert_orders_summary_visible(&h, "compact 72x20 fallback");
    assert!(
        h.area_of_part(SPLIT, PartRef::of(Part::SEAM)).is_none(),
        "fallback publishes no interactive seam"
    );
    assert!(
        !h.ring().reachable().any(|entry| entry.id == SPLIT),
        "fallback removes Split from the focus ring"
    );
    let compact_split = h
        .area_of(SPLIT)
        .expect("the fallback publishes its split root");
    let compact_editor = h
        .area_of(EDITOR_PANEL)
        .expect("fallback keeps the Editor panel");
    let compact_results = h
        .area_of(RESULTS_PANEL)
        .expect("fallback keeps the Results panel");
    assert!(rect_is_inside(compact_editor, compact_split));
    assert!(rect_is_inside(compact_results, compact_split));
    assert_eq!(compact_editor.y, compact_results.y);
    assert_eq!(compact_editor.height, compact_split.height);
    assert_eq!(compact_results.height, compact_split.height);
    assert_eq!(compact_editor.x, compact_split.x);
    assert_eq!(
        compact_editor
            .x
            .saturating_add(compact_editor.width)
            .saturating_add(1),
        compact_results.x,
        "the native 3:2 fallback keeps one column between panels"
    );
    assert_eq!(
        compact_editor
            .width
            .saturating_add(1)
            .saturating_add(compact_results.width),
        compact_split.width
    );
    assert!(compact_editor.width > 0 && compact_results.width > 0);
    let expected_fallback = layout::columns(
        compact_split,
        &[layout::Track::Flex(3), layout::Track::Flex(2)],
        1,
    );
    assert_eq!(
        expected_fallback,
        vec![compact_editor, compact_results],
        "the compact native fallback matches the expected 3:2 flex allocation"
    );

    let stale_x = x.min(
        compact_split
            .x
            .saturating_add(compact_split.width.saturating_sub(1)),
    );
    let stale_y = compact_split.y;
    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    assert_eq!(h.runtime().capture_owner(), None);

    let _ = h.resize(120, 40);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(
        h.area_of_part(SPLIT, PartRef::of(Part::SEAM)).is_some(),
        "the primary vertical seam returns at 120x40"
    );
    assert_eq!(
        h.area_of(EDITOR_PANEL).expect("Editor is restored").height,
        live_height,
        "the native split restores the precise live ratio after collapse"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn preset_modal_and_resizes_keep_the_six_tick_run_with_its_original_document_owner() {
    let mut h = harness();
    assert!(h.tab_to(EDITOR));
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 6 native ticks remaining")
    );
    h.ticks(2);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    click_tab(&mut h, customers_key());

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, EDITOR_SHARE, 1);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    press_ctrl_r(&mut h);
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );

    let _ = h.resize(72, 20);
    assert!(h.is_open(OPTIONS));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(OPTIONS), "Escape closes Options at compact size");
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining"),
        "closing Options leaves the original four-tick run active"
    );
    click_toolbar_options(&mut h);
    assert!(h.is_open(OPTIONS), "Options reopens at compact size");
    let _ = h.resize(120, 40);
    assert!(
        h.is_open(OPTIONS),
        "Options remains open after restoring size"
    );
    assert!(
        h.text()
            .contains("Running query · 4 native ticks remaining")
    );
    let _ = h.tick();
    assert!(
        h.text()
            .contains("Running query · 3 native ticks remaining")
    );
    h.ticks(2);
    assert!(
        h.text()
            .contains("Running query · 1 native ticks remaining")
    );
    let _ = h.tick();
    assert!(
        h.is_open(OPTIONS),
        "the Options modal stays open through completion"
    );
    assert!(!h.text().contains("Running query ·"));

    let _ = h.key(KeyCode::Esc);
    click_tab(&mut h, orders_key());
    assert!(h.text().contains("25 rows from orders"));
    click_tab(&mut h, customers_key());
    assert!(h.text().contains("Not run yet · choose Run."));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
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
    assert!(h.text().contains("Editor %"));
    assert!(h.text().contains("40"));
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
    choose_select_index(&mut h, RESULT_FIXTURE, 2);
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("relation \"ordrs\" does not exist"));

    click_toolbar_options(&mut h);
    choose_select_index(&mut h, RESULT_FIXTURE, 3);
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

fn toggle_show_explorer(h: &mut Harness<App>) {
    click_toolbar_options(h);
    assert!(
        h.tab_to(SHOW_EXPLORER),
        "Show Explorer is reachable in Options"
    );
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);
    let _ = h.tick();
}

fn set_explorer_share(h: &mut Harness<App>, index: usize) {
    click_toolbar_options(h);
    choose_select_index(h, EXPLORER_SHARE, index);
    let _ = h.key(KeyCode::Esc);
    let _ = h.tick();
}

fn assert_explorer_layout(
    h: &Harness<App>,
    explorer_visible: bool,
    context: &str,
    retained_outer: Option<Rect>,
) -> Rect {
    let main = h
        .area_of(SPLIT)
        .expect("the Editor and Results split remains in the second pane");
    let outer = if explorer_visible {
        let published = h
            .area_of(EXPLORER_SPLIT)
            .expect("the visible native horizontal split publishes its root area");
        if let Some(retained) = retained_outer {
            assert_eq!(
                published, retained,
                "{context}: visible split keeps its previously published outer area"
            );
        }
        published
    } else {
        retained_outer.unwrap_or(main)
    };
    let tabs = h.area_of(TABS).expect("Tabs remain in the main pane");
    let toolbar = h
        .area_of(TOOLBAR)
        .expect("Toolbar remains in the main pane");

    for (name, area) in [
        ("Tabs", tabs),
        ("Toolbar", toolbar),
        ("Editor/Results", main),
    ] {
        assert_eq!(
            area.x, main.x,
            "{context}: {name} starts at the full main-pane edge"
        );
        assert_eq!(
            area.width, main.width,
            "{context}: {name} uses the full main-pane width"
        );
    }

    if explorer_visible {
        let explorer = h
            .area_of(EXPLORER_PANEL)
            .expect("Explorer is visible in the first pane");
        let seam = h
            .area_of_part(EXPLORER_SPLIT, PartRef::of(Part::SEAM))
            .expect("the visible horizontal split publishes its native seam");
        assert!(
            rect_is_inside(explorer, outer),
            "{context}: Explorer is inside outer split"
        );
        assert_eq!(explorer.x, outer.x, "{context}: Explorer starts first");
        assert_eq!(
            explorer.y, outer.y,
            "{context}: Explorer fills workbench height"
        );
        assert_eq!(
            explorer.height, outer.height,
            "{context}: Explorer fills workbench height"
        );
        assert!(
            explorer.width >= 22,
            "{context}: Explorer meets its native minimum"
        );
        assert_eq!(
            explorer.x.saturating_add(explorer.width),
            seam.x,
            "{context}: Explorer ends at the native seam"
        );
        assert_eq!(
            seam.x.saturating_add(seam.width),
            main.x,
            "{context}: the complete main pane follows the native seam"
        );
        assert_eq!(
            main.x.saturating_add(main.width),
            outer.x.saturating_add(outer.width),
            "{context}: the complete main pane reaches the outer right edge"
        );
        assert!(
            h.area_of(EXPLORER_FILTER).is_some(),
            "{context}: Filter is published"
        );
        assert!(
            h.area_of(EXPLORER_TREE).is_some(),
            "{context}: Tree is published"
        );
    } else {
        assert_eq!(
            tabs.y, outer.y,
            "{context}: Tabs start at the retained outer work area"
        );
        assert_eq!(tabs.height, 1, "{context}: Tabs keep their native row");
        assert_eq!(
            toolbar.y,
            tabs.y.saturating_add(tabs.height),
            "{context}: Toolbar follows Tabs"
        );
        assert_eq!(toolbar.height, 1, "{context}: Toolbar keeps its native row");
        assert_eq!(
            main.y,
            toolbar.y.saturating_add(toolbar.height),
            "{context}: Editor/Results follows Toolbar"
        );
        assert_eq!(
            main.y.saturating_add(main.height),
            outer.y.saturating_add(outer.height),
            "{context}: Editor/Results reaches the retained outer bottom"
        );
        assert!(
            h.area_of(EXPLORER_PANEL).is_none(),
            "{context}: native Split hides Explorer"
        );
        assert!(
            h.area_of(EXPLORER_FILTER).is_none(),
            "{context}: hidden Filter has no area"
        );
        assert!(
            h.area_of(EXPLORER_TREE).is_none(),
            "{context}: hidden Tree has no area"
        );
        assert_eq!(
            main.x, outer.x,
            "{context}: second-only mode starts the main pane at the outer edge"
        );
        assert_eq!(
            main.width, outer.width,
            "{context}: second-only mode gives the complete outer width to the main pane"
        );
        assert!(
            h.area_of_part(EXPLORER_SPLIT, PartRef::of(Part::SEAM))
                .is_none(),
            "{context}: hidden Explorer has no interactive seam"
        );
        for owner in [EXPLORER_SPLIT, EXPLORER_FILTER, EXPLORER_TREE] {
            assert!(
                !h.ring().reachable().any(|entry| entry.id == owner),
                "{context}: hidden Explorer owner {owner:?} is absent from the focus ring"
            );
        }
    }
    outer
}

fn assert_explorer_share(h: &Harness<App>, percent: u8, context: &str) {
    let outer = h
        .area_of(EXPLORER_SPLIT)
        .expect("the Explorer split publishes its root");
    let explorer = h
        .area_of(EXPLORER_PANEL)
        .expect("Explorer is visible for share geometry proof");
    let available = outer.width.saturating_sub(1);
    let actual_scaled = u32::from(explorer.width) * 100;
    let expected_scaled = u32::from(available) * u32::from(percent);
    assert!(
        actual_scaled.abs_diff(expected_scaled) <= 100,
        "{context}: Explorer width {} follows its {percent}% preset over {available} usable columns",
        explorer.width
    );
}

fn assert_main_width(h: &Harness<App>, expected: u16, context: &str) {
    let main = h
        .area_of(SPLIT)
        .expect("the inner split publishes the complete main area");
    let toolbar = h.area_of(TOOLBAR).expect("Toolbar is in the main area");
    assert_eq!(main.width, expected, "{context}: main width");
    assert_eq!(toolbar.width, expected, "{context}: Toolbar width");
}

fn assert_toolbar_commands_reachable(h: &mut Harness<App>, expected_width: u16, context: &str) {
    let toolbar = h
        .area_of(TOOLBAR)
        .expect("Toolbar publishes its native bounds");
    assert_eq!(
        toolbar.width, expected_width,
        "{context}: Toolbar receives the full main-pane width"
    );
    assert!(
        h.tab_to(TOOLBAR),
        "{context}: Toolbar is keyboard reachable"
    );

    let labels = ["Run", "Explain", "Stop", "Clear", "Options"];
    let hidden = labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| {
            h.area_of_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(index)))
                .is_none()
                .then_some(*label)
        })
        .collect::<Vec<_>>();

    if hidden.is_empty() {
        for (index, label) in labels.into_iter().enumerate() {
            let row = h
                .area_of_part(TOOLBAR, PartRef::item(Part::ROW, ItemKey::index(index)))
                .expect("every command is a published native Toolbar row");
            assert!(
                rect_is_inside(row, toolbar),
                "{context}: {label} row is in Toolbar"
            );
        }
    } else {
        let _ = h.key(KeyCode::End);
        assert!(
            h.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
                .is_some(),
            "{context}: hidden commands are offered through native More overflow"
        );
        let _ = h.key(KeyCode::Enter);
        let menu = TOOLBAR.sub("more-menu");
        assert!(
            h.is_open(menu),
            "{context}: More opens the native command menu"
        );
        let contents = h.text();
        for label in hidden {
            assert!(
                contents.contains(label),
                "{context}: hidden {label} remains available in native overflow"
            );
        }
        let _ = h.key(KeyCode::Esc);
        assert!(
            !h.is_open(menu),
            "{context}: native More menu closes cleanly"
        );
    }
    assert!(
        h.diagnostics().is_empty(),
        "{context}: {:?}",
        h.diagnostics()
    );
}

#[test]
fn explorer_split_options_use_native_second_only_at_compact_widths_and_preserve_main_commands() {
    let mut wide = Harness::new(
        App::for_page_name("queryworkbench"),
        Theme::junie(),
        120,
        40,
    );
    let outer = assert_explorer_layout(&wide, true, "fitting workbench", None);
    assert_eq!(
        outer.width, 94,
        "the fitting workbench uses the measured outer width"
    );
    assert_explorer_share(&wide, 32, "32% default");
    assert_main_width(&wide, 64, "32% Explorer share");
    assert_toolbar_commands_reachable(&mut wide, 64, "64-cell main pane");

    for (index, percent, main_width) in [(1, 24, 71), (2, 40, 56), (0, 32, 64)] {
        set_explorer_share(&mut wide, index);
        assert_explorer_layout(&wide, true, "selected fitting Explorer share", Some(outer));
        assert_explorer_share(&wide, percent, "selected Explorer preset");
        assert_main_width(&wide, main_width, "selected Explorer preset");
        assert_toolbar_commands_reachable(&mut wide, main_width, "selected main width");
    }

    let seam = wide
        .area_of_part(EXPLORER_SPLIT, PartRef::of(Part::SEAM))
        .expect("the outer seam is available for native dragging");
    let start = (seam.x, seam.y.saturating_add(seam.height / 2));
    let before_drag = wide
        .area_of(EXPLORER_PANEL)
        .expect("Explorer is visible before dragging")
        .width;
    let _ = wide.drag(start, (start.0.saturating_add(5), start.1));
    let after_drag = wide
        .area_of(EXPLORER_PANEL)
        .expect("Explorer stays visible after native dragging")
        .width;
    assert!(
        after_drag > before_drag,
        "dragging the outer seam grows Explorer"
    );
    toggle_show_explorer(&mut wide);
    assert_explorer_layout(&wide, false, "explicitly hidden Explorer", Some(outer));
    toggle_show_explorer(&mut wide);
    assert_explorer_layout(&wide, true, "Explorer restored after hiding", Some(outer));
    assert_eq!(
        wide.area_of(EXPLORER_PANEL).map(|area| area.width),
        Some(after_drag),
        "hide and show preserve the resized outer ratio"
    );

    for (width, height, expected_outer) in [(80, 24, 59), (72, 20, 51)] {
        let mut h = Harness::new(
            App::for_page_name("queryworkbench"),
            Theme::junie(),
            width,
            height,
        );
        let context = format!("{width}x{height} compact workbench");
        let main = h
            .area_of(SPLIT)
            .expect("the second-only Editor/Results split publishes its native area");
        let tabs = h
            .area_of(TABS)
            .expect("compact Tabs publish their native area");
        let outer = Rect::new(
            main.x,
            tabs.y,
            main.width,
            main.y.saturating_add(main.height).saturating_sub(tabs.y),
        );
        assert_explorer_layout(&h, false, &context, Some(outer));
        assert_eq!(
            outer.width, expected_outer,
            "{context}: native workbench width"
        );
        assert_toolbar_commands_reachable(&mut h, expected_outer, "compact main toolbar");

        toggle_show_explorer(&mut h);
        assert_explorer_layout(&h, false, "explicitly hidden at compact width", Some(outer));
        toggle_show_explorer(&mut h);
        assert_explorer_layout(
            &h,
            false,
            "show request remains native-fallback hidden",
            Some(outer),
        );
        assert!(h.diagnostics().is_empty(), "{width}: {:?}", h.diagnostics());
    }

    let mut minimum = Harness::new(App::for_page_name("queryworkbench"), Theme::junie(), 84, 40);
    let outer = assert_explorer_layout(&minimum, true, "exact outer minimum", None);
    assert_eq!(
        outer.width, 63,
        "the native horizontal split reaches its exact 22+1+40 minimum"
    );
    assert_eq!(
        minimum.area_of(EXPLORER_PANEL).map(|area| area.width),
        Some(22),
        "the exact-minimum first pane preserves its 22-cell minimum"
    );
    assert_main_width(&minimum, 40, "exact 40-cell second pane");
    assert_toolbar_commands_reachable(&mut minimum, 40, "40-cell main pane");
    assert!(wide.diagnostics().is_empty(), "{:?}", wide.diagnostics());
}

#[test]
fn explorer_filter_draft_survives_hide_show_and_ctrl_r_uses_native_explorer_owners() {
    let mut h = harness();
    let outer = assert_explorer_layout(&h, true, "initial Explorer layout", None);
    assert!(h.tab_to(EXPLORER_FILTER));
    let _ = h.paste("orders");
    let customers = PartRef::item(
        Part::ROW,
        ItemKey::text("query.workbench.explorer.table.customers"),
    );
    assert!(h.area_of_part(EXPLORER_TREE, customers).is_none());

    click_toolbar_options(&mut h);
    assert!(h.tab_to(SHOW_EXPLORER));
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);
    let _ = h.tick();
    assert_explorer_layout(
        &h,
        false,
        "hidden during an edited filter draft",
        Some(outer),
    );
    assert_ne!(
        h.focus(),
        Some(EXPLORER_FILTER),
        "hidden filter releases native focus"
    );
    assert_ne!(
        h.focus(),
        Some(EXPLORER_TREE),
        "hidden tree releases native focus"
    );

    toggle_show_explorer(&mut h);
    assert_explorer_layout(
        &h,
        true,
        "shown after retaining the filter draft",
        Some(outer),
    );
    assert!(
        h.area_of_part(EXPLORER_TREE, customers).is_none(),
        "the restored native query still filters Customers"
    );
    assert!(
        h.area_of_part(
            EXPLORER_TREE,
            PartRef::item(
                Part::ROW,
                ItemKey::text("query.workbench.explorer.table.orders"),
            )
        )
        .is_some()
    );

    for owner in [EXPLORER_SPLIT, EXPLORER_FILTER, EXPLORER_TREE] {
        let mut focused = harness();
        assert!(
            focused.tab_to(owner),
            "native Explorer owner {owner:?} is reachable"
        );
        press_ctrl_r(&mut focused);
        assert!(
            focused
                .text()
                .contains("Running query · 6 native ticks remaining"),
            "Ctrl+R is captured from {owner:?}"
        );
        assert!(
            focused.diagnostics().is_empty(),
            "{owner:?}: {:?}",
            focused.diagnostics()
        );
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn explorer_lazy_load_ticks_through_hidden_and_modal_states_and_keeps_inert_rows_inert() {
    let mut h = harness();
    set_explorer_share(&mut h, 2);
    let outer = assert_explorer_layout(&h, true, "initial Explorer layout", None);
    assert_explorer_share(&h, 40, "lazy-load metadata viewport");
    let tree_area = h.area_of(EXPLORER_TREE);
    let analytics = PartRef::item(
        Part::ROW,
        ItemKey::text("query.workbench.explorer.schema.analytics"),
    );
    let analytics_icon = PartRef::item(
        Part::ICON,
        ItemKey::text("query.workbench.explorer.schema.analytics"),
    );
    let row_before = h.area_of_part(EXPLORER_TREE, analytics);
    let icon_before = h.area_of_part(EXPLORER_TREE, analytics_icon);
    let focus_before = h.focus();
    let diagnostics_before = format!("{:?}", h.diagnostics());
    let rendered_before = row_before.map(|area| visible_line_text(&h, area));
    assert!(
        h.diagnostics().is_empty(),
        "before Analytics click: tree={tree_area:?}, ROW={row_before:?}, ICON={icon_before:?}, focus={focus_before:?}, diagnostics={diagnostics_before}, row={rendered_before:?}"
    );

    let click_response = h.click_part(EXPLORER_TREE, analytics);
    let row_after = h.area_of_part(EXPLORER_TREE, analytics);
    let icon_after = h.area_of_part(EXPLORER_TREE, analytics_icon);
    let focus_after = h.focus();
    let diagnostics_after = format!("{:?}", h.diagnostics());
    let rendered_after = row_after.map(|area| visible_line_text(&h, area));
    assert!(
        h.text().contains("Loading…"),
        "expanding Analytics starts a lazy load: tree={tree_area:?}, ROW={row_before:?}->{row_after:?}, ICON={icon_before:?}->{icon_after:?}, focus={focus_before:?}->{focus_after:?}, diagnostics={diagnostics_before}->{diagnostics_after}, row={rendered_before:?}->{rendered_after:?}, changed={} ",
        click_response.is_changed()
    );
    assert!(
        h.diagnostics().is_empty(),
        "immediately after Analytics click: tree={tree_area:?}, ROW={row_after:?}, ICON={icon_after:?}, focus={focus_after:?}, diagnostics={diagnostics_after}, row={rendered_after:?}"
    );

    toggle_show_explorer(&mut h);
    assert_explorer_layout(&h, false, "load at hidden tick one", Some(outer));
    let _ = h.tick();
    assert_explorer_layout(&h, false, "load at hidden tick two", Some(outer));

    click_toolbar_options(&mut h);
    h.ticks(3);
    assert!(h.is_open(OPTIONS), "Options remains open during load ticks");
    assert!(
        !h.text().contains("events"),
        "the lazy row is not visible before its sixth native tick"
    );
    assert!(h.tab_to(SHOW_EXPLORER));
    let _ = h.key(KeyCode::Char(' '));
    let _ = h.key(KeyCode::Esc);
    assert!(
        h.area_of(EXPLORER_TREE).is_some(),
        "Explorer becomes visible after Options closes"
    );

    let events = PartRef::item(
        Part::ROW,
        ItemKey::text("query.workbench.explorer.table.analytics.events"),
    );
    assert!(
        h.area_of_part(EXPLORER_TREE, events).is_none(),
        "showing Explorer does not advance the pending load"
    );
    let _ = h.tick();
    assert_explorer_layout(
        &h,
        true,
        "load completes on its sixth hidden/modal tick",
        Some(outer),
    );
    assert!(
        h.area_of_part(EXPLORER_TREE, events).is_some(),
        "Analytics events appear on the sixth tick"
    );
    let restored_icon = h
        .area_of_part(EXPLORER_TREE, analytics_icon)
        .expect("the loaded schema keeps its native icon cell");
    assert_eq!(
        h.cell(restored_icon.x, restored_icon.y).symbol(),
        "S",
        "the ordinary schema icon returns after the sixth tick"
    );

    let _ = h.click_part(EXPLORER_TREE, events);
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(2)))
            .is_none(),
        "loaded analytics.events remains inert"
    );

    assert!(h.tab_to(EXPLORER_TREE));
    let _ = h.key(KeyCode::Char('*'));
    h.ticks(6);
    for (key, label) in [
        (
            ItemKey::text("query.workbench.explorer.view.active_customers"),
            "active_customers",
        ),
        (
            ItemKey::text("query.workbench.explorer.function.refresh_totals"),
            "refresh_totals()",
        ),
        (
            ItemKey::text("query.workbench.explorer.note.audit_empty"),
            "No objects",
        ),
    ] {
        let row = PartRef::item(Part::ROW, key);
        assert!(h.text().contains(label), "Expand All loads inert {label}");
        assert!(
            h.area_of_part(EXPLORER_TREE, row).is_some(),
            "{label} has a native row"
        );
        let _ = h.click_part(EXPLORER_TREE, row);
        let _ = h.key(KeyCode::Enter);
        assert!(
            h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(2)))
                .is_none(),
            "loaded {label} remains inert"
        );
    }

    let products = PartRef::item(
        Part::ROW,
        ItemKey::text("query.workbench.explorer.table.products"),
    );
    let _ = h.click_part(EXPLORER_TREE, products);
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(2)))
            .is_some()
    );
    assert!(h.text().contains("Opened products from Explorer."));
    assert!(h.text().contains("SELECT * FROM products LIMIT 100;"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
