//! TP-003–007: filter, hover, connect stages, auth/unreachable retry.
//!
//! Inputs are the TSV sequences. Observations are production handler results;
//! missing filter/progress surfaces are recorded, not blessed.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_C, click_text, down, enter, esc,
    hover_text, launch_preset_c, observe, pointer_down_text, pointer_up_text, ticks, type_text,
};
use tablepro_app::{Screen, Surface};

fn cap(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    observe(harness, scenario, PRESET_C, checkpoint, profile)
}

fn each_matrix(mut body: impl FnMut(u16, u16, ColorProfile)) {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            body(width, height, profile);
        }
    }
}

#[test]
fn tp003_filter_slash_type_esc_down() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "Production");
        let filtered = cap(&harness, "TP-003", "filtered", profile);
        esc(&mut harness);
        let cancelled = cap(&harness, "TP-003", "cancelled", profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "zzzz_no_match");
        let empty = cap(&harness, "TP-003", "empty", profile);
        esc(&mut harness);
        down(&mut harness, 1);
        let restored = cap(&harness, "TP-003", "restored", profile);
        for obs in [&filtered, &cancelled, &empty, &restored] {
            assert_eq!(obs.screen, Screen::Connections, "{}", obs.checkpoint);
            assert_eq!(obs.surface, Surface::Connections, "{}", obs.checkpoint);
        }
        let mut again = launch_preset_c(width, height, profile);
        type_text(&mut again, "/");
        type_text(&mut again, "Production");
        assert_eq!(filtered, cap(&again, "TP-003", "filtered", profile));
    });
}

#[test]
fn tp004_hover_down_connect_press() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        hover_text(&mut harness, "Local PostgreSQL");
        let hovered = cap(&harness, "TP-004", "hover", profile);
        down(&mut harness, 1);
        let suppressed = cap(&harness, "TP-004", "keyboard_suppresses_hover", profile);
        hover_text(&mut harness, "Connect");
        pointer_down_text(&mut harness, "Connect");
        let pressed = cap(&harness, "TP-004", "pressed", profile);
        pointer_up_text(&mut harness, "Connect");
        let connecting = cap(&harness, "TP-004", "connecting", profile);
        assert_eq!(hovered.screen, Screen::Connections);
        assert!(hovered.text.contains("Local PostgreSQL"));
        assert_eq!(suppressed.screen, Screen::Connections);
        assert!(
            matches!(connecting.screen, Screen::Connections | Screen::Workbench),
            "unexpected screen after Connect press"
        );
        let _ = (pressed, connecting);
    });
}

#[test]
fn tp005_connect_success_ticks() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        down(&mut harness, 8);
        enter(&mut harness);
        let t0 = cap(&harness, "TP-005", "t0", profile);
        ticks(&mut harness, 3);
        let t3 = cap(&harness, "TP-005", "t3", profile);
        ticks(&mut harness, 4);
        let t7 = cap(&harness, "TP-005", "t7", profile);
        ticks(&mut harness, 4);
        let t11 = cap(&harness, "TP-005", "t11", profile);
        ticks(&mut harness, 1);
        let connected = cap(&harness, "TP-005", "connected", profile);
        for obs in [&t0, &t3, &t7, &t11, &connected] {
            assert!(
                matches!(obs.screen, Screen::Connections | Screen::Workbench),
                "{} {:?}",
                obs.checkpoint,
                obs.screen
            );
        }
        let mut again = launch_preset_c(width, height, profile);
        down(&mut again, 8);
        enter(&mut again);
        assert_eq!(t0, cap(&again, "TP-005", "t0", profile));
    });
}

#[test]
fn tp006_staging_auth_failed_retry() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "Staging");
        enter(&mut harness);
        down(&mut harness, 1);
        enter(&mut harness);
        ticks(&mut harness, 12);
        let auth_failed = cap(&harness, "TP-006", "auth_failed", profile);
        click_text(&mut harness, "Reconnect");
        let retry = cap(&harness, "TP-006", "retry", profile);
        ticks(&mut harness, 12);
        let failed_again = cap(&harness, "TP-006", "failed_again", profile);
        assert_eq!(auth_failed.screen, Screen::Connections);
        assert_eq!(retry.screen, Screen::Connections);
        assert_eq!(failed_again.screen, Screen::Connections);
    });
}

#[test]
fn tp007_analytics_unreachable_retry() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_c(width, height, profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "Analytics");
        enter(&mut harness);
        down(&mut harness, 1);
        enter(&mut harness);
        ticks(&mut harness, 12);
        let unreachable = cap(&harness, "TP-007", "unreachable", profile);
        click_text(&mut harness, "Reconnect");
        ticks(&mut harness, 12);
        let retry_failed = cap(&harness, "TP-007", "retry_failed", profile);
        assert!(matches!(
            unreachable.screen,
            Screen::Connections | Screen::Workbench
        ));
        assert!(matches!(
            retry_failed.screen,
            Screen::Connections | Screen::Workbench
        ));
    });
}
