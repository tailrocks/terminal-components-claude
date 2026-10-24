//! Chosen-item ink under a modal backdrop: controllable, default unchanged.
//!
//! Frozen evidence: `crates/termrock-controls/tests/frozen_ink_pins.rs` pins
//! checked marks (`Part::MARKER`, `CHECKED`) keeping the accent while focus
//! is elsewhere and under a modal backdrop (owner decision 2026-10-17,
//! `InactiveSelection::Accent`), while `Theme::chosen_in(Accent, ..)` dropped
//! to primary ink under a backdrop — the delta that kept the recipes in place
//! (migration matrix, 2026-10-21). [`BackdropSelection`] makes the
//! under-backdrop behavior explicit; the reference default stays `Neutral`
//! and is not flipped.

use termrock_core::response::StateFlags;
use termrock_theme::{
    BACKDROP_SELECTIONS, BackdropSelection, COLOR_LEVELS, InactiveSelection, Theme,
};

#[test]
fn the_reference_default_still_neutralizes_accent_under_a_backdrop() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        for under_modal in [false, true] {
            assert_eq!(
                theme.inactive_look(InactiveSelection::Accent, under_modal),
                theme.inactive_look_with(
                    InactiveSelection::Accent,
                    under_modal,
                    BackdropSelection::Neutral
                ),
                "{level:?}: the legacy resolver is the Neutral backdrop policy"
            );
        }
        assert_eq!(
            theme.chosen_in(InactiveSelection::Accent, true, StateFlags::empty()),
            theme.color.fg[0],
            "{level:?}: the default still drops to primary under a backdrop"
        );
    }
}

#[test]
fn the_accent_backdrop_policy_keeps_the_frozen_mark_ink() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        for under_modal in [false, true] {
            for flags in [
                StateFlags::CHECKED,
                StateFlags::CHECKED | StateFlags::FOCUSED,
                StateFlags::SELECTED,
                StateFlags::SELECTED | StateFlags::FOCUSED,
                StateFlags::empty(),
            ] {
                assert_eq!(
                    theme.chosen_in_with(
                        InactiveSelection::Accent,
                        under_modal,
                        BackdropSelection::Accent,
                        flags
                    ),
                    theme.color.accent,
                    "{level:?} under_modal={under_modal} {flags:?}: the frozen mark keeps accent"
                );
            }
        }
    }
}

#[test]
fn only_the_accent_policy_reacts_to_a_backdrop() {
    for backdrop in BACKDROP_SELECTIONS {
        for under_modal in [false, true] {
            for policy in [InactiveSelection::Neutral, InactiveSelection::None] {
                assert_eq!(
                    Theme::junie().inactive_look_with(policy, under_modal, backdrop),
                    policy,
                    "{policy:?} {backdrop:?}: non-accent policies pass through"
                );
            }
        }
    }
    assert_eq!(BackdropSelection::default(), BackdropSelection::Neutral);
}

#[test]
fn focus_still_wins_over_every_policy() {
    for backdrop in BACKDROP_SELECTIONS {
        for policy in InactiveSelection::ALL {
            assert_eq!(
                Theme::junie().chosen_in_with(
                    policy,
                    true,
                    backdrop,
                    StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE
                ),
                Theme::junie().color.accent,
                "{policy:?} {backdrop:?}: a focused widget keeps the accent"
            );
        }
    }
}
