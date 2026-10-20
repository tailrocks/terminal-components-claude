use crate::workbench::Workbench;
use tablepro_domain::model::{SwitchItem, SwitchTarget, SwitcherIndex};
use tablepro_domain::tabs::{Tab, TabKey, TabRecord, TableMode};
use termrock::{
    AsItem, FilterPolicy, Id, Item, ItemKey, ItemRowLayout, Picker, PickerState, ScopeKey,
};

pub(crate) const ID: Id = Id::root("tablepro.quick-switcher");
pub(crate) const TAB_LIST_ID: Id = Id::root("tablepro.tab-list-picker");
const SCOPES: &[ScopeKey] = &[
    ScopeKey::new(0),
    ScopeKey::new(1),
    ScopeKey::new(2),
    ScopeKey::new(3),
];

#[derive(Default)]
pub(crate) struct QuickSwitcher {
    pub(crate) state: PickerState,
    pub(crate) items: Vec<SwitchItem>,
    pub(crate) owner: std::sync::Weak<()>,
    index: SwitcherIndex,
}
impl QuickSwitcher {
    pub(crate) fn open(&mut self, workbench: &Workbench) {
        self.state = PickerState::default();
        self.index = workbench.switcher();

        self.index
            .items
            .retain(|item| !matches!(item.target, SwitchTarget::Connection(_)));
        self.owner = workbench.owner_token();
        self.refresh();
    }
    pub(crate) fn component(&self) -> Picker<'static, SwitchItem> {
        let title = match self.state.scope(SCOPES).map(ScopeKey::get) {
            Some(1) => "Open Quickly · Tables · Tab scope",
            Some(2) => "Open Quickly · Schemas · Tab scope",
            Some(3) => "Open Quickly · Queries · Tab scope",
            _ => "Open Quickly · All · Tab scope",
        };
        Picker::new(ID)
            .title(title)
            .placeholder("Search tables, views, schemas, tabs, queries…")
            .width(88)
            .align(termrock::ScreenAlign::Center)
            .scopes(SCOPES)
            .filter(FilterPolicy::Caller)
            .item_layout(ItemRowLayout::Columns)
    }
    pub(crate) fn refresh(&mut self) {
        let scope = self.state.scope(SCOPES).map_or(0, ScopeKey::get);
        self.items = self.index.search(self.state.query());
        self.items.retain(|item| match scope {
            1 => matches!(
                item.target,
                SwitchTarget::Table { .. } | SwitchTarget::View { .. }
            ),
            2 => matches!(
                item.target,
                SwitchTarget::Schema(_) | SwitchTarget::Database(_)
            ),
            3 => matches!(item.target, SwitchTarget::Query(_)),
            _ => true,
        });
        if self.state.query().is_empty() && scope == 0 {
            self.items.truncate(24);
        }
        self.component().reconcile(&mut self.state, &self.items);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TabSummary {
    pub(crate) key: TabKey,
    label: String,
    detail: String,
    glyph: &'static str,
    tag: Option<&'static str>,
}

impl TabSummary {
    pub(crate) fn from_record(record: &TabRecord, active: bool) -> Self {
        let tab = record.payload();
        let (detail, glyph) = match tab {
            Tab::Table(table) => (
                format!(
                    "{}.{} · {}",
                    table.table.schema,
                    table.table.name,
                    match table.mode {
                        TableMode::Data => "data",
                        TableMode::Structure => "structure",
                    }
                ),
                "T",
            ),
            Tab::Query(query) => (
                if query.running {
                    "running".to_owned()
                } else {
                    query
                        .error
                        .as_ref()
                        .map(|error| error.lines().next().unwrap_or("query error").to_owned())
                        .or_else(|| query.result.as_ref().map(|_| "complete".to_owned()))
                        .unwrap_or_else(|| "query".to_owned())
                },
                "≡",
            ),
            Tab::History(_) => ("history".to_owned(), "H"),
        };
        Self {
            key: record.key(),
            label: tab.label(),
            detail,
            glyph,
            tag: if active {
                Some("active")
            } else if tab.dirty() {
                Some("unsaved")
            } else {
                None
            },
        }
    }
}

impl AsItem for TabSummary {
    fn as_item(&self) -> Item<'_> {
        let item = Item::new(ItemKey::num(self.key.get()), &self.label)
            .detail(&self.detail)
            .glyph(self.glyph);
        match self.tag {
            Some(tag) => item.tag(tag),
            None => item,
        }
    }
}

#[derive(Default)]
pub(crate) struct TabListPicker {
    pub(crate) state: PickerState,
    pub(crate) items: Vec<TabSummary>,
    pub(crate) owner: std::sync::Weak<()>,
}

impl TabListPicker {
    pub(crate) fn open(&mut self, workbench: &Workbench) {
        self.state = PickerState::default();
        self.items = workbench
            .tabs()
            .iter()
            .map(|record| {
                TabSummary::from_record(record, Some(record.key()) == workbench.active_key())
            })
            .collect();
        if let Some(active) = workbench.active_key()
            && let Some(index) = self.items.iter().position(|item| item.key == active)
        {
            self.state.set_cursor(index, ItemKey::num(active.get()));
        }
        self.owner = workbench.owner_token();
        Self::component().reconcile(&mut self.state, &self.items);
    }

    pub(crate) fn component() -> Picker<'static, TabSummary> {
        Picker::new(TAB_LIST_ID)
            .title("Open tabs")
            .placeholder("Filter tabs…")
            .width(64)
            .align(termrock::ScreenAlign::Center)
            .max_rows(12)
            .filter(FilterPolicy::Substring)
            .cursor_style(termrock::CursorStyle::Fill)
    }
}
