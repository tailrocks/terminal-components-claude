//! Application-owned switcher projection over the shared Picker.
use crate::model::{SwitchItem, SwitchTarget, SwitcherIndex};
use crate::workbench::Workbench;
use junie_tui::{
    AsItem, FilterPolicy, Id, Item, ItemKey, ItemRowLayout, Picker, PickerState, ScopeKey,
};

pub(crate) const ID: Id = Id::root("tablepro.quick-switcher");
const SCOPES: &[ScopeKey] = &[
    ScopeKey::new(0),
    ScopeKey::new(1),
    ScopeKey::new(2),
    ScopeKey::new(3),
];

impl AsItem for SwitchItem {
    fn as_item(&self) -> Item<'_> {
        let group = self.target.group();
        let glyph = match &self.target {
            SwitchTarget::Table { .. } => "T",
            SwitchTarget::View { .. } => "V",
            SwitchTarget::Schema(_) => "S",
            SwitchTarget::Database(_) => "D",
            SwitchTarget::OpenTab(_) => "≡",
            SwitchTarget::Query(_) => "Q",
            SwitchTarget::Connection(_) => "C",
        };
        let item = Item::new(ItemKey::text(&self.key), &self.label)
            .detail(&self.detail)
            .glyph(glyph)
            .group(group)
            .matched(&self.matched);
        if self.open && !matches!(self.target, SwitchTarget::OpenTab(_)) {
            item.tag("open")
        } else {
            item
        }
    }
}

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
        // The product switcher exposes objects, tabs, and history; connection
        // management remains on its own screen.
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
