use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, shift};
use crate::measure::{Constraints, Size};
use crate::scroll::ScrollState;
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, KeySet, Reconcile, Reconciliation, SelectMode,
};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_runtime::empty::{EmptyState, Status};
use termrock_runtime::event::{Chord, KeyCode, KeyModifiers};
use termrock_runtime::focus::Focusability;
use termrock_runtime::intent::{Intent, Phase};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::layer::LayerSize;
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;
use termrock_runtime::ui::{Cx, FrameRead, Ui};
use termrock_theme::{Family, GlyphRole, Slot, StylePatch, Variant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListAction {
    Moved,
    Chose(ItemKey),
    Toggled(ItemKey),
    Activated(ItemKey),
    ToggledAll,
    LeaveBackward,
    LeaveForward,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    ExtendUp,
    ExtendDown,
    Choose,
    Activate,
    ToggleAll,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: ListCmd,
    label: &'static str,
    visible: bool,
) -> Binding<ListCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BASE: [Binding<ListCmd>; 11] = [
    b("list.up", Chord::key(KeyCode::Up), ListCmd::Up, "Up", true),
    b(
        "list.down",
        Chord::key(KeyCode::Down),
        ListCmd::Down,
        "Down",
        true,
    ),
    b(
        "list.up-vim",
        Chord::key(KeyCode::Char('k')),
        ListCmd::Up,
        "Up",
        false,
    ),
    b(
        "list.down-vim",
        Chord::key(KeyCode::Char('j')),
        ListCmd::Down,
        "Down",
        false,
    ),
    b(
        "list.page-up",
        Chord::key(KeyCode::PageUp),
        ListCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "list.page-down",
        Chord::key(KeyCode::PageDown),
        ListCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "list.home",
        Chord::key(KeyCode::Home),
        ListCmd::Home,
        "First",
        false,
    ),
    b(
        "list.end",
        Chord::key(KeyCode::End),
        ListCmd::End,
        "Last",
        false,
    ),
    b(
        "list.home-vim",
        Chord::key(KeyCode::Char('g')),
        ListCmd::Home,
        "First",
        false,
    ),
    b(
        "list.end-vim",
        Chord::key(KeyCode::Char('G')),
        ListCmd::End,
        "Last",
        false,
    ),
    b(
        "list.activate",
        Chord::key(KeyCode::Enter),
        ListCmd::Activate,
        "Open",
        true,
    ),
];

const SINGLE: [Binding<ListCmd>; 12] = [
    BASE[0],
    BASE[1],
    BASE[2],
    BASE[3],
    BASE[4],
    BASE[5],
    BASE[6],
    BASE[7],
    BASE[8],
    BASE[9],
    BASE[10],
    b(
        "list.choose",
        Chord::key(KeyCode::Char(' ')),
        ListCmd::Choose,
        "Choose",
        true,
    ),
];

const MULTI: [Binding<ListCmd>; 15] = [
    BASE[0],
    BASE[1],
    BASE[2],
    BASE[3],
    BASE[4],
    BASE[5],
    BASE[6],
    BASE[7],
    BASE[8],
    BASE[9],
    BASE[10],
    b(
        "list.toggle",
        Chord::key(KeyCode::Char(' ')),
        ListCmd::Choose,
        "Toggle",
        true,
    ),
    b(
        "list.extend-up",
        Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
        ListCmd::ExtendUp,
        "Extend up",
        false,
    ),
    b(
        "list.extend-down",
        Chord::with(KeyCode::Down, KeyModifiers::SHIFT),
        ListCmd::ExtendDown,
        "Extend down",
        false,
    ),
    b(
        "list.toggle-all",
        Chord::key(KeyCode::Char('a')),
        ListCmd::ToggleAll,
        "All",
        false,
    ),
];

const NONE: [Binding<ListCmd>; 11] = BASE;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ListState {
    core: CollectionCore,
    chosen: Option<ItemKey>,
    anchor: Option<ItemKey>,

    click_cursor: Option<ItemKey>,

    row_stride: usize,
    row_gap: u16,
}

impl ListState {
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn chosen(&self) -> Option<ItemKey> {
        self.chosen
    }

    pub const fn checked(&self) -> &KeySet {
        self.core.checked()
    }

    pub const fn checked_mut(&mut self) -> &mut KeySet {
        self.core.checked_mut()
    }

    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    pub fn choose(&mut self, key: Option<ItemKey>) {
        self.chosen = key;
    }
}

impl Reconcile for ListState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let stride = self.row_stride.max(1);
        let extent =
            row_scroll_extent(len, stride, self.row_gap, self.core.scroll().viewport_len());
        let r = self.core.reconcile_with_extent(len, extent, &key, |_| true);
        if let Some(c) = self.chosen
            && !(0..len).any(|i| key(i) == c)
        {
            self.chosen = None;
        }
        if let Some(a) = self.anchor
            && !(0..len).any(|i| key(i) == a)
        {
            self.anchor = None;
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "whole rows, boundary exit, click activation and bare row chrome are independent opt-ins"
)]
pub struct List<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    render_row: Option<ListRowRenderer<'a, T>>,
    row_height: u16,
    row_gap: u16,
    whole_rows: bool,
    select_mode: SelectMode,
    leave_at_boundary: bool,
    activate_focused_on_click: bool,
    empty: Option<EmptyState<'a>>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    pointer_item: Option<&'a dyn Fn(&T) -> bool>,
    bare: bool,
    bare_item: Option<&'a dyn Fn(&T) -> bool>,
    status: Status,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    _t: PhantomData<fn(&T)>,
}

fn row_scroll_extent(len: usize, stride: usize, gap: u16, viewport: usize) -> usize {
    let extent =
        len.saturating_mul(stride)
            .saturating_sub(if len == 0 { 0 } else { usize::from(gap) });
    let excess = extent.saturating_sub(viewport);
    if excess == 0 {
        extent
    } else {
        excess
            .div_ceil(stride)
            .saturating_mul(stride)
            .saturating_add(viewport)
    }
}

type ListRowRenderer<'a, T> = &'a dyn Fn(&mut Ui<'_>, Rect, StateFlags, ItemKey, &T);

impl<T, K, R> fmt::Debug for List<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("List")
            .field("id", &self.id)
            .field("select_mode", &self.select_mode)
            .field("empty", &self.empty)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<T> List<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        List {
            id,
            key: ByIndex,
            row: DefaultRow,
            render_row: None,
            row_height: 1,
            row_gap: 0,
            whole_rows: false,
            select_mode: SelectMode::Single,
            leave_at_boundary: false,
            activate_focused_on_click: false,
            empty: None,
            disabled_item: None,
            pointer_item: None,
            bare: false,
            bare_item: None,
            status: Status::Ready,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> List<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::ICON,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];

    pub const PREFERRED_WIDTH: u16 = 24;

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn leave_at_boundary(mut self, leave: bool) -> Self {
        self.leave_at_boundary = leave;
        self
    }

    #[must_use]
    pub const fn activate_focused_on_click(mut self, enabled: bool) -> Self {
        self.activate_focused_on_click = enabled;
        self
    }

    #[must_use]
    pub const fn row_height(mut self, height: u16) -> Self {
        self.row_height = if height == 0 { 1 } else { height };
        self
    }

    #[must_use]
    pub const fn row_gap(mut self, gap: u16) -> Self {
        self.row_gap = gap;
        self
    }

    #[must_use]
    pub const fn whole_rows(mut self, enabled: bool) -> Self {
        self.whole_rows = enabled;
        self
    }

    fn stride(&self) -> usize {
        usize::from(self.row_height).saturating_add(usize::from(self.row_gap))
    }

    fn item_at_line(&self, line: usize) -> usize {
        line.checked_div(self.stride()).unwrap_or(0)
    }

    fn extent(&self, len: usize) -> usize {
        len.saturating_mul(self.stride())
            .saturating_sub(if len == 0 {
                0
            } else {
                usize::from(self.row_gap)
            })
    }

    fn scroll_extent(&self, len: usize, viewport: usize) -> usize {
        row_scroll_extent(len, self.stride(), self.row_gap, viewport)
    }

    fn scroll_view(&self, st: &ListState, len: usize, viewport: usize) -> ScrollState {
        let mut scroll = *st.core.scroll();
        if viewport == 0 {
            scroll.set_content(self.extent(len));
            return scroll;
        }
        let reveal = scroll.pending_reveal().or_else(|| {
            (self.stride() > 1 && scroll.viewport_len() != viewport && st.core.cursor().is_some())
                .then_some(st.core.cursor_index())
        });
        let start = reveal.map(|i| i.saturating_mul(self.stride()));
        if let Some(start) = start {
            scroll.ensure_visible_on_next_layout(
                start.saturating_add(usize::from(self.row_height).min(viewport).saturating_sub(1)),
            );
        }
        scroll.apply_layout(viewport, self.scroll_extent(len, viewport));
        if let Some(start) = start {
            scroll.ensure_visible(start);
        }
        let offset = scroll.offset();
        let aligned = if start.is_some_and(|start| start >= offset) {
            offset.div_ceil(self.stride()).saturating_mul(self.stride())
        } else {
            self.item_at_line(offset).saturating_mul(self.stride())
        };
        scroll.scroll_to(aligned);
        scroll
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R> {
        List {
            id: self.id,
            key: k,
            row: self.row,
            render_row: self.render_row,
            row_height: self.row_height,
            row_gap: self.row_gap,
            whole_rows: self.whole_rows,
            select_mode: self.select_mode,
            leave_at_boundary: self.leave_at_boundary,
            activate_focused_on_click: self.activate_focused_on_click,
            empty: self.empty,
            disabled_item: self.disabled_item,
            pointer_item: self.pointer_item,
            bare: self.bare,
            bare_item: self.bare_item,
            status: self.status,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2> {
        List {
            id: self.id,
            key: self.key,
            row: r,
            render_row: self.render_row,
            row_height: self.row_height,
            row_gap: self.row_gap,
            whole_rows: self.whole_rows,
            select_mode: self.select_mode,
            leave_at_boundary: self.leave_at_boundary,
            activate_focused_on_click: self.activate_focused_on_click,
            empty: self.empty,
            disabled_item: self.disabled_item,
            pointer_item: self.pointer_item,
            bare: self.bare,
            bare_item: self.bare_item,
            status: self.status,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub fn render_row(self, renderer: ListRowRenderer<'a, T>) -> List<'a, T, K, impl RowFn<T>> {
        let mut list = self.row(|_: &T, _: &mut RowUi<'_>| {});
        list.render_row = Some(renderer);
        list
    }

    #[must_use]
    pub fn select_mode(mut self, m: SelectMode) -> Self {
        self.select_mode = m;
        self
    }

    #[must_use]
    pub fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    #[must_use]
    pub fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    #[must_use]
    pub fn pointer_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.pointer_item = Some(f);
        self
    }

    #[must_use]
    pub fn bare(mut self, yes: bool) -> Self {
        self.bare = yes;
        self
    }

    #[must_use]
    pub fn bare_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.bare_item = Some(f);
        self
    }

    #[must_use]
    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut sr = ScrollRegion::new(self.id).patch_part(self.parts);
        if let Some(p) = self.patch {
            sr = sr.patch(p);
        }
        if let Some(f) = self.ov.slot_for(Part::TRACK) {
            sr = sr.slot(Part::TRACK, f);
        } else if let Some(f) = self.ov.slot_for(Part::THUMB) {
            sr = sr.slot(Part::THUMB, f);
        }
        sr
    }

    pub const fn table(&self) -> &'static [Binding<ListCmd>] {
        match self.select_mode {
            SelectMode::Single => &SINGLE,
            SelectMode::Multi | SelectMode::Range => &MULTI,
            SelectMode::None => &NONE,
        }
    }

    fn is_disabled(&self, item: &T) -> bool {
        self.disabled_item.is_some_and(|f| f(item))
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> List<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
    }

    fn enabled_at(&self, items: &[T], i: usize) -> bool {
        items.get(i).is_some_and(|it| !self.is_disabled(it))
    }

    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(h) = hint
            && h < items.len()
            && self.key_at(items, h) == key
        {
            return Some(h);
        }
        (0..items.len()).find(|&i| self.key_at(items, i) == key)
    }

    fn move_cursor(
        &self,
        st: &mut ListState,
        items: &[T],
        to: usize,
        extend: bool,
        acc: &mut Acc<ListAction>,
    ) {
        let len = items.len();
        if len == 0 {
            return;
        }
        let to = to.min(len.saturating_sub(1));
        let key = self.key_at(items, to);
        if extend && !matches!(self.select_mode, SelectMode::Single | SelectMode::None) {
            let from = st.core.cursor_index();
            let anchor_i = st
                .anchor
                .and_then(|a| self.index_of(items, a, Some(from)))
                .unwrap_or(from);
            if st.anchor.is_none() {
                st.anchor = Some(self.key_at(items, from));
            }
            if self.select_mode == SelectMode::Range {
                st.core.checked_mut().none();
            }
            let (a, b) = (anchor_i.min(to), anchor_i.max(to));
            for i in a..=b {
                if self.enabled_at(items, i) {
                    st.core.checked_mut().insert(self.key_at(items, i));
                }
            }
            st.core.set_cursor(to, key);
            acc.action(ListAction::Toggled(key));
        } else {
            st.anchor = None;
            st.core.set_cursor(to, key);
            acc.action(ListAction::Moved);
        }
    }

    fn toggle_all(&self, st: &mut ListState, items: &[T], acc: &mut Acc<ListAction>) {
        let all = (0..items.len())
            .filter(|&i| self.enabled_at(items, i))
            .all(|i| st.core.checked().contains(self.key_at(items, i)));
        if all {
            st.core.checked_mut().none();
        } else {
            let checked = st.core.checked_mut();
            checked.all();
            for i in 0..items.len() {
                if !self.enabled_at(items, i) {
                    checked.remove(self.key_at(items, i));
                }
            }
        }
        acc.action(ListAction::ToggledAll);
    }

    fn choose(&self, st: &mut ListState, items: &[T], i: usize, acc: &mut Acc<ListAction>) {
        if !self.enabled_at(items, i) {
            acc.consumed();
            return;
        }
        let key = self.key_at(items, i);
        match self.select_mode {
            SelectMode::Single => {
                st.chosen = Some(key);
                acc.action(ListAction::Chose(key));
            }
            SelectMode::Multi | SelectMode::Range => {
                st.core.checked_mut().toggle(key);
                acc.action(ListAction::Toggled(key));
            }
            SelectMode::None => acc.consumed(),
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the keymap dispatch and the pointer phases in one drain loop"
    )]
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction> {
        st.row_stride = self.stride();
        st.row_gap = self.row_gap;
        let prior_index = st.core.cursor_index();
        let prior_cursor = st.core.cursor();
        let prior_focus = self.activate_focused_on_click
            && cx.state(self.id).contains(StateFlags::FOCUSED)
            && !cx
                .intents(self.id)
                .any(|intent| matches!(intent, Intent::FocusIn { .. }));
        let len = items.len();
        let _ = st.core.reconcile_with_extent(
            len,
            self.scroll_extent(len, st.core.scroll().viewport_len()),
            |i| self.key_at(items, i),
            |i| self.enabled_at(items, i),
        );
        if let Some(a) = st.anchor
            && self
                .index_of(items, a, Some(st.core.cursor_index()))
                .is_none()
        {
            st.anchor = None;
        }
        if let Some(c) = st.chosen
            && self
                .index_of(items, c, Some(st.core.cursor_index()))
                .is_none()
        {
            st.chosen = None;
        }
        if st.core.cursor().is_none()
            && let Some(i) = (0..len).find(|&i| self.enabled_at(items, i))
        {
            let key = self.key_at(items, i);
            st.core.set_cursor(i, key);
        }
        if self.stride() > 1 && prior_index != st.core.cursor_index() && st.core.cursor().is_some()
        {
            let index = st.core.cursor_index();
            st.core.scroll_mut().ensure_visible_on_next_layout(index);
        }
        let mut acc = Acc::<ListAction>::new();
        let viewport = cx
            .layout(self.id)
            .map_or(st.core.scroll().viewport_len(), |l| l.viewport_len);
        *st.core.scroll_mut() = self.scroll_view(st, len, viewport);
        let extent = self.scroll_extent(len, viewport);
        let before_scroll = st.core.scroll().offset();
        let region = ScrollRegion::new(self.id);
        let track_len = region.prepare(cx, st.core.scroll_mut(), extent);
        let mut bar = Response::ignored();
        for intent in cx.intents(self.id) {
            if let Intent::Wheel { delta, .. } = intent {
                let scroll = st.core.scroll_mut();
                let before = scroll.offset();
                scroll.scroll_by(isize::from(delta).saturating_mul(self.stride() as isize));
                bar |= if scroll.offset() == before {
                    Response::consumed()
                } else {
                    Response::changed()
                };
            } else {
                bar |= region.handle_intent(cx, st.core.scroll_mut(), track_len, intent);
            }
        }
        if self.stride() > 1 {
            let scroll = st.core.scroll_mut();
            let offset = scroll.offset();
            let aligned = if offset > before_scroll {
                offset.div_ceil(self.stride()).saturating_mul(self.stride())
            } else {
                self.item_at_line(offset).saturating_mul(self.stride())
            };
            scroll.scroll_to(aligned);
        }
        acc.fold(&bar);
        let page_rows = self.item_at_line(st.core.scroll().viewport_len()).max(1);
        let table = self.table();
        let mut released = false;
        for it in cx.intents(self.id) {
            match it {
                Intent::FocusOut { .. } | Intent::Cancel => {
                    st.click_cursor = None;
                }
                Intent::Binding(action) => {
                    st.click_cursor = None;
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(ListCmd::Up)
                            if self.leave_at_boundary
                                && action == crate::ActionKey::custom("list.up")
                                && len > 0
                                && cur == 0
                                && !cx.state(self.id).contains(StateFlags::DISABLED) =>
                        {
                            acc.action(ListAction::LeaveBackward);
                        }
                        Some(ListCmd::Up) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), false, &mut acc);
                        }
                        Some(ListCmd::Down)
                            if self.leave_at_boundary
                                && action == crate::ActionKey::custom("list.down")
                                && len > 0
                                && cur == len.saturating_sub(1)
                                && !cx.state(self.id).contains(StateFlags::DISABLED) =>
                        {
                            acc.action(ListAction::LeaveForward);
                        }
                        Some(ListCmd::Down) => {
                            self.move_cursor(st, items, cur.saturating_add(1), false, &mut acc);
                        }
                        Some(ListCmd::PageUp) => self.move_cursor(
                            st,
                            items,
                            cur.saturating_sub(page_rows),
                            false,
                            &mut acc,
                        ),
                        Some(ListCmd::PageDown) => self.move_cursor(
                            st,
                            items,
                            cur.saturating_add(page_rows),
                            false,
                            &mut acc,
                        ),
                        Some(ListCmd::Home) => self.move_cursor(st, items, 0, false, &mut acc),
                        Some(ListCmd::End) => {
                            self.move_cursor(st, items, usize::MAX, false, &mut acc);
                        }
                        Some(ListCmd::ExtendUp) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), true, &mut acc);
                        }
                        Some(ListCmd::ExtendDown) => {
                            self.move_cursor(st, items, cur.saturating_add(1), true, &mut acc);
                        }
                        Some(ListCmd::Choose) => self.choose(st, items, cur, &mut acc),
                        Some(ListCmd::Activate) => {
                            if self.enabled_at(items, cur) && len > 0 {
                                acc.action(ListAction::Activated(self.key_at(items, cur)));
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(ListCmd::ToggleAll) => {
                            self.toggle_all(st, items, &mut acc);
                        }
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(k),
                        },
                    pos,
                    ..
                } => {
                    released |= phase == Phase::Release;
                    if matches!(phase, Phase::DragStart | Phase::Drag | Phase::DragEnd) {
                        st.click_cursor = None;
                    }

                    let hint = cx.area(self.id).map(|a| {
                        let view = self.scroll_view(st, len, usize::from(a.height));
                        self.item_at_line(
                            view.offset()
                                .saturating_add(usize::from(pos.y.saturating_sub(a.y))),
                        )
                    });
                    let Some(i) = self.index_of(items, k, hint) else {
                        st.click_cursor = None;
                        acc.consumed();
                        continue;
                    };
                    if !items
                        .get(i)
                        .is_some_and(|item| self.pointer_item.is_none_or(|f| f(item)))
                    {
                        st.click_cursor = None;
                        acc.consumed();
                        continue;
                    }
                    if self.activate_focused_on_click && !self.enabled_at(items, i) {
                        st.click_cursor = None;
                        acc.consumed();
                        continue;
                    }
                    match phase {
                        Phase::Press => {
                            st.click_cursor = prior_cursor.filter(|cursor| {
                                self.activate_focused_on_click && prior_focus && *cursor == k
                            });
                            st.anchor = None;
                            st.core.set_cursor(i, k);
                            acc.changed();
                        }
                        Phase::Click | Phase::DoubleClick if self.activate_focused_on_click => {
                            if st.click_cursor.take() == Some(k) {
                                acc.action(ListAction::Activated(k));
                            } else {
                                self.choose(st, items, i, &mut acc);
                            }
                        }
                        Phase::Click => self.choose(st, items, i, &mut acc),
                        Phase::DoubleClick => {
                            if self.enabled_at(items, i) {
                                acc.action(ListAction::Activated(k));
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer { phase, .. } => {
                    if matches!(
                        phase,
                        Phase::Press
                            | Phase::Release
                            | Phase::DragStart
                            | Phase::Drag
                            | Phase::DragEnd
                    ) {
                        st.click_cursor = None;
                    }
                    acc.consumed();
                }
                _ => {}
            }
        }
        if released {
            st.click_cursor = None;
        }
        *st.core.scroll_mut() = self.scroll_view(st, len, viewport);
        acc.finish(self.id)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the visible rows with gutter, marker and renderer"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        let len = items.len();
        if !ui.is_inert() {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let live = PartStyle::flags(ui.state(self.id), self.status.flags());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let ov = self.ov;
        let id = self.id;
        let container = ov.style(
            ui,
            id,
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let view = self.scroll_view(st, len, usize::from(area.height));
        let surface = self.scrollbar().draw(ui, area, &view, view.content_len());
        let has_readiness = !matches!(self.status, Status::Ready);
        let content = if has_readiness {
            shift(surface, 2)
        } else {
            surface
        };
        if has_readiness {
            let icon_cell = cell_at(first_row(surface), surface.x);
            if let Some(f) = ov.slot_for(Part::ICON) {
                f(ui, icon_cell);
            } else {
                let icon = ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::ICON, live);
                match self.status {
                    Status::Busy | Status::Loading => {
                        let frames = ui.design().motion.spinner_frames;
                        let frame = frames.first().copied().unwrap_or("");
                        ui.paint_str(icon_cell, frame, icon.style);
                    }
                    Status::Error => match icon.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(icon_cell, glyph, icon.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(icon_cell, GlyphRole::Error, icon.style);
                        }
                        Slot::Clear => ui.fill(icon_cell, icon.style),
                    },
                    Status::Ready => {}

                    _ => {}
                }
            }
        }
        if len == 0 {
            let empty = self.empty.unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            });
            let mid = Rect {
                y: area.y.saturating_add(area.height / 2),
                height: area.height.saturating_sub(area.height / 2),
                ..content
            };
            if let Some(f) = ov.slot_for(Part::EMPTY) {
                f(ui, mid);
            } else {
                let inherited = ov
                    .style(ui, id, Family::LIST, Variant::DEFAULT, Part::EMPTY, live)
                    .style;
                empty.draw_inherited(ui, mid, 0, inherited);
            }
            return area;
        }
        let cursor = st.core.cursor();
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);

        let status = live
            & (StateFlags::ERROR | StateFlags::WARNING | StateFlags::BUSY | StateFlags::LOADING);
        for i in self.item_at_line(view.offset())..len {
            let row_i = i
                .saturating_mul(self.stride())
                .saturating_sub(view.offset());
            if row_i >= usize::from(content.height)
                || (self.whole_rows
                    && row_i.saturating_add(usize::from(self.row_height))
                        > usize::from(content.height))
            {
                break;
            }
            let Some(item) = items.get(i) else { break };
            let key = self.key.key(item, i);
            let is_cursor = cursor == Some(key);
            let pointer_eligible = self.pointer_item.is_none_or(|f| f(item));
            let mut flags = status;
            if is_cursor {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            let row_part = PartRef::item(Part::ROW, key);
            if pointer_eligible && hovered == Some(row_part) {
                flags |= StateFlags::HOVERED;
            }
            if (pointer_eligible && pressed == Some(row_part))
                || (pressed.is_none() && is_cursor && live.contains(StateFlags::PRESSED))
            {
                flags |= StateFlags::PRESSED;
            }
            if st.chosen == Some(key) {
                flags |= StateFlags::SELECTED;
            }
            if st.core.checked().contains(key) {
                flags |= StateFlags::CHECKED;
            }
            if self.is_disabled(item) || live.contains(StateFlags::DISABLED) {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(StateFlags::PRESSED);
            }
            let row = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: self.row_height,
            };
            if let Some(renderer) = self.render_row {
                if !row.intersection(ui.full()).is_empty() {
                    ui.with_area(content, |ui| {
                        ui.with_area(row, |ui| renderer(ui, row, flags, key, item));
                    });
                }
                if !ui.is_inert() {
                    let part = if pointer_eligible {
                        PartRef::item(Part::ROW, key)
                    } else {
                        PartRef::of(Part::BODY)
                    };
                    ui.register_part(self.id, part, row.intersection(content));
                }
                continue;
            }
            ui.with_area(content, |ui| {
                let rs = ov.style(
                    ui,
                    id,
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    flags,
                );
                ui.fill(row, rs.style);
                let bare = self.bare || self.bare_item.is_some_and(|f| f(item));
                let rest = if bare {
                    row
                } else {
                    let gutter_cell = cell_at(row, row.x);
                    if let Some(f) = ov.slot_for(Part::GUTTER) {
                        f(ui, gutter_cell);
                    } else {
                        let g =
                            ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::GUTTER, flags);
                        match g.glyph {
                            Slot::Set(glyph) => {
                                ui.glyph(gutter_cell, glyph, g.style);
                            }
                            Slot::Inherit | Slot::Clear => ui.fill(gutter_cell, g.style),
                        }
                    }
                    let marker_cell = cell_at(row, row.x.saturating_add(1));
                    if let Some(f) = ov.slot_for(Part::MARKER) {
                        f(ui, marker_cell);
                    } else {
                        let m =
                            ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::MARKER, flags);
                        match m.glyph {
                            Slot::Set(glyph) => {
                                ui.glyph(marker_cell, glyph, m.style);
                            }
                            Slot::Inherit | Slot::Clear => ui.fill(marker_cell, m.style),
                        }
                    }
                    Rect {
                        x: row.x.saturating_add(3),
                        width: row.width.saturating_sub(3),
                        ..row
                    }
                };
                if !rest.is_empty() {
                    let mut r = RowUi::new_with_container_width(
                        ui,
                        id,
                        Family::LIST,
                        Variant::DEFAULT,
                        flags,
                        key,
                        rest,
                        row.width,
                    );
                    self.row.row(item, &mut r);
                }
            });
            if !ui.is_inert() {
                let part = if pointer_eligible {
                    PartRef::item(Part::ROW, key)
                } else {
                    PartRef::of(Part::BODY)
                };
                ui.register_part(self.id, part, row.intersection(content));
            }
        }
        area
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }

    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let d = cx.design();
        let w = Self::PREFERRED_WIDTH.clamp(d.size.popup_min_width, d.size.popup_max_width);
        let h = self
            .extent(items.len())
            .min(usize::from(d.size.popup_max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        LayerSize::Fixed(w, h)
    }
}

impl<T, K, R> Bindings for List<'_, T, K, R> {
    type Cmd = ListCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ListCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use termrock_runtime::Runtime;
    use termrock_runtime::stub::Stub;
    use termrock_theme::Theme;

    const ID: Id = Id::root("list.status.tests");
    const AREA: Rect = Rect::new(0, 0, 16, 2);

    #[test]
    fn range_selection_uses_the_anchor() {
        let items = ["zero", "one", "two", "three"];
        let list = List::new(ID).select_mode(SelectMode::Range);
        let mut state = ListState::default();
        state.set_cursor(2, ItemKey::index(2));

        let mut acc = Acc::<ListAction>::new();
        list.move_cursor(&mut state, &items, 0, true, &mut acc);
        assert_eq!(state.anchor, Some(ItemKey::index(2)));
        assert!(
            (0..=2).all(|i| state.checked().contains(ItemKey::index(i))),
            "the first extension must select the anchor-to-cursor range"
        );
        assert!(!state.checked().contains(ItemKey::index(3)));

        let mut acc = Acc::<ListAction>::new();
        list.move_cursor(&mut state, &items, 1, true, &mut acc);
        assert_eq!(state.cursor(), Some(ItemKey::index(1)));
        assert!(state.checked().contains(ItemKey::index(1)));
        assert!(state.checked().contains(ItemKey::index(2)));
        assert!(
            !state.checked().contains(ItemKey::index(0)),
            "moving the cursor back must contract around the original anchor"
        );
        assert!(!state.checked().contains(ItemKey::index(3)));
    }

    #[test]
    fn select_all_selects_only_enabled_items() {
        fn disabled(item: &u8) -> bool {
            matches!(*item, 1 | 3)
        }
        let list = List::new(ID)
            .select_mode(SelectMode::Multi)
            .disabled_item(&(disabled as fn(&u8) -> bool));
        let items = [0_u8, 1, 2, 3, 4];
        let mut state = ListState::default();
        let mut acc = Acc::<ListAction>::new();
        list.toggle_all(&mut state, &items, &mut acc);
        assert_eq!(acc.finish(ID).action_ref(), Some(&ListAction::ToggledAll));
        for (index, item) in items.iter().enumerate() {
            assert_eq!(
                state.checked().contains(ItemKey::index(index)),
                !disabled(item),
                "row {item} selection did not follow its enabled state"
            );
        }
        assert_eq!(state.checked().len_in(items.len()), 3);
    }

    fn draw(status: Status) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = ListState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID)
                    .status(status)
                    .draw(ui, area, &state, &["one", "two"]);
            })
            .commit_presented();
        buffer
    }

    #[test]
    fn readiness_rail_is_conditional_root_owned_and_patchable() {
        assert!(List::<&str>::PARTS.contains(&Part::ICON));
        assert_eq!(draw(Status::Ready), draw(Status::Ready));
        let busy = draw(Status::Busy);
        let frame = Theme::junie()
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        assert_eq!(
            busy.cell(Position::new(0, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(frame)
        );

        let seen = Cell::new(None);
        let slot = |_ui: &mut Ui<'_>, area: Rect| seen.set(Some(area));
        let patch = [(Part::ICON, StylePatch::new().add(Modifier::UNDERLINED))];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = ListState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID).status(Status::Busy).patch_part(&patch).draw(
                    ui,
                    area,
                    &state,
                    &["one"],
                );
            })
            .commit_presented();
        assert!(
            buffer
                .cell(Position::new(0, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID)
                    .status(Status::Error)
                    .patch_part(&patch)
                    .slot(Part::ICON, &slot)
                    .draw(ui, area, &state, &["one"]);
            })
            .commit_presented();
        assert_eq!(seen.get(), Some(Rect::new(0, 0, 1, 1)));
    }

    #[test]
    fn chosen_marker_comes_only_from_semantic_state() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = ListState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID).draw(ui, area, &state, &["one"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(" ")
        );

        let selected = ListState {
            chosen: Some(ItemKey::index(0)),
            ..ListState::default()
        };
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID).draw(ui, area, &selected, &["one"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Chosen))
        );
    }

    #[test]
    fn bare_row_paints_the_label_at_the_row_origin() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = ListState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                List::new(ID).bare(true).draw(ui, area, &state, &["one"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(0, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("o")
        );
    }
}
