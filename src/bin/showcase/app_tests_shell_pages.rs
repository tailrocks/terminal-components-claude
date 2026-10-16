//! App shell pages (plan §7, SC5 and SC6), driven through the real app:
//! Breadcrumbs & rails, Help & keys, Quit & leave.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;
use junie_tui::ui::specimen::SpecimenState;
use ratatui::layout::Position;

pub(crate) struct Harness {
    pub app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    pub fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    pub fn opt(&mut self, arg: &str) {
        self.app.apply_opt(options::parse_opt(arg).unwrap());
        self.draw();
    }

    pub fn section(&mut self, s: Section) {
        let page = self.app.page;
        self.app.pages[page.index()]
            .template_mut()
            .unwrap()
            .set_section(s);
        self.draw();
    }

    pub fn key_mods(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    pub fn key(&mut self, code: KeyCode) {
        self.key_mods(code, KeyModifiers::NONE);
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

    /// Where `needle` starts on screen.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        for y in 0..buf.area.height {
            let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            if let Some(i) = row.find(needle) {
                let x = row[..i].chars().count() as u16;
                return Some((x, y));
            }
        }
        None
    }

    /// The row holding `needle`, as text.
    pub fn row_of(&self, needle: &str) -> String {
        let (_, y) = self.find(needle).unwrap_or_else(|| panic!("{needle:?}"));
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    pub fn states(&self) -> Vec<SpecimenState> {
        self.app.pages[self.app.page.index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec()
    }

    pub fn live_bars(&self) -> usize {
        self.text().matches('▎').count()
    }
}

/// Print a page at 120×40 (`--no-capture`), for sketches.
#[test]
fn dump_shell_pages() {
    if std::env::var("SHOWCASE_DUMP").is_err() {
        return;
    }
    let page = std::env::var("SHOWCASE_DUMP").unwrap();
    let id = crate::app::PageId::from_name(&page).expect("page");
    let mut h = Harness::new(120, 40, id);
    if let Ok(opts) = std::env::var("SHOWCASE_OPTS") {
        for o in opts.split(',').filter(|o| !o.is_empty()) {
            h.opt(o);
        }
    }
    if let Ok(s) = std::env::var("SHOWCASE_SECTION") {
        h.section(match s.as_str() {
            "options" => Section::Options,
            "states" => Section::States,
            _ => Section::Playground,
        });
    }
    println!("{}", h.text());
}

// ------------------------------------------------------------ SC5 ----

mod breadcrumbs_rails {
    use super::*;
    use crate::pages::breadcrumbs_rails::LIST;

    fn page() -> Harness {
        Harness::new(120, 40, PageId::BreadcrumbsRails)
    }

    #[test]
    fn enter_descends_and_a_crumb_goes_back() {
        let mut h = page();
        let (x, y) = h.find("Projects").expect("the folder list");
        h.click(x, y);
        assert!(h.app.focus.is(LIST));
        assert!(
            h.text().contains("Disk › Users › alex › Projects"),
            "{}",
            h.text()
        );
        // Enter opens the folder under the cursor
        let (x, y) = h
            .find("payments-plat")
            .unwrap_or_else(|| panic!("{}", h.text()));
        h.click(x, y);
        assert!(h.text().contains("› Projects › payments-platform"));
        // a click on an earlier level goes back to it
        let (x, y) = h.find("Users ›").expect("the trail");
        h.click(x, y);
        assert!(h.row_of("Disk ›").contains("Disk › Users"));
        assert!(!h.text().contains("› alex ›"));
        assert!(h.text().contains("Users (level 1)"));
        // Backspace walks up a level
        h.app.focus.focus(LIST);
        h.key(KeyCode::Backspace);
        assert!(h.text().contains("Backspace → level 0"));
        assert!(h.find("Applications").is_some());
    }

    #[test]
    fn a_plain_trail_takes_no_click() {
        let mut h = page();
        h.opt("breadcrumbsrails.clickable=off");
        assert!(h.text().contains("clickable(false)"));
        let (x, y) = h.find("Disk ›").unwrap();
        h.click(x, y);
        assert!(h.text().contains("Last crumb    —") || h.row_of("Last crumb").contains('—'));
    }

    #[test]
    fn the_rail_collapses_and_expands_by_pointer_and_ctrl_b() {
        let mut h = page();
        // ‹ in the explorer's frame collapses it
        let (x, y) = h.find("‹").expect("the collapse button");
        h.click(x, y);
        assert!(h.row_of("Last RailAction").contains("Collapse"));
        assert!(h.find("─ Explorer").is_none(), "the frame is gone");
        // the rail: `›` over the name written down the column
        let (x, y) = h.find("›    Location").expect("the rail");
        let buf = h.term.backend().buffer().clone();
        assert_eq!(buf[(x, y + 2)].symbol(), "E");
        // a click anywhere on it opens the sidebar again and focuses the list
        h.click(x, y + 4);
        assert!(h.row_of("Last RailAction").contains("Expand"));
        assert!(h.app.focus.is(LIST));
        h.key_mods(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(h.row_of("Last RailAction").contains("Collapse"));
        assert!(!h.app.focus.is(LIST), "focus leaves the hidden list");
        h.key_mods(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert!(h.app.focus.is(LIST));
    }

    #[test]
    fn without_a_rail_a_collapsed_sidebar_leaves_nothing() {
        let mut h = page();
        h.opt("breadcrumbsrails.collapsed=on");
        let with = h.text();
        h.opt("breadcrumbsrails.collapsed_rail=off");
        assert_ne!(with, h.text());
        assert!(
            h.row_of("collapsed_rail")
                .contains("off: only the keyboard")
        );
    }

    #[test]
    fn the_deep_trail_collapses_and_cuts_its_current_page() {
        let mut h = page();
        h.opt("breadcrumbsrails.width=32");
        let row = h.row_of("settlement");
        assert!(row.contains("… › "), "{row}");
        h.opt("breadcrumbsrails.width=20");
        let row = h.row_of("settlement");
        assert!(row.contains("v2.md"), "middle cut keeps the end: {row}");
    }

    #[test]
    fn states_rows_are_drawn() {
        let mut h = page();
        h.section(Section::States);
        let rows = h.states();
        for s in [
            SpecimenState::Rest,
            SpecimenState::Hover,
            SpecimenState::Pressed,
        ] {
            assert!(rows.contains(&s), "{s:?} in {rows:?}");
        }
        assert!(h.live_bars() <= 1);
    }
}
