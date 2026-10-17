//! Text utilities (plan §7, SC26), driven through the real app: the
//! helpers follow the width and style options, the property list is
//! rebuilt through its builders and reports its actions.

use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Theme;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::props::PropsList;
use ratatui::layout::Position;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.open_page(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn opt(&mut self, arg: &str) {
        self.app.apply_opt(options::parse_opt(arg).unwrap());
        self.draw();
    }

    fn section(&mut self, s: Section) {
        let page = self.app.page();
        self.app.pages[page.index()]
            .template_mut()
            .unwrap()
            .set_section(s);
        self.draw();
    }

    fn key(&mut self, code: KeyCode) {
        self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
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

    /// Where `needle` starts on screen.
    fn find(&self, needle: &str) -> Option<(u16, u16)> {
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

    #[allow(dead_code)]
    fn cells(&self) -> Vec<ratatui::buffer::Cell> {
        self.term.backend().buffer().content().to_vec()
    }

    fn live_bars(&self) -> usize {
        self.text().matches('▎').count()
    }
}

impl Harness {
    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    #[allow(dead_code)]
    fn states(&self) -> Vec<SpecimenState> {
        self.app.pages[self.app.page().index()]
            .template()
            .unwrap()
            .state_rows()
            .to_vec()
    }

    /// The row holding `needle`, as text.
    fn row_of(&self, needle: &str) -> String {
        let (_, y) = self.find(needle).unwrap_or_else(|| panic!("{needle:?}"));
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }
}

impl Harness {
    fn list(&self) -> &PropsList {
        let page = self.app.pages[PageId::TextUtilities.index()]
            .as_any()
            .expect("the page exposes itself");
        page.downcast_ref::<crate::pages::text_utilities::TextUtilitiesPage>()
            .expect("text utilities page")
            .list()
    }
}

#[test]
fn helpers_follow_the_width_and_style_options() {
    let mut h = Harness::new(220, 80, PageId::TextUtilities);
    assert!(h.row_of("truncate_middle").contains("payments-plat…ackoff"));
    h.opt("textutilities.width=12");
    let row = h.row_of("truncate_middle");
    assert!(
        row.contains("payments…off "),
        "both ends kept in 12 cells: {row}"
    );
    assert!(h.text().contains("ui::text at width 12"));
    assert!(h.text().contains("thousands        1,234,567"));
    h.opt("textutilities.duration=compact");
    let row = h.row_of("duration Compact");
    assert!(row.contains("<1 ms · 840 ms · 1.25 s · 3m 2s"), "{row}");
    assert!(h.row_of("find_ranges").contains("0..6, 22..28"));
    h.opt("textutilities.case=on");
    let row = h.row_of("find_ranges");
    assert!(row.contains("case-sensitive → 22..28"), "{row}");
}

#[test]
fn wrap_options_change_the_wrapped_block() {
    let mut h = Harness::new(220, 80, PageId::TextUtilities);
    let indent = |h: &Harness| {
        let (x0, _) = h.find("Wrapping keeps").unwrap();
        let (x1, _) = h.find("An indented block").unwrap();
        x1 - x0
    };
    assert_eq!(indent(&h), 4, "the block keeps its indent");
    h.opt("textutilities.wrap=plain");
    assert!(h.text().contains("keep_indent false · hang_at_gap None"));
    assert_eq!(indent(&h), 0, "plain drops the indent");
    // the key list hangs its continuation under its second column
    h.opt("textutilities.wrap=default");
    let hang = |h: &Harness| {
        let (x0, _) = h.find("Ctrl+F").unwrap();
        let (x1, _) = h.find("Enter steps").unwrap();
        x1 - x0
    };
    assert_eq!(hang(&h), 10, "under `find`");
    h.opt("textutilities.wrap=no-hang");
    assert!(hang(&h) < 10, "no hanging indent");
}

#[test]
fn the_list_activates_and_copies_through_its_keymap() {
    let mut h = Harness::new(220, 80, PageId::TextUtilities);
    let (x, y) = h.find("Port  ").expect("the list");
    // M32: one click selects the row; a double click activates it
    h.click(x, y);
    assert!(h.app.shell.focus.is(WidgetId::of("textutilities.props")));
    assert!(!h.text().contains("Activate(Port)"), "a click only selects");
    h.click(x, y);
    assert!(h.text().contains("last PropsAction: Activate(Port)"));
    h.key(KeyCode::Char('y'));
    assert!(h.text().contains("last PropsAction: Copy(Port)"));
    // a custom key map: `c` copies, `y` is free
    h.opt("textutilities.copy_key=c");
    assert!(h.text().contains("copy c"));
    h.key(KeyCode::Up);
    h.key(KeyCode::Char('y'));
    assert!(
        h.text().contains("last PropsAction: Copy(Port)"),
        "y no longer copies"
    );
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("last PropsAction: Copy(Host)"));
}

#[test]
fn list_builders_rebuild_the_list() {
    let mut h = Harness::new(220, 80, PageId::TextUtilities);
    h.opt("textutilities.nav=vim");
    h.opt("textutilities.track_click=jump");
    h.opt("textutilities.scrollbar_gap=0");
    h.opt("textutilities.reveal=inline");
    h.opt("textutilities.sheet=narrow-labels");
    let text = h.text();
    assert!(
        text.contains(
            "PropsList · nav vim · copy y · track_click Jump · scrollbar_gap 0 · label share 25% · reveal InlineWrap"
        ),
        "{text}"
    );
    let l = h.list();
    assert_eq!(l.nav, junie_tui::core::keymap::NavKeys::VIM);
    assert_eq!(l.scrollbar_gap, 0);
    // vim letters move the cursor too
    let (x, y) = h.find("Port  ").unwrap();
    h.click(x, y);
    h.key(KeyCode::Char('j'));
    assert_eq!(h.list().cursor, 2);
    h.key(KeyCode::Down);
    assert_eq!(h.list().cursor, 3);
}

#[test]
fn list_states_are_drawn_by_the_list() {
    let mut h = Harness::new(220, 80, PageId::TextUtilities);
    h.section(Section::States);
    let rows = h.app.pages[PageId::TextUtilities.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Focused,
        SpecimenState::Overflow,
        SpecimenState::Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }
    let text = h.text();
    assert!(
        text.contains("y copy"),
        "a focused copyable row offers copy"
    );
    assert!(h.live_bars() <= 1, "{text}");
}

/// Not a check: prints the page for review (`--no-capture`).
#[test]
fn print_text_utilities() {
    if std::env::var("PRINT_PAGES").is_err() {
        return;
    }
    for (w, ht) in [(120, 40), (220, 80)] {
        let mut h = Harness::new(w, ht, PageId::TextUtilities);
        for s in Section::ALL {
            h.section(s);
            println!("{}", h.text());
        }
    }
}
