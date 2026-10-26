use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket, shift};
use crate::action::ActionKey;
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
    ByIndex, CollectionCore, DefaultRow, KeyFn, KeySet, Reconcile, Reconciliation, SelectMode,
};
use termrock_runtime::rowui::{RowFn, RowUi};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChipBarAction {
    Toggled(ItemKey),
    Closed(ItemKey),
    Activated(ItemKey),
    AddRequested,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChipBarCmd {
    Prev,
    Next,
    First,
    Last,
    Activate,
    Toggle,
    Close,
}

const fn b(
    chord: Chord,
    cmd: ChipBarCmd,
    label: &'static str,
    visible: bool,
) -> Binding<ChipBarCmd> {
    Binding {
        action: ActionKey::custom(label),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const MOVE: [Binding<ChipBarCmd>; 7] = [
    b(Chord::key(KeyCode::Left), ChipBarCmd::Prev, "Left", true),
    b(Chord::key(KeyCode::Right), ChipBarCmd::Next, "Right", true),
    b(
        Chord::key(KeyCode::Char('h')),
        ChipBarCmd::Prev,
        "Left (H)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('l')),
        ChipBarCmd::Next,
        "Right (L)",
        false,
    ),
    b(Chord::key(KeyCode::Home), ChipBarCmd::First, "First", false),
    b(Chord::key(KeyCode::End), ChipBarCmd::Last, "Last", false),
    b(
        Chord::key(KeyCode::Enter),
        ChipBarCmd::Activate,
        "Open",
        true,
    ),
];

const PLAIN: [Binding<ChipBarCmd>; 7] = MOVE;

const TOGGLING: [Binding<ChipBarCmd>; 8] = [
    MOVE[0],
    MOVE[1],
    MOVE[2],
    MOVE[3],
    MOVE[4],
    MOVE[5],
    MOVE[6],
    b(
        Chord::key(KeyCode::Char(' ')),
        ChipBarCmd::Toggle,
        "Toggle",
        true,
    ),
];

const CLOSABLE: [Binding<ChipBarCmd>; 10] = [
    MOVE[0],
    MOVE[1],
    MOVE[2],
    MOVE[3],
    MOVE[4],
    MOVE[5],
    MOVE[6],
    b(
        Chord::key(KeyCode::Delete),
        ChipBarCmd::Close,
        "Remove",
        true,
    ),
    b(
        Chord::key(KeyCode::Backspace),
        ChipBarCmd::Close,
        "Remove",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('x')),
        ChipBarCmd::Close,
        "Remove",
        false,
    ),
];

const TOGGLING_CLOSABLE: [Binding<ChipBarCmd>; 11] = [
    CLOSABLE[0],
    CLOSABLE[1],
    CLOSABLE[2],
    CLOSABLE[3],
    CLOSABLE[4],
    CLOSABLE[5],
    CLOSABLE[6],
    CLOSABLE[7],
    CLOSABLE[8],
    CLOSABLE[9],
    TOGGLING[7],
];

pub type LabelChips<'a> = ChipBar<'a, &'a str, ByIndex, DefaultRow>;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ChipBarState {
    core: CollectionCore,
    on_add: bool,
    first: Option<ItemKey>,
    first_index: usize,
}

impl ChipBarState {
    pub const fn cursor(&self) -> Option<ItemKey> {
        if self.on_add {
            None
        } else {
            self.core.cursor()
        }
    }

    pub const fn on_add(&self) -> bool {
        self.on_add
    }

    pub const fn checked(&self) -> &KeySet {
        self.core.checked()
    }

    pub const fn checked_mut(&mut self) -> &mut KeySet {
        self.core.checked_mut()
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.on_add = false;
        self.core.set_cursor(index, key);
    }

    pub const fn first(&self) -> Option<ItemKey> {
        self.first
    }
}

impl Reconcile for ChipBarState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
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

pub struct ChipBar<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    select_mode: SelectMode,
    closable: bool,
    add: Option<&'a str>,
    read_only: bool,
    disabled: bool,
    ov: PartStyle<'a>,
    _t: PhantomData<fn() -> T>,
}

impl<T, K, R> fmt::Debug for ChipBar<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChipBar")
            .field("id", &self.id)
            .field("select_mode", &self.select_mode)
            .field("closable", &self.closable)
            .field("add", &self.add)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<T> ChipBar<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        ChipBar {
            id,
            key: ByIndex,
            row: DefaultRow,
            select_mode: SelectMode::Multi,
            closable: false,
            add: None,
            read_only: false,
            disabled: false,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a> ChipBar<'a, &'a str, ByIndex, DefaultRow> {
    fn with_inherited_disabled(&self, inherited: bool) -> Self {
        ChipBar {
            id: self.id,
            key: ByIndex,
            row: DefaultRow,
            select_mode: self.select_mode,
            closable: self.closable,
            add: self.add,
            read_only: self.read_only,
            disabled: self.disabled || inherited,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ChipBarState,
        value: &mut KeySet,
        items: &[&'a str],
        inherited_disabled: bool,
    ) -> Response<ChipBarAction> {
        *st.checked_mut() = value.clone();
        let response = self
            .with_inherited_disabled(inherited_disabled)
            .update(cx, st, items);
        *value = st.checked().clone();
        response
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &ChipBarState,
        items: &[&'a str],
        inherited_disabled: bool,
    ) -> Rect {
        self.with_inherited_disabled(inherited_disabled)
            .draw(ui, area, st, items)
    }

    /// Registers this Form chip bar's keyboard stop without a pointer region.
    ///
    /// Use only when Form layout clips the native chip bar body. Its native
    /// disabled and read-only policy remains authoritative.
    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }
}

impl<'a, T, K, R> ChipBar<'a, T, K, R> {
    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::MARKER,
        Part::LABEL,
        Part::CLOSE,
        Part::OVERFLOW,
        Part::NEW,
    ];

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> ChipBar<'a, T, K2, R> {
        ChipBar {
            id: self.id,
            key: k,
            row: self.row,
            select_mode: self.select_mode,
            closable: self.closable,
            add: self.add,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> ChipBar<'a, T, K, R2> {
        ChipBar {
            id: self.id,
            key: self.key,
            row: r,
            select_mode: self.select_mode,
            closable: self.closable,
            add: self.add,
            read_only: self.read_only,
            disabled: self.disabled,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn select_mode(mut self, m: SelectMode) -> Self {
        self.select_mode = m;
        self
    }

    #[must_use]
    pub const fn closable(mut self, yes: bool) -> Self {
        self.closable = yes;
        self
    }

    #[must_use]
    pub const fn add(mut self, label: &'a str) -> Self {
        self.add = Some(label);
        self
    }

    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
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

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn toggles(&self) -> bool {
        matches!(self.select_mode, SelectMode::Multi | SelectMode::Range)
    }

    fn table(&self) -> &'static [Binding<ChipBarCmd>] {
        match (self.toggles(), self.closable) {
            (true, true) => &TOGGLING_CLOSABLE,
            (true, false) => &TOGGLING,
            (false, true) => &CLOSABLE,
            (false, false) => &PLAIN,
        }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> ChipBar<'_, T, K, R> {
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

    fn move_cursor(
        &self,
        st: &mut ChipBarState,
        items: &[T],
        to: usize,
        acc: &mut Acc<ChipBarAction>,
    ) {
        let len = items.len();
        let stops = len.saturating_add(usize::from(self.add.is_some()));
        if stops == 0 {
            acc.consumed();
            return;
        }
        let to = to.min(stops.saturating_sub(1));
        if to >= len {
            st.on_add = true;
        } else {
            st.set_cursor(to, self.key_at(items, to));
        }
        acc.changed();
    }

    fn follow(&self, st: &mut ChipBarState, items: &[T], fit: usize) {
        if st.on_add {
            return;
        }
        let Some(cursor) = st.core.cursor() else {
            return;
        };
        let Some(ci) = self.index_of(items, cursor, Some(st.core.cursor_index())) else {
            return;
        };
        if ci < st.first_index {
            st.first_index = ci;
            st.first = Some(cursor);
        } else if fit > 0 && ci >= st.first_index.saturating_add(fit) {
            let i = ci.saturating_add(1).saturating_sub(fit);
            st.first_index = i;
            st.first = Some(self.key_at(items, i));
        }
    }

    fn cursor_stop(st: &ChipBarState, items: &[T]) -> usize {
        if st.on_add {
            items.len()
        } else {
            st.core.cursor_index()
        }
    }

    fn activate(&self, st: &mut ChipBarState, items: &[T], i: usize, acc: &mut Acc<ChipBarAction>) {
        if i >= items.len() {
            match self.add {
                Some(_) => acc.action(ChipBarAction::AddRequested),
                None => acc.consumed(),
            }
            return;
        }
        let key = self.key_at(items, i);
        st.set_cursor(i, key);
        acc.action(ChipBarAction::Activated(key));
    }

    fn toggle(&self, st: &mut ChipBarState, items: &[T], i: usize, acc: &mut Acc<ChipBarAction>) {
        if i >= items.len() || !self.toggles() {
            acc.consumed();
            return;
        }
        let key = self.key_at(items, i);
        st.set_cursor(i, key);
        st.core.checked_mut().toggle(key);
        acc.action(ChipBarAction::Toggled(key));
    }

    fn close(&self, st: &mut ChipBarState, items: &[T], i: usize, acc: &mut Acc<ChipBarAction>) {
        if i >= items.len() || !self.closable {
            acc.consumed();
            return;
        }
        let key = self.key_at(items, i);
        st.set_cursor(i, key);
        acc.action(ChipBarAction::Closed(key));
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut ChipBarState,
        items: &[T],
    ) -> Response<ChipBarAction> {
        let len = items.len();
        let can = self.editable();
        if !self.disabled {
            let _ = st.reconcile(len, |i| self.key_at(items, i));
            if st.core.cursor().is_none() && len > 0 {
                st.set_cursor(0, self.key_at(items, 0));
            }
            if len == 0 && self.add.is_some() {
                st.on_add = true;
            }
            if st.first.is_none() && len > 0 {
                st.first = Some(self.key_at(items, 0));
                st.first_index = 0;
            }
        }
        let mut acc = Acc::<ChipBarAction>::new();
        let table = self.table();
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) if can => {
                    let cur = Self::cursor_stop(st, items);
                    match Binding::command(table, action) {
                        Some(ChipBarCmd::Prev) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(ChipBarCmd::Next) => {
                            self.move_cursor(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(ChipBarCmd::First) => self.move_cursor(st, items, 0, &mut acc),
                        Some(ChipBarCmd::Last) => {
                            self.move_cursor(st, items, usize::MAX, &mut acc);
                        }
                        Some(ChipBarCmd::Activate) => self.activate(st, items, cur, &mut acc),
                        Some(ChipBarCmd::Toggle) => self.toggle(st, items, cur, &mut acc),
                        Some(ChipBarCmd::Close) => self.close(st, items, cur, &mut acc),
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part: PartRef { part, item },
                    ..
                } if can => {
                    let hint = Some(Self::cursor_stop(st, items));
                    let index = item.and_then(|key| self.index_of(items, key, hint));
                    match (phase, part, item, index) {
                        (Phase::Press, Part::LABEL, Some(_), Some(i)) => {
                            self.move_cursor(st, items, i, &mut acc);
                        }
                        (Phase::Press, Part::NEW, None, None) => {
                            self.move_cursor(st, items, usize::MAX, &mut acc);
                        }
                        (Phase::Click | Phase::DoubleClick, Part::LABEL, Some(_), Some(i)) => {
                            if self.toggles() {
                                self.toggle(st, items, i, &mut acc);
                            } else {
                                self.activate(st, items, i, &mut acc);
                            }
                        }
                        (Phase::Click | Phase::DoubleClick, Part::NEW, None, None) => {
                            self.activate(st, items, items.len(), &mut acc);
                        }
                        (Phase::Click, Part::CLOSE, Some(_), Some(i)) => {
                            self.close(st, items, i, &mut acc);
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer { .. } => acc.consumed(),
                _ => {}
            }
        }
        if let Some(l) = cx.layout(self.id) {
            self.follow(st, items, l.viewport_len);
        }
        acc.finish(self.id)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the strip: chips, close affordances, overflow and the add stop"
    )]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ChipBarState, items: &[T]) -> Rect {
        let row0 = first_row(area);
        if row0.is_empty() {
            return row0;
        }
        if !ui.is_inert() {
            let f = self.form_focusability(false);
            ui.register_control(self.id, row0, f);
        }

        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, self.table());
        }
        let ov = self.ov;
        let id = self.id;
        let strip = ov.style(
            ui,
            id,
            Family::CHIP,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(row0, strip.style);
        let add_w = self
            .add
            .map_or(0, |l| crate::text::width(l).saturating_add(3));
        let right_limit = row0.right().saturating_sub(add_w);
        let first_index = st
            .first
            .and_then(|f| self.index_of(items, f, Some(st.first_index)))
            .unwrap_or(0)
            .min(items.len());
        let mut x = row0.x;
        let cursor = st.core.cursor();
        let mut truncated = false;
        let mut fit = 0usize;
        for (i, item) in items.iter().enumerate().skip(first_index) {
            let key = self.key.key(item, i);
            let avail = right_limit.saturating_sub(x);
            if avail < 4 {
                truncated = i < items.len();
                break;
            }
            let is_cursor = !st.on_add && cursor == Some(key);
            let mut flags = StateFlags::empty();
            if is_cursor {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            if ui.hovered_part(self.id) == Some(PartRef::item(Part::LABEL, key)) {
                flags |= StateFlags::HOVERED;
            }
            if ui.pressed_part(self.id) == Some(PartRef::item(Part::LABEL, key)) {
                flags |= StateFlags::PRESSED;
            }
            if st.core.checked().contains(key) {
                flags |= StateFlags::CHECKED;
            }
            if self.read_only {
                flags |= StateFlags::READ_ONLY;
            }
            if self.disabled || live.contains(StateFlags::DISABLED) {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(StateFlags::PRESSED | StateFlags::HOVERED);
            }

            let content = Rect {
                x: x.saturating_add(1),
                y: row0.y,
                width: avail.saturating_sub(1),
                height: 1,
            };
            {
                let mut r = RowUi::new_with_patches(
                    ui,
                    id,
                    Family::CHIP,
                    Variant::DEFAULT,
                    flags,
                    key,
                    content,
                    ov.part_patch(Part::CONTAINER),
                    ov.part_patch(Part::LABEL),
                );
                self.row.row(item, &mut r);
            }
            let label_w = painted_width(ui, content).max(1);
            let close_w: u16 = if self.closable { 2 } else { 0 };
            let chip_w = 1u16
                .saturating_add(label_w)
                .saturating_add(1)
                .saturating_add(close_w);
            if chip_w > avail {
                ui.fill(content, strip.style);
                truncated = true;
                break;
            }
            let chip = Rect {
                x,
                y: row0.y,
                width: chip_w,
                height: 1,
            };

            let cs = ov.style(
                ui,
                id,
                Family::CHIP,
                Variant::DEFAULT,
                Part::CONTAINER,
                flags,
            );
            ui.paint_style(cell_at(chip, chip.x), cs.style);
            if flags.contains(StateFlags::CHECKED) {
                let cell = cell_at(chip, chip.x);
                let marker = ov.style(ui, id, Family::CHIP, Variant::DEFAULT, Part::MARKER, flags);
                match marker.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(cell, glyph, marker.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(cell, GlyphRole::Checked, marker.style);
                    }
                    Slot::Clear => ui.fill(cell, marker.style),
                }
            }
            if flags.contains(StateFlags::PRESSED) {
                let ls = ov.style(ui, id, Family::CHIP, Variant::DEFAULT, Part::LABEL, flags);
                if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                    let right = chip
                        .right()
                        .saturating_sub(1)
                        .saturating_sub(close_w)
                        .max(chip.x);
                    paint_pressed_bracket(
                        ui,
                        cell_at(chip, chip.x),
                        cell_at(chip, right),
                        ls.style,
                    );
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::item(Part::LABEL, key), chip);
            }
            if self.closable {
                let close_cell = cell_at(chip, chip.right().saturating_sub(2));
                let mut close_flags = flags.difference(StateFlags::HOVERED | StateFlags::PRESSED);
                if ui.hovered_part(self.id) == Some(PartRef::item(Part::CLOSE, key)) {
                    close_flags |= StateFlags::HOVERED;
                }
                if ui.pressed_part(self.id) == Some(PartRef::item(Part::CLOSE, key)) {
                    close_flags |= StateFlags::PRESSED;
                }
                if let Some(f) = ov.slot_for(Part::CLOSE) {
                    f(ui, close_cell);
                } else {
                    let xs = ov.style(
                        ui,
                        id,
                        Family::CHIP,
                        Variant::DEFAULT,
                        Part::CLOSE,
                        close_flags,
                    );
                    match xs.glyph {
                        Slot::Set(g) => {
                            ui.glyph(close_cell, g, xs.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(close_cell, GlyphRole::Close, xs.style);
                        }
                        Slot::Clear => {
                            ui.fill(close_cell, xs.style);
                        }
                    }
                }
                if !ui.is_inert() {
                    ui.register_part(self.id, PartRef::item(Part::CLOSE, key), close_cell);
                }
            }
            x = chip.right().saturating_add(1);
            fit = fit.saturating_add(1);
        }
        if truncated {
            let cell = cell_at(row0, x.min(row0.right().saturating_sub(1)));
            if let Some(f) = ov.slot_for(Part::OVERFLOW) {
                f(ui, cell);
            } else {
                let os = ov.style(
                    ui,
                    id,
                    Family::CHIP,
                    Variant::DEFAULT,
                    Part::OVERFLOW,
                    StateFlags::empty(),
                );
                match os.glyph {
                    Slot::Set(g) => {
                        ui.glyph(cell, g, os.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(cell, GlyphRole::Ellipsis, os.style);
                    }
                    Slot::Clear => {
                        ui.fill(cell, os.style);
                    }
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::of(Part::OVERFLOW), cell);
            }
        }
        if let Some(label) = self.add {
            let cell = Rect {
                x: row0.right().saturating_sub(add_w).max(row0.x),
                y: row0.y,
                width: add_w.min(row0.width),
                height: 1,
            };
            if !cell.is_empty() {
                let mut flags = StateFlags::empty();
                if st.on_add {
                    flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
                }
                if ui.pressed_part(self.id) == Some(PartRef::of(Part::NEW)) {
                    flags |= StateFlags::PRESSED;
                }
                if ui.hovered_part(self.id) == Some(PartRef::of(Part::NEW)) {
                    flags |= StateFlags::HOVERED;
                }
                if self.disabled || live.contains(StateFlags::DISABLED) {
                    flags |= StateFlags::DISABLED;
                }
                let bg = ov.style(
                    ui,
                    id,
                    Family::CHIP,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    flags,
                );
                ui.fill(cell, bg.style);
                let ns = ov.style(ui, id, Family::CHIP, Variant::DEFAULT, Part::NEW, flags);
                ui.paint_str(shift(cell, 1), label, ns.style);
                if !ui.is_inert() {
                    ui.register_part(self.id, PartRef::of(Part::NEW), cell);
                }
            }
        }
        if !ui.is_inert() {
            ui.report_layout(
                self.id,
                LayoutFacts::new(fit, items.len(), row0.height, row0.width),
            );
        }
        row0
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (8, 1),
            preferred: (c.max.0, 1),
        }
        .fit(c)
    }
}

fn painted_width(ui: &mut Ui<'_>, row: Rect) -> u16 {
    ui.with_area(row, |ui| {
        let (buf, clip) = ui.raw();
        let mut last = 0u16;
        let mut blank_run = 0u16;
        let mut gap_start = None;
        for x in clip.columns().map(|c| c.x) {
            let non_blank = buf
                .cell(Position::new(x, clip.y))
                .is_some_and(|c| c.symbol() != " ");
            if non_blank {
                if blank_run >= 2 {
                    gap_start = Some(x.saturating_sub(blank_run).saturating_sub(clip.x));
                }
                blank_run = 0;
                last = x.saturating_sub(clip.x).saturating_add(1);
            } else {
                blank_run = blank_run.saturating_add(1);
            }
        }
        if last == clip.width {
            gap_start.unwrap_or(last)
        } else {
            last
        }
    })
}

impl<T, K, R> Bindings for ChipBar<'_, T, K, R> {
    type Cmd = ChipBarCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<ChipBarCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::{Buffer, Cell};
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{Stub, key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::theme::{ColorLevel, Theme};

    const BAR: Id = Id::root("chip.tests");
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 12,
        height: 1,
    };

    #[derive(Default)]
    struct AddApp {
        state: ChipBarState,
        actions: Vec<ChipBarAction>,
    }

    struct SlotApp(Option<Part>);

    impl App for SlotApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let replaced = |ui: &mut Ui<'_>, area: Rect| {
                let style = ui.surface_style();
                ui.paint_str(area, "########", style);
            };
            let row: fn(&&str, &mut RowUi<'_>) = |item, row| {
                row.label(item);
                row.meta("meta");
            };
            let close_items = ["a"];
            let mut close = ChipBar::new(BAR).row(row).closable(true).add("+ Add");
            let overflow_items = ["long label", "second"];
            let mut overflow = ChipBar::new(BAR.sub("overflow")).row(row);
            if let Some(part) = self.0 {
                close = close.slot(part, &replaced);
                overflow = overflow.slot(part, &replaced);
            }
            close.draw(
                ui,
                Rect::new(0, 0, 20, 1),
                &ChipBarState::default(),
                &close_items,
            );
            overflow.draw(
                ui,
                Rect::new(0, 1, 8, 1),
                &ChipBarState::default(),
                &overflow_items,
            );
        }
    }

    fn slot_buffer(part: Option<Part>) -> Buffer {
        let screen = Rect::new(0, 0, 20, 2);
        let mut runtime = Runtime::new(SlotApp(part), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut buffer).commit_presented();
        buffer
    }

    impl App for AddApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let items = ["a"];
            let response = ChipBar::new(BAR)
                .select_mode(SelectMode::Single)
                .add("+ Add")
                .update(cx, &mut self.state, &items);
            if let Some(action) = response.action_ref() {
                self.actions.push(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let items = ["a"];
            ChipBar::new(BAR)
                .select_mode(SelectMode::Single)
                .add("+ Add")
                .draw(ui, AREA, &self.state, &items);
        }
    }

    const POINTER_ITEMS: [&str; 2] = ["Alpha", "Beta"];
    const POINTER_AREA: Rect = Rect::new(0, 0, 28, 1);

    struct PointerApp {
        state: ChipBarState,
        actions: Vec<ChipBarAction>,
        mode: SelectMode,
        disabled: bool,
        read_only: bool,
    }

    impl PointerApp {
        fn new(mode: SelectMode, disabled: bool, read_only: bool) -> Self {
            let mut state = ChipBarState::default();
            state.checked_mut().insert(ItemKey::index(0));
            state.checked_mut().insert(ItemKey::index(1));
            Self {
                state,
                actions: Vec::new(),
                mode,
                disabled,
                read_only,
            }
        }

        fn bar(&self) -> ChipBar<'static, &'static str> {
            ChipBar::new(BAR)
                .closable(true)
                .select_mode(self.mode)
                .disabled(self.disabled)
                .read_only(self.read_only)
        }
    }

    impl App for PointerApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = self.bar().update(cx, &mut self.state, &POINTER_ITEMS);
            if let Some(action) = response.action_ref() {
                self.actions.push(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.bar()
                .draw(ui, POINTER_AREA, &self.state, &POINTER_ITEMS);
        }
    }

    fn pointer_app(mode: SelectMode, disabled: bool, read_only: bool) -> Runtime<PointerApp> {
        let mut runtime = Runtime::new(PointerApp::new(mode, disabled, read_only), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(POINTER_AREA);
        runtime
            .draw_buffer(POINTER_AREA, &mut buffer)
            .commit_presented();
        runtime
    }

    fn click_part<A: App>(runtime: &mut Runtime<A>, part: PartRef) {
        let area = runtime
            .area_of_part(BAR, part)
            .expect("chip part is registered");
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        let _ = crate::runtime::stub::deliver(runtime, mouse(MouseKind::Down, x, y));
        let _ = crate::runtime::stub::deliver(runtime, mouse(MouseKind::Up, x, y));
    }

    #[test]
    fn close_pointer_target_wins_and_label_keeps_mode_actions() {
        for (mode, label_action) in [
            (
                SelectMode::Single,
                ChipBarAction::Activated(ItemKey::index(0)),
            ),
            (SelectMode::Multi, ChipBarAction::Toggled(ItemKey::index(0))),
        ] {
            let mut runtime = pointer_app(mode, false, false);
            let close = PartRef::item(Part::CLOSE, ItemKey::index(1));
            let label = PartRef::item(Part::LABEL, ItemKey::index(1));
            let close_area = runtime.area_of_part(BAR, close).expect("close cell");
            let label_area = runtime.area_of_part(BAR, label).expect("label target");
            assert_eq!(close_area.intersection(label_area), close_area);
            let body = runtime
                .area_of_part(BAR, PartRef::item(Part::LABEL, ItemKey::index(0)))
                .expect("body label target");

            click_part(&mut runtime, close);
            assert_eq!(
                runtime.app().actions,
                [ChipBarAction::Closed(ItemKey::index(1))]
            );
            assert!(runtime.app().state.checked().contains(ItemKey::index(0)));
            assert!(runtime.app().state.checked().contains(ItemKey::index(1)));

            let body_x = body.x.saturating_add(1);
            let body_y = body.y;
            let _ =
                crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, body_x, body_y));
            let _ =
                crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, body_x, body_y));
            assert_eq!(
                runtime.app().actions,
                [ChipBarAction::Closed(ItemKey::index(1)), label_action]
            );
        }
    }

    #[test]
    fn disabled_and_read_only_close_targets_emit_no_actions() {
        for (disabled, read_only) in [(true, false), (false, true)] {
            let mut runtime = pointer_app(SelectMode::Multi, disabled, read_only);
            click_part(&mut runtime, PartRef::item(Part::CLOSE, ItemKey::index(1)));
            assert!(runtime.app().actions.is_empty());
            assert!(runtime.app().state.checked().contains(ItemKey::index(0)));
            assert!(runtime.app().state.checked().contains(ItemKey::index(1)));
        }
    }

    fn row_text(buf: &Buffer, width: u16) -> String {
        let mut text = String::new();
        for x in 0..width {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                text.push_str(cell.symbol());
            }
        }
        text
    }

    #[test]
    fn add_affordance_emits_add_requested_by_keyboard_and_mouse() {
        let items = ["a", "b"];
        let bar: ChipBar<'_, &str> = ChipBar::new(BAR).add("+ Add");
        let mut st = ChipBarState::default();
        let _ = st.core.reconcile(2, |i| bar.key_at(&items, i));
        st.set_cursor(0, ItemKey::index(0));
        let mut acc = Acc::<ChipBarAction>::new();
        bar.move_cursor(&mut st, &items, 1, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::index(1)));
        assert!(!st.on_add());
        bar.move_cursor(&mut st, &items, 2, &mut acc);
        assert!(st.on_add(), "the stop after the last chip is the add stop");
        assert_eq!(st.cursor(), None);
        let mut acc = Acc::<ChipBarAction>::new();
        bar.activate(&mut st, &items, 2, &mut acc);
        assert_eq!(
            acc.finish(BAR).action_ref(),
            Some(&ChipBarAction::AddRequested),
            "the add affordance has its own payloadless action"
        );

        let mut runtime = Runtime::new(AddApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::End));
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert_eq!(runtime.app().actions, [ChipBarAction::AddRequested]);

        let mut runtime = Runtime::new(AddApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let add = runtime
            .area_of_part(BAR, PartRef::of(Part::NEW))
            .unwrap_or(Rect::ZERO);
        assert!(
            !add.is_empty(),
            "the configured add affordance is registered"
        );
        let x = add.x.saturating_add(add.width / 2);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, x, add.y));
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, x, add.y));
        assert_eq!(runtime.app().actions, [ChipBarAction::AddRequested]);
    }

    #[test]
    fn add_affordance_cannot_collide_with_item_identity() {
        let item = ItemKey::index(0);
        let mut runtime = Runtime::new(AddApp::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();

        assert_ne!(PartRef::item(Part::LABEL, item), PartRef::of(Part::NEW));
        assert!(
            runtime
                .area_of_part(BAR, PartRef::item(Part::LABEL, item))
                .is_some()
        );
        assert!(runtime.area_of_part(BAR, PartRef::of(Part::NEW)).is_some());

        let items = ["a"];
        let bar: ChipBar<'_, &str> = ChipBar::new(BAR).add("+ Add");
        let mut state = ChipBarState::default();
        let mut item_action = Acc::<ChipBarAction>::new();
        bar.activate(&mut state, &items, 0, &mut item_action);
        assert_eq!(
            item_action.finish(BAR).action_ref(),
            Some(&ChipBarAction::Activated(item))
        );
        let mut add_action = Acc::<ChipBarAction>::new();
        bar.activate(&mut state, &items, items.len(), &mut add_action);
        assert_eq!(
            add_action.finish(BAR).action_ref(),
            Some(&ChipBarAction::AddRequested)
        );
    }

    #[test]
    fn chip_bar_parts_are_exact() {
        assert_eq!(
            ChipBar::<&str>::PARTS,
            &[
                Part::CONTAINER,
                Part::MARKER,
                Part::LABEL,
                Part::CLOSE,
                Part::OVERFLOW,
                Part::NEW,
            ]
        );
    }

    #[test]
    fn custom_row_meta_paints_without_joining_chip_bar_parts() {
        let buffer = slot_buffer(None);
        assert!(
            row_text(&buffer, 20).contains("meta"),
            "the caller-owned row metadata still paints"
        );
        assert!(!ChipBar::<&str>::PARTS.contains(&Part::META));
    }

    #[test]
    fn chip_label_patch_changes_label_but_not_row_meta() {
        let row: fn(&&str, &mut RowUi<'_>) = |item, row| {
            row.label(item);
            row.meta("meta");
        };
        let patches = [(Part::LABEL, StylePatch::new().add(Modifier::BOLD))];
        let items = ["a"];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 1));
        runtime
            .draw_scene(Rect::new(0, 0, 20, 1), &mut buffer, |ui, area| {
                ChipBar::new(BAR).row(row).patch_part(&patches).draw(
                    ui,
                    area,
                    &ChipBarState::default(),
                    &items,
                );
            })
            .commit_presented();
        assert!(
            buffer
                .cell(Position::new(1, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD))
        );
        assert!(
            buffer
                .cell(Position::new(16, 0))
                .is_some_and(|cell| !cell.modifier.contains(Modifier::BOLD)),
            "caller-owned META must not inherit ChipBar's LABEL patch"
        );
    }

    #[test]
    fn a_slot_changes_painted_cells_for_exactly_close_and_overflow() {
        let plain = slot_buffer(None);
        for (part, honoured) in [
            (Part::CONTAINER, false),
            (Part::MARKER, false),
            (Part::LABEL, false),
            (Part::META, false),
            (Part::CLOSE, true),
            (Part::OVERFLOW, true),
            (Part::NEW, false),
        ] {
            assert_eq!(
                slot_buffer(Some(part)) != plain,
                honoured,
                "unexpected `.slot({part:?}, …)` support"
            );
        }
    }

    #[test]
    fn checked_membership_paints_marker_and_unchecked_stays_blank() {
        let items = ["item"];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let unchecked = ChipBarState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ChipBar::new(BAR).draw(ui, area, &unchecked, &items);
            })
            .commit_presented();
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(Cell::symbol),
            Some(" ")
        );

        let mut checked = ChipBarState::default();
        checked.checked_mut().insert(ItemKey::index(0));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ChipBar::new(BAR).draw(ui, area, &checked, &items);
            })
            .commit_presented();
        assert_ne!(
            buffer.cell(Position::new(0, 0)).map(Cell::symbol),
            Some(" "),
            "checked membership paints the resolved marker glyph"
        );
    }

    #[test]
    fn toggle_action_uses_the_items_stable_key() {
        let items = ["alpha", "beta"];
        let by_text: fn(&&str) -> ItemKey = |item| ItemKey::text(item);
        let bar = ChipBar::new(BAR).key(by_text);
        let mut state = ChipBarState::default();
        let mut acc = Acc::<ChipBarAction>::new();

        bar.toggle(&mut state, &items, 1, &mut acc);

        assert_eq!(
            acc.finish(BAR).action_ref(),
            Some(&ChipBarAction::Toggled(ItemKey::text("beta")))
        );
        assert!(state.checked().contains(ItemKey::text("beta")));
    }

    #[test]
    fn checked_marker_honours_resolved_glyph_set_and_clear() {
        let items = ["item"];
        let mut state = ChipBarState::default();
        state.checked_mut().insert(ItemKey::index(0));
        let set = [(
            Part::MARKER,
            StylePatch::new().set_glyph(GlyphRole::WarningMark),
        )];
        let clear = [(
            Part::MARKER,
            StylePatch {
                glyph: Slot::Clear,
                ..StylePatch::new()
            },
        )];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ChipBar::new(BAR)
                    .patch_part(&set)
                    .draw(ui, area, &state, &items);
            })
            .commit_presented();
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::WarningMark))
        );
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ChipBar::new(BAR)
                    .patch_part(&clear)
                    .draw(ui, area, &state, &items);
            })
            .commit_presented();
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(Cell::symbol),
            Some(" ")
        );
    }

    #[test]
    fn checked_marker_paints_the_canonical_truecolor_glyph_in_one_cell() {
        let items = ["item"];
        let mut state = ChipBarState::default();
        state.checked_mut().insert(ItemKey::index(0));

        for theme in [Theme::junie(), Theme::paper()] {
            assert_eq!(theme.capability.color, ColorLevel::TrueColor);
            let checked = theme.design.glyphs.get(GlyphRole::Checked);
            let mut runtime = Runtime::new(Stub::default(), theme);
            let mut buffer = Buffer::empty(AREA);
            runtime
                .draw_scene(AREA, &mut buffer, |ui, area| {
                    ChipBar::new(BAR).draw(ui, area, &state, &items);
                })
                .commit_presented();
            assert_eq!(
                buffer.cell(Position::new(0, 0)).map(Cell::symbol),
                Some(checked)
            );
            assert_eq!(
                buffer.cell(Position::new(1, 0)).map(Cell::symbol),
                Some("i")
            );
        }
    }

    #[test]
    fn toggle_and_close_name_the_chip() {
        let items = ["a", "b", "c"];
        let bar: ChipBar<'_, &str> = ChipBar::new(BAR).closable(true);
        let mut st = ChipBarState::default();
        let _ = st.core.reconcile(3, |i| bar.key_at(&items, i));
        let mut acc = Acc::<ChipBarAction>::new();
        bar.toggle(&mut st, &items, 1, &mut acc);
        assert_eq!(
            acc.finish(BAR).action_ref(),
            Some(&ChipBarAction::Toggled(ItemKey::index(1)))
        );
        assert!(st.checked().contains(ItemKey::index(1)));
        let mut acc = Acc::<ChipBarAction>::new();
        bar.close(&mut st, &items, 2, &mut acc);
        assert_eq!(
            acc.finish(BAR).action_ref(),
            Some(&ChipBarAction::Closed(ItemKey::index(2)))
        );

        let plain: ChipBar<'_, &str> = ChipBar::new(BAR);
        let mut acc = Acc::<ChipBarAction>::new();
        plain.close(&mut st, &items, 0, &mut acc);
        let r = acc.finish(BAR);
        assert!(r.is_consumed() && r.action_ref().is_none());
    }

    #[test]
    fn the_strip_window_is_keyed_so_a_reordered_chip_stays_addressable() {
        const STRIP: Rect = Rect {
            x: 0,
            y: 0,
            width: 12,
            height: 1,
        };
        let by_text: fn(&&str) -> ItemKey = |s| ItemKey::text(s);
        let bar = ChipBar::new(BAR).key(by_text);
        let head = ItemKey::text("alpha");
        let items = ["alpha", "beta", "gamma", "delta"];
        let mut st = ChipBarState::default();
        let _ = st.reconcile(items.len(), |i| bar.key_at(&items, i));
        st.set_cursor(0, head);
        st.first = Some(head);

        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(STRIP);
        runtime
            .draw_scene(STRIP, &mut buffer, |ui, area| {
                bar.draw(ui, area, &st, &items);
            })
            .commit_presented();
        let before = runtime.area_of_part(BAR, PartRef::item(Part::LABEL, head));
        assert_eq!(
            before.map(|r| r.x),
            Some(0),
            "the window head is the leftmost chip"
        );

        let reversed = ["delta", "gamma", "beta", "alpha"];
        let _ = st.reconcile(reversed.len(), |i| bar.key_at(&reversed, i));
        assert_eq!(st.first(), Some(head), "the window head keeps its key");
        assert_eq!(st.first_index, 3, "and follows it to its new position");
        let mut buffer = Buffer::empty(STRIP);
        runtime
            .draw_scene(STRIP, &mut buffer, |ui, area| {
                bar.draw(ui, area, &st, &reversed);
            })
            .commit_presented();
        assert_eq!(
            runtime.area_of_part(BAR, PartRef::item(Part::LABEL, head)),
            before,
            "and the chip is painted and addressable in the same cells"
        );
    }

    #[test]
    fn mono_pressed_brackets_the_reserved_pad_cells() {
        const LABEL: &str = "Full width";
        let items = [LABEL];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        let state = ChipBarState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let target = crate::ReferenceTarget::new(
                    BAR,
                    crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                )
                .part(PartRef::item(Part::LABEL, ItemKey::index(0)));
                ui.reference(Some(target), |ui| {
                    ChipBar::new(BAR).draw(ui, area, &state, &items);
                });
            })
            .commit_presented();

        assert_eq!(row_text(&buffer, AREA.width), "[Full width]");
    }

    #[test]
    fn mono_pressed_closable_chip_keeps_the_close_cell_after_the_bracket() {
        let items = ["tab"];
        let key = ItemKey::index(0);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let target = crate::ReferenceTarget::new(
                    BAR,
                    crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                )
                .part(PartRef::item(Part::LABEL, key));
                ui.reference(Some(target), |ui| {
                    ChipBar::new(BAR).closable(true).draw(
                        ui,
                        area,
                        &ChipBarState::default(),
                        &items,
                    );
                });
            })
            .commit_presented();

        assert_eq!(
            buffer.cell(Position::new(4, 0)).map(Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::PressRight))
        );
        assert_eq!(
            buffer.cell(Position::new(5, 0)).map(Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Close))
        );
    }
}
