//! Searchable modal picker and its semantic item contract.

use core::marker::PhantomData;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;

use super::filter_list::{FilterList, FilterListAction, FilterListState, FilterPolicy};
use super::{Acc, PartStyle, SlotFn, overlay_chrome};
use crate::collection::{EmptyState, RowFn, RowUi};
use crate::id::{Id, ItemKey, Part};
use crate::layer::{Anchor, LayerSize, LayerSpec, ScreenAlign};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, FgStep, GlyphRole, Role, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};

pub use termrock_navigation::{AsItem, Item, ItemColumns, ItemRow, ItemRowLayout};

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
/// `.title`, `.width`, `.size`, `.searchable`, `.filter`, `.item_layout`,
/// `.placeholder`, `.scopes`, `.align`, `.empty`, `.row`, `.patch`, `.patch_part`, `.slot`.
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
    meta: Option<&'a str>,
    width: Option<u16>,
    searchable: bool,
    filter: FilterPolicy,
    placeholder: &'a str,
    scopes: &'a [ScopeKey],
    requested_size: Option<LayerSize>,
    align: Option<ScreenAlign>,
    empty: Option<EmptyState<'a>>,
    row: R,
    item_layout: ItemRowLayout,
    footer: Option<&'a str>,
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
            .field("meta", &self.meta)
            .field("scopes", &self.scopes)
            .field("searchable", &self.searchable)
            .field("requested_size", &self.requested_size)
            .field("align", &self.align)
            .finish_non_exhaustive()
    }
}

impl<T> Picker<'_, T, ItemRow> {
    /// Construct a semantic picker.
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            title: "Choose",
            meta: None,
            width: None,
            searchable: true,
            filter: FilterPolicy::Label,
            placeholder: "Type to search…",
            scopes: &[],
            requested_size: None,
            align: None,
            empty: None,
            row: ItemRow,
            item_layout: ItemRowLayout::Compact,
            footer: None,
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
    /// Optional metadata text displayed on the right side of the title row.
    #[must_use]
    pub const fn meta(mut self, meta: &'a str) -> Self {
        self.meta = Some(meta);
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
    /// Request an explicit modal size. The layer resolver still clamps to the
    /// viewport; omit this option to retain semantic sizing.
    #[must_use]
    pub const fn size(mut self, size: LayerSize) -> Self {
        self.requested_size = Some(size);
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
            meta: self.meta,
            width: self.width,
            searchable: self.searchable,
            filter: self.filter,
            placeholder: self.placeholder,
            scopes: self.scopes,
            requested_size: self.requested_size,
            align: self.align,
            empty: self.empty,
            row,
            item_layout: self.item_layout,
            footer: self.footer,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }
    /// Optional footer text displayed on the bottom row of the picker.
    #[must_use]
    pub const fn footer(mut self, footer: &'a str) -> Self {
        self.footer = Some(footer);
        self
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
        if !self.parts.is_empty() {
            list = list.patch_part(self.parts);
        }
        list
    }

    /// Requested modal size, pure in props, semantic labels, and design tokens.
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        if let Some(size) = self.requested_size {
            return size;
        }
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
        let mut live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        live.remove(StateFlags::PRESSED);
        let container_patch = self
            .ov
            .part_patch(Part::CONTAINER)
            .unwrap_or_else(|| StylePatch::new().clear_fg().set_bg(Role::CurrentSurface));
        let border_patch = self
            .ov
            .part_patch(Part::BORDER)
            .unwrap_or_else(|| StylePatch::new().set_fg(Role::BorderStrong));
        let chrome_patches = [
            (Part::CONTAINER, container_patch),
            (Part::BORDER, border_patch),
        ];
        let chrome_ov = self.ov.part(&chrome_patches);
        overlay_chrome(
            ui,
            self.id,
            area,
            Family::PICKER,
            Surface::Elevated,
            chrome_ov,
            live,
            live,
            |ui, inner| {
                if inner.is_empty() {
                    return area;
                }
                // An explicitly requested size insets the content lane inside
                // the frame instead of letting the list touch the border.
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
                let title_style = self
                    .ov
                    .part_patch(Part::TITLE)
                    .map(|p| {
                        ui.style_patched(Family::PICKER, Variant::DEFAULT, Part::TITLE, live, &p)
                    })
                    .unwrap_or_else(|| {
                        ui.style_patched(
                            Family::PICKER,
                            Variant::DEFAULT,
                            Part::TITLE,
                            live,
                            &StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Primary))
                                .add(Modifier::BOLD),
                        )
                    });
                self.ov.note(
                    ui,
                    self.id,
                    Family::PICKER,
                    Variant::DEFAULT,
                    Part::TITLE,
                    title_style,
                );
                ui.paint_str(title, self.title, title_style.style);
                if let Some(meta) = self.meta {
                    let meta_w = width(meta);
                    let meta_style = self
                        .ov
                        .part_patch(Part::META)
                        .map(|p| {
                            ui.style_patched(Family::PICKER, Variant::DEFAULT, Part::META, live, &p)
                        })
                        .unwrap_or_else(|| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::META,
                                live,
                                &StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
                            )
                        });
                    self.ov.note(
                        ui,
                        self.id,
                        Family::PICKER,
                        Variant::DEFAULT,
                        Part::META,
                        meta_style,
                    );
                    ui.paint_str(
                        Rect {
                            x: content.right().saturating_sub(meta_w),
                            y: content.y,
                            width: meta_w,
                            height: 1,
                        },
                        meta,
                        meta_style.style,
                    );
                }
                if self.searchable {
                    let query = Rect {
                        y: content.y.saturating_add(1),
                        height: 1,
                        ..content
                    };
                    let query_fill = self
                        .ov
                        .part_patch(Part::QUERY)
                        .map(|p| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::QUERY,
                                live,
                                &p,
                            )
                        })
                        .unwrap_or_else(|| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::QUERY,
                                live,
                                &StylePatch::new()
                                    .set_fg(Role::Fg(FgStep::Primary))
                                    .set_bg(Role::Surface(Surface::Field)),
                            )
                        });
                    ui.fill(query, query_fill.style);
                    let bar_style = self
                        .ov
                        .part_patch(Part::GUTTER)
                        .map(|p| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::GUTTER,
                                live | StateFlags::FOCUSED,
                                &p,
                            )
                        })
                        .unwrap_or_else(|| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::GUTTER,
                                live | StateFlags::FOCUSED,
                                &StylePatch::new()
                                    .set_fg(Role::Focus)
                                    .set_bg(Role::Surface(Surface::Field)),
                            )
                        });
                    ui.glyph(
                        Rect { width: 1, ..query },
                        GlyphRole::FocusBar,
                        bar_style.style,
                    );
                    let text = if st.query().is_empty() {
                        self.placeholder
                    } else {
                        st.query()
                    };
                    let text_style = if st.query().is_empty() {
                        self.ov
                            .part_patch(Part::HELP)
                            .map(|p| {
                                ui.style_patched(
                                    Family::PICKER,
                                    Variant::DEFAULT,
                                    Part::HELP,
                                    live,
                                    &p,
                                )
                            })
                            .unwrap_or_else(|| {
                                ui.style_patched(
                                    Family::PICKER,
                                    Variant::DEFAULT,
                                    Part::HELP,
                                    live,
                                    &StylePatch::new()
                                        .set_fg(Role::Fg(FgStep::Muted))
                                        .set_bg(Role::Surface(Surface::Field)),
                                )
                            })
                    } else {
                        self.ov
                            .part_patch(Part::QUERY)
                            .map(|p| {
                                ui.style_patched(
                                    Family::PICKER,
                                    Variant::DEFAULT,
                                    Part::QUERY,
                                    live | StateFlags::EDITING,
                                    &p,
                                )
                            })
                            .unwrap_or_else(|| {
                                ui.style_patched(
                                    Family::PICKER,
                                    Variant::DEFAULT,
                                    Part::QUERY,
                                    live | StateFlags::EDITING,
                                    &StylePatch::new()
                                        .set_fg(Role::Fg(FgStep::Primary))
                                        .set_bg(Role::Surface(Surface::Field))
                                        .add(Modifier::UNDERLINED)
                                        .set_underline(Role::Accent),
                                )
                            })
                    };
                    ui.paint_str(
                        Rect {
                            x: query.x.saturating_add(2),
                            width: query.width.saturating_sub(2),
                            ..query
                        },
                        text,
                        text_style.style,
                    );
                    ui.set_cursor(
                        self.id,
                        Position::new(
                            query.x.saturating_add(2).saturating_add(width(st.query())),
                            query.y,
                        ),
                    );
                }
                // Searchable pickers reserve title, query and spacing before
                // the list; nonsearchable pickers retain the bottom breathing
                // room so the drawn rows match the requested outer height.
                let list_offset = if self.searchable { 3 } else { 1 };
                let footer_reserve = 1;
                let list = Rect {
                    y: content.y.saturating_add(list_offset),
                    height: content
                        .height
                        .saturating_sub(list_offset)
                        .saturating_sub(footer_reserve),
                    ..content
                };
                self.list().draw(ui, list, &st.list, items);
                if let Some(footer) = self.footer {
                    let footer_style = self
                        .ov
                        .part_patch(Part::HELP)
                        .map(|p| {
                            ui.style_patched(Family::PICKER, Variant::DEFAULT, Part::HELP, live, &p)
                        })
                        .unwrap_or_else(|| {
                            ui.style_patched(
                                Family::PICKER,
                                Variant::DEFAULT,
                                Part::HELP,
                                live,
                                &StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
                            )
                        });
                    self.ov.note(
                        ui,
                        self.id,
                        Family::PICKER,
                        Variant::DEFAULT,
                        Part::HELP,
                        footer_style,
                    );
                    let footer_text = termrock_text::truncate(footer, content.width);
                    ui.paint_str(
                        Rect {
                            y: content.bottom().saturating_sub(1),
                            height: 1,
                            ..content
                        },
                        &footer_text,
                        footer_style.style,
                    );
                }
                area
            },
        )
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
