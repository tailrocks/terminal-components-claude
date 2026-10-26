use ratatui_core::layout::{Margin, Rect};
use termrock::controls::pane_tree::{
    Direction, Frames, PaneExtent, PaneNode, PaneTree, PaneTreeError, SizingPolicy, SplitConfig,
    nearest_pane,
};
use termrock::layout::{Maximized, SplitAxis};

fn ratio_split(
    axis: SplitAxis,
    current: u8,
    starting: u8,
    maximized: Maximized,
    first: PaneNode,
    second: PaneNode,
) -> PaneNode {
    let config = SplitConfig::ratio(current, starting)
        .expect("fixture ratios are in the native domain")
        .maximized(maximized);
    PaneNode::split(axis, config, first, second)
}

#[test]
fn public_nested_model_exposes_order_geometry_frames_and_nearest() {
    let inner = ratio_split(
        SplitAxis::Horizontal,
        70,
        35,
        Maximized::None,
        PaneNode::leaf(101),
        PaneNode::leaf(202),
    );
    let root = ratio_split(
        SplitAxis::Vertical,
        60,
        40,
        Maximized::None,
        inner,
        PaneNode::leaf(303),
    );
    let tree = PaneTree::new(root.clone()).expect("unique caller IDs form a valid tree");
    assert_eq!(tree.leaves(), vec![101, 202, 303]);
    assert_eq!(tree.path_to(202), Some(vec![0, 1]));
    assert_eq!(tree.leaf_at(&[1]), Some(303));
    let split_snapshots = tree.split_snapshots();
    assert_eq!(
        split_snapshots
            .iter()
            .map(|split| (
                split.id.index(),
                split.axis,
                split.current_percent,
                split.basis_points,
                split.starting_percent,
                split.min_first,
                split.min_second,
                split.maximized,
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, SplitAxis::Vertical, 60, 6_000, 40, 1, 1, Maximized::None),
            (
                1,
                SplitAxis::Horizontal,
                70,
                7_000,
                35,
                1,
                1,
                Maximized::None
            ),
        ]
    );

    let area = Rect::new(13, 4, 53, 19);
    let layout = tree.layout(area);
    assert_eq!(
        layout
            .panes()
            .iter()
            .map(|pane| (pane.id, pane.outer))
            .collect::<Vec<_>>(),
        vec![
            (101, Rect::new(13, 4, 36, 10)),
            (202, Rect::new(50, 4, 16, 10)),
            (303, Rect::new(13, 15, 53, 8)),
        ]
    );
    for pane in layout.panes() {
        assert_eq!(pane.content, pane.outer.inner(Margin::new(1, 1)));
    }
    assert_eq!(layout.seams().len(), 2);
    assert_eq!(
        layout
            .seams()
            .iter()
            .map(|seam| (seam.axis, seam.container, seam.rect))
            .collect::<Vec<_>>(),
        vec![
            (
                SplitAxis::Vertical,
                Rect::new(13, 4, 53, 19),
                Rect::new(13, 14, 53, 1),
            ),
            (
                SplitAxis::Horizontal,
                Rect::new(13, 4, 53, 10),
                Rect::new(49, 4, 1, 10),
            ),
        ]
    );
    let split_ids = split_snapshots
        .into_iter()
        .map(|split| split.id)
        .collect::<Vec<_>>();
    assert_eq!(
        layout
            .seams()
            .iter()
            .map(|seam| seam.id)
            .collect::<Vec<_>>(),
        split_ids
    );
    assert_eq!(
        split_ids,
        tree.split_snapshots()
            .iter()
            .map(|split| split.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        nearest_pane(layout.panes(), 101, Direction::Right),
        Some(202)
    );
    assert_eq!(
        nearest_pane(layout.panes(), 101, Direction::Down),
        Some(303)
    );

    let frame_area = Rect::new(2, 3, 9, 7);
    let always = PaneTree::new(root.clone())
        .unwrap()
        .frames(Frames::Always)
        .layout(frame_area);
    assert_eq!(always.panes().len(), 3);
    for pane in always.panes() {
        assert_eq!(pane.content, pane.outer.inner(Margin::new(1, 1)));
    }
    let never = PaneTree::new(root)
        .unwrap()
        .frames(Frames::Never)
        .layout(frame_area);
    assert_eq!(never.panes().len(), 3);
    for pane in never.panes() {
        assert_eq!(pane.content, pane.outer);
    }
}

#[test]
fn public_sizing_keeps_three_split_policies_cross_axis_and_outer_zoom_separate() {
    let root = ratio_split(
        SplitAxis::Horizontal,
        60,
        30,
        Maximized::First,
        PaneNode::leaf(1),
        PaneNode::leaf(2),
    );
    let mut tree = PaneTree::new(root).unwrap().frames(Frames::Never);
    assert_eq!(tree.leaves(), vec![1, 2]);
    let content = |pane| match pane {
        1 => PaneExtent::new(20, 5),
        2 => PaneExtent::new(12, 14),
        _ => PaneExtent::default(),
    };
    let before = tree.split_snapshots();
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Current, content),
        PaneExtent::new(20, 14)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Configured, content),
        PaneExtent::new(20, 14)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Reset, content),
        PaneExtent::new(68, 14)
    );
    tree.set_zoomed(Some(2)).unwrap();
    assert_eq!(
        tree.tree_extent(SizingPolicy::Current, content),
        PaneExtent::new(12, 14)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Current, content),
        PaneExtent::new(20, 14)
    );
    assert_eq!(tree.split_snapshots(), before);

    let invalid_ratio = SplitConfig::ratio(96, 50).unwrap_err();
    assert_eq!(
        invalid_ratio,
        PaneTreeError::PercentOutOfRange { percent: 96 }
    );
    let original = PaneTree::new(PaneNode::leaf(7)).unwrap();
    let mut original = original;
    let duplicate = ratio_split(
        SplitAxis::Vertical,
        50,
        50,
        Maximized::None,
        PaneNode::leaf(8),
        PaneNode::leaf(8),
    );
    assert_eq!(
        original.replace_root(duplicate),
        Err(PaneTreeError::DuplicatePaneId { pane: 8 })
    );
    assert_eq!(original.leaves(), vec![7]);
}

#[test]
fn public_empty_layout_omits_zoomed_and_unzoomed_panes_without_state_changes() {
    let root = ratio_split(
        SplitAxis::Horizontal,
        60,
        40,
        Maximized::None,
        PaneNode::leaf(101),
        PaneNode::leaf(202),
    );
    let mut tree = PaneTree::new(root).unwrap().frames(Frames::Always);

    for area in [Rect::new(7, 9, 0, 20), Rect::new(7, 9, 20, 0)] {
        for zoom in [None, Some(202)] {
            tree.set_zoomed(zoom).unwrap();
            let before_leaves = tree.leaves();
            let before_splits = tree.split_snapshots();
            let before_zoom = tree.zoomed();

            let layout = tree.layout(area);
            assert!(layout.panes().is_empty(), "area={area:?}, zoom={zoom:?}");
            assert!(layout.seams().is_empty(), "area={area:?}, zoom={zoom:?}");
            assert_eq!(tree.leaves(), before_leaves);
            assert_eq!(tree.split_snapshots(), before_splits);
            assert_eq!(tree.zoomed(), before_zoom);
        }
    }
}

#[test]
fn public_nested_sizing_preserves_maximized_descendant_policy_and_cross_axis() {
    let hidden_child = ratio_split(
        SplitAxis::Vertical,
        70,
        35,
        Maximized::First,
        PaneNode::leaf(1),
        PaneNode::leaf(2),
    );
    let root = ratio_split(
        SplitAxis::Horizontal,
        60,
        30,
        Maximized::None,
        hidden_child,
        PaneNode::leaf(3),
    );
    let mut tree = PaneTree::new(root).unwrap().frames(Frames::Never);
    let before = tree.split_snapshots();
    assert_eq!(
        before
            .iter()
            .map(|split| (
                split.axis,
                split.current_percent,
                split.starting_percent,
                split.maximized
            ))
            .collect::<Vec<_>>(),
        vec![
            (SplitAxis::Horizontal, 60, 30, Maximized::None),
            (SplitAxis::Vertical, 70, 35, Maximized::First),
        ]
    );
    let content = |pane| match pane {
        1 => PaneExtent::new(8, 5),
        2 => PaneExtent::new(20, 14),
        3 => PaneExtent::new(13, 8),
        _ => PaneExtent::default(),
    };

    assert_eq!(
        tree.subtree_extent(SizingPolicy::Current, content),
        PaneExtent::new(35, 8)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Configured, content),
        PaneExtent::new(68, 8)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Reset, content),
        PaneExtent::new(68, 22)
    );
    assert_eq!(tree.split_snapshots(), before);

    tree.set_zoomed(Some(2)).unwrap();
    assert_eq!(
        tree.tree_extent(SizingPolicy::Reset, content),
        PaneExtent::new(20, 14)
    );
    assert_eq!(
        tree.subtree_extent(SizingPolicy::Current, content),
        PaneExtent::new(35, 8)
    );
    assert_eq!(tree.split_snapshots(), before);
}
