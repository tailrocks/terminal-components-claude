//! JA-001: eight CLI worlds, Paused, frame 0, size and color matrix.

use jackin_app::{Route, Scenario};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA001_ID: &str = "JA-001";
/// First-leaf width.
pub const JA001_FIRST_LEAF_WIDTH: u16 = 80;
/// First-leaf height.
pub const JA001_FIRST_LEAF_HEIGHT: u16 = 24;
/// First-leaf viewport.
pub const JA001_FIRST_LEAF_SIZE: (u16, u16) = (JA001_FIRST_LEAF_WIDTH, JA001_FIRST_LEAF_HEIGHT);
/// First-leaf color label.
pub const JA001_FIRST_LEAF_COLOR: &str = "truecolor";

/// JA-001 sizes in source order.
pub const JA001_SIZES: [Viewport; 4] = [
    Viewport::new(80, 24),
    Viewport::new(100, 30),
    Viewport::new(120, 40),
    Viewport::new(160, 50),
];

/// Sizes after the 80×24 first leaf.
pub const JA001_REMAINING_SIZES: [Viewport; 3] = [
    Viewport::new(100, 30),
    Viewport::new(120, 40),
    Viewport::new(160, 50),
];

/// One world's initial draw and the paused T(5) follow-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja001Capture {
    /// `fresh(world, Paused, 0); draw`
    pub initial: ObservedFrame,
    /// `T(5)` after the initial draw; paused frames must be unchanged.
    pub after_ticks: ObservedFrame,
}

/// Source-enum world order for JA-001.
#[must_use]
pub fn ja001_worlds() -> [Scenario; 8] {
    Scenario::ALL
}

/// Capture every JA-001 world at Paused frame 0, 80×24, truecolor.
#[must_use]
pub fn ja001_paused_frame0_truecolor() -> Vec<Ja001Capture> {
    ja001_paused_frame0_at(Viewport::new(80, 24), CaptureColor::TrueColor)
}

/// Capture every JA-001 world at Paused frame 0 for one size/color.
#[must_use]
pub fn ja001_paused_frame0_at(viewport: Viewport, color: CaptureColor) -> Vec<Ja001Capture> {
    ja001_worlds()
        .into_iter()
        .map(|scenario| capture_world(scenario, viewport, color))
        .collect()
}

/// Capture every JA-001 world at every remaining size, truecolor.
#[must_use]
pub fn ja001_paused_frame0_remaining_sizes_truecolor() -> Vec<Ja001Capture> {
    JA001_REMAINING_SIZES
        .into_iter()
        .flat_map(|viewport| ja001_paused_frame0_at(viewport, CaptureColor::TrueColor))
        .collect()
}

/// Capture every JA-001 world at every listed size, truecolor.
#[must_use]
pub fn ja001_paused_frame0_all_sizes_truecolor() -> Vec<Ja001Capture> {
    JA001_SIZES
        .into_iter()
        .flat_map(|viewport| ja001_paused_frame0_at(viewport, CaptureColor::TrueColor))
        .collect()
}

/// Capture every JA-001 world at every size and remaining colour identity.
#[must_use]
pub fn ja001_paused_frame0_remaining_colors() -> Vec<Ja001Capture> {
    JA001_SIZES
        .into_iter()
        .flat_map(|viewport| {
            CaptureColor::ja001_remaining()
                .into_iter()
                .flat_map(move |color| ja001_paused_frame0_at(viewport, color))
        })
        .collect()
}

fn capture_world(scenario: Scenario, viewport: Viewport, color: CaptureColor) -> Ja001Capture {
    let mut session = DirectSession::paused_frame0(scenario, viewport, color);
    let initial = session.observe("initial");
    session.ticks(5);
    let after_ticks = session.observe("after-t5");
    Ja001Capture {
        initial,
        after_ticks,
    }
}

/// Initial route for Paused frame 0, matching `App::for_scenario`.
#[must_use]
pub const fn expected_route(scenario: Scenario) -> Route {
    match scenario {
        Scenario::FirstUse => Route::Intro,
        Scenario::Returning | Scenario::HardCases => Route::Manager,
        Scenario::AccountsMixed => Route::Accounts,
        Scenario::LaunchRunning | Scenario::LaunchFailure => Route::Cockpit,
        Scenario::CapsuleMulti | Scenario::OutroLast => Route::Capsule,
    }
}
