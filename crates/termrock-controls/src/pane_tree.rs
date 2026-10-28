use std::{
    collections::{BTreeMap, HashSet},
    fmt,
};

use ratatui_core::layout::{Margin, Rect};
use termrock_core::Id;
use termrock_layout::{Maximized, SplitAxis};

use crate::{SplitPane, SplitPaneState};

pub type PaneId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SplitId(usize);

impl SplitId {
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Frames {
    #[default]
    WhenSplit,

    Always,

    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,

    Right,

    Up,

    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizingPolicy {
    Current,

    Configured,

    Reset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaneExtent {
    pub width: u16,

    pub height: u16,
}

impl PaneExtent {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneTreeError {
    PercentOutOfRange { percent: u8 },

    DuplicatePaneId { pane: PaneId },

    UnknownPaneId { pane: PaneId },

    SplitIdExhausted,
}

impl fmt::Display for PaneTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PercentOutOfRange { percent } => {
                write!(f, "split percentage {percent} is outside 5–95")
            }
            Self::DuplicatePaneId { pane } => write!(f, "pane ID {pane} appears more than once"),
            Self::UnknownPaneId { pane } => write!(f, "pane ID {pane} is not in the tree"),
            Self::SplitIdExhausted => f.write_str("model split ID space is exhausted"),
        }
    }
}

impl std::error::Error for PaneTreeError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitConfig {
    current_percent: u8,
    starting_percent: u8,
    min_first: u16,
    min_second: u16,
    maximized: Maximized,
}

impl SplitConfig {
    pub fn ratio(current_percent: u8, starting_percent: u8) -> Result<Self, PaneTreeError> {
        validate_percent(current_percent)?;
        validate_percent(starting_percent)?;
        Ok(Self {
            current_percent,
            starting_percent,
            min_first: 1,
            min_second: 1,
            maximized: Maximized::None,
        })
    }

    #[must_use]
    pub const fn minimums(mut self, first: u16, second: u16) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    #[must_use]
    pub const fn maximized(mut self, side: Maximized) -> Self {
        self.maximized = side;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneNode {
    kind: NodeSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NodeSpec {
    Leaf(PaneId),
    Split {
        axis: SplitAxis,
        config: SplitConfig,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

impl PaneNode {
    pub const fn leaf(id: PaneId) -> Self {
        Self {
            kind: NodeSpec::Leaf(id),
        }
    }

    pub fn split(axis: SplitAxis, config: SplitConfig, first: Self, second: Self) -> Self {
        Self {
            kind: NodeSpec::Split {
                axis,
                config,
                first: Box::new(first),
                second: Box::new(second),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Leaf(PaneId),
    Split {
        id: SplitId,
        axis: SplitAxis,
        config: SplitConfig,
        state: SplitPaneState,
        first: Box<Node>,
        second: Box<Node>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitSnapshot {
    pub id: SplitId,

    pub axis: SplitAxis,

    pub current_percent: u8,

    pub basis_points: u16,

    pub starting_percent: u8,

    pub min_first: u16,

    pub min_second: u16,

    pub maximized: Maximized,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaneGeometry {
    pub id: PaneId,

    pub outer: Rect,

    pub content: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeamGeometry {
    pub id: SplitId,

    pub axis: SplitAxis,

    pub container: Rect,

    pub rect: Rect,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaneLayout {
    panes: Vec<PaneGeometry>,
    seams: Vec<SeamGeometry>,
}

impl PaneLayout {
    pub fn panes(&self) -> &[PaneGeometry] {
        &self.panes
    }

    pub fn seams(&self) -> &[SeamGeometry] {
        &self.seams
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PaneTree {
    root: Node,
    next_split_id: usize,
    frames: Frames,
    gap: u16,
    zoomed: Option<PaneId>,
}

impl PaneTree {
    pub fn new(root: PaneNode) -> Result<Self, PaneTreeError> {
        let mut seen = HashSet::new();
        let mut next_split_id = 0;
        let identities = BTreeMap::new();
        let root = build_node(&root, &mut seen, &mut next_split_id, &identities)?;
        Ok(Self {
            root,
            next_split_id,
            frames: Frames::WhenSplit,
            gap: 1,
            zoomed: None,
        })
    }

    pub fn replace_root(&mut self, root: PaneNode) -> Result<(), PaneTreeError> {
        let mut seen = HashSet::new();
        let mut next_split_id = self.next_split_id;
        let mut identities = BTreeMap::new();
        collect_split_identities(&self.root, &mut identities);
        let replacement = build_node(&root, &mut seen, &mut next_split_id, &identities)?;
        let zoomed = self
            .zoomed
            .filter(|pane| contains_leaf(&replacement, *pane));
        self.root = replacement;
        self.next_split_id = next_split_id;
        self.zoomed = zoomed;
        Ok(())
    }

    #[must_use]
    pub const fn frames(mut self, frames: Frames) -> Self {
        self.frames = frames;
        self
    }

    #[must_use]
    pub const fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn set_zoomed(&mut self, pane: Option<PaneId>) -> Result<(), PaneTreeError> {
        if let Some(pane) = pane
            && !contains_leaf(&self.root, pane)
        {
            return Err(PaneTreeError::UnknownPaneId { pane });
        }
        self.zoomed = pane;
        Ok(())
    }

    pub const fn zoomed(&self) -> Option<PaneId> {
        self.zoomed
    }

    fn is_framed(&self) -> bool {
        match self.frames {
            Frames::Always => true,
            Frames::Never => false,
            Frames::WhenSplit => leaf_count(&self.root) > 1 || self.zoomed.is_some(),
        }
    }

    pub fn leaves(&self) -> Vec<PaneId> {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves
    }

    pub fn path_to(&self, pane: PaneId) -> Option<Vec<u8>> {
        let mut path = Vec::new();
        find_path(&self.root, pane, &mut path).then_some(path)
    }

    pub fn leaf_at(&self, path: &[u8]) -> Option<PaneId> {
        node_at_path(&self.root, path)
    }

    pub fn split_snapshots(&self) -> Vec<SplitSnapshot> {
        let mut snapshots = Vec::new();
        collect_snapshots(&self.root, &mut snapshots);
        snapshots
    }

    pub fn layout(&self, area: Rect) -> PaneLayout {
        let framed = self.is_framed();
        let mut layout = PaneLayout::default();
        if let Some(pane) = self.zoomed {
            if contains_leaf(&self.root, pane) && !area.is_empty() {
                layout.panes.push(PaneGeometry {
                    id: pane,
                    outer: area,
                    content: content_rect(area, framed),
                });
            }
            return layout;
        }
        layout_node(&self.root, area, self.gap, framed, &mut layout);
        layout
    }

    pub fn subtree_extent(
        &self,
        policy: SizingPolicy,
        content: impl Fn(PaneId) -> PaneExtent,
    ) -> PaneExtent {
        let framed = self.is_framed();
        measure_node(&self.root, policy, self.gap, framed, &content)
    }

    pub fn tree_extent(
        &self,
        policy: SizingPolicy,
        content: impl Fn(PaneId) -> PaneExtent,
    ) -> PaneExtent {
        if let Some(pane) = self.zoomed {
            return add_frame(content(pane), self.is_framed());
        }
        self.subtree_extent(policy, content)
    }
}

pub fn nearest_pane(panes: &[PaneGeometry], from: PaneId, direction: Direction) -> Option<PaneId> {
    let from_rect = panes.iter().find(|pane| pane.id == from)?.outer;
    let fx = i32::from(from_rect.x) + i32::from(from_rect.width) / 2;
    let fy = i32::from(from_rect.y) + i32::from(from_rect.height) / 2;
    panes
        .iter()
        .filter(|pane| {
            let r = pane.outer;
            pane.id != from
                && match direction {
                    Direction::Left => r.right() as i32 <= i32::from(from_rect.x),
                    Direction::Right => r.x as i32 >= i32::from(from_rect.right()),
                    Direction::Up => r.bottom() as i32 <= i32::from(from_rect.y),
                    Direction::Down => r.y as i32 >= i32::from(from_rect.bottom()),
                }
        })
        .min_by_key(|pane| {
            let r = pane.outer;
            let cx = i32::from(r.x) + i32::from(r.width) / 2;
            let cy = i32::from(r.y) + i32::from(r.height) / 2;
            (cx - fx).abs() + (cy - fy).abs()
        })
        .map(|pane| pane.id)
}

fn validate_percent(percent: u8) -> Result<(), PaneTreeError> {
    if !(5..=95).contains(&percent) {
        return Err(PaneTreeError::PercentOutOfRange { percent });
    }
    Ok(())
}

fn build_node(
    spec: &PaneNode,
    seen: &mut HashSet<PaneId>,
    next_split_id: &mut usize,
    identities: &BTreeMap<SplitIdentityKey, SplitId>,
) -> Result<Node, PaneTreeError> {
    match &spec.kind {
        NodeSpec::Leaf(pane) => {
            if !seen.insert(*pane) {
                return Err(PaneTreeError::DuplicatePaneId { pane: *pane });
            }
            Ok(Node::Leaf(*pane))
        }
        NodeSpec::Split {
            axis,
            config,
            first,
            second,
        } => {
            validate_percent(config.current_percent)?;
            validate_percent(config.starting_percent)?;
            let key = split_identity_key(*axis, first, second);
            let id = if let Some(id) = identities.get(&key) {
                *id
            } else {
                let following = next_split_id
                    .checked_add(1)
                    .ok_or(PaneTreeError::SplitIdExhausted)?;
                let id = SplitId(*next_split_id);
                *next_split_id = following;
                id
            };
            let first = Box::new(build_node(first, seen, next_split_id, identities)?);
            let second = Box::new(build_node(second, seen, next_split_id, identities)?);
            let mut state = SplitPaneState::new(config.current_percent);
            if config.maximized != Maximized::None {
                state.toggle_max(config.maximized);
            }
            Ok(Node::Split {
                id,
                axis: *axis,
                config: *config,
                state,
                first,
                second,
            })
        }
    }
}

type SplitIdentityKey = (u8, PaneId, PaneId);

fn split_identity_key(axis: SplitAxis, first: &PaneNode, second: &PaneNode) -> SplitIdentityKey {
    (
        axis_key(axis),
        last_spec_leaf(first),
        first_spec_leaf(second),
    )
}

fn axis_key(axis: SplitAxis) -> u8 {
    match axis {
        SplitAxis::Horizontal => 0,
        SplitAxis::Vertical => 1,
    }
}

fn first_spec_leaf(spec: &PaneNode) -> PaneId {
    match &spec.kind {
        NodeSpec::Leaf(pane) => *pane,
        NodeSpec::Split { first, .. } => first_spec_leaf(first),
    }
}

fn last_spec_leaf(spec: &PaneNode) -> PaneId {
    match &spec.kind {
        NodeSpec::Leaf(pane) => *pane,
        NodeSpec::Split { second, .. } => last_spec_leaf(second),
    }
}

fn collect_split_identities(
    node: &Node,
    identities: &mut BTreeMap<SplitIdentityKey, SplitId>,
) -> (PaneId, PaneId) {
    match node {
        Node::Leaf(pane) => (*pane, *pane),
        Node::Split {
            id,
            axis,
            first,
            second,
            ..
        } => {
            let (first_leaf, last_first_leaf) = collect_split_identities(first, identities);
            let (first_second_leaf, last_leaf) = collect_split_identities(second, identities);
            identities.insert((axis_key(*axis), last_first_leaf, first_second_leaf), *id);
            (first_leaf, last_leaf)
        }
    }
}

fn collect_leaves(node: &Node, out: &mut Vec<PaneId>) {
    match node {
        Node::Leaf(pane) => out.push(*pane),
        Node::Split { first, second, .. } => {
            collect_leaves(first, out);
            collect_leaves(second, out);
        }
    }
}

fn contains_leaf(node: &Node, target: PaneId) -> bool {
    match node {
        Node::Leaf(pane) => *pane == target,
        Node::Split { first, second, .. } => {
            contains_leaf(first, target) || contains_leaf(second, target)
        }
    }
}

fn leaf_count(node: &Node) -> usize {
    match node {
        Node::Leaf(_) => 1,
        Node::Split { first, second, .. } => leaf_count(first).saturating_add(leaf_count(second)),
    }
}

fn find_path(node: &Node, target: PaneId, path: &mut Vec<u8>) -> bool {
    match node {
        Node::Leaf(pane) => *pane == target,
        Node::Split { first, second, .. } => {
            path.push(0);
            if find_path(first, target, path) {
                return true;
            }
            path.pop();
            path.push(1);
            if find_path(second, target, path) {
                return true;
            }
            path.pop();
            false
        }
    }
}

fn node_at_path(node: &Node, path: &[u8]) -> Option<PaneId> {
    let mut current = node;
    for step in path {
        current = match (current, step) {
            (Node::Split { first, .. }, 0) => first,
            (Node::Split { second, .. }, 1) => second,
            _ => return None,
        };
    }
    match current {
        Node::Leaf(pane) => Some(*pane),
        Node::Split { .. } => None,
    }
}

fn collect_snapshots(node: &Node, out: &mut Vec<SplitSnapshot>) {
    if let Node::Split {
        id,
        axis,
        config,
        state,
        first,
        second,
        ..
    } = node
    {
        out.push(SplitSnapshot {
            id: *id,
            axis: *axis,
            current_percent: state.percent(),
            basis_points: state.basis_points(),
            starting_percent: config.starting_percent,
            min_first: config.min_first,
            min_second: config.min_second,
            maximized: state.maximized(),
        });
        collect_snapshots(first, out);
        collect_snapshots(second, out);
    }
}

fn layout_node(node: &Node, area: Rect, gap: u16, framed: bool, out: &mut PaneLayout) {
    match node {
        Node::Leaf(pane) => {
            if !area.is_empty() {
                out.panes.push(PaneGeometry {
                    id: *pane,
                    outer: area,
                    content: content_rect(area, framed),
                });
            }
        }
        Node::Split {
            id,
            axis,
            config,
            state,
            first,
            second,
        } => {
            let split = split_control(*id, *axis, *config, gap);
            let (first_area, second_area) = split.panes(*state, area);
            let seam = split.seam(*state, area);
            if !seam.is_empty() {
                out.seams.push(SeamGeometry {
                    id: *id,
                    axis: *axis,
                    container: area,
                    rect: seam,
                });
            }
            if !first_area.is_empty() {
                layout_node(first, first_area, gap, framed, out);
            }
            if !second_area.is_empty() {
                layout_node(second, second_area, gap, framed, out);
            }
        }
    }
}

fn content_rect(area: Rect, framed: bool) -> Rect {
    if framed {
        area.inner(Margin::new(1, 1))
    } else {
        area
    }
}

fn add_frame(extent: PaneExtent, framed: bool) -> PaneExtent {
    let inset = if framed { 2 } else { 0 };
    PaneExtent {
        width: extent.width.saturating_add(inset),
        height: extent.height.saturating_add(inset),
    }
}

fn split_control(
    id: SplitId,
    axis: SplitAxis,
    config: SplitConfig,
    gap: u16,
) -> SplitPane<'static> {
    SplitPane::new(Id::root("termrock.pane_tree.model").index(id.0), axis)
        .gap(gap)
        .min_first(config.min_first)
        .min_second(config.min_second)
}

fn split_extent(
    id: SplitId,
    axis: SplitAxis,
    config: SplitConfig,
    state: SplitPaneState,
    gap: u16,
    first: PaneExtent,
    second: PaneExtent,
) -> PaneExtent {
    let split = split_control(id, axis, config, gap);
    match axis {
        SplitAxis::Horizontal => PaneExtent {
            width: split.required_extent(&state, first.width, second.width),
            height: first.height.max(second.height),
        },
        SplitAxis::Vertical => PaneExtent {
            width: first.width.max(second.width),
            height: split.required_extent(&state, first.height, second.height),
        },
    }
}

fn state_for_policy(
    mut state: SplitPaneState,
    config: SplitConfig,
    policy: SizingPolicy,
) -> SplitPaneState {
    match policy {
        SizingPolicy::Current => state,
        SizingPolicy::Configured => {
            state.set_percent(config.starting_percent);
            state
        }
        SizingPolicy::Reset => {
            state.set_percent(config.starting_percent);
            let maximized = state.maximized();
            if maximized != Maximized::None {
                state.toggle_max(maximized);
            }
            state
        }
    }
}

fn measure_node(
    node: &Node,
    policy: SizingPolicy,
    gap: u16,
    framed: bool,
    content: &impl Fn(PaneId) -> PaneExtent,
) -> PaneExtent {
    match node {
        Node::Leaf(pane) => add_frame(content(*pane), framed),
        Node::Split {
            id,
            axis,
            config,
            state,
            first,
            second,
        } => {
            let first = measure_node(first, policy, gap, framed, content);
            let second = measure_node(second, policy, gap, framed, content);
            split_extent(
                *id,
                *axis,
                *config,
                state_for_policy(*state, *config, policy),
                gap,
                first,
                second,
            )
        }
    }
}
