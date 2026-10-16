//! The "Selectable table" page driven through the real `App`: keyboard and
//! mouse selection (with modifiers), the editor it drives, bulk edits, the
//! delete confirmation and the empty state.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(PageId::Selectable);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        // focus the first control on the page: the members table
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

    fn click_mod(&mut self, x: u16, y: u16, mods: KeyModifiers) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            self.app.handle(Input::Mouse(
                Mouse::new(kind, Position::new(x, y)).with_mods(mods),
            ));
            self.draw();
        }
    }

    fn click(&mut self, x: u16, y: u16) {
        self.click_mod(x, y, KeyModifiers::NONE);
    }

    fn row(&self, y: u16) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Column-accurate position of `needle` (cells hold one grapheme).
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

    /// Click the first cell showing `needle`.
    fn click_text(&mut self, needle: &str, mods: KeyModifiers) {
        let (x, y) = self
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()));
        self.click_mod(x, y, mods);
    }

    /// The table row showing `name`.
    fn table_row(&self, name: &str) -> String {
        let (_, y) = self.find(name).expect(name);
        self.row(y)
    }

    fn has(&self, needle: &str) -> bool {
        self.text().contains(needle)
    }
}

#[test]
fn page_resolves_by_label_and_aliases() {
    for name in [
        "Selectable table",
        "selectable-table",
        "selection",
        "bulk-edit",
    ] {
        assert_eq!(PageId::from_name(name), Some(PageId::Selectable), "{name}");
    }
}

#[test]
fn nothing_selected_shows_an_empty_state_that_says_what_to_do() {
    let h = Harness::new(120, 40);
    assert!(h.has("No member selected"), "{}", h.text());
    assert!(h.has("Space selects"));
    assert!(h.has("[ ]"), "the header box is empty");
    assert!(!h.has("selected ·"), "no count in the meta yet");
}

#[test]
fn keyboard_selects_extends_and_clears_rows() {
    let mut h = Harness::new(120, 40);
    // the owner row is first and disabled: Space does nothing there
    h.key(KeyCode::Char(' '));
    assert!(h.has("No member selected"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.has("Edit member"));
    assert!(h.table_row("Ari Moreno").contains("[✓]"));
    assert!(h.has("[–]"), "the header box turns indeterminate");
    // the editor holds Ari's values
    assert!(h.has("ari@northwind.dev"));

    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert!(h.has("3 selected"), "{}", h.text());
    assert!(h.has("Edit 3 members"));
    assert!(h.table_row("Bo Lindqvist").contains("[✓]"));
    // Ari is an admin, Cleo and Bo are editors
    assert!(h.has("— mixed —"), "{}", h.text());

    h.key_mod(KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert!(h.has("11 selected"), "all but the owner");
    assert!(h.has("[✓]  Name") || h.find("[✓]").is_some());

    h.key(KeyCode::Esc);
    assert!(h.has("No member selected"));
    assert!(!h.has("selected ·"));
}

#[test]
fn mouse_selects_with_desktop_modifiers_and_checkboxes() {
    let mut h = Harness::new(120, 40);
    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.has("cleo@northwind.dev"));
    // Shift-click extends from Cleo to Fay (display order)
    h.click_text("Fay Okafor", KeyModifiers::SHIFT);
    assert!(h.has("4 selected"), "{}", h.text());
    // Ctrl-click (Cmd where reported) toggles one row out
    h.click_text("Eli Nakamura", KeyModifiers::CONTROL);
    assert!(h.has("3 selected"));
    assert!(!h.table_row("Eli Nakamura").contains("[✓]"));
    // a checkbox click toggles without touching the rest
    let (x, y) = h.find("Gus Petrov").unwrap();
    h.click(x - 4, y);
    assert!(h.has("4 selected"), "{}", h.text());
    // the disabled owner row never selects
    h.click_text("Dana Whitfield", KeyModifiers::CONTROL);
    assert!(h.has("4 selected"));
    // the header box: mixed → all → none
    let (hx, hy) = h.find("[–]").expect("indeterminate header box");
    h.click(hx + 1, hy);
    assert!(h.has("11 selected"));
    h.click(hx + 1, hy);
    assert!(h.has("No member selected"));
}

#[test]
fn selection_survives_sorting() {
    let mut h = Harness::new(120, 40);
    h.click_text("Hana Sato", KeyModifiers::NONE);
    h.click_text("Bo Lindqvist", KeyModifiers::CONTROL);
    // sort by name: the selected rows move, the selection follows them
    h.click_text("Name", KeyModifiers::NONE);
    assert!(h.has("2 selected"));
    assert!(h.table_row("Hana Sato").contains("[✓]"));
    assert!(h.table_row("Bo Lindqvist").contains("[✓]"));
    assert!(!h.table_row("Ari Moreno").contains("[✓]"));
}

#[test]
fn bulk_edit_applies_shared_fields_to_every_selected_member() {
    let mut h = Harness::new(120, 40);
    h.click_text("Ari Moreno", KeyModifiers::NONE);
    h.click_text("Bo Lindqvist", KeyModifiers::SHIFT);
    assert!(h.has("Edit 3 members"));
    // open the role list (the first mixed field) and choose Viewer
    h.click_text(super::pages::selectable::MIXED, KeyModifiers::NONE);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.has("Unsaved changes"), "{}", h.text());
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(h.has("Updated 3 members"), "{}", h.text());
    for name in ["Ari Moreno", "Cleo Barnes", "Bo Lindqvist"] {
        let row = h.table_row(name);
        assert!(row.contains("Viewer"), "{name}: {row}");
    }
    // fields left as `— mixed —` keep each member's own value
    assert!(h.table_row("Ari Moreno").contains("US"));
    assert!(h.table_row("Cleo Barnes").contains("EU"));
    // the selection stays, the form reloads clean
    assert!(h.has("3 selected"));
    assert!(!h.has("Unsaved changes"));
}

#[test]
fn single_edit_and_revert() {
    let mut h = Harness::new(120, 40);
    h.click_text("June Park", KeyModifiers::NONE);
    // Enter from the table moves into the editor's first field
    let rename = |h: &mut Harness| {
        h.key(KeyCode::Enter); // start editing the name
        h.key(KeyCode::End);
        for c in " Lee".chars() {
            h.key(KeyCode::Char(c));
        }
        h.key(KeyCode::Enter); // commit the field, not the form
    };
    h.key(KeyCode::Enter);
    rename(&mut h);
    assert!(h.has("June Park Lee"), "{}", h.text());
    assert!(h.has("Unsaved changes"));
    assert!(
        h.table_row("June Park").contains("[✓]") && !h.table_row("June Park").contains("Lee"),
        "nothing is written before Apply"
    );
    let (rx, ry) = h.find("Revert").unwrap();
    h.click(rx + 1, ry);
    assert!(!h.has("June Park Lee"));
    assert!(!h.has("Unsaved changes"));
    // edit again and apply with the button
    // the name field is the row under its label; a click edits in one go
    let (lx, ly) = h.find("Name *").unwrap();
    h.click(lx + 2, ly + 1);
    h.key(KeyCode::End);
    for c in " Lee".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    h.click_text("Apply", KeyModifiers::NONE);
    assert!(h.has("Saved June Park Lee"), "{}", h.text());
    assert!(h.table_row("June Park Lee").contains("[✓]"));
}

#[test]
fn delete_asks_with_the_count_and_removes_the_rows() {
    let mut h = Harness::new(120, 40);
    h.click_text("Gus Petrov", KeyModifiers::NONE);
    h.click_text("June Park", KeyModifiers::CONTROL);
    h.key(KeyCode::Char('d'));
    assert!(h.has("Delete 2 members?"), "{}", h.text());
    assert!(h.has("Delete 2 members"));
    // cancel keeps everything
    h.key(KeyCode::Esc);
    assert!(h.has("Gus Petrov") && h.has("2 selected"));
    // the editor's Delete… asks the same
    h.click_text("Delete…", KeyModifiers::NONE);
    assert!(h.has("Delete 2 members?"));
    let (x, y) = h
        .find("Delete 2 members ")
        .or_else(|| h.find(" Delete 2 members"))
        .unwrap();
    h.click(x + 2, y);
    assert!(!h.has("Gus Petrov"), "{}", h.text());
    assert!(!h.has("June Park"));
    assert!(h.has("Deleted 2 members"));
    assert!(h.has("No member selected"));
}

#[test]
fn narrow_terminal_stacks_the_editor_under_the_table() {
    for (w, hh) in [(72u16, 20u16), (80, 24), (100, 30)] {
        let mut h = Harness::new(w, hh);
        h.key(KeyCode::Down);
        h.key(KeyCode::Char(' '));
        assert!(h.has("1 selected"), "{w}x{hh}\n{}", h.text());
    }
}

/// Selection is its own mark (`[✓]`), apart from the one focus bar.
#[test]
fn selection_marks_stay_apart_from_the_single_focus_bar() {
    let mut h = Harness::new(160, 50);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    h.key(KeyCode::Down);
    let focus = Theme::junie().focus;
    let bars = |h: &Harness| {
        h.term
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.symbol() == "▎" && c.fg == focus)
            .count()
    };
    assert_eq!(bars(&h), 1, "one focus bar\n{}", h.text());
    // the cursor (Bo) is not selected; the two selected rows carry no bar
    let (_, bo) = h.find("Bo Lindqvist").unwrap();
    assert!(h.row(bo).contains("▎") && h.row(bo).contains("[ ]"));
    let (_, ari) = h.find("Ari Moreno").unwrap();
    assert!(!h.row(ari).contains("▎") && h.row(ari).contains("[✓]"));
    // with focus in the editor the table shows no bar of its own
    h.key(KeyCode::Tab);
    assert_eq!(bars(&h), 1, "{}", h.text());
}
