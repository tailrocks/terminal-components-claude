use crate::app_tests::H;

#[test]
fn the_first_connection_is_created_by_mouse_from_the_empty_list() {
    let mut h = H::new(120, 40);
    h.app.connections.connections.clear();
    h.draw();
    assert!(h.text().contains("No connections"), "{}", h.text());
    assert!(!h.text().contains("Save & connect"));
    let (x, y) = h.find("New connection").expect("a New connection button");
    assert_eq!(
        h.app.shell.hits.hit(ratatui::layout::Position::new(x, y)),
        Some(crate::connections::NEW_ID)
    );
    h.click(x, y);
    assert!(h.text().contains("Save & connect"), "{}", h.text());
}

#[test]
fn the_query_toolbar_runs_explains_and_cancels_by_mouse() {
    use crate::tabs::{TOOLBAR_CANCEL, TOOLBAR_EXPLAIN, TOOLBAR_RUN, TOOLBAR_RUN_ALL};
    let mut h = H::connected(120, 40);
    h.key(ratatui::crossterm::event::KeyCode::Tab);
    h.type_str("SELECT * FROM orders LIMIT 5");
    h.key(ratatui::crossterm::event::KeyCode::Esc);
    let at = |h: &H, id| {
        let r = h
            .app
            .shell
            .hits
            .area_of(id)
            .unwrap_or_else(|| panic!("toolbar button {id:?} missing:\n{}", h.text()));
        (r.x + 1, r.y)
    };
    for id in [TOOLBAR_RUN, TOOLBAR_RUN_ALL, TOOLBAR_EXPLAIN] {
        assert!(h.app.shell.hits.area_of(id).is_some(), "{}", h.text());
    }
    assert!(h.app.shell.hits.area_of(TOOLBAR_CANCEL).is_none());

    let (x, y) = at(&h, TOOLBAR_RUN);
    h.click(x, y);
    assert!(h.wb_query().is_running(), "Run did not start the query");

    let (x, y) = at(&h, TOOLBAR_CANCEL);
    h.click(x, y);
    assert!(!h.wb_query().is_running(), "Cancel did not stop it");

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

#[test]
fn the_explorer_collapses_and_comes_back_by_mouse() {
    use crate::workbench::{EXPLORER, EXPLORER_RAIL};
    let mut h = H::connected(120, 40);
    assert!(h.text().contains("Explorer"));
    assert!(
        h.app.shell.hits.area_of(EXPLORER_RAIL).is_none(),
        "no rail while open"
    );
    let collapse = h
        .app
        .shell
        .hits
        .area_of(EXPLORER_RAIL.part(junie_tui::core::id::PartRef::of(
            junie_tui::core::id::Part::ICON,
        )))
        .expect("the explorer has a collapse button");
    h.click(collapse.x + 1, collapse.y);
    assert!(!h.wb().explorer_visible, "{}", h.text());
    let rail = h
        .app
        .shell
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("a collapsed explorer leaves a rail");
    h.click(rail.x, rail.y + 3);
    assert!(h.wb().explorer_visible, "{}", h.text());
    assert_eq!(h.focus(), Some(EXPLORER));

    h.ctrl('b');
    assert!(!h.wb().explorer_visible);
    let rail = h
        .app
        .shell
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("rail after Ctrl+B");
    h.click(rail.x, rail.y);
    assert!(h.wb().explorer_visible);
}

#[test]
fn a_narrow_workbench_opens_the_explorer_drawer_from_its_rail() {
    use crate::workbench::{EXPLORER, EXPLORER_RAIL};
    let mut h = H::connected(90, 30);
    h.key(ratatui::crossterm::event::KeyCode::Tab);
    assert_ne!(h.focus(), Some(EXPLORER));
    let rail = h
        .app
        .shell
        .hits
        .area_of(EXPLORER_RAIL)
        .expect("the closed drawer leaves a rail");
    h.click(rail.x, rail.y + 1);
    assert_eq!(h.focus(), Some(EXPLORER), "{}", h.text());
    assert!(h.text().contains("Explorer"));
}

#[test]
fn the_app_menu_disconnects_and_quits_by_mouse() {
    use crate::app::STRIP_APP;
    let mut h = H::connected(120, 40);
    let open_menu = |h: &mut H| {
        let r = h
            .app
            .shell
            .hits
            .area_of(STRIP_APP)
            .expect("the TablePro segment");
        h.click(r.x, r.y);
    };
    open_menu(&mut h);
    assert!(
        h.text().contains("Disconnect") && h.text().contains("Quit TablePro"),
        "{}",
        h.text()
    );
    let (x, y) = h.find("Disconnect").unwrap();
    h.click(x, y);
    assert!(h.app.workbench.is_none(), "{}", h.text());
    assert!(h.text().contains("Connections"), "{}", h.text());

    open_menu(&mut h);
    assert!(!h.text().contains("Open tabs…"), "{}", h.text());
    let (x, y) = h.find("Quit TablePro").unwrap();
    h.click(x, y);
    assert!(h.app.shell.quit, "Quit did not quit");
}

#[test]
fn disconnect_keeps_an_unsaved_query() {
    use crate::app::STRIP_APP;
    let mut h = H::connected(120, 40);
    h.key(ratatui::crossterm::event::KeyCode::Tab);
    h.type_str("SELECT 1");
    h.key(ratatui::crossterm::event::KeyCode::Esc);
    let r = h.app.shell.hits.area_of(STRIP_APP).unwrap();
    h.click(r.x, r.y);
    let (x, y) = h.find("Disconnect").unwrap();
    h.click(x, y);
    assert!(h.app.workbench.is_some());
    assert!(h.text().contains("before disconnecting"), "{}", h.text());
}

#[test]
fn a_query_tab_close_plate_lights_whole_and_closes_the_tab() {
    use junie_tui::core::event::MouseKind;
    let mut h = H::connected(120, 40);
    let new = h.wb().strip.new_id();
    let plus = h.app.shell.hits.area_of(new).expect("the strip's +");
    h.click(plus.x + 1, plus.y);
    let n = h.wb().tabs.len();
    assert!(n >= 2, "{}", h.text());
    let i = h.wb().strip.active;
    let cid = h.wb().strip.close_id(i);
    let plate = h.app.shell.hits.area_of(cid).expect("the tab's ×");
    assert_eq!(plate.width, plus.width, "as wide as the +");
    let cell = |h: &H, x: u16| h.term.backend().buffer()[(x, plate.y)].clone();
    assert_eq!(cell(&h, plate.x + 1).symbol(), "×");
    let rest: Vec<_> = (plate.left()..plate.right())
        .map(|x| cell(&h, x).bg)
        .collect();
    h.mouse(MouseKind::Move, plate.right() - 1, plate.y);
    for (k, x) in (plate.left()..plate.right()).enumerate() {
        assert_ne!(cell(&h, x).bg, rest[k], "cell {k} of the plate lights");
        assert_eq!(cell(&h, x).bg, cell(&h, plate.x).bg, "one plate");
    }
    let tab = h.wb().strip.areas[i];
    assert_eq!(
        cell(&h, tab.x + 2).bg,
        rest[0],
        "the tab itself stays at rest"
    );
    h.click(plate.right() - 1, plate.y);
    assert_eq!(h.wb().tabs.len(), n - 1, "{}", h.text());
}
