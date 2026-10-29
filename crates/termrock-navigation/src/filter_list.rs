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

/// Controls cursor behavior when a query changes the result set.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SelectionPolicy {
    /// Preserve a surviving item key.
    #[default]
    Preserve,
    /// Move to the first enabled result after a query or scope change.
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

    /// Clear a query and end its edit session, for owners that hide the query
    /// field while retaining the collection state.
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

    /// Reset the cursor to the first enabled result on the next reconciliation.
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

    /// The picker card already painted this rectangle. Do not cover that ink.
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
        // Explicit requests are unconditional; the policy controls only
        // automatic cursor resets when a query changes.
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
                        // Escape clears the query without dismissing its owner.
                        // Keep the still-visible field editing so its caret and
                        // immediate typing/paste remain available.
                        st.query_input.begin("");
                        acc.action(FilterListAction::QueryChanged);
                    }
                }
                Some(TextAction::Committed) => {
                    self.item_action(st, items, FilterListCmd::Choose, &mut acc);
                    // A commit with no eligible result leaves this search
                    // owner open, so continue editing its current query.
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
        // Keep FilterList's established area contract. Compositions that own
        // a separate query row (such as Picker) place it explicitly via
        // `draw_query` and pass their result viewport to `draw_results`.
        if self.searchable && !area.is_empty() {
            // The standalone searchable FilterList owns the visible result
            // viewport as its focusable query owner. Picker draws a real
            // TextInput query row separately and calls draw_results directly.
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
            // Without the visible query TextInput there is no base binding
            // publisher for this control. Publish the list bindings here so
            // the following extensions (Back, Secondary, Cancel) and regular
            // navigation/activation commands can be routed to the focused
            // owner.
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
        // Baseline fades the last visible rows of an overflowing list toward the card.
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

#[cfg(test)]
mod tests {
    use super::{BINDINGS, FilterList, FilterListAction, FilterListState, contains_folded};
    use crate::Acc;
    use termrock_core::action::ActionKey;
    use termrock_core::id::{Id, ItemKey, Part};
    use termrock_core::response::StateFlags;
    use termrock_theme::{Family, Role, StylePatch, Theme};

    use crate::item::{AsItem, Item, ItemRow};
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use termrock_runtime::Status;
    use termrock_runtime::keymap::Binding;
    use termrock_runtime::ui::cx::LastFrame;
    use termrock_runtime::ui::{FrameState, Ui, UiCore};

    const AREA: Rect = Rect::new(2, 1, 20, 4);

    fn with_ui<R>(f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let theme = Theme::junie();
        with_ui_theme(&theme, f)
    }

    fn with_ui_theme<R>(theme: &Theme, f: impl FnOnce(&mut Ui<'_>) -> R) -> (R, Buffer) {
        let mut frame = FrameState::default();
        frame.reset(1, Rect::new(0, 0, 30, 8));
        let mut page = Buffer::empty(Rect::new(0, 0, 30, 8));
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let out = {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, theme, &last);
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
    fn picker_tag_default_ink_tracks_resolved_owner_contrast() {
        use ratatui_core::style::Color;
        use ratatui_core::style::Modifier;
        use termrock_runtime::runtime::App;
        use termrock_runtime::ui::Cx;
        use termrock_test_support::Harness;
        use termrock_theme::CursorStyle;
        use termrock_theme::{ColorLevel, FgStep, Overlay, OverlayRule, Variant, contrast_ratio};

        const ID: Id = Id::root("picker.light-meta");
        const AREA: Rect = Rect::new(0, 0, 30, 8);
        const WIDTH: u16 = 30;
        const HEIGHT: u16 = 8;
        const BG_RULES: [OverlayRule; 1] = [(
            Family::PICKER,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            StylePatch::new()
                .set_bg(Role::Custom(Color::White))
                .add(Modifier::ITALIC),
        )];
        const BG_AND_FG_RULES: [OverlayRule; 1] = [(
            Family::PICKER,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            StylePatch::new()
                .set_fg(Role::Custom(Color::Black))
                .set_bg(Role::Custom(Color::White)),
        )];
        const BG_OVERLAY: Overlay = Overlay::new(&BG_RULES);
        const BG_AND_FG_OVERLAY: Overlay = Overlay::new(&BG_AND_FG_RULES);
        struct TagApp {
            state: FilterListState,
            cursor: CursorStyle,
            owner_style: u8,
        }
        impl App for TagApp {
            fn update(&mut self, cx: &mut Cx<'_>) -> termrock_core::response::Response<()> {
                let items = [Item::new(ItemKey::num(1), "row").tag("T")];
                FilterList::<Item<'_>>::new(ID)
                    .searchable(false)
                    .cursor_style(self.cursor)
                    .update(cx, &mut self.state, &items)
                    .erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let items = [Item::new(ItemKey::num(1), "row").tag("T")];
                let draw = |ui: &mut Ui<'_>| {
                    FilterList::<Item<'_>>::new(ID)
                        .searchable(false)
                        .cursor_style(self.cursor)
                        .draw(ui, AREA, &self.state, &items)
                };
                match self.owner_style {
                    1 => ui.with_overlay(&BG_OVERLAY, draw),
                    2 => ui.with_overlay(&BG_AND_FG_OVERLAY, draw),
                    _ => draw(ui),
                };
            }
        }

        let levels = [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ];
        for level in levels {
            for cursor in [CursorStyle::Bar, CursorStyle::Fill] {
                // Default theme/profile styling stays frozen, including the
                // intentionally light Mono focused fill.
                let h = Harness::new(
                    TagApp {
                        state: FilterListState::default(),
                        cursor,
                        owner_style: 0,
                    },
                    Theme::junie(),
                    WIDTH,
                    HEIGHT,
                )
                .with_color(level);
                assert_eq!(
                    h.focus(),
                    Some(ID),
                    "real result list focus {level:?}/{cursor:?}"
                );
                let (x, y) = h.find("T").expect("tag cell remains visible");
                let tag = h.cell(x, y);
                let reference_theme = Theme::junie().downgrade(level);
                let expected_fg = match cursor {
                    CursorStyle::Bar => reference_theme.color.fg[FgStep::Secondary.index()],
                    CursorStyle::Fill => reference_theme.color.highlight_fg,
                };
                assert_eq!(
                    tag.fg, expected_fg,
                    "default reference ink preserved at {level:?}/{cursor:?}"
                );
            }
        }

        for level in levels {
            for cursor in [CursorStyle::Bar, CursorStyle::Fill] {
                // This overlay is an authored owner background, so contrast
                // fallback applies only while FG remains inherited.
                let h = Harness::new(
                    TagApp {
                        state: FilterListState::default(),
                        cursor,
                        owner_style: 1,
                    },
                    Theme::junie(),
                    WIDTH,
                    HEIGHT,
                )
                .with_color(level);
                let (x, y) = h.find("T").expect("tag cell remains visible");
                let tag = h.cell(x, y);
                assert_eq!(
                    tag.bg,
                    Color::White,
                    "owner background {level:?}/{cursor:?}"
                );
                let contrast = contrast_ratio(tag.fg, tag.bg).unwrap_or(0.0);
                assert!(
                    contrast >= 4.5,
                    "tag contrast {contrast:.2}:1 at {level:?}/{cursor:?}"
                );
                assert!(
                    tag.modifier.contains(Modifier::ITALIC),
                    "modifier preserved"
                );
            }
        }

        for level in levels {
            for cursor in [CursorStyle::Bar, CursorStyle::Fill] {
                let h = Harness::new(
                    TagApp {
                        state: FilterListState::default(),
                        cursor,
                        owner_style: 2,
                    },
                    Theme::junie(),
                    WIDTH,
                    HEIGHT,
                )
                .with_color(level);
                let (x, y) = h.find("T").expect("tag cell remains visible");
                let tag = h.cell(x, y);
                assert_eq!(tag.fg, Color::Black, "same-valued explicit FG preserved");
                assert_eq!(tag.bg, Color::White, "owner background preserved");
            }
        }
    }

    #[test]
    fn searchable_filter_list_draw_keeps_its_full_area_for_results() {
        let items = [Item::new(ItemKey::num(1), "alpha")];
        let (_, page) = with_ui(|ui| {
            FilterList::<Item<'_>>::new(Id::root("filter-list.area-contract")).draw(
                ui,
                AREA,
                &FilterListState::default(),
                &items,
            );
        });
        assert_eq!(page[(AREA.x.saturating_add(1), AREA.y)].symbol(), "a");
    }

    #[test]
    fn standalone_searchable_filter_list_owns_visible_focus_and_typing() {
        use termrock_core::event::{Input, Key, KeyCode, KeyModifiers};
        use termrock_runtime::runtime::{App, Runtime};

        const ID: Id = Id::root("filter-list.standalone-search");
        struct SearchApp {
            state: FilterListState,
        }
        impl App for SearchApp {
            fn update(
                &mut self,
                cx: &mut termrock_runtime::ui::Cx<'_>,
            ) -> termrock_core::response::Response<()> {
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                let response = FilterList::<Item<'_>>::new(ID).update(cx, &mut self.state, &items);
                response.erase()
            }

            fn draw(&self, ui: &mut Ui<'_>) {
                let items = [
                    Item::new(ItemKey::num(1), "alpha"),
                    Item::new(ItemKey::num(2), "beta"),
                ];
                FilterList::<Item<'_>>::new(ID).draw(
                    ui,
                    Rect::new(0, 0, 30, 8),
                    &self.state,
                    &items,
                );
            }
        }

        let area = Rect::new(0, 0, 30, 8);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(
            SearchApp {
                state: FilterListState::default(),
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let _ = termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(ID));
        assert_eq!(runtime.area_of(ID), Some(area));

        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('b'),
                mods: KeyModifiers::NONE,
            }),
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "b");
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
    fn explicit_reset_selects_first_enabled_item_even_with_default_policy() {
        let items = [
            Item::new(ItemKey::num(1), "disabled").disabled(true),
            Item::new(ItemKey::num(2), "enabled"),
            Item::new(ItemKey::num(3), "selected"),
        ];
        let list = FilterList::<Item<'_>, ItemRow>::new(Id::root("picker.reset"));
        let mut state = FilterListState::default();
        list.reconcile(&mut state, &items);
        state.set_cursor(2, ItemKey::num(3));

        state.reset_selection_on_next_reconcile();
        list.reconcile(&mut state, &items);
        assert_eq!(state.cursor(), Some(ItemKey::num(2)));

        state.reset_selection_on_next_reconcile();
        list.reconcile(&mut state, &[]);
        assert_eq!(state.cursor(), None);
    }

    #[test]
    fn live_query_uses_the_shared_editor_draft_and_external_set_keeps_active_editor() {
        let items = [
            Item::new(ItemKey::num(1), "prefix"),
            Item::new(ItemKey::num(2), "other"),
        ];
        let mut state = FilterListState::default();
        state.query_input.begin("pre");
        assert!(state.query_input.splice_completion(0, "fix"));
        assert_eq!(state.query(), "prefix");

        let list = FilterList::<Item<'_>, ItemRow>::new(Id::root("picker"));
        list.reconcile(&mut state, &items);
        assert_eq!(state.matched_len(), 1);

        state.set_query("other");
        assert_eq!(state.query(), "other");
        assert!(state.query_input.is_editing());
        list.reconcile(&mut state, &items);
        assert_eq!(state.matched_len(), 1);
        assert_eq!(state.cursor(), Some(ItemKey::num(2)));
    }

    #[test]
    fn results_have_stable_child_identity_and_arbitrate_query_keys() {
        let list = FilterList::<Item<'static>, ItemRow>::new(Id::root("picker"));
        assert_eq!(list.results_id, Id::root("picker").sub("results"));
        assert_ne!(list.results_id, list.id);
        assert_eq!(
            Binding::command(BINDINGS, ActionKey::custom("filter-list.back")),
            Some(super::FilterListCmd::Back)
        );
        assert_eq!(
            Binding::command(BINDINGS, ActionKey::custom("filter-list.secondary")),
            Some(super::FilterListCmd::Secondary)
        );
        assert_eq!(
            Binding::command(BINDINGS, ActionKey::custom("filter-list.previous-scope")),
            Some(super::FilterListCmd::PrevScope)
        );
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

    #[test]
    fn explicit_container_patch_reaches_an_opaque_preserved_surface() {
        let theme = Theme::junie();
        let patch = [(Part::CONTAINER, StylePatch::new().set_bg(Role::Danger))];
        let (_, page) = with_ui(|ui| {
            let surface = ui.surface_style();
            ui.fill(AREA, surface);
            FilterList::<Item<'_>>::new(Id::root("filter-list.opaque-patch"))
                .searchable(false)
                .preserve_surface(true)
                .patch_part(&patch)
                .draw(ui, AREA, &FilterListState::default(), &[]);
        });
        assert_eq!(page[(AREA.x, AREA.y)].bg, theme.color.danger);
    }

    #[test]
    fn themed_container_channels_reach_an_opaque_preserved_surface() {
        let theme = Theme::junie().override_family(Family::PICKER, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_bg(Role::Danger)
                    .add(ratatui_core::style::Modifier::UNDERLINED),
            );
        });
        let (_, page) = with_ui_theme(&theme, |ui| {
            let surface = ui.surface_style();
            ui.fill(AREA, surface);
            FilterList::<Item<'_>>::new(Id::root("filter-list.themed-opaque"))
                .searchable(false)
                .preserve_surface(true)
                .draw(ui, AREA, &FilterListState::default(), &[]);
        });
        let cell = &page[(AREA.x, AREA.y)];
        assert_eq!(cell.bg, theme.color.danger);
        assert!(
            cell.modifier
                .contains(ratatui_core::style::Modifier::UNDERLINED)
        );
    }

    #[test]
    fn container_fill_is_not_skipped_for_an_opaque_unpreserved_area() {
        let (_, page) = with_ui(|ui| {
            ui.fill(
                AREA,
                termrock_theme::PaintStyle::new().bg(ratatui_core::style::Color::Magenta),
            );
            FilterList::<Item<'_>>::new(Id::root("filter-list.opaque-fill"))
                .searchable(false)
                .draw(ui, AREA, &FilterListState::default(), &[]);
        });
        assert_ne!(
            page[(AREA.x, AREA.y)].bg,
            ratatui_core::style::Color::Magenta
        );
    }

    #[test]
    fn reverse_scope_bindings_accept_backend_shift_and_plain_backtab_without_conflict() {
        use ratatui::crossterm::event as backend;
        use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
        use termrock_runtime::keymap::{KeyPhase, binding_conflicts};

        let converted = Input::from_crossterm(backend::Event::Key(backend::KeyEvent::new(
            backend::KeyCode::BackTab,
            backend::KeyModifiers::SHIFT,
        )))
        .expect("a pressed backend BackTab must produce input");
        let Input::Key(shifted) = converted else {
            panic!("the backend key must remain a key input");
        };
        assert_eq!(shifted.code, KeyCode::BackTab);
        assert_eq!(shifted.mods, KeyModifiers::SHIFT);
        let plain = Key {
            code: KeyCode::BackTab,
            mods: KeyModifiers::NONE,
        };
        for (key, action) in [
            (shifted, ActionKey::custom("filter-list.previous-scope")),
            (plain, ActionKey::custom("filter-list.previous-scope")),
        ] {
            let bindings: Vec<_> = super::BINDINGS
                .iter()
                .filter(|binding| binding.chord.is_some_and(|chord| chord.matches(&key)))
                .collect();
            assert_eq!(bindings.len(), 1);
            let binding = bindings.first().expect("one reverse-scope binding");
            assert_eq!(binding.action, action);
            assert_eq!(binding.cmd, super::FilterListCmd::PrevScope);
            assert!(!binding.visible);
        }
        assert!(
            binding_conflicts(
                Id::root("filter-list.reverse-scope"),
                KeyPhase::Bubble,
                super::BINDINGS,
            )
            .is_empty()
        );
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
        use termrock_core::response::Response;
        use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
        use termrock_runtime::runtime::stub::key;
        use termrock_runtime::runtime::{App, Runtime};
        use termrock_runtime::ui::Cx;

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
                let list =
                    FilterList::<Item<'static>>::new(ID).filter(super::FilterPolicy::Substring);
                list.draw_query(ui, Rect::new(0, 0, SCREEN.width, 1), &self.state);
                list.draw_results(
                    ui,
                    Rect::new(0, 1, SCREEN.width, SCREEN.height.saturating_sub(1)),
                    &self.state,
                    &self.items,
                );
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

        let paste =
            termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Paste("console".into()));
        assert!(paste.is_consumed());
        assert!(paste.is_changed());
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "console");
        assert_eq!(runtime.app().state.matched_len(), 2);

        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('j'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(3)));
        let _ = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('k'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(1)));

        let left = termrock_runtime::runtime::stub::deliver(&mut runtime, key(KeyCode::Left));
        assert!(left.is_consumed());
        assert!(left.is_changed(), "caret movement requests a repaint");
        assert!(runtime.needs_present());
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query_input.cursor_offset(), Some(6));
        let paste =
            termrock_runtime::runtime::stub::deliver(&mut runtime, Input::Paste("x".into()));
        assert!(paste.is_consumed());
        assert!(paste.is_changed());
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.query(), "consolxe");
        assert_eq!(runtime.app().state.matched_len(), 0);

        let select = termrock_runtime::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Left,
                mods: KeyModifiers::SHIFT,
            }),
        );
        assert!(select.is_consumed());
        assert!(select.is_changed());
        assert_eq!(runtime.app().state.query_input.selection(), Some(6..7));
        assert!(runtime.needs_present());
    }

    #[test]
    fn substring_filter_keeps_unicode_keys_and_reuses_match_capacity() {
        let items = [
            Item::new(ItemKey::num(11), "École Console"),
            Item::new(ItemKey::num(22), "界 Console"),
            Item::new(ItemKey::num(33), "Disabled Console").disabled(true),
        ];
        let list = FilterList::<Item<'_>>::new(Id::root("filter-list.substring-unicode"))
            .filter(super::FilterPolicy::Substring);
        let mut state = FilterListState::default();
        list.reconcile(&mut state, &items);
        let capacity = state.matches.capacity();
        state.set_query("éCOLE");
        list.reconcile(&mut state, &items);
        assert_eq!(state.matched_len(), 1);
        assert_eq!(state.cursor(), Some(ItemKey::num(11)));
        state.set_query("界");
        list.reconcile(&mut state, &items);
        assert_eq!(state.matched_len(), 1);
        assert_eq!(state.cursor(), Some(ItemKey::num(22)));
        state.set_query("disabled");
        list.reconcile(&mut state, &items);
        assert_eq!(state.cursor(), None);
        assert_eq!(state.matches.capacity(), capacity);
    }
}
