//! Mouse-only navigation in tablepro (docs/audits/design-review-2026-10/mouse.md):
//! every state reachable, and every view left, by pointer alone.

use crate::app_tests::H;

// mouse #6: the empty connection list offers the way in as a button
#[test]
fn the_first_connection_is_created_by_mouse_from_the_empty_list() {
    let mut h = H::new(120, 40);
    h.app.connections.connections.clear();
    h.draw();
    assert!(h.text().contains("No connections"), "{}", h.text());
    assert!(!h.text().contains("Save & connect"));
    let (x, y) = h.find("New connection").expect("a New connection button");
    assert_eq!(
        h.app.hits.hit(ratatui::layout::Position::new(x, y)),
        Some(crate::connections::NEW_ID)
    );
    h.click(x, y);
    assert!(h.text().contains("Save & connect"), "{}", h.text());
}

// mouse #7: the query tab runs, runs all, explains and cancels by buttons
#[test]
fn the_query_toolbar_runs_explains_and_cancels_by_mouse() {
    use crate::tabs::{TOOLBAR_CANCEL, TOOLBAR_EXPLAIN, TOOLBAR_RUN, TOOLBAR_RUN_ALL};
    let mut h = H::connected(120, 40);
    h.key(ratatui::crossterm::event::KeyCode::Tab); // editor
    h.type_str("SELECT * FROM orders LIMIT 5");
    h.key(ratatui::crossterm::event::KeyCode::Esc);
    let at = |h: &H, id| {
        let r = h
            .app
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("toolbar button {id:?} missing:\n{}", h.text()));
        (r.x + 1, r.y)
    };
    for id in [TOOLBAR_RUN, TOOLBAR_RUN_ALL, TOOLBAR_EXPLAIN] {
        assert!(h.app.hits.area_of(id).is_some(), "{}", h.text());
    }
    assert!(h.app.hits.area_of(TOOLBAR_CANCEL).is_none());
    // Run
    let (x, y) = at(&h, TOOLBAR_RUN);
    h.click(x, y);
    assert!(h.wb_query().is_running(), "Run did not start the query");
    // Cancel shows while it runs
    let (x, y) = at(&h, TOOLBAR_CANCEL);
    h.click(x, y);
    assert!(!h.wb_query().is_running(), "Cancel did not stop it");
    // Run all, then Explain
    let (x, y) = at(&h, TOOLBAR_RUN_ALL);
    h.click(x, y);
    assert!(h.wb_query().is_running());
    h.ticks(10);
    assert!(!h.text().contains("Planning"), "{}", h.text());
    let (x, y) = at(&h, TOOLBAR_EXPLAIN);
    h.click(x, y);
    h.ticks(10);
    assert!(h.text().contains("Planning"), "{}", h.text());
}
