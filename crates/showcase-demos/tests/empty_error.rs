use showcase_demos::pages::{Page, empty_error::EmptyErrorPage};
use termrock::{
    App, Cx, Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Rect, Response, Theme, Ui,
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
const GALLERY_ACTION: Id = Id::root("showcase_ui::pages::empty_error.gallery.action");

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
fn reference_gallery_uses_inert_components_and_marks_noninteractive_states() {
    let mut h = host(100, 40);
    click_tab(&mut h, 2);
    let text = h.text();
    assert!(
        text.contains("Rest: No rows"),
        "{text}\npage={:?}",
        h.app().page
    );
    assert!(text.contains("Hover: No connections"), "{text}");
    assert!(text.contains("Pressed: No connections"), "{text}");
    assert!(text.contains("Focused: No connections"), "{text}");
    assert!(text.contains("Could not load connections"), "{text}");
    assert!(text.contains("Overflow:"), "{text}");
    assert!(text.contains("Narrow:"), "{text}");
    assert!(text.contains("N/A · no action"), "{text}");
    let hovered_action = GALLERY_ACTION.item(ItemKey::index(5));
    let focused_action = GALLERY_ACTION.item(ItemKey::index(11));
    assert!(h.area_of(hovered_action).is_none());
    assert!(h.area_of(focused_action).is_none());
    assert!(!h.tab_to(hovered_action));
    let _ = h.click(10, 4);
    assert!(h.app().status.is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn offset_and_short_page_geometry_never_registers_outside_the_supplied_area() {
    let area = Rect::new(7, 4, 58, 11);
    let mut h = area_host(80, 24, area);
    assert_eq!(h.cell(0, 0).symbol(), "S");
    assert_eq!(h.cell(6, 7).symbol(), "S");
    assert_eq!(h.cell(66, 7).symbol(), "S");

    for id in [ROW_ACTIONS, ORDERS_GRID] {
        if let Some(hit) = h.area_of(id) {
            assert!(
                hit.x >= area.x && hit.right() <= area.right(),
                "{id:?}: {hit:?}"
            );
            assert!(
                hit.y >= area.y && hit.bottom() <= area.bottom(),
                "{id:?}: {hit:?}"
            );
        }
    }
    assert!(h.area_of(CONNECTION_PRIMARY).is_none());
    let _ = h.click(2, 8);
    assert!(h.app().status.is_none());
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
