//! `List` (`COMPONENT_ARCHITECTURE.md` §12.2, §17.0 A7, Appendix A 4C).

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn, cell_at};
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, EmptyState, KeyFn, KeySet, Reconcile, Reconciliation,
    RowFn, RowUi, SelectMode, Status,
};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::LayerSize;
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::theme::{Family, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// What a list reports; every item action carries the item's key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListAction {
    /// The cursor moved.
    Moved,
    /// An item was chosen (`Single`).
    Chose(ItemKey),
    /// An item's check toggled (`Multi` / `Range`).
    Toggled(ItemKey),
    /// An item was activated (Enter or a double-click).
    Activated(ItemKey),
    /// Every enabled item was checked or unchecked.
    ToggledAll,
}

/// The const-constructible commands of the list keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListCmd {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Cursor up one viewport.
    PageUp,
    /// Cursor down one viewport.
    PageDown,
    /// Cursor to the first item.
    Home,
    /// Cursor to the last item.
    End,
    /// Cursor up, extending the checked range.
    ExtendUp,
    /// Cursor down, extending the checked range.
    ExtendDown,
    /// Choose / toggle the cursor item.
    Choose,
    /// Activate the cursor item.
    Activate,
    /// Toggle every enabled item.
    ToggleAll,
}

const fn b(chord: Chord, cmd: ListCmd, label: &'static str, visible: bool) -> Binding<ListCmd> {
    Binding {
        chord,
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BASE: [Binding<ListCmd>; 11] = [
    b(Chord::key(KeyCode::Up), ListCmd::Up, "Up", true),
    b(Chord::key(KeyCode::Down), ListCmd::Down, "Down", true),
    b(Chord::key(KeyCode::Char('k')), ListCmd::Up, "Up", false),
    b(Chord::key(KeyCode::Char('j')), ListCmd::Down, "Down", false),
    b(
        Chord::key(KeyCode::PageUp),
        ListCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        ListCmd::PageDown,
        "Page down",
        false,
    ),
    b(Chord::key(KeyCode::Home), ListCmd::Home, "First", false),
    b(Chord::key(KeyCode::End), ListCmd::End, "Last", false),
    b(
        Chord::key(KeyCode::Char('g')),
        ListCmd::Home,
        "First",
        false,
    ),
    b(Chord::key(KeyCode::Char('G')), ListCmd::End, "Last", false),
    b(Chord::key(KeyCode::Enter), ListCmd::Activate, "Open", true),
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
        Chord::key(KeyCode::Char(' ')),
        ListCmd::Choose,
        "Toggle",
        true,
    ),
    b(
        Chord::with(KeyCode::Up, KeyModifiers::SHIFT),
        ListCmd::ExtendUp,
        "Extend up",
        false,
    ),
    b(
        Chord::with(KeyCode::Down, KeyModifiers::SHIFT),
        ListCmd::ExtendDown,
        "Extend down",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('a')),
        ListCmd::ToggleAll,
        "All",
        false,
    ),
];

const NONE: [Binding<ListCmd>; 11] = BASE;

/// Durable state of a [`List`]: cursor key, chosen key, checked set,
/// scroll and the reconcile stamp.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ListState {
    core: CollectionCore,
    chosen: Option<ItemKey>,
    anchor: Option<ItemKey>,
}

impl ListState {
    /// The cursor key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The chosen key (`Single`).
    pub const fn chosen(&self) -> Option<ItemKey> {
        self.chosen
    }

    /// The checked set (`Multi` / `Range`).
    pub const fn checked(&self) -> &KeySet {
        self.core.checked()
    }

    /// The checked set, mutably.
    pub const fn checked_mut(&mut self) -> &mut KeySet {
        self.core.checked_mut()
    }

    /// The scroll state.
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    /// Point the cursor at `(index, key)` and reveal it on the next layout.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    /// Choose `key` (`Single`).
    pub fn choose(&mut self, key: Option<ItemKey>) {
        self.chosen = key;
    }
}

impl Reconcile for ListState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some(c) = self.chosen
            && !(0..len).any(|i| key(i) == c)
        {
            self.chosen = None;
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// A keyed, scrollable, single-focus-stop list over borrowed rows.
///
/// ## Construction
/// `List::new(id)`; items are passed to each phase, never held.
///
/// ## Ownership
/// The caller owns the items (`&[T]` per phase) and a [`ListState`]; the
/// runtime owns focus, hover, press, wheel routing and the scrollbar
/// capture.
///
/// ## Configuration
/// `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable under reorder),
/// `.row(Fn(&T, &mut RowUi))` (`DefaultRow`: `Display`), `.select_mode`
/// (`Single`), `.empty(EmptyState)` (a default "Nothing here yet"),
/// `.disabled_item(&dyn Fn(&T) -> bool)`, `.status`, `.patch`,
/// `.patch_part`, `.slot`, `.state_override`.
///
/// ## Variants
/// `Family::LIST`, `DEFAULT` only.
///
/// ## States
/// The list wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from the
/// runtime and `BUSY`/`LOADING`/`ERROR` from `.status`; the cursor row
/// derives `FOCUSED`/`PRESSED`, a chosen row `SELECTED`, a checked row
/// `CHECKED`, a disabled item `DISABLED`. The readiness flags reach every
/// row as well as the container, so an errored or stale list is visible
/// without colour.
///
/// ## Actions
/// `Moved`, `Chose(k)` (Space / click, `Single`), `Toggled(k)` (Space /
/// click, `Multi`/`Range`), `Activated(k)` (Enter / double-click),
/// `ToggledAll` (`a`, `Multi`/`Range`).
///
/// ## Focus
/// One `Focusable` stop for the whole list; does not swallow typing.
///
/// ## Keyboard
/// `↑`/`k`, `↓`/`j`, `PgUp`, `PgDn`, `Home`/`g`, `End`/`G`, `Enter`
/// activate, `Space` choose/toggle; `Multi`/`Range` add `Shift+↑`/`Shift+↓`
/// (extend) and `a` (toggle all).
///
/// ## Mouse
/// `PartRef::item(Part::ROW, k)`: press moves the cursor, click chooses /
/// toggles, double-click activates. `TRACK`/`THUMB` and the wheel go to the
/// embedded [`ScrollRegion`].
///
/// ## Layout
/// One row per item; gutter, marker, then the renderer's row; a scrollbar
/// column when the items overflow. `measure` is `(24…, items)`;
/// `measured_size` is the same arithmetic as a [`LayerSize`] for a list used
/// as popover content (§26 N1); `draw` returns `area`. `0×0` registers
/// nothing (R5).
///
/// ## Parts
/// `CONTAINER`, `GUTTER`, `MARKER`, `LABEL`, `META`, `TRACK`, `THUMB`,
/// `EMPTY`, `ROW` (hit regions only).
///
/// ## Overrides
/// `.patch`, `.patch_part`, `.slot` on `GUTTER`, `MARKER`, `EMPTY`,
/// `TRACK`, `THUMB`.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under
/// insert/remove/reorder. Every action carries an `ItemKey`.
///
/// ## Testing
/// `ListCase` with `ACTIVATES | FOCUSABLE | COLLECTION | SCROLLS`;
/// `render::components::list::*`. `CAPTURES` belongs to the embedded
/// [`ScrollRegion`], whose thumb claims the capture, and is declared by
/// `ScrollRegionCase`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted; only visible rows invoke
/// the renderer; a frame allocates nothing per row.
pub struct List<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    select_mode: SelectMode,
    empty: Option<EmptyState<'a>>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    status: Status,
    ov: Overrides<'a>,
    _t: PhantomData<fn(&T)>,
}

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
    /// A list keyed by index and painted through `Display`.
    pub const fn new(id: Id) -> Self {
        List {
            id,
            key: ByIndex,
            row: DefaultRow,
            select_mode: SelectMode::Single,
            empty: None,
            disabled_item: None,
            status: Status::Ready,
            ov: Overrides::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> List<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];

    /// The width `measure` prefers and `measured_size` clamps into the
    /// design's popup band.
    pub const PREFERRED_WIDTH: u16 = 24;

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R> {
        List {
            id: self.id,
            key: k,
            row: self.row,
            select_mode: self.select_mode,
            empty: self.empty,
            disabled_item: self.disabled_item,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// A row painter.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2> {
        List {
            id: self.id,
            key: self.key,
            row: r,
            select_mode: self.select_mode,
            empty: self.empty,
            disabled_item: self.disabled_item,
            status: self.status,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// The selection mode.
    #[must_use]
    pub fn select_mode(mut self, m: SelectMode) -> Self {
        self.select_mode = m;
        self
    }

    /// What to paint when there are no items.
    #[must_use]
    pub fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    /// Which items are disabled.
    #[must_use]
    pub fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    /// Data readiness.
    #[must_use]
    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    #[must_use]
    pub fn state_override(mut self, s: StateFlags) -> Self {
        self.ov = self.ov.state_override(s);
        self
    }

    fn table(&self) -> &'static [Binding<ListCmd>] {
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

    /// The index of `key` near `hint`, probing the hint before scanning.
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

    /// The update phase: reconcile, then drain keys, pointer and wheel.
    #[expect(
        clippy::too_many_lines,
        reason = "the keymap dispatch and the pointer phases in one drain loop"
    )]
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction> {
        let len = items.len();
        let _ = st.core.reconcile_with(
            len,
            |i| self.key_at(items, i),
            |i| self.enabled_at(items, i),
        );
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
        let mut acc = Acc::<ListAction>::new();
        let bar = ScrollRegion::new(self.id).update(cx, st.core.scroll_mut(), len);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => {
                    let cur = st.core.cursor_index();
                    match Binding::lookup(table, &k) {
                        Some(ListCmd::Up) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), false, &mut acc);
                        }
                        Some(ListCmd::Down) => {
                            self.move_cursor(st, items, cur.saturating_add(1), false, &mut acc);
                        }
                        Some(ListCmd::PageUp) => self.move_cursor(
                            st,
                            items,
                            cur.saturating_sub(viewport),
                            false,
                            &mut acc,
                        ),
                        Some(ListCmd::PageDown) => self.move_cursor(
                            st,
                            items,
                            cur.saturating_add(viewport),
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
                            let all = (0..len)
                                .filter(|&i| self.enabled_at(items, i))
                                .all(|i| st.core.checked().contains(self.key_at(items, i)));
                            if all {
                                st.core.checked_mut().none();
                            } else {
                                st.core.checked_mut().all();
                            }
                            acc.action(ListAction::ToggledAll);
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
                    // the row index from the pointer row and last frame's view
                    let hint = cx.area(self.id).map(|a| {
                        let view = ScrollRegion::view(st.core.scroll(), a, len);
                        view.offset()
                            .saturating_add(usize::from(pos.y.saturating_sub(a.y)))
                    });
                    let Some(i) = self.index_of(items, k, hint) else {
                        acc.consumed();
                        continue;
                    };
                    match phase {
                        Phase::Press => {
                            st.anchor = None;
                            st.core.set_cursor(i, k);
                            acc.changed();
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
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    /// The draw phase.
    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the visible rows with gutter, marker and renderer"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        let len = items.len();
        if !self.ov.is_forced() {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let live = self.ov.flags(ui.state(self.id)) | self.status.flags();
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
        let content = ScrollRegion::new(self.id).draw(ui, area, st.core.scroll(), len);
        if len == 0 {
            let empty = self.empty.unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            });
            let mid = Rect {
                y: area.y.saturating_add(area.height / 2),
                height: area.height.saturating_sub(area.height / 2),
                ..area
            };
            if let Some(f) = ov.slot_for(Part::EMPTY) {
                f(ui, mid);
            } else {
                let _ = ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::EMPTY, live);
                empty.draw(ui, mid, 0);
            }
            return area;
        }
        let view = ScrollRegion::view(st.core.scroll(), content, len);
        let cursor = st.core.cursor();
        let forced = self.ov.is_forced();
        // Data readiness is a property of the whole list, but the rows are the
        // only surface it has: a list whose `.status` is `Error` must say so in
        // the row chrome, or it is indistinguishable from a healthy one once
        // colour is removed (§11.4, §16.2 case 9).
        let status = live
            & (StateFlags::ERROR | StateFlags::WARNING | StateFlags::BUSY | StateFlags::LOADING);
        for (row_i, i) in view.visible_range().enumerate() {
            let Some(item) = items.get(i) else { break };
            let key = self.key.key(item, i);
            let is_cursor = cursor == Some(key) || (forced && cursor.is_none() && row_i == 0);
            let mut flags = status;
            if is_cursor {
                flags |=
                    live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::PRESSED);
                if forced {
                    flags |= live & StateFlags::SELECTED;
                }
            }
            if st.chosen == Some(key) {
                flags |= StateFlags::SELECTED;
            }
            if st.core.checked().contains(key) {
                flags |= StateFlags::CHECKED;
            }
            if self.is_disabled(item) || (forced && live.contains(StateFlags::DISABLED)) {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(StateFlags::PRESSED);
            }
            let row = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: 1,
            };
            let rs = ov.style(
                ui,
                id,
                Family::LIST,
                Variant::DEFAULT,
                Part::CONTAINER,
                flags,
            );
            ui.fill(row, rs.style);
            // gutter
            let gutter_cell = cell_at(row, row.x);
            if let Some(f) = ov.slot_for(Part::GUTTER) {
                f(ui, gutter_cell);
            } else {
                let g = ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::GUTTER, flags);
                match g.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(gutter_cell, glyph, g.style);
                    }
                    Slot::Inherit | Slot::Clear => ui.fill(gutter_cell, g.style),
                }
            }
            // marker
            let marker_cell = cell_at(row, row.x.saturating_add(1));
            if let Some(f) = ov.slot_for(Part::MARKER) {
                f(ui, marker_cell);
            } else {
                let m = ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::MARKER, flags);
                match m.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(marker_cell, glyph, m.style);
                    }
                    Slot::Inherit | Slot::Clear => ui.fill(marker_cell, m.style),
                }
            }
            let rest = Rect {
                x: row.x.saturating_add(3),
                width: row.width.saturating_sub(3),
                ..row
            };
            if !rest.is_empty() {
                let mut r = RowUi::new(ui, id, Family::LIST, Variant::DEFAULT, flags, key, rest);
                self.row.row(item, &mut r);
            }
            ui.register_part(self.id, PartRef::item(Part::ROW, key), row);
        }
        area
    }

    /// The natural size: 24 columns, one row per item.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }

    /// The layer size this list wants as popover content (§26 N1): its
    /// natural width clamped into the design's popup band, and one row per
    /// item up to `popup_max_rows`.
    ///
    /// The list does not own the layer — whoever opened it does — so the
    /// opener passes this to `LayerSpec::size` and, if the item slice can
    /// change while the popover is open, re-asserts it every frame with
    /// [`Cx::resize_layer`].
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let d = cx.design();
        let w = Self::PREFERRED_WIDTH.clamp(d.size.popup_min_width, d.size.popup_max_width);
        let h = items
            .len()
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
