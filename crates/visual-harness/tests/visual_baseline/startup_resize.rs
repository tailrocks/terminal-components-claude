use std::time::Duration;

use crate::scenarios::Bin;
use crate::support::{self, Case, Color};

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
        s.wait_idle(Duration::from_millis(100))
            .expect("first frame should remain stable during the initial wait delay");

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
    // The first-poll delay is consumed by the current session. Keep the old
    // keyboard variable as historical intent; this startup path selects Off
    // directly and does not consume that old variable name.
    let mut s = support::spawn_with_env(
        &case,
        &[
            ("JUNIE_TUI_FIRST_POLL_DELAY_MS", "1500"),
            ("JUNIE_TUI_KEYBOARD_PROTOCOL", "off"),
        ],
    );

    support::boot(&mut s, case.needle);
    s.wait_idle(Duration::from_millis(100))
        .expect("first frame should remain stable during the initial wait delay");
    s.resize(100, 30).expect("resize");
    std::thread::sleep(Duration::from_millis(100));
    s.find("100×30")
        .unwrap_or_else(|e| panic!("a resize during the first frame never reached the app: {e}"));
}

pub fn a_resize_co_ready_with_a_key_preserves_both_effects() {
    let case = Case::new(
        "startup_resize/showcase/overview_co_ready/120x40/truecolor",
        Bin::Showcase.path(),
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        "Junie Design system",
    );
    let mut s = support::spawn_with_env(&case, &[("JUNIE_TUI_FIRST_POLL_DELAY_MS", "1500")]);
    support::boot(&mut s, case.needle);
    // The first Overview frame is already visible. Chrome's source-bound Brand
    // lockup is absent until the real next-page key is handled.
    s.assert_wait_not_vacuous("Brand lockup");
    s.send_key("]").expect("next-page key");
    s.resize(100, 30).expect("co-ready resize");
    // Both readiness sources are queued, but the first-poll hold still leaves
    // the initial Overview frame visible before it releases them.
    s.wait_idle(Duration::from_millis(100))
        .expect("the presented Overview frame remains stable during the first-poll hold");
    s.assert_wait_not_vacuous("Brand lockup");

    s.find("Brand lockup")
        .unwrap_or_else(|error| panic!("the queued key did not select Chrome: {error}"));
    s.find("100×30")
        .unwrap_or_else(|error| panic!("the co-ready resize did not reach the app: {error}"));
}

pub fn a_resize_without_input_repaints_when_idle() {
    let case = Case::new(
        "startup_resize/showcase/overview_idle/120x40/truecolor",
        Bin::Showcase.path(),
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        "Junie Design system",
    );
    let mut s = support::spawn(&case);
    support::boot(&mut s, case.needle);
    s.wait_idle(Duration::from_millis(100))
        .expect("application should be idle before resizing");

    s.resize(100, 30).expect("idle resize");
    s.find("100×30")
        .unwrap_or_else(|error| panic!("the idle resize did not reach the app: {error}"));
}
