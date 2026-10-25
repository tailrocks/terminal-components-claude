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
        self.prepare_execution()?;
        let started = Instant::now();
        let outcome = self.execute_untimed(catalog);
        self.record_execution_status(outcome.is_ok(), started.elapsed());
        outcome
    }

    /// Executes the query while recording the supplied elapsed duration.
    ///
    /// This deterministic path does not sample a host clock.
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

    /// Explains the selected statement or the statement under the editor cursor.
    ///
    /// Returns the exact SQL source used to create the successful plan.
    ///
    /// # Errors
    ///
    /// Returns an error when the selected or cursor statement is empty, invalid,
    /// is not a SELECT, or cannot be planned.
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

#[cfg(test)]
mod query_execution_timing_tests {
    use super::*;

    fn result_values(tab: &QueryTab) -> (Vec<(String, ColType)>, Vec<Vec<Value>>) {
        let Some(grid) = tab.result.as_ref() else {
            unreachable!("query result")
        };
        let rows = (0..grid.model.row_count())
            .map(|row| {
                (0..grid.columns.len())
                    .map(|column| {
                        let Some(value) = grid.model.pending().value(row, column) else {
                            unreachable!("result cell")
                        };
                        value.clone()
                    })
                    .collect()
            })
            .collect();
        (grid.columns.clone(), rows)
    }

    fn status(tab: &QueryTab) -> (String, bool, u32) {
        let Some(status) = tab.last_status.as_deref() else {
            unreachable!("execution status")
        };
        (status.summary.clone(), status.is_error, status.duration_ms)
    }

    #[test]
    fn explicit_duration_changes_only_the_reported_query_timing() {
        let catalog = Catalog::acme_prod();
        let query = "SELECT id, currency FROM orders LIMIT 2";
        let mut live = QueryTab::new(1, query);
        let mut fixed = live.clone();

        assert_eq!(
            live.execute(&catalog),
            fixed.execute_with_duration(&catalog, Duration::from_millis(1234))
        );
        assert!(!status(&live).1);
        assert_eq!(status(&fixed), ("ok · 1.23 s".to_owned(), false, 1234));
        assert_eq!(result_values(&live), result_values(&fixed));
    }

    #[test]
    fn explicit_duration_clamps_milliseconds_and_formats_submillisecond_time() {
        let catalog = Catalog::acme_prod();
        let mut tab = QueryTab::new(1, "SELECT id FROM orders LIMIT 1");

        assert_eq!(
            tab.execute_with_duration(&catalog, Duration::from_nanos(999_999)),
            Ok(1)
        );
        assert_eq!(status(&tab), ("ok · <1 ms".to_owned(), false, 0));

        let beyond_u32 = Duration::from_millis(u64::from(u32::MAX) + 1);
        assert_eq!(tab.execute_with_duration(&catalog, beyond_u32), Ok(1));
        assert_eq!(
            status(&tab),
            ("ok · 71582m 47s".to_owned(), false, u32::MAX)
        );
    }

    #[test]
    fn explicit_duration_records_errors_without_replacing_the_last_result() {
        let catalog = Catalog::acme_prod();
        let mut tab = QueryTab::new(1, "SELECT id, currency FROM orders LIMIT 2");
        assert_eq!(
            tab.execute_with_duration(&catalog, Duration::from_millis(10)),
            Ok(2)
        );
        let result = result_values(&tab);

        tab.replace_query("SELECT FROM");
        assert!(
            tab.execute_with_duration(&catalog, Duration::from_millis(1234))
                .is_err()
        );
        assert_eq!(status(&tab), ("error".to_owned(), true, 1234));
        assert_eq!(result_values(&tab), result);

        tab.replace_query("SELECT missing_column FROM orders LIMIT 1");
        assert!(
            tab.execute_with_duration(&catalog, Duration::from_millis(2345))
                .is_err()
        );
        assert_eq!(status(&tab), ("error".to_owned(), true, 2345));
        assert_eq!(result_values(&tab), result);
    }

    #[test]
    fn pending_result_rejection_preserves_status_and_pending_rows() {
        let catalog = Catalog::acme_prod();
        let mut tab = QueryTab::new(1, "SELECT id, currency FROM orders LIMIT 1");
        assert_eq!(
            tab.execute_with_duration(&catalog, Duration::from_millis(1234)),
            Ok(1)
        );
        let previous_status = status(&tab);
        let Some(grid) = tab.result.as_mut() else {
            unreachable!("result")
        };
        assert!(grid.model.delete_row(0));
        let pending = grid.model.pending().clone();

        assert_eq!(
            tab.execute_with_duration(&catalog, Duration::from_millis(9000)),
            Err("Pending result edits require confirmation".to_owned())
        );
        assert_eq!(status(&tab), previous_status);
        let Some(grid) = tab.result.as_ref() else {
            unreachable!("result")
        };
        assert_eq!(grid.model.pending(), &pending);
    }
}

#[cfg(test)]
mod query_explain_tests {
    use super::*;

    const MULTI_STATEMENT_SQL: &str =
        "SELECT * FROM customers WHERE id = '界;🙂'; -- ; comment\nSELECT * FROM orders;";

    fn plan_relation(plan: &PlanNode) -> Option<&str> {
        plan.relation
            .as_deref()
            .or_else(|| plan.children.iter().find_map(|child| plan_relation(child)))
    }

    fn explained_relation(tab: &QueryTab) -> Option<&str> {
        tab.plan.as_ref().and_then(plan_relation)
    }

    #[test]
    fn explain_and_analyze_use_the_live_statement_at_the_cursor() {
        let catalog = Catalog::acme_prod();
        let mut tab = QueryTab::new(1, "SELECT * FROM customers");
        tab.editor_state.editor_mut().set_text(MULTI_STATEMENT_SQL);
        tab.editor_state.editor_mut().begin_edit();
        tab.editor_state
            .editor_mut()
            .jump_to(MULTI_STATEMENT_SQL.len());
        let saved_text = tab.saved_text.clone();
        let cursor = tab.editor_state.editor().cursor_offset();
        let selection = tab.editor_state.editor().selected_text().map(str::to_owned);
        assert!(tab.dirty());

        assert_eq!(tab.explain(&catalog), Ok(()));
        assert_eq!(explained_relation(&tab), Some("public.orders"));
        assert!(
            tab.plan
                .as_ref()
                .is_some_and(|plan| plan.actual_ms.is_none())
        );
        assert_eq!(tab.query, MULTI_STATEMENT_SQL);
        assert_eq!(tab.editor_state.editor().text(), MULTI_STATEMENT_SQL);
        assert_eq!(tab.editor_state.editor().cursor_offset(), cursor);
        assert_eq!(
            tab.editor_state.editor().selected_text().map(str::to_owned),
            selection
        );
        assert!(tab.editor_state.editor().is_editing());
        assert_eq!(tab.saved_text, saved_text);
        assert!(tab.dirty());

        assert_eq!(
            tab.explain_statement(&catalog, true),
            Ok("-- ; comment\nSELECT * FROM orders".to_owned())
        );
        assert_eq!(explained_relation(&tab), Some("public.orders"));
        assert!(
            tab.plan
                .as_ref()
                .is_some_and(|plan| plan.actual_ms.is_some())
        );
        assert_eq!(tab.query, MULTI_STATEMENT_SQL);
        assert_eq!(tab.editor_state.editor().cursor_offset(), cursor);
        assert_eq!(
            tab.editor_state.editor().selected_text().map(str::to_owned),
            selection
        );
        assert!(tab.editor_state.editor().is_editing());
        assert_eq!(tab.saved_text, saved_text);
        assert!(tab.dirty());

        let Some(first_statement) = MULTI_STATEMENT_SQL.find("customers") else {
            unreachable!("first statement anchor exists")
        };
        tab.editor_state.editor_mut().jump_to(first_statement);
        let first_cursor = tab.editor_state.editor().cursor_offset();
        assert_eq!(tab.explain(&catalog), Ok(()));
        assert_eq!(explained_relation(&tab), Some("public.customers"));
        assert!(
            tab.plan
                .as_ref()
                .is_some_and(|plan| plan.actual_ms.is_none())
        );
        assert_eq!(tab.editor_state.editor().cursor_offset(), first_cursor);
        assert_eq!(tab.editor_state.editor().text(), MULTI_STATEMENT_SQL);
        assert_eq!(tab.query, MULTI_STATEMENT_SQL);
    }

    #[test]
    fn empty_and_non_select_explain_errors_keep_the_last_plan() {
        let catalog = Catalog::acme_prod();
        let mut tab = QueryTab::new(1, "SELECT * FROM orders");
        assert_eq!(tab.explain(&catalog), Ok(()));
        let previous_plan = tab.plan.clone();

        tab.replace_query("");
        assert_eq!(
            tab.explain(&catalog),
            Err("No SQL statement at cursor".to_owned())
        );
        assert_eq!(tab.plan, previous_plan);

        tab.replace_query("DELETE FROM orders;");
        assert_eq!(
            tab.explain_statement(&catalog, true),
            Err("Explain accepts SELECT statements".to_owned())
        );
        assert_eq!(tab.plan, previous_plan);
    }

    #[test]
    fn selected_statement_ignores_trailing_line_comment_only_range() {
        let selected = "SELECT * FROM customers;\n-- trailing ; comment";

        assert_eq!(
            QueryTab::explain_selection_statement(selected),
            Ok("SELECT * FROM customers")
        );
    }

    #[test]
    fn selected_statement_ignores_trailing_block_comment_only_range() {
        let selected = "SELECT * FROM customers; /* trailing ; comment */";

        assert_eq!(
            QueryTab::explain_selection_statement(selected),
            Ok("SELECT * FROM customers")
        );
    }

    #[test]
    fn comment_only_selection_is_rejected_as_empty() {
        let selected = "-- comment; still comment\n/* another ; comment */";

        assert_eq!(
            QueryTab::explain_selection_statement(selected),
            Err("Explain requires a non-empty selection".to_owned())
        );
    }

    #[test]
    fn selected_batch_counts_only_meaningful_statements() {
        let selected = "SELECT * FROM customers; -- comment\nSELECT * FROM orders;";

        assert_eq!(
            QueryTab::explain_selection_statement(selected),
            Err("Explain selection must contain exactly one statement".to_owned())
        );
    }
}
