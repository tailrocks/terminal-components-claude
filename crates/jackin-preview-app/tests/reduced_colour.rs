#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    missing_docs,
    unreachable_pub
)]

//! The reduced-colour frames keep the corpus ink of the card border.

use termrock::ColorLevel;
use termrock::{KeyCode, MouseKind};

use jackin_preview_app::{Motion, Route, Scenario};
mod support;
use support::H;

#[test]
fn the_cancel_card_border_keeps_its_corpus_index_in_256_colours() {
    let mut h = H::new_with_color(
        Scenario::LaunchRunning,
        Motion::Paused,
        40,
        120,
        40,
        ColorLevel::Ansi256,
    );
    assert_eq!(h.app().route(), Route::Cockpit);
    h.key(KeyCode::Char('x'));
    assert!(
        h.text().contains("Cancel the launch?"),
        "the cancel card must open:\n{}",
        h.text()
    );
    // The card is a bordered block in the middle of the screen.
    let border = h.buffer().cell((33, 15)).expect("card border");
    assert_eq!(
        border.fg,
        termrock::Color::Indexed(242),
        "the card border keeps index 242: {border:?}"
    );
}

#[test]
fn the_cancel_card_answers_the_cancel_key() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Paused, 0, 120, 40);
    let (x, y) = h.find("Cancel").expect("cockpit cancel");
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Up, x, y);
    assert!(
        h.text().contains("Cancel the launch?"),
        "the cancel card must open:\n{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(
        !h.text().contains("Cancel the launch?"),
        "escape must close the card:\n{}",
        h.text()
    );
}
