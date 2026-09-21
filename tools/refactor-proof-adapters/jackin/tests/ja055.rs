//! JA-055: usage modal, chips, info, menus.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA055_CHIPS, JA055_ID, MOTION_SEED, ja055_usage_chips_info};

#[test]
fn ja055_covers_usage_chips_info_and_menus() {
    let captures = ja055_usage_chips_info();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA055_CHIPS.len(), 2);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.chips.len(), 2);
        assert_eq!(capture.chip_base.len(), 2);
        let mut frames = vec![
            &capture.usage,
            &capture.usage_scrolled,
            &capture.usage_wheeled,
            &capture.usage_closed,
            &capture.info,
            &capture.info_copied,
        ];
        frames.extend(capture.chips.iter());
        frames.extend(capture.chip_base.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA055_ID), "{}", frame.identity);
        }
        for frame in [
            &capture.usage,
            &capture.usage_scrolled,
            &capture.usage_wheeled,
            &capture.info,
            &capture.info_copied,
        ] {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        for frame in capture.chips.iter().chain(capture.chip_base.iter()) {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        // `Esc` with other instances running departs for the manager even
        // with the usage modal open.
        assert_eq!(capture.usage_closed.route, "manager");
        assert!(
            capture.usage.text.contains("Usage"),
            "{}",
            capture.usage.text
        );
        assert!(capture.usage_at.is_some(), "usage wheel must resolve");
        // Chips paint but own no hitbox: clicks change nothing.
        for (chip, at) in JA055_CHIPS.iter().zip(capture.chip_at.iter()) {
            assert!(
                at.is_some(),
                "chip {chip} must paint at {}x{}",
                viewport.width,
                viewport.height
            );
        }
        for (base, clicked) in capture.chip_base.iter().zip(capture.chips.iter()) {
            assert_eq!(clicked.digest, base.digest);
        }
        assert!(capture.about_found, "About must resolve in a menu");
    }
}

#[test]
fn ja055_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja055_usage_chips_info(), ja055_usage_chips_info());
}
