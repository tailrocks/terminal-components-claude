use junie_tui::core::event::Input;
use ratatui::crossterm::event::KeyCode;

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
