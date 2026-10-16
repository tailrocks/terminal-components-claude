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
        let o = self
            .app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
        o
    }
    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }
    pub fn resize(&mut self, w: u16, h: u16) {
        self.term.backend_mut().resize(w, h);
        self.app.handle(Input::Resize(w, h));
        self.draw();
    }
    pub fn tab_to(&mut self, id: WidgetId) {
        for _ in 0..32 {
            if self.focus() == Some(id) {
                return;
            }
            self.key(KeyCode::Tab);
        }
        panic!("focus never reached {id:?}, current={:?}", self.focus());
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
fn grid_footer_names_the_keys_its_keymap_binds() {
    use junie_tui::core::keymap::KeyBinding;
    use junie_tui::widgets::grid::GridAction;
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let footer = |h: &H| h.text().lines().last().unwrap_or_default().to_owned();
    assert!(footer(&h).contains("s Sort"), "{}", footer(&h));
    match h.app.workbench.as_mut().unwrap().active_tab_mut() {
        Some(WorkTab::Table(t)) => {
            t.grid.keymap = vec![(KeyBinding::char('o'), GridAction::Sort)];
        }
        _ => unreachable!(),
    }
    h.draw();
    let text = footer(&h);
    assert!(
        text.contains("o Sort"),
        "the remapped key is advertised: {text}"
    );
    assert!(!text.contains("s Sort"), "{text}");
    assert!(
        !text.contains("Filter"),
        "an unbound action has no hint: {text}"
    );
    h.key(KeyCode::Char('o'));
    assert!(h.text().contains("sort "), "the remapped key sorts");
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
    assert!(matches!(h.app.modal(), Some(Modal::Filter(_))));
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
fn duplicate_row_chord_preserves_structure_editing_and_modal_routes() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let original = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t.grid.rows().len(),
        _ => unreachable!(),
    };
    h.alt('d');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t))
        if t.mode_tabs.active == 0 && t.grid.rows().len() == original + 1
            && t.grid.pending.inserted.len() == 1));
    h.ctrl('d');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t))
        if t.mode_tabs.active == 1 && t.grid.rows().len() == original + 1));
    h.ctrl('d');
    h.key(KeyCode::Home);
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.grid.is_editing()),
        "{}",
        h.text()
    );
    h.alt('d');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t))
        if t.grid.is_editing() && t.grid.rows().len() == original + 1));
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('?'));
    assert!(h.app.modal().is_some());
    h.alt('d');
    assert!(h.app.modal().is_some());
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t))
        if t.grid.rows().len() == original + 1));
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
fn explain_labels_end_before_the_metric_columns_and_shares_sit_under_their_header() {
    for (w, hgt) in [(100u16, 30u16), (72, 30)] {
        let mut h = H::connected(w, hgt);
        h.key(KeyCode::Tab);
        h.key(KeyCode::Char('i'));
        h.type_str("SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10");
        h.key(KeyCode::Esc);
        h.alt('x');
        h.ticks(10);
        let text = h.text();
        let lines: Vec<Vec<char>> = text.lines().map(|l| l.chars().collect()).collect();
        let (hy, header) = lines
            .iter()
            .enumerate()
            .find(|(_, l)| {
                let s: String = l.iter().collect();
                s.contains("Operation") && s.contains("actual")
            })
            .expect("plan header");
        let pct = header.iter().position(|c| *c == '%').expect("% header");
        // "cost" is right-aligned in 13 cells: the strip starts 9 cells before it
        let cost_x = (0..header.len())
            .find(|&x| header[x..].starts_with(&['c', 'o', 's', 't']))
            .unwrap();
        let strip = cost_x - 9;
        let mut plan_rows = 0;
        let mut cut = 0;
        for l in &lines[hy + 1..] {
            let s: String = l.iter().collect();
            if !s.contains("..") {
                continue;
            }
            plan_rows += 1;
            assert!(
                l[pct].is_ascii_digit(),
                "{w}x{hgt}: the share's last digit sits under %: {s:?}\n{text}"
            );
            assert_eq!(
                l[strip - 1],
                ' ',
                "{w}x{hgt}: a gap before the metrics: {s:?}"
            );
            let label: String = l[..strip].iter().collect();
            if label.trim_end().ends_with('…') {
                cut += 1;
            }
        }
        assert!(plan_rows >= 3, "{w}x{hgt}: plan rows shown\n{text}");
        if w < 100 {
            assert!(
                cut >= 1,
                "{w}x{hgt}: long labels are cut with … before the metrics\n{text}"
            );
        } else {
            assert!(
                text.contains("Parallel Seq Scan on public.orders "),
                "{w}x{hgt}: a label that fits is never painted over\n{text}"
            );
        }
    }
}

#[test]
fn safety_gate_intercepts_dangerous_statement_on_production() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(matches!(h.app.modal(), Some(Modal::Dialog(_))));
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
    assert!(h.app.modal().is_some(), "a wrong token never submits");
    assert!(h.text().contains("Does not match"), "{}", h.text());
    // Esc reverts the edit, a second Esc cancels the dialog
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none(), "{}", h.text());
    assert!(!h.wb_query().is_running());
    h.ctrl('r');
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
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
    assert!(
        h.app.modal().is_none(),
        "focus={:?}\n{}",
        h.focus(),
        h.text()
    );
    assert!(h.wb_query().is_running());
    h.ticks(10);
    assert!(h.text().contains("rows affected"));
}

#[test]
fn safety_gate_acknowledgement_supports_mouse_focus_edit_and_confirmation() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    let (ack_id, ack_area, confirm_area) = match h.app.modal() {
        Some(Modal::Dialog(d)) => match &d.body {
            junie_tui::widgets::dialog::DialogBody::Facts { ack: Some(a), .. } => {
                (a.input.id, a.input.area, d.actions[1].area)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    h.key(KeyCode::Tab); // Leave the acknowledgement before clicking it.
    // one click focuses the field and starts editing; a second click keeps it
    h.click(ack_area.x + 2, ack_area.y);
    assert_eq!(h.focus(), Some(ack_id));
    assert!(matches!(h.app.modal(), Some(Modal::Dialog(d)) if d.is_editing()));
    h.click(ack_area.x + 2, ack_area.y);
    assert!(matches!(h.app.modal(), Some(Modal::Dialog(d)) if d.is_editing()));
    h.type_str("wrong");
    h.click(confirm_area.x, confirm_area.y);
    assert!(h.app.modal().is_some());
    assert!(!h.wb_query().is_running());
    h.click(ack_area.x + 2, ack_area.y);
    h.click(ack_area.x + 2, ack_area.y);
    h.ctrl('l');
    h.type_str("orders");
    assert!(matches!(h.app.modal(), Some(Modal::Dialog(d)) if d.armed()));
    h.click(confirm_area.x, confirm_area.y);
    assert!(h.app.modal().is_none());
    assert!(h.wb_query().is_running());
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
    assert!(h.app.modal().is_none());
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
        h.app.modal().is_none(),
        "UPDATE without WHERE is a plain write in TablePro"
    );
    h.ticks(10);
    h.key(KeyCode::Char('i'));
    h.ctrl('l');
    h.type_str("TRUNCATE orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(
        matches!(h.app.modal(), Some(Modal::Dialog(_))),
        "destructive always confirms"
    );
    assert!(h.text().contains("TRUNCATE"));
}

#[test]
fn quick_switcher_details_are_not_cut_beside_unused_label_cells() {
    let mut h = H::connected(72, 20);
    h.ctrl('o');
    let t = h.text();
    let row = t
        .lines()
        .find(|l| l.contains("login_attempts"))
        .unwrap_or_else(|| panic!("a table row\n{t}"));
    // the schema detail fits once the label column gives back the cells
    // its short labels do not use
    assert!(
        row.contains("audit") && !row.contains("audit ·…"),
        "the detail is not cut while the label column has room: {row:?}\n{t}"
    );
}

#[test]
fn quick_switcher_opens_table() {
    let mut h = H::connected(120, 40);
    h.ctrl('o');
    assert!(matches!(h.app.modal(), Some(Modal::Picker(..))));
    h.type_str("cust");
    assert!(h.text().contains("customers"));
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers"));
    // Esc clears then closes
    h.ctrl('o');
    h.type_str("x");
    h.key(KeyCode::Esc);
    assert!(matches!(h.app.modal(), Some(Modal::Picker(..))));
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
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
    assert!(matches!(h.app.modal(), Some(Modal::Picker(..))));
    assert!(h.text().contains("Open tabs"));
    h.key(KeyCode::Up);
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
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
    assert!(h.app.pointer.hover.is_some());
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

#[test]
fn narrow_terminals_turn_the_explorer_into_a_drawer() {
    let mut h = H::connected(80, 24);
    // the explorer has focus after connecting, so the drawer is open
    assert!(h.text().contains("Filter objects"));
    assert!(!h.text().contains("Type SQL"));
    // Tab leaves the drawer and lands in the query editor
    h.key(KeyCode::Tab);
    assert!(!h.text().contains("Filter objects"));
    assert_eq!(h.focus(), Some(h.wb_query().editor.id));
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT 1");
    assert!(h.wb_query().editor.text().contains("SELECT 1"));
    h.key(KeyCode::Esc);
    // `0` reopens the drawer; opening a table closes it again
    h.key(KeyCode::Char('0'));
    assert!(h.text().contains("Filter objects"));
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(!h.text().contains("Filter objects"));
    assert!(h.text().contains("public › orders"));
}

/// The end-to-end flow from GOAL2, keyboard only.
#[test]
fn acceptance_flow_keyboard_only() {
    let mut h = H::new(120, 40);
    // connect to Production
    for _ in 0..8 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(14);
    assert_eq!(h.app.screen, Screen::Workbench);
    assert_eq!(h.wb().connection.name, "Production");
    // explorer → orders
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    // grid: sort by created_at, then filter status = pending
    for _ in 0..12 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Char('s'));
    assert!(h.text().contains("sort created_at ▴"));
    h.key(KeyCode::Home);
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Char('f'));
    assert!(matches!(h.app.modal(), Some(Modal::Filter(_))));
    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("pending");
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("status = 'pending'"));
    assert!(h.text().contains("filtered (1)"));
    // structure and back
    h.ctrl('d');
    assert!(h.text().contains("Foreign keys"));
    h.ctrl('d');
    // a new query with completion, run, result tab
    h.ctrl('t');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));
    assert_eq!(
        h.focus(),
        Some(h.wb_query().editor.id),
        "new query focuses its editor"
    );
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM ord");
    assert!(h.wb_query().completion.is_open());
    h.key(KeyCode::Enter);
    assert!(h.wb_query().editor.text().ends_with("orders"));
    h.type_str(" WHERE status = 'pending' LIMIT 25");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    h.ticks(10);
    assert!(h.text().contains("SELECT orders (25)"));
    // EXPLAIN as a second result tab (pin the first so it survives)
    h.key(KeyCode::Tab); // result tabs
    h.key(KeyCode::Char('p'));
    h.key(KeyCode::BackTab);
    h.alt('x');
    h.ticks(10);
    assert!(h.text().contains("EXPLAIN ANALYZE"));
    assert!(h.text().contains("Seq Scan") || h.text().contains("Index Scan"));
    assert_eq!(
        h.wb_query().results.len(),
        2,
        "pinned result kept beside the plan"
    );
    // history reopens a query
    h.ctrl('y');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::History(_))));
    h.key(KeyCode::Enter);
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Query(q)) if q.editor.text().contains("status = 'pending' LIMIT 25")),
        "the newest entry is the query just run"
    );
    // quick switcher
    h.ctrl('o');
    h.type_str("cust");
    h.key(KeyCode::Enter);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers"));
    // a dangerous query is intercepted and cancelled
    h.ctrl('t');
    h.key(KeyCode::Char('i'));
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(matches!(h.app.modal(), Some(Modal::Dialog(_))));
    assert!(h.text().contains("Type orders to confirm"));
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(!h.wb_query().is_running());
    assert!(h.text().contains("Cancelled · nothing was executed"));
}

/// The same journey driven by the mouse where the mouse has a role.
#[test]
fn acceptance_flow_mouse() {
    let mut h = H::new(120, 40);
    let (x, y) = h.find("Production").expect("connection row");
    h.click(x, y);
    let (x, y) = h.find("Connect ").expect("connect button");
    h.click(x + 1, y);
    h.ticks(14);
    assert_eq!(h.app.screen, Screen::Workbench);
    // explorer row → table tab
    let (x, y) = h.find("T orders").expect("orders row");
    h.click(x + 2, y);
    h.ticks(3);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    // header click sorts
    let (x, y) = h.find("order_number").expect("header");
    h.click(x, y);
    assert!(h.text().contains("sort order_number ▴"));
    // mode tab
    let (x, y) = h.find("Structure").expect("mode tab");
    h.click(x, y);
    assert!(h.text().contains("Foreign keys"));
    // `+` opens a new query, clicking the editor focuses it
    let (x, y) = h.find("+").expect("new tab button");
    h.click(x, y);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));
    let (x, y) = h.find("Type SQL").expect("editor placeholder");
    h.click(x, y);
    assert_eq!(h.focus(), Some(h.wb_query().editor.id));
    // wheel over the explorer scrolls it, not the editor
    let (ex, ey) = h.find("Filter objects").expect("explorer");
    let before = h.wb().explorer.scroll.offset;
    h.mouse(MouseKind::WheelDown, ex, ey + 3);
    assert!(h.wb().explorer.scroll.offset >= before);
    assert_eq!(
        h.focus(),
        Some(h.wb_query().editor.id),
        "wheel does not move focus"
    );
}

#[test]
fn cli_parser_accepts_color_connection_and_help_contracts() {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    let cli =
        crate::Cli::try_parse_from(["tablepro", "--color", "mono", "--connect", "Production"])
            .expect("valid CLI options");
    assert!(matches!(cli.color, Some(crate::ColorArg::Mono)));
    assert_eq!(cli.connect.as_deref(), Some("Production"));

    let defaults = crate::Cli::try_parse_from(["tablepro"]).expect("default CLI options");
    assert!(defaults.color.is_none());
    assert!(defaults.connect.is_none());
    assert!(crate::Cli::try_parse_from(["tablepro", "--color"]).is_err());

    for color in ["truecolor", "24bit", "256", "16", "none", "mono"] {
        assert!(
            crate::Cli::try_parse_from(["tablepro", "--color", color]).is_ok(),
            "color value {color:?}"
        );
    }
    let short = crate::Cli::try_parse_from(["tablepro", "-c", "24bit"]).unwrap();
    assert!(matches!(short.color, Some(crate::ColorArg::TrueColor)));

    assert!(matches!(
        crate::Cli::try_parse_from(["tablepro", "--color", "invalid"]),
        Err(error) if error.kind() == ErrorKind::InvalidValue
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["tablepro", "--connect"]),
        Err(error) if error.kind() == ErrorKind::InvalidValue
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["tablepro", "--ignored"]),
        Err(error) if error.kind() == ErrorKind::UnknownArgument
    ));
    let help = crate::Cli::command()
        .after_help(crate::cli_after_help())
        .render_help()
        .to_string();
    assert!(help.contains("Ctrl+O open quickly"));
    assert!(help.contains("Ctrl+Y history"));
    assert!(matches!(
        crate::Cli::try_parse_from(["tablepro", "--help"]),
        Err(error) if error.kind() == ErrorKind::DisplayHelp
    ));

    let connections = crate::db::connections();
    let production = crate::connection_index(&connections, "production").unwrap();
    assert_eq!(connections[production].name, "Production");
    let error = crate::connection_index(&connections, "not-configured").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::ValueValidation);
    assert!(error.to_string().contains("no connection named"));
}

#[test]
fn clicking_the_editor_closes_completion_and_places_the_caret() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab); // editor
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM ord");
    assert!(h.wb_query().completion.is_open());
    let before = h.wb_query().editor.cursor_offset();
    let popup = h.wb_query().completion.area;
    let caret = h.wb_query().editor.cursor_cell().expect("caret on screen");
    // a click on the same line, left of the caret and outside the popup
    let x = caret.x.saturating_sub(5);
    assert!(!popup.contains(ratatui::layout::Position::new(x, caret.y)));
    h.click(x, caret.y);
    assert!(
        !h.wb_query().completion.is_open(),
        "outside click closed the list"
    );
    assert!(
        h.wb_query().editor.cursor_offset() < before,
        "the same click moved the caret"
    );
}

fn footer(h: &H) -> String {
    h.text().lines().last().unwrap_or_default().to_owned()
}

#[test]
fn dialog_footers_name_the_focused_button_and_the_typed_token() {
    // the delete dialog opens on Cancel: Enter cancels, so the footer says so
    let mut h = H::new(120, 40);
    h.key(KeyCode::Char('d'));
    assert!(
        matches!(h.app.modal(), Some(Modal::Dialog(_))),
        "{}",
        h.text()
    );
    let f = footer(&h);
    assert!(f.contains("Enter Cancel"), "{f}");
    assert!(!f.contains("y Delete"), "no one-key Delete: {f}");
    assert!(!f.contains("Confirm"), "{f}");

    // a typed acknowledgement says what to type, not "Enter Confirm"
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    let f = footer(&h);
    assert!(f.contains("Type \"orders\" to confirm"), "{f}");
    // Enter presses the gated action once armed; until then it only
    // says the text does not match, so the footer offers no Enter
    assert!(!f.contains("Enter Next"), "{f}");
    // the Safe Mode fact says what to do, without jargon
    let t = h.text();
    assert!(t.contains("type to confirm"), "{t}");
    assert!(!t.contains("deliberate"), "{t}");
}

#[test]
fn the_switcher_is_the_only_layer_showing_focus() {
    let mut h = H::connected(120, 40);
    let cues = |h: &H| {
        let t = h.app.theme;
        let buf = h.term.backend().buffer();
        let bars = buf.content().iter().filter(|c| c.symbol() == "▎").count();
        let tint = buf.content().iter().filter(|c| c.bg == t.accent_bg).count();
        (bars, tint)
    };
    h.ctrl('o');
    assert!(matches!(h.app.modal(), Some(Modal::Picker(..))));
    assert_eq!(cues(&h), (1, 0), "only the query field\n{}", h.text());
    h.key(KeyCode::Esc);
    h.ctrl('l');
    assert!(h.text().contains("Safe Mode · this connection"));
    // a fixed-choice picker marks its cursor with a fill, like a menu
    assert_eq!(cues(&h), (0, 0), "{}", h.text());
}

#[test]
fn cell_edit_footer_says_apply_not_commit() {
    // "Commit" collides with the database commit behind Ctrl+S
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter); // orders
    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    let f = footer(&h);
    assert!(f.contains("Enter Apply"), "{f}");
    assert!(f.contains("Esc Cancel"), "{f}");
    assert!(!f.contains("Commit"), "{f}");
}

/// The keyboard reference wraps its descriptions under their own column,
/// never back to the dialog's left edge (layout audit #11).
#[test]
fn help_descriptions_hang_under_their_column_when_they_wrap() {
    let mut h = H::connected(72, 40);
    h.key(KeyCode::Char('?'));
    let text = h.text();
    let lines: Vec<&str> = text.lines().collect();
    let first = lines
        .iter()
        .position(|l| l.contains("Ctrl+D") && l.contains("Data / Structure"))
        .unwrap_or_else(|| panic!("help shows Ctrl+D:\n{text}"));
    let desc_col = lines[first].find("Data").unwrap();
    let next = lines[first + 1];
    let cont_col = next
        .find(|c: char| c.is_alphanumeric() || c == '(')
        .unwrap();
    assert_eq!(
        cont_col, desc_col,
        "the wrapped part of Ctrl+D's description hangs under it:\n{text}"
    );
}

#[test]
fn hover_follows_the_rows_that_scroll_under_a_still_pointer() {
    // interaction #22: after a wheel scroll the hovered row is the one now
    // under the pointer, not the one that scrolled away
    let mut h = H::connected(120, 20);
    let (x, y) = h
        .find("customers")
        .unwrap_or_else(|| panic!("{}", h.text()));
    h.mouse(MouseKind::Move, x, y);
    let before = h.app.pointer.hover;
    assert!(before.is_some());
    for _ in 0..4 {
        h.mouse(MouseKind::WheelDown, x, y);
    }
    let now = h.app.hits.hit(Position::new(x, y));
    assert_ne!(now, before, "the tree did not scroll\n{}", h.text());
    assert_eq!(h.app.pointer.hover, now);
}

/// Layout audit #1: on a short terminal the connection form stacked its
/// fields with `y += h`, so labels overwrote the hint bar and the button
/// row landed on "Database". Every field now stays above the buttons, the
/// focused one is scrolled into view, and the footer is untouched.
#[test]
fn a_short_connection_form_keeps_fields_above_the_buttons_and_scrolls_to_focus() {
    let form = WidgetId::of("connections").sub("form");
    // only the fields: the tab strip, the buttons and the header show
    // focus their own way
    let fields: Vec<WidgetId> = [
        "name", "engine", "host", "port", "db", "user", "pw", "promptpw", "env", "group", "safe",
        "ssl", "ssh", "sshhost", "sshuser", "local", "startup",
    ]
    .iter()
    .map(|n| form.sub(n))
    .collect();
    for (w, hh) in [(72u16, 20u16), (80, 24)] {
        for advanced in [false, true] {
            let mut h = H::new(w, hh);
            h.ctrl('n');
            if advanced {
                h.tab_to(form.sub("tabs"));
                h.key(KeyCode::Right);
            }
            let buttons = h.app.hits.area_of(form.sub("test")).expect("buttons drawn");
            let footer = hh - 1;
            let mut seen = std::collections::HashSet::new();
            for _ in 0..40 {
                let Some(f) = h.focus() else { break };
                if !seen.insert(f) {
                    break;
                }
                if fields.contains(&f) {
                    if let Some(a) = h.app.hits.area_of(f) {
                        assert!(
                            a.bottom() <= buttons.y,
                            "{w}x{hh} advanced={advanced}: {f:?} at {a:?} runs into the buttons at {buttons:?}\n{}",
                            h.text()
                        );
                    }
                    // the single focus gutter is on screen above the
                    // buttons: the focused field was scrolled into view
                    let text = h.text();
                    let gutter = text.lines().position(|l| l.contains('▎'));
                    assert!(
                        gutter.is_some_and(|y| (y as u16) < buttons.y),
                        "{w}x{hh} advanced={advanced}: focused {f:?} is not in view\n{text}"
                    );
                }
                h.key(KeyCode::Tab);
            }
            // nothing from the form is drawn on the hint row
            let last = h.text().lines().nth(footer as usize).unwrap().to_owned();
            for label in [
                "Database",
                "Username",
                "Password",
                "Prompt",
                "SSH user",
                "Local only",
            ] {
                assert!(!last.contains(label), "{w}x{hh}: {last}");
            }
        }
    }
}

#[test]
fn the_wheel_scrolls_a_short_connection_form_by_whole_fields() {
    let mut h = H::new(72, 20);
    h.ctrl('n');
    let (x, y) = h.find("Name *").expect("the form opens at its first field");
    // the label belongs to its field (label_hit); the form scrolls
    // beneath both
    assert!(
        h.app
            .hits
            .scroll_stack(Position::new(x, y))
            .contains(&WidgetId::of("connections").sub("form").sub("left"))
    );
    h.mouse(MouseKind::WheelDown, x, y);
    let text = h.text();
    assert!(!text.contains("Name *"), "{text}");
    assert!(text.contains("Engine"), "one field per notch: {text}");
    h.mouse(MouseKind::WheelUp, x, y);
    assert!(h.text().contains("Name *"));
}

#[test]
fn a_click_marks_focus_as_pointer_focus_until_the_next_key() {
    // interaction #24: KeyboardOnly controls hide their ring after a click
    let mut h = H::connected(120, 40);
    let (x, y) = h.find("customers").unwrap();
    h.click(x, y);
    assert!(h.app.pointer.pointer_focus, "focus came from the click");
    h.key(KeyCode::Down);
    assert!(!h.app.pointer.pointer_focus, "a key shows focus again");
}

/// Mouse audit #11: the wheel over an overflowing tab strip scrolls it (it
/// did nothing), the active tab stays put, and a later sync keeps the
/// scrolled window.
#[test]
fn the_wheel_over_the_tab_strip_scrolls_it() {
    let mut h = H::connected(80, 24);
    {
        let wb = h.app.workbench.as_mut().unwrap();
        for _ in 0..12 {
            wb.new_query("");
        }
        wb.set_active(0);
    }
    h.draw();
    let strip = h
        .app
        .hits
        .area_of(crate::workbench::TABSTRIP)
        .unwrap_or_else(|| panic!("{}", h.text()));
    assert!(
        h.wb().strip.hidden().1 > 0,
        "the strip overflows\n{}",
        h.text()
    );
    assert_eq!(h.wb().strip.first, 0);
    h.mouse(MouseKind::WheelDown, strip.x + 6, strip.y);
    assert_eq!(h.wb().strip.first, 1, "a vertical notch scrolls one tab");
    h.mouse(MouseKind::WheelRight, strip.x + 6, strip.y);
    assert_eq!(h.wb().strip.first, 2, "a horizontal notch too");
    assert_eq!(h.wb().active, 0, "the wheel does not switch tabs");
}

/// Mouse audit #11: the `+N` button of an overflowing strip opens a menu
/// of the hidden tabs; clicking one activates it, Esc closes the menu.
#[test]
fn the_overflow_button_lists_hidden_tabs_and_a_click_activates_one() {
    let mut h = H::connected(80, 24);
    {
        let wb = h.app.workbench.as_mut().unwrap();
        for _ in 0..12 {
            wb.new_query("");
        }
        wb.set_active(0);
    }
    h.draw();
    let more = h
        .app
        .hits
        .area_of(h.wb().strip.more_id())
        .unwrap_or_else(|| panic!("no +N button\n{}", h.text()));
    h.click(more.x + 1, more.y);
    assert!(h.wb().strip.menu_open(), "{}", h.text());
    let last = h.wb().tabs.len() - 1;
    let label = h.wb().strip.items[last].label.clone();
    let text = h.text();
    let (x, y) = text
        .lines()
        .enumerate()
        .skip(more.y as usize + 1)
        .filter_map(|(y, line)| {
            let col = line.find(&label)?;
            Some((line[..col].chars().count() as u16, y as u16))
        })
        .last()
        .unwrap_or_else(|| panic!("{label} not listed\n{text}"));
    h.click(x, y);
    assert!(!h.wb().strip.menu_open());
    assert_eq!(
        h.wb().active,
        last,
        "the chosen tab is active\n{}",
        h.text()
    );
    // Esc closes a reopened menu without switching
    let more = h.app.hits.area_of(h.wb().strip.more_id()).unwrap();
    h.click(more.x + 1, more.y);
    assert!(h.wb().strip.menu_open());
    h.key(KeyCode::Esc);
    assert!(!h.wb().strip.menu_open());
    assert_eq!(h.wb().active, last);
}
