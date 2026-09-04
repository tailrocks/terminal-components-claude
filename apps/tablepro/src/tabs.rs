//! TablePro tab models.  Each tab owns product state; terminal components
//! only receive controlled values and generic grid adapters.

use crate::db::{Catalog, ColType, Table, Value};
use crate::domain::ResultGrid;
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry};
use crate::sql::{self, PlanNode};

/// Table tab body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Data,
    Structure,
}

/// A table tab with data and structure modes.
#[derive(Debug, Clone)]
pub struct TableTab {
    /// Catalog table.
    pub table: Table,
    /// Current mode.
    pub mode: TableMode,
    /// Data result adapter.
    pub result: ResultGrid,
    /// Active local filters.
    pub filters: Vec<Filter>,
    /// Last sort direction per column.
    pub sort: Option<(usize, junie_tui::SortDir)>,
}

impl TableTab {
    /// Load a bounded deterministic result for a table.
    pub fn new(table: Table, catalog: &Catalog) -> Self {
        let query = format!("SELECT * FROM {}.{}", table.schema, table.name);
        let result = sql::parse(&query)
            .ok()
            .and_then(|statement| match statement {
                sql::Statement::Select(select) => sql::run_select(catalog, &select).ok(),
                _ => None,
            })
            .map_or_else(ResultGrid::empty, |result| ResultGrid::from_result(&result));
        Self {
            table,
            mode: TableMode::Data,
            result,
            filters: Vec::new(),
            sort: None,
        }
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
        self.result.sort(
            junie_tui::ColumnKey::num((column as u16).saturating_add(1)),
            direction,
        );
        self.sort = Some((column, direction));
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
#[derive(Debug, Clone)]
pub struct QueryTab {
    /// Stable tab id.
    pub id: usize,
    /// Display name.
    pub name: String,
    /// SQL text.
    pub query: String,
    /// Last result, when successful.
    pub result: Option<ResultGrid>,
    /// Last execution error.
    pub error: Option<String>,
    /// Last explain plan.
    pub plan: Option<PlanNode>,
    /// Whether execution is in flight.
    pub running: bool,
}

impl QueryTab {
    /// New empty query tab.
    pub fn new(id: usize, query: impl Into<String>) -> Self {
        Self {
            id,
            name: format!("Query {id}"),
            query: query.into(),
            result: None,
            error: None,
            plan: None,
            running: false,
        }
    }
    /// Execute this query through the deterministic app-owned engine.
    pub fn execute(&mut self, catalog: &Catalog) -> Result<usize, String> {
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("The demo executor only runs SELECT statements".to_owned());
        };
        let result = sql::run_select(catalog, &select).map_err(|error| error.message)?;
        let rows = result.rows.len();
        self.result = Some(ResultGrid::from_result(&result));
        self.error = None;
        Ok(rows)
    }
    /// Build an explain plan for this query.
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
        self.query.is_empty() || self.result.is_none()
    }
}

/// History tab.
#[derive(Debug, Clone)]
pub struct HistoryTab {
    /// Search text.
    pub search: String,
    /// Selected entry index.
    pub selected: usize,
    /// Filtered entries.
    pub entries: Vec<HistoryEntry>,
}

impl HistoryTab {
    /// Build from history.
    pub fn new(history: &History) -> Self {
        Self {
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
    Table(TableTab),
    Query(QueryTab),
    History(HistoryTab),
}

impl Tab {
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
            || matches!(self, Self::Query(tab) if tab.dirty())
    }
}

/// One explorer row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub schema: String,
    pub name: String,
    pub kind: crate::db::ObjectKind,
    pub rows: usize,
}

/// Build explorer rows from a catalog.
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
