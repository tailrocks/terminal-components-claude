use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, shift};
use crate::action::ActionKey;
pub use crate::item::AsItem;
use crate::scroll::ScrollState;
use termrock_collections::collection::{CollectionCore, Reconcile, Reconciliation};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_fields::{TextCmd, TextInput, TextInputState};
use termrock_runtime::empty::{EmptyState, Status};
use termrock_runtime::event::{Chord, KeyCode, KeyModifiers};
use termrock_runtime::focus::Focusability;
use termrock_runtime::intent::{Intent, Phase};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;
use termrock_runtime::ui::{Cx, FrameRead, Ui};
use termrock_theme::{CursorStyle, Family, GlyphRole, Slot, StylePatch, Variant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterListAction {
    QueryChanged,
    Moved,
    Chose(ItemKey),
    ChoseAlt(ItemKey),
    Secondary(ItemKey),
    Back,
    Cancel,
    NextScope,
    PrevScope,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterListCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Choose,
    ChooseAlt,
    Secondary,
    Back,
    NextScope,
    PrevScope,
    Cancel,
}

const fn binding(
    action: ActionKey,
    chord: Chord,
    cmd: FilterListCmd,
    label: &'static str,
    visible: bool,
) -> Binding<FilterListCmd> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

const BINDINGS: &[Binding<FilterListCmd>] = &[
    binding(
        ActionKey::custom("filter-list.up"),
        Chord::key(KeyCode::Up),
        FilterListCmd::Up,
        "Up",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.down"),
        Chord::key(KeyCode::Down),
        FilterListCmd::Down,
        "Down",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.ctrl-up"),
        Chord::with(KeyCode::Char('p'), KeyModifiers::CONTROL),
        FilterListCmd::Up,
        "Up",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.ctrl-down"),
        Chord::with(KeyCode::Char('n'), KeyModifiers::CONTROL),
        FilterListCmd::Down,
        "Down",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.ctrl-k"),
        Chord::with(KeyCode::Char('k'), KeyModifiers::CONTROL),
        FilterListCmd::Up,
        "Up",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.ctrl-j"),
        Chord::with(KeyCode::Char('j'), KeyModifiers::CONTROL),
        FilterListCmd::Down,
        "Down",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.prev-scope"),
        Chord::key(KeyCode::BackTab),
        FilterListCmd::PrevScope,
        "Previous scope",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.page-up"),
        Chord::key(KeyCode::PageUp),
        FilterListCmd::PageUp,
        "Page up",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.page-down"),
        Chord::key(KeyCode::PageDown),
        FilterListCmd::PageDown,
        "Page down",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.home"),
        Chord::key(KeyCode::Home),
        FilterListCmd::Home,
        "First",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.end"),
        Chord::key(KeyCode::End),
        FilterListCmd::End,
        "Last",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.choose"),
        Chord::key(KeyCode::Enter),
        FilterListCmd::Choose,
        "Choose",
        true,
    ),
    binding(
        ActionKey::custom("filter-list.choose-alt"),
        Chord::with(KeyCode::Enter, KeyModifiers::ALT),
        FilterListCmd::ChooseAlt,
        "Choose alternate",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.secondary"),
        Chord::key(KeyCode::Delete),
        FilterListCmd::Secondary,
        "Secondary",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.back"),
        Chord::key(KeyCode::Backspace),
        FilterListCmd::Back,
        "Back",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.scope"),
        Chord::key(KeyCode::Tab),
        FilterListCmd::NextScope,
        "Next scope",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.cancel"),
        Chord::key(KeyCode::Esc),
        FilterListCmd::Cancel,
        "Cancel",
        false,
    ),
];

// The query field owns Home and End; all other list actions keep their baseline chords.
const SEARCHABLE_BINDINGS: &[Binding<FilterListCmd>] = &[
    BINDINGS[0],
    BINDINGS[1],
    BINDINGS[2],
    BINDINGS[3],
    BINDINGS[4],
    BINDINGS[5],
    BINDINGS[6],
    BINDINGS[7],
    BINDINGS[8],
    BINDINGS[11],
    BINDINGS[12],
    BINDINGS[13],
    BINDINGS[14],
    BINDINGS[15],
    BINDINGS[16],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FilterPolicy {
    #[default]
    Label,
    Caller,
    /// Case-insensitive contiguous substring matching.
    Substring,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct FilterListState {
    query_value: String,
    query_editor: TextInputState,
    core: CollectionCore,
    matches: Vec<usize>,
    initialized: bool,
}

impl FilterListState {
    pub fn query(&self) -> &str {
        self.query_editor.draft_text().unwrap_or(&self.query_value)
    }
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query_value = query.into();
        self.query_editor.cancel();
        self.query_editor.begin(&self.query_value);
        self.core.invalidate();
    }
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }
    pub const fn matched_len(&self) -> usize {
        self.matches.len()
    }
}

impl Reconcile for FilterListState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        self.core.reconcile(len, key)
    }
    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

pub struct FilterList<'a, T, R = crate::item::ItemRow> {
    id: Id,
    row: R,
    item_layout: crate::item::ItemRowLayout,
    cursor_style: CursorStyle,
    metadata_gap: u16,
    empty: Option<EmptyState<'a>>,
    status: Status,
    frame: usize,
    searchable: bool,
    register_owner: bool,
    filter: FilterPolicy,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    _item: PhantomData<fn(&T)>,
}

impl<T, R> core::fmt::Debug for FilterList<'_, T, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FilterList")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("searchable", &self.searchable)
            .finish_non_exhaustive()
    }
}

impl<T> FilterList<'_, T, crate::item::ItemRow> {
    pub const fn new(id: Id) -> Self {
        Self {
            id,
            row: crate::item::ItemRow,
            item_layout: crate::item::ItemRowLayout::Compact,
            cursor_style: CursorStyle::Bar,
            metadata_gap: 1,
            empty: None,
            status: Status::Ready,
            frame: 0,
            searchable: true,
            register_owner: true,
            filter: FilterPolicy::Label,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _item: PhantomData,
        }
    }
}

impl<T> FilterList<'_, T, crate::item::ItemRow> {
    #[must_use]
    pub const fn item_layout(mut self, layout: crate::item::ItemRowLayout) -> Self {
        self.item_layout = layout;
        self
    }
}

impl<'a, T, R> FilterList<'a, T, R> {
    pub const fn with_item_layout(mut self, layout: crate::item::ItemRowLayout) -> Self {
        self.item_layout = layout;
        self
    }

    #[must_use]
    pub const fn cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        self
    }

    #[must_use]
    pub const fn metadata_gap(mut self, gap: u16) -> Self {
        self.metadata_gap = gap;
        self
    }

    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::ROW,
        Part::LABEL,
        Part::META,
        Part::ICON,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];
    pub const fn id(&self) -> Id {
        self.id
    }
    pub fn row<R2: RowFn<T>>(self, row: R2) -> FilterList<'a, T, R2> {
        FilterList {
            id: self.id,
            row,
            item_layout: crate::item::ItemRowLayout::Compact,
            cursor_style: self.cursor_style,
            metadata_gap: self.metadata_gap,
            empty: self.empty,
            status: self.status,
            frame: self.frame,
            searchable: self.searchable,
            register_owner: self.register_owner,
            filter: self.filter,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }
    #[must_use]
    pub const fn empty(mut self, empty: EmptyState<'a>) -> Self {
        self.empty = Some(empty);
        self
    }
    #[must_use]
    pub const fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
    #[must_use]
    pub const fn frame(mut self, frame: usize) -> Self {
        self.frame = frame;
        self
    }
    #[must_use]
    pub const fn filter(mut self, policy: FilterPolicy) -> Self {
        self.filter = policy;
        self
    }
    #[must_use]
    pub const fn searchable(mut self, yes: bool) -> Self {
        self.searchable = yes;
        self
    }
    #[doc(hidden)]
    pub const fn register_owner(mut self, yes: bool) -> Self {
        self.register_owner = yes;
        self
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
    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut bar = ScrollRegion::new(self.id).patch_part(self.parts);
        if let Some(p) = self.patch {
            bar = bar.patch(p);
        }
        if let Some(slot) = self.ov.slot_for(Part::TRACK) {
            bar = bar.slot(Part::TRACK, slot);
        } else if let Some(slot) = self.ov.slot_for(Part::THUMB) {
            bar = bar.slot(Part::THUMB, slot);
        }
        bar
    }

    fn readiness_area(&self, ui: &mut Ui<'_>, area: Rect, live: StateFlags) -> Rect {
        if self.status == Status::Ready || area.is_empty() {
            return area;
        }
        let icon_area = Rect {
            width: area.width.min(1),
            height: area.height.min(1),
            ..area
        };
        let icon = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::ICON,
            live,
        );
        if let Some(slot) = self.ov.slot_for(Part::ICON) {
            slot(ui, icon_area);
        } else {
            match self.status {
                Status::Busy | Status::Loading => {
                    let frames = ui.design().motion.spinner_frames;
                    let glyph = frames
                        .get(self.frame.checked_rem(frames.len()).unwrap_or(0))
                        .copied()
                        .unwrap_or("");
                    ui.paint_str(icon_area, glyph, icon.style);
                }
                Status::Error => {
                    let glyph = match icon.glyph {
                        Slot::Set(glyph) => Some(glyph),
                        Slot::Inherit => Some(GlyphRole::Error),
                        Slot::Clear => None,
                    };
                    if let Some(glyph) = glyph {
                        ui.glyph(icon_area, glyph, icon.style);
                    } else {
                        ui.fill(icon_area, icon.style);
                    }
                }
                Status::Ready => {}

                _ => {}
            }
        }
        shift(area, area.width.min(2))
    }
}

fn contains_folded(label: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut wanted = query.chars().flat_map(char::to_lowercase);
    let mut next = wanted.next();
    for c in label.chars().flat_map(char::to_lowercase) {
        if Some(c) == next {
            next = wanted.next();
            if next.is_none() {
                return true;
            }
        }
    }
    false
}

fn contains_substring_folded(label: &str, query: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(&query.to_lowercase())
}

impl<T: AsItem, R: RowFn<T>> FilterList<'_, T, R> {
    pub fn draw_query(&self, ui: &mut Ui<'_>, area: Rect, st: &FilterListState, placeholder: &str) {
        ui.register_part(self.id, PartRef::of(Part::QUERY), area);
        TextInput::new(self.id)
            .value(&st.query_value)
            .placeholder(placeholder)
            .placeholder_while_editing(true)
            .pointer_enabled(false)
            .patch_part(self.parts)
            .draw(ui, area, &st.query_editor);
    }

    pub fn semantic_width(items: &[T]) -> u16 {
        items
            .iter()
            .map(|value| {
                let item = value.as_item();
                crate::text::width(item.label)
                    .saturating_add(crate::text::width(item.glyph))
                    .saturating_add(crate::text::width(item.detail))
                    .saturating_add(item.tag.map_or(0, crate::text::width))
                    .saturating_add(item.group.map_or(0, crate::text::width))
                    .saturating_add(8)
            })
            .max()
            .unwrap_or(12)
    }

    pub fn reconcile(&self, st: &mut FilterListState, items: &[T]) {
        let query = st.query_editor.draft_text().unwrap_or(&st.query_value);
        st.matches.clear();
        st.matches
            .extend(items.iter().enumerate().filter_map(|(i, item)| {
                (match self.filter {
                    FilterPolicy::Label => contains_folded(item.as_item().label, query),
                    FilterPolicy::Caller => true,
                    FilterPolicy::Substring => {
                        contains_substring_folded(item.as_item().label, query)
                    }
                })
                .then_some(i)
            }));
        st.initialized = true;
        let matches = &st.matches;
        let _ = st.core.reconcile_with(
            matches.len(),
            |i| {
                items
                    .get(matches.get(i).copied().unwrap_or(usize::MAX))
                    .map_or(ItemKey::index(i), |item| item.as_item().key)
            },
            |i| {
                items
                    .get(matches.get(i).copied().unwrap_or(usize::MAX))
                    .is_some_and(|item| !item.as_item().disabled)
            },
        );
        if st.core.cursor().is_none()
            && let Some((i, item)) = matches
                .iter()
                .filter_map(|&source| items.get(source))
                .enumerate()
                .find(|(_, item)| !item.as_item().disabled)
        {
            st.core.set_cursor(i, item.as_item().key);
        }
    }

    fn current<'i>(st: &FilterListState, items: &'i [T]) -> Option<&'i T> {
        st.matches
            .get(st.core.cursor_index())
            .and_then(|&i| items.get(i))
    }

    fn move_to(
        st: &mut FilterListState,
        items: &[T],
        target: usize,
        acc: &mut Acc<FilterListAction>,
    ) {
        if st.matches.is_empty() {
            acc.consumed();
            return;
        }
        let mut i = target.min(st.matches.len().saturating_sub(1));
        let forward = i >= st.core.cursor_index();
        loop {
            let enabled = st
                .matches
                .get(i)
                .and_then(|&s| items.get(s))
                .is_some_and(|x| !x.as_item().disabled);
            if enabled {
                break;
            }
            let next = if forward {
                i.saturating_add(1)
            } else {
                i.saturating_sub(1)
            };
            if next == i || next >= st.matches.len() {
                acc.consumed();
                return;
            }
            i = next;
        }
        let key = st
            .matches
            .get(i)
            .and_then(|&s| items.get(s))
            .map(|x| x.as_item().key);
        if key == st.core.cursor() {
            if st.core.scroll().visible_range().contains(&i) {
                acc.consumed();
            } else {
                st.core.scroll_mut().ensure_visible_on_next_layout(i);
                acc.repaint();
            }
            return;
        }
        if let Some(key) = key {
            st.core.set_cursor(i, key);
            acc.action(FilterListAction::Moved);
        }
    }

    fn item_action(
        &self,
        st: &FilterListState,
        items: &[T],
        kind: FilterListCmd,
        acc: &mut Acc<FilterListAction>,
    ) {
        let Some(item) = Self::current(st, items).map(AsItem::as_item) else {
            acc.consumed();
            return;
        };
        if item.disabled || self.status != Status::Ready {
            acc.consumed();
            return;
        }
        let action = match kind {
            FilterListCmd::Choose => FilterListAction::Chose(item.key),
            FilterListCmd::ChooseAlt => FilterListAction::ChoseAlt(item.key),
            FilterListCmd::Secondary => FilterListAction::Secondary(item.key),
            _ => return,
        };
        acc.action(action);
    }

    fn row_is_pressed(&self, ui: &Ui<'_>, key: ItemKey) -> bool {
        FrameRead::pressed_part(ui, self.id) == Some(PartRef::item(Part::ROW, key))
    }

    fn handle_cmd(
        &self,
        cmd: FilterListCmd,
        st: &mut FilterListState,
        items: &[T],
        page: usize,
        acc: &mut Acc<FilterListAction>,
    ) {
        let cur = st.core.cursor_index();
        match cmd {
            FilterListCmd::Up => {
                Self::move_to(st, items, cur.saturating_sub(1), acc);
            }
            FilterListCmd::Down => {
                Self::move_to(st, items, cur.saturating_add(1), acc);
            }
            FilterListCmd::PageUp => {
                Self::move_to(st, items, cur.saturating_sub(page), acc);
            }
            FilterListCmd::PageDown => {
                Self::move_to(st, items, cur.saturating_add(page), acc);
            }
            FilterListCmd::Home => {
                Self::move_to(st, items, 0, acc);
            }
            FilterListCmd::End => {
                Self::move_to(st, items, usize::MAX, acc);
            }
            cmd @ (FilterListCmd::Choose | FilterListCmd::ChooseAlt | FilterListCmd::Secondary) => {
                self.item_action(st, items, cmd, acc);
            }
            FilterListCmd::Back if st.query().is_empty() => {
                acc.action(FilterListAction::Back);
            }
            FilterListCmd::Back if self.searchable => {
                if st.query_editor.backspace() {
                    st.core.invalidate();
                    acc.action(FilterListAction::QueryChanged);
                } else {
                    acc.consumed();
                }
            }
            FilterListCmd::NextScope => {
                acc.action(FilterListAction::NextScope);
            }
            FilterListCmd::PrevScope => {
                acc.action(FilterListAction::PrevScope);
            }
            FilterListCmd::Cancel if st.query().is_empty() => {
                acc.action(FilterListAction::Cancel);
            }
            FilterListCmd::Cancel if self.searchable => {
                st.set_query("");
                st.core.invalidate();
                acc.action(FilterListAction::QueryChanged);
            }
            _ => {}
        }
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FilterListState,
        items: &[T],
    ) -> Response<FilterListAction> {
        self.reconcile(st, items);
        let query_before = st.query().to_owned();
        if self.searchable {
            let input = TextInput::new(self.id);
            let _ = input.update(cx, &mut st.query_editor, &mut st.query_value);
        }
        let mut acc = Acc::new();
        let bar = self
            .scrollbar()
            .update(cx, st.core.scroll_mut(), st.matches.len());
        acc.fold(&bar);
        let page = st.core.scroll().viewport_len().max(1);
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(action) => {
                    if let Some(cmd) = Binding::command(BINDINGS, action) {
                        self.handle_cmd(cmd, st, items, page, &mut acc);
                    } else if Binding::command(
                        TextInput::new(self.id).bindings(BindingState::default()),
                        action,
                    ) == Some(TextCmd::Cancel)
                    {
                        self.handle_cmd(FilterListCmd::Cancel, st, items, page, &mut acc);
                    }
                }
                Intent::Paste(_) | Intent::Key(_) if self.searchable => {}
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(key),
                        },
                    ..
                } => {
                    let Some(i) = st
                        .matches
                        .iter()
                        .position(|&s| items.get(s).is_some_and(|x| x.as_item().key == key))
                    else {
                        acc.consumed();
                        continue;
                    };
                    st.core.set_cursor(i, key);
                    match phase {
                        Phase::Secondary => {
                            self.item_action(st, items, FilterListCmd::Secondary, &mut acc);
                        }
                        Phase::Click | Phase::DoubleClick => {
                            self.item_action(st, items, FilterListCmd::Choose, &mut acc);
                        }
                        Phase::Press => {
                            acc.changed();
                        }
                        _ => {
                            acc.consumed();
                        }
                    }
                }
                Intent::Pointer {
                    phase: Phase::Press | Phase::Click | Phase::DragStart | Phase::Drag,
                    local,
                    part:
                        PartRef {
                            part: Part::QUERY,
                            item: None,
                        },
                    ..
                } if self.searchable => {
                    let area = cx.area(self.id).unwrap_or_default();
                    if st.query_editor.place_cursor(
                        usize::from(local.x.saturating_sub(2)),
                        area.width.saturating_sub(3),
                    ) {
                        acc.changed();
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer { .. } => {
                    acc.consumed();
                }
                _ => {}
            }
        }
        if st.query() != query_before {
            st.core.invalidate();
            acc.action(FilterListAction::QueryChanged);
        }
        let query_width = cx
            .area(self.id)
            .map_or(0, |area| area.width.saturating_sub(3));
        st.query_editor.ensure_cursor_visible(query_width);
        self.reconcile(st, items);
        acc.finish(self.id)
    }

    fn draw_rows(
        &self,
        ui: &mut Ui<'_>,
        content: Rect,
        st: &FilterListState,
        items: &[T],
        live: StateFlags,
    ) {
        let identity = !st.initialized && st.query().is_empty();
        let visible_len = if identity {
            items.len()
        } else {
            st.matches.len()
        };
        let columns = (self.item_layout == crate::item::ItemRowLayout::Columns).then(|| {
            crate::item::ItemColumns::measure(
                (0..visible_len).filter_map(|i| {
                    let source = if identity {
                        Some(i)
                    } else {
                        st.matches.get(i).copied()
                    };
                    source
                        .and_then(|index| items.get(index))
                        .map(AsItem::as_item)
                }),
                content.width,
                self.metadata_gap,
            )
        });
        let mut last_group = "";
        let view = ScrollRegion::view(st.core.scroll(), content, visible_len);
        for (row_i, filtered_i) in view.visible_range().enumerate() {
            let source = if identity {
                Some(filtered_i)
            } else {
                st.matches.get(filtered_i).copied()
            };
            let Some(item) = source.and_then(|index| items.get(index)) else {
                break;
            };
            let semantic = item.as_item();
            let mut flags = self.status.flags();
            if st.core.cursor() == Some(semantic.key) {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
                if self.row_is_pressed(ui, semantic.key) {
                    flags |= StateFlags::PRESSED;
                }
            }
            if semantic.disabled {
                flags |= StateFlags::DISABLED;
                flags.remove(StateFlags::PRESSED);
            }
            let row = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: 1,
            };
            let visible = !row.intersection(ui.full()).is_empty();
            let mut row_ui = RowUi::new(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                flags,
                semantic.key,
                row,
            )
            .with_cursor_style(self.cursor_style);
            if let Some(columns) = &columns {
                let group = semantic.group.unwrap_or("");
                columns.paint(
                    semantic,
                    !group.is_empty() && group != last_group,
                    &mut row_ui,
                );
                if visible {
                    last_group = group;
                }
            } else {
                self.row.row(item, &mut row_ui);
            }
            ui.register_part(self.id, PartRef::item(Part::ROW, semantic.key), row);
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &FilterListState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        if self.register_owner {
            if self.searchable {
                ui.register_editor(self.id, area, Focusability::Focusable, StateFlags::EDITING);
            } else {
                ui.register_control(self.id, area, Focusability::Focusable);
            }
        }
        let live = ui.state(self.id);
        ui.publish_bindings(self.id, live, self.bindings(BindingState { flags: live }));
        if self.searchable {
            let query_bindings = TextInput::new(self.id).bindings(BindingState { flags: live });
            ui.publish_dynamic_bindings(
                self.id,
                live,
                query_bindings
                    .iter()
                    .filter(|binding| {
                        !matches!(
                            binding.cmd,
                            TextCmd::Commit
                                | TextCmd::Cancel
                                | TextCmd::Backspace
                                | TextCmd::Delete
                                | TextCmd::DeleteToLineEnd
                        )
                    })
                    .map(|binding| (binding.action, binding.chord)),
            );
        }
        let mut live = PartStyle::flags(ui.state(self.id), self.status.flags());
        live.remove(StateFlags::PRESSED);
        let container = self.ov.style(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, container.style);
        let identity = !st.initialized && st.query().is_empty();
        let visible_len = if identity {
            items.len()
        } else {
            st.matches.len()
        };
        let list_area = self.readiness_area(ui, area, live);
        let content = self
            .scrollbar()
            .draw(ui, list_area, st.core.scroll(), visible_len);
        if visible_len == 0 {
            let empty = self.empty.unwrap_or(EmptyState::Empty {
                title: "No matches",
                hint: None,
            });
            if let Some(slot) = self.ov.slot_for(Part::EMPTY) {
                slot(ui, content);
            } else {
                let inherited = self
                    .ov
                    .style(
                        ui,
                        self.id,
                        Family::PICKER,
                        Variant::DEFAULT,
                        Part::EMPTY,
                        live,
                    )
                    .style;
                empty.draw_inherited(ui, content, 0, inherited);
            }
            return area;
        }
        self.draw_rows(ui, content, st, items, live);
        area
    }
}

impl<T, R> Bindings for FilterList<'_, T, R> {
    type Cmd = FilterListCmd;
    fn bindings(&self, _st: BindingState) -> &'static [Binding<Self::Cmd>] {
        if self.searchable {
            SEARCHABLE_BINDINGS
        } else {
            BINDINGS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FilterList, FilterListAction, FilterListState, contains_folded};
    use crate::Acc;
    use termrock_core::id::{Id, ItemKey, Part};
    use termrock_core::response::StateFlags;
    use termrock_theme::Theme;

    use crate::event::{Key, KeyCode, KeyModifiers};
    use crate::item::{AsItem, Item, ItemRow};
    use crate::response::Response;
    use crate::runtime::stub::key;
    use crate::runtime::{App, Runtime};
    use crate::ui::Cx;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use termrock_runtime::Status;
    use termrock_runtime::event::Input;
    use termrock_runtime::ui::cx::LastFrame;
    use termrock_runtime::ui::{FrameState, Ui, UiCore};

    const AREA: Rect = Rect::new(2, 1, 20, 4);

    fn with_ui<R>(f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, Rect::new(0, 0, 30, 8));
        let mut page = Buffer::empty(Rect::new(0, 0, 30, 8));
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            f(&mut ui)
        };
        (out, page)
    }

    fn slot_icon(ui: &mut Ui<'_>, area: Rect) {
        ui.paint_str(area, "#", ui.surface_style());
    }

    struct Domain(&'static str);

    impl AsItem for Domain {
        fn as_item(&self) -> Item<'_> {
            Item::new(ItemKey::num(9), self.0)
        }
    }

    #[test]
    fn filters_and_measures_from_semantic_label() {
        let domain = Domain("CommandPalette");
        let item = domain.as_item();
        assert!(contains_folded(item.label, "cpa"));
        assert_eq!(FilterList::<Domain, ItemRow>::semantic_width(&[domain]), 22);
        assert!(!contains_folded("picker", "px"));
    }

    #[test]
    fn filtering_borrowed_domain_items_reuses_one_index_buffer() {
        let items = [Domain("alpha"), Domain("beta"), Domain("gamma")];
        let mut state = FilterListState::default();
        FilterList::<Domain, ItemRow>::new(Id::root("test")).reconcile(&mut state, &items);
        let capacity = state.matches.capacity();
        state.set_query("a");
        FilterList::<Domain, ItemRow>::new(Id::root("test")).reconcile(&mut state, &items);
        assert_eq!(state.matches.capacity(), capacity);
    }

    #[test]
    fn substring_policy_is_contiguous_and_distinct_from_label_fuzzy_matching() {
        let items = [Item::new(ItemKey::num(4), "CommandPalette")];
        let mut fuzzy = FilterListState::default();
        fuzzy.set_query("cpa");
        FilterList::<Item<'_>>::new(Id::root("filter-list.fuzzy"))
            .filter(super::FilterPolicy::Label)
            .reconcile(&mut fuzzy, &items);
        assert_eq!(fuzzy.matched_len(), 1);

        let mut substring = FilterListState::default();
        substring.set_query("cpa");
        FilterList::<Item<'_>>::new(Id::root("filter-list.substring"))
            .filter(super::FilterPolicy::Substring)
            .reconcile(&mut substring, &items);
        assert_eq!(substring.matched_len(), 0);

        substring.set_query("palette");
        FilterList::<Item<'_>>::new(Id::root("filter-list.substring"))
            .filter(super::FilterPolicy::Substring)
            .reconcile(&mut substring, &items);
        assert_eq!(substring.matched_len(), 1);
    }

    #[test]
    fn substring_filter_uses_typed_field_caret_and_paste() {
        const ID: Id = Id::root("filter-list.query-editor-contract");
        const SCREEN: Rect = Rect::new(0, 0, 40, 8);

        struct FilterApp {
            state: FilterListState,
            items: [Item<'static>; 3],
        }

        impl App for FilterApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                FilterList::<Item<'static>>::new(ID)
                    .filter(super::FilterPolicy::Substring)
                    .update(cx, &mut self.state, &self.items)
                    .erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                FilterList::<Item<'static>>::new(ID)
                    .filter(super::FilterPolicy::Substring)
                    .draw(ui, SCREEN, &self.state, &self.items);
            }
        }

        let app = FilterApp {
            state: FilterListState::default(),
            items: [
                Item::new(ItemKey::num(1), "Orders Console"),
                Item::new(ItemKey::num(2), "Customer Notes"),
                Item::new(ItemKey::num(3), "CONSOLE Logs"),
            ],
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Paste("console".into()));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "console");
        assert_eq!(runtime.app().state.matched_len(), 2);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('j'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(3)));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('k'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(1)));

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Left));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Paste("x".into()));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "consolxe");
        assert_eq!(runtime.app().state.matched_len(), 0);
    }

    #[test]
    fn same_key_navigation_reveals_a_reconciled_offscreen_cursor() {
        let items = [
            Item::new(ItemKey::num(1), "one"),
            Item::new(ItemKey::num(2), "two"),
        ];
        let mut state = FilterListState::default();
        FilterList::<Item<'_>, ItemRow>::new(Id::root("test")).reconcile(&mut state, &items);
        state.core.set_cursor(1, ItemKey::num(2));
        state.core.scroll_mut().apply_layout(1, items.len());
        state.core.scroll_mut().scroll_to(0);

        let mut acc: Acc<FilterListAction> = Acc::new();
        FilterList::<Item<'_>, ItemRow>::move_to(&mut state, &items, 1, &mut acc);
        let response = acc.finish(Id::root("filter-list.same-key-reveal"));

        assert!(response.is_changed());
        assert_eq!(state.core.scroll().pending_reveal(), Some(1));
    }

    #[test]
    fn readiness_icon_is_declared_and_ready_reserves_no_geometry() {
        assert!(FilterList::<Item<'static>>::PARTS.contains(&Part::ICON));
        let (ready, _) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.ready"))
                .status(Status::Ready)
                .readiness_area(ui, AREA, StateFlags::empty())
        });
        let (busy, _) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.busy"))
                .status(Status::Busy)
                .readiness_area(ui, AREA, StateFlags::BUSY)
        });
        assert_eq!(ready, AREA);
        assert_eq!(
            busy,
            Rect {
                x: 4,
                width: 18,
                ..AREA
            }
        );
    }

    #[test]
    fn busy_error_and_icon_slot_are_visible() {
        let (_, busy) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.busy"))
                .status(Status::Busy)
                .readiness_area(ui, AREA, StateFlags::BUSY)
        });
        assert_ne!(busy[(AREA.x, AREA.y)].symbol(), " ");

        let (_, error) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.error"))
                .status(Status::Error)
                .readiness_area(ui, AREA, StateFlags::ERROR)
        });
        assert_ne!(error[(AREA.x, AREA.y)].symbol(), " ");

        let (_, slotted) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.slot"))
                .status(Status::Loading)
                .slot(Part::ICON, &slot_icon)
                .readiness_area(ui, AREA, StateFlags::LOADING)
        });
        assert_eq!(slotted[(AREA.x, AREA.y)].symbol(), "#");
    }
}
