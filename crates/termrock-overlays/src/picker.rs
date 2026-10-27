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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PickerLayout {
    legacy: bool,
    max_rows: Option<u16>,
    inner_padding_x: Option<u16>,
    query_gap: u16,
    footer_rows: Option<u16>,
    spare_rows: u16,
    viewport_margin: Option<(u16, u16)>,
}

impl PickerLayout {
    const fn compatibility() -> Self {
        Self {
            legacy: true,
            max_rows: None,
            inner_padding_x: None,
            query_gap: 1,
            footer_rows: None,
            spare_rows: 0,
            viewport_margin: None,
        }
    }

    pub const fn card() -> Self {
        Self {
            legacy: false,
            max_rows: Some(12),
            inner_padding_x: Some(1),
            query_gap: 1,
            footer_rows: Some(1),
            spare_rows: 1,
            viewport_margin: Some((4, 2)),
        }
    }

    #[must_use]
    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = Some(rows);
        self
    }

    #[must_use]
    pub const fn inner_padding_x(mut self, cells: u16) -> Self {
        self.inner_padding_x = Some(cells);
        self.legacy = false;
        self
    }

    #[must_use]
    pub const fn query_gap(mut self, rows: u16) -> Self {
        self.query_gap = rows;
        self.legacy = false;
        self
    }

    #[must_use]
    pub const fn footer_rows(mut self, rows: u16) -> Self {
        self.footer_rows = Some(rows);
        self.legacy = false;
        self
    }

    #[must_use]
    pub const fn spare_rows(mut self, rows: u16) -> Self {
        self.spare_rows = rows;
        self.legacy = false;
        self
    }

    #[must_use]
    pub const fn viewport_margin(mut self, horizontal: u16, vertical: u16) -> Self {
        self.viewport_margin = Some((horizontal, vertical));
        self.legacy = false;
        self
    }

    const fn margins(self) -> (u16, u16) {
        match self.viewport_margin {
            Some(margins) => margins,
            None => (0, 0),
        }
    }
}

impl Default for PickerLayout {
    fn default() -> Self {
        Self::compatibility()
    }
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

    tail: u16,
    selection_policy: SelectionPolicy,
    layout: PickerLayout,
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
            .field("layout", &self.layout)
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
            layout: PickerLayout::compatibility(),
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
        Part::PLACEHOLDER,
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

    #[must_use]
    pub const fn layout(mut self, layout: PickerLayout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub const fn max_rows(mut self, rows: u16) -> Self {
        self.layout = self.layout.max_rows(rows);
        self
    }

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
            layout: self.layout,
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
        let d = cx.design();
        let rows = items
            .len()
            .min(usize::from(
                self.layout.max_rows.unwrap_or(d.size.popup_max_rows),
            ))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        let viewport = cx.viewport();
        let (margin_x, margin_y) = self.layout.margins();
        if let Some(size) = self.requested_size {
            return match size {
                LayerSize::Fixed(_, _) if self.layout.legacy => {
                    clamp_fixed_width(size, viewport.width)
                }
                LayerSize::Fixed(width, height) => LayerSize::Fixed(
                    width.min(viewport.width.saturating_sub(margin_x).max(1)),
                    height.min(viewport.height.saturating_sub(margin_y).max(1)),
                ),
                LayerSize::Fill => LayerSize::Fill,
                other => other,
            };
        }
        let desired_width = self.width.unwrap_or_else(|| {
            FilterList::<T, BorrowedRow<'_, R>>::semantic_width(items)
                .clamp(d.size.popup_min_width, d.size.popup_max_width)
        });
        let width = if self.layout.legacy {
            desired_width
        } else {
            desired_width.min(viewport.width.saturating_sub(margin_x).max(1))
        };
        if self.layout.legacy {
            return LayerSize::Fixed(width, rows.saturating_add(5));
        }

        let query = if self.searchable {
            1u16.saturating_add(self.layout.query_gap)
        } else {
            0
        };
        let footer = self.layout.footer_rows.unwrap_or(0);
        let height = 2u16
            .saturating_add(1)
            .saturating_add(query)
            .saturating_add(rows)
            .saturating_add(self.layout.spare_rows)
            .saturating_add(footer);
        LayerSize::Fixed(
            width,
            height.min(viewport.height.saturating_sub(margin_y).max(1)),
        )
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
            .viewport_margin(self.layout.margins().0, self.layout.margins().1)
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
                    let count = self.scopes.len();
                    let current = st.active_scope.checked_rem(count).unwrap_or(0);
                    st.active_scope = if current == 0 {
                        count.saturating_sub(1)
                    } else {
                        current.saturating_sub(1)
                    };
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

                let padding = self
                    .layout
                    .inner_padding_x
                    .unwrap_or_else(|| if self.requested_size.is_some() { 1 } else { 0 });
                let content = Rect {
                    x: inner.x.saturating_add(padding),
                    width: inner.width.saturating_sub(padding.saturating_mul(2)),
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
                    self.list().draw_query(ui, query, &st.list);
                }

                let show_hint = !self.hints.is_empty()
                    && (self.layout.legacy || self.layout.footer_rows != Some(0));
                if show_hint {
                    let hint_area = Rect {
                        y: content.y.saturating_add(content.height.saturating_sub(1)),
                        height: 1,
                        ..content
                    };

                    let hint_style = ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Faint))
                            .set_bg(Role::CurrentSurface),
                    );

                    let line = termrock_text::truncate(self.hints, hint_area.width);
                    ui.paint_str(hint_area, &line, hint_style);
                }
                let list = Rect {
                    y: content.y.saturating_add(if self.layout.legacy {
                        if self.searchable { 3 } else { 1 }
                    } else {
                        1u16.saturating_add(if self.searchable {
                            1u16.saturating_add(self.layout.query_gap)
                        } else {
                            0
                        })
                    }),
                    height: content.height.saturating_sub(
                        if self.layout.legacy {
                            match (self.searchable, show_hint) {
                                (true, true) => 4u16,
                                (true, false) => 3,
                                (false, _) => 2,
                            }
                        } else {
                            1u16.saturating_add(if self.searchable {
                                1u16.saturating_add(self.layout.query_gap)
                            } else {
                                0
                            })
                            .saturating_add(self.layout.footer_rows.unwrap_or(u16::from(show_hint)))
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
