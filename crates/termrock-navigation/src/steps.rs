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

    #[must_use]
    pub const fn numbered(mut self, yes: bool) -> Self {
        self.numbered = yes;
        self
    }

    pub fn metadata<F>(mut self, f: &'a F) -> Self
    where
        F: for<'item> Fn(&'item T) -> Option<&'item str> + 'a,
    {
        self.metadata = Some(f);
        self
    }

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
