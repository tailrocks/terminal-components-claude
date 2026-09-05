//! `TablePro` workbench state: explorer, tabs, history and query routing.

use crate::db::{Catalog, Connection, ObjectKind};
use crate::filter_editor::Filter;
use crate::model::{History, HistoryEntry, HistorySource, SwitcherIndex};
use crate::tabs::{self, ExplorerItem, HistoryTab, QueryTab, Tab, TableTab};

/// Workbench state for one active connection.
#[derive(Debug, Clone)]
pub struct Workbench {
    /// Active connection.
    pub connection: Connection,
    /// Database catalog.
    pub catalog: Catalog,
    /// Explorer rows.
    pub explorer: Vec<ExplorerItem>,
    /// Explorer filter text.
    pub explorer_filter: String,
    /// Selected explorer row.
    pub explorer_selected: usize,
    /// Open tabs.
    pub tabs: Vec<Tab>,
    /// Active tab index.
    pub active: usize,
    /// Next query number.
    pub query_counter: usize,
    /// Query history.
    pub history: History,
    /// Whether the active tab is maximised.
    pub maximized: bool,
}

impl Workbench {
    /// Build a workbench for a connection.
    pub fn new(connection: Connection, catalog: Catalog) -> Self {
        Self {
            explorer: tabs::explorer_items(&catalog),
            connection,
            catalog,
            explorer_filter: String::new(),
            explorer_selected: 0,
            tabs: Vec::new(),
            active: 0,
            query_counter: 0,
            history: History::seeded(),
            maximized: false,
        }
    }
    /// Visible explorer rows.
    pub fn visible_explorer(&self) -> Vec<(usize, &ExplorerItem)> {
        let query = self.explorer_filter.to_ascii_lowercase();
        self.explorer
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                query.is_empty()
                    || item.name.to_ascii_lowercase().contains(&query)
                    || item.schema.to_ascii_lowercase().contains(&query)
            })
            .collect()
    }
    /// Set explorer search text.
    pub fn filter_explorer(&mut self, query: impl Into<String>) {
        self.explorer_filter = query.into();
        self.explorer_selected = self
            .explorer_selected
            .min(self.visible_explorer().len().saturating_sub(1));
    }
    /// Open the named table.
    pub fn open_table(&mut self, name: &str) -> bool {
        let Some(table) = self.catalog.find(Some("public"), name).cloned() else {
            return false;
        };
        self.tabs
            .push(Tab::Table(TableTab::new(table, &self.catalog)));
        self.active = self.tabs.len().saturating_sub(1);
        true
    }
    /// Open the selected explorer table.
    pub fn open_selected(&mut self) -> bool {
        let name = self
            .visible_explorer()
            .get(self.explorer_selected)
            .map(|(_, item)| item.name.clone());
        name.is_some_and(|name| self.open_table(&name))
    }
    /// Open a new query tab.
    pub fn new_query(&mut self, query: impl Into<String>) -> usize {
        self.query_counter = self.query_counter.saturating_add(1);
        self.tabs
            .push(Tab::Query(QueryTab::new(self.query_counter, query)));
        self.active = self.tabs.len().saturating_sub(1);
        self.active
    }
    /// Open history.
    pub fn open_history(&mut self) -> usize {
        self.tabs.push(Tab::History(HistoryTab::new(&self.history)));
        self.active = self.tabs.len().saturating_sub(1);
        self.active
    }
    /// Close one tab.
    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(index);
        self.active = self.active.min(self.tabs.len().saturating_sub(1));
        true
    }
    /// Active tab.
    pub fn active(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }
    /// Active tab mutably.
    pub fn active_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }
    /// Active table tab.
    pub fn active_table(&self) -> Option<&TableTab> {
        match self.active()? {
            Tab::Table(tab) => Some(tab),
            _ => None,
        }
    }
    /// Active table tab mutably.
    pub fn active_table_mut(&mut self) -> Option<&mut TableTab> {
        match self.active_mut()? {
            Tab::Table(tab) => Some(tab),
            _ => None,
        }
    }
    /// Apply a filter to the active table.
    pub fn apply_filter(&mut self, filter: Filter) -> bool {
        self.active_table_mut()
            .map(|table| table.set_filter(filter))
            .is_some()
    }
    /// Toggle the active table's structure view.
    pub fn toggle_structure(&mut self) -> bool {
        self.active_table_mut()
            .map(TableTab::toggle_structure)
            .is_some()
    }
    /// Toggle maximisation.
    pub const fn toggle_maximized(&mut self) {
        self.maximized = !self.maximized;
    }
    /// Execute the active query and record successful/failed history.
    ///
    /// # Errors
    ///
    /// Returns an error when the active tab is not a query or its SQL cannot
    /// be executed by the deterministic catalog.
    pub fn execute_active(&mut self) -> Result<usize, String> {
        let (query, source) = match self.active() {
            Some(Tab::Query(tab)) => (tab.query.clone(), HistorySource::Editor),
            _ => return Err("Active tab is not a query".to_owned()),
        };
        let catalog = self.catalog.clone();
        let result = match self.active_mut() {
            Some(Tab::Query(tab)) => tab.execute(&catalog),
            _ => return Err("Active tab is not a query".to_owned()),
        };
        let (rows, error) = match &result {
            Ok(rows) => (Some(*rows), None),
            Err(error) => (None, Some(error.clone())),
        };
        self.history.push(HistoryEntry {
            id: 0,
            sql: query,
            connection: self.connection.name.clone(),
            database: self.connection.database.clone(),
            schema: "public".to_owned(),
            minutes_ago: 0,
            duration_ms: Some(1),
            rows,
            error,
            source,
        });
        result
    }
    /// Build the quick-switcher index.
    pub fn switcher(&self) -> SwitcherIndex {
        SwitcherIndex::from_catalog(
            &self.catalog,
            &self.history,
            std::slice::from_ref(&self.connection),
        )
    }
    /// Return tables only, useful to draw a structure/data explorer.
    pub fn table_count(&self) -> usize {
        self.explorer
            .iter()
            .filter(|item| item.kind == ObjectKind::Table)
            .count()
    }
}
