//! Application-owned tab list projection over shared Picker.

use termrock::{AsItem, Id, Item, ItemKey, ItemRowLayout, LayerSize, Picker, PickerState};

use crate::tabs::{Tab, TabKey};
use crate::workbench::Workbench;

pub const ID: Id = Id::root("tablepro.tab-list");

#[derive(Debug, Clone)]
pub struct TabListItem {
    pub key: String,
    pub label: String,
    pub detail: String,
    pub glyph: &'static str,
    pub is_active: bool,
    pub tab_key: TabKey,
}

impl AsItem for TabListItem {
    fn as_item(&self) -> Item<'_> {
        let item = Item::new(ItemKey::text(&self.key), &self.label)
            .detail(&self.detail)
            .glyph(self.glyph);
        if self.is_active {
            item.tag("active")
        } else {
            item
        }
    }
}

impl TabListItem {
    pub fn from_workbench(workbench: &Workbench) -> (Vec<Self>, usize) {
        let tabs = workbench.tabs();
        let active_key = workbench.active_key();
        let mut items = Vec::with_capacity(tabs.len());
        let mut active_index = 0;

        for (idx, record) in tabs.iter().enumerate() {
            let tab_key = record.key();
            let is_active = Some(tab_key) == active_key;
            if is_active {
                active_index = idx;
            }
            let (glyph, label, detail) = match record.payload() {
                Tab::Table(tab) => {
                    let kind = if tab.is_structure() {
                        "structure"
                    } else {
                        "data"
                    };
                    (
                        "T",
                        tab.table.name.clone(),
                        format!("{}.{} · {kind}", tab.table.schema, tab.table.name),
                    )
                }
                Tab::Query(tab) => ("≡", tab.name.clone(), "query".to_string()),
                Tab::History(_) => ("H", "History".to_string(), "history".to_string()),
            };
            items.push(TabListItem {
                key: format!("{glyph}:{label}:{idx}"),
                label,
                detail,
                glyph,
                is_active,
                tab_key,
            });
        }
        (items, active_index)
    }
}

#[derive(Default)]
pub struct TabList {
    pub state: PickerState,
    pub items: Vec<TabListItem>,
}

impl TabList {
    pub fn open(&mut self, workbench: &Workbench) {
        self.state = PickerState::default();
        let (items, active_idx) = TabListItem::from_workbench(workbench);
        self.items = items;
        if let Some(active_item) = self.items.get(active_idx) {
            self.state
                .set_cursor(active_idx, ItemKey::text(&active_item.key));
        }
        self.component(120, 40)
            .reconcile(&mut self.state, &self.items);
    }

    pub fn component(&self, cols: u16, screen_rows: u16) -> Picker<'static, TabListItem> {
        let w = 64.min(cols.saturating_sub(4));
        let rows = (self.items.len() as u16).clamp(1, 12);
        let h = (7 + rows).min(screen_rows.saturating_sub(2));
        Picker::new(ID)
            .title("Open tabs")
            .placeholder("Filter tabs…")
            .width(w)
            .size(LayerSize::Fixed(w, h))
            .footer("↑↓ Move · Enter Switch · Delete Close tab · Esc Close")
            .item_layout(ItemRowLayout::Columns)
            .align(termrock::ScreenAlign::UpperThird)
    }
}
