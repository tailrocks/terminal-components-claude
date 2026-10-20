use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, overlay_chrome};
use crate::id::{Id, ItemKey, Part};
use crate::layer::{Anchor, LayerSize, LayerSpec, ScreenAlign};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{CursorStyle, Family, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_navigation::filter_list::{
    FilterList, FilterListAction, FilterListState, FilterPolicy,
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
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.list.set_query(query);
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
    scopes: &'a [ScopeKey],
    requested_size: Option<LayerSize>,
    align: Option<ScreenAlign>,
    empty: Option<EmptyState<'a>>,
    row: R,
    item_layout: ItemRowLayout,
    surface: Surface,
    cursor_style: Option<CursorStyle>,
    metadata_gap: u16,
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
            scopes: &[],
            requested_size: None,
            align: None,
            empty: None,
            row: ItemRow,
            item_layout: ItemRowLayout::Compact,
            surface: Surface::Overlay,
            cursor_style: None,
            metadata_gap: 1,
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
        self.cursor_style = Some(style);
        self
    }
    #[must_use]
    pub const fn metadata_gap(mut self, gap: u16) -> Self {
        self.metadata_gap = gap;
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
    pub const fn scopes(mut self, scopes: &'a [ScopeKey]) -> Self {
        self.scopes = scopes;
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
            scopes: self.scopes,
            requested_size: self.requested_size,
            align: self.align,
            empty: self.empty,
            row,
            item_layout: ItemRowLayout::Compact,
            surface: self.surface,
            cursor_style: self.cursor_style,
            metadata_gap: self.metadata_gap,
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
            .register_owner(!self.searchable)
            .filter(self.filter)
            .row(BorrowedRow(&self.row))
            .with_item_layout(self.item_layout)
            .cursor_style(self.cursor_style.unwrap_or(if self.searchable {
                CursorStyle::Fill
            } else {
                CursorStyle::Bar
            }))
            .metadata_gap(self.metadata_gap);
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
        let max_rows = self.max_rows.unwrap_or(d.size.popup_max_rows).max(1);
        let rows = items
            .len()
            .min(usize::from(max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;

        LayerSize::Fixed(width, rows.saturating_add(5))
    }

    pub fn layer(&self, cx: &Cx<'_>, items: &[T]) -> LayerSpec {
        LayerSpec::modal(self.id)
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
            st.set_query("");
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
        let query_changed = self.clear_hidden_query(st);
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
                        acc.action(PickerAction::Scope(scope));
                    }
                }
                FilterListAction::PrevScope if !self.scopes.is_empty() => {
                    st.active_scope = if st.active_scope == 0 {
                        self.scopes.len().saturating_sub(1)
                    } else {
                        st.active_scope.saturating_sub(1)
                    };
                    if let Some(scope) = self.scopes.get(st.active_scope).copied() {
                        acc.action(PickerAction::Scope(scope));
                    }
                }
                FilterListAction::Moved | FilterListAction::NextScope => {
                    acc.repaint();
                }
                FilterListAction::PrevScope => acc.repaint(),
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
                    self.list()
                        .draw_query(ui, query, &st.list, self.placeholder);
                }

                let list = Rect {
                    y: content
                        .y
                        .saturating_add(if self.searchable { 3 } else { 1 }),
                    height: content
                        .height
                        .saturating_sub(if self.searchable { 3 } else { 2 }),
                    ..content
                };
                self.list().draw(ui, list, &st.list, items);
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
    fn query_field_resolves_picker_family_theme_and_part_patch() {
        const ID: Id = Id::root("picker.query-presentation");
        const QUERY_PATCH: &[(Part, StylePatch)] = &[(
            Part::QUERY,
            StylePatch::new().set_fg(crate::theme::Role::Danger),
        )];
        const GLOBAL_PATCH: StylePatch = StylePatch::new().set_bg(crate::theme::Role::Danger);

        struct PaintApp {
            state: PickerState,
            patch_kind: u8,
        }

        impl termrock_runtime::runtime::App for PaintApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Self::picker(self.patch_kind);
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &[]));
                }
                picker.update(cx, &mut self.state, &[]).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Self::picker(self.patch_kind);
                ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, &[]));
            }
        }

        impl PaintApp {
            fn picker(patch_kind: u8) -> Picker<'static, Item<'static>> {
                let picker = Picker::new(ID)
                    .placeholder("X")
                    .size(LayerSize::Fixed(20, 7));
                match patch_kind {
                    1 => picker.patch(&GLOBAL_PATCH),
                    2 => picker.patch_part(QUERY_PATCH),
                    _ => picker,
                }
            }
        }

        let render = |patch_kind: u8, themed, focused_style| {
            let theme = if themed {
                Theme::junie().override_family(Family::PICKER, |recipe| {
                    recipe
                        .part(Part::QUERY)
                        .base(StylePatch::new().set_fg(crate::theme::Role::Warning));
                    if focused_style {
                        recipe.part(Part::QUERY).when(
                            StateFlags::FOCUSED,
                            StylePatch::new().set_fg(crate::theme::Role::Success),
                        );
                    }
                })
            } else {
                Theme::junie()
            };
            let mut runtime = termrock_runtime::runtime::Runtime::new(
                PaintApp {
                    state: PickerState::default(),
                    patch_kind,
                },
                theme,
            );
            let _ = runtime.initialize();
            let area = Rect::new(0, 0, 30, 12);
            let mut buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            buffer
        };

        let plain = render(0, false, false);
        let base_theme = render(0, true, false);
        let themed = render(0, true, true);
        let patched = render(2, true, true);
        let globally_patched = render(1, false, false);
        assert_eq!(plain[(9, 2)].symbol(), "X");
        assert_ne!(plain[(9, 2)].fg, base_theme[(9, 2)].fg);
        assert_ne!(base_theme[(9, 2)].fg, themed[(9, 2)].fg);
        assert_ne!(themed[(9, 2)].fg, patched[(9, 2)].fg);
        assert_ne!(plain[(9, 2)].bg, globally_patched[(9, 2)].bg);
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

    #[test]
    fn max_rows_defaults_to_theme_and_explicit_value_overrides_it() {
        struct MeasureApp {
            max_rows: Option<u16>,
            items: Vec<Item<'static>>,
            measured: Option<LayerSize>,
        }
        impl termrock_runtime::runtime::App for MeasureApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let mut picker = Picker::<Item<'static>>::new(Id::root("picker.max-rows"));
                if let Some(max_rows) = self.max_rows {
                    picker = picker.max_rows(max_rows);
                }
                self.measured = Some(picker.measured_size(cx, &self.items));
                Response::ignored()
            }

            fn draw(&self, _ui: &mut Ui<'_>) {}
        }

        let items: Vec<_> = (0..20)
            .map(|index| Item::new(ItemKey::num(index), "row"))
            .collect();
        let measure = |popup_max_rows, max_rows| {
            let mut theme = Theme::junie();
            theme.design.size.popup_max_rows = popup_max_rows;
            let mut runtime = termrock_runtime::runtime::Runtime::new(
                MeasureApp {
                    max_rows,
                    items: items.clone(),
                    measured: None,
                },
                theme,
            );
            let _ = runtime.initialize();
            let area = Rect::new(0, 0, 120, 40);
            let mut buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            let _ = termrock_runtime::runtime::stub::deliver(
                &mut runtime,
                termrock_runtime::event::Input::Tick,
            );
            runtime.app().measured.expect("measured_size ran")
        };
        assert_eq!(measure(5, None), LayerSize::Fixed(12, 10));
        assert_eq!(measure(5, Some(3)), LayerSize::Fixed(12, 8));
    }

    #[test]
    fn searchable_picker_fills_cursor_row_while_query_owns_cursor() {
        const ID: Id = Id::root("picker.single-focus-marker");

        struct FocusApp {
            state: PickerState,
            items: Vec<Item<'static>>,
        }

        impl termrock_runtime::runtime::App for FocusApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID).size(LayerSize::Fixed(20, 8));
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &self.items));
                }
                picker.update(cx, &mut self.state, &self.items).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID).size(LayerSize::Fixed(20, 8));
                ui.layer(ID, |ui, area| {
                    picker.draw(ui, area, &self.state, &self.items)
                });
            }
        }

        let mut runtime = termrock_runtime::runtime::Runtime::new(
            FocusApp {
                state: PickerState::default(),
                items: vec![
                    Item::new(ItemKey::num(1), "first"),
                    Item::new(ItemKey::num(2), "second"),
                ],
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::runtime::stub::key(termrock_runtime::event::KeyCode::Char('s')),
        );
        let area = Rect::new(0, 0, 30, 12);
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let find_label = |label: &str| {
            (0..area.height).find_map(|y| {
                (0..area.width).find_map(|x| {
                    let matches = label.chars().enumerate().all(|(offset, ch)| {
                        buffer
                            .content
                            .get(
                                usize::from(y) * usize::from(area.width)
                                    + usize::from(x).saturating_add(offset),
                            )
                            .is_some_and(|cell| cell.symbol() == ch.to_string())
                    });
                    matches.then_some((x, y))
                })
            })
        };
        let (first_x, first_y) = find_label("first").expect("first row is visible");
        let (second_x, second_y) = find_label("second").expect("second row is visible");
        assert!(
            runtime.cursor_position().is_some(),
            "query owns a terminal cursor"
        );
        assert_ne!(
            buffer[(first_x.saturating_sub(1), first_y)].symbol(),
            "▎",
            "cursor row uses a fill instead of a second focus gutter"
        );
        assert_ne!(
            buffer[(first_x, first_y)].bg,
            buffer[(second_x, second_y)].bg,
            "focused cursor row receives its fill"
        );
    }

    #[test]
    fn tab_and_backtab_advance_and_reverse_picker_scope() {
        const ID: Id = Id::root("picker.scope-navigation");
        const SCOPES: &[ScopeKey] = &[ScopeKey::new(1), ScopeKey::new(2)];

        #[derive(Default)]
        struct ScopeApp {
            state: PickerState,
        }

        impl termrock_runtime::runtime::App for ScopeApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID).scopes(SCOPES);
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &[]));
                }
                picker.update(cx, &mut self.state, &[]).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID).scopes(SCOPES);
                ui.layer(ID, |ui, area| picker.draw(ui, area, &self.state, &[]));
            }
        }

        let mut runtime =
            termrock_runtime::runtime::Runtime::new(ScopeApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let area = Rect::new(0, 0, 80, 24);
        let mut buffer = ratatui_core::buffer::Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::runtime::stub::key(termrock_runtime::event::KeyCode::Tab),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.scope(SCOPES), Some(SCOPES[1]));
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::runtime::stub::key(termrock_runtime::event::KeyCode::BackTab),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.scope(SCOPES), Some(SCOPES[0]));
        assert!(
            runtime.diagnostics().is_empty(),
            "{:?}",
            runtime.diagnostics()
        );
    }
}
