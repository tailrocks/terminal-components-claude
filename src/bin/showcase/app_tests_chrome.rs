//! Chrome and Status & hints (plan §7, SC4), driven through the real app:
//! the menu bar's options rebuild it through its builders, the identity
//! strip and status bar show their parts, the hint bar resolves layers and
//! drops hints by priority.

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

    fn opt(&mut self, arg: &str) {
        self.app.apply_opt(options::parse_opt(arg).unwrap());
        self.draw();
    }

    fn section(&mut self, s: Section) {
        let page = self.app.page;
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

    fn states(&self) -> Vec<SpecimenState> {
        self.app.pages[self.app.page.index()]
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

#[test]
fn menu_bar_options_rebuild_the_bar_through_its_builders() {
    let mut h = Harness::new(160, 50, PageId::Chrome);
    let text = h.text();
    assert!(text.contains("open_on Press"), "{text}");
    assert!(text.contains("brand lockup"));
    h.opt("chrome.hover_switch=off");
    h.opt("chrome.nav=vim");
    h.opt("chrome.dismiss=escape");
    let text = h.text();
    assert!(text.contains("hover_switch off"), "{text}");
    assert!(text.contains("nav vim"));
    assert!(text.contains("dismiss escape"));
    h.opt("chrome.brand=none");
    assert!(!h.text().contains("app❯"));
}

#[test]
fn open_on_release_waits_for_the_click() {
    let mut h = Harness::new(160, 50, PageId::Chrome);
    let (x, y) = h.find("File").unwrap();
    // OpenOn::Press (default): the menu is open while the button is held
    h.mouse(MouseKind::Down, x, y);
    assert!(h.text().contains("New tab"), "{}", h.text());
    h.mouse(MouseKind::Up, x, y);
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("New tab"));
    h.opt("chrome.open_on=release");
    assert!(h.text().contains("open_on Release"));
    h.mouse(MouseKind::Down, x, y);
    assert!(!h.text().contains("New tab"), "the press does nothing");
    h.mouse(MouseKind::Up, x, y);
    assert!(h.text().contains("New tab"), "the click opens it");
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("last MenuBarAction: Closed"),
        "{}",
        h.text()
    );
}

#[test]
fn a_narrow_bar_hides_or_collapses_its_titles() {
    let mut h = Harness::new(160, 50, PageId::Chrome);
    h.opt("chrome.bar=narrow");
    let bar = h.row_of("app❯");
    assert!(!bar.contains("Help"), "{bar}");
    assert!(!bar.contains('»'), "Overflow::Hide draws no chevron: {bar}");
    h.opt("chrome.overflow=chevron");
    let bar = h.row_of("app❯");
    assert!(bar.contains('»'), "Overflow::Chevron: {bar}");
    assert!(h.text().contains("overflow Chevron"));
}

#[test]
fn the_environment_segment_opens_the_app_menu_where_it_was_clicked() {
    let mut h = Harness::new(160, 50, PageId::Chrome);
    let (x, y) = h.find("◆ production").unwrap();
    h.click(x + 2, y);
    let (mx, my) = h.find("Switch to staging").expect("the app menu opens");
    assert!(my > y && my <= y + 3, "under the segment");
    assert!(mx.abs_diff(x) < 6, "at the segment");
    h.click(mx + 1, my);
    assert!(h.find("Copy cluster URL").is_none(), "choosing closes it");
    assert!(h.text().contains("production › Switch to staging"));
}

#[test]
fn the_status_bar_places_items_by_priority() {
    let h = Harness::new(220, 80, PageId::Chrome);
    let text = h.text();
    assert!(
        text.contains("places 8 of 8 · Left 2, Center 1, Right 5"),
        "{text}"
    );
    // busy spinner, meter and stale meter all draw
    assert!(text.contains("quota ╍"));
    assert!(text.contains("disk ━"));
    let h = Harness::new(100, 40, PageId::Chrome);
    let row = h.row_of("places");
    assert!(!row.contains("places 8 of 8"), "{row}");
    assert!(row.contains("Center 0"), "the center leaves first: {row}");
}

#[test]
fn chrome_states_show_segments_and_status_items() {
    let mut h = Harness::new(220, 80, PageId::Chrome);
    h.section(Section::States);
    let rows = h.states();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Pressed,
        SpecimenState::Busy,
        SpecimenState::Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }
    let text = h.text();
    assert!(text.contains("Clickable segment") && text.contains("Status meter"));
    assert!(h.live_bars() <= 1);
}

#[test]
fn the_topmost_hint_layer_wins() {
    let mut h = Harness::new(160, 50, PageId::StatusHints);
    assert!(h.row_of("resolved →").contains("Enter Open"));
    h.opt("statushints.layer=menu");
    assert!(h.row_of("resolved →").contains("Enter Choose"));
    h.opt("statushints.layer=modal");
    let row = h.row_of("resolved →");
    assert!(row.contains("Enter Delete"), "{row}");
    assert!(h.row_of("modal layer").contains("Esc Cancel"));
}

#[test]
fn a_narrow_bar_keeps_pinned_hints_longest() {
    let mut h = Harness::new(160, 50, PageId::StatusHints);
    h.opt("statushints.width=narrow");
    assert!(!h.row_of("resolved →").contains("F5"), "auto: F5 drops");
    h.opt("statushints.priority=pinned");
    let row = h.row_of("resolved →");
    assert!(row.contains("F5 Refresh"), "Hint::pinned stays: {row}");
    h.opt("statushints.compact=on");
    assert!(h.text().contains("compact on"));
    let before = h.cells();
    h.opt("statushints.compact=off");
    assert_ne!(before, h.cells(), "compact shows keys only");
}

#[test]
fn status_badge_and_alignment_apply_to_the_resolved_bar() {
    let mut h = Harness::new(160, 50, PageId::StatusHints);
    h.opt("statushints.status=error");
    assert!(h.row_of("resolved →").contains("! Connection lost"));
    h.opt("statushints.badge=on");
    assert!(h.row_of("resolved →").contains("EDIT"));
    let (x0, _) = h.find("resolved →").unwrap();
    let start = h.row_of("resolved →").find("Enter").unwrap();
    h.opt("statushints.status=none");
    h.opt("statushints.badge=off");
    h.opt("statushints.align=center");
    let centred = h.row_of("resolved →").find("Enter").unwrap();
    assert!(centred > start, "centred hints move right");
    let _ = x0;
    h.opt("statushints.hidden=on");
    assert!(h.text().contains("Ctrl+H Hide hidden"));
}

#[test]
fn hint_bar_states_are_drawn_by_the_layer() {
    let mut h = Harness::new(220, 80, PageId::StatusHints);
    h.section(Section::States);
    let rows = h.states();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Overflow,
        SpecimenState::Error,
        SpecimenState::Editing,
        SpecimenState::Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }
    let text = h.text();
    assert!(text.contains("! Connection lost"));
    assert!(text.contains("EDIT"));
    assert!(text.contains("Pinned F5"));
}

/// Not a check: prints the pages for review (`--no-capture`).
#[test]
fn print_chrome_and_status_hints() {
    if std::env::var("PRINT_PAGES").is_err() {
        return;
    }
    for page in [PageId::Chrome, PageId::StatusHints] {
        for (w, ht) in [(120, 40), (220, 80)] {
            let mut h = Harness::new(w, ht, page);
            for s in Section::ALL {
                h.section(s);
                println!("{}", h.text());
            }
        }
    }
}
