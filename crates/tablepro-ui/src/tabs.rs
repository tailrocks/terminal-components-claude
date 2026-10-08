//! `TablePro` tab models. Each tab owns product state; terminal components
//! only receive controlled values and generic grid adapters.

use tablepro_demo::{PlanNode, ROW_CAP, explain, run_select};
use tablepro_domain::{Catalog, ColType, ObjectKind, ResultSet, Table, Value};
use tablepro_domain::{History, HistoryEntry};
use tablepro_sql as sql;
use termrock::{
    Align, CodeEditorState, Column, ColumnKey, GRID_MAX_COLUMNS, GlyphRole, Grid, GridModel,
    GridState, Id, ItemKey, ScrollState, SortDir, TextInputState, WidthSample,
};

use crate::domain::ResultGrid;
use crate::filter_editor::Filter;

pub fn column_specs<'a>(
    columns: &'a [(String, ColType)],
    editable: bool,
    table: Option<&Table>,
    filters: &[Filter],
) -> ([Column<'a>; GRID_MAX_COLUMNS], usize) {
    let count = columns.len().min(GRID_MAX_COLUMNS);
    let mut specs = [Column::new(ColumnKey::num(0), ""); GRID_MAX_COLUMNS];
    for (index, (name, ty)) in columns.iter().take(count).enumerate() {
        let mut col = Column::new(
            ColumnKey::num((index as u16).saturating_add(1)),
            name.as_str(),
        );
        col.sortable = true;
        col.editable = editable;
        col.sticky = false;
        if filters
            .iter()
            .any(|f| f.enabled && f.column == name.as_str())
        {
            col.filtered = true;
        }
        match ty {
            ColType::Uuid => {
                col.min_width = 9;
                col.max_width = 36;
            }
            ColType::Text => {
                col.min_width = 6;
                col.max_width = 40;
            }
            ColType::Int | ColType::Numeric => {
                col.min_width = 4;
                col.max_width = 22;
                col.align = Align::Right;
            }
            ColType::Bool => {
                col.min_width = 5;
                col.max_width = 5;
            }
            ColType::Timestamp | ColType::Date => {
                col.min_width = 10;
                col.max_width = 29;
            }
            ColType::Json => {
                col.min_width = 8;
                col.max_width = 40;
            }
            ColType::Enum => {
                col.min_width = 6;
                col.max_width = 16;
            }
        }
        if table.is_some_and(|t| t.column(name).is_some_and(|c| c.primary)) {
            col.prefix_glyph = Some(GlyphRole::PrimaryKey);
        }
        if let Some(slot) = specs.get_mut(index) {
            *slot = col;
        }
    }
    (specs, count)
}

/// Stable identity for an open workbench tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabKey(u64);

impl TabKey {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn control(self, name: &'static str) -> Id {
        Id::root("tablepro.tab")
            .item(ItemKey::num(self.0))
            .sub(name)
    }
}

/// One tab's grid, with borrowed column props independent of mutable row data.
#[derive(Clone)]
pub struct GridView {
    pub columns: Vec<(String, ColType)>,
    pub model: ResultGrid,
    pub state: GridState,
}

impl GridView {
    pub fn from_result(result: &ResultSet) -> Self {
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

    pub fn empty() -> Self {
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

/// Table tab body mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Data,
    Structure,
}

/// A table tab with data and structure modes.
#[derive(Debug, Clone)]
pub struct TableTab {
    pub table: Table,
    pub mode: TableMode,
    pub result: GridView,
    pub structure: Box<GridView>,
    pub filters: Vec<Filter>,
    pub match_all: bool,
}

impl TableTab {
    pub fn new(table: Table, catalog: &Catalog) -> Self {
        let query = format!("SELECT * FROM {}.{}", table.schema, table.name);
        let result = sql::parse(&query)
            .ok()
            .and_then(|statement| match statement {
                sql::Statement::Select(select) => run_select(catalog, &select).ok(),
                _ => None,
            })
            .map_or_else(GridView::empty, |result| GridView::from_result(&result));
        let mut tab = Self {
            table,
            mode: TableMode::Data,
            result,
            structure: Box::new(GridView::empty()),
            filters: Vec::new(),
            match_all: true,
        };
        tab.structure = Box::new(GridView::from_result(&ResultSet {
            columns: tab.structure_columns(),
            rows: tab.structure(),
            total: tab.table.columns.len(),
            source: None,
            duration_ms: 0,
            editable: false,
        }));
        let (columns, count) = column_specs(
            &tab.result.columns,
            tab.result.model.is_editable(),
            Some(&tab.table),
            &tab.filters,
        );
        let grid = Grid::new(
            Id::root("tablepro").sub("grid"),
            columns.get(..count).unwrap_or(&[]),
        );
        if let Ok(policy) = WidthSample::new(200, 95) {
            grid.sample_column_widths(&mut tab.result.state, &tab.result.model, policy);
        }
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

    pub fn sort(&mut self, column: usize, direction: SortDir) {
        self.result
            .model
            .sort(ColumnKey::num((column as u16).saturating_add(1)), direction);
    }

    pub fn reload_sorted(
        &mut self,
        catalog: &Catalog,
        col_idx: usize,
        next_sort: Option<(ColumnKey, SortDir)>,
    ) -> String {
        self.result.state.set_sort(next_sort);
        let col_name = self
            .result
            .columns
            .get(col_idx)
            .map(|(name, _)| name.clone());
        let order = match next_sort {
            Some((_, dir)) => col_name
                .as_ref()
                .map(|name| (name.clone(), dir == SortDir::Asc)),
            None => None,
        };
        let mut predicates = Vec::new();
        for f in self.filters.iter().filter(|f| f.enabled) {
            predicates.extend(f.predicates());
        }
        let sel = sql::Select {
            columns: vec!["*".into()],
            schema: Some(self.table.schema.clone()),
            table: self.table.name.clone(),
            predicates,
            order,
            limit: Some(ROW_CAP),
            count_only: false,
        };
        let has_filter = self.filters.iter().any(|f| f.enabled);
        if let Ok(rs) = run_select(catalog, &sel) {
            let saved_state = self.result.state.clone();
            self.result = GridView::from_result(&rs);
            self.result.model.set_estimated(has_filter);
            self.result.state = saved_state;
        }

        match next_sort {
            Some((_, dir)) => {
                let name = col_name.unwrap_or_default();
                let dir_str = if dir == SortDir::Asc {
                    "ascending"
                } else {
                    "descending"
                };
                format!("Sorted by {name} {dir_str}")
            }
            None => "Sort cleared".to_string(),
        }
    }

    pub fn reload(&mut self, catalog: &Catalog) {
        let order = match self.result.state.sort() {
            Some((col_key, dir)) => {
                let col_idx = usize::from(col_key.raw()).saturating_sub(1);
                self.result
                    .columns
                    .get(col_idx)
                    .map(|(name, _)| (name.clone(), dir == SortDir::Asc))
            }
            None => None,
        };
        let mut predicates = Vec::new();
        for f in self.filters.iter().filter(|f| f.enabled) {
            predicates.extend(f.predicates());
        }
        let sel = sql::Select {
            columns: vec!["*".into()],
            schema: Some(self.table.schema.clone()),
            table: self.table.name.clone(),
            predicates,
            order,
            limit: Some(ROW_CAP),
            count_only: false,
        };
        let has_filter = self.filters.iter().any(|f| f.enabled);
        if let Ok(rs) = run_select(catalog, &sel) {
            let saved_state = self.result.state.clone();
            self.result = GridView::from_result(&rs);
            self.result.model.set_estimated(has_filter);
            self.result.state = saved_state;
        }
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

/// Which pane of the query tab is maximized, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryPaneMaximized {
    #[default]
    None,
    Editor,
    Results,
}

/// Query editor tab.
#[derive(Clone)]
pub struct QueryTab {
    pub id: usize,
    pub name: String,
    pub query: String,
    pub editor_state: TextInputState,
    pub saved_text: String,
    pub result: Option<GridView>,
    pub error: Option<String>,
    pub plan: Option<PlanNode>,
    pub running: bool,
    pub editing: bool,
    pub last_duration: Option<u32>,
    pub affected: Option<(usize, String)>,
    pub maximized: QueryPaneMaximized,
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
            .field("editing", &self.editing)
            .field("maximized", &self.maximized)
            .finish()
    }
}

impl QueryTab {
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
            editing: false,
            last_duration: None,
            affected: None,
            maximized: QueryPaneMaximized::None,
        }
    }

    pub fn execute(&mut self, catalog: &Catalog) -> Result<usize, String> {
        if self.has_pending_result() {
            return Err("Pending result edits require confirmation".to_owned());
        }
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("The demo executor only runs SELECT statements".to_owned());
        };
        let result = run_select(catalog, &select).map_err(|error| error.message)?;
        let rows = result.rows.len();
        let mut grid_view = GridView::from_result(&result);
        let (columns, count) =
            column_specs(&grid_view.columns, grid_view.model.is_editable(), None, &[]);
        let grid = Grid::new(
            Id::root("tablepro").sub("grid"),
            columns.get(..count).unwrap_or(&[]),
        );
        if let Ok(policy) = WidthSample::new(200, 95) {
            grid.sample_column_widths(&mut grid_view.state, &grid_view.model, policy);
        }
        self.result = Some(grid_view);
        self.error = None;
        Ok(rows)
    }

    pub fn explain(&mut self, catalog: &Catalog) -> Result<(), String> {
        let statement = sql::parse(self.query.trim()).map_err(|error| error.message)?;
        let sql::Statement::Select(select) = statement else {
            return Err("Explain accepts SELECT statements".to_owned());
        };
        self.plan = Some(explain(catalog, &select, false).map_err(|error| error.message)?);
        Ok(())
    }

    pub fn has_pending_result(&self) -> bool {
        self.result
            .as_ref()
            .is_some_and(|grid| grid.pending_total() > 0)
    }

    pub fn dirty(&self) -> bool {
        self.query != self.saved_text
    }
}

/// History tab.
#[derive(Debug, Clone)]
pub struct HistoryTab {
    pub search: String,
    pub search_state: TextInputState,
    pub selected: usize,
    pub scope_all: bool,
    pub failed_only: bool,
    pub entries: Vec<HistoryEntry>,
    pub code_state: CodeEditorState,
    pub scroll_state: ScrollState,
}

impl HistoryTab {
    pub fn new(history: &History) -> Self {
        let mut tab = Self {
            search: String::new(),
            search_state: TextInputState::default(),
            selected: 0,
            scope_all: false,
            failed_only: false,
            entries: Vec::new(),
            code_state: CodeEditorState::default(),
            scroll_state: ScrollState::default(),
        };
        tab.refresh(history, "Production");
        tab
    }

    pub fn refresh(&mut self, history: &History, connection: &str) {
        let conn_opt = if self.scope_all {
            None
        } else {
            Some(connection)
        };
        self.entries = history
            .search(&self.search, conn_opt, self.failed_only)
            .into_iter()
            .cloned()
            .collect();
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.scroll_state.ensure_visible(self.selected);
        self.sync_detail();
    }

    pub fn sync_detail(&mut self) {
        let text = self.current_entry().map(|e| e.sql.as_str()).unwrap_or("");
        if self.code_state.text() != text {
            self.code_state = CodeEditorState::new(text);
        }
    }

    pub fn current_entry(&self) -> Option<&HistoryEntry> {
        self.entries.get(self.selected)
    }

    pub fn filter(&mut self, history: &History) {
        self.refresh(history, "Production");
    }

    pub fn selected_query(&self) -> Option<String> {
        self.entries
            .get(self.selected)
            .map(|entry| entry.sql.clone())
    }
}

/// Product tab union.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
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

/// An owned tab identity, separate from its freely replaceable payload.
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

/// One explorer row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerItem {
    pub schema: String,
    pub name: String,
    pub kind: ObjectKind,
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
