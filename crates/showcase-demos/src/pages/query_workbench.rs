use std::ops::Range;
use std::time::Duration;

use termrock::grid::{Column, ColumnKey, Table, TableColumn, TableState};
use termrock::{
    ActionKey, Align, CodeAction, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Cx,
    FrameRead, GridModel, Id, ItemKey, Maximized, Panel, PanelKind, Rect, ResetPolicy, Response,
    Role, SplitAxis, SplitFallback, SplitPane, SplitPaneState, StateFlags, StatusBar, StatusItem,
    Tabs, TabsAction, TabsState, ToolItem, Toolbar, ToolbarAction, ToolbarState, TreeAction, Ui,
    UpdateCause, layout,
};

mod explorer;
mod options;

use super::{Page, PageStatus, PageUpdate, frame};
use explorer::{ExplorerState, SampleTable};
use options::{Change as OptionChange, ResultFixture, Settings as WorkbenchOptions};

const TABS: Id = Id::root("showcase_ui::pages::query_workbench.tabs");
const TOOLBAR: Id = Id::root("showcase_ui::pages::query_workbench.toolbar");
const EDITOR_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.editor_panel");
const RESULTS_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.results_panel");
const EXPLORER_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.explorer_panel");
const EXPLORER_SPLIT: Id = Id::root("showcase_ui::pages::query_workbench.explorer_split");
const SPLIT: Id = Id::root("showcase_ui::pages::query_workbench.split");
const EDITOR: Id = Id::root("showcase_ui::pages::query_workbench.editor");
const RESULTS: Id = Id::root("showcase_ui::pages::query_workbench.results");
const OPTIONS_DIALOG: Id = options::DIALOG_ID;
const ENVIRONMENT_STATUS: Id = Id::root("showcase_ui::pages::query_workbench.environment_status");

/// Native owner ID for the query tabs.
pub const TABS_ID: Id = TABS;
/// Native owner ID for the query toolbar.
pub const TOOLBAR_ID: Id = TOOLBAR;
/// Native owner ID for the SQL editor.
pub const EDITOR_ID: Id = EDITOR;
/// Native owner ID for the results table.
pub const RESULTS_ID: Id = RESULTS;
/// Native owner ID for the editor panel.
pub const EDITOR_PANEL_ID: Id = EDITOR_PANEL;
/// Native owner ID for the results panel.
pub const RESULTS_PANEL_ID: Id = RESULTS_PANEL;

const RUN_CAPTURE_OWNERS: [Id; 8] = [
    EXPLORER_SPLIT,
    explorer::FILTER,
    explorer::TREE,
    TABS,
    TOOLBAR,
    SPLIT,
    EDITOR,
    RESULTS,
];
/// Application action key for running the active query.
pub const RUN_QUERY: ActionKey = ActionKey::application("showcase.query_workbench.run");

const RUN_TICKS: u8 = 6;
const FIRST_QUERY: &str = "\
-- Pending orders, newest first
SELECT id, customer, status, total, created_at
FROM orders
WHERE status = 'pending'
ORDER BY created_at DESC
LIMIT 25;
";
const SECOND_QUERY: &str = "\
-- Recent customer accounts
SELECT id, name, created_at
FROM customers
ORDER BY created_at DESC
LIMIT 25;
";
const TABS_HINTS: &[(&str, &str)] = &[("← →", "Switch query"), ("Middle click", "Close tab")];
const TOOLBAR_HINTS: &[(&str, &str)] = &[
    ("← →", "Choose command"),
    ("Enter", "Activate"),
    ("Ctrl+R", "Run query"),
];
const EDITOR_HINTS: &[(&str, &str)] = &[
    ("Ctrl+R", "Run query"),
    ("Ctrl+F", "Find"),
    ("Esc", "Done editing"),
];
const RESULTS_HINTS: &[(&str, &str)] = &[("↑ ↓", "Move rows"), ("← →", "Move columns")];
const EXPLORER_SPLIT_HINTS: &[(&str, &str)] = &[
    ("← →", "Resize panes"),
    ("Home", "Reset Explorer share"),
    ("Ctrl+R", "Run query"),
];
const FILTER_HINTS: &[(&str, &str)] = &[("Ctrl+R", "Run query"), ("Esc", "Cancel filter")];
const TREE_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Move objects"),
    ("← →", "Expand or collapse"),
    ("Enter", "Open table"),
    ("Ctrl+R", "Run query"),
];
const SPLIT_HINTS: &[(&str, &str)] = &[
    ("↑ ↓", "Resize panes"),
    ("Home", "Reset editor share"),
    ("Ctrl+R", "Run query"),
];

const RESULT_COLUMNS: [TableColumn<'static>; 3] = [
    TableColumn::new(Column {
        min_width: 12,
        max_width: 22,
        ..Column::new(ColumnKey::num(0), "Field")
    })
    .flexible(),
    TableColumn::new(Column {
        min_width: 12,
        max_width: 30,
        ..Column::new(ColumnKey::num(1), "Value")
    })
    .flexible(),
    TableColumn::new(Column {
        min_width: 8,
        max_width: 18,
        ..Column::new(ColumnKey::num(2), "Details")
    })
    .flexible(),
];

const ORDERS_RESULT_COLUMNS: [TableColumn<'static>; 5] = [
    TableColumn::new(Column {
        min_width: 7,
        max_width: 7,
        ..Column::new(ColumnKey::num(0), "id")
    }),
    TableColumn::new(Column {
        min_width: 16,
        max_width: u16::MAX,
        ..Column::new(ColumnKey::num(1), "customer")
    })
    .flexible(),
    TableColumn::new(Column {
        min_width: 10,
        max_width: 10,
        ..Column::new(ColumnKey::num(2), "status")
    }),
    TableColumn::new(Column {
        align: Align::Right,
        min_width: 10,
        max_width: 10,
        ..Column::new(ColumnKey::num(3), "total")
    }),
    TableColumn::new(Column {
        min_width: 17,
        max_width: 17,
        ..Column::new(ColumnKey::num(4), "created_at")
    }),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ResultSchema {
    #[default]
    Generic,
    Orders,
}

impl ResultSchema {
    const fn columns(self) -> &'static [TableColumn<'static>] {
        match self {
            Self::Generic => &RESULT_COLUMNS,
            Self::Orders => &ORDERS_RESULT_COLUMNS,
        }
    }
}

const SAMPLE_CUSTOMERS: [&str; 3] = ["Northwind", "Globex", "Initech"];
const ORDER_STATUSES: [&str; 4] = ["pending", "pending", "paid", "shipped"];

/// Result state owned by one query document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryResultState {
    /// The document has no completed query result.
    NotRun,
    /// A run completed with empty SQL.
    Empty,
    /// A run returned rows from a sample table.
    Rows {
        /// Sample table name.
        table: String,
        /// Number of returned rows.
        count: usize,
    },
    /// An explain run completed.
    Plan,
    /// A run failed against the sample schema.
    Error {
        /// Short failure message.
        message: String,
        /// Failure detail for the results table.
        detail: String,
    },
}

type ResultView = QueryResultState;

impl QueryResultState {
    fn summary(&self) -> String {
        match self {
            Self::NotRun => "Not run yet · choose Run.".to_owned(),
            Self::Empty => "Enter SQL, then choose Run.".to_owned(),
            Self::Rows { table, count } => {
                format!("{count} rows from {table} · simulated in 15 ms")
            }
            Self::Plan => "Query plan · simulated in 2 ms".to_owned(),
            Self::Error { message, .. } => format!("Query failed · {message}"),
        }
    }

    fn model(&self) -> ResultModel {
        let (schema, rows) = match self {
            Self::NotRun | Self::Empty => (ResultSchema::Generic, Vec::new()),
            Self::Rows { table, count } if table.eq_ignore_ascii_case("orders") => (
                ResultSchema::Orders,
                (0..*count).map(order_result_row).collect(),
            ),
            Self::Rows { count, .. } => (
                ResultSchema::Generic,
                (0..*count)
                    .map(|index| {
                        let customer = SAMPLE_CUSTOMERS
                            .get(index % SAMPLE_CUSTOMERS.len())
                            .copied()
                            .unwrap_or("Sample");
                        vec![
                            index.saturating_add(1).to_string(),
                            format!("{customer} Account"),
                            format!("2026-10-{:02}", 27_usize.saturating_sub(index % 26)),
                        ]
                    })
                    .collect(),
            ),
            Self::Plan => (
                ResultSchema::Generic,
                vec![
                    vec!["Limit".to_owned(), "0.42..2.10".to_owned(), "25".to_owned()],
                    vec![
                        "Index Scan Backward on orders".to_owned(),
                        "0.42..8 410".to_owned(),
                        "1 204".to_owned(),
                    ],
                    vec![
                        "Index condition".to_owned(),
                        "status = 'pending'".to_owned(),
                        String::new(),
                    ],
                    vec![
                        "Using index".to_owned(),
                        "orders_created_at_idx".to_owned(),
                        String::new(),
                    ],
                ],
            ),
            Self::Error { message, detail } => (
                ResultSchema::Generic,
                vec![vec!["ERROR".to_owned(), message.clone(), detail.clone()]],
            ),
        };
        ResultModel { schema, rows }
    }
}

fn order_result_row(index: usize) -> Vec<String> {
    let customer = SAMPLE_CUSTOMERS
        .get(index % SAMPLE_CUSTOMERS.len())
        .copied()
        .unwrap_or("Sample");
    let status = ORDER_STATUSES
        .get(index % ORDER_STATUSES.len())
        .copied()
        .unwrap_or("pending");
    // Reduce the index before multiplying to keep arbitrary row counts safe.
    let total_whole = 40 + ((index % 900) * 37) % 900;
    let total_fraction = ((index % 100) * 13) % 100;
    let day = 17_usize.saturating_sub(index / 6);
    let hour = 23_usize.saturating_sub(index % 24);
    let minute = ((index % 60) * 7) % 60;

    vec![
        (10_480_usize.saturating_sub(index.saturating_mul(7))).to_string(),
        format!("{customer} Example"),
        status.to_owned(),
        format!("{total_whole}.{total_fraction:02}"),
        format!("2026-10-{day:02} {hour:02}:{minute:02}"),
    ]
}

#[derive(Clone, Debug)]
struct Document {
    key: ItemKey,
    title: String,
    editor: CodeEditorState,
    result: ResultView,
    source_table: Option<SampleTable>,
}

impl Document {
    fn new(key: ItemKey, title: String, sql: &str) -> Self {
        Self {
            key,
            title,
            editor: CodeEditorState::new(sql),
            result: ResultView::NotRun,
            source_table: None,
        }
    }

    fn for_sample_table(key: ItemKey, title: String, sql: &str, source_table: SampleTable) -> Self {
        let mut document = Self::new(key, title, sql);
        document.source_table = Some(source_table);
        document
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.title)
    }
}

#[derive(Clone, Debug)]
struct PendingRun {
    owner: ItemKey,
    explain: bool,
    remaining_ticks: u8,
}

#[derive(Clone, Debug, Default)]
struct ResultModel {
    schema: ResultSchema,
    rows: Vec<Vec<String>>,
}

impl GridModel for ResultModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::num(u64::try_from(row).unwrap_or(u64::MAX).saturating_add(1))
    }

    fn cell(&self, row: usize, col: usize) -> Option<termrock::CellRef<'_>> {
        Some(termrock::CellRef::new(self.rows.get(row)?.get(col)?))
    }

    fn row_decor(&self, row: usize) -> termrock::RowDecor<'_> {
        let mut decor = termrock::RowDecor::default();
        if self
            .rows
            .get(row)
            .is_some_and(|cells| cells.first().is_some_and(|field| field == "ERROR"))
        {
            decor.tone = Some(Role::Danger);
        }
        decor
    }
}

/// Query workbench page with native tabs, editor, toolbar, and results controls.
#[derive(Debug)]
pub struct QueryWorkbenchPage {
    documents: Vec<Document>,
    next_document_id: u64,
    tabs: TabsState,
    toolbar: ToolbarState,
    results: TableState,
    run: Option<PendingRun>,
    status: String,
    options: WorkbenchOptions,
    explorer: ExplorerState,
    explorer_split: SplitPaneState,
    split: SplitPaneState,
    options_state: termrock::FormDialogState,
    options_open: bool,
    options_close_pending: bool,
}

impl QueryWorkbenchPage {
    /// Creates a workbench with the sample orders and customers documents.
    pub fn new() -> Self {
        let orders_key = ItemKey::text("query.workbench.orders");
        let customers_key = ItemKey::text("query.workbench.customers");
        let mut orders = Document::for_sample_table(
            orders_key,
            "orders.sql".to_owned(),
            FIRST_QUERY,
            SampleTable::Orders,
        );
        orders.result = ResultView::Rows {
            table: "orders".to_owned(),
            count: 25,
        };
        let mut tabs = TabsState::default();
        tabs.set_active(0, orders_key);

        Self {
            documents: vec![
                orders,
                Document::for_sample_table(
                    customers_key,
                    "customers.sql".to_owned(),
                    SECOND_QUERY,
                    SampleTable::Customers,
                ),
            ],
            next_document_id: 2,
            tabs,
            toolbar: ToolbarState::default(),
            results: TableState::default(),
            run: None,
            status: "Ready · sample data only".to_owned(),
            options: WorkbenchOptions::default(),
            explorer: ExplorerState::new(),
            explorer_split: SplitPaneState::new(32),
            split: SplitPaneState::new(40),
            options_state: termrock::FormDialogState::default(),
            options_open: false,
            options_close_pending: false,
        }
    }

    /// Returns the number of open query documents.
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Returns stable document keys in tab order.
    pub fn document_keys(&self) -> impl Iterator<Item = ItemKey> + '_ {
        self.documents.iter().map(|document| document.key)
    }

    /// Returns the active native SQL editor state, when a document is open.
    pub fn active_editor(&self) -> Option<&CodeEditorState> {
        self.active_index()
            .and_then(|index| self.documents.get(index))
            .map(|document| &document.editor)
    }

    /// Returns the active document's SQL, when a document is open.
    pub fn active_sql(&self) -> Option<&str> {
        self.active_editor().map(|editor| editor.text())
    }

    /// Returns the active document's result state, when a document is open.
    pub fn active_result_state(&self) -> Option<&QueryResultState> {
        self.active_index()
            .and_then(|index| self.documents.get(index))
            .map(|document| &document.result)
    }

    /// Returns the native tabs state.
    pub fn tabs_state(&self) -> &TabsState {
        &self.tabs
    }

    /// Returns the native toolbar state.
    pub fn toolbar_state(&self) -> &ToolbarState {
        &self.toolbar
    }

    /// Returns the native results table state.
    pub fn results_state(&self) -> &TableState {
        &self.results
    }

    /// Returns whether a query is counting down to completion.
    pub fn is_running(&self) -> bool {
        self.run.is_some()
    }

    fn new_document(&mut self) -> bool {
        if !self.options.allow_new() {
            return false;
        }
        let title = format!("Query {}", self.next_document_id);
        if self.append_document(title, "", None) {
            true
        } else {
            self.status = "Cannot open another query tab.".to_owned();
            false
        }
    }

    fn open_sample_table(&mut self, table: SampleTable) -> bool {
        if let Some(index) = self
            .documents
            .iter()
            .position(|document| document.source_table == Some(table))
        {
            let Some(key) = self.documents.get(index).map(|document| document.key) else {
                return false;
            };
            self.tabs.set_active(index, key);
            return true;
        }

        let title = format!("{}.sql", table.name());
        let sql = format!("SELECT * FROM {} LIMIT 100;\n", table.name());
        self.append_document(title, &sql, Some(table))
    }

    fn append_document(
        &mut self,
        title: String,
        sql: &str,
        source_table: Option<SampleTable>,
    ) -> bool {
        let Some(key) = self.allocate_document_key() else {
            return false;
        };
        let document = match source_table {
            Some(table) => Document::for_sample_table(key, title, sql, table),
            None => Document::new(key, title, sql),
        };
        self.documents.push(document);
        self.tabs.set_active(self.documents.len() - 1, key);
        if self.run.is_none() {
            self.status = "Ready · sample data only".to_owned();
        }
        true
    }

    fn allocate_document_key(&mut self) -> Option<ItemKey> {
        let document_id = self.next_document_id;
        let next_document_id = document_id.checked_add(1)?;
        self.next_document_id = next_document_id;
        Some(ItemKey::num(document_id))
    }

    #[cfg(test)]
    fn apply_explorer_action(&mut self, action: TreeAction, explorer: &ExplorerState) -> bool {
        let TreeAction::Activated(key) = action else {
            return false;
        };
        let Some(table) = explorer.sample_table_for(key) else {
            return false;
        };
        self.apply_sample_table(table)
    }

    fn apply_sample_table(&mut self, table: SampleTable) -> bool {
        let opened = self.open_sample_table(table);
        if self.run.is_none() {
            self.status = if opened {
                format!("Opened {} from Explorer.", table.name())
            } else {
                "Cannot open another query tab.".to_owned()
            };
        }
        opened
    }

    fn active_index(&self) -> Option<usize> {
        let active = self.tabs.active()?;
        self.documents
            .iter()
            .position(|document| document.key == active)
    }

    fn toolbar(&self) -> Toolbar {
        let running = self.run.is_some();
        let run = ToolItem::new("Run")
            .shortcut("Ctrl+R")
            .tip("Run the current SQL");
        let explain = ToolItem::new("Explain").tip("Show the query plan");
        let stop = ToolItem::new("Stop").tip("Cancel the running query");
        let run = if running {
            run.reason("A query is running")
        } else {
            run
        };
        let explain = if running {
            explain.reason("A query is running")
        } else {
            explain
        };
        let stop = if running {
            stop
        } else {
            stop.reason("No query is running")
        };
        Toolbar::new(
            TOOLBAR,
            vec![
                run,
                explain,
                stop,
                ToolItem::new("Clear").tip("Clear this query"),
                ToolItem::new("Options").tip("Configure this query workbench"),
            ],
        )
    }

    fn explorer_results_split(&self) -> SplitPane<'static> {
        SplitPane::new(EXPLORER_SPLIT, SplitAxis::Horizontal)
            .min_first(22)
            .min_second(40)
            .gap(1)
            .resizable(true)
            .reset_policy(ResetPolicy::Percent(self.options.explorer_share()))
            .undersized_fallback(SplitFallback::second_only())
    }

    fn set_explorer_visible(&mut self, visible: bool) {
        let maximized = self.explorer_split.maximized();
        if visible == (maximized == Maximized::Second) {
            self.explorer_split.toggle_max(Maximized::Second);
        }
    }

    fn explorer_is_effectively_visible(&self, cx: &Cx<'_>) -> bool {
        let area = cx.area(EXPLORER_SPLIT).unwrap_or(Rect::ZERO);
        self.options.show_explorer()
            && self
                .explorer_results_split()
                .resolved_for(&self.explorer_split, area)
                .effective_maximized(&self.explorer_split)
                != Maximized::Second
    }

    fn editor_results_split(&self) -> SplitPane<'static> {
        SplitPane::new(SPLIT, SplitAxis::Vertical)
            .min_first(5)
            .min_second(6)
            .gap(1)
            .resizable(true)
            .reset_policy(ResetPolicy::Percent(self.options.editor_share()))
            .undersized_fallback(SplitFallback::new(3, 2).gap(1))
    }

    fn open_options(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(OPTIONS_DIALOG) {
            self.options_open = true;
            return;
        }
        let data = self.options.dialog_data();
        self.options_close_pending = false;
        cx.open_layer(OPTIONS_DIALOG, options::dialog().layer(cx, &data));
        self.options_open = cx.is_open(OPTIONS_DIALOG);
    }

    fn set_option_value(&mut self, key: &str, value: usize) -> bool {
        let Some(change) = self.options.set_option(key, value) else {
            return false;
        };
        self.apply_option_change(change);
        true
    }

    fn apply_option_change(&mut self, change: OptionChange) {
        match change {
            OptionChange::EditorShare => {
                self.split.set_percent(self.options.editor_share());
                return;
            }
            OptionChange::ExplorerShare => {
                self.explorer_split
                    .set_percent(self.options.explorer_share());
                return;
            }
            OptionChange::ShowExplorer(visible) => {
                self.set_explorer_visible(visible);
                return;
            }
            OptionChange::Result(_) => {}
            OptionChange::AllowNew
            | OptionChange::CloseReveal
            | OptionChange::ReadOnly
            | OptionChange::Environment => return,
        }
        let OptionChange::Result(fixture) = change else {
            return;
        };
        let Some(index) = self.active_index() else {
            return;
        };
        let Some(document) = self.documents.get_mut(index) else {
            return;
        };
        match fixture {
            ResultFixture::Run => {}
            ResultFixture::Empty => {
                document.editor.set_diagnostics(Vec::new());
                document.result = ResultView::Empty;
            }
            ResultFixture::Error => {
                document.editor.set_diagnostics(Vec::new());
                document.result = ResultView::Error {
                    message: "relation \"ordrs\" does not exist".to_owned(),
                    detail: "SQLSTATE 42P01 · line 3 · Did you mean \"orders\"?".to_owned(),
                };
            }
            ResultFixture::Plan => {
                document.editor.set_diagnostics(Vec::new());
                document.result = ResultView::Plan;
            }
        }
    }

    fn start_run(&mut self, cx: &mut Cx<'_>, explain: bool) -> PageStatus {
        if self.run.is_some() {
            return PageStatus(self.status.clone());
        }
        let Some(index) = self.active_index() else {
            self.status = "No query is open.".to_owned();
            return PageStatus(self.status.clone());
        };
        let Some(document) = self.documents.get_mut(index) else {
            self.status = "No query is open.".to_owned();
            return PageStatus(self.status.clone());
        };
        if document.editor.text().trim().is_empty() {
            document.result = ResultView::Empty;
            self.status = "Nothing to run · the SQL editor is empty.".to_owned();
            return PageStatus(self.status.clone());
        }
        let owner = document.key;
        self.run = Some(PendingRun {
            owner,
            explain,
            remaining_ticks: RUN_TICKS,
        });
        self.status = if explain {
            "Explaining query · 6 native ticks remaining".to_owned()
        } else {
            "Running query · 6 native ticks remaining".to_owned()
        };
        cx.request_repaint_after(Duration::from_millis(40));
        PageStatus(self.status.clone())
    }

    fn stop_run(&mut self) -> bool {
        if self.run.take().is_none() {
            return false;
        }
        self.status = "Run stopped.".to_owned();
        true
    }

    fn clear_active(&mut self) -> bool {
        let Some(index) = self.active_index() else {
            return false;
        };
        let Some(document) = self.documents.get_mut(index) else {
            return false;
        };
        document.editor.set_text("");
        self.status = "Editor and saved query cleared.".to_owned();
        true
    }

    fn close_document(&mut self, key: ItemKey) -> bool {
        let Some(index) = self
            .documents
            .iter()
            .position(|document| document.key == key)
        else {
            return false;
        };
        let was_active = self.tabs.active() == Some(key);
        self.documents.remove(index);
        let cancelled_run = self.run.as_ref().is_some_and(|run| run.owner == key);
        if cancelled_run {
            self.run = None;
            self.status = "Run cancelled · its query tab was closed.".to_owned();
        }
        if self.documents.is_empty() {
            self.tabs = TabsState::default();
            if !cancelled_run {
                self.status = "No query is open.".to_owned();
            }
        } else if was_active
            && let Some(next) = self
                .documents
                .get(index.min(self.documents.len().saturating_sub(1)))
        {
            self.tabs
                .set_active(index.min(self.documents.len().saturating_sub(1)), next.key);
        }
        true
    }

    fn apply_toolbar_action(&mut self, cx: &mut Cx<'_>, index: usize) -> Option<PageStatus> {
        match index {
            0 => Some(self.start_run(cx, false)),
            1 => Some(self.start_run(cx, true)),
            2 if self.stop_run() => Some(PageStatus(self.status.clone())),
            3 if self.clear_active() => Some(PageStatus(self.status.clone())),
            4 => {
                self.open_options(cx);
                None
            }
            _ => None,
        }
    }

    fn advance_run(&mut self, cx: &mut Cx<'_>) -> Option<PageStatus> {
        if cx.update_cause() != UpdateCause::Tick {
            return None;
        }
        let run = self.run.as_mut()?;
        run.remaining_ticks = run.remaining_ticks.saturating_sub(1);
        if run.remaining_ticks > 0 {
            let verb = if run.explain {
                "Explaining query"
            } else {
                "Running query"
            };
            self.status = format!("{verb} · {} native ticks remaining", run.remaining_ticks);
            cx.request_repaint_after(Duration::from_millis(40));
            return Some(PageStatus(self.status.clone()));
        }

        let run = self.run.take()?;
        if let Some(document) = self
            .documents
            .iter_mut()
            .find(|document| document.key == run.owner)
        {
            let evaluation = evaluate_query(document.editor.text(), run.explain);
            let status = evaluation.result.summary();
            document.result = evaluation.result;
            document.editor.set_diagnostics(evaluation.diagnostics);
            self.status = status;
        } else {
            self.status = "Run cancelled · its query tab was closed.".to_owned();
        }
        Some(PageStatus(self.status.clone()))
    }

    fn active_result_model(&self) -> ResultModel {
        self.active_index()
            .and_then(|index| self.documents.get(index))
            .map_or_else(ResultModel::default, |document| document.result.model())
    }

    fn apply_tabs_action(&mut self, action: TabsAction) -> Option<PageStatus> {
        match action {
            TabsAction::Close(key) if self.close_document(key) => {
                Some(PageStatus(self.status.clone()))
            }
            TabsAction::New if self.options.allow_new() => {
                let _ = self.new_document();
                Some(PageStatus(self.status.clone()))
            }
            TabsAction::New | TabsAction::Activated(_) | TabsAction::Close(_) => None,
        }
    }
}

impl Default for QueryWorkbenchPage {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a fresh query workbench page behind the native page interface.
pub fn page() -> Box<dyn Page> {
    Box::new(QueryWorkbenchPage::new())
}

impl Page for QueryWorkbenchPage {
    fn title(&self) -> &'static str {
        "Query workbench"
    }

    fn quit_confirmation(&self) -> Option<&'static str> {
        self.run
            .as_ref()
            .map(|_| "The query is still running. Leaving stops it.")
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        self.options.option_value(key)
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let _ = self.set_option_value(key, value);
    }

    fn global_capture_owners(&self, action: ActionKey) -> &'static [Id] {
        if action == RUN_QUERY {
            &RUN_CAPTURE_OWNERS
        } else {
            &[]
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        let options_were_close_pending = core::mem::take(&mut self.options_close_pending);
        if options_were_close_pending {
            let mut data = self.options.dialog_data();
            response |= options::dialog()
                .update(cx, &mut self.options_state, &mut data)
                .erase();
        }

        let options_were_open = self.options_open || cx.is_open(OPTIONS_DIALOG);
        if options_were_open {
            let mut data = self.options.dialog_data();
            let update = options::dialog().update(cx, &mut self.options_state, &mut data);
            let action = update.action_ref().copied();
            response |= update.erase();
            if let Some(termrock::FormDialogAction::Committed(id)) = action
                && let Some((key, value)) = data.option_for(id)
            {
                let _ = self.set_option_value(key, value);
            }
            self.options_open = cx.is_open(OPTIONS_DIALOG);
            if !self.options_open {
                self.options_close_pending = true;
            }
        } else if !options_were_close_pending {
            self.options_open = false;
            response |= self
                .explorer_results_split()
                .update(cx, &mut self.explorer_split)
                .erase();
            response |= self
                .editor_results_split()
                .update(cx, &mut self.split)
                .erase();
            let tabs = Tabs::new(TABS)
                .key(|document: &Document| document.key)
                .allow_new(self.options.allow_new())
                .close_reveal(self.options.close_reveal())
                .closable(true);
            let tab_update = tabs.update(cx, &mut self.tabs, &self.documents);
            let tab_action = tab_update.action_ref().copied();
            response |= tab_update.erase();
            if let Some(action) = tab_action
                && let Some(next_status) = self.apply_tabs_action(action)
            {
                response |= Response::changed();
                status = Some(next_status);
            }

            if let Some(index) = self.active_index()
                && let Some(document) = self.documents.get_mut(index)
            {
                let update = CodeEditor::new(EDITOR, 12)
                    .fill_height(true)
                    .placeholder("Write a query against the sample data")
                    .read_only(self.options.read_only())
                    .update(cx, &mut document.editor);
                let action = update.action_ref().cloned();
                response |= update.erase();
                if matches!(action, Some(CodeAction::Changed)) {
                    document.editor.set_diagnostics(Vec::new());
                }
            }

            let toolbar = self.toolbar();
            let toolbar_update = toolbar.update(cx, &mut self.toolbar);
            let actions = toolbar_update
                .action_ref()
                .map(|actions| actions.as_slice().to_vec())
                .unwrap_or_default();
            response |= toolbar_update.erase();
            for action in actions {
                if let ToolbarAction::Activated(index) = action.action
                    && let Some(next_status) = self.apply_toolbar_action(cx, index)
                {
                    status = Some(next_status);
                    response |= Response::changed();
                }
            }

            let model = self.active_result_model();
            response |= Table::new(RESULTS, model.schema.columns())
                .update(cx, &mut self.results, &model)
                .erase();
        } else {
            self.options_open = false;
        }

        let explorer_controls_allowed = !options_were_open
            && !options_were_close_pending
            && !self.options_open
            && !self.options_close_pending
            && !cx.is_open(OPTIONS_DIALOG)
            && self.explorer_is_effectively_visible(cx);
        if explorer_controls_allowed {
            let update = self.explorer.update(cx);
            response |= update.response;
            if update.filter_action.is_some() {
                response |= Response::changed();
            }
            if let Some(TreeAction::Activated(key)) = update.tree_action
                && let Some(table) = self.explorer.sample_table_for(key)
            {
                let _ = self.apply_sample_table(table);
                response |= Response::changed();
                status = Some(PageStatus(self.status.clone()));
            }
        } else {
            response |= self.explorer.inactive_update(cx);
        }

        if let Some(next_status) = self.advance_run(cx) {
            status = Some(next_status);
            response |= Response::changed();
        }

        PageUpdate { response, status }
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action != RUN_QUERY
            || self.run.is_some()
            || self.options_open
            || self.options_close_pending
            || cx.is_open(OPTIONS_DIALOG)
        {
            return Response::ignored();
        }
        let _ = self.start_run(cx, false);
        Response::changed()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let title_width = termrock::width(self.title());
        frame(ui, area, self.title(), "", |ui, body| {
            let rows = layout::rows(body, &[layout::Track::Flex(1), layout::Track::Fixed(1)]);
            let work_area = rows.first().copied().unwrap_or(Rect::new(0, 0, 0, 0));
            let status_area = rows.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
            let model = self.active_result_model();
            let summary = self
                .active_index()
                .and_then(|index| self.documents.get(index))
                .map_or_else(
                    || "No query is open.".to_owned(),
                    |document| document.result.summary(),
                );

            self.explorer_results_split().draw(
                ui,
                work_area,
                &self.explorer_split,
                |ui, explorer_area, main_area| {
                    Panel::new(EXPLORER_PANEL)
                        .kind(PanelKind::Framed)
                        .title("Explorer")
                        .draw(ui, explorer_area, |ui, inner| {
                            self.explorer.draw(ui, inner);
                        });

                    let main_rows = layout::rows(
                        main_area,
                        &[
                            layout::Track::Fixed(1),
                            layout::Track::Fixed(1),
                            layout::Track::Flex(1),
                        ],
                    );
                    let tabs_area = main_rows.first().copied().unwrap_or(Rect::new(0, 0, 0, 0));
                    let toolbar_area = main_rows.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
                    let editor_results_area =
                        main_rows.get(2).copied().unwrap_or(Rect::new(0, 0, 0, 0));

                    Tabs::new(TABS)
                        .key(|document: &Document| document.key)
                        .allow_new(self.options.allow_new())
                        .close_reveal(self.options.close_reveal())
                        .closable(true)
                        .draw(ui, tabs_area, &self.tabs, &self.documents);
                    let _ = self.toolbar().draw(ui, toolbar_area, &self.toolbar);

                    self.editor_results_split().draw(
                        ui,
                        editor_results_area,
                        &self.split,
                        |ui, editor_area, results_area| {
                            if let Some(document) = self
                                .active_index()
                                .and_then(|index| self.documents.get(index))
                            {
                                Panel::new(EDITOR_PANEL)
                                    .kind(PanelKind::Framed)
                                    .title(&document.title)
                                    .draw(ui, editor_area, |ui, inner| {
                                        let _ = CodeEditor::new(EDITOR, inner.height)
                                            .fill_height(true)
                                            .placeholder("Write a query against the sample data")
                                            .read_only(self.options.read_only())
                                            .draw(ui, inner, &document.editor);
                                    });
                            }

                            Panel::new(RESULTS_PANEL)
                                .kind(PanelKind::Framed)
                                .title("Results")
                                .draw(ui, results_area, |ui, inner| {
                                    let result_rows = layout::rows(
                                        inner,
                                        &[layout::Track::Fixed(1), layout::Track::Flex(1)],
                                    );
                                    if let Some(summary_area) = result_rows.first()
                                        && !summary_area.is_empty()
                                    {
                                        let _ = ui.paint_str(
                                            *summary_area,
                                            &summary,
                                            ui.surface_style(),
                                        );
                                    }
                                    let table_area = result_rows
                                        .get(1)
                                        .copied()
                                        .unwrap_or(Rect::new(0, 0, 0, 0));
                                    let _ = Table::new(RESULTS, model.schema.columns()).draw(
                                        ui,
                                        table_area,
                                        &self.results,
                                        &model,
                                    );
                                });
                        },
                    );
                },
            );

            if !status_area.is_empty() {
                let _ = ui.paint_str(status_area, &self.status, ui.surface_style());
            }
        });
        let header_row = termrock::runtime::first_row(area);
        let fixed_title_prefix = title_width.saturating_add(2);
        let header_columns = [
            layout::ColumnSpec::new(
                fixed_title_prefix,
                fixed_title_prefix,
                fixed_title_prefix,
                0,
                None,
            ),
            layout::ColumnSpec::new(2, 2, u16::MAX, 1, Some(0)),
            layout::ColumnSpec::new(1, 1, 1, 0, None),
        ];
        let [title_prefix_width, status_width] =
            layout::solve_columns_head(&header_columns, header_row.width, 0).unwrap_or([0, 0]);
        let status_area = Rect {
            x: header_row.x.saturating_add(title_prefix_width),
            width: status_width,
            ..header_row
        };
        let (environment_label, environment_role) = self.options.environment_status();
        let environment = [StatusItem::new(environment_label)
            .tone(environment_role)
            .priority(9)];
        let sample = [StatusItem::new("sample data · no database connection").priority(1)];
        StatusBar::new(ENVIRONMENT_STATUS)
            .left(&environment)
            .right(&sample)
            .draw(ui, status_area);
        if self.options_open {
            let data = self.options.dialog_data();
            ui.layer(OPTIONS_DIALOG, |ui, layer| {
                options::dialog().draw(ui, layer, &self.options_state, &data);
            });
        }
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(TABS).contains(StateFlags::FOCUSED) {
            return TABS_HINTS;
        }
        if ui.state(TOOLBAR).contains(StateFlags::FOCUSED) {
            return TOOLBAR_HINTS;
        }
        if ui.state(EXPLORER_SPLIT).contains(StateFlags::FOCUSED) {
            return EXPLORER_SPLIT_HINTS;
        }
        if ui.state(explorer::FILTER).contains(StateFlags::FOCUSED) {
            return FILTER_HINTS;
        }
        if ui.state(explorer::TREE).contains(StateFlags::FOCUSED) {
            return TREE_HINTS;
        }
        if ui.state(SPLIT).contains(StateFlags::FOCUSED) {
            return SPLIT_HINTS;
        }
        if ui.state(EDITOR).contains(StateFlags::FOCUSED) {
            return EDITOR_HINTS;
        }
        if ui.state(RESULTS).contains(StateFlags::FOCUSED) {
            return RESULTS_HINTS;
        }
        &[]
    }

    fn editing(&self, ui: &Ui<'_>) -> bool {
        (self.explorer.is_filter_editing()
            && ui.state(explorer::FILTER).contains(StateFlags::FOCUSED))
            || self
                .active_index()
                .and_then(|index| self.documents.get(index))
                .is_some_and(|document| document.editor.is_editing())
    }
}

struct QueryEvaluation {
    result: ResultView,
    diagnostics: Vec<CodeDiagnostic>,
}

fn evaluate_query(sql: &str, explain: bool) -> QueryEvaluation {
    if sql.trim().is_empty() {
        return QueryEvaluation {
            result: ResultView::Empty,
            diagnostics: Vec::new(),
        };
    }
    let Some((table, range)) = from_table(sql) else {
        return QueryEvaluation {
            result: ResultView::Error {
                message: "Syntax error".to_owned(),
                detail: "Expected a FROM clause.".to_owned(),
            },
            diagnostics: Vec::new(),
        };
    };
    let known = [
        "organizations",
        "customers",
        "products",
        "orders",
        "order_items",
        "payments",
        "subscriptions",
    ];
    if !known.iter().any(|known| table.eq_ignore_ascii_case(known)) {
        return QueryEvaluation {
            result: ResultView::Error {
                message: format!("relation \"{table}\" does not exist"),
                detail: "Check the table name in the current sample schema.".to_owned(),
            },
            diagnostics: vec![CodeDiagnostic::new(
                range,
                CodeSeverity::Error,
                "unknown relation",
            )],
        };
    }
    if explain {
        return QueryEvaluation {
            result: ResultView::Plan,
            diagnostics: Vec::new(),
        };
    }
    let count = if sql.to_ascii_lowercase().contains("limit 25") {
        25
    } else {
        40
    };
    QueryEvaluation {
        result: ResultView::Rows {
            table: table.to_owned(),
            count,
        },
        diagnostics: Vec::new(),
    }
}

fn from_table(sql: &str) -> Option<(&str, Range<usize>)> {
    let bytes = sql.as_bytes();
    let from = bytes
        .windows(5)
        .position(|window| window.eq_ignore_ascii_case(b"from "))?
        .saturating_add(5);
    let mut start = from;
    while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start = start.saturating_add(1);
    }
    let end = bytes
        .get(start..)?
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        .map_or(bytes.len(), |offset| start.saturating_add(offset));
    if end <= start {
        return None;
    }
    let table = sql.get(start..end)?;
    Some((table, start..end))
}

#[cfg(test)]
mod tests {
    use super::{
        CodeDiagnostic, CodeSeverity, Document, EDITOR_ID, FIRST_QUERY, OPTIONS_DIALOG,
        OptionChange, PendingRun, QueryResultState, QueryWorkbenchPage, RESULTS_ID, RUN_QUERY,
        RUN_TICKS, ResultFixture, ResultView, SampleTable, TABS_ID, TOOLBAR, TOOLBAR_ID,
        evaluate_query, options,
    };
    use crate::pages::Page;
    use ratatui::layout::Rect;
    use termrock::{
        ActionKey, App as TermrockApp, Chord, Cx, GridModel, Id, ItemKey, KeyCode, KeyMap,
        KeyModifiers, KeyPhase, Part, PartRef, Response, TabsAction, Theme, TreeAction, Ui,
    };
    use termrock_test_support::Harness;

    const OPTIONS_ALLOW_NEW_FIELD: Id = OPTIONS_DIALOG.part(Part::BODY).part(Part::FIELD).index(0);
    const OPTIONS_READ_ONLY_FIELD: Id = OPTIONS_DIALOG.part(Part::BODY).part(Part::FIELD).index(2);

    #[derive(Debug)]
    struct Host {
        page: QueryWorkbenchPage,
        area: Rect,
        keymap: KeyMap,
        last_command: Option<(ActionKey, bool)>,
        open_options_before_update: bool,
    }

    impl TermrockApp for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if std::mem::take(&mut self.open_options_before_update) {
                self.page.open_options(cx);
            }
            let command = cx.command();
            let mut response = self.page.update(cx).response;
            if let Some(action) = command {
                let command_response = self.page.command(cx, action);
                self.last_command = Some((action, command_response.is_changed()));
                response |= command_response;
            }
            response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, self.area);
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn host_keymap() -> KeyMap {
        KeyMap::new().bind(
            KeyPhase::Capture,
            Chord::with(KeyCode::Char('r'), KeyModifiers::CONTROL),
            RUN_QUERY,
        )
    }

    fn overflow_harness() -> Harness<Host> {
        Harness::new(
            Host {
                page: QueryWorkbenchPage::new(),
                area: Rect::new(0, 0, 14, 18),
                keymap: host_keymap(),
                last_command: None,
                open_options_before_update: false,
            },
            Theme::junie(),
            14,
            18,
        )
    }

    fn harness_with_page(page: QueryWorkbenchPage) -> Harness<Host> {
        Harness::new(
            Host {
                page,
                area: Rect::new(0, 0, 160, 40),
                keymap: host_keymap(),
                last_command: None,
                open_options_before_update: false,
            },
            Theme::junie(),
            160,
            40,
        )
    }

    fn native_harness() -> Harness<Host> {
        harness_with_page(QueryWorkbenchPage::new())
    }

    fn open_options(h: &mut Harness<Host>) {
        let options = PartRef::item(Part::ROW, ItemKey::index(4));
        assert!(h.area_of_part(TOOLBAR, options).is_some());
        let _ = h.click_part(TOOLBAR, options);
        assert!(h.is_open(OPTIONS_DIALOG));
    }

    fn more_menu() -> Id {
        TOOLBAR.sub("more-menu")
    }

    fn menu_row(h: &Harness<Host>, index: usize) -> Option<String> {
        let area = h.area_of_part(more_menu(), PartRef::item(Part::ROW, ItemKey::index(index)))?;
        let mut text = String::new();
        for x in area.x..area.right() {
            text.push_str(h.cell(x, area.y).symbol());
        }
        Some(text)
    }

    fn activate_overflow_action(h: &mut Harness<Host>, label: &str) {
        assert!(
            h.area_of_part(TOOLBAR, PartRef::of(Part::OVERFLOW))
                .is_some(),
            "the narrow Toolbar exposes its native More part"
        );
        let _ = h.click_part(TOOLBAR, PartRef::of(Part::OVERFLOW));
        let menu = more_menu();
        assert!(h.is_open(menu), "native Toolbar overflow menu opens");
        let index = (0..8)
            .find(|index| menu_row(h, *index).is_some_and(|row| row.contains(label)))
            .unwrap_or_else(|| panic!("overflow action {label:?} is absent\n{}", h.text()));
        let _ = h.click_part(menu, PartRef::item(Part::ROW, ItemKey::index(index)));
    }

    #[test]
    fn public_factory_and_accessors_expose_native_document_state() {
        let factory_page = super::page();
        assert_eq!(factory_page.title(), "Query workbench");

        let mut page = QueryWorkbenchPage::new();
        let keys: Vec<_> = page.document_keys().collect();
        assert_eq!(page.document_count(), 2);
        assert_eq!(keys.len(), 2);
        assert_eq!(page.tabs_state().active(), Some(keys[0]));
        assert_eq!(page.documents[0].source_table, Some(SampleTable::Orders));
        assert_eq!(page.documents[1].source_table, Some(SampleTable::Customers));
        assert_eq!(page.active_sql(), Some(super::FIRST_QUERY));
        assert_eq!(
            page.active_editor().map(|editor| editor.text()),
            Some(super::FIRST_QUERY)
        );
        assert!(matches!(
            page.active_result_state(),
            Some(QueryResultState::Rows { table, count })
                if table == "orders" && *count == 25
        ));
        assert_eq!(
            page.documents[0].result.summary(),
            "25 rows from orders · simulated in 15 ms"
        );
        assert!(!page.is_running());
        let _ = page.toolbar_state();
        let _ = page.results_state();
        assert_eq!(
            TABS_ID,
            Id::root("showcase_ui::pages::query_workbench.tabs")
        );
        assert_eq!(TOOLBAR_ID, TOOLBAR);
        assert_eq!(
            EDITOR_ID,
            Id::root("showcase_ui::pages::query_workbench.editor")
        );
        assert_eq!(
            RESULTS_ID,
            Id::root("showcase_ui::pages::query_workbench.results")
        );

        page.tabs.set_active(1, keys[1]);
        assert_eq!(page.active_sql(), Some(super::SECOND_QUERY));
        assert!(matches!(
            page.active_result_state(),
            Some(QueryResultState::NotRun)
        ));
    }

    #[test]
    fn new_documents_keep_unique_keys_after_close_and_empty_state_recovery() {
        let mut page = QueryWorkbenchPage::new();
        let orders_key = ItemKey::text("query.workbench.orders");
        let customers_key = ItemKey::text("query.workbench.customers");
        let first_new_key = ItemKey::num(2);
        let second_new_key = ItemKey::num(3);

        assert!(page.new_document());
        assert_eq!(page.document_count(), 3);
        assert_eq!(page.tabs_state().active(), Some(first_new_key));
        assert_eq!(page.documents[2].title, "Query 2");
        assert_eq!(page.documents[2].source_table, None);
        assert_eq!(page.active_sql(), Some(""));
        assert!(matches!(
            page.active_result_state(),
            Some(QueryResultState::NotRun)
        ));
        assert_ne!(first_new_key, ItemKey::index(2));

        assert!(page.close_document(first_new_key));
        assert!(page.new_document());
        assert_eq!(page.tabs_state().active(), Some(second_new_key));
        assert!(!page.document_keys().any(|key| key == first_new_key));
        assert!(page.document_keys().any(|key| key == second_new_key));

        assert!(page.close_document(orders_key));
        assert!(page.close_document(customers_key));
        assert!(page.close_document(second_new_key));
        assert_eq!(page.document_count(), 0);
        assert_eq!(page.tabs_state().active(), None);
        assert_eq!(page.active_editor(), None);
        assert_eq!(page.active_result_state(), None);
        assert_eq!(page.status, "No query is open.");

        assert!(page.new_document());
        assert_eq!(page.document_keys().collect::<Vec<_>>(), [ItemKey::num(4)]);
        assert_eq!(page.active_sql(), Some(""));
        assert!(matches!(
            page.active_result_state(),
            Some(QueryResultState::NotRun)
        ));
    }

    #[test]
    fn query_workbench_simulates_empty_rows_plan_and_error_results() {
        assert_eq!(evaluate_query("  \n", false).result, ResultView::Empty);
        assert_eq!(
            evaluate_query("SELECT id FROM orders LIMIT 25", false).result,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 25,
            }
        );
        assert_eq!(
            evaluate_query("SELECT id FROM orders", true).result,
            ResultView::Plan
        );
        let default_rows = evaluate_query("SELECT id FROM orders", false).result;
        assert_eq!(
            default_rows,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 40,
            }
        );
        assert_eq!(default_rows.model().row_count(), 40);
        let limited_rows = evaluate_query("SELECT id FROM orders LIMIT 25", false).result;
        assert_eq!(limited_rows.model().row_count(), 25);
        let page = QueryWorkbenchPage::new();
        assert_eq!(
            page.documents[0].result,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 25,
            }
        );
        assert_eq!(page.documents[0].result.model().row_count(), 25);
        assert_eq!(page.documents[1].result, ResultView::NotRun);
        assert!(matches!(
            evaluate_query("SELECT * FROM missing_table", false).result,
            ResultView::Error { .. }
        ));
        assert!(matches!(
            evaluate_query("SELECT 1", false).result,
            ResultView::Error { .. }
        ));
    }

    #[test]
    fn unknown_relation_evaluation_returns_exact_byte_range_diagnostic() {
        let sql = "-- café\nSELECT id FROM missing_table;";
        let evaluation = evaluate_query(sql, false);
        assert_eq!(
            evaluation.result,
            ResultView::Error {
                message: "relation \"missing_table\" does not exist".to_owned(),
                detail: "Check the table name in the current sample schema.".to_owned(),
            }
        );
        assert_eq!(evaluation.diagnostics.len(), 1);
        let diagnostic = &evaluation.diagnostics[0];
        assert_eq!(diagnostic.range, 24..37);
        assert_eq!(sql.find("missing_table"), Some(diagnostic.range.start));
        assert_eq!(sql.get(diagnostic.range.clone()), Some("missing_table"));
        assert_eq!(diagnostic.severity, CodeSeverity::Error);
        assert_eq!(diagnostic.message, "unknown relation");
        assert!(
            evaluate_query("SELECT * FROM orders;", false)
                .diagnostics
                .is_empty(),
            "known sample relations have no editor diagnostic"
        );
    }

    #[test]
    fn native_editor_change_clears_completed_relation_diagnostic() {
        let mut h = native_harness();
        h.app_mut().page.documents[0]
            .editor
            .set_text("SELECT * FROM missing_table;");
        assert!(h.tab_to(EDITOR_ID));
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        h.ticks(6);
        assert_eq!(
            h.app().page.documents[0].editor.diagnostics()[0].message,
            "unknown relation"
        );

        assert!(h.tab_to(EDITOR_ID));
        let _ = h.paste(" ");
        assert!(
            h.app().page.documents[0].editor.diagnostics().is_empty(),
            "CodeAction::Changed clears diagnostics for the edited document"
        );
    }

    #[test]
    fn successful_run_clears_diagnostics_from_the_document() {
        let mut h = native_harness();
        h.app_mut().page.documents[0]
            .editor
            .set_diagnostics(vec![CodeDiagnostic::new(
                0..6,
                CodeSeverity::Error,
                "previous relation error",
            )]);
        assert!(h.tab_to(EDITOR_ID));
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        h.ticks(6);

        assert!(matches!(
            h.app().page.documents[0].result,
            ResultView::Rows { .. }
        ));
        assert!(h.app().page.documents[0].editor.diagnostics().is_empty());
    }

    #[test]
    fn error_fixture_preserves_its_message_without_a_false_editor_diagnostic() {
        let mut page = QueryWorkbenchPage::new();
        page.documents[0].editor.set_text("SELECT * FROM orders;");
        page.documents[0]
            .editor
            .set_diagnostics(vec![CodeDiagnostic::new(
                14..20,
                CodeSeverity::Error,
                "old diagnostic",
            )]);
        page.apply_option_change(OptionChange::Result(ResultFixture::Error));

        assert_eq!(
            page.documents[0].result,
            ResultView::Error {
                message: "relation \"ordrs\" does not exist".to_owned(),
                detail: "SQLSTATE 42P01 · line 3 · Did you mean \"orders\"?".to_owned(),
            }
        );
        assert!(page.documents[0].editor.diagnostics().is_empty());
    }

    #[test]
    fn run_completion_attaches_diagnostics_to_the_original_tab_after_switching() {
        let mut page = QueryWorkbenchPage::new();
        let owner = page.documents[0].key;
        let sql = "SELECT * FROM missing_table;";
        page.documents[0].editor.set_text(sql);
        page.run = Some(PendingRun {
            owner,
            explain: false,
            remaining_ticks: 1,
        });
        let customers = page.documents[1].key;
        page.tabs.set_active(1, customers);
        let mut h = harness_with_page(page);
        let _ = h.tick();

        assert_eq!(h.app().page.tabs.active(), Some(customers));
        let owner_diagnostics = h
            .app()
            .page
            .documents
            .iter()
            .find(|document| document.key == owner)
            .map(|document| document.editor.diagnostics());
        assert_eq!(
            owner_diagnostics,
            Some(
                [CodeDiagnostic::new(
                    14..27,
                    CodeSeverity::Error,
                    "unknown relation",
                )]
                .as_slice()
            )
        );
        assert!(h.app().page.documents[1].editor.diagnostics().is_empty());
    }

    #[test]
    fn editing_during_run_uses_current_text_and_shifts_relation_diagnostic() {
        let mut h = native_harness();
        let owner = h.app().page.documents[0].key;
        h.app_mut().page.documents[0]
            .editor
            .set_text("SELECT * FROM missing_table;");
        assert!(h.tab_to(EDITOR_ID));
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(
            h.app()
                .page
                .run
                .as_ref()
                .map(|run| (run.owner, run.remaining_ticks)),
            Some((owner, RUN_TICKS))
        );

        let _ = h.key(KeyCode::Home);
        let _ = h.paste(" ");
        assert_eq!(
            h.app().page.documents[0].editor.text(),
            " SELECT * FROM missing_table;",
            "the native edit inserts one leading space"
        );
        assert_eq!(
            h.app()
                .page
                .run
                .as_ref()
                .map(|run| (run.owner, run.remaining_ticks)),
            Some((owner, RUN_TICKS)),
            "editing neither changes run ownership nor advances its countdown"
        );
        h.ticks(6);

        assert_eq!(
            h.app().page.documents[0].result,
            ResultView::Error {
                message: "relation \"missing_table\" does not exist".to_owned(),
                detail: "Check the table name in the current sample schema.".to_owned(),
            }
        );
        let diagnostics = h.app().page.documents[0].editor.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range, 15..28);
        assert_eq!(diagnostics[0].severity, CodeSeverity::Error);
        assert_eq!(diagnostics[0].message, "unknown relation");
    }

    #[test]
    fn editing_during_run_to_known_relation_completes_without_diagnostics() {
        let mut h = native_harness();
        h.app_mut().page.documents[0]
            .editor
            .set_text("SELECT * FROM missing_table;");
        assert!(h.tab_to(EDITOR_ID));
        let owner = h.app().page.documents[0].key;
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(
            h.app()
                .page
                .run
                .as_ref()
                .map(|run| (run.owner, run.remaining_ticks)),
            Some((owner, RUN_TICKS))
        );

        h.app_mut().page.documents[0]
            .editor
            .set_text("SELECT * FROM orders;");
        assert_eq!(
            h.app().page.documents[0].editor.text(),
            "SELECT * FROM orders;"
        );
        assert_eq!(
            h.app()
                .page
                .run
                .as_ref()
                .map(|run| (run.owner, run.remaining_ticks)),
            Some((owner, RUN_TICKS)),
            "editing leaves the original owner and pending countdown intact"
        );
        h.ticks(6);

        assert_eq!(
            h.app().page.documents[0].result,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 40,
            }
        );
        assert!(h.app().page.documents[0].editor.diagnostics().is_empty());
    }

    #[test]
    fn native_toolbar_overflow_activates_run_explain_stop_and_clear() {
        let mut run = overflow_harness();
        activate_overflow_action(&mut run, "Run");
        assert_eq!(
            run.app().page.run.as_ref().map(|run| run.remaining_ticks),
            Some(6)
        );
        run.ticks(6);
        assert_eq!(
            run.app().page.documents[0].result,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 25,
            }
        );

        let mut explain = overflow_harness();
        activate_overflow_action(&mut explain, "Explain");
        assert!(
            explain
                .app()
                .page
                .run
                .as_ref()
                .is_some_and(|run| run.explain)
        );
        explain.ticks(6);
        assert_eq!(explain.app().page.documents[0].result, ResultView::Plan);

        let mut stop = overflow_harness();
        activate_overflow_action(&mut stop, "Run");
        activate_overflow_action(&mut stop, "Stop");
        stop.ticks(8);
        assert!(stop.app().page.run.is_none());
        assert_eq!(
            stop.app().page.documents[0].result,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 25,
            }
        );
        assert_eq!(stop.app().page.status, "Run stopped.");

        let mut clear = overflow_harness();
        activate_overflow_action(&mut clear, "Clear");
        assert!(clear.app().page.documents[0].editor.text().is_empty());
        assert_eq!(clear.app().page.status, "Editor and saved query cleared.");
        for h in [&run, &explain, &stop, &clear] {
            assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
        }
    }

    #[test]
    fn page_option_adapter_preserves_sql_owner_and_countdown_for_noop_and_valid_changes() {
        let mut h = native_harness();
        assert!(h.tab_to(EDITOR_ID));
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(
            h.app().last_command,
            Some((RUN_QUERY, true)),
            "the host routes a real Ctrl+R command to the page"
        );
        assert_eq!(
            h.app().page.run.as_ref().map(|run| run.remaining_ticks),
            Some(6)
        );
        h.ticks(2);

        let page = &h.app().page;
        let original_sql = page.active_sql().map(str::to_owned);
        let original_active = page.tabs.active();
        let original_run = page
            .run
            .as_ref()
            .map(|run| (run.owner, run.remaining_ticks));
        assert_eq!(original_run.map(|(_, ticks)| ticks), Some(4));
        let unchanged = format!("{page:?}");
        for (key, value) in [
            (options::ALLOW_NEW_KEY, 1),
            (options::ALLOW_NEW_KEY, 2),
            (options::CLOSE_REVEAL_KEY, 0),
            (options::CLOSE_REVEAL_KEY, 2),
            (options::READ_ONLY_KEY, 0),
            (options::READ_ONLY_KEY, 2),
            (options::RESULT_KEY, 0),
            (options::RESULT_KEY, 4),
            (options::ENVIRONMENT_KEY, 0),
            (options::ENVIRONMENT_KEY, 4),
            (options::EDITOR_SHARE_KEY, 0),
            (options::EDITOR_SHARE_KEY, 3),
            (options::SHOW_EXPLORER_KEY, 2),
            (options::EXPLORER_SHARE_KEY, 3),
            ("queryworkbench.unknown", 1),
        ] {
            Page::set_option(&mut h.app_mut().page, key, value);
            assert_eq!(format!("{:?}", h.app().page), unchanged, "{key}={value}");
            assert_eq!(h.app().page.active_sql(), original_sql.as_deref());
            assert_eq!(h.app().page.tabs.active(), original_active);
            assert_eq!(
                h.app()
                    .page
                    .run
                    .as_ref()
                    .map(|run| (run.owner, run.remaining_ticks)),
                original_run
            );
        }

        for (key, value) in [
            (options::ALLOW_NEW_KEY, 0),
            (options::CLOSE_REVEAL_KEY, 1),
            (options::READ_ONLY_KEY, 1),
            (options::ENVIRONMENT_KEY, 1),
            (options::EDITOR_SHARE_KEY, 1),
            (options::EDITOR_SHARE_KEY, 2),
            (options::EDITOR_SHARE_KEY, 0),
            (options::RESULT_KEY, 1),
            (options::RESULT_KEY, 2),
            (options::RESULT_KEY, 3),
            (options::RESULT_KEY, 0),
            (options::SHOW_EXPLORER_KEY, 0),
            (options::EXPLORER_SHARE_KEY, 1),
            (options::SHOW_EXPLORER_KEY, 1),
            (options::EXPLORER_SHARE_KEY, 2),
            (options::EXPLORER_SHARE_KEY, 0),
        ] {
            Page::set_option(&mut h.app_mut().page, key, value);
            assert_eq!(Page::option_value(&h.app().page, key), Some(value), "{key}");
            assert_eq!(h.app().page.active_sql(), original_sql.as_deref(), "{key}");
            assert_eq!(h.app().page.tabs.active(), original_active, "{key}");
            assert_eq!(
                h.app()
                    .page
                    .run
                    .as_ref()
                    .map(|run| (run.owner, run.remaining_ticks)),
                original_run,
                "{key} does not replace the pending owner or advance its countdown"
            );
        }
        assert!(
            matches!(
                h.app().page.active_result_state(),
                Some(QueryResultState::Plan)
            ),
            "selecting Run preserves the active Plan fixture"
        );
        let _ = h.tick();
        assert_eq!(
            h.app().page.run.as_ref().map(|run| run.remaining_ticks),
            Some(3),
            "only a native Tick advances the countdown"
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn disabled_stale_new_action_does_not_consume_the_next_document_key() {
        let mut page = QueryWorkbenchPage::new();
        Page::set_option(&mut page, options::ALLOW_NEW_KEY, 0);
        let next_document_id = page.next_document_id;
        let document_count = page.documents.len();
        let tab_state = page.tabs.clone();

        assert!(page.apply_tabs_action(TabsAction::New).is_none());
        assert_eq!(page.next_document_id, next_document_id);
        assert_eq!(page.documents.len(), document_count);
        assert_eq!(page.tabs, tab_state);

        Page::set_option(&mut page, options::ALLOW_NEW_KEY, 1);
        assert!(page.apply_tabs_action(TabsAction::New).is_some());
        assert_eq!(page.next_document_id, next_document_id.saturating_add(1));
        assert_eq!(page.document_count(), document_count.saturating_add(1));
        assert_eq!(page.tabs.active(), Some(ItemKey::num(next_document_id)));
    }

    #[test]
    fn read_only_input_preserves_the_exact_host_sql_after_keyboard_and_paste() {
        let mut h = native_harness();
        open_options(&mut h);
        assert!(h.tab_to(OPTIONS_READ_ONLY_FIELD));
        let _ = h.key(KeyCode::Char(' '));
        let _ = h.key(KeyCode::Esc);

        assert!(h.tab_to(EDITOR_ID));
        let _ = h.type_str(" DROP");
        let _ = h.paste(" UPDATE orders SET status = 'bad'");
        assert_eq!(h.app().page.active_sql(), Some(FIRST_QUERY));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn direct_run_command_is_rejected_while_the_native_options_layer_is_open() {
        let mut h = native_harness();
        open_options(&mut h);
        assert_eq!(h.focus(), Some(OPTIONS_ALLOW_NEW_FIELD));
        let _ = h.key_mod(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(
            h.app().last_command,
            Some((RUN_QUERY, false)),
            "the real command reaches Page::command and is rejected by the open modal"
        );
        assert!(h.is_open(OPTIONS_DIALOG));
        assert!(!h.app().page.is_running());
        assert_eq!(
            h.app().page.active_result_state(),
            Some(&QueryResultState::Rows {
                table: "orders".to_owned(),
                count: 25,
            })
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    fn document_snapshot(
        page: &QueryWorkbenchPage,
    ) -> Vec<(ItemKey, String, String, ResultView, Option<SampleTable>)> {
        page.documents
            .iter()
            .map(|document| {
                (
                    document.key,
                    document.title.clone(),
                    document.editor.text().to_owned(),
                    document.result.clone(),
                    document.source_table,
                )
            })
            .collect()
    }

    fn document_snapshot_from(
        documents: &[Document],
    ) -> Vec<(ItemKey, String, String, ResultView, Option<SampleTable>)> {
        documents
            .iter()
            .map(|document| {
                (
                    document.key,
                    document.title.clone(),
                    document.editor.text().to_owned(),
                    document.result.clone(),
                    document.source_table,
                )
            })
            .collect()
    }

    #[test]
    fn explorer_activation_reuses_source_identity_and_ignores_duplicate_titles() {
        let mut page = QueryWorkbenchPage::new();
        let explorer = super::explorer::ExplorerState::new();
        let orders_key = ItemKey::text("query.workbench.orders");
        page.documents[0]
            .editor
            .set_text("SELECT edited FROM orders;");
        page.documents[0].result = ResultView::Plan;
        let next_document_id = page.next_document_id;

        assert!(page.apply_explorer_action(
            TreeAction::Activated(SampleTable::Orders.key()),
            &explorer,
        ));
        assert_eq!(page.tabs.active(), Some(orders_key));
        assert_eq!(
            page.documents[0].editor.text(),
            "SELECT edited FROM orders;"
        );
        assert_eq!(page.documents[0].result, ResultView::Plan);
        assert_eq!(page.next_document_id, next_document_id);
        assert_eq!(page.status, "Opened orders from Explorer.");

        assert!(page.clear_active());
        assert_eq!(page.documents[0].editor.text(), "");
        assert_eq!(page.documents[0].source_table, Some(SampleTable::Orders));
        assert!(page.apply_explorer_action(
            TreeAction::Activated(SampleTable::Orders.key()),
            &explorer,
        ));
        assert_eq!(page.tabs.active(), Some(orders_key));
        assert_eq!(page.documents[0].editor.text(), "");
        assert_eq!(page.documents[0].result, ResultView::Plan);
        assert_eq!(page.next_document_id, next_document_id);

        page.documents.push(Document::new(
            ItemKey::num(99),
            "products.sql".to_owned(),
            "SELECT edited FROM products;",
        ));
        let count_before_open = page.document_count();
        assert!(page.apply_explorer_action(
            TreeAction::Activated(SampleTable::Products.key()),
            &explorer,
        ));
        assert_eq!(page.document_count(), count_before_open + 1);
        assert_eq!(
            page.documents.last().map(|document| document.key),
            Some(ItemKey::num(2))
        );
        assert_eq!(
            page.documents
                .last()
                .map(|document| document.title.as_str()),
            Some("products.sql")
        );
        assert_eq!(
            page.documents.last().map(|document| document.editor.text()),
            Some("SELECT * FROM products LIMIT 100;\n")
        );
    }

    #[test]
    fn explorer_open_ignores_allow_new_and_preserves_pending_run_owner_and_status() {
        let mut page = QueryWorkbenchPage::new();
        Page::set_option(&mut page, options::ALLOW_NEW_KEY, 0);
        let orders_key = ItemKey::text("query.workbench.orders");
        page.documents[0]
            .editor
            .set_text("SELECT id FROM customers LIMIT 25;");
        page.run = Some(PendingRun {
            owner: orders_key,
            explain: false,
            remaining_ticks: 4,
        });
        page.status = "Running query · 4 native ticks remaining".to_owned();
        let run_status = page.status.clone();
        let explorer = super::explorer::ExplorerState::new();

        assert!(!page.new_document());
        assert!(page.apply_explorer_action(
            TreeAction::Activated(SampleTable::Products.key()),
            &explorer,
        ));
        assert_eq!(page.tabs.active(), Some(ItemKey::num(2)));
        assert_eq!(
            page.run
                .as_ref()
                .map(|run| (run.owner, run.remaining_ticks)),
            Some((orders_key, 4))
        );
        assert_eq!(
            page.status, run_status,
            "opening Explorer does not replace a running status"
        );
        assert_eq!(page.documents[0].source_table, Some(SampleTable::Orders));
        assert_eq!(page.documents[2].source_table, Some(SampleTable::Products));

        let mut harness = Harness::new(
            Host {
                page,
                area: Rect::new(0, 0, 160, 40),
                keymap: host_keymap(),
                last_command: None,
                open_options_before_update: false,
            },
            Theme::junie(),
            160,
            40,
        );
        harness.ticks(4);
        assert!(!harness.app().page.is_running());
        assert_eq!(
            harness.app().page.documents[0].result,
            ResultView::Rows {
                table: "customers".to_owned(),
                count: 25
            }
        );
        assert_eq!(harness.app().page.documents[2].result, ResultView::NotRun);
        assert_eq!(harness.app().page.tabs.active(), Some(ItemKey::num(2)));
    }

    #[test]
    fn explorer_activation_is_inert_for_unknown_rows_and_reports_allocator_exhaustion() {
        let mut page = QueryWorkbenchPage::new();
        let explorer = super::explorer::ExplorerState::new();
        let documents = page.documents.clone();
        let tabs = page.tabs.clone();
        assert!(!page.apply_explorer_action(
            TreeAction::Activated(ItemKey::text("query.workbench.unknown")),
            &explorer,
        ));
        for key in [
            ItemKey::text("query.workbench.explorer.schema.public"),
            ItemKey::text("query.workbench.explorer.schema.analytics"),
            ItemKey::text("query.workbench.explorer.schema.audit"),
            ItemKey::text("query.workbench.explorer.collection.views"),
            ItemKey::text("query.workbench.explorer.collection.functions"),
        ] {
            assert!(!page.apply_explorer_action(TreeAction::Activated(key), &explorer));
        }
        assert_eq!(document_snapshot(&page), document_snapshot_from(&documents));
        assert_eq!(page.tabs, tabs);

        page.next_document_id = u64::MAX;
        let documents = page.documents.clone();
        let tabs = page.tabs.clone();
        let active_tab = page.tabs.active();
        assert!(!page.apply_explorer_action(
            TreeAction::Activated(SampleTable::Products.key()),
            &explorer,
        ));
        assert_eq!(document_snapshot(&page), document_snapshot_from(&documents));
        assert_eq!(page.tabs, tabs);
        assert_eq!(page.tabs.active(), active_tab);
        assert_eq!(page.next_document_id, u64::MAX);
        assert_eq!(page.status, "Cannot open another query tab.");
    }

    #[test]
    fn direct_page_option_hide_commits_filter_and_drains_hidden_tree_focus() {
        let mut filter = native_harness();
        assert!(filter.tab_to(super::explorer::FILTER));
        let _ = filter.paste("  orders  ");
        assert!(filter.app().page.explorer.is_filter_editing());
        assert_eq!(filter.app().page.explorer.filter_value(), "");
        assert_eq!(filter.app().page.explorer.normalized_query(), "orders");

        Page::set_option(&mut filter.app_mut().page, "queryworkbench.explorer", 0);
        filter.draw();
        assert!(
            filter.diagnostics().is_empty(),
            "direct hidden Filter draw: {:?}",
            filter.diagnostics()
        );
        assert_ne!(
            filter.focus(),
            Some(super::explorer::FILTER),
            "native draw releases focused Filter after direct hide"
        );
        assert!(
            !filter.app().page.explorer.is_filter_editing(),
            "the native FocusOut commits before another input or tick"
        );
        assert_eq!(
            filter.app().page.explorer.filter_value(),
            "  orders  ",
            "direct hide draw commits the current filter draft"
        );
        let mut filter_focus_released = false;
        for tick in 0..3 {
            let _ = filter.tick();
            assert!(
                filter.diagnostics().is_empty(),
                "hidden Filter lifecycle tick {tick}: {:?}",
                filter.diagnostics()
            );
            if filter.focus() != Some(super::explorer::FILTER) {
                filter_focus_released = true;
                break;
            }
        }
        assert!(filter_focus_released, "native hide settles Filter FocusOut");
        assert!(
            !filter.app().page.explorer.is_filter_editing(),
            "the native FocusOut commits and ends the hidden filter edit"
        );
        assert_eq!(filter.app().page.explorer.filter_value(), "  orders  ");
        assert_eq!(filter.app().page.explorer.normalized_query(), "orders");
        assert!(filter.area_of(super::EXPLORER_PANEL).is_none());
        assert_ne!(filter.focus(), Some(super::explorer::FILTER));
        assert_ne!(filter.focus(), Some(super::explorer::TREE));
        let committed_filter = filter.app().page.explorer.filter_value().to_owned();
        let _ = filter.key(KeyCode::Char('x'));
        let _ = filter.paste("hidden input");
        let _ = filter.tick();
        assert_eq!(
            filter.app().page.explorer.filter_value(),
            committed_filter,
            "ordinary input does not reach a hidden Filter"
        );
        assert!(
            filter.diagnostics().is_empty(),
            "{:?}",
            filter.diagnostics()
        );

        let mut tree = native_harness();
        assert!(tree.tab_to(super::explorer::TREE));
        Page::set_option(&mut tree.app_mut().page, "queryworkbench.explorer", 0);
        tree.draw();
        assert!(
            tree.diagnostics().is_empty(),
            "direct hidden Tree draw: {:?}",
            tree.diagnostics()
        );
        assert_ne!(
            tree.focus(),
            Some(super::explorer::TREE),
            "native draw releases focused Tree after direct hide"
        );
        let mut tree_focus_released = false;
        for tick in 0..3 {
            let _ = tree.tick();
            assert!(
                tree.diagnostics().is_empty(),
                "hidden Tree lifecycle tick {tick}: {:?}",
                tree.diagnostics()
            );
            if tree.focus() != Some(super::explorer::TREE) {
                tree_focus_released = true;
                break;
            }
        }
        assert!(tree_focus_released, "native hide settles Tree FocusOut");
        assert_ne!(tree.focus(), Some(super::explorer::TREE));
        assert!(tree.area_of(super::EXPLORER_PANEL).is_none());
        assert!(tree.diagnostics().is_empty(), "{:?}", tree.diagnostics());
    }

    #[test]
    fn options_modal_drains_filter_and_tree_focus_lifecycle_without_editing() {
        let mut filter = native_harness();
        assert!(filter.tab_to(super::explorer::FILTER));
        let _ = filter.paste("orders");
        assert_eq!(filter.focus(), Some(super::explorer::FILTER));
        assert!(filter.app().page.explorer.is_filter_editing());
        filter.app_mut().open_options_before_update = true;
        let _ = filter.tick();
        assert!(filter.is_open(OPTIONS_DIALOG));
        assert_ne!(filter.focus(), Some(super::explorer::FILTER));
        assert!(
            !filter.app().page.explorer.is_filter_editing(),
            "modal FocusOut commits the native filter draft"
        );
        assert_eq!(filter.app().page.explorer.filter_value(), "orders");
        assert!(
            filter.diagnostics().is_empty(),
            "direct modal open from Filter focus: {:?}",
            filter.diagnostics()
        );

        let _ = filter.key(KeyCode::Esc);
        assert!(!filter.is_open(OPTIONS_DIALOG));
        let mut tree = native_harness();
        assert!(tree.tab_to(super::explorer::TREE));
        assert_eq!(tree.focus(), Some(super::explorer::TREE));
        tree.app_mut().open_options_before_update = true;
        let _ = tree.tick();
        assert!(tree.is_open(OPTIONS_DIALOG));
        assert_ne!(tree.focus(), Some(super::explorer::TREE));
        assert!(
            tree.diagnostics().is_empty(),
            "direct modal open from Tree focus: {:?}",
            tree.diagnostics()
        );
    }

    #[test]
    fn orders_result_adapter_restores_native_schema_and_historical_values() {
        let model_25 = QueryResultState::Rows {
            table: "orders".to_owned(),
            count: 25,
        }
        .model();
        let columns = model_25.schema.columns();

        assert_eq!(model_25.schema, super::ResultSchema::Orders);
        assert_eq!(
            columns
                .iter()
                .map(|column| column.column.title)
                .collect::<Vec<_>>(),
            ["id", "customer", "status", "total", "created_at"]
        );
        assert_eq!(
            columns
                .iter()
                .map(|column| column.column.key.raw())
                .collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            columns
                .iter()
                .map(|column| column.column.min_width)
                .collect::<Vec<_>>(),
            [7, 16, 10, 10, 17]
        );
        assert_eq!(
            columns
                .iter()
                .map(|column| column.column.max_width)
                .collect::<Vec<_>>(),
            [7, u16::MAX, 10, 10, 17]
        );
        assert_eq!(
            columns
                .iter()
                .map(|column| column.flexible)
                .collect::<Vec<_>>(),
            [false, true, false, false, false]
        );
        assert_eq!(
            columns
                .iter()
                .map(|column| column.column.align)
                .collect::<Vec<_>>(),
            [
                termrock::Align::Left,
                termrock::Align::Left,
                termrock::Align::Left,
                termrock::Align::Right,
                termrock::Align::Left,
            ]
        );
        assert!(!columns.iter().any(|column| column.column.sortable));
        assert_eq!(model_25.row_count(), 25);
        assert_eq!(
            model_25.rows[..3]
                .iter()
                .map(|row| row[..3].to_vec())
                .collect::<Vec<_>>(),
            vec![
                vec![
                    "10480".to_owned(),
                    "Northwind Example".to_owned(),
                    "pending".to_owned(),
                ],
                vec![
                    "10473".to_owned(),
                    "Globex Example".to_owned(),
                    "pending".to_owned(),
                ],
                vec![
                    "10466".to_owned(),
                    "Initech Example".to_owned(),
                    "paid".to_owned(),
                ],
            ]
        );
        assert_eq!(
            model_25.rows[0],
            vec![
                "10480".to_owned(),
                "Northwind Example".to_owned(),
                "pending".to_owned(),
                "40.00".to_owned(),
                "2026-10-17 23:00".to_owned(),
            ]
        );
        assert_eq!(
            model_25.rows[24],
            vec![
                "10312".to_owned(),
                "Northwind Example".to_owned(),
                "pending".to_owned(),
                "928.12".to_owned(),
                "2026-10-13 23:48".to_owned(),
            ]
        );

        let model_40 = QueryResultState::Rows {
            table: "ORDERS".to_owned(),
            count: 40,
        }
        .model();
        assert_eq!(model_40.row_count(), 40);
        assert_eq!(
            model_40.rows[39],
            vec![
                "10207".to_owned(),
                "Northwind Example".to_owned(),
                "shipped".to_owned(),
                "583.07".to_owned(),
                "2026-10-11 08:33".to_owned(),
            ]
        );
        assert_eq!(model_40.row_key(0), ItemKey::num(1));
        assert_eq!(model_40.row_key(39), ItemKey::num(40));

        let extreme = super::order_result_row(usize::MAX);
        assert_eq!(extreme.len(), 5);
        assert!(
            extreme[3]
                .split_once('.')
                .is_some_and(|(_, cents)| cents.len() == 2)
        );
        assert!(extreme[4].starts_with("2026-10-"));
    }

    #[test]
    fn non_orders_plans_and_errors_keep_the_generic_three_cell_schema() {
        let customers = QueryResultState::Rows {
            table: "customers".to_owned(),
            count: 2,
        }
        .model();
        assert_eq!(customers.schema, super::ResultSchema::Generic);
        assert_eq!(
            customers
                .schema
                .columns()
                .iter()
                .map(|column| column.column.title)
                .collect::<Vec<_>>(),
            ["Field", "Value", "Details"]
        );
        assert_eq!(
            customers.rows,
            [
                vec![
                    "1".to_owned(),
                    "Northwind Account".to_owned(),
                    "2026-10-27".to_owned(),
                ],
                vec![
                    "2".to_owned(),
                    "Globex Account".to_owned(),
                    "2026-10-26".to_owned(),
                ],
            ]
        );

        let plan = QueryResultState::Plan.model();
        assert_eq!(plan.schema, super::ResultSchema::Generic);
        assert_eq!(plan.rows.len(), 4);
        assert!(plan.rows.iter().all(|row| row.len() == 3));
        assert_eq!(
            plan.rows[0],
            vec!["Limit".to_owned(), "0.42..2.10".to_owned(), "25".to_owned()]
        );

        let error = QueryResultState::Error {
            message: "missing relation".to_owned(),
            detail: "check the table name".to_owned(),
        }
        .model();
        assert_eq!(error.schema, super::ResultSchema::Generic);
        assert_eq!(
            error.rows,
            [vec![
                "ERROR".to_owned(),
                "missing relation".to_owned(),
                "check the table name".to_owned(),
            ]]
        );
        assert_eq!(error.rows[0].len(), 3);

        for result in [QueryResultState::NotRun, QueryResultState::Empty] {
            let model = result.model();
            assert_eq!(model.schema, super::ResultSchema::Generic);
            assert!(model.rows.is_empty());
        }
    }

    #[test]
    fn native_wide_orders_table_renders_all_columns_and_sample_cells() {
        let h = native_harness();
        let area = h.area_of(RESULTS_ID).unwrap_or(Rect::ZERO);
        assert!(!area.is_empty(), "the native Results table is drawn");
        let header = h.row(area.y);
        for name in ["id", "customer", "status", "total", "created_at"] {
            assert!(header.contains(name), "missing {name:?} from {header:?}");
        }

        let screen = h.text();
        for value in [
            "10480",
            "Northwind Example",
            "pending",
            "40.00",
            "2026-10-17 23:00",
        ] {
            assert!(screen.contains(value), "missing {value:?}\\n{screen}");
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    #[test]
    fn compact_orders_table_scrolls_horizontally_and_reconciles_generic_results() {
        let mut h = Harness::new(
            Host {
                page: QueryWorkbenchPage::new(),
                area: Rect::new(0, 0, 80, 24),
                keymap: host_keymap(),
                last_command: None,
                open_options_before_update: false,
            },
            Theme::junie(),
            80,
            24,
        );
        assert!(h.area_of(RESULTS_ID).is_some());
        let original_sql = h.app().page.active_sql().map(str::to_owned);
        assert!(h.tab_to(EDITOR_ID));
        let _ = h.key(KeyCode::F(6));
        assert_eq!(h.focus(), Some(RESULTS_ID));
        assert_eq!(h.app().page.active_sql(), original_sql.as_deref());
        let original_state = h.app().page.results_state() as *const _;
        let _ = h.key(KeyCode::Down);
        let selected_row = h.app().page.results_state().cursor().map(|(row, _)| row);
        assert!(
            selected_row.is_some(),
            "native row navigation selects an Orders result row"
        );

        let mut reached_trailing_columns = false;
        for _ in 0..super::ORDERS_RESULT_COLUMNS.len() {
            let area = h.area_of(RESULTS_ID).unwrap_or(Rect::ZERO);
            let header = h.row(area.y);
            let screen = h.text();
            if header.contains("total")
                && header.contains("created_at")
                && screen.contains("40.00")
                && screen.contains("2026-10-17 23:00")
            {
                reached_trailing_columns = true;
                break;
            }
            let _ = h.key(KeyCode::Right);
        }
        assert!(
            reached_trailing_columns,
            "native horizontal navigation must expose total and the full timestamp\n{}",
            h.text()
        );
        assert!(
            h.app().page.results_state().grid_state().col_offset() > 0,
            "the native table owns horizontal column movement"
        );
        assert_eq!(h.focus(), Some(RESULTS_ID));
        assert_eq!(
            h.app().page.results_state().cursor().map(|(row, _)| row),
            selected_row,
            "horizontal navigation retains the selected result row"
        );
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

        for _ in 0..super::ORDERS_RESULT_COLUMNS.len() {
            if h.app().page.results_state().grid_state().col_offset() == 0 {
                break;
            }
            let _ = h.key(KeyCode::Left);
        }
        assert_eq!(h.app().page.results_state().grid_state().col_offset(), 0);
        assert_eq!(
            h.app().page.results_state().cursor().map(|(row, _)| row),
            selected_row
        );
        let area = h.area_of(RESULTS_ID).unwrap_or(Rect::ZERO);
        let header = h.row(area.y);
        assert!(header.contains("id"), "{header:?}");
        assert!(header.contains("customer"), "{header:?}");

        let customers_key = h.app().page.documents[1].key;
        h.app_mut().page.documents[1].result = QueryResultState::Rows {
            table: "customers".to_owned(),
            count: 25,
        };
        h.app_mut().page.tabs.set_active(1, customers_key);
        let _ = h.tick();

        assert_eq!(h.app().page.tabs.active(), Some(customers_key));
        assert_eq!(
            h.app().page.results_state() as *const _,
            original_state,
            "schema changes retain the same native TableState"
        );
        assert_eq!(
            h.app().page.active_result_model().schema,
            super::ResultSchema::Generic
        );
        assert_eq!(h.app().page.active_result_model().row_count(), 25);
        assert_eq!(
            h.app().page.results_state().cursor().map(|(row, _)| row),
            selected_row,
            "native reconciliation retains the selected row across schemas"
        );
        assert!(h.tab_to(RESULTS_ID));
        let _ = h.key(KeyCode::Right);
        assert_eq!(
            h.app().page.results_state().cursor().map(|(row, _)| row),
            selected_row,
            "native navigation remains usable after the schema change"
        );
        assert_eq!(h.focus(), Some(RESULTS_ID));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
