//! Switcher projections retain typed destination identity across tab changes.
use tablepro_app::{SwitchTarget, Tab, TableProApp};

#[test]
fn captured_open_tab_target_survives_reorder_but_never_retargets_after_removal()
-> Result<(), String> {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("orders"));
    let original = app.workbench.active_key().ok_or("orders key")?;
    let _ = app.workbench.new_query("");
    let index = app.workbench.switcher();
    let captured = index
        .search("orders")
        .into_iter()
        .find_map(|item| match item.target {
            SwitchTarget::OpenTab(key) if key == original => Some(key),
            _ => None,
        })
        .ok_or("open orders target")?;
    let mut keys = app
        .workbench
        .tabs()
        .iter()
        .map(tablepro_app::TabRecord::key)
        .collect::<Vec<_>>();
    keys.reverse();
    assert!(app.workbench.reorder_tabs(&keys));
    assert!(app.workbench.activate(captured));
    assert!(
        matches!(app.workbench.active(), Some(Tab::Table(table)) if table.table.name == "orders")
    );
    let position = app.workbench.active_index().ok_or("orders position")?;
    assert!(app.workbench.close_tab(position));
    assert!(!app.workbench.activate(captured));
    let current = app.workbench.switcher();
    assert!(
        !current
            .items
            .iter()
            .any(|item| matches!(item.target, SwitchTarget::OpenTab(key) if key == original))
    );
    assert!(current.items.iter().any(|item| item.label == "orders"
        && matches!(item.target, SwitchTarget::Table { .. })
        && !item.open));
    assert!(app.workbench.open_table("orders"));
    assert_ne!(app.workbench.active_key(), Some(original));
    assert!(!app.workbench.activate(captured));
    assert!(
        index
            .items
            .iter()
            .any(|item| matches!(item.target, SwitchTarget::OpenTab(key) if key == original))
    );
    Ok(())
}

#[test]
fn source_groups_open_markers_path_search_and_match_offsets_are_preserved() -> Result<(), String> {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("orders"));
    let index = app.workbench.switcher();
    let matches = index.search("ord");
    let first = matches.first().ok_or("orders match")?;
    assert_eq!(first.label, "orders");
    assert!(first.open);
    assert_eq!(first.matched, vec![0, 1, 2]);
    assert_eq!(first.target.group(), "Tables");
    assert!(
        matches
            .iter()
            .any(|item| item.target.group() == "Recent queries")
    );
    let path = index.search("  public  ");
    assert!(
        path.iter()
            .any(|item| item.label == "orders" && item.matched.is_empty())
    );
    let all = index.search("");
    assert!(all.len() > 15);
    for group in [
        "Tables",
        "Views",
        "Open tabs",
        "Schemas",
        "Databases",
        "Recent queries",
        "Connections",
    ] {
        assert!(
            all.iter().any(|item| item.target.group() == group),
            "{group}"
        );
    }
    assert_eq!(all.first().map(|item| item.target.group()), Some("Tables"));
    Ok(())
}

#[test]
fn history_sql_remains_visible_to_renderer_but_not_debug() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let _ = app
        .workbench
        .new_query("SELECT id AS secret_alias FROM orders LIMIT 1");
    assert!(app.workbench.execute_active().is_ok());
    let index = app.workbench.switcher();
    assert!(
        index
            .items
            .iter()
            .any(|item| item.label.contains("secret_alias"))
    );
    assert!(!format!("{index:?}").contains("secret_alias"));
}
