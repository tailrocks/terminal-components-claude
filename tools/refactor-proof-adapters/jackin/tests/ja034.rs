//! JA-034: registration journey and form grids.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA034_ID, JA034_SIZES, MOTION_SEED, ja034_registration_grid};

#[test]
fn ja034_registers_five_accounts_and_grids_selectors() {
    let captures = ja034_registration_grid();
    assert_eq!(captures.len(), JA034_SIZES.len());
    for capture in &captures {
        assert_eq!(capture.registered.len(), 5);
        assert_eq!(capture.providers.len(), 4);
        assert_eq!(capture.sources.len(), 3);
        let mut frames = vec![&capture.manager, &capture.validated];
        frames.extend(capture.registered.iter());
        frames.extend(capture.providers.iter());
        frames.extend(capture.sources.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA034_ID), "{}", frame.identity);
        }
        assert_eq!(capture.manager.route, "manager");
        for frame in capture.providers.iter().chain(capture.sources.iter()) {
            assert_eq!(frame.route, "accounts", "{}", frame.identity);
        }
        if !(capture.folder_reachable && capture.op_reachable && capture.secret_reachable) {
            // Narrow form: credential inputs never resolve; the journey stalls.
            assert_eq!(capture.account_count, 0);
            continue;
        }
        for frame in capture
            .registered
            .iter()
            .chain(std::iter::once(&capture.validated))
        {
            assert_eq!(frame.route, "accounts", "{}", frame.identity);
        }
        assert!(
            capture.registered[0]
                .text
                .contains("Saved Claude · Personal"),
            "{}",
            capture.registered[0].text
        );
        assert!(
            capture.registered[1].text.contains("Saved Claude · Work"),
            "{}",
            capture.registered[1].text
        );
        assert!(
            capture.registered[4].text.contains("Saved OpenCode · Go"),
            "{}",
            capture.registered[4].text
        );
        assert_eq!(capture.account_count, 5);
        assert!(
            capture.validated.text.contains("Accounts › Claude › Work")
                || capture.validated.text.contains("Default set"),
            "{}",
            capture.validated.text
        );
    }
    let wide = &captures[1];
    assert!(wide.folder_reachable && wide.op_reachable && wide.secret_reachable);
}

#[test]
fn ja034_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja034_registration_grid(), ja034_registration_grid());
}
