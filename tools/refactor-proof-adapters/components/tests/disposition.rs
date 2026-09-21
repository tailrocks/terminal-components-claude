#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]
//! Dispositions: non-frame bindings, mappings, explicit non-applicability.

use oracle_components::{
    AdapterError, ComponentState, ExpandedCase, Family, Lane, architecture_owners, capture,
    disposition, excluded_states, mapping,
};
use oracle_components::{ColorSpec, Facet, Origin, TerminalSize};

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
fn testing_registry_is_architecture_only_with_task_073_and_031_binding() {
    let family = Family::TestingRegistry;
    let it = disposition(family);
    assert_eq!(it.lane, Lane::Architecture);
    assert_eq!(architecture_owners(family), &["TASK-073", "TASK-031"]);
    assert!(
        it.rationale.contains("no screenshot")
            && it.rationale.contains("direct-capture")
            && it.rationale.contains("PTY identity"),
        "rationale must name every forbidden identity: {}",
        it.rationale
    );
    // No frame exists for this row under any state.
    for state in ComponentState::ALL {
        let error =
            capture(&plain(family, state)).expect_err("testing-registry must have no frame");
        assert_eq!(
            error,
            AdapterError::ArchitectureHasNoFrame { family },
            "state {} leaked a frame",
            state.token()
        );
    }
}

#[test]
fn every_architecture_family_has_owners_and_no_frame() {
    for family in Family::ALL {
        if disposition(family).lane != Lane::Architecture {
            continue;
        }
        assert!(
            !architecture_owners(family).is_empty(),
            "{} binds no future owner",
            family.slug()
        );
        let error =
            capture(&plain(family, ComponentState::Base)).expect_err("architecture frame leaked");
        assert_eq!(error, AdapterError::ArchitectureHasNoFrame { family });
    }
}

#[test]
fn every_composition_family_has_a_mapping_or_a_corpus_rationale() {
    for family in Family::ALL {
        if disposition(family).lane != Lane::Composition {
            continue;
        }
        let rationale = disposition(family).rationale;
        assert!(
            rationale.contains("observed through") || mapping(family).is_some(),
            "{} has neither corpus binding nor mapping",
            family.slug()
        );
        let error =
            capture(&plain(family, ComponentState::Base)).expect_err("composition frame leaked");
        assert_eq!(error, AdapterError::ArchitectureHasNoFrame { family });
    }
}

#[test]
fn new_architecture_components_carry_explicit_old_to_new_mappings() {
    let mapped = [
        Family::Form,
        Family::FilterList,
        Family::NavList,
        Family::PickerChain,
        Family::HelpOverlay,
        Family::Wizard,
        Family::Meter,
        Family::TooSmall,
        Family::Table,
        Family::ScrollPanel,
        Family::Chips,
        Family::SplitPane,
        Family::StatusbarSegments,
        Family::ScrollStateRegion,
    ];
    assert_eq!(mapped.len(), 14);
    for family in mapped {
        let entry = mapping(family)
            .unwrap_or_else(|| panic!("{} lacks an old-to-new mapping", family.slug()));
        assert!(!entry.oracle_side.is_empty());
        assert!(!entry.production_side.is_empty());
        assert!(!entry.fixture.is_empty());
    }
}

#[test]
fn every_excluded_state_has_an_evidence_backed_reason() {
    for family in Family::ALL {
        for (state, reason) in excluded_states(family) {
            assert!(
                reason.contains(state.requirement()),
                "{} / {} reason drops the requirement: {reason}",
                family.slug(),
                state.token()
            );
        }
    }
}

#[test]
fn direct_families_cover_every_cp_common_state_somewhere() {
    // Each CP-COMMON state must be applicable to at least one Direct family,
    // or the corpus silently drops a required state axis.
    for state in ComponentState::ALL {
        let covered = Family::ALL.into_iter().any(|family| {
            disposition(family).lane == Lane::Direct
                && oracle_components::applicable_states(family).contains(&state)
        });
        assert!(covered, "no Direct family covers {}", state.token());
    }
}
