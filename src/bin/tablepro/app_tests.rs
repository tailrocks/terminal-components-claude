use crate::filter_editor::FilterEditor;
use junie_tui::runtime::Application;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::picker::Picker;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

use crate::app::{App, Route};
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

    pub fn cmd(&mut self, c: char) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::SUPER,
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

            let in_editor = matches!(
                self.app.workbench.as_ref().and_then(|w| w.active_tab()),
                Some(WorkTab::Query(q)) if Some(q.editor.id) == self.focus() && q.editor.editing
            );
            if in_editor {
                self.key(KeyCode::Esc);
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
        self.app.shell.focus.current()
    }
    pub fn wb(&self) -> &crate::workbench::Workbench {
        self.app.workbench.as_ref().unwrap()
    }
}

#[test]
fn connections_screen_lists_and_connects_with_keyboard() {
    let mut h = H::new(120, 40);
    assert_eq!(h.app.route(), Route::Connections);
    assert!(h.text().contains("Local PostgreSQL"));
    assert!(h.text().contains("Production"));

    for _ in 0..8 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Safe Mode"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Opening SSH tunnel") || h.text().contains("Authenticating"));
    h.ticks(14);
    assert_eq!(h.app.route(), Route::Workbench);
    assert_eq!(h.wb().connection.name, "Production");
    assert!(h.text().contains("production"));
    assert!(
        h.text().contains("safe"),
        "safe mode token visible in the strip"
    );
}

#[test]
fn a_double_click_on_a_connection_connects_and_one_click_selects() {
    use crate::connections::ConnState;
    let mut h = H::new(120, 40);
    let (x, y) = h.find("Production").expect("the connection row");
    h.click(x, y);

    h.app.shell.pointer.reset_clicks();
    h.click(x, y);
    assert!(
        matches!(h.app.connections.state, ConnState::Idle),
        "a slow second click only selects"
    );
    assert!(h.text().contains("Safe Mode"), "the row is selected");
    h.app.shell.pointer.reset_clicks();
    h.click(x, y);
    h.click(x, y);
    assert!(
        matches!(h.app.connections.state, ConnState::Connecting { .. }),
        "the double click connects"
    );
    h.ticks(14);
    assert_eq!(h.app.route(), Route::Workbench);
    assert_eq!(h.wb().connection.name, "Production");
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
    assert_eq!(h.app.route(), Route::Connections);
    assert!(h.text().contains("Could not reach the host"));
    assert!(h.text().contains("Reconnect"));
}

#[test]
fn explorer_opens_table_and_grid_navigates() {
    let mut h = H::connected(120, 40);
    assert!(h.text().contains("organizations"));

    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let wb = h.wb();
    assert!(matches!(wb.active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));
    assert!(h.text().contains("public › orders"));
    assert!(h.text().contains("order_number"));

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
    assert!(g.scroll.h.offset > 0, "wide table scrolled horizontally");
    assert!(h.text().contains("cols "));
}

#[test]
fn grid_footer_names_the_keys_its_keymap_binds() {
    use junie_tui::core::keymap::KeyBinding;
    use junie_tui::widgets::grid::GridCmd;
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let footer = |h: &H| h.text().lines().last().unwrap_or_default().to_owned();
    assert!(footer(&h).contains("s Sort"), "{}", footer(&h));
    match h.app.workbench.as_mut().unwrap().active_tab_mut() {
        Some(WorkTab::Table(t)) => {
            t.grid.keymap = vec![(KeyBinding::char('o'), GridCmd::Sort)];
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

    h.key(KeyCode::Home);
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Char('f'));
    assert!(h.app.top_as::<FilterEditor>().is_some());

    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);

    assert!(h.term.backend().cursor_visible());
    h.type_str("pending");
    h.key(KeyCode::Enter);
    let t = match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => t,
        _ => unreachable!(),
    };
    assert_eq!(t.filters.len(), 1);
    assert_eq!(t.filters[0].value, "pending");
    assert!(h.text().contains("filtered (1)"), "{}", h.text());
    assert!(h.text().contains("status = 'pending'"));

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
    h.key(KeyCode::Tab);
    let q = match h.wb().active_tab() {
        Some(WorkTab::Query(q)) => q,
        _ => unreachable!(),
    };
    assert_eq!(h.focus(), Some(q.editor.id));
    h.type_str("SELECT * FROM ord");
    assert!(
        h.wb_query().completion.is_open(),
        "autocomplete opened after FROM + prefix"
    );

    assert!(
        h.text().matches("order_items").count() >= 2,
        "completion popup is drawn"
    );
    h.key(KeyCode::Enter);
    assert!(h.wb_query().editor.text().ends_with("orders"));
    h.type_str(" WHERE st");
    assert!(h.wb_query().completion.is_open());
    assert_eq!(
        h.wb_query().completion.current().map(|c| c.label.as_str()),
        Some("status")
    );
    h.key(KeyCode::Tab);
    h.type_str(" = 'pending' ORDER BY created_at DESC LIMIT 25");
    h.key(KeyCode::Esc);
    assert!(!h.wb_query().editor.editing);
    h.ctrl('r');
    assert!(h.wb_query().is_running());
    assert!(h.text().contains("running"));
    h.ticks(10);
    assert!(!h.wb_query().is_running());
    assert_eq!(h.wb_query().results.len(), 1);
    assert!(h.text().contains("SELECT orders (25)"));
    assert!(h.text().contains("25 rows"));

    assert!(h.app.history.entries[0].sql.contains("LIMIT 25"));
}

#[test]
fn execution_error_marks_editor_and_result() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
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
    h.type_str("SELECT * FROM orders WHERE notes LIKE '%gift%' ORDER BY created_at LIMIT 10");
    h.key(KeyCode::Esc);
    h.alt('x');
    h.ticks(10);
    assert!(h.text().contains("EXPLAIN ANALYZE"));
    assert!(h.text().contains("Limit"));
    assert!(h.text().contains("Sort"));
    assert!(h.text().contains("Seq Scan"));

    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    let before = h.text();
    h.key(KeyCode::Left);
    assert_ne!(before, h.text());

    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("cost="));
}

#[test]
fn explain_labels_end_before_the_metric_columns_and_shares_sit_under_their_header() {
    for (w, hgt) in [(100u16, 30u16), (72, 30)] {
        let mut h = H::connected(w, hgt);
        h.key(KeyCode::Tab);
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
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(h.app.top_as::<Dialog>().is_some());
    let t = h.text();
    assert!(t.contains("DELETE without WHERE"));
    assert!(t.contains("every row in orders"));
    assert!(t.contains("Production"));
    assert!(t.contains("Type orders to confirm"));

    h.key(KeyCode::Enter);
    h.type_str("wrong");
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_some(), "a wrong token never submits");
    assert!(h.text().contains("Does not match"), "{}", h.text());

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
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');

    assert!(h.text().contains("Type orders to confirm"));

    assert!(h.term.backend().cursor_visible());
    h.type_str("orders");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
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
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    let (ack_id, ack_area, confirm_area) = match h.app.top_as::<Dialog>() {
        Some(d) => match &d.body {
            junie_tui::widgets::dialog::DialogBody::Facts { ack: Some(a), .. } => {
                (a.input.id, a.input.area, d.actions[1].area)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    h.key(KeyCode::Tab);

    h.click(ack_area.x + 2, ack_area.y);
    assert_eq!(h.focus(), Some(ack_id));
    assert!(h.app.top_as::<Dialog>().is_some_and(|d| d.is_editing()));
    h.click(ack_area.x + 2, ack_area.y);
    assert!(h.app.top_as::<Dialog>().is_some_and(|d| d.is_editing()));
    h.type_str("wrong");
    h.click(confirm_area.x, confirm_area.y);
    assert!(h.app.modal().is_some());
    assert!(!h.wb_query().is_running());
    h.click(ack_area.x + 2, ack_area.y);
    h.click(ack_area.x + 2, ack_area.y);
    h.cmd('a');
    h.type_str("orders");
    assert!(h.app.top_as::<Dialog>().is_some_and(|d| d.armed()));
    h.click(confirm_area.x, confirm_area.y);
    assert!(h.app.modal().is_none());
    assert!(h.wb_query().is_running());
}

#[test]
fn read_only_connection_refuses_writes() {
    let mut h = H::connected(120, 40);
    h.app.workbench.as_mut().unwrap().connection.safe_mode = crate::db::SafeMode::ReadOnly;
    h.key(KeyCode::Tab);
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
    h.type_str("UPDATE orders SET status = 'paid'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(
        h.app.modal().is_none(),
        "UPDATE without WHERE is a plain write in TablePro"
    );
    h.ticks(10);
    h.key(KeyCode::Char('i'));
    h.cmd('a');
    h.type_str("TRUNCATE orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(
        h.app.top_as::<Dialog>().is_some(),
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

    assert!(
        row.contains("audit") && !row.contains("audit ·…"),
        "the detail is not cut while the label column has room: {row:?}\n{t}"
    );
}

#[test]
fn quick_switcher_opens_table() {
    let mut h = H::connected(120, 40);
    h.ctrl('o');
    assert!(h.app.top_as::<Picker>().is_some());
    h.type_str("cust");
    assert!(h.text().contains("customers"));
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers"));

    h.ctrl('o');
    h.type_str("x");
    h.key(KeyCode::Esc);
    assert!(h.app.top_as::<Picker>().is_some());
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
    assert!(h.app.top_as::<Picker>().is_some());
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
    h.key(KeyCode::Enter);

    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.cmd('a');
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
    h.key(KeyCode::Down);
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

    h.click(x, y);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if !t.preview));

    let (qx, qy) = h.find("Query 1").unwrap();
    h.click(qx, qy);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));

    let (cx, cy) = h.find("customers").unwrap();
    h.mouse(MouseKind::Move, cx + 30, cy);
    h.mouse(MouseKind::Move, cx, cy);
    assert!(h.app.shell.pointer.hover.is_some());
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
        assert!(!h.app.shell.quit);
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

    assert!(h.text().contains("Filter objects"));
    assert!(!h.text().contains("Type SQL"));

    h.key(KeyCode::Tab);
    assert!(!h.text().contains("Filter objects"));
    assert_eq!(h.focus(), Some(h.wb_query().editor.id));
    h.type_str("SELECT 1");
    assert!(h.wb_query().editor.text().contains("SELECT 1"));
    h.key(KeyCode::Esc);

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

#[test]
fn a_vanished_control_hands_focus_to_its_nearest_neighbour() {
    let mut h = H::new(120, 30);
    let del = junie_tui::core::id::WidgetId::of("connections").sub("del");
    h.tab_to(del);
    assert_eq!(h.focus(), Some(del));
    h.resize(76, 24);
    h.draw();
    assert_eq!(h.focus(), Some(crate::connections::TREE));
    assert_ne!(
        h.focus(),
        h.app.shell.ring.first(),
        "the First fallback lands on the ring's first control"
    );
}

#[test]
fn a_form_covering_the_narrow_connection_list_keeps_tab_inside_it() {
    let mut h = H::new(76, 24);
    h.ctrl('n');
    h.draw();
    let list = [
        crate::connections::TREE,
        junie_tui::core::id::WidgetId::of("connections").sub("filter"),
    ];
    let mut seen = vec![];
    for _ in 0..40 {
        h.key(KeyCode::Tab);
        seen.push(h.focus());
    }
    assert!(
        seen.iter().all(|f| f.is_some_and(|f| !list.contains(&f))),
        "Tab reached the covered list: {seen:?}"
    );
}

#[test]
fn acceptance_flow_keyboard_only() {
    let mut h = H::new(120, 40);

    for _ in 0..8 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(14);
    assert_eq!(h.app.route(), Route::Workbench);
    assert_eq!(h.wb().connection.name, "Production");

    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));

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
    assert!(h.app.top_as::<FilterEditor>().is_some());
    h.key(KeyCode::BackTab);
    h.key(KeyCode::BackTab);

    assert!(h.term.backend().cursor_visible());
    h.type_str("pending");
    h.key(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    assert!(h.text().contains("status = 'pending'"));
    assert!(h.text().contains("filtered (1)"));

    h.ctrl('d');
    assert!(h.text().contains("Foreign keys"));
    h.ctrl('d');

    h.ctrl('t');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));
    assert_eq!(
        h.focus(),
        Some(h.wb_query().editor.id),
        "new query focuses its editor"
    );
    h.type_str("SELECT * FROM ord");
    assert!(h.wb_query().completion.is_open());
    h.key(KeyCode::Enter);
    assert!(h.wb_query().editor.text().ends_with("orders"));
    h.type_str(" WHERE status = 'pending' LIMIT 25");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    h.ticks(10);
    assert!(h.text().contains("SELECT orders (25)"));

    h.key(KeyCode::Tab);
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

    h.ctrl('y');
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::History(_))));
    h.key(KeyCode::Enter);
    assert!(
        matches!(h.wb().active_tab(), Some(WorkTab::Query(q)) if q.editor.text().contains("status = 'pending' LIMIT 25")),
        "the newest entry is the query just run"
    );

    h.ctrl('o');
    h.type_str("cust");
    h.key(KeyCode::Enter);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "customers"));

    h.ctrl('t');
    h.type_str("DELETE FROM orders");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    assert!(h.app.top_as::<Dialog>().is_some());
    assert!(h.text().contains("Type orders to confirm"));
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(!h.wb_query().is_running());
    assert!(h.text().contains("Cancelled · nothing was executed"));
}

#[test]
fn acceptance_flow_mouse() {
    let mut h = H::new(120, 40);
    let (x, y) = h.find("Production").expect("connection row");
    h.click(x, y);
    let (x, y) = h.find("Connect ").expect("connect button");
    h.click(x + 1, y);
    h.ticks(14);
    assert_eq!(h.app.route(), Route::Workbench);

    let (x, y) = h.find("T orders").expect("orders row");
    h.click(x + 2, y);
    h.ticks(3);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Table(t)) if t.name == "orders"));

    let (x, y) = h.find("order_number").expect("header");
    h.click(x, y);
    assert!(h.text().contains("sort order_number ▴"));

    let (x, y) = h.find("Structure").expect("mode tab");
    h.click(x, y);
    assert!(h.text().contains("Foreign keys"));

    let (x, y) = h.find("+").expect("new tab button");
    h.click(x, y);
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));
    let (x, y) = h.find("Type SQL").expect("editor placeholder");
    h.click(x, y);
    assert_eq!(h.focus(), Some(h.wb_query().editor.id));

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
    assert!(matches!(
        cli.common.color,
        Some(junie_tui::cli::ColorArg::Mono)
    ));
    assert_eq!(cli.connect.as_deref(), Some("Production"));

    let defaults = crate::Cli::try_parse_from(["tablepro"]).expect("default CLI options");
    assert!(defaults.common.color.is_none());
    assert!(defaults.connect.is_none());
    assert!(crate::Cli::try_parse_from(["tablepro", "--color"]).is_err());

    for color in ["truecolor", "24bit", "256", "16", "none", "mono"] {
        assert!(
            crate::Cli::try_parse_from(["tablepro", "--color", color]).is_ok(),
            "color value {color:?}"
        );
    }
    let short = crate::Cli::try_parse_from(["tablepro", "-c", "24bit"]).unwrap();
    assert!(matches!(
        short.common.color,
        Some(junie_tui::cli::ColorArg::TrueColor)
    ));

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
    h.key(KeyCode::Tab);
    h.type_str("SELECT * FROM ord");
    assert!(h.wb_query().completion.is_open());
    let before = h.wb_query().editor.cursor_offset();
    let popup = h.wb_query().completion.area;
    let caret = h.wb_query().editor.cursor_cell().expect("caret on screen");

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
    let mut h = H::new(120, 40);
    h.key(KeyCode::Char('d'));
    assert!(h.app.top_as::<Dialog>().is_some(), "{}", h.text());
    let f = footer(&h);
    assert!(f.contains("Enter Cancel"), "{f}");
    assert!(!f.contains("y Delete"), "no one-key Delete: {f}");
    assert!(!f.contains("Confirm"), "{f}");

    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    h.type_str("UPDATE orders SET status = 'paid' WHERE id = 'x'");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    let f = footer(&h);
    assert!(f.contains("Type \"orders\" to confirm"), "{f}");

    assert!(!f.contains("Enter Next"), "{f}");

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
    assert!(h.app.top_as::<Picker>().is_some());
    assert_eq!(cues(&h), (1, 0), "only the query field\n{}", h.text());
    h.key(KeyCode::Esc);
    h.ctrl('l');
    assert!(h.text().contains("Safe Mode · this connection"));

    assert_eq!(cues(&h), (0, 0), "{}", h.text());
}

fn active_rule(h: &H, strip: &junie_tui::widgets::tabs::Tabs) -> ratatui::style::Color {
    let a = strip.areas[strip.active];
    let cell = &h.term.backend().buffer()[(a.x + 1, a.y + 1)];
    assert_eq!(cell.symbol(), "━", "{}", h.text());
    cell.fg
}

#[test]
fn the_active_query_tab_shows_the_accent_and_goes_neutral_under_a_modal() {
    let mut h = H::connected(120, 40);
    let t = h.app.theme;
    assert!(matches!(h.wb().active_tab(), Some(WorkTab::Query(_))));

    assert_ne!(h.focus(), Some(h.wb().strip.id));
    assert_eq!(active_rule(&h, &h.wb().strip), t.accent);

    h.ctrl('o');
    assert!(h.app.top_as::<Picker>().is_some());
    let under = active_rule(&h, &h.wb().strip);
    assert_ne!(under, t.accent, "{}", h.text());
    assert_ne!(under, t.focus);

    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(active_rule(&h, &h.wb().strip), t.accent);
}

#[test]
fn a_table_tab_shows_its_chosen_mode_in_the_accent() {
    let mut h = H::connected(120, 40);
    let t = h.app.theme;
    let (x, y) = h.find("customers").unwrap();
    h.click(x, y);
    let Some(WorkTab::Table(tab)) = h.wb().active_tab() else {
        panic!("a table tab: {}", h.text());
    };

    assert_eq!(active_rule(&h, &tab.mode_tabs), t.accent);
}

#[test]
fn cell_edit_footer_says_apply_not_commit() {
    let mut h = H::connected(120, 40);
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    for _ in 0..6 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    let f = footer(&h);
    assert!(f.contains("Enter Apply"), "{f}");
    assert!(f.contains("Esc Cancel"), "{f}");
    assert!(!f.contains("Commit"), "{f}");
}

#[test]
fn the_keyboard_reference_lists_the_live_shell_keys() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Char('?'));
    assert!(
        h.app
            .top_as::<junie_tui::widgets::help::HelpOverlay>()
            .is_some(),
        "{}",
        h.text()
    );
    let text = h.text();
    for (key, label) in [
        ("Ctrl+D", "Data / Structure"),
        ("Ctrl+O", "Open Quickly"),
        ("F1", "This reference"),
        ("Ctrl+PgDn", "Next page"),
    ] {
        assert!(
            text.lines().any(|l| l.contains(key) && l.contains(label)),
            "{key} {label}:\n{text}"
        );
    }
}

#[test]
fn hover_follows_the_rows_that_scroll_under_a_still_pointer() {
    let mut h = H::connected(120, 20);
    let (x, y) = h
        .find("customers")
        .unwrap_or_else(|| panic!("{}", h.text()));
    h.mouse(MouseKind::Move, x, y);
    let before = h.app.shell.pointer.hover;
    assert!(before.is_some());
    for _ in 0..4 {
        h.mouse(MouseKind::WheelDown, x, y);
    }
    let now = h.app.shell.hits.hit(Position::new(x, y));
    assert_ne!(now, before, "the tree did not scroll\n{}", h.text());
    assert_eq!(h.app.shell.pointer.hover, now);
}

#[test]
fn a_short_connection_form_keeps_fields_above_the_buttons_and_scrolls_to_focus() {
    let form = WidgetId::of("connections").sub("form");

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
            let buttons = h
                .app
                .shell
                .hits
                .area_of(form.sub("test"))
                .expect("buttons drawn");
            let footer = hh - 1;
            let mut seen = std::collections::HashSet::new();
            for _ in 0..40 {
                let Some(f) = h.focus() else { break };
                if !seen.insert(f) {
                    break;
                }
                if fields.contains(&f) {
                    if let Some(a) = h.app.shell.hits.area_of(f) {
                        assert!(
                            a.bottom() <= buttons.y,
                            "{w}x{hh} advanced={advanced}: {f:?} at {a:?} runs into the buttons at {buttons:?}\n{}",
                            h.text()
                        );
                    }

                    let text = h.text();
                    let gutter = text.lines().position(|l| l.contains('▎'));
                    assert!(
                        gutter.is_some_and(|y| (y as u16) < buttons.y),
                        "{w}x{hh} advanced={advanced}: focused {f:?} is not in view\n{text}"
                    );
                }
                h.key(KeyCode::Tab);
            }

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

    assert!(
        h.app
            .shell
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
    let mut h = H::connected(120, 40);
    let (x, y) = h.find("customers").unwrap();
    h.click(x, y);
    assert!(
        h.app.shell.pointer.pointer_focus,
        "focus came from the click"
    );
    h.key(KeyCode::Down);
    assert!(
        !h.app.shell.pointer.pointer_focus,
        "a key shows focus again"
    );
}

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
        .shell
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
        .shell
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

    let more = h.app.shell.hits.area_of(h.wb().strip.more_id()).unwrap();
    h.click(more.x + 1, more.y);
    assert!(h.wb().strip.menu_open());
    h.key(KeyCode::Esc);
    assert!(!h.wb().strip.menu_open());
    assert_eq!(h.wb().active, last);
}

#[test]
fn the_query_editor_takes_letters_at_once_and_its_find_bar_counts_as_editing() {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Tab);
    let editor = h.wb_query().editor.id;
    assert_eq!(h.focus(), Some(editor));
    assert!(h.term.backend().cursor_visible(), "caret on arrival");
    let active = h.wb().active;
    h.type_str("z[]q");
    assert!(!h.app.shell.quit, "q was typed, not quit");
    assert!(!h.wb().maximized, "z was typed, not zoom");
    assert_eq!(
        h.wb().active,
        active,
        "[ and ] were typed, not tab switches"
    );
    assert_eq!(h.wb_query().editor.text(), "z[]q");
    assert!(
        h.text().contains("Tab Indent"),
        "the footer says Tab indents"
    );
    h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(editor), "Tab indents while editing");
    h.key(KeyCode::Esc);
    assert!(!h.wb_query().editor.editing, "Esc is Done");
    assert!(h.text().contains("Enter Edit"), "{}", h.text());

    h.key(KeyCode::Char('/'));
    h.type_str("z]");
    assert!(!h.wb().maximized, "z went to the find query");
    assert_eq!(h.wb().active, active, "] went to the find query");
    assert_eq!(
        h.wb_query().editor.find.as_ref().map(|f| f.needle.text()),
        Some("z]")
    );
    h.key(KeyCode::Esc);
    h.key(KeyCode::Tab);
    assert_ne!(h.focus(), Some(editor), "after Esc, Tab leaves the editor");
}

#[test]
fn tab_walk_shows_a_caret_at_every_connection_form_field() {
    let mut h = H::new(120, 40);
    h.ctrl('n');
    let mut carets = 0;
    let mut stops = Vec::new();
    for _ in 0..30 {
        h.key(KeyCode::Tab);
        let f = h.focus();
        if stops.contains(&f) {
            continue;
        }
        stops.push(f);
        if h.app.connections.is_editing() {
            assert!(h.term.backend().cursor_visible(), "{f:?}\n{}", h.text());
            carets += 1;
        }
    }
    assert!(
        carets >= 4,
        "the form's text fields edit on focus: {carets}"
    );
}

#[test]
fn the_query_editor_selects_all_with_cmd_a() {
    let mut h = H::connected(80, 24);
    h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(h.wb_query().editor.id));
    h.type_str("SELECT 1");
    h.key(KeyCode::Enter);
    h.type_str("FROM orders");
    h.cmd('a');
    assert_eq!(
        h.wb_query().editor.field.buffer.selected_text(),
        Some("SELECT 1\nFROM orders")
    );
    h.type_str("SELECT 2");
    assert_eq!(h.wb_query().editor.text(), "SELECT 2");
}

#[test]
fn the_open_overflow_menu_is_the_only_layer_showing_focus() {
    let mut h = H::connected(80, 24);
    {
        let wb = h.app.workbench.as_mut().unwrap();
        for _ in 0..12 {
            wb.new_query("");
        }
        wb.set_active(0);
    }
    h.draw();
    let more = h.app.shell.hits.area_of(h.wb().strip.more_id()).unwrap();
    h.click(more.x + 1, more.y);
    assert!(h.wb().strip.menu_open(), "{}", h.text());
    let text = h.text();
    let strip_line = text.lines().nth(more.y as usize).unwrap();
    assert!(
        !strip_line.contains('▎'),
        "the strip under the open menu shows no focus\n{text}"
    );

    assert_eq!(text.matches('▎').count(), 0, "{text}");
    h.key(KeyCode::Esc);
    assert!(!h.wb().strip.menu_open());
    assert_eq!(
        h.text().matches('▎').count(),
        1,
        "focus is back on the strip"
    );
}

#[test]
fn down_on_the_strip_opens_the_overflow_menu() {
    let mut h = H::connected(80, 24);
    h.app.shell.focus.focus(crate::workbench::TABSTRIP);
    h.draw();
    h.key(KeyCode::Down);
    assert!(!h.wb().strip.menu_open(), "nothing hidden: no menu");
    {
        let wb = h.app.workbench.as_mut().unwrap();
        for _ in 0..12 {
            wb.new_query("");
        }
        wb.set_active(0);
    }
    h.app.shell.focus.focus(crate::workbench::TABSTRIP);
    h.draw();
    let hidden = h.wb().strip.hidden().1;
    assert!(hidden > 0, "{}", h.text());
    h.key(KeyCode::Down);
    assert!(h.wb().strip.menu_open(), "{}", h.text());
    let first_hidden = h.wb().tabs.len() - hidden;
    h.key(KeyCode::Enter);
    assert!(!h.wb().strip.menu_open());
    assert_eq!(h.wb().active, first_hidden, "{}", h.text());
}

#[test]
fn q_quits_from_the_focused_connection_tree_and_other_letters_type() {
    let tree = WidgetId::of("connections").sub("tree");
    let mut h = H::new(120, 36);
    assert_eq!(h.app.shell.focus.current(), Some(tree));
    let name = |h: &H| {
        let i = h.app.connections.selected.unwrap();
        h.app.connections.connections[i].name.clone()
    };
    let before = name(&h);
    h.key(KeyCode::Char('s'));
    let after = name(&h);
    assert!(
        after != before && after.starts_with('S'),
        "an unbound letter type-selects: {before} → {after}"
    );

    h.key(KeyCode::Char('?'));
    assert!(h.app.modal().is_some(), "? opens help: {}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app.shell.focus.current(), Some(tree));
    h.key(KeyCode::Char('q'));
    assert!(h.app.shell.quit, "q quits from the tree");
}

#[test]
fn q_quits_from_the_focused_explorer_tree() {
    let mut h = H::connected(120, 36);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(crate::workbench::EXPLORER)
    );
    h.key(KeyCode::Char('q'));
    assert!(h.app.shell.quit, "q quits from the explorer");
}

#[test]
fn the_frame_that_hides_the_focused_explorer_shows_where_focus_went() {
    let mut h = H::connected(100, 30);
    let explorer = h.wb().explorer.id;
    assert_eq!(h.app.shell.focus.current(), Some(explorer));
    h.key(KeyCode::Char('z'));
    assert!(h.wb().maximized);
    let focus = h.app.shell.focus.current();
    assert!(focus.is_some() && focus != Some(explorer));
    assert!(h.text().contains('▎'), "one focus indicator\n{}", h.text());
}

#[test]
fn ctrl_page_keys_step_workbench_tabs_over_an_editing_field() {
    let mut h = H::connected(120, 40);
    h.ctrl('t');
    assert_eq!(h.wb().active, 1);
    assert!(
        junie_tui::shell::KeyHost::capturing(&h.app),
        "the new query edits"
    );
    let step = |h: &mut H, code| {
        h.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::CONTROL,
        }));
        h.draw();
    };
    step(&mut h, KeyCode::PageUp);
    assert_eq!(h.wb().active, 0);
    step(&mut h, KeyCode::PageDown);
    assert_eq!(h.wb().active, 1);
}
