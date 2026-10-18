





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



#[test]
fn history_buttons_show_their_tooltip_after_a_rest() {
    let mut h = H::connected(120, 40);
    h.ctrl('y');
    let (x, y) = h.find("Open in new tab").expect("the open button");
    h.mouse(MouseKind::Move, x + 2, y);
    assert!(!h.text().contains("Open the selected query"), "not at once");
    let rested = h
        .app
        .clock
        .origin()
        .checked_sub(std::time::Duration::from_millis(700));
    h.app.clock = junie_tui::clock::WallClock::with_origin(rested.expect("an origin 700 ms back"));
    h.draw();
    let text = h.text();
    assert!(
        text.contains("Open the selected query in a new tab  Enter"),
        "{text}"
    );

    h.mouse(MouseKind::Move, 0, 0);
    assert!(!h.text().contains("Open the selected query"));
}



#[test]
fn the_pointer_shape_follows_fields_and_seams() {
    use junie_tui::core::shape::{CaretShape, PointerShape, shapes_for};
    let mut h = H::connected(120, 40);
    let shape = |h: &H| shapes_for(&h.app.shell.pointer, &h.app.shell.hits).pointer;
    let seam = h.wb().explorer_seam.area;
    h.mouse(MouseKind::Move, seam.x, seam.y + 3);
    assert_eq!(shape(&h), Some(PointerShape::ColResize));
    let (qx, qy) = h.find("Query 1").expect("the query tab");
    h.click(qx, qy);
    let (qs, _) = query_seam(&h);
    h.mouse(MouseKind::Move, qs.x + 4, qs.y);
    assert_eq!(shape(&h), Some(PointerShape::RowResize));
    h.mouse(MouseKind::Move, qs.x + 4, qs.y - 3);
    assert_eq!(shape(&h), Some(PointerShape::Text), "the SQL editor");
    h.mouse(MouseKind::Move, qx, qy);
    assert_eq!(shape(&h), Some(PointerShape::Default), "a tab: the arrow");
    let caret = shapes_for(&h.app.shell.pointer, &h.app.shell.hits).caret;
    assert_eq!(caret, Some(CaretShape::Bar), "the macOS caret");

    h.app.shell.pointer.config = h.app.shell.pointer.config.pointer_shapes(false);
    assert_eq!(shape(&h), None);
}
