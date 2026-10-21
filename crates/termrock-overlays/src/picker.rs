use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, overlay_chrome};
use crate::id::{Id, ItemKey, Part};
use crate::layer::{Anchor, Dismiss, LayerSize, LayerSpec, ScreenAlign};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{CursorStyle, Family, FgStep, Role, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_navigation::filter_list::{
    FilterList, FilterListAction, FilterListState, FilterPolicy, SelectionPolicy,
};
pub use termrock_navigation::item::{AsItem, Item, ItemColumns, ItemRow, ItemRowLayout};
use termrock_runtime::empty::EmptyState;
use termrock_runtime::rowui::{RowFn, RowUi};

struct BorrowedRow<'a, R>(&'a R);

impl<T, R: RowFn<T>> RowFn<T> for BorrowedRow<'_, R> {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        self.0.row(value, row);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ScopeKey(u16);

impl ScopeKey {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerAction {
    Chosen(ItemKey),
    ChosenAlt(ItemKey),
    Secondary(ItemKey),
    Back,
    Scope(ScopeKey),
    QueryChanged,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PickerState {
    list: FilterListState,
    active_scope: usize,
}

impl PickerState {
    pub fn query(&self) -> &str {
        self.list.query()
    }
    pub const fn is_query_editing(&self) -> bool {
        self.list.is_query_editing()
    }
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.list.set_query(query);
    }
    fn reset_query(&mut self, query: impl Into<String>) {
        self.list.reset_query(query);
    }
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.list.cursor()
    }
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.list.set_cursor(index, key);
    }
    pub fn scope(&self, scopes: &[ScopeKey]) -> Option<ScopeKey> {
        scopes.get(self.active_scope).copied()
    }
    pub const fn list(&self) -> &FilterListState {
        &self.list
    }
    pub fn take_query_edit_menu_request(
        &mut self,
    ) -> Option<termrock_fields::input::EditMenuRequest> {
        self.list.take_query_edit_menu_request()
    }
}

pub struct Picker<'a, T, R = ItemRow> {
    id: Id,
    title: &'a str,
    scope_label: Option<&'a str>,
    width: Option<u16>,
    max_rows: Option<u16>,
    searchable: bool,
    filter: FilterPolicy,
    placeholder: &'a str,
    hints: &'a str,
    scopes: &'a [ScopeKey],
    requested_size: Option<LayerSize>,
    align: Option<ScreenAlign>,
    empty: Option<EmptyState<'a>>,
    row: R,
    item_layout: ItemRowLayout,
    surface: Surface,
    cursor_style: CursorStyle,
    delete_is_secondary: bool,
    metadata_gap: u16,
    /// Blank rows kept under the list, inside the card.
    tail: u16,
    selection_policy: SelectionPolicy,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    _item: PhantomData<fn(&T)>,
}

impl<T, R> core::fmt::Debug for Picker<'_, T, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Picker")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("scopes", &self.scopes)
            .field("searchable", &self.searchable)
            .field("requested_size", &self.requested_size)
            .field("align", &self.align)
            .finish_non_exhaustive()
    }
}

impl<T> Picker<'_, T, ItemRow> {
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            title: "Choose",
            scope_label: None,
            width: None,
            max_rows: None,
            searchable: true,
            filter: FilterPolicy::Label,
            placeholder: "Type to search…",
            hints: "",
            scopes: &[],
            requested_size: None,
            align: None,
            empty: None,
            row: ItemRow,
            item_layout: ItemRowLayout::Compact,
            surface: Surface::Overlay,
            cursor_style: CursorStyle::Bar,
            delete_is_secondary: true,
            metadata_gap: 1,
            tail: 0,
            selection_policy: SelectionPolicy::Preserve,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _item: PhantomData,
        }
    }
}

impl<T> Picker<'_, T, ItemRow> {
    #[must_use]
    pub const fn item_layout(mut self, layout: ItemRowLayout) -> Self {
        self.item_layout = layout;
        self
    }
}

impl<'a, T, R> Picker<'a, T, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BORDER,
        Part::TITLE,
        Part::QUERY,
        Part::GUTTER,
        Part::ICON,
        Part::ROW,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];
    pub const fn id(&self) -> Id {
        self.id
    }
    #[must_use]
    pub const fn surface(mut self, surface: Surface) -> Self {
        self.surface = surface;
        self
    }
    #[must_use]
    pub const fn cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        self
    }
    #[must_use]
    pub const fn delete_is_secondary(mut self, enabled: bool) -> Self {
        self.delete_is_secondary = enabled;
        self
    }
    #[must_use]
    pub const fn metadata_gap(mut self, gap: u16) -> Self {
        self.metadata_gap = gap;
        self
    }
    /// Keep this many card rows blank under the results.
    #[must_use]
    pub const fn tail(mut self, rows: u16) -> Self {
        self.tail = rows;
        self
    }
    #[must_use]
    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    #[must_use]
    pub const fn scope_label(mut self, label: &'a str) -> Self {
        self.scope_label = Some(label);
        self
    }
    #[must_use]
    pub const fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
    /// Caps the number of item rows requested from the layer.
    ///
    /// The default is `Theme::design.size.popup_max_rows`. Zero is treated as
    /// one so a picker always shows at least one row.
    #[must_use]
    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = Some(if rows == 0 { 1 } else { rows });
        self
    }
    #[must_use]
    pub const fn filter(mut self, policy: FilterPolicy) -> Self {
        self.filter = policy;
        self
    }
    #[must_use]
    pub const fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }
    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
    #[must_use]
    pub const fn hints(mut self, hints: &'a str) -> Self {
        self.hints = hints;
        self
    }
    #[must_use]
    pub const fn scopes(mut self, scopes: &'a [ScopeKey]) -> Self {
        self.scopes = scopes;
        self
    }

    /// Select the first enabled result after a query or scope change.
    #[must_use]
    pub const fn selection_policy(mut self, policy: SelectionPolicy) -> Self {
        self.selection_policy = policy;
        self
    }
    #[must_use]
    pub const fn size(mut self, size: LayerSize) -> Self {
        self.requested_size = Some(size);
        self
    }
    #[must_use]
    pub const fn align(mut self, align: ScreenAlign) -> Self {
        self.align = Some(align);
        self
    }
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    pub fn row<R2: RowFn<T>>(self, row: R2) -> Picker<'a, T, R2> {
        Picker {
            id: self.id,
            title: self.title,
            scope_label: self.scope_label,
            width: self.width,
            max_rows: self.max_rows,
            searchable: self.searchable,
            filter: self.filter,
            placeholder: self.placeholder,
            hints: self.hints,
            scopes: self.scopes,
            requested_size: self.requested_size,
            align: self.align,
            empty: self.empty,
            row,
            item_layout: ItemRowLayout::Compact,
            surface: self.surface,
            cursor_style: self.cursor_style,
            delete_is_secondary: self.delete_is_secondary,
            metadata_gap: self.metadata_gap,
            tail: self.tail,
            selection_policy: self.selection_policy,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self.ov = self.ov.part(parts);
        self
    }
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }
}

fn clamp_fixed_width(size: LayerSize, viewport_width: u16) -> LayerSize {
    match size {
        LayerSize::Fixed(width, height) if viewport_width > 4 => {
            LayerSize::Fixed(width.min(viewport_width.saturating_sub(4)), height)
        }
        other => other,
    }
}

impl<T: AsItem, R: RowFn<T>> Picker<'_, T, R> {
    fn list(&self) -> FilterList<'_, T, BorrowedRow<'_, R>> {
        let mut list = FilterList::new(self.id)
            .searchable(self.searchable)
            .placeholder(self.placeholder)
            .delete_is_secondary(self.delete_is_secondary)
            .filter(self.filter)
            .row(BorrowedRow(&self.row))
            .with_item_layout(self.item_layout)
            .cursor_style(self.cursor_style)
            .metadata_gap(self.metadata_gap)
            .selection_policy(self.selection_policy)
            .preserve_surface(true);
        if let Some(empty) = self.empty {
            list = list.empty(empty);
        }
        if let Some(patch) = self.patch {
            list = list.patch(patch);
        }
        list = list.patch_part(self.parts);
        list
    }

    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        if let Some(size) = self.requested_size {
            return clamp_fixed_width(size, cx.viewport().width);
        }
        let d = cx.design();
        let width = self.width.unwrap_or_else(|| {
            FilterList::<T, BorrowedRow<'_, R>>::semantic_width(items)
                .clamp(d.size.popup_min_width, d.size.popup_max_width)
        });
        let rows = items
            .len()
            .min(usize::from(self.max_rows.unwrap_or(d.size.popup_max_rows)))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;

        // The card is a rule, a query row and its gap, the item rows, and two
        // trailing content rows. Non-searchable pickers omit the query and gap.
        LayerSize::Fixed(width, rows.saturating_add(5))
    }

    pub fn layer(&self, cx: &Cx<'_>, items: &[T]) -> LayerSpec {
        LayerSpec::modal(self.id)
            .dismiss(Dismiss {
                esc: false,
                outside_click: true,
                focus_out: false,
                ..Default::default()
            })
            .anchor(Anchor::Screen(
                self.align.unwrap_or(ScreenAlign::UpperThird),
            ))
            .initial_focus(self.id)
            .size(self.measured_size(cx, items))
    }

    pub fn reconcile(&self, st: &mut PickerState, items: &[T]) -> bool {
        let changed = self.clear_hidden_query(st);
        self.list().reconcile(&mut st.list, items);
        changed
    }

    fn clear_hidden_query(&self, st: &mut PickerState) -> bool {
        if !self.searchable && !st.query().is_empty() {
            st.reset_query("");
            true
        } else {
            false
        }
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut PickerState,
        items: &[T],
    ) -> Response<PickerAction> {
        if cx.is_open(self.id) {
            cx.resize_layer(self.id, self.measured_size(cx, items));
        }
        // Let FilterList clear hidden query state during update so it can also
        // consume the typed binding delivered by the previous searchable
        // frame (for example Esc or Backspace on the transition event).
        let query_changed = !self.searchable && !st.query().is_empty();
        let inner = self.list().update(cx, &mut st.list, items);
        let mut acc = Acc::new();
        if inner.is_consumed() {
            acc.consumed();
        }
        if inner.is_changed() {
            acc.repaint();
        }
        if query_changed {
            acc.repaint();
            acc.action(PickerAction::QueryChanged);
        }
        if let Some(action) = inner.action_ref().copied() {
            match action {
                FilterListAction::Chose(_)
                | FilterListAction::ChoseAlt(_)
                | FilterListAction::Secondary(_)
                    if query_changed => {}
                FilterListAction::QueryChanged => {
                    if self.selection_policy == SelectionPolicy::FirstEnabledOnFilterChange {
                        // The owner may replace the projected items after this
                        // response. Keep the reset pending for that reconcile.
                        st.list.reset_selection_on_next_reconcile();
                    }
                    acc.action(PickerAction::QueryChanged);
                }
                FilterListAction::Chose(key) => {
                    acc.action(PickerAction::Chosen(key));
                }
                FilterListAction::ChoseAlt(key) => {
                    acc.action(PickerAction::ChosenAlt(key));
                }
                FilterListAction::Secondary(key) => {
                    acc.action(PickerAction::Secondary(key));
                }
                FilterListAction::Back => {
                    acc.action(PickerAction::Back);
                }
                FilterListAction::Cancel => {
                    cx.close_layer(self.id, None);
                    acc.repaint();
                }
                FilterListAction::NextScope if !self.scopes.is_empty() => {
                    st.active_scope = st
                        .active_scope
                        .saturating_add(1)
                        .checked_rem(self.scopes.len())
                        .unwrap_or(0);
                    if let Some(scope) = self.scopes.get(st.active_scope).copied() {
                        if self.selection_policy == SelectionPolicy::FirstEnabledOnFilterChange {
                            st.list.reset_selection_on_next_reconcile();
                        }
                        acc.action(PickerAction::Scope(scope));
                    }
                }
                FilterListAction::PrevScope if !self.scopes.is_empty() => {
                    st.active_scope = st
                        .active_scope
                        .checked_sub(1)
                        .unwrap_or(self.scopes.len() - 1);
                    if let Some(scope) = self.scopes.get(st.active_scope).copied() {
                        if self.selection_policy == SelectionPolicy::FirstEnabledOnFilterChange {
                            st.list.reset_selection_on_next_reconcile();
                        }
                        acc.action(PickerAction::Scope(scope));
                    }
                }
                FilterListAction::Moved
                | FilterListAction::NextScope
                | FilterListAction::PrevScope => {
                    acc.repaint();
                }
            }
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &PickerState, items: &[T]) -> Rect {
        let mut live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        live.remove(StateFlags::PRESSED);
        overlay_chrome(
            ui,
            self.id,
            area,
            Family::PICKER,
            self.surface,
            self.ov,
            live,
            live,
            |ui, inner| {
                if inner.is_empty() {
                    return area;
                }

                let content = if self.requested_size.is_some() {
                    Rect {
                        x: inner.x.saturating_add(1),
                        width: inner.width.saturating_sub(2),
                        ..inner
                    }
                } else {
                    inner
                };
                if content.is_empty() {
                    return area;
                }
                let title = Rect {
                    height: 1,
                    ..content
                };
                let title_style = self.ov.style(
                    ui,
                    self.id,
                    Family::PICKER,
                    Variant::DEFAULT,
                    Part::TITLE,
                    live,
                );
                ui.paint_str(title, self.title, title_style.style);
                if let Some(label) = self.scope_label {
                    let available = content
                        .width
                        .saturating_sub(width(self.title))
                        .saturating_sub(3);
                    if available >= 6 {
                        let label = crate::text::truncate(label, available);
                        let label_width = width(&label);
                        let label_area = Rect {
                            x: content.right().saturating_sub(label_width),
                            width: label_width,
                            y: content.y,
                            height: 1,
                        };
                        let meta_style = self.ov.style(
                            ui,
                            self.id,
                            Family::PICKER,
                            Variant::DEFAULT,
                            Part::META,
                            live,
                        );
                        ui.paint_str(label_area, &label, meta_style.style);
                    }
                }
                if self.searchable {
                    let query = Rect {
                        y: content.y.saturating_add(1),
                        height: 1,
                        ..content
                    };
                    self.list().draw_query(ui, query, &st.list);
                }

                let show_hint = !self.hints.is_empty();
                if show_hint {
                    let hint_area = Rect {
                        y: content.y.saturating_add(content.height.saturating_sub(1)),
                        height: 1,
                        ..content
                    };
                    // Baseline hint ink is faint, including the cells the string covers.
                    let hint_style = ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Faint))
                            .set_bg(Role::CurrentSurface),
                    );
                    ui.paint_str(hint_area, self.hints, hint_style);
                }
                let list = Rect {
                    y: content
                        .y
                        .saturating_add(if self.searchable { 3 } else { 1 }),
                    height: content.height.saturating_sub(
                        match (self.searchable, show_hint) {
                            (true, true) => 4u16,
                            (true, false) => 3,
                            (false, _) => 2,
                        }
                        .saturating_add(self.tail),
                    ),
                    ..content
                };
                self.list().draw_results(ui, list, &st.list, items);
                area
            },
        )
    }
}

pub type CommandPalette<'a, T, R = ItemRow> = Picker<'a, T, R>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    struct Domain {
        id: u64,
        name: &'static str,
    }
    impl AsItem for Domain {
        fn as_item(&self) -> Item<'_> {
            Item::new(ItemKey::num(self.id), self.name)
        }
    }

    #[test]
    fn domain_item_needs_as_item_not_display() {
        let domain = Domain {
            id: 7,
            name: "seven",
        };
        assert_eq!(domain.as_item().key, ItemKey::num(7));
    }

    #[test]
    fn actions_use_semantic_item_key() {
        let item = Item::new(ItemKey::num(41), "display text");
        assert_eq!(item.as_item().key, ItemKey::num(41));
    }

    #[test]
    fn matched_indices_are_original_grapheme_ordinals() {
        let matched = [1usize, 3];
        let item = Item::new(ItemKey::num(1), "aé日z").matched(&matched);
        assert_eq!(item.matched, &[1, 3]);
    }

    #[test]
    fn query_change_is_visible_through_state_query() {
        let mut state = FilterListState::default();
        state.set_query("a");
        assert_eq!(state.query(), "a");
    }

    #[test]
    fn picker_routes_query_editing_navigation_and_clear_through_text_input() {
        use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
        use termrock_runtime::runtime::{App, Runtime};

        const ID: Id = Id::root("picker.query-runtime");
        struct QueryApp {
            state: PickerState,
            opened: bool,
            actions: Vec<PickerAction>,
        }
        impl App for QueryApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::new(ID).title("Find");
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(ID, picker.layer(cx, &items));
                }
                let mut response = picker.update(cx, &mut self.state, &items);
                if let Some(action) = response.take_action() {
                    self.actions.push(action);
                }
                response.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::new(ID).title("Find");
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, &items));
            }
        }

        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        let mut runtime = Runtime::new(
            QueryApp {
                state: PickerState::default(),
                opened: false,
                actions: Vec::new(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(ID));

        let key = |code| {
            Input::Key(Key {
                code,
                mods: KeyModifiers::NONE,
            })
        };
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('b')));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "b");
        assert_eq!(runtime.app().state.list().matched_len(), 1);
        assert_eq!(
            runtime.app().actions.last(),
            Some(&PickerAction::QueryChanged)
        );

        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('k'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "b", "Ctrl+K navigates results");

        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Delete));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "b");
        assert!(
            runtime
                .app()
                .actions
                .contains(&PickerAction::Secondary(ItemKey::num(2))),
            "actions={:?}, cursor={:?}",
            runtime.app().actions,
            runtime.app().state.cursor()
        );

        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Backspace));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().state.query(),
            "",
            "editing={}, draft={:?}, actions={:?}",
            runtime.app().state.is_query_editing(),
            runtime.app().state.query(),
            runtime.app().actions
        );
        assert_eq!(runtime.app().state.list().matched_len(), 2);
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Backspace));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert!(runtime.app().actions.contains(&PickerAction::Back));
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('b')));
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let query_caret = runtime.cursor().expect("visible query owns its cursor");
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().state.query(),
            "",
            "actions: {:?}",
            runtime.app().actions
        );
        assert!(
            runtime.is_open(ID),
            "first Esc clears query, keeps picker open"
        );
        assert!(runtime.app().state.list().is_query_editing());
        assert_eq!(runtime.cursor().map(|cursor| cursor.y), Some(query_caret.y));
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Paste("a".into()));
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().state.query(),
            "a",
            "paste immediately after Esc"
        );
        let resized = Rect::new(0, 0, 90, 26);
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Resize(resized.width, resized.height),
        );
        buffer = ratatui_core::buffer::Buffer::empty(resized);
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "a");
        assert!(runtime.app().state.list().is_query_editing());
        assert!(
            runtime.cursor().is_some(),
            "resized query retains its caret"
        );
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "");
        for code in [KeyCode::Char('z'), KeyCode::Char('z')] {
            let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(code));
            runtime.draw_buffer(resized, &mut buffer).commit_presented();
        }
        assert_eq!(runtime.app().state.query(), "zz");
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert!(runtime.is_open(ID));
        assert!(runtime.app().state.list().is_query_editing());
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Paste("a".into()));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "zza");

        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "");

        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert!(!runtime.is_open(ID), "second Esc dismisses empty picker");
        assert!(runtime.app().actions.contains(&PickerAction::QueryChanged));

        runtime.app_mut().opened = false;
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert!(runtime.is_open(ID));
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('b')));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "b");
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        runtime.draw_buffer(resized, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "");
        assert!(runtime.cursor().is_some());
    }

    #[test]
    fn filter_navigation_remapping_uses_typed_action_not_delete_chord_claim() {
        use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
        use termrock_runtime::keymap::KeyMap;
        use termrock_runtime::runtime::{App, Runtime};
        use termrock_runtime::{ActionKey, Chord};

        const ID: Id = Id::root("picker.remapped-filter-navigation");
        const DOWN: ActionKey = ActionKey::custom("filter-list.down");
        const SECONDARY: ActionKey = ActionKey::custom("filter-list.secondary");
        struct RemappedApp {
            state: PickerState,
            opened: bool,
            actions: Vec<PickerAction>,
            keymap: KeyMap,
        }
        impl App for RemappedApp {
            fn keymap(&self) -> &KeyMap {
                &self.keymap
            }

            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::new(ID).title("Find");
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                if !self.opened {
                    self.opened = true;
                    cx.open_layer(ID, picker.layer(cx, &items));
                }
                let mut response = picker.update(cx, &mut self.state, &items);
                if let Some(action) = response.take_action() {
                    self.actions.push(action);
                }
                response.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::new(ID).title("Find");
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, &items));
            }
        }

        let mut keymap = KeyMap::new();
        keymap.remove_component(ID, SECONDARY);
        keymap.remap_component(ID, DOWN, Chord::key(KeyCode::Delete));
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        let mut runtime = Runtime::new(
            RemappedApp {
                state: PickerState::default(),
                opened: false,
                actions: Vec::new(),
                keymap,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Delete,
                mods: KeyModifiers::NONE,
            }),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(2)));
        assert!(
            !runtime
                .app()
                .actions
                .iter()
                .any(|action| matches!(action, PickerAction::Secondary(_)))
        );
    }

    fn measured_at(size: LayerSize, viewport_width: u16) -> LayerSize {
        struct MeasureApp {
            size: LayerSize,
            measured: Option<LayerSize>,
        }
        impl termrock_runtime::runtime::App for MeasureApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                self.measured = Some(
                    Picker::<Domain>::new(Id::root("picker.fixed-width"))
                        .size(self.size)
                        .measured_size(cx, &[]),
                );
                Response::ignored()
            }

            fn draw(&self, _ui: &mut Ui<'_>) {}
        }
        let area = Rect::new(0, 0, viewport_width, 9);
        let mut runtime = termrock_runtime::runtime::Runtime::new(
            MeasureApp {
                size,
                measured: None,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::event::Input::Tick,
        );
        runtime.app().measured.expect("measured_size ran")
    }

    #[test]
    fn fixed_width_keeps_a_four_column_screen_margin() {
        let requested = LayerSize::Fixed(84, 9);
        assert_eq!(measured_at(requested, 72), LayerSize::Fixed(68, 9));
        assert_eq!(measured_at(requested, 80), LayerSize::Fixed(76, 9));
        assert_eq!(measured_at(requested, 100), LayerSize::Fixed(84, 9));
        assert_eq!(measured_at(requested, 0), LayerSize::Fixed(84, 9));
        assert_eq!(measured_at(requested, 4), LayerSize::Fixed(84, 9));
        assert_eq!(measured_at(LayerSize::Fill, 72), LayerSize::Fill);
    }
}
