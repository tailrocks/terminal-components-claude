//! Focused performance smoke tests for the migrated app.
use std::hint::black_box;

use showcase_app::{App, PageId};
use tui_next::{Axis, KeyCode, Theme};
use tui_next_testing::Harness;

fn app(page: PageId, width: u16, height: u16) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), width, height)
}

#[test]
fn frame_showcase_lists_120x40() {
    let mut h = app(PageId::Lists, 120, 40);
    for _ in 0..8 {
        h.draw();
    }
    black_box(h.snapshot().digest());
}

#[test]
fn frame_showcase_lists_80x24() {
    let mut h = app(PageId::Lists, 80, 24);
    for _ in 0..8 {
        h.draw();
    }
    black_box(h.snapshot().digest());
}

#[test]
fn frame_showcase_dialog_open() {
    let mut h = app(PageId::Dialogs, 120, 40);
    h.key(KeyCode::Char('?'));
    h.draw();
    black_box(h.snapshot().digest());
}

#[test]
fn key_showcase_down_lists() {
    let mut h = app(PageId::Lists, 120, 40);
    for _ in 0..16 {
        h.key(KeyCode::Down);
    }
    black_box(h.snapshot().digest());
}

#[test]
fn mouse_move_showcase_frame() {
    let mut h = app(PageId::Overview, 120, 40);
    for x in (0..120).step_by(5) {
        h.mouse(tui_next::MouseKind::Move, x, 10);
    }
    black_box(h.snapshot().digest());
}

#[test]
fn wheel_showcase_lists() {
    let mut h = app(PageId::Lists, 120, 40);
    for _ in 0..8 {
        h.wheel(Axis::V, 2, 80, 20);
    }
    black_box(h.snapshot().digest());
}

#[test]
fn render_twice_allocates_the_same() {
    let mut h = app(PageId::Overview, 120, 40);
    h.draw();
    let first = h.snapshot().digest();
    h.draw();
    let second = h.snapshot().digest();
    assert_eq!(first, second);
}
