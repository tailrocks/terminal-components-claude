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
    hints: Option<&'a str>,
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
            hints: None,
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
    /// Shows application-provided keyboard hints on the picker's bottom row.
    #[must_use]
    pub const fn hints(mut self, hints: &'a str) -> Self {
        self.hints = Some(hints);
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
            hints: self.hints,
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
            return size;
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

        // Match the picker's content contract: border + title + searchable
        // query/gap + rows + two trailing content cells. Non-searchable
        // pickers omit the query and its gap.
        let chrome_rows = if self.searchable { 7 } else { 5 };
        LayerSize::Fixed(width, rows.saturating_add(chrome_rows))
    }

    pub fn layer(&self, cx: &Cx<'_>, items: &[T]) -> LayerSpec {
        // The frozen Picker defaults to geometric centering. An explicit
        // alignment is passed through unchanged for runtime-level control.
        let align = self.align.unwrap_or(ScreenAlign::GeometricCenter);
        LayerSpec::modal(self.id)
            .anchor(Anchor::Screen(align))
            .initial_focus(self.id)
            .viewport_margin(4, 2)
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

                // Preserve the picker's second horizontal content inset in
                // addition to the shared frame border.
                let content = Rect {
                    x: inner.x.saturating_add(1),
                    width: inner.width.saturating_sub(2),
                    ..inner
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
                        .saturating_sub(if self.searchable { 4 } else { 2 }),
                    ..content
                };
                self.list().draw(ui, list, &st.list, items);
                if let Some(hints) = self.hints.filter(|hints| !hints.is_empty()) {
                    let hint_style = self.ov.style(
                        ui,
                        self.id,
                        Family::PICKER,
                        Variant::DEFAULT,
                        Part::META,
                        live,
                    );
                    let hint = Rect {
                        y: content.bottom().saturating_sub(1),
                        height: 1,
                        ..content
                    };
                    ui.paint_str(
                        hint,
                        crate::text::truncate(hints, hint.width).as_str(),
                        hint_style.style,
                    );
                }
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
        let query_cell = plain
            .content
            .iter()
            .position(|cell| cell.symbol() == "X")
            .expect("query placeholder is visible");
        let query_x = u16::try_from(query_cell % 30).expect("screen x fits u16");
        let query_y = u16::try_from(query_cell / 30).expect("screen y fits u16");
        assert_ne!(
            plain[(query_x, query_y)].fg,
            base_theme[(query_x, query_y)].fg
        );
        assert_ne!(
            base_theme[(query_x, query_y)].fg,
            themed[(query_x, query_y)].fg
        );
        assert_ne!(
            themed[(query_x, query_y)].fg,
            patched[(query_x, query_y)].fg
        );
        assert_ne!(
            plain[(query_x, query_y)].bg,
            globally_patched[(query_x, query_y)].bg
        );
    }

    fn measured_at(size: LayerSize, viewport_width: u16, viewport_height: u16) -> LayerSize {
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
        let area = Rect::new(0, 0, viewport_width, viewport_height);
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
    fn requested_size_remains_intrinsic_and_the_layer_resolver_applies_viewport_margins() {
        let requested = LayerSize::Fixed(84, 9);
        assert_eq!(measured_at(requested, 72, 40), requested);
        assert_eq!(measured_at(requested, 80, 40), requested);
        assert_eq!(measured_at(requested, 100, 40), requested);
        assert_eq!(measured_at(requested, 0, 40), requested);
        assert_eq!(measured_at(requested, 4, 40), requested);
        assert_eq!(measured_at(requested, 1, 40), requested);
        assert_eq!(measured_at(LayerSize::Fill, 72, 40), LayerSize::Fill);
        assert_eq!(measured_at(requested, 120, 12), requested);
        assert_eq!(measured_at(requested, 120, 2), requested);
        assert_eq!(measured_at(requested, 120, 1), requested);
        assert_eq!(
            measured_at(LayerSize::Fixed(84, 30), 120, 12),
            LayerSize::Fixed(84, 30)
        );
        assert_eq!(measured_at(requested, 120, 0), requested);
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
        let measure = |popup_max_rows, max_rows, viewport_height| {
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
            let area = Rect::new(0, 0, 120, viewport_height);
            let mut buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            let _ = termrock_runtime::runtime::stub::deliver(
                &mut runtime,
                termrock_runtime::event::Input::Tick,
            );
            runtime.app().measured.expect("measured_size ran")
        };
        assert_eq!(measure(5, None, 40), LayerSize::Fixed(12, 12));
        assert_eq!(measure(5, Some(3), 40), LayerSize::Fixed(12, 10));
        assert_eq!(measure(12, None, 12), LayerSize::Fixed(12, 19));
        assert_eq!(measure(12, None, 2), LayerSize::Fixed(12, 19));
        assert_eq!(measure(12, None, 1), LayerSize::Fixed(12, 19));
        assert_eq!(measure(12, None, 0), LayerSize::Fixed(12, 19));
    }

    #[test]
    fn picker_first_draw_applies_viewport_margin_before_any_input() {
        const ID: Id = Id::root("picker.first-draw-short-viewport");
        struct FirstDrawApp {
            state: PickerState,
            items: Vec<Item<'static>>,
        }
        impl termrock_runtime::runtime::App for FirstDrawApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID).width(64).max_rows(12);
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &self.items));
                }
                picker.update(cx, &mut self.state, &self.items).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID).width(64).max_rows(12);
                ui.layer(ID, |ui, area| {
                    picker.draw(ui, area, &self.state, &self.items)
                });
            }
        }

        let mut runtime = termrock_runtime::runtime::Runtime::new(
            FirstDrawApp {
                state: PickerState::default(),
                items: (0..12)
                    .map(|index| Item::new(ItemKey::num(index), "row"))
                    .collect(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let screen = Rect::new(0, 0, 120, 12);
        let mut buffer = ratatui_core::buffer::Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut buffer).commit_presented();
        assert_eq!(
            runtime.layer_area(ID),
            Some(Rect::new(28, 1, 64, 10)),
            "the bootstrap layer is constrained by the terminal viewport before its first frame"
        );
    }

    #[test]
    fn searchable_picker_measures_and_centers_its_complete_content_area() {
        const ID: Id = Id::root("picker.centered-content-geometry");

        struct PickerApp {
            state: PickerState,
            items: Vec<Item<'static>>,
        }

        impl termrock_runtime::runtime::App for PickerApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Open tabs")
                    .width(64)
                    .hints("Enter Open  Esc Cancel");
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &self.items));
                }
                picker.update(cx, &mut self.state, &self.items).erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Open tabs")
                    .width(64)
                    .hints("Enter Open  Esc Cancel");
                ui.layer(ID, |ui, area| {
                    picker.draw(ui, area, &self.state, &self.items)
                });
            }
        }

        let mut runtime = termrock_runtime::runtime::Runtime::new(
            PickerApp {
                state: PickerState::default(),
                items: vec![Item::new(ItemKey::num(1), "orders")],
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();

        let render = |runtime: &mut termrock_runtime::runtime::Runtime<PickerApp>, area| {
            let mut buffer = ratatui_core::buffer::Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            buffer
        };

        let screen = Rect::new(0, 0, 120, 40);
        let frame = render(&mut runtime, screen);
        assert_eq!(
            runtime.layer_area(ID),
            Some(Rect::new(28, 16, 64, 8)),
            "center uses the full resolved viewport and includes the frozen searchable height"
        );
        assert_eq!(
            frame[(30, 17)].symbol(),
            "O",
            "title starts one cell inside the border"
        );
        assert_eq!(
            frame[(30, 22)].symbol(),
            "E",
            "application-provided footer hints render on the reserved bottom content row"
        );
        let row_origin = (0..screen.height).find_map(|y| {
            (0..screen.width).find_map(|x| {
                let matches = "orders".chars().enumerate().all(|(offset, ch)| {
                    frame
                        .content
                        .get(usize::from(y) * usize::from(screen.width) + usize::from(x) + offset)
                        .is_some_and(|cell| cell.symbol() == ch.to_string())
                });
                matches.then_some((x, y))
            })
        });
        assert_eq!(
            row_origin.map(|(_, y)| y),
            Some(20),
            "the row follows the query and one-row gap"
        );

        let resized = Rect::new(0, 0, 80, 24);
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::event::Input::Resize(resized.width, resized.height),
        );
        let _ = render(&mut runtime, resized);
        assert_eq!(runtime.layer_area(ID), Some(Rect::new(8, 8, 64, 8)));

        let narrow = Rect::new(0, 0, 40, 12);
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::event::Input::Resize(narrow.width, narrow.height),
        );
        let _ = render(&mut runtime, narrow);
        assert_eq!(runtime.layer_area(ID), Some(Rect::new(2, 2, 36, 8)));

        let empty = Rect::new(0, 0, 120, 0);
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            termrock_runtime::event::Input::Resize(empty.width, empty.height),
        );
        let _ = render(&mut runtime, empty);
        assert!(runtime.layer_area(ID).is_some_and(|area| area.is_empty()));
    }

    #[test]
    fn picker_height_tracks_visible_capacity_policy_for_empty_and_many_items() {
        struct MeasureApp {
            items: Vec<Item<'static>>,
            searchable: bool,
            measured: Option<LayerSize>,
        }
        impl termrock_runtime::runtime::App for MeasureApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                self.measured = Some(
                    Picker::<Item<'static>>::new(Id::root("picker.height-policy"))
                        .width(20)
                        .max_rows(12)
                        .searchable(self.searchable)
                        .measured_size(cx, &self.items),
                );
                Response::ignored()
            }
            fn draw(&self, _ui: &mut Ui<'_>) {}
        }

        let measure = |items, searchable| {
            let mut runtime = termrock_runtime::runtime::Runtime::new(
                MeasureApp {
                    items,
                    searchable,
                    measured: None,
                },
                Theme::junie(),
            );
            let _ = runtime.initialize();
            let screen = Rect::new(0, 0, 120, 40);
            let mut buffer = ratatui_core::buffer::Buffer::empty(screen);
            runtime.draw_buffer(screen, &mut buffer).commit_presented();
            let _ = termrock_runtime::runtime::stub::deliver(
                &mut runtime,
                termrock_runtime::event::Input::Tick,
            );
            runtime.app().measured.expect("measurement ran")
        };

        assert_eq!(measure(vec![], true), LayerSize::Fixed(20, 8));
        assert_eq!(
            measure(
                (0..30)
                    .map(|index| Item::new(ItemKey::num(index), "row"))
                    .collect(),
                true
            ),
            LayerSize::Fixed(20, 19)
        );
        assert_eq!(measure(vec![], false), LayerSize::Fixed(20, 6));
    }

    #[test]
    fn nonsearchable_picker_reserves_its_hint_row_in_the_list_viewport() {
        const ID: Id = Id::root("picker.nonsearchable-hint-row");
        struct NonSearchApp {
            state: PickerState,
            items: Vec<Item<'static>>,
        }
        impl termrock_runtime::runtime::App for NonSearchApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Choose row")
                    .width(32)
                    .max_rows(1)
                    .searchable(false);
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &self.items));
                }
                picker.update(cx, &mut self.state, &self.items).erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Choose row")
                    .width(32)
                    .max_rows(1)
                    .searchable(false);
                ui.layer(ID, |ui, area| {
                    picker.draw(ui, area, &self.state, &self.items)
                });
            }
        }

        let mut runtime = termrock_runtime::runtime::Runtime::new(
            NonSearchApp {
                state: PickerState::default(),
                items: vec![
                    Item::new(ItemKey::num(1), "first"),
                    Item::new(ItemKey::num(2), "second"),
                    Item::new(ItemKey::num(3), "third"),
                ],
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let screen = Rect::new(0, 0, 80, 16);
        let mut frame = ratatui_core::buffer::Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut frame).commit_presented();
        assert_eq!(runtime.layer_area(ID), Some(Rect::new(24, 5, 32, 6)));
        let contains = |label: &str| {
            frame.content.windows(label.len()).any(|cells| {
                cells
                    .iter()
                    .zip(label.chars())
                    .all(|(cell, ch)| cell.symbol() == ch.to_string())
            })
        };
        assert!(contains("first"));
        assert!(contains("second"), "one trailing hint cell is reserved");
        assert!(!contains("third"));
    }

    #[test]
    fn default_picker_centers_but_explicit_screen_alignment_is_preserved() {
        const ID: Id = Id::root("picker.alignment-contract");
        struct AlignmentApp {
            anchors: Vec<Anchor>,
        }
        impl termrock_runtime::runtime::App for AlignmentApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    self.anchors
                        .push(Picker::<Item<'static>>::new(ID).layer(cx, &[]).anchor);
                    self.anchors.push(
                        Picker::<Item<'static>>::new(ID)
                            .align(ScreenAlign::Center)
                            .layer(cx, &[])
                            .anchor,
                    );
                    self.anchors.push(
                        Picker::<Item<'static>>::new(ID)
                            .align(ScreenAlign::GeometricCenter)
                            .layer(cx, &[])
                            .anchor,
                    );
                }
                Response::ignored()
            }
            fn draw(&self, _ui: &mut Ui<'_>) {}
        }

        let mut runtime = termrock_runtime::runtime::Runtime::new(
            AlignmentApp {
                anchors: Vec::new(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        assert_eq!(
            runtime.app().anchors,
            vec![
                Anchor::Screen(ScreenAlign::GeometricCenter),
                Anchor::Screen(ScreenAlign::Center),
                Anchor::Screen(ScreenAlign::GeometricCenter),
            ]
        );
    }

    #[test]
    fn filtering_changes_rows_without_changing_the_picker_geometry_policy() {
        const ID: Id = Id::root("picker.filtered-geometry");
        struct FilteredApp {
            state: PickerState,
            items: Vec<Item<'static>>,
        }
        impl termrock_runtime::runtime::App for FilteredApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Open tabs")
                    .width(40)
                    .max_rows(6)
                    .align(ScreenAlign::GeometricCenter);
                picker.reconcile(&mut self.state, &self.items);
                if cx.update_cause() == termrock_runtime::runtime::UpdateCause::Bootstrap {
                    cx.open_layer(ID, picker.layer(cx, &self.items));
                }
                picker.update(cx, &mut self.state, &self.items).erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                let picker = Picker::<Item<'static>>::new(ID)
                    .title("Open tabs")
                    .width(40)
                    .max_rows(6)
                    .align(ScreenAlign::GeometricCenter);
                ui.layer(ID, |ui, area| {
                    picker.draw(ui, area, &self.state, &self.items)
                });
            }
        }

        let mut state = PickerState::default();
        state.set_query("beta");
        let mut runtime = termrock_runtime::runtime::Runtime::new(
            FilteredApp {
                state,
                items: vec![
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                    Item::new(ItemKey::num(3), "gamma"),
                ],
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let screen = Rect::new(0, 0, 120, 40);
        let mut frame = ratatui_core::buffer::Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut frame).commit_presented();

        assert_eq!(runtime.app().state.query(), "beta");
        assert_eq!(runtime.layer_area(ID), Some(Rect::new(40, 15, 40, 10)));
        let contains = |label: &str| {
            frame.content.windows(label.len()).any(|cells| {
                cells
                    .iter()
                    .zip(label.chars())
                    .all(|(cell, ch)| cell.symbol() == ch.to_string())
            })
        };
        assert!(contains("beta"), "the matching row remains rendered");
        assert!(!contains("alpha"), "nonmatching rows are filtered out");
        assert!(!contains("gamma"), "nonmatching rows are filtered out");
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
