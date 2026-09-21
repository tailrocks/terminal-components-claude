//! Exact SC membership: 23 oracle pages including Diff, 368 default frames.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]

use std::collections::BTreeSet;

use oracle_showcase::{
    DEFAULT_PAGE_FRAME_COUNT, ORACLE_PAGE_COUNT, OraclePageId, UI_ORACLE, default_page_frames,
    expand, membership, rows,
};

#[test]
fn oracle_pages_are_twenty_three_including_diff() {
    assert_eq!(OraclePageId::ALL.len(), ORACLE_PAGE_COUNT);
    assert_eq!(UI_ORACLE, "02f5294bfdbf38004cc49130d0aff1d01f31434c");
    assert_eq!(OraclePageId::Diff.slug(), "diff");
    assert_eq!(OraclePageId::Diff.title(), "Diff");
    assert!(
        !OraclePageId::Diff.production_present(),
        "candidate production has no Diff route; adapter must not invent one"
    );
    assert_eq!(oracle_showcase::production_page_len(), 22);
    assert_eq!(oracle_showcase::production_nav_len(), 22);
}

#[test]
fn catalog_membership_is_exact() {
    let facts = membership().unwrap();
    assert_eq!(facts.oracle_pages, 23);
    assert_eq!(facts.default_page_frames, DEFAULT_PAGE_FRAME_COUNT);
    assert_eq!(facts.production_pages, 22);
    assert_eq!(facts.production_nav, 22);
    assert!(!facts.diff_production_present);
    assert_eq!(facts.scenario_rows, rows().unwrap().len());
    assert!(facts.scenario_rows >= 23, "at least the 23 SC-BASE rows");
    assert!(facts.expanded_cases >= facts.default_page_frames);

    let ids: BTreeSet<_> = rows()
        .unwrap()
        .into_iter()
        .map(|row| row.scenario_id)
        .collect();
    assert!(ids.contains("SC-BASE-overview"));
    assert!(ids.contains("SC-BASE-diff"));
    assert!(ids.contains("SC-FOCUS"));
    assert!(ids.contains("SC-RESIZE"));
    assert!(ids.contains("SC-SHELL-NAV"));
    assert!(ids.contains("SC-SETTINGS-GENERAL"));
    assert!(ids.contains("SC-SETTINGS-MEMBERS"));
    assert!(ids.contains("SC-SETTINGS-ENV"));
    assert!(ids.contains("SC-GRID-EDIT"));
    assert!(ids.contains("SC-FORM-SUBMIT"));
    assert!(ids.contains("SC-BUTTON-TIME"));
}

#[test]
fn default_page_frames_cover_every_oracle_page_size_and_color() {
    let frames = default_page_frames().unwrap();
    assert_eq!(frames.len(), 23 * 4 * 4);
    let mut pages = BTreeSet::new();
    let mut sizes = BTreeSet::new();
    let mut colors = BTreeSet::new();
    for frame in &frames {
        assert!(frame.scenario_id.starts_with("SC-BASE-"));
        pages.insert(frame.page);
        sizes.insert(frame.size);
        colors.insert(frame.color);
    }
    assert_eq!(pages.len(), 23);
    assert!(pages.contains(&OraclePageId::Diff));
    assert_eq!(sizes.len(), 4);
    assert_eq!(colors.len(), 4);
}

#[test]
fn expansion_has_no_duplicate_identities() {
    let cases = expand().unwrap();
    let ids: BTreeSet<_> = cases
        .iter()
        .map(oracle_showcase::ExpandedCase::identity)
        .collect();
    assert_eq!(ids.len(), cases.len());
}
