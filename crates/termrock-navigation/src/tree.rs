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
    trailing_reserve: u16,
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
            trailing_reserve: 2,
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
    pub const fn trailing_reserve(mut self, cells: u16) -> Self {
        self.trailing_reserve = if cells < 2 { 2 } else { cells };
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
            trailing_reserve: self.trailing_reserve,
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

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tree<'a, T, K, R2> {
        self.row_adapter(r)
    }

    pub fn row_adapter<R2: RowFn<T>>(self, r: R2) -> Tree<'a, T, K, R2> {
        Tree {
            id: self.id,
            key: self.key,
            row: r,
            render_row: self.render_row,
            node: self.node,
            icon: self.icon,
            trailing_mark: self.trailing_mark,
            trailing_reserve: self.trailing_reserve,
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
        x: u16,
        row_style: crate::theme::PaintStyle,
    ) {
        let mark = cell_at(row.rect, x);
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
        let content_lane = rest;
        let reserve = if trailing.is_some() {
            self.trailing_reserve
        } else {
            0
        };
        if reserve > 0 {
            rest.width = rest.width.saturating_sub(reserve);
        }
        if rest.width > 0 && rest.x < row.rect.right() {
            let mut r = RowUi::new_with_patches_and_container_width(
                ui,
                self.id,
                Family::TREE,
                Variant::DEFAULT,
                row.flags,
                row.key,
                rest,
                row.rect.width,
                merge_focus(self.ov.part_patch(Part::CONTAINER), focus),
                merge_focus(self.ov.part_patch(Part::LABEL), focus),
            );
            self.row.row(item, &mut r);
        }
        if let Some((role, patch)) = trailing {
            let mark_x = if reserve == 2 {
                Some(row.rect.right().saturating_sub(reserve))
            } else {
                row.rect.right().checked_sub(reserve).filter(|x| {
                    *x >= row.rect.x
                        && *x < row.rect.right()
                        && *x >= content_lane.x
                        && *x < content_lane.right()
                })
            };
            if let Some(mark_x) = mark_x {
                Self::paint_trailing(ui, row, role, patch, mark_x, row_style);
            }
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
