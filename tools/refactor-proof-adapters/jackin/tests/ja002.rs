//! JA-002: first-use paused rain phase frames.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    CaptureColor, DirectSession, JA001_SIZES, JA002_ID, MOTION_SEED, Motion, Scenario, Viewport,
    ja002_frames, ja002_paused_first_use, knock_caption, production_intro_message,
};
use jackin_app::rain::{INTRO_END, IntroPhase, KNOCK_START, P1_LEN, P2_LEN, PHRASES, WARP_START};

#[test]
fn ja002_frames_match_source_rain_constants() {
    assert_eq!(
        ja002_frames(),
        [
            0,
            45,
            P1_LEN - 1,
            P1_LEN,
            P1_LEN + P2_LEN,
            KNOCK_START,
            KNOCK_START + 1,
            WARP_START,
            WARP_START + 1,
            INTRO_END - 1,
        ]
    );
    assert_eq!(IntroPhase::of(P1_LEN - 1), IntroPhase::Phrases);
    assert_eq!(IntroPhase::of(WARP_START), IntroPhase::Warp);
    assert_eq!(IntroPhase::of(INTRO_END - 1), IntroPhase::Warp);
    assert_eq!(knock_caption(), "Knock, knock, operator.");
}

#[test]
fn ja002_paused_first_use_covers_frames_and_sizes() {
    let captures = ja002_paused_first_use();
    assert_eq!(captures.len(), JA001_SIZES.len() * ja002_frames().len());
    let mut idx = 0;
    for viewport in JA001_SIZES {
        for frame in ja002_frames() {
            let observed = &captures[idx];
            idx += 1;
            assert_eq!(observed.scenario, "first-use");
            assert_eq!(observed.motion, "paused");
            assert_eq!(observed.construct_frame, frame);
            assert_eq!(observed.width, viewport.width);
            assert_eq!(observed.height, viewport.height);
            assert_eq!(observed.color, CaptureColor::TrueColor.label());
            assert_eq!(observed.route, "intro");
            assert!(!observed.clock_running);
            assert_eq!(observed.motion_seed, MOTION_SEED);
            assert!(observed.is_complete(), "{}", observed.identity);
            assert!(
                observed.identity.starts_with(&format!(
                    "{JA002_ID}/first-use/paused/{frame}/{}x{}/truecolor/",
                    viewport.width, viewport.height
                )),
                "{}",
                observed.identity
            );
            let message = production_intro_message(frame);
            assert!(
                observed.text.contains(message),
                "frame {frame} missing {message:?}\n{}",
                observed.text
            );
            assert!(observed.text.contains("jackin❯"), "{}", observed.identity);
            match IntroPhase::of(frame) {
                IntroPhase::Phrases => {
                    assert!(
                        PHRASES
                            .iter()
                            .any(|(text, _, _)| observed.text.contains(*text)),
                        "phrase frame {frame} lacked a PHRASES line\n{}",
                        observed.text
                    );
                }
                IntroPhase::Warp => {
                    assert!(
                        observed.text.contains(knock_caption()),
                        "warp frame {frame} lacked caption\n{}",
                        observed.text
                    );
                }
                IntroPhase::Done => panic!("JA-002 must stay inside intro, frame {frame}"),
            }
        }
    }
}

#[test]
fn ja002_phase_boundaries_change_production_draw() {
    let viewport = Viewport::new(80, 24);
    let before_p2 = DirectSession::fresh(
        JA002_ID,
        Scenario::FirstUse,
        Motion::Paused,
        P1_LEN - 1,
        viewport,
        CaptureColor::TrueColor,
    )
    .observe("initial");
    let at_p2 = DirectSession::fresh(
        JA002_ID,
        Scenario::FirstUse,
        Motion::Paused,
        P1_LEN,
        viewport,
        CaptureColor::TrueColor,
    )
    .observe("initial");
    assert_ne!(before_p2.digest, at_p2.digest);
    assert!(before_p2.text.contains(PHRASES[0].0));
    assert!(at_p2.text.contains(PHRASES[1].0));

    let before_warp = DirectSession::fresh(
        JA002_ID,
        Scenario::FirstUse,
        Motion::Paused,
        WARP_START - 1,
        viewport,
        CaptureColor::TrueColor,
    )
    .observe("initial");
    let warp = DirectSession::fresh(
        JA002_ID,
        Scenario::FirstUse,
        Motion::Paused,
        WARP_START,
        viewport,
        CaptureColor::TrueColor,
    )
    .observe("initial");
    assert_ne!(before_warp.digest, warp.digest);
    assert!(warp.text.contains("opening the Construct"));
}

#[test]
fn ja002_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja002_paused_first_use(), ja002_paused_first_use());
}

#[test]
fn ja002_nonzero_frames_use_for_scenario_at() {
    let frame = DirectSession::fresh(
        JA002_ID,
        Scenario::FirstUse,
        Motion::Paused,
        45,
        Viewport::new(80, 24),
        CaptureColor::TrueColor,
    )
    .observe("initial");
    assert_eq!(frame.construct_frame, 45);
    assert_eq!(frame.route, "intro");
    assert!(frame.text.contains("Stand up, operator…"));
    assert!(!frame.clock_running);
}
