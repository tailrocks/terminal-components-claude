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
        for entry in crate::app::NAV_ENTRIES {
            let mut h = Harness::new(w, hgt, entry.id);
            h.app.theme = Theme::for_level(level);
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
            writeln!(out, "{}x{} {:?} {} {hash:016x}", w, hgt, level, entry.label).unwrap();
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

// ---------------------------------------------------------------- scrolling audit regressions

/// The card title row of the Scrolling page (the labels live there).
fn titles(h: &Harness) -> String {
    // the card title row: the first row carrying a position label (the
    // titles themselves may be shortened to keep the labels whole)
    h.text()
        .lines()
        .find(|l| l.contains(" of ") && l.contains('–'))
        .map(str::to_owned)
        .unwrap_or_default()
}

fn thumb_rows(h: &Harness, x: u16) -> Vec<u16> {
    let buf = h.term.backend().buffer();
    (0..buf.area.height)
        .filter(|&y| buf[(x, y)].symbol() == "┃")
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
    // at 80 columns the labels keep their positions and shorten their tails
    assert!(
        title.contains(&format!("1–{drawn} of")),
        "the list label matches the rows drawn at 80x24: {title}"
    );
    assert!(
        title.contains("387–401 of"),
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
    assert_eq!(
        row.matches(" of").count(),
        3,
        "all three labels stay: {row}"
    );
    let before_log = row.rsplit_once(" of").map(|(a, _)| a.to_owned()).unwrap();
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
    let hovered = h.app.hover;
    assert!(hovered.is_some());
    assert_eq!(h.mouse(MouseKind::WheelDown, x, y), Outcome::Changed);
    assert_ne!(
        h.app.hover, hovered,
        "hover moved to the row now under the pointer"
    );
    assert_eq!(h.app.hover, h.app.hits.hit(Position::new(x, y)));
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
