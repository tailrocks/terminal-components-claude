//! The Inspector's State | Options pane and `--opt`, driven through the
//! real app (plan §7, SC1).

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::app::{App, PageId};
use crate::inspector::{self, Tab};
use crate::options::{self, Effect};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::{ColorLevel, Theme};

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

    fn key(&mut self, code: KeyCode) {
        self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
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

    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        (0..buf.area.height).find_map(|y| {
            let row = self.row(y);
            let i = row.find(needle)?;
            Some((row[..i].chars().count() as u16, y))
        })
    }

    fn focus_bars(&self) -> usize {
        let t = &self.app.theme;
        self.term
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|c| c.symbol() == t.glyphs.focus_bar)
            .count()
    }

    /// The cells of `area`, symbols and styles.
    fn cells(&self, area: Rect) -> Vec<ratatui::buffer::Cell> {
        let buf = self.term.backend().buffer();
        let mut out = Vec::new();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                out.push(buf[(x, y)].clone());
            }
        }
        out
    }

    fn open_options(&mut self) {
        self.key(KeyCode::Char('i'));
        assert!(self.app.inspector);
        self.app.insp.set_tab(Tab::Options);
        self.draw();
    }

    /// The area of option `key`'s control.
    fn control(&self, key: &str) -> Rect {
        let id = self.app.insp.options.control_id(key).expect("an option");
        self.app.hits.area_of(id).expect("drawn")
    }
}

fn opt(arg: &str) -> options::OptSet {
    options::parse_opt(arg).unwrap()
}

#[test]
fn the_options_tab_toggles_a_knob_and_the_playground_changes() {
    let mut h = Harness::new(160, 60, PageId::Buttons);
    h.open_options();
    let main = Rect::new(0, 2, 100, 50);
    let before = h.cells(main);
    // the glyph set: a Select; open it, choose ascii, the page redraws
    let field = h.control("shell.glyphs");
    h.click(field.x + 2, field.y);
    assert!(h.app.insp.options.popup_open(), "the list opens");
    assert_eq!(
        h.app.focus.current(),
        h.app.insp.options.control_id("shell.glyphs"),
        "the click focuses the select"
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.options.value("shell.glyphs"), "ascii");
    assert_eq!(
        h.app.theme.glyphs.focus_bar,
        junie_tui::theme::Glyphs::ASCII.focus_bar
    );
    assert_ne!(
        h.cells(main),
        before,
        "the playground redraws with ASCII glyphs"
    );
    // a toggle: Space flips it, through the PointerConfig builder
    let tog = h.control("shell.autoscroll");
    h.click(tog.x + 2, tog.y);
    assert!(!h.app.pointer.config.autoscroll);
    h.key(KeyCode::Char(' '));
    assert!(h.app.pointer.config.autoscroll);
}

#[test]
fn a_colour_level_flip_rerenders_at_that_level() {
    let mut h = Harness::new(160, 60, PageId::Overview);
    h.open_options();
    let field = h.control("shell.color");
    h.click(field.x + 2, field.y);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.theme.level, ColorLevel::Mono);
    assert!(h.text().contains("mono"), "the header names the level");
    // every cell is drawn at the mono level: no RGB colours
    let rgb = h
        .term
        .backend()
        .buffer()
        .content()
        .iter()
        .any(|c| matches!(c.fg, ratatui::style::Color::Rgb(..)));
    assert!(!rgb, "mono draws no truecolor cell");
}

#[test]
fn every_shell_option_flips_through_the_pane_and_through_opt() {
    for o in options::SHELL {
        for (i, value) in o.values.iter().enumerate() {
            // --opt
            let mut by_flag = Harness::new(160, 60, PageId::Overview);
            by_flag.app.apply_opt(opt(&format!("{}={value}", o.key)));
            by_flag.draw();
            // the pane: the same option through its control
            let mut by_pane = Harness::new(160, 60, PageId::Overview);
            by_pane.open_options();
            let id = by_pane.app.insp.options.control_id(o.key).unwrap();
            by_pane.app.focus.set(Some(id));
            by_pane.draw();
            let area = by_pane.control(o.key);
            if o.is_toggle() {
                if (i == 1) != (o.default == 1) {
                    by_pane.key(KeyCode::Char(' '));
                }
            } else {
                by_pane.click(area.x + 2, area.y);
                by_pane.key(KeyCode::Home);
                for _ in 0..i {
                    by_pane.key(KeyCode::Down);
                }
                by_pane.key(KeyCode::Enter);
            }
            for h in [&by_flag, &by_pane] {
                assert_eq!(h.app.options.get(o.key), i, "{}={value}", o.key);
            }
            assert_eq!(
                by_flag.app.live_config(),
                by_pane.app.live_config(),
                "{}={value}: one path",
                o.key
            );
            assert_eq!(by_flag.app.theme.level, by_pane.app.theme.level);
        }
    }
}

#[test]
fn below_100_columns_the_inspector_is_an_overlay() {
    let mut h = Harness::new(80, 24, PageId::Buttons);
    let before_focus = h.app.focus.current();
    h.key(KeyCode::Char('i'));
    assert!(h.app.inspector_overlay());
    assert_eq!(
        h.app.focus.current(),
        Some(h.app.insp.tabs.id),
        "the overlay takes focus"
    );
    assert!(h.find("Inspector").is_some(), "{}", h.text());
    assert!(h.find("State").is_some() && h.find("Options").is_some());
    assert_eq!(h.focus_bars(), 1, "one focus mark, in the overlay");
    // Tab stays inside the overlay
    for _ in 0..6 {
        h.key(KeyCode::Tab);
        assert!(h.app.insp.has_focus(h.app.focus.current()));
    }
    // Esc closes and gives focus back
    h.key(KeyCode::Esc);
    assert!(!h.app.inspector);
    assert_eq!(h.app.focus.current(), before_focus);
    assert!(h.find("Inspector ").is_none() || !h.text().contains("│ State"));
    // a click outside closes it too (light dismiss)
    h.key(KeyCode::Char('i'));
    assert!(h.app.inspector);
    h.click(2, 10);
    assert!(!h.app.inspector, "light dismiss");
}

#[test]
fn the_overlay_options_work_at_80_by_24() {
    let mut h = Harness::new(80, 24, PageId::Overview);
    h.key(KeyCode::Char('i'));
    h.key(KeyCode::Right);
    assert_eq!(h.app.insp.tab(), Tab::Options);
    h.key(KeyCode::Tab);
    let first = h.app.insp.options.control_id("shell.color").unwrap();
    assert_eq!(h.app.focus.current(), Some(first));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.theme.level, ColorLevel::Ansi256);
    assert!(h.app.inspector, "choosing keeps the overlay open");
    assert_eq!(h.focus_bars(), 1);
}

#[test]
fn docked_pane_focus_and_escape() {
    let mut h = Harness::new(160, 50, PageId::Overview);
    // the page draws a sample focus mark of its own; the real one is on
    // the sidebar now
    let marks = h.focus_bars();
    h.key(KeyCode::Char('i'));
    assert!(!h.app.inspector_overlay());
    // the docked pane's controls join the Tab order after the page
    let mut reached = false;
    for _ in 0..60 {
        h.key(KeyCode::Tab);
        if h.app.focus.current() == Some(h.app.insp.tabs.id) {
            reached = true;
            break;
        }
    }
    assert!(reached, "Tab reaches the Inspector");
    assert_eq!(h.focus_bars(), marks, "one real focus mark: {}", h.text());
    let tabs = h.app.hits.area_of(h.app.insp.tabs.id).unwrap();
    assert!(h.row(tabs.y).contains('▎'), "it is on the Inspector's tabs");
    h.key(KeyCode::Right);
    assert_eq!(h.app.insp.tab(), Tab::Options);
    // Esc leaves for navigation, the pane stays open
    h.key(KeyCode::Esc);
    assert!(h.app.inspector);
    assert!(!h.app.insp.has_focus(h.app.focus.current()));
    // `i` closes it
    h.key(KeyCode::Char('i'));
    assert!(!h.app.inspector);
}

#[test]
fn the_state_tab_logs_inputs() {
    let mut h = Harness::new(160, 50, PageId::Overview);
    h.key(KeyCode::Char('i'));
    h.mouse(MouseKind::Move, 40, 10);
    assert_eq!(h.app.insp.tab(), Tab::State);
    assert!(h.find("Events").is_some(), "{}", h.text());
    assert!(h.find("Move 40·10").is_some(), "{}", h.text());
    assert!(h.find("Key i").is_some(), "{}", h.text());
    assert!(h.app.insp.log.len() <= inspector::LOG_LEN);
}

#[test]
fn behaviour_options_change_the_live_configuration() {
    for o in options::SHELL
        .iter()
        .filter(|o| o.effect == Effect::Behaviour)
    {
        let base = Harness::new(120, 40, PageId::Overview).app.live_config();
        for (i, value) in o.values.iter().enumerate() {
            if i == o.default {
                continue;
            }
            let mut h = Harness::new(120, 40, PageId::Overview);
            h.app.apply_opt(opt(&format!("{}={value}", o.key)));
            assert_ne!(h.app.live_config(), base, "{}={value}", o.key);
        }
    }
}
