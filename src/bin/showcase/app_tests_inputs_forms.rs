//! The "Inputs & forms" widget pages through the real app (plan §7, SC7
//! Buttons, SC8 Inputs and Text areas, SC9 Checks & radios, SC10 Chips &
//! selects): options set through `--opt` change the playground, the States
//! rows render, and the key interactions work.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

#[allow(dead_code)] // each page's tests use what they need
impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    fn page(&self) -> PageId {
        self.app.page
    }

    fn opt(&mut self, arg: &str) {
        self.app
            .apply_opt(options::parse_opt(arg).unwrap_or_else(|e| panic!("{arg}: {e}")));
        self.draw();
    }

    fn section(&mut self, s: Section) {
        let p = self.page();
        self.app.pages[p.index()]
            .template_mut()
            .expect("templated page")
            .set_section(s);
        self.draw();
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

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
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

    /// Column and row of `needle`'s first cell.
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
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()))
    }

    fn status(&self) -> String {
        self.app.status.text().unwrap_or_default().to_owned()
    }
}

/// Print every section of the pages in `DUMP_PAGES` at 120×40, after the
/// `--opt` values in `DUMP_OPTS` (`--run-ignored only --no-capture`).
#[test]
#[ignore]
fn dump_pages() {
    let pages = std::env::var("DUMP_PAGES").unwrap_or_else(|_| "buttons".into());
    for name in pages.split(',') {
        let page = PageId::from_name(name).unwrap();
        let mut h = Harness::new(120, 40, page);
        if let Ok(opts) = std::env::var("DUMP_OPTS") {
            for o in opts.split(',').filter(|o| !o.is_empty()) {
                h.opt(o);
            }
        }
        for s in Section::ALL {
            h.section(s);
            println!("==== {name} · {}\n{}", s.label(), h.text());
        }
    }
}

// ------------------------------------------------------------------ SC7

fn tool_id(i: usize) -> junie_tui::core::id::WidgetId {
    junie_tui::core::id::WidgetId::of("buttons")
        .sub("tools")
        .child(i)
}

/// The focus bar (`▎` in the focus ink) on row `y`, if any.
fn focus_bar_on(h: &Harness, y: u16) -> bool {
    let buf = h.term.backend().buffer();
    (0..buf.area.width).any(|x| {
        let c = &buf[(x, y)];
        c.symbol() == "▎" && c.fg == Theme::junie().focus
    })
}

#[test]
fn buttons_toolbar_is_off_the_tab_order_until_the_option_puts_it_in() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    h.at("▶ Run");
    assert!(
        !h.app.ring.contains(tool_id(0)),
        "toolbar buttons take clicks only"
    );
    assert!(h.text().contains("not in the Tab order"));
    h.opt("buttons.toolbar_focus=on");
    assert!(h.app.ring.contains(tool_id(0)));
    assert!(!h.text().contains("not in the Tab order"));
}

#[test]
fn buttons_toolbar_runs_and_cancels_by_shortcut_and_click() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    // Cancel is disabled until a query runs: a click does nothing
    let (ex, y) = h.at("Explain");
    let cancel = ex + "Explain".len() as u16 + 4;
    h.click(cancel, y);
    assert!(!h.status().contains("cancelled"), "{}", h.status());
    h.key(KeyCode::F(5));
    assert!(h.status().contains("Running"), "{}", h.status());
    h.click(cancel, y);
    assert_eq!(h.status(), "Query cancelled");
    h.key(KeyCode::F(5));
    h.key_mod(KeyCode::F(5), KeyModifiers::SHIFT);
    assert_eq!(h.status(), "Query cancelled");
    let (rx, ry) = h.at("▶ Run");
    h.click(rx + 1, ry);
    assert!(h.status().contains("Running"), "{}", h.status());
}

#[test]
fn buttons_focus_visible_keyboard_only_hides_the_ring_after_a_click() {
    for (value, ring) in [("always", true), ("keyboard-only", false)] {
        let mut h = Harness::new(120, 40, PageId::Buttons);
        h.opt(&format!("buttons.focus_visible={value}"));
        let (x, y) = h.at("Preview");
        h.click(x, y);
        assert!(h.status().contains("Preview"), "{}", h.status());
        assert_eq!(focus_bar_on(&h, y), ring, "{value}: {}", h.row(y));
    }
}

#[test]
fn buttons_row_folds_wraps_and_aligns_through_the_options() {
    let mut h = Harness::new(120, 60, PageId::Buttons);
    assert!(h.text().contains("every button fits"));
    h.opt("buttons.row_width=30");
    assert!(
        h.text().contains("behind …: Validate, Duplicate, Edit…"),
        "{}",
        h.text()
    );
    // the `…` button names what folded
    let (x, y) = h.at("…    Remove…");
    h.click(x, y);
    assert_eq!(h.status(), "3 more: Validate, Duplicate, Edit…");
    // wrapping keeps every label on screen
    h.opt("buttons.row_overflow=wrap");
    for label in ["Refresh", "Validate", "Duplicate", "Edit…", "Remove…"] {
        assert!(h.find(label).is_some(), "{label}");
    }
    let (_, refresh) = h.at("Refresh");
    let (_, remove) = h.at("Remove…");
    assert!(remove > refresh, "the row wraps onto a second line");
    // End packs the row against its right edge
    h.opt("buttons.row_overflow=more");
    let start = h.at("Refresh").0;
    h.opt("buttons.row_align=end");
    assert!(h.at("Refresh").0 > start);
}

#[test]
fn buttons_options_section_drives_the_playground() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let start = h.at("Refresh").0;
    h.section(Section::Options);
    let (x, y) = h.at("start");
    h.click(x, y);
    // the list is open; End is its second option
    let (ex, ey) = h.at("end ");
    h.click(ex, ey);
    assert_eq!(
        h.app.pages[PageId::Buttons.index()].option_value("buttons.row_align"),
        Some(1)
    );
    h.section(Section::Playground);
    assert!(h.at("Refresh").0 > start);
}

#[test]
fn buttons_states_grid_follows_its_options_and_shows_the_icon_button() {
    let mut h = Harness::new(160, 50, PageId::Buttons);
    h.section(Section::States);
    assert!(h.text().contains("Icon ×"));
    let (x15, _) = h.at("Secondary");
    h.opt("buttons.label_width=20");
    assert_eq!(h.at("Secondary").0, x15 + 5);
    let marks = |h: &Harness| h.text().matches('▏').count();
    assert!(marks(&h) > 0);
    h.opt("buttons.focus_mark=colon");
    assert_eq!(marks(&h), 0);
}

/// The States legend explains the mark the grid draws: `:` under ASCII
/// glyphs or a `focus_mark` override, never a `▏` the grid no longer shows.
#[test]
fn states_legend_names_the_mark_the_grid_draws() {
    for opt in ["shell.glyphs=ascii", "buttons.focus_mark=colon"] {
        let mut h = Harness::new(160, 50, PageId::Buttons);
        h.section(Section::States);
        h.opt(opt);
        let text = h.text();
        assert!(text.contains(": depicts a focused sample"), "{opt}: {text}");
        assert!(!text.contains('▏'), "{opt}: {text}");
    }
}

#[test]
fn buttons_icon_pad_sets_the_plate() {
    let mut h = Harness::new(120, 60, PageId::Buttons);
    let gap = |h: &Harness| {
        let (_, y) = h.at("Danger hover");
        let row = h.row(y);
        let i = row.find("Danger hover").unwrap() + "Danger hover".len();
        row[i..].chars().take_while(|c| *c != '×').count()
    };
    let one = gap(&h);
    h.opt("buttons.icon_pad=2");
    assert_eq!(gap(&h), one + 1);
}
