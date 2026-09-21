//! JA-067: color identities.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA067_ID, JA067_SIZES, MOTION_SEED, ja067_color_identities};

#[test]
fn ja067_identities_downgrade_and_detect() {
    let captures = ja067_color_identities();
    assert_eq!(captures.len(), JA067_SIZES.len());
    for capture in &captures {
        assert!(capture.labels_distinct);
        assert!(capture.levels_ok);
        assert_eq!(capture.rgb_cells.len(), 5);
        assert!(capture.rgb_cells[0] > 0, "{:?}", capture.rgb_cells);
        assert_eq!(capture.rgb_cells[3], 0, "{:?}", capture.rgb_cells);
        assert_eq!(capture.rgb_cells[4], 0, "{:?}", capture.rgb_cells);
        assert!(capture.mono_differs);
        assert!(capture.none_equals_nocolor);
        assert!(capture.detect_table);
        assert_eq!(capture.frames.len(), 5);
        for frame in &capture.frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA067_ID), "{}", frame.identity);
        }
    }
}
