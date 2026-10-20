use crate::filter_editor::Filter;
use crate::tabs::{self, ExplorerItem, HistoryTab, QueryTab, Tab, TabKey, TabRecord, TableTab};
use tablepro_demo::db::{Catalog, Connection, ObjectKind};
use tablepro_domain::model::{History, HistoryEntry, HistorySource, SwitcherIndex};

#[derive(Debug)]
pub struct Workbench {
    owner: std::sync::Arc<()>,
    pub connection: Connection,
    pub catalog: Catalog,
    current_schema: String,
    schema_caption: String,
    pub explorer: Vec<ExplorerItem>,
    pub explorer_filter: String,
    pub explorer_selected: usize,
    tabs: Vec<TabRecord>,
    active: Option<TabKey>,
    pub query_counter: usize,
    next_tab_key: Option<u64>,
    pub history: History,
    pub maximized: bool,
}

impl Workbench {
    pub fn new(connection: Connection, catalog: Catalog) -> Self {
        Self {
            owner: std::sync::Arc::new(()),
            current_schema: "public".to_owned(),
            schema_caption: "public ".to_owned(),
            explorer: tabs::explorer_items(&catalog),
            connection,
            catalog,
            explorer_filter: String::new(),
            explorer_selected: 0,
            tabs: Vec::new(),
            active: None,
            query_counter: 0,
            next_tab_key: Some(1),
            history: History::seeded(),
            maximized: false,
        }
    }
    pub fn current_schema(&self) -> &str {
        &self.current_schema
    }

    pub(crate) fn schema_caption(&self) -> &str {
        &self.schema_caption
    }

    pub fn select_schema(&mut self, schema: &str) -> bool {
        if !self
            .catalog
            .schemas
            .iter()
            .any(|candidate| candidate == schema)
        {
            return false;
        }
        schema.clone_into(&mut self.current_schema);
        self.schema_caption = format!("{schema} ");
        self.explorer_filter.clear();
        self.explorer_selected = 0;
        true
    }

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
    pub fn filter_explorer(&mut self, query: impl Into<String>) {
        self.explorer_filter = query.into();
        self.explorer_selected = self
            .explorer_selected
            .min(self.visible_explorer().len().saturating_sub(1));
    }
    pub fn open_table(&mut self, name: &str) -> bool {
        self.open_table_in_schema("public", name)
    }

    pub fn open_table_in_schema(&mut self, schema: &str, name: &str) -> bool {
        if let Some(key) = self.tabs.iter().find_map(|record| match record.payload() {
            Tab::Table(tab) if tab.table.schema == schema && tab.table.name == name => {
                Some(record.key())
            }
            _ => None,
        }) {
            self.active = Some(key);
            return true;
        }
        let Some(table) = self.catalog.find(Some(schema), name).cloned() else {
            return false;
        };
        let payload = Tab::Table(TableTab::new(table, &self.catalog));
        self.insert_tab(self.tabs.len(), payload).is_some()
    }

    pub fn open_explorer_item(&mut self, item: &ExplorerItem) -> bool {
        self.open_table_in_schema(&item.schema, &item.name)
    }
    pub fn open_selected(&mut self) -> bool {
        let item = self
            .visible_explorer()
            .get(self.explorer_selected)
            .map(|(_, item)| (*item).clone());
        if let Some(item) = item {
            self.open_explorer_item(&item)
        } else {
            false
        }
    }
    pub(crate) fn can_insert_tab(&self) -> bool {
        self.next_tab_key.is_some()
    }

    pub fn insert_tab(&mut self, index: usize, payload: Tab) -> Option<TabKey> {
        if index > self.tabs.len() {
            return None;
        }
        let next = self.next_tab_key?;
        let key = TabKey::new(next);
        self.tabs.insert(index, TabRecord::new(key, payload));
        self.next_tab_key = next.checked_add(1);
        self.active = Some(key);
        Some(key)
    }

    pub fn tabs(&self) -> &[TabRecord] {
        &self.tabs
    }

    pub fn tab(&self, key: TabKey) -> Option<&Tab> {
        self.tabs
            .iter()
            .find(|record| record.key() == key)
            .map(TabRecord::payload)
    }

    pub fn tab_mut(&mut self, key: TabKey) -> Option<&mut Tab> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .map(TabRecord::payload_mut)
    }

    pub(crate) fn payloads_mut(&mut self) -> impl Iterator<Item = (TabKey, &mut Tab)> {
        self.tabs.iter_mut().map(|record| {
            let key = record.key();
            (key, record.lifecycle_payload_mut())
        })
    }

    pub(crate) fn owner_token(&self) -> std::sync::Weak<()> {
        std::sync::Arc::downgrade(&self.owner)
    }

    pub(crate) fn matches_owner(&self, token: &std::sync::Weak<()>) -> bool {
        std::sync::Weak::ptr_eq(token, &self.owner_token())
    }

    pub(crate) fn generation(&self, key: TabKey) -> Option<u64> {
        self.tabs
            .iter()
            .find(|record| record.key() == key)?
            .generation()
    }

    pub(crate) fn destructive_scope(&self) -> Option<Vec<(TabKey, u64)>> {
        self.tabs
            .iter()
            .map(|record| Some((record.key(), record.generation()?)))
            .collect()
    }

    pub(crate) fn matches_scope(&self, scope: &[(TabKey, u64)]) -> bool {
        self.tabs.len() == scope.len()
            && scope
                .iter()
                .all(|(key, generation)| self.generation(*key) == Some(*generation))
    }

    pub fn reorder_tabs(&mut self, keys: &[TabKey]) -> bool {
        if keys.len() != self.tabs.len() {
            return false;
        }
        let mut order = Vec::with_capacity(keys.len());
        for key in keys {
            let Some(index) = self.tabs.iter().position(|record| record.key() == *key) else {
                return false;
            };
            if order.contains(&index) {
                return false;
            }
            order.push(index);
        }
        let mut records: Vec<_> = core::mem::take(&mut self.tabs)
            .into_iter()
            .map(Some)
            .collect();
        self.tabs = order
            .into_iter()
            .filter_map(|index| records.get_mut(index).and_then(Option::take))
            .collect();
        true
    }

    pub fn new_query(&mut self, query: impl Into<String>) -> Option<TabKey> {
        let number = self.query_counter.checked_add(1)?;
        let key = self.insert_tab(self.tabs.len(), Tab::Query(QueryTab::new(number, query)))?;
        self.query_counter = number;
        Some(key)
    }
    pub fn open_history(&mut self) -> Option<TabKey> {
        self.insert_tab(
            self.tabs.len(),
            Tab::History(HistoryTab::new(&self.history)),
        )
    }
    pub fn close_tab(&mut self, index: usize) -> bool {
        let Some(tab) = self.tabs.get(index) else {
            return false;
        };
        if tab.dirty() {
            return false;
        }
        self.close_tab_confirmed(tab.key())
    }

    /// Close a tab chosen for deletion from the tab picker, discarding its
    /// unsaved query or result-grid edits as the reference picker does.
    pub fn close_tab_from_picker(&mut self, key: TabKey) -> bool {
        self.close_tab_confirmed(key)
    }

    pub(crate) fn close_tab_confirmed(&mut self, key: TabKey) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.key() == key) else {
            return false;
        };
        let removed = self.tabs.remove(index);
        if self.active == Some(removed.key()) {
            self.active = self
                .tabs
                .get(index.min(self.tabs.len().saturating_sub(1)))
                .map(TabRecord::key);
        }
        true
    }

    pub fn active_key(&self) -> Option<TabKey> {
        self.active_index()
            .and_then(|index| self.tabs.get(index))
            .map(TabRecord::key)
    }
    pub fn active_index(&self) -> Option<usize> {
        let key = self.active?;
        self.tabs.iter().position(|tab| tab.key() == key)
    }
    pub fn activate(&mut self, key: TabKey) -> bool {
        if !self.tabs.iter().any(|tab| tab.key() == key) {
            return false;
        }
        self.active = Some(key);
        true
    }
    pub fn reconnect(&mut self, connection: Connection, catalog: Catalog) -> bool {
        if self.has_unsaved_work() {
            return false;
        }
        self.reconnect_confirmed(connection, catalog);
        true
    }

    pub fn has_unsaved_work(&self) -> bool {
        self.tabs.iter().any(TabRecord::dirty)
    }

    pub(crate) fn reconnect_confirmed(&mut self, connection: Connection, catalog: Catalog) {
        let next_tab_key = self.next_tab_key;
        *self = Self::new(connection, catalog);
        self.next_tab_key = next_tab_key;
    }

    pub fn active_grid(&self) -> Option<(termrock::Id, &tabs::GridView)> {
        self.tabs.get(self.active_index()?)?.grid()
    }
    pub fn active_grid_mut(&mut self) -> Option<(termrock::Id, &mut tabs::GridView)> {
        let index = self.active_index()?;
        let record = self.tabs.get_mut(index)?;
        let key = record.key();
        record.payload_mut().grid_mut(key)
    }
    pub fn active(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index()?).map(TabRecord::payload)
    }
    pub fn active_mut(&mut self) -> Option<&mut Tab> {
        let index = self.active_index()?;
        self.tabs.get_mut(index).map(TabRecord::payload_mut)
    }
    pub fn active_table(&self) -> Option<&TableTab> {
        match self.active()? {
            Tab::Table(tab) => Some(tab),
            _ => None,
        }
    }
    pub fn active_table_mut(&mut self) -> Option<&mut TableTab> {
        match self.active_mut()? {
            Tab::Table(tab) => Some(tab),
            _ => None,
        }
    }
    pub fn apply_filter(&mut self, filter: Filter) -> bool {
        self.active_table_mut()
            .map(|table| table.set_filter(filter))
            .is_some()
    }
    pub fn toggle_structure(&mut self) -> bool {
        self.active_table_mut()
            .map(TableTab::toggle_structure)
            .is_some()
    }
    pub const fn toggle_maximized(&mut self) {
        self.maximized = !self.maximized;
    }
    pub fn execute_active(&mut self) -> Result<usize, String> {
        let (query, source) = match self.active() {
            Some(Tab::Query(tab)) if tab.has_pending_result() => {
                return Err("Pending result edits require confirmation".to_owned());
            }
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
    pub fn switcher(&self) -> SwitcherIndex {
        SwitcherIndex::from_workbench(&self.catalog, &self.history, &self.connection, &self.tabs)
    }
    pub fn table_count(&self) -> usize {
        self.explorer
            .iter()
            .filter(|item| item.kind == ObjectKind::Table)
            .count()
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn final_identity_is_allocated_once_and_exhaustion_is_atomic() {
        let mut workbench = crate::app::TableProApp::default().workbench;
        workbench.next_tab_key = Some(u64::MAX);
        assert!(
            workbench
                .insert_tab(usize::MAX, Tab::Query(QueryTab::new(9, "")))
                .is_none()
        );
        let Some(key) = workbench.new_query("") else {
            unreachable!("last identity");
        };
        assert_eq!(key.get(), u64::MAX);
        let count = workbench.tabs().len();
        let counter = workbench.query_counter;
        assert!(workbench.new_query("refused").is_none());
        assert_eq!(workbench.tabs().len(), count);
        assert_eq!(workbench.query_counter, counter);
        assert_eq!(workbench.active_key(), Some(key));
        let Some(index) = workbench.active_index() else {
            unreachable!("active");
        };
        assert!(workbench.close_tab(index));
        assert!(workbench.new_query("still refused").is_none());
    }
    #[test]
    fn exhausted_app_insert_and_reconnect_preserve_existing_state() {
        let mut app = crate::app::TableProApp::default();
        app.set_surface(crate::app::Surface::PendingChangeBar);
        app.workbench.next_tab_key = None;
        let key = app.workbench.active_key();
        let count = app.workbench.tabs().len();
        let surface = app.surface();
        assert!(!app.connect(0));
        assert!(matches!(
            app.run_query("SELECT 1"),
            crate::app::QueryOutcome::Rejected { .. }
        ));
        assert_eq!(app.workbench.active_key(), key);
        assert_eq!(app.workbench.tabs().len(), count);
        assert_eq!(app.surface(), surface);
        assert_eq!(app.result().pending_total(), 1);
    }
}
