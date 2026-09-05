//! `Tree` (`COMPONENT_ARCHITECTURE.md` §12.4, §18.2, §20.9-8, Appendix A 4C).
//!
//! The legacy `TreeView` owned a `Vec<FlatRow>` of cloned labels and rebuilt
//! it on every expand, collapse, filter change and lazy delivery — ≈300 000
//! allocations per toggle on a 100 k-node tree (§16.6
//! `tree_100k_nodes_flatten`). The runtime now owns a keyed `TreeIndex` of
//! source positions and structural metadata. It holds no borrowed item or
//! text, is built once per explicit source/query revision, and is shared by
//! update and draw. A branch toggle splices only its contiguous subtree;
//! unchanged phases read the visible window directly.

use core::fmt;
use core::marker::PhantomData;

use std::collections::HashMap;

use ratatui_core::layout::Rect;

use super::scroll_region::ScrollRegion;
use super::{Acc, PartStyle, SlotFn, cell_at};
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, EmptyState, KeyFn, KeySet, Reconcile, Reconciliation,
    RowFn, RowUi,
};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// Whether a node can be opened, and whether its children are already in the
/// slice.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NodeKind {
    /// No children; the disclosure cell stays blank.
    #[default]
    Leaf,
    /// Children follow in the slice at a greater depth.
    Parent,
    /// Children are fetched on first expand. Expanding emits
    /// [`TreeAction::Expanded`] and the caller appends the children to the
    /// slice it passes on the next frame.
    Lazy,
}

/// What the tree needs to know about one item: its depth, whether it opens,
/// and optionally its stable key.
///
/// The node carries **no text**: the label, the metadata and any kind glyph
/// are the row renderer's job, which is what deletes `FlatRow`'s duplicated
/// `label`/`meta` (§18.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TreeNode {
    depth: u16,
    kind: NodeKind,
    key: Option<ItemKey>,
}

impl TreeNode {
    /// A childless node at `depth`.
    #[must_use]
    pub const fn leaf(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Leaf,
            key: None,
        }
    }

    /// A node whose children follow in the slice at a greater depth.
    #[must_use]
    pub const fn parent(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Parent,
            key: None,
        }
    }

    /// A node whose children arrive after [`TreeAction::Expanded`].
    #[must_use]
    pub const fn lazy(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Lazy,
            key: None,
        }
    }

    /// Give the node a caller key, overriding the tree's `.key(…)` accessor
    /// for this item (§12.4).
    #[must_use]
    pub const fn keyed(mut self, k: ItemKey) -> Self {
        self.key = Some(k);
        self
    }

    /// The node's depth; the root level is `0`.
    #[must_use]
    pub const fn depth(self) -> u16 {
        self.depth
    }

    /// The node's kind.
    #[must_use]
    pub const fn kind(self) -> NodeKind {
        self.kind
    }

    /// The caller key set by [`TreeNode::keyed`], if any.
    #[must_use]
    pub const fn key(self) -> Option<ItemKey> {
        self.key
    }

    /// Whether the node has a disclosure affordance.
    #[must_use]
    pub const fn has_children(self) -> bool {
        matches!(self.kind, NodeKind::Parent | NodeKind::Lazy)
    }
}

/// What a tree reports; every action carries the node's key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeAction {
    /// The cursor moved.
    Moved,
    /// A node was opened. For a [`NodeKind::Lazy`] node this is the request
    /// to fetch its children.
    Expanded(ItemKey),
    /// A node was closed.
    Collapsed(ItemKey),
    /// A leaf was chosen (Space, or a click on a leaf row).
    Chose(ItemKey),
    /// A leaf was activated (Enter, or a double-click).
    Activated(ItemKey),
}

/// The const-constructible commands of the tree keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeCmd {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Cursor up one viewport.
    PageUp,
    /// Cursor down one viewport.
    PageDown,
    /// Cursor to the first row.
    Home,
    /// Cursor to the last row.
    End,
    /// Open the cursor node, or descend into it.
    Expand,
    /// Close the cursor node, or move to its parent.
    Collapse,
    /// Toggle a branch, activate a leaf.
    Activate,
    /// Choose the cursor leaf.
    Choose,
    /// Open every node.
    ExpandAll,
    /// Close every node.
    CollapseAll,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: TreeCmd,
    label: &'static str,
    visible: bool,
) -> Binding<TreeCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const TABLE: [Binding<TreeCmd>; 16] = [
    b("tree.up", Chord::key(KeyCode::Up), TreeCmd::Up, "Up", true),
    b(
        "tree.down",
        Chord::key(KeyCode::Down),
        TreeCmd::Down,
        "Down",
        true,
    ),
    b(
        "tree.up-vim",
        Chord::key(KeyCode::Char('k')),
        TreeCmd::Up,
        "Up",
        false,
    ),
    b(
        "tree.down-vim",
        Chord::key(KeyCode::Char('j')),
        TreeCmd::Down,
        "Down",
        false,
    ),
    b(
        "tree.page-up",
        Chord::key(KeyCode::PageUp),
        TreeCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "tree.page-down",
        Chord::key(KeyCode::PageDown),
        TreeCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "tree.home",
        Chord::key(KeyCode::Home),
        TreeCmd::Home,
        "First",
        false,
    ),
    b(
        "tree.end",
        Chord::key(KeyCode::End),
        TreeCmd::End,
        "Last",
        false,
    ),
    b(
        "tree.home-vim",
        Chord::key(KeyCode::Char('g')),
        TreeCmd::Home,
        "First",
        false,
    ),
    b(
        "tree.end-vim",
        Chord::key(KeyCode::Char('G')),
        TreeCmd::End,
        "Last",
        false,
    ),
    b(
        "tree.expand",
        Chord::key(KeyCode::Right),
        TreeCmd::Expand,
        "Open",
        true,
    ),
    b(
        "tree.expand-vim",
        Chord::key(KeyCode::Char('l')),
        TreeCmd::Expand,
        "Open",
        false,
    ),
    b(
        "tree.collapse",
        Chord::key(KeyCode::Left),
        TreeCmd::Collapse,
        "Close",
        true,
    ),
    b(
        "tree.collapse-vim",
        Chord::key(KeyCode::Char('h')),
        TreeCmd::Collapse,
        "Close",
        false,
    ),
    b(
        "tree.activate",
        Chord::key(KeyCode::Enter),
        TreeCmd::Activate,
        "Activate",
        true,
    ),
    b(
        "tree.choose",
        Chord::key(KeyCode::Char(' ')),
        TreeCmd::Choose,
        "Choose",
        false,
    ),
];

/// The same table plus the two whole-tree chords, used when the tree has at
/// least one openable node.
const TABLE_FOLDABLE: [Binding<TreeCmd>; 18] = [
    TABLE[0],
    TABLE[1],
    TABLE[2],
    TABLE[3],
    TABLE[4],
    TABLE[5],
    TABLE[6],
    TABLE[7],
    TABLE[8],
    TABLE[9],
    TABLE[10],
    TABLE[11],
    TABLE[12],
    TABLE[13],
    TABLE[14],
    TABLE[15],
    b(
        "tree.expand-all",
        Chord::key(KeyCode::Char('*')),
        TreeCmd::ExpandAll,
        "Expand all",
        false,
    ),
    b(
        "tree.collapse-all",
        Chord::key(KeyCode::Char('-')),
        TreeCmd::CollapseAll,
        "Collapse all",
        false,
    ),
];

/// Durable state of a [`Tree`]: the cursor key, the expanded set, the chosen
/// leaf, the scroll offset, reconcile stamp and monotonic structural
/// generations consumed by the runtime-owned index.
///
/// Everything here is a **key**, never a path and never an index, which is
/// what deletes the legacy `expanded: HashSet<Vec<usize>>` and the
/// `object_at`/`schema_at` path reconstruction (§18.2).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TreeState {
    core: CollectionCore,
    expanded: KeySet,
    chosen: Option<ItemKey>,
    expand_generation: u64,
    source_generation: u64,
    expand_generation_saturated: bool,
    source_generation_saturated: bool,
    last_expansion: Option<ExpansionChange>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ExpansionChange {
    generation: u64,
    key: ItemKey,
    expanded: bool,
}

impl TreeState {
    /// A collapsed tree with no cursor.
    #[must_use]
    pub const fn new() -> Self {
        TreeState {
            core: CollectionCore::new(),
            expanded: KeySet::new(),
            chosen: None,
            expand_generation: 0,
            source_generation: 0,
            expand_generation_saturated: false,
            source_generation_saturated: false,
            last_expansion: None,
        }
    }

    /// The cursor key.
    #[must_use]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The chosen leaf.
    #[must_use]
    pub const fn chosen(&self) -> Option<ItemKey> {
        self.chosen
    }

    /// Choose a leaf, or clear the choice.
    pub const fn choose(&mut self, key: Option<ItemKey>) {
        self.chosen = key;
    }

    /// The expanded set.
    #[must_use]
    pub const fn expanded(&self) -> &KeySet {
        &self.expanded
    }

    /// Whether `key` is open.
    #[must_use]
    pub fn is_expanded(&self, key: ItemKey) -> bool {
        self.expanded.contains(key)
    }

    /// Open `key`.
    pub fn expand(&mut self, key: ItemKey) {
        if !self.expanded.contains(key) {
            self.expanded.insert(key);
            self.record_expansion(key, true);
        }
    }

    /// Close `key`.
    pub fn collapse(&mut self, key: ItemKey) {
        if self.expanded.contains(key) {
            self.expanded.remove(key);
            self.record_expansion(key, false);
        }
    }

    /// Toggle `key`; returns whether it is open afterwards.
    pub fn toggle(&mut self, key: ItemKey) -> bool {
        self.expanded.toggle(key);
        let expanded = self.expanded.contains(key);
        self.record_expansion(key, expanded);
        expanded
    }

    /// Open every node, without naming one (`KeySet::AllExcept(∅)`, so this
    /// allocates nothing however large the tree is).
    pub fn expand_all(&mut self) {
        self.expanded.all();
        self.bump_expand_generation();
        self.last_expansion = None;
    }

    /// Close every node.
    pub fn collapse_all(&mut self) {
        if !self.expanded.is_empty() {
            self.expanded.none();
            self.bump_expand_generation();
            self.last_expansion = None;
        }
    }

    /// The scroll state.
    #[must_use]
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    /// Point the cursor at the row showing `key`, which is display row
    /// `index`, and reveal it on the next layout.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    /// Report that the borrowed source changed without changing its length.
    ///
    /// Appending or removing nodes is detected from the length. In-place
    /// edits, reorderings and replacements of an equal-length slice require
    /// this explicit invalidation before the next phase.
    pub fn invalidate(&mut self) {
        if let Some(next) = self.source_generation.checked_add(1) {
            self.source_generation = next;
        } else {
            // A saturated revision cannot identify later mutations. Enter a
            // fail-safe mode where the runtime never reuses the source index.
            self.source_generation_saturated = true;
        }
        self.core.invalidate();
    }

    fn record_expansion(&mut self, key: ItemKey, expanded: bool) {
        self.bump_expand_generation();
        self.last_expansion = Some(ExpansionChange {
            generation: self.expand_generation,
            key,
            expanded,
        });
        self.core.invalidate();
    }

    fn bump_expand_generation(&mut self) {
        if let Some(next) = self.expand_generation.checked_add(1) {
            self.expand_generation = next;
        } else {
            // Preserve correctness after saturation: every later phase must
            // rebuild instead of treating equal revisions as unchanged.
            self.expand_generation_saturated = true;
        }
    }

    /// The cursor and the expanded set, borrowed disjointly so the reconcile
    /// closures can read the expansion while the core is being written.
    fn parts_mut(&mut self) -> (&mut CollectionCore, &KeySet) {
        (&mut self.core, &self.expanded)
    }
}

impl Reconcile for TreeState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some(c) = self.chosen
            && !(0..len).any(|i| key(i) == c)
        {
            self.chosen = None;
        }
        r
    }

    fn invalidate(&mut self) {
        TreeState::invalidate(self);
    }
}

/// A keyed, scrollable hierarchy over a borrowed pre-order slice.
///
/// ## Construction
/// `Tree::new(id)`; the nodes are passed to each phase, never held. Without
/// a `.node(…)` accessor every item is a depth-0 leaf, so an unconfigured
/// tree is a flat list.
///
/// ## Ownership
/// The caller owns the nodes (`&[T]` per phase, **pre-order**: a node's
/// descendants follow it contiguously at a greater depth) and a
/// [`TreeState`]. The runtime owns focus, hover, press, wheel routing and
/// the scrollbar capture.
///
/// ## Configuration
/// `.node(&dyn Fn(&T) -> TreeNode)` (default: every item is
/// `TreeNode::leaf(0)`), `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable
/// under reorder — [`TreeNode::keyed`] wins over it), `.row(Fn(&T, &mut
/// RowUi))` (`DefaultRow`: `Display`), `.disabled_item(&dyn Fn(&T) -> bool)`,
/// `.query(revision, &dyn Fn(&T) -> bool)` (strict matches plus ancestors),
/// `.disabled(bool)` (default `false`), `.empty(EmptyState)` (a default
/// "Nothing here yet"), `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::TREE`, `DEFAULT` only.
///
/// ## States
/// The tree wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` and `PRESSED` from
/// the runtime and `DISABLED` from `.disabled`. A row derives `FOCUSED` /
/// `FOCUS_VISIBLE` / `PRESSED` when it is the cursor, `SELECTED` when it is
/// the chosen leaf, `EXPANDED` when it is an open branch, and `DISABLED`
/// from `.disabled_item` or from the whole tree being disabled. The tree
/// takes **no** `.status(Status)` prop: readiness in a tree is per node
/// (`NodeKind::Lazy`), not per component, and §11.4 forbids accepting a
/// readiness prop without painting its affordance.
///
/// ## Actions
/// `Moved`, `Expanded(k)`, `Collapsed(k)`, `Chose(k)`, `Activated(k)`.
/// `Expanded(k)` on a [`NodeKind::Lazy`] node is the fetch request the
/// legacy `TreeEvent::Expand(path)` was, now carrying a key instead of a
/// `Vec<usize>`.
///
/// ## Focus
/// One `Focusable` stop for the whole tree (`Disabled` when `.disabled`);
/// does not swallow typing.
///
/// ## Keyboard
/// `↑`/`k`, `↓`/`j`, `PgUp`, `PgDn`, `Home`/`g`, `End`/`G`; `→`/`l` opens a
/// closed branch or descends into an open one; `←`/`h` closes an open branch
/// or moves to its parent; `Enter` toggles a branch and activates a leaf;
/// `Space` chooses a leaf. When at least one node in the slice can open,
/// `*` and `-` open and close everything.
///
/// ## Mouse
/// `PartRef::item(Part::ROW, k)`: press moves the cursor, click toggles a
/// branch or chooses a leaf, double-click activates a leaf.
/// `PartRef::item(Part::ICON, k)` is the disclosure cell: press or click
/// toggles that node without choosing it. `TRACK`/`THUMB` and the wheel go
/// to the embedded [`ScrollRegion`].
///
/// ## Layout
/// One row per visible node: gutter, `depth × design.space.tree_indent`
/// columns of indent, the one-cell disclosure, then the renderer's row; a
/// scrollbar column when the visible rows overflow. `measure` is
/// `(8…, offered height)`; `draw` returns `area`. `0×0` registers nothing
/// (R5).
///
/// ## Parts
/// `CONTAINER` (the tree surface and each row's fill), `GUTTER` (the focus
/// column), `ICON` (the disclosure cell and its hit zone), `MARKER` (the
/// reserved cell after disclosure), `LABEL` (resolved through [`RowUi`] by
/// the row renderer), `TRACK` / `THUMB` (the embedded [`ScrollRegion`]),
/// `EMPTY` (the no-nodes state). `Part::ROW` is a hit region only and is
/// deliberately not styled.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach `Part::CONTAINER`, `Part::GUTTER`,
/// `Part::ICON`, `Part::MARKER`, `Part::LABEL` and `Part::EMPTY`, and are
/// forwarded to the embedded [`ScrollRegion`] so they also reach
/// `Part::TRACK` and `Part::THUMB`. Owner `CONTAINER` and `LABEL` patches are
/// forwarded only to [`RowUi`]'s automatic row fill and label painters;
/// caller-painted row parts keep their own styles.
/// `.slot(p, …)` changes painted cells for exactly `Part::GUTTER`,
/// `Part::ICON`, `Part::MARKER`, `Part::EMPTY`, `Part::TRACK` and
/// `Part::THUMB`.
/// `Part::CONTAINER` and `Part::LABEL` are **not** slot-addressable: the
/// container fill is overpainted by every row, and the label belongs to the
/// row renderer, which is the caller's own painter already.
///
/// ## Identity
/// [`TreeNode::keyed`] first, then `.key`, then `ByIndex` — which is
/// unstable under insert, remove and reorder. Expansion, the cursor and the
/// chosen leaf are all stored as `ItemKey`, so a node keeps its state across
/// an expand, a collapse and a reorder
/// (`tree::expand_collapse_is_keyed_not_positional`).
///
/// ## Testing
/// `TreeCase` with `ACTIVATES | DISABLEABLE | FOCUSABLE | COLLECTION |
/// SCROLLS`; `render::components::tree::*`. `CAPTURES` belongs to the
/// embedded [`ScrollRegion`], whose thumb claims the capture, and is
/// declared by `ScrollRegionCase`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted. The runtime cache owns only
/// source indexes, keys, depth and kind — never `T` or borrowed text. Initial
/// source/query changes rebuild once; ordinary expand/collapse splices the
/// affected contiguous subtree; unchanged update and draw are bounded by the
/// viewport. Only visible rows invoke the borrowed row renderer.
pub struct Tree<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    node: Option<&'a dyn Fn(&T) -> TreeNode>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    query: Option<TreeQuery<'a, T>>,
    disabled: bool,
    empty: Option<EmptyState<'a>>,
    ov: PartStyle<'a>,
    /// The same three override channels again, kept because `PartStyle` has
    /// no readers: forwarding a caller's `.patch` / `.patch_part` / `.slot`
    /// into the embedded [`ScrollRegion`] is the §45.1 defect `List` still
    /// carries, and it cannot be done from the stored `PartStyle` alone.
    fwd_patch: Option<&'a StylePatch>,
    fwd_parts: &'a [(Part, StylePatch)],
    fwd_slot: Option<(Part, SlotFn<'a>)>,
    _t: PhantomData<fn(&T)>,
}

#[derive(Clone, Copy)]
struct TreeQuery<'a, T> {
    revision: u64,
    matcher: &'a dyn Fn(&T) -> bool,
}

impl<T, K, R> fmt::Debug for Tree<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree")
            .field("id", &self.id)
            .field("disabled", &self.disabled)
            .field("empty", &self.empty)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<T> Tree<'_, T, ByIndex, DefaultRow> {
    /// A tree keyed by index and painted through `Display`.
    #[must_use]
    pub const fn new(id: Id) -> Self {
        Tree {
            id,
            key: ByIndex,
            row: DefaultRow,
            node: None,
            disabled_item: None,
            query: None,
            disabled: false,
            empty: None,
            ov: PartStyle::new(),
            fwd_patch: None,
            fwd_parts: &[],
            fwd_slot: None,
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> Tree<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::ICON,
        Part::MARKER,
        Part::LABEL,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];

    /// The width `measure` prefers.
    pub const PREFERRED_WIDTH: u16 = 28;

    /// The id.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The hierarchy accessor: depth, disclosure and an optional caller key.
    #[must_use]
    pub const fn node(mut self, f: &'a dyn Fn(&T) -> TreeNode) -> Self {
        self.node = Some(f);
        self
    }

    /// A stable key accessor. [`TreeNode::keyed`] overrides it per node.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tree<'a, T, K2, R> {
        Tree {
            id: self.id,
            key: k,
            row: self.row,
            node: self.node,
            disabled_item: self.disabled_item,
            query: self.query,
            disabled: self.disabled,
            empty: self.empty,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    /// A row painter, called only for the visible rows.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tree<'a, T, K, R2> {
        Tree {
            id: self.id,
            key: self.key,
            row: r,
            node: self.node,
            disabled_item: self.disabled_item,
            query: self.query,
            disabled: self.disabled,
            empty: self.empty,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    /// Which nodes are not selectable (the legacy `note` rows).
    #[must_use]
    pub const fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    /// Filter nodes with a borrowed matcher and a stable caller revision.
    ///
    /// Matching nodes retain their ancestor path. The caller must change
    /// `revision` whenever the matcher's meaning changes; the runtime then
    /// rebuilds the filtered index once and shares it across update/draw.
    #[must_use]
    pub const fn query(mut self, revision: u64, matcher: &'a dyn Fn(&T) -> bool) -> Self {
        self.query = Some(TreeQuery { revision, matcher });
        self
    }

    /// Disable the whole tree: it stays in the ring, unreachable, and
    /// ignores every input.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// What to paint when there are no nodes.
    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self.fwd_patch = Some(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self.fwd_parts = ps;
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self.fwd_slot = Some((p, f));
        self
    }

    /// The embedded scroll region, carrying the caller's overrides (§45.1:
    /// a nested component built bare silently drops `.patch_part` and
    /// `.slot`).
    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut s = ScrollRegion::new(self.id).patch_part(self.fwd_parts);
        if let Some(p) = self.fwd_patch {
            s = s.patch(p);
        }
        if let Some((p, f)) = self.fwd_slot {
            s = s.slot(p, f);
        }
        s
    }

    /// The derived half of the state (§39.2, Invariant Q).
    const fn derived(&self) -> StateFlags {
        if self.disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::empty()
        }
    }

    fn node_of(&self, item: &T) -> TreeNode {
        self.node.map_or(TreeNode::leaf(0), |f| f(item))
    }

    fn is_disabled(&self, item: &T) -> bool {
        self.disabled || self.disabled_item.is_some_and(|f| f(item))
    }
}

/// One source row in the runtime-owned tree index. It contains no borrowed
/// item or text, so clearing the cache can never invalidate application data.
#[derive(Clone, Copy, Debug)]
struct FlatRef {
    source: usize,
    key: ItemKey,
    depth: u16,
    kind: NodeKind,
    included: bool,
}

/// Derived structure shared by update and draw for one tree id.
#[derive(Default)]
struct TreeIndex {
    initialized: bool,
    source_generation: u64,
    expand_generation: u64,
    query_revision: Option<u64>,
    query_active: bool,
    source_len: usize,
    rows: Vec<FlatRef>,
    by_key: HashMap<ItemKey, usize>,
    visible: Vec<usize>,
    foldable: bool,
    #[cfg(test)]
    source_rebuilds: usize,
    #[cfg(test)]
    query_rebuilds: usize,
    #[cfg(test)]
    splices: usize,
    #[cfg(test)]
    visible_rebuilds: usize,
}

impl TreeIndex {
    fn sync<T, K: KeyFn<T>, R>(
        &mut self,
        tree: &Tree<'_, T, K, R>,
        state: &TreeState,
        items: &[T],
    ) {
        let query_revision = tree.query.as_ref().map(|query| query.revision);
        let source_changed = !self.initialized
            || self.source_generation != state.source_generation
            || state.source_generation_saturated
            || self.source_len != items.len();
        if source_changed {
            self.rebuild_source(tree, state, items, query_revision);
            return;
        }
        if self.query_revision != query_revision {
            self.apply_query(tree, items);
            self.query_revision = query_revision;
            self.query_active = query_revision.is_some();
            self.rebuild_visible(&state.expanded);
            self.expand_generation = state.expand_generation;
            #[cfg(test)]
            {
                self.query_rebuilds = self.query_rebuilds.saturating_add(1);
            }
            return;
        }
        if state.expand_generation_saturated {
            self.rebuild_visible(&state.expanded);
            self.expand_generation = state.expand_generation;
            return;
        }
        if self.expand_generation == state.expand_generation {
            return;
        }
        if self.query_active {
            self.expand_generation = state.expand_generation;
            return;
        }
        let incremental = self.expand_generation.saturating_add(1) == state.expand_generation
            && state
                .last_expansion
                .is_some_and(|change| change.generation == state.expand_generation);
        if incremental {
            if let Some(change) = state.last_expansion {
                self.splice(change, &state.expanded);
            }
        } else {
            self.rebuild_visible(&state.expanded);
        }
        self.expand_generation = state.expand_generation;
    }

    fn rebuild_source<T, K: KeyFn<T>, R>(
        &mut self,
        tree: &Tree<'_, T, K, R>,
        state: &TreeState,
        items: &[T],
        query_revision: Option<u64>,
    ) {
        self.rows.clear();
        self.by_key.clear();
        self.rows.reserve(items.len());
        self.by_key.reserve(items.len());
        self.foldable = false;
        for (source, item) in items.iter().enumerate() {
            let node = tree.node_of(item);
            let key = node.key().unwrap_or_else(|| tree.key.key(item, source));
            self.foldable |= node.has_children();
            self.by_key.insert(key, source);
            self.rows.push(FlatRef {
                source,
                key,
                depth: node.depth(),
                kind: node.kind(),
                included: true,
            });
        }
        self.query_active = query_revision.is_some();
        self.apply_query(tree, items);
        self.rebuild_visible(&state.expanded);
        self.initialized = true;
        self.source_generation = state.source_generation;
        self.expand_generation = state.expand_generation;
        self.query_revision = query_revision;
        self.source_len = items.len();
        #[cfg(test)]
        {
            self.source_rebuilds = self.source_rebuilds.saturating_add(1);
        }
    }

    fn apply_query<T, K, R>(&mut self, tree: &Tree<'_, T, K, R>, items: &[T]) {
        let Some(query) = tree.query.as_ref() else {
            for row in &mut self.rows {
                row.included = true;
            }
            return;
        };
        for row in &mut self.rows {
            row.included = false;
        }
        let mut ancestors: Vec<usize> = Vec::new();
        for index in 0..self.rows.len() {
            let Some(row) = self.rows.get(index).copied() else {
                continue;
            };
            ancestors.truncate(usize::from(row.depth));
            if items.get(row.source).is_some_and(query.matcher) {
                if let Some(current) = self.rows.get_mut(index) {
                    current.included = true;
                }
                for ancestor in &ancestors {
                    if let Some(parent) = self.rows.get_mut(*ancestor) {
                        parent.included = true;
                    }
                }
            }
            ancestors.push(index);
        }
    }

    fn rebuild_visible(&mut self, expanded: &KeySet) {
        #[cfg(test)]
        {
            self.visible_rebuilds = self.visible_rebuilds.saturating_add(1);
        }
        self.visible.clear();
        self.visible.reserve(self.rows.len());
        if self.query_active {
            self.visible.extend(
                self.rows
                    .iter()
                    .enumerate()
                    .filter_map(|(index, row)| row.included.then_some(index)),
            );
            return;
        }
        let mut collapsed_depth: Option<u16> = None;
        for (index, row) in self.rows.iter().enumerate() {
            if collapsed_depth.is_some_and(|depth| row.depth > depth) {
                continue;
            }
            collapsed_depth = None;
            if !row.included {
                continue;
            }
            self.visible.push(index);
            if matches!(row.kind, NodeKind::Parent | NodeKind::Lazy) && !expanded.contains(row.key)
            {
                collapsed_depth = Some(row.depth);
            }
        }
    }

    fn splice(&mut self, change: ExpansionChange, expanded: &KeySet) {
        let Some(&source) = self.by_key.get(&change.key) else {
            return;
        };
        let Ok(display) = self.visible.binary_search(&source) else {
            return;
        };
        let Some(parent) = self.rows.get(source).copied() else {
            return;
        };
        if change.expanded {
            let mut inserted = Vec::new();
            let mut collapsed_depth: Option<u16> = None;
            for index in source.saturating_add(1)..self.rows.len() {
                let Some(row) = self.rows.get(index).copied() else {
                    break;
                };
                if row.depth <= parent.depth {
                    break;
                }
                if collapsed_depth.is_some_and(|depth| row.depth > depth) {
                    continue;
                }
                collapsed_depth = None;
                if !row.included {
                    continue;
                }
                inserted.push(index);
                if matches!(row.kind, NodeKind::Parent | NodeKind::Lazy)
                    && !expanded.contains(row.key)
                {
                    collapsed_depth = Some(row.depth);
                }
            }
            self.visible.splice(
                display.saturating_add(1)..display.saturating_add(1),
                inserted,
            );
        } else {
            let start = display.saturating_add(1);
            let mut end = start;
            while self
                .visible
                .get(end)
                .and_then(|index| self.rows.get(*index))
                .is_some_and(|row| row.depth > parent.depth)
            {
                end = end.saturating_add(1);
            }
            self.visible.drain(start..end);
        }
        #[cfg(test)]
        {
            self.splices = self.splices.saturating_add(1);
        }
    }

    fn row(&self, display: usize) -> Option<FlatRef> {
        self.visible
            .get(display)
            .and_then(|source| self.rows.get(*source))
            .copied()
    }

    fn display_of(&self, key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(hint) = hint
            && self.row(hint).is_some_and(|row| row.key == key)
        {
            return Some(hint);
        }
        let source = *self.by_key.get(&key)?;
        self.visible.binary_search(&source).ok()
    }

    fn has_visible_descendant(&self, display: usize) -> bool {
        let Some(row) = self.row(display) else {
            return false;
        };
        self.row(display.saturating_add(1))
            .is_some_and(|next| next.depth > row.depth)
    }
}

/// One visible row, resolved once and painted once.
#[derive(Clone, Copy)]
struct Row {
    rect: Rect,
    flags: StateFlags,
    key: ItemKey,
    depth: u16,
    disclosure: Option<GlyphRole>,
}

#[derive(Clone, Copy)]
struct RowContext {
    node: FlatRef,
    live: StateFlags,
    rect: Rect,
    disabled: bool,
    query_active: bool,
    has_visible_descendant: bool,
}

#[derive(Clone, Copy)]
struct PointerIntent {
    phase: Phase,
    part: PartRef,
    hint: Option<usize>,
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Tree<'_, T, K, R> {
    fn table_for(&self, foldable: bool) -> &'static [Binding<TreeCmd>] {
        if self.disabled {
            &[]
        } else if foldable {
            &TABLE_FOLDABLE
        } else {
            &TABLE
        }
    }

    fn move_to(st: &mut TreeState, index: &TreeIndex, to: usize, acc: &mut Acc<TreeAction>) {
        if index.visible.is_empty() {
            acc.consumed();
            return;
        }
        let to = to.min(index.visible.len().saturating_sub(1));
        if let Some(row) = index.row(to) {
            st.core.set_cursor(to, row.key);
            acc.action(TreeAction::Moved);
        } else {
            acc.consumed();
        }
    }

    /// Open or close the node at display row `d`; emits nothing for a leaf.
    fn toggle_at(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        d: usize,
        acc: &mut Acc<TreeAction>,
    ) -> bool {
        let Some(row) = index.row(d) else {
            acc.consumed();
            return false;
        };
        let Some(it) = items.get(row.source) else {
            acc.consumed();
            return false;
        };
        if !self.node_of(it).has_children() {
            return false;
        }
        let key = row.key;
        if index.query_active {
            if row.kind == NodeKind::Lazy
                && !index.has_visible_descendant(d)
                && !st.is_expanded(key)
            {
                st.expand(key);
                index.sync(self, st, items);
                acc.action(TreeAction::Expanded(key));
            } else {
                acc.consumed();
            }
            return true;
        }
        let open = st.toggle(key);
        index.sync(self, st, items);
        acc.action(if open {
            TreeAction::Expanded(key)
        } else {
            TreeAction::Collapsed(key)
        });
        true
    }

    /// Enter / a click on a row: toggle a branch, otherwise choose or
    /// activate the leaf.
    fn engage(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        d: usize,
        activate: bool,
        acc: &mut Acc<TreeAction>,
    ) {
        let Some(row) = index.row(d) else {
            acc.consumed();
            return;
        };
        let Some(it) = items.get(row.source) else {
            acc.consumed();
            return;
        };
        if self.is_disabled(it) {
            acc.consumed();
            return;
        }
        if self.node_of(it).has_children() {
            let _ = self.toggle_at(st, index, items, d, acc);
            return;
        }
        let key = row.key;
        st.chosen = Some(key);
        acc.action(if activate {
            TreeAction::Activated(key)
        } else {
            TreeAction::Chose(key)
        });
    }

    /// `←` / `h`: close an open branch, else move to the parent row.
    fn collapse_or_parent(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        d: usize,
        acc: &mut Acc<TreeAction>,
    ) {
        let Some(row) = index.row(d) else {
            acc.consumed();
            return;
        };
        let Some(it) = items.get(row.source) else {
            acc.consumed();
            return;
        };
        let n = self.node_of(it);
        if index.query_active {
            if n.depth() == 0 {
                acc.consumed();
                return;
            }
            let mut probe = d;
            while probe > 0 {
                probe = probe.saturating_sub(1);
                if index
                    .row(probe)
                    .is_some_and(|parent| parent.depth < n.depth())
                {
                    Self::move_to(st, index, probe, acc);
                    return;
                }
            }
            acc.consumed();
            return;
        }
        if n.has_children() && st.is_expanded(row.key) {
            let _ = self.toggle_at(st, index, items, d, acc);
            return;
        }
        if n.depth() == 0 {
            acc.consumed();
            return;
        }
        // the parent is the nearest earlier row at a smaller depth
        let mut probe = d;
        while probe > 0 {
            probe = probe.saturating_sub(1);
            let Some(parent) = index.row(probe) else {
                break;
            };
            if items
                .get(parent.source)
                .is_some_and(|p| self.node_of(p).depth() < n.depth())
            {
                Self::move_to(st, index, probe, acc);
                return;
            }
        }
        acc.consumed();
    }

    /// `→` / `l`: open a closed branch, else step into it.
    fn expand_or_descend(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        d: usize,
        acc: &mut Acc<TreeAction>,
    ) {
        let Some(row) = index.row(d) else {
            acc.consumed();
            return;
        };
        let open = items
            .get(row.source)
            .is_some_and(|it| self.node_of(it).has_children())
            && st.is_expanded(row.key);
        if index.query_active {
            if index.has_visible_descendant(d) {
                Self::move_to(st, index, d.saturating_add(1), acc);
            } else if row.kind == NodeKind::Lazy {
                let _ = self.toggle_at(st, index, items, d, acc);
            } else {
                acc.consumed();
            }
            return;
        }
        if open {
            Self::move_to(st, index, d.saturating_add(1), acc);
        } else if !self.toggle_at(st, index, items, d, acc) {
            acc.consumed();
        }
    }

    /// The update phase: reconcile over the **visible** rows, then drain
    /// keys, pointer and wheel.
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TreeState, items: &[T]) -> Response<TreeAction> {
        if self.disabled {
            return Response::ignored();
        }
        let len = {
            let index = cx.cache::<TreeIndex>(self.id);
            index.sync(self, st, items);
            self.reconcile(st, items, index)
        };
        let mut acc = Acc::<TreeAction>::new();
        let bar = self.scrollbar().update(cx, st.core.scroll_mut(), len);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let hint_area = cx.area(self.id);
        let (intents, index) = cx.intents_with_cache::<TreeIndex>(self.id);
        let table = self.table_for(index.foldable && !index.query_active);
        for it in intents {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(TreeCmd::Up) => {
                            Self::move_to(st, index, cur.saturating_sub(1), &mut acc);
                        }
                        Some(TreeCmd::Down) => {
                            Self::move_to(st, index, cur.saturating_add(1), &mut acc);
                        }
                        Some(TreeCmd::PageUp) => {
                            Self::move_to(st, index, cur.saturating_sub(viewport), &mut acc);
                        }
                        Some(TreeCmd::PageDown) => {
                            Self::move_to(st, index, cur.saturating_add(viewport), &mut acc);
                        }
                        Some(TreeCmd::Home) => Self::move_to(st, index, 0, &mut acc),
                        Some(TreeCmd::End) => Self::move_to(st, index, usize::MAX, &mut acc),
                        Some(TreeCmd::Expand) => {
                            self.expand_or_descend(st, index, items, cur, &mut acc);
                        }
                        Some(TreeCmd::Collapse) => {
                            self.collapse_or_parent(st, index, items, cur, &mut acc);
                        }
                        Some(TreeCmd::Activate) => {
                            self.engage(st, index, items, cur, true, &mut acc);
                        }
                        Some(TreeCmd::Choose) => {
                            self.engage(st, index, items, cur, false, &mut acc);
                        }
                        Some(TreeCmd::ExpandAll) => {
                            if index.query_active {
                                acc.consumed();
                            } else {
                                st.expand_all();
                                index.sync(self, st, items);
                                acc.action(TreeAction::Moved);
                            }
                        }
                        Some(TreeCmd::CollapseAll) => {
                            if index.query_active {
                                acc.consumed();
                            } else {
                                st.collapse_all();
                                index.sync(self, st, items);
                                acc.action(TreeAction::Moved);
                            }
                        }
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase, part, pos, ..
                } => {
                    let hint = hint_area.map(|a| {
                        let view = ScrollRegion::view(st.core.scroll(), a, len);
                        view.offset()
                            .saturating_add(usize::from(pos.y.saturating_sub(a.y)))
                    });
                    self.pointer(
                        st,
                        index,
                        items,
                        PointerIntent { phase, part, hint },
                        &mut acc,
                    );
                }
                _ => {}
            }
        }
        // a toggle changed how many rows there are; the scrollbar must not
        // spend a frame believing the old count
        let after = index.visible.len();
        st.core.scroll_mut().set_content(after);
        acc.finish(self.id)
    }

    /// Reconcile the cursor and the chosen leaf against the visible rows and
    /// seed the cursor when there is none. Returns the visible row count.
    fn reconcile(&self, st: &mut TreeState, items: &[T], index: &TreeIndex) -> usize {
        let len = index.visible.len();
        {
            let (core, _) = st.parts_mut();
            let _ = core.reconcile_with(
                len,
                |d| index.row(d).map_or(ItemKey::index(d), |row| row.key),
                |d| {
                    index
                        .row(d)
                        .and_then(|row| items.get(row.source))
                        .is_some_and(|item| !self.is_disabled(item))
                },
            );
        }
        if let Some(c) = st.chosen
            && !index.by_key.contains_key(&c)
        {
            st.chosen = None;
        }
        if st.core.cursor().is_none()
            && let Some(d) = (0..len).find(|&d| {
                index
                    .row(d)
                    .and_then(|row| items.get(row.source))
                    .is_some_and(|item| !self.is_disabled(item))
            })
            && let Some(row) = index.row(d)
        {
            st.core.set_cursor(d, row.key);
        }
        len
    }

    fn pointer(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        pointer: PointerIntent,
        acc: &mut Acc<TreeAction>,
    ) {
        let PointerIntent { phase, part, hint } = pointer;
        let Some(key) = part.item else {
            acc.consumed();
            return;
        };
        let Some(d) = index.display_of(key, hint) else {
            acc.consumed();
            return;
        };
        if part.part == Part::ICON {
            match phase {
                Phase::Press | Phase::Click => {
                    st.core.set_cursor(d, key);
                    if !self.toggle_at(st, index, items, d, acc) {
                        acc.changed();
                    }
                }
                _ => acc.consumed(),
            }
            return;
        }
        if part.part != Part::ROW {
            acc.consumed();
            return;
        }
        match phase {
            Phase::Press => {
                st.core.set_cursor(d, key);
                acc.changed();
            }
            Phase::Click => self.engage(st, index, items, d, false, acc),
            Phase::DoubleClick => self.engage(st, index, items, d, true, acc),
            _ => acc.consumed(),
        }
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TreeState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        if !ui.is_inert() {
            ui.register_control(
                self.id,
                area,
                if self.disabled {
                    Focusability::Disabled
                } else {
                    Focusability::Focusable
                },
            );
        }
        let live = PartStyle::flags(ui.state(self.id), self.derived());
        let (len, foldable, query_active) = {
            let index = ui.cache::<TreeIndex>(self.id);
            index.sync(self, st, items);
            (index.visible.len(), index.foldable, index.query_active)
        };
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table_for(foldable && !query_active));
        }
        let container = self.ov.style(
            ui,
            self.id,
            Family::TREE,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let content = self.scrollbar().draw(ui, area, st.core.scroll(), len);
        if len == 0 {
            self.draw_empty(ui, area, live);
            return area;
        }
        let view = ScrollRegion::view(st.core.scroll(), content, len);
        let indent = ui.design().space.tree_indent;
        for (offset, d) in view.visible_range().enumerate() {
            let Some(flat) = ui.cache::<TreeIndex>(self.id).row(d) else {
                break;
            };
            let Some(item) = items.get(flat.source) else {
                break;
            };
            let rect = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(offset.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: 1,
            };
            let (query_active, has_visible_descendant) = {
                let index = ui.cache::<TreeIndex>(self.id);
                (index.query_active, index.has_visible_descendant(d))
            };
            let row = Self::row_of(
                st,
                RowContext {
                    node: flat,
                    live,
                    rect,
                    disabled: self.is_disabled(item),
                    query_active,
                    has_visible_descendant,
                },
            );
            self.paint_row(ui, row, indent, item);
        }
        area
    }

    fn draw_empty(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) {
        let mid = Rect {
            y: area.y.saturating_add(area.height / 2),
            height: area.height.saturating_sub(area.height / 2),
            ..area
        };
        if let Some(f) = self.ov.slot_for(Part::EMPTY) {
            f(ui, mid);
            return;
        }
        let _ = self.ov.style(
            ui,
            self.id,
            Family::TREE,
            Variant::DEFAULT,
            Part::EMPTY,
            live,
        );
        self.empty
            .unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            })
            .draw(ui, mid, 0);
    }

    /// Resolve one visible row's identity, flags and disclosure glyph.
    fn row_of(st: &TreeState, context: RowContext) -> Row {
        let RowContext {
            node,
            live,
            rect,
            disabled,
            query_active,
            has_visible_descendant,
        } = context;
        let key = node.key;
        let has_children = matches!(node.kind, NodeKind::Parent | NodeKind::Lazy);
        let open = has_children
            && if query_active {
                has_visible_descendant
            } else {
                st.is_expanded(key)
            };
        let is_cursor = st.core.cursor() == Some(key);
        let mut flags = StateFlags::empty();
        if is_cursor {
            flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::PRESSED);
        }
        if st.chosen == Some(key) {
            flags |= StateFlags::SELECTED;
        }
        if open {
            flags |= StateFlags::EXPANDED;
        }
        if disabled || live.contains(StateFlags::DISABLED) {
            flags |= StateFlags::DISABLED;
            flags = flags.difference(StateFlags::PRESSED);
        }
        Row {
            rect,
            flags,
            key,
            depth: node.depth,
            disclosure: if has_children {
                Some(if open {
                    GlyphRole::Expanded
                } else {
                    GlyphRole::Collapsed
                })
            } else {
                None
            },
        }
    }

    fn paint_row(&self, ui: &mut Ui<'_>, row: Row, indent: u16, item: &T) {
        let rs = self.ov.style(
            ui,
            self.id,
            Family::TREE,
            Variant::DEFAULT,
            Part::CONTAINER,
            row.flags,
        );
        ui.fill(row.rect, rs.style);
        let gutter = cell_at(row.rect, row.rect.x);
        if let Some(f) = self.ov.slot_for(Part::GUTTER) {
            f(ui, gutter);
        } else {
            let g = self.ov.style(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                Part::GUTTER,
                row.flags,
            );
            match g.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter, glyph, g.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(gutter, g.style),
            }
        }
        let fold_x = row
            .rect
            .x
            .saturating_add(1)
            .saturating_add(row.depth.saturating_mul(indent));
        let fold = cell_at(row.rect, fold_x);
        if let Some(f) = self.ov.slot_for(Part::ICON) {
            f(ui, fold);
        } else {
            let icon = self.ov.style(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                Part::ICON,
                row.flags,
            );
            let glyph = match icon.glyph {
                Slot::Set(g) => Some(g),
                Slot::Inherit => row.disclosure,
                Slot::Clear => None,
            };
            match glyph {
                Some(g) => {
                    ui.glyph(fold, g, icon.style);
                }
                None => ui.fill(fold, icon.style),
            }
        }
        let marker = cell_at(row.rect, fold_x.saturating_add(1));
        self.paint_marker(ui, row, marker);
        let rest = Rect {
            x: fold_x.saturating_add(2),
            width: row
                .rect
                .right()
                .saturating_sub(fold_x.saturating_add(2))
                .min(row.rect.width),
            ..row.rect
        };
        if rest.width > 0 && rest.x < row.rect.right() {
            let mut r = RowUi::new_with_patches(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                row.flags,
                row.key,
                rest,
                self.ov.part_patch(Part::CONTAINER),
                self.ov.part_patch(Part::LABEL),
            );
            self.row.row(item, &mut r);
        }
        if ui.is_inert() {
            return;
        }
        ui.register_part(self.id, PartRef::item(Part::ROW, row.key), row.rect);
        // the disclosure is registered last so it wins hit-testing over the
        // row it sits inside
        if row.disclosure.is_some() {
            ui.register_part(self.id, PartRef::item(Part::ICON, row.key), fold);
        }
    }

    fn paint_marker(&self, ui: &mut Ui<'_>, row: Row, marker: Rect) {
        if !row.flags.contains(StateFlags::SELECTED) {
            return;
        }
        if let Some(f) = self.ov.slot_for(Part::MARKER) {
            f(ui, marker);
        } else {
            let marker_style = self.ov.style(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                Part::MARKER,
                row.flags,
            );
            match marker_style.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(marker, glyph, marker_style.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(marker, marker_style.style),
            }
        }
    }

    /// The natural size: 28 columns, whatever height is offered.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (8, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }
}

impl<T, K, R> Bindings for Tree<'_, T, K, R> {
    type Cmd = TreeCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TreeCmd>] {
        if self.disabled {
            &[]
        } else if self.query.is_some() {
            &TABLE
        } else {
            &TABLE_FOLDABLE
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::{Cell, RefCell};

    use ratatui_core::buffer::{Buffer, Cell as BufferCell};
    use ratatui_core::layout::{Position, Rect};

    use super::{Acc, TABLE, Tree, TreeAction, TreeIndex, TreeNode, TreeState};
    use crate::collection::RowUi;
    use crate::id::{Id, ItemKey, Part};
    use crate::response::StateFlags;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{ColorLevel, GlyphRole, Modifier, Role, Slot, StylePatch, Theme};
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, Ui, UiCore};

    const TREE: Id = Id::root("tree.tests");

    /// A node the tests describe positionally: `(name, depth, parent)`.
    #[derive(Clone, Copy, Debug)]
    struct N(&'static str, u16, bool);

    impl core::fmt::Display for N {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.0)
        }
    }

    fn node(n: &N) -> TreeNode {
        let t = if n.2 {
            TreeNode::parent(n.1)
        } else {
            TreeNode::leaf(n.1)
        };
        t.keyed(ItemKey::text(n.0))
    }

    fn tree<'a>() -> Tree<'a, N> {
        Tree::new(TREE).node(&node)
    }

    /// The labels the tree would paint, in order, without painting anything.
    fn visible(t: &Tree<'_, N>, items: &[N], st: &TreeState) -> Vec<&'static str> {
        let mut index = TreeIndex::default();
        index.sync(t, st, items);
        (0..index.visible.len())
            .filter_map(|d| index.row(d))
            .filter_map(|row| items.get(row.source).map(|n| n.0))
            .collect()
    }

    /// A forest with two roots, each with two children, the second of which
    /// has a grandchild.
    const FOREST: [N; 8] = [
        N("alpha", 0, true),
        N("a1", 1, false),
        N("a2", 1, true),
        N("a2x", 2, false),
        N("beta", 0, true),
        N("b1", 1, false),
        N("b2", 1, true),
        N("b2x", 2, false),
    ];

    /// §16.1 `tree::expand_collapse_is_keyed_not_positional`, and the §33
    /// hazard verbatim: `ChipBar` lost a chip's identity after a reorder
    /// because `draw` iterated from index 0 while the state was
    /// key-addressed. A tree with a scroll window has the same shape, so the
    /// expanded set, the cursor and the chosen leaf are all `ItemKey` and
    /// every one of them must survive an expand, a collapse **and** a
    /// reorder of the slice.
    #[test]
    fn expand_collapse_is_keyed_not_positional() {
        let t = tree();
        let mut st = TreeState::new();
        let beta = ItemKey::text("beta");
        let alpha = ItemKey::text("alpha");

        // nothing is open: two roots
        assert_eq!(visible(&t, &FOREST, &st), ["alpha", "beta"]);

        // open the SECOND root, and remember it by key
        st.expand(beta);
        st.set_cursor(2, ItemKey::text("b1"));
        st.choose(Some(ItemKey::text("b1")));
        assert_eq!(visible(&t, &FOREST, &st), ["alpha", "beta", "b1", "b2"]);

        // reorder: `beta`'s subtree moves to the front. Positional identity
        // would now have `alpha` open — index 0 was the open one.
        let reordered = [
            FOREST[4], FOREST[5], FOREST[6], FOREST[7], FOREST[0], FOREST[1], FOREST[2], FOREST[3],
        ];
        assert!(
            st.is_expanded(beta),
            "the expanded set is keyed, so a reorder cannot move it"
        );
        assert!(!st.is_expanded(alpha));
        assert_eq!(
            visible(&t, &reordered, &st),
            ["beta", "b1", "b2", "alpha"],
            "the same NODE is open after the reorder, not the same index"
        );

        // the cursor and the chosen leaf still name `b1`, and reconciling
        // against the reordered slice moves them to its new display row
        let mut st2 = st.clone();
        let mut index = TreeIndex::default();
        index.sync(&t, &st2, &reordered);
        let len = t.reconcile(&mut st2, &reordered, &index);
        assert_eq!(len, 4);
        assert_eq!(st2.cursor(), Some(ItemKey::text("b1")));
        assert_eq!(st2.chosen(), Some(ItemKey::text("b1")));

        // collapse it again by key; `alpha` is untouched throughout
        st.collapse(beta);
        assert_eq!(visible(&t, &reordered, &st), ["beta", "alpha"]);
        assert!(!st.is_expanded(beta));

        // a grandchild only appears when BOTH ancestors are open, and each
        // is addressed by its own key
        st.expand(beta);
        st.expand(ItemKey::text("b2"));
        assert_eq!(
            visible(&t, &reordered, &st),
            ["beta", "b1", "b2", "b2x", "alpha"]
        );
    }

    /// §16.1 `tree::lazy_children_do_not_reflatten_the_world`.
    ///
    /// The legacy `TreeView::flatten` built a `Vec<FlatRow>` with a cloned
    /// `label` and `meta` for **every** node on every toggle and every lazy
    /// delivery. The property that forbids it is observable without an
    /// allocator: the row renderer — the only thing that can touch a node's
    /// text — must run once per **visible** row, never once per node, and a
    /// collapsed subtree must not reach it at all.
    #[test]
    fn lazy_children_do_not_reflatten_the_world() {
        // one root with 10 000 lazy-delivered children, all present in the
        // slice, in a six-row viewport
        let mut items = vec![N("root", 0, true)];
        items.extend((0..10_000).map(|_| N("child", 1, false)));

        let painted: Cell<usize> = Cell::new(0);
        let seen_depth1: Cell<usize> = Cell::new(0);
        let row = |n: &N, u: &mut RowUi<'_>| {
            painted.set(painted.get().saturating_add(1));
            if n.1 == 1 {
                seen_depth1.set(seen_depth1.get().saturating_add(1));
            }
            u.label(n.0);
        };
        let t = Tree::new(TREE).node(&node_by_index).row(row);

        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 6,
        };
        let mut st = TreeState::new();

        // collapsed: one visible row, one renderer call, and not one of the
        // 10 000 hidden children is touched
        draw(&t, &st, &items, area);
        assert_eq!(painted.get(), 1);
        assert_eq!(
            seen_depth1.get(),
            0,
            "a collapsed subtree reached the renderer"
        );

        // expanded: the viewport, not the world
        painted.set(0);
        st.expand(ItemKey::index(0));
        draw(&t, &st, &items, area);
        assert_eq!(
            painted.get(),
            6,
            "the renderer ran once per node instead of once per visible row"
        );
        assert!(seen_depth1.get() <= 6);

        // and the toggle itself sees the new row count without building one
        let mut index = TreeIndex::default();
        index.sync(&t, &st, &items);
        assert_eq!(index.visible.len(), 10_001);
        st.collapse(ItemKey::index(0));
        index.sync(&t, &st, &items);
        assert_eq!(index.visible.len(), 1);
    }

    fn node_by_index(n: &N) -> TreeNode {
        if n.2 {
            TreeNode::parent(n.1)
        } else {
            TreeNode::leaf(n.1)
        }
    }

    fn draw<R: crate::collection::RowFn<N>>(
        t: &Tree<'_, N, crate::collection::ByIndex, R>,
        st: &TreeState,
        items: &[N],
        area: Rect,
    ) {
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
        t.draw(&mut ui, area, st, items);
    }

    fn draw_cached<T, K: crate::collection::KeyFn<T>, R: crate::collection::RowFn<T>>(
        t: &Tree<'_, T, K, R>,
        st: &TreeState,
        items: &[T],
        area: Rect,
        core: &mut UiCore,
    ) {
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, area);
        let mut page = Buffer::empty(area);
        let last = LastFrame::default();
        let mut ui = Ui::new(&mut fs, &mut page, core, &theme, &last);
        t.draw(&mut ui, area, st, items);
    }

    fn render<K: crate::collection::KeyFn<N>, R: crate::collection::RowFn<N>>(
        theme: Theme,
        tree: &Tree<'_, N, K, R>,
        state: &TreeState,
        items: &[N],
    ) -> Buffer {
        let area = Rect::new(0, 0, 24, items.len().max(1) as u16);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, area| {
            tree.draw(ui, area, state, items);
        });
        buffer
    }

    /// A `NodeKind::Lazy` node reports its expansion by key, which is the
    /// caller's cue to fetch. The legacy contract carried a `Vec<usize>`
    /// path and forced `object_at`-style reconstruction on the other side.
    #[test]
    fn a_lazy_node_reports_its_expansion_by_key() {
        let items = [N("db", 0, false)];
        let lazy = |n: &N| TreeNode::lazy(n.1).keyed(ItemKey::text(n.0));
        let t = Tree::new(TREE).node(&lazy);
        let mut st = TreeState::new();
        let mut acc = Acc::<TreeAction>::new();

        let mut index = TreeIndex::default();
        index.sync(&t, &st, &items);
        assert_eq!(index.visible.len(), 1);
        assert!(
            t.toggle_at(&mut st, &mut index, &items, 0, &mut acc),
            "a lazy node has a disclosure affordance"
        );
        assert_eq!(
            acc.finish(TREE).action_ref(),
            Some(&TreeAction::Expanded(ItemKey::text("db"))),
            "expanding a lazy node is the fetch request, carrying its key"
        );
        assert!(st.is_expanded(ItemKey::text("db")));

        // a leaf has no disclosure and reports nothing
        let leaf = |n: &N| TreeNode::leaf(n.1).keyed(ItemKey::text(n.0));
        let t = Tree::new(TREE).node(&leaf);
        let mut index = TreeIndex::default();
        index.sync(&t, &st, &items);
        let mut acc = Acc::<TreeAction>::new();
        assert!(!t.toggle_at(&mut st, &mut index, &items, 0, &mut acc));
        assert!(acc.finish(TREE).action_ref().is_none());
    }

    /// `expand_all` must not name a key per node: it is the inverted
    /// `KeySet`, so it costs nothing on a tree of any size and a single
    /// later `collapse` still names one node.
    #[test]
    fn expand_all_is_the_inverted_key_set() {
        let t = tree();
        let mut st = TreeState::new();
        st.expand_all();
        assert_eq!(
            visible(&t, &FOREST, &st),
            ["alpha", "a1", "a2", "a2x", "beta", "b1", "b2", "b2x"]
        );
        assert!(
            st.expanded().keys().is_empty(),
            "expand_all named a key per node instead of inverting the set"
        );
        st.collapse(ItemKey::text("a2"));
        assert_eq!(
            visible(&t, &FOREST, &st),
            ["alpha", "a1", "a2", "beta", "b1", "b2", "b2x"]
        );
        st.collapse_all();
        assert_eq!(visible(&t, &FOREST, &st), ["alpha", "beta"]);
    }

    #[derive(Clone, Copy)]
    struct CountedNode {
        key: u64,
        depth: u16,
        parent: bool,
    }

    /// The old implementation scanned all 100k source nodes in both phases
    /// and again after every toggle. A structural generation now turns an
    /// ordinary branch toggle into one affected-subtree splice with no
    /// accessor call and no source rebuild.
    #[test]
    fn tree_100k_nodes_flatten() {
        let mut items = Vec::with_capacity(100_000);
        items.push(CountedNode {
            key: 0,
            depth: 0,
            parent: true,
        });
        items.extend((1..100_000).map(|key| CountedNode {
            key,
            depth: 1,
            parent: false,
        }));
        let accesses = Cell::new(0usize);
        let node = |item: &CountedNode| {
            accesses.set(accesses.get().saturating_add(1));
            if item.parent {
                TreeNode::parent(item.depth)
            } else {
                TreeNode::leaf(item.depth)
            }
            .keyed(ItemKey::num(item.key))
        };
        let tree = Tree::new(TREE).node(&node);
        let mut state = TreeState::new();
        let mut index = TreeIndex::default();

        index.sync(&tree, &state, &items);
        assert_eq!(accesses.get(), items.len());
        assert_eq!(index.visible.len(), 1);
        accesses.set(0);

        state.expand(ItemKey::num(0));
        index.sync(&tree, &state, &items);
        assert_eq!(accesses.get(), 0, "expand rescanned the borrowed source");
        assert_eq!(index.visible.len(), items.len());
        state.collapse(ItemKey::num(0));
        index.sync(&tree, &state, &items);
        assert_eq!(accesses.get(), 0, "collapse rescanned the borrowed source");
        assert_eq!(index.visible.len(), 1);
        assert_eq!(index.source_rebuilds, 1);
        assert_eq!(index.splices, 2);
    }

    /// Update builds the runtime-owned index once. Draw consumes that same
    /// entry and touches only viewport items; a second draw repeats neither
    /// the source walk nor the first phase's work.
    #[test]
    fn tree_100k_nodes_render() {
        use crate::intent::IntentQueue;
        use crate::ui::Cx;
        use crate::ui::cx::FrameServices;

        let mut items = Vec::with_capacity(100_000);
        items.push(CountedNode {
            key: 0,
            depth: 0,
            parent: true,
        });
        items.extend((1..100_000).map(|key| CountedNode {
            key,
            depth: 1,
            parent: false,
        }));
        let node_accesses = Cell::new(0usize);
        let painted = Cell::new(0usize);
        let node = |item: &CountedNode| {
            node_accesses.set(node_accesses.get().saturating_add(1));
            if item.parent {
                TreeNode::parent(item.depth)
            } else {
                TreeNode::leaf(item.depth)
            }
            .keyed(ItemKey::num(item.key))
        };
        let row = |_item: &CountedNode, _ui: &mut RowUi<'_>| {
            painted.set(painted.get().saturating_add(1));
        };
        let tree = Tree::new(TREE).node(&node).row(row);
        let mut state = TreeState::new();
        state.expand(ItemKey::num(0));
        let mut core = UiCore::default();
        let intents = IntentQueue::new();
        let mut services = FrameServices::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        {
            let mut cx = Cx::new(&intents, &mut services, &mut core, &last, &theme, None);
            let _ = tree.update(&mut cx, &mut state, &items);
        }
        assert_eq!(node_accesses.get(), items.len());
        node_accesses.set(0);

        let area = Rect::new(0, 0, 30, 6);
        draw_cached(&tree, &state, &items, area, &mut core);
        assert_eq!(node_accesses.get(), 0, "draw rebuilt update's index");
        assert_eq!(painted.get(), 6);
        painted.set(0);
        draw_cached(&tree, &state, &items, area, &mut core);
        assert_eq!(node_accesses.get(), 0, "unchanged draw scanned the source");
        assert_eq!(painted.get(), 6);
    }

    #[test]
    fn query_revision_rebuilds_once_preserves_ancestors_and_restores_expansion() {
        let matcher_calls = Cell::new(0usize);
        let matcher = |node: &N| {
            matcher_calls.set(matcher_calls.get().saturating_add(1));
            node.0 == "b2x"
        };
        let mut state = TreeState::new();
        state.expand(ItemKey::text("alpha"));
        let queried = tree().query(7, &matcher);
        let mut index = TreeIndex::default();
        index.sync(&queried, &state, &FOREST);
        assert_eq!(
            (0..index.visible.len())
                .filter_map(|display| index.row(display))
                .filter_map(|row| FOREST.get(row.source).map(|node| node.0))
                .collect::<Vec<_>>(),
            ["beta", "b2", "b2x"]
        );
        assert_eq!(matcher_calls.get(), FOREST.len());
        matcher_calls.set(0);
        index.sync(&queried, &state, &FOREST);
        assert_eq!(matcher_calls.get(), 0, "unchanged query reran its matcher");

        let revised = tree().query(8, &matcher);
        index.sync(&revised, &state, &FOREST);
        assert_eq!(matcher_calls.get(), FOREST.len());
        assert_eq!(index.query_rebuilds, 1);

        let unfiltered = tree();
        index.sync(&unfiltered, &state, &FOREST);
        assert_eq!(
            (0..index.visible.len())
                .filter_map(|display| index.row(display))
                .filter_map(|row| FOREST.get(row.source).map(|node| node.0))
                .collect::<Vec<_>>(),
            ["alpha", "a1", "a2", "beta"]
        );
        assert!(state.is_expanded(ItemKey::text("alpha")));
    }

    #[test]
    fn query_navigation_reveals_results_without_mutating_expansion() {
        let matcher = |node: &N| node.0 == "b2x";
        let tree = tree().query(1, &matcher);
        let mut state = TreeState::new();
        let before = state.expanded().clone();
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &FOREST);
        let mut acc = Acc::<TreeAction>::new();

        tree.expand_or_descend(&mut state, &mut index, &FOREST, 0, &mut acc);
        assert_eq!(state.cursor(), Some(ItemKey::text("b2")));
        tree.collapse_or_parent(&mut state, &mut index, &FOREST, 2, &mut acc);
        assert_eq!(state.cursor(), Some(ItemKey::text("b2")));
        let _ = tree.toggle_at(&mut state, &mut index, &FOREST, 0, &mut acc);
        assert_eq!(state.expanded(), &before);
        assert_eq!(
            tree.table_for(index.foldable && !index.query_active),
            &TABLE
        );
    }

    #[test]
    fn query_preserves_chosen_until_the_source_removes_it() {
        let matcher = |node: &N| node.0 == "b2x";
        let tree = tree().query(1, &matcher);
        let mut state = TreeState::new();
        state.choose(Some(ItemKey::text("a1")));
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &FOREST);
        let _ = tree.reconcile(&mut state, &FOREST, &index);
        assert_eq!(state.chosen(), Some(ItemKey::text("a1")));

        let without_a1 = [
            FOREST[0], FOREST[2], FOREST[3], FOREST[4], FOREST[5], FOREST[6], FOREST[7],
        ];
        state.invalidate();
        index.sync(&tree, &state, &without_a1);
        let _ = tree.reconcile(&mut state, &without_a1, &index);
        assert_eq!(state.chosen(), None);
    }

    #[test]
    fn matching_lazy_node_can_request_children_during_query() {
        let items = [N("db", 0, false)];
        let lazy = |node: &N| TreeNode::lazy(node.1).keyed(ItemKey::text(node.0));
        let matcher = |_node: &N| true;
        let tree = Tree::new(TREE).node(&lazy).query(1, &matcher);
        let mut state = TreeState::new();
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &items);
        let mut acc = Acc::<TreeAction>::new();

        assert!(tree.toggle_at(&mut state, &mut index, &items, 0, &mut acc));
        assert_eq!(
            acc.finish(TREE).action_ref(),
            Some(&TreeAction::Expanded(ItemKey::text("db")))
        );
    }

    #[test]
    fn source_invalidation_rebuilds_once_and_cache_clear_drops_only_derived_state() {
        let accesses = Cell::new(0usize);
        let counted_node = |item: &N| {
            accesses.set(accesses.get().saturating_add(1));
            node(item)
        };
        let tree = Tree::new(TREE).node(&counted_node);
        let mut state = TreeState::new();
        state.expand(ItemKey::text("alpha"));
        state.choose(Some(ItemKey::text("a1")));
        let mut core = UiCore::default();
        let area = Rect::new(0, 0, 30, 6);

        draw_cached(&tree, &state, &FOREST, area, &mut core);
        assert_eq!(accesses.get(), FOREST.len());
        accesses.set(0);
        draw_cached(&tree, &state, &FOREST, area, &mut core);
        assert_eq!(accesses.get(), 0);
        state.invalidate();
        draw_cached(&tree, &state, &FOREST, area, &mut core);
        assert_eq!(accesses.get(), FOREST.len());
        accesses.set(0);
        core.clear_caches();
        draw_cached(&tree, &state, &FOREST, area, &mut core);
        assert_eq!(accesses.get(), FOREST.len());
        assert!(state.is_expanded(ItemKey::text("alpha")));
        assert_eq!(state.chosen(), Some(ItemKey::text("a1")));
    }

    #[test]
    fn saturated_source_generation_never_reuses_a_stale_index() {
        let accesses = Cell::new(0usize);
        let counted_node = |item: &N| {
            accesses.set(accesses.get().saturating_add(1));
            node(item)
        };
        let tree = Tree::new(TREE).node(&counted_node);
        let original = [N("alpha", 0, false), N("beta", 0, false)];
        let reordered = [original[1], original[0]];
        let mut state = TreeState::new();
        state.source_generation = u64::MAX;
        let mut index = TreeIndex::default();

        index.sync(&tree, &state, &original);
        accesses.set(0);
        state.invalidate();
        index.sync(&tree, &state, &reordered);
        assert_eq!(accesses.get(), reordered.len());
        assert_eq!(index.row(0).map(|row| row.key), Some(ItemKey::text("beta")));

        accesses.set(0);
        state.invalidate();
        index.sync(&tree, &state, &original);
        assert_eq!(accesses.get(), original.len());
        assert_eq!(
            index.row(0).map(|row| row.key),
            Some(ItemKey::text("alpha"))
        );
    }

    #[test]
    fn saturated_expansion_generation_rebuilds_after_every_mutation() {
        let tree = tree();
        let alpha = ItemKey::text("alpha");
        let mut state = TreeState::new();
        state.expand_generation = u64::MAX;
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &FOREST);
        let initial_rebuilds = index.visible_rebuilds;

        state.expand(alpha);
        index.sync(&tree, &state, &FOREST);
        assert_eq!(index.visible.len(), 4);
        assert_eq!(index.visible_rebuilds, initial_rebuilds.saturating_add(1));

        assert!(!state.toggle(alpha));
        index.sync(&tree, &state, &FOREST);
        assert_eq!(index.visible.len(), 2);
        assert_eq!(index.visible_rebuilds, initial_rebuilds.saturating_add(2));

        state.expand_all();
        index.sync(&tree, &state, &FOREST);
        assert_eq!(index.visible.len(), FOREST.len());
        assert_eq!(index.visible_rebuilds, initial_rebuilds.saturating_add(3));

        state.collapse_all();
        index.sync(&tree, &state, &FOREST);
        assert_eq!(index.visible.len(), 2);
        assert_eq!(index.visible_rebuilds, initial_rebuilds.saturating_add(4));
    }

    #[test]
    fn parts_include_the_component_owned_marker_in_paint_order() {
        assert_eq!(
            Tree::<N>::PARTS,
            &[
                Part::CONTAINER,
                Part::GUTTER,
                Part::ICON,
                Part::MARKER,
                Part::LABEL,
                Part::TRACK,
                Part::THUMB,
                Part::EMPTY,
            ]
        );
    }

    #[test]
    fn mono_chosen_row_paints_marker_without_moving_the_label() {
        let items = [N("item", 0, false)];
        let mut state = TreeState::new();
        state.choose(Some(ItemKey::text("item")));
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let chosen = theme.design.glyphs.get(GlyphRole::Chosen);
        let buffer = render(theme, &tree(), &state, &items);

        assert_eq!(
            buffer.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some(chosen)
        );
        assert_eq!(
            buffer.cell(Position::new(3, 0)).map(BufferCell::symbol),
            Some("i")
        );
    }

    #[test]
    fn chosen_marker_follows_key_across_reorder() {
        let items = [N("alpha", 0, false), N("beta", 0, false)];
        let reordered = [items[1], items[0]];
        let mut state = TreeState::new();
        state.choose(Some(ItemKey::text("beta")));
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let chosen = theme.design.glyphs.get(GlyphRole::Chosen);
        let buffer = render(theme, &tree(), &state, &reordered);

        assert_eq!(
            buffer.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some(chosen)
        );
        assert_eq!(
            buffer.cell(Position::new(2, 1)).map(BufferCell::symbol),
            Some(" ")
        );
    }

    #[test]
    fn chosen_marker_comes_only_from_semantic_key_and_survives_reorder() {
        let items = [N("alpha", 0, false), N("beta", 0, false)];
        let reordered = [items[1], items[0]];
        let empty = TreeState::new();
        let mut state = TreeState::new();
        state.set_cursor(0, ItemKey::text("alpha"));
        state.choose(Some(ItemKey::text("beta")));
        let theme = Theme::junie().downgrade(ColorLevel::Mono);
        let chosen = theme.design.glyphs.get(GlyphRole::Chosen);
        let tree = tree();

        let marker_rows = |buffer: &Buffer| {
            (0..2)
                .filter(|y| {
                    buffer
                        .cell(Position::new(2, *y))
                        .is_some_and(|cell| cell.symbol() == chosen)
                })
                .collect::<Vec<_>>()
        };
        let unselected = render(theme.clone(), &tree, &empty, &items);
        assert_eq!(marker_rows(&unselected), []);

        let original = render(theme.clone(), &tree, &state, &items);
        assert_eq!(marker_rows(&original), [1]);

        let moved = render(theme, &tree, &state, &reordered);
        assert_eq!(marker_rows(&moved), [0]);
    }

    #[test]
    fn reference_focus_styles_only_the_runtime_cursor_without_selecting_it() {
        let items = [N("alpha", 0, false), N("beta", 0, false)];
        let flags = RefCell::new(Vec::new());
        let row = |_item: &N, row: &mut RowUi<'_>| {
            flags.borrow_mut().push((row.key(), row.flags()));
        };
        let tree = tree().row(&row);
        let mut state = TreeState::new();
        state.set_cursor(0, ItemKey::text("alpha"));
        let area = Rect::new(0, 0, 24, 2);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, area| {
            ui.reference(
                Some(crate::ReferenceTarget::new(
                    TREE,
                    crate::ReferenceState::FOCUSED,
                )),
                |ui| tree.draw(ui, area, &state, &items),
            );
        });

        assert_eq!(
            flags.borrow().as_slice(),
            [
                (ItemKey::text("alpha"), StateFlags::FOCUSED),
                (ItemKey::text("beta"), StateFlags::empty()),
            ]
        );
    }

    #[test]
    fn choose_action_uses_the_reordered_nodes_stable_key() {
        let items = [N("alpha", 0, false), N("beta", 0, false)];
        let reordered = [items[1], items[0]];
        let tree = tree();
        let mut state = TreeState::new();
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &reordered);
        let mut acc = Acc::<TreeAction>::new();

        tree.engage(&mut state, &mut index, &reordered, 0, false, &mut acc);

        assert_eq!(
            acc.finish(TREE).action_ref(),
            Some(&TreeAction::Chose(ItemKey::text("beta")))
        );
        assert_eq!(state.chosen(), Some(ItemKey::text("beta")));
    }

    #[test]
    fn marker_honours_clear_and_slot_replacement() {
        let items = [N("item", 0, false)];
        let mut state = TreeState::new();
        state.choose(Some(ItemKey::text("item")));
        let clear = [(
            Part::MARKER,
            StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        )];
        let cleared = render(Theme::junie(), &tree().patch_part(&clear), &state, &items);
        assert_eq!(
            cleared.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some(" ")
        );

        let replacement = |ui: &mut Ui<'_>, area: Rect| {
            let style = ui.surface_style();
            ui.paint_str(area, "#", style);
        };
        let replaced = render(
            Theme::junie(),
            &tree().slot(Part::MARKER, &replacement),
            &state,
            &items,
        );
        assert_eq!(
            replaced.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some("#")
        );
    }

    #[test]
    fn unselected_marker_is_blank_and_skips_theme_and_slot_resolution() {
        let items = [N("item", 0, false)];
        let state = TreeState::new();
        let set = [(
            Part::MARKER,
            StylePatch {
                glyph: Slot::Set(GlyphRole::Chosen),
                ..StylePatch::new()
            },
        )];
        let themed = render(Theme::junie(), &tree().patch_part(&set), &state, &items);
        assert_eq!(
            themed.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some(" ")
        );

        let calls = Cell::new(0usize);
        let replacement = |ui: &mut Ui<'_>, area: Rect| {
            calls.set(calls.get().saturating_add(1));
            let style = ui.surface_style();
            ui.paint_str(area, "#", style);
        };
        let slotted = render(
            Theme::junie(),
            &tree().slot(Part::MARKER, &replacement),
            &state,
            &items,
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            slotted.cell(Position::new(2, 0)).map(BufferCell::symbol),
            Some(" ")
        );
    }

    #[test]
    fn owner_patches_reach_tree_parts_without_leaking_into_row_owned_parts() {
        let bold = StylePatch::new().add(Modifier::BOLD);
        let custom = Part::custom("tree.tests.custom");
        let owner_parts = [
            (Part::CONTAINER, StylePatch::new().set_bg(Role::Danger)),
            (Part::GUTTER, bold),
            (Part::ICON, bold),
            (Part::MARKER, bold),
            (Part::LABEL, bold),
            (Part::META, bold),
            (Part::CELL, bold),
            (custom, bold),
        ];
        let row = move |_item: &N, row: &mut RowUi<'_>| {
            row.meta("m");
            row.part(custom, 1).text("x");
            row.part(Part::CELL, 1).text("c");
            row.label("L");
        };
        let tree = tree().row(row).patch_part(&owner_parts);
        let items = [N("item", 0, false)];
        let mut state = TreeState::new();
        state.choose(Some(ItemKey::text("item")));
        let area = Rect::new(0, 0, 40, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            tree.draw(ui, rect, &state, &items);
        });

        for x in [0, 1, 2, 3] {
            assert!(
                buffer
                    .cell(Position::new(x, 0))
                    .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
                "component-owned cell {x} dropped its patch"
            );
        }
        assert!(
            buffer
                .cell(Position::new(4, 0))
                .is_some_and(|cell| cell.bg == Theme::junie().color.danger),
            "CONTAINER patch was overwritten by the automatic RowUi fill"
        );
        for symbol in ["m", "x", "c"] {
            assert!(
                buffer
                    .content()
                    .iter()
                    .find(|cell| cell.symbol() == symbol)
                    .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD)),
                "owner patch leaked into row-owned {symbol}"
            );
        }
    }
}
