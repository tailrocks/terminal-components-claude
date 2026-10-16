//! The "Cell editors" page driven through the real `App`: constrained
//! columns edited by keyboard and mouse, the Tags checklist, the Combo
//! search, flagged imported values, and the page's place in the sidebar.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::pages::celleditors::{ENV, LABELS, OWNER, PRIORITY};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;
use junie_tui::widgets::table::DataTable;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(PageId::CellEditors);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        // focus the table, the page's one stop
        h.key(KeyCode::Tab);
        h
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }

    fn click(&mut self, x: u16, y: u16) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            self.app
                .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
            self.draw();
        }
    }

    fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn table(&self) -> &DataTable {
        let page = self.app.pages[PageId::CellEditors.index()]
            .as_any()
            .and_then(|p| p.downcast_ref::<crate::pages::celleditors::CellEditorsPage>())
            .expect("the cell editors page");
        &page.table
    }

    fn cell(&self, row: usize, col: usize) -> String {
        self.table().rows[row][col].text.clone()
    }

    fn status(&self) -> Option<String> {
        self.app.status.text().map(str::to_owned)
    }
}

#[test]
fn the_page_sits_under_tables_and_answers_to_its_names() {
    for name in [
        "cell-editors",
        "celleditors",
        "optioneditors",
        "options",
        "enums",
    ] {
        assert_eq!(PageId::from_name(name), Some(PageId::CellEditors), "{name}");
    }
    let h = Harness::new(140, 40);
    let side = h.app.sidebar_area();
    let rows: Vec<String> = (side.top()..side.bottom())
        .map(|y| {
            (side.left()..side.right())
                .map(|x| h.term.backend().buffer()[(x, y)].symbol())
                .collect()
        })
        .collect();
    let at = |s: &str| rows.iter().position(|r| r.contains(s)).unwrap();
    assert!(at("Tables") < at("Cell editors"), "{rows:#?}");
    assert!(h.text().contains("/ Tables / Cell editors"));
}

#[test]
fn a_select_cell_is_edited_by_keyboard() {
    let mut h = Harness::new(140, 40);
    assert_eq!(h.table().cursor_col, ENV);
    assert!(h.text().contains("Production"), "labels, not stored values");
    h.key(KeyCode::Enter);
    assert!(h.app.pages[PageId::CellEditors.index()].editing());
    assert!(
        h.text().contains("Disaster recovery"),
        "the popup lists every option"
    );
    assert!(
        h.text().contains("Frozen until Friday"),
        "with the disabled reason"
    );
    // type-ahead to Staging, apply
    h.type_str("st");
    h.key(KeyCode::Enter);
    assert_eq!(h.cell(0, ENV), "stage");
    assert_eq!(h.status().as_deref(), Some("Env saved"));
    assert!(!h.app.pages[PageId::CellEditors.index()].editing());
    // Esc cancels
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Down);
    h.key(KeyCode::Esc);
    assert_eq!(h.cell(0, ENV), "stage");
    assert_eq!(h.status().as_deref(), Some("Edit cancelled"));
}

#[test]
fn a_select_cell_is_edited_by_mouse() {
    let mut h = Harness::new(140, 40);
    let cell = h.app.hits.area_of(h.table().cell_id(1, ENV)).unwrap();
    // first click moves the cursor, second click opens (select, then edit)
    h.click(cell.x, cell.y);
    assert!(h.table().choice.is_none());
    h.click(cell.x, cell.y);
    assert!(h.table().choice.is_some(), "{}", h.text());
    let opt = h.table().choice.as_ref().unwrap().editor.option_id(5);
    let at = h.app.hits.area_of(opt).expect("Performance is clickable");
    h.click(at.x + 2, at.y);
    assert_eq!(h.cell(1, ENV), "perf");
    // the ▾ on the focused cell opens in one click; a click outside cancels
    let mark = h.app.hits.area_of(h.table().open_id(1, ENV)).expect("▾");
    h.click(mark.x, mark.y);
    assert!(h.table().choice.is_some());
    h.click(1, 1);
    assert!(h.table().choice.is_none());
    assert_eq!(h.cell(1, ENV), "perf");
}

#[test]
fn the_tags_checklist_commits_several_values() {
    let mut h = Harness::new(140, 40);
    for _ in ENV..LABELS {
        h.key(KeyCode::Right);
    }
    assert_eq!(h.cell(0, LABELS), "api, web");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Retired label"));
    // uncheck api, check db; End stops at the last enabled tag (edge),
    // never on the retired one
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::End);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Enter);
    assert_eq!(h.cell(0, LABELS), "web, db, edge");
}

#[test]
fn the_owner_combo_filters_and_takes_only_listed_people() {
    let mut h = Harness::new(140, 40);
    for _ in ENV..OWNER {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.type_str("ros");
    assert!(h.text().contains("Sofia Rossi"));
    assert!(!h.text().contains("Ana Lima"), "filtered:\n{}", h.text());
    h.key(KeyCode::Enter);
    assert_eq!(h.cell(0, OWNER), "Sofia Rossi");
    h.key(KeyCode::Enter);
    h.type_str("nobody");
    assert!(h.text().contains("No matches"));
    h.key(KeyCode::Enter);
    assert!(h.table().choice.is_some(), "no free text");
    h.key(KeyCode::Esc);
    assert_eq!(h.cell(0, OWNER), "Sofia Rossi");
}

#[test]
fn imported_values_outside_the_set_are_flagged_until_fixed() {
    let mut h = Harness::new(140, 40);
    assert!(h.text().contains("values outside the set"), "{}", h.text());
    assert!(h.text().contains("qa-legacy"));
    // fix the env of the imported row: the popup has no current value
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    assert_eq!(h.table().choice.as_ref().unwrap().editor.current, None);
    h.key(KeyCode::Enter);
    assert_eq!(h.cell(5, ENV), "dev");
    assert!(!h.table().out_of_set_cells().contains(&(5, ENV)));
}

#[test]
fn the_priority_radio_skips_its_disabled_option_and_tab_moves_on() {
    let mut h = Harness::new(140, 40);
    for _ in ENV..PRIORITY {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Page on-call now"),
        "descriptions:\n{}",
        h.text()
    );
    h.key(KeyCode::End);
    h.key(KeyCode::Tab);
    assert_eq!(h.cell(0, PRIORITY), "P3", "P4 is disabled: End stops at P3");
    assert!(
        h.table().choice.is_none(),
        "Status is read only: Tab leaves"
    );
}
