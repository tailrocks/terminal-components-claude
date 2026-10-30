#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]

use crate::domain::ResultGrid;
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry};
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
    /// Shared multiline editor and completion state for this query tab.
    pub editor_state: Box<CompletingCodeEditorState>,
    pub saved_text: String,
    pub result: Option<GridView>,
    pub error: Option<String>,
    pub plan: Option<PlanNode>,
    pub running: bool,
    /// Most recent execution summary and whether it represents an error.
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
        self.sync_query_text();
        if self.has_pending_result() {
            return Err("Pending result edits require confirmation".to_owned());
        }
        let started = Instant::now();
        let outcome = (|| {
            let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
            let sql::Statement::Select(select) = statement else {
                return Err("The demo executor only runs SELECT statements".to_owned());
            };
            let result = sql::run_select(catalog, &select).map_err(|error| error.message)?;
            let rows = result.rows.len();
            self.result = Some(GridView::from_result(&result));
            self.error = None;
            Ok(rows)
        })();
        self.record_execution_status(outcome.is_ok(), started.elapsed());
        outcome
    }
    pub fn explain(&mut self, catalog: &Catalog) -> Result<(), String> {
        self.sync_query_text();
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("Explain accepts SELECT statements".to_owned());
        };
        self.plan = Some(sql::explain(catalog, &select, false).map_err(|error| error.message)?);
        Ok(())
    }
    pub fn has_pending_result(&self) -> bool {
        self.result
            .as_ref()
            .is_some_and(|grid| grid.pending_total() > 0)
    }
    pub fn dirty(&self) -> bool {
        self.editor_state.editor().text() != self.saved_text
    }

    /// Copies the editor's authoritative buffer into the SQL execution snapshot.
    pub fn sync_query_text(&mut self) {
        self.editor_state
            .editor()
            .text()
            .clone_into(&mut self.query);
    }

    /// Replaces the query and resets editor mechanics for a newly loaded statement.
    pub fn replace_query(&mut self, query: impl Into<String>) {
        let query = query.into();
        *self.editor_state = CompletingCodeEditorState::new(&query);
        self.query = query;
    }

    /// Commits active editing without changing the editor's text.
    pub fn commit_query_edit(&mut self) {
        self.editor_state.editor_mut().commit();
        self.sync_query_text();
    }
}

/// SQL syntax ranges used by the shared code editor.
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

/// Statement boundaries used for running the SQL statement at the cursor.
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
}

impl TabRecord {
    pub fn new(key: TabKey, payload: Tab) -> Self {
        Self {
            key,
            payload,
            generation: Some(0),
        }
    }
    pub fn payload_mut(&mut self) -> &mut Tab {
        self.generation = self
            .generation
            .and_then(|generation| generation.checked_add(1));
        &mut self.payload
    }
    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }

    pub fn lifecycle_payload_mut(&mut self) -> &mut Tab {
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

#[cfg(test)]
mod generation_tests {
    use super::*;
    #[test]
    fn mutable_payload_generation_never_reuses_an_exhausted_value() {
        let mut record = TabRecord::new(TabKey::new(1), Tab::Query(QueryTab::new(1, "")));
        assert_eq!(record.generation(), Some(0));
        let _ = record.payload_mut();
        assert_eq!(record.generation(), Some(1));
        record.generation = Some(u64::MAX);
        let _ = record.payload_mut();
        assert_eq!(record.generation(), None);
        let _ = record.payload_mut();
        assert_eq!(record.generation(), None);
        let _ = record.lifecycle_payload_mut();
        assert_eq!(record.generation(), None);
    }
}
