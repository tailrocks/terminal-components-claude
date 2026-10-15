//! Tuiscotti PTY qualification against the shipped `showcase` binary:
//! launch, boot needle, six-format capture invariants, input navigation and
//! clean exit. Lives beside the binary it qualifies.

use std::time::Duration;

use tuiscotti::Profile;
use tuiscotti::VENDORED_FACES;
use tuiscotti::formats::{
    assert_no_escapes, assert_normalized_sgr, assert_opaque_rgb, assert_seven_bit,
    assert_static_offline, capture_all, parse_canonical,
};
use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::Tui;

#[test]
fn qualify_pty_session_lifecycle_and_capture() {
    let showcase_bin = env!("CARGO_BIN_EXE_showcase").to_string();
    let cols = 80;
    let rows = 24;

    let session = Tui::new([
        showcase_bin,
        "--page".into(),
        "overview".into(),
        "--color".into(),
        "truecolor".into(),
    ])
    .size(cols, rows)
    .env_remove("NO_COLOR")
    .env_remove("HOLLA_NO_MOTION")
    .spawn()
    .expect("session spawn succeeds");

    // Wait until boot needle or settled
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let cancel = tuiscotti::tui::CancelToken::new();
    let obs = session
        .wait_predicate(
            |obs| {
                frame_from_screen(&obs.screen, "default")
                    .text()
                    .contains("Junie")
            },
            deadline,
            &cancel,
        )
        .expect("overview page loaded with needle");

    assert_eq!(obs.screen.cols(), cols);
    assert_eq!(obs.screen.rows(), rows);

    // Convert observed screen to Frame
    let profile = Profile::default_profile();
    let frame = frame_from_screen(&obs.screen, &profile.name);

    let mut renderer = profile
        .renderer(&VENDORED_FACES)
        .expect("vendored faces parse");
    let bundle = capture_all(&mut renderer, &frame, "pty-showcase-overview")
        .expect("capture_all on pty frame succeeds");

    assert_seven_bit(&bundle.ascii.text).expect("ascii 7-bit");
    assert_no_escapes(&bundle.txt).expect("txt clean");
    assert_normalized_sgr(&bundle.ansi).expect("ansi normalized");
    assert_opaque_rgb(&bundle.png).expect("png opaque");
    assert_static_offline(&bundle.html).expect("html static");
    let parsed = parse_canonical(&bundle.json).expect("valid frame json");
    assert_eq!(parsed.digest(), frame.digest());

    // Send key input to navigate
    session.press("Tab").expect("press Tab succeeds");
    let settle_deadline = std::time::Instant::now() + Duration::from_secs(2);
    session
        .wait_stable(settle_deadline, &cancel)
        .expect("session settled after Tab");

    // Clean exit
    session.press("q").expect("press q succeeds");
    let exit_deadline = std::time::Instant::now() + Duration::from_secs(3);
    let exit_wait = session
        .wait_exit(exit_deadline, &cancel)
        .expect("session exits cleanly");
    assert!(exit_wait.status.success(), "session exit was successful");
}
