//! End-to-end interaction tests through the real App on a TestBackend.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::theme::Theme;

use crate::app::{App, Route};
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
        let o = self.app.handle(Input::Key(Key { code, mods: KeyModifiers::NONE }));
        self.draw();
        o
    }
    pub fn ctrl(&mut self, c: char) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::CONTROL,
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
        let o = self.app.handle(Input::Mouse(Mouse { kind, pos: Position::new(x, y) }));
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
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        let want: Vec<&str> = unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect();
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
}

#[test]
fn first_use_plays_intro_then_manager_and_no_replay_when_returning() {
    let mut h = H::new(Scenario::FirstUse, Motion::Full, 0, 120, 40);
    assert_eq!(h.app.route, Route::Intro);
    h.ticks(30);
    assert!(h.text().contains("Stand up, operator"));
    // skip during phrases jumps to the warp, then finishes into the manager
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Intro);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert!(h.text().contains("Current directory"));
    assert!(h.text().contains("+ New workspace"));
    let r = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    assert_eq!(r.app.route, Route::Manager, "an active Construct joins without replay");
    assert!(r.text().contains("2 running"));
}

#[test]
fn reduced_motion_and_paused_frames_are_deterministic() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 80, 24);
    assert_eq!(h.app.route, Route::Intro);
    assert!(h.text().contains("Enter Continue"));
    h.ticks(3);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    let a = H::new(Scenario::FirstUse, Motion::Paused, 282, 100, 30);
    let b = H::new(Scenario::FirstUse, Motion::Paused, 282, 100, 30);
    assert_eq!(a.text(), b.text());
    let mut p = H::new(Scenario::FirstUse, Motion::Paused, 20, 80, 24);
    p.ticks(5);
    assert!(p.text().contains("Stand up, operator"), "paused frames never advance");
}

#[test]
fn manager_navigation_expand_and_detail_focus() {
    let mut h = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    h.key(KeyCode::Down);
    assert!(h.text().contains("payments-platform"));
    h.key(KeyCode::Right);
    assert!(h.text().contains("7f3a"), "instance children visible after expand");
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    assert!(h.text().contains("Live topology"));
    h.key(KeyCode::Esc);
    assert_eq!(h.app.focus.current(), Some(crate::screens::manager::TREE));
    // mouse: click the row of infra-control-plane
    let (x, y) = h.find("infra-control-plane").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Workspaces › infra-control-plane"));
}

#[test]
fn launch_runs_all_stages_and_hands_off_to_the_capsule() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Full, 0, 120, 40);
    assert_eq!(h.app.route, Route::Cockpit);
    for _ in 0..40 {
        h.ticks(10);
        if h.app.route != Route::Cockpit {
            break;
        }
    }
    assert!(matches!(h.app.route, Route::Handoff | Route::Capsule), "route {:?}", h.app.route);
    h.ticks(15);
    assert_eq!(h.app.route, Route::Capsule);
    assert!(h.text().contains("jackin❯"));
    // type into the pane and see the echo
    h.ticks(60);
    h.type_str("hello");
    assert!(h.text().contains("hello"));
}

#[test]
fn launch_failure_returns_to_the_construct_when_another_instance_runs() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Full, 0, 120, 40);
    for _ in 0..60 {
        h.ticks(10);
        if h.text().contains("Launch failed") {
            break;
        }
    }
    assert!(h.text().contains("Launch failed"), "{}", h.text());
    assert!(h.text().contains("Network"));
    h.key(KeyCode::Esc);
    assert_eq!(h.app.route, Route::Manager);
    assert!(h.text().contains("still running"));
}

#[test]
fn detach_reconnect_and_final_exit_plays_one_outro() {
    let mut h = H::new(Scenario::OutroLast, Motion::Full, 0, 120, 40);
    assert_eq!(h.app.route, Route::Capsule);
    h.ctrl('b');
    h.key(KeyCode::Char('d'));
    assert_eq!(h.app.route, Route::Manager);
    assert!(h.text().contains("Detached"));
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Capsule, "reconnect restores the Capsule");
    h.ctrl('q');
    assert!(h.text().contains("Unsaved work"));
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter); // exit & keep
    assert_eq!(h.app.route, Route::Outro);
    h.key(KeyCode::Enter);
    h.ticks(25);
    assert!(h.text().contains("You were in the Construct for 2 h 14 min"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.app.quit);
}

#[test]
fn still_inside_feedback_when_other_instances_remain() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Full, 0, 120, 40);
    assert_eq!(h.app.route, Route::Capsule);
    h.ctrl('q');
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Manager);
    assert!(h.text().contains("Still inside the Construct"));
    assert_eq!(h.app.world.running_count(), 1);
}

#[test]
fn too_small_state_and_resize_recover() {
    let mut h = H::new(Scenario::Returning, Motion::Full, 0, 120, 40);
    h.resize(60, 18);
    assert!(h.text().contains("Terminal too small"));
    h.resize(80, 24);
    assert!(h.text().contains("Workspaces"));
}
