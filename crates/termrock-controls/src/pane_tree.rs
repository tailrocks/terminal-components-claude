//! A caller-authored recursive pane model over native [`SplitPane`] geometry.
//!
//! This first public slice owns topology, unique pane IDs, model split IDs,
//! read-only geometry, and three separate subtree sizing policies. It does
//! not route input, draw pane bodies, register runtime targets, or manage
//! pointer capture. A later host adapter owns those runtime responsibilities.

use std::{
    collections::{BTreeMap, HashSet},
    fmt,
};

use ratatui_core::layout::{Margin, Rect};
use termrock_core::Id;
use termrock_layout::{Maximized, SplitAxis};

use crate::{SplitPane, SplitPaneState};

/// Caller-owned key for pane content.
pub type PaneId = u64;

/// A model split identity stable while its axis and adjacent child-pane
/// boundary remain identifiable during root replacement.
///
/// This model-only identity is not a registered runtime owner. A future host
/// adapter must combine it with the host tree's own `Id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SplitId(usize);

impl SplitId {
    /// The model-local allocation number.
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Frame policy for pane geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Frames {
    /// Frame when the tree has several panes or an active zoom.
    #[default]
    WhenSplit,
    /// Always inset pane content by one cell on every edge.
    Always,
    /// Use each pane's full outer rectangle as its content rectangle.
    Never,
}

/// Direction used by the pure nearest-pane geometry helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Choose a pane wholly to the left.
    Left,
    /// Choose a pane wholly to the right.
    Right,
    /// Choose a pane wholly above.
    Up,
    /// Choose a pane wholly below.
    Down,
}

/// State policy for a direct-node subtree extent query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizingPolicy {
    /// Use live ratio, precision, and maximization.
    Current,
    /// Use the authored starting ratio, clear precision, and retain max.
    Configured,
    /// Use the authored starting ratio, clear precision, and clear max.
    Reset,
}

/// A requested content extent in terminal cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaneExtent {
    /// Requested horizontal content cells.
    pub width: u16,
    /// Requested vertical content cells.
    pub height: u16,
}

impl PaneExtent {
    /// Construct a requested width and height in cells.
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// Error returned when authoring or replacing a pane tree fails validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneTreeError {
    /// A requested split ratio is outside the native 5–95 range.
    PercentOutOfRange { percent: u8 },
    /// One caller pane ID appears more than once in the tree.
    DuplicatePaneId { pane: PaneId },
    /// The selected zoom pane is not a leaf in the current tree.
    UnknownPaneId { pane: PaneId },
    /// The model-local split identity counter cannot allocate another ID.
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

/// Authored state for one ratio-based native split.
///
/// Initial-cell and fixed-share configurations are not supported by this
/// model slice. Ratios are checked before any native clamping constructor is
/// called.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitConfig {
    current_percent: u8,
    starting_percent: u8,
    min_first: u16,
    min_second: u16,
    maximized: Maximized,
}

impl SplitConfig {
    /// Make a ratio-based split with native default minima and no maximized
    /// side. Both percentages must be in the inclusive native range 5–95.
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

    /// Set the native along-axis minima.
    #[must_use]
    pub const fn minimums(mut self, first: u16, second: u16) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    /// Set the authored maximized side for initial model state.
    #[must_use]
    pub const fn maximized(mut self, side: Maximized) -> Self {
        self.maximized = side;
        self
    }
}

/// Caller-authored tree topology. A split's current and starting ratios remain
/// separate model inputs; split owner IDs are assigned by [`PaneTree::new`].
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
    /// Author a leaf using a caller-owned ID.
    pub const fn leaf(id: PaneId) -> Self {
        Self {
            kind: NodeSpec::Leaf(id),
        }
    }

    /// Author a split with explicit native-ratio and minimum policy.
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

/// Read-only snapshot of one split's native state and authored start ratio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitSnapshot {
    /// Stable model-local split identity.
    pub id: SplitId,
    /// Native split direction.
    pub axis: SplitAxis,
    /// Current native coarse percentage.
    pub current_percent: u8,
    /// Current native ratio in basis points.
    pub basis_points: u16,
    /// Authored starting percentage.
    pub starting_percent: u8,
    /// First-child along-axis minimum.
    pub min_first: u16,
    /// Second-child along-axis minimum.
    pub min_second: u16,
    /// Current maximized side.
    pub maximized: Maximized,
}

/// One visible pane's outer rectangle and caller content rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaneGeometry {
    /// Caller-owned pane identifier.
    pub id: PaneId,
    /// Visible pane area before frame inset.
    pub outer: Rect,
    /// Area available to caller-owned content.
    pub content: Rect,
}

/// One visible native split seam in the last pure layout result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeamGeometry {
    /// Stable model-local split identity.
    pub id: SplitId,
    /// Native split direction.
    pub axis: SplitAxis,
    /// Rectangle passed to the native splitter.
    pub container: Rect,
    /// Resolved seam rectangle.
    pub rect: Rect,
}

/// Read-only visible geometry in tree order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaneLayout {
    panes: Vec<PaneGeometry>,
    seams: Vec<SeamGeometry>,
}

impl PaneLayout {
    /// Visible pane rectangles in authored tree order.
    pub fn panes(&self) -> &[PaneGeometry] {
        &self.panes
    }

    /// Visible native split seams in authored tree order.
    pub fn seams(&self) -> &[SeamGeometry] {
        &self.seams
    }
}

/// An additive model over native binary `SplitPane` geometry.
///
/// This model does not own a runtime `Cx`, draw buffers, or pointer capture.
/// It cannot release a runtime owner when callers replace its topology.
#[derive(Debug, PartialEq, Eq)]
pub struct PaneTree {
    root: Node,
    next_split_id: usize,
    frames: Frames,
    gap: u16,
    zoomed: Option<PaneId>,
}

impl PaneTree {
    /// Validate and take a caller-authored tree. Duplicate leaf IDs and
    /// unsupported percentages fail before the tree exposes any geometry.
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

    /// Replace the complete authored topology atomically. A successful
    /// replacement retains IDs for splits with the same axis and adjacent
    /// leaf boundary between their ordered children; new split IDs never reuse
    /// retired values. An invalid candidate leaves the current tree unchanged.
    /// Zoom is retained only if its leaf remains.
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

    /// Select the pure frame policy. Default: [`Frames::WhenSplit`].
    #[must_use]
    pub const fn frames(mut self, frames: Frames) -> Self {
        self.frames = frames;
        self
    }

    /// Set the gap between split children. Default: one cell.
    #[must_use]
    pub const fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Set a model zoom leaf. This changes pure geometry only; it does not
    /// perform a user command or any runtime focus/capture transition.
    pub fn set_zoomed(&mut self, pane: Option<PaneId>) -> Result<(), PaneTreeError> {
        if let Some(pane) = pane
            && !contains_leaf(&self.root, pane)
        {
            return Err(PaneTreeError::UnknownPaneId { pane });
        }
        self.zoomed = pane;
        Ok(())
    }

    /// Current model zoom leaf, when set.
    pub const fn zoomed(&self) -> Option<PaneId> {
        self.zoomed
    }

    /// Whether the current pure frame policy insets pane content.
    fn is_framed(&self) -> bool {
        match self.frames {
            Frames::Always => true,
            Frames::Never => false,
            Frames::WhenSplit => leaf_count(&self.root) > 1 || self.zoomed.is_some(),
        }
    }

    /// Caller pane IDs in first-child/second-child traversal order.
    pub fn leaves(&self) -> Vec<PaneId> {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves
    }

    /// Return the first/second path to a caller leaf ID.
    pub fn path_to(&self, pane: PaneId) -> Option<Vec<u8>> {
        let mut path = Vec::new();
        find_path(&self.root, pane, &mut path).then_some(path)
    }

    /// Return the leaf at a first/second path, only if the path ends at a leaf.
    pub fn leaf_at(&self, path: &[u8]) -> Option<PaneId> {
        node_at_path(&self.root, path)
    }

    /// Read model split state in authored traversal order.
    pub fn split_snapshots(&self) -> Vec<SplitSnapshot> {
        let mut snapshots = Vec::new();
        collect_snapshots(&self.root, &mut snapshots);
        snapshots
    }

    /// Lay out visible panes and seams with the native `SplitPane` resolver.
    /// This returns pure rectangles and registers no runtime targets.
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

    /// Measure the complete root subtree, ignoring the outer zoom shortcut.
    /// Every leaf requirement includes its frame when this tree is framed.
    pub fn subtree_extent(
        &self,
        policy: SizingPolicy,
        content: impl Fn(PaneId) -> PaneExtent,
    ) -> PaneExtent {
        let framed = self.is_framed();
        measure_node(&self.root, policy, self.gap, framed, &content)
    }

    /// Measure the visible tree. An active outer zoom returns only that leaf's
    /// framed content extent; otherwise this delegates to [`Self::subtree_extent`].
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

/// Select the nearest visible pane using whole-side eligibility, Manhattan
/// distance between cell centers, and layout-order tie breaking.
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

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;
    use termrock_layout::{Maximized, SplitAxis};

    use super::{
        Direction, Frames, PaneExtent, PaneGeometry, PaneNode, PaneTree, PaneTreeError,
        SizingPolicy, SplitConfig, nearest_pane,
    };

    fn split(
        axis: SplitAxis,
        current: u8,
        starting: u8,
        maximized: Maximized,
        first: PaneNode,
        second: PaneNode,
    ) -> PaneNode {
        let config = SplitConfig::ratio(current, starting)
            .unwrap()
            .maximized(maximized);
        PaneNode::split(axis, config, first, second)
    }

    #[test]
    fn rejects_duplicate_ids_and_invalid_percentages_atomically() {
        let duplicate = split(
            SplitAxis::Horizontal,
            50,
            50,
            Maximized::None,
            PaneNode::leaf(8),
            PaneNode::leaf(8),
        );
        assert_eq!(
            PaneTree::new(duplicate),
            Err(PaneTreeError::DuplicatePaneId { pane: 8 })
        );
        assert_eq!(
            SplitConfig::ratio(4, 50),
            Err(PaneTreeError::PercentOutOfRange { percent: 4 })
        );
        assert_eq!(
            SplitConfig::ratio(50, 96),
            Err(PaneTreeError::PercentOutOfRange { percent: 96 })
        );
        assert!(SplitConfig::ratio(5, 95).is_ok());
        let edge_tree = PaneTree::new(PaneNode::split(
            SplitAxis::Horizontal,
            SplitConfig::ratio(5, 95).unwrap(),
            PaneNode::leaf(4),
            PaneNode::leaf(5),
        ))
        .unwrap();
        assert_eq!(
            edge_tree
                .split_snapshots()
                .iter()
                .map(|split| (split.current_percent, split.starting_percent))
                .collect::<Vec<_>>(),
            vec![(5, 95)]
        );

        let mut tree = PaneTree::new(PaneNode::leaf(1)).unwrap();
        let old_leaves = tree.leaves();
        let old_snapshots = tree.split_snapshots();
        assert_eq!(
            tree.replace_root(split(
                SplitAxis::Vertical,
                50,
                50,
                Maximized::None,
                PaneNode::leaf(2),
                PaneNode::leaf(2),
            )),
            Err(PaneTreeError::DuplicatePaneId { pane: 2 })
        );
        assert_eq!(tree.leaves(), old_leaves);
        assert_eq!(tree.split_snapshots(), old_snapshots);

        tree.replace_root(split(
            SplitAxis::Horizontal,
            55,
            45,
            Maximized::None,
            PaneNode::leaf(2),
            PaneNode::leaf(3),
        ))
        .unwrap();
        let first_replacement_id = tree.split_snapshots().first().map(|split| split.id.index());
        assert_eq!(first_replacement_id, Some(0));
        tree.replace_root(split(
            SplitAxis::Horizontal,
            50,
            45,
            Maximized::None,
            PaneNode::leaf(2),
            PaneNode::leaf(3),
        ))
        .unwrap();
        assert_eq!(
            tree.split_snapshots()
                .first()
                .map(|split| (split.id.index(), split.current_percent)),
            Some((0, 50))
        );
        tree.replace_root(split(
            SplitAxis::Horizontal,
            50,
            45,
            Maximized::None,
            PaneNode::leaf(2),
            PaneNode::leaf(4),
        ))
        .unwrap();
        assert_eq!(
            tree.split_snapshots().first().map(|split| split.id.index()),
            Some(1)
        );
        tree.replace_root(split(
            SplitAxis::Horizontal,
            50,
            45,
            Maximized::None,
            PaneNode::leaf(2),
            PaneNode::leaf(3),
        ))
        .unwrap();
        assert_eq!(
            tree.split_snapshots().first().map(|split| split.id.index()),
            Some(2)
        );
    }

    #[test]
    fn root_replacement_retains_a_still_identifiable_nested_split() {
        let retained = split(
            SplitAxis::Horizontal,
            50,
            45,
            Maximized::None,
            PaneNode::leaf(20),
            PaneNode::leaf(30),
        );
        let mut tree = PaneTree::new(retained.clone()).unwrap();
        let retained_id = tree.split_snapshots().first().map(|split| split.id);
        assert!(retained_id.is_some());

        tree.replace_root(split(
            SplitAxis::Vertical,
            50,
            50,
            Maximized::None,
            PaneNode::leaf(10),
            retained,
        ))
        .unwrap();
        let split_ids = tree
            .split_snapshots()
            .iter()
            .map(|split| split.id)
            .collect::<Vec<_>>();
        assert_eq!(split_ids.len(), 2);
        assert!(retained_id.is_some_and(|id| split_ids.contains(&id)));
    }

    #[test]
    fn layout_uses_native_geometry_and_framed_content_at_nonzero_origin() {
        let root = split(
            SplitAxis::Horizontal,
            50,
            50,
            Maximized::None,
            PaneNode::leaf(11),
            PaneNode::leaf(22),
        );
        let tree = PaneTree::new(root).unwrap();
        let layout = tree.layout(Rect::new(7, 11, 41, 15));
        assert_eq!(layout.panes().len(), 2);
        assert_eq!(layout.seams().len(), 1);
        assert_eq!(
            layout
                .panes()
                .iter()
                .map(|pane| (pane.id, pane.outer, pane.content))
                .collect::<Vec<_>>(),
            vec![
                (11, Rect::new(7, 11, 20, 15), Rect::new(8, 12, 18, 13)),
                (22, Rect::new(28, 11, 20, 15), Rect::new(29, 12, 18, 13)),
            ]
        );
        assert_eq!(
            layout
                .seams()
                .iter()
                .map(|seam| seam.rect)
                .collect::<Vec<_>>(),
            vec![Rect::new(27, 11, 1, 15)]
        );
        assert_eq!(tree.leaves(), vec![11, 22]);
        assert_eq!(tree.path_to(22), Some(vec![1]));
        assert_eq!(tree.leaf_at(&[0]), Some(11));
        assert_eq!(tree.leaf_at(&[0, 0]), None);
        assert_eq!(nearest_pane(layout.panes(), 11, Direction::Right), Some(22));
        assert_eq!(nearest_pane(layout.panes(), 22, Direction::Left), Some(11));
    }

    #[test]
    fn nearest_pane_keeps_first_layout_candidate_on_a_distance_tie() {
        let panes = [
            PaneGeometry {
                id: 1,
                outer: Rect::new(10, 10, 2, 2),
                content: Rect::new(10, 10, 2, 2),
            },
            PaneGeometry {
                id: 3,
                outer: Rect::new(12, 8, 2, 2),
                content: Rect::new(12, 8, 2, 2),
            },
            PaneGeometry {
                id: 2,
                outer: Rect::new(8, 8, 2, 2),
                content: Rect::new(8, 8, 2, 2),
            },
        ];
        assert_eq!(nearest_pane(&panes, 1, Direction::Up), Some(3));
    }

    #[test]
    fn every_frame_mode_and_zoom_visibility_has_exact_geometry() {
        let area = Rect::new(3, 4, 8, 6);
        let single = PaneTree::new(PaneNode::leaf(5)).unwrap();
        assert_eq!(
            single.layout(area).panes().first().map(|pane| pane.content),
            Some(area)
        );
        assert_eq!(
            PaneTree::new(PaneNode::leaf(5))
                .unwrap()
                .frames(Frames::Always)
                .layout(area)
                .panes()
                .first()
                .map(|pane| pane.content),
            Some(Rect::new(4, 5, 6, 4))
        );
        assert_eq!(
            PaneTree::new(PaneNode::leaf(5))
                .unwrap()
                .frames(Frames::Never)
                .layout(area)
                .panes()
                .first()
                .map(|pane| pane.content),
            Some(area)
        );

        let root = split(
            SplitAxis::Vertical,
            50,
            50,
            Maximized::None,
            PaneNode::leaf(5),
            PaneNode::leaf(9),
        );
        let mut tree = PaneTree::new(root).unwrap();
        tree.set_zoomed(Some(9)).unwrap();
        let layout = tree.layout(area);
        assert_eq!(layout.panes().len(), 1);
        assert_eq!(
            layout
                .panes()
                .iter()
                .map(|pane| (pane.id, pane.outer, pane.content))
                .collect::<Vec<_>>(),
            vec![(9, area, Rect::new(4, 5, 6, 4))]
        );
        assert!(layout.seams().is_empty());
        assert_eq!(
            tree.set_zoomed(Some(99)),
            Err(PaneTreeError::UnknownPaneId { pane: 99 })
        );
        assert_eq!(tree.zoomed(), Some(9));

        let mut single = PaneTree::new(PaneNode::leaf(6)).unwrap();
        single.set_zoomed(Some(6)).unwrap();
        assert_eq!(single.layout(area).panes().len(), 1);
        assert_eq!(
            single.layout(area).panes().first().map(|pane| pane.content),
            Some(Rect::new(4, 5, 6, 4))
        );
        single.replace_root(PaneNode::leaf(6)).unwrap();
        assert_eq!(single.zoomed(), Some(6));
    }

    #[test]
    fn three_sizing_policies_keep_direct_cross_axis_and_outer_zoom_distinct() {
        let root = split(
            SplitAxis::Horizontal,
            60,
            30,
            Maximized::First,
            PaneNode::leaf(1),
            PaneNode::leaf(2),
        );
        let tree = PaneTree::new(root).unwrap().frames(Frames::Never);
        assert_eq!(
            tree.split_snapshots()
                .iter()
                .map(|split| (
                    split.current_percent,
                    split.basis_points,
                    split.starting_percent,
                    split.maximized,
                ))
                .collect::<Vec<_>>(),
            vec![(60, 6_000, 30, Maximized::First)]
        );
        assert_eq!(tree.leaves(), vec![1, 2]);
        let before = tree.split_snapshots();
        let content = |pane| match pane {
            1 => PaneExtent::new(20, 5),
            2 => PaneExtent::new(12, 14),
            _ => PaneExtent::default(),
        };
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
        assert_eq!(tree.split_snapshots(), before);

        let unmaximized = PaneTree::new(split(
            SplitAxis::Horizontal,
            60,
            30,
            Maximized::None,
            PaneNode::leaf(1),
            PaneNode::leaf(2),
        ))
        .unwrap()
        .frames(Frames::Never);
        assert_eq!(
            unmaximized.subtree_extent(SizingPolicy::Current, content),
            PaneExtent::new(35, 14)
        );
        assert_eq!(
            unmaximized.subtree_extent(SizingPolicy::Configured, content),
            PaneExtent::new(68, 14)
        );

        let vertical = PaneTree::new(split(
            SplitAxis::Vertical,
            60,
            30,
            Maximized::Second,
            PaneNode::leaf(3),
            PaneNode::leaf(4),
        ))
        .unwrap()
        .frames(Frames::Never);
        assert_eq!(vertical.leaves(), vec![3, 4]);
        let vertical_content = |pane| match pane {
            3 => PaneExtent::new(18, 5),
            4 => PaneExtent::new(25, 12),
            _ => PaneExtent::default(),
        };
        assert_eq!(
            vertical.subtree_extent(SizingPolicy::Current, vertical_content),
            PaneExtent::new(25, 12)
        );
        assert_eq!(
            vertical.subtree_extent(SizingPolicy::Configured, vertical_content),
            PaneExtent::new(25, 12)
        );
        assert_eq!(
            vertical.subtree_extent(SizingPolicy::Reset, vertical_content),
            PaneExtent::new(25, 18)
        );

        let mut zoomed = tree;
        zoomed.set_zoomed(Some(2)).unwrap();
        assert_eq!(
            zoomed.subtree_extent(SizingPolicy::Current, content),
            PaneExtent::new(20, 14)
        );
        assert_eq!(
            zoomed.tree_extent(SizingPolicy::Current, content),
            PaneExtent::new(12, 14)
        );
    }

    #[test]
    fn too_small_geometry_uses_native_split_fallback_and_omits_empty_targets() {
        let root = PaneNode::split(
            SplitAxis::Horizontal,
            SplitConfig::ratio(50, 50).unwrap().minimums(4, 4),
            PaneNode::leaf(1),
            PaneNode::leaf(2),
        );
        let tree = PaneTree::new(root).unwrap().frames(Frames::Never);
        let layout = tree.layout(Rect::new(9, 6, 5, 7));
        assert_eq!(layout.panes().len(), 1);
        assert_eq!(
            layout
                .panes()
                .iter()
                .map(|pane| (pane.id, pane.outer))
                .collect::<Vec<_>>(),
            vec![(1, Rect::new(9, 6, 5, 7))]
        );
        assert!(layout.seams().is_empty());
    }

    #[test]
    fn extent_saturates_at_native_cell_limit_without_mutating_state() {
        let root = PaneNode::split(
            SplitAxis::Horizontal,
            SplitConfig::ratio(50, 50).unwrap().minimums(u16::MAX, 1),
            PaneNode::leaf(1),
            PaneNode::leaf(2),
        );
        let tree = PaneTree::new(root).unwrap().frames(Frames::Never);
        let before = tree.split_snapshots();
        assert_eq!(
            tree.subtree_extent(SizingPolicy::Current, |_| PaneExtent::new(
                u16::MAX,
                u16::MAX
            )),
            PaneExtent::new(u16::MAX, u16::MAX)
        );
        assert_eq!(tree.split_snapshots(), before);
        let framed = PaneTree::new(PaneNode::leaf(3))
            .unwrap()
            .frames(Frames::Always);
        assert_eq!(
            framed.tree_extent(SizingPolicy::Current, |_| PaneExtent::new(
                u16::MAX,
                u16::MAX
            )),
            PaneExtent::new(u16::MAX, u16::MAX)
        );
    }
}
