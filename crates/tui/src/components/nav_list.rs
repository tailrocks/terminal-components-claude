//! `NavList` (`COMPONENT_ARCHITECTURE.md` §18.3 #1, §18.3 #2, Appendix A 4C).
//!
//! The showcase's hand-rolled sidebar navigation, promoted to the library.
//! There is no legacy *widget* to replace — the control lived inside
//! `showcase/pages/sidebars.rs`, registered its own focus entry with
//! `ctx.ring.register` and located a clicked row with a reverse scan over 22
//! derived ids. Both disappear here: the runtime owns the focus entry, and a
//! click arrives already carrying the row's [`ItemKey`].
//!
//! What the sidebar had and this keeps: section headings, a collapsed
//! icon-only mode, per-item badges, disabled items the cursor skips, and the
//! separation of the *current* destination from the keyboard *cursor*.

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at};
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation, RowFn, RowUi,
};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{Family, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// An entry's badge accessor: the trailing text for an entry, or `None`
/// when it has no badge.
pub type BadgeFn<'a, T> = &'a dyn Fn(&T) -> Option<&str>;

/// How much of a nav row is shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NavMode {
    /// Icon, label and badge.
    #[default]
    Full,
    /// The icon column only — the narrow sidebar.
    Collapsed,
}

/// What a nav list reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavListAction {
    /// The cursor moved; the current destination is unchanged.
    Moved(ItemKey),
    /// A destination was chosen. This is the navigation event.
    Chose(ItemKey),
    /// A destination was chosen and its content pane should receive focus.
    EnterContent(ItemKey),
}

/// The const-constructible commands of the nav keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavListCmd {
    /// Cursor to the previous enabled entry.
    Up,
    /// Cursor to the next enabled entry.
    Down,
    /// Cursor to the first enabled entry.
    Home,
    /// Cursor to the last enabled entry.
    End,
    /// Choose the cursor entry.
    Choose,
    /// Choose the cursor entry and request entry into its content pane.
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

/// Durable state of a [`NavList`]: the keyboard cursor and the current
/// destination, both keys.
///
/// They are separate on purpose: moving the cursor through a sidebar must
/// not navigate, which is the same cursor/value split `RadioGroup` makes
/// (§12.2).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct NavListState {
    core: CollectionCore,
    current: Option<ItemKey>,
    current_index: usize,
}

impl NavListState {
    /// A nav list with no cursor and no destination.
    #[must_use]
    pub const fn new() -> Self {
        NavListState {
            core: CollectionCore::new(),
            current: None,
            current_index: 0,
        }
    }

    /// The keyboard cursor.
    #[must_use]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The current destination.
    #[must_use]
    pub const fn current(&self) -> Option<ItemKey> {
        self.current
    }

    /// Set the current destination without moving the cursor. The caller
    /// uses this to reflect navigation that happened elsewhere.
    pub const fn set_current(&mut self, key: Option<ItemKey>) {
        self.current = key;
        self.current_index = 0;
    }

    /// Point the cursor at `(index, key)`.
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

/// A sidebar navigation list: sections, a current destination, a keyboard
/// cursor that skips disabled entries, badges and a collapsed icon-only
/// mode.
///
/// ## Construction
/// `NavList::new(id)`; the entries are passed to each phase, never held.
///
/// ## Ownership
/// The caller owns the entries (`&[T]` per phase) and a [`NavListState`].
/// The runtime owns focus, hover and press.
///
/// ## Configuration
/// `.mode(NavMode)` (`Full`), `.section(&dyn Fn(&T) -> &str)` (no sections),
/// `.icon(&dyn Fn(&T) -> &str)` (no icons), `.badge(&dyn Fn(&T) ->
/// Option<&str>)` (no badges), `.disabled_item(&dyn Fn(&T) -> bool)`,
/// `.disabled(bool)` (default `false`), `.key(Fn(&T) -> ItemKey)`
/// (`ByIndex`, unstable under reorder), `.row(Fn(&T, &mut RowUi))`
/// (`DefaultRow`: `Display`), `.patch`, `.patch_part`, `.slot`,
/// runtime state.
///
/// ## Variants
/// `Family::LIST`, `DEFAULT` only. A nav list resolves through the list
/// recipe deliberately: it *is* a list, the recipe carries the row state
/// rules a sidebar needs, and there is no `Family::NAV`.
///
/// ## States
/// The list wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` and `PRESSED` from
/// the runtime and `DISABLED` from `.disabled`. A row derives `SELECTED`
/// when it is the current destination, the runtime's focus flags when it is
/// the cursor, and `DISABLED` from `.disabled_item`. The list takes **no**
/// `.status(Status)` prop: it paints no readiness affordance, and §11.4
/// forbids accepting the prop without one.
///
/// ## Actions
/// `Moved(k)`, `Chose(k)`, `EnterContent(k)`. Moving the cursor never
/// navigates. `Enter`, `Space` and click choose while Right and `l` choose
/// and ask the shell to focus the content pane.
///
/// ## Focus
/// One `Focusable` stop for the whole list (`Disabled` when `.disabled`);
/// does not swallow typing. Individual entries are `Part` regions, not focus
/// stops, which is what deletes the sidebar's own `ctx.ring.register`.
///
/// ## Keyboard
/// `↑`/`k` and `↓`/`j` move to the neighbouring **enabled** entry,
/// `Home`/`g` and `End`/`G` to the first and last enabled entry, `Enter` and
/// `Space` choose, and Right/`l` enter content. Modified `l` is not bound.
///
/// ## Mouse
/// `PartRef::item(Part::ROW, k)`: press moves the cursor, click chooses.
/// Disabled entries register no region at all, so they cannot be clicked.
///
/// ## Layout
/// Every section change after the first gets one blank separator. Full mode
/// then paints a nonempty heading; collapsed mode paints no heading. Every
/// entry gets gutter, current-marker and icon cells; full mode also invokes
/// the renderer with the badge budget already reserved on the right. Rows
/// are laid out from the top and clipped at the bottom — a nav list does
/// **not** scroll. `measure` is `(6…, entries)` collapsed and
/// `(12…, entries)` full; `draw` returns `area`. `0×0` registers nothing
/// (R5).
///
/// ## Parts
/// `CONTAINER` (the sidebar surface and each row's fill), `GUTTER` (the
/// focus column), `MARKER` (the current-destination affordance), `ICON` (the
/// icon column), `HEADER` (a section heading), `BADGE` (an entry's badge),
/// `LABEL` (resolved through [`RowUi`] by the row renderer, `NavMode::Full`
/// only). `Part::ROW` is a hit region only and is deliberately not styled.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach every member of [`Self::PARTS`]. The
/// scoped [`RowUi`] carrier forwards owner patches only to `CONTAINER` and
/// `LABEL`; arbitrary row parts remain row-owned. `.slot(p, …)` changes
/// painted cells for exactly `GUTTER`, `MARKER`, `ICON`, `HEADER` and
/// `BADGE`.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under insert, remove
/// and reorder. The cursor and the current destination are both `ItemKey`,
/// so a reorder of the sidebar moves neither.
///
/// ## Testing
/// `NavListCase` with `ACTIVATES | DISABLEABLE | FOCUSABLE | COLLECTION |
/// SELECTS`; `render::components::nav_list::*`. The fixture must give at
/// least one entry a section and at least one a badge, or `Part::HEADER` and
/// `Part::BADGE` are never resolved and the parts check has nothing to
/// compare (§33.4).
///
/// ## Invariants
/// `reconcile` runs before any action is emitted; a disabled entry
/// registers no region and the cursor never lands on one; a frame allocates
/// nothing per row; the cursor and the destination are independent, so
/// arrowing through the sidebar never navigates; the row renderer runs only
/// for visible full-mode item rows.
pub struct NavList<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    mode: NavMode,
    section: Option<&'a dyn Fn(&T) -> &str>,
    icon: Option<&'a dyn Fn(&T) -> &str>,
    badge: Option<BadgeFn<'a, T>>,
    disabled_item: Option<&'a dyn Fn(&T) -> bool>,
    disabled: bool,
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
    /// A nav list keyed by index and painted through `Display`.
    #[must_use]
    pub const fn new(id: Id) -> Self {
        NavList {
            id,
            key: ByIndex,
            row: DefaultRow,
            mode: NavMode::Full,
            section: None,
            icon: None,
            badge: None,
            disabled_item: None,
            disabled: false,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> NavList<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::ICON,
        Part::LABEL,
        Part::HEADER,
        Part::BADGE,
    ];

    /// The width a full sidebar prefers.
    pub const PREFERRED_WIDTH: u16 = 24;

    /// The width a collapsed sidebar prefers.
    pub const COLLAPSED_WIDTH: u16 = 6;

    /// The id.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    /// The width this list prefers in its current mode.
    #[must_use]
    pub const fn width(&self) -> u16 {
        match self.mode {
            NavMode::Full => Self::PREFERRED_WIDTH,
            NavMode::Collapsed => Self::COLLAPSED_WIDTH,
        }
    }

    /// Full or collapsed.
    #[must_use]
    pub const fn mode(mut self, m: NavMode) -> Self {
        self.mode = m;
        self
    }

    /// The section an entry belongs to. A heading is painted whenever the
    /// text changes from the previous entry's; an empty section suppresses
    /// the heading.
    #[must_use]
    pub const fn section(mut self, f: &'a dyn Fn(&T) -> &str) -> Self {
        self.section = Some(f);
        self
    }

    /// The icon column's text for an entry, clipped to one cell.
    #[must_use]
    pub const fn icon(mut self, f: &'a dyn Fn(&T) -> &str) -> Self {
        self.icon = Some(f);
        self
    }

    /// A trailing badge for an entry.
    #[must_use]
    pub const fn badge(mut self, f: BadgeFn<'a, T>) -> Self {
        self.badge = Some(f);
        self
    }

    /// Which entries are not selectable.
    #[must_use]
    pub const fn disabled_item(mut self, f: &'a dyn Fn(&T) -> bool) -> Self {
        self.disabled_item = Some(f);
        self
    }

    /// Disable the whole list: it stays in the ring, unreachable, and
    /// ignores every input.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> NavList<'a, T, K2, R> {
        NavList {
            id: self.id,
            key: k,
            row: self.row,
            mode: self.mode,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// A row painter, used in `NavMode::Full` only.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> NavList<'a, T, K, R2> {
        NavList {
            id: self.id,
            key: self.key,
            row: r,
            mode: self.mode,
            section: self.section,
            icon: self.icon,
            badge: self.badge,
            disabled_item: self.disabled_item,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// Showcase / fixture use only (A11).
    /// The derived half of the state (§39.2, Invariant Q).
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

    /// The nearest enabled entry at or after `from` when `forward`, at or
    /// before it otherwise. This is the sidebar's disabled-skipping rule,
    /// which the legacy control implemented twice, once per direction.
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

    /// The update phase: reconcile, then drain keys and the pointer.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut NavListState,
        items: &[T],
    ) -> Response<NavListAction> {
        if self.disabled {
            return Response::ignored();
        }
        let len = items.len();
        let _ = st.core.reconcile_with(
            len,
            |i| self.key_at(items, i),
            |i| self.enabled_at(items, i),
        );
        st.reconcile_current(len, &|i| self.key_at(items, i));
        if st.core.cursor().is_none()
            && let Some(i) = self.seek(items, 0, true)
        {
            st.core.set_cursor(i, self.key_at(items, i));
        }
        let mut acc = Acc::<NavListAction>::new();
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(NavListCmd::Up) => {
                            if cur == 0 {
                                acc.consumed();
                            } else {
                                self.move_cursor(st, items, cur.saturating_sub(1), false, &mut acc);
                            }
                        }
                        Some(NavListCmd::Down) => {
                            self.move_cursor(st, items, cur.saturating_add(1), true, &mut acc);
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

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &NavListState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
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
        let container = self.ov.style(
            ui,
            self.id,
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let mut y = area.y;
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let mut section: Option<&str> = None;
        for (i, item) in items.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }
            let s = self.section_of(item);
            let new_group = section != Some(s);
            if new_group {
                if section.is_some() {
                    y = y.saturating_add(1);
                }
                if y >= area.bottom() {
                    break;
                }
                if self.mode == NavMode::Full && !s.is_empty() {
                    self.paint_header(ui, row_at(area, y), s, live);
                    y = y.saturating_add(1);
                }
                section = Some(s);
                if y >= area.bottom() {
                    break;
                }
            }
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
            let rect = row_at(area, y);
            self.paint_row(ui, rect, flags, key, item);
            if !ui.is_inert() && !self.is_disabled(item) {
                ui.register_part(self.id, PartRef::item(Part::ROW, key), rect);
            }
            y = y.saturating_add(1);
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
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(rect, h.style);
        let inner = Rect {
            x: rect.x.saturating_add(1),
            width: rect.width.saturating_sub(1),
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
        let body = Rect {
            x: rect.x.saturating_add(4),
            width: rect.width.saturating_sub(4),
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
            );
            self.row.row(item, &mut r);
        }
        if let Some(text) = badge {
            self.paint_cell_part(ui, badge_area, Part::BADGE, flags, Some(text));
        }
    }

    /// Paint one reserved cell of a row: the caller's `text` when there is
    /// one and the recipe does not name a glyph, else the resolved glyph,
    /// else the bare style.
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

    /// The natural size: the mode's width, one row per entry plus its
    /// section headings.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (
                match self.mode {
                    NavMode::Full => 12,
                    NavMode::Collapsed => 4,
                },
                1,
            ),
            preferred: (self.width(), c.max.1),
        }
        .fit(c)
    }
}

/// One row of `area` at `y`, or an empty rect when `y` is outside.
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
    use crate::collection::{KeyFn, Reconcile, RowFn, RowUi};
    use crate::event::{Chord, KeyCode, KeyModifiers};
    use crate::id::{Id, ItemKey, Part, PartRef};
    use crate::intent::{IntentQueue, Phase};
    use crate::keymap::Binding;
    use crate::response::{Response, StateFlags};
    use crate::theme::{StylePatch, Theme};
    use crate::ui::cx::{FrameServices, LastFrame};
    use crate::ui::{Cx, FrameState, Ui, UiCore};

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

    fn update_action(
        nav: &NavList<'_, E, fn(&E) -> ItemKey>,
        state: &mut NavListState,
        action: ActionKey,
    ) -> (Response<NavListAction>, Option<Id>) {
        let mut intents = IntentQueue::new();
        intents.binding(NAV, action, Chord::key(KeyCode::Enter));
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let response = {
            let mut cx = Cx::new(&intents, &mut services, &mut core, &last, &theme, None);
            nav.update(&mut cx, state, &ITEMS)
        };
        (response, services.focus_request)
    }

    fn update_click(
        nav: &NavList<'_, E, fn(&E) -> ItemKey>,
        state: &mut NavListState,
        key: ItemKey,
    ) -> Response<NavListAction> {
        let mut intents = IntentQueue::new();
        intents.pointer(
            NAV,
            Phase::Click,
            PartRef::item(Part::ROW, key),
            Position::new(0, 0),
            Position::new(0, 0),
            KeyModifiers::NONE,
        );
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let mut cx = Cx::new(&intents, &mut services, &mut core, &last, &theme, None);
        nav.update(&mut cx, state, &ITEMS)
    }

    fn update_no_intents<T, K: KeyFn<T>, R: RowFn<T>>(
        nav: &NavList<'_, T, K, R>,
        state: &mut NavListState,
        items: &[T],
    ) -> Response<NavListAction> {
        let intents = IntentQueue::new();
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let mut cx = Cx::new(&intents, &mut services, &mut core, &last, &theme, None);
        nav.update(&mut cx, state, items)
    }

    /// The sidebar's disabled-skipping rule, in both directions. The legacy
    /// control wrote it twice, once per arrow, and neither copy handled
    /// "there is nothing enabled left" — it reported `Changed` regardless.
    #[test]
    fn the_cursor_skips_disabled_entries_in_both_directions() {
        let nav = list();
        let mut st = NavListState::new();
        st.set_cursor(0, ItemKey::text("Tasks"));

        // down from Tasks(0) skips Runs(1, disabled) and lands on Branches(2)
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 1, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Branches")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            Some(&NavListAction::Moved(ItemKey::text("Branches")))
        );

        // down again skips nothing; Members(3) is enabled
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 3, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            Some(&NavListAction::Moved(ItemKey::text("Members")))
        );

        // down from the last enabled entry has nowhere to go: Billing(4) is
        // disabled, so the cursor must NOT move and must not report a move
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 4, true, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(
            acc.finish(NAV).action_ref(),
            None,
            "a cursor that cannot move must not report Moved"
        );

        // and upward, skipping Runs the other way
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 2, false, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Branches")));
        let mut acc = Acc::<NavListAction>::new();
        nav.move_cursor(&mut st, &ITEMS, 1, false, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::text("Tasks")));
    }

    #[test]
    fn boundary_commands_are_consumed_without_false_moved_actions() {
        let nav = list();
        let mut st = NavListState::new();
        st.set_cursor(0, ItemKey::text("Tasks"));
        for action in [
            ActionKey::custom("nav-list.up"),
            ActionKey::custom("nav-list.home"),
            ActionKey::custom("nav-list.home-vim"),
        ] {
            let (response, _) = update_action(&nav, &mut st, action);
            assert!(response.is_consumed());
            assert_eq!(response.action_ref(), None);
            assert_eq!(st.cursor(), Some(ItemKey::text("Tasks")));
        }
    }

    #[test]
    fn steady_current_reconciliation_is_constant_time_and_invalidation_repairs_reorder() {
        let accesses = Cell::new(0usize);
        let key_of = |item: &u64| {
            accesses.set(accesses.get().saturating_add(1));
            ItemKey::num(*item)
        };
        let nav = NavList::new(NAV).key(key_of);
        let mut items: Vec<u64> = (0..100_000).collect();
        let middle = items.len() / 2;
        let current = ItemKey::num(items[middle]);
        let mut state = NavListState::new();
        state.set_cursor(middle, current);
        state.set_current(Some(current));

        let _ = update_no_intents(&nav, &mut state, &items);
        accesses.set(0);
        let response = update_no_intents(&nav, &mut state, &items);
        assert_eq!(response.action_ref(), None);
        assert_eq!(
            accesses.get(),
            3,
            "steady update must probe only stamp endpoints and cached current"
        );

        items.swap(middle, middle.saturating_add(1));
        state.invalidate();
        let _ = update_no_intents(&nav, &mut state, &items);
        assert_eq!(state.cursor(), Some(current));
        assert_eq!(state.current(), Some(current));

        accesses.set(0);
        let _ = update_no_intents(&nav, &mut state, &items);
        assert_eq!(
            accesses.get(),
            3,
            "reordered state must return to cached steady probes"
        );
    }

    #[test]
    fn right_and_plain_l_enter_content_while_choose_inputs_stay_distinct() {
        let nav = list();
        for action in [
            ActionKey::custom("nav-list.enter-content"),
            ActionKey::custom("nav-list.enter-content-vim"),
        ] {
            let mut st = NavListState::new();
            st.set_cursor(2, ItemKey::text("Branches"));
            let (response, focus_request) = update_action(&nav, &mut st, action);
            assert_eq!(
                response.action_ref(),
                Some(&NavListAction::EnterContent(ItemKey::text("Branches")))
            );
            assert_eq!(st.current(), Some(ItemKey::text("Branches")));
            assert_eq!(focus_request, None, "NavList must not move focus itself");
        }

        for action in [
            ActionKey::custom("nav-list.choose"),
            ActionKey::custom("nav-list.choose-space"),
        ] {
            let mut st = NavListState::new();
            st.set_cursor(2, ItemKey::text("Branches"));
            let (response, _) = update_action(&nav, &mut st, action);
            assert_eq!(
                response.action_ref(),
                Some(&NavListAction::Chose(ItemKey::text("Branches")))
            );
        }

        assert_eq!(
            Binding::command(&TABLE, ActionKey::custom("nav-list.unknown")),
            None
        );

        let mut st = NavListState::new();
        let response = update_click(&nav, &mut st, ItemKey::text("Members"));
        assert_eq!(
            response.action_ref(),
            Some(&NavListAction::Chose(ItemKey::text("Members")))
        );
        assert_eq!(st.cursor(), Some(ItemKey::text("Members")));
        assert_eq!(st.current(), Some(ItemKey::text("Members")));
    }

    /// Choosing sets the destination; moving the cursor does not. That split
    /// is the whole reason `NavListState` carries two keys.
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

        // a disabled entry can be neither chosen nor made current
        let mut acc = Acc::<NavListAction>::new();
        nav.choose(&mut st, &ITEMS, 4, &mut acc);
        assert_eq!(acc.finish(NAV).action_ref(), None);
        assert_eq!(st.current(), Some(ItemKey::text("Branches")));
    }

    /// A heading is painted once per section change, separated by a blank
    /// row, and never before the first row.
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
        // the second heading is preceded by a blank separator row
        let second = headings.get(1).copied().unwrap_or_default();
        assert!(second >= 2);
        assert_eq!(
            painted.get(second.saturating_sub(1)).map(String::as_str),
            Some(""),
            "the second heading has no separator above it: {painted:?}"
        );
        // and every entry got exactly one row
        for e in ITEMS {
            assert_eq!(
                painted.iter().filter(|r| r.contains(e.0)).count(),
                1,
                "{} is not painted exactly once: {painted:?}",
                e.0
            );
        }
    }

    /// `NavMode::Collapsed` paints the icon column and nothing else. The
    /// legacy control had the same mode; what is new is that the labels are
    /// genuinely absent rather than painted and clipped, so a narrow sidebar
    /// cannot leak a truncated label.
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
        let mut last = LastFrame::default();
        last.snapshot.hover = Some((NAV, PartRef::item(Part::ROW, ItemKey::text("Branches"))));
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            nav.draw(&mut ui, area, &NavListState::new(), &ITEMS);
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
        let area = Rect::new(0, 0, 24, 8);
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.reference(
                Some(crate::ReferenceTarget::new(
                    NAV,
                    crate::ReferenceState::FOCUSED,
                )),
                |ui| list().draw(ui, area, &NavListState::new(), &ITEMS),
            );
        }
        assert_eq!(frame.registry.regions().len(), 0);
        assert_eq!(frame.ring.reachable().count(), 0);
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
