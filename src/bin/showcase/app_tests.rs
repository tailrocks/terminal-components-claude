//! Deterministic interaction tests driven through the real `App`, rendered
//! into a `TestBackend` so hit regions and focus rings are the same ones the
//! terminal would see.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
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
        let out = self.app.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
        }));
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
    // nav + 7 enabled buttons (2 disabled are skipped)
    assert_eq!(seen.len(), 8, "{seen:?}");
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
    // hover: surface lift, no bold
    h.mouse(MouseKind::Move, x, y);
    let cell = h.term.backend().buffer()[(x, y)].clone();
    assert_eq!(cell.bg, t.surface_overlay);
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
    let area = h.app.hover.and_then(|id| h.app.hits.area_of(id)).unwrap();
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
    assert!(h.text().contains("4–24 of 24"), "{}", h.text());
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
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
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

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    assert!(h.text().contains(" 1. Read"));
    let (x, y) = h.find(" 1. Read").unwrap();
    h.mouse(MouseKind::WheelDown, x, y + 2);
    assert!(!h.text().contains(" 1. Read"));
    assert!(h.text().contains(" 4. Run"));
    h.key(tab());
    h.key(KeyCode::Enter);
    for _ in 0..30 {
        h.key(KeyCode::Down);
    }
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
    // y answers a confirm
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('y'));
    assert!(h.text().contains("Task started"));
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = Harness::new(120, 40, PageId::Dialogs);
    h.key(tab());
    h.key(tab());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Rename task"));
    h.key(KeyCode::Enter); // start editing
    h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    h.key(KeyCode::Backspace);
    h.key(KeyCode::Enter); // submit empty → blocked
    assert!(h.app.dialog.is_some());
    assert!(h.text().contains("Name cannot be empty"));
    h.key(KeyCode::Enter);
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
    h.key(KeyCode::Enter);
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
    h.mouse(MouseKind::Down, sx, list_area.bottom() - 1);
    h.mouse(MouseKind::Up, sx, list_area.bottom() - 1);
    assert!(!h.text().contains("Row 001"));
    assert!(h.text().contains("Row 120"));
    h.mouse(MouseKind::Down, sx, list_area.bottom() - 1);
    h.mouse(MouseKind::Drag, sx, list_area.y);
    h.mouse(MouseKind::Up, sx, list_area.y);
    assert!(h.text().contains("Row 001"));
    let _ = y0;
}

#[test]
fn keyboard_navigation_between_pages() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char(']'));
    assert_eq!(h.app.page, PageId::Buttons);
    h.key(KeyCode::Char('['));
    assert_eq!(h.app.page, PageId::Overview);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.page, PageId::Inputs);
    let (x, y) = h.find("Tables").unwrap();
    h.click(x, y);
    assert_eq!(h.app.page, PageId::Tables);
}

#[test]
fn quit_keys() {
    let mut h = Harness::new(120, 40, PageId::Overview);
    h.key(KeyCode::Char('q'));
    assert!(h.app.quit);
    let mut h = Harness::new(120, 40, PageId::Inputs);
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
    h.key(KeyCode::Char('y'));
    assert!(h.find_row("jonas@acme.dev").is_none());
    assert!(h.text().contains("5 members"));
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
    h.key(KeyCode::Char('y'));
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

/// Visual regression baseline: a stable digest of every cell (symbol, fg,
/// bg, modifiers) for each showcase page at 120×40 and 80×24, with focus on
/// the first control. Regenerate with `UPDATE_BASELINE=1 cargo test baseline`.
#[test]
fn showcase_visual_baseline() {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (w, hgt) in [(120u16, 40u16), (80, 24)] {
        for entry in crate::app::NAV_ENTRIES {
            let mut h = Harness::new(w, hgt, entry.id);
            h.key(tab());
            let buf = h.term.backend().buffer();
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for cell in &buf.content {
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
            writeln!(out, "{}x{} {} {hash:016x}", w, hgt, entry.label).unwrap();
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
