//! JA-053: menu bar replay and extensions.

#![allow(
    missing_docs,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

use jackin_adapter::{JA001_SIZES, JA053_ID, JA053_MENUS, MOTION_SEED, ja053_menu_bar};

#[test]
fn ja053_replay_walks_clicks_and_hovers_menus() {
    let captures = ja053_menu_bar();
    assert_eq!(captures.len(), JA001_SIZES.len());
    assert_eq!(JA053_MENUS.len(), 5);
    for (capture, viewport) in captures.iter().zip(JA001_SIZES) {
        assert_eq!(capture.menu_clicks.len(), JA053_MENUS.len());
        let mut frames = vec![
            &capture.bar,
            &capture.next_menu,
            &capture.dismissed,
            &capture.spawn,
            &capture.spawn_dismissed,
            &capture.view_clicked,
            &capture.usage_clicked,
            &capture.arrows,
            &capture.brand,
        ];
        frames.extend(capture.menu_clicks.iter());
        for frame in frames {
            assert!(frame.is_complete(), "{}", frame.identity);
            assert_eq!(frame.width, viewport.width);
            assert_eq!(frame.height, viewport.height);
            assert_eq!(frame.motion_seed, MOTION_SEED);
            assert!(frame.identity.starts_with(JA053_ID), "{}", frame.identity);
        }
        for frame in [
            &capture.bar,
            &capture.next_menu,
            &capture.dismissed,
            &capture.spawn,
            &capture.spawn_dismissed,
            &capture.view_clicked,
            &capture.arrows,
            &capture.brand,
        ] {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        for frame in &capture.menu_clicks {
            assert_eq!(frame.route, "capsule", "{}", frame.identity);
        }
        assert!(capture.bar.text.contains("New tab"), "{}", capture.bar.text);
        assert!(
            capture.next_menu.text.contains("Copy selection"),
            "{}",
            capture.next_menu.text
        );
        // The 80-column bar clips every title after File, so title clicks
        // only resolve at wider sizes.
        if viewport.width >= 100 {
            assert!(capture.view_at.is_some(), "View must resolve");
        }
        // The narrow View menu never opens, so the Usage click only lands
        // where the menu resolves; then it navigates to the Usage route.
        if capture.view_clicked.text.contains("Zoom pane") {
            assert!(capture.usage_at.is_some(), "Usage must resolve");
            assert_eq!(capture.usage_clicked.route, "usage");
            assert!(
                capture.usage_clicked.text.contains("Overview"),
                "{}",
                capture.usage_clicked.text
            );
        } else {
            assert_eq!(capture.usage_clicked.route, "capsule");
        }
        // Painted titles depend on width: File only at 80, File/Edit/View at
        // 100 (`Ses…` clips Session), all five at 120+.
        let painted = if viewport.width < 100 {
            JA053_MENUS.len() - 4
        } else if viewport.width < 120 {
            JA053_MENUS.len() - 2
        } else {
            JA053_MENUS.len()
        };
        for (index, (title, at)) in JA053_MENUS.iter().zip(capture.menu_at.iter()).enumerate() {
            if index < painted {
                assert!(
                    at.is_some(),
                    "menu {title} must resolve at {}x{}",
                    viewport.width,
                    viewport.height
                );
            } else {
                assert!(
                    at.is_none(),
                    "menu {title} must clip at {}x{}",
                    viewport.width,
                    viewport.height
                );
            }
        }
        // Bar titles own no hover hitbox: moving across the bar resolves no
        // owner at any size.
        assert!(capture.hover.is_none(), "hover={:?}", capture.hover);
        assert!(capture.brand_at.is_some(), "brand must resolve");
    }
}

#[test]
fn ja053_repeat_from_fresh_worlds_matches() {
    assert_eq!(ja053_menu_bar(), ja053_menu_bar());
}
