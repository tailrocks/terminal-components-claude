//! Boundary policy changes only source-defined ranking bonuses.
use junie_tui::{FuzzyBoundary, fuzzy, fuzzy_with_boundary};
#[test]
fn default_word_boundary_results_are_unchanged() {
    for label in [
        "orders",
        "my_orders",
        "public.orders",
        "ORDER BY",
        "order-items",
        "reorders",
        "o r d",
        "İstanbul_Ünïted",
        "👨‍👩‍👧‍👦x",
    ] {
        for word in ["", "ord", "BY", "items", "ün", "x", "absent"] {
            assert_eq!(
                fuzzy(label, word),
                fuzzy_with_boundary(label, word, FuzzyBoundary::Word)
            );
        }
    }
}
#[test]
fn identifier_boundary_preserves_pinned_sql_scores_and_order() {
    for (label, query, word_score, identifier_score) in [
        ("GROUP BY", "B", 10, 30),
        ("ORDER BY", "B", 10, 30),
        ("INSERT INTO", "INT", 10, 30),
        ("DELETE FROM", "FR", 10, 30),
        ("SELECT * FROM", "FR", 10, 30),
        ("order-items", "items", 10, 30),
        ("my_orders", "ord", 10, 10),
        ("public.orders", "ord", 10, 10),
    ] {
        assert_eq!(
            fuzzy_with_boundary(label, query, FuzzyBoundary::Word).map(|m| m.0),
            Some(word_score)
        );
        assert_eq!(
            fuzzy_with_boundary(label, query, FuzzyBoundary::Identifier).map(|m| m.0),
            Some(identifier_score)
        );
    }
    let mut rows = [("INSERT INTO", 400u32), ("INTO", 420u32)];
    rows.sort_by_key(|(label, base)| {
        base.saturating_add(
            fuzzy_with_boundary(label, "INT", FuzzyBoundary::Identifier).map_or(u32::MAX, |m| m.0),
        )
    });
    assert_eq!(rows.first().map(|row| row.0), Some("INTO"));
}
#[test]
fn policies_keep_original_grapheme_ordinals_and_case_folding() {
    for boundary in [FuzzyBoundary::Word, FuzzyBoundary::Identifier] {
        assert_eq!(
            fuzzy_with_boundary("İstanbul_Ünïted", "ün", boundary),
            Some((10, vec![9, 10]))
        );
        assert_eq!(
            fuzzy_with_boundary("👨‍👩‍👧‍👦x", "x", boundary),
            Some((30, vec![1]))
        );
        assert_eq!(
            fuzzy_with_boundary("orders", "ord", boundary),
            Some((0, vec![0, 1, 2]))
        );
        assert_eq!(fuzzy_with_boundary("abc", "z", boundary), None);
        assert_eq!(
            fuzzy_with_boundary("o r d", "ord", boundary),
            Some((64, vec![0, 2, 4]))
        );
    }
}
