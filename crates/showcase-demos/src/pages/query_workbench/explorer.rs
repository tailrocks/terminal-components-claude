use std::time::Duration;

use termrock::{
    Cx, FrameRead, GlyphRole, Id, ItemKey, Rect, Response, RowUi, StylePatch, TextAction,
    TextInput, TextInputState, Tree, TreeAction, TreeNode, TreeState, Ui, UpdateCause,
};

pub(super) const FILTER: Id = Id::root("showcase_ui::pages::query_workbench.explorer_filter");
pub(super) const TREE: Id = Id::root("showcase_ui::pages::query_workbench.explorer_tree");
const LAZY_TICKS: u8 = 6;
const ALL_LAZY_BRANCHES: [LazyBranch; 4] = [
    LazyBranch::Views,
    LazyBranch::Functions,
    LazyBranch::Analytics,
    LazyBranch::Audit,
];

fn database_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.database.acme_prod")
}

fn public_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.schema.public")
}

fn tables_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.collection.tables")
}

fn views_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.collection.views")
}

fn functions_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.collection.functions")
}

fn analytics_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.schema.analytics")
}

fn audit_key() -> ItemKey {
    ItemKey::text("query.workbench.explorer.schema.audit")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum SampleTable {
    Organizations,
    Customers,
    Products,
    Orders,
    OrderItems,
    Payments,
    Subscriptions,
}

impl SampleTable {
    const ALL: [Self; 7] = [
        Self::Organizations,
        Self::Customers,
        Self::Products,
        Self::Orders,
        Self::OrderItems,
        Self::Payments,
        Self::Subscriptions,
    ];

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Organizations => "organizations",
            Self::Customers => "customers",
            Self::Products => "products",
            Self::Orders => "orders",
            Self::OrderItems => "order_items",
            Self::Payments => "payments",
            Self::Subscriptions => "subscriptions",
        }
    }

    const fn row_count(self) -> &'static str {
        match self {
            Self::Organizations => "1.2k",
            Self::Customers => "48k",
            Self::Products => "3.1k",
            Self::Orders => "1.2M",
            Self::OrderItems => "4.8M",
            Self::Payments => "1.1M",
            Self::Subscriptions => "9.4k",
        }
    }

    pub(super) fn key(self) -> ItemKey {
        match self {
            Self::Organizations => ItemKey::text("query.workbench.explorer.table.organizations"),
            Self::Customers => ItemKey::text("query.workbench.explorer.table.customers"),
            Self::Products => ItemKey::text("query.workbench.explorer.table.products"),
            Self::Orders => ItemKey::text("query.workbench.explorer.table.orders"),
            Self::OrderItems => ItemKey::text("query.workbench.explorer.table.order_items"),
            Self::Payments => ItemKey::text("query.workbench.explorer.table.payments"),
            Self::Subscriptions => ItemKey::text("query.workbench.explorer.table.subscriptions"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LazyBranch {
    Views,
    Functions,
    Analytics,
    Audit,
}

impl LazyBranch {
    fn key(self) -> ItemKey {
        match self {
            Self::Views => views_key(),
            Self::Functions => functions_key(),
            Self::Analytics => analytics_key(),
            Self::Audit => audit_key(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LazyLoadState {
    Unloaded,
    Loading { remaining_ticks: u8 },
    Loaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowKind {
    Database,
    Schema,
    Collection,
    Table,
    View,
    Function,
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExplorerRow {
    key: ItemKey,
    label: &'static str,
    node: TreeNode,
    kind: RowKind,
    is_loading: bool,
    meta: Option<&'static str>,
    trailing_mark: Option<GlyphRole>,
    sample_table: Option<SampleTable>,
}

#[derive(Debug)]
struct LazyBranchState {
    branch: LazyBranch,
    state: LazyLoadState,
}

#[derive(Debug)]
pub(super) struct ExplorerState {
    rows: Vec<ExplorerRow>,
    filter_value: String,
    filter_input: TextInputState,
    normalized_query: String,
    query_revision: u64,
    tree_state: TreeState,
    lazy_branches: Vec<LazyBranchState>,
    pending_model_change: bool,
}

pub(super) struct ExplorerUpdate {
    pub(super) response: Response<()>,
    pub(super) tree_action: Option<TreeAction>,
    pub(super) filter_action: Option<TextAction>,
}

impl ExplorerState {
    pub(super) fn new() -> Self {
        let mut tree_state = TreeState::new();
        tree_state.expand(database_key());
        tree_state.expand(public_key());
        tree_state.expand(tables_key());
        let lazy_branches = ALL_LAZY_BRANCHES
            .into_iter()
            .map(|branch| LazyBranchState {
                branch,
                state: LazyLoadState::Unloaded,
            })
            .collect();
        let mut state = Self {
            rows: Vec::new(),
            filter_value: String::new(),
            filter_input: TextInputState::default(),
            normalized_query: String::new(),
            query_revision: 0,
            tree_state,
            lazy_branches,
            pending_model_change: false,
        };
        state.rows = state.build_rows();
        state
    }

    pub(super) fn is_filter_editing(&self) -> bool {
        self.filter_input.is_editing()
    }

    pub(super) fn inactive_update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tick_advanced = self.advance_on_tick(cx);

        let filter = TextInput::new(FILTER)
            .disabled(true)
            .placeholder("Filter database objects");
        let filter_update = filter.update(cx, &mut self.filter_input, &mut self.filter_value);
        self.refresh_filter_query();

        let tree = Tree::new(TREE).row(tree_row).disabled(true);
        let tree_update = tree.update(cx, &mut self.tree_state, &self.rows);
        for _ in cx.intents(TREE) {}

        let model_changed = core::mem::take(&mut self.pending_model_change);
        if self.has_loading_branch() {
            cx.request_repaint_after(Duration::from_millis(40));
        }

        let mut response = filter_update.erase() | tree_update.erase();
        if tick_advanced || model_changed {
            response |= Response::changed();
        }
        response
    }

    pub(super) fn update(&mut self, cx: &mut Cx<'_>) -> ExplorerUpdate {
        let tick_advanced = self.advance_on_tick(cx);
        let filter = TextInput::new(FILTER).placeholder("Filter database objects");
        let filter_update = filter.update(cx, &mut self.filter_input, &mut self.filter_value);
        self.refresh_filter_query();

        let spinner_frame = first_spinner_frame(cx.design().motion.spinner_frames);
        let icon = |row: &ExplorerRow| tree_icon(row, spinner_frame);
        let tree = Tree::new(TREE)
            .key(tree_key)
            .node(&tree_node)
            .row(tree_row)
            .icon(&icon)
            .trailing_mark(&tree_trailing_mark);
        let tree_update = if self.normalized_query.is_empty() {
            tree.update(cx, &mut self.tree_state, &self.rows)
        } else {
            let query = self.normalized_query.as_str();
            let matcher = |row: &ExplorerRow| row.label.to_lowercase().contains(query);
            tree.query(self.query_revision, &matcher)
                .update(cx, &mut self.tree_state, &self.rows)
        };
        let tree_action = tree_update.action_ref().copied();
        let mut lazy_state_changed = false;
        if let Some(TreeAction::Expanded(key)) = tree_action {
            lazy_state_changed |= self.queue_lazy_key(key);
        }
        lazy_state_changed |= self.queue_expanded_lazy_branches();
        if lazy_state_changed {
            let rows = self.build_rows();
            self.replace_rows(rows);
        }
        let model_changed = core::mem::take(&mut self.pending_model_change);
        if self.has_loading_branch() {
            cx.request_repaint_after(Duration::from_millis(40));
        }

        let filter_action = filter_update.action_ref().copied();
        let mut response = filter_update.erase() | tree_update.erase();
        if tick_advanced || model_changed {
            response |= Response::changed();
        }
        ExplorerUpdate {
            response,
            tree_action,
            filter_action,
        }
    }

    pub(super) fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let rows = termrock::layout::rows(
            area,
            &[
                termrock::layout::Track::Fixed(1),
                termrock::layout::Track::Flex(1),
            ],
        );
        let filter_area = rows.first().copied().unwrap_or(Rect::new(0, 0, 0, 0));
        let tree_area = rows.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
        let filter = TextInput::new(FILTER)
            .value(&self.filter_value)
            .placeholder("Filter database objects");
        let _ = filter.draw(ui, filter_area, &self.filter_input);

        let spinner_frame = first_spinner_frame(ui.design().motion.spinner_frames);
        let icon = |row: &ExplorerRow| tree_icon(row, spinner_frame);
        let tree = Tree::new(TREE)
            .key(tree_key)
            .node(&tree_node)
            .row(tree_row)
            .icon(&icon)
            .trailing_mark(&tree_trailing_mark);
        if self.normalized_query.is_empty() {
            let _ = tree.draw(ui, tree_area, &self.tree_state, &self.rows);
        } else {
            let query = self.normalized_query.as_str();
            let matcher = |row: &ExplorerRow| row.label.to_lowercase().contains(query);
            let _ = tree.query(self.query_revision, &matcher).draw(
                ui,
                tree_area,
                &self.tree_state,
                &self.rows,
            );
        }
    }

    pub(super) fn sample_table_for(&self, key: ItemKey) -> Option<SampleTable> {
        self.rows
            .iter()
            .find(|row| row.key == key)
            .and_then(|row| row.sample_table)
    }

    fn advance_on_tick(&mut self, cx: &mut Cx<'_>) -> bool {
        if cx.update_cause() != UpdateCause::Tick {
            return false;
        }
        let mut tick_advanced = false;
        let mut rows_changed = false;
        for branch in &mut self.lazy_branches {
            if let LazyLoadState::Loading { remaining_ticks } = branch.state {
                tick_advanced = true;
                let remaining_ticks = remaining_ticks.saturating_sub(1);
                if remaining_ticks == 0 {
                    branch.state = LazyLoadState::Loaded;
                    rows_changed = true;
                } else {
                    branch.state = LazyLoadState::Loading { remaining_ticks };
                }
            }
        }
        if rows_changed {
            let rows = self.build_rows();
            self.replace_rows(rows);
        }
        tick_advanced
    }

    fn refresh_filter_query(&mut self) {
        let input = self
            .filter_input
            .draft_text()
            .unwrap_or(self.filter_value.as_str());
        let normalized = input.trim().to_lowercase();
        if normalized == self.normalized_query {
            return;
        }
        self.normalized_query = normalized;
        if let Some(revision) = self.query_revision.checked_add(1) {
            self.query_revision = revision;
        } else {
            self.query_revision = 0;
            self.tree_state.invalidate();
        }
    }

    fn queue_lazy_key(&mut self, key: ItemKey) -> bool {
        let Some(branch) = ALL_LAZY_BRANCHES
            .into_iter()
            .find(|branch| branch.key() == key)
        else {
            return false;
        };
        self.queue_lazy_branch(branch)
    }

    fn queue_lazy_branch(&mut self, branch: LazyBranch) -> bool {
        if !self.tree_state.is_expanded(branch.key()) {
            return false;
        }
        let Some(load) = self
            .lazy_branches
            .iter_mut()
            .find(|load| load.branch == branch)
        else {
            return false;
        };
        if load.state != LazyLoadState::Unloaded {
            return false;
        }
        load.state = LazyLoadState::Loading {
            remaining_ticks: LAZY_TICKS,
        };
        self.pending_model_change = true;
        true
    }

    fn queue_expanded_lazy_branches(&mut self) -> bool {
        let mut changed = false;
        for branch in ALL_LAZY_BRANCHES {
            changed |= self.queue_lazy_branch(branch);
        }
        changed
    }

    fn has_loading_branch(&self) -> bool {
        self.lazy_branches
            .iter()
            .any(|branch| matches!(branch.state, LazyLoadState::Loading { .. }))
    }

    fn branch_state(&self, branch: LazyBranch) -> LazyLoadState {
        self.lazy_branches
            .iter()
            .find(|load| load.branch == branch)
            .map_or(LazyLoadState::Unloaded, |load| load.state)
    }

    fn replace_rows(&mut self, rows: Vec<ExplorerRow>) -> bool {
        if self.rows == rows {
            return false;
        }
        self.rows = rows;
        self.tree_state.invalidate();
        self.pending_model_change = true;
        true
    }

    fn build_rows(&self) -> Vec<ExplorerRow> {
        let mut rows = Vec::with_capacity(20);
        rows.push(parent_row(
            database_key(),
            "acme_prod",
            0,
            RowKind::Database,
        ));
        rows.push(parent_row(public_key(), "public", 1, RowKind::Schema));
        rows.push(parent_row(tables_key(), "Tables", 2, RowKind::Collection));
        for table in SampleTable::ALL {
            rows.push(ExplorerRow {
                key: table.key(),
                label: table.name(),
                node: TreeNode::leaf(3).keyed(table.key()),
                kind: RowKind::Table,
                is_loading: false,
                meta: Some(table.row_count()),
                trailing_mark: None,
                sample_table: Some(table),
            });
        }

        self.push_lazy_row(
            &mut rows,
            LazyBranch::Views,
            "Views",
            2,
            RowKind::Collection,
        );
        if self.branch_state(LazyBranch::Views) == LazyLoadState::Loaded {
            rows.push(leaf_row(
                ItemKey::text("query.workbench.explorer.view.active_customers"),
                "active_customers",
                3,
                RowKind::View,
                None,
            ));
            rows.push(leaf_row(
                ItemKey::text("query.workbench.explorer.view.monthly_revenue"),
                "monthly_revenue",
                3,
                RowKind::View,
                None,
            ));
        }

        self.push_lazy_row(
            &mut rows,
            LazyBranch::Functions,
            "Functions",
            2,
            RowKind::Collection,
        );
        if self.branch_state(LazyBranch::Functions) == LazyLoadState::Loaded {
            rows.push(leaf_row(
                ItemKey::text("query.workbench.explorer.function.refresh_totals"),
                "refresh_totals()",
                3,
                RowKind::Function,
                None,
            ));
        }

        self.push_lazy_row(
            &mut rows,
            LazyBranch::Analytics,
            "analytics",
            1,
            RowKind::Schema,
        );
        if self.branch_state(LazyBranch::Analytics) == LazyLoadState::Loaded {
            rows.push(leaf_row(
                ItemKey::text("query.workbench.explorer.table.analytics.events"),
                "events",
                2,
                RowKind::Table,
                Some("88M"),
            ));
        }

        self.push_lazy_row(&mut rows, LazyBranch::Audit, "audit", 1, RowKind::Schema);
        if self.branch_state(LazyBranch::Audit) == LazyLoadState::Loaded {
            rows.push(leaf_row(
                ItemKey::text("query.workbench.explorer.note.audit_empty"),
                "No objects",
                2,
                RowKind::Note,
                None,
            ));
        }
        rows
    }

    fn push_lazy_row(
        &self,
        rows: &mut Vec<ExplorerRow>,
        branch: LazyBranch,
        label: &'static str,
        depth: u16,
        kind: RowKind,
    ) {
        let state = self.branch_state(branch);
        let node = match state {
            LazyLoadState::Loaded => TreeNode::parent(depth),
            LazyLoadState::Unloaded | LazyLoadState::Loading { .. } => TreeNode::lazy(depth),
        }
        .keyed(branch.key());
        let (meta, trailing_mark) = match state {
            LazyLoadState::Loading { .. } => (Some("Loading…"), None),
            LazyLoadState::Loaded => (None, Some(GlyphRole::ProgressDone)),
            LazyLoadState::Unloaded => (None, None),
        };
        rows.push(ExplorerRow {
            key: branch.key(),
            label,
            node,
            kind,
            is_loading: matches!(state, LazyLoadState::Loading { .. }),
            meta,
            trailing_mark,
            sample_table: None,
        });
    }
}

fn parent_row(key: ItemKey, label: &'static str, depth: u16, kind: RowKind) -> ExplorerRow {
    ExplorerRow {
        key,
        label,
        node: TreeNode::parent(depth).keyed(key),
        kind,
        is_loading: false,
        meta: None,
        trailing_mark: None,
        sample_table: None,
    }
}

fn leaf_row(
    key: ItemKey,
    label: &'static str,
    depth: u16,
    kind: RowKind,
    meta: Option<&'static str>,
) -> ExplorerRow {
    ExplorerRow {
        key,
        label,
        node: TreeNode::leaf(depth).keyed(key),
        kind,
        is_loading: false,
        meta,
        trailing_mark: None,
        sample_table: None,
    }
}

fn tree_key(row: &ExplorerRow) -> ItemKey {
    row.key
}

fn tree_node(row: &ExplorerRow) -> TreeNode {
    row.node
}

fn tree_row(row: &ExplorerRow, ui: &mut RowUi<'_>) {
    ui.label(row.label);
    if let Some(meta) = row.meta {
        ui.meta(meta);
    }
}

fn first_spinner_frame(frames: &[&'static str]) -> Option<&'static str> {
    frames.iter().copied().find(|frame| !frame.is_empty())
}

fn tree_icon(
    row: &ExplorerRow,
    spinner_frame: Option<&'static str>,
) -> Option<(&'static str, StylePatch)> {
    let icon = if row.is_loading {
        spinner_frame.filter(|frame| !frame.is_empty())
    } else {
        None
    }
    .unwrap_or(match row.kind {
        RowKind::Database => "D",
        RowKind::Schema => "S",
        RowKind::Collection => "▤",
        RowKind::Table => "T",
        RowKind::View => "V",
        RowKind::Function => "ƒ",
        RowKind::Note => "·",
    });
    Some((icon, StylePatch::new()))
}

fn tree_trailing_mark(row: &ExplorerRow) -> Option<(GlyphRole, StylePatch)> {
    row.trailing_mark.map(|mark| (mark, StylePatch::new()))
}
