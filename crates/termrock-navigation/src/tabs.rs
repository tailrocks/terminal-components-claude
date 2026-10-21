use core::fmt;
use core::marker::PhantomData;

use ratatui_core::buffer::CellWidth;
use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_runtime::empty::Status;
use termrock_runtime::rowui::{RowFn, RowUi};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsAction {
    Activated(ItemKey),
    Close(ItemKey),
    New,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsCmd {
    Prev,
    Next,
    Activate,
    Nth(u8),
    Close,
    New,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: TabsCmd,
    label: &'static str,
    visible: bool,
) -> Binding<TabsCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BASE: [Binding<TabsCmd>; 15] = [
    b(
        "tabs.previous",
        Chord::key(KeyCode::Left),
        TabsCmd::Prev,
        "Prev tab",
        true,
    ),
    b(
        "tabs.next",
        Chord::key(KeyCode::Right),
        TabsCmd::Next,
        "Next tab",
        true,
    ),
    b(
        "tabs.previous-vim",
        Chord::key(KeyCode::Char('h')),
        TabsCmd::Prev,
        "Prev tab",
        false,
    ),
    b(
        "tabs.next-vim",
        Chord::key(KeyCode::Char('l')),
        TabsCmd::Next,
        "Next tab",
        false,
    ),
    b(
        "tabs.activate",
        Chord::key(KeyCode::Enter),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        "tabs.activate-space",
        Chord::key(KeyCode::Char(' ')),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        "tabs.tab-1",
        Chord::key(KeyCode::Char('1')),
        TabsCmd::Nth(1),
        "Tab 1",
        false,
    ),
    b(
        "tabs.tab-2",
        Chord::key(KeyCode::Char('2')),
        TabsCmd::Nth(2),
        "Tab 2",
        false,
    ),
    b(
        "tabs.tab-3",
        Chord::key(KeyCode::Char('3')),
        TabsCmd::Nth(3),
        "Tab 3",
        false,
    ),
    b(
        "tabs.tab-4",
        Chord::key(KeyCode::Char('4')),
        TabsCmd::Nth(4),
        "Tab 4",
        false,
    ),
    b(
        "tabs.tab-5",
        Chord::key(KeyCode::Char('5')),
        TabsCmd::Nth(5),
        "Tab 5",
        false,
    ),
    b(
        "tabs.tab-6",
        Chord::key(KeyCode::Char('6')),
        TabsCmd::Nth(6),
        "Tab 6",
        false,
    ),
    b(
        "tabs.tab-7",
        Chord::key(KeyCode::Char('7')),
        TabsCmd::Nth(7),
        "Tab 7",
        false,
    ),
    b(
        "tabs.tab-8",
        Chord::key(KeyCode::Char('8')),
        TabsCmd::Nth(8),
        "Tab 8",
        false,
    ),
    b(
        "tabs.tab-9",
        Chord::key(KeyCode::Char('9')),
        TabsCmd::Nth(9),
        "Tab 9",
        false,
    ),
];
const CLOSE_X: Binding<TabsCmd> = b(
    "tabs.close",
    Chord::key(KeyCode::Char('x')),
    TabsCmd::Close,
    "Close",
    true,
);
const CLOSE_DEL: Binding<TabsCmd> = b(
    "tabs.close-delete",
    Chord::key(KeyCode::Delete),
    TabsCmd::Close,
    "Close",
    false,
);
const NEW_N: Binding<TabsCmd> = b(
    "tabs.new",
    Chord::key(KeyCode::Char('n')),
    TabsCmd::New,
    "New",
    true,
);

macro_rules! table {
    ($name:ident, $n:expr, [$($extra:expr),*]) => {
        const $name: [Binding<TabsCmd>; $n] = [
            BASE[0], BASE[1], BASE[2], BASE[3], BASE[4], BASE[5], BASE[6], BASE[7], BASE[8],
            BASE[9], BASE[10], BASE[11], BASE[12], BASE[13], BASE[14], $($extra),*
        ];
    };
}
table!(PLAIN, 15, []);
table!(CLOSABLE, 17, [CLOSE_X, CLOSE_DEL]);
table!(NEWABLE, 16, [NEW_N]);
table!(FULL, 18, [CLOSE_X, CLOSE_DEL, NEW_N]);

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TabsState {
    core: CollectionCore,
    active: Option<ItemKey>,
    first: Option<ItemKey>,
    first_index: usize,
}

impl TabsState {
    pub const fn active(&self) -> Option<ItemKey> {
        self.active
    }

    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn first(&self) -> Option<ItemKey> {
        self.first
    }

    pub fn set_active(&mut self, index: usize, key: ItemKey) {
        self.active = Some(key);
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for TabsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some(a) = self.active
            && !(0..len).any(|i| key(i) == a)
        {
            self.active = self.core.cursor();
        }
        match self.first {
            Some(f) => {
                if let Some(i) = (0..len).find(|&i| key(i) == f) {
                    self.first_index = i;
                } else {
                    let i = self.first_index.min(len.saturating_sub(1));
                    self.first_index = i;
                    self.first = (len > 0).then(|| key(i));
                }
            }
            None => self.first_index = 0,
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// Committed viewport counts supplied to a pure tabs layout pass.
///
/// This receipt is explicit because planning must not inspect a previously
/// painted frame. Pass the receipt returned by the preceding committed layout
/// when preserving the warm-frame overflow reservation rule.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TabWindowReceipt {
    pub viewport_len: usize,
    pub content_len: usize,
}

/// One immutable semantic row allocation for a [`TabLayout`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TabRowLayout {
    pub key: ItemKey,
    pub index: usize,
    pub logical_content: Rect,
    pub visible_content: Rect,
    pub logical_tab: Rect,
    pub visible_tab: Rect,
    pub logical_close: Option<Rect>,
    pub visible_close: Option<Rect>,
    label_segments: Vec<String>,
    pub label_extent: u16,
}

/// Pure layout result for default display rows.
///
/// Layout owns the formatted row segments and all tab allocations. Drawing a
/// layout never invokes the item's `Display` implementation or scans rendered
/// buffer cells to recover label extents.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TabLayout {
    id: Id,
    area: Rect,
    clip: Rect,
    row_area: Rect,
    used_area: Rect,
    rule_area: Option<Rect>,
    rows: Vec<TabRowLayout>,
    first_index: usize,
    active: Option<ItemKey>,
    cursor: Option<ItemKey>,
    first: Option<ItemKey>,
    first_tab_x: u16,
    allow_new: bool,
    closable: bool,
    status: Status,
    left_overflow: Option<Rect>,
    right_overflow: Option<Rect>,
    new_area: Option<Rect>,
    status_area: Option<Rect>,
    right_limit: u16,
    hidden_right: usize,
    pub logical_area: Rect,
    pub visible_area: Rect,
    pub viewport_len: usize,
    pub content_len: usize,
    pub overflow_reserved: bool,
}

impl TabLayout {
    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn clip(&self) -> Rect {
        self.clip
    }

    pub fn rows(&self) -> &[TabRowLayout] {
        &self.rows
    }

    pub const fn active_key(&self) -> Option<ItemKey> {
        self.active
    }

    pub const fn cursor_key(&self) -> Option<ItemKey> {
        self.cursor
    }

    pub const fn first_key(&self) -> Option<ItemKey> {
        self.first
    }

    pub const fn first_index(&self) -> usize {
        self.first_index
    }

    pub const fn hidden_right(&self) -> usize {
        self.hidden_right
    }

    pub const fn used_area(&self) -> Rect {
        self.used_area
    }

    /// Return logical and clipped bounds for an interactive tabs part.
    pub fn part_bounds(&self, part: PartRef) -> Option<(Rect, Rect)> {
        match part.part {
            Part::TAB => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .map(|row| (row.logical_tab, row.visible_tab)),
            Part::CLOSE => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .and_then(|row| Some((row.logical_close?, row.visible_close?))),
            Part::OVERFLOW => match part.item {
                Some(ItemKey::Index(0)) => self
                    .left_overflow
                    .map(|area| (area, area.intersection(self.clip))),
                Some(ItemKey::Index(1)) => self
                    .right_overflow
                    .map(|area| (area, area.intersection(self.clip))),
                _ => None,
            },
            Part::NEW if part.item.is_none() => self
                .new_area
                .map(|area| (area, area.intersection(self.clip))),
            Part::ICON if part.item.is_none() => self
                .status_area
                .map(|area| (area, area.intersection(self.clip))),
            _ => None,
        }
    }

    pub const fn window_receipt(&self) -> Option<TabWindowReceipt> {
        if self.area.is_empty() {
            return None;
        }
        Some(TabWindowReceipt {
            viewport_len: self.viewport_len,
            content_len: self.content_len,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TabLayoutMismatch;

struct SegmentWriter(Vec<String>);

impl fmt::Write for SegmentWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.push(value.to_owned());
        Ok(())
    }
}

struct DisplaySegments<'a>(&'a [String]);

impl fmt::Display for DisplaySegments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in self.0 {
            f.write_str(segment)?;
        }
        Ok(())
    }
}

pub struct Tabs<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    allow_new: bool,
    closable: bool,
    status: Status,
    ov: PartStyle<'a>,
    _t: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for Tabs<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tabs")
            .field("id", &self.id)
            .field("allow_new", &self.allow_new)
            .field("closable", &self.closable)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl<T> Tabs<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        Tabs {
            id,
            key: ByIndex,
            row: DefaultRow,
            allow_new: false,
            closable: false,
            status: Status::Ready,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> Tabs<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TAB,
        Part::LABEL,
        Part::META,
        Part::CLOSE,
        Part::NEW,
        Part::RULE,
        Part::OVERFLOW,
        Part::BADGE,
        Part::ICON,
    ];

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tabs<'a, T, K2, R> {
        Tabs {
            id: self.id,
            key: k,
            row: self.row,
            allow_new: self.allow_new,
            closable: self.closable,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tabs<'a, T, K, R2> {
        Tabs {
            id: self.id,
            key: self.key,
            row: r,
            allow_new: self.allow_new,
            closable: self.closable,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub fn allow_new(mut self, yes: bool) -> Self {
        self.allow_new = yes;
        self
    }

    #[must_use]
    pub fn closable(mut self, yes: bool) -> Self {
        self.closable = yes;
        self
    }

    #[must_use]
    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    fn table(&self) -> &'static [Binding<TabsCmd>] {
        table_for(self.closable, self.allow_new)
    }
}

pub(crate) const fn table_for(closable: bool, allow_new: bool) -> &'static [Binding<TabsCmd>] {
    match (closable, allow_new) {
        (false, false) => &PLAIN,
        (true, false) => &CLOSABLE,
        (false, true) => &NEWABLE,
        (true, true) => &FULL,
    }
}

fn digits(n: usize, buf: &mut [u8; 2]) -> &str {
    let n = n.min(99);
    if n >= 10 {
        buf[0] = b'0'.saturating_add((n / 10) as u8);
        buf[1] = b'0'.saturating_add((n % 10) as u8);
        core::str::from_utf8(&buf[..]).unwrap_or("")
    } else {
        buf[0] = b'0'.saturating_add(n as u8);
        core::str::from_utf8(&buf[..1]).unwrap_or("")
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Tabs<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
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

    fn activate(&self, st: &mut TabsState, items: &[T], i: usize, acc: &mut Acc<TabsAction>) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = i.min(items.len().saturating_sub(1));
        let key = self.key_at(items, i);
        st.set_active(i, key);
        acc.action(TabsAction::Activated(key));
    }

    fn follow(&self, st: &mut TabsState, items: &[T], fit: usize) {
        let Some(active) = st.active else { return };
        let Some(ai) = self.index_of(items, active, Some(st.core.cursor_index())) else {
            return;
        };
        if ai < st.first_index {
            st.first_index = ai;
            st.first = Some(active);
        } else if fit > 0 && ai >= st.first_index.saturating_add(fit) {
            let i = ai.saturating_add(1).saturating_sub(fit);
            st.first_index = i;
            st.first = Some(self.key_at(items, i));
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState, items: &[T]) -> Response<TabsAction> {
        let len = items.len();
        let _ = st.reconcile(len, |i| self.key_at(items, i));
        if st.core.cursor().is_none() && len > 0 {
            let key = self.key_at(items, 0);
            st.core.set_cursor(0, key);
        }
        if st.active.is_none() {
            st.active = st.core.cursor();
        }
        if st.first.is_none() && len > 0 {
            st.first = Some(self.key_at(items, 0));
            st.first_index = 0;
        }
        let mut acc = Acc::<TabsAction>::new();
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(TabsCmd::Prev) => {
                            self.activate(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(TabsCmd::Next) => {
                            self.activate(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(TabsCmd::Activate) => self.activate(st, items, cur, &mut acc),
                        Some(TabsCmd::Nth(n)) => {
                            let i = usize::from(n).saturating_sub(1);
                            if i < len {
                                self.activate(st, items, i, &mut acc);
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::Close) => {
                            if len > 0 {
                                acc.action(TabsAction::Close(self.key_at(items, cur)));
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::New) => acc.action(TabsAction::New),
                        None => {}
                    }
                }
                Intent::Pointer { phase, part, .. } => match (phase, part.part, part.item) {
                    (Phase::Press, Part::TAB, Some(k)) => {
                        if let Some(i) = self.index_of(items, k, Some(st.core.cursor_index())) {
                            st.core.set_cursor(i, k);
                        }
                        acc.changed();
                    }
                    (Phase::Click | Phase::DoubleClick, Part::TAB, Some(k)) => {
                        match self.index_of(items, k, Some(st.core.cursor_index())) {
                            Some(i) => self.activate(st, items, i, &mut acc),
                            None => acc.consumed(),
                        }
                    }
                    (Phase::Click, Part::CLOSE, Some(k)) => acc.action(TabsAction::Close(k)),
                    (Phase::Click, Part::NEW, _) => acc.action(TabsAction::New),
                    (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(0))) => {
                        let i = st.first_index.saturating_sub(1);
                        st.first_index = i;
                        st.first = (len > 0).then(|| self.key_at(items, i));
                        acc.changed();
                    }
                    (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(_))) => {
                        let i = st.first_index.saturating_add(1).min(len.saturating_sub(1));
                        st.first_index = i;
                        st.first = (len > 0).then(|| self.key_at(items, i));
                        acc.changed();
                    }
                    _ => acc.consumed(),
                },
                _ => {}
            }
        }
        if let Some(l) = cx.layout(self.id) {
            self.follow(st, items, l.viewport_len);
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TabsState, items: &[T]) -> Rect {
        self.draw_impl(ui, area, Some(st), Some(items), None)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one shared paint pass consumes either legacy rows or an owned semantic layout"
    )]
    fn draw_impl(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: Option<&TabsState>,
        items: Option<&[T]>,
        planned: Option<&TabLayout>,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let len = planned.map_or_else(|| items.map_or(0, <[T]>::len), |layout| layout.content_len);
        let row0 = planned.map_or_else(|| first_row(area), |layout| layout.row_area);
        let used = planned.map_or_else(
            || Rect {
                height: area.height.min(2),
                ..area
            },
            |layout| layout.used_area,
        );
        if !ui.is_inert() {
            ui.register_control(self.id, used, Focusability::Focusable);
        }
        let live = PartStyle::flags(ui.state(self.id), self.status.flags());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let ov = self.ov;
        let id = self.id;
        let strip = ov.style(
            ui,
            id,
            Family::TABS,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(used, strip.style);
        let rule_row = planned.map_or_else(
            || {
                if area.height >= 2 {
                    Some(Rect {
                        y: area.y.saturating_add(1),
                        height: 1,
                        ..row0
                    })
                } else {
                    None
                }
            },
            |layout| layout.rule_area,
        );
        if let Some(rr) = rule_row {
            let quiet = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::RULE,
                StateFlags::empty(),
            );
            for col in rr.columns() {
                match quiet.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(col, glyph, quiet.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(col, GlyphRole::RuleQuiet, quiet.style);
                    }
                    Slot::Clear => {
                        ui.fill(col, quiet.style);
                    }
                }
            }
        }
        let overflow_last = planned.map_or_else(
            || {
                ui.layout(self.id)
                    .is_some_and(|l| l.viewport_len < l.content_len)
            },
            |layout| layout.overflow_reserved,
        );
        let first_index = planned.map_or_else(
            || {
                let st = state.expect("legacy tabs draw requires component state");
                let items = items.expect("legacy tabs draw requires item data");
                st.first
                    .and_then(|f| self.index_of(items, f, Some(st.first_index)))
                    .unwrap_or(0)
                    .min(len)
            },
            |layout| layout.first_index,
        );
        let status_w: u16 = if matches!(self.status, Status::Ready) {
            0
        } else {
            2
        };
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let right_w: u16 = if overflow_last || first_index > 0 {
            4
        } else {
            0
        };
        let right_limit = planned.map_or_else(
            || {
                row0.right()
                    .saturating_sub(status_w)
                    .saturating_sub(right_w)
                    .saturating_sub(new_w)
            },
            |layout| layout.right_limit,
        );
        let first_tab_x = planned.map_or_else(
            || row0.x.saturating_add(left_w),
            |layout| layout.first_tab_x,
        );
        let mut x = first_tab_x;
        let mut fit = 0usize;
        let cursor = planned.map_or_else(
            || {
                state
                    .expect("legacy tabs draw requires component state")
                    .core
                    .cursor()
            },
            |layout| layout.cursor,
        );
        let active = planned.map_or_else(
            || {
                state
                    .expect("legacy tabs draw requires component state")
                    .active
            },
            |layout| layout.active,
        );
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let row_count = planned.map_or(len.saturating_sub(first_index), |layout| layout.rows.len());
        for offset in 0..row_count {
            let i = first_index.saturating_add(offset);
            let row_plan = planned.and_then(|layout| layout.rows.get(offset));
            let item = items.and_then(|items| items.get(i));
            let key = row_plan.map_or_else(
                || {
                    let item = item.expect("legacy tabs draw requires item data");
                    self.key.key(item, i)
                },
                |row| row.key,
            );
            let avail = right_limit.saturating_sub(x);
            if row_plan.is_none() && avail < 3 {
                break;
            }

            let is_cursor = cursor == Some(key);
            let is_active = active == Some(key);
            let mut flags = StateFlags::empty();
            if is_active {
                flags |= StateFlags::ACTIVE;
            }
            if is_cursor {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
                if pressed.is_none() {
                    flags |= live & StateFlags::PRESSED;
                }
            }
            let tab_part = PartRef::item(Part::TAB, key);
            if hovered == Some(tab_part) {
                flags |= StateFlags::HOVERED;
            }
            if pressed == Some(tab_part) {
                flags |= StateFlags::PRESSED;
            }

            let content = row_plan.map_or_else(
                || Rect {
                    x: x.saturating_add(1),
                    y: row0.y,
                    width: avail.saturating_sub(1),
                    height: 1,
                },
                |row| row.logical_content,
            );
            {
                let mut r = RowUi::new(ui, id, Family::TABS, Variant::DEFAULT, flags, key, content);
                if let Some(row) = row_plan {
                    r.label_fmt(format_args!("{}", DisplaySegments(&row.label_segments)));
                } else {
                    self.row
                        .row(item.expect("legacy tabs draw requires item data"), &mut r);
                }
            }
            let label_w =
                row_plan.map_or_else(|| painted_width(ui, content).max(1), |row| row.label_extent);
            let close_w: u16 = if self.closable { 2 } else { 0 };
            let tab = row_plan.map_or_else(
                || {
                    let tab_w = 1u16
                        .saturating_add(label_w)
                        .saturating_add(1)
                        .saturating_add(close_w);
                    Rect {
                        x,
                        y: row0.y,
                        width: tab_w.min(avail),
                        height: 1,
                    }
                },
                |row| row.logical_tab,
            );
            let tail = Rect {
                x: tab.right(),
                y: row0.y,
                width: right_limit.saturating_sub(tab.right()),
                height: 1,
            };
            ui.fill(tail, strip.style);
            let ts = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::TAB, flags);
            ui.paint_style(tab, ts.style);

            if flags.contains(StateFlags::PRESSED) {
                let ls = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::LABEL, flags);
                if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                    paint_pressed_bracket(
                        ui,
                        cell_at(tab, tab.x),
                        cell_at(tab, tab.x.saturating_add(1).saturating_add(label_w)),
                        ls.style,
                    );
                }
            }
            if self.closable {
                let close_cell = row_plan
                    .and_then(|row| row.logical_close)
                    .unwrap_or_else(|| cell_at(tab, tab.right().saturating_sub(2)));
                let close_part = PartRef::item(Part::CLOSE, key);
                let mut close_flags = flags.difference(StateFlags::HOVERED | StateFlags::PRESSED);
                if hovered == Some(close_part) {
                    close_flags |= StateFlags::HOVERED;
                }
                if pressed == Some(close_part) {
                    close_flags |= StateFlags::PRESSED;
                }
                let cs = ov.style(
                    ui,
                    id,
                    Family::TABS,
                    Variant::DEFAULT,
                    Part::CLOSE,
                    close_flags,
                );
                match cs.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(close_cell, glyph, cs.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(close_cell, GlyphRole::Close, cs.style);
                    }
                    Slot::Clear => {
                        ui.fill(close_cell, cs.style);
                    }
                }
            }
            if is_active && let Some(rr) = rule_row {
                let rs = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::RULE, flags);
                let span = Rect {
                    x: tab.x,
                    width: tab.width,
                    ..rr
                };
                for col in span.columns() {
                    match rs.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(col, glyph, rs.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(col, GlyphRole::RuleActive, rs.style);
                        }
                        Slot::Clear => {
                            ui.fill(col, rs.style);
                        }
                    }
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::item(Part::TAB, key), tab);
            }
            if self.closable && !ui.is_inert() {
                let close_cell = row_plan
                    .and_then(|row| row.logical_close)
                    .unwrap_or_else(|| cell_at(tab, tab.right().saturating_sub(2)));
                ui.register_part(self.id, PartRef::item(Part::CLOSE, key), close_cell);
            }
            x = tab.right().saturating_add(1);
            fit = fit.saturating_add(1);
        }
        let mut buf = [0u8; 2];
        if first_index > 0 {
            let cell = planned
                .and_then(|layout| layout.left_overflow)
                .unwrap_or(Rect {
                    x: row0.x,
                    y: row0.y,
                    width: left_w.min(row0.width),
                    height: 1,
                });
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            let used = ui.glyph(cell, GlyphRole::OverflowLeft, os.style);
            ui.paint_str(
                super::shift(cell, used),
                digits(first_index, &mut buf),
                os.style,
            );
            if !ui.is_inert() {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::OVERFLOW, ItemKey::index(0)),
                    cell,
                );
            }
        }
        let hidden_right = planned.map_or_else(
            || len.saturating_sub(first_index.saturating_add(fit)),
            |layout| layout.hidden_right,
        );
        if hidden_right > 0 {
            let cell = planned
                .and_then(|layout| layout.right_overflow)
                .unwrap_or(Rect {
                    x: row0
                        .right()
                        .saturating_sub(status_w)
                        .saturating_sub(new_w)
                        .saturating_sub(4)
                        .max(row0.x),
                    y: row0.y,
                    width: 4.min(row0.width),
                    height: 1,
                });
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            ui.fill(cell, strip.style);
            let d = digits(hidden_right, &mut buf);
            let dw = crate::text::width(d);
            let text = Rect {
                x: cell.right().saturating_sub(1).saturating_sub(dw),
                width: dw,
                ..cell
            }
            .intersection(cell);
            ui.paint_str(text, d, os.style);
            ui.glyph(
                cell_at(cell, cell.right().saturating_sub(1)),
                GlyphRole::OverflowRight,
                os.style,
            );
            if !ui.is_inert() {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::OVERFLOW, ItemKey::index(1)),
                    cell,
                );
            }
        }
        if self.allow_new {
            let cell = planned.and_then(|layout| layout.new_area).unwrap_or(Rect {
                x: row0
                    .right()
                    .saturating_sub(status_w)
                    .saturating_sub(new_w)
                    .max(row0.x),
                y: row0.y,
                width: new_w.min(row0.width),
                height: 1,
            });
            let ns = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::NEW,
                StateFlags::empty(),
            );
            let inner = super::shift(cell, 1);
            match ns.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(inner, glyph, ns.style);
                }
                Slot::Inherit => {
                    ui.glyph(inner, GlyphRole::NewTab, ns.style);
                }
                Slot::Clear => {
                    ui.fill(inner, ns.style);
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::of(Part::NEW), cell);
            }
        }
        if status_w > 0 {
            let icon_cell = planned
                .and_then(|layout| layout.status_area)
                .unwrap_or(Rect {
                    x: row0.right().saturating_sub(status_w).max(row0.x),
                    y: row0.y,
                    width: 1.min(row0.width),
                    height: 1,
                });
            if let Some(slot) = ov.slot_for(Part::ICON) {
                slot(ui, icon_cell);
            } else {
                let icon = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::ICON, live);
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
        let viewport_len = planned.map_or(fit, |layout| layout.viewport_len);
        ui.report_layout(
            self.id,
            LayoutFacts::new(viewport_len, len, used.height, used.width),
        );
        used
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let h = ui.design().size.tabs_height;
        Size {
            min: (8, h),
            preferred: (c.max.0, h),
        }
        .fit(c)
    }
}

impl<T: fmt::Display, K: KeyFn<T>> Tabs<'_, T, K, DefaultRow> {
    /// Resolve default display rows without painting or reading a buffer.
    ///
    /// `previous` is the caller-owned receipt from the last committed draw.
    /// It preserves the existing overflow reservation behavior across cold
    /// and warm frames while keeping measurement independent of `Ui`.
    pub fn layout_default(
        &self,
        area: Rect,
        clip: Rect,
        state: &TabsState,
        items: &[T],
        previous: Option<TabWindowReceipt>,
    ) -> TabLayout {
        let len = items.len();
        let first_index = state
            .first
            .and_then(|first| self.index_of(items, first, Some(state.first_index)))
            .unwrap_or(0)
            .min(len);
        let overflow_reserved =
            previous.is_some_and(|receipt| receipt.viewport_len < receipt.content_len);
        if area.is_empty() {
            return TabLayout {
                id: self.id,
                area,
                clip,
                row_area: first_row(area),
                used_area: area,
                rule_area: None,
                rows: Vec::new(),
                first_index,
                active: state.active,
                cursor: state.core.cursor(),
                first: state.first,
                first_tab_x: area.x,
                allow_new: self.allow_new,
                closable: self.closable,
                status: self.status,
                left_overflow: None,
                right_overflow: None,
                new_area: None,
                status_area: None,
                right_limit: area.x,
                hidden_right: len.saturating_sub(first_index),
                logical_area: area,
                visible_area: area.intersection(clip),
                viewport_len: 0,
                content_len: len,
                overflow_reserved,
            };
        }
        let status_w: u16 = if matches!(self.status, Status::Ready) {
            0
        } else {
            2
        };
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let right_w: u16 = if overflow_reserved || first_index > 0 {
            4
        } else {
            0
        };
        let row0 = first_row(area);
        let rule_area = (area.height >= 2).then(|| Rect {
            y: area.y.saturating_add(1),
            height: 1,
            ..row0
        });
        let first_tab_x = row0.x.saturating_add(left_w);
        let right_limit = row0
            .right()
            .saturating_sub(status_w)
            .saturating_sub(right_w)
            .saturating_sub(new_w);
        let mut x = first_tab_x;
        let mut rows = Vec::new();
        for i in first_index..len {
            let Some(item) = items.get(i) else { break };
            let key = self.key_at(items, i);
            let avail = right_limit.saturating_sub(x);
            if avail < 3 {
                break;
            }
            let content = Rect {
                x: x.saturating_add(1),
                y: row0.y,
                width: avail.saturating_sub(1),
                height: 1,
            };
            let mut writer = SegmentWriter(Vec::new());
            let _ = fmt::write(&mut writer, format_args!("{item}"));
            let label_extent = semantic_label_extent(&writer.0, content, clip).max(1);
            let close_w: u16 = if self.closable { 2 } else { 0 };
            let tab_w = 1u16
                .saturating_add(label_extent)
                .saturating_add(1)
                .saturating_add(close_w);
            let tab = Rect {
                x,
                y: row0.y,
                width: tab_w.min(avail),
                height: 1,
            };
            let close = self
                .closable
                .then(|| cell_at(tab, tab.right().saturating_sub(2)));
            rows.push(TabRowLayout {
                key,
                index: i,
                logical_content: content,
                visible_content: content.intersection(clip),
                logical_tab: tab,
                visible_tab: tab.intersection(clip),
                logical_close: close,
                visible_close: close.map(|rect| rect.intersection(clip)),
                label_segments: writer.0,
                label_extent,
            });
            x = tab.right().saturating_add(1);
        }
        let viewport_len = rows.len();
        let used = Rect {
            height: area.height.min(2),
            ..area
        };
        let left_overflow = (first_index > 0).then(|| Rect {
            x: row0.x,
            y: row0.y,
            width: left_w.min(row0.width),
            height: 1,
        });
        let hidden_right = len.saturating_sub(first_index.saturating_add(viewport_len));
        let right_overflow = (hidden_right > 0).then(|| Rect {
            x: row0
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .saturating_sub(4)
                .max(row0.x),
            y: row0.y,
            width: 4.min(row0.width),
            height: 1,
        });
        let new_area = self.allow_new.then(|| Rect {
            x: row0
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .max(row0.x),
            y: row0.y,
            width: new_w.min(row0.width),
            height: 1,
        });
        let status_area = (status_w > 0).then(|| Rect {
            x: row0.right().saturating_sub(status_w).max(row0.x),
            y: row0.y,
            width: 1.min(row0.width),
            height: 1,
        });
        TabLayout {
            id: self.id,
            area,
            clip,
            row_area: row0,
            used_area: used,
            rule_area,
            rows,
            first_index,
            active: state.active,
            cursor: state.core.cursor(),
            first: state.first,
            first_tab_x,
            allow_new: self.allow_new,
            closable: self.closable,
            status: self.status,
            left_overflow,
            right_overflow,
            new_area,
            status_area,
            right_limit,
            hidden_right,
            logical_area: used,
            visible_area: used.intersection(clip),
            viewport_len,
            content_len: len,
            overflow_reserved,
        }
    }

    /// Draw a previously resolved default-row layout.
    ///
    /// Rebuild the plan after any state or data change. Returns `Err` when
    /// component identity or rendering configuration differs from planning.
    pub fn draw_layout(
        &self,
        ui: &mut Ui<'_>,
        layout: &TabLayout,
    ) -> Result<Rect, TabLayoutMismatch> {
        if layout.id != self.id
            || layout.allow_new != self.allow_new
            || layout.closable != self.closable
            || layout.status != self.status
        {
            return Err(TabLayoutMismatch);
        }
        Ok(self.draw_impl(ui, layout.area, None, None, Some(layout)))
    }
}

fn semantic_label_extent(segments: &[String], content: Rect, clip: Rect) -> u16 {
    // `RowUi::label_fmt` writes each Display segment independently. Each write
    // clips from the current logical writer position, while the painter starts
    // at the clipped left edge. Before the writer reaches that edge, successive
    // segments can therefore overlap and replace cells. Model only those
    // visible semantic cells; do not render into a scratch Buffer or inspect a
    // Ui during measurement.
    let visible = content.intersection(clip);
    if visible.is_empty() {
        return 0;
    }

    let mut cells = vec![false; usize::from(visible.width)];
    let mut writer_x = content.x;
    let content_right = content.right();

    for segment in segments {
        if writer_x >= content_right {
            break;
        }
        let write_x = writer_x.max(clip.x).max(content.x);
        let write_right = content_right.min(clip.right());
        if write_x >= write_right || content.y < clip.y || content.y >= clip.bottom() {
            continue;
        }

        let mut x = write_x;
        let mut remaining = write_right.saturating_sub(write_x);
        for (_, grapheme) in termrock_text::measure::graphemes(segment) {
            if grapheme.contains(char::is_control) {
                continue;
            }
            let width = grapheme.cell_width();
            if width == 0 {
                continue;
            }
            let Some(next_remaining) = remaining.checked_sub(width) else {
                break;
            };

            let offset = usize::from(x.saturating_sub(visible.x));
            if let Some(cell) = cells.get_mut(offset) {
                *cell = grapheme != " ";
            }
            for continuation in 1..width {
                let offset = usize::from(x.saturating_add(continuation).saturating_sub(visible.x));
                if let Some(cell) = cells.get_mut(offset) {
                    *cell = false;
                }
            }

            remaining = next_remaining;
            x = x.saturating_add(width);
            if remaining == 0 {
                break;
            }
        }
        writer_x = writer_x.saturating_add(x.saturating_sub(write_x));
    }

    cells
        .iter()
        .rposition(|nonblank| *nonblank)
        .map_or(0, |index| u16::try_from(index + 1).unwrap_or(u16::MAX))
}

fn painted_width(ui: &mut Ui<'_>, row: Rect) -> u16 {
    ui.with_area(row, |ui| {
        let (buf, clip) = ui.raw();
        let mut last = 0u16;
        for x in clip.columns().map(|c| c.x) {
            if let Some(c) = buf.cell(Position::new(x, clip.y))
                && c.symbol() != " "
            {
                last = x.saturating_sub(clip.x).saturating_add(1);
            }
        }
        last
    })
}

impl<T, K, R> Bindings for Tabs<'_, T, K, R> {
    type Cmd = TabsCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TabsCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::{Buffer, Cell as BufferCell};
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{ColorLevel, Theme};

    const TABS: Id = Id::root("tabs.tests");
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 12,
        height: 2,
    };

    fn row_text(buf: &Buffer, width: u16) -> String {
        let mut text = String::new();
        for x in 0..width {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                text.push_str(cell.symbol());
            }
        }
        text
    }

    fn draw_status(status: Option<Status>) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = TabsState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let mut tabs = Tabs::new(TABS);
                if let Some(status) = status {
                    tabs = tabs.status(status);
                }
                tabs.draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        buffer
    }

    #[test]
    fn close_targets_the_logical_tab_after_a_reorder() {
        let mut st = TabsState::default();
        let keys = |v: &[u64]| {
            let v: Vec<ItemKey> = v.iter().map(|n| ItemKey::num(*n)).collect();
            v
        };
        let a = keys(&[1, 2, 3]);
        let _ = st.reconcile(3, |i| a[i]);
        st.set_active(1, ItemKey::num(2));
        st.first = Some(ItemKey::num(1));

        let b = keys(&[9, 3, 2, 1]);
        let r = st.reconcile(4, |i| b[i]);
        assert_eq!(r, Reconciliation::Unchanged);
        assert_eq!(st.active(), Some(ItemKey::num(2)));
        assert_eq!(st.cursor(), Some(ItemKey::num(2)));
        assert_eq!(st.first(), Some(ItemKey::num(1)));
        assert_eq!(st.first_index, 3);

        let c = keys(&[9, 3, 1]);
        let _ = st.reconcile(3, |i| c[i]);
        assert_eq!(st.active(), st.cursor());
        assert!(st.active().is_some());
        let mut buf = [0u8; 2];
        assert_eq!(digits(7, &mut buf), "7");
        assert_eq!(digits(42, &mut buf), "42");
        assert_eq!(digits(500, &mut buf), "99");
    }

    #[test]
    fn mono_pressed_brackets_the_reserved_pad_cells() {
        const LABEL: &str = "Full width";
        let items = [LABEL];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::index(0));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(
                        crate::ReferenceTarget::new(
                            TABS,
                            crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                        )
                        .part(PartRef::item(Part::TAB, ItemKey::index(0))),
                    ),
                    |ui| Tabs::new(TABS).draw(ui, area, &state, &items),
                );
            })
            .commit_presented();

        assert_eq!(row_text(&buffer, AREA.width), "[Full width]");
    }

    #[test]
    fn mono_pressed_closable_tab_keeps_the_close_cell_after_the_bracket() {
        let items = ["tab"];
        let key = ItemKey::index(0);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        let mut state = TabsState::default();
        state.set_active(0, key);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let target = crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                )
                .part(PartRef::item(Part::TAB, key));
                ui.reference(Some(target), |ui| {
                    Tabs::new(TABS)
                        .closable(true)
                        .draw(ui, area, &state, &items);
                });
            })
            .commit_presented();

        assert_eq!(
            buffer.cell(Position::new(4, 0)).map(BufferCell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::PressRight))
        );
        assert_eq!(
            buffer.cell(Position::new(5, 0)).map(BufferCell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Close))
        );
    }

    #[test]
    fn readiness_lane_is_far_right_conditional_patchable_and_replaceable() {
        assert!(Tabs::<&str>::PARTS.contains(&Part::ICON));
        assert_eq!(draw_status(None), draw_status(Some(Status::Ready)));
        let busy = draw_status(Some(Status::Busy));
        let frame = Theme::junie()
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        assert_eq!(
            busy.cell(Position::new(10, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(frame)
        );

        let patch = [(Part::ICON, StylePatch::new().add(Modifier::UNDERLINED))];
        let state = TabsState::default();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS)
                    .status(Status::Busy)
                    .patch_part(&patch)
                    .draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert!(
            buffer
                .cell(Position::new(10, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );

        let seen = Cell::new(None);
        let slot = |_ui: &mut Ui<'_>, area: Rect| seen.set(Some(area));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS)
                    .status(Status::Error)
                    .slot(Part::ICON, &slot)
                    .draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert_eq!(seen.get(), Some(Rect::new(10, 0, 1, 1)));
    }

    #[test]
    fn active_rule_comes_only_from_semantic_state() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = TabsState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS).draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(0, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::RuleQuiet))
        );

        let active = TabsState {
            active: Some(ItemKey::index(0)),
            ..TabsState::default()
        };
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS).draw(ui, area, &active, &["tab"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(0, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::RuleActive))
        );
    }

    fn render_semantic_pair(
        semantic: bool,
        area: Rect,
        state: &TabsState,
        items: &[&str],
        previous: Option<TabWindowReceipt>,
        level: ColorLevel,
        target: Option<crate::ReferenceTarget>,
    ) -> (
        Buffer,
        Vec<termrock_runtime::hit::Region>,
        Option<Position>,
        TabLayout,
    ) {
        let tabs = Tabs::new(TABS)
            .allow_new(true)
            .closable(true)
            .status(Status::Error);
        let layout = tabs.layout_default(area, area, state, items, previous);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(level));
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(target, |ui| {
                    if semantic {
                        tabs.draw_layout(ui, &layout)
                            .expect("same tabs configuration draws its planned layout")
                    } else {
                        tabs.draw(ui, area, state, items)
                    }
                });
            })
            .commit_presented();
        (
            buffer,
            runtime.registry().regions().to_vec(),
            runtime.cursor(),
            layout,
        )
    }

    #[test]
    fn default_semantic_layout_matches_legacy_cells_parts_and_cursor_profiles() {
        let items = ["", "one ", "界e\u{301}", "x\ty", "tail    ", "next"];
        let mut state = TabsState::default();
        state.set_active(2, ItemKey::index(2));
        let targets = [
            None,
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::FOCUSED | crate::ReferenceState::FOCUS_VISIBLE,
                )
                .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(TABS, crate::ReferenceState::HOVERED)
                    .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::FOCUSED | crate::ReferenceState::PRESSED,
                )
                .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::HOVERED | crate::ReferenceState::PRESSED,
                )
                .part(PartRef::item(Part::CLOSE, ItemKey::index(2))),
            ),
        ];

        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            for width in [0, 1, 2, 8, 13, 24, 40] {
                let area = Rect::new(3, 2, width, 2);
                for target in targets {
                    let legacy =
                        render_semantic_pair(false, area, &state, &items, None, level, target);
                    let semantic =
                        render_semantic_pair(true, area, &state, &items, None, level, target);
                    assert_eq!(semantic.0, legacy.0, "cells differ at {width} / {level:?}");
                    assert_eq!(semantic.1, legacy.1, "parts differ at {width} / {level:?}");
                    assert_eq!(
                        semantic.2, legacy.2,
                        "cursor differs at {width} / {level:?}"
                    );
                    assert_eq!(semantic.3.viewport_len, legacy.3.viewport_len);
                    assert_eq!(semantic.3.content_len, items.len());
                    assert_eq!(semantic.3.clip(), area);
                }
            }
        }
    }

    #[test]
    fn explicit_window_receipt_preserves_warm_overflow_reservation() {
        let items = ["one", "two", "three", "four", "five"];
        let area = Rect::new(0, 0, 18, 2);
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let mut legacy = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy_buf = Buffer::empty(area);
        let mut semantic_buf = Buffer::empty(area);

        legacy
            .draw_scene(area, &mut legacy_buf, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        let first = tabs.layout_default(area, area, &state, &items, None);
        semantic
            .draw_scene(area, &mut semantic_buf, |ui, _| {
                tabs.draw_layout(ui, &first).expect("valid first layout");
            })
            .commit_presented();
        assert_eq!(semantic_buf, legacy_buf);

        legacy_buf = Buffer::empty(area);
        semantic_buf = Buffer::empty(area);
        legacy
            .draw_scene(area, &mut legacy_buf, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        let warm = tabs.layout_default(area, area, &state, &items, first.window_receipt());
        assert!(warm.overflow_reserved);
        semantic
            .draw_scene(area, &mut semantic_buf, |ui, _| {
                tabs.draw_layout(ui, &warm).expect("valid warm layout");
            })
            .commit_presented();
        assert_eq!(semantic_buf, legacy_buf);
        assert_eq!(semantic.registry().regions(), legacy.registry().regions());
    }

    struct SegmentedDisplay;

    impl fmt::Display for SegmentedDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("e")?;
            f.write_str("\u{301}")?;
            f.write_str("界 ")
        }
    }

    struct PiecesDisplay(&'static [&'static str]);

    impl fmt::Display for PiecesDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for piece in self.0 {
                f.write_str(piece)?;
            }
            Ok(())
        }
    }

    struct CountingDisplay<'a>(&'a Cell<usize>);

    impl fmt::Display for CountingDisplay<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            f.write_str("counted")
        }
    }

    struct KeyedDisplay {
        key: u64,
        label: &'static str,
    }

    impl fmt::Display for KeyedDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.label)
        }
    }

    #[test]
    fn semantic_layout_preserves_display_write_boundaries_and_custom_rows_stay_legacy() {
        let area = Rect::new(0, 0, 20, 2);
        let items = [SegmentedDisplay];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);
        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("valid segmented layout");
            })
            .commit_presented();
        assert_eq!(semantic, legacy);

        let calls = Cell::new(0usize);
        let custom = Tabs::new(TABS).row(|_: &&str, row: &mut RowUi<'_>| {
            calls.set(calls.get() + 1);
            row.label("custom");
        });
        let mut custom_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut custom_buffer = Buffer::empty(area);
        custom_runtime
            .draw_scene(area, &mut custom_buffer, |ui, area| {
                custom.draw(ui, area, &state, &["item"]);
            })
            .commit_presented();
        assert_eq!(calls.get(), 1);
        assert_eq!(
            custom_buffer
                .cell(Position::new(1, 0))
                .map(BufferCell::symbol),
            Some("c")
        );
    }

    #[test]
    fn semantic_layout_matches_legacy_for_left_clipped_segmented_labels() {
        let area = Rect::new(0, 0, 30, 3);
        let clip = Rect::new(5, 0, 20, 3);
        let items = [PiecesDisplay(&["ab", "cd"]), PiecesDisplay(&["next"])];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let layout = tabs.layout_default(area, clip, &state, &items, None);
        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);

        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, _| {
                ui.with_area(clip, |ui| tabs.draw(ui, area, &state, &items));
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                ui.with_area(clip, |ui| {
                    tabs.draw_layout(ui, &layout)
                        .expect("same tabs configuration draws its planned layout");
                });
            })
            .commit_presented();

        assert_eq!(semantic, legacy);
        assert_eq!(
            semantic_runtime.registry().regions(),
            legacy_runtime.registry().regions()
        );
        assert_eq!(semantic_runtime.cursor(), legacy_runtime.cursor());
    }

    #[test]
    fn planned_draw_consumes_owned_text_and_rejects_a_different_configuration() {
        let area = Rect::new(0, 0, 20, 2);
        let formats = Cell::new(0);
        let items = [CountingDisplay(&formats)];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(formats.get(), 1);
        assert_eq!(
            layout
                .part_bounds(PartRef::item(Part::TAB, ItemKey::index(0)))
                .map(|(logical, visible)| (logical.width, visible.width)),
            Some((11, 11))
        );
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("same configuration draws the plan");
            })
            .commit_presented();
        assert_eq!(formats.get(), 1, "draw must use owned formatted text");
        let mismatch = Cell::new(None);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                mismatch.set(Some(
                    Tabs::<CountingDisplay<'_>>::new(TABS).draw_layout(ui, &layout),
                ));
            })
            .commit_presented();
        assert_eq!(mismatch.get(), Some(Err(TabLayoutMismatch)));
    }

    #[test]
    fn semantic_rows_and_parts_keep_stable_keys_after_reordering() {
        let items = [
            KeyedDisplay {
                key: 30,
                label: "third",
            },
            KeyedDisplay {
                key: 10,
                label: "first",
            },
            KeyedDisplay {
                key: 20,
                label: "second",
            },
        ];
        let mut state = TabsState::default();
        state.set_active(1, ItemKey::num(20));
        state.first = Some(ItemKey::num(10));
        state.first_index = 0;
        let tabs = Tabs::new(TABS).key(|item: &KeyedDisplay| ItemKey::num(item.key));
        let area = Rect::new(0, 0, 24, 2);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(layout.first_index(), 1);
        assert_eq!(
            layout.rows().iter().map(|row| row.key).collect::<Vec<_>>(),
            [ItemKey::num(10), ItemKey::num(20)]
        );
        assert_eq!(layout.active_key(), Some(ItemKey::num(20)));
        assert!(
            layout
                .part_bounds(PartRef::item(Part::TAB, ItemKey::num(20)))
                .is_some()
        );

        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);
        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("same keyed tabs configuration draws its plan");
            })
            .commit_presented();
        assert_eq!(semantic, legacy);
        assert_eq!(
            semantic_runtime.registry().regions(),
            legacy_runtime.registry().regions()
        );
    }
}
