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

/// A public sample table that can open a query document.
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

/// Native Tree and TextInput model owned by the Query Workbench Explorer.
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

        // Keep the native blur path active while the Explorer is hidden or a
        // modal owns input. Disabled TextInput still handles FocusOut, while
        // its edit guards reject ordinary hidden keys, paste, and pointer input.
        let filter = TextInput::new(FILTER)
            .disabled(true)
            .placeholder("Filter database objects");
        let filter_update = filter.update(cx, &mut self.filter_input, &mut self.filter_value);
        self.refresh_filter_query();

        // Tree::update returns before consuming intents when disabled. Drain
        // them here without routing hidden tree input or creating actions.
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

    #[cfg(test)]
    pub(super) fn filter_value(&self) -> &str {
        &self.filter_value
    }

    #[cfg(test)]
    pub(super) fn normalized_query(&self) -> &str {
        &self.normalized_query
    }

    #[cfg(test)]
    pub(super) const fn query_revision(&self) -> u64 {
        self.query_revision
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

    #[cfg(test)]
    fn set_committed_filter(&mut self, value: &str) {
        self.filter_value = value.to_owned();
        self.refresh_filter_query();
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

#[cfg(test)]
mod tests {
    use super::super::QueryWorkbenchPage;
    use super::{
        ALL_LAZY_BRANCHES, ExplorerState, LazyLoadState, TREE, audit_key, functions_key,
        public_key, views_key,
    };
    use ratatui::layout::Rect;
    use termrock::{
        App, Cx, GlyphRole, ItemKey, KeyCode, KeyMap, Response, TextAction, Theme, TreeAction,
        TreeNode, Ui,
    };
    use termrock_test_support::Harness;

    #[derive(Debug)]
    struct Host {
        explorer: ExplorerState,
        page: QueryWorkbenchPage,
        area: Rect,
        keymap: KeyMap,
        last_tree_action: Option<TreeAction>,
        last_filter_action: Option<TextAction>,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let update = self.explorer.update(cx);
            self.last_tree_action = update.tree_action;
            self.last_filter_action = update.filter_action;
            let mut response = update.response;
            if let Some(action) = update.tree_action
                && self.page.apply_explorer_action(action, &self.explorer)
            {
                response |= Response::changed();
            }
            response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.explorer.draw(ui, self.area);
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn harness_with_area(area: Rect) -> Harness<Host> {
        Harness::new(
            Host {
                explorer: ExplorerState::new(),
                page: QueryWorkbenchPage::new(),
                area,
                keymap: KeyMap::default(),
                last_tree_action: None,
                last_filter_action: None,
            },
            Theme::junie(),
            area.width,
            area.height,
        )
    }

    fn harness() -> Harness<Host> {
        harness_with_area(Rect::new(0, 0, 100, 28))
    }

    fn focus_row(harness: &mut Harness<Host>, key: ItemKey) {
        assert!(harness.tab_to(TREE));
        let index = harness
            .app()
            .explorer
            .rows
            .iter()
            .position(|row| row.key == key)
            .expect("the row exists in the current tree source");
        harness.app_mut().explorer.tree_state.set_cursor(index, key);
    }

    fn loaded_state(harness: &Harness<Host>, branch: super::LazyBranch) -> LazyLoadState {
        harness.app().explorer.branch_state(branch)
    }

    #[test]
    fn sample_rows_use_stable_identity_and_non_sample_rows_have_no_table_identity() {
        let explorer = ExplorerState::new();
        for table in super::SampleTable::ALL {
            assert_eq!(explorer.sample_table_for(table.key()), Some(table));
        }
        for key in [
            ItemKey::text("query.workbench.explorer.database.acme_prod"),
            public_key(),
            ItemKey::text("query.workbench.explorer.collection.tables"),
            views_key(),
            functions_key(),
            ItemKey::text("query.workbench.explorer.schema.analytics"),
            ItemKey::text("query.workbench.explorer.schema.audit"),
            ItemKey::text("query.workbench.unknown"),
        ] {
            assert_eq!(explorer.sample_table_for(key), None);
        }
    }

    #[test]
    fn native_tree_activation_reports_the_stable_sample_row_key() {
        let mut harness = harness();
        let key = super::SampleTable::Orders.key();
        focus_row(&mut harness, key);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(
            harness.app().last_tree_action,
            Some(TreeAction::Activated(key))
        );
        assert_eq!(
            harness.app().explorer.sample_table_for(key),
            Some(super::SampleTable::Orders)
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn native_tree_expansion_queues_one_six_tick_load() {
        let route_tree = Rect::new(0, 0, 23, 30);
        let mut click_harness = harness_with_area(Rect::new(
            0,
            0,
            route_tree.width,
            route_tree.height.saturating_add(1),
        ));
        assert_eq!(
            click_harness.area_of(TREE),
            Some(Rect::new(0, 1, route_tree.width, route_tree.height)),
            "the isolated Explorer Host uses the route Tree rectangle"
        );
        let analytics = ItemKey::text("query.workbench.explorer.schema.analytics");
        let analytics_row = termrock::PartRef::item(termrock::Part::ROW, analytics);
        let analytics_icon = termrock::PartRef::item(termrock::Part::ICON, analytics);
        let row_before = click_harness.area_of_part(TREE, analytics_row);
        let icon_before = click_harness.area_of_part(TREE, analytics_icon);
        let focus_before = click_harness.focus();
        let diagnostics_before = format!("{:?}", click_harness.diagnostics());
        let rendered_before = row_before.map(|area| {
            (area.x..area.x.saturating_add(area.width))
                .map(|x| click_harness.cell(x, area.y).symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        });
        let click_response = click_harness.click_part(TREE, analytics_row);
        let row_after = click_harness.area_of_part(TREE, analytics_row);
        let icon_after = click_harness.area_of_part(TREE, analytics_icon);
        let focus_after = click_harness.focus();
        let diagnostics_after = format!("{:?}", click_harness.diagnostics());
        let rendered_after = row_after.map(|area| {
            (area.x..area.x.saturating_add(area.width))
                .map(|x| click_harness.cell(x, area.y).symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        });
        let isolated_tree_action = click_harness.app().last_tree_action;
        let isolated_expanded = click_harness
            .app()
            .explorer
            .tree_state
            .is_expanded(analytics);
        let isolated_lazy_state = loaded_state(&click_harness, super::LazyBranch::Analytics);
        let spinner_frame = super::first_spinner_frame(Theme::junie().design.motion.spinner_frames);
        assert!(
            isolated_tree_action == Some(TreeAction::Expanded(analytics))
                && isolated_expanded
                && isolated_lazy_state == LazyLoadState::Loading { remaining_ticks: 6 },
            "route-sized isolated ROW click: tree={:?}, ROW={row_before:?}->{row_after:?}, ICON={icon_before:?}->{icon_after:?}, focus={focus_before:?}->{focus_after:?}, diagnostics={diagnostics_before}->{diagnostics_after}, row={rendered_before:?}->{rendered_after:?}, changed={}, action={isolated_tree_action:?}, expanded={isolated_expanded}, lazy={isolated_lazy_state:?}",
            click_harness.area_of(TREE),
            click_response.is_changed()
        );
        {
            let loading_row = click_harness
                .app()
                .explorer
                .rows
                .iter()
                .find(|row| row.key == analytics)
                .expect("Analytics remains in the model while loading");
            assert!(loading_row.is_loading, "loading is explicit model state");
            assert_eq!(super::first_spinner_frame(&[]), None);
            assert_eq!(super::first_spinner_frame(&[""]), None);
            assert_eq!(
                super::tree_icon(loading_row, None).map(|(icon, _)| icon),
                Some("S"),
                "a missing spinner frame falls back to the ordinary schema icon"
            );
            assert_eq!(
                super::tree_icon(loading_row, Some("")).map(|(icon, _)| icon),
                Some("S"),
                "an empty spinner frame falls back to the ordinary schema icon"
            );
        }
        let icon_area = icon_after.expect("the loading branch keeps its icon cell");
        assert_eq!(
            click_harness.cell(icon_area.x, icon_area.y).symbol(),
            spinner_frame.unwrap_or("S"),
            "the first nonempty theme spinner frame paints in the reserved icon cell"
        );
        assert!(
            !click_harness.text().contains("Loading…"),
            "the narrow 23-cell row clips trailing Loading metadata; the icon gives immediate feedback"
        );

        click_harness.ticks(6);
        let loaded_row = click_harness
            .app()
            .explorer
            .rows
            .iter()
            .find(|row| row.key == analytics)
            .expect("Analytics remains in the model after loading");
        assert!(
            !loaded_row.is_loading,
            "loading clears when the model is loaded"
        );
        let loaded_icon = click_harness
            .area_of_part(TREE, analytics_icon)
            .expect("the loaded branch keeps its icon cell");
        assert_eq!(
            click_harness.cell(loaded_icon.x, loaded_icon.y).symbol(),
            "S",
            "the schema icon returns after loading completes"
        );

        let mut harness = harness();
        focus_row(&mut harness, views_key());
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Unloaded
        );

        let _ = harness.key(KeyCode::Right);
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Loading { remaining_ticks: 6 }
        );
        let loading_row = harness
            .app()
            .explorer
            .rows
            .iter()
            .find(|row| row.key == views_key())
            .expect("the expanded Views row remains present while it loads");
        assert_eq!(loading_row.meta, Some("Loading…"));
        assert_eq!(loading_row.trailing_mark, None);
        for remaining_ticks in (0..super::LAZY_TICKS).rev() {
            let response = harness.tick();
            assert!(
                response.is_changed(),
                "lazy tick {remaining_ticks} repaints"
            );
            let expected = if remaining_ticks == 0 {
                LazyLoadState::Loaded
            } else {
                LazyLoadState::Loading { remaining_ticks }
            };
            assert_eq!(loaded_state(&harness, super::LazyBranch::Views), expected);
        }
        assert_eq!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .find(|row| row.key == views_key())
                .and_then(|row| row.trailing_mark),
            Some(GlyphRole::ProgressDone)
        );
        assert!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .any(|row| row.label == "active_customers")
        );
        assert_eq!(
            harness.app().explorer.sample_table_for(ItemKey::text(
                "query.workbench.explorer.view.active_customers"
            )),
            None
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn independent_branches_finish_on_their_own_sixth_tick() {
        let mut harness = harness();
        focus_row(&mut harness, views_key());
        let _ = harness.key(KeyCode::Right);
        harness.ticks(2);

        focus_row(&mut harness, functions_key());
        let _ = harness.key(KeyCode::Right);
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Loading { remaining_ticks: 4 }
        );
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Functions),
            LazyLoadState::Loading { remaining_ticks: 6 }
        );
        harness.ticks(4);
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Loaded
        );
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Functions),
            LazyLoadState::Loading { remaining_ticks: 2 }
        );
        harness.ticks(2);
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Functions),
            LazyLoadState::Loaded
        );
    }

    #[test]
    fn collapsing_during_load_keeps_the_load_and_does_not_reexpand() {
        let mut harness = harness();
        focus_row(&mut harness, views_key());
        let _ = harness.key(KeyCode::Right);
        let _ = harness.key(KeyCode::Left);
        assert!(!harness.app().explorer.tree_state.is_expanded(views_key()));
        harness.ticks(6);

        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Loaded
        );
        assert!(!harness.app().explorer.tree_state.is_expanded(views_key()));
        assert!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .any(|row| row.label == "active_customers")
        );
    }

    #[test]
    fn filtering_a_loading_branch_does_not_cancel_its_load() {
        let mut harness = harness();
        focus_row(&mut harness, views_key());
        let _ = harness.key(KeyCode::Right);
        harness.app_mut().explorer.set_committed_filter("orders");
        harness.ticks(6);

        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Views),
            LazyLoadState::Loaded
        );
        assert_eq!(harness.app().explorer.normalized_query(), "orders");
        assert!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .any(|row| row.label == "active_customers")
        );
    }

    #[test]
    fn native_expand_all_queues_every_unloaded_branch() {
        let mut harness = harness();
        assert!(harness.tab_to(TREE));
        let _ = harness.key(KeyCode::Char('*'));
        for branch in ALL_LAZY_BRANCHES {
            assert_eq!(
                loaded_state(&harness, branch),
                LazyLoadState::Loading { remaining_ticks: 6 }
            );
        }
        harness.ticks(2);
        for branch in ALL_LAZY_BRANCHES {
            assert_eq!(
                loaded_state(&harness, branch),
                LazyLoadState::Loading { remaining_ticks: 4 },
                "the first ExpandAll queues each branch once"
            );
        }
        let _ = harness.key(KeyCode::Char('*'));
        let _ = harness.key(KeyCode::Char('*'));
        for branch in ALL_LAZY_BRANCHES {
            assert_eq!(
                loaded_state(&harness, branch),
                LazyLoadState::Loading { remaining_ticks: 4 },
                "repeated ExpandAll does not reset an active load"
            );
        }
        for remaining_ticks in (0..4).rev() {
            let response = harness.tick();
            assert!(
                response.is_changed(),
                "ExpandAll tick {remaining_ticks} repaints"
            );
            let expected = if remaining_ticks == 0 {
                LazyLoadState::Loaded
            } else {
                LazyLoadState::Loading { remaining_ticks }
            };
            for branch in ALL_LAZY_BRANCHES {
                assert_eq!(loaded_state(&harness, branch), expected);
            }
        }
        assert!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .any(|row| row.label == "events")
        );
        assert!(
            harness
                .app()
                .explorer
                .rows
                .iter()
                .any(|row| row.label == "No objects")
        );
    }

    #[test]
    fn filter_visibility_drawing_and_metadata_do_not_load_branches() {
        let mut harness = harness();
        assert!(harness.tab_to(TREE));
        harness.app_mut().explorer.set_committed_filter("analytics");
        let _ = harness.tick();
        assert_eq!(harness.app().explorer.normalized_query(), "analytics");
        assert!(harness.text().contains("analytics"));
        assert!(!harness.text().contains("events"));
        assert!(
            ALL_LAZY_BRANCHES
                .iter()
                .all(|branch| loaded_state(&harness, *branch) == LazyLoadState::Unloaded)
        );

        assert!(harness.tab_to(TREE));
        harness.app_mut().explorer.set_committed_filter("48k");
        let _ = harness.tick();
        assert_eq!(harness.app().explorer.normalized_query(), "48k");
        assert!(!harness.text().contains("customers"));
        assert!(
            ALL_LAZY_BRANCHES
                .iter()
                .all(|branch| loaded_state(&harness, *branch) == LazyLoadState::Unloaded)
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn filter_reads_live_typing_paste_cancel_blur_and_trimmed_query() {
        let mut harness = harness();
        assert!(harness.tab_to(super::FILTER));
        let _ = harness.type_str("orders");
        assert_eq!(harness.app().explorer.filter_value(), "");
        assert_eq!(harness.app().explorer.normalized_query(), "orders");
        assert!(!harness.text().contains("customers"));

        let _ = harness.key(KeyCode::Esc);
        assert_eq!(
            harness.app().last_filter_action,
            Some(TextAction::Cancelled)
        );
        assert_eq!(harness.app().explorer.normalized_query(), "");
        assert!(harness.text().contains("customers"));

        assert!(!harness.app().explorer.is_filter_editing());
        assert!(harness.tab_to(TREE));
        assert!(harness.tab_to(super::FILTER));
        assert!(harness.app().explorer.is_filter_editing());
        let _ = harness.paste("  orders  ");
        assert_eq!(harness.app().explorer.normalized_query(), "orders");
        assert_eq!(harness.app().explorer.filter_value(), "");
        assert!(harness.tab_to(TREE));
        assert_eq!(harness.app().explorer.filter_value(), "  orders  ");
        assert_eq!(harness.app().explorer.normalized_query(), "orders");
        assert_eq!(
            harness.app().last_filter_action,
            Some(TextAction::Committed)
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn whitespace_does_not_advance_revision_and_empty_query_uses_unfiltered_tree() {
        let mut harness = harness();
        let original_revision = harness.app().explorer.query_revision();
        assert!(harness.tab_to(super::FILTER));
        let _ = harness.type_str("   ");
        assert_eq!(harness.app().explorer.normalized_query(), "");
        assert_eq!(harness.app().explorer.query_revision(), original_revision);
        assert!(harness.text().contains("customers"));

        let _ = harness.paste("orders");
        assert_eq!(
            harness.app().explorer.query_revision(),
            original_revision + 1
        );
        let _ = harness.paste("   ");
        assert_eq!(harness.app().explorer.normalized_query(), "orders");
        assert_eq!(
            harness.app().explorer.query_revision(),
            original_revision + 1
        );
    }

    #[test]
    fn matching_lazy_branch_can_expand_explicitly_while_filtered() {
        let mut harness = harness();
        let analytics = super::analytics_key();
        assert!(harness.tab_to(TREE));
        harness.app_mut().explorer.set_committed_filter("analytics");
        let _ = harness.tick();
        assert!(harness.text().contains("acme_prod"));
        assert!(harness.text().contains("analytics"));
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Analytics),
            LazyLoadState::Unloaded
        );

        assert!(harness.tab_to(TREE));
        harness
            .app_mut()
            .explorer
            .tree_state
            .set_cursor(1, analytics);
        let _ = harness.key(KeyCode::Right);
        assert_eq!(
            harness.app().last_tree_action,
            Some(TreeAction::Expanded(analytics))
        );
        assert_eq!(
            loaded_state(&harness, super::LazyBranch::Analytics),
            LazyLoadState::Loading { remaining_ticks: 6 }
        );
    }

    #[test]
    fn query_revision_overflow_invalidates_the_native_tree_state() {
        let mut explorer = ExplorerState::new();
        explorer.query_revision = u64::MAX;
        explorer.normalized_query = "prior".to_owned();
        let before = explorer.tree_state.clone();
        explorer.set_committed_filter("next");
        assert_eq!(explorer.query_revision(), 0);
        assert_ne!(explorer.tree_state, before);
    }

    #[test]
    fn same_length_row_kind_replacement_invalidates_native_tree_state() {
        let mut explorer = ExplorerState::new();
        let before = explorer.tree_state.clone();
        let mut replacement = explorer.rows.clone();
        let replacement_len = replacement.len();
        let row = replacement
            .iter_mut()
            .find(|row| row.key == super::SampleTable::Orders.key())
            .expect("orders row exists");
        row.node = TreeNode::parent(3).keyed(row.key);
        explorer.replace_rows(replacement);
        assert_eq!(explorer.rows.len(), replacement_len);
        assert_ne!(explorer.tree_state, before);
    }

    #[test]
    fn same_length_row_replacement_invalidates_cached_rows_and_repaints() {
        let mut harness = harness();
        assert!(
            harness.text().contains("orders"),
            "initial draw primes native rows"
        );
        let mut replacement = harness.app().explorer.rows.clone();
        let replacement_len = replacement.len();
        let orders = replacement
            .iter_mut()
            .find(|row| row.key == super::SampleTable::Orders.key())
            .expect("orders row exists");
        orders.label = "archive-only";
        orders.node = TreeNode::parent(3).keyed(orders.key);
        assert!(harness.app_mut().explorer.replace_rows(replacement));
        assert_eq!(harness.app().explorer.rows.len(), replacement_len);

        let response = harness.tick();
        assert!(response.is_changed());
        assert!(harness.text().contains("archive-only"));
        assert!(!harness.text().contains("orders"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn same_length_replacement_invalidates_cached_filter_matches_without_revision_change() {
        let mut harness = harness();
        assert!(harness.text().contains("orders"));

        assert!(harness.tab_to(TREE));
        harness.app_mut().explorer.set_committed_filter("archive");
        let _ = harness.tick();
        let query_revision = harness.app().explorer.query_revision();
        assert_eq!(harness.app().explorer.normalized_query(), "archive");
        assert!(!harness.text().contains("orders"));

        let mut replacement = harness.app().explorer.rows.clone();
        let replacement_len = replacement.len();
        let orders = replacement
            .iter_mut()
            .find(|row| row.key == super::SampleTable::Orders.key())
            .expect("orders row exists");
        orders.label = "archive-only";
        orders.node = TreeNode::parent(3).keyed(orders.key);
        assert!(harness.app_mut().explorer.replace_rows(replacement));
        assert_eq!(harness.app().explorer.rows.len(), replacement_len);
        assert_eq!(harness.app().explorer.query_revision(), query_revision);

        let response = harness.tick();
        assert!(response.is_changed());
        assert_eq!(harness.app().explorer.query_revision(), query_revision);
        assert!(harness.text().contains("archive-only"));
        assert!(!harness.text().contains("orders"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn analytics_events_and_empty_note_never_receive_sample_table_identity() {
        let mut harness = harness();
        let analytics_key = super::analytics_key();
        focus_row(&mut harness, analytics_key);
        let _ = harness.key(KeyCode::Right);
        harness.ticks(6);

        let events_key = ItemKey::text("query.workbench.explorer.table.analytics.events");
        focus_row(&mut harness, events_key);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(
            harness.app().last_tree_action,
            Some(TreeAction::Activated(events_key))
        );
        assert_eq!(harness.app().explorer.sample_table_for(events_key), None);
        assert_eq!(harness.app().page.document_count(), 2);
        assert_eq!(
            harness.app().page.tabs_state().active(),
            Some(ItemKey::text("query.workbench.orders"))
        );

        let audit_key = audit_key();
        focus_row(&mut harness, audit_key);
        let _ = harness.key(KeyCode::Right);
        harness.ticks(6);
        let note_key = ItemKey::text("query.workbench.explorer.note.audit_empty");
        focus_row(&mut harness, note_key);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(
            harness.app().last_tree_action,
            Some(TreeAction::Activated(note_key))
        );
        assert_eq!(harness.app().explorer.sample_table_for(note_key), None);
        assert_eq!(harness.app().page.document_count(), 2);
        assert_eq!(
            harness.app().page.tabs_state().active(),
            Some(ItemKey::text("query.workbench.orders"))
        );
    }
}
