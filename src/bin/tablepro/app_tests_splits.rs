//! Pane seams in TablePro through the real App: the explorer, the query
//! editor / results split and the history list / detail split resize by
//! mouse (they were fixed or keyboard-only), cell by cell, and a
//! double-click restores the default.

use junie_tui::core::event::MouseKind;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::workbench::WorkTab;

fn query_seam(h: &H) -> (Rect, u16) {
    match h.wb().active_tab() {
        Some(WorkTab::Query(q)) => (
            q.seam.area,
            q.split.layout(q.seam.dir, q.split_area, 1).0.height,
        ),
        _ => panic!("expected a query tab"),
    }
}

fn drag(h: &mut H, from: (u16, u16), to: (u16, u16)) {
    h.mouse(MouseKind::Down, from.0, from.1);
    h.mouse(MouseKind::Drag, to.0, to.1);
    h.mouse(MouseKind::Up, to.0, to.1);
}

fn double_click(h: &mut H, x: u16, y: u16) {
    // let the click chain of the drag release run out first
    h.ticks(20);
    h.click(x, y);
    h.click(x, y);
}

#[test]
fn the_query_editor_and_results_resize_by_the_seam() {
    let mut h = H::connected(120, 40);
    let (qx, qy) = h.find("Query 1").expect("the query tab");
    h.click(qx, qy);
    let (seam, editor_h) = query_seam(&h);
    assert!(!seam.is_empty(), "the split draws a seam");
    let x = seam.x + seam.width / 2;
    drag(&mut h, (x, seam.y), (x, seam.y + 3));
    let (moved, grown) = query_seam(&h);
    assert_eq!(grown, editor_h + 3, "the editor grew by the rows dragged");
    assert_eq!(moved.y, seam.y + 3, "the seam stays under the pointer");
    double_click(&mut h, x, moved.y);
    assert_eq!(
        query_seam(&h).1,
        editor_h,
        "a double-click resets the split"
    );
}

#[test]
fn the_explorer_resizes_by_its_seam() {
    let mut h = H::connected(120, 40);
    let seam = h.wb().explorer_seam.area;
    assert!(!seam.is_empty(), "the explorer has a seam");
    let y = seam.y + seam.height / 2;
    // grab the seam one cell left of the rule (the padding)
    drag(&mut h, (seam.x - 1, y), (seam.x + 5, y));
    assert_eq!(h.wb().explorer_seam.area.x, seam.x + 6);
    double_click(&mut h, seam.x + 6, y);
    assert_eq!(
        h.wb().explorer_seam.area.x,
        seam.x,
        "back to the default width"
    );
}

#[test]
fn the_history_list_and_detail_resize_by_the_seam() {
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    let seam = match h.wb().active_tab() {
        Some(WorkTab::History(t)) => t.seam.area,
        _ => panic!("expected the history tab"),
    };
    assert!(!seam.is_empty(), "the history split draws a seam");
    let y = seam.y + seam.height / 2;
    drag(&mut h, (seam.x, y), (seam.x - 4, y));
    let now = match h.wb().active_tab() {
        Some(WorkTab::History(t)) => t.seam.area,
        _ => unreachable!(),
    };
    assert_eq!(now.x, seam.x - 4);
}
