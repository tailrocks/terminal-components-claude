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

    support::boot(&mut s, case.needle);
    s.resize(100, 30).expect("resize");
    std::thread::sleep(Duration::from_millis(100));
    s.find("100×30")
        .unwrap_or_else(|e| panic!("a resize during the first frame never reached the app: {e}"));
}
