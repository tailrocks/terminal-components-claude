use crate::filter_editor::Filter;
use crate::tabs::{
    self, ApplicationMutation, ExplorerItem, HistoryTab, QueryOutputId, QueryOutputInfo,
    QueryOutputInput, QueryOutputState, QueryRunFinish, QueryTab, Tab, TabKey, TabRecord, TableTab,
};
use std::time::Duration;
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

    pub(crate) fn application_update_payload<R>(
        &mut self,
        key: TabKey,
        update: impl FnOnce(&mut Tab) -> (R, ApplicationMutation),
    ) -> Option<R> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .application_update_payload(update)
    }

    pub fn query_output_state(&self, key: TabKey) -> Option<QueryOutputState> {
        self.tabs
            .iter()
            .find(|record| record.key() == key)?
            .query_output_state()
    }

    pub(crate) fn query_run_matches(&self, key: TabKey, run: &std::sync::Arc<()>) -> bool {
        self.tabs
            .iter()
            .find(|record| record.key() == key)
            .is_some_and(|record| record.query_run_matches(run))
    }

    pub(crate) fn can_advance_query_currency(&self, key: TabKey, increments: u64) -> bool {
        self.tabs
            .iter()
            .find(|record| record.key() == key)
            .is_some_and(|record| record.can_advance_query_currency(increments))
    }

    pub fn query_outputs(&self, key: TabKey) -> Option<Vec<QueryOutputInfo>> {
        self.tabs
            .iter()
            .find(|record| record.key() == key)?
            .query_outputs()
    }

    pub fn selected_query_output(&self, key: TabKey) -> Option<QueryOutputId> {
        self.tabs
            .iter()
            .find(|record| record.key() == key)?
            .selected_query_output()
    }

    pub fn query_has_pending_result(&self, key: TabKey) -> Option<bool> {
        Some(
            self.tabs
                .iter()
                .find(|record| record.key() == key)?
                .has_pending_query_result(),
        )
    }

    pub fn pending_query_result_total(&self, key: TabKey) -> Option<usize> {
        Some(
            self.tabs
                .iter()
                .find(|record| record.key() == key)?
                .pending_query_result_total(),
        )
    }

    pub fn prepare_query_outputs_for_run(&mut self, key: TabKey) -> bool {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .is_some_and(TabRecord::prepare_query_outputs_for_run)
    }

    pub fn append_query_outputs(
        &mut self,
        key: TabKey,
        outputs: Vec<QueryOutputInput>,
    ) -> Option<Vec<QueryOutputId>> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .append_query_outputs(outputs)
    }

    pub fn select_query_output(&mut self, key: TabKey, id: QueryOutputId) -> bool {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .is_some_and(|record| record.select_query_output(id))
    }

    pub fn close_query_output(&mut self, key: TabKey, id: QueryOutputId) -> bool {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .is_some_and(|record| record.close_query_output(id))
    }

    pub fn toggle_query_output_pin(&mut self, key: TabKey, id: QueryOutputId) -> bool {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .is_some_and(|record| record.toggle_query_output_pin(id))
    }

    pub(crate) fn application_update_query<R>(
        &mut self,
        key: TabKey,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .application_update_query(update)
    }

    pub(crate) fn begin_query_run<R>(
        &mut self,
        key: TabKey,
        run: &std::sync::Arc<()>,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .begin_query_run(run, update)
    }

    pub(crate) fn application_update_query_run<R>(
        &mut self,
        key: TabKey,
        run: &std::sync::Arc<()>,
        update: impl FnOnce(&mut QueryTab) -> R,
    ) -> Option<R> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .application_update_query_run(run, update)
    }

    pub(crate) fn finish_query_run(
        &mut self,
        key: TabKey,
        run: &std::sync::Arc<()>,
        status: &str,
        is_error: bool,
        duration_ms: u32,
    ) -> QueryRunFinish {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)
            .map_or(QueryRunFinish::NotOwned, |record| {
                record.finish_query_run(run, status, is_error, duration_ms)
            })
    }

    pub(crate) fn application_grid_mut(
        &mut self,
        key: TabKey,
    ) -> Option<(termrock::Id, &mut tabs::GridView)> {
        self.tabs
            .iter_mut()
            .find(|record| record.key() == key)?
            .application_grid_mut()
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
        if let Some(index) = self
            .tabs
            .iter()
            .position(|record| matches!(record.payload(), Tab::History(_)))
        {
            let key = {
                let record = self.tabs.get_mut(index)?;
                let key = record.key();
                let Tab::History(history) = record.payload_mut() else {
                    return None;
                };
                history.filter(&self.history);
                key
            };
            self.active = Some(key);
            return Some(key);
        }
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
    pub(crate) fn application_active_grid_mut(
        &mut self,
    ) -> Option<(termrock::Id, &mut tabs::GridView)> {
        let index = self.active_index()?;
        self.tabs.get_mut(index)?.application_grid_mut()
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
        self.execute_active_inner(None)
    }

    pub fn execute_active_with_duration(&mut self, elapsed: Duration) -> Result<usize, String> {
        self.execute_active_inner(Some(elapsed))
    }

    fn execute_active_inner(&mut self, elapsed: Option<Duration>) -> Result<usize, String> {
        let (query, source) = match self.active() {
            Some(Tab::Query(tab)) if tab.has_pending_result() => {
                return Err("Pending result edits require confirmation".to_owned());
            }
            Some(Tab::Query(tab)) => (tab.query.clone(), HistorySource::Editor),
            _ => return Err("Active tab is not a query".to_owned()),
        };
        let catalog = self.catalog.clone();
        let result = match self.active_mut() {
            Some(Tab::Query(tab)) => match elapsed {
                Some(elapsed) => tab.execute_with_duration(&catalog, elapsed),
                None => tab.execute(&catalog),
            },
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
            duration_ms: Some(elapsed.map_or(1, |duration| {
                u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
            })),
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
