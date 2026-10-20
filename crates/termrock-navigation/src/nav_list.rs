use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at};
use crate::measure::{Constraints, Size};
use crate::scroll::ScrollState;
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_runtime::event::{Chord, KeyCode};
use termrock_runtime::focus::Focusability;
use termrock_runtime::intent::{Intent, Phase};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;
use termrock_runtime::ui::{Cx, FrameRead, Ui};
use termrock_text::width;
use termrock_theme::{CursorStyle, Family, Slot, StylePatch, Variant};

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
    content_gap: u16,
    cursor_style: CursorStyle,
    render_row: Option<NavRowRenderer<'a, T>>,
    section: Option<&'a dyn Fn(&T) -> &str>,
    icon: Option<&'a dyn Fn(&T) -> &str>,
    badge: Option<BadgeFn<'a, T>>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    disabled: bool,
    scrollable: bool,
    scrollbar_visible: bool,
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
            content_gap: 1,
            cursor_style: CursorStyle::Bar,
            render_row: None,
            section: None,
            icon: None,
            badge: None,
            disabled_item: None,
            disabled: false,
            scrollable: false,
            scrollbar_visible: true,
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

    /// Set the number of cells between the fixed leading slots and row content.
    ///
    /// The default of one preserves the existing content origin at `x + 4`.
    /// A gap of zero starts content at `x + 3`; gutter, marker and icon slots
    /// keep their existing positions.
    #[must_use]
    pub const fn content_gap(mut self, cells: u16) -> Self {
        self.content_gap = cells;
        self
    }

    /// Choose how the active row is presented. The default shows the theme's
    /// focus gutter; `Fill` uses the shared row-fill treatment.
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
            content_gap: self.content_gap,
            cursor_style: self.cursor_style,
            render_row: self.render_row,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            scrollable: self.scrollable,
            scrollbar_visible: self.scrollbar_visible,
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
            content_gap: self.content_gap,
            cursor_style: self.cursor_style,
            render_row: self.render_row,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            scrollable: self.scrollable,
            scrollbar_visible: self.scrollbar_visible,
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
        ui.paint_str(inner, text, h.style);
    }

    fn paint_row(&self, ui: &mut Ui<'_>, rect: Rect, flags: StateFlags, key: ItemKey, item: &T) {
        let rs = self.ov.style(
            ui,
            self.id,
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            flags,
        );
        ui.fill(rect, rs.style);
        self.paint_cell_part(ui, cell_at(rect, rect.x), Part::GUTTER, flags, None);
        self.paint_cell_part(
            ui,
            cell_at(rect, rect.x.saturating_add(1)),
            Part::MARKER,
            flags,
            None,
        );
        let icon = self.icon.map(|f| f(item));
        self.paint_cell_part(
            ui,
            cell_at(rect, rect.x.saturating_add(2)),
            Part::ICON,
            flags,
            icon,
        );
        if self.mode == NavMode::Collapsed {
            return;
        }
        let content_offset = 3_u16.saturating_add(self.content_gap);
        let body = Rect {
            x: rect.x.saturating_add(content_offset),
            width: rect.width.saturating_sub(content_offset),
            ..rect
        };
        if body.is_empty() {
            return;
        }
        let badge = self.badge.and_then(|f| f(item)).filter(|s| !s.is_empty());
        let (row_body, badge_area) = badge.map_or((body, Rect::ZERO), |text| {
            let badge_width = width(text).min(body.width);
            let gap = u16::from(badge_width < body.width);
            (
                Rect {
                    width: body.width.saturating_sub(badge_width).saturating_sub(gap),
                    ..body
                },
                Rect {
                    x: body.right().saturating_sub(badge_width),
                    width: badge_width,
                    ..body
                },
            )
        });
        if !row_body.is_empty() {
            let mut r = RowUi::new_with_patches(
                ui,
                self.id,
                Family::LIST,
                Variant::DEFAULT,
                flags,
                key,
                row_body,
                self.ov.part_patch(Part::CONTAINER),
                self.ov.part_patch(Part::LABEL),
            )
            .with_cursor_style(self.cursor_style);
            self.row.row(item, &mut r);
        }
        if let Some(text) = badge {
            self.paint_cell_part(ui, badge_area, Part::BADGE, flags, Some(text));
        }
    }

    fn paint_cell_part(
        &self,
        ui: &mut Ui<'_>,
        cell: Rect,
        part: Part,
        flags: StateFlags,
        text: Option<&str>,
    ) {
        if let Some(f) = self.ov.slot_for(part) {
            f(ui, cell);
            return;
        }
        let r = self
            .ov
            .style(ui, self.id, Family::LIST, Variant::DEFAULT, part, flags);
        match r.glyph {
            Slot::Set(g) => {
                ui.glyph(cell, g, r.style);
            }
            Slot::Inherit => match text {
                Some(t) if !t.is_empty() => {
                    ui.fill(cell, r.style);
                    ui.paint_str(cell, t, r.style);
                }
                _ => ui.fill(cell, r.style),
            },
            Slot::Clear => ui.fill(cell, r.style),
        }
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (
                match self.mode {
                    NavMode::Full => 11_u16.saturating_add(self.content_gap),
                    NavMode::Collapsed => 4,
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

#[cfg(test)]
mod tests {
    use core::cell::{Cell, RefCell};

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::{Acc, NavList, NavListAction, NavListState, NavMode, TABLE};
    use crate::action::ActionKey;
    use termrock_core::id::{Id, ItemKey, Part, PartRef};
    use termrock_core::response::{Response, StateFlags};
    use termrock_runtime::keymap::Binding;
    use termrock_theme::{StylePatch, Theme};

    use termrock_collections::collection::Reconcile;
    use termrock_runtime::rowui::RowUi;
    use termrock_runtime::ui::cx::LastFrame;
    use termrock_runtime::ui::{Cx, FrameState, Ui, UiCore};

    const NAV: Id = Id::root("nav.tests");

    #[derive(Clone, Copy, Debug)]
    struct E(&'static str, &'static str, &'static str, bool);

    impl core::fmt::Display for E {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.0)
        }
    }

    fn section(e: &E) -> &str {
        e.1
    }
    fn icon(e: &E) -> &str {
        e.2
    }
    fn off(e: &E) -> bool {
        e.3
    }
    fn keyed(e: &E) -> ItemKey {
        ItemKey::text(e.0)
    }
    fn badge(e: &E) -> Option<&str> {
        (!e.0.is_empty()).then_some("9")
    }

    const ITEMS: [E; 5] = [
        E("Tasks", "Workspace", "T", false),
        E("Runs", "Workspace", "R", true),
        E("Branches", "Workspace", "B", false),
        E("Members", "Project", "M", false),
        E("Billing", "Project", "$", true),
    ];

    fn rows(
        list: &NavList<'_, E, fn(&E) -> ItemKey>,
        st: &NavListState,
        w: u16,
        items: &[E],
    ) -> Vec<String> {
        let area = Rect {
            x: 0,
            y: 0,
            width: w,
            height: 12,
        };
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let previous = LastFrame::default();
        {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &previous);
            list.draw(&mut ui, area, st, items);
        }
        (0..area.height)
            .map(|y| {
                (0..w)
                    .filter_map(|x| page.cell((x, y)).map(|c| c.symbol().to_owned()))
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect()
    }

    fn contains_symbol(page: &Buffer, area: Rect, symbol: &str) -> bool {
        (area.y..area.bottom()).any(|y| {
            (area.x..area.right()).any(|x| page.cell((x, y)).is_some_and(|c| c.symbol() == symbol))
        })
    }

    fn modifier_at_symbol(page: &Buffer, area: Rect, symbol: &str) -> Option<Modifier> {
        (area.y..area.bottom()).find_map(|y| {
            (area.x..area.right()).find_map(|x| {
                page.cell((x, y))
                    .filter(|c| c.symbol() == symbol)
                    .map(|c| c.modifier)
            })
        })
    }

    fn list<'a>() -> NavList<'a, E, fn(&E) -> ItemKey> {
        NavList::new(NAV)
            .section(&section)
            .icon(&icon)
            .disabled_item(&off)
            .key(keyed as fn(&E) -> ItemKey)
    }

    struct NavApp {
        nav: NavList<'static, E, fn(&E) -> ItemKey>,
        state: NavListState,
        last: Option<NavListAction>,
        consumed: bool,
    }

    impl crate::runtime::App for NavApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = self.nav.update(cx, &mut self.state, &ITEMS);
            self.last = response.action_ref().copied();
            self.consumed = response.is_consumed();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.nav
                .draw(ui, Rect::new(0, 0, 24, 12), &self.state, &ITEMS);
        }
    }

    fn nav_runtime(
        nav: NavList<'static, E, fn(&E) -> ItemKey>,
        state: NavListState,
    ) -> crate::runtime::Runtime<NavApp> {
        let mut rt = crate::runtime::Runtime::new(
            NavApp {
                nav,
                state,
                last: None,
                consumed: false,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 12));
        for _ in 0..16 {
            rt.draw_buffer(Rect::new(0, 0, 24, 12), &mut buf)
                .commit_presented();
            if !rt.needs_settle() {
                return rt;
            }
            let _ = rt.settle();
        }
        panic!("nav fixture focus did not settle");
    }

    fn update_action(
        nav: NavList<'static, E, fn(&E) -> ItemKey>,
        state: NavListState,
        action: ActionKey,
    ) -> (Option<NavListAction>, bool, NavListState, Option<Id>) {
        use termrock_runtime::stub::{deliver, key};
        let chord = TABLE
            .iter()
            .find(|b| b.action == action)
            .and_then(|b| b.chord)
            .unwrap_or_else(|| panic!("{action:?} declares no default chord"));
        let mut rt = nav_runtime(nav, state);
        let _ = deliver(&mut rt, key(chord.code));
        let app = rt.app();
        (app.last, app.consumed, app.state.clone(), rt.focus())
    }

    fn update_click(
        nav: NavList<'static, E, fn(&E) -> ItemKey>,
        state: NavListState,
        key: ItemKey,
    ) -> (Option<NavListAction>, NavListState) {
        use termrock_runtime::event::MouseKind;
        use termrock_runtime::stub::{deliver, mouse};
        let mut rt = nav_runtime(nav, state);
        let area = rt
            .area_of_part(NAV, PartRef::item(Part::ROW, key))
            .expect("the clicked row was laid out");
        let pos = Position::new(area.x + area.width / 2, area.y + area.height / 2);
        let _ = deliver(&mut rt, mouse(MouseKind::Down, pos.x, pos.y));
        let _ = deliver(&mut rt, mouse(MouseKind::Up, pos.x, pos.y));
        let app = rt.app();
        (app.last, app.state.clone())
    }

    #[test]
    fn steady_current_reconciliation_is_constant_time_and_invalidation_repairs_reorder() {
        use std::rc::Rc;

        use termrock_runtime::event::Input;

        struct SteadyApp<K> {
            nav: NavList<'static, u64, K>,
            state: NavListState,
            items: Vec<u64>,
            last: Option<NavListAction>,
        }

        impl<K: Fn(&u64) -> ItemKey> crate::runtime::App for SteadyApp<K> {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                let response = self.nav.update(cx, &mut self.state, &self.items);
                self.last = response.action_ref().copied();
                response.erase()
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                self.nav
                    .draw(ui, Rect::new(0, 0, 24, 4), &self.state, &self.items);
            }
        }

        let accesses = Rc::new(Cell::new(0usize));
        let key_of = {
            let accesses = Rc::clone(&accesses);
            move |item: &u64| {
                accesses.set(accesses.get().saturating_add(1));
                ItemKey::num(*item)
            }
        };
        let items: Vec<u64> = (0..100_000).collect();
        let middle = items.len() / 2;
        let current = ItemKey::num(items[middle]);
        let mut state = NavListState::new();
        state.set_cursor(middle, current);
        state.set_current(Some(current));

        let mut rt = crate::runtime::Runtime::new(
            SteadyApp {
                nav: NavList::new(NAV).key(key_of),
                state,
                items,
                last: None,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 4));
        rt.draw_buffer(Rect::new(0, 0, 24, 4), &mut buf)
            .commit_presented();
        let _ = rt.settle();

        accesses.set(0);

        let _ = rt.handle(Input::Tick);
        assert_eq!(rt.app().last, None);
        assert!(
            accesses.get() <= 3,
            "steady update must probe at most the stamp endpoints and cached current"
        );

        rt.app_mut().items.swap(middle, middle.saturating_add(1));
        rt.app_mut().state.invalidate();
        let _ = rt.handle(Input::Tick);
        assert_eq!(rt.app().state.cursor(), Some(current));
        assert_eq!(rt.app().state.current(), Some(current));

        accesses.set(0);
        let _ = rt.handle(Input::Tick);
        assert!(
            accesses.get() <= 3,
            "reordered state must return to cached steady probes"
        );
    }

    #[test]
    fn the_cursor_skips_disabled_entries_in_both_directions() {
        let nav = list();
        let mut st = NavListState::new();
        st.set_cursor(0, ItemKey::text("Tasks"));

        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 1, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Branches")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            Some(&NavListAction::Moved(ItemKey::text("Branches")))
        );

        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 3, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            Some(&NavListAction::Moved(ItemKey::text("Members")))
        );

        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 4, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            None,
            "a cursor that cannot move must not report Moved"
        );

        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 2, false, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Branches")));
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 1, false, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Tasks")));
    }

    #[test]
    fn boundary_commands_are_consumed_without_false_moved_actions() {
        let mut st = NavListState::new();
        st.set_cursor(0, ItemKey::text("Tasks"));
        for action in [
            ActionKey::custom("nav-list.up"),
            ActionKey::custom("nav-list.home"),
            ActionKey::custom("nav-list.home-vim"),
        ] {
            let (action, consumed, state_after, _) = update_action(list(), st, action);
            st = state_after;
            assert!(consumed);
            assert_eq!(action, None);
            assert_eq!(st.cursor(), Some(ItemKey::text("Tasks")));
        }
    }

    #[test]
    fn right_and_plain_l_enter_content_while_choose_inputs_stay_distinct() {
        for action in [
            ActionKey::custom("nav-list.enter-content"),
            ActionKey::custom("nav-list.enter-content-vim"),
        ] {
            let mut initial = NavListState::new();
            initial.set_cursor(2, ItemKey::text("Branches"));
            let (action, _, st, focus) = update_action(list(), initial, action);
            assert_eq!(
                action.as_ref(),
                Some(&NavListAction::EnterContent(ItemKey::text("Branches")))
            );
            assert_eq!(st.current(), Some(ItemKey::text("Branches")));

            assert_eq!(focus, Some(NAV));
        }

        for action in [
            ActionKey::custom("nav-list.choose"),
            ActionKey::custom("nav-list.choose-space"),
        ] {
            let mut initial = NavListState::new();
            initial.set_cursor(2, ItemKey::text("Branches"));
            let (action, _, _, _) = update_action(list(), initial, action);
            assert_eq!(
                action.as_ref(),
                Some(&NavListAction::Chose(ItemKey::text("Branches")))
            );
        }

        assert_eq!(
            Binding::command(&TABLE, ActionKey::custom("nav-list.unknown")),
            None
        );

        let (action, st) = update_click(list(), NavListState::new(), ItemKey::text("Members"));
        assert_eq!(
            action.as_ref(),
            Some(&NavListAction::Chose(ItemKey::text("Members")))
        );
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(st.current(), Some(ItemKey::text("Members")));
    }

    #[test]
    fn moving_the_cursor_never_navigates() {
        let nav = list();
        let mut st = NavListState::new();
        st.set_cursor(0, ItemKey::text("Tasks"));
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 2, true, &mut acc);
        assert_eq!(st.current(), None, "arrowing navigated");

        let mut acc = Acc::<NavListAction>::new();
        nav.choose(&mut st, &ITEMS, 2, &mut acc);
        assert_eq!(
            acc.finish(NAV).action_ref(),
            Some(&NavListAction::Chose(ItemKey::text("Branches")))
        );
        assert_eq!(st.current(), Some(ItemKey::text("Branches")));

        let mut acc = Acc::<NavListAction>::new();
        nav.choose(&mut st, &ITEMS, 4, &mut acc);
        assert_eq!(acc.finish(NAV).action_ref(), None);
        assert_eq!(st.current(), Some(ItemKey::text("Branches")));
    }

    #[test]
    fn a_section_heading_is_painted_once_per_change() {
        let nav = list();
        let st = NavListState::new();
        let painted = rows(&nav, &st, 24, &ITEMS);
        let headings: Vec<usize> = painted
            .iter()
            .enumerate()
            .filter(|(_, r)| r.contains("Workspace") || r.contains("Project"))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(headings.len(), 2, "one heading per section: {painted:?}");
        assert_eq!(headings.first(), Some(&0), "no blank row before the first");

        let second = headings.get(1).copied().unwrap_or_default();
        assert!(second >= 2);
        assert_eq!(
            painted.get(second.saturating_sub(1)).map(String::as_str),
            Some(""),
            "the second heading has no separator above it: {painted:?}"
        );

        for e in ITEMS {
            assert_eq!(
                painted.iter().filter(|r| r.contains(e.0)).count(),
                1,
                "{} is not painted exactly once: {painted:?}",
                e.0
            );
        }
    }

    #[test]
    fn the_collapsed_mode_paints_the_icon_and_not_the_label() {
        let st = NavListState::new();
        let full = rows(&list(), &st, 24, &ITEMS);
        assert!(full.iter().any(|r| r.contains("Branches")));

        let nav = list().mode(NavMode::Collapsed);
        let narrow = rows(&nav, &st, 6, &ITEMS);
        assert!(
            narrow.iter().all(|r| !r.contains("Bra")),
            "a collapsed sidebar painted label text: {narrow:?}"
        );
        assert!(
            narrow.iter().any(|r| r.contains('B')),
            "a collapsed sidebar painted no icon: {narrow:?}"
        );
        assert!(
            narrow
                .iter()
                .all(|r| !r.contains("Workspace") && !r.contains("Project")),
            "collapsed groups leaked heading text: {narrow:?}"
        );
        assert_eq!(&narrow[..6], ["  T", "  R", "  B", "", "  M", "  $"]);
    }

    #[test]
    fn empty_sections_are_real_group_sentinels() {
        const GROUPS: [E; 3] = [
            E("one", "A", "1", false),
            E("none", "", "0", false),
            E("two", "A", "2", false),
        ];
        let st = NavListState::new();
        let full = rows(&list(), &st, 24, &GROUPS);
        assert_eq!(
            &full[..7],
            [" A", "  1 one", "", "  0 none", "", " A", "  2 two"]
        );

        let collapsed = rows(&list().mode(NavMode::Collapsed), &st, 6, &GROUPS);
        assert_eq!(&collapsed[..5], ["  1", "", "  0", "", "  2"]);
    }

    #[test]
    fn renderer_runs_only_for_visible_full_mode_item_rows() {
        let calls = Cell::new(0usize);
        let row = |_item: &E, row: &mut RowUi<'_>| {
            calls.set(calls.get().saturating_add(1));
            row.label("row");
        };
        let nav = NavList::new(NAV).key(keyed as fn(&E) -> ItemKey).row(row);
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 20, 2);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            nav.draw(&mut ui, area, &NavListState::new(), &ITEMS);
        }
        assert_eq!(calls.get(), 2);

        calls.set(0);
        let collapsed = nav.mode(NavMode::Collapsed);
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            collapsed.draw(&mut ui, area, &NavListState::new(), &ITEMS);
        }
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn hover_state_belongs_only_to_the_exact_keyed_row() {
        let flags = RefCell::new(Vec::new());
        let row = |_item: &E, row: &mut RowUi<'_>| {
            flags.borrow_mut().push((row.key(), row.flags()));
        };
        let nav = NavList::new(NAV).key(keyed as fn(&E) -> ItemKey).row(row);
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 20, 5);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);

            ui.reference(
                Some(
                    crate::ReferenceTarget::new(NAV, crate::ReferenceState::HOVERED)
                        .part(PartRef::item(Part::ROW, ItemKey::text("Branches"))),
                ),
                |ui| nav.draw(ui, area, &NavListState::new(), &ITEMS),
            );
        }
        let flags = flags.borrow();
        assert_eq!(flags.len(), ITEMS.len());
        for (key, state) in flags.iter().copied() {
            assert_eq!(
                state.contains(StateFlags::HOVERED),
                key == ItemKey::text("Branches")
            );
        }
    }

    #[test]
    fn reference_rendering_registers_no_control_or_row_regions() {
        use termrock_runtime::Runtime;
        use termrock_runtime::stub::Stub;
        let area = Rect::new(0, 0, 24, 12);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut page = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut page, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        NAV,
                        crate::ReferenceState::FOCUSED,
                    )),
                    |ui| list().draw(ui, area, &NavListState::new(), &ITEMS),
                );
            })
            .commit_presented();

        assert_eq!(runtime.ring().reachable().count(), 0);
    }

    #[test]
    fn selected_style_comes_only_from_the_actual_current_key() {
        let flags = RefCell::new(Vec::new());
        let row = |_item: &E, row: &mut RowUi<'_>| {
            flags.borrow_mut().push((row.key(), row.flags()));
        };
        let nav = NavList::new(NAV).key(keyed as fn(&E) -> ItemKey).row(row);
        let area = Rect::new(0, 0, 20, 8);
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();

        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            nav.draw(&mut ui, area, &NavListState::new(), &ITEMS);
        }
        assert!(
            flags
                .borrow()
                .iter()
                .all(|(_, state)| !state.contains(StateFlags::SELECTED))
        );

        flags.borrow_mut().clear();
        let mut state = NavListState::new();
        state.set_current(Some(ItemKey::text("Branches")));
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            nav.draw(&mut ui, area, &state, &ITEMS);
        }
        let selected = flags
            .borrow()
            .iter()
            .filter(|(_, state)| state.contains(StateFlags::SELECTED))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        assert_eq!(selected, [ItemKey::text("Branches")]);
    }

    #[test]
    fn owner_patches_and_badge_slots_do_not_leak_into_row_owned_parts() {
        let bold = StylePatch::new().add(Modifier::BOLD);
        let custom = Part::custom("nav.tests.custom");
        let owner_parts = [
            (Part::LABEL, bold),
            (Part::BADGE, bold),
            (Part::META, bold),
            (Part::CELL, bold),
            (custom, bold),
        ];
        let row = |item: &E, row: &mut RowUi<'_>| {
            row.label(item.0);
            row.meta("m");
            row.part(Part::CELL, 1).text("c");
            row.part(custom, 1).text("x");
        };
        let nav = NavList::new(NAV)
            .key(keyed as fn(&E) -> ItemKey)
            .row(row)
            .badge(&badge)
            .patch_part(&owner_parts);
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 24, 1);
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            nav.draw(&mut ui, area, &NavListState::new(), &ITEMS[..1]);
        }
        assert!(
            page.cell((4, 0))
                .is_some_and(|c| c.modifier.contains(Modifier::BOLD))
        );
        assert!(
            page.cell((23, 0))
                .is_some_and(|c| c.modifier.contains(Modifier::BOLD))
        );
        assert!(
            page.cell((21, 0))
                .is_some_and(|c| !c.modifier.contains(Modifier::BOLD))
        );
        assert!(
            modifier_at_symbol(&page, area, "c").is_some_and(|m| !m.contains(Modifier::BOLD)),
            "owner CELL patch leaked into caller row"
        );
        assert!(
            modifier_at_symbol(&page, area, "x").is_some_and(|m| !m.contains(Modifier::BOLD)),
            "owner custom-part patch leaked into caller row"
        );

        let badge_calls = Cell::new(0usize);
        let label_calls = Cell::new(0usize);
        let slot = |ui: &mut Ui<'_>, rect: Rect| {
            badge_calls.set(badge_calls.get().saturating_add(1));
            let style = ui.surface_style();
            ui.paint_str(rect, "#", style);
        };
        let forbidden = |_ui: &mut Ui<'_>, _rect: Rect| {
            label_calls.set(label_calls.get().saturating_add(1));
        };
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            NavList::new(NAV)
                .key(keyed as fn(&E) -> ItemKey)
                .row(row)
                .badge(&badge)
                .slot(Part::BADGE, &slot)
                .draw(&mut ui, area, &NavListState::new(), &ITEMS[..1]);
            NavList::new(NAV)
                .key(keyed as fn(&E) -> ItemKey)
                .row(row)
                .badge(&badge)
                .slot(Part::LABEL, &forbidden)
                .draw(&mut ui, area, &NavListState::new(), &ITEMS[..1]);
        }
        assert_eq!(badge_calls.get(), 1);
        assert_eq!(label_calls.get(), 0);
        assert_eq!(
            page.cell((23, 0)).map(ratatui_core::buffer::Cell::symbol),
            Some("9")
        );
    }

    #[test]
    fn slot_scope_is_exact_and_every_allowed_slot_substitutes() {
        let area = Rect::new(0, 0, 24, 3);
        let theme = Theme::junie();
        for part in [
            Part::GUTTER,
            Part::MARKER,
            Part::ICON,
            Part::HEADER,
            Part::BADGE,
        ] {
            let calls = Cell::new(0usize);
            let slot = |ui: &mut Ui<'_>, rect: Rect| {
                calls.set(calls.get().saturating_add(1));
                let style = ui.surface_style();
                ui.paint_str(rect, "#", style);
            };
            let mut frame = FrameState::default();
            frame.reset(1, area);
            let mut page = Buffer::empty(area);
            let mut core = UiCore::default();
            let last = LastFrame::default();
            {
                let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
                list().badge(&badge).slot(part, &slot).draw(
                    &mut ui,
                    area,
                    &NavListState::new(),
                    &ITEMS[..1],
                );
            }
            assert_eq!(calls.get(), 1, "allowed slot {part:?} execution count");
            assert!(
                contains_symbol(&page, area, "#"),
                "allowed slot {part:?} did not substitute cells"
            );
        }

        let custom = Part::custom("nav.tests.slot.custom");
        for part in [Part::CONTAINER, Part::LABEL, Part::META, Part::CELL, custom] {
            let calls = Cell::new(0usize);
            let slot = |_ui: &mut Ui<'_>, _rect: Rect| {
                calls.set(calls.get().saturating_add(1));
            };
            let row = |item: &E, row: &mut RowUi<'_>| {
                row.label(item.0);
                row.meta("m");
                row.part(Part::CELL, 1).text("c");
                row.part(custom, 1).text("x");
            };
            let mut frame = FrameState::default();
            frame.reset(1, area);
            let mut page = Buffer::empty(area);
            let mut core = UiCore::default();
            let last = LastFrame::default();
            {
                let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
                NavList::new(NAV)
                    .key(keyed as fn(&E) -> ItemKey)
                    .row(row)
                    .badge(&badge)
                    .slot(part, &slot)
                    .draw(&mut ui, area, &NavListState::new(), &ITEMS[..1]);
            }
            assert_eq!(calls.get(), 0, "forbidden slot {part:?} executed");
        }
    }
}
