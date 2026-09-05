//! `List` (`COMPONENT_ARCHITECTURE.md` §12.2, §17.0 A7, Appendix A 4C).

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::scroll_region::ScrollRegion;
use super::{Acc, PartStyle, SlotFn, cell_at, first_row, shift};
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
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
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
/// `.patch_part`, `.slot`.
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
/// One row per item; an optional two-cell readiness rail, gutter, marker,
/// then the renderer's row; a scrollbar column when the items overflow.
/// `measure` is `(24…, items)`;
/// `measured_size` is the same arithmetic as a [`LayerSize`] for a list used
/// as popover content (§26 N1); `draw` returns `area`. `0×0` registers
/// nothing (R5).
///
/// ## Parts
/// `CONTAINER`, `ICON`, `GUTTER`, `MARKER`, `LABEL`, `META`, `TRACK`,
/// `THUMB`, `EMPTY`, `ROW` (hit regions only).
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part; `.slot` on exactly `ICON`,
/// `GUTTER`, `MARKER`, `EMPTY`, `TRACK` and `THUMB`. `TRACK` and `THUMB` are the
/// embedded [`ScrollRegion`]'s, which paints them under this list's own
/// `Id`, so all three overrides are forwarded to it. `CONTAINER`, `LABEL`,
/// `META` and `ROW` are not slot-addressable: `CONTAINER` is the list's own
/// fill, `LABEL` and `META` are painted by the `.row(…)` painter the caller
/// already supplies, and `ROW` is a hit region rather than a painted part.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under
/// insert/remove/reorder. Every action carries an `ItemKey`.
///
/// ## Testing
/// `ListCase` with `ACTIVATES | FOCUSABLE | COLLECTION | SCROLLS |
/// REPORTS_STATUS`;
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
    /// Kept beside `ov` so the nested [`ScrollRegion`] can be built with the
    /// caller's own overrides. `PartStyle` reads back only the slot, and a
    /// scrollbar the container constructs bare drops the container's
    /// `.patch` and `.patch_part` on `TRACK` and `THUMB` — a drop
    /// `Invariant P` cannot see, because the scrollbar styles those parts
    /// under *this* list's `Id` (§45.7 obligation 2).
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
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
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> List<'a, T, K, R> {
    /// The parts this component styles.
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
            patch: self.patch,
            parts: self.parts,
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
            patch: self.patch,
            parts: self.parts,
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
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The embedded scrollbar, wearing this list's own overrides.
    ///
    /// The scrollbar paints `TRACK` and `THUMB` under the **list's** `Id`,
    /// so a caller's `.patch`, `.patch_part` and `.slot` on those two parts
    /// are the list's to forward; constructing it bare dropped all three
    /// (§45.1, §45.7 obligation 2).
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

    /// Showcase / fixture use only (A11).
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
        let mut acc = Acc::<ListAction>::new();
        let bar = ScrollRegion::new(self.id).update(cx, st.core.scroll_mut(), len);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
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
        let surface = self.scrollbar().draw(ui, area, st.core.scroll(), len);
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
                let _ = ov.style(ui, id, Family::LIST, Variant::DEFAULT, Part::EMPTY, live);
                empty.draw(ui, mid, 0);
            }
            return area;
        }
        let view = ScrollRegion::view(st.core.scroll(), content, len);
        let cursor = st.core.cursor();
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        // Data readiness is a property of the whole list, but the rows are the
        // only surface it has: a list whose `.status` is `Error` must say so in
        // the row chrome, or it is indistinguishable from a healthy one once
        // colour is removed (§11.4, §16.2 case 9).
        let status = live
            & (StateFlags::ERROR | StateFlags::WARNING | StateFlags::BUSY | StateFlags::LOADING);
        for (row_i, i) in view.visible_range().enumerate() {
            let Some(item) = items.get(i) else { break };
            let key = self.key.key(item, i);
            let is_cursor = cursor == Some(key);
            let mut flags = status;
            if is_cursor {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            let row_part = PartRef::item(Part::ROW, key);
            if hovered == Some(row_part) {
                flags |= StateFlags::HOVERED;
            }
            if pressed == Some(row_part)
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
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), row);
            }
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::action::ActionKey;
    use crate::intent::IntentQueue;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;
    use crate::ui::UiCore;
    use crate::ui::cx::{FrameServices, LastFrame};

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
        let disabled = |item: &u8| matches!(*item, 1 | 3);
        let list = List::new(ID)
            .select_mode(SelectMode::Multi)
            .disabled_item(&disabled);
        let items = [0_u8, 1, 2, 3, 4];
        let mut state = ListState::default();
        let action = ActionKey::custom("list.toggle-all");
        let mut intents = IntentQueue::new();
        intents.binding(ID, action, Chord::key(KeyCode::Char('a')));
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last_frame = LastFrame::default();
        let response = {
            let theme = Theme::junie();
            let mut cx = Cx::new(
                &intents,
                &mut services,
                &mut core,
                &last_frame,
                &theme,
                None,
            );
            list.update(&mut cx, &mut state, &items)
        };

        assert_eq!(response.action_ref(), Some(&ListAction::ToggledAll));
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
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            List::new(ID)
                .status(status)
                .draw(ui, area, &state, &["one", "two"]);
        });
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
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            List::new(ID)
                .status(Status::Busy)
                .patch_part(&patch)
                .draw(ui, area, &state, &["one"]);
        });
        assert!(
            buffer
                .cell(Position::new(0, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            List::new(ID)
                .status(Status::Error)
                .patch_part(&patch)
                .slot(Part::ICON, &slot)
                .draw(ui, area, &state, &["one"]);
        });
        assert_eq!(seen.get(), Some(Rect::new(0, 0, 1, 1)));
    }

    #[test]
    fn chosen_marker_comes_only_from_semantic_state() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = ListState::default();
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            List::new(ID).draw(ui, area, &state, &["one"]);
        });
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
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            List::new(ID).draw(ui, area, &selected, &["one"]);
        });
        assert_eq!(
            buffer
                .cell(Position::new(1, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Chosen))
        );
    }
}
