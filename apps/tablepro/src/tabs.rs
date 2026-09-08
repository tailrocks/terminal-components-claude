//! `TablePro` tab models. Each tab owns product state; terminal components
//! only receive controlled values and generic grid adapters.

use crate::db::{Catalog, ColType, Table, Value};
use crate::domain::ResultGrid;
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry};
use crate::sql::{self, PlanNode};
use junie_tui::{GridModel, GridState, Id, ItemKey, TextInputState};

/// Stable identity for an open workbench tab.
///
/// The identity is allocated by [`crate::workbench::Workbench`] and remains
/// stable when another tab is inserted or closed.  UI collections use this
/// value rather than a position, so focus and pending state cannot drift to a
/// neighbouring tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabKey(u64);

impl TabKey {
    /// Build a tab identity from a monotonically increasing value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the underlying value for diagnostics and keyed UI adapters.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Namespace a control by this logical tab, independent of display position.
    pub fn control(self, name: &'static str) -> Id {
        Id::root("tablepro.tab")
            .item(ItemKey::num(self.0))
            .sub(name)
    }
}

/// One tab's grid, with borrowed column props independent of mutable row data.
#[derive(Clone)]
pub struct GridView {
    /// Column metadata published with this result.
    pub columns: Vec<(String, ColType)>,
    /// Sole owner of row values, pending edits and undo state.
    pub model: ResultGrid,
    /// Durable selection, scrolling and editor state for this grid.
    pub state: GridState,
}

impl GridView {
    /// Publish a complete result and its matching headers together.
    pub fn from_result(result: &sql::ResultSet) -> Self {
        Self {
            columns: result.columns.clone(),
            model: ResultGrid::from_result(result),
            state: GridState::default(),
        }
    }

    /// Pending row operations, including an uncommitted changed inline draft.
    /// An existing row update or inserted row already accounts for that draft.
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

/// Table tab body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableMode {
    /// Data grid mode.
    Data,
    /// Structure grid mode.
    Structure,
}

/// A table tab with data and structure modes.
#[derive(Debug, Clone)]
pub struct TableTab {
    /// Stable identity in the workbench tab strip.
    pub key: TabKey,
    /// Catalog table.
    pub table: Table,
    /// Current mode.
    pub(crate) mode: TableMode,
    /// Data result adapter.
    pub result: GridView,
    /// Independent, read-only schema grid; switching modes never replaces data.
    pub structure: Box<GridView>,
    /// Active local filters.
    pub filters: Vec<Filter>,
}

impl TableTab {
    /// Load a bounded deterministic result for a table.
    pub fn new(table: Table, catalog: &Catalog) -> Self {
        Self::with_key(TabKey::new(0), table, catalog)
    }

    /// Load a table tab with a caller-assigned stable identity.
    pub fn with_key(key: TabKey, table: Table, catalog: &Catalog) -> Self {
        let query = format!("SELECT * FROM {}.{}", table.schema, table.name);
        let result = sql::parse(&query)
            .ok()
            .and_then(|statement| match statement {
                sql::Statement::Select(select) => sql::run_select(catalog, &select).ok(),
                _ => None,
            })
            .map_or_else(GridView::empty, |result| GridView::from_result(&result));
        let mut tab = Self {
            key,
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
    /// Toggle Data/Structure.
    pub const fn toggle_structure(&mut self) {
        self.mode = match self.mode {
            TableMode::Data => TableMode::Structure,
            TableMode::Structure => TableMode::Data,
        };
    }
    /// Whether the table is in structure mode.
    pub const fn is_structure(&self) -> bool {
        matches!(self.mode, TableMode::Structure)
    }
    /// Add or replace a filter chip.
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
    /// Remove all filters.
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }
    /// Apply a local sort while preserving adapter row identity.
    pub fn sort(&mut self, column: usize, direction: junie_tui::SortDir) {
        self.result.model.sort(
            junie_tui::ColumnKey::num((column as u16).saturating_add(1)),
            direction,
        );
    }
    /// Structure rows as generic grid data.
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
    /// Structure column definitions.
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
    /// Exact pending SQL preview.
    pub fn preview(&self) -> Vec<String> {
        crate::grid_model::preview_for(&self.table, &self.result)
    }
}

/// Query editor tab.
#[derive(Clone)]
pub struct QueryTab {
    /// Stable identity in the workbench tab strip.
    pub key: TabKey,
    /// Stable tab id.
    pub id: usize,
    /// Display name.
    pub name: String,
    /// SQL text.
    pub query: String,
    /// Caller-owned editor draft, cursor and selection for this query.
    pub editor_state: TextInputState,
    /// Last saved editor text; executing a query does not save it.
    pub saved_text: String,
    /// Last result, when successful.
    pub result: Option<GridView>,
    /// Last execution error.
    pub error: Option<String>,
    /// Last explain plan.
    pub(crate) plan: Option<PlanNode>,
    /// Whether execution is in flight.
    pub running: bool,
}

impl core::fmt::Debug for QueryTab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QueryTab")
            .field("key", &self.key)
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
    /// New empty query tab.
    pub fn new(id: usize, query: impl Into<String>) -> Self {
        Self::with_key(TabKey::new(id as u64), id, query)
    }

    /// Build a query tab with a caller-assigned stable identity.
    pub fn with_key(key: TabKey, id: usize, query: impl Into<String>) -> Self {
        let query = query.into();
        Self {
            key,
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
    /// Execute this query through the deterministic app-owned engine.
    ///
    /// # Errors
    ///
    /// Returns a parser or executor message when the query is unsupported or
    /// the catalog cannot evaluate it.
    pub fn execute(&mut self, catalog: &Catalog) -> Result<usize, String> {
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
    /// Build an explain plan for this query.
    ///
    /// # Errors
    ///
    /// Returns a parser or planner message when the query is not a supported
    /// `SELECT` statement.
    pub fn explain(&mut self, catalog: &Catalog) -> Result<(), String> {
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("Explain accepts SELECT statements".to_owned());
        };
        self.plan = Some(sql::explain(catalog, &select, false).map_err(|error| error.message)?);
        Ok(())
    }
    /// Whether the editor has changed text since its last saved copy.
    pub fn dirty(&self) -> bool {
        self.editor_state.draft_text().unwrap_or(&self.query) != self.saved_text
    }
}

/// History tab.
#[derive(Debug, Clone)]
pub struct HistoryTab {
    /// Stable identity in the workbench tab strip.
    pub key: TabKey,
    /// Search text.
    pub search: String,
    /// Selected entry index.
    pub selected: usize,
    /// Filtered entries.
    pub(crate) entries: Vec<HistoryEntry>,
}

impl HistoryTab {
    /// Build from history.
    pub fn new(history: &History) -> Self {
        Self::with_key(TabKey::new(0), history)
    }

    /// Build a history tab with a caller-assigned stable identity.
    pub fn with_key(key: TabKey, history: &History) -> Self {
        Self {
            key,
            search: String::new(),
            selected: 0,
            entries: history.entries.clone(),
        }
    }
    /// Search this tab.
    pub fn filter(&mut self, history: &History) {
        self.entries = history
            .search(&self.search, None, false)
            .into_iter()
            .cloned()
            .collect();
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
    }
    /// Reopen the selected query.
    pub fn selected_query(&self) -> Option<String> {
        self.entries
            .get(self.selected)
            .map(|entry| entry.sql.clone())
    }
}

/// Product tab union.
#[derive(Debug, Clone)]
pub enum Tab {
    /// Table data or structure tab.
    Table(TableTab),
    /// SQL query tab.
    Query(QueryTab),
    /// Query history tab.
    History(HistoryTab),
}

impl Tab {
    /// Stable identity used by keyed public UI collections.
    #[must_use]
    pub const fn key(&self) -> TabKey {
        match self {
            Self::Table(tab) => tab.key,
            Self::Query(tab) => tab.key,
            Self::History(tab) => tab.key,
        }
    }

    /// Current grid and its stable control identity.
    pub fn grid(&self) -> Option<(Id, &GridView)> {
        match self {
            Self::Table(tab) if tab.is_structure() => {
                Some((tab.key.control("structure"), &tab.structure))
            }
            Self::Table(tab) => Some((tab.key.control("data"), &tab.result)),
            Self::Query(tab) => tab
                .result
                .as_ref()
                .map(|grid| (tab.key.control("results"), grid)),
            Self::History(_) => None,
        }
    }
    /// Current grid mutably, with no mirrored model or interaction state.
    pub fn grid_mut(&mut self) -> Option<(Id, &mut GridView)> {
        match self {
            Self::Table(tab) => {
                if tab.is_structure() {
                    Some((tab.key.control("structure"), &mut tab.structure))
                } else {
                    Some((tab.key.control("data"), &mut tab.result))
                }
            }
            Self::Query(tab) => tab
                .result
                .as_mut()
                .map(|grid| (tab.key.control("results"), grid)),
            Self::History(_) => None,
        }
    }

    /// Display label.
    pub fn label(&self) -> String {
        match self {
            Self::Table(tab) => tab.table.name.clone(),
            Self::Query(tab) => tab.name.clone(),
            Self::History(_) => "History".to_owned(),
        }
    }
    /// Whether this tab owns pending changes.
    pub fn dirty(&self) -> bool {
        matches!(self, Self::Table(tab) if tab.result.pending_total() > 0)
            || matches!(self, Self::Query(tab) if tab.dirty() || tab.result.as_ref().is_some_and(|grid| grid.pending_total() > 0))
    }
}

/// One explorer row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    /// Schema name.
    pub schema: String,
    /// Object name.
    pub name: String,
    /// Catalog object kind.
    pub(crate) kind: crate::db::ObjectKind,
    /// Estimated row count.
    pub rows: usize,
}

/// Build explorer rows from a catalog.
pub(crate) fn explorer_items(catalog: &Catalog) -> Vec<ExplorerItem> {
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
