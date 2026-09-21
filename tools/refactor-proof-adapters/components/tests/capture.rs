#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]
//! Capture: every case executes through production paths and observes a
//! complete frame; non-base states repaint; repeats are equal.

use oracle_components::{
    ColorSpec, ComponentState, ExpandedCase, Facet, FadePosition, Family, Lane, Origin,
    TerminalSize, applicable_states, capture, capture_repeat_pair, disposition, expand,
};

fn plain(family: Family, state: ComponentState) -> ExpandedCase {
    ExpandedCase {
        family,
        state,
        size: TerminalSize::new(120, 40),
        origin: Origin::AXIS[0],
        color: ColorSpec::TrueColor,
        facet: Facet::None,
    }
}

#[test]
fn every_expanded_case_captures_a_complete_frame() {
    let cases = expand().expect("expand");
    assert!(!cases.is_empty());
    for case in &cases {
        let observation =
            capture(case).unwrap_or_else(|error| panic!("{} failed: {error}", case.identity()));
        assert!(
            observation.is_complete(),
            "{} is incomplete",
            case.identity()
        );
        assert_eq!(observation.family, case.family);
        assert_eq!(observation.state, case.state);
    }
}

#[test]
fn every_direct_base_repeats_exactly() {
    for family in Family::ALL {
        if disposition(family).lane != Lane::Direct {
            continue;
        }
        let case = plain(family, ComponentState::Base);
        let (first, second) = capture_repeat_pair(&case)
            .unwrap_or_else(|error| panic!("{} failed: {error}", case.identity()));
        assert_eq!(first, second, "{} is not repeat-equal", case.identity());
    }
}

#[test]
fn every_state_repaints_against_its_base() {
    // A state that paints exactly its base frame is a simulated state.
    for family in Family::ALL {
        if disposition(family).lane != Lane::Direct {
            continue;
        }
        let base = capture(&plain(family, ComponentState::Base)).expect("base captures");
        for state in applicable_states(family) {
            if *state == ComponentState::Base {
                continue;
            }
            let observation = capture(&plain(family, *state))
                .unwrap_or_else(|error| panic!("{} failed: {error}", family.slug()));
            assert_ne!(
                observation.cells,
                base.cells,
                "{} state {} paints its base frame",
                family.slug(),
                state.token()
            );
        }
    }
}

#[test]
fn fade_positions_scroll_truthfully() {
    // Top/Middle/Bottom at a scrolled height must all paint differently;
    // Fits (short content) must differ from Top (tall content at offset 0).
    for family in oracle_components::expansion::FADE_FAMILIES {
        for height in oracle_components::fade_heights(family).iter().copied() {
            let at = |position: FadePosition| {
                capture(&ExpandedCase {
                    family,
                    state: ComponentState::Base,
                    size: TerminalSize::new(120, 40),
                    origin: Origin::AXIS[0],
                    color: ColorSpec::TrueColor,
                    facet: Facet::FadeViewport { height, position },
                })
                .unwrap_or_else(|error| {
                    panic!("{} h{height} {position:?} failed: {error}", family.slug())
                })
            };
            let (fits, top, middle, bottom) = (
                at(FadePosition::Fits),
                at(FadePosition::Top),
                at(FadePosition::Middle),
                at(FadePosition::Bottom),
            );
            for observation in [&fits, &top, &middle, &bottom] {
                assert!(observation.is_complete(), "{} h{height}", family.slug());
            }
            assert_ne!(
                fits.cells,
                top.cells,
                "{} h{height}: fits == top",
                family.slug()
            );
            assert_ne!(
                top.cells,
                middle.cells,
                "{} h{height}: top == middle",
                family.slug()
            );
            assert_ne!(
                middle.cells,
                bottom.cells,
                "{} h{height}: middle == bottom",
                family.slug()
            );
            assert_ne!(
                top.cells,
                bottom.cells,
                "{} h{height}: top == bottom",
                family.slug()
            );
        }
    }
}

#[test]
fn text_corpus_carries_wide_continuation_cells() {
    for family in [
        Family::TextInput,
        Family::TextArea,
        Family::CodeEditor,
        Family::TextViewport,
    ] {
        let observation = capture(&plain(family, ComponentState::Base)).expect("base captures");
        assert!(
            observation.has_continuation(),
            "{} base frame has no wide-continuation cell",
            family.slug()
        );
    }
}

#[test]
fn scroll_panel_follow_policy_is_visible() {
    let at = |state: ComponentState| {
        capture(&ExpandedCase {
            family: Family::ScrollPanel,
            state,
            size: TerminalSize::new(40, 10),
            origin: Origin::AXIS[0],
            color: ColorSpec::TrueColor,
            facet: Facet::None,
        })
        .expect("scroll-panel captures")
        .text()
    };
    let (base, active, inactive) = (
        at(ComponentState::Base),
        at(ComponentState::Active),
        at(ComponentState::Inactive),
    );
    assert!(
        active.contains("10:10 live line"),
        "active tails:\n{active}"
    );
    assert!(!base.contains("live line"), "base moved:\n{base}");
    assert!(
        inactive.contains("caught up"),
        "inactive shifted:\n{inactive}"
    );
    assert!(
        !inactive.contains("live line"),
        "inactive tailed:\n{inactive}"
    );
}

#[test]
fn focus_hover_states_publish_runtime_identity() {
    let observation =
        capture(&plain(Family::Button, ComponentState::Focus)).expect("focus captures");
    assert!(
        observation.focus.is_some(),
        "button focus frame has no focused id"
    );
    let observation =
        capture(&plain(Family::Button, ComponentState::Hover)).expect("hover captures");
    assert!(
        observation.hover.is_some(),
        "button hover frame has no hovered id"
    );
}
