//! Application-owned Safe Mode picker projection over shared Picker.

use termrock::{
    AsItem, Id, Item, ItemKey, ItemRowLayout, LayerSize, Picker, PickerState,
};

use crate::SafeMode;

pub const ID: Id = Id::root("tablepro.safe-mode-picker");

#[derive(Debug, Clone)]
pub struct SafeModeItem {
    pub mode: SafeMode,
    pub is_current: bool,
}

impl AsItem for SafeModeItem {
    fn as_item(&self) -> Item<'_> {
        let glyph = if self.is_current { "›" } else { " " };
        let item = Item::new(ItemKey::index(self.mode as usize), self.mode.label())
            .glyph(glyph)
            .detail(self.mode.description());
        if self.is_current {
            item.tag("current")
        } else {
            item
        }
    }
}

#[derive(Default)]
pub struct SafeModePicker {
    pub state: PickerState,
    pub items: Vec<SafeModeItem>,
}

impl SafeModePicker {
    pub fn open(&mut self, current_mode: SafeMode) {
        self.state = PickerState::default();
        let mut items = Vec::with_capacity(SafeMode::ALL.len());
        let mut active_index = 0;
        for (idx, &mode) in SafeMode::ALL.iter().enumerate() {
            let is_current = mode == current_mode;
            if is_current {
                active_index = idx;
            }
            items.push(SafeModeItem { mode, is_current });
        }
        self.items = items;
        self.state.set_cursor(active_index, ItemKey::index(active_index));
        self.component(120, 40).reconcile(&mut self.state, &self.items);
    }

    pub fn component(&self, cols: u16, screen_rows: u16) -> Picker<'static, SafeModeItem> {
        let w = 112.min(cols.saturating_sub(4));
        let rows = SafeMode::ALL.len() as u16;
        let h = (5 + rows).min(screen_rows.saturating_sub(2));
        Picker::new(ID)
            .title("Safe Mode · this connection")
            .width(w)
            .size(LayerSize::Fixed(w, h))
            .searchable(false)
            .footer("↑↓ Move · Enter Set level · Esc Keep · levels are saved to the connection")
            .item_layout(ItemRowLayout::Columns)
            .align(termrock::ScreenAlign::UpperThird)
    }
}
