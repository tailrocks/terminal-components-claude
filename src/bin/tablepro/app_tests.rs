//! End-to-end interaction tests through the real App on a TestBackend.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

use crate::app::{App, Modal, Screen};
use crate::workbench::WorkTab;

pub struct H {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

impl H {
    pub fn new(w: u16, h: u16) -> Self {
        let app = App::new(Theme::junie());
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut hh = Self { app, term };
        hh.draw();
        hh
    }
    pub fn connected(w: u16, h: u16) -> Self {
        let mut hh = Self::new(w, h);
        let i = hh
            .app
            .connections
            .connections
            .iter()
            .position(|c| c.name == "Production")
            .unwrap();
        hh.app.connect(i);
        hh.draw();
        hh
    }
    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }
    pub fn key(&mut self, code: KeyCode) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
        o
    }
    pub fn ctrl(&mut self, c: char) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::CONTROL,
        }));
        self.draw();
        o
    }
    pub fn alt(&mut self, c: char) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::ALT,
        }));
        self.draw();
        o
    }
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }
    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.app.handle(Input::Tick);
        }
        self.draw();
    }
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
        let o = self.app.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
        }));
        self.draw();
        o
    }
    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }
    pub fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        let want: Vec<&str> =
            unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect();
        for y in 0..buf.area.height {
            let cells: Vec<&str> = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            for x in 0..cells.len().saturating_sub(want.len() - 1) {
                if cells[x..x + want.len()] == want[..] {
                    return Some((x as u16, y));
                }
            }
        }
        None
    }
    pub fn focus(&self) -> Option<WidgetId> {
        self.app.focus.current()
    }
    pub fn wb(&self) -> &crate::workbench::Workbench {
        self.app.workbench.as_ref().unwrap()
    }
}

#[test]
fn connections_screen_lists_and_connects_with_keyboard() {
    let mut h = H::new(120, 40);
    assert_eq!(h.app.screen, Screen::Connections);
    assert!(h.text().contains("Local PostgreSQL"));
    assert!(h.text().contains("Production"));
    // navigate to Production and connect
    for _ in 0..8 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Safe Mode"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Opening SSH tunnel") || h.text().contains("Authenticating"));
    h.ticks(14);
    assert_eq!(h.app.screen, Screen::Workbench);
    assert_eq!(h.wb().connection.name, "Production");
    assert!(h.text().contains("production"));
    assert!(
        h.text().contains("safe"),
        "safe mode token visible in the strip"
    );
}

#[test]
fn failed_connection_shows_error_and_retry() {
    let mut h = H::new(120, 40);
    let i = h
        .app
        .connections
        .connections
        .iter()
        .position(|c| c.name == "Analytics")
        .unwrap();
    h.app.connections.start_connect(i);
    h.ticks(14);
    assert_eq!(h.app.screen, Screen::Connections);
    assert!(h.text().contains("Could not reach the host"));
    assert!(h.text().contains("Reconnect"));
}

#[test]
fn explorer_opens_table_and_grid_navigates() {
    let mut h = H::connected(120, 40);
    assert!(h.text().contains("organizations"));
    // cursor starts on the schema row; move to `orders` (4th table) and open
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let wb = h.wb();
    assert!(matches!(wb.active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    assert!(h.text().contains("public › orders"));
    assert!(h.text().contains("order_number"));
    // grid navigation
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::End);
    let g = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => &t.grid,
        _ => unreachable!(),
    };
    assert_eq!(g.cursor.0, 1);
    assert_eq!(g.cursor.1, g.columns.len() - 1);
    assert!(g.hscroll.offset > 0, "wide table scrolled horizontally");
    assert!(h.text().contains("cols "));
}

#[test]
fn sort_and_filter_on_table_tab() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    // move to created_at column: it is the 13th column (index 12)
    for _ in 0..12 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Char('s'));
    let t = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => unreachable!(),
    };
    assert_eq!(
        t.sort.map(|s| t.columns[s.0].0.clone()),
        Some("created_at".into())
    );
    assert!(h.text().contains("sort created_at ▴"));
    h.key(KeyCode::Char('s'));
    assert!(h.text().contains("sort created_at ▾"));
    // filter status = pending via the filter editor
    h.key(KeyCode::Home);
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Char('f')); // filter on the current cell value
    assert!(matches!(h.app.modal, Some(Modal::Filter(_))));
    // the value field is prefilled with the cell's value and Apply has focus;
    // go back to the value field and replace it with pending
    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("pending");
    h.key(KeyCode::Enter); // commit field → applies
    let t = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => unreachable!(),
    };
    assert_eq!(t.filters.len(), 1);
    assert_eq!(t.filters[0].value, "pending");
    assert!(h.text().contains("filtered (1)"), "{}", h.text());
    assert!(h.text().contains("status = 'pending'"));
    // every visible status cell is pending
    let g = &t.grid;
    let status = t.columns.iter().position(|c| c.0 == "status").unwrap();
    assert!(g.rows().iter().all(|r| r[status].text() == "pending"));
}

#[test]
fn structure_view_toggle() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ctrl('d');
    assert!(h.text().contains("Columns"));
    assert!(h.text().contains("Indexes"));
    assert!(h.text().contains("timestamptz"));
    // jump to Foreign keys section
    let t = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => unreachable!(),
    };
    assert_eq!(t.mode_tabs.active, 1);
    h.ctrl('d');
    assert!(h.text().contains("rows 1–"));
}

#[test]
fn editor_completion_and_execution() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab); // editor
    let q = match h.wb().active_tab() {
        Some(WorkTab::Query(q)) => q,
        _ => unreachable!(),
    };
    assert_eq!(h.focus(), Some(q.editor.id));
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM ord");
    assert!(
        h.wb_query().completion.is_open(),
        "autocomplete opened after FROM + prefix"
    );
    // the explorer lists order_items too; the popup adds a second copy
    assert!(
        h.text().matches("order_items").count() >= 2,
        "completion popup is drawn"
    );
    h.key(KeyCode::Enter); // accept `orders`
    assert!(h.wb_query().editor.text().ends_with("orders"));
    h.type_str(" WHERE st");
    assert!(h.wb_query().completion.is_open());
    assert_eq!(
        h.wb_query().completion.current().map(|c| c.label.as_str()),
        Some("status")
    );
    h.key(KeyCode::Tab);
    h.type_str(" = 'pending' ORDER BY created_at DESC LIMIT 25");
    h.key(KeyCode::Esc); // nav mode
    assert!(!h.wb_query().editor.editing);
    h.ctrl('r');
    assert!(h.wb_query().is_running());
    assert!(h.text().contains("running"));
    h.ticks(10);
    assert!(!h.wb_query().is_running());
    assert_eq!(h.wb_query().results.len(), 1);
    assert!(h.text().contains("SELECT orders (25)"));
    assert!(h.text().contains("25 rows"));
    // history recorded it
    assert!(h.app.history.entries[0].sql.contains("LIMIT 25"));
}

#[test]
fn execution_error_marks_editor_and_result() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT nope FROM orders");
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    h.ctrl('r');
    h.ticks(10);
    assert!(h.text().contains("column \"nope\" does not exist"));
    assert!(!h.wb_query().editor.diagnostics.is_empty());
    assert!(h.text().contains("Error 1"));
}

#[test]
fn cancel_running_query() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM events");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(h.wb_query().is_running());
    h.key(KeyCode::Esc);
    assert!(!h.wb_query().is_running());
    assert!(h.text().contains("Cancelled"));
    assert_ne!(
        h.app.history.entries[0].sql, "SELECT * FROM events",
        "cancelled runs are not recorded"
    );
}

#[test]
fn explain_opens_plan_tree() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10");
    h.key(KeyCode::Esc);
    h.alt('x');
    h.ticks(10);
    assert!(h.text().contains("EXPLAIN ANALYZE"));
    assert!(h.text().contains("Limit"));
    assert!(h.text().contains("Sort"));
    assert!(h.text().contains("Seq Scan"));
    // plan tree is focusable: Tab to the result tabs, then to the tree; collapse a node
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    let before = h.text();
    h.key(KeyCode::Left);
    assert_ne!(before, h.text());
    // raw mode
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("cost="));
}

#[test]
fn safety_gate_intercepts_dangerous_statement_on_production() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(matches!(h.app.modal, Some(Modal::Dialog(_))));
    let t = h.text();
    assert!(t.contains("DELETE without WHERE"));
    assert!(t.contains("every row in orders"));
    assert!(t.contains("Production"));
    assert!(t.contains("Type orders to confirm"));
    // the confirming button is disabled until the token matches: a wrong token
    // keeps it out of the focus ring and Right cannot reach it
    h.key(KeyCode::Enter);
    h.type_str("wrong");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.app.modal.is_none(), "Enter landed on Cancel");
    assert!(!h.wb_query().is_running());
    h.ctrl('r');
    h.key(KeyCode::Esc);
    assert!(h.app.modal.is_none());
    assert!(!h.wb_query().is_running());
    assert!(h.text().contains("Cancelled · nothing was executed"));
}

#[test]
fn safety_gate_typed_token_executes() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    // Safe Mode level on Production requires the deliberate acknowledgement
    assert!(h.text().contains("Type orders to confirm"));
    h.key(KeyCode::Enter); // start editing the ack field
    h.type_str("orders");
    h.key(KeyCode::Enter); // moves focus to the buttons
    h.key(KeyCode::Right); // Execute
    h.key(KeyCode::Enter);
    assert!(h.app.modal.is_none(), "focus={:?}\n{}", h.focus(), h.text());
    assert!(h.wb_query().is_running());
    h.ticks(10);
    assert!(h.text().contains("rows affected"));
}

#[test]
fn read_only_connection_refuses_writes() {
    let mut h = H::connected(120, 40);
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::ReadOnly;
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("DELETE FROM orders WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(h.app.modal.is_none());
    assert!(!h.wb_query().is_running());
    assert!(h.text().contains("read-only for this connection"));
}

#[test]
fn silent_level_runs_scoped_writes_but_confirms_destructive() {
    let mut h = H::connected(120, 40);
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::Silent;
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("UPDATE orders SET status = 'paid'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(
        h.app.modal.is_none(),
        "UPDATE without WHERE is a plain write in TablePro"
    );
    h.ticks(10);
    h.key(KeyCode::Char('i'));
    h.ctrl('l');
    h.type_str("TRUNCATE orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(
        matches!(h.app.modal, Some(Modal::Dialog(_))),
        "destructive always confirms"
    );
    assert!(h.text().contains("TRUNCATE"));
}

#[test]
fn quick_switcher_opens_table() {
    let mut h = H::connected(120, 40);
    h.ctrl('o');
    assert!(matches!(h.app.modal, Some(Modal::Picker(..))));
    h.type_str("cust");
    assert!(h.text().contains("customers"));
    h.key(KeyCode::Enter);
    assert!(h.app.modal.is_none());
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers"));
    // Esc clears then closes
    h.ctrl('o');
    h.type_str("x");
    h.key(KeyCode::Esc);
    assert!(matches!(h.app.modal, Some(Modal::Picker(..))));
    h.key(KeyCode::Esc);
    assert!(h.app.modal.is_none());
}

#[test]
fn history_tab_reopens_query() {
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::History(_))));
    assert!(h.text().contains("SELECT plan, count(*)"));
    h.key(KeyCode::Char('/'));
    h.type_str("payments");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    assert!(h.text().contains("payments"));
    h.key(KeyCode::Enter);
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Query(q)) if q.editor.text().contains("payments"))
    );
}

#[test]
fn tab_strip_overflow_and_tab_list() {
    let mut h = H::connected(100, 30);
    let names = [
        "organizations",
        "customers",
        "products",
        "orders",
        "order_items",
        "payments",
        "subscriptions",
    ];
    for n in names {
        h.app
            .workbench
            .as_mut()
            .unwrap()
            .open_table("public", n, false);
    }
    for _ in 0..4 {
        h.ctrl('t');
    }
    h.draw();
    assert!(h.wb().tabs.len() >= 12);
    assert!(
        h.text().contains("›") || h.text().contains("‹"),
        "overflow indicators"
    );
    h.ctrl('g');
    assert!(matches!(h.app.modal, Some(Modal::Picker(..))));
    assert!(h.text().contains("Open tabs"));
    h.key(KeyCode::Up);
    h.key(KeyCode::Enter);
    assert!(h.app.modal.is_none());
}

#[test]
fn pending_edits_preview_and_save() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter); // orders
    // edit the currency column (index 6)
    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("EUR");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("• 1 pending"), "{}", h.text());
    h.key(KeyCode::Char('p'));
    assert!(
        h.text()
            .contains("UPDATE public.orders SET currency = 'EUR'")
    );
    h.key(KeyCode::Esc);
    h.ctrl('s');
    assert!(h.text().contains("Save changes?"));
    // Production is Safe Mode: token required
    h.key(KeyCode::Enter);
    h.type_str("orders");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(8);
    assert!(h.text().contains("Saved 1 change"));
    let t = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => unreachable!(),
    };
    assert!(t.grid.pending.is_empty());
}

#[test]
fn safe_mode_picker_changes_level_and_strip() {
    let mut h = H::connected(120, 40);
    h.ctrl('l');
    assert!(h.text().contains("Safe Mode · this connection"));
    h.key(KeyCode::Down); // Safe Mode (Full)
    h.key(KeyCode::Enter);
    assert_eq!(h.wb().connection.safe_mode, crate::db::SafeMode::SafeFull);
    assert!(h.text().contains("safe+"));
}

#[test]
fn mouse_opens_table_and_switches_tabs() {
    let mut h = H::connected(120, 40);
    let (x, y) = h.find("customers").unwrap();
    h.click(x, y);
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers" && t.preview)
    );
    // second click promotes the preview tab
    h.click(x, y);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if !t.preview));
    // click the Query 1 tab in the strip
    let (qx, qy) = h.find("Query 1").unwrap();
    h.click(qx, qy);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));
    // hover on a tab lifts it
    let (cx, cy) = h.find("customers").unwrap();
    h.mouse(MouseKind::Move, cx + 30, cy); // somewhere harmless
    h.mouse(MouseKind::Move, cx, cy);
    assert!(h.app.hover.is_some());
}

#[test]
fn every_screen_renders_at_representative_sizes() {
    for (w, hgt) in [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)] {
        let mut h = H::new(w, hgt);
        for _ in 0..6 {
            h.key(KeyCode::Tab);
            h.key(KeyCode::Down);
        }
        let mut h = H::connected(w, hgt);
        for _ in 0..5 {
            h.key(KeyCode::Down);
        }
        h.key(KeyCode::Enter);
        for _ in 0..6 {
            h.key(KeyCode::Right);
        }
        h.ctrl('d');
        h.ctrl('t');
        h.key(KeyCode::Char('i'));
        h.type_str("SELECT * FROM orders LIMIT 5");
        h.key(KeyCode::Esc);
        h.ctrl('r');
        h.ticks(10);
        h.alt('x');
        h.ticks(10);
        h.ctrl('y');
        h.ctrl('o');
        h.key(KeyCode::Esc);
        h.ctrl('g');
        h.key(KeyCode::Esc);
        h.ctrl('l');
        h.key(KeyCode::Esc);
        h.key(KeyCode::Char('z'));
        h.key(KeyCode::Esc);
        h.key(KeyCode::Char('?'));
        h.key(KeyCode::Esc);
        assert!(!h.app.quit);
    }
    let h = H::new(60, 15);
    assert!(h.text().contains("Terminal too small"));
}

impl H {
    pub fn wb_query(&self) -> &crate::tabs::QueryTab {
        match self.wb().active_tab() {
            Some(WorkTab::Query(q)) => q,
            _ => panic!("active tab is not a query"),
        }
    }
}
