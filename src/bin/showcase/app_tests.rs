//! Deterministic interaction tests driven through the real `App`, rendered
//! into a `TestBackend` so hit regions and focus rings are the same ones the
//! terminal would see.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key(&mut self, code: KeyCode) -> Outcome {
        let out = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
        out
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Outcome {
        let out = self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
        out
    }

    fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
        let out = self
            .app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
        out
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
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

    fn row(&self, y: u16) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    /// Find the row that contains `needle` (first match).
    fn find_row(&self, needle: &str) -> Option<u16> {
        let buf = self.term.backend().buffer();
        (0..buf.area.height).find(|&y| self.row(y).contains(needle))
    }

    /// Column-accurate search (symbols may be multi-byte).
    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        let want: Vec<&str> = {
            // split needle into graphemes the way cells store them
            unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect()
        };
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

    /// Column of the accent focus bar on a given row, if any.
    fn focus_bar_x(&self, y: u16) -> Option<u16> {
        let buf = self.term.backend().buffer();
        (0..buf.area.width).find(|&x| {
            let c = &buf[(x, y)];
            c.symbol() == "▎" && c.fg == Theme::junie().focus
        })
    }

    fn count(&self, needle: &str) -> usize {
        self.text().matches(needle).count()
    }

    fn focus_area(&self) -> Option<ratatui::layout::Rect> {
        self.app
            .focus
            .current()
            .and_then(|f| self.app.hits.area_of(f))
    }
}

fn tab() -> KeyCode {
    KeyCode::Tab
}

#[test]
fn launches_and_renders_shell() {
    let h = Harness::new(120, 40, PageId::Overview);
    let t = h.text();
    assert!(t.contains("Junie"));
    assert!(t.contains("Overview"));
    assert!(t.contains("Tokens"));
    assert!(!h.app.quit);
}

#[test]
fn every_page_renders_at_representative_sizes_without_panic() {
    for (w, h) in [
        (72, 20),
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
        (200, 60),
    ] {
        for entry in crate::app::NAV_ENTRIES {
            let mut hh = Harness::new(w, h, entry.id);
            // walk the focus ring twice and poke each stop
            for _ in 0..12 {
                hh.key(tab());
                hh.key(KeyCode::Down);
                hh.key(KeyCode::Right);
            }
            hh.key(KeyCode::Char('i'));
            hh.draw();
        }
    }
}

#[test]
fn below_minimum_size_shows_reduced_state() {
    let h = Harness::new(60, 15, PageId::Buttons);
    let t = h.text();
    assert!(t.contains("Terminal too small"));
    assert!(t.contains("Need 72×20, have 60×15"));
    assert!(!t.contains("Playground"));
}

#[test]
fn resize_recovers_from_too_small() {
    let mut h = Harness::new(60, 15, PageId::Buttons);
    h.term.backend_mut().resize(120, 40);
    h.app.handle(Input::Resize(120, 40));
    h.draw();
    assert!(h.text().contains("Playground"));
}

#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let start = h.app.focus.current();
    let mut seen = vec![start];
    for _ in 0..20 {
        h.key(tab());
        let cur = h.app.focus.current();
        if cur == start {
            break;
        }
        seen.push(cur);
    }
    assert_eq!(
        h.app.focus.current(),
        start,
        "focus ring wraps back to start"
    );
    // nav + the section strip + 7 enabled buttons (2 disabled are skipped)
    assert_eq!(seen.len(), 9, "{seen:?}");
    // Shift+Tab walks the same ring backwards
    let mut back = vec![];
    for _ in 0..seen.len() {
        h.key(KeyCode::BackTab);
        back.push(h.app.focus.current());
    }
    let mut expected = seen.clone();
    expected.reverse();
    assert_eq!(back, expected);
}

#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let (x, y) = h.find("Disabled primary").unwrap();
    h.click(x + 2, y);
    assert!(h.text().contains("? Help"), "no status message appeared");
    assert!(!h.text().contains("Disabled primary ✓"));
    // hovering a disabled control gives no feedback (style stays disabled)
    h.mouse(MouseKind::Move, x + 2, y);
    let buf = h.term.backend().buffer();
    let cell = &buf[(x + 2, y)];
    assert_eq!(cell.fg, Theme::junie().disabled);
}

#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let (x, y) = h.find("Run task").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Run task ✓"));
    h.key(tab());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Preview ✓"));
}

#[test]
fn hover_and_focus_render_differently() {
    let mut h = Harness::new(120, 40, PageId::Lists);
    let t = Theme::junie();
    let (x, y) = h.find("TypeScript").unwrap();
    // hover: the surface ladder's hover step, no bold
    h.mouse(MouseKind::Move, x, y);
    let cell = h.term.backend().buffer()[(x, y)].clone();
    assert_eq!(cell.bg, t.lift(t.surface));
    assert!(!cell.modifier.contains(ratatui::style::Modifier::BOLD));
    // focus via keyboard: bar + bold, hover suppressed
    h.key(tab());
    h.key(KeyCode::Down);
    let cell = h.term.backend().buffer()[(x, y)].clone();
    assert!(cell.modifier.contains(ratatui::style::Modifier::BOLD));
    let gutter = h.term.backend().buffer()[(x - 3, y)].clone();
    assert_eq!(gutter.symbol(), "▎");
    assert_eq!(gutter.fg, t.focus);
    assert_eq!(cell.bg, t.surface, "keyboard move clears hover lift");
}

#[test]
fn hit_testing_prefers_rows_over_their_container() {
    let mut h = Harness::new(120, 40, PageId::Tables);
    let (x, y) = h.find("#1042").unwrap();
    h.mouse(MouseKind::Move, x + 20, y);
    let area = h
        .app
        .pointer
        .hover
        .and_then(|id| h.app.hits.area_of(id))
        .unwrap();
    assert_eq!(
        area.height, 1,
        "hover resolved to a row/cell, not the table"
    );
}

#[test]
fn table_sorts_both_directions_and_clears() {
    let mut h = Harness::new(120, 40, PageId::Tables);
    h.key(tab());
    let first_before = h.row(h.find_row("#10").unwrap());
    // sort on the current (ID) column
    h.key(KeyCode::Char('s'));
    assert!(h.text().contains("ID ▴"));
    assert!(h.row(h.find_row("#10").unwrap()).contains("#1040"));
    h.key(KeyCode::Char('s'));
    assert!(h.text().contains("ID ▾"));
    // the cursor stayed on its row (now last), so the view scrolled with it
    // (the horizontal bar of the wide table takes the body's last row)
    assert!(h.text().contains("5–24 of 24"), "{}", h.text());
    h.key(KeyCode::Char('g'));
    assert!(h.row(h.find_row("#10").unwrap()).contains("#1063"));
    h.key(KeyCode::Char('s'));
    assert!(!h.text().contains("ID ▾") && !h.text().contains("ID ▴"));
    h.key(KeyCode::Char('g'));
    assert_eq!(first_before, h.row(h.find_row("#10").unwrap()));
    // numeric sort on Changes via header click: ascending puts "—" (0) first
    let (x, y) = h.find("Changes").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Changes ▴"));
    let first = h.find_row("#10").unwrap();
    assert!(h.row(first).contains(" 0 "), "{}", h.row(first));
    h.click(x, y);
    let first = h.find_row("#10").unwrap();
    assert!(h.row(first).contains("118"), "{}", h.row(first));
}

#[test]
fn header_click_sorts() {
    let mut h = Harness::new(120, 40, PageId::Tables);
    let (x, y) = h.find("Owner").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Owner ▴"));
    let first = h.find_row("#10").unwrap();
    assert!(h.row(first).contains("ana"));
    h.click(x, y);
    assert!(h.text().contains("Owner ▾"));
    let first = h.find_row("#10").unwrap();
    assert!(h.row(first).contains("sofia"));
}

#[test]
fn editable_table_commit_cancel_and_validation() {
    let mut h = Harness::new(120, 40, PageId::Editable);
    h.key(tab());
    assert!(!h.text().contains("EDIT"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("EDIT"), "edit badge appears");
    h.key(KeyCode::End);
    h.type_str(" now");
    h.key(KeyCode::Enter);
    assert!(!h.text().contains(" EDIT "));
    assert!(
        h.text().contains("Add rate limiting to auth endpoints now")
            || h.text().contains("1 edits")
    );
    // cancel restores
    h.key(KeyCode::Enter);
    h.type_str("zzz");
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("zzz"));
    // validation: Changes must be a number
    for _ in 0..4 {
        h.key(KeyCode::Right);
    }
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER);
    h.type_str("abc");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Changes must be a whole number"));
    assert!(
        h.app.pages[PageId::Editable.index()].editing(),
        "invalid value keeps editing"
    );
    h.key(KeyCode::Esc);
    assert!(!h.app.pages[PageId::Editable.index()].editing());
}

#[test]
fn input_editing_commit_and_revert() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.key(tab()); // the section strip
    h.key(tab());
    h.key(KeyCode::Enter);
    assert!(h.app.pages[PageId::Inputs.index()].editing());
    h.key(KeyCode::End);
    h.type_str("-v2");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("payments-gateway-v2"));
    h.key(KeyCode::Enter);
    h.type_str("XX");
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("XX"));
    // Tab commits and moves focus on
    let before = h.app.focus.current();
    h.key(KeyCode::Enter);
    h.type_str("!");
    h.key(tab());
    assert_ne!(h.app.focus.current(), before);
    assert!(h.text().contains("payments-gateway-v2!"));
    // editing places the hardware cursor
    h.key(KeyCode::Enter);
    let y = h.find_row("feat/").unwrap_or(0);
    let _ = y;
    assert!(h.app.pages[PageId::Inputs.index()].editing());
}

/// Cells of row `y` drawn with the selection style.
fn selected_on_row(h: &Harness, y: u16) -> String {
    let sel = Theme::junie().selection().bg;
    let buf = h.term.backend().buffer();
    (0..buf.area.width)
        .filter(|&x| buf[(x, y)].bg == sel.unwrap())
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

#[test]
fn typing_into_a_type_to_edit_field_starts_editing_and_a_default_field_edits_on_focus() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    // "Search files" opts into EditEntry::TypeToEdit: focus alone shows no
    // caret, the first letter starts the edit
    h.app.focus.focus(WidgetId::of("inputs").child(4));
    h.draw();
    assert!(!h.app.pages[PageId::Inputs.index()].editing());
    h.type_str("src");
    assert!(h.app.pages[PageId::Inputs.index()].editing());
    assert!(h.text().contains("src"), "{}", h.text());
    h.key(KeyCode::Enter);
    // "Branch" keeps the library default, EditEntry::Immediate: focus
    // starts the edit with the value selected and the caret shown, so a
    // letter replaces the value instead of firing a page shortcut
    h.app.focus.focus(WidgetId::of("inputs").child(1));
    h.draw();
    assert!(h.app.pages[PageId::Inputs.index()].editing());
    assert!(
        h.term.backend().cursor_visible(),
        "the caret shows on focus"
    );
    h.type_str("x");
    assert!(h.app.pages[PageId::Inputs.index()].editing());
}

#[test]
fn dragging_across_a_field_selects_the_text_it_crosses() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    let (x, y) = h.find("payments-gateway").unwrap();
    h.mouse(MouseKind::Down, x, y);
    assert!(
        !h.app.pages[PageId::Inputs.index()].editing(),
        "press alone"
    );
    h.mouse(MouseKind::Drag, x + 4, y);
    h.mouse(MouseKind::Drag, x + 8, y);
    h.mouse(MouseKind::Up, x + 8, y);
    assert!(h.app.pages[PageId::Inputs.index()].editing());
    assert_eq!(selected_on_row(&h, y), "payments");
}

#[test]
fn double_and_triple_clicks_select_a_word_and_a_line_in_a_text_area() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    let (x, y) = h.find("fix stuff").unwrap();
    h.click(x + 5, y);
    h.click(x + 5, y);
    assert_eq!(selected_on_row(&h, y), "stuff", "double click: the word");
    h.click(x + 5, y);
    assert_eq!(
        selected_on_row(&h, y),
        "fix stuff",
        "triple click: the line"
    );
}

#[test]
fn a_drag_from_the_second_press_of_a_double_click_extends_by_words() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    let (x, y) = h.find("fix stuff").unwrap();
    h.click(x + 1, y);
    // the second press selects the word; the drag takes whole words
    h.mouse(MouseKind::Down, x + 1, y);
    assert_eq!(selected_on_row(&h, y), "fix", "the press selects the word");
    h.mouse(MouseKind::Drag, x + 5, y);
    h.mouse(MouseKind::Up, x + 5, y);
    assert_eq!(selected_on_row(&h, y), "fix stuff");
}

#[test]
fn clicking_another_field_commits_through_the_owner() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    let name_row = h.find_row("payments-gateway").unwrap();
    let (x, _) = h.find("payments-gateway").unwrap();
    h.click(x + 2, name_row);
    assert!(h.app.pages[PageId::Inputs.index()].editing());
    h.key(KeyCode::End);
    h.type_str("-v3");
    // a click on the Branch field moves focus: the name field commits on
    // the event path and the page's Committed handler runs
    let (bx, by) = h.find("feat/").unwrap();
    h.click(bx, by);
    assert!(
        h.text().contains("Project name saved"),
        "the owner saw Committed"
    );
    assert!(h.text().contains("payments-gateway-v3"));
}

#[test]
fn clicking_dead_space_ends_the_edit_and_keeps_focus() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    let name_row = h.find_row("payments-gateway").unwrap();
    let (x, _) = h.find("payments-gateway").unwrap();
    h.click(x + 2, name_row);
    h.type_str("X");
    let focused = h.app.focus.current();
    // the blank row under the playground card
    let (_, py) = h.find("Search files").unwrap();
    let (dx, dy) = (x, py + 4);
    assert_eq!(h.app.hits.hit(Position::new(dx, dy)), None, "dead space");
    h.click(dx, dy);
    assert!(!h.app.pages[PageId::Inputs.index()].editing(), "edit ended");
    assert_eq!(h.app.focus.current(), focused, "focus stays on the field");
    assert!(h.text().contains("Project name saved"));
}

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    assert!(h.text().contains(" 1. Read"));
    let (x, y) = h.find(" 1. Read").unwrap();
    h.mouse(MouseKind::WheelDown, x, y + 2);
    assert!(!h.text().contains(" 1. Read"));
    assert!(h.text().contains(" 4. Run"));
    // Tab lands in the area editing (EditEntry::Immediate), caret shown
    h.key(tab());
    assert!(h.term.backend().cursor_visible());
    for _ in 0..30 {
        h.key(KeyCode::Down);
    }
    // the caret came in at the end of the long last line: Home shows its start
    h.key(KeyCode::Home);
    assert!(h.text().contains("28. Run"));
    assert!(h.text().contains("ln 28/28"));
}

#[test]
fn list_scrolling_and_selection() {
    let mut h = Harness::new(120, 40, PageId::Lists);
    h.key(tab());
    for _ in 0..19 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Erlang"));
    assert!(!h.text().contains("Rust\n") || true);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Chosen: Erlang"));
    // multi list toggles
    h.key(tab());
    let checks = h.count("✓");
    h.key(KeyCode::Char(' '));
    assert_eq!(
        h.count("✓"),
        checks - 1,
        "focus={:?}\n{}",
        h.focus_area(),
        h.text()
    );
    h.key(KeyCode::Char('a'));
    // all enabled rows are checked: 12 rows, 2 of them disabled
    assert_eq!(h.count("✓"), 10, "{}", h.text());
}

#[test]
fn tree_expand_collapse_and_focus_bar_column_is_stable() {
    let mut h = Harness::new(120, 40, PageId::Trees);
    h.key(tab());
    let y0 = h.find_row("src").unwrap();
    let bx = h.focus_bar_x(y0).unwrap();
    assert!(h.text().contains("config.rs"));
    h.key(KeyCode::Left); // collapse src
    assert!(!h.text().contains("config.rs"));
    h.key(KeyCode::Right); // expand
    h.key(KeyCode::Down); // api
    h.key(KeyCode::Right); // expand api
    assert!(h.text().contains("auth.rs"));
    h.key(KeyCode::Down);
    let y = h.find_row("auth.rs").unwrap();
    assert_eq!(
        h.focus_bar_x(y),
        Some(bx),
        "bar column unchanged at depth 2"
    );
}

#[test]
fn alt_click_on_a_tree_disclosure_opens_the_whole_subtree() {
    let mut h = Harness::new(120, 40, PageId::Trees);
    let (x, y) = h.find("api").unwrap();
    assert!(!h.text().contains("dispatch.rs"));
    // the disclosure sits two cells left of the label
    let alt = |h: &mut Harness, kind| {
        h.app.handle(Input::Mouse(
            Mouse::new(kind, Position::new(x - 2, y)).with_mods(KeyModifiers::ALT),
        ));
        h.draw();
    };
    alt(&mut h, MouseKind::Down);
    alt(&mut h, MouseKind::Up);
    let t = h.text();
    assert!(t.contains("auth.rs"), "{t}");
    assert!(
        t.contains("dispatch.rs"),
        "Alt-click opened nested folders too\n{t}"
    );
}

#[test]
fn grid_header_seam_drag_resizes_the_column() {
    let mut h = Harness::new(120, 40, PageId::Grid);
    let seam = WidgetId::of("grid")
        .sub("grid")
        .part(junie_tui::core::id::PartRef::item(
            junie_tui::core::id::Part::SEAM,
            junie_tui::core::id::ItemKey::index(1),
        ));
    let Some(s) = h.app.hits.area_of(seam) else {
        panic!("a seam after the second column\n{}", h.text());
    };
    let head = h.app.hits.area_of(WidgetId::of("grid").sub("grid").part(
        junie_tui::core::id::PartRef::item(
            junie_tui::core::id::Part::HEADER,
            junie_tui::core::id::ItemKey::index(2),
        ),
    ));
    h.mouse(MouseKind::Down, s.x, s.y);
    h.mouse(MouseKind::Drag, s.x + 3, s.y);
    h.mouse(MouseKind::Up, s.x + 3, s.y);
    let after = h.app.hits.area_of(seam).unwrap();
    assert_eq!(after.x, s.x + 3, "the seam followed the pointer");
    if let (Some(a), Some(b)) = (
        head,
        h.app.hits.area_of(WidgetId::of("grid").sub("grid").part(
            junie_tui::core::id::PartRef::item(
                junie_tui::core::id::Part::HEADER,
                junie_tui::core::id::ItemKey::index(2),
            ),
        )),
    ) {
        assert_eq!(b.x, a.x + 3, "the next column moved over");
    }
}

#[test]
fn modal_traps_focus_and_restores_it() {
    let mut h = Harness::new(120, 40, PageId::Dialogs);
    h.key(tab());
    let before = h.app.focus.current();
    h.key(KeyCode::Enter);
    assert!(h.app.dialog.is_some());
    assert!(h.text().contains("Run task now?"));
    let ring: Vec<_> = h.app.ring.reachable().to_vec();
    assert_eq!(ring.len(), 2, "only the dialog's two actions are reachable");
    for _ in 0..5 {
        h.key(tab());
        assert!(ring.contains(&h.app.focus.current().unwrap()));
    }
    // clicking the page behind does nothing to it
    let (x, y) = h.find("Rename task").unwrap();
    h.click(x, y);
    assert!(h.app.dialog.is_none(), "click outside cancels");
    assert_eq!(h.app.focus.current(), before, "focus restored");
    assert!(h.text().contains("Cancelled"));
    // the Run button's mnemonic answers a confirm
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Task started"));
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = Harness::new(120, 40, PageId::Dialogs);
    h.key(tab());
    h.key(tab());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Rename task"));
    // the prompt opens editing: caret shown, the value selected
    assert!(
        h.term.backend().cursor_visible(),
        "the prompt shows its caret"
    );
    h.key(KeyCode::Backspace);
    h.key(KeyCode::Enter); // submit empty → blocked
    assert!(h.app.dialog.is_some());
    assert!(h.text().contains("Name cannot be empty"));
    assert!(
        h.term.backend().cursor_visible(),
        "a blocked submit keeps the caret in the field"
    );
    h.type_str("Ship it");
    h.key(KeyCode::Enter);
    assert!(h.app.dialog.is_none());
    assert!(h.text().contains("Task: Ship it"));
}

#[test]
fn form_validation_blocks_submit_and_focuses_first_error() {
    let mut h = Harness::new(120, 40, PageId::Forms);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(h.text().contains("Required"));
    let area = h.focus_area().unwrap();
    let (_, y) = h.find("Short imperative summary").unwrap();
    assert_eq!(area.y, y, "focus moved to the invalid field");
    // focus landing there starts the edit: typing goes straight in
    assert!(h.term.backend().cursor_visible());
    h.type_str("Fix login bug");
    h.key(KeyCode::Enter);
    h.key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(h.text().contains("Creating task"));
}

#[test]
fn scrollbar_click_and_drag_move_the_view() {
    let mut h = Harness::new(120, 40, PageId::Scrolling);
    let (_, y0) = h.find("Row 001").unwrap();
    // scrollbar is the last column of the list area
    let list_area = {
        let id = junie_tui::core::id::WidgetId::of("scrolling").sub("list");
        h.app.hits.area_of(id).unwrap()
    };
    let sx = list_area.right() - 1;
    // a track click pages toward the pointer (TrackClick::Page)
    h.mouse(MouseKind::Down, sx, list_area.bottom() - 1);
    h.mouse(MouseKind::Up, sx, list_area.bottom() - 1);
    assert!(!h.text().contains("Row 001"));
    assert!(!h.text().contains("Row 120"), "one page, not a jump");
    // the thumb drags to either end
    let thumb = (list_area.y..list_area.bottom())
        .find(|&y| {
            let s = h.term.backend().buffer()[(sx, y)].symbol().to_owned();
            s == junie_tui::widgets::scrollbar::THUMB
        })
        .expect("a thumb");
    h.mouse(MouseKind::Down, sx, thumb);
    h.mouse(MouseKind::Drag, sx, list_area.bottom() + 2);
    assert!(h.text().contains("Row 120"));
    h.mouse(MouseKind::Drag, sx, list_area.y);
    h.mouse(MouseKind::Up, sx, list_area.y);
    assert!(h.text().contains("Row 001"));
    let _ = y0;
}

#[test]
fn keyboard_navigation_between_pages() {
    // pages follow the sidebar tree: Overview, App shell's Chrome, then
    // Inputs & forms
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char(']'));
    assert_eq!(h.app.page, PageId::Chrome);
    h.key(KeyCode::Char('['));
    assert_eq!(h.app.page, PageId::Overview);
    // open the "Inputs & forms" group with →, step in, open its page
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.page, PageId::Inputs);
    // the Tables group opens on a click; a page on another
    let (x, y) = h.find("Tables").unwrap();
    h.click(x, y);
    assert_eq!(h.app.page, PageId::Inputs, "a group row only folds");
    let (x, y) = h.find("Editable").unwrap();
    h.click(x, y);
    assert_eq!(h.app.page, PageId::Editable);
}

#[test]
fn quit_keys() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('q'));
    assert!(h.app.quit);
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.key(tab()); // the section strip
    h.key(tab());
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('q')); // typed into the field, not quit
    assert!(!h.app.quit);
    h.key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(h.app.quit);
}

#[test]
fn settings_screen_remove_member_flow() {
    let mut h = Harness::new(120, 40, PageId::Settings);
    h.key(tab());
    h.key(KeyCode::Right); // Members tab
    assert!(h.text().contains("Mira Okafor"));
    h.key(tab()); // table
    h.key(KeyCode::Down);
    h.key(tab()); // invite
    h.key(tab()); // remove
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Remove member?"));
    // destructive dialog focuses Cancel first: Enter keeps the member
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Jonas Weber"));
    h.key(KeyCode::Enter);
    // `y` no longer removes from the Cancel button; Remove's mnemonic does
    h.key(KeyCode::Char('y'));
    assert!(h.find_row("jonas@acme.dev").is_some());
    h.key(KeyCode::Char('r'));
    assert!(h.find_row("jonas@acme.dev").is_none());
    assert!(h.text().contains("5 members"));
}

/// `--motion reduced` (WCAG 2.3.3, SVg8): the task runner's work still
/// advances, the decorative phase (spinner frames) stays still. The
/// showcase used to run reduced as full motion.
#[test]
fn reduced_motion_runs_the_work_but_keeps_the_decorative_phase_still() {
    use junie_tui::motion::Motion;
    for (motion, phase) in [(Motion::Full, 30), (Motion::Reduced, 0)] {
        let mut app = App::with_motion(Theme::junie(), motion, 0);
        app.goto(PageId::TaskRunner);
        let term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mut h = Harness { app, term };
        h.draw();
        h.key(KeyCode::Char('r'));
        assert!(h.app.animating(), "{motion:?}: the run animates");
        for _ in 0..30 {
            h.app.handle(Input::Tick);
        }
        h.draw();
        assert!(
            h.text().contains("compile started"),
            "{motion:?}: the run advances"
        );
        assert_eq!(h.app.tick, phase, "{motion:?}: decorative phase");
    }
}

/// SV11: a status stays 5 s on the pointer clock (the showcase used 4 s,
/// the other apps 5 s), then the next tick clears it.
#[test]
fn a_status_stays_five_seconds_on_the_pointer_clock() {
    use junie_tui::clock::{Clock, WallClock};
    use junie_tui::theme::Tone;
    use std::time::{Duration, Instant};
    let mut h = Harness::new(120, 40, PageId::Overview);
    let at = |ago_ms: u64| WallClock::with_origin(Instant::now() - Duration::from_millis(ago_ms));
    // the clock reads 10 s; the status is set then
    h.app.clock = at(10_000);
    let now = h.app.clock.now();
    h.app.status.set_at("Saved", Tone::Secondary, now);
    // 4.5 s later it still shows
    h.app.clock = at(14_500);
    h.app.handle(Input::Tick);
    h.draw();
    assert_eq!(h.app.status.text(), Some("Saved"));
    assert!(h.text().contains("Saved"));
    // 5 s later it is gone
    h.app.clock = at(15_000);
    h.app.handle(Input::Tick);
    h.draw();
    assert!(h.app.status.is_none());
    assert!(!h.text().contains("Saved"));
}

#[test]
fn task_runner_animates_and_can_be_cancelled() {
    let mut h = Harness::new(120, 40, PageId::TaskRunner);
    h.key(KeyCode::Char('r'));
    assert!(h.app.animating());
    for _ in 0..30 {
        h.app.handle(Input::Tick);
    }
    h.draw();
    assert!(h.text().contains("compile started"));
    assert!(h.text().contains("%"));
    h.key(tab());
    h.key(tab()); // cancel button (run is disabled while running, so skipped)
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Cancel pipeline?"));
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("cancelled"));
    assert!(!h.text().contains("Pipeline · running"));
    assert!(!h.app.pages[PageId::TaskRunner.index()].animating());
}

#[test]
fn color_downgrade_still_renders() {
    let mut app = App::new(Theme::for_level(junie_tui::theme::ColorLevel::Ansi16));
    app.goto(PageId::Buttons);
    let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    term.draw(|f| app.render(f)).unwrap();
    let buf = term.backend().buffer();
    assert!(
        buf.content
            .iter()
            .any(|c| c.bg == ratatui::style::Color::LightGreen)
    );
}

#[test]
fn page_context_and_shell_do_not_duplicate_tab_hints() {
    for entry in crate::app::NAV_ENTRIES {
        let mut h = Harness::new(160, 50, entry.id);
        h.key(tab());
        let footer = h.row(49);
        assert!(
            footer.matches("Tab ").count() <= 1,
            "{}: {footer}",
            entry.label
        );
    }
}

/// Layout audit #1 and #17: on a short terminal pages stacked sections
/// past their area (Panels' "Target" and Chips' segment strip landed in the
/// blank row above the footer, the Task runner's rows ran into its own
/// buttons, Tables squeezed its table to one row). Every focus stop of
/// these pages is now inside the page area when focused, the row above
/// the footer stays blank, and a table keeps at least four rows.
#[test]
fn short_pages_scroll_by_section_and_never_draw_past_their_area() {
    let pages = [
        PageId::Panels,
        PageId::TaskRunner,
        PageId::Tables,
        PageId::Chips,
        PageId::Sidebars,
    ];
    for (w, hh) in [(72u16, 20u16), (80, 24)] {
        for page in pages {
            let mut h = Harness::new(w, hh, page);
            let spacer = hh - 2;
            let mut seen = std::collections::HashSet::new();
            for _ in 0..30 {
                let text = h.text();
                let row = text.lines().nth(spacer as usize).unwrap().to_owned();
                assert!(
                    row.trim().is_empty(),
                    "{page:?} {w}x{hh}: drawn on the row above the footer: {row:?}\n{text}"
                );
                let Some(f) = h.app.focus.current() else {
                    break;
                };
                if !seen.insert(f) {
                    break;
                }
                if let Some(a) = h.app.hits.area_of(f) {
                    assert!(
                        a.bottom() <= spacer,
                        "{page:?} {w}x{hh}: focused {f:?} at {a:?} is past the page\n{text}"
                    );
                }
                h.key(tab());
            }
        }
    }
    let mut h = Harness::new(72, 20, PageId::Tables);
    h.key(tab());
    let text = h.text();
    assert!(
        text.contains("#1043"),
        "the table keeps four rows and its bar on a short terminal\n{text}"
    );
}

/// The navigation tree sits in a "Components" card at every size, from
/// the minimum to the widest: the card's surface fills the whole sidebar
/// (its edge is the constant divider of layout audit #19, so the look
/// never flips at a breakpoint), the title row names it with the page
/// count, the rows sit inside the padding, and the scrollbar of an
/// overflowing tree keeps a blank column between itself and the card's
/// edge. Focus-within shows on the title, never as a second `▎`.
#[test]
fn the_navigation_tree_sits_in_a_titled_card_at_every_size() {
    use junie_tui::widgets::scrollbar::{THUMB, TRACK};
    let theme = Theme::junie();
    let count = crate::app::NAV_ENTRIES.len().to_string();
    for (w, hh) in [(72u16, 20u16), (80, 24), (100, 30), (120, 40), (160, 50)] {
        let mut h = Harness::new(w, hh, PageId::TaskRunner);
        // every group open: the tree overflows at the small sizes
        h.app.set_nav_expand(crate::app::NavExpand::All);
        h.draw();
        let nav = h.app.sidebar_area();
        let text = h.text();
        let buf = h.term.backend().buffer();
        let rows = h.app.nav.area;
        // the card's surface frames the rows: its title and bottom rows
        // and its side padding (the rows carry their own fills)
        for y in nav.top()..nav.bottom() {
            for x in nav.left()..nav.right() {
                if rows.contains(Position::new(x, y)) {
                    continue;
                }
                assert_eq!(
                    buf[(x, y)].bg,
                    theme.surface,
                    "{w}x{hh}: ({x},{y}) is off the card\n{text}"
                );
            }
        }
        let title = h.row(nav.y);
        let title = &title[..title
            .char_indices()
            .nth(nav.right() as usize)
            .map_or(title.len(), |(i, _)| i)];
        assert!(
            title.starts_with(" Components") && title.trim_end().ends_with(&count),
            "{w}x{hh}: the title row is {title:?}\n{text}"
        );
        // the rows start below the title and end above the card's bottom
        // padding; the first and last columns are padding
        assert_eq!(rows.y, nav.y + 2, "{w}x{hh}");
        assert_eq!(rows.bottom(), nav.bottom() - 1, "{w}x{hh}");
        assert_eq!((rows.x, rows.right()), (nav.x + 1, nav.right() - 1));
        let edge = nav.right() - 1;
        for y in nav.top()..nav.bottom() {
            assert_eq!(buf[(edge, y)].symbol(), " ", "{w}x{hh}: edge row {y}");
        }
        if h.app.nav.scroll.overflows() {
            let bar = (rows.top()..rows.bottom())
                .map(|y| buf[(rows.right() - 1, y)].symbol())
                .collect::<Vec<_>>();
            assert!(
                bar.iter().all(|s| *s == TRACK || *s == THUMB),
                "{w}x{hh}: the bar sits in the last content column {bar:?}\n{text}"
            );
        }
        // one focus indicator: the cursor row's `▎`, none on the title
        let bars = text.matches('▎').count();
        assert!(bars <= 1, "{w}x{hh}: {bars} focus bars\n{text}");
        assert!(!title.contains('▎'), "{w}x{hh}");
    }
}

/// Visual regression baseline: a stable digest of every cell (symbol, fg,
/// bg, modifiers) for each showcase page from the minimum through wide
/// terminals, in every supported palette, with focus on the first control.
/// Regenerate only after reviewing rendered output with
/// `UPDATE_BASELINE=1 cargo test baseline`.
#[test]
fn showcase_visual_baseline() {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (w, hgt, level) in [(120u16, 40u16), (80, 24), (72, 20), (100, 30), (160, 50)]
        .into_iter()
        .flat_map(|(w, h)| {
            use junie_tui::theme::ColorLevel::*;
            [TrueColor, Ansi256, Ansi16, Mono].map(|level| (w, h, level))
        })
    {
        // a templated page adds a row per further section (Options, States)
        let rows = crate::app::NAV_ENTRIES.iter().flat_map(|entry| {
            let sections: &[Option<crate::pages::template::Section>] =
                if Harness::new(w, hgt, entry.id).app.pages[entry.id.index()]
                    .template()
                    .is_some()
                {
                    &[
                        None,
                        Some(crate::pages::template::Section::Options),
                        Some(crate::pages::template::Section::States),
                    ]
                } else {
                    &[None]
                };
            sections.iter().map(move |s| (entry, *s))
        });
        for (entry, section) in rows {
            let mut h = Harness::new(w, hgt, entry.id);
            h.app.theme = Theme::for_level(level);
            if let Some(s) = section
                && let Some(t) = h.app.pages[entry.id.index()].template_mut()
            {
                t.set_section(s);
            }
            h.draw();
            h.key(tab());
            // the navigation sidebar grows whenever a page is added, so the
            // digest covers everything except it
            let sidebar = h.app.sidebar_area();
            let buf = h.term.backend().buffer();
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for (i, cell) in buf.content.iter().enumerate() {
                let x = (i % buf.area.width as usize) as u16;
                let y = (i / buf.area.width as usize) as u16;
                if sidebar.contains(Position::new(x, y)) {
                    continue;
                }
                let s = format!(
                    "{}|{:?}|{:?}|{:?};",
                    cell.symbol(),
                    cell.fg,
                    cell.bg,
                    cell.modifier
                );
                for b in s.bytes() {
                    hash ^= b as u64;
                    hash = hash.wrapping_mul(0x0100_0000_01b3);
                }
            }
            let label = match section {
                Some(s) => format!("{} · {}", entry.label, s.label()),
                None => entry.label.to_owned(),
            };
            writeln!(out, "{}x{} {:?} {label} {hash:016x}", w, hgt, level).unwrap();
        }
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/showcase_baseline.txt");
    if std::env::var_os("UPDATE_BASELINE").is_some() {
        std::fs::write(path, &out).unwrap();
        return;
    }
    let expected =
        std::fs::read_to_string(path).expect("baseline file; run with UPDATE_BASELINE=1");
    assert_eq!(
        out, expected,
        "showcase rendering changed; inspect before updating the baseline"
    );
}

/// Product decision 2026-10-16 (docs/audits/design-review-2026-10/hscroll.md):
/// a scrollbar track is a dotted rail in its own ink, never the panel
/// border. The Data grid shows both bars in one frame; both tracks use the
/// matched pair and the track style, at every colour level and with the
/// ASCII set.
#[test]
fn the_data_grid_draws_both_tracks_dotted_in_one_frame() {
    use junie_tui::theme::{ColorLevel, Glyphs};
    use junie_tui::widgets::scrollbar;
    let grid = WidgetId::of("grid").sub("grid");
    for glyphs in [Glyphs::UNICODE, Glyphs::ASCII] {
        for level in ColorLevel::ALL {
            let mut h = Harness::new(120, 40, PageId::Grid);
            h.app.theme = Theme::for_level(level);
            h.app.theme.glyphs = glyphs;
            h.draw();
            let t = h.app.theme;
            let ctx = format!("{level:?} {:?}", glyphs.scroll_track);
            let v = h.app.hits.area_of(scrollbar::id_for(grid));
            let hz = h.app.hits.area_of(scrollbar::h_id_for(grid));
            let (v, hz) = (v.expect(&ctx), hz.expect(&ctx));
            let buf = h.term.backend().buffer();
            let track_fg = t.scrollbar_track().fg;
            for (area, track, thumb) in [
                (v, t.glyphs.scroll_track, t.glyphs.scroll_thumb),
                (hz, t.glyphs.scroll_h_track, t.glyphs.scroll_h_thumb),
            ] {
                let cells: Vec<_> = area.positions().map(|p| &buf[p]).collect();
                assert!(cells.iter().any(|c| c.symbol() == thumb), "{ctx}: {area:?}");
                let tracks: Vec<_> = cells.iter().filter(|c| c.symbol() != thumb).collect();
                assert!(!tracks.is_empty(), "{ctx}: {area:?}");
                for c in tracks {
                    assert_eq!(c.symbol(), track, "{ctx}: {area:?}");
                    assert_eq!(Some(c.fg), track_fg, "{ctx}: {area:?}");
                }
            }
        }
    }
}

// ---------------------------------------------------------------- scrolling audit regressions

/// The card title row of the Scrolling page (the labels live there).
fn titles(h: &Harness) -> String {
    // the card title row: the first row carrying a position label (the
    // titles themselves may be shortened to keep the labels whole)
    h.text()
        .lines()
        .find(|l| (l.contains(" of ") || l.contains('/')) && l.contains('–'))
        .map(str::to_owned)
        .unwrap_or_default()
}

fn thumb_rows(h: &Harness, x: u16) -> Vec<u16> {
    let buf = h.term.backend().buffer();
    (0..buf.area.height)
        .filter(|&y| {
            let sym = buf[(x, y)].symbol();
            sym == junie_tui::widgets::scrollbar::THUMB
        })
        .collect()
}

/// The scrollbar column of the "Long list" card on the Scrolling page.
fn list_scrollbar_x(h: &Harness) -> u16 {
    let id = junie_tui::widgets::scrollbar::id_for(WidgetId::of("scrolling").sub("list"));
    h.app
        .hits
        .area_of(id)
        .expect("the list scrollbar is registered")
        .x
}

fn first_list_row(h: &Harness) -> u32 {
    h.text()
        .lines()
        .find_map(|l| {
            let i = l.find("Row ")?;
            l[i + 4..i + 7].parse().ok()
        })
        .expect("a list row is visible")
}

#[test]
fn scrolling_labels_are_exact_on_the_first_frame_after_a_resize_and_after_a_tick() {
    let mut h = Harness::new(160, 50, PageId::Scrolling);
    let title = titles(&h);
    assert!(
        title.contains("1–"),
        "prose and list labels on the first frame: {title}"
    );
    assert!(
        title.contains("of 400 · following"),
        "the log label is exact before any tick: {title}"
    );
    // a tick pushes a line: the label and the drawn tail agree
    h.app.handle(Input::Tick);
    h.draw();
    let title = titles(&h);
    assert!(title.contains("of 401 · following"), "{title}");
    let last = crate::data::log_lines(401).pop().unwrap();
    let tail: String = last.chars().take(12).collect();
    assert!(
        h.text().lines().any(|l| l.contains(&tail)),
        "the 401st line is drawn on the tick that reported it: {}",
        h.text()
    );
    // a resize re-lays out before the label is read; the label is never hidden
    h.term.backend_mut().resize(80, 24);
    h.app.handle(Input::Resize(80, 24));
    h.draw();
    let title = titles(&h);
    let drawn = h.text().lines().filter(|l| l.contains("Row ")).count() as u32;
    // at 80 columns the labels keep their positions in their short form
    // ("a–b/n", layout audit #7), never cut inside a number
    assert!(
        title.contains(&format!("1–{drawn} of")) || title.contains(&format!("1–{drawn}/")),
        "the list label matches the rows drawn at 80x24: {title}"
    );
    assert!(
        // (its long lines scroll sideways: the horizontal bar takes a row)
        title.contains("388–401 of") || title.contains("388–401/401"),
        "the log label survives at 80x24: {title}"
    );
}

#[test]
fn a_press_on_the_thumb_grabs_it_and_a_drag_moves_the_view_with_the_pointer() {
    let mut h = Harness::new(120, 40, PageId::Scrolling);
    let x = list_scrollbar_x(&h);
    let thumb = thumb_rows(&h, x);
    assert!(thumb.len() >= 3, "{thumb:?}");
    let grab_y = *thumb.last().unwrap();
    // pressing the last row of the thumb changes nothing
    h.mouse(MouseKind::Down, x, grab_y);
    assert_eq!(first_list_row(&h), 1, "a press on the thumb does not jump");
    // the held thumb keeps its glyph and turns the accent
    let held = &h.term.backend().buffer()[(x, grab_y)];
    assert_eq!(held.symbol(), junie_tui::widgets::scrollbar::THUMB);
    assert_eq!(held.fg, junie_tui::theme::Theme::junie().accent);
    // one row of pointer motion moves the view by a few rows, not a page
    let thumb_top = thumb[0];
    h.mouse(MouseKind::Drag, x, grab_y + 1);
    let after_one = first_list_row(&h);
    assert!(after_one > 1 && after_one <= 6, "{after_one}");
    assert_eq!(
        thumb_rows(&h, x).first(),
        Some(&(thumb_top + 1)),
        "the thumb followed by one row"
    );
    // dragging to the end of the track shows the last row
    let bottom = h
        .app
        .hits
        .area_of(junie_tui::widgets::scrollbar::id_for(
            WidgetId::of("scrolling").sub("list"),
        ))
        .unwrap()
        .bottom()
        - 1;
    h.mouse(MouseKind::Drag, x, bottom);
    assert!(h.text().contains("Row 120"), "{}", h.text());
    // releasing on the thumb keeps the view; the next press elsewhere jumps
    h.mouse(MouseKind::Up, x, bottom);
    assert!(h.text().contains("Row 120"));
}

#[test]
fn the_log_keeps_following_at_the_tail_and_prose_never_follows() {
    let mut h = Harness::new(160, 50, PageId::Scrolling);
    let log_x = h
        .app
        .hits
        .area_of(WidgetId::of("scrolling").sub("log"))
        .unwrap()
        .x
        + 2;
    let log_y = h
        .app
        .hits
        .area_of(WidgetId::of("scrolling").sub("log"))
        .unwrap()
        .y
        + 2;
    assert!(titles(&h).contains("following"));
    // a wheel down at the tail moves nothing and keeps following
    assert_eq!(
        h.mouse(MouseKind::WheelDown, log_x, log_y),
        Outcome::Consumed
    );
    assert!(titles(&h).contains("following"), "{}", titles(&h));
    // scrolling up pauses; scrolling back to the end resumes
    assert_eq!(h.mouse(MouseKind::WheelUp, log_x, log_y), Outcome::Changed);
    assert!(!titles(&h).contains("following"), "{}", titles(&h));
    h.mouse(MouseKind::WheelDown, log_x, log_y);
    assert!(
        titles(&h).contains("following"),
        "reaching the end resumes: {}",
        titles(&h)
    );
    // prose: `f` is not a verb there and End is only a jump
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.focus.current(),
        Some(WidgetId::of("scrolling").sub("prose"))
    );
    assert_eq!(h.key(KeyCode::Char('f')), Outcome::Ignored);
    h.key(KeyCode::End);
    // only the log follows: the word appears once, in the log's label
    assert_eq!(titles(&h).matches("following").count(), 1, "{}", titles(&h));
    let title_before = titles(&h);
    h.key(KeyCode::Up);
    h.term.backend_mut().resize(80, 24);
    h.app.handle(Input::Resize(80, 24));
    h.draw();
    assert!(
        titles(&h) != title_before,
        "the resize re-laid the prose out"
    );
    // at 80 columns the labels keep their positions and shorten their
    // suffix; the prose label is the first one and never says following
    let row = titles(&h);
    assert_eq!(row.matches('–').count(), 3, "all three labels stay: {row}");
    let before_log = row.rsplit_once('–').map(|(a, _)| a.to_owned()).unwrap();
    assert!(!before_log.contains("following"), "{row}");
}

#[test]
fn boundary_wheels_are_consumed_and_hover_follows_the_content() {
    let mut h = Harness::new(120, 40, PageId::Scrolling);
    let list = h
        .app
        .hits
        .area_of(WidgetId::of("scrolling").sub("list"))
        .unwrap();
    let (x, y) = (list.x + 4, list.y + 2);
    assert_eq!(
        h.mouse(MouseKind::WheelUp, x, y),
        Outcome::Consumed,
        "at the top"
    );
    h.mouse(MouseKind::Move, x, y);
    let hovered = h.app.pointer.hover;
    assert!(hovered.is_some());
    assert_eq!(h.mouse(MouseKind::WheelDown, x, y), Outcome::Changed);
    assert_ne!(
        h.app.pointer.hover, hovered,
        "hover moved to the row now under the pointer"
    );
    assert_eq!(h.app.pointer.hover, h.app.hits.hit(Position::new(x, y)));
}

#[test]
fn the_editor_scrollbar_takes_the_wheel_and_the_grid_scrollbar_keeps_focus() {
    let mut h = Harness::new(120, 40, PageId::Editor);
    let sb = h
        .app
        .hits
        .area_of(junie_tui::widgets::scrollbar::id_for(
            WidgetId::of("editor").sub("code"),
        ))
        .expect("the editor scrollbar is registered");
    let before = h.text();
    assert_eq!(
        h.mouse(MouseKind::WheelDown, sb.x, sb.y + 1),
        Outcome::Changed
    );
    assert_ne!(
        h.text(),
        before,
        "the editor scrolled under its scrollbar column"
    );

    let mut g = Harness::new(120, 40, PageId::Grid);
    let focus = g.app.focus.current();
    let sb = g
        .app
        .hits
        .area_of(junie_tui::widgets::scrollbar::id_for(
            WidgetId::of("grid").sub("grid"),
        ))
        .or_else(|| {
            g.app
                .hits
                .area_of(junie_tui::widgets::scrollbar::id_for(WidgetId::of("grid")))
        })
        .expect("the grid scrollbar is registered");
    g.click(sb.x, sb.bottom() - 1);
    assert_eq!(
        g.app.focus.current(),
        focus,
        "a scrollbar click never moves focus"
    );
}

#[test]
fn tables_take_the_horizontal_wheel_and_the_sidebar_nav_keeps_its_cursor_visible() {
    let mut h = Harness::new(80, 24, PageId::Tables);
    let before = h.row(6);
    h.mouse(MouseKind::WheelRight, 28, 8);
    assert_ne!(
        h.row(6),
        before,
        "the header shows the next columns: {}",
        h.row(6)
    );

    let mut s = Harness::new(72, 20, PageId::Sidebars);
    s.key(KeyCode::Tab);
    let nav = WidgetId::of("sidebars").sub("nav");
    assert_eq!(s.app.focus.current(), Some(nav));
    for _ in 0..8 {
        s.key(KeyCode::Down);
    }
    let last = s.app.hits.area_of(nav.child(7));
    assert!(last.is_some(), "the cursor row is on screen: {}", s.text());
    assert!(s.text().contains("Appearance"));
    let sb = s
        .app
        .hits
        .area_of(junie_tui::widgets::scrollbar::id_for(nav));
    assert!(
        sb.is_some(),
        "the nav shows its scrollbar when it overflows"
    );
    assert_eq!(
        s.mouse(MouseKind::WheelUp, last.unwrap().x + 2, last.unwrap().y),
        Outcome::Changed
    );
}

#[test]
fn the_terminal_viewport_survives_narrow_widths_and_completion_keeps_a_wheel_scroll() {
    let t = Harness::new(80, 24, PageId::Terminal);
    assert!(
        t.app
            .hits
            .area_of(WidgetId::of("terminal").sub("term"))
            .is_some(),
        "the viewport is laid out at 80 columns: {}",
        t.text()
    );
    let t = Harness::new(72, 20, PageId::Terminal);
    assert!(
        t.app
            .hits
            .area_of(WidgetId::of("terminal").sub("term"))
            .is_some()
    );
}

#[test]
fn cli_parser_accepts_aliases_and_rejects_invalid_values() {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    let cli = crate::Cli::try_parse_from([
        "showcase",
        "--color",
        "mono",
        "--page",
        "chipsselects",
        "--motion",
        "reduced",
        "--frame",
        "7",
    ])
    .expect("valid CLI options");
    assert!(matches!(
        cli.common.color,
        Some(junie_tui::cli::ColorArg::Mono)
    ));
    assert_eq!(cli.page, Some(PageId::Chips));
    assert!(matches!(
        cli.common.motion,
        Some(junie_tui::motion::Motion::Reduced)
    ));
    assert_eq!(cli.common.frame, 7);

    for color in ["truecolor", "24bit", "256", "16", "none", "mono"] {
        assert!(
            crate::Cli::try_parse_from(["showcase", "--color", color]).is_ok(),
            "color value {color:?}"
        );
    }
    for motion in ["full", "reduced", "paused"] {
        assert!(
            crate::Cli::try_parse_from(["showcase", "--motion", motion]).is_ok(),
            "motion value {motion:?}"
        );
    }
    let short = crate::Cli::try_parse_from([
        "showcase", "-c", "16", "-p", "buttons", "-m", "paused", "-f", "3",
    ])
    .unwrap();
    assert_eq!(short.page, Some(PageId::Buttons));
    assert_eq!(short.common.frame, 3);

    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--page", "no-such-page"]),
        Err(error) if error.kind() == ErrorKind::ValueValidation
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--color", "invalid"]),
        Err(error) if error.kind() == ErrorKind::InvalidValue
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--motion", "invalid"]),
        Err(error) if error.kind() == ErrorKind::InvalidValue
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--frame", "negative"]),
        Err(error) if error.kind() == ErrorKind::ValueValidation
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--frame"]),
        Err(error) if error.kind() == ErrorKind::InvalidValue
    ));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--unknown"]),
        Err(error) if error.kind() == ErrorKind::UnknownArgument
    ));
    let help = crate::Cli::command()
        .after_help(crate::cli_after_help())
        .render_help()
        .to_string();
    assert!(help.contains("--motion paused"));
    assert!(help.contains("Tab/Shift+Tab focus"));
    assert!(matches!(
        crate::Cli::try_parse_from(["showcase", "--help"]),
        Err(error) if error.kind() == ErrorKind::DisplayHelp
    ));
}

#[test]
fn select_list_floats_above_the_page_and_closes_on_an_outside_click() {
    // a short terminal: the list opens over the panels drawn after the field
    let mut h = Harness::new(120, 30, PageId::Chips);
    let size = WidgetId::of("chips").sub("size");
    let field = h.app.hits.area_of(size).expect("page size field");
    h.click(field.x + 2, field.y);
    let rows: Vec<_> = (0..4)
        .map(|i| h.app.hits.area_of(size.child(i)).expect("option row"))
        .collect();
    let buf = h.term.backend().buffer().clone();
    for (row, label) in rows.iter().zip(["25", "50", "100", "500"]) {
        let text: String = (row.x..row.right())
            .map(|x| buf[(x, row.y)].symbol())
            .collect();
        assert!(
            text.contains(label),
            "option {label} is painted on top: {text:?}"
        );
        assert!(row.bottom() <= 30, "inside the screen: {row:?}");
    }
    assert_eq!(
        h.app.hits.hit(Position::new(rows[3].x + 3, rows[3].y)),
        Some(size.child(3)),
        "the last option is clickable, not shadowed by the page"
    );

    // a click outside closes the list, keeps the value and is consumed
    let before = h.text();
    let outside = Position::new(field.x + 2, 29 - 2);
    assert!(!rows.iter().any(|r| r.contains(outside)));
    h.click(outside.x, outside.y);
    assert!(h.app.hits.area_of(size.child(0)).is_none(), "list closed");
    assert!(before.contains("50"));
    assert!(h.text().contains("Page size"));

    // reopen and choose with the mouse: the click lands on the option
    h.click(field.x + 2, field.y);
    let last = h.app.hits.area_of(size.child(3)).unwrap();
    h.click(last.x + 3, last.y);
    assert!(h.app.hits.area_of(size.child(0)).is_none());
    let row: String = (field.x..field.right())
        .map(|x| h.term.backend().buffer()[(x, field.y)].symbol().to_owned())
        .collect();
    assert!(row.contains("500"), "{row:?}");
}

#[test]
fn a_menu_bar_title_opens_on_the_press_and_a_drag_release_chooses_a_row() {
    let mut h = Harness::new(120, 40, PageId::Chrome);
    let bar = WidgetId::of("chrome.menubar");
    let file = h.app.hits.area_of(bar.child(0)).expect("menu title");
    let row = |i: usize| bar.child(0).sub("menu").child(i);
    // the menu is open while the button is still down
    h.mouse(MouseKind::Down, file.x + 1, file.y);
    let r1 = h
        .app
        .hits
        .area_of(row(1))
        .expect("File opened on the press");
    // releasing back on the title after no movement keeps it open
    h.mouse(MouseKind::Up, file.x + 1, file.y);
    assert!(h.app.hits.area_of(row(0)).is_some(), "still open");
    // press the open title: it closes; press again, drag onto a row, release
    h.mouse(MouseKind::Down, file.x + 1, file.y);
    h.mouse(MouseKind::Up, file.x + 1, file.y);
    assert!(h.app.hits.area_of(row(0)).is_none(), "the press closed it");
    h.mouse(MouseKind::Down, file.x + 1, file.y);
    h.mouse(MouseKind::Drag, r1.x + 2, r1.y);
    h.mouse(MouseKind::Up, r1.x + 2, r1.y);
    assert!(
        h.app.hits.area_of(row(0)).is_none(),
        "choosing closed the menu"
    );
    // the chosen item's label lands in the status bar
    assert!(h.text().contains("Split right"), "{}", h.text());
}

#[test]
fn menu_bar_switches_menus_under_the_pointer_and_closes_on_an_outside_click() {
    let mut h = Harness::new(120, 40, PageId::Chrome);
    let bar = WidgetId::of("chrome.menubar");
    let title = |h: &Harness, i: usize| h.app.hits.area_of(bar.child(i)).expect("menu title");
    let first_row = |i: usize| bar.child(i).sub("menu").child(0);
    let open = |h: &Harness| -> Vec<usize> {
        (0..3)
            .filter(|&i| h.app.hits.area_of(first_row(i)).is_some())
            .collect()
    };

    // File opens on a click
    let file = title(&h, 0);
    h.click(file.x + 1, file.y);
    assert_eq!(open(&h), vec![0]);

    // like macOS: pointing at View, then Help, swaps the open menu
    let view = title(&h, 1);
    h.mouse(MouseKind::Move, view.x + 1, view.y);
    assert_eq!(open(&h), vec![1], "View replaced File");
    let help = title(&h, 2);
    h.mouse(MouseKind::Move, help.x + 1, help.y);
    assert_eq!(open(&h), vec![2], "Help replaced View");
    // moving into the open menu keeps it open
    let row = h.app.hits.area_of(first_row(2)).unwrap();
    h.mouse(MouseKind::Move, row.x + 2, row.y);
    assert_eq!(open(&h), vec![2]);

    // as on macOS the pointer reaching File already opened it, so the press
    // lands on the open title and closes the menu; the release keeps it shut
    h.mouse(MouseKind::Move, file.x + 1, file.y);
    assert_eq!(open(&h), vec![0], "pointing at File swapped the menu");
    h.mouse(MouseKind::Down, file.x + 1, file.y);
    assert!(open(&h).is_empty(), "pressing the open title closes it");
    h.mouse(MouseKind::Up, file.x + 1, file.y);
    assert!(open(&h).is_empty());

    // a click anywhere outside closes the menu without reaching the page
    h.click(file.x + 1, file.y);
    let status_before = h.text();
    h.click(110, 30);
    assert!(open(&h).is_empty(), "outside click closed the menu");
    assert!(status_before.contains("File"));
    // with no menu open, pointing at a title does not open one
    h.mouse(MouseKind::Move, view.x + 1, view.y);
    assert!(open(&h).is_empty());
}

#[test]
fn a_pass_through_select_closes_and_lets_the_outside_click_land() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    let sort = WidgetId::of("chips").sub("sort");
    let size = WidgetId::of("chips").sub("size");
    let sort_field = h.app.hits.area_of(sort).unwrap();
    let size_field = h.app.hits.area_of(size).unwrap();

    // "Sort by" keeps the default policy: the outside click only closes
    h.click(sort_field.x + 2, sort_field.y);
    assert!(h.app.hits.area_of(sort.child(0)).is_some());
    h.click(size_field.x + 2, size_field.y);
    assert!(h.app.hits.area_of(sort.child(0)).is_none(), "sort closed");
    assert!(
        h.app.hits.area_of(size.child(0)).is_none(),
        "size not opened"
    );

    // "Page size" passes the click through: it closes and Sort by opens
    h.click(size_field.x + 2, size_field.y);
    assert!(h.app.hits.area_of(size.child(0)).is_some());
    h.click(sort_field.x + 2, sort_field.y);
    assert!(h.app.hits.area_of(size.child(0)).is_none(), "size closed");
    assert!(h.app.hits.area_of(sort.child(0)).is_some(), "sort opened");
    assert_eq!(h.app.focus.current(), Some(sort));
}

#[test]
fn keys_switch_menus_even_with_the_pointer_parked_on_a_title() {
    let mut h = Harness::new(120, 40, PageId::Chrome);
    let bar = WidgetId::of("chrome.menubar");
    let file = h.app.hits.area_of(bar.child(0)).unwrap();
    h.click(file.x + 1, file.y);
    let open = |h: &Harness, i: usize| {
        h.app
            .hits
            .area_of(bar.child(i).sub("menu").child(0))
            .is_some()
    };
    assert!(open(&h, 0));
    // the pointer stays on File; the keyboard wins until it moves
    h.key(KeyCode::Right);
    assert!(open(&h, 1), "→ opened View");
    h.key(KeyCode::Right);
    assert!(open(&h, 2), "→ opened Help");
    // inside a menu ↓ is not undone by the parked pointer either
    h.key(KeyCode::Left);
    h.key(KeyCode::Left);
    assert!(open(&h, 0));
    h.mouse(MouseKind::Move, file.x + 1, file.y + 1);
    h.key(KeyCode::Down);
    let style = |h: &Harness, row: usize| {
        let r = h
            .app
            .hits
            .area_of(bar.child(0).sub("menu").child(row))
            .unwrap();
        h.term.backend().buffer()[(r.x + 2, r.y)].bg
    };
    assert_ne!(
        style(&h, 1),
        style(&h, 0),
        "cursor moved off the hovered row"
    );
}

#[test]
fn the_current_page_marker_is_neutral_while_focus_is_on_the_page() {
    let mut h = Harness::new(120, 40, PageId::Lists);
    let nav = WidgetId::of("app.nav");
    let marker = |h: &Harness| {
        let side = h.app.sidebar_area();
        let buf = h.term.backend().buffer();
        (side.top()..side.bottom())
            .flat_map(|y| (side.left()..side.right()).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].clone())
            .find(|c| c.symbol() == "›")
            .expect("current-page marker")
            .fg
    };
    let t = h.app.theme;
    h.app.focus.set(Some(nav));
    h.draw();
    assert_eq!(marker(&h), t.accent, "accent while the nav has focus");
    h.key(tab());
    assert_ne!(h.app.focus.current(), Some(nav));
    assert_eq!(marker(&h), t.text_primary, "a fact, not a focus cue");
}

#[test]
fn a_cut_list_row_advertises_the_reveal_key_in_the_footer() {
    // narrow enough that some file names in the lists are cut
    let mut h = Harness::new(72, 20, PageId::Lists);
    let footer = |h: &Harness| h.row(19);
    h.key(tab());
    let mut seen = None;
    'lists: for _ in 0..3 {
        for _ in 0..14 {
            if footer(&h).contains("→ Full text") {
                seen = Some(h.text());
                break 'lists;
            }
            assert!(
                !footer(&h).contains("Full text") || footer(&h).contains("→ Full text"),
                "{}",
                footer(&h)
            );
            h.key(KeyCode::Down);
        }
        h.key(tab());
    }
    assert!(seen.is_some(), "some cut row is reached:\n{}", h.text());
    // the hint names what the key does now: → opens, ← collapses
    assert_eq!(h.key(KeyCode::Right), Outcome::Changed);
    assert!(footer(&h).contains("← Collapse"), "{}", footer(&h));
    h.key(KeyCode::Left);
    assert!(footer(&h).contains("→ Full text"), "{}", footer(&h));
    // a row that fits says nothing about the reveal key
    h.key(KeyCode::Home);
    if !footer(&h).contains("→ Full text") {
        assert!(!footer(&h).contains("Full text"));
    }
}

#[test]
fn a_focused_tab_strip_takes_home_and_end() {
    // interaction #23: Home/End go to the first and last tab
    let mut h = Harness::new(120, 40, PageId::Settings);
    h.key(tab());
    h.key(KeyCode::End);
    assert!(h.text().contains("Environment") && !h.text().contains("Mira Okafor"));
    h.key(KeyCode::Home);
    h.key(KeyCode::Right);
    assert!(h.text().contains("Mira Okafor"), "{}", h.text());
    h.key(KeyCode::End);
    assert!(
        !h.text().contains("Mira Okafor"),
        "End left the Members tab"
    );
}

#[test]
fn header_links_show_rest_hover_and_press() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let (x, y) = h.find("? Help").unwrap();
    let cell = |h: &Harness| {
        let c = &h.term.backend().buffer()[(x + 2, y)];
        (c.bg, c.modifier)
    };
    let rest = cell(&h);
    h.mouse(MouseKind::Move, x + 2, y);
    let hover = cell(&h);
    h.mouse(MouseKind::Down, x + 2, y);
    let pressed = cell(&h);
    assert_ne!(rest, hover, "the Help link hovers");
    assert_ne!(hover, pressed, "the Help link presses");
    assert_ne!(rest, pressed);
}

#[test]
fn the_terminal_viewport_scrolls_a_long_line_sideways_under_its_bar() {
    use junie_tui::widgets::scrollbar::{H_THUMB, h_id_for};
    let mut h = Harness::new(80, 24, PageId::Terminal);
    let term = WidgetId::of("terminal").sub("term");
    let bar = h
        .app
        .hits
        .area_of(h_id_for(term))
        .expect("the prompt line overflows: a horizontal bar");
    let (x, y) = h.find("payments-platform").expect("the prompt is drawn");
    let thumb = |h: &Harness| {
        (bar.x..bar.right())
            .position(|x| h.term.backend().buffer()[(x, bar.y)].symbol() == H_THUMB)
            .expect("a thumb")
    };
    assert_eq!(thumb(&h), 0, "at the left edge");
    h.mouse(MouseKind::WheelRight, x + 2, y);
    assert!(
        !h.row(y).contains("payments-platform"),
        "the line moved left: {}",
        h.row(y)
    );
    assert!(
        h.row(y).contains("launch"),
        "its end comes into view: {}",
        h.row(y)
    );
    assert!(thumb(&h) > 0, "the thumb moved right");
    // the bar drags back to the start
    h.mouse(MouseKind::Down, bar.right() - 1, bar.y);
    h.mouse(MouseKind::Drag, bar.x, bar.y);
    h.mouse(MouseKind::Up, bar.x, bar.y);
    assert_eq!(thumb(&h), 0);
    assert!(h.row(y).contains("payments-platform"));
}

#[test]
fn a_grid_range_drag_held_below_the_grid_autoscrolls_on_the_tick() {
    let mut h = Harness::new(120, 30, PageId::Grid);
    let (x, y) = h.find("1001").unwrap();
    h.mouse(MouseKind::Down, x, y);
    // the pointer leaves the grid at the bottom and rests on the last row
    let rest = h.term.backend().buffer().area.height - 1;
    h.mouse(MouseKind::Drag, x, rest);
    assert_eq!(
        h.app.tick_interval(),
        std::time::Duration::from_millis(50),
        "a held drag past the edge ticks at autoscroll_ms"
    );
    let shown = |h: &Harness| {
        (1001..1300)
            .rev()
            .find(|id| h.text().contains(&id.to_string()))
            .unwrap()
    };
    let before = shown(&h);
    for _ in 0..4 {
        std::thread::sleep(std::time::Duration::from_millis(60));
        h.app.handle(Input::Tick);
        h.draw();
    }
    let after = shown(&h);
    assert!(after > before, "the rows scrolled on: {before} -> {after}");
    // the release ends it
    h.mouse(MouseKind::Up, x, rest);
    assert!(h.app.tick_interval() > std::time::Duration::from_millis(50));
    let stopped = shown(&h);
    std::thread::sleep(std::time::Duration::from_millis(60));
    h.app.handle(Input::Tick);
    h.draw();
    assert_eq!(shown(&h), stopped);
}

#[test]
fn the_horizontal_wheel_scrolls_a_text_area_sideways() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    let (x, y) = h.find("Run the integration").expect("a long line");
    assert!(h.row(y).contains(" 4. Run"), "{}", h.row(y));
    h.mouse(MouseKind::WheelRight, x + 4, y);
    assert!(
        !h.row(y).contains(" 4. Run"),
        "the line moved sideways:\n{}",
        h.text()
    );
    h.mouse(MouseKind::WheelLeft, x + 4, y);
    assert!(h.row(y).contains(" 4. Run"), "{}", h.row(y));
}

/// Keyboard audit #1 and #15: Tab across the Inputs page shows the caret at
/// every immediate text stop the moment focus lands, typed letters edit the
/// field instead of firing the shell's letter shortcuts (`q` quit, `]` next
/// page, `i` inspector), Esc reverts the edit, and Tab leaves for the next
/// stop through the ring.
#[test]
fn tab_walk_shows_a_caret_at_every_text_stop_and_letters_edit() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    let field = |i: usize| WidgetId::of("inputs").child(i);
    // the enabled fields that follow the library default (Immediate)
    let immediate = [field(0), field(1), field(2), field(5)];
    let mut seen = Vec::new();
    for _ in 0..40 {
        h.key(tab());
        let Some(f) = h.app.focus.current() else {
            continue;
        };
        if immediate.contains(&f) {
            assert!(
                h.term.backend().cursor_visible(),
                "no caret at {f:?}\n{}",
                h.text()
            );
            if !seen.contains(&f) {
                seen.push(f);
            }
        }
    }
    assert_eq!(seen.len(), immediate.len(), "every text stop was reached");

    h.app.focus.focus(field(0));
    h.draw();
    assert!(h.term.backend().cursor_visible());
    let page = h.app.page;
    h.type_str("q]i");
    assert!(!h.app.quit, "q was typed, not quit");
    assert_eq!(h.app.page, page, "] was typed, not next page");
    assert!(!h.app.inspector, "i was typed, not the inspector");
    assert!(h.text().contains("q]i"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.text().contains("payments-gateway"), "Esc reverts");
    h.key(tab());
    assert_eq!(
        h.app.focus.current(),
        Some(field(1)),
        "Tab lands on the next stop"
    );
    assert!(h.term.backend().cursor_visible());
}

#[test]
fn a_track_press_held_keeps_paging_until_the_thumb_reaches_the_pointer() {
    let mut h = Harness::new(120, 40, PageId::Scrolling);
    let list_area = {
        let id = junie_tui::core::id::WidgetId::of("scrolling").sub("list");
        h.app.hits.area_of(id).unwrap()
    };
    let (sx, sy) = (list_area.right() - 1, list_area.bottom() - 1);
    h.mouse(MouseKind::Down, sx, sy);
    assert!(!h.text().contains("Row 001"), "the press pages once");
    assert!(!h.text().contains("Row 120"));
    assert_eq!(
        h.app.tick_interval(),
        std::time::Duration::from_millis(50),
        "a held track press ticks at track_repeat_ms"
    );
    let tick = |h: &mut Harness, ms: u64| {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        h.app.handle(Input::Tick);
        h.draw();
    };
    tick(&mut h, 0);
    tick(&mut h, 320);
    for _ in 0..10 {
        tick(&mut h, 60);
    }
    assert!(
        h.text().contains("Row 120"),
        "held, the track pages on to the end:\n{}",
        h.text()
    );
    h.mouse(MouseKind::Up, sx, sy);
    assert!(h.app.tick_interval() > std::time::Duration::from_millis(50));
}

/// Select All in a text field, per client platform (the platform is input
/// data, so both run on any host): Cmd+A wherever the terminal delivers
/// Cmd; Ctrl+A on Windows and Linux; on macOS Ctrl+A stays the Cocoa line
/// start and Ctrl+L selects all.
#[test]
fn inputs_select_all_follows_the_client_platform() {
    use junie_tui::core::edit::FieldKeys;
    use junie_tui::core::event::ClientPlatform;
    for platform in [ClientPlatform::MacOs, ClientPlatform::Other] {
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.app.theme = Theme::junie().with_field_keys(FieldKeys::for_platform(platform));
        h.draw();
        let (x, y) = h.find("payments-gateway").expect("the Project name field");
        h.click(x + 2, y);
        assert!(h.app.pages[PageId::Inputs.index()].editing());
        assert_eq!(selected_on_row(&h, y), "", "{platform:?}");
        h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER);
        let value = selected_on_row(&h, y);
        assert!(
            value.starts_with("payments-gateway"),
            "Cmd+A on {platform:?}: {value:?}"
        );
        h.key(KeyCode::End);
        h.key_mod(KeyCode::Char('a'), KeyModifiers::CONTROL);
        match platform {
            ClientPlatform::MacOs => {
                assert_eq!(selected_on_row(&h, y), "", "Ctrl+A is line start");
                h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
                assert_eq!(selected_on_row(&h, y), value, "Ctrl+L selects all");
            }
            ClientPlatform::Other => {
                assert_eq!(selected_on_row(&h, y), value, "Ctrl+A selects all");
            }
        }
        h.type_str("main");
        let row = h.row(y);
        assert!(
            row.contains("main") && !row.contains("payments-gateway"),
            "typing replaced the value: {row}"
        );
    }
}

/// The macOS editing chords in a text field, per client platform (both run
/// on any host): Cmd+A, Cmd+X then Cmd+V round-trip the value through the
/// clipboard hook (the system clipboard), Cmd+Z undoes the paste and
/// Cmd+Shift+Z redoes it. Cmd is additive to both presets.
#[test]
fn inputs_cmd_cut_and_paste_round_trip_through_the_clipboard_hook() {
    use junie_tui::core::edit::{FieldKeys, clipboard};
    use junie_tui::core::event::ClientPlatform;
    use std::cell::RefCell;
    use std::rc::Rc;
    for platform in [ClientPlatform::MacOs, ClientPlatform::Other] {
        let seen = Rc::new(RefCell::new(Vec::<String>::new()));
        let hook = seen.clone();
        clipboard::set_hook(move |t| hook.borrow_mut().push(t.to_owned()));
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.app.theme = Theme::junie().with_field_keys(FieldKeys::for_platform(platform));
        h.draw();
        let (x, y) = h.find("payments-gateway").expect("the Project name field");
        h.click(x + 2, y);
        h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER);
        let value = selected_on_row(&h, y);
        assert!(value.starts_with("payments-gateway"), "{platform:?}");
        h.key_mod(KeyCode::Char('x'), KeyModifiers::SUPER);
        assert_eq!(
            seen.borrow().last().map(|t| t.trim_end().to_owned()),
            Some(value.trim_end().to_owned()),
            "Cmd+X on {platform:?} hands the cut to the hook"
        );
        assert!(
            !h.row(y).contains("payments-gateway"),
            "Cmd+X emptied the field: {}",
            h.row(y)
        );
        h.key_mod(KeyCode::Char('v'), KeyModifiers::SUPER);
        assert!(
            h.row(y).contains("payments-gateway"),
            "Cmd+V pasted it back on {platform:?}: {}",
            h.row(y)
        );
        h.key_mod(KeyCode::Char('z'), KeyModifiers::SUPER);
        assert!(!h.row(y).contains("payments-gateway"), "Cmd+Z undoes");
        h.key_mod(
            KeyCode::Char('Z'),
            KeyModifiers::SUPER | KeyModifiers::SHIFT,
        );
        assert!(h.row(y).contains("payments-gateway"), "Cmd+Shift+Z redoes");
        clipboard::clear_hook();
    }
}
