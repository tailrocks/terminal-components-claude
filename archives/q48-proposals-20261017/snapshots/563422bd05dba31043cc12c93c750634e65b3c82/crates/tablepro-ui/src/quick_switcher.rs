//! Application-owned switcher projection over shared Picker.

use termrock::{
    AsItem, FilterPolicy, Id, Item, ItemKey, ItemRowLayout, LayerSize, Picker, PickerState,
    ScopeKey,
};

use crate::model::{SwitchItem, SwitchTarget, SwitcherIndex};
use crate::workbench::Workbench;

pub const ID: Id = Id::root("tablepro.quick-switcher");
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
pub struct QuickSwitcher {
    pub state: PickerState,
    pub items: Vec<SwitchItem>,
    pub owner: std::sync::Weak<()>,
    index: SwitcherIndex,
}

impl QuickSwitcher {
    pub fn open(&mut self, workbench: &Workbench) {
        self.state = PickerState::default();
        self.index = workbench.switcher();
        self.index
            .items
            .retain(|item| !matches!(item.target, SwitchTarget::Connection(_)));
        self.owner = workbench.owner_token();
        self.refresh();
    }

    pub fn component(&self, cols: u16, screen_rows: u16) -> Picker<'static, SwitchItem> {
        let meta = match self.state.scope(SCOPES).map(ScopeKey::get) {
            Some(1) => "Tables · Tab scope",
            Some(2) => "Schemas · Tab scope",
            Some(3) => "Queries · Tab scope",
            _ => "All · Tab scope",
        };
        let w = 88.min(cols.saturating_sub(4));
        let rows = (self.items.len() as u16).clamp(1, 12);
        let h = (7 + rows).min(screen_rows.saturating_sub(2));
        Picker::new(ID)
            .title("Open Quickly")
            .meta(meta)
            .placeholder("Search tables, views, schemas, tabs, queries…")
            .width(w)
            .size(LayerSize::Fixed(w, h))
            .footer("↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close")
            .scopes(SCOPES)
            .filter(FilterPolicy::Caller)
            .item_layout(ItemRowLayout::Columns)
            .align(termrock::ScreenAlign::UpperThird)
    }

    pub fn refresh(&mut self) {
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
        self.component(120, 40)
            .reconcile(&mut self.state, &self.items);
    }
}
