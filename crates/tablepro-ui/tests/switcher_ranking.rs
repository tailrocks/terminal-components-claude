//! Switcher ranking test carried from the monolith `model::tests`, which
//! needs the application shell that now lives in `tablepro-ui`.

#[test]
fn switcher_ranks_tables_first_and_prefix_first() -> Result<(), String> {
    let mut app = tablepro_ui::TableProApp::default();
    assert!(app.connect(0));
    assert!(app.workbench.open_table("orders"));
    let index = app.workbench.switcher();
    let items = index.search("ord");
    let first = items.first().ok_or("orders missing")?;
    assert_eq!(first.label, "orders");
    assert!(first.open);
    assert!(items.iter().any(|item| item.label == "order_items"));
    assert!(
        items
            .iter()
            .any(|item| item.target.group() == "Recent queries")
    );
    let all = index.search("");
    assert!(all.len() > 15);
    assert_eq!(all.first().map(|item| item.target.group()), Some("Tables"));
    assert!(
        index
            .search("public")
            .iter()
            .any(|item| item.label == "orders")
    );
    Ok(())
}
