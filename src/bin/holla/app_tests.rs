//! End-to-end interaction tests through the real App on a TestBackend.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::theme::Theme;

use crate::app::{App, TabKind};
use crate::scenario::{Motion, Scenario};

pub struct H {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

impl H {
    pub fn new(scenario: Scenario, motion: Motion, frame: u64, w: u16, h: u16) -> Self {
        let app = App::for_scenario(scenario, motion, frame, Theme::junie());
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut hh = Self { app, term };
        hh.draw();
        hh
    }
    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }
    pub fn key(&mut self, code: KeyCode) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
        o
    }
    pub fn ctrl(&mut self, code: KeyCode) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::CONTROL,
        }));
        self.draw();
        o
    }
    pub fn alt(&mut self, code: KeyCode) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::ALT,
        }));
        self.draw();
        o
    }
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }
    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.app.handle(Input::Tick);
        }
        self.draw();
    }
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
        let o = self.app.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
        }));
        self.draw();
        o
    }
    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }
    pub fn resize(&mut self, w: u16, h: u16) {
        self.term.backend_mut().resize(w, h);
        self.app.handle(Input::Resize(w, h));
        self.draw();
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
    pub fn row(&self, y: u16) -> String {
        self.text().lines().nth(y as usize).unwrap_or("").to_owned()
    }
    pub fn last_row(&self) -> String {
        let t = self.text();
        t.lines().next_back().unwrap_or("").to_owned()
    }
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
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
    pub fn tab_kind(&self) -> TabKind {
        self.app.tabs[self.app.active].kind.clone()
    }
}

#[test]
fn every_scenario_renders_at_every_size_without_panicking() {
    for s in Scenario::ALL {
        for (w, h) in [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)] {
            let mut hh = H::new(s, Motion::Reduced, 0, w, h);
            assert!(hh.row(0).contains("holla❯"), "{s:?} {w}x{h}: {}", hh.row(0));
            hh.ticks(5);
            hh.key(KeyCode::Tab);
            hh.key(KeyCode::Tab);
            hh.key(KeyCode::Esc);
        }
    }
}

#[test]
fn opening_without_typing_suggests_an_action_with_a_reason() {
    let h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    let t = h.text();
    assert!(t.contains("Suggested here"), "{t}");
    assert!(t.contains("branch is 3 commits behind"), "{t}");
    assert!(t.contains("Review 5 modified files"), "{t}");
    assert!(t.contains("Recent here"));
    assert!(t.contains("Explore"));
    assert!(h.row(0).contains("mbp · local"), "{}", h.row(0));
    assert!(t.contains("~/work/holla"), "path in the status bar");
    // the preview answers the §7 questions for the cursor row
    assert!(t.contains("Does"));
    assert!(t.contains("Runs"));
    assert!(t.contains("Gate"));
}

#[test]
fn paused_frames_are_deterministic() {
    let a = H::new(Scenario::ActivitiesMulti, Motion::Paused, 30, 100, 30);
    let b = H::new(Scenario::ActivitiesMulti, Motion::Paused, 30, 100, 30);
    assert_eq!(a.text(), b.text());
    let mut p = H::new(Scenario::ActivitiesMulti, Motion::Paused, 30, 100, 30);
    let before = p.text();
    p.ticks(10);
    assert_eq!(p.text(), before, "paused frames never advance");
}

#[test]
fn too_small_notice_and_recovery() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 60, 15);
    assert!(h.text().contains("Terminal too small"));
    assert!(h.text().contains("Need 72×20"));
    h.resize(100, 30);
    assert!(!h.text().contains("Terminal too small"));
    assert!(h.text().contains("Suggested here"));
}
