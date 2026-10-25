use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Weak;

use termrock::{
    Button, Cx, Diagnostic, FrameRead, Intent, ItemKey, LayerId, List, ListAction, ListState, Rect,
    Response, RowUi, SelectMode, StateFlags, Ui, UpdateCause,
};

use tablepro_domain::model::HistoryEntry;

use crate::tabs::{Tab, TabKey};
use crate::workbench::Workbench;

use super::HISTORY_LIST;
use super::history_controls::{ConnectionScope, HistoryControls};

const OPEN_CONTROL: &str = "history.open";
const COPY_CONTROL: &str = "history.copy";
const COPY_BUTTON_WIDTH: u16 = 6;

pub(super) enum HistoryAction {
    Open(String),
    Copy(String),
}

#[derive(Default)]
pub(super) struct HistoryRows {
    state: RefCell<RowsState>,
}

#[derive(Default)]
struct RowsState {
    tabs: HashMap<TabKey, TabRows>,
    displayed: Option<DisplayedRows>,
    last_active: Option<(TabKey, Weak<()>)>,
}

struct TabRows {
    owner: Weak<()>,
    list: ListState,
    initialized: bool,
}

struct DisplayedRows {
    tab: TabKey,
    owner: Weak<()>,
    visible: Vec<HistoryEntry>,
    cursor: Option<ItemKey>,
    open_entry: Option<HistoryEntry>,
    open_visible: bool,
    copy_entry: Option<HistoryEntry>,
    copy_visible: bool,
}

impl TabRows {
    fn new(owner: Weak<()>) -> Self {
        Self {
            owner,
            list: ListState::default(),
            initialized: false,
        }
    }
}

impl RowsState {
    fn tab_mut(&mut self, key: TabKey, owner: &Weak<()>) -> &mut TabRows {
        let rows = self
            .tabs
            .entry(key)
            .or_insert_with(|| TabRows::new(owner.clone()));
        if !Weak::ptr_eq(&rows.owner, owner) {
            *rows = TabRows::new(owner.clone());
            self.displayed = None;
        }
        rows
    }

    fn displayed_for(&self, key: TabKey, owner: &Weak<()>) -> Option<&DisplayedRows> {
        self.displayed
            .as_ref()
            .filter(|displayed| displayed.tab == key && Weak::ptr_eq(&displayed.owner, owner))
    }
}

pub(super) fn eligible_entries<'a>(
    workbench: &'a Workbench,
    key: TabKey,
    controls: &HistoryControls,
) -> Option<Vec<&'a HistoryEntry>> {
    let Tab::History(history) = workbench.tab(key)? else {
        return None;
    };
    let connection = match controls.scope() {
        ConnectionScope::All => None,
        ConnectionScope::Current => Some(workbench.connection.name.as_str()),
    };
    Some(
        workbench
            .history
            .search(&history.search, connection, controls.failed_only()),
    )
}

fn item_key(entry: &HistoryEntry) -> ItemKey {
    ItemKey::num(u64::try_from(entry.id).unwrap_or(u64::MAX))
}

fn row_key(entry: &&HistoryEntry) -> ItemKey {
    item_key(entry)
}

fn render_row(entry: &&HistoryEntry, row: &mut RowUi<'_>) {
    let first_line = entry.sql.lines().next().unwrap_or_default();
    if entry.error.is_some() {
        row.label_fmt(format_args!("!{first_line}"));
    } else {
        row.label(first_line);
    }
    let meta = format!("{} · {}", entry.when(), entry.duration());
    row.meta(&meta);
}

fn duplicate_keys(entries: &[&HistoryEntry]) -> HashSet<ItemKey> {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();
    for entry in entries {
        let key = item_key(entry);
        if !seen.insert(key) {
            duplicates.insert(key);
        }
    }
    duplicates
}

fn unique_index(entries: &[&HistoryEntry], key: ItemKey) -> Option<usize> {
    let mut found = None;
    for (index, entry) in entries.iter().enumerate() {
        if item_key(entry) == key {
            if found.is_some() {
                return None;
            }
            found = Some(index);
        }
    }
    found
}

fn unique_entry<'a>(entries: &'a [&HistoryEntry], key: ItemKey) -> Option<&'a HistoryEntry> {
    entries.get(unique_index(entries, key)?).copied()
}

fn prepare_cursor(
    rows: &mut TabRows,
    entries: &[&HistoryEntry],
    fallback_index: usize,
    duplicates: &HashSet<ItemKey>,
) {
    let previous = rows.initialized.then(|| rows.list.cursor()).flatten();
    let previous_index = previous.and_then(|key| unique_index(entries, key));
    let requested = fallback_index.min(entries.len().saturating_sub(1));
    let requested_index = entries
        .get(requested)
        .filter(|entry| !duplicates.contains(&item_key(entry)))
        .map(|_| requested);
    let index = previous_index.or(requested_index).or_else(|| {
        entries
            .iter()
            .position(|entry| !duplicates.contains(&item_key(entry)))
    });
    let cursor = index.and_then(|index| entries.get(index).map(|entry| (index, item_key(entry))));
    if let Some((index, key)) = cursor {
        rows.list.set_cursor(index, key);
    } else if entries.is_empty() || !rows.initialized {
        rows.list = ListState::default();
    }
    rows.list.choose(cursor.map(|(_, key)| key));
    rows.initialized = true;
}

fn open_area(inner: Rect, list_y: u16) -> Rect {
    Rect::new(inner.x.saturating_add(44), list_y.saturating_add(10), 17, 1)
}

fn copy_area(inner: Rect, list_y: u16) -> Option<Rect> {
    let x = inner.x.saturating_add(44);
    let right = inner.x.saturating_add(inner.width);
    let width = right.saturating_sub(x);
    if width < COPY_BUTTON_WIDTH {
        return None;
    }

    let preferred_y = list_y.saturating_add(11);
    let compact_y = list_y.saturating_add(5);
    // At the 72x20 minimum, the old Copy row is below the framed body.
    // The compact slot is between the two-line SQL preview and the facts.
    let y = if preferred_y >= inner.y && preferred_y < inner.bottom() {
        preferred_y
    } else if compact_y >= inner.y && compact_y < inner.bottom() {
        compact_y
    } else {
        return None;
    };

    Some(Rect::new(x, y, COPY_BUTTON_WIDTH, 1))
}

impl HistoryRows {
    pub(super) fn update(
        &self,
        cx: &mut Cx<'_>,
        workbench: &mut Workbench,
        controls_by_tab: &HashMap<TabKey, HistoryControls>,
        history_route_visible: bool,
    ) -> (Response<()>, Option<HistoryAction>) {
        let Some(key) = workbench.active_key() else {
            return (self.service_hidden(cx, workbench, controls_by_tab), None);
        };
        let is_history = matches!(workbench.tab(key), Some(Tab::History(_)));
        if !is_history {
            return (self.service_hidden(cx, workbench, controls_by_tab), None);
        }
        let Some(history) = (match workbench.tab(key) {
            Some(Tab::History(history)) => Some(history),
            _ => None,
        }) else {
            return (Response::ignored(), None);
        };
        let fallback = HistoryControls::default();
        let controls = controls_by_tab.get(&key).unwrap_or(&fallback);
        let Some(entries) = eligible_entries(workbench, key, controls) else {
            return (Response::ignored(), None);
        };
        let entries_unchanged = history.entries.len() == entries.len()
            && history
                .entries
                .iter()
                .zip(&entries)
                .all(|(stored, current)| stored == *current);
        let refreshed_entries = (!entries_unchanged).then(|| {
            entries
                .iter()
                .map(|entry| (**entry).clone())
                .collect::<Vec<_>>()
        });
        let owner = workbench.owner_token();
        let route_active = history_route_visible
            && cx.top_layer() == LayerId::PAGE
            && workbench.active_key() == Some(key);
        if !route_active {
            return (self.service_hidden(cx, workbench, controls_by_tab), None);
        }

        let mut response = Response::ignored();
        let mut accepted = None;
        let selected_fallback = history.selected;
        let duplicate = duplicate_keys(&entries);
        let duplicate_item = |entry: &&HistoryEntry| duplicate.contains(&item_key(entry));
        let list = List::<&HistoryEntry>::new(HISTORY_LIST)
            .key(row_key)
            .row(render_row)
            .disabled_item(&duplicate_item)
            .select_mode(SelectMode::Single);

        let mut store = self.state.borrow_mut();
        let (action, selected_index) = {
            let rows = store.tab_mut(key, &owner);
            prepare_cursor(rows, &entries, selected_fallback, &duplicate);
            let list_response = list.update(cx, &mut rows.list, &entries);
            let action = list_response.action_ref().copied();
            response |= list_response.erase();
            let cursor = rows.list.cursor();
            let selected_index = cursor.and_then(|cursor| unique_index(&entries, cursor));
            rows.list.choose(
                selected_index.and_then(|index| entries.get(index).map(|entry| item_key(entry))),
            );
            (action, selected_index)
        };

        if let Some(ListAction::Activated(activated)) = action
            && let Some(displayed) = store.displayed_for(key, &owner)
        {
            accepted = activation_entry(displayed.cursor, activated, &displayed.visible, &entries)
                .map(|current| HistoryAction::Open(current.sql.clone()));
        }

        let button_id = key.control(OPEN_CONTROL);
        let button_entry = store
            .displayed_for(key, &owner)
            .filter(|displayed| displayed.open_visible)
            .and_then(|displayed| {
                displayed
                    .open_entry
                    .as_ref()
                    .filter(|entry| displayed.cursor == Some(item_key(entry)))
            })
            .and_then(|captured| {
                unique_entry(&entries, item_key(captured)).filter(|current| *current == captured)
            });
        let button = Button::new(button_id, "Open in new tab").disabled(button_entry.is_none());
        let button_response = button.update(cx);
        if button_response.activated()
            && let Some(current) = button_entry
            && accepted.is_none()
        {
            accepted = Some(HistoryAction::Open(current.sql.clone()));
        }
        response |= button_response.erase();

        let copy_id = key.control(COPY_CONTROL);
        let copy_entry = store
            .displayed_for(key, &owner)
            .and_then(|displayed| current_copy_entry(displayed, &entries));
        let copy_button = Button::new(copy_id, "Copy").disabled(copy_entry.is_none());
        let copy_response = copy_button.update(cx);
        if copy_response.activated()
            && let Some(current) = copy_entry
            && accepted.is_none()
        {
            accepted = Some(HistoryAction::Copy(current.sql.clone()));
        }
        response |= copy_response.erase();

        drop(entries);
        let mut history_changed = false;
        if let Some(Tab::History(history)) = workbench.tab_mut(key) {
            if let Some(entries) = refreshed_entries {
                history.entries = entries;
                history_changed = true;
            }
            let selected = selected_index.unwrap_or_else(|| {
                history
                    .selected
                    .min(history.entries.len().saturating_sub(1))
            });
            if history.selected != selected {
                history.selected = selected;
                history_changed = true;
            }
        }
        if history_changed {
            response |= Response::changed();
        }
        store.last_active = Some((key, owner.clone()));
        let (inactive_response, _) =
            service_inactive_buttons(cx, &mut store, workbench, Some(key), true, None);
        response |= inactive_response;
        (response, accepted)
    }

    fn service_hidden(
        &self,
        cx: &mut Cx<'_>,
        workbench: &Workbench,
        controls_by_tab: &HashMap<TabKey, HistoryControls>,
    ) -> Response<()> {
        let mut store = self.state.borrow_mut();
        store.displayed = None;
        let mut response = Response::ignored();
        let mut list_bucket_safe = true;
        let mut retired_last_list = None;
        if let Some((key, owner)) = store.last_active.clone() {
            let owner_matches = workbench.matches_owner(&owner);
            let live_history = owner_matches && matches!(workbench.tab(key), Some(Tab::History(_)));
            let fallback = HistoryControls::default();
            let controls = controls_by_tab.get(&key).unwrap_or(&fallback);
            let entries = if live_history {
                eligible_entries(workbench, key, controls).unwrap_or_default()
            } else {
                Vec::new()
            };
            let duplicate = duplicate_keys(&entries);
            let duplicate_item = |entry: &&HistoryEntry| duplicate.contains(&item_key(entry));
            let list = List::<&HistoryEntry>::new(HISTORY_LIST)
                .key(row_key)
                .row(render_row)
                .disabled_item(&duplicate_item)
                .select_mode(SelectMode::Single);
            let mut empty_state = ListState::default();
            {
                let list_state = store
                    .tabs
                    .get_mut(&key)
                    .map_or(&mut empty_state, |rows| &mut rows.list);
                match lifecycle_bucket(cx, HISTORY_LIST) {
                    LifecycleBucket::Empty if cx.update_cause() == UpdateCause::Settle => {
                        match list.update_focus_out(cx, list_state) {
                            Some(settled) => response |= settled.erase(),
                            None => response |= list.update(cx, list_state, &entries).erase(),
                        }
                    }
                    LifecycleBucket::Empty => {}
                    LifecycleBucket::Lifecycle if owner_matches => {
                        response |= list.update(cx, list_state, &entries).erase();
                    }
                    LifecycleBucket::Lifecycle => match list.update_focus_out(cx, list_state) {
                        Some(settled) => response |= settled.erase(),
                        None => response |= list.update(cx, list_state, &entries).erase(),
                    },
                    LifecycleBucket::Other => {
                        diagnose(cx, HISTORY_LIST);
                        list_bucket_safe = false;
                    }
                }
            }
            let list_owner_retired = !live_history
                && cx.update_cause() == UpdateCause::Settle
                && list_bucket_safe
                && retired(cx, HISTORY_LIST);
            if list_owner_retired {
                retired_last_list = Some((key, owner));
            }
        }
        let (button_response, retired_button_owners) = service_inactive_buttons(
            cx,
            &mut store,
            workbench,
            None,
            list_bucket_safe,
            retired_last_list.as_ref(),
        );
        response |= button_response;
        if retired_button_owners && let Some((key, owner)) = retired_last_list {
            if store
                .tabs
                .get(&key)
                .is_some_and(|rows| Weak::ptr_eq(&rows.owner, &owner))
            {
                store.tabs.remove(&key);
            }
            if store
                .last_active
                .as_ref()
                .is_some_and(|(last_key, last_owner)| {
                    *last_key == key && Weak::ptr_eq(last_owner, &owner)
                })
            {
                store.last_active = None;
            }
        }
        response
    }

    pub(super) fn draw_list(
        &self,
        ui: &mut Ui<'_>,
        key: TabKey,
        owner: &Weak<()>,
        area: Rect,
        entries: &[&HistoryEntry],
        selected_fallback: usize,
    ) -> Option<usize> {
        let visible = RefCell::new(Vec::new());
        let mut store = self.state.borrow_mut();
        let (selected, cursor) = {
            let duplicate = duplicate_keys(entries);
            let duplicate_item = |entry: &&HistoryEntry| duplicate.contains(&item_key(entry));
            let render_visible = |entry: &&HistoryEntry, row: &mut RowUi<'_>| {
                visible.borrow_mut().push((**entry).clone());
                render_row(entry, row);
            };
            let list = List::<&HistoryEntry>::new(HISTORY_LIST)
                .key(row_key)
                .row(render_visible)
                .disabled_item(&duplicate_item)
                .select_mode(SelectMode::Single);
            let rows = store.tab_mut(key, owner);
            prepare_cursor(rows, entries, selected_fallback, &duplicate);
            let _ = list.draw(ui, area, &rows.list, entries);
            let cursor = rows.list.cursor();
            let selected = cursor.and_then(|cursor| unique_index(entries, cursor));
            (selected, cursor)
        };
        let visible = visible.into_inner();
        store.last_active = Some((key, owner.clone()));
        store.displayed = if ui.is_inert() || area.intersection(ui.full()).is_empty() {
            None
        } else {
            Some(DisplayedRows {
                tab: key,
                owner: owner.clone(),
                visible,
                cursor,
                open_entry: None,
                open_visible: false,
                copy_entry: None,
                copy_visible: false,
            })
        };
        selected
    }

    pub(super) fn draw_open_button(
        &self,
        ui: &mut Ui<'_>,
        key_owner: (TabKey, &Weak<()>),
        inner: Rect,
        list_y: u16,
        entries: &[&HistoryEntry],
        selected: Option<usize>,
    ) {
        let (key, owner) = key_owner;
        let area = open_area(inner, list_y);
        if area.intersection(ui.full()).is_empty() {
            return;
        }
        let selected_entry = selected.and_then(|index| entries.get(index)).copied();
        let mut store = self.state.borrow_mut();
        let Some(displayed) = store.displayed_for(key, owner) else {
            return;
        };
        let captured = selected_entry
            .filter(|entry| {
                let item = item_key(entry);
                displayed
                    .visible
                    .iter()
                    .filter(|visible| item_key(visible) == item)
                    .count()
                    == 1
                    && displayed.visible.iter().any(|visible| visible == *entry)
            })
            .cloned();
        let button =
            Button::new(key.control(OPEN_CONTROL), "Open in new tab").disabled(captured.is_none());
        let drawn = button.draw(ui, area);
        if let Some(displayed) = store.displayed.as_mut()
            && displayed.tab == key
            && Weak::ptr_eq(&displayed.owner, owner)
        {
            displayed.open_visible = !ui.is_inert() && !drawn.intersection(ui.full()).is_empty();
            displayed.open_entry = displayed.open_visible.then_some(captured).flatten();
        }
    }

    pub(super) fn draw_copy_button(
        &self,
        ui: &mut Ui<'_>,
        key_owner: (TabKey, &Weak<()>),
        inner: Rect,
        list_y: u16,
        entries: &[&HistoryEntry],
        selected: Option<usize>,
    ) {
        let (key, owner) = key_owner;
        let mut store = self.state.borrow_mut();
        if let Some(displayed) = store.displayed.as_mut()
            && displayed.tab == key
            && Weak::ptr_eq(&displayed.owner, owner)
        {
            displayed.copy_visible = false;
            displayed.copy_entry = None;
        }
        let Some(displayed) = store.displayed_for(key, owner) else {
            return;
        };
        let Some(area) = copy_area(inner, list_y) else {
            return;
        };
        if ui.is_inert() || area.intersection(inner) != area || area.intersection(ui.full()) != area
        {
            return;
        }
        let selected_entry = selected.and_then(|index| entries.get(index)).copied();
        let captured = selected_entry
            .filter(|entry| {
                let item = item_key(entry);
                displayed.cursor == Some(item)
                    && displayed
                        .visible
                        .iter()
                        .filter(|visible| item_key(visible) == item)
                        .count()
                        == 1
                    && displayed.visible.iter().any(|visible| visible == *entry)
            })
            .cloned();
        let button = Button::new(key.control(COPY_CONTROL), "Copy").disabled(captured.is_none());
        let drawn = button.draw(ui, area);
        let fully_visible = drawn == area
            && drawn.width == COPY_BUTTON_WIDTH
            && drawn.height == 1
            && drawn.intersection(inner) == drawn
            && drawn.intersection(ui.full()) == drawn;
        if let Some(displayed) = store.displayed.as_mut()
            && displayed.tab == key
            && Weak::ptr_eq(&displayed.owner, owner)
        {
            displayed.copy_visible = fully_visible;
            displayed.copy_entry = fully_visible.then_some(captured).flatten();
        }
    }

    pub(super) fn selected_copy_sql(
        &self,
        workbench: &Workbench,
        controls_by_tab: &HashMap<TabKey, HistoryControls>,
    ) -> Option<String> {
        let key = workbench.active_key()?;
        if !matches!(workbench.tab(key), Some(Tab::History(_))) {
            return None;
        }
        let fallback = HistoryControls::default();
        let controls = controls_by_tab.get(&key).unwrap_or(&fallback);
        let entries = eligible_entries(workbench, key, controls)?;
        let owner = workbench.owner_token();
        let store = self.state.borrow();
        let displayed = store.displayed_for(key, &owner)?;
        current_copy_entry(displayed, &entries).map(|entry| entry.sql.clone())
    }

    pub(super) fn selected_detail_entry(
        &self,
        workbench: &Workbench,
        controls_by_tab: &HashMap<TabKey, HistoryControls>,
    ) -> Option<HistoryEntry> {
        let key = workbench.active_key()?;
        if !matches!(workbench.tab(key), Some(Tab::History(_))) {
            return None;
        }
        let fallback = HistoryControls::default();
        let controls = controls_by_tab.get(&key).unwrap_or(&fallback);
        let entries = eligible_entries(workbench, key, controls)?;
        let owner = workbench.owner_token();
        let store = self.state.borrow();
        let displayed = store.displayed_for(key, &owner)?;
        let cursor = displayed.cursor?;
        let captured = displayed_entry(&displayed.visible, cursor)?;
        let current = unique_entry(&entries, cursor)?;
        (current == captured).then(|| current.clone())
    }

    pub(super) fn invalidate_display(&self) {
        self.state.borrow_mut().displayed = None;
    }
}

fn service_inactive_buttons(
    cx: &mut Cx<'_>,
    store: &mut RowsState,
    workbench: &Workbench,
    active_key: Option<TabKey>,
    list_bucket_safe: bool,
    retire_list_owner: Option<&(TabKey, Weak<()>)>,
) -> (Response<()>, bool) {
    let mut candidates: Vec<_> = store
        .tabs
        .iter()
        .map(|(key, rows)| (*key, rows.owner.clone()))
        .collect();
    let current_owner = workbench.owner_token();
    for record in workbench.tabs() {
        let key = record.key();
        if matches!(record.payload(), Tab::History(_))
            && !candidates
                .iter()
                .any(|(candidate, owner)| *candidate == key && Weak::ptr_eq(owner, &current_owner))
        {
            candidates.push((key, current_owner.clone()));
        }
    }
    if let Some((key, owner)) = retire_list_owner
        && !candidates.iter().any(|(candidate, candidate_owner)| {
            *candidate == *key && Weak::ptr_eq(candidate_owner, owner)
        })
    {
        candidates.push((*key, owner.clone()));
    }

    let mut response = Response::ignored();
    let mut prune = Vec::new();
    let mut retired_button_owners = false;
    for (key, owner) in candidates {
        let owner_matches = workbench.matches_owner(&owner);
        let live_history = owner_matches && matches!(workbench.tab(key), Some(Tab::History(_)));
        if active_key == Some(key) && live_history {
            continue;
        }

        let retiring_owner = retire_list_owner.is_some_and(|(retire_key, retire_owner)| {
            *retire_key == key && Weak::ptr_eq(retire_owner, &owner)
        });
        let mut buttons_safe = true;
        let mut buttons_retired = true;
        for (control, label) in [(OPEN_CONTROL, "Open in new tab"), (COPY_CONTROL, "Copy")] {
            let owner_id = key.control(control);
            let bucket_safe = match lifecycle_bucket(cx, owner_id) {
                LifecycleBucket::Empty => true,
                LifecycleBucket::Lifecycle => {
                    response |= Button::new(owner_id, label)
                        .disabled(true)
                        .update(cx)
                        .erase();
                    true
                }
                LifecycleBucket::Other => {
                    diagnose(cx, owner_id);
                    false
                }
            };
            buttons_safe &= bucket_safe;
            buttons_retired &= bucket_safe && retired(cx, owner_id);
        }
        if retiring_owner {
            retired_button_owners = buttons_retired;
        }
        let last_active_owner = store
            .last_active
            .as_ref()
            .is_some_and(|(last_key, last_owner)| {
                *last_key == key && Weak::ptr_eq(last_owner, &owner)
            });
        if !live_history
            && cx.update_cause() == UpdateCause::Settle
            && buttons_safe
            && list_bucket_safe
            && !last_active_owner
        {
            let controls_retired = [OPEN_CONTROL, COPY_CONTROL]
                .into_iter()
                .all(|control| retired(cx, key.control(control)));
            if controls_retired {
                prune.push((key, owner));
            }
        }
    }

    for (key, owner) in prune {
        if store
            .tabs
            .get(&key)
            .is_some_and(|rows| Weak::ptr_eq(&rows.owner, &owner))
        {
            store.tabs.remove(&key);
        }
    }
    (response, retired_button_owners)
}

fn activation_entry<'a>(
    displayed_cursor: Option<ItemKey>,
    activated: ItemKey,
    displayed: &[HistoryEntry],
    current: &'a [&'a HistoryEntry],
) -> Option<&'a HistoryEntry> {
    if displayed_cursor != Some(activated) {
        return None;
    }
    let captured = displayed_entry(displayed, activated)?;
    let current = unique_entry(current, activated)?;
    (current == captured).then_some(current)
}

fn current_copy_entry<'a>(
    displayed: &DisplayedRows,
    current: &'a [&'a HistoryEntry],
) -> Option<&'a HistoryEntry> {
    if !displayed.copy_visible {
        return None;
    }
    let cursor = displayed.cursor?;
    let captured = displayed.copy_entry.as_ref()?;
    if item_key(captured) != cursor {
        return None;
    }
    let shown = displayed_entry(&displayed.visible, cursor)?;
    let current = unique_entry(current, cursor)?;
    (shown == captured && current == shown).then_some(current)
}

fn displayed_entry(displayed: &[HistoryEntry], key: ItemKey) -> Option<&HistoryEntry> {
    let mut found = None;
    for entry in displayed {
        if item_key(entry) == key {
            if found.is_some() {
                return None;
            }
            found = Some(entry);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{
        DisplayedRows, HistoryRows, RowsState, activation_entry, copy_area, current_copy_entry,
        item_key,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Weak;
    use tablepro_domain::model::{History, HistoryEntry, HistorySource};
    use termrock::{ItemKey, Rect};

    use crate::tabs::Tab;
    use crate::workbench::Workbench;

    fn entry(id: usize, sql: &str) -> HistoryEntry {
        HistoryEntry {
            id,
            sql: sql.to_owned(),
            connection: "Production".to_owned(),
            database: "warehouse".to_owned(),
            schema: "public".to_owned(),
            minutes_ago: 1,
            duration_ms: Some(7),
            rows: Some(2),
            error: None,
            source: HistorySource::Editor,
        }
    }

    fn displayed_copy(entries: Vec<HistoryEntry>, selected: HistoryEntry) -> DisplayedRows {
        DisplayedRows {
            tab: crate::tabs::TabKey::new(1),
            owner: Weak::new(),
            visible: entries,
            cursor: Some(item_key(&selected)),
            open_entry: None,
            open_visible: false,
            copy_entry: Some(selected),
            copy_visible: true,
        }
    }

    fn history_workbench(
        entries: &[HistoryEntry],
    ) -> Result<(Workbench, crate::tabs::TabKey), &'static str> {
        let connection = tablepro_demo::db::connections()
            .into_iter()
            .next()
            .ok_or("demo connection")?;
        let mut workbench = Workbench::new(connection, tablepro_demo::db::Catalog::acme_prod());
        workbench.history = History::default();
        workbench.history.entries = entries.to_vec();
        let key = workbench.open_history().ok_or("History tab")?;
        Ok((workbench, key))
    }

    fn displayed_detail(
        workbench: &Workbench,
        key: crate::tabs::TabKey,
        visible: Vec<HistoryEntry>,
        cursor: Option<ItemKey>,
    ) -> HistoryRows {
        let rows = HistoryRows::default();
        rows.state.borrow_mut().displayed = Some(DisplayedRows {
            tab: key,
            owner: workbench.owner_token(),
            visible,
            cursor,
            open_entry: None,
            open_visible: false,
            copy_entry: None,
            copy_visible: false,
        });
        rows
    }

    #[test]
    fn activation_rejects_same_key_with_changed_full_entry() {
        let displayed = [entry(7, "SELECT old\nFROM old_source")];
        let replacement = entry(7, "SELECT replacement\nFROM new_source");
        let current = [&replacement];

        assert!(
            activation_entry(Some(ItemKey::num(7)), ItemKey::num(7), &displayed, &current,)
                .is_none()
        );
    }

    #[test]
    fn activation_keeps_exact_entry_after_reordering() {
        let displayed = [entry(7, "SELECT first"), entry(8, "SELECT selected")];
        let selected = displayed[1].clone();
        let current = [&displayed[1], &displayed[0]];

        let activated =
            activation_entry(Some(ItemKey::num(8)), ItemKey::num(8), &displayed, &current);

        assert_eq!(activated, Some(&selected));
    }

    #[test]
    fn activation_rejects_duplicate_current_ids_and_wrong_displayed_cursor() {
        let displayed = [entry(7, "SELECT first")];
        let current_first = entry(7, "SELECT first");
        let current_second = entry(7, "SELECT conflicting");
        let duplicate_current = [&current_first, &current_second];

        assert!(
            activation_entry(
                Some(ItemKey::num(7)),
                ItemKey::num(7),
                &displayed,
                &duplicate_current,
            )
            .is_none()
        );
        assert!(
            activation_entry(
                Some(ItemKey::num(8)),
                ItemKey::num(7),
                &displayed,
                &[&current_first],
            )
            .is_none()
        );
    }

    #[test]
    fn activation_rejects_a_removed_displayed_entry() {
        let displayed = [entry(7, "SELECT displayed")];
        let replacement = entry(8, "SELECT current");

        assert!(
            activation_entry(
                Some(ItemKey::num(7)),
                ItemKey::num(7),
                &displayed,
                &[&replacement],
            )
            .is_none()
        );
    }

    #[test]
    fn copy_keeps_the_exact_full_entry_after_reordering() {
        let first = entry(7, "SELECT first");
        let selected = entry(8, "SELECT selected\nFROM full_source WHERE note = '尾部🙂'");
        let displayed = displayed_copy(vec![first.clone(), selected.clone()], selected.clone());
        let current = [&selected, &first];

        assert_eq!(current_copy_entry(&displayed, &current), Some(&selected));
    }

    #[test]
    fn detail_capture_requires_a_unique_current_full_equal_displayed_entry()
    -> Result<(), &'static str> {
        let first = entry(7, "SELECT first");
        let selected = entry(8, "SELECT selected\nFROM all_rows WHERE note = '尾部🙂'");
        let (mut workbench, key) = history_workbench(&[first.clone(), selected.clone()])?;
        let rows = displayed_detail(
            &workbench,
            key,
            vec![first.clone(), selected.clone()],
            Some(item_key(&selected)),
        );

        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            Some(selected.clone())
        );

        workbench.history.entries.swap(0, 1);
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            Some(selected.clone()),
            "reordering does not change the captured record"
        );
        workbench
            .history
            .entries
            .first_mut()
            .ok_or("selected History entry")?
            .database = "replacement_db".to_owned();
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "metadata replacement fails closed"
        );
        *workbench
            .history
            .entries
            .first_mut()
            .ok_or("selected History entry")? = selected.clone();
        workbench.history.entries.push(selected.clone());
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "duplicate cursor keys fail closed"
        );
        Ok(())
    }

    #[test]
    fn detail_capture_rejects_changed_sql_removed_filtered_and_wrong_cursor()
    -> Result<(), &'static str> {
        let selected = entry(7, "SELECT displayed\nFROM original");
        let (mut workbench, key) = history_workbench(std::slice::from_ref(&selected))?;
        let rows = displayed_detail(
            &workbench,
            key,
            vec![selected.clone()],
            Some(item_key(&selected)),
        );

        workbench
            .history
            .entries
            .first_mut()
            .ok_or("selected History entry")?
            .sql = "SELECT changed\nFROM replacement".to_owned();
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "same id with changed SQL fails closed"
        );
        *workbench
            .history
            .entries
            .first_mut()
            .ok_or("selected History entry")? = selected.clone();
        workbench.history.entries.clear();
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "removed entries fail closed"
        );
        workbench.history.entries.push(selected.clone());
        let Some(Tab::History(history)) = workbench.tab_mut(key) else {
            return Err("History remains active");
        };
        history.search = "not a match".to_owned();
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "filtered entries fail closed"
        );

        let (workbench, key) = history_workbench(std::slice::from_ref(&selected))?;
        let rows = displayed_detail(
            &workbench,
            key,
            vec![selected.clone()],
            Some(ItemKey::num(99)),
        );
        assert_eq!(
            rows.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "a cursor absent from the displayed rows fails closed"
        );
        Ok(())
    }

    #[test]
    fn detail_capture_rejects_duplicate_display_and_replaced_workbench_owner()
    -> Result<(), &'static str> {
        let selected = entry(7, "SELECT selected");
        let duplicate = entry(7, "SELECT duplicate");
        let (workbench, key) = history_workbench(std::slice::from_ref(&selected))?;
        let duplicate_display = displayed_detail(
            &workbench,
            key,
            vec![selected.clone(), duplicate],
            Some(item_key(&selected)),
        );
        assert_eq!(
            duplicate_display.selected_detail_entry(&workbench, &HashMap::new()),
            None,
            "duplicate visible keys fail closed"
        );

        let rows = displayed_detail(
            &workbench,
            key,
            vec![selected.clone()],
            Some(item_key(&selected)),
        );
        let (replacement, replacement_key) = history_workbench(std::slice::from_ref(&selected))?;
        assert_eq!(key, replacement_key, "fixture deliberately reuses the key");
        assert_eq!(
            rows.selected_detail_entry(&replacement, &HashMap::new()),
            None,
            "a replacement Workbench owner cannot reuse displayed state"
        );
        Ok(())
    }

    #[test]
    fn copy_rejects_changed_removed_ambiguous_or_hidden_entries() {
        let selected = entry(8, "SELECT selected\nFROM full_source");
        let displayed = displayed_copy(vec![selected.clone()], selected.clone());

        let mut changed_metadata = selected.clone();
        changed_metadata.connection = "Staging".to_owned();
        assert!(current_copy_entry(&displayed, &[&changed_metadata]).is_none());

        let mut changed_sql = selected.clone();
        changed_sql.sql.push_str(" WHERE replaced = true");
        assert!(current_copy_entry(&displayed, &[&changed_sql]).is_none());

        let replacement = entry(9, "SELECT replacement");
        assert!(current_copy_entry(&displayed, &[&replacement]).is_none());

        let duplicate = selected.clone();
        assert!(current_copy_entry(&displayed, &[&selected, &duplicate]).is_none());
        assert!(current_copy_entry(&displayed, &[]).is_none());

        let mut wrong_cursor = displayed_copy(vec![selected.clone()], selected.clone());
        wrong_cursor.cursor = Some(ItemKey::num(9));
        assert!(current_copy_entry(&wrong_cursor, &[&selected]).is_none());

        let mut hidden = displayed;
        hidden.copy_visible = false;
        assert!(current_copy_entry(&hidden, &[&selected]).is_none());
    }

    #[test]
    fn displayed_copy_witness_requires_the_same_weak_owner() {
        let entry = entry(8, "SELECT selected");
        let owner_token = Arc::new(());
        let owner = Arc::downgrade(&owner_token);
        let other_token = Arc::new(());
        let other = Arc::downgrade(&other_token);
        let key = crate::tabs::TabKey::new(1);
        let displayed = DisplayedRows {
            tab: key,
            owner: owner.clone(),
            visible: vec![entry.clone()],
            cursor: Some(item_key(&entry)),
            open_entry: None,
            open_visible: false,
            copy_entry: Some(entry),
            copy_visible: true,
        };
        let state = RowsState {
            displayed: Some(displayed),
            ..RowsState::default()
        };

        assert!(state.displayed_for(key, &owner).is_some());
        assert!(state.displayed_for(key, &other).is_none());
    }

    #[test]
    fn copy_area_fails_closed_when_its_full_button_cannot_fit() {
        assert!(copy_area(Rect::new(0, 0, 49, 20), 3).is_none());
        assert!(copy_area(Rect::new(0, 0, 66, 8), 3).is_none());
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LifecycleBucket {
    Empty,
    Lifecycle,
    Other,
}

fn lifecycle_bucket(cx: &mut Cx<'_>, owner: termrock::Id) -> LifecycleBucket {
    let mut saw = false;
    let mut other = false;
    for intent in cx.intents(owner) {
        saw = true;
        if !matches!(intent, Intent::FocusIn { .. } | Intent::FocusOut { .. }) {
            other = true;
        }
    }
    if other {
        LifecycleBucket::Other
    } else if saw {
        LifecycleBucket::Lifecycle
    } else {
        LifecycleBucket::Empty
    }
}

fn diagnose(cx: &mut Cx<'_>, owner: termrock::Id) {
    cx.diagnostic(Diagnostic::UndeliveredIntent { owner });
}

fn retired(cx: &Cx<'_>, owner: termrock::Id) -> bool {
    cx.area(owner).is_none()
        && !cx.state(owner).contains(StateFlags::FOCUSED)
        && cx.capture_owner() != Some(owner)
}
