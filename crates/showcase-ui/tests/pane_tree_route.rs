use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{
    ColorLevel, FrameRead, Id, ItemKey, KeyCode, KeyModifiers, MouseKind, Part, PartRef, Rect,
    Theme,
};
use termrock_test_support::{Harness, Scene};

const NAV: Id = Id::root("showcase_ui::app::navigation");
const NAV_ENTRY_KEYSPACE: u64 = 0x4e41_5645_4e54_5259;
const PANE_TREE: Id = Id::root("showcase_ui::pane_tree");

fn leaf(key: u64) -> Id {
    PANE_TREE.sub("panes").item(ItemKey::num(key))
}

fn content(key: u64) -> Id {
    leaf(key).sub("content")
}

fn split(key: u64) -> Id {
    PANE_TREE.sub("splits").item(ItemKey::num(key))
}

fn button(index: usize) -> Id {
    PANE_TREE.sub("buttons").index(index)
}

fn footer(h: &Harness<App>) -> String {
    h.row(h.buffer().area().height.saturating_sub(1))
}

fn row_ref(index: usize) -> PartRef {
    PartRef::item(Part::ROW, ItemKey::pair(NAV_ENTRY_KEYSPACE, index as u64))
}

fn filter_navigation(h: &mut Harness<App>, query: &str) {
    let _ = h.key(KeyCode::Char('/'));
    let _ = h.type_str(query);
    assert!(
        h.text().contains(query),
        "the real navigation query is visible for {query:?}\n{}",
        h.text()
    );
}

fn route_to_tree_by_keyboard(h: &mut Harness<App>) {
    filter_navigation(h, "Pane tree");
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.area_of(leaf(1)).is_some(),
        "slot 38 mounted its real page"
    );
}

fn route_to_tree_by_click(h: &mut Harness<App>) {
    filter_navigation(h, "Pane tree");
    let index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Pane tree")
        .expect("Pane tree remains in the catalog");
    assert!(h.area_of_part(NAV, row_ref(index)).is_some());
    let _ = h.click_part(NAV, row_ref(index));
    assert!(
        h.area_of(leaf(1)).is_some(),
        "slot 38 mounted its real page"
    );
}

fn tree_harness(width: u16, height: u16) -> Harness<App> {
    Harness::new(
        App::for_page_name("Pane tree"),
        Theme::junie(),
        width,
        height,
    )
}

fn click_pane(h: &mut Harness<App>, key: u64) {
    assert!(h.area_of(leaf(key)).is_some(), "pane {key} is live");
    let _ = h.click_id(leaf(key));
    assert_eq!(h.focus(), Some(leaf(key)), "pane {key} received real focus");
}

fn click_button(h: &mut Harness<App>, index: usize) {
    assert!(
        h.area_of(button(index)).is_some(),
        "toolbar button {index} is live"
    );
    let _ = h.click_id(button(index));
}

fn seam(h: &Harness<App>, key: u64) -> Rect {
    h.area_of_part(split(key), PartRef::of(Part::SEAM))
        .unwrap_or_else(|| panic!("split {key} publishes its real seam"))
}

fn frame(h: &Harness<App>, key: u64) -> Rect {
    h.area_of(leaf(key))
        .unwrap_or_else(|| panic!("pane {key} publishes its frame area"))
}

fn body(h: &Harness<App>, key: u64) -> Option<Rect> {
    let snapshot = h
        .runtime()
        .render_snapshot()
        .expect("the settled Harness publishes a render snapshot");
    let area = *h.buffer().area();
    let mut scene = Scene::new(
        "pane-tree-route-body",
        Theme::junie(),
        ColorLevel::TrueColor,
        area.width,
        area.height,
    );
    scene.set_snapshot(snapshot);

    let mut reported = None;
    scene.draw(|ui, _| {
        reported = ui.layout(content(key)).and_then(|facts| facts.logical_area);
    });
    reported
}

fn center(area: Rect) -> (u16, u16) {
    (
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    )
}

fn alt_shift(h: &mut Harness<App>, key: KeyCode) {
    let _ = h.key_mod(key, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
}

fn assert_pane_group(h: &Harness<App>, keys: &[u64]) {
    for &key in keys {
        let pane = frame(h, key);
        let content =
            body(h, key).unwrap_or_else(|| panic!("pane {key} publishes its body layout facts"));
        assert!(!pane.is_empty(), "pane {key} is not clipped away: {pane:?}");
        assert!(
            !content.is_empty(),
            "pane {key} has visible body: {content:?}"
        );
        assert!(content.x >= pane.x && content.y >= pane.y);
        assert!(content.right() <= pane.right() && content.bottom() <= pane.bottom());
    }
}

fn assert_no_pane_group(h: &Harness<App>, keys: &[u64]) {
    for &key in keys {
        assert!(h.area_of(leaf(key)).is_none(), "pane {key} is not mounted");
        assert!(
            body(h, key).is_none(),
            "pane {key} has no published body geometry while hidden"
        );
    }
}

fn assert_non_overlapping(first: Rect, second: Rect, label: &str) {
    assert!(
        first.right() <= second.x
            || second.right() <= first.x
            || first.bottom() <= second.y
            || second.bottom() <= first.y,
        "{label} rectangles do not overlap: {first:?} and {second:?}"
    );
}

fn assert_default_topology(h: &Harness<App>) {
    let shell = frame(h, 1);
    let build = frame(h, 2);
    let logs = frame(h, 3);
    assert!(
        shell.x < build.x && shell.right() <= build.x,
        "shell is left of build: {shell:?}, {build:?}"
    );
    assert!(
        build.y < logs.y && build.bottom() <= logs.y,
        "build is above logs: {build:?}, {logs:?}"
    );
    assert_non_overlapping(shell, build, "shell/build");
    assert_non_overlapping(shell, logs, "shell/logs");
    assert_non_overlapping(build, logs, "build/logs");
}

fn contains_point(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.right() && y >= area.y && y < area.bottom()
}

fn assert_fresh_padded_vertical_drag(h: &mut Harness<App>, split_key: u64) {
    let before = seam(h, split_key);
    let container = h
        .area_of(split(split_key))
        .expect("split container is live");
    assert_eq!(
        before.height, 3,
        "the registered hit rectangle contains one seam row and one padding row on each side"
    );
    let x = before.x.saturating_add(before.width / 2);
    let seam_y = before.y.saturating_add(before.height / 2);
    let down_y = seam_y.saturating_add(1);
    assert!(
        contains_point(before, x, down_y),
        "the padded Down cell is inside the registered Part::SEAM hit rectangle"
    );
    assert!(
        x >= container.x && x < container.right() && down_y < container.bottom(),
        "the padded press is also inside the live split"
    );

    let _ = h.mouse(MouseKind::Down, x, down_y);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(split(split_key)),
        "a fresh press one cell into the second pane captures the real seam"
    );
    let drag_y = down_y.saturating_add(3);
    let _ = h.mouse(MouseKind::Drag, x, drag_y);
    assert_eq!(
        seam(h, split_key).y,
        before.y.saturating_add(3),
        "the new padded press establishes its own click offset"
    );
    let _ = h.mouse(MouseKind::Up, x, drag_y);
    assert_eq!(h.runtime().capture_owner(), None);
}

#[test]
fn slot_38_routes_through_the_real_catalog_and_preserves_default_and_neighbors() {
    let index = NAV_ENTRIES
        .iter()
        .position(|entry| entry.label == "Pane tree")
        .expect("Pane tree remains in the navigation catalog");
    assert_eq!(index, 38);
    assert_eq!(NAV_ENTRIES.len(), 54);
    assert_eq!(PageId::ALL.len(), 26);
    assert_eq!(NAV_ENTRIES[index].id, PageId::Overview);
    assert_eq!(PageId::from_name("Pane tree"), None);

    let mut h = Harness::new(App::new(), Theme::junie(), 120, 40);
    assert_eq!(h.app().page(), PageId::Overview);
    assert_no_pane_group(&h, &[1, 2, 3]);

    route_to_tree_by_click(&mut h);
    assert_eq!(h.app().page(), PageId::Overview);
    assert_pane_group(&h, &[1, 2, 3]);
    assert!(h.text().contains("shell"), "{}", h.text());
    assert!(h.text().contains("build"), "{}", h.text());
    assert!(h.text().contains("logs"), "{}", h.text());

    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Char(']'));
    assert!(
        format!("{:?}", h.app()).contains("nav_target: Live(Editor)"),
        "{:?}",
        h.app()
    );
    assert_no_pane_group(&h, &[1, 2, 3]);
    let _ = h.key(KeyCode::Char('['));
    assert_eq!(h.app().page(), PageId::Editor);
    assert!(
        format!("{:?}", h.app()).contains("nav_target: Placeholder(38)"),
        "{:?}",
        h.app()
    );
    assert_pane_group(&h, &[1, 2, 3]);
    assert!(h.text().contains("shell"), "{}", h.text());
    assert!(h.text().contains("build"), "{}", h.text());
    assert!(h.text().contains("logs"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let mut keyboard = Harness::new(App::new(), Theme::junie(), 120, 40);
    route_to_tree_by_keyboard(&mut keyboard);
    assert_eq!(keyboard.app().page(), PageId::Overview);
    assert_pane_group(&keyboard, &[1, 2, 3]);
    assert_default_topology(&keyboard);
    assert!(
        keyboard.diagnostics().is_empty(),
        "{:?}",
        keyboard.diagnostics()
    );
}

#[test]
fn frames_and_content_reflow_at_live_sizes_and_stop_at_the_shell_minimum() {
    let mut h = tree_harness(120, 40);
    assert_pane_group(&h, &[1, 2, 3]);
    let initial = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];
    assert_default_topology(&h);
    assert!(h.area_of(button(0)).is_some());
    assert!(h.text().contains("F6 cycle"));

    let _ = h.resize(80, 24);
    assert_pane_group(&h, &[1, 2, 3]);
    let compact = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];
    assert_default_topology(&h);
    assert_ne!(
        initial, compact,
        "the real page geometry follows host resize"
    );

    let _ = h.resize(72, 20);
    assert_pane_group(&h, &[1, 2, 3]);
    assert!(
        h.area_of(button(2)).is_some(),
        "controls remain inside shell minimum"
    );

    let _ = h.resize(71, 20);
    assert_no_pane_group(&h, &[1, 2, 3]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn split_close_and_reentry_keep_leaf_owners_stable_and_recover_sibling_space() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    let build_before = frame(&h, 2);
    click_button(&mut h, 2);

    assert!(
        h.area_of(leaf(3)).is_none(),
        "closing logs removes only logs"
    );
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.area_of(leaf(2)).is_some());
    assert!(
        frame(&h, 2).height > build_before.height,
        "build recovers its collapsed sibling's space"
    );
    assert_eq!(
        h.focus(),
        Some(leaf(1)),
        "nested close selects the whole-tree first leaf"
    );

    click_pane(&mut h, 2);
    click_button(&mut h, 0);
    assert_pane_group(&h, &[1, 2, 4]);
    assert_eq!(
        h.focus(),
        Some(leaf(4)),
        "Split right focuses the fresh editor leaf"
    );
    assert!(footer(&h).contains("Opened editor"), "{}", footer(&h));

    click_pane(&mut h, 4);
    click_button(&mut h, 1);
    assert_pane_group(&h, &[1, 2, 4, 5]);
    assert_eq!(
        h.focus(),
        Some(leaf(5)),
        "Split below focuses the fresh monitor leaf"
    );
    assert!(footer(&h).contains("Opened monitor"), "{}", footer(&h));

    click_button(&mut h, 2);
    assert!(h.area_of(leaf(5)).is_none());
    assert!(h.area_of(leaf(4)).is_some());
    assert!(h.area_of(leaf(1)).is_some());
    assert_eq!(h.focus(), Some(leaf(1)));

    click_pane(&mut h, 2);
    click_button(&mut h, 1);
    assert!(
        h.area_of(leaf(6)).is_some(),
        "closed pane keys are never reused"
    );
    assert!(h.area_of(leaf(5)).is_none());
    assert!(footer(&h).contains("Opened notes"), "{}", footer(&h));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn cyclic_and_directional_focus_keep_exact_refusal_feedback() {
    let mut h = tree_harness(120, 41);
    click_pane(&mut h, 1);

    let _ = h.key(KeyCode::F(6));
    assert_eq!(h.focus(), Some(leaf(2)));
    let _ = h.key(KeyCode::F(6));
    assert_eq!(h.focus(), Some(leaf(3)));
    let _ = h.key(KeyCode::F(6));
    assert_eq!(h.focus(), Some(leaf(1)));
    let _ = h.key_mod(KeyCode::F(6), KeyModifiers::SHIFT);
    assert_eq!(h.focus(), Some(leaf(3)));
    let _ = h.key_mod(KeyCode::F(6), KeyModifiers::SHIFT);
    assert_eq!(h.focus(), Some(leaf(2)));

    let _ = h.key_mod(KeyCode::Left, KeyModifiers::ALT);
    assert_eq!(h.focus(), Some(leaf(1)));
    let _ = h.key_mod(KeyCode::Left, KeyModifiers::ALT);
    assert_eq!(h.focus(), Some(leaf(1)));
    assert!(footer(&h).contains("No pane that way"), "{}", footer(&h));

    let _ = h.key_mod(KeyCode::Down, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
    assert!(
        footer(&h).contains("Nothing to resize in that direction"),
        "{}",
        footer(&h)
    );

    let source = frame(&h, 1);
    let pane_2 = frame(&h, 2);
    let pane_3 = frame(&h, 3);
    let center = |area: Rect| {
        (
            u32::from(area.x) + u32::from(area.width / 2),
            u32::from(area.y) + u32::from(area.height / 2),
        )
    };
    let manhattan = |from: Rect, to: Rect| {
        let (from_x, from_y) = center(from);
        let (to_x, to_y) = center(to);
        from_x.abs_diff(to_x) + from_y.abs_diff(to_y)
    };
    assert_eq!(
        manhattan(source, pane_2),
        manhattan(source, pane_3),
        "the independent frame geometry makes both rightward choices equidistant"
    );

    let _ = h.key_mod(KeyCode::Right, KeyModifiers::ALT);
    assert_eq!(
        h.focus(),
        Some(leaf(2)),
        "equal-distance focus uses traversal tie order"
    );
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::ALT);
    assert_eq!(h.focus(), Some(leaf(3)));
    let _ = h.key_mod(KeyCode::Up, KeyModifiers::ALT);
    assert_eq!(h.focus(), Some(leaf(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn keyboard_resize_moves_the_deepest_matching_seam_by_exactly_two_cells() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 2);
    let root_before = seam(&h, 1);
    let vertical_before = seam(&h, 2);
    alt_shift(&mut h, KeyCode::Down);
    assert_eq!(seam(&h, 2).y, vertical_before.y + 2);
    assert_eq!(
        seam(&h, 1),
        root_before,
        "vertical resize leaves root seam unchanged"
    );

    click_button(&mut h, 0);
    assert!(h.area_of(leaf(4)).is_some());
    assert!(h.area_of(split(3)).is_some());
    let root_before = seam(&h, 1);
    let vertical_before = seam(&h, 2);
    let nested_before = seam(&h, 3);
    alt_shift(&mut h, KeyCode::Right);
    assert_eq!(seam(&h, 3).x, nested_before.x + 2);
    assert_eq!(
        seam(&h, 1),
        root_before,
        "nested resize leaves root seam unchanged"
    );
    assert_eq!(
        seam(&h, 2),
        vertical_before,
        "horizontal resize leaves vertical seam unchanged"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn two_cell_keyboard_resize_refuses_a_one_cell_partial_clamp_without_moving() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 2);
    click_button(&mut h, 0);

    let container = h
        .area_of(split(3))
        .expect("nested horizontal split is live");
    let initial_seam = seam(&h, 3);
    let first_before = frame(&h, 2);
    assert_eq!(
        first_before.x, container.x,
        "build starts at the split's leading edge"
    );
    let actual_first_edge = first_before.right();
    let target_first_width = 13_u16;
    let target_first_edge = first_before.x + target_first_width;
    let delta = i32::from(target_first_edge) - i32::from(actual_first_edge);
    let (press_x, press_y) = center(initial_seam);
    assert!(contains_point(initial_seam, press_x, press_y));
    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    let target_x = u16::try_from(i32::from(press_x) + delta)
        .expect("the target split edge remains inside terminal coordinates");
    let _ = h.mouse(MouseKind::Drag, target_x, press_y);
    let _ = h.mouse(MouseKind::Up, target_x, press_y);
    let first_one_cell_from_minimum = frame(&h, 2);
    assert_eq!(first_one_cell_from_minimum.width, target_first_width);
    assert_eq!(first_one_cell_from_minimum.right(), target_first_edge);
    let one_cell_from_minimum = seam(&h, 3);

    alt_shift(&mut h, KeyCode::Left);
    assert_eq!(
        frame(&h, 2),
        first_one_cell_from_minimum,
        "a partial clamp is rolled back without changing the build pane"
    );
    assert_eq!(
        seam(&h, 3),
        one_cell_from_minimum,
        "a partial clamp is rolled back without moving the registered seam"
    );
    assert!(
        footer(&h).contains("Cannot resize: pane at minimum size"),
        "{}",
        footer(&h)
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn tab_focus_selects_the_actual_leaf_for_keyboard_resize() {
    let mut h = tree_harness(120, 40);
    assert!(
        h.tab_to(leaf(3)),
        "real Tab traversal reaches the logs leaf"
    );
    assert_eq!(h.focus(), Some(leaf(3)));
    let root_before = seam(&h, 1);
    let vertical_before = seam(&h, 2);

    alt_shift(&mut h, KeyCode::Down);

    assert_eq!(
        seam(&h, 2).y,
        vertical_before.y + 2,
        "keyboard focus on logs must select its vertical ancestor for resize"
    );
    assert_eq!(seam(&h, 1), root_before);
    assert_eq!(h.focus(), Some(leaf(3)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn shift_tab_focus_selects_the_actual_leaf_for_close() {
    let mut h = tree_harness(120, 40);
    assert!(
        h.tab_to(leaf(3)),
        "real Tab traversal reaches the logs leaf"
    );
    let _ = h.key(KeyCode::BackTab);
    assert_eq!(
        h.focus(),
        Some(leaf(2)),
        "real Shift+Tab reaches the build leaf"
    );
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.area_of(leaf(2)).is_some());
    assert!(h.area_of(leaf(3)).is_some());

    click_button(&mut h, 2);

    assert!(
        h.area_of(leaf(2)).is_none(),
        "Close follows Shift+Tab focus"
    );
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.area_of(leaf(3)).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn zoom_hides_other_leaves_refuses_focus_and_resize_and_restores_live_geometry() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    let root_before = seam(&h, 1);
    let nested_before = seam(&h, 2);
    click_button(&mut h, 3);
    assert!(h.area_of(leaf(3)).is_some());
    assert!(
        h.area_of_part(split(1), PartRef::of(Part::SEAM)).is_none(),
        "zoom hides ancestor seams with the other leaves"
    );
    assert_no_pane_group(&h, &[1, 2]);
    assert!(h.text().contains("zoomed"), "{}", h.text());

    let _ = h.key_mod(KeyCode::Left, KeyModifiers::ALT);
    assert!(
        footer(&h).contains("Unzoom to move focus"),
        "{}",
        footer(&h)
    );
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
    assert!(footer(&h).contains("Unzoom to resize"), "{}", footer(&h));

    click_button(&mut h, 3);
    assert_pane_group(&h, &[1, 2, 3]);
    assert_eq!(seam(&h, 1), root_before);
    assert_eq!(seam(&h, 2), nested_before);

    click_button(&mut h, 3);
    click_button(&mut h, 0);
    assert!(
        h.area_of(leaf(4)).is_some(),
        "split while zoomed first restores the tree"
    );
    assert_eq!(seam(&h, 1), root_before);
    assert_eq!(seam(&h, 2), nested_before);

    click_pane(&mut h, 4);
    let root_before_close = seam(&h, 1);
    click_button(&mut h, 3);
    assert!(
        h.area_of_part(split(1), PartRef::of(Part::SEAM)).is_none(),
        "zoom hides the root seam before closing its target"
    );
    click_button(&mut h, 2);
    assert!(
        h.area_of(leaf(4)).is_none(),
        "closing a zoom target restores then closes it"
    );
    assert_eq!(seam(&h, 1), root_before_close);
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.area_of(leaf(2)).is_some());
    assert!(h.area_of(leaf(3)).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn closing_the_last_leaf_keeps_the_shared_one_pane_refusal_message() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    click_button(&mut h, 2);
    click_pane(&mut h, 2);
    click_button(&mut h, 2);
    assert!(h.area_of(leaf(1)).is_some());
    assert_no_pane_group(&h, &[2, 3]);

    click_button(&mut h, 2);
    assert!(
        footer(&h).contains("Nothing to zoom: one pane"),
        "{}",
        footer(&h)
    );
    click_button(&mut h, 3);
    assert!(
        footer(&h).contains("Nothing to zoom: one pane"),
        "{}",
        footer(&h)
    );
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inserting_a_split_keeps_a_held_drag_on_its_retained_ancestor_owner() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let root_before = seam(&h, 1);
    let (x, y) = center(root_before);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));

    assert!(
        h.tab_to(button(0)),
        "Split right remains keyboard reachable during capture"
    );
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.area_of(leaf(4)).is_some(),
        "the real action inserts editor leaf 4"
    );
    assert!(
        h.area_of(split(1)).is_some(),
        "the captured ancestor remains mounted"
    );
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));

    let _ = h.mouse(MouseKind::Drag, x.saturating_add(3), y);
    assert_eq!(
        seam(&h, 1).x,
        root_before.x + 3,
        "a retained owner preserves its active grab across child insertion"
    );
    let _ = h.mouse(MouseKind::Up, x.saturating_add(3), y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn removing_a_leaf_during_a_held_seam_drag_cancels_capture_and_stale_motion() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    let root_before = seam(&h, 1);
    let nested_before = seam(&h, 2);
    let (x, y) = center(nested_before);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));

    assert!(
        h.tab_to(button(2)),
        "Close remains keyboard reachable during capture"
    );
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.area_of(leaf(3)).is_none(),
        "the selected logs leaf was removed"
    );
    assert!(
        h.area_of(split(2)).is_none(),
        "its captured split owner was removed"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "runtime drops stale capture"
    );

    let _ = h.mouse(MouseKind::Drag, x.saturating_add(6), y.saturating_add(4));
    let _ = h.mouse(MouseKind::Up, x.saturating_add(6), y.saturating_add(4));
    assert_eq!(
        seam(&h, 1),
        root_before,
        "stale motion cannot resize a surviving split"
    );
    assert!(h.area_of(leaf(1)).is_some());
    assert!(h.area_of(leaf(2)).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn zoom_cancellation_clears_stale_grab_offset_and_allows_a_fresh_padded_press() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    let before = seam(&h, 2);
    let (x, y) = center(before);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));

    let _ = h.key_mod(KeyCode::Char('z'), KeyModifiers::ALT);
    assert_no_pane_group(&h, &[1, 2]);
    assert_eq!(h.runtime().capture_owner(), None);
    let _ = h.key_mod(KeyCode::Char('z'), KeyModifiers::ALT);
    assert_pane_group(&h, &[1, 2, 3]);
    assert_eq!(seam(&h, 2), before);

    let _ = h.mouse(MouseKind::Drag, x, y.saturating_add(5));
    assert_eq!(
        seam(&h, 2),
        before,
        "unpressed stale motion cannot move the restored seam"
    );
    let _ = h.mouse(MouseKind::Up, x, y.saturating_add(5));
    assert_fresh_padded_vertical_drag(&mut h, 2);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn navigation_cancellation_clears_stale_grab_offset_and_allows_a_fresh_padded_press() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 3);
    let before = seam(&h, 2);
    let (x, y) = center(before);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));

    let _ = h.key(KeyCode::Char(']'));
    assert_no_pane_group(&h, &[1, 2, 3]);
    assert_eq!(h.runtime().capture_owner(), None);
    let _ = h.key(KeyCode::Char('['));
    assert_pane_group(&h, &[1, 2, 3]);
    assert_eq!(
        seam(&h, 2),
        before,
        "the private page retains its live split layout"
    );

    let _ = h.mouse(MouseKind::Drag, x, y.saturating_add(5));
    assert_eq!(
        seam(&h, 2),
        before,
        "unpressed stale motion cannot move the restored seam"
    );
    let _ = h.mouse(MouseKind::Up, x, y.saturating_add(5));
    assert_fresh_padded_vertical_drag(&mut h, 2);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn help_modal_cancels_a_held_seam_and_stale_motion_before_and_after_close() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let initial_seam = seam(&h, 1);
    let (press_x, press_y) = center(initial_seam);
    let initial_edge = frame(&h, 1).right();
    let initial_grab_offset = i32::from(press_x) - i32::from(initial_edge);
    assert_eq!(
        initial_grab_offset, 0,
        "the held gesture starts at the registered seam center"
    );
    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));

    let _ = h.key(KeyCode::Char('?'));
    assert!(
        h.text().contains("Keyboard & mouse"),
        "the real Help command opened its dialog\n{}",
        h.text()
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "the modal makes the captured page seam ineligible"
    );

    let stale_x = press_x.saturating_add(6);
    let stale_y = press_y.saturating_add(4);
    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    let _ = h.mouse(MouseKind::Up, stale_x, stale_y);
    assert!(
        h.text().contains("Keyboard & mouse"),
        "stale pointer input leaves the actual Help dialog open"
    );

    let _ = h.key(KeyCode::Esc);
    assert!(
        !h.text().contains("Keyboard & mouse"),
        "Escape closes the Help dialog through the host"
    );
    assert_eq!(h.runtime().capture_owner(), None);
    assert_eq!(
        seam(&h, 1),
        initial_seam,
        "stale motion cannot resize the seam after the modal closes"
    );

    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    assert_eq!(seam(&h, 1), initial_seam);

    let restored = seam(&h, 1);
    let container = h.area_of(split(1)).expect("the same root split is live");
    assert_eq!(restored.width, 3);
    let fresh_x = restored.right().saturating_sub(1);
    let fresh_y = restored.y.saturating_add(restored.height / 4);
    assert!(contains_point(restored, fresh_x, fresh_y));
    assert!(
        !contains_point(seam(&h, 2), fresh_x, fresh_y),
        "the fresh parent press avoids the nested child seam"
    );
    assert!(
        fresh_x >= container.x
            && fresh_x < container.right()
            && fresh_y >= container.y
            && fresh_y < container.bottom(),
        "the fresh padded Down is contained by the registered root seam target"
    );
    assert_ne!(
        fresh_x,
        center(restored).0,
        "the fresh press uses a noncentral padded cell"
    );
    let fresh_edge = frame(&h, 1).right();
    let fresh_grab_offset = i32::from(fresh_x) - i32::from(fresh_edge);
    assert_ne!(
        fresh_grab_offset, initial_grab_offset,
        "the new press uses a distinct offset from the gesture Help cancelled"
    );
    assert_eq!(
        fresh_grab_offset, 1,
        "the padded press stores a one-cell offset"
    );

    let _ = h.mouse(MouseKind::Down, fresh_x, fresh_y);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(split(1)),
        "a fresh gesture captures the same root seam after Help closes"
    );
    let drag_x = fresh_x.saturating_add(3);
    let _ = h.mouse(MouseKind::Drag, drag_x, fresh_y);
    assert_eq!(
        seam(&h, 1).x,
        restored.x.saturating_add(3),
        "the fresh offset moves the same seam by exactly three cells"
    );
    let _ = h.mouse(MouseKind::Up, drag_x, fresh_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn shell_minimum_suppression_clears_a_held_drag_and_return_accepts_a_fresh_padded_press() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let initial_seam = seam(&h, 1);
    let (press_x, press_y) = center(initial_seam);
    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));

    let _ = h.resize(71, 20);
    assert_no_pane_group(&h, &[1, 2, 3]);
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "the 71x20 shell-minimum suppression makes every page seam ineligible"
    );
    let stale_x = press_x.saturating_add(6);
    let stale_y = press_y.saturating_add(4);
    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    let _ = h.mouse(MouseKind::Up, stale_x, stale_y);

    let _ = h.resize(120, 40);
    assert_pane_group(&h, &[1, 2, 3]);
    assert_eq!(seam(&h, 1), initial_seam);
    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    let _ = h.mouse(MouseKind::Up, stale_x, stale_y);
    assert_eq!(
        seam(&h, 1),
        initial_seam,
        "a stale Drag cannot revive the pre-suppression capture"
    );

    let restored = seam(&h, 1);
    let container = h.area_of(split(1)).expect("root split is live again");
    assert_eq!(
        restored.width, 3,
        "the root target exposes its one-cell seam with one-cell padding"
    );
    let fresh_x = restored.right().saturating_sub(1);
    let fresh_y = restored.y.saturating_add(restored.height / 4);
    assert!(contains_point(restored, fresh_x, fresh_y));
    assert!(
        !contains_point(seam(&h, 2), fresh_x, fresh_y),
        "the fresh parent press avoids the nested child seam"
    );
    assert!(
        fresh_x >= container.x
            && fresh_x < container.right()
            && fresh_y >= container.y
            && fresh_y < container.bottom(),
        "the fresh Down uses a padded cell inside the real split target"
    );

    let fresh_grab_offset = i32::from(fresh_x) - i32::from(frame(&h, 1).right());
    assert_eq!(
        fresh_grab_offset, 1,
        "the padded press stores a one-cell offset"
    );
    let _ = h.mouse(MouseKind::Down, fresh_x, fresh_y);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(split(1)),
        "a new padded press acquires a new capture after return"
    );
    let drag_x = fresh_x.saturating_add(3);
    let _ = h.mouse(MouseKind::Drag, drag_x, fresh_y);
    assert_eq!(
        seam(&h, 1).x,
        restored.x.saturating_add(3),
        "the fresh grab moves the root seam by exactly three cells"
    );
    let _ = h.mouse(MouseKind::Up, drag_x, fresh_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inline_inspector_reflow_keeps_the_captured_root_seam_drag_active() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let viewport = *h.buffer().area();
    let initial_frames = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];
    let initial_seam = seam(&h, 1);
    assert_eq!(initial_seam.width, 3);
    let press_x = initial_seam.right().saturating_sub(1);
    let press_y = initial_seam.y.saturating_add(initial_seam.height / 4);
    let initial_edge = frame(&h, 1).right();
    let grab_offset = i32::from(press_x) - i32::from(initial_edge);
    assert!(contains_point(initial_seam, press_x, press_y));
    assert_eq!(
        grab_offset, 1,
        "the padded parent press stores a one-cell offset"
    );
    assert!(
        !contains_point(seam(&h, 2), press_x, press_y),
        "the parent press avoids the nested child seam"
    );

    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    assert_eq!(
        h.runtime().capture_owner(),
        Some(split(1)),
        "the root seam owns this padded pointer position"
    );

    let _ = h.key(KeyCode::Char('i'));
    assert_eq!(
        *h.buffer().area(),
        viewport,
        "the inspector keeps the real terminal viewport"
    );
    assert_pane_group(&h, &[1, 2, 3]);
    let visible_after_reflow = seam(&h, 1);
    let reflowed_frames = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];
    assert_ne!(
        initial_frames, reflowed_frames,
        "the published split geometry follows the real inline inspector dock"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        Some(split(1)),
        "a still-visible target keeps its valid captured gesture"
    );

    let resized_edge = frame(&h, 1).right();
    let drag_x = u16::try_from(i32::from(resized_edge) + grab_offset + 3)
        .expect("the resized root seam remains inside terminal coordinates");
    let drag_y = visible_after_reflow
        .y
        .saturating_add(visible_after_reflow.height / 2);
    let _ = h.mouse(MouseKind::Drag, drag_x, drag_y);
    assert_eq!(
        seam(&h, 1).x,
        visible_after_reflow.x.saturating_add(3),
        "the retained owner continues the same grab from reflowed geometry"
    );
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));
    let _ = h.mouse(MouseKind::Up, drag_x, drag_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn terminal_resize_cancels_capture_and_fresh_padded_press_uses_a_new_offset() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let initial_seam = seam(&h, 1);
    let initial_x = initial_seam.right().saturating_sub(1);
    let initial_y = initial_seam.y.saturating_add(initial_seam.height / 4);
    let initial_grab_offset = i32::from(initial_x) - i32::from(frame(&h, 1).right());
    assert_eq!(initial_grab_offset, 1);
    assert!(contains_point(initial_seam, initial_x, initial_y));
    assert!(!contains_point(seam(&h, 2), initial_x, initial_y));

    let _ = h.mouse(MouseKind::Down, initial_x, initial_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));
    let initial_geometry = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];

    let _ = h.resize(100, 32);
    assert_pane_group(&h, &[1, 2, 3]);
    assert_ne!(
        initial_geometry,
        [frame(&h, 1), frame(&h, 2), frame(&h, 3)],
        "the real terminal resize changes visible split geometry"
    );
    assert_eq!(
        h.runtime().capture_owner(),
        None,
        "Input::Resize cancels the established native capture"
    );

    let stale_x = initial_x.saturating_add(6);
    let stale_y = initial_y.saturating_add(4);
    let after_resize = seam(&h, 1);
    let _ = h.mouse(MouseKind::Drag, stale_x, stale_y);
    let _ = h.mouse(MouseKind::Up, stale_x, stale_y);
    assert_eq!(
        seam(&h, 1),
        after_resize,
        "stale motion cannot move the seam after resize cancellation"
    );
    assert_eq!(h.runtime().capture_owner(), None);

    let restored = seam(&h, 1);
    let fresh_x = restored.x;
    let fresh_y = restored.y.saturating_add(restored.height / 4);
    assert!(contains_point(restored, fresh_x, fresh_y));
    assert!(!contains_point(seam(&h, 2), fresh_x, fresh_y));
    let fresh_grab_offset = i32::from(fresh_x) - i32::from(frame(&h, 1).right());
    assert_eq!(fresh_grab_offset, -1);
    assert_ne!(fresh_grab_offset, initial_grab_offset);

    let _ = h.mouse(MouseKind::Down, fresh_x, fresh_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(1)));
    let resized_edge = frame(&h, 1).right();
    let drag_x = u16::try_from(i32::from(resized_edge) + fresh_grab_offset + 3)
        .expect("the fresh root drag remains inside terminal coordinates");
    let _ = h.mouse(MouseKind::Drag, drag_x, fresh_y);
    assert_eq!(
        seam(&h, 1).x,
        restored.x.saturating_add(3),
        "the fresh -1 grab offset moves the seam by exactly three cells"
    );
    let _ = h.mouse(MouseKind::Up, drag_x, fresh_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn overlapping_t_junction_routes_the_exact_old_press_to_the_nested_child() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let root_before = seam(&h, 1);
    let child_before = seam(&h, 2);
    let frames_before = [frame(&h, 1), frame(&h, 2), frame(&h, 3)];
    let press_x = root_before.right().saturating_sub(1);
    let press_y = child_before.bottom().saturating_sub(1);
    assert_eq!((press_x, press_y), (73, 21));
    assert!(contains_point(root_before, press_x, press_y));
    assert!(contains_point(child_before, press_x, press_y));

    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));
    let drag_x = press_x.saturating_add(3);
    let _ = h.mouse(MouseKind::Drag, drag_x, press_y);
    assert_eq!(
        seam(&h, 1),
        root_before,
        "perpendicular motion leaves the root seam unchanged"
    );
    assert_eq!(
        seam(&h, 2),
        child_before,
        "perpendicular motion leaves the child seam unchanged"
    );
    assert_eq!([frame(&h, 1), frame(&h, 2), frame(&h, 3)], frames_before);
    let _ = h.mouse(MouseKind::Up, drag_x, press_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn child_physical_seam_t_junction_moves_only_the_child_split() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let root_before = seam(&h, 1);
    let child_before = seam(&h, 2);
    let first_leaf_before = frame(&h, 1);
    let press_x = child_before.x;
    let press_y = child_before.y.saturating_add(child_before.height / 3);
    assert_eq!(press_y, 20);
    assert!(contains_point(root_before, press_x, press_y));
    assert!(contains_point(child_before, press_x, press_y));

    let _ = h.mouse(MouseKind::Down, press_x, press_y);
    assert_eq!(h.runtime().capture_owner(), Some(split(2)));
    let drag_y = press_y.saturating_add(3);
    let _ = h.mouse(MouseKind::Drag, press_x, drag_y);
    assert_eq!(
        seam(&h, 1),
        root_before,
        "the nested resize leaves the root seam unchanged"
    );
    let child_after = seam(&h, 2);
    assert_eq!(child_after.y, child_before.y.saturating_add(3));
    assert_eq!(child_after.x, child_before.x);
    assert_eq!(child_after.width, child_before.width);
    assert_eq!(child_after.height, child_before.height);
    assert_eq!(frame(&h, 1), first_leaf_before);
    let _ = h.mouse(MouseKind::Up, press_x, drag_y);
    assert_eq!(h.runtime().capture_owner(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn native_double_click_resets_root_split_to_independently_saved_initial_geometry() {
    let mut h = tree_harness(120, 40);
    click_pane(&mut h, 1);
    let initial_geometry = [
        frame(&h, 1),
        frame(&h, 2),
        frame(&h, 3),
        seam(&h, 1),
        seam(&h, 2),
    ];

    alt_shift(&mut h, KeyCode::Right);
    let resized_geometry = [
        frame(&h, 1),
        frame(&h, 2),
        frame(&h, 3),
        seam(&h, 1),
        seam(&h, 2),
    ];
    assert_ne!(
        resized_geometry, initial_geometry,
        "real keyboard resize moves the root away from the captured starting geometry"
    );

    let reset_target = seam(&h, 1);
    let (x, y) = center(reset_target);
    let _ = h.double_click(x, y);

    assert_eq!(
        [
            frame(&h, 1),
            frame(&h, 2),
            frame(&h, 3),
            seam(&h, 1),
            seam(&h, 2),
        ],
        initial_geometry,
        "the real double-click reset returns every published split rectangle to its saved start"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
