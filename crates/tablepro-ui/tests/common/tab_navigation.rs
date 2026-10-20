use tablepro_ui::{Tab, TableProApp};
use termrock::{Id, KeyCode};
use termrock_test_support::Harness;

/// Drive focus with the reference editor's key contract. Plain Tab indents an
/// active code editor; F6 leaves it. This keeps traversal tests from changing
/// the live SQL draft while preserving their original destination assertion.
pub(super) fn tab_to(harness: &mut Harness<TableProApp>, target: Id) -> bool {
    let original_query = match harness.app().workbench.active() {
        Some(Tab::Query(query)) => Some(query.editor_state.editor().text().to_owned()),
        _ => None,
    };
    let query_editor = harness
        .app()
        .workbench
        .active_key()
        .map(|key| key.control("query"));
    let budget = harness.ring().entries().len().saturating_add(1);
    for _ in 0..budget {
        if harness.focus() == Some(target) {
            break;
        }
        let key = if harness.focus().is_some() && harness.focus() == query_editor {
            KeyCode::F(6)
        } else {
            KeyCode::Tab
        };
        let _ = harness.key(key);
        if let Some(before) = original_query.as_deref()
            && let Some(Tab::Query(query)) = harness.app().workbench.active()
        {
            assert_eq!(
                query.editor_state.editor().text(),
                before,
                "focus traversal changed the SQL draft"
            );
        }
    }
    harness.focus() == Some(target)
}
