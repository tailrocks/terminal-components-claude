use core::fmt;
use core::marker::PhantomData;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;

use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::layer::{Anchor, CrossAlign, Dismiss, LayerEvent, LayerSize, LayerSpec, Side};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Slot, StylePatch, Surface, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_controls::choice::InheritedFormState;
use termrock_runtime::empty::EmptyState;
use termrock_runtime::rowui::{RowFn, RowUi};
use termrock_runtime::scroll_region::ScrollRegion;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectAction {
    Chose(ItemKey),
    Opened,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectCmd {
    Choose,
    Prev,
    Next,
    First,
    Last,
    PageUp,
    PageDown,
}

const fn b(chord: Chord, cmd: SelectCmd, label: &'static str, visible: bool) -> Binding<SelectCmd> {
    Binding {
        action: ActionKey::custom(label),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 70 } else { 10 },
        visible,
    }
}

const BINDINGS: &[Binding<SelectCmd>] = &[
    b(
        Chord::key(KeyCode::Enter),
        SelectCmd::Choose,
        "Choose",
        true,
    ),
    b(
        Chord::key(KeyCode::Char(' ')),
        SelectCmd::Choose,
        "Choose (Space)",
        false,
    ),
    b(Chord::key(KeyCode::Up), SelectCmd::Prev, "Up", true),
    b(Chord::key(KeyCode::Down), SelectCmd::Next, "Down", true),
    b(
        Chord::key(KeyCode::Char('k')),
        SelectCmd::Prev,
        "Up (K)",
        false,
    ),
    b(
        Chord::key(KeyCode::Char('j')),
        SelectCmd::Next,
        "Down (J)",
        false,
    ),
    b(Chord::key(KeyCode::Home), SelectCmd::First, "First", false),
    b(Chord::key(KeyCode::End), SelectCmd::Last, "Last", false),
    b(
        Chord::key(KeyCode::PageUp),
        SelectCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        Chord::key(KeyCode::PageDown),
        SelectCmd::PageDown,
        "Page down",
        false,
    ),
];

pub type LabelSelect<'a> = Select<'a, &'a str, ByIndex, DefaultRow>;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SelectState {
    core: CollectionCore,
    value: Option<ItemKey>,
    open: bool,
}

impl SelectState {
    pub const fn value(&self) -> Option<ItemKey> {
        self.value
    }

    pub fn set_value(&mut self, k: Option<ItemKey>) {
        self.value = k;
    }

    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub const fn scroll(&self) -> &crate::scroll::ScrollState {
        self.core.scroll()
    }
}

impl Reconcile for SelectState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if let Some(v) = self.value
            && !(0..len).any(|i| key(i) == v)
        {
            self.value = None;
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

pub struct Select<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    placeholder: Option<&'a str>,
    popup_rows: Option<u16>,
    empty: Option<EmptyState<'a>>,
    read_only: bool,
    disabled: bool,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
    _t: PhantomData<fn() -> T>,
}

impl<T, K, R> fmt::Debug for Select<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Select")
            .field("id", &self.id)
            .field("placeholder", &self.placeholder)
            .field("popup_rows", &self.popup_rows)
            .field("read_only", &self.read_only)
            .field("disabled", &self.disabled)
            .finish_non_exhaustive()
    }
}

impl<T> Select<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        Select {
            id,
            key: ByIndex,
            row: DefaultRow,
            placeholder: None,
            popup_rows: None,
            empty: None,
            read_only: false,
            disabled: false,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a> Select<'a, &'a str, ByIndex, DefaultRow> {
    fn with_inherited_disabled(&self, inherited_disabled: bool) -> Self {
        Select {
            id: self.id,
            key: ByIndex,
            row: DefaultRow,
            placeholder: self.placeholder,
            popup_rows: self.popup_rows,
            empty: self.empty,
            read_only: self.read_only,
            disabled: self.disabled || inherited_disabled,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn update_in_form(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SelectState,
        value: &mut usize,
        items: &[&'a str],
        inherited_disabled: bool,
    ) -> Response<SelectAction> {
        st.set_value(Some(ItemKey::index(*value)));
        let response = self
            .with_inherited_disabled(inherited_disabled)
            .update(cx, st, items);
        if let Some(SelectAction::Chose(ItemKey::Index(index))) = response.action_ref() {
            *value = *index;
        }
        response
    }

    pub fn draw_in_form(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &SelectState,
        _value: usize,
        items: &[&'a str],
        inherited: InheritedFormState,
    ) -> Rect {
        self.with_inherited_disabled(inherited.disabled)
            .draw(ui, area, st, items)
    }

    pub fn register_focus_only_in_form(&self, ui: &mut Ui<'_>, inherited_disabled: bool) {
        if !ui.is_inert() {
            ui.register_focus_only(self.id, self.form_focusability(inherited_disabled));
        }
    }
}

impl<'a, T, K, R> Select<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[
        Part::FIELD,
        Part::GUTTER,
        Part::LABEL,
        Part::PLACEHOLDER,
        Part::MARKER,
        Part::ROW,
        Part::TRACK,
        Part::THUMB,
        Part::EMPTY,
    ];

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Select<'a, T, K2, R> {
        Select {
            id: self.id,
            key: k,
            row: self.row,
            placeholder: self.placeholder,
            popup_rows: self.popup_rows,
            empty: self.empty,
            read_only: self.read_only,
            disabled: self.disabled,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Select<'a, T, K, R2> {
        Select {
            id: self.id,
            key: self.key,
            row: r,
            placeholder: self.placeholder,
            popup_rows: self.popup_rows,
            empty: self.empty,
            read_only: self.read_only,
            disabled: self.disabled,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub const fn placeholder(mut self, s: &'a str) -> Self {
        self.placeholder = Some(s);
        self
    }

    #[must_use]
    pub const fn popup_rows(mut self, n: u16) -> Self {
        self.popup_rows = Some(n);
        self
    }

    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
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
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

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

    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    const fn form_focusability(&self, inherited_disabled: bool) -> Focusability {
        if self.disabled || inherited_disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        }
    }

    fn max_rows(&self, d: &crate::theme::DesignTokens) -> u16 {
        self.popup_rows.unwrap_or(d.size.popup_max_rows).max(1)
    }

    fn anchor(&self, cx: &Cx<'_>) -> Anchor {
        match cx.area(self.id) {
            Some(rect) => Anchor::Rect {
                rect,
                side: Side::Below,
                align: CrossAlign::Start,
            },

            None => Anchor::Screen(crate::layer::ScreenAlign::Center),
        }
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Select<'_, T, K, R> {
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

    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize {
        let d = cx.design();
        let w = cx
            .area(self.id)
            .map_or(d.size.popup_min_width, |a| a.width)
            .clamp(d.size.popup_min_width, d.size.popup_max_width);
        let rows = items
            .len()
            .min(usize::from(self.max_rows(d)))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        LayerSize::Fixed(w, rows.saturating_add(2))
    }

    pub fn layer(&self, cx: &Cx<'_>, items: &[T]) -> LayerSpec {
        LayerSpec::popover(self.id, self.anchor(cx))
            .dismiss(Dismiss::ALL)
            .size(self.measured_size(cx, items))
    }

    fn move_cursor(
        &self,
        st: &mut SelectState,
        items: &[T],
        to: usize,
        acc: &mut Acc<SelectAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let to = to.min(items.len().saturating_sub(1));
        st.core.set_cursor(to, self.key_at(items, to));

        acc.changed();
    }

    fn open(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SelectState,
        items: &[T],
        acc: &mut Acc<SelectAction>,
    ) {
        st.open = true;
        if let Some(i) = st.value.and_then(|v| self.index_of(items, v, None)) {
            st.core.set_cursor(i, self.key_at(items, i));
        }
        cx.open_layer(self.id, self.layer(cx, items));
        acc.action(SelectAction::Opened);
    }

    fn close_restoring(&self, st: &mut SelectState, items: &[T]) {
        st.open = false;
        if let Some(i) = st.value.and_then(|v| self.index_of(items, v, None)) {
            st.core.set_cursor(i, self.key_at(items, i));
        }
    }

    fn choose(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SelectState,
        items: &[T],
        i: usize,
        acc: &mut Acc<SelectAction>,
    ) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = i.min(items.len().saturating_sub(1));
        let key = self.key_at(items, i);
        st.core.set_cursor(i, key);
        st.value = Some(key);
        if st.open {
            st.open = false;
            cx.close_layer(self.id, None);
        }
        acc.action(SelectAction::Chose(key));
    }

    fn reconcile_and_seed(&self, st: &mut SelectState, items: &[T]) {
        if self.disabled {
            return;
        }
        let _ = st.reconcile(items.len(), |i| self.key_at(items, i));
        if st.core.cursor().is_none() && !items.is_empty() {
            let i = st
                .value
                .and_then(|v| self.index_of(items, v, None))
                .unwrap_or(0);
            st.core.set_cursor(i, self.key_at(items, i));
        }
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SelectState,
        items: &[T],
    ) -> Response<SelectAction> {
        let len = items.len();
        let was_open = st.open;
        self.reconcile_and_seed(st, items);
        let open = st.open && cx.is_open(self.id);
        if open {
            let size = self.measured_size(cx, items);
            let anchor = self.anchor(cx);
            cx.resize_layer(self.id, size);
            cx.reanchor_layer(self.id, anchor);
        } else if st.open {
            self.close_restoring(st, items);
        }
        let mut acc = Acc::<SelectAction>::new();
        if open {
            let bar = ScrollRegion::new(self.id).update(cx, st.core.scroll_mut(), len);
            acc.fold(&bar);
        }
        let page = st.core.scroll().viewport_len().max(1);
        let can = self.editable();
        for it in cx.intents(self.id) {
            match it {
                Intent::Layer(LayerEvent::Dismissed(_)) => {
                    if was_open {
                        self.close_restoring(st, items);
                        acc.action(SelectAction::Closed);
                    }
                }
                Intent::Layer(_) => acc.repaint(),
                Intent::Binding(action) if can => {
                    let cur = st.core.cursor_index();
                    match Binding::command(BINDINGS, action) {
                        Some(SelectCmd::Prev) => {
                            self.move_cursor(st, items, cur.saturating_sub(1), &mut acc);
                        }
                        Some(SelectCmd::Next) => {
                            self.move_cursor(st, items, cur.saturating_add(1), &mut acc);
                        }
                        Some(SelectCmd::First) => self.move_cursor(st, items, 0, &mut acc),
                        Some(SelectCmd::Last) => {
                            self.move_cursor(st, items, usize::MAX, &mut acc);
                        }
                        Some(SelectCmd::PageUp) => {
                            self.move_cursor(st, items, cur.saturating_sub(page), &mut acc);
                        }
                        Some(SelectCmd::PageDown) => {
                            self.move_cursor(st, items, cur.saturating_add(page), &mut acc);
                        }
                        Some(SelectCmd::Choose) => {
                            if st.open {
                                self.choose(cx, st, items, cur, &mut acc);
                            } else {
                                self.open(cx, st, items, &mut acc);
                            }
                        }
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part: PartRef { part, item },
                    ..
                } if can => match (phase, part, item) {
                    (Phase::Click | Phase::DoubleClick, Part::ROW, Some(k)) => {
                        match self.index_of(items, k, Some(st.core.cursor_index())) {
                            Some(i) => self.choose(cx, st, items, i, &mut acc),
                            None => acc.consumed(),
                        }
                    }
                    (Phase::Press, Part::ROW, Some(k)) => {
                        match self.index_of(items, k, Some(st.core.cursor_index())) {
                            Some(i) => self.move_cursor(st, items, i, &mut acc),
                            None => acc.consumed(),
                        }
                    }
                    (Phase::Click | Phase::DoubleClick, _, _) => {
                        if st.open {
                            self.close_restoring(st, items);
                            cx.close_layer(self.id, None);
                            acc.action(SelectAction::Closed);
                        } else {
                            self.open(cx, st, items, &mut acc);
                        }
                    }
                    _ => acc.consumed(),
                },
                Intent::Pointer { .. } => acc.consumed(),
                Intent::Cancel if st.open => {
                    self.close_restoring(st, items);
                    cx.close_layer(self.id, None);
                    acc.action(SelectAction::Closed);
                }
                _ => {}
            }
        }
        acc.finish(self.id)
    }

    fn live_flags(&self, ui: &Ui<'_>, st: &SelectState) -> StateFlags {
        let mut derived = StateFlags::empty();
        if st.open {
            derived |= StateFlags::EXPANDED;
        }
        if st.value.is_some() {
            derived |= StateFlags::SELECTED;
        }
        if self.read_only {
            derived |= StateFlags::READ_ONLY;
        }
        if self.disabled {
            derived |= StateFlags::DISABLED;
        }
        let mut live = PartStyle::flags(ui.state(self.id), derived);
        if st.value.is_none() {
            live = live.difference(StateFlags::SELECTED);
        }
        if self.disabled {
            live = live.difference(StateFlags::HOVERED);
        }
        live
    }

    fn register_field(&self, ui: &mut Ui<'_>, area: Rect) {
        if ui.is_inert() {
            return;
        }
        let f = self.form_focusability(false);
        ui.register_control(self.id, area, f);
    }

    fn paint_gutter(&self, ui: &mut Ui<'_>, cell: Rect, live: StateFlags) {
        if let Some(f) = self.ov.slot_for(Part::GUTTER) {
            f(ui, cell);
            return;
        }
        let g = self.ov.style(
            ui,
            self.id,
            Family::SELECT,
            Variant::DEFAULT,
            Part::GUTTER,
            live,
        );
        match g.glyph {
            Slot::Set(glyph) => {
                ui.glyph(cell, glyph, g.style);
            }
            Slot::Inherit | Slot::Clear => ui.fill(cell, g.style),
        }
    }

    fn paint_value(
        &self,
        ui: &mut Ui<'_>,
        cell: Rect,
        st: &SelectState,
        items: &[T],
        live: StateFlags,
        field: PaintStyle,
    ) {
        if cell.is_empty() {
            return;
        }
        let chosen = st
            .value
            .and_then(|v| self.index_of(items, v, None))
            .and_then(|i| items.get(i).map(|it| (i, it)));
        let Some((i, item)) = chosen else {
            if let Some(p) = self.placeholder {
                let ps = self.ov.style(
                    ui,
                    self.id,
                    Family::SELECT,
                    Variant::DEFAULT,
                    Part::PLACEHOLDER,
                    live,
                );
                ui.paint_str(cell, p, ps.style);
            }
            return;
        };
        let key = self.key.key(item, i);
        {
            let mut r = RowUi::new(
                ui,
                self.id,
                Family::SELECT,
                Variant::DEFAULT,
                live,
                key,
                cell,
            );
            self.row.row(item, &mut r);
        }

        ui.paint_style(cell, field);
    }

    fn paint_marker(&self, ui: &mut Ui<'_>, cell: Rect, live: StateFlags, open: bool) {
        if let Some(f) = self.ov.slot_for(Part::MARKER) {
            f(ui, cell);
            return;
        }
        let marker_flags = live.difference(StateFlags::SELECTED);
        let ms = self.ov.style(
            ui,
            self.id,
            Family::SELECT,
            Variant::DEFAULT,
            Part::MARKER,
            marker_flags,
        );
        let arrow = if open {
            GlyphRole::SelectOpen
        } else {
            GlyphRole::SelectClosed
        };
        match ms.glyph {
            Slot::Set(glyph) => {
                ui.glyph(cell, glyph, ms.style);
            }
            Slot::Inherit => {
                ui.glyph(cell, arrow, ms.style);
            }
            Slot::Clear => {
                ui.fill(cell, ms.style);
            }
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState, items: &[T]) -> Rect {
        let area = first_row(area);
        if area.is_empty() {
            return area;
        }
        let live = self.live_flags(ui, st);
        self.register_field(ui, area);
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, BINDINGS);
        }
        let field = self.ov.style(
            ui,
            self.id,
            Family::SELECT,
            Variant::DEFAULT,
            Part::FIELD,
            live,
        );
        ui.fill(area, field.style);
        self.paint_gutter(ui, cell_at(area, area.x), live);
        let value = Rect {
            x: area.x.saturating_add(2),
            y: area.y,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        self.paint_value(ui, value, st, items, live, field.style);
        self.paint_marker(
            ui,
            cell_at(area, area.right().saturating_sub(2)),
            live,
            st.open,
        );
        if !ui.is_inert() {
            ui.register_part(self.id, PartRef::of(Part::FIELD), area);
        }
        self.draw_popup(ui, st, items);
        area
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the popup surface, the empty state and the visible rows"
    )]
    fn draw_popup(&self, ui: &mut Ui<'_>, st: &SelectState, items: &[T]) {
        let ov = self.ov;
        let id = self.id;
        let row_fn = &self.row;
        let key_fn = &self.key;
        let empty = self.empty;
        let scrollbar = self.scrollbar();
        ui.layer(self.id, |ui, layer| {
            ui.with_surface(Surface::Popover, |ui| {
                let base = ov.style(
                    ui,
                    id,
                    Family::SELECT,
                    Variant::DEFAULT,
                    Part::ROW,
                    StateFlags::empty(),
                );
                ui.fill(layer, base.style);

                let list = Rect {
                    x: layer.x,
                    y: layer.y.saturating_add(1),
                    width: layer.width,
                    height: layer.height.saturating_sub(2),
                };
                if list.is_empty() {
                    return;
                }
                if items.is_empty() {
                    let e = empty.unwrap_or(EmptyState::Empty {
                        title: "No options",
                        hint: None,
                    });
                    if let Some(f) = ov.slot_for(Part::EMPTY) {
                        f(ui, list);
                    } else {
                        let inherited = ov
                            .style(
                                ui,
                                id,
                                Family::SELECT,
                                Variant::DEFAULT,
                                Part::EMPTY,
                                StateFlags::empty(),
                            )
                            .style;
                        e.draw_inherited(ui, list, 0, inherited);
                    }
                    return;
                }
                let content = scrollbar.draw(ui, list, st.core.scroll(), items.len());
                let view = ScrollRegion::view(st.core.scroll(), content, items.len());
                for (row_i, i) in view.visible_range().enumerate() {
                    let Some(item) = items.get(i) else { break };
                    let key = key_fn.key(item, i);
                    let mut flags = StateFlags::empty();
                    if st.core.cursor() == Some(key) {
                        flags |= StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE;
                    }
                    if st.value == Some(key) {
                        flags |= StateFlags::SELECTED;
                    }
                    let row = Rect {
                        x: content.x,
                        y: content
                            .y
                            .saturating_add(row_i.min(usize::from(u16::MAX)) as u16),
                        width: content.width,
                        height: 1,
                    };
                    let rest = Rect {
                        x: row.x.saturating_add(3),
                        width: row.width.saturating_sub(3),
                        ..row
                    };
                    if !rest.is_empty() {
                        let mut r =
                            RowUi::new(ui, id, Family::SELECT, Variant::DEFAULT, flags, key, rest);
                        row_fn.row(item, &mut r);
                    }
                    if flags.contains(StateFlags::SELECTED) {
                        let rs =
                            ov.style(ui, id, Family::SELECT, Variant::DEFAULT, Part::ROW, flags);
                        ui.paint_style(row, rs.style);
                    }

                    let gutter_cell = cell_at(row, row.x);
                    if let Some(f) = ov.slot_for(Part::GUTTER) {
                        f(ui, gutter_cell);
                    } else {
                        let g = ov.style(
                            ui,
                            id,
                            Family::SELECT,
                            Variant::DEFAULT,
                            Part::GUTTER,
                            flags,
                        );
                        match g.glyph {
                            Slot::Set(glyph) => {
                                ui.glyph(gutter_cell, glyph, g.style);
                            }
                            Slot::Inherit | Slot::Clear => {
                                ui.fill(gutter_cell, g.style);
                            }
                        }
                    }
                    let marker_cell = cell_at(row, row.x.saturating_add(1));
                    if let Some(f) = ov.slot_for(Part::MARKER) {
                        f(ui, marker_cell);
                    } else {
                        let ms = ov.style(
                            ui,
                            id,
                            Family::SELECT,
                            Variant::DEFAULT,
                            Part::MARKER,
                            flags,
                        );
                        let glyph = match ms.glyph {
                            Slot::Set(glyph) => Some(glyph),
                            Slot::Inherit => flags
                                .contains(StateFlags::SELECTED)
                                .then_some(GlyphRole::Chosen),
                            Slot::Clear => None,
                        };
                        match glyph {
                            Some(glyph) => {
                                ui.glyph(marker_cell, glyph, ms.style);
                            }
                            None => ui.fill(marker_cell, ms.style),
                        }
                    }
                    if !ui.is_inert() {
                        ui.register_part(id, PartRef::item(Part::ROW, key), row);
                    }
                }
            });
        });
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (24, 1),
        }
        .fit(c)
    }
}

impl<T, K, R> Bindings for Select<'_, T, K, R> {
    type Cmd = SelectCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<SelectCmd>] {
        BINDINGS
    }
}
