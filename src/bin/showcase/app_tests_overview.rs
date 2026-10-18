



use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key};
use junie_tui::theme::Theme;
use junie_tui::ui::specimen::SpecimenState;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

impl Harness {
    fn new(w: u16, h: u16) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(PageId::Overview);
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
        self.app.pages[PageId::Overview.index()]
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

#[test]
fn playground_shows_every_card_at_full_size() {
    let h = Harness::new(220, 80);
    let text = h.text();
    for title in [
        "Tokens · planes and text",
        "Tokens · accent and status",
        "Glyphs and strings",
        "Ladders · theme.ladders",
        "Contrast on the card, per level",
        "Principles",
        "Roles",
        "Explorer",
        "State modifiers · theme.mods",
        "Theme settings",
        "State language",
        "Spacing · buttons · badges",
    ] {
        assert!(text.contains(title), "{title} missing:\n{text}");
    }
}

#[test]
fn playground_scrolls_by_card_band_on_a_short_terminal() {
    let mut h = Harness::new(120, 40);
    assert!(h.text().contains("Tokens · planes and text"));
    assert!(!h.text().contains("Spacing · buttons · badges"));
    for _ in 0..20 {
        h.app
            .handle(Input::Mouse(junie_tui::core::event::Mouse::new(
                junie_tui::core::event::MouseKind::WheelDown,
                ratatui::layout::Position::new(80, 20),
            )));
    }
    h.draw();
    assert!(
        h.text().contains("Spacing · buttons · badges"),
        "{}",
        h.text()
    );
}



fn sample_bg(h: &Harness, row: u16) -> ratatui::style::Color {
    let (x, y) = h.find("plane ").expect("explorer");
    h.term.backend().buffer()[(x + 11, y + 3 + row)].bg
}

#[test]
fn explorer_follows_the_kind_and_plane_options() {
    let mut h = Harness::new(220, 80);
    assert!(h.text().contains("plane Card · kind Secondary"));
    let before = sample_bg(&h, 2);
    h.opt("overview.kind=primary");
    assert!(h.text().contains("plane Card · kind Primary"));

    let t = Theme::junie();
    assert_ne!(before, sample_bg(&h, 2));
    assert_eq!(sample_bg(&h, 2), t.ladders.accent.pressed);
    h.opt("overview.plane=popover");
    assert!(h.text().contains("plane Popover · kind Primary"));
}

#[test]
fn custom_ladder_repaints_the_explorer_through_with_ladder() {
    let mut h = Harness::new(220, 80);

    h.opt("overview.kind=row");
    let theme_hover = sample_bg(&h, 1);
    h.opt("overview.ladder=custom");
    assert!(h.text().contains("ladder custom (with_ladder)"));
    let t = Theme::junie();
    assert_ne!(theme_hover, sample_bg(&h, 1));
    assert_eq!(sample_bg(&h, 1), t.accent_bg);
}

#[test]
fn state_modifiers_option_carries_state_in_the_explorer() {
    let mut h = Harness::new(220, 80);
    let before = h.cells();
    h.opt("overview.mods=mono");
    assert!(h.text().contains("modifiers MONO"));
    assert_ne!(before, h.cells());
}

#[test]
fn states_grid_draws_every_kind_with_one_focus_bar() {
    let mut h = Harness::new(220, 80);
    h.section(Section::States);
    let text = h.text();
    for label in [
        "Row",
        "Cursor fill",
        "Cursor bar",
        "Toggle",
        "Handle",
        "Seam",
    ] {
        assert!(text.contains(label), "{label}:\n{text}");
    }
    let rows = h.app.pages[PageId::Overview.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Pressed,
        SpecimenState::Focused,
        SpecimenState::Disabled,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }

    assert!(h.live_bars() <= 1, "{text}");
    let before = h.cells();
    h.opt("overview.plane=accent");
    assert_ne!(before, h.cells(), "the grid follows the plane");
}

#[test]
fn options_section_sets_an_option_like_opt() {
    let mut h = Harness::new(160, 50);
    h.section(Section::Options);
    let text = h.text();
    for label in ["Plane", "Kind", "Ladder", "State modifiers"] {
        assert!(text.contains(label), "{label}:\n{text}");
    }

    let strip = h.app.pages[PageId::Overview.index()]
        .template()
        .unwrap()
        .tabs
        .id;
    h.app.shell.focus.focus(strip);
    h.draw();
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    assert_eq!(
        h.app.pages[PageId::Overview.index()].option_value("overview.plane"),
        Some(3),
        "{}",
        h.text()
    );
    assert_eq!(h.live_bars(), 1, "{}", h.text());
}


#[test]
fn print_overview() {
    if std::env::var("PRINT_PAGES").is_err() {
        return;
    }
    for (w, ht) in [(120, 40), (220, 80)] {
        let mut h = Harness::new(w, ht);
        for s in Section::ALL {
            h.section(s);
            println!("{}", h.text());
        }
    }
}
