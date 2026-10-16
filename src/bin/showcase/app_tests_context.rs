//! Context menus across the showcase: a secondary click on a row opens
//! the row's menu at the pointer, Shift+F10 opens it under the cursor row,
//! and a secondary click outside an open popup closes it and opens the new
//! menu at the new spot.

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::app::{App, PageId};

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let mut term = Terminal::new(TestBackend::new(140, 45)).unwrap();
        term.draw(|f| app.render(f)).unwrap();
        Self { app, term }
    }

    fn send(&mut self, input: Input) -> Outcome {
        let out = self.app.handle(input);
        self.term.draw(|f| self.app.render(f)).unwrap();
        out
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        self.key_mod(code, KeyModifiers::NONE)
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Outcome {
        self.send(Input::Key(Key { code, mods }))
    }

    fn shift_f10(&mut self) -> Outcome {
        self.key_mod(KeyCode::F(10), KeyModifiers::SHIFT)
    }

    fn mouse(&mut self, kind: MouseKind, (x, y): (u16, u16)) -> Outcome {
        self.send(Input::Mouse(Mouse::new(kind, Position::new(x, y))))
    }

    fn right_click(&mut self, at: (u16, u16)) -> Outcome {
        self.mouse(MouseKind::Secondary, at)
    }

    fn click(&mut self, at: (u16, u16)) {
        self.mouse(MouseKind::Down, at);
        self.mouse(MouseKind::Up, at);
    }

    fn text(&self) -> String {
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

    /// The text of `r` on screen.
    fn text_in(&self, r: Rect) -> String {
        let buf = self.term.backend().buffer();
        let mut s = String::new();
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    /// Cell position of the first `needle` on screen.
    fn find(&self, needle: &str) -> Option<(u16, u16)> {
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

    fn at(&self, needle: &str) -> (u16, u16) {
        self.find(needle)
            .unwrap_or_else(|| panic!("{needle:?} on screen\n{}", self.text()))
    }

    /// The open menu `id`'s rectangle this frame.
    fn menu(&self, id: WidgetId) -> Option<Rect> {
        self.app.hits.area_of(id)
    }

    fn status(&self) -> String {
        self.app
            .status
            .as_ref()
            .map(|(s, _)| s.clone())
            .unwrap_or_default()
    }
}

const CHROME_MENU: WidgetId = WidgetId::of("chrome.context");
const LISTS_MENU: WidgetId = WidgetId::of("lists.menu");
const TREES_MENU: WidgetId = WidgetId::of("trees.menu");
const TABLES_MENU: WidgetId = WidgetId::of("tables.menu");

// ------------------------------------------------------------------ router

#[test]
fn a_secondary_click_outside_an_open_menu_reopens_it_on_the_new_row() {
    let mut h = Harness::new(PageId::Chrome);
    let codex = h.at("2 Codex (Primary)");
    let claude = h.at("1 Claude Code");
    let claude = (claude.0 + 3, claude.1);
    h.right_click(codex);
    let first = h.menu(CHROME_MENU).expect("the row menu is open");
    assert!(h.text_in(first).contains("2 Codex (Primary)"));

    assert!(
        !first.contains(Position::new(claude.0, claude.1)),
        "the second click lands outside the open menu"
    );
    assert_eq!(h.right_click(claude), Outcome::Changed);
    let menu = h.menu(CHROME_MENU).expect("a menu opens at the new spot");
    assert_eq!(
        (menu.x, menu.y),
        (claude.0, claude.1 + 1),
        "the new menu opens at the pointer\n{}",
        h.text()
    );
    let shown = h.text_in(menu);
    assert!(
        shown.contains("1 Claude Code"),
        "about the new row\n{shown}"
    );
    assert!(!shown.contains("Codex"), "the first menu closed");
    assert_eq!(h.count_menus(), 1, "one menu at a time");
}

impl Harness {
    fn count_menus(&self) -> usize {
        self.text().matches("Change title…").count()
    }
}

#[test]
fn a_secondary_click_outside_the_open_menu_bar_closes_it_and_opens_the_row_menu() {
    let mut h = Harness::new(PageId::Chrome);
    let shell = h.at("3 Shell");
    h.click(h.at("File"));
    assert!(h.text().contains("New tab"), "the File menu is open");

    assert_eq!(h.right_click(shell), Outcome::Changed);
    let text = h.text();
    assert!(
        !text.contains("New tab"),
        "the menu bar menu closed\n{text}"
    );
    let menu = h.menu(CHROME_MENU).expect("the row menu opened");
    assert_eq!((menu.x, menu.y), (shell.0, shell.1 + 1));
    assert!(h.text_in(menu).contains("3 Shell"));
}

#[test]
fn a_secondary_click_outside_a_dialog_stays_swallowed() {
    let mut h = Harness::new(PageId::Chrome);
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Keyboard & mouse"));
    let codex = h.at("2 Codex (Primary)");
    assert_eq!(h.right_click(codex), Outcome::Consumed);
    assert!(h.text().contains("Keyboard & mouse"), "the dialog stays");
    assert!(
        h.menu(CHROME_MENU).is_none(),
        "no menu opens under a dialog"
    );
}

// ------------------------------------------------------------------ chrome

#[test]
fn chrome_shift_f10_opens_the_tab_menu_under_the_cursor_row() {
    let mut h = Harness::new(PageId::Chrome);
    let shell = h.at("3 Shell");
    h.click(shell);
    assert_eq!(h.shift_f10(), Outcome::Changed);
    let menu = h.menu(CHROME_MENU).expect("Shift+F10 opens the tab menu");
    assert_eq!(menu.y, shell.1 + 1, "right under the cursor row");
    assert!(h.text_in(menu).contains("3 Shell"));
    assert!(
        !h.text().contains("New tab"),
        "Shift+F10 is not the menu bar"
    );
    h.key(KeyCode::Esc);
    assert!(h.menu(CHROME_MENU).is_none());

    h.key(KeyCode::Menu);
    assert!(h.menu(CHROME_MENU).is_some(), "the Menu key");
    h.key(KeyCode::Esc);
    h.key(KeyCode::Char('m'));
    assert!(
        h.menu(CHROME_MENU).is_none(),
        "m is no longer a context key"
    );
    assert!(h.text().contains("Shift+F10"), "the hint names the key");
}

// ------------------------------------------------------------------ lists

#[test]
fn lists_right_click_opens_the_row_menu_and_choosing_toggles_the_file() {
    let mut h = Harness::new(PageId::Lists);
    assert!(h.text().contains("2 selected"));
    let billing = h.at("src/api/billing.rs");
    let at = (billing.0 + 2, billing.1);
    assert_eq!(h.right_click(at), Outcome::Changed);
    let menu = h.menu(LISTS_MENU).expect("the row menu");
    assert_eq!((menu.x, menu.y), (at.0, at.1 + 1), "at the pointer");
    let shown = h.text_in(menu);
    assert!(shown.contains("src/api/billing.rs"), "titled by the row");
    assert!(shown.contains("Exclude") && shown.contains("Space"));
    assert!(shown.contains("Select all"));

    h.click(h.at("Exclude"));
    assert!(h.menu(LISTS_MENU).is_none(), "choosing closes the menu");
    assert!(h.text().contains("1 selected"), "the file was excluded");

    // a locked file says why it cannot be included
    let lock = h.at("Cargo.lock");
    h.right_click((lock.0 + 1, lock.1));
    let menu = h.menu(LISTS_MENU).expect("the row menu");
    assert!(h.text_in(menu).contains("locked file"));
    // the cursor starts on the first available item
    h.key(KeyCode::Enter);
    assert!(h.menu(LISTS_MENU).is_none());
    assert!(
        h.text().contains("10 selected"),
        "Select all skips the locked files"
    );
}

#[test]
fn lists_shift_f10_opens_under_the_cursor_row_and_chooses() {
    let mut h = Harness::new(PageId::Lists);
    h.click(h.at("TypeScript"));
    h.key(KeyCode::Down);
    let python = h.at("Python");
    h.key(KeyCode::Char('j'));
    h.key(KeyCode::Char('k'));
    assert_eq!(h.shift_f10(), Outcome::Changed);
    let menu = h.menu(LISTS_MENU).expect("Shift+F10 opens the row menu");
    assert_eq!(menu.y, python.1 + 1, "under the cursor row\n{}", h.text());
    assert!(h.text_in(menu).contains("Python"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Chosen: Python"));
    assert!(h.menu(LISTS_MENU).is_none());
}

// ------------------------------------------------------------------ trees

#[test]
fn trees_right_click_folds_a_folder_and_shift_f10_opens_a_file() {
    let mut h = Harness::new(PageId::Trees);
    let api = h.at("api");
    let at = (api.0 + 1, api.1);
    h.right_click(at);
    let menu = h.menu(TREES_MENU).expect("the row menu");
    assert_eq!((menu.x, menu.y), (at.0, at.1 + 1));
    let shown = h.text_in(menu);
    assert!(shown.contains("api") && shown.contains("Expand all"));
    assert!(
        shown.contains("Expand          →"),
        "a folded folder\n{shown}"
    );
    assert!(!h.text().contains("auth.rs"));
    h.key(KeyCode::Enter);
    assert!(h.menu(TREES_MENU).is_none());
    assert!(h.text().contains("auth.rs"), "the folder unfolded");
    h.right_click(at);
    assert!(h.text().contains("Collapse        ←"), "{}", h.text());
    h.key(KeyCode::Esc);

    h.click(h.at("auth.rs"));
    h.key(KeyCode::Down);
    let billing = h.at("billing.rs");
    assert_eq!(h.shift_f10(), Outcome::Changed);
    let menu = h.menu(TREES_MENU).expect("Shift+F10");
    assert_eq!(menu.y, billing.1 + 1);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("src/api/billing.rs"), "{}", h.text());
}

// ------------------------------------------------------------------ tables

#[test]
fn tables_right_click_a_cell_sorts_its_column_and_selects_its_row() {
    let mut h = Harness::new(PageId::Tables);
    let owner = h.at("jonas");
    let at = (owner.0 + 1, owner.1);
    h.right_click(at);
    let menu = h.menu(TABLES_MENU).expect("the cell menu");
    assert_eq!((menu.x, menu.y), (at.0, at.1 + 1));
    let shown = h.text_in(menu);
    assert!(
        shown.contains("Migrate sessions"),
        "titled by the task\n{shown}"
    );
    assert!(shown.contains("Sort by owner"));
    assert!(shown.contains("Read-only"), "edit is unavailable here");

    h.click(h.at("Sort by owner"));
    assert!(h.text().contains("sorted by owner ▴"), "{}", h.text());

    let owner = h.at("jonas");
    h.right_click(owner);
    h.click(h.at("Select  "));
    assert_eq!(h.status(), "Selected Migrate sessions table to UUID keys");
}

#[test]
fn tables_header_seam_resizes_and_the_header_menu_sorts_and_resets() {
    let mut h = Harness::new(PageId::Tables);
    let tasks = WidgetId::of("tables").sub("tasks");
    let (head, seam) = (tasks.sub("header").child(2), tasks.sub("seam").child(2));
    let before = h.app.hits.area_of(head).expect("the Owner title");
    let s = h.app.hits.area_of(seam).expect("a seam after Owner");
    assert_eq!(s.x, before.right());
    // drag the seam four cells right: the Owner column follows the pointer
    h.mouse(MouseKind::Down, (s.x, s.y));
    h.mouse(MouseKind::Drag, (s.x + 4, s.y));
    h.mouse(MouseKind::Up, (s.x + 4, s.y));
    assert_eq!(h.app.hits.area_of(head).unwrap().width, before.width + 4);
    // right-click the title: the header menu
    h.right_click((before.x + 1, before.y));
    let menu = h
        .menu(WidgetId::of("tables.header-menu"))
        .expect("the header menu");
    let shown = h.text_in(menu);
    assert!(shown.contains("Sort Descending"), "{shown}");
    h.click(h.at("Sort Descending"));
    assert!(h.text().contains("sorted by owner ▾"), "{}", h.text());
    h.right_click((before.x + 1, before.y));
    h.click(h.at("Reset Column Widths"));
    assert_eq!(h.app.hits.area_of(head).unwrap().width, before.width);
}

#[test]
fn tables_shift_f10_opens_under_the_cursor_row() {
    let mut h = Harness::new(PageId::Tables);
    h.click(h.at("jonas"));
    h.key(KeyCode::Down);
    let row = h
        .app
        .hits
        .area_of(WidgetId::of("tables").sub("tasks").child(2));
    assert_eq!(h.shift_f10(), Outcome::Changed);
    let menu = h.menu(TABLES_MENU).expect("Shift+F10");
    if let Some(row) = row {
        assert_eq!(menu.y, row.y + 1, "under the cursor row");
    }
    h.key(KeyCode::Esc);
    assert!(h.menu(TABLES_MENU).is_none(), "Esc dismisses");
}
