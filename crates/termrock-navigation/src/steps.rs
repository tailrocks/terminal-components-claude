use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::scroll::ScrollState;
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::response::{Response, StateFlags};
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum StepState {
    #[default]
    Queued,
    Running,
    Done,
    Skipped,
    Failed,
    Blocked,
}

impl StepState {
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

    #[must_use]
    pub const fn terminal(self) -> bool {
        matches!(
            self,
            StepState::Done | StepState::Skipped | StepState::Failed
        )
    }

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepsAction {
    Moved(ItemKey),
    Activated(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepsCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
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

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct StepsState {
    core: CollectionCore,
    frontier_revision: u64,
}

type MetadataFn<'a, T> = dyn for<'item> Fn(&'item T) -> Option<&'item str> + 'a;
type LabelWidthFn<'a, T> = dyn Fn(&T) -> u16 + 'a;

impl StepsState {
    #[must_use]
    pub const fn new() -> Self {
        StepsState {
            core: CollectionCore::new(),
            frontier_revision: 0,
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

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

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

pub struct Steps<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    step: Option<&'a dyn Fn(&T) -> StepState>,
    metadata: Option<&'a MetadataFn<'a, T>>,
    label_width: Option<&'a LabelWidthFn<'a, T>>,
    numbered: bool,
    navigable: bool,
    disabled: bool,
    ov: PartStyle<'a>,
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
    #[must_use]
    pub const fn new(id: Id) -> Self {
        Steps {
            id,
            key: ByIndex,
            row: DefaultRow,
            step: None,
            metadata: None,
            label_width: None,
            numbered: false,
            navigable: false,
            disabled: false,
            ov: PartStyle::new(),
            fwd_patch: None,
            fwd_parts: &[],
            fwd_slot: None,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn navigable(id: Id) -> Self {
        let mut s = Steps::new(id);
        s.navigable = true;
        s
    }
}

impl<'a, T, K, R> Steps<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::LABEL,
        Part::GUTTER,
        Part::ICON,
        Part::META,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const PREFERRED_WIDTH: u16 = 32;

    #[must_use]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn is_navigable(&self) -> bool {
        self.navigable
    }

    #[must_use]
    pub const fn step(mut self, f: &'a dyn Fn(&T) -> StepState) -> Self {
        self.step = Some(f);
        self
    }

    /// Show a one-based ordinal before each label.
    ///
    /// The default leaves the existing row layout unchanged. Numbered rows
    /// reserve a three-cell number lane, including its following gap.
    #[must_use]
    pub const fn numbered(mut self, yes: bool) -> Self {
        self.numbered = yes;
        self
    }

    /// Read optional trailing metadata from borrowed item data.
    ///
    /// Without this accessor, the current state label remains the trailing
    /// metadata. When configured, `None` hides metadata for that item.
    pub fn metadata<F>(mut self, f: &'a F) -> Self
    where
        F: for<'item> Fn(&'item T) -> Option<&'item str> + 'a,
    {
        self.metadata = Some(f);
        self
    }

    /// Supply the terminal-cell width of the label produced by a custom row.
    ///
    /// Measurement cannot invoke the row renderer because it paints into the
    /// terminal. Custom rows that need intrinsic sizing beyond the 12-cell
    /// label minimum can provide a pure data projection here. The projection
    /// must describe the label actually rendered by the row callback.
    pub fn measure_label_width<F>(mut self, f: &'a F) -> Self
    where
        F: Fn(&T) -> u16 + 'a,
    {
        self.label_width = Some(f);
        self
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Steps<'a, T, K2, R> {
        Steps {
            id: self.id,
            key: k,
            row: self.row,
            step: self.step,
            metadata: self.metadata,
            label_width: self.label_width,
            numbered: self.numbered,
            navigable: self.navigable,
            disabled: self.disabled,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Steps<'a, T, K, R2> {
        Steps {
            id: self.id,
            key: self.key,
            row: r,
            step: self.step,
            metadata: self.metadata,
            label_width: self.label_width,
            numbered: self.numbered,
            navigable: self.navigable,
            disabled: self.disabled,
            ov: self.ov,
            fwd_patch: self.fwd_patch,
            fwd_parts: self.fwd_parts,
            fwd_slot: self.fwd_slot,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self.fwd_patch = Some(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self.fwd_parts = ps;
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self.fwd_slot = Some((p, f));
        self
    }

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

    pub fn frontier(&self, items: &[T]) -> Option<ItemKey> {
        items
            .iter()
            .position(|it| !self.state_of(it).terminal())
            .map(|i| self.key_at(items, i))
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Steps<'_, T, K, R> {
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
        let live = PartStyle::flags(ui.state(self.id), self.derived());
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
            self.paint_row(ui, rect, flags, key, step, i, item);
        }
        area
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one row uses its resolved state, stable identity, ordinal, and borrowed item"
    )]
    fn paint_row(
        &self,
        ui: &mut Ui<'_>,
        rect: Rect,
        flags: StateFlags,
        key: ItemKey,
        step: StepState,
        ordinal: usize,
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
        if self.numbered {
            let number = format!("{:02}", ordinal.saturating_add(1));
            let number_width = number.len().max(2);
            let number_rect = Rect {
                x: body.x,
                width: number_width.min(usize::from(body.width)) as u16,
                ..body
            };
            let number_style = self.ov.style(
                ui,
                self.id,
                Family::STEPS,
                Variant::DEFAULT,
                Part::META,
                flags,
            );
            ui.paint_str(number_rect, &number, number_style.style);
            let advance = number_width.saturating_add(1).min(usize::from(u16::MAX)) as u16;
            body.x = body.x.saturating_add(advance);
            body.width = body.width.saturating_sub(advance);
        }
        let metadata = self
            .metadata
            .and_then(|metadata| metadata(item))
            .or_else(|| self.metadata.is_none().then(|| step.label()));
        let meta_width = metadata.map_or(0, crate::text::width);
        let configured_layout = self.numbered || self.metadata.is_some();
        let metadata_fits = metadata.is_some()
            && meta_width > 0
            && if configured_layout {
                meta_width.saturating_add(15) <= body.width
            } else {
                meta_width.saturating_add(2) <= body.width
            };
        if metadata_fits {
            let meta = Rect {
                x: body
                    .right()
                    .saturating_sub(meta_width)
                    .saturating_sub(u16::from(configured_layout)),
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
            ui.paint_str(meta, metadata.unwrap_or_default(), style.style);
            body.width = body
                .width
                .saturating_sub(meta_width.saturating_add(if configured_layout { 3 } else { 1 }));
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

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12 + if self.numbered { 6 } else { 0 }, 1),
            preferred: (Self::PREFERRED_WIDTH, c.max.1),
        }
        .fit(c)
    }

    /// Measure this rail from its borrowed items without updating component state.
    ///
    /// Labels use the optional pure width projection first, then the row's
    /// intrinsic label width when available, and otherwise a 12-cell minimum.
    /// Measurement does not call the row painter or application state
    /// projection.
    pub fn measure_with_items(&self, _ui: &Ui<'_>, c: Constraints, items: &[T]) -> Size {
        let number_lane = if self.numbered {
            items
                .len()
                .max(1)
                .to_string()
                .len()
                .max(2)
                .saturating_add(1)
        } else {
            0
        };
        let prefix = 3usize.saturating_add(number_lane);
        let mut preferred = usize::from(Self::PREFERRED_WIDTH);
        for (ordinal, item) in items.iter().enumerate() {
            let label_width = self.label_width.map_or_else(
                || self.row.intrinsic_label_width(item).unwrap_or(12),
                |measure| measure(item),
            );
            let metadata = self.metadata.and_then(|metadata| metadata(item));
            let metadata_width = if self.metadata.is_some() {
                metadata.map_or(0, crate::text::width)
            } else if self.numbered || self.step.is_some() {
                // Keep the fallback state label visible without invoking the
                // application-supplied state projection during measurement.
                crate::text::width("skipped").max(crate::text::width("blocked"))
            } else {
                crate::text::width(StepState::Queued.label())
            };
            let row_prefix = if self.numbered {
                ordinal
                    .saturating_add(1)
                    .to_string()
                    .len()
                    .max(2)
                    .saturating_add(1)
            } else {
                0
            };
            let item_prefix = 3usize.saturating_add(row_prefix);
            let label_and_meta = if metadata_width == 0 {
                item_prefix.saturating_add(usize::from(label_width))
            } else if self.numbered || self.metadata.is_some() {
                item_prefix
                    .saturating_add(usize::from(label_width.max(12)))
                    .saturating_add(usize::from(metadata_width))
                    .saturating_add(3)
            } else {
                item_prefix
                    .saturating_add(usize::from(label_width))
                    .saturating_add(usize::from(metadata_width))
                    .saturating_add(1)
            };
            preferred = preferred.max(label_and_meta);
        }

        let minimum = if self.numbered || self.metadata.is_some() {
            prefix.saturating_add(12)
        } else {
            12
        };
        Size {
            min: (minimum.min(usize::from(u16::MAX)) as u16, 1),
            preferred: (preferred.min(usize::from(u16::MAX)) as u16, c.max.1),
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
    use crate::Acc;
    use termrock_core::id::{Id, ItemKey, Part, PartRef};
    use termrock_core::response::Response;
    use termrock_core::response::StateFlags;
    use termrock_runtime::Runtime;
    use termrock_runtime::stub::Stub;
    use termrock_theme::{Role, Slot, StylePatch, Theme};

    use termrock_collections::collection::{Reconcile, Reconciliation};
    use termrock_runtime::rowui::RowUi;
    use termrock_runtime::ui::{Cx, Ui};

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

    struct StepsApp {
        rail: Steps<'static, S, fn(&S) -> ItemKey>,
        state: StepsState,
        items: Vec<S>,
        last: Option<StepsAction>,
        consumed: bool,
        changed: bool,
    }

    const STEPS_AREA: Rect = Rect::new(0, 0, 30, 8);

    impl crate::runtime::App for StepsApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = self.rail.update(cx, &mut self.state, &self.items);
            self.last = response.action_ref().copied();
            self.consumed = response.is_consumed();
            self.changed = response.is_changed();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.rail.draw(ui, STEPS_AREA, &self.state, &self.items);
        }
    }

    fn steps_runtime(
        rail: Steps<'static, S, fn(&S) -> ItemKey>,
        state: StepsState,
        items: Vec<S>,
    ) -> Runtime<StepsApp> {
        let mut rt = Runtime::new(
            StepsApp {
                rail,
                state,
                items,
                last: None,
                consumed: false,
                changed: false,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(STEPS_AREA);
        for _ in 0..16 {
            rt.draw_buffer(STEPS_AREA, &mut buf).commit_presented();
            if !rt.needs_settle() {
                return rt;
            }
            let _ = rt.settle();
        }
        panic!("steps fixture focus did not settle");
    }

    fn click_row(rt: &mut Runtime<StepsApp>, key: ItemKey) {
        use crate::event::MouseKind;
        use termrock_runtime::stub::{deliver, mouse};
        let area = rt
            .area_of_part(RAIL, PartRef::item(Part::ROW, key))
            .expect("the clicked row was laid out");
        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        let _ = deliver(rt, mouse(MouseKind::Down, x, y));
        let _ = deliver(rt, mouse(MouseKind::Up, x, y));
    }

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
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut page = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut page, |ui, area| {
                rail.draw(ui, area, &StepsState::new(), items);
            })
            .commit_presented();
        let rows = (0..h)
            .map(|y| {
                (0..area.width)
                    .filter_map(|x| page.cell((x, y)).map(|c| c.symbol().to_owned()))
                    .collect::<String>()
            })
            .collect();
        (rows, runtime.ring().reachable().map(|e| e.id).collect())
    }

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

        items[2].1 = StepState::Failed;
        assert_eq!(rail.frontier(&items), Some(ItemKey::index(3)));

        items[3].1 = StepState::Blocked;
        assert_eq!(rail.frontier(&items), Some(ItemKey::index(3)));

        items[3].1 = StepState::Done;
        assert_eq!(rail.frontier(&items), None);
        assert_eq!(rail.frontier(&[]), None);
    }

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
        let area = Rect::new(0, 0, 30, 6);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut page = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut page, |ui, area| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        RAIL,
                        crate::ReferenceState::FOCUSED,
                    )),
                    |ui| {
                        Steps::navigable(RAIL).draw(ui, area, &StepsState::new(), &items);
                    },
                );
            })
            .commit_presented();

        assert_eq!(runtime.ring().reachable().count(), 0);
    }

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

        let mut rt = steps_runtime(rail, steps_state, items.to_vec());

        use termrock_runtime::stub::{deliver, key};
        let _ = deliver(&mut rt, key(crate::event::KeyCode::Enter));
        assert_eq!(rt.app().last, Some(StepsAction::Activated(skip)));

        click_row(&mut rt, skip);
        assert_eq!(rt.app().last, Some(StepsAction::Activated(skip)));
        assert_eq!(rt.app().state.cursor(), Some(skip));
    }

    #[test]
    fn fresh_navigable_update_seeds_first_key_silently() {
        let items = [
            S("stable-first", StepState::Queued),
            S("second", StepState::Queued),
        ];
        let rail = Steps::navigable(RAIL).key(keyed as fn(&S) -> ItemKey);

        let mut rt = steps_runtime(rail, StepsState::new(), items.to_vec());
        assert_eq!(rt.app().state.cursor(), Some(ItemKey::text("stable-first")));

        {
            let app = rt.app_mut();
            app.last = None;
            app.consumed = false;
            app.changed = false;
        }
        use crate::event::Input;
        use termrock_runtime::stub::deliver;
        let _ = deliver(&mut rt, Input::Tick);
        assert_eq!(rt.app().state.cursor(), Some(ItemKey::text("stable-first")));
        assert_eq!(rt.app().last, None);
        assert!(!rt.app().consumed);
        assert!(!rt.app().changed);
    }

    #[test]
    fn display_click_moves_and_double_click_activates_exact_key() {
        let items = [
            S("first", StepState::Queued),
            S("stable-second", StepState::Skipped),
        ];
        let rail = Steps::new(RAIL).key(keyed as fn(&S) -> ItemKey);
        let key = ItemKey::text("stable-second");

        let mut rt = steps_runtime(rail, StepsState::new(), items.to_vec());
        click_row(&mut rt, key);
        assert_eq!(rt.app().last, Some(StepsAction::Moved(key)));
        assert_eq!(rt.app().state.cursor(), Some(key));

        click_row(&mut rt, key);
        assert_eq!(rt.app().last, Some(StepsAction::Activated(key)));
        assert_eq!(rt.app().state.cursor(), Some(key));
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
            runtime
                .draw_scene(area, &mut buffer, |ui, rect| {
                    rail.draw(ui, rect, &StepsState::new(), &items);
                })
                .commit_presented();
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
            runtime
                .draw_scene(area, &mut buffer, |ui, rect| {
                    rail.draw(ui, rect, &steps_state, &items);
                })
                .commit_presented();
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
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &StepsState::new(), &items);
            })
            .commit_presented();
        let mut plain_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut plain = Buffer::empty(area);
        plain_runtime
            .draw_scene(area, &mut plain, |ui, rect| {
                Steps::new(RAIL)
                    .step(&state)
                    .draw(ui, rect, &StepsState::new(), &items);
            })
            .commit_presented();

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
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &StepsState::new(), &items);
            })
            .commit_presented();

        let track_cell = buffer.cell(Position::new(19, 3)).expect("track cell");
        assert!(track_cell.modifier.contains(Modifier::UNDERLINED));
        assert!(track_cell.modifier.contains(Modifier::BOLD));
        let thumb = runtime
            .area_of_part(RAIL, PartRef::of(Part::THUMB))
            .expect("thumb part");
        assert_eq!(
            buffer
                .cell(Position::new(thumb.x, thumb.y))
                .map(BufferCell::symbol),
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

        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &state, &items);
            })
            .commit_presented();
        assert_eq!(
            accesses.get(),
            5,
            "one cold frontier probe plus four visible-row state reads"
        );
        accesses.set(0);
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &state, &items);
            })
            .commit_presented();
        assert_eq!(
            accesses.get(),
            5,
            "steady draw cost is frontier probe plus viewport, not collection length"
        );

        items[0].state.set(StepState::Done);
        accesses.set(0);
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                rail.draw(ui, rect, &state, &items);
            })
            .commit_presented();
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

        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut page = Buffer::empty(area);
        let draw = |rt: &mut Runtime<Stub>, page: &mut Buffer| {
            rt.draw_scene(area, page, |ui, area| {
                rail.draw(ui, area, &state, &items);
            })
            .commit_presented();
        };

        draw(&mut rt, &mut page);
        assert_eq!(accesses.get(), items.len() + usize::from(area.height));
        accesses.set(0);
        draw(&mut rt, &mut page);
        assert_eq!(accesses.get(), usize::from(area.height));
        rt.set_theme(Theme::junie());
        accesses.set(0);
        draw(&mut rt, &mut page);
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
