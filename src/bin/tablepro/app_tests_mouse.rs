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
