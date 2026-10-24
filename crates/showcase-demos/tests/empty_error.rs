use showcase_demos::pages::{Page, empty_error::EmptyErrorPage};
use termrock::{
    App, Axis, Cx, GlyphRole, Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Rect, Response,
    Theme, Ui,
};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::empty_error.page_tabs");
const OPTION_KIND: Id = Id::root("showcase_ui::pages::empty_error.option.kind");
const OPTION_HINT: Id = Id::root("showcase_ui::pages::empty_error.option.hint");
const OPTION_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.option.actions");
const OPTION_REASON: Id = Id::root("showcase_ui::pages::empty_error.option.reason");
const SAVED_LIST: Id = Id::root("showcase_ui::pages::empty_error.saved_queries");
const SEARCH_EMPTY: Id = Id::root("showcase_ui::pages::empty_error.search_results");
const CONNECTION_PRIMARY: Id = Id::root("showcase_ui::pages::empty_error.connection.primary");
const CONNECTION_IMPORT: Id = Id::root("showcase_ui::pages::empty_error.connection.import");
const ORDERS_GRID: Id = Id::root("showcase_ui::pages::empty_error.orders.grid");
const ROW_ACTIONS: Id = Id::root("showcase_ui::pages::empty_error.orders.actions");
const ROW_MENU: Id = Id::root("showcase_ui::pages::empty_error.orders.menu");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::empty_error.states.scroll");
const GALLERY_ACTION: Id = Id::root("showcase_ui::pages::empty_error.gallery.action");
const GALLERY_LABEL_WIDTH: u16 = 14;
const GALLERY_HEIGHT: usize = 22;
const GALLERY_ROWS: [&str; 7] = [
    "Rest", "Hover", "Pressed", "Focused", "Error", "Overflow", "Narrow",
];

struct PageHost {
    page: EmptyErrorPage,
    status: Option<String>,
    area: Option<Rect>,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let update = self.page.update(cx);
        if let Some(status) = update.status {
            self.status = Some(status.0);
        }
        update.response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if let Some(area) = self.area {
            let full = ui.full();
            let sentinel = "S".repeat(usize::from(full.width));
            for y in full.y..full.bottom() {
                let _ = ui.paint_str(
                    Rect::new(full.x, y, full.width, 1),
                    &sentinel,
                    ui.surface_style(),
                );
            }
            self.page.draw(ui, area);
        } else {
            self.page.draw(ui, ui.full());
        }
    }
}

fn host(width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: EmptyErrorPage::new(),
            status: None,
            area: None,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn area_host(width: u16, height: u16, area: Rect) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: EmptyErrorPage::new(),
            status: None,
            area: Some(area),
        },
        Theme::junie(),
        width,
        height,
    )
}

fn click_tab(h: &mut Harness<PageHost>, index: usize) {
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn choose(h: &mut Harness<PageHost>, id: Id, current: usize, target: usize) {
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

fn page_debug(h: &Harness<PageHost>) -> String {
    format!("{:?}", h.app().page)
}

fn view_width(scroll: Rect) -> u16 {
    scroll
        .width
        .saturating_sub(u16::from(usize::from(scroll.height) < GALLERY_HEIGHT))
}

fn gallery_column(scroll: Rect, column: usize) -> Rect {
    let width = view_width(scroll);
    let grid_x = scroll.x.saturating_add(GALLERY_LABEL_WIDTH.min(width));
    let grid_width = width.saturating_sub(GALLERY_LABEL_WIDTH.min(width));
    let base = grid_width / 3;
    let remainder = grid_width % 3;
    let mut x = grid_x;
    for current in 0..column {
        x = x
            .saturating_add(base)
            .saturating_add(u16::from((current as u16) < remainder));
    }
    Rect::new(
        x,
        scroll.y,
        base.saturating_add(u16::from((column as u16) < remainder)),
        scroll.height,
    )
}

fn text_in(h: &Harness<PageHost>, area: Rect) -> String {
    let mut text = String::new();
    for y in area.y..area.bottom() {
        if !text.is_empty() {
            text.push('\n');
        }
        for x in area.x..area.right() {
            text.push_str(h.cell(x, y).symbol());
        }
    }
    text.trim().to_owned()
}

fn gallery_cell_text(h: &Harness<PageHost>, scroll: Rect, top: u16, column: usize) -> String {
    let column = gallery_column(scroll, column);
    text_in(
        h,
        Rect::new(
            column.x,
            top,
            column.width,
            3.min(scroll.bottom().saturating_sub(top)),
        ),
    )
}

fn assert_gallery_row(h: &Harness<PageHost>, scroll: Rect, top: u16, row: usize) {
    let columns = [
        gallery_cell_text(h, scroll, top, 0),
        gallery_cell_text(h, scroll, top, 1),
        gallery_cell_text(h, scroll, top, 2),
    ];
    let error_glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);
    match row {
        0 => {
            assert!(columns[0].contains("No rows"), "{columns:?}");
            assert!(columns[1].contains("No rows"), "{columns:?}");
            assert!(columns[1].contains("Run the query"), "{columns:?}");
            assert!(columns[2].contains("No connections"), "{columns:?}");
            assert!(columns[2].contains("New"), "{columns:?}");
        }
        1..=3 => {
            assert!(columns[0].contains("N/A · no action"), "{columns:?}");
            assert!(columns[1].contains("N/A · no action"), "{columns:?}");
            assert!(columns[2].contains("No connections"), "{columns:?}");
            assert!(columns[2].contains("New"), "{columns:?}");
        }
        4 => {
            for (column, text) in columns.iter().enumerate() {
                let expected = if column == 2 {
                    "No connections"
                } else {
                    "No rows"
                };
                assert!(
                    text.contains(error_glyph),
                    "error glyph missing: {columns:?}"
                );
                assert!(text.contains(expected), "{columns:?}");
            }
            assert!(columns[1].contains("Run the query"), "{columns:?}");
            assert!(columns[2].contains("New"), "{columns:?}");
        }
        5 => {
            assert!(columns[0].contains("No rows in a view"), "{columns:?}");
            assert!(columns[1].contains("No rows in a view"), "{columns:?}");
            assert!(columns[1].contains("Run the query"), "{columns:?}");
            assert!(columns[2].contains("No connections in"), "{columns:?}");
            assert!(columns[2].contains("New"), "{columns:?}");
        }
        6 => {
            assert!(columns[0].contains("No rows"), "{columns:?}");
            assert!(columns[1].contains("Run the"), "{columns:?}");
            assert!(columns[2].contains("No"), "{columns:?}");
            assert!(columns[2].contains("New"), "{columns:?}");
        }
        _ => panic!("unknown gallery row {row}"),
    }
}

#[test]
fn playground_keeps_saved_search_and_connection_empty_states_distinct() {
    let mut h = host(100, 40);
    let text = h.text();
    assert!(text.contains("No saved queries"), "{text}");
    assert!(text.contains("Queries you save appear here"), "{text}");
    assert!(text.contains("Nothing matches “ordrs”"), "{text}");
    assert!(text.contains("No connections yet"), "{text}");
    assert!(text.contains("A connection holds a host"), "{text}");
    assert!(h.area_of(CONNECTION_PRIMARY).is_some());
    assert!(h.area_of(CONNECTION_IMPORT).is_some());
    assert!(h.area_of(ORDERS_GRID).is_some());
    assert!(h.area_of(SAVED_LIST).is_some());
    assert!(
        h.tab_to(SAVED_LIST),
        "the empty List retains its real focus target"
    );
    assert!(
        h.area_of(SEARCH_EMPTY).is_none(),
        "search specimen is noninteractive"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn all_four_options_change_live_content_and_action_registration() {
    let mut h = host(100, 40);
    click_tab(&mut h, 1);
    assert!(h.area_of(OPTION_KIND).is_some());
    assert!(h.area_of(OPTION_HINT).is_some());
    assert!(h.area_of(OPTION_ACTIONS).is_some());
    assert!(h.area_of(OPTION_REASON).is_some());

    choose(&mut h, OPTION_KIND, 0, 1);
    choose(&mut h, OPTION_HINT, 1, 0);
    choose(&mut h, OPTION_ACTIONS, 2, 0);
    choose(&mut h, OPTION_REASON, 0, 1);
    let state = page_debug(&h);
    assert!(state.contains("kind: 1"), "{state}");
    assert!(state.contains("hint: 0"), "{state}");
    assert!(state.contains("actions: 0"), "{state}");
    assert!(state.contains("reason: 1"), "{state}");

    click_tab(&mut h, 0);
    let text = h.text();
    assert!(text.contains("Could not load connections"), "{text}");
    assert!(!text.contains("config file is unreadable"), "{text}");
    assert!(h.area_of(CONNECTION_PRIMARY).is_none());
    assert!(h.area_of(CONNECTION_IMPORT).is_none());
    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app().status.as_deref().is_some_and(|status| {
            status.contains("orders_view has no primary key, so rows cannot be told apart")
        }),
        "{:?}",
        h.app().status
    );
    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.text().contains("no primary key"));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROW_MENU));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    click_tab(&mut h, 1);
    choose(&mut h, OPTION_REASON, 1, 2);
    click_tab(&mut h, 0);
    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app().status.as_deref().is_some_and(
            |status| status.contains("A query result is read-only: edit the table instead")
        ),
        "{:?}",
        h.app().status
    );
    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.text().contains("query result"));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROW_MENU));
}

#[test]
fn connection_actions_are_real_activations_with_truthful_local_status() {
    let mut h = host(100, 40);
    assert!(h.tab_to(CONNECTION_PRIMARY));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().status.as_deref(),
        Some("New connection action selected")
    );
    h.app_mut().status = None;
    let _ = h.click_id(CONNECTION_PRIMARY);
    assert_eq!(
        h.app().status.as_deref(),
        Some("New connection action selected")
    );
    let _ = h.click_id(CONNECTION_IMPORT);
    assert_eq!(h.app().status.as_deref(), Some("Import action selected"));
    h.app_mut().status = None;
    assert!(h.tab_to(CONNECTION_IMPORT));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().status.as_deref(), Some("Import action selected"));

    click_tab(&mut h, 1);
    choose(&mut h, OPTION_KIND, 0, 1);
    click_tab(&mut h, 0);
    let text = h.text();
    assert!(text.contains("Could not load connections"), "{text}");
    assert!(
        text.contains("The config file is unreadable: check its permissions"),
        "{text}"
    );
    let retry_area = h.area_of(CONNECTION_PRIMARY).expect("Retry is drawn");
    assert!(h.tab_to(CONNECTION_PRIMARY));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().status.as_deref(),
        Some("Retry selected for the connection sample")
    );
    h.app_mut().status = None;
    let _ = h.click_id(CONNECTION_PRIMARY);
    assert_eq!(
        h.app().status.as_deref(),
        Some("Retry selected for the connection sample")
    );

    click_tab(&mut h, 1);
    choose(&mut h, OPTION_ACTIONS, 2, 1);
    click_tab(&mut h, 0);
    assert!(h.area_of(CONNECTION_PRIMARY).is_some());
    assert!(h.area_of(CONNECTION_IMPORT).is_none());
    assert!(h.tab_to(CONNECTION_PRIMARY));
    h.app_mut().status = None;
    let _ = h.click_id(CONNECTION_PRIMARY);
    assert_eq!(
        h.app().status.as_deref(),
        Some("Retry selected for the connection sample")
    );

    click_tab(&mut h, 1);
    choose(&mut h, OPTION_ACTIONS, 1, 0);
    click_tab(&mut h, 0);
    assert!(h.area_of(CONNECTION_PRIMARY).is_none());
    assert!(h.area_of(CONNECTION_IMPORT).is_none());
    assert_ne!(h.focus(), Some(CONNECTION_PRIMARY));
    let prior_status = h.app().status.clone();
    let _ = h.click(
        retry_area.x.saturating_add(retry_area.width / 2),
        retry_area.y,
    );
    assert_eq!(
        h.app().status,
        prior_status,
        "hidden actions cannot activate"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn orders_refuse_enter_and_f2_without_changing_rows_and_keep_copy_live() {
    let mut h = host(100, 40);
    let before = h.text();
    assert!(before.contains("1041"), "{before}");
    assert!(before.contains("Acme Corp"), "{before}");
    assert!(before.contains("1042"), "{before}");
    assert!(before.contains("Globex"), "{before}");

    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("orders_view is a view: views cannot be edited")),
        "{:?}",
        h.app().status
    );
    assert!(!page_debug(&h).contains("edit: Some"), "{}", page_debug(&h));

    h.app_mut().status = None;
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::F(2));
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("orders_view is a view: views cannot be edited")),
        "{:?}",
        h.app().status
    );
    let _ = h.key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("Grid copy preview (not written)")),
        "{:?}",
        h.app().status
    );
    let after = h.text();
    assert!(after.contains("1041"), "{after}");
    assert!(after.contains("Acme Corp"), "{after}");
    assert!(after.contains("Globex"), "{after}");
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("Grid copy preview (not written): 1042\n")),
        "{:?}",
        h.app().status
    );

    click_tab(&mut h, 1);
    choose(&mut h, OPTION_REASON, 0, 1);
    click_tab(&mut h, 0);
    assert!(h.tab_to(ORDERS_GRID));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app().status.as_deref().is_some_and(|status| {
            status.contains("orders_view has no primary key, so rows cannot be told apart")
        }),
        "{:?}",
        h.app().status
    );
}

#[test]
fn row_menu_blocks_mutations_and_reports_copy_previews_without_claiming_a_write() {
    let mut h = host(100, 40);
    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.area_of(ROW_MENU).is_some());

    for row in 0..3 {
        let _ = h.click_part(ROW_MENU, PartRef::item(Part::ROW, ItemKey::index(row)));
        assert_eq!(h.app().status.as_deref(), Some("Row actions opened"));
        assert!(
            h.area_of(ROW_MENU).is_some(),
            "disabled item {row} leaves menu open"
        );
    }
    for _ in 0..3 {
        let _ = h.key(KeyCode::Up);
        assert!(
            h.text().contains("read-only view"),
            "{}\n{:?}",
            h.text(),
            h.app().page
        );
    }

    let _ = h.click_part(ROW_MENU, PartRef::item(Part::ROW, ItemKey::index(3)));
    assert!(
        h.app().status.as_deref().is_some_and(|status| {
            status.starts_with("Copy preview (not written): 1041\tAcme Corp\t1204.5")
        }),
        "status={:?}; menu-open={}; text={}",
        h.app().status,
        h.is_open(ROW_MENU),
        h.text()
    );
    assert!(
        !h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.starts_with("Copied"))
    );
    let text = h.text();
    assert!(text.contains("1041"), "{text}");
    assert!(text.contains("1042"), "{text}");
    assert!(!h.is_open(ROW_MENU));

    let _ = h.click_id(ROW_ACTIONS);
    let _ = h.click_part(ROW_MENU, PartRef::item(Part::ROW, ItemKey::index(4)));
    assert!(h.app().status.as_deref().is_some_and(|status| {
        status.starts_with("CSV preview (not written): order,customer,total")
            && status.contains("1041,Acme Corp,1204.5")
            && status.contains("1042,Globex,88.0")
    }));
    assert!(!h.is_open(ROW_MENU));

    assert!(h.tab_to(ROW_ACTIONS));
    let _ = h.key(KeyCode::Enter);
    assert!(h.is_open(ROW_MENU));
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROW_MENU));

    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.is_open(ROW_MENU));
    let _ = h.click(0, 0);
    assert!(!h.is_open(ROW_MENU));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn reference_gallery_preserves_canonical_columns_rows_and_payloads() {
    let mut h = host(200, 60);
    click_tab(&mut h, 2);
    let scroll = h
        .area_of(STATES_SCROLL)
        .expect("the States section registers its ScrollRegion");
    assert!(scroll.height >= GALLERY_HEIGHT as u16, "{scroll:?}");
    let header = text_in(&h, Rect::new(scroll.x, scroll.y, view_width(scroll), 1));
    assert!(header.contains("State"), "{header:?}");
    for column in ["Title", "Title + hint", "With actions"] {
        assert!(header.contains(column), "missing {column:?}: {header:?}");
    }
    for (row, label) in GALLERY_ROWS.iter().enumerate() {
        let top = scroll.y.saturating_add(1 + (row as u16).saturating_mul(3));
        let row_label = text_in(
            &h,
            Rect::new(scroll.x, top.saturating_add(1), GALLERY_LABEL_WIDTH, 1),
        );
        assert!(
            row_label.contains(label),
            "missing {label:?}: {row_label:?}"
        );
        assert_gallery_row(&h, scroll, top, row);
    }

    let overflow_top = scroll.y.saturating_add(1 + 5 * 3);
    let overflow_columns = [
        gallery_cell_text(&h, scroll, overflow_top, 0),
        gallery_cell_text(&h, scroll, overflow_top, 1),
        gallery_cell_text(&h, scroll, overflow_top, 2),
    ];
    assert!(
        overflow_columns[0].contains("No rows in a view far too narrow for it"),
        "{overflow_columns:?}"
    );
    assert!(
        overflow_columns[1].contains("No rows in a view far too narrow for it")
            && overflow_columns[1].contains("Run the query to fill it"),
        "{overflow_columns:?}"
    );
    assert!(
        overflow_columns[2].contains("No connections in a view far too narrow for it")
            && overflow_columns[2].contains("New"),
        "{overflow_columns:?}"
    );

    let narrow_top = scroll.y.saturating_add(1 + 6 * 3);
    let narrow_hint = gallery_cell_text(&h, scroll, narrow_top, 1);
    let narrow_action = gallery_cell_text(&h, scroll, narrow_top, 2);
    assert!(narrow_hint.contains("Run the"), "{narrow_hint:?}");
    assert!(
        !narrow_hint.contains("Run the query to fill it"),
        "{narrow_hint:?}"
    );
    assert!(narrow_action.contains("No"), "{narrow_action:?}");
    assert!(
        !narrow_action.contains("No connections"),
        "{narrow_action:?}"
    );
    assert!(narrow_action.contains("New"), "{narrow_action:?}");

    let gallery_text = h.text();
    assert!(gallery_text.contains("N/A · no action"), "{gallery_text}");
    for row in 0..7 {
        let action = GALLERY_ACTION.item(ItemKey::index(row * 3 + 2));
        assert!(
            h.area_of(action).is_none(),
            "live gallery target: {action:?}"
        );
        assert!(
            !h.tab_to(action),
            "gallery action must remain inert: {action:?}"
        );
    }
    let action_column = gallery_column(scroll, 2);
    let _ = h.click(
        action_column.x.saturating_add(1),
        scroll.y.saturating_add(3),
    );
    assert!(h.app().status.is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn short_offset_gallery_scroll_keeps_every_canonical_row_reachable_and_inert() {
    let page_area = Rect::new(7, 4, 66, 16);
    let mut h = area_host(80, 24, page_area);
    click_tab(&mut h, 2);
    let mut seen = [false; 7];

    for _ in 0..24 {
        let scroll = h
            .area_of(STATES_SCROLL)
            .expect("the offset States section keeps its ScrollRegion");
        for (row, label) in GALLERY_ROWS.iter().enumerate() {
            for y in scroll.y..scroll.bottom() {
                let row_label = text_in(&h, Rect::new(scroll.x, y, GALLERY_LABEL_WIDTH, 1));
                if !row_label.contains(label) {
                    continue;
                }
                let Some(top) = y.checked_sub(1) else {
                    continue;
                };
                if top < scroll.y || top.saturating_add(3) > scroll.bottom() {
                    continue;
                }
                seen[row] = true;
                assert_gallery_row(&h, scroll, top, row);
            }
        }
        for row in 0..7 {
            let action = GALLERY_ACTION.item(ItemKey::index(row * 3 + 2));
            assert!(
                h.area_of(action).is_none(),
                "ghost gallery target: {action:?}"
            );
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
        let _ = h.wheel(Axis::V, 1, scroll.x.saturating_add(1), scroll.y);
    }

    assert!(
        seen.into_iter().all(|visible| visible),
        "unreached rows: {seen:?}"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn offset_and_short_page_geometry_never_registers_outside_the_supplied_area() {
    let area = Rect::new(7, 4, 58, 11);
    let mut h = area_host(80, 24, area);
    let row_actions = h
        .area_of(ROW_ACTIONS)
        .expect("the offset Playground draws its Row actions button");
    let orders_grid = h
        .area_of(ORDERS_GRID)
        .expect("the offset Playground draws its Orders grid");
    for (id, hit) in [(ROW_ACTIONS, row_actions), (ORDERS_GRID, orders_grid)] {
        assert!(
            hit.x >= area.x && hit.right() <= area.right(),
            "{id:?}: {hit:?}"
        );
        assert!(
            hit.y >= area.y && hit.bottom() <= area.bottom(),
            "{id:?}: {hit:?}"
        );
    }
    assert!(h.area_of(CONNECTION_PRIMARY).is_none());

    assert!(
        h.tab_to(ROW_ACTIONS),
        "Row actions remains a real hit target"
    );
    let _ = h.click_id(ROW_ACTIONS);
    assert!(h.is_open(ROW_MENU), "Row actions opens its menu by pointer");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.is_open(ROW_MENU));

    assert!(
        h.tab_to(ORDERS_GRID),
        "the read-only grid remains focusable"
    );
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app().status.as_deref().is_some_and(|status| {
            status.contains("orders_view is a view: views cannot be edited")
        }),
        "the real grid hit explains its read-only state: {:?}",
        h.app().status
    );
    h.app_mut().status = None;
    let _ = h.click(2, 8);
    assert!(h.app().status.is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn offset_states_page_keeps_scroll_and_inert_gallery_inside_its_area() {
    let area = Rect::new(7, 4, 58, 11);
    let mut h = area_host(80, 24, area);
    click_tab(&mut h, 2);

    for y in 0..24 {
        for x in 0..80 {
            let inside = x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
            if !inside {
                assert_eq!(h.cell(x, y).symbol(), "S", "outside page at ({x}, {y})");
            }
        }
    }

    let scroll = h
        .area_of(STATES_SCROLL)
        .expect("the offset States section keeps its ScrollRegion");
    assert!(!scroll.is_empty(), "the States ScrollRegion is hittable");
    assert!(
        scroll.x >= area.x && scroll.right() <= area.right(),
        "{scroll:?}"
    );
    assert!(
        scroll.y >= area.y && scroll.bottom() <= area.bottom(),
        "{scroll:?}"
    );

    for row in 0..7 {
        let action = GALLERY_ACTION.item(ItemKey::index(row * 3 + 2));
        assert!(
            h.area_of(action).is_none(),
            "the inert States gallery registers no action target: {action:?}"
        );
        assert!(
            !h.tab_to(action),
            "the inert States gallery adds no action to the focus ring: {action:?}"
        );
    }
    let _ = h.click(scroll.right().saturating_sub(1), scroll.y);
    assert!(h.app().status.is_none(), "the gallery remains inert");
    let _ = h.click(2, 8);
    assert!(
        h.app().status.is_none(),
        "outside clicks do not reach the page"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn live_controls_are_focusable_at_supported_shell_sizes() {
    for (width, height) in [(80, 24), (100, 30), (60, 24)] {
        let mut h = host(width, height);
        for id in [
            CONNECTION_PRIMARY,
            CONNECTION_IMPORT,
            ROW_ACTIONS,
            ORDERS_GRID,
        ] {
            let area = h
                .area_of(id)
                .unwrap_or_else(|| panic!("{id:?} is not drawn at {width}x{height}\n{}", h.text()));
            assert!(
                area.right() <= width && area.bottom() <= height,
                "{id:?}: {area:?} at {width}x{height}"
            );
        }
        assert!(h.tab_to(CONNECTION_PRIMARY), "{width}x{height}");
        assert!(h.tab_to(CONNECTION_IMPORT), "{width}x{height}");
        assert!(h.tab_to(ROW_ACTIONS), "{width}x{height}");
        assert!(h.tab_to(ORDERS_GRID), "{width}x{height}");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
