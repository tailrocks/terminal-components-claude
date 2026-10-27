use std::time::Duration;

use termrock::grid::{Column, ColumnKey, Table, TableColumn, TableState};
use termrock::{
    ActionKey, CodeEditor, CodeEditorState, Cx, FrameRead, GridModel, Id, ItemKey, Panel,
    PanelKind, Rect, Response, Role, StateFlags, Tabs, TabsAction, TabsState, ToolItem, Toolbar,
    ToolbarAction, ToolbarState, Ui, UpdateCause, layout,
};

use super::{Page, PageStatus, PageUpdate, frame};

const TABS: Id = Id::root("showcase_ui::pages::query_workbench.tabs");
const TOOLBAR: Id = Id::root("showcase_ui::pages::query_workbench.toolbar");
const EDITOR_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.editor_panel");
const RESULTS_PANEL: Id = Id::root("showcase_ui::pages::query_workbench.results_panel");
const EDITOR: Id = Id::root("showcase_ui::pages::query_workbench.editor");
const RESULTS: Id = Id::root("showcase_ui::pages::query_workbench.results");

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

const RUN_CAPTURE_OWNERS: [Id; 4] = [TABS, TOOLBAR, EDITOR, RESULTS];
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
}

impl Document {
    fn new(key: ItemKey, title: String, sql: &str) -> Self {
        Self {
            key,
            title,
            editor: CodeEditorState::new(sql),
            result: ResultView::NotRun,
        }
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
}

impl QueryWorkbenchPage {
    /// Creates a workbench with the sample orders and customers documents.
    pub fn new() -> Self {
        let orders_key = ItemKey::text("query.workbench.orders");
        let customers_key = ItemKey::text("query.workbench.customers");
        let mut orders = Document::new(orders_key, "orders.sql".to_owned(), FIRST_QUERY);
        orders.result = ResultView::Rows {
            table: "orders".to_owned(),
            count: 25,
        };
        let mut tabs = TabsState::default();
        tabs.set_active(0, orders_key);

        Self {
            documents: vec![
                orders,
                Document::new(customers_key, "customers.sql".to_owned(), SECOND_QUERY),
            ],
            next_document_id: 2,
            tabs,
            toolbar: ToolbarState::default(),
            results: TableState::default(),
            run: None,
            status: "Ready · sample data only".to_owned(),
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
        let document_id = self.next_document_id;
        let Some(next_document_id) = document_id.checked_add(1) else {
            self.status = "Cannot open another query tab.".to_owned();
            return false;
        };
        self.next_document_id = next_document_id;

        let key = ItemKey::num(document_id);
        let title = format!("Query {document_id}");
        self.documents.push(Document::new(key, title, ""));
        self.tabs.set_active(self.documents.len() - 1, key);
        if self.run.is_none() {
            self.status = "Ready · sample data only".to_owned();
        }
        true
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
            ],
        )
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

        let tabs = Tabs::new(TABS)
            .key(|document: &Document| document.key)
            .allow_new(true)
            .closable(true);
        let tab_update = tabs.update(cx, &mut self.tabs, &self.documents);
        let tab_action = tab_update.action_ref().copied();
        response |= tab_update.erase();
        match tab_action {
            Some(TabsAction::Close(key)) if self.close_document(key) => {
                response |= Response::changed();
                status = Some(PageStatus(self.status.clone()));
            }
            Some(TabsAction::New) => {
                let _ = self.new_document();
                response |= Response::changed();
                status = Some(PageStatus(self.status.clone()));
            }
            Some(TabsAction::Activated(_)) | Some(TabsAction::Close(_)) | None => {}
        }

        if let Some(index) = self.active_index()
            && let Some(document) = self.documents.get_mut(index)
        {
            let update = CodeEditor::new(EDITOR, 12)
                .fill_height(true)
                .placeholder("Write a query against the sample data")
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

        if let Some(next_status) = self.advance_run(cx) {
            status = Some(next_status);
            response |= Response::changed();
        }

        PageUpdate { response, status }
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action != RUN_QUERY || self.run.is_some() {
            return Response::ignored();
        }
        let _ = self.start_run(cx, false);
        Response::changed()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "sample data · no database connection",
            |ui, body| {
                let rows = layout::rows(
                    body,
                    &[
                        layout::Track::Fixed(1),
                        layout::Track::Fixed(1),
                        layout::Track::Flex(1),
                        layout::Track::Fixed(1),
                    ],
                );
                let tabs_area = rows.first().copied().unwrap_or(Rect::new(0, 0, 0, 0));
                let toolbar_area = rows.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
                let work_area = rows.get(2).copied().unwrap_or(Rect::new(0, 0, 0, 0));
                let status_area = rows.get(3).copied().unwrap_or(Rect::new(0, 0, 0, 0));

                Tabs::new(TABS)
                    .key(|document: &Document| document.key)
                    .allow_new(true)
                    .closable(true)
                    .draw(ui, tabs_area, &self.tabs, &self.documents);

                let _ = self.toolbar().draw(ui, toolbar_area, &self.toolbar);

                let columns = layout::columns(
                    work_area,
                    &[layout::Track::Flex(3), layout::Track::Flex(2)],
                    1,
                );
                let editor_area = columns.first().copied().unwrap_or(Rect::new(0, 0, 0, 0));
                let results_area = columns.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
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
                                .draw(ui, inner, &document.editor);
                        });
                }

                let model = self.active_result_model();
                let summary = self
                    .active_index()
                    .and_then(|index| self.documents.get(index))
                    .map_or_else(
                        || "No query is open.".to_owned(),
                        |document| document.result.summary(),
                    );
                Panel::new(RESULTS_PANEL)
                    .kind(PanelKind::Framed)
                    .title("Results")
                    .draw(ui, results_area, |ui, inner| {
                        let result_rows =
                            layout::rows(inner, &[layout::Track::Fixed(1), layout::Track::Flex(1)]);
                        if let Some(summary_area) = result_rows.first()
                            && !summary_area.is_empty()
                        {
                            let _ = ui.paint_str(*summary_area, &summary, ui.surface_style());
                        }
                        let table_area =
                            result_rows.get(1).copied().unwrap_or(Rect::new(0, 0, 0, 0));
                        let _ = Table::new(RESULTS, &RESULT_COLUMNS).draw(
                            ui,
                            table_area,
                            &self.results,
                            &model,
                        );
                    });

                if !status_area.is_empty() {
                    let _ = ui.paint_str(status_area, &self.status, ui.surface_style());
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(TABS).contains(StateFlags::FOCUSED) {
            return TABS_HINTS;
        }
        if ui.state(TOOLBAR).contains(StateFlags::FOCUSED) {
            return TOOLBAR_HINTS;
        }
        if ui.state(EDITOR).contains(StateFlags::FOCUSED) {
            return EDITOR_HINTS;
        }
        if ui.state(RESULTS).contains(StateFlags::FOCUSED) {
            return RESULTS_HINTS;
        }
        &[]
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.active_index()
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

#[cfg(test)]
mod tests {
    use super::{
        EDITOR_ID, QueryResultState, QueryWorkbenchPage, RESULTS_ID, ResultView, TABS_ID, TOOLBAR,
        TOOLBAR_ID, evaluate_query,
    };
    use crate::pages::Page;
    use ratatui::layout::Rect;
    use termrock::{
        App as TermrockApp, Cx, GridModel, Id, ItemKey, Part, PartRef, Response, Theme, Ui,
    };
    use termrock_test_support::Harness;

    #[derive(Debug)]
    struct Host {
        page: QueryWorkbenchPage,
        area: Rect,
    }

    impl TermrockApp for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, self.area);
        }
    }

    fn overflow_harness() -> Harness<Host> {
        Harness::new(
            Host {
                page: QueryWorkbenchPage::new(),
                area: Rect::new(0, 0, 14, 18),
            },
            Theme::junie(),
            14,
            18,
        )
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
        assert_eq!(evaluate_query("  \n", false), ResultView::Empty);
        assert_eq!(
            evaluate_query("SELECT id FROM orders LIMIT 25", false),
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 25,
            }
        );
        assert_eq!(
            evaluate_query("SELECT id FROM orders", true),
            ResultView::Plan
        );
        let default_rows = evaluate_query("SELECT id FROM orders", false);
        assert_eq!(
            default_rows,
            ResultView::Rows {
                table: "orders".to_owned(),
                count: 40,
            }
        );
        assert_eq!(default_rows.model().row_count(), 40);
        let limited_rows = evaluate_query("SELECT id FROM orders LIMIT 25", false);
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
            evaluate_query("SELECT * FROM missing_table", false),
            ResultView::Error { .. }
        ));
        assert!(matches!(
            evaluate_query("SELECT 1", false),
            ResultView::Error { .. }
        ));
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
}
