//! JA-052: tab menu replay and comma rename.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA052_ID, MOTION_SEED, ja052_tab_menu};

#[test]
fn ja052_replay_renames_dismisses_and_renames_via_comma() {
    let captures = ja052_tab_menu();
    assert_eq!(captures.len(), JA001_SIZES.len());
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        for frame in [
            &capture.menu,
            &capture.prompt,
            &capture.typing,
            &capture.renamed,
            &capture.close_ask,
            &capture.close_dismissed,
            &capture.menu_dismissed,
            &capture.comma_prompt,
            &capture.comma_renamed,
        ] {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA052_ID), "{}", frame.identity);
        }
        for frame in [
            &capture.menu,
            &capture.prompt,
            &capture.typing,
            &capture.renamed,
            &capture.close_ask,
            &capture.close_dismissed,
            &capture.menu_dismissed,
            &capture.comma_prompt,
        ] {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        assert!(
            capture.shell_at.is_some(),
            "Shell tab must resolve at {}x{}",
            viewport.width,
            viewport.height
        );
        assert!(
            capture.menu.text.contains("Change title…"),
            "{}",
            capture.menu.text
        );
        assert!(
            capture.menu.text.contains("Close tab"),
            "{}",
            capture.menu.text
        );
        assert!(
            capture.prompt.text.contains("Change tab title"),
            "{}",
            capture.prompt.text
        );
        assert!(
            capture.typing.text.contains("ops"),
            "{}",
            capture.typing.text
        );
        assert!(
            capture
                .renamed
                .text
                .lines()
                .nth(2)
                .unwrap_or("")
                .contains("ops"),
            "{}",
            capture.renamed.text
        );
        assert!(
            capture.close_ask.text.contains("Close tab?"),
            "{}",
            capture.close_ask.text
        );
        // `Ctrl-B,comma` opens no prompt; the rename typing hits globals
        // (`m` → manager).
        assert!(!capture.comma_prompt.text.contains("Change tab title"));
        assert_eq!(capture.comma_renamed.route, "manager");
    }
}

#[test]
fn ja052_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja052_tab_menu(), ja052_tab_menu());
}
