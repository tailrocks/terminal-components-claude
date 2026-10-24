use crate::workbench::Workbench;
use tablepro_domain::model::{SwitchItem, SwitchTarget, SwitcherIndex};
use termrock::{
    CursorStyle, FilterPolicy, Id, ItemRowLayout, LayerSize, Picker, PickerState, ScopeKey,
    ScreenAlign, SelectionPolicy, Surface,
};

pub(crate) const ID: Id = Id::root("tablepro.quick-switcher");
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
        let scope_label = match self.state.scope(SCOPES).map(ScopeKey::get) {
            Some(1) => "Tables · Tab scope",
            Some(2) => "Schemas · Tab scope",
            Some(3) => "Queries · Tab scope",
            _ => "All · Tab scope",
        };
        // Baseline card: borders + title + query + gap + up to 12 rows + hint row.
        let rows = u16::try_from(self.items.len().clamp(1, 12)).unwrap_or(12);
        Picker::new(ID)
            .title("Open Quickly")
            .scope_label(scope_label)
            .placeholder("Search tables, views, schemas, tabs, queries…")
            .hints("↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close")
            .scopes(SCOPES)
            .selection_policy(SelectionPolicy::FirstEnabledOnFilterChange)
            .filter(FilterPolicy::Caller)
            .item_layout(ItemRowLayout::Columns)
            .cursor_style(CursorStyle::Fill)
            .align(ScreenAlign::GeometricCenter)
            .surface(Surface::Elevated)
            .size(LayerSize::Fixed(88, rows.saturating_add(7)))
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
