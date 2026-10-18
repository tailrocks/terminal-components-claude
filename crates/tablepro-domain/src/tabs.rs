#![doc = r" `TablePro` tab models. Each tab owns product state; terminal components"]
#![doc = r" only receive controlled values and generic grid adapters."]
#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]

use crate::domain::ResultGrid;
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry};
use tablepro_demo::db::{Catalog, ColType, Table, Value};
use tablepro_sql::sql::{self, PlanNode};
use termrock::{GridModel, GridState, Id, ItemKey, TextInputState};

#[doc = r" Stable identity for an open workbench tab."]
#[doc = r""]
#[doc = r" The identity is allocated by the workbench and remains"]
#[doc = r" stable when another tab is inserted or closed.  UI collections use this"]
#[doc = r" value rather than a position, so focus and pending state cannot drift to a"]
#[doc = r" neighbouring tab."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabKey(u64);

impl TabKey {
    #[doc = r" Build a tab identity from a monotonically increasing value."]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[doc = r" Return the underlying value for diagnostics and keyed UI adapters."]
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    #[doc = r" Namespace a control by this logical tab, independent of display position."]
    pub fn control(self, name: &'static str) -> Id {
        Id::root("tablepro.tab")
            .item(ItemKey::num(self.0))
            .sub(name)
    }
}

#[doc = r" One tab's grid, with borrowed column props independent of mutable row data."]
#[derive(Clone)]
pub struct GridView {
    #[doc = r" Column metadata published with this result."]
    pub columns: Vec<(String, ColType)>,
    #[doc = r" Sole owner of row values, pending edits and undo state."]
    pub model: ResultGrid,
    #[doc = r" Durable selection, scrolling and editor state for this grid."]
    pub state: GridState,
}

impl GridView {
    #[doc = r" Publish a complete result and its matching headers together."]
    pub fn from_result(result: &sql::ResultSet) -> Self {
        Self {
            columns: result.columns.clone(),
            model: ResultGrid::from_result(result),
            state: GridState::default(),
        }
    }

    #[doc = r" Pending row operations, including an uncommitted changed inline draft."]
    #[doc = r" An existing row update or inserted row already accounts for that draft."]
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

#[doc = r" Table tab body."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    #[doc = r" Data grid mode."]
    Data,
    #[doc = r" Structure grid mode."]
    Structure,
}

#[doc = r" A table tab with data and structure modes."]
#[derive(Debug, Clone)]
pub struct TableTab {
    #[doc = r" Catalog table."]
    pub table: Table,
    #[doc = r" Current mode."]
    pub mode: TableMode,
    #[doc = r" Data result adapter."]
    pub result: GridView,
    #[doc = r" Independent, read-only schema grid; switching modes never replaces data."]
    pub structure: Box<GridView>,
    #[doc = r" Active local filters."]
    pub filters: Vec<Filter>,
}

impl TableTab {
    #[doc = r" Load a bounded deterministic result for a table."]
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
    #[doc = r" Toggle Data/Structure."]
    pub const fn toggle_structure(&mut self) {
        self.mode = match self.mode {
            TableMode::Data => TableMode::Structure,
            TableMode::Structure => TableMode::Data,
        };
    }
    #[doc = r" Whether the table is in structure mode."]
    pub const fn is_structure(&self) -> bool {
        matches!(self.mode, TableMode::Structure)
    }
    #[doc = r" Add or replace a filter chip."]
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
    #[doc = r" Remove all filters."]
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }
    #[doc = r" Apply a local sort while preserving adapter row identity."]
    pub fn sort(&mut self, column: usize, direction: termrock::SortDir) {
        self.result.model.sort(
            termrock::ColumnKey::num((column as u16).saturating_add(1)),
            direction,
        );
    }
    #[doc = r" Structure rows as generic grid data."]
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
    #[doc = r" Structure column definitions."]
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
    #[doc = r" Exact pending SQL preview."]
    pub fn preview(&self) -> Vec<String> {
        crate::grid_model::preview_for(&self.table, &self.result)
    }
}

#[doc = r" Query editor tab."]
#[derive(Clone)]
pub struct QueryTab {
    #[doc = r" Stable tab id."]
    pub id: usize,
    #[doc = r" Display name."]
    pub name: String,
    #[doc = r" SQL text."]
    pub query: String,
    #[doc = r" Caller-owned editor draft, cursor and selection for this query."]
    pub editor_state: TextInputState,
    #[doc = r" Last saved editor text; executing a query does not save it."]
    pub saved_text: String,
    #[doc = r" Last result, when successful."]
    pub result: Option<GridView>,
    #[doc = r" Last execution error."]
    pub error: Option<String>,
    #[doc = r" Last explain plan."]
    pub plan: Option<PlanNode>,
    #[doc = r" Whether execution is in flight."]
    pub running: bool,
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
            .finish()
    }
}

impl QueryTab {
    #[doc = r" New empty query tab."]
    pub fn new(id: usize, query: impl Into<String>) -> Self {
        let query = query.into();
        Self {
            id,
            name: format!("Query {id}"),
            saved_text: query.clone(),
            query,
            editor_state: TextInputState::default(),
            result: None,
            error: None,
            plan: None,
            running: false,
        }
    }
    #[doc = r" Execute this query through the deterministic app-owned engine."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns a parser or executor message when the query is unsupported or"]
    #[doc = r" the catalog cannot evaluate it."]
    pub fn execute(&mut self, catalog: &Catalog) -> Result<usize, String> {
        if self.has_pending_result() {
            return Err("Pending result edits require confirmation".to_owned());
        }
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
    #[doc = r" Build an explain plan for this query."]
    #[doc = r""]
    #[doc = r" # Errors"]
    #[doc = r""]
    #[doc = r" Returns a parser or planner message when the query is not a supported"]
    #[doc = r" `SELECT` statement."]
    pub fn explain(&mut self, catalog: &Catalog) -> Result<(), String> {
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("Explain accepts SELECT statements".to_owned());
        };
        self.plan = Some(sql::explain(catalog, &select, false).map_err(|error| error.message)?);
        Ok(())
    }
    #[doc = r" Whether replacing this result would discard committed or retained inline edits."]
    pub fn has_pending_result(&self) -> bool {
        self.result
            .as_ref()
            .is_some_and(|grid| grid.pending_total() > 0)
    }
    #[doc = r" Whether the editor has changed text since its last saved copy."]
    pub fn dirty(&self) -> bool {
        self.editor_state.draft_text().unwrap_or(&self.query) != self.saved_text
    }
}

#[doc = r" History tab."]
#[derive(Debug, Clone)]
pub struct HistoryTab {
    #[doc = r" Search text."]
    pub search: String,
    #[doc = r" Selected entry index."]
    pub selected: usize,
    #[doc = r" Filtered entries."]
    pub entries: Vec<HistoryEntry>,
}

impl HistoryTab {
    #[doc = r" Build from history."]
    pub fn new(history: &History) -> Self {
        Self {
            search: String::new(),
            selected: 0,
            entries: history.entries.clone(),
        }
    }
    #[doc = r" Search this tab."]
    pub fn filter(&mut self, history: &History) {
        self.entries = history
            .search(&self.search, None, false)
            .into_iter()
            .cloned()
            .collect();
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
    }
    #[doc = r" Reopen the selected query."]
    pub fn selected_query(&self) -> Option<String> {
        self.entries
            .get(self.selected)
            .map(|entry| entry.sql.clone())
    }
}

#[doc = r" Product tab union."]
#[derive(Debug, Clone)]
pub enum Tab {
    #[doc = r" Table data or structure tab."]
    Table(TableTab),
    #[doc = r" SQL query tab."]
    Query(QueryTab),
    #[doc = r" Query history tab."]
    History(HistoryTab),
}

impl Tab {
    #[doc = r" Current grid and its stable control identity."]
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
    #[doc = r" Current grid mutably, with no mirrored model or interaction state."]
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

    #[doc = r" Display label."]
    pub fn label(&self) -> String {
        match self {
            Self::Table(tab) => tab.table.name.clone(),
            Self::Query(tab) => tab.name.clone(),
            Self::History(_) => "History".to_owned(),
        }
    }
    #[doc = r" Whether this tab owns pending changes."]
    pub fn dirty(&self) -> bool {
        matches!(self, Self::Table(tab) if tab.result.pending_total() > 0)
            || matches!(self, Self::Query(tab) if tab.dirty() || tab.result.as_ref().is_some_and(|grid| grid.pending_total() > 0))
    }
}

#[doc = r" An owned tab identity, separate from its freely replaceable payload."]
#[doc = r" Construction and structural mutation belong exclusively to the workbench."]
#[doc = r""]
#[doc = r" Payload cloning never includes its enclosing identity; record keys are private."]
#[doc = r" ```compile_fail"]
#[doc = r" let mut app = tablepro_ui::TableProApp::default();"]
#[doc = r#" app.workbench.new_query("");"#]
#[doc = r" app.workbench.tabs()[0].key = app.workbench.active_key().unwrap();"]
#[doc = r" ```"]
#[doc = r" The structural collection cannot be replaced or extended directly."]
#[doc = r" ```compile_fail"]
#[doc = r" let mut app = tablepro_ui::TableProApp::default();"]
#[doc = r#" app.workbench.tabs.push(tablepro_ui::Tab::Query(tablepro_ui::QueryTab::new(1, "")));"#]
#[doc = r" ```"]
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

    #[doc = r" Immutable logical identity, never copied from an inserted payload."]
    pub const fn key(&self) -> TabKey {
        self.key
    }
    #[doc = r" Borrow the application payload without structural mutation access."]
    pub const fn payload(&self) -> &Tab {
        &self.payload
    }
    #[doc = r" Whether this record owns unsaved work."]
    pub fn dirty(&self) -> bool {
        self.payload.dirty()
    }
    #[doc = r" Current grid and its enclosing record's stable control identity."]
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

#[doc = r" One explorer row."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    #[doc = r" Schema name."]
    pub schema: String,
    #[doc = r" Object name."]
    pub name: String,
    #[doc = r" Catalog object kind."]
    pub kind: tablepro_demo::db::ObjectKind,
    #[doc = r" Estimated row count."]
    pub rows: usize,
}

#[doc = r" Build explorer rows from a catalog."]
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
