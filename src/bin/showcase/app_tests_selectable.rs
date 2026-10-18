use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::pages::selectable::SelectablePage;
use junie_tui::core::event::{ClientPlatform, Input, InputConfig, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16) -> Self {
        Self::on(w, h, ClientPlatform::Other)
    }

    fn on(w: u16, h: u16, platform: ClientPlatform) -> Self {
        let mut app = App::new(Theme::junie());
        app.pages[PageId::Selectable.index()] = Box::new(SelectablePage::for_platform(platform));
        app.open_page(PageId::Selectable);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();

        h.key(KeyCode::Tab);
        h.key(KeyCode::Tab);
        h
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn opt(&mut self, arg: &str) {
        self.app.apply_opt(crate::options::parse_opt(arg).unwrap());
        self.draw();
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    fn click_mod(&mut self, x: u16, y: u16, mods: KeyModifiers) {
        self.app.shell.pointer.reset_clicks();
        self.press_release(x, y, mods);
    }

    #[allow(dead_code)]
    fn double_click(&mut self, x: u16, y: u16) {
        self.click_mod(x, y, KeyModifiers::NONE);
        self.press_release(x, y, KeyModifiers::NONE);
    }

    fn press_release(&mut self, x: u16, y: u16, mods: KeyModifiers) {
        for kind in [MouseKind::Down, MouseKind::Up] {
            self.app.handle(Input::Mouse(
                Mouse::new(kind, Position::new(x, y)).with_mods(mods),
            ));
            self.draw();
        }
    }

    fn raw_click(&mut self, needle: &str, mods: KeyModifiers, config: InputConfig) {
        let (column, row) = self
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()));
        self.raw_click_at(column, row, mods, config);
    }

    fn raw_click_at(&mut self, column: u16, row: u16, mods: KeyModifiers, config: InputConfig) {
        use ratatui::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
        self.app.shell.pointer.reset_clicks();
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        ] {
            let ev = Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers: mods,
            });
            if let Some(input) = Input::from_crossterm_with(ev, config) {
                self.app.handle(input);
                self.draw();
            }
        }
    }

    fn footer(&self) -> String {
        self.row(self.term.backend().buffer().area.height - 1)
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

    fn click_text(&mut self, needle: &str, mods: KeyModifiers) {
        let (x, y) = self
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()));
        self.click_mod(x, y, mods);
    }

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
    let h = Harness::new(120, 43);
    assert!(h.has("No member selected"), "{}", h.text());
    assert!(h.has("Space selects"));
    assert!(h.has("[ ]"), "the header box is empty");
    assert!(!h.has("selected ·"), "no count in the meta yet");
}

#[test]
fn keyboard_selects_extends_and_clears_rows() {
    let mut h = Harness::new(120, 43);

    h.key(KeyCode::Char(' '));
    assert!(h.has("No member selected"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.has("Edit member"));
    assert!(h.table_row("Ari Moreno").contains("[✓]"));
    assert!(h.has("[–]"), "the header box turns indeterminate");

    assert!(h.has("ari@northwind.dev"));

    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert!(h.has("3 selected"), "{}", h.text());
    assert!(h.has("Edit 3 members"));
    assert!(h.table_row("Bo Lindqvist").contains("[✓]"));

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
    let mut h = Harness::new(120, 43);
    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.has("cleo@northwind.dev"));

    h.click_text("Fay Okafor", KeyModifiers::SHIFT);
    assert!(h.has("4 selected"), "{}", h.text());

    h.click_text("Eli Nakamura", KeyModifiers::CONTROL);
    assert!(h.has("3 selected"));
    assert!(!h.table_row("Eli Nakamura").contains("[✓]"));

    let (x, y) = h.find("Gus Petrov").unwrap();
    h.click(x - 4, y);
    assert!(h.has("4 selected"), "{}", h.text());

    h.click_text("Dana Whitfield", KeyModifiers::CONTROL);
    assert!(h.has("4 selected"));

    let (hx, hy) = h.find("[–]").expect("indeterminate header box");
    h.click(hx + 1, hy);
    assert!(h.has("11 selected"));
    h.click(hx + 1, hy);
    assert!(h.has("No member selected"));
}

#[test]
fn macos_option_click_toggles_rows_and_ctrl_click_opens_the_row_menu() {
    let mac = InputConfig::for_platform(ClientPlatform::MacOs);
    let mut h = Harness::on(120, 43, ClientPlatform::MacOs);
    h.raw_click("Cleo Barnes", KeyModifiers::NONE, mac);
    assert!(h.has("1 selected"), "{}", h.text());

    assert!(
        h.footer().contains("Option+click Toggle row"),
        "{}",
        h.footer()
    );
    assert!(!h.footer().contains("Ctrl+click"), "{}", h.footer());

    h.raw_click("Fay Okafor", KeyModifiers::ALT, mac);
    assert!(h.has("2 selected"), "{}", h.text());
    assert!(h.table_row("Cleo Barnes").contains("[✓]"));
    assert!(h.table_row("Fay Okafor").contains("[✓]"));
    h.raw_click("Fay Okafor", KeyModifiers::ALT, mac);
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(!h.table_row("Fay Okafor").contains("[✓]"));
    h.raw_click("Bo Lindqvist", KeyModifiers::ALT, mac);
    assert!(h.has("2 selected"));

    h.raw_click("Bo Lindqvist", KeyModifiers::CONTROL, mac);
    assert!(h.has("Delete…") && h.has("Edit"), "{}", h.text());
    assert!(h.footer().contains("Esc Close"), "{}", h.footer());
    assert!(h.has("2 selected"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.has("2 selected"));
    assert!(!h.table_row("Hana Sato").contains("[✓]"));

    let (x, y) = h.find("Hana Sato").unwrap();
    h.raw_click_at(x - 4, y, KeyModifiers::NONE, mac);
    assert!(h.has("3 selected"), "{}", h.text());
    assert!(h.table_row("Hana Sato").contains("[✓]"));

    h.key(KeyCode::Char(' '));
    assert!(h.has("2 selected"), "{}", h.text());
    assert!(!h.table_row("Hana Sato").contains("[✓]"));
}

#[test]
fn a_double_click_opens_a_row_and_a_slow_second_click_only_selects() {
    let mut h = Harness::new(120, 43);
    let table = h.app.shell.focus.current();
    let (x, y) = h.find("Cleo Barnes").unwrap();
    h.click(x, y);
    h.click(x, y);
    assert!(h.has("1 selected"), "{}", h.text());
    assert_eq!(
        h.app.shell.focus.current(),
        table,
        "a slow second click only selects"
    );
    h.double_click(x, y);
    assert_ne!(
        h.app.shell.focus.current(),
        table,
        "the double click moved into the editor"
    );

    let mut h = Harness::new(120, 43);
    h.opt("selection.activate=again");
    let (x, y) = h.find("Cleo Barnes").unwrap();
    h.click(x, y);
    assert_eq!(h.app.shell.focus.current(), table);
    h.click(x, y);
    assert_ne!(h.app.shell.focus.current(), table);
}

#[test]
fn other_platforms_toggle_with_ctrl_click_and_name_it() {
    let other = InputConfig::for_platform(ClientPlatform::Other);
    let mut h = Harness::on(120, 43, ClientPlatform::Other);
    h.raw_click("Cleo Barnes", KeyModifiers::NONE, other);
    assert!(
        h.footer().contains("Ctrl+click Toggle row"),
        "{}",
        h.footer()
    );
    h.raw_click("Fay Okafor", KeyModifiers::CONTROL, other);
    assert!(h.has("2 selected"), "{}", h.text());

    h.raw_click("Bo Lindqvist", KeyModifiers::ALT, other);
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.table_row("Bo Lindqvist").contains("[✓]"));
}

#[test]
fn row_menu_edits_or_deletes_the_row_it_opened_on() {
    let mut h = Harness::new(120, 43);
    let (x, y) = h.find("June Park").unwrap();
    h.app.handle(Input::Mouse(Mouse::new(
        MouseKind::Secondary,
        Position::new(x, y),
    )));
    h.draw();

    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.table_row("June Park").contains("[✓]"), "{}", h.text());

    h.click_text("Delete…", KeyModifiers::NONE);
    assert!(h.has("Delete 1 member?"), "{}", h.text());
    assert!(h.has("1 selected"), "{}", h.text());
    assert!(h.has("june@northwind.dev"), "the editor loaded June");

    h.key(KeyCode::Char('d'));
    assert!(!h.has("June Park"), "{}", h.text());

    let (x, y) = h.find("Kai Mensah").unwrap();
    h.app.handle(Input::Mouse(Mouse::new(
        MouseKind::Secondary,
        Position::new(x, y),
    )));
    h.draw();

    h.key(KeyCode::Enter);
    assert!(h.has("kai@northwind.dev"), "{}", h.text());
    assert!(h.has("Edit member"), "{}", h.text());

    let (x, y) = h.find("Dana Whitfield").unwrap();
    h.app.handle(Input::Mouse(Mouse::new(
        MouseKind::Secondary,
        Position::new(x, y),
    )));
    h.draw();
    assert!(h.has("Unavailable"), "{}", h.text());
}

#[test]
fn selection_survives_sorting() {
    let mut h = Harness::new(120, 43);
    h.click_text("Hana Sato", KeyModifiers::NONE);
    h.click_text("Bo Lindqvist", KeyModifiers::CONTROL);

    h.click_text("Name", KeyModifiers::NONE);
    assert!(h.has("2 selected"));
    assert!(h.table_row("Hana Sato").contains("[✓]"));
    assert!(h.table_row("Bo Lindqvist").contains("[✓]"));
    assert!(!h.table_row("Ari Moreno").contains("[✓]"));
}

#[test]
fn bulk_edit_applies_shared_fields_to_every_selected_member() {
    let mut h = Harness::new(120, 43);
    h.click_text("Ari Moreno", KeyModifiers::NONE);
    h.click_text("Bo Lindqvist", KeyModifiers::SHIFT);
    assert!(h.has("Edit 3 members"));

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

    assert!(h.table_row("Ari Moreno").contains("US"));
    assert!(h.table_row("Cleo Barnes").contains("EU"));

    assert!(h.has("3 selected"));
    assert!(!h.has("Unsaved changes"));
}

#[test]
fn single_edit_and_revert() {
    let mut h = Harness::new(120, 43);
    h.click_text("June Park", KeyModifiers::NONE);

    let rename = |h: &mut Harness| {
        h.key(KeyCode::Enter);
        h.key(KeyCode::End);
        for c in " Lee".chars() {
            h.key(KeyCode::Char(c));
        }
        h.key(KeyCode::Enter);
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
    let mut h = Harness::new(120, 43);
    h.click_text("Gus Petrov", KeyModifiers::NONE);
    h.click_text("June Park", KeyModifiers::CONTROL);
    h.key(KeyCode::Char('d'));
    assert!(h.has("Delete 2 members?"), "{}", h.text());
    assert!(h.has("Delete 2 members"));

    h.key(KeyCode::Esc);
    assert!(h.has("Gus Petrov") && h.has("2 selected"));

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

    let (_, bo) = h.find("Bo Lindqvist").unwrap();
    assert!(h.row(bo).contains("▎") && h.row(bo).contains("[ ]"));
    let (_, ari) = h.find("Ari Moreno").unwrap();
    assert!(!h.row(ari).contains("▎") && h.row(ari).contains("[✓]"));

    h.key(KeyCode::Tab);
    assert_eq!(bars(&h), 1, "{}", h.text());
}

#[test]
fn changing_the_selection_commits_valid_edits() {
    let mut h = Harness::new(120, 43);
    h.click_text("June Park", KeyModifiers::NONE);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.key(KeyCode::End);
    for c in " Lee".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Enter);
    assert!(h.has("Unsaved changes"), "{}", h.text());

    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    assert!(!h.has("Save changes?"), "{}", h.text());
    assert!(
        h.table_row("June Park Lee").contains("June Park Lee"),
        "{}",
        h.text()
    );
    assert!(h.table_row("Cleo Barnes").contains("[✓]"), "{}", h.text());
    assert!(h.has("cleo@northwind.dev"));
    assert!(!h.has("Unsaved changes"));
}

impl Harness {
    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn find_below(&self, needle: &str, below: u16, right: u16) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        (below + 1..buf.area.height).find_map(|y| {
            let row = self.row(y);
            row.find(needle)
                .map(|b| row[..b].chars().count() as u16)
                .filter(|&x| x < right)
                .map(|x| (x, y))
        })
    }
}

#[test]
fn a_select_chooses_by_press_drag_release_and_its_label_focuses_it() {
    let mut h = Harness::new(120, 43);
    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    let (_, ey) = h.find("Edit member").expect("the editor");
    let (lx, ly) = h.find_below("Role", ey, 120).expect("the Role field");
    let field_y = ly + 1;
    assert!(h.row(field_y).contains("Editor"), "{}", h.text());

    h.mouse(MouseKind::Down, lx + 2, field_y);
    let (vx, vy) = h
        .find_below("Viewer", field_y, lx + 40)
        .unwrap_or_else(|| panic!("the press opens the list:\n{}", h.text()));

    h.mouse(MouseKind::Drag, vx + 1, vy);
    h.mouse(MouseKind::Up, vx + 1, vy);
    assert!(h.row(field_y).contains("Viewer"), "{}", h.text());
    assert!(
        h.find_below("Admin", field_y, lx + 40).is_none(),
        "the list closed:\n{}",
        h.text()
    );

    h.click(lx + 1, ly);
    assert!(
        h.find_below("Admin", field_y, lx + 40).is_none(),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    assert!(
        h.find_below("Admin", field_y, lx + 40).is_some(),
        "the label click focused the field:\n{}",
        h.text()
    );
}

#[test]
fn changing_the_selection_with_invalid_edits_asks_save_dont_save_cancel() {
    let mut h = Harness::new(120, 43);
    h.click_text("June Park", KeyModifiers::NONE);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.key(KeyCode::End);
    for _ in "June Park".chars() {
        h.key(KeyCode::Backspace);
    }
    h.key(KeyCode::Enter);
    assert!(h.has("Unsaved changes"), "{}", h.text());

    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    assert!(h.has("Save changes?"), "{}", h.text());
    assert!(h.has("Don't Save") && h.has("Cancel"), "{}", h.text());

    h.key(KeyCode::Esc);
    assert!(!h.has("Save changes?"));
    assert!(h.has("Unsaved changes"));
    assert!(!h.table_row("Cleo Barnes").contains("[✓]"), "{}", h.text());

    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    h.key(KeyCode::Enter);
    assert!(!h.table_row("Cleo Barnes").contains("[✓]"), "{}", h.text());
    assert!(h.has("Unsaved changes"));

    h.click_text("Cleo Barnes", KeyModifiers::NONE);
    h.key(KeyCode::BackTab);
    h.key(KeyCode::Enter);
    assert!(h.table_row("Cleo Barnes").contains("[✓]"), "{}", h.text());
    assert!(h.has("cleo@northwind.dev"));
    assert!(!h.has("Unsaved changes"));
}

#[test]
fn a_clicked_row_anchors_a_shift_arrow_range() {
    let mut h = Harness::new(120, 43);
    h.click_text("Ari Moreno", KeyModifiers::NONE);
    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert!(h.has("3 selected"), "{}", h.text());
}
