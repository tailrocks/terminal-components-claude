use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, shift};
use crate::action::ActionKey;
pub use crate::item::AsItem;
use crate::scroll::ScrollState;
use termrock_collections::collection::{CollectionCore, Reconcile, Reconciliation};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_fields::input::{BlurPolicy, TextAction, TextCmd, TextInput, TextInputState};
use termrock_runtime::author;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SelectionPolicy {
    #[default]
    Preserve,

    FirstEnabledOnFilterChange,
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
        ActionKey::custom("filter-list.scope"),
        Chord::key(KeyCode::Tab),
        FilterListCmd::NextScope,
        "Next scope",
        false,
    ),
    binding(
        ActionKey::custom("filter-list.previous-scope"),
        Chord::key(KeyCode::BackTab),
        FilterListCmd::PrevScope,
        "Previous scope",
        false,
    ),
    Binding {
        action: ActionKey::custom("filter-list.back"),
        chord: Some(Chord::key(KeyCode::Backspace)),
        cmd: FilterListCmd::Back,
        label: "Back",
        priority: 60,
        visible: false,
    },
    Binding {
        action: ActionKey::custom("filter-list.secondary"),
        chord: Some(Chord::key(KeyCode::Delete)),
        cmd: FilterListCmd::Secondary,
        label: "Secondary",
        priority: 70,
        visible: false,
    },
    Binding {
        action: ActionKey::custom("filter-list.cancel"),
        chord: Some(Chord::key(KeyCode::Esc)),
        cmd: FilterListCmd::Cancel,
        label: "Cancel",
        priority: 60,
        visible: false,
    },
];

const QUERY_EXCLUDE_NONE: &[TextCmd] = &[TextCmd::DeleteToLineEnd];
const QUERY_EXCLUDE_DELETE: &[TextCmd] = &[TextCmd::Delete, TextCmd::DeleteToLineEnd];
const QUERY_EXCLUDE_EMPTY: &[TextCmd] = &[
    TextCmd::Backspace,
    TextCmd::Delete,
    TextCmd::Cancel,
    TextCmd::DeleteToLineEnd,
];
const QUERY_EXCLUDE_EMPTY_NO_DELETE: &[TextCmd] = &[
    TextCmd::Backspace,
    TextCmd::Cancel,
    TextCmd::DeleteToLineEnd,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FilterPolicy {
    #[default]
    Label,
    Caller,
    Substring,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct FilterListState {
    query: String,
    query_input: TextInputState,
    query_edit_menu: Option<termrock_fields::input::EditMenuRequest>,
    core: CollectionCore,
    matches: Vec<usize>,
    initialized: bool,
    last_query: Option<String>,
    reset_selection: bool,
}

impl FilterListState {
    pub fn query(&self) -> &str {
        self.query_input.draft_text().unwrap_or(&self.query)
    }
    pub const fn is_query_editing(&self) -> bool {
        self.query_input.is_editing()
    }
    pub fn set_query(&mut self, query: impl Into<String>) {
        let editing = self.query_input.is_editing();
        self.query = query.into();
        self.query_input.cancel();
        if editing {
            self.query_input.begin(&self.query);
        }
        self.core.invalidate();
    }

    pub fn reset_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.query_input.cancel();
        self.core.invalidate();
    }
    pub fn take_query_edit_menu_request(
        &mut self,
    ) -> Option<termrock_fields::input::EditMenuRequest> {
        self.query_edit_menu.take()
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

    pub fn reset_selection_on_next_reconcile(&mut self) {
        self.reset_selection = true;
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
    results_id: Id,
    row: R,
    item_layout: crate::item::ItemRowLayout,
    cursor_style: CursorStyle,
    metadata_gap: u16,
    empty: Option<EmptyState<'a>>,
    status: Status,
    frame: usize,
    selection_policy: SelectionPolicy,
    searchable: bool,
    placeholder: &'a str,
    delete_is_secondary: bool,
    filter: FilterPolicy,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    preserve_surface: bool,
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
            results_id: id.sub("results"),
            row: crate::item::ItemRow,
            item_layout: crate::item::ItemRowLayout::Compact,
            cursor_style: CursorStyle::Bar,
            metadata_gap: 1,
            empty: None,
            status: Status::Ready,
            frame: 0,
            selection_policy: SelectionPolicy::Preserve,
            searchable: true,
            placeholder: "Type to search…",
            delete_is_secondary: true,
            filter: FilterPolicy::Label,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            preserve_surface: false,
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
    #[must_use]
    pub const fn selection_policy(mut self, policy: SelectionPolicy) -> Self {
        self.selection_policy = policy;
        self
    }

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
    pub const fn preserve_surface(mut self, preserve: bool) -> Self {
        self.preserve_surface = preserve;
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
            results_id: self.results_id,
            row,
            item_layout: crate::item::ItemRowLayout::Compact,
            cursor_style: self.cursor_style,
            metadata_gap: self.metadata_gap,
            empty: self.empty,
            status: self.status,
            frame: self.frame,
            selection_policy: self.selection_policy,
            searchable: self.searchable,
            placeholder: self.placeholder,
            delete_is_secondary: self.delete_is_secondary,
            filter: self.filter,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            preserve_surface: self.preserve_surface,
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
    #[must_use]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
    #[must_use]
    pub const fn delete_is_secondary(mut self, enabled: bool) -> Self {
        self.delete_is_secondary = enabled;
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
        let mut bar = ScrollRegion::new(self.results_id)
            .patch_part(self.parts)
            .preserve_underlay(self.preserve_surface);
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

    fn query_input(&self, query_empty: bool) -> TextInput<'a> {
        let excluded = match (query_empty, self.delete_is_secondary) {
            (true, true) => QUERY_EXCLUDE_EMPTY,
            (true, false) => QUERY_EXCLUDE_EMPTY_NO_DELETE,
            (false, true) => QUERY_EXCLUDE_DELETE,
            (false, false) => QUERY_EXCLUDE_NONE,
        };
        TextInput::new(self.id)
            .placeholder(self.placeholder)
            .placeholder_while_editing(true)
            .blur(BlurPolicy::Keep)
            .exclude_commands(excluded)
            .auxiliary_style_target(Family::PICKER, Variant::DEFAULT)
            .style_target(Family::PICKER, Variant::DEFAULT, Part::QUERY)
    }

    fn searchable_binding(&self, cmd: FilterListCmd, query_empty: bool) -> bool {
        match cmd {
            FilterListCmd::Home | FilterListCmd::End | FilterListCmd::Choose => false,
            FilterListCmd::Back | FilterListCmd::Cancel => query_empty,
            FilterListCmd::Secondary => self.delete_is_secondary,
            _ => true,
        }
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

fn surface_already_opaque(ui: &mut Ui<'_>, area: Rect) -> bool {
    ui.with_area(area, |ui| {
        let (buf, clip) = ui.raw();
        !clip.is_empty()
            && clip.positions().all(|pos| {
                buf.cell(pos)
                    .is_some_and(|cell| cell.bg != ratatui_core::style::Color::Reset)
            })
    })
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
        st.matches.clear();
        let query = st.query().to_owned();
        let reset_for_query =
            if self.selection_policy == SelectionPolicy::FirstEnabledOnFilterChange {
                let query_changed = st
                    .last_query
                    .as_deref()
                    .is_some_and(|previous| previous != query);
                st.last_query = Some(query.clone());
                query_changed
            } else {
                st.last_query = None;
                false
            };

        if st.reset_selection || reset_for_query {
            st.core.clear_cursor();
        }
        st.reset_selection = false;
        st.matches
            .extend(items.iter().enumerate().filter_map(|(i, item)| {
                (match self.filter {
                    FilterPolicy::Label => contains_folded(item.as_item().label, &query),
                    FilterPolicy::Caller => true,
                    FilterPolicy::Substring => {
                        contains_substring_folded(item.as_item().label, &query)
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
        let index = if !st.initialized && st.query().is_empty() {
            Some(st.core.cursor_index())
        } else {
            st.matches.get(st.core.cursor_index()).copied()
        };
        index.and_then(|i| items.get(i))
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
        let part = PartRef::item(Part::ROW, key);
        FrameRead::pressed_part(ui, self.id) == Some(part)
            || (self.searchable && FrameRead::pressed_part(ui, self.results_id) == Some(part))
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
            FilterListCmd::Back => acc.consumed(),
            FilterListCmd::NextScope => {
                acc.action(FilterListAction::NextScope);
            }
            FilterListCmd::PrevScope => {
                acc.action(FilterListAction::PrevScope);
            }
            FilterListCmd::Cancel if st.query().is_empty() => {
                acc.action(FilterListAction::Cancel);
            }
            FilterListCmd::Cancel => acc.consumed(),
        }
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FilterListState,
        items: &[T],
    ) -> Response<FilterListAction> {
        let hidden_query_changed = !self.searchable && !st.query().is_empty();
        if hidden_query_changed {
            st.reset_query("");
        }
        let mut acc = Acc::new();
        if hidden_query_changed {
            acc.action(FilterListAction::QueryChanged);
        }

        if !st.initialized {
            self.reconcile(st, items);
        }

        if self.searchable {
            if !st.query_input.is_editing() {
                st.query_input.begin(&st.query);
            }
            let previous = st.query().to_owned();
            let committed = st.query.clone();
            let input = self
                .query_input(previous.is_empty())
                .value(&committed)
                .update(cx, &mut st.query_input, &mut st.query);
            match input.action_ref().copied() {
                Some(TextAction::Changed) => {
                    st.core.invalidate();
                    acc.action(FilterListAction::QueryChanged);
                }
                Some(TextAction::Cancelled) => {
                    st.set_query("");
                    if previous.is_empty() {
                        acc.action(FilterListAction::Cancel);
                    } else {
                        st.query_input.begin("");
                        acc.action(FilterListAction::QueryChanged);
                    }
                }
                Some(TextAction::Committed) => {
                    self.item_action(st, items, FilterListCmd::Choose, &mut acc);

                    st.query_input.begin(&st.query);
                }
                _ => {}
            }
            st.query_edit_menu = st.query_input.take_edit_menu_request();
            acc.fold(&input.erase());
        }

        self.reconcile(st, items);
        let bar = self
            .scrollbar()
            .update(cx, st.core.scroll_mut(), st.matches.len());
        acc.fold(&bar);
        let page = st.core.scroll().viewport_len().max(1);
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(action) => {
                    let cmd = Binding::command(BINDINGS, action).or_else(|| {
                        if hidden_query_changed {
                            Binding::command(
                                TextInput::new(self.id).bindings(BindingState {
                                    flags: StateFlags::empty(),
                                }),
                                action,
                            )
                            .and_then(|command| match command {
                                TextCmd::Cancel => Some(FilterListCmd::Cancel),
                                TextCmd::Backspace => Some(FilterListCmd::Back),
                                _ => None,
                            })
                        } else {
                            None
                        }
                    });
                    if let Some(cmd) = cmd {
                        self.handle_cmd(cmd, st, items, page, &mut acc);
                    }
                }
                Intent::Paste(_) | Intent::Key(_) if self.searchable => {}
                _ => {}
            }
        }
        let pointer_owners = if self.searchable {
            [Some(self.id), Some(self.results_id)]
        } else {
            [Some(self.id), None]
        };
        for owner in pointer_owners.into_iter().flatten() {
            for intent in cx.intents(owner) {
                match intent {
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
                    Intent::Pointer { .. } => {
                        acc.consumed();
                    }
                    _ => {}
                }
            }
        }
        acc.finish(self.id)
    }

    pub fn draw_query(&self, ui: &mut Ui<'_>, area: Rect, st: &FilterListState) {
        if self.searchable {
            let mut parts = Vec::new();
            for &(part, patch) in self.parts {
                match part {
                    Part::QUERY => {
                        parts.push((Part::FIELD, patch));
                        parts.push((Part::TEXT, patch));
                    }
                    Part::GUTTER => parts.push((Part::GUTTER, patch)),
                    Part::PLACEHOLDER => parts.push((Part::PLACEHOLDER, patch)),
                    _ => {}
                }
            }
            let mut input = self.query_input(st.query().is_empty()).value(st.query());
            if let Some(patch) = self.patch {
                input = input.patch(patch);
            }
            input = input.patch_part(&parts);
            for (part, target) in [
                (Part::GUTTER, Part::GUTTER),
                (Part::PLACEHOLDER, Part::PLACEHOLDER),
            ] {
                if let Some(slot) = self.ov.slot_for(part) {
                    input = input.slot(target, slot);
                }
            }
            input.draw(ui, area, &st.query_input);
        }
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
            let mut row_ui = RowUi::new_with_container_width(
                ui,
                self.id,
                Family::PICKER,
                Variant::DEFAULT,
                flags,
                semantic.key,
                row,
                row.width,
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
            let part = PartRef::item(Part::ROW, semantic.key);
            if self.searchable {
                ui.register_part(self.results_id, part, row);
            }
            ui.register_part(self.id, part, row);
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &FilterListState, items: &[T]) -> Rect {
        if self.searchable && !area.is_empty() {
            self.query_input(st.query().is_empty())
                .value(st.query())
                .register_editor(ui, area, &st.query_input);
        }
        self.draw_results(ui, area, st, items)
    }

    #[doc(hidden)]
    pub fn draw_results(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &FilterListState,
        items: &[T],
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let live = ui.state(self.id);
        if !self.searchable {
            ui.register_control(self.id, area, Focusability::Focusable);

            ui.publish_bindings(self.id, live, BINDINGS);
        }
        if self.searchable {
            let _ = author::extend_bindings(
                ui,
                self.id,
                live,
                self.bindings(BindingState { flags: live }),
                |cmd| self.searchable_binding(cmd, st.query().is_empty()),
            );
        } else {
            ui.publish_bindings(self.id, live, BINDINGS);
        }
        let mut live = PartStyle::flags(ui.state(self.id), self.status.flags());
        live.remove(StateFlags::PRESSED);
        let container_patch = self.ov.part_patch(Part::CONTAINER);
        let detailed = ui.style_defaults_with_explicit(
            Family::PICKER,
            Variant::DEFAULT,
            Part::CONTAINER,
            live,
            crate::theme::StyleDefaults::default(),
            container_patch.as_ref(),
        );
        self.ov.note(
            ui,
            self.id,
            Family::PICKER,
            Variant::DEFAULT,
            Part::CONTAINER,
            detailed.resolved,
        );
        if !self.preserve_surface || !surface_already_opaque(ui, area) {
            ui.fill(area, detailed.resolved.style);
        } else {
            termrock_runtime::author::patch_style(
                ui,
                area,
                detailed.resolved.style,
                detailed.explicit,
            );
        }
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

        let mut view = *st.scroll();
        view.apply_layout(usize::from(content.height), visible_len);
        ui.scroll_edges(content, &view);
        area
    }
}

impl<T, R> Bindings for FilterList<'_, T, R> {
    type Cmd = FilterListCmd;
    fn bindings(&self, _st: BindingState) -> &'static [Binding<Self::Cmd>] {
        BINDINGS
    }
}
