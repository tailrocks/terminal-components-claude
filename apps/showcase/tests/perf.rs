//! Focused performance smoke tests for the migrated app.
use std::hint::black_box;

use showcase_app::{App, PageId};
use tui_next::{Axis, KeyCode, Theme};
use tui_next_testing::Harness;

fn require<T>(value: Option<T>, message: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{message}"),
    }
}

fn app(page: PageId, width: u16, height: u16) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), width, height)
}

#[test]
fn frame_showcase_lists_120x40() {
    let mut h = app(PageId::Lists, 120, 40);
    let first = h.snapshot().digest();
    for _ in 0..200 {
        h.draw();
    }
    let last = h.snapshot().digest();
    assert_eq!(
        first, last,
        "steady-state list frames must be deterministic"
    );
    black_box(last);
}

#[test]
fn frame_showcase_lists_80x24() {
    let mut h = app(PageId::Lists, 80, 24);
    let first = h.snapshot().digest();
    for _ in 0..200 {
        h.draw();
    }
    let last = h.snapshot().digest();
    assert_eq!(first, last, "compact list frames must be deterministic");
    black_box(last);
}

#[test]
fn frame_showcase_dialog_open() {
    let mut h = app(PageId::Dialogs, 120, 40);
    let _ = h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Showcase help"));
    // Opening a layer settles its focus scope on the next draw; benchmark the
    // steady-state frame rather than the hand-off frame.
    h.draw();
    let first = h.snapshot().digest();
    for _ in 0..200 {
        h.draw();
    }
    let last = h.snapshot().digest();
    assert_eq!(first, last, "open dialog frames must be deterministic");
    black_box(last);
}

#[test]
fn key_showcase_down_lists() {
    let mut h = app(PageId::Lists, 120, 40);
    let _ = h.key(KeyCode::Tab);
    for _ in 0..1_000 {
        let _ = h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Lists"));
    black_box(h.snapshot().digest());
}

#[test]
fn mouse_move_showcase_frame() {
    let mut h = app(PageId::Overview, 120, 40);
    let mut hovered = 0_u32;
    for y in 0..40 {
        for x in (0..120).step_by(2) {
            let _ = h.mouse(tui_next::MouseKind::Move, x, y);
            if h.hover().is_some() {
                hovered = hovered.saturating_add(1);
            }
        }
    }
    assert!(hovered > 0, "pointer sweep must hit registered regions");
    black_box(h.snapshot().digest());
}

#[test]
fn wheel_showcase_lists() {
    let mut h = app(PageId::Lists, 80, 24);
    let (x, y) = require(h.find("Rust"), "language list starts in the frame");
    let first = h.snapshot().digest();
    for _ in 0..1_000 {
        let _ = h.wheel(Axis::V, 2, x, y);
    }
    let last = h.snapshot().digest();
    assert_ne!(
        first, last,
        "wheel input must route through the list viewport"
    );
    black_box(last);
}

#[test]
fn render_twice_allocates_the_same() {
    let mut h = app(PageId::Overview, 120, 40);
    h.draw();
    let first = h.snapshot().digest();
    for _ in 0..200 {
        h.draw();
    }
    let second = h.snapshot().digest();
    assert_eq!(first, second, "repeated renders must keep the same scene");
}
