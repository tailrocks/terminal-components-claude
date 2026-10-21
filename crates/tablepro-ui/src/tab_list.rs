use tablepro_domain::tabs::{Tab, TabKey, TabRecord};
use termrock::{
    AsItem, CursorStyle, FilterPolicy, Id, Item, ItemKey, ItemRowLayout, LayerSize, Picker,
    PickerState, ScreenAlign, SelectionPolicy, Surface,
};

use crate::workbench::Workbench;

pub(crate) const ID: Id = Id::root("tablepro.tab-list");

pub(crate) struct TabRow {
    pub(crate) key: TabKey,
    label: String,
    detail: String,
    glyph: &'static str,
    tag: Option<&'static str>,
}

impl AsItem for TabRow {
    fn as_item(&self) -> Item<'_> {
        let item = Item::new(ItemKey::num(self.key.get()), self.label.as_str())
            .detail(self.detail.as_str())
            .glyph(self.glyph);
        match self.tag {
            Some(tag) => item.tag(tag),
            None => item,
        }
    }
}

#[derive(Default)]
pub(crate) struct TabList {
    pub(crate) state: PickerState,
    pub(crate) rows: Vec<TabRow>,
    pub(crate) owner: std::sync::Weak<()>,
}

impl TabList {
    pub(crate) fn open(&mut self, workbench: &Workbench) {
        self.state = PickerState::default();
        self.owner = workbench.owner_token();
        self.rows = rows(workbench);
        if let Some((index, row)) = self
            .rows
            .iter()
            .enumerate()
            .find(|(_, row)| workbench.active_key() == Some(row.key))
        {
            self.state.set_cursor(index, ItemKey::num(row.key.get()));
        }
        picker(self.rows.len()).reconcile(&mut self.state, &self.rows);
    }
}

pub(crate) fn picker(visible: usize) -> Picker<'static, TabRow> {
    let rows = u16::try_from(visible.clamp(1, 12)).unwrap_or(12);
    Picker::new(ID)
        .title("Open tabs")
        .placeholder("Filter tabs…")
        .hints("↑↓ Move · Enter Switch · Delete Close tab · Esc Close")
        .filter(FilterPolicy::Label)
        .selection_policy(SelectionPolicy::FirstEnabledOnFilterChange)
        .item_layout(ItemRowLayout::Columns)
        .cursor_style(CursorStyle::Fill)
        .align(ScreenAlign::GeometricCenter)
        .surface(Surface::Elevated)
        .size(LayerSize::Fixed(64, rows.saturating_add(7)))
}

pub(crate) fn rows(workbench: &Workbench) -> Vec<TabRow> {
    let active = workbench.active_key();
    workbench
        .tabs()
        .iter()
        .map(|record| row(record, active))
        .collect()
}

fn row(record: &TabRecord, active: Option<TabKey>) -> TabRow {
    let (glyph, detail) = match record.payload() {
        Tab::Query(query) => (
            "≡",
            if query.running {
                "running".to_owned()
            } else {
                query
                    .last_status
                    .as_ref()
                    .map_or_else(|| "query".to_owned(), |status| status.summary.clone())
            },
        ),
        Tab::Table(table) => {
            let qualified = table.table.qualified();
            let detail = if table.is_structure() {
                format!("{qualified} · structure")
            } else {
                format!("{qualified} · data")
            };
            ("T", detail)
        }
        Tab::History(_) => ("H", "history".to_owned()),
    };
    let tag = if Some(record.key()) == active {
        Some("active")
    } else if record.dirty() {
        Some("unsaved")
    } else {
        None
    };
    TabRow {
        key: record.key(),
        label: record.payload().label(),
        detail,
        glyph,
        tag,
    }
}
