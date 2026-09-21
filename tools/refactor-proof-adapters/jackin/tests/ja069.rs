//! JA-069: complete journey.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{
    JA069_ID, JA069_MILESTONES, JA069_SIZES, MOTION_SEED, ja069_complete_journey,
};

#[test]
fn ja069_all_forty_steps() {
    let captures = ja069_complete_journey();
    assert_eq!(captures.len(), JA069_SIZES.len());
    for capture in &captures {
        assert_eq!(capture.milestones, JA069_MILESTONES);
        assert_eq!(capture.accounts_len, 5);
        assert_eq!(capture.workspaces_len, 1);
        assert_eq!(capture.tabs_after_new, 2);
        assert_eq!(capture.panes_after_split, 3);
        assert_eq!(capture.running_after_second, 2);
        assert_eq!(capture.running_after_exit, 1);
        assert!(capture.final_quit);
        assert_eq!(capture.frames.len(), 25);
        let routes: Vec<&str> = capture
            .frames
            .iter()
            .map(|frame| frame.route.as_str())
            .collect();
        assert_eq!(routes[0], "manager", "s01");
        assert_eq!(routes[1], "accounts", "s04");
        assert_eq!(routes[7], "manager", "s13");
        assert_eq!(routes[8], "editor", "s14");
        assert_eq!(routes[11], "manager", "s17");
        assert_eq!(routes[12], "cockpit", "s20");
        assert_eq!(routes[14], "capsule", "s23");
        assert_eq!(routes[20], "capsule", "s33");
        assert_eq!(routes[21], "capsule", "s34");
        assert_eq!(routes[22], "manager", "s36");
        assert_eq!(routes[23], "capsule", "s37");
        let texts: Vec<&str> = capture
            .frames
            .iter()
            .map(|frame| frame.text.as_str())
            .collect();
        assert!(texts[2].contains("Saved Claude · Work"), "s06");
        assert!(texts[3].contains("· Team"), "s08");
        assert!(!texts[3].contains("valid-"), "s08 leak");
        assert!(texts[4].contains("Saved OpenCode · Go"), "s09");
        assert!(texts[5].contains("Default set"), "s10");
        assert!(texts[6].contains("Quota"), "s12");
        assert!(texts[8].contains("new workspace › edit"), "s14");
        assert!(texts[9].contains("API_BASE"), "s15");
        assert!(
            texts[10].contains("5 effective · 4 inherited · 1 enabled here"),
            "s16"
        );
        assert!(texts[14].contains("hello"), "s23");
        assert!(texts[15].contains("(Work)"), "s24");
        assert!(texts[16].contains("zoom"), "s27");
        assert!(texts[22].contains("Still inside the Construct"), "s36");
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA069_ID), "{}", frame.identity);
        }
    }
}
