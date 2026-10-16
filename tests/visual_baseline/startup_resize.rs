//! A terminal resized right after the app's first frame must still reach
//! the app. crossterm starts its input reader (and its `SIGWINCH` handler)
//! lazily, on the first poll; the shared runtime used to draw the first
//! frame before that poll, so a resize landing in between went to the
//! signal's default disposition (ignored) and was lost: the app never
//! learned the new size and never repainted. Under the default
//! `KeyboardProtocol::Auto` the protocol probe happened to start the reader
//! first, so the race hit only with `JUNIE_TUI_KEYBOARD_PROTOCOL=on|off`;
//! the test forces `off`.
//!
//! `JUNIE_TUI_FIRST_POLL_DELAY_MS` (runtime test instrumentation) widens
//! that window to a fixed delay, so the race is hit every time: the app
//! draws its first frame, the test resizes during the delay, and the app
//! must then report the new size in its status bar (`truecolor · 100×30`),
//! which only a delivered `Input::Resize` updates.

use std::time::Duration;

use crate::scenarios::Bin;
use crate::support::{self, Case, Color};

/// A key and a resize waiting together must both reach the app. crossterm
/// 0.29's reader returns the first key it parses from a poll and drops the
/// rest of that poll's readiness; the `SIGWINCH` readiness is
/// edge-triggered, so when the key's readiness came first the resize was
/// never reported and the app never repainted (`showcase/resize/overview_shrunk`
/// resizes and sends Ctrl+L at once: 2 of 500 stressed runs hung this way).
/// The runtime now checks the terminal size itself (`SizeWatch`) and
/// delivers a missed `Input::Resize`. The delay holds the app off its first
/// poll so the key and then the resize wait for the same poll.
pub fn a_resize_queued_with_a_key_still_reaches_the_app() {
    let case = Case::new(
        "startup_resize/showcase/overview_key/120x40/truecolor",
        Bin::Showcase.path(),
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        "Junie Design system",
    );
    for _ in 0..3 {
        let mut s = support::spawn_with_env(&case, &[("JUNIE_TUI_FIRST_POLL_DELAY_MS", "1500")]);
        support::boot(&mut s, case.needle);
        // the key first: its readiness is reported (and returned) first
        s.send_key("ctrl-l").expect("key");
        std::thread::sleep(Duration::from_millis(50));
        s.resize(100, 30).expect("resize");
        s.find("100×30")
            .unwrap_or_else(|e| panic!("a resize queued with a key never reached the app: {e}"));
    }
}

pub fn a_resize_right_after_the_first_frame_repaints() {
    let case = Case::new(
        "startup_resize/showcase/overview/120x40/truecolor",
        Bin::Showcase.path(),
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        "Junie Design system",
    );
    let mut s = support::spawn_with_env(
        &case,
        &[
            ("JUNIE_TUI_FIRST_POLL_DELAY_MS", "1500"),
            ("JUNIE_TUI_KEYBOARD_PROTOCOL", "off"),
        ],
    );
    // the first frame is up; the event loop now waits before its first poll
    support::boot(&mut s, case.needle);
    s.resize(100, 30).expect("resize");
    std::thread::sleep(Duration::from_millis(100));
    s.find("100×30")
        .unwrap_or_else(|e| panic!("a resize during the first frame never reached the app: {e}"));
}
