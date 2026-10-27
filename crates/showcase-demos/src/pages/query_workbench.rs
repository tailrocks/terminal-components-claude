use std::time::Duration;

use termrock::grid::{Column, ColumnKey, Table, TableColumn, TableState};
use termrock::{
    ActionKey, CodeEditor, CodeEditorState, Cx, FrameRead, GridModel, Id, ItemKey, Maximized,
    Panel, PanelKind, Rect, ResetPolicy, Response, Role, SplitAxis, SplitFallback, SplitPane,
    SplitPaneState, StateFlags, StatusBar, StatusItem, Tabs, TabsAction, TabsState, ToolItem,
    Toolbar, ToolbarAction, ToolbarState, TreeAction, Ui, UpdateCause, layout,
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

pub const TABS_ID: Id = TABS;

pub const TOOLBAR_ID: Id = TOOLBAR;

pub const EDITOR_ID: Id = EDITOR;

pub const RESULTS_ID: Id = RESULTS;

pub const EDITOR_PANEL_ID: Id = EDITOR_PANEL;

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
const SAMPLE_CUSTOMERS: [&str; 3] = ["Northwind", "Globex", "Initech"];
const ORDER_STATUSES: [&str; 4] = ["pending", "pending", "paid", "shipped"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryResultState {
    NotRun,

    Empty,

    Rows { table: String, count: usize },

    Plan,

    Error { message: String, detail: String },
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
        let rows = match self {
            Self::NotRun | Self::Empty => Vec::new(),
            Self::Rows { table, count } => (0..*count)
                .map(|index| {
                    if table.eq_ignore_ascii_case("orders") {
                        let customer = SAMPLE_CUSTOMERS
                            .get(index % SAMPLE_CUSTOMERS.len())
                            .copied()
                            .unwrap_or("Sample");
                        let status = ORDER_STATUSES
                            .get(index % ORDER_STATUSES.len())
                            .copied()
                            .unwrap_or("pending");
                        [
                            (10_480_usize.saturating_sub(index.saturating_mul(7))).to_string(),
                            format!("{customer} Example"),
                            status.to_owned(),
                        ]
                    } else {
                        let customer = SAMPLE_CUSTOMERS
                            .get(index % SAMPLE_CUSTOMERS.len())
                            .copied()
                            .unwrap_or("Sample");
                        [
                            index.saturating_add(1).to_string(),
                            format!("{customer} Account"),
                            format!("2026-10-{:02}", 27_usize.saturating_sub(index % 26)),
                        ]
                    }
                })
                .collect(),
            Self::Plan => [
                ["Limit".to_owned(), "0.42..2.10".to_owned(), "25".to_owned()],
                [
                    "Index Scan Backward on orders".to_owned(),
                    "0.42..8 410".to_owned(),
                    "1 204".to_owned(),
                ],
                [
                    "Index condition".to_owned(),
                    "status = 'pending'".to_owned(),
                    String::new(),
                ],
                [
                    "Using index".to_owned(),
                    "orders_created_at_idx".to_owned(),
                    String::new(),
                ],
            ]
            .into_iter()
            .collect(),
            Self::Error { message, detail } => {
                vec![["ERROR".to_owned(), message.clone(), detail.clone()]]
            }
        };
        ResultModel { rows }
    }
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
    rows: Vec<[String; 3]>,
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

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn document_keys(&self) -> impl Iterator<Item = ItemKey> + '_ {
        self.documents.iter().map(|document| document.key)
    }

    pub fn active_editor(&self) -> Option<&CodeEditorState> {
        self.active_index()
            .and_then(|index| self.documents.get(index))
            .map(|document| &document.editor)
    }

    pub fn active_sql(&self) -> Option<&str> {
        self.active_editor().map(|editor| editor.text())
    }

    pub fn active_result_state(&self) -> Option<&QueryResultState> {
        self.active_index()
            .and_then(|index| self.documents.get(index))
            .map(|document| &document.result)
    }

    pub fn tabs_state(&self) -> &TabsState {
        &self.tabs
    }

    pub fn toolbar_state(&self) -> &ToolbarState {
        &self.toolbar
    }

    pub fn results_state(&self) -> &TableState {
        &self.results
    }

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
            ResultFixture::Empty => document.result = ResultView::Empty,
            ResultFixture::Error => {
                document.result = ResultView::Error {
                    message: "relation \"ordrs\" does not exist".to_owned(),
                    detail: "SQLSTATE 42P01 · line 3 · Did you mean \"orders\"?".to_owned(),
                };
            }
            ResultFixture::Plan => document.result = ResultView::Plan,
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
            let result = evaluate_query(document.editor.text(), run.explain);
            let status = result.summary();
            document.result = result;
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

pub fn page() -> Box<dyn Page> {
    Box::new(QueryWorkbenchPage::new())
}

impl Page for QueryWorkbenchPage {
    fn title(&self) -> &'static str {
        "Query workbench"
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
                response |= update.erase();
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
            response |= Table::new(RESULTS, &RESULT_COLUMNS)
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
                                    let _ = Table::new(RESULTS, &RESULT_COLUMNS).draw(
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

fn evaluate_query(sql: &str, explain: bool) -> ResultView {
    if sql.trim().is_empty() {
        return ResultView::Empty;
    }
    let Some(table) = from_table(sql) else {
        return ResultView::Error {
            message: "Syntax error".to_owned(),
            detail: "Expected a FROM clause.".to_owned(),
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
        return ResultView::Error {
            message: format!("relation \"{table}\" does not exist"),
            detail: "Check the table name in the current sample schema.".to_owned(),
        };
    }
    if explain {
        return ResultView::Plan;
    }
    let count = if sql.to_ascii_lowercase().contains("limit 25") {
        25
    } else {
        40
    };
    ResultView::Rows {
        table: table.to_owned(),
        count,
    }
}

fn from_table(sql: &str) -> Option<&str> {
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
    (end > start).then(|| sql.get(start..end)).flatten()
}
