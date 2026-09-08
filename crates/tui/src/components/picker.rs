//! Searchable modal picker and its semantic item contract.

use core::marker::PhantomData;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::filter_list::{FilterList, FilterListAction, FilterListState, FilterPolicy};
use super::{Acc, PartStyle, SlotFn};
use crate::collection::{EmptyState, RowFn, RowUi};
use crate::id::{Id, ItemKey, Part};
use crate::layer::{Anchor, LayerSize, LayerSpec, ScreenAlign};
use crate::layout::Track;
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// Borrowed semantic data shared by picker and completion rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Item<'i> {
    /// One-cell kind glyph.
    pub glyph: &'i str,
    /// Visible and searchable label.
    pub label: &'i str,
    /// Grapheme ordinals in `label` that matched.
    pub matched: &'i [usize],
    /// Secondary description.
    pub detail: &'i str,
    /// Text inserted by completion; `None` inserts `label`.
    pub insert: Option<&'i str>,
    /// Optional trailing tag.
    pub tag: Option<&'i str>,
    /// Optional group label.
    pub group: Option<&'i str>,
    /// Whether activation is refused.
    pub disabled: bool,
    /// Stable semantic identity.
    pub key: ItemKey,
}

impl<'i> Item<'i> {
    /// A minimal enabled item. Optional columns are empty and completion inserts the label.
    pub const fn new(key: ItemKey, label: &'i str) -> Self {
        Self {
            glyph: "",
            label,
            matched: &[],
            detail: "",
            insert: None,
            tag: None,
            group: None,
            disabled: false,
            key,
        }
    }
    /// Set the completion insertion text independently of the label.
    #[must_use]
    pub const fn insert(mut self, text: &'i str) -> Self {
        self.insert = Some(text);
        self
    }
    /// Set the glyph.
    #[must_use]
    pub const fn glyph(mut self, glyph: &'i str) -> Self {
        self.glyph = glyph;
        self
    }
    /// Set matched grapheme ordinals.
    #[must_use]
    pub const fn matched(mut self, matched: &'i [usize]) -> Self {
        self.matched = matched;
        self
    }
    /// Set detail.
    #[must_use]
    pub const fn detail(mut self, detail: &'i str) -> Self {
        self.detail = detail;
        self
    }
    /// Set a tag.
    #[must_use]
    pub const fn tag(mut self, tag: &'i str) -> Self {
        self.tag = Some(tag);
        self
    }
    /// Set a group.
    #[must_use]
    pub const fn group(mut self, group: &'i str) -> Self {
        self.group = Some(group);
        self
    }
    /// Disable activation.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Completion insertion text, falling back to the visible label.
    pub const fn insertion(self) -> &'i str {
        match self.insert {
            Some(text) => text,
            None => self.label,
        }
    }
}

/// Convert a domain item to the borrowed semantic picker/completion view.
///
/// A custom row painter does not replace this semantic contract:
///
/// ```compile_fail
/// use junie_tui::{AsItem, FilterList, Id};
/// struct PaintedOnly;
/// fn requires_semantics<T: AsItem>(_: &FilterList<'_, T>) {}
/// let list = FilterList::<PaintedOnly>::new(Id::root("painted-only"));
/// requires_semantics(&list);
/// ```
pub trait AsItem {
    /// Borrow this value as an item.
    fn as_item(&self) -> Item<'_>;
}

impl AsItem for Item<'_> {
    fn as_item(&self) -> Item<'_> {
        *self
    }
}

/// Built-in semantic item layout. Custom row callbacks remain authoritative.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ItemRowLayout {
    /// Existing compact label with trailing glyph and metadata.
    #[default]
    Compact,
    /// Leading glyph, aligned label/detail/tag/group columns and matched emphasis.
    Columns,
}

/// One measurement across the complete semantic projection, not only visible rows.
#[derive(Default)]
pub(crate) struct ItemColumns {
    label: u16,
    tag: u16,
    group: u16,
}

impl ItemColumns {
    pub(crate) fn measure<'a>(items: impl Iterator<Item = Item<'a>>, row_width: u16) -> Self {
        let mut columns = Self::default();
        for item in items {
            columns.label = columns.label.max(width(item.label));
            columns.tag = columns.tag.max(width(item.tag.unwrap_or("")));
            columns.group = columns.group.max(width(item.group.unwrap_or("")));
        }
        let limit = u16::try_from(u32::from(row_width).saturating_mul(45) / 100)
            .unwrap_or(u16::MAX)
            .max(6);
        columns.label = columns.label.clamp(6, limit);
        columns
    }

    pub(crate) fn paint(&self, item: Item<'_>, show_group: bool, row: &mut RowUi<'_>) {
        let focused = row.flags().contains(StateFlags::FOCUSED);
        row.gutter();
        let mut cells = row.columns_with_gap(
            &[
                Track::Fixed(1),
                Track::Fixed(1),
                Track::Fixed(self.label),
                Track::Fixed(2),
                Track::Flex(1),
                Track::Fixed(1),
                Track::Fixed(self.tag),
                Track::Fixed(if self.tag > 0 { 2 } else { 0 }),
                Track::Fixed(self.group),
                Track::Fixed(u16::from(self.group > 0)),
            ],
            0,
        );
        // A focused list fills every row from the CONTAINER recipe, whose
        // bold bleeds into painted text. Strip it only where the resolved
        // part style did not author bold, so theme, overlay and instance
        // part styles stay authoritative. Color hierarchy also stays in the
        // theme recipes: a built-in painter never tones over the resolved
        // part style.
        {
            let mut icon = cells.cell_part(0, Part::ICON);
            if !icon.authored_modifiers().contains(Modifier::BOLD) {
                icon.remove_modifier(Modifier::BOLD);
            }
            icon.text(item.glyph);
        }
        {
            let mut label = cells.cell_part(2, Part::LABEL);
            if !focused && !label.authored_modifiers().contains(Modifier::BOLD) {
                label.remove_modifier(Modifier::BOLD);
            }
            label.text_matched(item.label, item.matched);
        }
        if cells.rect(4).width >= 4 {
            let mut meta = cells.cell_part(4, Part::META);
            if !meta.authored_modifiers().contains(Modifier::BOLD) {
                meta.remove_modifier(Modifier::BOLD);
            }
            meta.text_matched(item.detail, &[]);
        }
        {
            let mut tag = cells.cell_part(6, Part::META);
            if !tag.authored_modifiers().contains(Modifier::BOLD) {
                tag.remove_modifier(Modifier::BOLD);
            }
            tag.text(item.tag.unwrap_or(""));
        }
        if show_group {
            let mut group = cells.cell_part(8, Part::META);
            if !group.authored_modifiers().contains(Modifier::BOLD) {
                group.remove_modifier(Modifier::BOLD);
            }
            group.text(item.group.unwrap_or(""));
        }
    }
}

/// Default semantic row painter.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ItemRow;

impl<T: AsItem> RowFn<T> for ItemRow {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        let item = value.as_item();
        row.gutter();
        if let Some(tag) = item.tag {
            row.meta(tag);
        }
        if !item.detail.is_empty() {
            row.meta(item.detail);
        }
        if !item.glyph.is_empty() {
            row.part(Part::ICON, 1).text(item.glyph);
        }
        row.label(item.label);
    }
}

struct BorrowedRow<'a, R>(&'a R);

impl<T, R: RowFn<T>> RowFn<T> for BorrowedRow<'_, R> {
    fn row(&self, value: &T, row: &mut RowUi<'_>) {
        self.0.row(value, row);
    }
}

/// Typed picker scope.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ScopeKey(u16);

impl ScopeKey {
    /// Construct from an application-local numeric key.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
    /// Raw value.
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Picker events.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PickerAction {
    /// Normal activation.
    Chosen(ItemKey),
    /// Alt activation.
    ChosenAlt(ItemKey),
    /// Secondary row action.
    Secondary(ItemKey),
    /// Rewind one owner-defined level.
    Back,
    /// Active scope changed.
    Scope(ScopeKey),
    /// Query text changed.
    QueryChanged,
}

/// Durable picker state.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PickerState {
    list: FilterListState,
    active_scope: usize,
}

impl PickerState {
    /// Current query.
    pub fn query(&self) -> &str {
        self.list.query()
    }
    /// Replace the query.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.list.set_query(query);
    }
    /// Current cursor key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.list.cursor()
    }
    /// Set the initial cursor before the first draw.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.list.set_cursor(index, key);
    }
    /// Current scope.
    pub fn scope(&self, scopes: &[ScopeKey]) -> Option<ScopeKey> {
        scopes.get(self.active_scope).copied()
    }
    /// Filtered-list state for controller compositions.
    pub const fn list(&self) -> &FilterListState {
        &self.list
    }
}

/// A modal semantic picker.
///
/// ## Construction
/// `Picker::new(id)`; items arrive per phase and must implement [`AsItem`].
///
/// ## Ownership
/// Caller owns items and [`PickerState`]; runtime owns the modal layer and focus trap.
///
/// ## Configuration
/// `.title`, `.placeholder`, `.scopes`, `.align`, `.empty`, `.row`, `.patch`, `.patch_part`, `.slot`.
///
/// ## Variants
/// `Family::PICKER`, `DEFAULT`.
///
/// ## States
/// Query editor and cursor state are derived by the embedded [`FilterList`].
///
/// ## Actions
/// [`PickerAction`] always carries semantic keys or scopes.
///
/// ## Focus
/// Modal trap; its filter list is the initial focus and swallows typing.
///
/// ## Keyboard
/// Typing filters; Esc clears then dismisses; Enter/Alt+Enter choose; Tab cycles scope.
///
/// ## Mouse
/// Click chooses; secondary click matches the keyboard secondary command.
///
/// ## Layout
/// Upper-third modal, design-clamped width, query plus bounded result rows.
///
/// ## Parts
/// `CONTAINER`, `BORDER`, `TITLE`, `QUERY`, row parts, scrollbar parts, `EMPTY`.
///
/// ## Overrides
/// Standard patch/part/slot overrides are forwarded into the embedded list.
///
/// ## Identity
/// [`AsItem`] is mandatory; `Display` and positional keys are never consulted.
///
/// ## Testing
/// `PickerCase`; semantic key, query, and wheel/cursor contracts are covered.
///
/// ## Invariants
/// The component reasserts its own layer size every update and computes no screen rect.
pub struct Picker<'a, T, R = ItemRow> {
    id: Id,
    title: &'a str,
    width: Option<u16>,
    searchable: bool,
    filter: FilterPolicy,
    placeholder: &'a str,
    scopes: &'a [ScopeKey],
    align: Option<ScreenAlign>,
    empty: Option<EmptyState<'a>>,
    row: R,
    item_layout: ItemRowLayout,
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
            .finish_non_exhaustive()
    }
}

impl<T> Picker<'_, T, ItemRow> {
    /// Construct a semantic picker.
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            title: "Choose",
            width: None,
            searchable: true,
            filter: FilterPolicy::Label,
            placeholder: "Type to search…",
            scopes: &[],
            align: None,
            empty: None,
            row: ItemRow,
            item_layout: ItemRowLayout::Compact,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _item: PhantomData,
        }
    }
}

impl<T> Picker<'_, T, ItemRow> {
    /// Opt into aligned semantic columns without replacing the row painter.
    #[must_use]
    pub const fn item_layout(mut self, layout: ItemRowLayout) -> Self {
        self.item_layout = layout;
        self
    }
}

impl<'a, T, R> Picker<'a, T, R> {
    /// Styled parts.
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
    /// Component id.
    pub const fn id(&self) -> Id {
        self.id
    }
    /// Title.
    #[must_use]
    pub const fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    /// Override the requested width; the layer resolver still clamps to the viewport.
    /// Omit this option to retain semantic sizing within the theme's popup bounds.
    #[must_use]
    pub const fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
    /// Select caller-owned filtering while retaining query editing and navigation.
    #[must_use]
    pub const fn filter(mut self, policy: FilterPolicy) -> Self {
        self.filter = policy;
        self
    }
    /// Whether the picker displays and accepts query input. Enabled by default.
    /// Disabling clears any existing query on update or reconciliation. Caller-filtered
    /// owners must refresh their projection when that transition reports a query change.
    #[must_use]
    pub const fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }
    /// Query placeholder.
    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
    /// Available typed scopes.
    #[must_use]
    pub const fn scopes(mut self, scopes: &'a [ScopeKey]) -> Self {
        self.scopes = scopes;
        self
    }
    /// Choose the screen placement for this modal.
    #[must_use]
    pub const fn align(mut self, align: ScreenAlign) -> Self {
        self.align = Some(align);
        self
    }
    /// Empty/loading/error presentation.
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    /// Replace row painting.
    pub fn row<R2: RowFn<T>>(self, row: R2) -> Picker<'a, T, R2> {
        Picker {
            id: self.id,
            title: self.title,
            width: self.width,
            searchable: self.searchable,
            filter: self.filter,
            placeholder: self.placeholder,
            scopes: self.scopes,
            align: self.align,
            empty: self.empty,
            row,
            item_layout: ItemRowLayout::Compact,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }
    /// Patch every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }
    /// Patch selected parts.
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self.ov = self.ov.part(parts);
        self
    }
    /// Replace one part.
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
            .filter(self.filter)
            .row(BorrowedRow(&self.row))
            .with_item_layout(self.item_layout);
        if let Some(empty) = self.empty {
            list = list.empty(empty);
        }
        if let Some(patch) = self.patch {
            list = list.patch(patch);
        }
        list = list.patch_part(self.parts);
        list
    }

    /// Requested modal size, pure in props, semantic labels, and design tokens.
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let d = cx.design();
        let width = self.width.unwrap_or_else(|| {
            FilterList::<T, BorrowedRow<'_, R>>::semantic_width(items)
                .clamp(d.size.popup_min_width, d.size.popup_max_width)
        });
        let rows = items
            .len()
            .min(usize::from(d.size.popup_max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        // Searchable pickers reserve title, query and spacing before the list.
        // Nonsearchable pickers move the list above that query space and retain
        // the bottom breathing room. Both request the same outer height.
        LayerSize::Fixed(width, rows.saturating_add(5))
    }

    /// Layer specification supplied by this picker.
    pub fn layer(&self, cx: &Cx<'_>, items: &[T]) -> LayerSpec {
        LayerSpec::modal(self.id)
            .anchor(Anchor::Screen(
                self.align.unwrap_or(ScreenAlign::UpperThird),
            ))
            .initial_focus(self.id)
            .size(self.measured_size(cx, items))
    }

    /// Reconcile a replaced projection without consuming another input update.
    /// Call after handling query or scope changes and before drawing new items.
    /// Returns whether disabling search cleared the query. When true, caller-filtered
    /// owners must rebuild their items for the empty query and reconcile again.
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

    /// Update the embedded filter and map its actions to picker semantics.
    /// When disabling search clears a query, stale row activation is suppressed.
    /// Back and scope navigation take precedence over `QueryChanged`; their owners
    /// rebuild using the already-empty query. Cancellation still dismisses the layer.
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
            // A caller-filtered projection still describes the old query until its
            // owner handles this signal. Never activate a row from that projection.
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
                FilterListAction::Moved | FilterListAction::NextScope => {
                    acc.repaint();
                }
            }
        }
        acc.finish(self.id)
    }

    /// Draw into the resolved modal area supplied by the owner's layer closure.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &PickerState, items: &[T]) -> Rect {
        ui.with_surface(crate::theme::Surface::Overlay, |ui| {
            let mut live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
            live.remove(StateFlags::PRESSED);
            let base = self.ov.style(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                Part::CONTAINER,
                live,
            );
            ui.fill(area, base.style);
            let border = self.ov.style(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                Part::BORDER,
                live,
            );
            let inner = ui.frame(area, border.style);
            if inner.is_empty() {
                return area;
            }
            let title = Rect { height: 1, ..inner };
            let title_style = self.ov.style(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                Part::TITLE,
                live,
            );
            ui.paint_str(title, self.title, title_style.style);
            if self.searchable {
                let query = Rect {
                    y: inner.y.saturating_add(1),
                    height: 1,
                    ..inner
                };
                let query_style = self.ov.style(
                    ui,
                    self.id,
                    Family::PICKER,
                    Variant::DEFAULT,
                    Part::QUERY,
                    live | StateFlags::EDITING,
                );
                ui.fill(query, query_style.style);
                let text = if st.query().is_empty() {
                    self.placeholder
                } else {
                    st.query()
                };
                ui.paint_str(
                    Rect {
                        x: query.x.saturating_add(2),
                        width: query.width.saturating_sub(2),
                        ..query
                    },
                    text,
                    query_style.style,
                );
                ui.set_cursor(
                    self.id,
                    Position::new(
                        query
                            .x
                            .saturating_add(2)
                            .saturating_add(crate::text::width(st.query())),
                        query.y,
                    ),
                );
            }
            let list = Rect {
                y: inner.y.saturating_add(if self.searchable { 3 } else { 1 }),
                height: inner
                    .height
                    .saturating_sub(if self.searchable { 3 } else { 2 }),
                ..inner
            };
            self.list().draw(ui, list, &st.list, items);
            area
        })
    }
}

/// Command palette is the picker surface over semantic actions.
pub type CommandPalette<'a, T, R = ItemRow> = Picker<'a, T, R>;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn query_change_emits_query_changed() {
        let mut state = FilterListState::default();
        let action = state.push_query_char('a');
        assert_eq!(action, FilterListAction::QueryChanged);
        assert_eq!(state.query(), "a");
    }
}
