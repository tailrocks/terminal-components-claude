use junie_tui::core::event::{Input, MouseKind};
use junie_tui::widgets::scrollbar;
use ratatui::crossterm::event::KeyCode;

use crate::app::Modal;
use crate::app_tests::H;
use crate::connections::ConnState;
use crate::db::SafeMode;
use crate::tabs::{Filter, FilterOp, ResultBody};
use crate::workbench::WorkTab;

fn open_orders(h: &mut H) {
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}

fn set_query(h: &mut H, sql: &str) {
    h.ctrl('t');
    match h.app.workbench.as_mut().unwrap().active_tab_mut() {
        Some(WorkTab::Query(query)) => query.editor.set_text(sql),
        _ => panic!("expected query tab"),
    }
    h.draw();
}

fn set_safe_mode(h: &mut H, mode: SafeMode) {
    h.app.workbench.as_mut().unwrap().connection.safe_mode = mode;
    h.draw();
}

fn accept_dialog(h: &mut H) {
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
}

fn confirm_destructive(h: &mut H, token: &str) {
    h.key(KeyCode::Enter);
    h.type_str(token);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
}

fn active_query(h: &H) -> &crate::tabs::QueryTab {
    match h.wb().active_tab() {
        Some(WorkTab::Query(query)) => query,
        _ => panic!("expected active query"),
    }
}

#[test]
fn auth_retry_and_connection_test_outcomes() {
    let mut h = H::new(120, 40);
    let (x, y) = h.find("Staging").unwrap();
    h.click(x, y);
    h.key(KeyCode::Enter);
    h.ticks(12);
    assert!(matches!(
        &h.app.connections.state,
        ConnState::Failed { message, .. } if message == "Authentication failed"
    ));

    let (x, y) = h.find("Reconnect").unwrap();
    h.click(x, y);
    h.ticks(12);
    assert!(matches!(
        &h.app.connections.state,
        ConnState::Failed { message, .. } if message == "Authentication failed"
    ));

    let mut h = H::new(120, 40);
    let (x, y) = h.find("Local PostgreSQL").unwrap();
    h.click(x, y);
    h.key(KeyCode::Char('e'));
    let (x, y) = h.find("Test connection").unwrap();
    h.click(x, y);
    h.ticks(10);
    assert!(matches!(&h.app.connections.state, ConnState::Tested(Ok(_))));

    let mut h = H::new(120, 40);
    let (x, y) = h.find("Analytics").unwrap();
    h.click(x, y);
    h.key(KeyCode::Char('e'));
    let (x, y) = h.find("Test connection").unwrap();
    h.click(x, y);
    h.ticks(10);
    assert!(matches!(
        &h.app.connections.state,
        ConnState::Tested(Err(message)) if message.contains("timed out")
    ));
}

#[test]
fn connection_validation_save_edit_and_delete() {
    let mut h = H::new(120, 40);
    h.ctrl('n');
    h.ctrl('s');
    assert!(h.text().contains("Fix the highlighted fields"));

    h.key(KeyCode::Enter);
    h.type_str("Coverage");
    h.key(KeyCode::Enter);
    h.ctrl('s');
    assert!(
        h.app
            .connections
            .connections
            .iter()
            .any(|c| c.name == "Coverage")
    );

    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("Coverage edited");
    h.key(KeyCode::Enter);
    h.ctrl('s');
    assert!(
        h.app
            .connections
            .connections
            .iter()
            .any(|c| c.name == "Coverage edited")
    );

    let (x, y) = h.find("Coverage edited").unwrap();
    h.click(x, y);
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("Delete connection?"));
    accept_dialog(&mut h);
    assert!(
        !h.app
            .connections
            .connections
            .iter()
            .any(|c| c.name == "Coverage edited")
    );
}

#[test]
fn structure_sections_and_workbench_grid_routes() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.ctrl('d');

    let structure_id = match h.wb().active_tab() {
        Some(WorkTab::Table(table)) => table.structure_tabs.id,
        _ => panic!("expected table tab"),
    };
    h.app.focus.focus(structure_id);
    h.draw();
    for (index, digit) in ['1', '2', '3', '4', '5', '6'].into_iter().enumerate() {
        h.key(KeyCode::Char(digit));
        match h.wb().active_tab() {
            Some(WorkTab::Table(table)) => assert_eq!(table.structure_tabs.active, index),
            _ => panic!("expected table tab"),
        }
    }

    h.ctrl('d');
    h.key(KeyCode::Char('G'));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Fetch more"), "{}", h.text());
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Refreshed"));
    h.key(KeyCode::Char('y'));
    assert!(h.text().contains("Copied"));

    let catalog = h.wb().catalog.clone();
    if let Some(WorkTab::Table(table)) = h.app.workbench.as_mut().unwrap().active_tab_mut() {
        table.filters = vec![Filter {
            column: "status".into(),
            op: FilterOp::Eq,
            value: "pending".into(),
            value2: String::new(),
            enabled: true,
        }];
        table.load(&catalog);
    }
    h.draw();
    h.key(KeyCode::Char('F'));
    match h.wb().active_tab() {
        Some(WorkTab::Table(table)) => assert!(table.filters.is_empty()),
        _ => panic!("expected table tab"),
    }

    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.key(KeyCode::Home);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.ctrl(']');
    match h.wb().active_tab() {
        Some(WorkTab::Table(table)) => assert_eq!(table.filters[0].column, "id"),
        _ => panic!("expected referenced table"),
    }

    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.key(KeyCode::Home);
    for _ in 0..7 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    assert!(h.app.modal.is_some());
    assert!(h.text().contains("shipping_address"), "{}", h.text());
    h.key(KeyCode::Esc);
}

#[test]
fn query_execution_and_result_routes() {
    let mut h = H::connected(120, 40);
    set_query(&mut h, "SELECT * FROM");
    h.ctrl('r');
    h.ticks(8);
    assert!(matches!(
        &active_query(&h).results[0].body,
        ResultBody::Error { .. }
    ));

    let mut h = H::connected(120, 40);
    set_safe_mode(&mut h, SafeMode::Silent);
    set_query(
        &mut h,
        "SELECT * FROM orders LIMIT 1; INSERT INTO orders; ALTER TABLE orders ADD COLUMN coverage_flag boolean",
    );
    h.alt('r');
    if h.app.modal.is_some() {
        accept_dialog(&mut h);
    }
    h.ticks(32);
    {
        let query = active_query(&h);
        assert!(
            query
                .results
                .iter()
                .any(|r| matches!(&r.body, ResultBody::Rows(_)))
        );
        assert!(query.results.iter().any(|r| {
            matches!(&r.body, ResultBody::Affected { verb, .. } if verb.contains("INSERT"))
        }));
        assert!(query.results.iter().any(|r| {
            matches!(&r.body, ResultBody::Affected { verb, .. } if verb.contains("ALTER"))
        }));
    }

    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('p'));
    assert!(active_query(&h).results.iter().any(|r| r.pinned));
    h.key(KeyCode::Char('p'));
    assert!(active_query(&h).results.iter().all(|r| !r.pinned));
    let result_count = active_query(&h).results.len();
    h.key(KeyCode::Char('x'));
    assert_eq!(active_query(&h).results.len(), result_count - 1);

    let mut h = H::connected(120, 40);
    set_safe_mode(&mut h, SafeMode::Silent);
    set_query(&mut h, "DELETE FROM orders WHERE id = 1");
    h.ctrl('r');
    if h.app.modal.is_some() {
        accept_dialog(&mut h);
    }
    h.ticks(10);
    assert!(active_query(&h).results.iter().any(|r| {
        matches!(&r.body, ResultBody::Affected { verb, .. } if verb.contains("DELETE"))
    }));

    let mut h = H::connected(120, 40);
    set_safe_mode(&mut h, SafeMode::Silent);
    set_query(&mut h, "TRUNCATE orders");
    h.ctrl('r');
    assert!(h.app.modal.is_some());
    confirm_destructive(&mut h, "orders");
    h.ticks(10);
    assert!(
        active_query(&h).results.iter().any(|r| {
            matches!(&r.body, ResultBody::Affected { verb, .. } if verb.contains("TRUNCATE"))
        }),
        "{}",
        h.text()
    );

    let mut h = H::connected(120, 40);
    h.ctrl('t');
    h.key(KeyCode::Char('i'));
    h.app.handle(Input::Paste("SELECT * FROM ord".into()));
    h.draw();
    h.ctrl(' ');
    assert!(h.wb_query().completion.is_open());
    h.key(KeyCode::Enter);
    assert!(h.wb_query().editor.text().ends_with("orders"));
}

#[test]
fn table_preview_discard_and_query_close_quit_routes_are_confirmed() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("EUR");
    h.key(KeyCode::Enter);
    assert_eq!(h.wb_table().grid.pending.total(), 1);

    h.key(KeyCode::Char('p'));
    assert!(h.app.modal.is_some());
    assert!(h.text().contains("UPDATE public.orders"));
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('U'));
    assert!(h.text().contains("Discard unsaved changes?"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.wb_table().grid.pending.is_empty());

    let mut h = H::connected(120, 40);
    h.ctrl('t');
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT 1");
    h.key(KeyCode::Esc);
    let tabs = h.wb().tabs.len();
    h.ctrl('w');
    assert!(h.app.modal.is_some());
    h.key(KeyCode::Esc);
    assert_eq!(h.wb().tabs.len(), tabs);
    h.ctrl('w');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.wb().tabs.len(), tabs - 1);

    h.ctrl('t');
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT 2");
    h.key(KeyCode::Esc);
    h.ctrl('c');
    assert!(h.app.modal.is_some());
    h.key(KeyCode::Esc);
    assert!(!h.app.quit);
    h.ctrl('c');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.app.quit);
}

impl H {
    fn wb_table(&self) -> &crate::tabs::TableTab {
        match self.wb().active_tab() {
            Some(WorkTab::Table(table)) => table,
            _ => panic!("active tab is not a table"),
        }
    }
}

#[test]
fn history_picker_and_filter_routes() {
    let mut h = H::connected(120, 40);
    h.ctrl('o');
    assert!(h.text().contains("All"));
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Tables"));
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Schemas"));
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Recent queries"));
    h.key(KeyCode::Esc);

    h.ctrl('l');
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.wb().connection.safe_mode, SafeMode::SafeFull);

    h.ctrl('y');
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Char('s'));
    match h.wb().active_tab() {
        Some(WorkTab::History(history)) => {
            assert!(history.scope_all);
            assert!(history.failed_only);
        }
        _ => panic!("expected history tab"),
    }
    h.key(KeyCode::Char('y'));
    assert!(h.text().contains("Query copied"));
    h.key(KeyCode::Char('/'));
    h.type_str("ordres");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('r'));
    assert!(h.app.modal.is_some(), "{}", h.text());
    h.key(KeyCode::Esc);
    h.app.workbench.as_mut().unwrap().connection.safe_mode = SafeMode::Silent;
    h.draw();
    h.ctrl('r');
    h.ticks(8);
    assert!(
        active_query(&h)
            .results
            .first()
            .is_some_and(|r| matches!(&r.body, ResultBody::Error { .. })),
        "{}",
        h.text()
    );

    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.ctrl('f');
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("A value is required"));
    h.key(KeyCode::Esc);
    assert!(h.app.modal.is_none());
}

#[test]
fn connection_password_is_masked_in_rendered_form() {
    let mut h = H::new(120, 40);
    h.ctrl('n');
    let (x, y) = h.find("Password").expect("password field label");
    h.click(x + 1, y + 1);
    h.type_str("s3cret");
    let text = h.text();
    assert!(
        !text.contains("s3cret"),
        "secret leaked into the buffer: {text}"
    );
    assert!(
        text.contains("••••••"),
        "masked value was not rendered: {text}"
    );
    h.key(KeyCode::Esc);
}

#[test]
fn resize_ladder_recovers_drawer_focus_and_hit_regions() {
    let mut h = H::connected(120, 40);
    let explorer = h.wb().explorer.id;
    assert_eq!(h.focus(), Some(explorer));

    h.resize(60, 15);
    assert!(h.text().contains("Terminal too small"));
    h.resize(120, 40);
    assert!(h.text().contains("Filter objects"));
    assert!(h.app.hits.area_of(explorer).is_some());

    h.resize(80, 24);
    assert!(h.text().contains("Filter objects"));
    assert!(!h.text().contains("Type SQL"));
    let editor = h.wb_query().editor.id;
    h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(editor));
    assert!(!h.text().contains("Filter objects"));

    h.resize(160, 50);
    assert!(h.text().contains("Type SQL"));
    assert!(h.app.hits.area_of(editor).is_some());
}

#[test]
fn overlays_trap_focus_and_restore_after_outside_dismissal() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let origin = h.focus();

    h.ctrl('o');
    assert!(matches!(h.app.modal, Some(Modal::Picker(..))));
    assert_ne!(h.focus(), origin);
    h.key(KeyCode::Tab);
    assert!(h.app.modal.is_some());
    assert_ne!(h.focus(), origin);
    h.click(0, 0);
    assert!(h.app.modal.is_none());
    assert_eq!(h.focus(), origin);

    h.ctrl('f');
    assert!(matches!(h.app.modal, Some(Modal::Filter(_))));
    h.click(0, 0);
    assert!(h.app.modal.is_none());
    assert_eq!(h.focus(), origin);
}

#[test]
fn structure_sections_render_catalog_evidence() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.ctrl('d');
    let structure_id = match h.wb().active_tab() {
        Some(WorkTab::Table(table)) => table.structure_tabs.id,
        _ => panic!("expected table tab"),
    };
    h.app.focus.focus(structure_id);
    h.draw();

    let sections = [
        ('1', 0, "Columns", "order_number"),
        ('2', 1, "Indexes", "orders_status_created_idx"),
        ('3', 2, "Foreign keys", "customers"),
        ('4', 3, "Constraints", "orders_status_check"),
        ('5', 4, "Triggers", "orders_audit"),
        ('6', 5, "DDL", "CREATE TABLE public.orders"),
    ];
    for (digit, index, label, evidence) in sections {
        h.key(KeyCode::Char(digit));
        match h.wb().active_tab() {
            Some(WorkTab::Table(table)) => assert_eq!(table.structure_tabs.active, index),
            _ => panic!("expected table tab"),
        }
        let text = h.text();
        assert!(text.contains(label), "section label missing: {text}");
        assert!(text.contains(evidence), "section evidence missing: {text}");
    }
}

#[test]
fn grid_pointer_drag_scrollbar_and_horizontal_wheel_update_state() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    let grid_id = h.wb_table().grid.id;
    let first = h
        .app
        .hits
        .area_of(h.wb_table().grid.cell_id(0, 0))
        .expect("first grid cell hit");
    let second = h
        .app
        .hits
        .area_of(h.wb_table().grid.cell_id(1, 1))
        .expect("second grid cell hit");
    h.mouse(MouseKind::Down, first.x + 1, first.y);
    h.mouse(MouseKind::Drag, second.x + 1, second.y);
    h.mouse(MouseKind::Up, second.x + 1, second.y);
    assert_eq!(h.wb_table().grid.cursor, (1, 1));

    let grid_area = h.app.hits.area_of(grid_id).expect("grid hit");
    let before = h.wb_table().grid.scroll.offset;
    h.mouse(MouseKind::WheelDown, grid_area.x + 8, grid_area.y + 3);
    assert!(h.wb_table().grid.scroll.offset > before);

    if let Some(WorkTab::Table(table)) = h.app.workbench.as_mut().unwrap().active_tab_mut() {
        table.grid.scroll.offset = 0;
        table.grid.hscroll.offset = 0;
    }
    h.draw();
    let scrollbar_area = h
        .app
        .hits
        .area_of(scrollbar::id_for(grid_id))
        .expect("grid scrollbar hit");
    let before = h.wb_table().grid.scroll.offset;
    h.mouse(MouseKind::Down, scrollbar_area.x, scrollbar_area.y);
    h.mouse(
        MouseKind::Drag,
        scrollbar_area.x,
        scrollbar_area.bottom().saturating_sub(1),
    );
    h.mouse(
        MouseKind::Up,
        scrollbar_area.x,
        scrollbar_area.bottom().saturating_sub(1),
    );
    assert!(h.wb_table().grid.scroll.offset > before);

    let grid_area = h.app.hits.area_of(grid_id).expect("grid hit");
    let before = h.wb_table().grid.hscroll.offset;
    h.mouse(MouseKind::WheelRight, grid_area.x + 8, grid_area.y + 3);
    assert!(h.wb_table().grid.hscroll.offset > before);
}

#[test]
fn grid_edit_paste_validation_and_read_only_columns_preserve_state() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);

    h.key(KeyCode::Home);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        !h.wb_table().grid.is_editing(),
        "generated column became editable"
    );

    h.key(KeyCode::Home);
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("bogus");
    h.key(KeyCode::Enter);
    assert!(h.wb_table().grid.is_editing());
    assert!(
        h.wb_table()
            .grid
            .edit_error()
            .is_some_and(|error| error.contains("one of"))
    );
    assert!(
        h.find("!").is_some(),
        "validation marker was not rendered: {}",
        h.text()
    );
    h.key(KeyCode::Esc);

    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.app.handle(Input::Paste("paid".into()));
    h.draw();
    h.key(KeyCode::Enter);
    assert!(!h.wb_table().grid.is_editing());
    assert_eq!(h.wb_table().grid.pending.total(), 1);
    assert_eq!(h.wb_table().grid.value(0, 4).text(), "paid");
    assert!(h.text().contains("paid"));
}

#[test]
fn result_tabs_route_back_to_statement_anchor() {
    let mut h = H::connected(120, 40);
    set_safe_mode(&mut h, SafeMode::Silent);
    let sql = "SELECT * FROM orders LIMIT 1; SELECT * FROM customers LIMIT 1";
    set_query(&mut h, sql);
    h.alt('r');
    h.ticks(16);
    assert_eq!(active_query(&h).results.len(), 2);
    let second_anchor = active_query(&h).results[1].anchor.start;
    let result_tabs = active_query(&h).result_tabs.id;
    h.tab_to(result_tabs);
    h.key(KeyCode::Left);
    h.key(KeyCode::Enter);
    assert_eq!(active_query(&h).active_result, 0);
    assert_eq!(
        active_query(&h).editor.cursor_offset(),
        active_query(&h).results[0].anchor.start
    );
    assert_ne!(active_query(&h).editor.cursor_offset(), second_anchor);
    assert!(h.text().contains("SELECT orders (1)"));
}
