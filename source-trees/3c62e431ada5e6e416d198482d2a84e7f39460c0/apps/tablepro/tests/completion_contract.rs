//! Completion is a statement-scoped model contract, independent of popup input.
use tablepro_app::{Catalog, CompletionKind, auto_trigger, completion_batch};

#[test]
fn alias_member_uses_the_statement_at_cursor_including_future_from() {
    let catalog = Catalog::acme_prod();
    let source = "SELECT o. FROM orders AS o; SELECT c. FROM customers c";
    let orders = completion_batch(source, 9, &catalog);
    assert_eq!(orders.replace, 9..9);
    assert!(
        orders
            .items
            .iter()
            .all(|item| item.kind == CompletionKind::Column)
    );
    assert!(orders.items.iter().any(|item| item.label == "total_amount"));
    let cursor = source.find("c.").map_or(0, |index| index + 2);
    let customers = completion_batch(source, cursor, &catalog);
    assert!(customers.items.iter().any(|item| item.label == "email"));
    assert!(
        !customers
            .items
            .iter()
            .any(|item| item.label == "total_amount")
    );
    let source = "SELECT * FROM orders o; SELECT o.";
    assert!(
        completion_batch(source, source.len(), &catalog)
            .items
            .is_empty()
    );
}

#[test]
fn replacement_range_and_schema_insert_preserve_surrounding_sql() -> Result<(), String> {
    let catalog = Catalog::acme_prod();
    let source = "SELECT * FROM ord WHERE status = 'pending'";
    let batch = completion_batch(source, 17, &catalog);
    assert_eq!(batch.replace, 14..17);
    let first = batch.items.first().ok_or("orders suggestion missing")?;
    assert_eq!(first.label, "orders");
    assert_eq!(first.matched, vec![0, 1, 2]);
    let mut applied = source.to_owned();
    applied.replace_range(batch.replace, &first.text);
    assert_eq!(applied, "SELECT * FROM orders WHERE status = 'pending'");
    let source = "SELECT * FROM eve";
    assert!(
        completion_batch(source, source.len(), &catalog)
            .items
            .iter()
            .any(|item| item.label == "events" && item.text == "analytics.events")
    );
    let source = "SELECT * FROM analytics.";
    assert!(
        completion_batch(source, source.len(), &catalog)
            .items
            .iter()
            .any(|item| item.label == "events" && item.text == "events")
    );
    Ok(())
}

#[test]
fn arbitrary_cursor_offsets_stay_at_utf8_boundaries_and_trigger_by_context() {
    let catalog = Catalog::acme_prod();
    let source = "SELECT * FROM café";
    for cursor in 0..=source.len().saturating_add(2) {
        let batch = completion_batch(source, cursor, &catalog);
        assert!(source.get(batch.replace.clone()).is_some());
        assert!(batch.replace.end <= source.len());
    }
    assert!(auto_trigger("SELECT * FROM ord", usize::MAX));
    assert!(auto_trigger("SELECT * FROM orders WHERE st", usize::MAX));
    assert!(!auto_trigger("SELECT * FROM orders WHERE s", usize::MAX));
    assert!(auto_trigger("SELECT o.", usize::MAX));
    assert!(!auto_trigger("SE", usize::MAX));
    assert!(auto_trigger("SEL", usize::MAX));
}

#[test]
fn completion_debug_does_not_publish_sql_alias_content() {
    let catalog = Catalog::acme_prod();
    let source = "SELECT secr FROM orders secret_alias";
    let batch = completion_batch(source, 11, &catalog);
    assert!(
        batch
            .items
            .iter()
            .any(|item| item.kind == CompletionKind::Alias && item.text == "secret_alias")
    );
    assert!(!format!("{batch:?}").contains("secret_alias"));
}

#[test]
fn pinned_boundary_policy_preserves_keyword_ranking_at_partial_cursor() {
    let catalog = Catalog::acme_prod();
    for (source, cursor, expected) in [
        ("INSERT INTO ord", 10, "INTO"),
        ("INSERT INTO ord", 11, "INTO"),
        ("DELETE FROM ord", 8, "FROM"),
        ("DELETE FROM ord", 9, "FROM"),
        ("DELETE FROM ord", 10, "FROM"),
        ("DELETE FROM ord", 11, "FROM"),
    ] {
        let batch = completion_batch(source, cursor, &catalog);
        assert_eq!(
            batch.items.first().map(|item| item.label.as_str()),
            Some(expected),
            "{source} at {cursor}"
        );
    }
    let batch = completion_batch("SELECT * FROM orders ORDER BY cre", 28, &catalog);
    for label in ["GROUP BY", "ORDER BY"] {
        assert_eq!(
            batch
                .items
                .iter()
                .find(|item| item.label == label)
                .map(|item| item.score),
            Some(430)
        );
    }
}

#[test]
fn alias_matches_keep_original_grapheme_ordinals() {
    let catalog = Catalog::acme_prod();
    let source = "SELECT él FROM orders élève";
    let cursor = "SELECT él".len();
    let batch = completion_batch(source, cursor, &catalog);
    assert!(
        batch
            .items
            .iter()
            .any(|item| item.kind == CompletionKind::Alias
                && item.label == "élève"
                && item.matched == vec![0, 1])
    );
    assert_eq!(source.get(batch.replace), Some("él"));
}
