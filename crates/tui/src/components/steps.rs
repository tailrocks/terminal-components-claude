//! `Steps` (`COMPONENT_ARCHITECTURE.md` §12.4, §18.2, Appendix A 4C).
//!
//! The ordered-lifecycle rail. `Steps` deliberately has **no selection**:
//! that is the difference §12.4 preserves between it and `List`. What it has
//! instead is a *frontier* — the first step that has not finished — derived
//! from the step states the caller supplies, and painted.
//!
//! Whether the rail is navigable is chosen by the **constructor**, never by a
//! flag: [`Steps::new`] is a display rail that still answers the pointer and
//! the wheel, and [`Steps::navigable`] adds the focus stop, the cursor and
//! the keymap. That is §23 K2's G4 shape ("capability is chosen by the entry
//! point") rather than the legacy `selectable: bool`.

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::scroll_region::ScrollRegion;
use super::{Acc, Overrides, SlotFn, cell_at};
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
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// Where one step is in its lifecycle.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum StepState {
    /// Not started.
    #[default]
    Queued,
    /// In flight.
    Running,
    /// Finished successfully.
    Done,
    /// Deliberately not run.
    Skipped,
    /// Finished unsuccessfully.
    Failed,
    /// Cannot start until something outside the rail changes.
    Blocked,
}

impl StepState {
    /// The word the rail paints as the step's trailing metadata when the row
    /// renderer supplies none.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            StepState::Queued => "queued",
            StepState::Running => "running",
            StepState::Done => "done",
            StepState::Skipped => "skipped",
            StepState::Failed => "failed",
            StepState::Blocked => "blocked",
        }
    }

    /// Whether the step has finished, one way or another. The frontier is
    /// the first step for which this is `false`.
    #[must_use]
    pub const fn terminal(self) -> bool {
        matches!(
            self,
            StepState::Done | StepState::Skipped | StepState::Failed
        )
    }

    /// The flags a row wears for this state.
    #[must_use]
    pub const fn flags(self) -> StateFlags {
        match self {
            StepState::Queued => StateFlags::empty(),
            StepState::Running => StateFlags::BUSY.union(StateFlags::ACTIVE),
            StepState::Done => StateFlags::CHECKED,
            StepState::Skipped => StateFlags::READ_ONLY,
            StepState::Failed => StateFlags::ERROR,
            StepState::Blocked => StateFlags::WARNING,
        }
    }

    /// The glyph the rail paints into `Part::ICON`, if any.
    #[must_use]
    pub const fn glyph(self) -> Option<GlyphRole> {
        match self {
            StepState::Queued | StepState::Skipped => None,
            StepState::Running => Some(GlyphRole::Bullet),
            StepState::Done => Some(GlyphRole::Checked),
            StepState::Failed => Some(GlyphRole::Error),
            StepState::Blocked => Some(GlyphRole::WarningMark),
        }
    }
}

/// What a navigable rail reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepsAction {
    /// The cursor moved.
    Moved(ItemKey),
    /// A step was activated (Enter, or a click on its row).
    Activated(ItemKey),
}

/// The const-constructible commands of the steps keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepsCmd {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Cursor up one viewport.
    PageUp,
    /// Cursor down one viewport.
    PageDown,
    /// Cursor to the first step.
    Home,
    /// Cursor to the last step.
    End,
    /// Activate the cursor step.
    Activate,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: StepsCmd,
    label: &'static str,
    visible: bool,
) -> Binding<StepsCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const TABLE: [Binding<StepsCmd>; 11] = [
    b(
        "steps.up",
        Chord::key(KeyCode::Up),
        StepsCmd::Up,
        "Up",
        true,
    ),
    b(
        "steps.down",
        Chord::key(KeyCode::Down),
        StepsCmd::Down,
        "Down",
        true,
    ),
    b(
        "steps.up-vim",
        Chord::key(KeyCode::Char('k')),
        StepsCmd::Up,
        "Up",
        false,
    ),
    b(
        "steps.down-vim",
        Chord::key(KeyCode::Char('j')),
        StepsCmd::Down,
        "Down",
        false,
    ),
    b(
        "steps.page-up",
        Chord::key(KeyCode::PageUp),
        StepsCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "steps.page-down",
        Chord::key(KeyCode::PageDown),
        StepsCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "steps.home",
        Chord::key(KeyCode::Home),
        StepsCmd::Home,
        "First",
        false,
    ),
    b(
        "steps.end",
        Chord::key(KeyCode::End),
        StepsCmd::End,
        "Last",
        false,
    ),
    b(
        "steps.home-vim",
        Chord::key(KeyCode::Char('g')),
        StepsCmd::Home,
        "First",
        false,
    ),
    b(
        "steps.end-vim",
        Chord::key(KeyCode::Char('G')),
        StepsCmd::End,
        "Last",
        false,
    ),
    b(
        "steps.activate",
        Chord::key(KeyCode::Enter),
        StepsCmd::Activate,
        "Open",
        true,
    ),
];

/// Durable state of a [`Steps`] rail: the cursor key, the scroll offset and
/// the reconcile stamp. There is no selection and no checked set — that is
/// the point of the component.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct StepsState {
    core: CollectionCore,
    frontier_revision: u64,
}

impl StepsState {
    /// A rail with no cursor.
    #[must_use]
    pub const fn new() -> Self {
        StepsState {
            core: CollectionCore::new(),
            frontier_revision: 0,
        }
    }

    /// The cursor key. Always `None` for a display rail that has never been
    /// clicked.
    #[must_use]
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The scroll state.
    #[must_use]
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    /// Point the cursor at `(index, key)` and reveal it on the next layout.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

    /// Force collection reconciliation and a cold frontier scan on the next
    /// frame.
    ///
    /// Monotonic lifecycle progress needs no invalidation. Call this after a
    /// retry/reset, a same-length interior reorder, or a state regression
    /// before the cached frontier. Once the revision saturates, correctness
    /// wins over cache reuse and every draw scans from the beginning.
    pub fn invalidate(&mut self) {
        self.frontier_revision = self.frontier_revision.saturating_add(1);
        self.core.invalidate();
    }
}

impl Reconcile for StepsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        self.core.reconcile(len, key)
    }

    fn invalidate(&mut self) {
        StepsState::invalidate(self);
    }
}

/// An ordered lifecycle rail with a frontier.
///
/// ## Construction
/// `Steps::new(id)` — the display rail: a hit target and a wheel target, no
/// focus stop, no keymap. `Steps::navigable(id)` — the same rail plus a
/// focus stop, a cursor and the keymap. The two are separate constructors
/// because they are semantically different modes, not a `bool` (§13, §23 K2
/// G4). The steps are passed to each phase, never held.
///
/// ## Ownership
/// The caller owns the steps (`&[T]` per phase) and a [`StepsState`]. The
/// runtime owns focus, hover, press, wheel routing and the scrollbar
/// capture.
///
/// ## Configuration
/// `.step(&dyn Fn(&T) -> StepState)` (default: every step is
/// `StepState::Queued`), `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable
/// under reorder), `.row(Fn(&T, &mut RowUi))` (`DefaultRow`: `Display`),
/// `.disabled(bool)` (default `false`), `.patch`, `.patch_part`, `.slot`,
/// runtime state.
///
/// ## Variants
/// `Family::STEPS`, `DEFAULT` only.
///
/// ## States
/// The rail wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` and `PRESSED` from
/// the runtime and `DISABLED` from `.disabled`. Each row derives its own
/// flags from its [`StepState`] — `BUSY | ACTIVE` running, `CHECKED` done,
/// `READ_ONLY` skipped, `ERROR` failed, `WARNING` blocked — plus `ACTIVE` on
/// the frontier and the runtime's focus flags on the cursor row. The rail
/// takes **no** `.status(Status)` prop: readiness in a rail is per step, and
/// §11.4 forbids accepting a readiness prop without painting its affordance.
///
/// ## Actions
/// `Moved(k)`, `Activated(k)`. A display rail emits `Moved(k)` on a click (the
/// cursor is how it reports which step the pointer chose) and `Activated(k)`
/// on a double-click; a navigable rail also emits `Activated(k)` for `Enter`
/// and a single click.
///
/// ## Focus
/// `Steps::navigable` registers one `Focusable` stop (`Disabled` when
/// `.disabled`); `Steps::new` registers `ClickOnly`, so the rail is
/// addressable and clickable but never in the ring. Neither swallows typing.
///
/// ## Keyboard
/// A navigable rail binds `↑`/`k`, `↓`/`j`, `PgUp`, `PgDn`, `Home`/`g`,
/// `End`/`G` and `Enter`. A display rail binds nothing.
///
/// ## Mouse
/// `PartRef::item(Part::ROW, k)`: press and click move the cursor;
/// double-click activates, and on a navigable rail a single click activates
/// too. `TRACK`/`THUMB` and the wheel go to the embedded [`ScrollRegion`].
///
/// ## Layout
/// One row per step: gutter, the one-cell state glyph, the component-owned
/// lifecycle word at the far right, then the renderer's row in what remains.
/// The renderer may add its own [`RowUi::meta`](crate::collection::RowUi::meta)
/// inside that remaining body. A scrollbar column appears when the steps
/// overflow. `measure` is `(12…, offered height)`; `draw` returns `area`.
/// `0×0` registers nothing (R5).
///
/// ## Parts
/// `CONTAINER` (the rail surface and each row's fill), `GUTTER` (the focus
/// column), `ICON` (the lifecycle glyph), `META` (the direct lifecycle word),
/// `LABEL` (resolved through [`RowUi`] by the row renderer), `TRACK` /
/// `THUMB` (the embedded [`ScrollRegion`]). Caller-painted row metadata also
/// resolves as `META`, but is not component-owned. `Part::ROW` is a hit
/// region only and is deliberately not styled.
///
/// ## Overrides
/// `.patch` and `.patch_part` reach exactly `CONTAINER`, `LABEL`, `GUTTER`,
/// `ICON`, `META`, `TRACK` and `THUMB`. The scroll parts are forwarded to the
/// embedded [`ScrollRegion`]; lifecycle `META` is direct component-owned
/// output. Owner patches are scoped into [`RowUi`]'s automatic `CONTAINER`
/// and `LABEL` only, so caller-painted `META`, `CELL` and custom parts stay
/// row-owned. `.slot(p, …)` changes painted cells for exactly `GUTTER`,
/// `ICON`, `TRACK` and `THUMB`.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under insert, remove
/// and reorder. Every action carries an `ItemKey`.
///
/// ## Testing
/// `StepsCase` with `ACTIVATES | DISABLEABLE | FOCUSABLE | COLLECTION |
/// SCROLLS` over `Steps::navigable`; `render::components::steps::*`.
/// `CAPTURES` belongs to the embedded [`ScrollRegion`] and is declared by
/// `ScrollRegionCase`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted; only visible rows invoke
/// the renderer; a frame allocates nothing per row; the rail never mutates
/// a step's state — the lifecycle belongs to the caller, and the frontier is
/// derived from it in a runtime-owned cache rather than stored semantically.
pub struct Steps<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    step: Option<&'a dyn Fn(&T) -> StepState>,
    navigable: bool,
    disabled: bool,
    ov: Overrides<'a>,
    /// The same three override channels again, kept because `Overrides` has
    /// no readers: forwarding a caller's `.patch` / `.patch_part` / `.slot`
    /// into the embedded [`ScrollRegion`] is the §45.1 defect `List` still
    /// carries, and it cannot be done from the stored `Overrides` alone.
    fwd_patch: Option<&'a StylePatch>,
    fwd_parts: &'a [(Part, StylePatch)],
    fwd_slot: Option<(Part, SlotFn<'a>)>,
    _t: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for Steps<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Steps")
            .field("id", &self.id)
            .field("navigable", &self.navigable)
            .field("disabled", &self.disabled)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<T> Steps<'_, T, ByIndex, DefaultRow> {
    /// The display rail: clickable and scrollable, never a focus stop.
    #[must_use]
    pub const fn new(id: Id) -> Self {
        Steps {
            id,
            key: ByIndex,
            row: DefaultRow,
            step: None,
            navigable: false,
            disabled: false,
            ov: Overrides::new(),
            fwd_patch: None,
            fwd_parts: &[],
            fwd_slot: None,
            _t: PhantomData,
        }
    }

    /// The same rail with a focus stop, a cursor and the keymap.
    #[must_use]
    pub const fn navigable(id: Id) -> Self {
        let mut s = Steps::new(id);
        s.navigable = true;
        s
    }
}

impl<'a, T, K, R> Steps<'a, T, K, R> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::LABEL,
        Part::GUTTER,
        Part::ICON,
        Part::META,
        Part::TRACK,
        Part::THUMB,
    ];

    /// The width `measure` prefers.
    pub const PREFERRED_WIDTH: u16 = 32;

    /// The id.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Whether this rail is a focus stop with a cursor.
    #[must_use]
    pub const fn is_navigable(&self) -> bool {
        self.navigable
    }

    /// The lifecycle accessor.
    #[must_use]
    pub const fn step(mut self, f: &'a dyn Fn(&T) -> StepState) -> Self {
        self.step = Some(f);
        self
    }

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Steps<'a, T, K2, R> {
        Steps {
            id: self.id,
            key: k,
            row: self.row,
            step: self.step,
            navigable: self.navigable,
            disabled: self.disabled,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    /// A row painter, called only for the visible rows.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Steps<'a, T, K, R2> {
        Steps {
            id: self.id,
            key: self.key,
            row: r,
            step: self.step,
            navigable: self.navigable,
            disabled: self.disabled,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    /// Disable the whole rail: it ignores every input, and a navigable rail
    /// stays in the ring unreachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.patch(p);
        self.fwd_patch = Some(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self.fwd_parts = ps;
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self.fwd_slot = Some((p, f));
        self
    }

    /// The embedded scroll region, carrying the caller's overrides (§45.1).
    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut s = ScrollRegion::new(self.id).patch_part(self.fwd_parts);
        if let Some(p) = self.fwd_patch {
            s = s.patch(p);
        }
        if let Some((p, f)) = self.fwd_slot {
            s = s.slot(p, f);
        }
        s
    }

    /// The derived half of the state (§39.2, Invariant Q).
    const fn derived(&self) -> StateFlags {
        if self.disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::empty()
        }
    }

    fn state_of(&self, item: &T) -> StepState {
        self.step.map_or(StepState::Queued, |f| f(item))
    }

    fn table(&self) -> &'static [Binding<StepsCmd>] {
        if self.navigable && !self.disabled {
            &TABLE
        } else {
            &[]
        }
    }
}

/// Runtime-owned acceleration for the derived lifecycle frontier.
///
/// It contains only keys, indexes and caller-controlled stamps. Dropping or
/// clearing it can change cost, never component semantics.
#[derive(Clone, Copy, Debug, Default)]
struct FrontierCache {
    initialized: bool,
    revision: u64,
    len: usize,
    first: Option<ItemKey>,
    last: Option<ItemKey>,
    frontier_index: usize,
    frontier_key: Option<ItemKey>,
}

impl FrontierCache {
    fn sync<T, K: KeyFn<T>, R>(
        &mut self,
        steps: &Steps<'_, T, K, R>,
        state: &StepsState,
        items: &[T],
    ) -> Option<ItemKey> {
        let len = items.len();
        let first = (!items.is_empty()).then(|| steps.key_at(items, 0));
        let last = (!items.is_empty()).then(|| steps.key_at(items, len.saturating_sub(1)));
        let stamp_matches = state.frontier_revision != u64::MAX
            && self.initialized
            && self.revision == state.frontier_revision
            && self.len == len
            && self.first == first
            && self.last == last;

        let start = if stamp_matches {
            match self.frontier_key {
                Some(key)
                    if self.frontier_index < len
                        && steps.key_at(items, self.frontier_index) == key =>
                {
                    let Some(item) = items.get(self.frontier_index) else {
                        return self.rebuild(steps, state, items, first, last, 0);
                    };
                    if !steps.state_of(item).terminal() {
                        return Some(key);
                    }
                    self.frontier_index.saturating_add(1)
                }
                Some(_) => 0,
                None => return None,
            }
        } else {
            0
        };
        self.rebuild(steps, state, items, first, last, start)
    }

    fn rebuild<T, K: KeyFn<T>, R>(
        &mut self,
        steps: &Steps<'_, T, K, R>,
        state: &StepsState,
        items: &[T],
        first: Option<ItemKey>,
        last: Option<ItemKey>,
        start: usize,
    ) -> Option<ItemKey> {
        let found = items
            .iter()
            .enumerate()
            .skip(start)
            .find(|(_, item)| !steps.state_of(item).terminal());
        let (frontier_index, frontier_key) = found.map_or((items.len(), None), |(index, _)| {
            (index, Some(steps.key_at(items, index)))
        });
        *self = FrontierCache {
            initialized: true,
            revision: state.frontier_revision,
            len: items.len(),
            first,
            last,
            frontier_index,
            frontier_key,
        };
        frontier_key
    }
}

impl<T, K: KeyFn<T>, R> Steps<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
    }

    /// The first step that has not finished — the frontier. `None` when
    /// every step is terminal, which is how a caller knows the rail is done.
    /// This explicit query is O(n); [`Steps::draw`] uses the runtime cache.
    pub fn frontier(&self, items: &[T]) -> Option<ItemKey> {
        items
            .iter()
            .position(|it| !self.state_of(it).terminal())
            .map(|i| self.key_at(items, i))
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Steps<'_, T, K, R> {
    /// The index of `key`, probing `hint` before scanning.
    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(h) = hint
            && h < items.len()
            && self.key_at(items, h) == key
        {
            return Some(h);
        }
        (0..items.len()).find(|&i| self.key_at(items, i) == key)
    }

    fn move_cursor(&self, st: &mut StepsState, items: &[T], to: usize, acc: &mut Acc<StepsAction>) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let to = to.min(items.len().saturating_sub(1));
        let key = self.key_at(items, to);
        if st.core.cursor() == Some(key) {
            acc.consumed();
            return;
        }
        st.core.set_cursor(to, key);
        acc.action(StepsAction::Moved(key));
    }

    /// The update phase: reconcile, then drain keys, pointer and wheel.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut StepsState,
        items: &[T],
    ) -> Response<StepsAction> {
        if self.disabled {
            return Response::ignored();
        }
        let mut acc = Acc::<StepsAction>::new();
        let len = items.len();
        if let Reconciliation::CursorMoved(key) = st.core.reconcile(len, |i| self.key_at(items, i))
        {
            acc.action(StepsAction::Moved(key));
        }
        if self.navigable && st.core.cursor().is_none() && !items.is_empty() {
            st.core.set_cursor(0, self.key_at(items, 0));
        }
        let bar = self.scrollbar().update(cx, st.core.scroll_mut(), len);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    let cur = st.core.cursor_index();
                    match Binding::command(table, action) {
                        Some(StepsCmd::Up) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(StepsCmd::Down) => {
                            self.move_cursor(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(StepsCmd::PageUp) => {
                            self.move_cursor(st, items, cur.saturating_sub(viewport), &mut acc);
                        }
                        Some(StepsCmd::PageDown) => {
                            self.move_cursor(st, items, cur.saturating_add(viewport), &mut acc);
                        }
                        Some(StepsCmd::Home) => self.move_cursor(st, items, 0, &mut acc),
                        Some(StepsCmd::End) => self.move_cursor(st, items, usize::MAX, &mut acc),
                        Some(StepsCmd::Activate) => {
                            if len == 0 {
                                acc.consumed();
                            } else {
                                acc.action(StepsAction::Activated(self.key_at(items, cur)));
                            }
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
                            if st.core.cursor() == Some(k) {
                                acc.consumed();
                            } else {
                                st.core.set_cursor(i, k);
                                acc.changed();
                            }
                        }
                        Phase::Click => {
                            st.core.set_cursor(i, k);
                            if self.navigable {
                                acc.action(StepsAction::Activated(k));
                            } else {
                                acc.action(StepsAction::Moved(k));
                            }
                        }
                        Phase::DoubleClick => {
                            st.core.set_cursor(i, k);
                            acc.action(StepsAction::Activated(k));
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
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &StepsState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        if !ui.is_inert() {
            let f = match (self.navigable, self.disabled) {
                (false, _) => Focusability::ClickOnly,
                (true, true) => Focusability::Disabled,
                (true, false) => Focusability::Focusable,
            };
            ui.register_control(self.id, area, f);
        }
        let live = Overrides::flags(ui.state(self.id), self.derived());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let container = self.ov.style(
            ui,
            self.id,
            Family::STEPS,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let len = items.len();
        let content = self.scrollbar().draw(ui, area, st.core.scroll(), len);
        if len == 0 {
            return area;
        }
        let view = ScrollRegion::view(st.core.scroll(), content, len);
        let frontier = ui.cache::<FrontierCache>(self.id).sync(self, st, items);
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        for (offset, i) in view.visible_range().enumerate() {
            let Some(item) = items.get(i) else { break };
            let key = self.key_at(items, i);
            let step = self.state_of(item);
            let mut flags = step.flags();
            if frontier == Some(key) {
                flags |= StateFlags::ACTIVE;
            }
            let is_cursor = st.core.cursor() == Some(key);
            if is_cursor && self.navigable {
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
            if live.contains(StateFlags::DISABLED) {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(StateFlags::PRESSED | StateFlags::HOVERED);
            }
            let rect = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(offset.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: 1,
            };
            self.paint_row(ui, rect, flags, key, step, item);
        }
        area
    }

    fn paint_row(
        &self,
        ui: &mut Ui<'_>,
        rect: Rect,
        flags: StateFlags,
        key: ItemKey,
        step: StepState,
        item: &T,
    ) {
        let row_style = self.ov.style(
            ui,
            self.id,
            Family::STEPS,
            Variant::DEFAULT,
            Part::CONTAINER,
            flags,
        );
        ui.fill(rect, row_style.style);
        let gutter = cell_at(rect, rect.x);
        if let Some(f) = self.ov.slot_for(Part::GUTTER) {
            f(ui, gutter);
        } else {
            let g = self.ov.style(
                ui,
                self.id,
                Family::STEPS,
                Variant::DEFAULT,
                Part::GUTTER,
                flags,
            );
            match g.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter, glyph, g.style);
                }
                Slot::Inherit | Slot::Clear => ui.fill(gutter, g.style),
            }
        }
        let mark = cell_at(rect, rect.x.saturating_add(1));
        if let Some(f) = self.ov.slot_for(Part::ICON) {
            f(ui, mark);
        } else {
            let icon = self.ov.style(
                ui,
                self.id,
                Family::STEPS,
                Variant::DEFAULT,
                Part::ICON,
                flags,
            );
            let glyph = match icon.glyph {
                Slot::Set(g) => Some(g),
                Slot::Inherit => step.glyph(),
                Slot::Clear => None,
            };
            match glyph {
                Some(g) => {
                    ui.glyph(mark, g, icon.style);
                }
                None => ui.fill(mark, icon.style),
            }
        }
        let mut body = Rect {
            x: rect.x.saturating_add(3),
            width: rect.width.saturating_sub(3),
            ..rect
        };
        let meta_width = crate::text::width(step.label());
        if meta_width > 0 && meta_width.saturating_add(2) <= body.width {
            let meta = Rect {
                x: body.right().saturating_sub(meta_width),
                width: meta_width,
                ..body
            };
            let style = self.ov.style(
                ui,
                self.id,
                Family::STEPS,
                Variant::DEFAULT,
                Part::META,
                flags,
            );
            ui.paint_str(meta, step.label(), style.style);
            body.width = body.width.saturating_sub(meta_width.saturating_add(1));
        }
        if !body.is_empty() {
            let mut r = RowUi::new_with_patches(
                ui,
                self.id,
                Family::STEPS,
                Variant::DEFAULT,
                flags,
                key,
                body,
                self.ov.part_patch(Part::CONTAINER),
                self.ov.part_patch(Part::LABEL),
            );
            self.row.row(item, &mut r);
        }
        if !ui.is_inert() {
            ui.register_part(self.id, PartRef::item(Part::ROW, key), rect);
        }
    }

    /// The natural size: 32 columns, whatever height is offered.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }
}

impl<T, K, R> Bindings for Steps<'_, T, K, R> {
    type Cmd = StepsCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<StepsCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::{Buffer, Cell as BufferCell};
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::{FrontierCache, StepState, Steps, StepsAction, StepsState};
    use crate::action::ActionKey;
    use crate::collection::{Reconcile, Reconciliation, RowUi};
    use crate::components::Acc;
    use crate::event::{Chord, KeyCode, KeyModifiers};
    use crate::id::{Id, ItemKey, Part, PartRef};
    use crate::intent::{IntentQueue, Phase};
    use crate::response::Response;
    use crate::response::StateFlags;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{Role, Slot, StylePatch, Theme};
    use crate::ui::cx::{FrameServices, LastFrame};
    use crate::ui::{Cx, FrameState, Ui, UiCore};

    const RAIL: Id = Id::root("steps.tests");

    #[derive(Clone, Copy, Debug)]
    struct S(&'static str, StepState);

    impl core::fmt::Display for S {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(self.0)
        }
    }

    fn state(s: &S) -> StepState {
        s.1
    }

    fn keyed(s: &S) -> ItemKey {
        ItemKey::text(s.0)
    }

    fn update_steps(
        rail: &Steps<'_, S, fn(&S) -> ItemKey>,
        state: &mut StepsState,
        items: &[S],
        intents: &IntentQueue,
    ) -> Response<StepsAction> {
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let mut cx = Cx::new(intents, &mut services, &mut core, &last, &theme, None);
        rail.update(&mut cx, state, items)
    }

    /// Paints `rail` and returns the rows as text plus the frame's reachable
    /// focus ring.
    fn render<T: core::fmt::Display>(
        rail: &Steps<'_, T>,
        items: &[T],
        h: u16,
    ) -> (Vec<String>, Vec<Id>) {
        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: h,
        };
        let theme = Theme::junie();
        let mut fs = FrameState::default();
        fs.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
            rail.draw(&mut ui, area, &StepsState::new(), items);
        }
        let rows = (0..h)
            .map(|y| {
                (0..area.width)
                    .filter_map(|x| page.cell((x, y)).map(|c| c.symbol().to_owned()))
                    .collect::<String>()
            })
            .collect();
        (rows, fs.ring.reachable().map(|e| e.id).collect())
    }

    /// The frontier is the first step that has not finished, and it is a
    /// **derived** property, recomputed from the caller's data on every
    /// frame. The legacy rail had the same rule; what changes is that it now
    /// names the step by key instead of by index.
    #[test]
    fn the_frontier_is_the_first_unfinished_step() {
        let rail: Steps<'_, S> = Steps::new(RAIL).step(&state);
        let mut items = [
            S("a", StepState::Done),
            S("b", StepState::Skipped),
            S("c", StepState::Running),
            S("d", StepState::Queued),
        ];
        assert_eq!(rail.frontier(&items), Some(ItemKey::index(2)));

        // a failed step is terminal, so the frontier moves past it — the
        // legacy `frontier_and_counts` assertion verbatim
        items[2].1 = StepState::Failed;
        assert_eq!(rail.frontier(&items), Some(ItemKey::index(3)));

        // blocked is NOT terminal: the rail is still waiting on it
        items[3].1 = StepState::Blocked;
        assert_eq!(rail.frontier(&items), Some(ItemKey::index(3)));

        // every step terminal: no frontier at all
        items[3].1 = StepState::Done;
        assert_eq!(rail.frontier(&items), None);
        assert_eq!(rail.frontier(&[]), None);
    }

    /// Navigability is chosen by the constructor, and the difference is
    /// observable in the focus ring — not in a builder flag that `update`
    /// and `draw` could disagree about (§13, §23 K2 G4).
    #[test]
    fn a_display_rail_is_click_only_and_a_navigable_rail_is_in_the_ring() {
        let items = [S("a", StepState::Queued), S("b", StepState::Running)];

        let display: Steps<'_, S> = Steps::new(RAIL).step(&state);
        let (_, ring) = render(&display, &items, 2);
        assert!(
            ring.is_empty(),
            "a display rail must not put a stop in the ring: {ring:?}"
        );
        assert!(display.table().is_empty());

        let nav: Steps<'_, S> = Steps::navigable(RAIL).step(&state);
        let (_, ring) = render(&nav, &items, 2);
        assert_eq!(
            ring,
            vec![RAIL],
            "a navigable rail is exactly one reachable stop"
        );
        assert!(!nav.table().is_empty());

        // and a disabled navigable rail is registered but unreachable
        let off: Steps<'_, S> = Steps::navigable(RAIL).step(&state).disabled(true);
        let (_, ring) = render(&off, &items, 2);
        assert!(
            ring.is_empty(),
            "a disabled rail is not reachable: {ring:?}"
        );
        assert!(off.table().is_empty());
    }

    #[test]
    fn forced_reference_rendering_registers_no_nested_or_row_regions() {
        let items = [S("row", StepState::Queued); 10];
        let area = Rect::new(0, 0, 20, 5);
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
                    RAIL,
                    crate::ReferenceState::FOCUSED,
                )),
                |ui| {
                    Steps::navigable(RAIL).draw(ui, area, &StepsState::new(), &items);
                },
            );
        }
        assert_eq!(frame.registry.regions().len(), 0);
        assert_eq!(frame.ring.reachable().count(), 0);
    }

    /// Two lifecycle states must be distinguishable without colour, which is
    /// what §16.2 case 9 asserts over the whole rail. The rail's two
    /// symbol-carrying affordances are the `Part::ICON` glyph and the
    /// `Part::META` state word; `Queued` and `Skipped` share a blank glyph,
    /// so the word is the only thing separating them and it must be painted.
    #[test]
    fn every_lifecycle_state_paints_a_symbol_that_separates_it() {
        let items = [
            S("a", StepState::Queued),
            S("b", StepState::Running),
            S("c", StepState::Done),
            S("d", StepState::Skipped),
            S("e", StepState::Failed),
            S("f", StepState::Blocked),
        ];
        let rail: Steps<'_, S> = Steps::new(RAIL).step(&state);
        let (rows, _) = render(&rail, &items, 6);
        assert_eq!(rows.len(), 6);
        for (i, (step, row)) in items.iter().zip(rows.iter()).enumerate() {
            let want = step.1.label();
            assert!(
                row.contains(want),
                "row {i} does not carry its state word {want:?}: {row:?}"
            );
        }
        // pairwise: no two rows are the same run of symbols
        for (i, a) in rows.iter().enumerate() {
            for (j, b) in rows.iter().enumerate() {
                assert!(
                    i == j || a != b,
                    "rows {i} and {j} are indistinguishable without colour: {a:?}"
                );
            }
        }
    }

    #[test]
    fn skipped_is_read_only_terminal_and_still_navigable() {
        assert_eq!(StepState::Skipped.flags(), StateFlags::READ_ONLY);
        assert!(StepState::Skipped.terminal());

        let items = [S("a", StepState::Done), S("b", StepState::Skipped)];
        let rail: Steps<'_, S> = Steps::navigable(RAIL).step(&state);
        let mut state = StepsState::new();
        state.set_cursor(0, ItemKey::index(0));
        let mut acc = Acc::new();
        rail.move_cursor(&mut state, &items, 1, &mut acc);
        assert_eq!(state.cursor(), Some(ItemKey::index(1)));
        assert_eq!(
            acc.finish(RAIL).action_ref(),
            Some(&StepsAction::Moved(ItemKey::index(1)))
        );
    }

    #[test]
    fn skipped_row_enter_and_click_remain_activatable_by_stable_key() {
        let items = [S("first", StepState::Done), S("skip", StepState::Skipped)];
        let rail = Steps::navigable(RAIL)
            .key(keyed as fn(&S) -> ItemKey)
            .step(&state);
        let skip = ItemKey::text("skip");
        let mut steps_state = StepsState::new();
        steps_state.set_cursor(1, skip);
        assert_eq!(StepState::Skipped.flags(), StateFlags::READ_ONLY);
        assert!(!StepState::Skipped.flags().contains(StateFlags::DISABLED));

        let mut enter = IntentQueue::new();
        enter.binding(
            RAIL,
            ActionKey::custom("steps.activate"),
            Chord::key(KeyCode::Enter),
        );
        let response = update_steps(&rail, &mut steps_state, &items, &enter);
        assert_eq!(response.action_ref(), Some(&StepsAction::Activated(skip)));

        let mut click = IntentQueue::new();
        click.pointer(
            RAIL,
            Phase::Click,
            PartRef::item(Part::ROW, skip),
            Position::new(0, 1),
            Position::new(0, 1),
            KeyModifiers::NONE,
        );
        let response = update_steps(&rail, &mut steps_state, &items, &click);
        assert_eq!(response.action_ref(), Some(&StepsAction::Activated(skip)));
        assert_eq!(steps_state.cursor(), Some(skip));
    }

    #[test]
    fn fresh_navigable_update_seeds_first_key_silently() {
        let items = [
            S("stable-first", StepState::Queued),
            S("second", StepState::Queued),
        ];
        let rail = Steps::navigable(RAIL).key(keyed as fn(&S) -> ItemKey);
        let mut state = StepsState::new();
        let intents = IntentQueue::new();

        let response = update_steps(&rail, &mut state, &items, &intents);
        assert_eq!(state.cursor(), Some(ItemKey::text("stable-first")));
        assert_eq!(response.action_ref(), None);
        assert!(!response.is_consumed());
        assert!(!response.is_changed());
    }

    #[test]
    fn display_click_moves_and_double_click_activates_exact_key() {
        let items = [
            S("first", StepState::Queued),
            S("stable-second", StepState::Skipped),
        ];
        let rail = Steps::new(RAIL).key(keyed as fn(&S) -> ItemKey);
        let key = ItemKey::text("stable-second");
        let mut state = StepsState::new();

        let pointer = |phase| {
            let mut intents = IntentQueue::new();
            intents.pointer(
                RAIL,
                phase,
                PartRef::item(Part::ROW, key),
                Position::new(0, 1),
                Position::new(0, 1),
                KeyModifiers::NONE,
            );
            intents
        };
        let click = pointer(Phase::Click);
        let response = update_steps(&rail, &mut state, &items, &click);
        assert_eq!(response.action_ref(), Some(&StepsAction::Moved(key)));
        assert_eq!(state.cursor(), Some(key));

        let double_click = pointer(Phase::DoubleClick);
        let response = update_steps(&rail, &mut state, &items, &double_click);
        assert_eq!(response.action_ref(), Some(&StepsAction::Activated(key)));
        assert_eq!(state.cursor(), Some(key));
    }

    #[test]
    fn boundary_movement_is_consumed_without_action_or_repaint() {
        let items = [S("a", StepState::Queued), S("b", StepState::Queued)];
        let rail: Steps<'_, S> = Steps::navigable(RAIL);
        let mut state = StepsState::new();
        state.set_cursor(0, ItemKey::index(0));
        let mut acc = Acc::<StepsAction>::new();
        rail.move_cursor(&mut state, &items, 0, &mut acc);
        let response = acc.finish(RAIL);
        assert!(response.is_consumed());
        assert!(!response.is_changed());
        assert_eq!(response.action_ref(), None);
        assert_eq!(state.cursor(), Some(ItemKey::index(0)));
    }

    #[test]
    fn stable_cursor_identity_survives_same_endpoints_reorder() {
        let original = [
            S("a", StepState::Queued),
            S("b", StepState::Queued),
            S("c", StepState::Queued),
            S("d", StepState::Queued),
        ];
        let reordered = [original[0], original[2], original[1], original[3]];
        let rail = Steps::navigable(RAIL).key(|step: &S| ItemKey::text(step.0));
        let mut state = StepsState::new();
        state.set_cursor(1, ItemKey::text("b"));
        assert_eq!(
            state.reconcile(original.len(), |i| rail.key_at(&original, i)),
            Reconciliation::Unchanged
        );

        state.invalidate();
        assert_eq!(
            state.reconcile(reordered.len(), |i| rail.key_at(&reordered, i)),
            Reconciliation::Unchanged,
            "a surviving cursor key is not a semantic move"
        );
        assert_eq!(state.cursor(), Some(ItemKey::text("b")));

        let mut acc = Acc::new();
        rail.move_cursor(&mut state, &reordered, 1, &mut acc);
        assert_eq!(
            acc.finish(RAIL).action_ref(),
            Some(&StepsAction::Moved(ItemKey::text("c"))),
            "movement starts from b's new physical index"
        );
    }

    #[test]
    fn steps_parts_and_slot_parts_are_exact() {
        assert_eq!(
            Steps::<S>::PARTS,
            &[
                Part::CONTAINER,
                Part::LABEL,
                Part::GUTTER,
                Part::ICON,
                Part::META,
                Part::TRACK,
                Part::THUMB,
            ]
        );
        let slot_parts = [Part::GUTTER, Part::ICON, Part::TRACK, Part::THUMB];
        assert!(
            slot_parts
                .iter()
                .all(|part| Steps::<S>::PARTS.contains(part))
        );
    }

    #[test]
    fn only_exact_slot_parts_invoke_real_replacement_painters() {
        let items = [S("row", StepState::Queued); 10];
        let area = Rect::new(0, 0, 20, 5);
        for part in [
            Part::CONTAINER,
            Part::LABEL,
            Part::GUTTER,
            Part::ICON,
            Part::META,
            Part::TRACK,
            Part::THUMB,
        ] {
            let calls = Cell::new(0usize);
            let replacement = |ui: &mut Ui<'_>, rect: Rect| {
                calls.set(calls.get().saturating_add(1));
                ui.paint_str(rect, "#", ui.surface_style());
            };
            let rail = Steps::new(RAIL).slot(part, &replacement);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime.draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &StepsState::new(), &items);
            });
            assert_eq!(
                calls.get() > 0,
                matches!(part, Part::GUTTER | Part::ICON | Part::TRACK | Part::THUMB),
                "unexpected slot behavior for {part:?}"
            );
        }
    }

    #[test]
    fn icon_slot_replacement_and_glyph_clear_preserve_row_geometry() {
        let items = [S("label", StepState::Running)];
        let steps_state = StepsState::new();
        let area = Rect::new(0, 0, 24, 1);
        let render = |rail: &Steps<'_, S>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime.draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &steps_state, &items);
            });
            let row = runtime.area_of_part(RAIL, PartRef::item(Part::ROW, ItemKey::index(0)));
            (buffer, row)
        };

        let replacement = |ui: &mut Ui<'_>, rect: Rect| {
            ui.paint_str(rect, "#", ui.surface_style());
        };
        let replaced = Steps::new(RAIL).step(&state).slot(Part::ICON, &replacement);
        let (replaced_buffer, replaced_row) = render(&replaced);
        assert_eq!(
            replaced_buffer
                .cell(Position::new(1, 0))
                .map(BufferCell::symbol),
            Some("#")
        );
        assert_eq!(
            replaced_buffer
                .cell(Position::new(3, 0))
                .map(BufferCell::symbol),
            Some("l")
        );

        let clear = [(
            Part::ICON,
            StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        )];
        let cleared = Steps::new(RAIL).step(&state).patch_part(&clear);
        let (cleared_buffer, cleared_row) = render(&cleared);
        assert_eq!(
            cleared_buffer
                .cell(Position::new(1, 0))
                .map(BufferCell::symbol),
            Some(" ")
        );
        assert_eq!(
            cleared_buffer
                .cell(Position::new(3, 0))
                .map(BufferCell::symbol),
            Some("l")
        );
        assert_eq!(replaced_row, Some(area));
        assert_eq!(cleared_row, replaced_row);
    }

    #[test]
    fn owner_patch_reaches_owned_parts_but_not_row_meta_cell_or_custom_part() {
        let bold = StylePatch::new().add(Modifier::BOLD);
        let owner_parts = [
            (Part::CONTAINER, StylePatch::new().set_bg(Role::Danger)),
            (Part::LABEL, bold),
            (Part::GUTTER, bold),
            (Part::ICON, bold),
            (Part::META, bold),
            (Part::TRACK, bold),
            (Part::THUMB, bold),
        ];
        let custom = Part::custom("steps.test.custom");
        let row = move |item: &S, row: &mut RowUi<'_>| {
            row.meta("rm");
            row.part(custom, 2).text("cu");
            row.part(Part::CELL, 2).text("ce");
            row.label(item.0);
        };
        let rail = Steps::new(RAIL)
            .step(&state)
            .row(row)
            .patch_part(&owner_parts);
        let items = [S("label", StepState::Running)];
        let area = Rect::new(0, 0, 40, 1);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            rail.draw(ui, rect, &StepsState::new(), &items);
        });
        let mut plain_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut plain = Buffer::empty(area);
        plain_runtime.draw_scene(area, &mut plain, |ui, rect| {
            Steps::new(RAIL)
                .step(&state)
                .draw(ui, rect, &StepsState::new(), &items);
        });

        assert_ne!(
            buffer.cell(Position::new(2, 0)).map(|cell| cell.bg),
            plain.cell(Position::new(2, 0)).map(|cell| cell.bg),
            "CONTAINER patch must reach automatic row fill"
        );
        for x in [0, 1, 3, 33] {
            assert!(
                buffer
                    .cell(Position::new(x, 0))
                    .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
                "component-owned cell {x} did not receive owner patch"
            );
        }
        for (text, x) in [("rm", 30), ("cu", 27), ("ce", 24)] {
            assert!(
                buffer
                    .cell(Position::new(x, 0))
                    .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD)),
                "owner patch leaked into row-owned {text}"
            );
        }
    }

    #[test]
    fn every_override_channel_is_forwarded_to_scrollbar() {
        let global = StylePatch::new().add(Modifier::UNDERLINED);
        let track = [(Part::TRACK, StylePatch::new().add(Modifier::BOLD))];
        let thumb_slot = |ui: &mut Ui<'_>, rect: Rect| {
            ui.paint_str(rect, "#", ui.surface_style());
        };
        let items = [S("row", StepState::Queued); 10];
        let rail = Steps::new(RAIL)
            .patch(&global)
            .patch_part(&track)
            .slot(Part::THUMB, &thumb_slot);
        let area = Rect::new(0, 0, 20, 5);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            rail.draw(ui, rect, &StepsState::new(), &items);
        });

        let track_cell = buffer.cell(Position::new(19, 3)).expect("track cell");
        assert!(track_cell.modifier.contains(Modifier::UNDERLINED));
        assert!(track_cell.modifier.contains(Modifier::BOLD));
        assert_eq!(
            buffer.cell(Position::new(19, 1)).map(BufferCell::symbol),
            Some("#"),
            "thumb slot is painted by embedded ScrollRegion"
        );
    }

    #[derive(Debug)]
    struct CountedStep {
        key: ItemKey,
        state: Cell<StepState>,
    }

    fn counted_key(step: &CountedStep) -> ItemKey {
        step.key
    }

    #[test]
    fn steps_100k_rows_render_uses_incremental_frontier_accesses() {
        let accesses = Cell::new(0usize);
        let state_of = |step: &CountedStep| {
            accesses.set(accesses.get().saturating_add(1));
            step.state.get()
        };
        let items: Vec<_> = (0..100_000)
            .map(|i| CountedStep {
                key: ItemKey::num(i as u64),
                state: Cell::new(if i == 0 {
                    StepState::Running
                } else {
                    StepState::Queued
                }),
            })
            .collect();
        let rail = Steps::new(RAIL)
            .key(counted_key)
            .row(|_: &CountedStep, _: &mut RowUi<'_>| {})
            .step(&state_of);
        let state = StepsState::new();
        let area = Rect::new(0, 0, 30, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);

        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            rail.draw(ui, rect, &state, &items);
        });
        assert_eq!(
            accesses.get(),
            5,
            "one cold frontier probe plus four visible-row state reads"
        );
        accesses.set(0);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            rail.draw(ui, rect, &state, &items);
        });
        assert_eq!(
            accesses.get(),
            5,
            "steady draw cost is frontier probe plus viewport, not collection length"
        );

        items[0].state.set(StepState::Done);
        accesses.set(0);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            rail.draw(ui, rect, &state, &items);
        });
        assert_eq!(
            accesses.get(),
            6,
            "advance reads the old frontier, one crossed step, and the viewport"
        );
    }

    #[test]
    fn completed_frontier_is_cached_until_invalidation() {
        let accesses = Cell::new(0usize);
        let state_of = |step: &CountedStep| {
            accesses.set(accesses.get().saturating_add(1));
            step.state.get()
        };
        let items: Vec<_> = (0..100_000)
            .map(|i| CountedStep {
                key: ItemKey::num(i as u64),
                state: Cell::new(StepState::Done),
            })
            .collect();
        let rail = Steps::new(RAIL)
            .key(counted_key)
            .row(|_: &CountedStep, _: &mut RowUi<'_>| {})
            .step(&state_of);
        let mut state = StepsState::new();
        let mut cache = FrontierCache::default();

        assert_eq!(cache.sync(&rail, &state, &items), None);
        assert_eq!(accesses.get(), items.len());
        accesses.set(0);
        assert_eq!(cache.sync(&rail, &state, &items), None);
        assert_eq!(accesses.get(), 0, "completion is a reusable cache result");

        items[0].state.set(StepState::Queued);
        state.invalidate();
        assert_eq!(cache.sync(&rail, &state, &items), Some(ItemKey::num(0)));
        assert_eq!(accesses.get(), 1, "reset invalidation scans from zero");
    }

    #[test]
    fn runtime_cache_clear_drops_only_frontier_acceleration() {
        let accesses = Cell::new(0usize);
        let state_of = |step: &CountedStep| {
            accesses.set(accesses.get().saturating_add(1));
            step.state.get()
        };
        let items: Vec<_> = (0..8)
            .map(|i| CountedStep {
                key: ItemKey::num(i),
                state: Cell::new(StepState::Done),
            })
            .collect();
        let rail = Steps::new(RAIL)
            .key(counted_key)
            .row(|_: &CountedStep, _: &mut RowUi<'_>| {})
            .step(&state_of);
        let mut state = StepsState::new();
        state.set_cursor(4, ItemKey::num(4));
        let area = Rect::new(0, 0, 30, 2);
        let mut core = UiCore::default();
        let draw = |core: &mut UiCore| {
            let theme = Theme::junie();
            let mut frame = FrameState::default();
            frame.reset(1, area);
            let mut page = Buffer::empty(area);
            let last = LastFrame::default();
            let mut ui = Ui::new(&mut frame, &mut page, core, &theme, &last);
            rail.draw(&mut ui, area, &state, &items);
        };

        draw(&mut core);
        assert_eq!(accesses.get(), items.len() + usize::from(area.height));
        accesses.set(0);
        draw(&mut core);
        assert_eq!(accesses.get(), usize::from(area.height));
        core.clear_caches();
        accesses.set(0);
        draw(&mut core);
        assert_eq!(accesses.get(), items.len() + usize::from(area.height));
        assert_eq!(state.cursor(), Some(ItemKey::num(4)));
    }

    #[test]
    fn saturated_revision_recomputes_every_time() {
        let accesses = Cell::new(0usize);
        let state_of = |step: &CountedStep| {
            accesses.set(accesses.get().saturating_add(1));
            step.state.get()
        };
        let items = [CountedStep {
            key: ItemKey::num(7),
            state: Cell::new(StepState::Done),
        }];
        let rail = Steps::new(RAIL).key(counted_key).step(&state_of);
        let mut state = StepsState::new();
        state.frontier_revision = u64::MAX;
        let mut cache = FrontierCache::default();

        assert_eq!(cache.sync(&rail, &state, &items), None);
        items[0].state.set(StepState::Queued);
        state.invalidate();
        assert_eq!(cache.sync(&rail, &state, &items), Some(ItemKey::num(7)));
        items[0].state.set(StepState::Done);
        state.invalidate();
        assert_eq!(cache.sync(&rail, &state, &items), None);
        assert_eq!(
            accesses.get(),
            3,
            "every saturated invalidation scans from zero and observes regressions"
        );
    }
}
