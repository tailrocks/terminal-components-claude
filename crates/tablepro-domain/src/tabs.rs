#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]

use crate::domain::ResultGrid;
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry};
use std::ops::Range;
use std::time::{Duration, Instant};
use tablepro_demo::db::{Catalog, ColType, Table, Value};
use tablepro_sql::sql::{self, PlanNode};
use termrock::{CompletingCodeEditorState, GridModel, GridState, Id, ItemKey, SyntaxRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabKey(u64);

impl TabKey {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn control(self, name: &'static str) -> Id {
        Id::root("tablepro.tab")
            .item(ItemKey::num(self.0))
            .sub(name)
    }
}

#[derive(Clone)]
pub struct GridView {
    pub columns: Vec<(String, ColType)>,
    pub model: ResultGrid,
    pub state: GridState,
}

impl GridView {
    pub fn from_result(result: &sql::ResultSet) -> Self {
        Self {
            columns: result.columns.clone(),
            model: ResultGrid::from_result(result),
            state: GridState::default(),
        }
    }

    pub fn pending_total(&self) -> usize {
        let pending = self.model.pending_total();
        let (Some((key, column)), Some(draft)) = (self.state.edit_cell(), self.state.edit_draft())
        else {
            return pending;
        };
        let Some(row) = (0..self.model.row_count()).find(|row| self.model.row_key(*row) == key)
        else {
            return pending;
        };
        let Some(column) = column.raw().checked_sub(1).map(usize::from) else {
            return pending;
        };
        let changed = self
            .model
            .cell(row, column)
            .is_some_and(|cell| cell.text != draft);
        let already_counted = self.model.pending().is_inserted(row)
            || (0..self.columns.len()).any(|column| self.model.pending().is_dirty(row, column));
        pending.saturating_add(usize::from(changed && !already_counted))
    }

    fn empty() -> Self {
        Self {
            columns: Vec::new(),
            model: ResultGrid::empty(),
            state: GridState::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QueryOutputId(u64);

impl QueryOutputId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryOutputKind {
    Rows,
    Error,
    Plan,
    Cancelled,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryOutputState {
    selected: Option<QueryOutputId>,
    revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationMutation {
    Unchanged,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRunFinish {
    Finished,
    StoppedWithoutCurrency,
    AlreadyStopped,
    NotOwned,
}

impl QueryOutputState {
    #[must_use]
    pub const fn selected(self) -> Option<QueryOutputId> {
        self.selected
    }

    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }
}

#[derive(Clone)]
pub struct QueryOutputInfo {
    id: QueryOutputId,
    label: String,
    source_range: Option<Range<usize>>,
    duration_ms: u32,
    kind: QueryOutputKind,
    pinned: bool,
    closable: bool,
}

impl QueryOutputInfo {
    #[must_use]
    pub const fn id(&self) -> QueryOutputId {
        self.id
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn source_range(&self) -> Option<Range<usize>> {
        self.source_range.clone()
    }

    #[must_use]
    pub const fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    #[must_use]
    pub const fn kind(&self) -> QueryOutputKind {
        self.kind
    }

    #[must_use]
    pub const fn is_pinned(&self) -> bool {
        self.pinned
    }

    #[must_use]
    pub const fn can_close(&self) -> bool {
        self.closable
    }
}

pub struct QueryOutputBody {
    result: Option<GridView>,
    error: Option<String>,
    plan: Option<PlanNode>,
    cancelled: bool,
}

impl QueryOutputBody {
    #[must_use]
    pub fn rows(result: GridView) -> Self {
        Self {
            result: Some(result),
            error: None,
            plan: None,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            result: None,
            error: Some(message.into()),
            plan: None,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn plan(plan: PlanNode) -> Self {
        Self {
            result: None,
            error: None,
            plan: Some(plan),
            cancelled: false,
        }
    }

    #[must_use]
    pub const fn cancelled() -> Self {
        Self {
            result: None,
            error: None,
            plan: None,
            cancelled: true,
        }
    }

    fn from_query_tab(tab: &mut QueryTab, kind: QueryOutputKind) -> Self {
        let body = Self {
            result: tab.result.take(),
            error: tab.error.take(),
            plan: tab.plan.take(),
            cancelled: false,
        };
        if body.is_empty() && kind == QueryOutputKind::Cancelled {
            Self::cancelled()
        } else {
            body
        }
    }

    fn install(self, tab: &mut QueryTab) {
        tab.result = self.result;
        tab.error = self.error;
        tab.plan = self.plan;
    }

    #[must_use]
    pub fn kind(&self) -> QueryOutputKind {
        if self.plan.is_some() {
            QueryOutputKind::Plan
        } else if self.error.is_some() {
            QueryOutputKind::Error
        } else if self.result.is_some() {
            QueryOutputKind::Rows
        } else if self.cancelled {
            QueryOutputKind::Cancelled
        } else {
            QueryOutputKind::Empty
        }
    }

    fn pending_total(&self) -> usize {
        self.result.as_ref().map_or(0, GridView::pending_total)
    }

    fn is_empty(&self) -> bool {
        self.result.is_none() && self.error.is_none() && self.plan.is_none() && !self.cancelled
    }
}

pub struct QueryOutputInput {
    label: String,
    source_range: Option<Range<usize>>,
    duration_ms: u32,
    body: QueryOutputBody,
}

impl QueryOutputInput {
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        source_range: Option<Range<usize>>,
        duration_ms: u32,
        body: QueryOutputBody,
    ) -> Self {
        Self {
            label: label.into(),
            source_range,
            duration_ms,
            body,
        }
    }
}

struct RetainedQueryOutput {
    info: QueryOutputInfo,
    body: Option<QueryOutputBody>,
}

impl core::fmt::Debug for GridView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GridView")
            .field("columns", &self.columns.len())
            .field("rows", &self.model.row_count())
            .field("pending", &self.pending_total())
            .field("state", &self.state)
            .finish()
    }
}

impl core::ops::Deref for GridView {
    type Target = ResultGrid;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Data,
    Structure,
}

#[derive(Debug, Clone)]
pub struct TableTab {
    pub table: Table,
    pub mode: TableMode,
    pub result: GridView,
    pub structure: Box<GridView>,
    pub filters: Vec<Filter>,
}

impl TableTab {
    pub fn new(table: Table, catalog: &Catalog) -> Self {
        let query = format!("SELECT * FROM {}.{}", table.schema, table.name);
        let result = sql::parse(&query)
            .ok()
            .and_then(|statement| match statement {
                sql::Statement::Select(select) => sql::run_select(catalog, &select).ok(),
                _ => None,
            })
            .map_or_else(GridView::empty, |result| GridView::from_result(&result));
        let mut tab = Self {
            table,
            mode: TableMode::Data,
            result,
            structure: Box::new(GridView::empty()),
            filters: Vec::new(),
        };
        tab.structure = Box::new(GridView::from_result(&sql::ResultSet {
            columns: tab.structure_columns(),
            rows: tab.structure(),
            total: tab.table.columns.len(),
            source: None,
            duration_ms: 0,
            editable: false,
        }));
        tab
    }
    pub const fn toggle_structure(&mut self) {
        self.mode = match self.mode {
            TableMode::Data => TableMode::Structure,
            TableMode::Structure => TableMode::Data,
        };
    }
    pub const fn is_structure(&self) -> bool {
        matches!(self.mode, TableMode::Structure)
    }
    pub fn set_filter(&mut self, filter: Filter) {
        if let Some(existing) = self
            .filters
            .iter_mut()
            .find(|old| old.column.eq_ignore_ascii_case(&filter.column))
        {
            *existing = filter;
        } else {
            self.filters.push(filter);
        }
    }
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }
    pub fn sort(&mut self, column: usize, direction: termrock::SortDir) {
        self.result.model.sort(
            termrock::ColumnKey::num((column as u16).saturating_add(1)),
            direction,
        );
    }
    pub fn structure(&self) -> Vec<Vec<Value>> {
        self.table
            .columns
            .iter()
            .map(|column| {
                vec![
                    Value::Text(column.name.clone()),
                    Value::Text(column.ty.sql().to_owned()),
                    Value::Bool(column.nullable),
                    Value::Bool(column.primary),
                    Value::Text(column.default.clone().unwrap_or_default()),
                    Value::Text(
                        column
                            .references
                            .as_ref()
                            .map(|(table, col)| format!("{table}.{col}"))
                            .unwrap_or_default(),
                    ),
                ]
            })
            .collect()
    }
    pub fn structure_columns(&self) -> Vec<(String, ColType)> {
        vec![
            ("name".to_owned(), ColType::Text),
            ("type".to_owned(), ColType::Text),
            ("nullable".to_owned(), ColType::Bool),
            ("primary".to_owned(), ColType::Bool),
            ("default".to_owned(), ColType::Text),
            ("references".to_owned(), ColType::Text),
        ]
    }
    pub fn preview(&self) -> Vec<String> {
        crate::grid_model::preview_for(&self.table, &self.result)
    }
}

#[derive(Debug, Clone)]
pub struct QueryStatus {
    pub summary: String,
    pub is_error: bool,
    pub duration_ms: u32,
}

#[derive(Clone)]
pub struct QueryTab {
    pub id: usize,
    pub name: String,
    pub query: String,

    pub editor_state: Box<CompletingCodeEditorState>,
    pub saved_text: String,
    pub result: Option<GridView>,
    pub error: Option<String>,
    pub plan: Option<PlanNode>,
    pub running: bool,

    pub last_status: Option<Box<QueryStatus>>,
}

impl core::fmt::Debug for QueryTab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueryTab")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("query", &"[redacted]")
            .field("editor_state", &"<input state>")
            .field("saved_text", &"[redacted]")
            .field("has_result", &self.result.is_some())
            .field("has_error", &self.error.is_some())
            .field("has_plan", &self.plan.is_some())
            .field("running", &self.running)
            .field("last_status", &self.last_status)
            .finish()
    }
}

impl QueryTab {
    pub fn new(id: usize, query: impl Into<String>) -> Self {
        let query = query.into();
        let editor_state = CompletingCodeEditorState::new(&query);
        Self {
            id,
            name: format!("Query {id}"),
            saved_text: query.clone(),
            editor_state: Box::new(editor_state),
            query,
            result: None,
            error: None,
            plan: None,
            running: false,
            last_status: None,
        }
    }

    pub fn record_status(&mut self, summary: impl Into<String>, is_error: bool, duration_ms: u32) {
        self.last_status = Some(Box::new(QueryStatus {
            summary: summary.into(),
            is_error,
            duration_ms,
        }));
    }

    pub fn record_execution_status(&mut self, succeeded: bool, elapsed: Duration) {
        let duration_ms = u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX);
        let summary = if succeeded {
            format!("ok · {}", compact_duration(duration_ms))
        } else {
            "error".to_owned()
        };
        self.record_status(summary, !succeeded, duration_ms);
    }

    pub fn execute(&mut self, catalog: &Catalog) -> Result<usize, String> {
        self.prepare_execution()?;
        let started = Instant::now();
        let outcome = self.execute_untimed(catalog);
        self.record_execution_status(outcome.is_ok(), started.elapsed());
        outcome
    }

    pub fn execute_with_duration(
        &mut self,
        catalog: &Catalog,
        elapsed: Duration,
    ) -> Result<usize, String> {
        self.prepare_execution()?;
        let outcome = self.execute_untimed(catalog);
        self.record_execution_status(outcome.is_ok(), elapsed);
        outcome
    }

    fn prepare_execution(&mut self) -> Result<(), String> {
        self.sync_query_text();
        if self.has_pending_result() {
            return Err("Pending result edits require confirmation".to_owned());
        }
        Ok(())
    }

    fn execute_untimed(&mut self, catalog: &Catalog) -> Result<usize, String> {
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("The demo executor only runs SELECT statements".to_owned());
        };
        let result = sql::run_select(catalog, &select).map_err(|error| error.message)?;
        let rows = result.rows.len();
        self.result = Some(GridView::from_result(&result));
        self.error = None;
        Ok(rows)
    }
    pub fn explain(&mut self, catalog: &Catalog) -> Result<(), String> {
        self.explain_statement(catalog, false).map(|_| ())
    }

    pub fn explain_statement(
        &mut self,
        catalog: &Catalog,
        analyze: bool,
    ) -> Result<String, String> {
        self.sync_query_text();
        let statement_text = {
            let editor = self.editor_state.editor();
            if let Some(selected) = editor.selected_text() {
                Self::explain_selection_statement(selected)?
            } else if let Some((start, end)) =
                sql::statement_at(editor.text(), editor.cursor_offset())
            {
                editor
                    .text()
                    .get(start..end)
                    .ok_or_else(|| "Explain statement is not a valid range".to_owned())?
            } else {
                return Err("No SQL statement at cursor".to_owned());
            }
        };
        let statement = sql::parse(statement_text).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("Explain accepts SELECT statements".to_owned());
        };
        let plan = sql::explain(catalog, &select, analyze).map_err(|error| error.message)?;
        self.plan = Some(plan);
        Ok(statement_text.to_owned())
    }

    fn explain_selection_statement(selected: &str) -> Result<&str, String> {
        let mut statement = None;
        for (start, end) in sql::split_statements(selected) {
            let candidate = selected
                .get(start..end)
                .ok_or_else(|| "Explain selection is not a valid statement".to_owned())?;
            let has_sql_tokens = sql::tokenize(candidate).into_iter().any(|token| {
                !matches!(token.kind, sql::TokKind::Whitespace | sql::TokKind::Comment)
            });
            if !has_sql_tokens {
                continue;
            }
            if statement.is_some() {
                return Err("Explain selection must contain exactly one statement".to_owned());
            }
            statement = Some(candidate);
        }
        statement.ok_or_else(|| "Explain requires a non-empty selection".to_owned())
    }

    pub fn has_pending_result(&self) -> bool {
        self.result
            .as_ref()
            .is_some_and(|grid| grid.pending_total() > 0)
    }
    pub fn dirty(&self) -> bool {
        self.editor_state.editor().text() != self.saved_text
    }

    pub fn sync_query_text(&mut self) {
        self.editor_state
            .editor()
            .text()
            .clone_into(&mut self.query);
    }

    pub fn replace_query(&mut self, query: impl Into<String>) {
        let query = query.into();
        *self.editor_state = CompletingCodeEditorState::new(&query);
        self.query = query;
    }

    pub fn commit_query_edit(&mut self) {
        self.editor_state.editor_mut().commit();
        self.sync_query_text();
    }
}

pub fn highlight_sql(src: &str) -> Vec<(core::ops::Range<usize>, SyntaxRole)> {
    sql::tokenize(src)
        .into_iter()
        .filter_map(|token| {
            let role = match token.kind {
                sql::TokKind::Keyword => SyntaxRole::Keyword,
                sql::TokKind::Ident => SyntaxRole::Ident,
                sql::TokKind::Number => SyntaxRole::Number,
                sql::TokKind::String => SyntaxRole::Str,
                sql::TokKind::Operator => SyntaxRole::Operator,
                sql::TokKind::Punct => SyntaxRole::Punct,
                sql::TokKind::Comment => SyntaxRole::Comment,
                sql::TokKind::Whitespace => return None,
            };
            Some((token.start..token.end, role))
        })
        .collect()
}

pub fn segment_sql(src: &str) -> Vec<core::ops::Range<usize>> {
    sql::split_statements(src)
        .into_iter()
        .map(|(start, end)| start..end)
        .collect()
}

fn compact_duration(duration_ms: u32) -> String {
    match duration_ms {
        0 => "<1 ms".to_owned(),
        milliseconds if milliseconds < 1_000 => format!("{milliseconds} ms"),
        milliseconds if milliseconds < 60_000 => {
            format!("{:.2} s", f64::from(milliseconds) / 1_000.0)
        }
        milliseconds => format!(
            "{}m {}s",
            milliseconds / 60_000,
            (milliseconds % 60_000) / 1_000
        ),
    }
}

#[derive(Debug, Clone)]
pub struct HistoryTab {
    pub search: String,
    pub selected: usize,
    pub entries: Vec<HistoryEntry>,
}

impl HistoryTab {
    pub fn new(history: &History) -> Self {
        Self {
            search: String::new(),
            selected: 0,
            entries: history.entries.clone(),
        }
    }
    pub fn filter(&mut self, history: &History) {
        self.entries = history
            .search(&self.search, None, false)
            .into_iter()
            .cloned()
            .collect();
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
    }
    pub fn selected_query(&self) -> Option<String> {
        self.entries
            .get(self.selected)
            .map(|entry| entry.sql.clone())
    }
}

#[derive(Debug, Clone)]
pub enum Tab {
    Table(TableTab),
    Query(QueryTab),
    History(HistoryTab),
}

impl Tab {
    pub fn grid(&self, key: TabKey) -> Option<(Id, &GridView)> {
        match self {
            Self::Table(tab) if tab.is_structure() => {
                Some((key.control("structure"), &tab.structure))
            }
            Self::Table(tab) => Some((key.control("data"), &tab.result)),
            Self::Query(tab) => tab
                .result
                .as_ref()
                .map(|grid| (key.control("results"), grid)),
            Self::History(_) => None,
        }
    }
    pub fn grid_mut(&mut self, key: TabKey) -> Option<(Id, &mut GridView)> {
        match self {
            Self::Table(tab) => {
                if tab.is_structure() {
                    Some((key.control("structure"), &mut tab.structure))
                } else {
                    Some((key.control("data"), &mut tab.result))
                }
            }
            Self::Query(tab) => tab
                .result
                .as_mut()
                .map(|grid| (key.control("results"), grid)),
            Self::History(_) => None,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Table(tab) => tab.table.name.clone(),
            Self::Query(tab) => tab.name.clone(),
            Self::History(_) => "History".to_owned(),
        }
    }
    pub fn dirty(&self) -> bool {
        matches!(self, Self::Table(tab) if tab.result.pending_total() > 0)
            || matches!(self, Self::Query(tab) if tab.dirty() || tab.result.as_ref().is_some_and(|grid| grid.pending_total() > 0))
    }
}

pub struct TabRecord {
    key: TabKey,
    payload: Tab,
    generation: Option<u64>,
    query_outputs: Vec<RetainedQueryOutput>,
    selected_query_output: Option<QueryOutputId>,
    next_query_output_id: Option<u64>,
    query_output_revision: Option<u64>,
    active_query_run: Option<std::sync::Weak<()>>,
}

fn query_tab_output_kind(tab: &QueryTab) -> QueryOutputKind {
    if tab.plan.is_some() {
        QueryOutputKind::Plan
    } else if tab.error.is_some() {
        QueryOutputKind::Error
    } else if tab.result.is_some() {
        QueryOutputKind::Rows
    } else {
        QueryOutputKind::Empty
    }
}

impl TabRecord {
    pub fn new(key: TabKey, payload: Tab) -> Self {
        let mut record = Self {
            key,
            payload,
            generation: Some(0),
            query_outputs: Vec::new(),
            selected_query_output: None,
            next_query_output_id: Some(1),
            query_output_revision: Some(0),
            active_query_run: None,
        };
        record.capture_initial_query_output();
        record
    }

    pub fn payload_mut(&mut self) -> &mut Tab {
        let _ = self.invalidate_query_sql_provenance();
        &mut self.payload
    }

    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }

    pub fn lifecycle_payload_mut(&mut self) -> &mut Tab {
        let _ = self.invalidate_query_sql_provenance();
        &mut self.payload
    }

    pub const fn key(&self) -> TabKey {
        self.key
    }
    pub const fn payload(&self) -> &Tab {
        &self.payload
    }
    pub fn dirty(&self) -> bool {
        self.payload.dirty()
            || self
                .query_outputs
                .iter()
                .filter_map(|output| output.body.as_ref())
                .any(|body| body.pending_total() > 0)
    }

    pub fn pending_query_result_total(&self) -> usize {
        let active = match &self.payload {
            Tab::Query(tab) => tab.result.as_ref().map_or(0, GridView::pending_total),
            _ => 0,
        };
        self.query_outputs
            .iter()
            .filter_map(|output| output.body.as_ref())
            .fold(active, |total, body| {
                total.saturating_add(body.pending_total())
            })
    }

    pub fn has_pending_query_result(&self) -> bool {
        self.pending_query_result_total() > 0
    }

    pub fn query_output_state(&self) -> Option<QueryOutputState> {
        let Tab::Query(_) = self.payload else {
            return None;
        };
        self.generation?;
        Some(QueryOutputState {
            selected: self.selected_query_output,
            revision: self.query_output_revision?,
        })
    }

    pub fn query_run_matches(&self, run: &std::sync::Arc<()>) -> bool {
        self.owns_query_run(run)
    }

    pub fn can_advance_query_currency(&self, increments: u64) -> bool {
        self.next_query_currency(increments).is_some()
    }

    pub fn query_outputs(&self) -> Option<Vec<QueryOutputInfo>> {
        let Tab::Query(tab) = &self.payload else {
            return None;
        };
        self.query_output_state()?;
        Some(
            self.query_outputs
                .iter()
                .map(|output| {
                    let mut info = output.info.clone();
                    if Some(info.id) == self.selected_query_output {
                        let current_kind = query_tab_output_kind(tab);
                        if current_kind != QueryOutputKind::Empty {
                            info.kind = current_kind;
                        } else if info.kind != QueryOutputKind::Cancelled {
                            info.kind = QueryOutputKind::Empty;
                        }
                    }
                    info.closable = !info.pinned && !self.query_output_pending(info.id);
                    info
                })
                .collect(),
        )
    }

    pub fn selected_query_output(&self) -> Option<QueryOutputId> {
        self.selected_query_output
    }

    pub fn append_query_outputs(
        &mut self,
        inputs: Vec<QueryOutputInput>,
    ) -> Option<Vec<QueryOutputId>> {
        if inputs.is_empty() || !matches!(self.payload, Tab::Query(_)) {
            return None;
        }
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return None;
        };
        if !self.can_store_selected_query_body() {
            return None;
        }
        let base_id = self.next_query_output_id?;
        let first_input_id = if self.legacy_body_needs_adoption() {
            base_id.checked_add(1)?
        } else {
            base_id
        };
        let input_count = u64::try_from(inputs.len()).ok()?;
        let next_id = first_input_id.checked_add(input_count)?;
        let mut ids = Vec::with_capacity(inputs.len());
        let mut input_id = first_input_id;
        for _ in &inputs {
            ids.push(QueryOutputId(input_id));
            input_id = input_id.checked_add(1)?;
        }
        if !self.store_selected_query_body() {
            return None;
        }
        let input_len = ids.len();
        for (index, (input, id)) in inputs.into_iter().zip(ids.iter().copied()).enumerate() {
            let QueryOutputInput {
                label,
                source_range,
                duration_ms,
                body,
            } = input;
            let kind = body.kind();
            let info = QueryOutputInfo {
                id,
                label,
                source_range,
                duration_ms,
                kind,
                pinned: false,
                closable: true,
            };
            if index.saturating_add(1) < input_len {
                self.query_outputs.push(RetainedQueryOutput {
                    info,
                    body: Some(body),
                });
            } else {
                self.query_outputs
                    .push(RetainedQueryOutput { info, body: None });
                if let (Tab::Query(tab), Some(output)) =
                    (&mut self.payload, self.query_outputs.last_mut())
                {
                    body.install(tab);
                    output.info.kind = kind;
                }
                self.selected_query_output = Some(id);
            }
        }
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        self.next_query_output_id = Some(next_id);
        Some(ids)
    }

    pub fn select_query_output(&mut self, id: QueryOutputId) -> bool {
        if self.selected_query_output == Some(id) {
            return true;
        }
        let Some(index) = self
            .query_outputs
            .iter()
            .position(|output| output.info.id == id)
        else {
            return false;
        };
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return false;
        };
        if self.query_outputs[index].body.is_none()
            || !matches!(self.payload, Tab::Query(_))
            || !self.can_store_selected_query_body()
            || !self.store_selected_query_body()
        {
            return false;
        }
        let output = &mut self.query_outputs[index];
        let Some(body) = output.body.take() else {
            return false;
        };
        let anchor = output.info.source_range.clone();
        let kind = body.kind();
        let Tab::Query(tab) = &mut self.payload else {
            return false;
        };
        body.install(tab);
        output.info.kind = kind;
        self.selected_query_output = Some(id);
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        if let Some(anchor) = anchor {
            tab.editor_state.editor_mut().jump_to(anchor.start);
        }
        true
    }

    pub fn close_query_output(&mut self, id: QueryOutputId) -> bool {
        let Some(index) = self
            .query_outputs
            .iter()
            .position(|output| output.info.id == id)
        else {
            return false;
        };
        if self.query_outputs[index].info.pinned || self.query_output_pending(id) {
            return false;
        }
        let selected = self.selected_query_output == Some(id);
        let replacement = if selected {
            self.query_outputs
                .iter()
                .skip(index.saturating_add(1))
                .find(|output| output.info.id != id)
                .or_else(|| {
                    self.query_outputs[..index]
                        .iter()
                        .rev()
                        .find(|output| output.info.id != id)
                })
                .map(|output| output.info.id)
        } else {
            None
        };
        let currency_steps = if selected && replacement.is_some() {
            2
        } else {
            1
        };
        if self.next_query_currency(currency_steps).is_none() {
            self.fail_closed_query_currency();
            return false;
        }
        if selected {
            if let Some(replacement) = replacement {
                if !self.select_query_output(replacement) {
                    return false;
                }
            } else if let Tab::Query(tab) = &mut self.payload {
                tab.result = None;
                tab.error = None;
                tab.plan = None;
                self.selected_query_output = None;
            }
        }
        if !self.advance_query_currency(1) {
            return false;
        }
        self.query_outputs.retain(|output| output.info.id != id);
        true
    }

    pub fn toggle_query_output_pin(&mut self, id: QueryOutputId) -> bool {
        let Some(index) = self
            .query_outputs
            .iter()
            .position(|output| output.info.id == id)
        else {
            return false;
        };
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return false;
        };
        self.query_outputs[index].info.pinned = !self.query_outputs[index].info.pinned;
        self.query_outputs.sort_by_key(|output| !output.info.pinned);
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        true
    }

    pub fn prepare_query_outputs_for_run(&mut self) -> bool {
        if !matches!(self.payload, Tab::Query(_)) || self.query_outputs.is_empty() {
            return true;
        }
        let selected = self.selected_query_output;
        let dirty_ids: Vec<_> = self
            .query_outputs
            .iter()
            .filter(|output| self.query_output_pending(output.info.id))
            .map(|output| output.info.id)
            .collect();
        let removes_output = self
            .query_outputs
            .iter()
            .any(|output| !output.info.pinned && !dirty_ids.contains(&output.info.id));
        if !removes_output {
            return true;
        }
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return false;
        };
        let mut removed_selected = false;
        self.query_outputs.retain(|output| {
            let keep = output.info.pinned || dirty_ids.contains(&output.info.id);
            if !keep && Some(output.info.id) == selected {
                removed_selected = true;
            }
            keep
        });
        if removed_selected {
            if let Tab::Query(tab) = &mut self.payload {
                tab.result = None;
                tab.error = None;
                tab.plan = None;
            }
            self.selected_query_output = None;
        }
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        true
    }

    pub fn invalidate_query_output_state(&mut self) -> bool {
        self.invalidate_query_state(false)
    }

    pub fn invalidate_query_sql_provenance(&mut self) -> bool {
        self.invalidate_query_state(true)
    }

    pub fn application_update_payload<R>(
        &mut self,
        update: impl FnOnce(&mut Tab) -> (R, ApplicationMutation),
    ) -> Option<R> {
        let Some((generation, revision)) = self.next_application_currency() else {
            self.fail_closed_query_currency();
            return None;
        };
        let before_sql = match &self.payload {
            Tab::Query(tab) => Some((
                tab.query.clone(),
                tab.editor_state.editor().text().to_owned(),
            )),
            _ => None,
        };
        let (result, mutation) = update(&mut self.payload);
        let sql_changed = match (&before_sql, &self.payload) {
            (Some((query, editor_text)), Tab::Query(tab)) => {
                query != &tab.query || editor_text != tab.editor_state.editor().text()
            }
            (Some(_), _) => true,
            (None, _) => false,
        };
        if mutation == ApplicationMutation::Changed || sql_changed {
            self.generation = Some(generation);
            if let Some(revision) = revision {
                self.query_output_revision = Some(revision);
            }
        }
        if sql_changed {
            self.clear_query_source_ranges();
        }
        Some(result)
    }

    pub fn application_update_query<R>(
        &mut self,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        if !matches!(self.payload, Tab::Query(_)) {
            return None;
        }
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return None;
        };
        let before_sql = match &self.payload {
            Tab::Query(tab) => (
                tab.query.clone(),
                tab.editor_state.editor().text().to_owned(),
            ),
            _ => return None,
        };
        let Tab::Query(tab) = &mut self.payload else {
            return None;
        };
        let result = update(tab);
        let sql_changed =
            before_sql.0 != tab.query || before_sql.1 != tab.editor_state.editor().text();
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        if sql_changed {
            self.clear_query_source_ranges();
        }
        Some(result)
    }

    pub fn begin_query_run<R>(
        &mut self,
        run: &std::sync::Arc<()>,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        if !matches!(&self.payload, Tab::Query(tab) if !tab.running) {
            return None;
        }
        if self
            .active_query_run
            .as_ref()
            .is_some_and(|active| active.upgrade().is_none())
        {
            self.active_query_run = None;
        }
        if self.active_query_run.is_some() {
            return None;
        }
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return None;
        };
        let Tab::Query(tab) = &mut self.payload else {
            return None;
        };
        let before_sql = (
            tab.query.clone(),
            tab.editor_state.editor().text().to_owned(),
        );
        let result = update(tab);
        let sql_changed =
            before_sql.0 != tab.query || before_sql.1 != tab.editor_state.editor().text();
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        self.active_query_run = Some(std::sync::Arc::downgrade(run));
        if sql_changed {
            self.clear_query_source_ranges();
        }
        Some(result)
    }

    pub fn application_update_query_run<R>(
        &mut self,
        run: &std::sync::Arc<()>,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        if !self.owns_query_run(run) || !matches!(&self.payload, Tab::Query(tab) if tab.running) {
            return None;
        }
        let Some((generation, revision)) = self.next_query_currency(1) else {
            self.fail_closed_query_currency();
            return None;
        };
        let Tab::Query(tab) = &mut self.payload else {
            return None;
        };
        let before_sql = (
            tab.query.clone(),
            tab.editor_state.editor().text().to_owned(),
        );
        let result = update(tab);
        let sql_changed =
            before_sql.0 != tab.query || before_sql.1 != tab.editor_state.editor().text();
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        if sql_changed {
            self.clear_query_source_ranges();
        }
        Some(result)
    }

    pub fn finish_query_run(
        &mut self,
        run: &std::sync::Arc<()>,
        status: &str,
        is_error: bool,
        duration_ms: u32,
    ) -> QueryRunFinish {
        if !self.owns_query_run(run) || !matches!(self.payload, Tab::Query(_)) {
            return QueryRunFinish::NotOwned;
        }
        if !matches!(&self.payload, Tab::Query(tab) if tab.running) {
            self.active_query_run = None;
            return QueryRunFinish::AlreadyStopped;
        }
        let currency = self.next_query_currency(1);
        let currency_available = if let Some((generation, revision)) = currency {
            self.generation = Some(generation);
            self.query_output_revision = Some(revision);
            true
        } else {
            self.fail_closed_query_currency();
            false
        };
        let Tab::Query(tab) = &mut self.payload else {
            return QueryRunFinish::NotOwned;
        };
        tab.running = false;
        tab.editor_state.editor_mut().set_running(None);
        if currency_available {
            tab.record_status(status, is_error, duration_ms);
        }
        self.active_query_run = None;
        if currency_available {
            QueryRunFinish::Finished
        } else {
            QueryRunFinish::StoppedWithoutCurrency
        }
    }

    fn owns_query_run(&self, run: &std::sync::Arc<()>) -> bool {
        self.active_query_run
            .as_ref()
            .is_some_and(|active| std::sync::Weak::ptr_eq(active, &std::sync::Arc::downgrade(run)))
    }

    pub fn application_grid_mut(&mut self) -> Option<(Id, &mut GridView)> {
        self.payload.grid(self.key)?;
        let Some((generation, revision)) = self.next_application_currency() else {
            self.fail_closed_query_currency();
            return None;
        };
        self.generation = Some(generation);
        if let Some(revision) = revision {
            self.query_output_revision = Some(revision);
        }
        self.payload.grid_mut(self.key)
    }

    fn invalidate_query_state(&mut self, clear_source_ranges: bool) -> bool {
        let Some((generation, revision)) = self.next_application_currency() else {
            self.fail_closed_query_currency();
            return false;
        };
        self.generation = Some(generation);
        if let Some(revision) = revision {
            self.query_output_revision = Some(revision);
        }
        if clear_source_ranges {
            self.clear_query_source_ranges();
        }
        true
    }

    fn next_application_currency(&self) -> Option<(u64, Option<u64>)> {
        let generation = self.generation?.checked_add(1)?;
        let revision = if matches!(self.payload, Tab::Query(_)) {
            Some(self.query_output_revision?.checked_add(1)?)
        } else {
            None
        };
        Some((generation, revision))
    }

    fn next_query_currency(&self, increments: u64) -> Option<(u64, u64)> {
        Some((
            self.generation?.checked_add(increments)?,
            self.query_output_revision?.checked_add(increments)?,
        ))
    }

    fn advance_query_currency(&mut self, increments: u64) -> bool {
        let Some((generation, revision)) = self.next_query_currency(increments) else {
            self.fail_closed_query_currency();
            return false;
        };
        self.generation = Some(generation);
        self.query_output_revision = Some(revision);
        true
    }

    fn fail_closed_query_currency(&mut self) {
        self.generation = self
            .generation
            .and_then(|generation| generation.checked_add(1));
        if matches!(self.payload, Tab::Query(_)) {
            self.query_output_revision = None;
        }
    }

    fn clear_query_source_ranges(&mut self) {
        for output in &mut self.query_outputs {
            output.info.source_range = None;
        }
    }

    fn legacy_body_needs_adoption(&self) -> bool {
        self.selected_query_output.is_none()
            && matches!(&self.payload, Tab::Query(tab) if query_tab_output_kind(tab) != QueryOutputKind::Empty)
    }

    fn capture_initial_query_output(&mut self) {
        let Some(kind) = (match &self.payload {
            Tab::Query(tab) => {
                let kind = query_tab_output_kind(tab);
                (kind != QueryOutputKind::Empty).then_some(kind)
            }
            _ => None,
        }) else {
            return;
        };
        let id = QueryOutputId(1);
        self.query_outputs.push(RetainedQueryOutput {
            info: QueryOutputInfo {
                id,
                label: "Result 1".to_owned(),
                source_range: None,
                duration_ms: 0,
                kind,
                pinned: false,
                closable: true,
            },
            body: None,
        });
        self.selected_query_output = Some(id);
        self.next_query_output_id = Some(2);
    }

    fn store_selected_query_body(&mut self) -> bool {
        let id = if let Some(id) = self.selected_query_output {
            id
        } else {
            let kind = match &self.payload {
                Tab::Query(tab) => query_tab_output_kind(tab),
                _ => return false,
            };
            if kind == QueryOutputKind::Empty {
                return true;
            }
            let Some(raw_id) = self.next_query_output_id else {
                return false;
            };
            let Some(next_id) = raw_id.checked_add(1) else {
                return false;
            };
            self.query_outputs.push(RetainedQueryOutput {
                info: QueryOutputInfo {
                    id: QueryOutputId(raw_id),
                    label: format!("Result {raw_id}"),
                    source_range: None,
                    duration_ms: 0,
                    kind,
                    pinned: false,
                    closable: true,
                },
                body: None,
            });
            self.selected_query_output = Some(QueryOutputId(raw_id));
            self.next_query_output_id = Some(next_id);
            QueryOutputId(raw_id)
        };
        let Some(index) = self
            .query_outputs
            .iter()
            .position(|output| output.info.id == id)
        else {
            return false;
        };
        if self.query_outputs[index].body.is_some() {
            return false;
        }
        let Tab::Query(tab) = &mut self.payload else {
            return false;
        };
        let kind = self.query_outputs[index].info.kind;
        let body = QueryOutputBody::from_query_tab(tab, kind);
        self.query_outputs[index].info.kind = body.kind();
        self.query_outputs[index].body = Some(body);
        true
    }

    fn query_output_pending(&self, id: QueryOutputId) -> bool {
        if self.selected_query_output == Some(id) {
            return matches!(&self.payload, Tab::Query(tab) if tab.result.as_ref().is_some_and(|grid| grid.pending_total() > 0));
        }
        self.query_outputs
            .iter()
            .find(|output| output.info.id == id)
            .and_then(|output| output.body.as_ref())
            .is_some_and(|body| body.pending_total() > 0)
    }

    fn can_store_selected_query_body(&self) -> bool {
        if let Some(id) = self.selected_query_output {
            return self
                .query_outputs
                .iter()
                .find(|output| output.info.id == id)
                .is_some_and(|output| output.body.is_none());
        }
        let Tab::Query(tab) = &self.payload else {
            return false;
        };
        query_tab_output_kind(tab) == QueryOutputKind::Empty
            || self
                .next_query_output_id
                .is_some_and(|id| id.checked_add(1).is_some())
    }
    pub fn grid(&self) -> Option<(Id, &GridView)> {
        self.payload.grid(self.key)
    }
}

impl core::fmt::Debug for TabRecord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let kind = match self.payload {
            Tab::Table(_) => "table",
            Tab::Query(_) => "query",
            Tab::History(_) => "history",
        };
        f.debug_struct("TabRecord")
            .field("key", &self.key)
            .field("kind", &kind)
            .field("generation", &self.generation)
            .field("dirty", &self.dirty())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub schema: String,
    pub name: String,
    pub kind: tablepro_demo::db::ObjectKind,
    pub rows: usize,
}

pub fn explorer_items(catalog: &Catalog) -> Vec<ExplorerItem> {
    catalog
        .tables
        .iter()
        .map(|table| ExplorerItem {
            schema: table.schema.clone(),
            name: table.name.clone(),
            kind: table.kind,
            rows: table.row_count,
        })
        .collect()
}
