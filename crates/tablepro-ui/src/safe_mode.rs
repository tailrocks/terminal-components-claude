use tablepro_demo::db::SafeMode;
use termrock::{
    AsItem, CursorStyle, FilterPolicy, Id, Item, ItemKey, ItemRowLayout, LayerSize, Picker,
    PickerState, ScreenAlign, Surface,
};

pub(crate) const ID: Id = Id::root("tablepro.safe-mode");

pub(crate) struct SafeRow {
    index: u16,
    label: &'static str,
    detail: &'static str,
    glyph: &'static str,
    tag: Option<&'static str>,
}

impl AsItem for SafeRow {
    fn as_item(&self) -> Item<'_> {
        let item = Item::new(ItemKey::num(u64::from(self.index)), self.label)
            .detail(self.detail)
            .glyph(self.glyph);
        match self.tag {
            Some(tag) => item.tag(tag),
            None => item,
        }
    }
}

#[derive(Default)]
pub(crate) struct SafeModePicker {
    pub(crate) state: PickerState,
    pub(crate) rows: Vec<SafeRow>,
}

impl SafeModePicker {
    pub(crate) fn open(&mut self, current: SafeMode) {
        self.rows = rows(current);
        self.state = PickerState::default();
        if let Some(row) = self.rows.iter().find(|row| row.tag.is_some()) {
            self.state
                .set_cursor(usize::from(row.index), ItemKey::num(u64::from(row.index)));
        }
        picker(self.rows.len()).reconcile(&mut self.state, &self.rows);
    }
}

pub(crate) fn picker(visible: usize) -> Picker<'static, SafeRow> {
    // Baseline non-searchable card: borders + title + rows + gap + hint.
    let rows = u16::try_from(visible.clamp(1, 12)).unwrap_or(12);
    Picker::new(ID)
        .title("Safe Mode · this connection")
        .hints("↑↓ Move · Enter Set level · Esc Keep · levels are saved to the connection")
        .searchable(false)
        .filter(FilterPolicy::Label)
        .item_layout(ItemRowLayout::Columns)
        .cursor_style(CursorStyle::Fill)
        .align(ScreenAlign::GeometricCenter)
        .surface(Surface::Elevated)
        .size(LayerSize::Fixed(112, rows.saturating_add(5)))
}

pub(crate) fn rows(current: SafeMode) -> Vec<SafeRow> {
    SafeMode::ALL
        .into_iter()
        .enumerate()
        .map(|(index, mode)| SafeRow {
            index: u16::try_from(index).unwrap_or(u16::MAX),
            label: mode.label(),
            detail: mode.description(),
            glyph: if mode == current { "›" } else { " " },
            tag: (mode == current).then_some("current"),
        })
        .collect()
}

pub(crate) fn mode_of(key: ItemKey) -> Option<SafeMode> {
    let ItemKey::Num(n) = key else {
        return None;
    };
    let index = usize::try_from(n).ok()?;
    SafeMode::ALL.get(index).copied()
}
