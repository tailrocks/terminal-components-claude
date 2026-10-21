use core::fmt;
use core::marker::PhantomData;

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at};
use crate::measure::{Constraints, Size};
use crate::scroll::ScrollState;
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, KeySet, Reconcile, Reconciliation,
};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_runtime::empty::EmptyState;
use termrock_runtime::event::{Chord, KeyCode};
use termrock_runtime::focus::Focusability;
use termrock_runtime::intent::{Intent, Phase};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;
use termrock_runtime::ui::{Cx, FrameRead, Ui};
use termrock_theme::{Family, GlyphRole, Slot, StylePatch, Variant};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NodeKind {
    #[default]
    Leaf,
    Parent,
    Lazy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeBranchActivation {
    #[default]
    Toggle,
    Activate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeBranchClick {
    #[default]
    Toggle,
    Choose,
}

#[derive(Clone, Copy)]
enum Engagement {
    Activate,
    Choose,
    Click,
    SelectedClick,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TreeNode {
    depth: u16,
    kind: NodeKind,
    key: Option<ItemKey>,
}

impl TreeNode {
    #[must_use]
    pub const fn leaf(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Leaf,
            key: None,
        }
    }

    #[must_use]
    pub const fn parent(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Parent,
            key: None,
        }
    }

    #[must_use]
    pub const fn lazy(depth: u16) -> Self {
        TreeNode {
            depth,
            kind: NodeKind::Lazy,
            key: None,
        }
    }

    #[must_use]
    pub const fn keyed(mut self, k: ItemKey) -> Self {
        self.key = Some(k);
        self
    }

    #[must_use]
    pub const fn depth(self) -> u16 {
        self.depth
    }

    #[must_use]
    pub const fn kind(self) -> NodeKind {
        self.kind
    }

    #[must_use]
    pub const fn key(self) -> Option<ItemKey> {
        self.key
    }

    #[must_use]
    pub const fn has_children(self) -> bool {
        matches!(self.kind, NodeKind::Parent | NodeKind::Lazy)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeAction {
    Moved,
    Expanded(ItemKey),
    Collapsed(ItemKey),
    Chose(ItemKey),
    Activated(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Expand,
    Collapse,
    Activate,
    Choose,
    ExpandAll,
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

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TreeState {
    core: CollectionCore,
    source_lineage: Option<Arc<()>>,
    expansion_lineage: Option<Arc<()>>,
    expanded: KeySet,
    chosen: Option<ItemKey>,

    click_cursor: Option<ItemKey>,
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
    #[must_use]
    pub const fn new() -> Self {
        TreeState {
            core: CollectionCore::new(),
            source_lineage: None,
            expansion_lineage: None,
            expanded: KeySet::new(),
            chosen: None,
            click_cursor: None,
            expand_generation: 0,
            source_generation: 0,
            expand_generation_saturated: false,
            source_generation_saturated: false,
            last_expansion: None,
        }
    }

    #[must_use]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    #[must_use]
    pub const fn chosen(&self) -> Option<ItemKey> {
        self.chosen
    }

    pub const fn choose(&mut self, key: Option<ItemKey>) {
        self.chosen = key;
    }

    #[must_use]
    pub const fn expanded(&self) -> &KeySet {
        &self.expanded
    }

    #[must_use]
    pub fn is_expanded(&self, key: ItemKey) -> bool {
        self.expanded.contains(key)
    }

    pub fn expand(&mut self, key: ItemKey) {
        if !self.expanded.contains(key) {
            Self::prepare_lineage(&mut self.expansion_lineage);
            self.expanded.insert(key);
            self.record_expansion(key, true);
        }
    }

    pub fn collapse(&mut self, key: ItemKey) {
        if self.expanded.contains(key) {
            Self::prepare_lineage(&mut self.expansion_lineage);
            self.expanded.remove(key);
            self.record_expansion(key, false);
        }
    }

    pub fn toggle(&mut self, key: ItemKey) -> bool {
        Self::prepare_lineage(&mut self.expansion_lineage);
        self.expanded.toggle(key);
        let expanded = self.expanded.contains(key);
        self.record_expansion(key, expanded);
        expanded
    }

    pub fn expand_all(&mut self) {
        Self::prepare_lineage(&mut self.expansion_lineage);
        self.expanded.all();
        self.bump_expand_generation();
        self.last_expansion = None;
    }

    pub fn collapse_all(&mut self) {
        if !self.expanded.is_empty() {
            Self::prepare_lineage(&mut self.expansion_lineage);
            self.expanded.none();
            self.bump_expand_generation();
            self.last_expansion = None;
        }
    }

    #[must_use]
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    pub fn invalidate(&mut self) {
        Self::prepare_lineage(&mut self.source_lineage);
        if let Some(next) = self.source_generation.checked_add(1) {
            self.source_generation = next;
        } else {
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
            self.expand_generation_saturated = true;
        }
    }

    fn prepare_lineage(lineage: &mut Option<Arc<()>>) {
        if lineage
            .as_ref()
            .is_none_or(|token| Arc::strong_count(token) > 1)
        {
            *lineage = Some(Arc::new(()));
        }
    }

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

pub struct Tree<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    render_row: Option<TreeRowRenderer<'a, T>>,
    node: Option<&'a dyn Fn(&T) -> TreeNode>,
    icon: Option<TreeIconFn<'a, T>>,
    trailing_mark: Option<TreeTrailingMarkFn<'a, T>>,
    focused_patch: Option<&'a StylePatch>,
    branch_activation: TreeBranchActivation,
    branch_click: TreeBranchClick,
    activate_selected_on_click: bool,
    gutter_gap: u16,
    disclosure_hit_width: u16,
    cursor_selected: bool,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    query: Option<TreeQuery<'a, T>>,
    disabled: bool,
    empty: Option<EmptyState<'a>>,
    ov: PartStyle<'a>,
    fwd_patch: Option<&'a StylePatch>,
    fwd_parts: &'a [(Part, StylePatch)],
    fwd_slot: Option<(Part, SlotFn<'a>)>,
    _t: PhantomData<fn(&T)>,
}

pub type TreeIconFn<'a, T> = &'a dyn Fn(&T) -> Option<(&'static str, StylePatch)>;

pub type TreeTrailingMarkFn<'a, T> = &'a dyn Fn(&T) -> Option<(GlyphRole, StylePatch)>;

type TreeRowRenderer<'a, T> = &'a dyn Fn(&mut Ui<'_>, Rect, StateFlags, ItemKey, &T);

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
    #[must_use]
    pub const fn new(id: Id) -> Self {
        Tree {
            id,
            key: ByIndex,
            row: DefaultRow,
            render_row: None,
            node: None,
            icon: None,
            trailing_mark: None,
            focused_patch: None,
            branch_activation: TreeBranchActivation::Toggle,
            branch_click: TreeBranchClick::Toggle,
            activate_selected_on_click: false,
            gutter_gap: 0,
            disclosure_hit_width: 1,
            cursor_selected: false,
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

    pub const PREFERRED_WIDTH: u16 = 28;

    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn node(mut self, f: &'a dyn Fn(&T) -> TreeNode) -> Self {
        self.node = Some(f);
        self
    }

    #[must_use]
    pub const fn icon(mut self, f: TreeIconFn<'a, T>) -> Self {
        self.icon = Some(f);
        self
    }

    #[must_use]
    pub const fn trailing_mark(mut self, f: TreeTrailingMarkFn<'a, T>) -> Self {
        self.trailing_mark = Some(f);
        self
    }

    #[must_use]
    pub const fn focused_patch(mut self, patch: &'a StylePatch) -> Self {
        self.focused_patch = Some(patch);
        self
    }

    #[must_use]
    pub const fn branch_activation(mut self, policy: TreeBranchActivation) -> Self {
        self.branch_activation = policy;
        self
    }

    #[must_use]
    pub const fn branch_click(mut self, policy: TreeBranchClick) -> Self {
        self.branch_click = policy;
        self
    }

    #[must_use]
    pub const fn activate_selected_on_click(mut self, enabled: bool) -> Self {
        self.activate_selected_on_click = enabled;
        self
    }

    #[must_use]
    pub const fn gutter_gap(mut self, cells: u16) -> Self {
        self.gutter_gap = cells;
        self
    }

    #[must_use]
    pub const fn disclosure_hit_width(mut self, cells: u16) -> Self {
        self.disclosure_hit_width = cells;
        self
    }

    #[must_use]
    pub const fn cursor_selected(mut self, enabled: bool) -> Self {
        self.cursor_selected = enabled;
        self
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tree<'a, T, K2, R> {
        Tree {
            id: self.id,
            key: k,
            row: self.row,
            render_row: self.render_row,
            node: self.node,
            icon: self.icon,
            trailing_mark: self.trailing_mark,
            focused_patch: self.focused_patch,
            branch_activation: self.branch_activation,
            branch_click: self.branch_click,
            activate_selected_on_click: self.activate_selected_on_click,
            gutter_gap: self.gutter_gap,
            disclosure_hit_width: self.disclosure_hit_width,
            cursor_selected: self.cursor_selected,
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

    #[must_use]
    pub fn render_row(self, renderer: TreeRowRenderer<'a, T>) -> Tree<'a, T, K, impl RowFn<T>> {
        let mut tree = self.row(|_: &T, _: &mut RowUi<'_>| {});
        tree.render_row = Some(renderer);
        tree
    }

    /// Set a row painter closure.
    ///
    /// Keep the `Fn` bound here (rather than only `RowFn`) so the item type
    /// remains inferable for existing unannotated closure consumers.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tree<'a, T, K, R2> {
        self.row_adapter(r)
    }

    /// Set a reusable row adapter implementing [`RowFn`].
    ///
    /// Use this for semantic row types such as `SemanticItemRow`; closures
    /// should normally use [`Tree::row`] to preserve their inferred item type.
    pub fn row_adapter<R2: RowFn<T>>(self, r: R2) -> Tree<'a, T, K, R2> {
        Tree {
            id: self.id,
            key: self.key,
            row: r,
            render_row: self.render_row,
            node: self.node,
            icon: self.icon,
            trailing_mark: self.trailing_mark,
            focused_patch: self.focused_patch,
            branch_activation: self.branch_activation,
            branch_click: self.branch_click,
            activate_selected_on_click: self.activate_selected_on_click,
            gutter_gap: self.gutter_gap,
            disclosure_hit_width: self.disclosure_hit_width,
            cursor_selected: self.cursor_selected,
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

    #[must_use]
    pub const fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    #[must_use]
    pub const fn query(mut self, revision: u64, matcher: &'a dyn Fn(&T) -> bool) -> Self {
        self.query = Some(TreeQuery { revision, matcher });
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self.fwd_patch = Some(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self.fwd_parts = ps;
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self.fwd_slot = Some((p, f));
        self
    }

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

#[derive(Clone, Copy, Debug)]
struct FlatRef {
    source: usize,
    key: ItemKey,
    depth: u16,
    kind: NodeKind,
    included: bool,
}

#[derive(Default)]
struct TreeIndex {
    initialized: bool,
    source_lineage: Option<Weak<()>>,
    expansion_lineage: Option<Weak<()>>,
    source_generation: u64,
    expand_generation: u64,
    query_revision: Option<u64>,
    query_active: bool,
    source_len: usize,
    rows: Vec<FlatRef>,
    by_key: HashMap<ItemKey, usize>,
    visible: Vec<usize>,
    splice_scratch: Vec<usize>,
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
    fn same_lineage(cached: Option<&Weak<()>>, current: Option<&Arc<()>>) -> bool {
        match (cached, current) {
            (None, None) => true,
            (Some(cached), Some(current)) => core::ptr::eq(cached.as_ptr(), Arc::as_ptr(current)),
            _ => false,
        }
    }

    fn sync<T, K: KeyFn<T>, R>(
        &mut self,
        tree: &Tree<'_, T, K, R>,
        state: &TreeState,
        items: &[T],
    ) {
        let query_revision = tree.query.as_ref().map(|query| query.revision);
        let source_changed = !self.initialized
            || !Self::same_lineage(self.source_lineage.as_ref(), state.source_lineage.as_ref())
            || self.source_generation != state.source_generation
            || state.source_generation_saturated
            || self.source_len != items.len();
        if source_changed {
            self.rebuild_source(tree, state, items, query_revision);
            return;
        }
        let same_expansion = Self::same_lineage(
            self.expansion_lineage.as_ref(),
            state.expansion_lineage.as_ref(),
        );

        let from_empty = self.expansion_lineage.is_none() && self.expand_generation == 0;
        if !same_expansion {
            self.expansion_lineage = state.expansion_lineage.as_ref().map(Arc::downgrade);
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
        if same_expansion && self.expand_generation == state.expand_generation {
            return;
        }
        if self.query_active {
            self.expand_generation = state.expand_generation;
            return;
        }
        let incremental = (same_expansion || from_empty)
            && self.expand_generation.saturating_add(1) == state.expand_generation
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

        self.source_lineage = state.source_lineage.as_ref().map(Arc::downgrade);
        self.expansion_lineage = state.expansion_lineage.as_ref().map(Arc::downgrade);
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
            self.splice_scratch.clear();
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
                self.splice_scratch.push(index);
                if matches!(row.kind, NodeKind::Parent | NodeKind::Lazy)
                    && !expanded.contains(row.key)
                {
                    collapsed_depth = Some(row.depth);
                }
            }
            self.visible.splice(
                display.saturating_add(1)..display.saturating_add(1),
                self.splice_scratch.drain(..),
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
    prior_cursor: Option<ItemKey>,
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

    fn engage(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        d: usize,
        engagement: Engagement,
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
        let toggles_branch = match engagement {
            Engagement::Activate => self.branch_activation == TreeBranchActivation::Toggle,
            Engagement::Choose => true,
            Engagement::Click => self.branch_click == TreeBranchClick::Toggle,
            Engagement::SelectedClick => false,
        };
        if self.node_of(it).has_children() && toggles_branch {
            let _ = self.toggle_at(st, index, items, d, acc);
            return;
        }
        let key = row.key;
        st.chosen = Some(key);
        acc.action(
            if matches!(engagement, Engagement::Activate | Engagement::SelectedClick) {
                TreeAction::Activated(key)
            } else {
                TreeAction::Chose(key)
            },
        );
    }

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

    fn command(
        &self,
        st: &mut TreeState,
        index: &mut TreeIndex,
        items: &[T],
        command: TreeCmd,
        viewport: usize,
        acc: &mut Acc<TreeAction>,
    ) {
        let cur = st.core.cursor_index();
        match command {
            TreeCmd::Up => {
                Self::move_to(st, index, cur.saturating_sub(1), acc);
            }
            TreeCmd::Down => {
                Self::move_to(st, index, cur.saturating_add(1), acc);
            }
            TreeCmd::PageUp => {
                Self::move_to(st, index, cur.saturating_sub(viewport), acc);
            }
            TreeCmd::PageDown => {
                Self::move_to(st, index, cur.saturating_add(viewport), acc);
            }
            TreeCmd::Home => Self::move_to(st, index, 0, acc),
            TreeCmd::End => Self::move_to(st, index, usize::MAX, acc),
            TreeCmd::Expand => {
                self.expand_or_descend(st, index, items, cur, acc);
            }
            TreeCmd::Collapse => {
                self.collapse_or_parent(st, index, items, cur, acc);
            }
            TreeCmd::Activate => {
                self.engage(st, index, items, cur, Engagement::Activate, acc);
            }
            TreeCmd::Choose => {
                self.engage(st, index, items, cur, Engagement::Choose, acc);
            }
            TreeCmd::ExpandAll => {
                if index.query_active {
                    acc.consumed();
                } else {
                    st.expand_all();
                    index.sync(self, st, items);
                    acc.action(TreeAction::Moved);
                }
            }
            TreeCmd::CollapseAll => {
                if index.query_active {
                    acc.consumed();
                } else {
                    st.collapse_all();
                    index.sync(self, st, items);
                    acc.action(TreeAction::Moved);
                }
            }
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TreeState, items: &[T]) -> Response<TreeAction> {
        if self.disabled {
            st.click_cursor = None;
            return Response::ignored();
        }
        let prior_cursor = st.core.cursor();
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
        let mut released = false;
        for it in intents {
            match it {
                Intent::Binding(action) => {
                    st.click_cursor = None;
                    if let Some(command) = Binding::command(table, action) {
                        self.command(st, index, items, command, viewport, &mut acc);
                    }
                }
                Intent::Pointer {
                    phase, part, pos, ..
                } => {
                    released |= phase == Phase::Release;
                    let hint = hint_area.map(|a| {
                        let view = ScrollRegion::view(st.core.scroll(), a, len);
                        view.offset()
                            .saturating_add(usize::from(pos.y.saturating_sub(a.y)))
                    });
                    self.pointer(
                        st,
                        index,
                        items,
                        PointerIntent {
                            phase,
                            prior_cursor,
                            part,
                            hint,
                        },
                        &mut acc,
                    );
                }
                _ => {}
            }
        }
        if released {
            st.click_cursor = None;
        }

        let after = index.visible.len();
        st.core.scroll_mut().set_content(after);
        acc.finish(self.id)
    }

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
        let PointerIntent {
            phase,
            part,
            hint,
            prior_cursor,
        } = pointer;
        if matches!(phase, Phase::DragStart | Phase::Drag | Phase::DragEnd) {
            st.click_cursor = None;
        }
        if phase == Phase::Press && part.part != Part::ROW {
            st.click_cursor = None;
        }
        let Some(key) = part.item else {
            acc.consumed();
            return;
        };
        let Some(d) = index.display_of(key, hint) else {
            st.click_cursor = None;
            acc.consumed();
            return;
        };
        if index
            .row(d)
            .and_then(|row| items.get(row.source))
            .is_none_or(|item| self.is_disabled(item))
        {
            st.click_cursor = None;
            acc.consumed();
            return;
        }
        let fold_is_disclosure = part.part == Part::ICON
            && index
                .row(d)
                .and_then(|row| items.get(row.source))
                .is_some_and(|item| self.node_of(item).has_children());
        if fold_is_disclosure {
            match phase {
                Phase::Press => {
                    st.core.set_cursor(d, key);
                    acc.changed();
                }
                Phase::Click => {
                    st.core.set_cursor(d, key);
                    if !self.toggle_at(st, index, items, d, acc) {
                        acc.changed();
                    }
                }
                _ => acc.consumed(),
            }
            return;
        }
        if part.part == Part::ICON {
            match phase {
                Phase::Press => {
                    st.core.set_cursor(d, key);
                    acc.changed();
                }
                Phase::Click => {
                    st.core.set_cursor(d, key);
                    self.engage(st, index, items, d, Engagement::Choose, acc);
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
                st.click_cursor =
                    prior_cursor.filter(|cursor| self.activate_selected_on_click && *cursor == key);
                st.core.set_cursor(d, key);
                acc.changed();
            }
            Phase::Click | Phase::DoubleClick if self.activate_selected_on_click => {
                let engagement = if st.click_cursor.take() == Some(key) {
                    Engagement::SelectedClick
                } else {
                    Engagement::Click
                };
                self.engage(st, index, items, d, engagement, acc);
            }
            Phase::Click => self.engage(st, index, items, d, Engagement::Click, acc),
            Phase::DoubleClick => self.engage(st, index, items, d, Engagement::Activate, acc),
            _ => acc.consumed(),
        }
    }

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
            let mut row = Self::row_of(
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
            if self.cursor_selected {
                row.flags
                    .set(StateFlags::SELECTED, st.core.cursor() == Some(row.key));
            }
            self.project_pointer_flags(ui, st, live, &mut row);
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
        let inherited = self
            .ov
            .style(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                Part::EMPTY,
                live,
            )
            .style;
        self.empty
            .unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            })
            .draw_inherited(ui, mid, 0, inherited);
    }

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

    fn project_pointer_flags(&self, ui: &Ui<'_>, st: &TreeState, live: StateFlags, row: &mut Row) {
        let matches_row = |part: PartRef| {
            part.item == Some(row.key) && matches!(part.part, Part::ROW | Part::ICON)
        };
        let pressed = ui.pressed_part(self.id);
        row.flags.set(
            StateFlags::HOVERED,
            ui.hovered_part(self.id).is_some_and(matches_row),
        );
        row.flags.set(
            StateFlags::PRESSED,
            !row.flags.contains(StateFlags::DISABLED)
                && (pressed.is_some_and(matches_row)
                    || (pressed.is_none()
                        && st.core.cursor() == Some(row.key)
                        && live.contains(StateFlags::PRESSED))),
        );
    }

    fn paint_row(&self, ui: &mut Ui<'_>, row: Row, indent: u16, item: &T) {
        if let Some(renderer) = self.render_row {
            if !row.rect.intersection(ui.full()).is_empty() {
                ui.with_area(row.rect, |ui| {
                    renderer(ui, row.rect, row.flags, row.key, item);
                });
            }
            self.register_row(ui, row, indent, false);
            return;
        }
        self.paint_default_row(ui, row, indent, item);
    }

    fn row_fill(
        &self,
        ui: &mut Ui<'_>,
        flags: StateFlags,
        focus: Option<StylePatch>,
    ) -> crate::theme::PaintStyle {
        if let Some(focus_patch) = focus {
            let patch = self
                .ov
                .part_patch(Part::CONTAINER)
                .map_or(focus_patch, |part| part.merge(focus_patch));
            ui.style_patched(
                Family::TREE,
                Variant::DEFAULT,
                Part::CONTAINER,
                flags,
                &patch,
            )
            .style
        } else {
            self.ov
                .style(
                    ui,
                    self.id,
                    Family::TREE,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    flags,
                )
                .style
        }
    }

    fn paint_fold(
        &self,
        ui: &mut Ui<'_>,
        row: Row,
        item: &T,
        fold: Rect,
        row_style: crate::theme::PaintStyle,
    ) -> bool {
        let custom_icon = self.icon.and_then(|paint| paint(item));
        let has_custom_icon = custom_icon.is_some();
        if let Some((text, patch)) = custom_icon {
            let style = ui
                .style_patched(
                    Family::TREE,
                    Variant::DEFAULT,
                    Part::ICON,
                    row.flags,
                    &patch,
                )
                .style;

            let style = if matches!(patch.bg, Slot::Inherit) {
                style.with_bg_from(row_style)
            } else {
                style
            };
            ui.paint_str(fold, text, style);
        } else if let Some(f) = self.ov.slot_for(Part::ICON) {
            f(ui, fold);
        } else {
            let has_icon_patch = self.ov.part_patch(Part::ICON).is_some();
            let icon = self.ov.style(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                Part::ICON,
                row.flags,
            );
            match icon.glyph {
                Slot::Set(g) => {
                    ui.glyph(fold, g, icon.style);
                }

                Slot::Inherit if row.disclosure.is_none() && !has_icon_patch => {
                    ui.fill(fold, row_style);
                }

                Slot::Inherit => {
                    if let Some(glyph) = row.disclosure {
                        ui.glyph(fold, glyph, icon.style);
                    } else {
                        ui.fill(fold, icon.style);
                    }
                }
                Slot::Clear => ui.fill(fold, icon.style),
            }
        }
        has_custom_icon
    }

    fn paint_trailing(
        ui: &mut Ui<'_>,
        row: Row,
        role: GlyphRole,
        patch: StylePatch,
        reserve: u16,
        row_style: crate::theme::PaintStyle,
    ) {
        let mark = cell_at(row.rect, row.rect.right().saturating_sub(reserve));
        let style = ui
            .style_patched(
                Family::TREE,
                Variant::DEFAULT,
                Part::CONTAINER,
                row.flags,
                &patch,
            )
            .style;

        let style = if matches!(patch.bg, Slot::Inherit) {
            style.with_bg_from(row_style)
        } else {
            style
        };
        ui.glyph(mark, role, style);
    }

    fn paint_default_row(&self, ui: &mut Ui<'_>, row: Row, indent: u16, item: &T) {
        let focus = row
            .flags
            .contains(StateFlags::FOCUSED)
            .then(|| self.focused_patch.copied())
            .flatten();
        let row_style = self.row_fill(ui, row.flags, focus);
        ui.fill(row.rect, row_style);
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
        let fold_x = self.fold_x(row, indent);
        let fold = cell_at(row.rect, fold_x);
        let has_custom_icon = self.paint_fold(ui, row, item, fold, row_style);
        let marker = cell_at(row.rect, fold_x.saturating_add(1));
        self.paint_marker(ui, row, marker);
        let trailing = self.trailing_mark.and_then(|paint| paint(item));

        let mut rest = Rect {
            x: fold_x.saturating_add(2),
            width: row
                .rect
                .right()
                .saturating_sub(fold_x.saturating_add(2))
                .min(row.rect.width),
            ..row.rect
        };
        let reserve = if trailing.is_some() { 2 } else { 0 };
        if reserve > 0 {
            rest.width = rest.width.saturating_sub(reserve);
        }
        if rest.width > 0 && rest.x < row.rect.right() {
            let mut r = RowUi::new_with_patches(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                row.flags,
                row.key,
                rest,
                merge_focus(self.ov.part_patch(Part::CONTAINER), focus),
                merge_focus(self.ov.part_patch(Part::LABEL), focus),
            );
            self.row.row(item, &mut r);
        }
        if let Some((role, patch)) = trailing {
            Self::paint_trailing(ui, row, role, patch, reserve, row_style);
        }
        if ui.is_inert() {
            return;
        }
        self.register_row(ui, row, indent, has_custom_icon);
    }

    fn fold_x(&self, row: Row, indent: u16) -> u16 {
        row.rect
            .x
            .saturating_add(1)
            .saturating_add(self.gutter_gap)
            .saturating_add(row.depth.saturating_mul(indent))
    }

    fn register_row(&self, ui: &mut Ui<'_>, row: Row, indent: u16, has_custom_icon: bool) {
        ui.register_part(self.id, PartRef::item(Part::ROW, row.key), row.rect);

        if self.disclosure_hit_width > 0 && (row.disclosure.is_some() || !has_custom_icon) {
            let fold = Rect {
                x: self.fold_x(row, indent),
                width: self.disclosure_hit_width,
                ..row.rect
            }
            .intersection(row.rect);
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

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (8, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }
}

fn merge_focus(base: Option<StylePatch>, focus: Option<StylePatch>) -> Option<StylePatch> {
    match (base, focus) {
        (Some(base), Some(focus)) => Some(base.merge(focus)),
        (Some(base), None) => Some(base),
        (None, Some(focus)) => Some(focus),
        (None, None) => None,
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

    use super::{
        Acc, PointerIntent, TABLE, Tree, TreeAction, TreeBranchClick, TreeIconFn, TreeIndex,
        TreeNode, TreeState, TreeTrailingMarkFn,
    };
    use crate::Phase;
    use termrock_core::id::{Id, ItemKey, Part, PartRef};
    use termrock_core::response::StateFlags;
    use termrock_runtime::Runtime;
    use termrock_runtime::stub::Stub;
    use termrock_theme::{ColorLevel, GlyphRole, Modifier, Role, Slot, StylePatch, Theme};

    use termrock_runtime::rowui::RowUi;
    use termrock_runtime::ui::cx::LastFrame;
    use termrock_runtime::ui::{FrameState, Ui, UiCore};

    const TREE: Id = Id::root("tree.tests");

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

    fn visible(t: &Tree<'_, N>, items: &[N], st: &TreeState) -> Vec<&'static str> {
        let mut index = TreeIndex::default();
        index.sync(t, st, items);
        (0..index.visible.len())
            .filter_map(|d| index.row(d))
            .filter_map(|row| items.get(row.source).map(|n| n.0))
            .collect()
    }

    #[test]
    fn selected_click_without_prior_cursor_only_chooses() {
        let t = tree()
            .activate_selected_on_click(true)
            .branch_click(TreeBranchClick::Choose);
        let items = [N("leaf", 0, false)];
        let mut st = TreeState::new();
        let mut index = TreeIndex::default();
        index.sync(&t, &st, &items);
        let key = ItemKey::text("leaf");
        let mut acc = Acc::new();
        for phase in [Phase::Press, Phase::Release, Phase::Click] {
            t.pointer(
                &mut st,
                &mut index,
                &items,
                PointerIntent {
                    phase,
                    prior_cursor: None,
                    part: PartRef::item(Part::ROW, key),
                    hint: Some(0),
                },
                &mut acc,
            );
        }
        assert_eq!(acc.finish(TREE).action_ref(), Some(&TreeAction::Chose(key)));
    }

    fn phases(tree: &Tree<'_, N>, st: &mut TreeState, items: &[N], part: PartRef) -> TreeAction {
        let mut index = TreeIndex::default();
        index.sync(tree, st, items);
        let mut acc = Acc::new();
        for phase in [Phase::Press, Phase::Release, Phase::Click] {
            tree.pointer(
                st,
                &mut index,
                items,
                PointerIntent {
                    phase,
                    prior_cursor: None,
                    part,
                    hint: Some(0),
                },
                &mut acc,
            );
        }
        acc.finish(TREE)
            .action_ref()
            .copied()
            .expect("pointer click emits an action")
    }

    #[test]
    fn branch_disclosure_click_never_chooses_or_activates() {
        let t = tree();
        let items = [N("branch", 0, true), N("leaf", 1, false)];
        let mut st = TreeState::new();
        let key = ItemKey::text("branch");
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                t.draw(ui, area, &st, &items);
            })
            .commit_presented();
        let fold = runtime
            .registry()
            .hit(Position::new(1, 0))
            .expect("fold hit");
        assert_eq!(fold.part, PartRef::item(Part::ICON, key));

        let action = phases(&t, &mut st, &items, fold.part);
        assert_eq!(action, TreeAction::Expanded(key));
        assert_ne!(action, TreeAction::Chose(key));
        assert_ne!(action, TreeAction::Activated(key));
        assert!(st.is_expanded(key));
        assert_eq!(st.chosen(), None);
    }

    #[test]
    fn leaf_fold_cell_click_sets_cursor_and_chooses() {
        let t = tree();
        let items = [N("leaf", 0, false)];
        let mut st = TreeState::new();
        let key = ItemKey::text("leaf");
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                t.draw(ui, area, &st, &items);
            })
            .commit_presented();
        let fold = runtime
            .registry()
            .hit(Position::new(1, 0))
            .expect("fold hit");
        assert_eq!(fold.part, PartRef::item(Part::ICON, key));

        assert_eq!(st.cursor(), None);
        let action = phases(&t, &mut st, &items, fold.part);
        assert_eq!(action, TreeAction::Chose(key));
        assert_ne!(action, TreeAction::Activated(key));
        assert_eq!(st.cursor(), Some(key));
        assert_eq!(st.chosen(), Some(key));
    }

    #[test]
    fn leaf_fold_cell_registers_over_the_row() {
        let t = tree();
        let items = [N("leaf", 0, false)];
        let st = TreeState::new();
        let key = ItemKey::text("leaf");
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                t.draw(ui, area, &st, &items);
            })
            .commit_presented();

        let fold = runtime
            .registry()
            .hit(Position::new(1, 0))
            .expect("fold hit");
        assert_eq!(fold.owner, TREE);
        assert_eq!(fold.part, PartRef::item(Part::ICON, key));
        let body = runtime
            .registry()
            .hit(Position::new(2, 0))
            .expect("body hit");
        assert_eq!(body.owner, TREE);
        assert_eq!(body.part, PartRef::item(Part::ROW, key));
    }

    #[test]
    fn custom_leaf_icon_click_is_row_body() {
        let icon = |_: &N| Some(("*", StylePatch::new()));
        let t = Tree::new(TREE).node(&node).icon(&icon);
        let items = [N("leaf", 0, false)];
        let mut st = TreeState::new();
        let key = ItemKey::text("leaf");
        let area = Rect::new(0, 0, 12, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                t.draw(ui, area, &st, &items);
            })
            .commit_presented();

        let hit = runtime
            .registry()
            .hit(Position::new(1, 0))
            .expect("body hit");
        assert_eq!(hit.owner, TREE);
        assert_eq!(hit.part, PartRef::item(Part::ROW, key));
        assert_ne!(hit.part.part, Part::ICON);

        let action = phases(&t, &mut st, &items, hit.part);
        assert_eq!(action, TreeAction::Chose(key));
        assert_ne!(action, TreeAction::Activated(key));
        assert!(!st.is_expanded(key));
        assert_eq!(st.cursor(), Some(key));
        assert_eq!(st.chosen(), Some(key));
    }

    #[test]
    fn icon_trailing_mark_and_focused_patch_affect_paint() {
        fn paint_icon(item: &N) -> Option<(&'static str, StylePatch)> {
            (item.0 == "leaf").then_some(("◉", StylePatch::new().set_fg(Role::Accent)))
        }
        fn paint_mark(item: &N) -> Option<(GlyphRole, StylePatch)> {
            (item.0 == "leaf")
                .then_some((GlyphRole::Dirty, StylePatch::new().set_fg(Role::Warning)))
        }
        let icon: TreeIconFn<'_, N> = &paint_icon;
        let trailing: TreeTrailingMarkFn<'_, N> = &paint_mark;
        let focus = StylePatch::new().set_bg(Role::Danger);
        let themed = Tree::new(TREE)
            .node(&node)
            .icon(icon)
            .trailing_mark(trailing)
            .focused_patch(&focus);
        let plain = Tree::new(TREE).node(&node);
        let items = [N("parent", 0, true), N("leaf", 1, false)];
        let mut state = TreeState::new();
        let parent = ItemKey::text("parent");
        state.expand(parent);
        state.set_cursor(0, parent);
        let theme = Theme::junie();
        let expanded = theme.design.glyphs.get(GlyphRole::Expanded);
        let dirty = theme.design.glyphs.get(GlyphRole::Dirty);
        let area = Rect::new(0, 0, 24, 2);
        let painted = draw_focused(theme.clone(), &themed, &state, &items, area);
        let unpatched = draw_focused(theme.clone(), &plain, &state, &items, area);

        assert_eq!(
            painted.cell(Position::new(1, 0)).map(BufferCell::symbol),
            Some(expanded),
            "parent icon None keeps the disclosure"
        );
        assert_eq!(
            painted.cell(Position::new(3, 1)).map(BufferCell::symbol),
            Some("◉"),
            "a leaf icon paints the provided glyph"
        );
        assert_eq!(
            painted.cell(Position::new(3, 1)).map(|cell| cell.fg),
            Some(theme.color.accent),
            "the leaf icon uses its patch"
        );
        let right_2 = 24u16.saturating_sub(2);
        assert_eq!(
            painted
                .cell(Position::new(right_2, 1))
                .map(BufferCell::symbol),
            Some(dirty),
            "trailing Dirty sits at right-2"
        );
        assert_eq!(
            painted.cell(Position::new(right_2, 1)).map(|cell| cell.fg),
            Some(theme.color.warning)
        );
        assert_ne!(
            painted
                .cell(Position::new(right_2, 0))
                .map(BufferCell::symbol),
            Some(dirty),
            "trailing None does not paint a mark on the sibling row"
        );
        assert_eq!(
            painted.cell(Position::new(20, 0)).map(|cell| cell.bg),
            Some(theme.color.danger),
            "focused_patch tints only the focused cursor row"
        );
        assert_ne!(
            painted.cell(Position::new(20, 1)).map(|cell| cell.bg),
            Some(theme.color.danger)
        );
        assert_ne!(
            unpatched.cell(Position::new(20, 0)).map(|cell| cell.bg),
            Some(theme.color.danger),
            "without focused_patch the cursor row keeps the shared fill"
        );
    }

    fn draw_focused<K, R>(
        theme: Theme,
        tree: &Tree<'_, N, K, R>,
        state: &TreeState,
        items: &[N],
        area: Rect,
    ) -> Buffer
    where
        K: termrock_collections::collection::key::KeyFn<N>,
        R: termrock_runtime::rowui::RowFn<N>,
    {
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        TREE,
                        crate::ReferenceState::FOCUSED,
                    )),
                    |ui| {
                        tree.draw(ui, area, state, items);
                    },
                );
            })
            .commit_presented();
        buffer
    }

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

    #[test]
    fn expand_collapse_is_keyed_not_positional() {
        let t = tree();
        let mut st = TreeState::new();
        let beta = ItemKey::text("beta");
        let alpha = ItemKey::text("alpha");

        assert_eq!(visible(&t, &FOREST, &st), ["alpha", "beta"]);

        st.expand(beta);
        st.set_cursor(2, ItemKey::text("b1"));
        st.choose(Some(ItemKey::text("b1")));
        assert_eq!(visible(&t, &FOREST, &st), ["alpha", "beta", "b1", "b2"]);

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

        let mut st2 = st.clone();
        let mut index = TreeIndex::default();
        index.sync(&t, &st2, &reordered);
        let len = t.reconcile(&mut st2, &reordered, &index);
        assert_eq!(len, 4);
        assert_eq!(st2.cursor(), Some(ItemKey::text("b1")));
        assert_eq!(st2.chosen(), Some(ItemKey::text("b1")));

        st.collapse(beta);
        assert_eq!(visible(&t, &reordered, &st), ["beta", "alpha"]);
        assert!(!st.is_expanded(beta));

        st.expand(beta);
        st.expand(ItemKey::text("b2"));
        assert_eq!(
            visible(&t, &reordered, &st),
            ["beta", "b1", "b2", "b2x", "alpha"]
        );
    }

    #[test]
    fn lazy_children_do_not_reflatten_the_world() {
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

        draw(&t, &st, &items, area);
        assert_eq!(painted.get(), 1);
        assert_eq!(
            seen_depth1.get(),
            0,
            "a collapsed subtree reached the renderer"
        );

        painted.set(0);
        st.expand(ItemKey::index(0));
        draw(&t, &st, &items, area);
        assert_eq!(
            painted.get(),
            6,
            "the renderer ran once per node instead of once per visible row"
        );
        assert!(seen_depth1.get() <= 6);

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

    fn draw<R: termrock_runtime::rowui::RowFn<N>>(
        t: &Tree<'_, N, termrock_collections::collection::key::ByIndex, R>,
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

    fn render<
        K: termrock_collections::collection::key::KeyFn<N>,
        R: termrock_runtime::rowui::RowFn<N>,
    >(
        theme: Theme,
        tree: &Tree<'_, N, K, R>,
        state: &TreeState,
        items: &[N],
    ) -> Buffer {
        let area = Rect::new(0, 0, 24, items.len().max(1) as u16);
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                tree.draw(ui, area, state, items);
            })
            .commit_presented();
        buffer
    }

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

        let leaf = |n: &N| TreeNode::leaf(n.1).keyed(ItemKey::text(n.0));
        let t = Tree::new(TREE).node(&leaf);
        let mut index = TreeIndex::default();
        index.sync(&t, &st, &items);
        let mut acc = Acc::<TreeAction>::new();
        assert!(!t.toggle_at(&mut st, &mut index, &items, 0, &mut acc));
        assert!(acc.finish(TREE).action_ref().is_none());
    }

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

    #[test]
    fn tree_100k_nodes_render() {
        use std::rc::Rc;

        use termrock_collections::collection::KeyFn;
        use termrock_core::response::Response;
        use termrock_runtime::event::Input;
        use termrock_runtime::stub::deliver;
        use termrock_runtime::ui::Cx;

        struct TreeApp<'a, K, R> {
            tree: Tree<'a, CountedNode, K, R>,
            state: TreeState,
            items: Vec<CountedNode>,
        }
        impl<K: KeyFn<CountedNode>, R: Fn(&CountedNode, &mut RowUi<'_>)> crate::runtime::App
            for TreeApp<'_, K, R>
        {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                self.tree.update(cx, &mut self.state, &self.items).erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                self.tree
                    .draw(ui, Rect::new(0, 0, 30, 6), &self.state, &self.items);
            }
        }

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
        let node_accesses = Rc::new(Cell::new(0usize));
        let painted = Rc::new(Cell::new(0usize));
        let node = {
            let node_accesses = Rc::clone(&node_accesses);
            move |item: &CountedNode| {
                node_accesses.set(node_accesses.get().saturating_add(1));
                if item.parent {
                    TreeNode::parent(item.depth)
                } else {
                    TreeNode::leaf(item.depth)
                }
                .keyed(ItemKey::num(item.key))
            }
        };
        let row = {
            let painted = Rc::clone(&painted);
            move |_item: &CountedNode, _ui: &mut RowUi<'_>| {
                painted.set(painted.get().saturating_add(1));
            }
        };
        let mut state = TreeState::new();
        state.expand(ItemKey::num(0));
        let area = Rect::new(0, 0, 30, 6);

        {
            let mut rt = Runtime::new(
                TreeApp {
                    tree: Tree::new(TREE).node(&node).row(row),
                    state,
                    items,
                },
                Theme::junie(),
            );
            let _ = rt.initialize();
            let mut page = Buffer::empty(area);
            rt.draw_buffer(area, &mut page).commit_presented();
            let _ = rt.settle();

            assert!(node_accesses.get() >= rt.app().items.len());
            node_accesses.set(0);
            painted.set(0);

            rt.draw_buffer(area, &mut page).commit_presented();
            assert_eq!(node_accesses.get(), 0, "draw rebuilt update's index");
            assert_eq!(painted.get(), 6);

            let _ = deliver(&mut rt, Input::Tick);
            painted.set(0);
            rt.draw_buffer(area, &mut page).commit_presented();
            assert_eq!(node_accesses.get(), 0, "unchanged draw scanned the source");
            assert_eq!(painted.get(), 6);
        }
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
        let area = Rect::new(0, 0, 30, 6);

        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut page = Buffer::empty(area);
        let draw = |rt: &mut Runtime<Stub>, page: &mut Buffer, state: &TreeState| {
            rt.draw_scene(area, page, |ui, area| {
                tree.draw(ui, area, state, &FOREST);
            })
            .commit_presented();
        };

        draw(&mut rt, &mut page, &state);
        assert_eq!(accesses.get(), FOREST.len());
        accesses.set(0);
        draw(&mut rt, &mut page, &state);
        assert_eq!(accesses.get(), 0);
        state.invalidate();
        draw(&mut rt, &mut page, &state);
        assert_eq!(accesses.get(), FOREST.len());
        accesses.set(0);
        rt.set_theme(Theme::junie());
        draw(&mut rt, &mut page, &state);
        assert_eq!(accesses.get(), FOREST.len());
        assert!(state.is_expanded(ItemKey::text("alpha")));
        assert_eq!(state.chosen(), Some(ItemKey::text("a1")));
    }

    #[test]
    fn equal_source_generations_from_replaced_and_cloned_states_rebuild() {
        let tree = tree();
        let original = [N("alpha", 0, false), N("beta", 0, false)];
        let reordered = [original[1], original[0]];
        let mut first = TreeState::new();
        first.invalidate();
        let mut second = TreeState::new();
        second.invalidate();
        let mut index = TreeIndex::default();
        index.sync(&tree, &first, &original);
        index.sync(&tree, &second, &reordered);
        assert_eq!(index.row(0).map(|row| row.key), Some(ItemKey::text("beta")));
        let mut sibling = second.clone();
        second.invalidate();
        sibling.invalidate();
        index.sync(&tree, &second, &reordered);
        index.sync(&tree, &sibling, &original);
        assert_eq!(
            index.row(0).map(|row| row.key),
            Some(ItemKey::text("alpha"))
        );
        assert_eq!(index.source_rebuilds, 4);
        index.sync(&tree, &sibling, &original);
        assert_eq!(index.source_rebuilds, 4);
    }

    #[test]
    fn cached_weak_identity_prevents_replacement_address_reuse() {
        let tree = tree();
        let mut state = TreeState::new();
        state.expand(ItemKey::text("alpha"));
        let mut index = TreeIndex::default();
        index.sync(&tree, &state, &FOREST);
        let weak = index.expansion_lineage.as_ref().unwrap();
        let old_address = weak.as_ptr();
        drop(state);
        assert!(weak.upgrade().is_none());
        let mut replacement = TreeState::new();
        replacement.expand(ItemKey::text("beta"));
        assert_ne!(
            old_address,
            std::sync::Arc::as_ptr(replacement.expansion_lineage.as_ref().unwrap())
        );
        index.sync(&tree, &replacement, &FOREST);
        assert_eq!(
            index.source_rebuilds, 1,
            "expansion replacement must not rescan source"
        );
        assert_eq!(index.visible_rebuilds, 2);
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
    fn leaf_disclosure_placeholder_inherits_row_style_but_branch_keeps_icon_style() {
        let leaf = [N("item", 0, false)];
        let leaf_theme = Theme::junie();
        let primary = leaf_theme.color.fg[0];
        let leaf_buffer = render(leaf_theme, &tree(), &TreeState::new(), &leaf);
        assert_eq!(
            leaf_buffer.cell(Position::new(1, 0)).map(|cell| cell.fg),
            Some(primary),
            "a leaf's blank disclosure cell must inherit the row foreground"
        );
        assert_eq!(
            leaf_buffer.cell(Position::new(2, 0)).map(|cell| cell.fg),
            Some(primary),
            "the reserved blank marker cell must keep the row foreground"
        );

        let branch = [N("folder", 0, true)];
        let branch_theme = Theme::junie();
        let secondary = branch_theme.color.fg[1];
        let branch_buffer = render(branch_theme, &tree(), &TreeState::new(), &branch);
        assert_eq!(
            branch_buffer.cell(Position::new(1, 0)).map(|cell| cell.fg),
            Some(secondary),
            "a real disclosure glyph must retain the ICON recipe"
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
        assert_eq!(marker_rows(&unselected), Vec::<u16>::new());

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
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        TREE,
                        crate::ReferenceState::FOCUSED,
                    )),
                    |ui| tree.draw(ui, area, &state, &items),
                );
            })
            .commit_presented();

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

        tree.engage(
            &mut state,
            &mut index,
            &reordered,
            0,
            super::Engagement::Choose,
            &mut acc,
        );

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
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                tree.draw(ui, rect, &state, &items);
            })
            .commit_presented();

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
