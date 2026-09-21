#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]
//! Membership: 54 rows, lane split, finite expansion totals.

use oracle_components::{
    ARCHITECTURE_FAMILY_COUNT, COMPOSITION_FAMILY_COUNT, DIRECT_FAMILY_COUNT, FAMILY_COUNT, Family,
    disposition, expand, membership,
};

#[test]
fn parity_rows_are_54_unique_in_ledger_order() {
    let rows = oracle_components::rows().expect("parse parity TSV");
    assert_eq!(rows.len(), FAMILY_COUNT);
    for (row, expected) in rows.iter().zip(Family::ALL) {
        assert_eq!(row.family, expected);
    }
    for row in &rows {
        assert!(
            row.owning_task_ids.contains("TASK-006"),
            "{} has no TASK-006 owner: {}",
            row.family.slug(),
            row.owning_task_ids
        );
        assert!(
            !row.reference_implementation.is_empty(),
            "{} O: empty",
            row.family.slug()
        );
        assert!(
            !row.main_implementation.is_empty(),
            "{} M: empty",
            row.family.slug()
        );
    }
}

#[test]
fn lane_split_is_42_5_7() {
    let mut counts = [0_usize; 3];
    for family in Family::ALL {
        match disposition(family).lane {
            oracle_components::Lane::Direct => counts[0] += 1,
            oracle_components::Lane::Composition => counts[1] += 1,
            oracle_components::Lane::Architecture => counts[2] += 1,
        }
    }
    assert_eq!(counts[0], DIRECT_FAMILY_COUNT);
    assert_eq!(counts[1], COMPOSITION_FAMILY_COUNT);
    assert_eq!(counts[2], ARCHITECTURE_FAMILY_COUNT);
    assert_eq!(
        counts[0] + counts[1] + counts[2],
        FAMILY_COUNT,
        "lanes must partition all 54 families"
    );
}

#[test]
fn expansion_totals_match_the_closed_formula() {
    let member = membership().expect("membership");
    assert_eq!(member.family_rows, 54);
    assert_eq!(member.direct_families, 42);
    assert_eq!(member.composition_families, 5);
    assert_eq!(member.architecture_families, 7);
    // Plain frames: state pairs x 2 sizes x 2 origins x 4 colors.
    assert_eq!(member.plain_frames, member.state_pairs * 16);
    // Fade: 10 families x 4 heights, the picker at 2 (list threshold),
    // the scroll panel at 3 (framed h3 is position-invariant); each height
    // carries 4 positions x 4 colors: 640 + 32 + 48.
    assert_eq!(member.fade_frames, 10 * 4 * 4 * 4 + 2 * 4 * 4 + 3 * 4 * 4);
    assert_eq!(member.fade_frames, 720);
    // Mutations: 5 kinds x 4 colors.
    assert_eq!(member.mutation_frames, 20);
    assert_eq!(
        member.expanded_cases,
        member.plain_frames + member.fade_frames + member.mutation_frames
    );
    let expanded = expand().expect("expand");
    assert_eq!(expanded.len(), member.expanded_cases);
}

#[test]
fn expansion_identities_carry_the_components_prefix() {
    for case in expand().expect("expand") {
        let identity = case.identity();
        assert!(
            identity.starts_with("components/"),
            "identity escapes namespace: {identity}"
        );
        assert!(
            identity.contains(case.family.slug()),
            "identity drops family: {identity}"
        );
    }
}
