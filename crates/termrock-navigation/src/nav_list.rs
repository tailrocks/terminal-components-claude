use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn};
use crate::measure::{Constraints, Size};
use crate::scroll::ScrollState;
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_runtime::UpdateCause;
use termrock_runtime::event::{Chord, KeyCode};
use termrock_runtime::focus::Focusability;
use termrock_runtime::intent::{Intent, Phase};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::{ScrollRegion, ScrollbarGutter};
use termrock_runtime::ui::{Cx, FrameRead, Ui};
use termrock_text::width;
use termrock_theme::{CursorStyle, Family, StylePatch, Variant};

pub type BadgeFn<'a, T> = &'a dyn Fn(&T) -> Option<&str>;

type NavRowRenderer<'a, T> = &'a dyn Fn(&mut Ui<'_>, Rect, StateFlags, ItemKey, &T);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NavMode {
    #[default]
    Full,
    Collapsed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavListAction {
    Moved(ItemKey),
    Chose(ItemKey),
    EnterContent(ItemKey),
    LeaveBackward,
    LeaveForward,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavListCmd {
    Up,
    Down,
    Home,
    End,
    Choose,
    EnterContent,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: NavListCmd,
    label: &'static str,
    visible: bool,
) -> Binding<NavListCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const TABLE: [Binding<NavListCmd>; 12] = [
    b(
        "nav-list.up",
        Chord::key(KeyCode::Up),
        NavListCmd::Up,
        "Up",
        true,
    ),
    b(
        "nav-list.down",
        Chord::key(KeyCode::Down),
        NavListCmd::Down,
        "Down",
        true,
    ),
    b(
        "nav-list.up-vim",
        Chord::key(KeyCode::Char('k')),
        NavListCmd::Up,
        "Up",
        false,
    ),
    b(
        "nav-list.down-vim",
        Chord::key(KeyCode::Char('j')),
        NavListCmd::Down,
        "Down",
        false,
    ),
    b(
        "nav-list.home",
        Chord::key(KeyCode::Home),
        NavListCmd::Home,
        "First",
        false,
    ),
    b(
        "nav-list.end",
        Chord::key(KeyCode::End),
        NavListCmd::End,
        "Last",
        false,
    ),
    b(
        "nav-list.home-vim",
        Chord::key(KeyCode::Char('g')),
        NavListCmd::Home,
        "First",
        false,
    ),
    b(
        "nav-list.end-vim",
        Chord::key(KeyCode::Char('G')),
        NavListCmd::End,
        "Last",
        false,
    ),
    b(
        "nav-list.choose",
        Chord::key(KeyCode::Enter),
        NavListCmd::Choose,
        "Go",
        true,
    ),
    b(
        "nav-list.choose-space",
        Chord::key(KeyCode::Char(' ')),
        NavListCmd::Choose,
        "Go",
        false,
    ),
    b(
        "nav-list.enter-content",
        Chord::key(KeyCode::Right),
        NavListCmd::EnterContent,
        "Enter content",
        false,
    ),
    b(
        "nav-list.enter-content-vim",
        Chord::key(KeyCode::Char('l')),
        NavListCmd::EnterContent,
        "Enter content",
        false,
    ),
];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct NavListState {
    core: CollectionCore,
    current: Option<ItemKey>,
    current_index: usize,
}

impl NavListState {
    #[must_use]
    pub const fn new() -> Self {
        NavListState {
            core: CollectionCore::new(),
            current: None,
            current_index: 0,
        }
    }

    #[must_use]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    #[must_use]
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    #[must_use]
    pub const fn current(&self) -> Option<ItemKey> {
        self.current
    }

    pub const fn set_current(&mut self, key: Option<ItemKey>) {
        self.current = key;
        self.current_index = 0;
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    fn reconcile_current(&mut self, len: usize, key: &impl Fn(usize) -> ItemKey) {
        let Some(current) = self.current else {
            return;
        };
        let probe = self.current_index < len && key(self.current_index) == current;
        if probe {
            return;
        }
        let Some(i) = (0..len).find(|&i| key(i) == current) else {
            self.current = None;
            self.current_index = 0;
            return;
        };
        self.current_index = i;
    }
}

impl Reconcile for NavListState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        self.reconcile_current(len, &key);
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "Disabled admission, scrolling, scrollbar presentation and boundary exit are independent opt-in policies, not exclusive runtime states"
)]
pub struct NavList<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    mode: NavMode,
    compact: Option<bool>,
    header_indent: u16,
    marker_gap: u16,
    content_gap: u16,
    header_text_patch: Option<&'a StylePatch>,
    cursor_style: CursorStyle,
    render_row: Option<NavRowRenderer<'a, T>>,
    section: Option<&'a dyn Fn(&T) -> &str>,
    icon: Option<&'a dyn Fn(&T) -> &str>,
    badge: Option<BadgeFn<'a, T>>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    disabled: bool,
    scrollable: bool,
    scrollbar_visible: bool,
    scrollbar_gutter: ScrollbarGutter,
    leave_at_boundary: bool,
    ov: PartStyle<'a>,
    _t: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for NavList<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavList")
            .field("id", &self.id)
            .field("mode", &self.mode)
            .field("disabled", &self.disabled)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<T> NavList<'_, T, ByIndex, DefaultRow> {
    #[must_use]
    pub const fn new(id: Id) -> Self {
        NavList {
            id,
            key: ByIndex,
            row: DefaultRow,
            mode: NavMode::Full,
            compact: Some(false),
            header_indent: 1,
            marker_gap: 0,
            content_gap: 1,
            header_text_patch: None,
            cursor_style: CursorStyle::Bar,
            render_row: None,
            section: None,
            icon: None,
            badge: None,
            disabled_item: None,
            disabled: false,
            scrollable: false,
            scrollbar_visible: true,
            scrollbar_gutter: ScrollbarGutter::Auto,
            leave_at_boundary: false,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> NavList<'a, T, K, R> {
    #[must_use]
    pub const fn leave_at_boundary(mut self, leave: bool) -> Self {
        self.leave_at_boundary = leave;
        self
    }
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::ROW,
        Part::GUTTER,
        Part::MARKER,
        Part::ICON,
        Part::LABEL,
        Part::HEADER,
        Part::BADGE,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const PREFERRED_WIDTH: u16 = 24;

    pub const COLLAPSED_WIDTH: u16 = 6;

    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn update_focus_out(&self, cx: &mut Cx<'_>) -> Option<crate::response::Response<()>> {
        if cx.update_cause() != UpdateCause::Settle {
            return None;
        }

        for intent in cx.intents(self.id) {
            if !matches!(intent, Intent::FocusOut { .. }) {
                return None;
            }
        }
        Some(crate::response::Response::ignored())
    }

    #[must_use]
    pub const fn width(&self) -> u16 {
        match self.mode {
            NavMode::Full => Self::PREFERRED_WIDTH,
            NavMode::Collapsed => Self::COLLAPSED_WIDTH,
        }
    }

    #[must_use]
    pub const fn scrollable(mut self, yes: bool) -> Self {
        self.scrollable = yes;
        self
    }

    #[must_use]
    pub const fn scrollbar_visible(mut self, visible: bool) -> Self {
        self.scrollbar_visible = visible;
        self
    }

    #[must_use]
    pub const fn scrollbar_gutter(mut self, gutter: ScrollbarGutter) -> Self {
        self.scrollbar_gutter = gutter;
        self
    }

    #[must_use]
    pub const fn mode(mut self, m: NavMode) -> Self {
        self.mode = m;
        self
    }

    #[must_use]
    pub const fn compact(mut self) -> Self {
        self.compact = Some(true);
        self
    }

    #[must_use]
    pub const fn compact_when_clipped(mut self) -> Self {
        self.compact = None;
        self
    }

    #[must_use]
    pub const fn header_indent(mut self, cells: u16) -> Self {
        self.header_indent = cells;
        self
    }

    #[must_use]
    pub const fn marker_gap(mut self, cells: u16) -> Self {
        self.marker_gap = cells;
        self
    }

    #[must_use]
    pub const fn content_gap(mut self, cells: u16) -> Self {
        self.content_gap = cells;
        self
    }

    #[must_use]
    pub const fn header_text_patch(mut self, patch: &'a StylePatch) -> Self {
        self.header_text_patch = Some(patch);
        self
    }

    #[must_use]
    pub const fn cursor_style(mut self, style: CursorStyle) -> Self {
        self.cursor_style = style;
        self
    }

    #[must_use]
    pub fn render_row(mut self, renderer: NavRowRenderer<'a, T>) -> Self {
        self.render_row = Some(renderer);
        self
    }

    #[must_use]
    pub const fn section(mut self, f: &'a dyn Fn(&T) -> &str) -> Self {
        self.section = Some(f);
        self
    }

    #[must_use]
    pub const fn icon(mut self, f: &'a dyn Fn(&T) -> &str) -> Self {
        self.icon = Some(f);
        self
    }

    #[must_use]
    pub const fn badge(mut self, f: BadgeFn<'a, T>) -> Self {
        self.badge = Some(f);
        self
    }

    #[must_use]
    pub const fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> NavList<'a, T, K2, R> {
        NavList {
            id: self.id,
            key: k,
            row: self.row,
            mode: self.mode,
            compact: self.compact,
            header_indent: self.header_indent,
            marker_gap: self.marker_gap,
            content_gap: self.content_gap,
            header_text_patch: self.header_text_patch,
            cursor_style: self.cursor_style,
            render_row: self.render_row,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            scrollable: self.scrollable,
            scrollbar_visible: self.scrollbar_visible,
            scrollbar_gutter: self.scrollbar_gutter,
            leave_at_boundary: self.leave_at_boundary,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> NavList<'a, T, K, R2> {
        NavList {
            id: self.id,
            key: self.key,
            row: r,
            mode: self.mode,
            compact: self.compact,
            header_indent: self.header_indent,
            marker_gap: self.marker_gap,
            content_gap: self.content_gap,
            header_text_patch: self.header_text_patch,
            cursor_style: self.cursor_style,
            render_row: self.render_row,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            scrollable: self.scrollable,
            scrollbar_visible: self.scrollbar_visible,
            scrollbar_gutter: self.scrollbar_gutter,
            leave_at_boundary: self.leave_at_boundary,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    const fn derived(&self) -> StateFlags {
        if self.disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::empty()
        }
    }

    fn section_of<'i>(&self, item: &'i T) -> &'i str {
        self.section.map_or("", |f| f(item))
    }

    fn is_disabled(&self, item: &T) -> bool {
        self.disabled || self.disabled_item.is_some_and(|f| f(item))
    }

    fn table(&self) -> &'static [Binding<NavListCmd>] {
        if self.disabled { &[] } else { &TABLE }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> NavList<'_, T, K, R> {
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

    fn seek(&self, items: &[T], from: usize, forward: bool) -> Option<usize> {
        if forward {
            (from..items.len()).find(|&i| self.enabled_at(items, i))
        } else {
            (0..=from.min(items.len().saturating_sub(1)))
                .rev()
                .find(|&i| self.enabled_at(items, i))
        }
    }

    fn move_cursor(
        &self,
        st: &mut NavListState,
        items: &[T],
        from: usize,
        forward: bool,
        acc: &mut Acc<NavListAction>,
    ) {
        match self.seek(items, from, forward) {
            Some(i) => {
                let key = self.key_at(items, i);
                if st.core.cursor() == Some(key) {
                    acc.consumed();
                } else {
                    st.core.set_cursor(i, key);
                    acc.action(NavListAction::Moved(key));
                }
            }
            None => acc.consumed(),
        }
    }

    fn choose(&self, st: &mut NavListState, items: &[T], i: usize, acc: &mut Acc<NavListAction>) {
        if !self.enabled_at(items, i) {
            acc.consumed();
            return;
        }
        let key = self.key_at(items, i);
        st.core.set_cursor(i, key);
        st.current = Some(key);
        st.current_index = i;
        acc.action(NavListAction::Chose(key));
    }

    fn step_cursor(
        &self,
        st: &mut NavListState,
        items: &[T],
        forward: bool,
        acc: &mut Acc<NavListAction>,
    ) {
        let current = st.core.cursor_index();
        let next = if forward {
            self.seek(items, current.saturating_add(1), true)
        } else {
            current
                .checked_sub(1)
                .and_then(|from| self.seek(items, from, false))
        };
        if let Some(index) = next {
            let key = self.key_at(items, index);
            if st.core.cursor() == Some(key) {
                acc.consumed();
            } else {
                st.core.set_cursor(index, key);
                acc.action(NavListAction::Moved(key));
            }
        } else if self.leave_at_boundary && self.enabled_at(items, current) {
            acc.action(if forward {
                NavListAction::LeaveForward
            } else {
                NavListAction::LeaveBackward
            });
        } else {
            acc.consumed();
        }
    }

    fn enter_content(
        &self,
        st: &mut NavListState,
        items: &[T],
        i: usize,
        acc: &mut Acc<NavListAction>,
    ) {
        if !self.enabled_at(items, i) {
            acc.consumed();
            return;
        }
        let key = self.key_at(items, i);
        st.core.set_cursor(i, key);
        st.current = Some(key);
        st.current_index = i;
        acc.action(NavListAction::EnterContent(key));
    }

    fn prepare_state(&self, cx: &Cx<'_>, st: &mut NavListState, items: &[T]) -> usize {
        let len = items.len();
        let area = cx.area(self.id).unwrap_or_default();
        let compact = self.is_compact(area, items);
        let before_cursor = (st.core.cursor(), st.core.cursor_index());
        let extent = if self.scrollable {
            self.extent(items, compact)
        } else {
            len
        };
        let _ = st.core.reconcile_with_extent(
            len,
            extent,
            |i| self.key_at(items, i),
            |i| self.enabled_at(items, i),
        );
        st.reconcile_current(len, &|i| self.key_at(items, i));
        if st.core.cursor().is_none()
            && let Some(i) = self.seek(items, 0, true)
        {
            st.core.set_cursor(i, self.key_at(items, i));
        }
        if self.scrollable
            && st.core.cursor().is_some()
            && !self.enabled_at(items, st.core.cursor_index())
        {
            if let Some(i) = self
                .seek(items, st.core.cursor_index(), true)
                .or_else(|| self.seek(items, st.core.cursor_index(), false))
            {
                st.core.set_cursor(i, self.key_at(items, i));
            } else {
                st.core.clear_cursor();
            }
        }
        if self.scrollable
            && before_cursor != (st.core.cursor(), st.core.cursor_index())
            && st.core.cursor().is_some()
        {
            let index = st.core.cursor_index();
            st.core.scroll_mut().ensure_visible_on_next_layout(index);
        }
        if self.scrollable {
            let view = self.scroll_view(st, items, area, compact);
            *st.core.scroll_mut() = view;
        }
        extent
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut NavListState,
        items: &[T],
    ) -> Response<NavListAction> {
        if self.disabled {
            return Response::ignored();
        }
        let extent = self.prepare_state(cx, st, items);
        let mut acc = Acc::<NavListAction>::new();
        if self.scrollable {
            acc.fold(&self.scrollbar().update(cx, st.core.scroll_mut(), extent));
        }
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(NavListCmd::Up) => {
                            self.step_cursor(st, items, false, &mut acc);
                        }
                        Some(NavListCmd::Down) => {
                            self.step_cursor(st, items, true, &mut acc);
                        }
                        Some(NavListCmd::Home) => self.move_cursor(st, items, 0, true, &mut acc),
                        Some(NavListCmd::End) => {
                            self.move_cursor(st, items, usize::MAX, false, &mut acc);
                        }
                        Some(NavListCmd::Choose) => self.choose(st, items, cur, &mut acc),
                        Some(NavListCmd::EnterContent) => {
                            self.enter_content(st, items, cur, &mut acc);
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
                    ..
                } => {
                    let Some(i) = self.index_of(items, k, None) else {
                        acc.consumed();
                        continue;
                    };
                    match phase {
                        Phase::Press => {
                            if self.enabled_at(items, i) {
                                st.core.set_cursor(i, k);
                                acc.changed();
                            } else {
                                acc.consumed();
                            }
                        }
                        Phase::Click | Phase::DoubleClick => self.choose(st, items, i, &mut acc),
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    fn rows<'b>(
        &'b self,
        items: &'b [T],
        compact: bool,
    ) -> impl Iterator<Item = (usize, usize, Option<(usize, &'b str)>)> + 'b {
        let mut next = 0_usize;
        let mut previous = None;
        items.iter().enumerate().map(move |(i, item)| {
            let section = self.section_of(item);
            let mut header = None;
            if !compact && previous != Some(section) {
                next = next.saturating_add(usize::from(previous.is_some()));
                if self.mode == NavMode::Full && !section.is_empty() {
                    header = Some((next, section));
                    next = next.saturating_add(1);
                }
            }
            previous = Some(section);
            let row = next;
            next = next.saturating_add(1);
            (i, row, header)
        })
    }

    fn extent(&self, items: &[T], compact: bool) -> usize {
        self.rows(items, compact)
            .last()
            .map_or(0, |(_, row, _)| row.saturating_add(1))
    }

    fn visual_row(&self, items: &[T], compact: bool, index: usize) -> Option<usize> {
        self.rows(items, compact)
            .find_map(|(i, row, _)| (i == index).then_some(row))
    }

    fn scroll_view(
        &self,
        st: &NavListState,
        items: &[T],
        area: Rect,
        compact: bool,
    ) -> ScrollState {
        let mut scroll = *st.core.scroll();
        if (scroll.pending_reveal().is_some() || scroll.viewport_len() != usize::from(area.height))
            && let Some(row) = st
                .core
                .cursor()
                .and_then(|_| self.visual_row(items, compact, st.core.cursor_index()))
        {
            scroll.ensure_visible_on_next_layout(row);
        }
        scroll.apply_layout(usize::from(area.height), self.extent(items, compact));
        scroll
    }

    fn scrollbar(&self) -> ScrollRegion<'_> {
        ScrollRegion::new(self.id)
            .scrollbar_visible(self.scrollbar_visible)
            .scrollbar_gutter(self.scrollbar_gutter)
            .inherit_family(Family::LIST)
            .with_overrides(self.ov)
    }

    fn is_compact(&self, area: Rect, items: &[T]) -> bool {
        self.compact
            .unwrap_or_else(|| self.extent(items, false) > usize::from(area.height))
    }

    fn register(&self, ui: &mut Ui<'_>, area: Rect) -> StateFlags {
        if !ui.is_inert() {
            ui.register_control(
                self.id,
                area,
                if self.disabled {
                    Focusability::Disabled
                } else {
                    Focusability::Focusable
                },
            );
        }
        let live = PartStyle::flags(ui.state(self.id), self.derived());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        live
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &NavListState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        let live = self.register(ui, area);
        let container = self.ov.style(
            ui,
            self.id,
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(
                StateFlags::FOCUSED
                    | StateFlags::PRESSED
                    | StateFlags::SELECTED
                    | StateFlags::HOVERED,
            ),
        );
        let compact = self.is_compact(area, items);
        let view = if self.scrollable {
            self.scroll_view(st, items, area, compact)
        } else {
            *st.core.scroll()
        };
        let content = if self.scrollable {
            self.scrollbar().draw(ui, area, &view, view.content_len())
        } else {
            area
        };
        ui.fill(content, container.style);
        let offset = if self.scrollable { view.offset() } else { 0 };
        let limit = offset.saturating_add(usize::from(content.height));
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        for (i, visual_row, header) in self.rows(items, compact) {
            if let Some((row, text)) = header
                && (offset..limit).contains(&row)
            {
                self.paint_header(
                    ui,
                    row_at(
                        content,
                        content.y.saturating_add(row.saturating_sub(offset) as u16),
                    ),
                    text,
                    live,
                );
            }
            if visual_row >= limit {
                break;
            }
            if visual_row < offset {
                continue;
            }
            let y = content
                .y
                .saturating_add(visual_row.saturating_sub(offset) as u16);
            let Some(item) = items.get(i) else {
                continue;
            };
            let key = self.key_at(items, i);
            let mut flags = StateFlags::empty();
            let is_cursor = st.core.cursor() == Some(key);
            if is_cursor {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            let row_part = PartRef::item(Part::ROW, key);
            if hovered == Some(row_part) {
                flags |= live & StateFlags::HOVERED;
            }
            if pressed == Some(row_part)
                || (pressed.is_none() && is_cursor && live.contains(StateFlags::PRESSED))
            {
                flags |= StateFlags::PRESSED;
            }
            if st.current == Some(key) {
                flags |= StateFlags::SELECTED;
            }
            if self.is_disabled(item) {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(StateFlags::PRESSED | StateFlags::HOVERED);
            }
            let rect = row_at(content, y);
            if let Some(renderer) = self.render_row {
                ui.with_area(rect, |ui| renderer(ui, rect, flags, key, item));
            } else {
                self.paint_row(ui, rect, flags, key, item);
            }
            if !ui.is_inert() && !self.is_disabled(item) {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), rect);
            }
        }
        area
    }

    fn paint_header(&self, ui: &mut Ui<'_>, rect: Rect, text: &str, live: StateFlags) {
        if let Some(f) = self.ov.slot_for(Part::HEADER) {
            f(ui, rect);
            return;
        }
        let h = self.ov.style(
            ui,
            self.id,
            Family::LIST,
            Variant::DEFAULT,
            Part::HEADER,
            live.difference(
                StateFlags::FOCUSED
                    | StateFlags::PRESSED
                    | StateFlags::SELECTED
                    | StateFlags::HOVERED,
            ),
        );
        ui.fill(rect, h.style);
        let inner = Rect {
            x: rect.x.saturating_add(self.header_indent),
            width: rect.width.saturating_sub(self.header_indent),
            ..rect
        };
        if let Some(text_patch) = self.header_text_patch {
            let patch = self
                .ov
                .part_patch(Part::HEADER)
                .unwrap_or_default()
                .merge(*text_patch);

            let text_style = ui.style_patched(
                Family::LIST,
                Variant::DEFAULT,
                Part::HEADER,
                live.difference(
                    StateFlags::FOCUSED
                        | StateFlags::PRESSED
                        | StateFlags::SELECTED
                        | StateFlags::HOVERED,
                ),
                &patch,
            );
            ui.paint_str(inner, text, text_style.style);
        } else {
            ui.paint_str(inner, text, h.style);
        }
    }

    fn paint_row(&self, ui: &mut Ui<'_>, rect: Rect, flags: StateFlags, key: ItemKey, item: &T) {
        let mut row = RowUi::new_with_author_style(
            ui,
            self.id,
            Family::LIST,
            Variant::DEFAULT,
            flags,
            key,
            rect,
            self.ov,
        )
        .with_author_parts(Self::PARTS)
        .with_cursor_style(self.cursor_style);
        row.leading_part(Part::GUTTER, 1, 0, |_| {});
        row.leading_part(Part::MARKER, 1, self.marker_gap, |_| {});
        let icon = self.icon.map(|f| f(item));
        row.leading_part(Part::ICON, 1, self.content_gap, |cell| {
            if let Some(icon) = icon {
                cell.text(icon);
            }
        });
        if self.mode == NavMode::Collapsed {
            return;
        }
        let badge = self.badge.and_then(|f| f(item)).filter(|s| !s.is_empty());
        if let Some(text) = badge {
            let available = row.remaining_width();
            let badge_width = width(text).min(available);
            let gap = u16::from(badge_width < available);
            row.trailing_part(Part::BADGE, badge_width, gap, |cell| {
                cell.text(text);
            });
        }
        if row.has_space() {
            row.begin_content();
            self.row.row(item, &mut row);
        }
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (
                match self.mode {
                    NavMode::Full => 11_u16
                        .saturating_add(self.marker_gap)
                        .saturating_add(self.content_gap),
                    NavMode::Collapsed => 4_u16.saturating_add(self.marker_gap),
                },
                1,
            ),
            preferred: (self.width(), c.max.1),
        }
        .fit(c)
    }
}

const fn row_at(area: Rect, y: u16) -> Rect {
    Rect {
        x: area.x,
        y,
        width: area.width,
        height: if y >= area.y && y < area.bottom() {
            1
        } else {
            0
        },
    }
}

impl<T, K, R> Bindings for NavList<'_, T, K, R> {
    type Cmd = NavListCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<NavListCmd>] {
        self.table()
    }
}
