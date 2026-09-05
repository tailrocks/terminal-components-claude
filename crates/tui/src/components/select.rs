//! `Select` — the dropdown field and its popup layer
//! (`COMPONENT_ARCHITECTURE.md` §17.0 A7, §24 M3, §26 N1, §18.2,
//! Appendix A 4B).

use core::fmt;
use core::marker::PhantomData;

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use super::form::InheritedFormState;
use super::scroll_region::ScrollRegion;
use super::{Acc, PartStyle, SlotFn, cell_at, first_row};
use crate::action::ActionKey;
use crate::collection::{
    ByIndex, CollectionCore, DefaultRow, EmptyState, KeyFn, Reconcile, Reconciliation, RowFn, RowUi,
};
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

/// What a select reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectAction {
    /// An option was committed as the value.
    Chose(ItemKey),
    /// The popup opened.
    Opened,
    /// The popup closed without choosing.
    Closed,
}

/// The const-constructible commands of the select keymap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectCmd {
    /// Open the popup, or commit the cursor option while it is open.
    Choose,
    /// Cursor to the previous option.
    Prev,
    /// Cursor to the next option.
    Next,
    /// Cursor to the first option.
    First,
    /// Cursor to the last option.
    Last,
    /// Cursor up one popup page.
    PageUp,
    /// Cursor down one popup page.
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

/// One table for both phases of the control's life: `Enter` / `Space` open
/// the popup and, once it is open, commit the cursor option; the motion
/// chords move the **cursor**, never the value (`select::
/// arrows_move_the_cursor_not_the_value_while_closed`).
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

/// The default instantiation a form field holds (§15.1, §24 M3): options
/// are `&str` labels, keyed positionally, painted through `Display`.
///
/// `ByIndex` is forced by the form's value channel
/// (`FieldMut::Choice(&mut usize)` is positional), not a default that
/// leaked: a keyed, custom-row or non-string choice inside a form is
/// `FieldKind::Chooser` plus the owner's own popup (§24 M3‑3, M3‑4).
pub type LabelSelect<'a> = Select<'a, &'a str, ByIndex, DefaultRow>;

/// Durable state of a [`Select`]: the popup cursor, the committed value,
/// the open flag, the popup scroll and the reconcile stamp.
///
/// The value is **uncontrolled** — the documented per-component exception of
/// §13: `Select::update` takes no `&mut` value, because the value is one of
/// the items the phase call already carries and is named by its `ItemKey`.
/// Read it with [`SelectState::value`] and seed it with
/// [`SelectState::set_value`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct SelectState {
    core: CollectionCore,
    value: Option<ItemKey>,
    open: bool,
}

impl SelectState {
    /// The committed value.
    pub const fn value(&self) -> Option<ItemKey> {
        self.value
    }

    /// Set the committed value; the cursor follows it on the next `update`.
    pub fn set_value(&mut self, k: Option<ItemKey>) {
        self.value = k;
    }

    /// The popup cursor.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// Whether the popup is open.
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// The popup scroll.
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

/// A dropdown: closed it is a one-row field with a trailing indicator; open
/// it is a `Popover` layer the component sizes itself (§26 N1).
///
/// ## Construction
/// `Select::new(id)`; options are passed to each phase, never held (§21
/// item 1, §24 M3). The default instantiation `Select<'a, &'a str, ByIndex,
/// DefaultRow>` is aliased as `LabelSelect<'a>` for form fields.
///
/// ## Ownership
/// The caller owns the options (`&[T]` per phase) and a [`SelectState`]
/// (cursor, value, open flag, popup scroll). The runtime owns focus, hover,
/// press and the layer stack.
///
/// ## Configuration
/// `.key(Fn(&T) -> ItemKey)` (`ByIndex`, unstable under reorder),
/// `.row(Fn(&T, &mut RowUi))` (`DefaultRow`: `Display`),
/// `.placeholder(&str)`, `.popup_rows(u16)`
/// (`design.size.popup_max_rows`), `.empty(EmptyState)`,
/// `.read_only(bool)`, `.disabled(bool)`, `.patch`, `.patch_part`,
/// `.slot`.
///
/// ## Variants
/// `Family::SELECT`, `DEFAULT` only.
///
/// ## States
/// The field wears `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED` from
/// the runtime, `READ_ONLY` / `DISABLED` from the props, and `EXPANDED`
/// while the popup is open, and `SELECTED` when it has a value. A popup row
/// wears `FOCUSED` when it is the cursor and `SELECTED` when it is the value.
///
/// ## Actions
/// [`SelectAction`]: `Chose(k)` (`Enter` / `Space` / a click on a row),
/// `Opened`, `Closed` (Esc, an outside click, focus leaving the field, or
/// the toggle). Moving the
/// cursor — open or closed — reports nothing: the cursor is not the value.
///
/// ## Focus
/// One `Focusable` stop (`FocusableReadOnly` / `Disabled`); does not
/// swallow typing. The popup is a `Popover`, which is a **pointer barrier
/// only** (§9.1): it registers no focus stop of its own, so the field
/// remains the one stop, which is the legacy one-focus-stop contract.
/// Focus is not *retained* there, though — the popup opens with
/// [`Dismiss::ALL`], so moving focus off the field (`Tab` is runtime focus
/// policy and never reaches a component) dismisses the layer with
/// `DismissReason::FocusOut` and leaves focus on its new target (§29.8).
///
/// ## Keyboard
/// `Enter` / `Space` open the popup and commit the cursor option once it is
/// open; `↑`/`k`, `↓`/`j`, `Home`, `End`, `PgUp`, `PgDn` move the cursor;
/// `Esc` dismisses the layer, which restores the cursor to the value.
///
/// ## Mouse
/// `PartRef::of(Part::FIELD)`: a click toggles the popup.
/// `PartRef::item(Part::ROW, k)`: a press moves the cursor, a click commits
/// option `k`. `TRACK` / `THUMB` and the wheel go to the popup's
/// [`ScrollRegion`].
///
/// ## Layout
/// Closed: one row — gutter, two-cell indent, the value or the
/// placeholder, and the indicator in the last column but one. Open: the
/// layer [`Select::measured_size`] asked for, one blank pad row above and
/// below the option rows. `measure` is `(12…24, 1)`; `draw` returns the
/// field row; `0×0` registers nothing (R5).
///
/// ## Parts
/// In paint order: `FIELD` (the closed row), `GUTTER` (the focus bar, in
/// the closed field's first column and in each popup row's), `LABEL` (the
/// value, through [`RowUi`]), `PLACEHOLDER` (what the closed field shows
/// while there is no value), `MARKER` (the indicator and the popup's chosen
/// mark), `ROW` (a popup row), `TRACK` / `THUMB` (the popup scrollbar),
/// `EMPTY` (no options).
///
/// ## Overrides
/// `.patch` and `.patch_part` on any part; `.slot` on exactly `GUTTER`,
/// `MARKER`, `EMPTY`, `TRACK` and `THUMB`. `GUTTER` and `MARKER` answer for
/// the closed field's two chrome columns **and** for every popup row's, so
/// one slot covers every cell this component paints under that part.
/// `TRACK` and `THUMB` are the popup's [`ScrollRegion`], which paints them
/// under this select's own `Id`; all three overrides are forwarded to it.
/// `FIELD`, `ROW`, `LABEL` and `PLACEHOLDER` are not slot-addressable:
/// `FIELD` and `ROW` are surface fills, `LABEL` is painted by the `.row(…)`
/// painter the caller already supplies — in the closed field and in every
/// popup row alike — and `PLACEHOLDER` is `.placeholder(…)`'s own text in
/// the cell `FIELD` owns.
///
/// ## Identity
/// `.key` supplies stable keys; `ByIndex` is unstable under
/// insert/remove/reorder. The popup layer is owned by the select's own
/// `Id`, so its lifecycle events arrive as this component's intents.
///
/// ## Testing
/// `SelectCase` with `ACTIVATES | FOCUSABLE | DISABLEABLE | OVERLAY`;
/// `render::components::select::*`;
/// `select::escape_closes_and_restores_the_cursor`,
/// `select::arrows_move_the_cursor_not_the_value_while_closed`,
/// `select::standalone_select_takes_items_per_phase`.
///
/// ## Invariants
/// `reconcile` runs before any action is emitted, and is skipped entirely
/// while `disabled`, which reports no action of its own — the one action a
/// disabled select can still emit is the `Closed` of a layer it opened
/// before it was disabled, and that carries no item; the cursor is separate
/// from the value and Esc restores it; the component re-asserts its layer's
/// size and anchor every `update` (invariant D1, §26 N1), so a changing
/// option list corrects the popup on the next draw without the opener
/// predicting anything.
pub struct Select<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    placeholder: Option<&'a str>,
    popup_rows: Option<u16>,
    empty: Option<EmptyState<'a>>,
    read_only: bool,
    disabled: bool,
    /// Kept beside `ov` so the popup's [`ScrollRegion`] can be built with the
    /// caller's own overrides: it paints `TRACK` and `THUMB` under *this*
    /// select's `Id`, so a bare construction dropped the caller's `.patch`
    /// and `.patch_part` on those parts where `Invariant P` could not see it
    /// (§45.7 obligation 2).
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
    /// A select keyed by index and painted through `Display`.
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

    pub(crate) fn update_in_form(
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

    pub(crate) fn draw_in_form(
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
}

impl<'a, T, K, R> Select<'a, T, K, R> {
    /// The parts this component styles.
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

    /// The id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// A stable key accessor.
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

    /// A row painter for the popup and for the closed value.
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

    /// Text shown while there is no value.
    #[must_use]
    pub const fn placeholder(mut self, s: &'a str) -> Self {
        self.placeholder = Some(s);
        self
    }

    /// The tallest the popup's option list may be.
    #[must_use]
    pub const fn popup_rows(mut self, n: u16) -> Self {
        self.popup_rows = Some(n);
        self
    }

    /// What the popup paints when there are no options.
    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    /// Read-only: stays in the ring, never opens.
    #[must_use]
    pub const fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Disabled: registered, never reachable.
    #[must_use]
    pub const fn disabled(mut self, yes: bool) -> Self {
        self.disabled = yes;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.patch = Some(p);
        self.ov = self.ov.global(p);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.parts = ps;
        self.ov = self.ov.part(ps);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    /// The popup's scrollbar, wearing this select's own overrides.
    ///
    /// It paints `TRACK` and `THUMB` under the **select's** `Id`, so those
    /// two parts' `.patch`, `.patch_part` and `.slot` are the select's to
    /// forward; constructing it bare dropped all three (§45.1).
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
    const fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    /// Rows the popup's option list may occupy.
    fn max_rows(&self, d: &crate::theme::DesignTokens) -> u16 {
        self.popup_rows.unwrap_or(d.size.popup_max_rows).max(1)
    }

    /// The anchor the popup wants: below the field, aligned to its left
    /// edge. The resolver flips and clamps it (§9.1); no component computes
    /// a screen rect.
    fn anchor(&self, cx: &Cx<'_>) -> Anchor {
        match cx.area(self.id) {
            Some(rect) => Anchor::Rect {
                rect,
                side: Side::Below,
                align: CrossAlign::Start,
            },
            // frame 1: the field has no geometry yet, so the popup centres
            // itself and corrects on the next frame (S3)
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

    /// The size this popup asks its layer for (§26 N1): the field's width
    /// clamped into the design's popup band, and one row per option up to
    /// `popup_rows`, plus the pad row above and below.
    ///
    /// Pure in `(props, last frame's field width, DesignTokens, items)` and
    /// re-asserted every `update`, so a changing option list corrects the
    /// layer on the next draw.
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

    /// The layer this popup wants. Call it at the moment of opening —
    /// `cx.open_layer(id, select().layer(cx, items))` — and let
    /// [`Select::update`] re-assert it every frame (§26 N1, invariant D1).
    /// The popup opens with [`Dismiss::ALL`] (§29.8 D1): a `Popover` is a
    /// pointer barrier only, so focus-out dismissal — not a focus trap and
    /// not key swallowing — is what keeps focus off a control behind the
    /// open popup.
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
        // the cursor is not the value (§15): moving it repaints and reports
        // nothing, open or closed
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

    /// Close the popup and restore the cursor to the committed value — the
    /// Esc contract (`select::escape_closes_and_restores_the_cursor`).
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

    /// Reconcile the durable state against `items` and seed the cursor on
    /// the value, or on the first option when there is none.
    ///
    /// Skipped entirely while `disabled` (§16.2 case 1): a disabled control
    /// is registered but inert, and must not initialise the caller's state.
    /// It is a skip, not a deferral — [`Reconcile::reconcile`] is stamped on
    /// the current `(len, keys)`, so however many disabled frames pass, the
    /// first enabled one reconciles the list as it then is.
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

    /// The update phase: reconcile when enabled, re-assert the layer, then
    /// drain keys, pointer intents and the layer's lifecycle events.
    ///
    /// The body runs **unconditionally**, whether or not the popup is open
    /// (§13, §28 P3): a dismissal is delivered in the pass after the layer
    /// closed, and a gated call would drain nothing. Only the durable-state
    /// half — the reconcile and the cursor seed — is gated on `disabled`
    /// (§16.2 case 1), because a disabled control must not mutate the
    /// caller's state; the intent drain, the layer bookkeeping and the
    /// dismissal that closes a popup opened before the control was disabled
    /// all still run.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut SelectState,
        items: &[T],
    ) -> Response<SelectAction> {
        let len = items.len();
        self.reconcile_and_seed(st, items);
        let open = st.open && cx.is_open(self.id);
        if open {
            // invariant D1: re-assert the geometry every frame
            let size = self.measured_size(cx, items);
            let anchor = self.anchor(cx);
            cx.resize_layer(self.id, size);
            cx.reanchor_layer(self.id, anchor);
        } else if st.open {
            // the layer went away without a dismissal reaching us
            st.open = false;
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
                    self.close_restoring(st, items);
                    acc.action(SelectAction::Closed);
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

    /// The live state flags of the closed field: the frame's own flags plus
    /// the ones this instance forces.
    fn live_flags(&self, ui: &Ui<'_>, st: &SelectState) -> StateFlags {
        // runtime: the frame's own focus/hover/press; derived: the caller's
        // `.read_only` and `.disabled` props and the caller-owned `st.open`
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

    /// Registers the closed field as this component's one control.
    ///
    /// A reference rendering registers nothing.
    fn register_field(&self, ui: &mut Ui<'_>, area: Rect) {
        if ui.is_inert() {
            return;
        }
        let f = if self.disabled {
            Focusability::Disabled
        } else if self.read_only {
            Focusability::FocusableReadOnly
        } else {
            Focusability::Focusable
        };
        ui.register_control(self.id, area, f);
    }

    /// Paints the gutter column of the closed field.
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

    /// Paints the value cell: the chosen option's row body, or the
    /// placeholder when nothing is chosen.
    fn paint_value(
        &self,
        ui: &mut Ui<'_>,
        cell: Rect,
        st: &SelectState,
        items: &[T],
        live: StateFlags,
        field: Style,
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
        // the value sits on the FIELD surface, not on a row surface:
        // `RowUi` fills with the family's `CONTAINER` style, so the
        // field's own colours are re-applied over the painted symbols
        ui.paint_style(cell, field);
    }

    /// Paints the open/closed indicator: the recipe's glyph when it has one,
    /// else the semantic select disclosure.
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

    /// The draw phase: the closed field, and the popup inside its layer.
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

    /// The popup, inside the layer this component opened and sized.
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
                // the layer is one pad row taller than the option list at
                // each end (`measured_size`), which is what makes a
                // borderless popover read as a floating surface
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
                        let _ = ov.style(
                            ui,
                            id,
                            Family::SELECT,
                            Variant::DEFAULT,
                            Part::EMPTY,
                            StateFlags::empty(),
                        );
                        e.draw(ui, list, 0);
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
                    // the popup's gutter and marker columns are the same two
                    // parts the closed field paints, so one `.slot(GUTTER, …)`
                    // or `.slot(MARKER, …)` answers for every cell this
                    // component paints under that part (§45.3, Invariant R)
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

    /// The natural size: one row, twelve columns minimum.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::RowUi;
    use crate::event::{Input, Key, KeyModifiers};
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::runtime::{App, Runtime};
    use crate::theme::{ColorLevel, Role, Theme};
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Position;

    const SEL: Id = Id::root("select.tests");
    const OTHER: Id = Id::root("select.tests.other");

    /// A page with a select and one other focus stop, driven by the real
    /// runtime — the only way to reach `Tab`, which is runtime focus policy
    /// and never becomes a component intent.
    #[derive(Default)]
    struct SelectPage {
        st: SelectState,
        items: Vec<&'static str>,
        closed: usize,
    }

    impl App for SelectPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let s: Select<'_, &str> = Select::new(SEL);
            let mut r = s.update(cx, &mut self.st, &self.items);
            if r.take_action() == Some(SelectAction::Closed) {
                self.closed = self.closed.saturating_add(1);
            }
            // the other control drains its own bucket, or the focus pair the
            // runtime addressed to it is reported as undelivered
            for _ in cx.intents(OTHER) {}
            r.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let s: Select<'_, &str> = Select::new(SEL);
            s.draw(ui, Rect::new(0, 0, 20, 1), &self.st, &self.items);
            ui.register_control(OTHER, Rect::new(0, 3, 10, 1), Focusability::Focusable);
        }
    }

    fn press(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    /// §29.8 D1: the popup opens with `Dismiss::ALL`, so the runtime closes
    /// it the moment focus leaves the field.
    ///
    /// The legacy widget held this invariant twice over — it cleared `open`
    /// whenever it drew unfocused, and it swallowed every unhandled key
    /// while open. Neither route survives: the popup is a `Popover`, which
    /// is a pointer barrier only, and `Tab` is intercepted as runtime focus
    /// policy before any intent is enqueued. Focus-out dismissal is the
    /// replacement, and it must leave focus on the `Tab` target rather than
    /// restoring it to the field.
    #[test]
    fn an_open_popup_closes_when_focus_leaves_the_field() {
        let mut st = SelectState::default();
        st.set_value(Some(ItemKey::index(0)));
        let app = SelectPage {
            st,
            items: vec!["a", "b", "c"],
            closed: 0,
        };
        let mut rt = Runtime::new(app, Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.focus(), Some(SEL));
        let _ = rt.handle(press(KeyCode::Enter));
        rt.draw_buffer(SCREEN, &mut buf);
        assert!(rt.app().st.is_open(), "Enter opened the popup");
        assert!(rt.is_open(SEL), "the popover layer is open");
        // browse away from the committed value without committing
        let _ = rt.handle(press(KeyCode::Down));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.app().st.cursor(), Some(ItemKey::index(1)));
        assert_eq!(rt.app().st.value(), Some(ItemKey::index(0)));
        assert_eq!(rt.app().closed, 0);

        let _ = rt.handle(press(KeyCode::Tab));
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.focus(), Some(OTHER), "Tab moved focus off the field");
        assert!(!rt.is_open(SEL), "the popover layer was dismissed");
        assert!(!rt.app().st.is_open(), "the component agrees it is closed");
        assert_eq!(
            rt.app().st.cursor(),
            Some(ItemKey::index(0)),
            "the cursor is restored to the committed value"
        );
        assert_eq!(rt.app().st.value(), Some(ItemKey::index(0)));
        assert_eq!(rt.app().closed, 1, "`Closed` is reported exactly once");
        assert!(
            rt.diagnostics().is_empty(),
            "no diagnostic on the dismissal pass: {:?}",
            rt.diagnostics()
        );
    }

    #[test]
    fn open_popup_paints_exactly_one_chosen_marker() {
        let mut state = SelectState::default();
        state.set_value(Some(ItemKey::index(1)));
        let app = SelectPage {
            st: state,
            items: vec!["alpha", "beta", "gamma"],
            closed: 0,
        };
        let theme = Theme::junie();
        let chosen = theme.design.glyphs.get(GlyphRole::Chosen);
        let mut runtime = Runtime::new(app, theme);
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);

        let count = (SCREEN.y..SCREEN.bottom())
            .flat_map(|y| (SCREEN.x..SCREEN.right()).map(move |x| Position::new(x, y)))
            .filter(|position| {
                buffer
                    .cell(*position)
                    .is_some_and(|cell| cell.symbol() == chosen)
            })
            .count();
        assert_eq!(count, 1);
    }

    /// §16.1: Esc dismisses the popup and puts the cursor back on the
    /// committed value, so a cancelled dropdown leaves no half-made choice
    /// behind.
    #[test]
    fn escape_closes_and_restores_the_cursor() {
        let items = ["a", "b", "c"];
        let s: Select<'_, &str> = Select::new(SEL);
        let mut st = SelectState::default();
        let _ = st.reconcile(3, |i| s.key_at(&items, i));
        st.set_value(Some(ItemKey::index(1)));
        st.core.set_cursor(1, ItemKey::index(1));
        st.open = true;
        let mut acc = Acc::<SelectAction>::new();
        s.move_cursor(&mut st, &items, 2, &mut acc);
        assert_eq!(st.cursor(), Some(ItemKey::index(2)));
        assert_eq!(
            st.value(),
            Some(ItemKey::index(1)),
            "the value is untouched"
        );
        s.close_restoring(&mut st, &items);
        assert!(!st.is_open());
        assert_eq!(
            st.cursor(),
            Some(ItemKey::index(1)),
            "Esc restores the cursor to the value"
        );
        assert_eq!(st.value(), Some(ItemKey::index(1)));
    }

    /// §16.1: while the popup is closed the motion chords move the cursor
    /// and leave the value alone — the intentional change from the legacy
    /// fused behaviour, where `↑`/`↓` on a closed select committed a new
    /// value per keystroke.
    #[test]
    fn arrows_move_the_cursor_not_the_value_while_closed() {
        let items = ["a", "b", "c"];
        let s: Select<'_, &str> = Select::new(SEL);
        let mut st = SelectState::default();
        let _ = st.reconcile(3, |i| s.key_at(&items, i));
        st.set_value(Some(ItemKey::index(0)));
        st.core.set_cursor(0, ItemKey::index(0));
        assert!(!st.is_open());
        let mut acc = Acc::<SelectAction>::new();
        s.move_cursor(&mut st, &items, 1, &mut acc);
        s.move_cursor(&mut st, &items, 2, &mut acc);
        let r = acc.finish(SEL);
        assert!(r.is_changed());
        assert_eq!(r.action_ref(), None, "no value changed, so no action");
        assert_eq!(st.cursor(), Some(ItemKey::index(2)));
        assert_eq!(st.value(), Some(ItemKey::index(0)));
    }

    /// §16.1 / §24 M3: `Select::new(id)` carries no items; a `T` that is not
    /// `&str` with `.key(..)` / `.row(..)` compiles, takes its items per
    /// phase and reconciles.
    #[test]
    fn standalone_select_takes_items_per_phase() {
        struct Engine {
            id: u64,
            name: String,
        }
        fn engine_key(e: &Engine) -> ItemKey {
            ItemKey::num(e.id)
        }
        fn engine_row(e: &Engine, u: &mut RowUi<'_>) {
            u.label(&e.name);
        }
        let engines = vec![
            Engine {
                id: 7,
                name: "postgres".to_owned(),
            },
            Engine {
                id: 9,
                name: "sqlite".to_owned(),
            },
        ];
        let key: fn(&Engine) -> ItemKey = engine_key;
        let row: fn(&Engine, &mut RowUi<'_>) = engine_row;
        let s = Select::new(SEL).key(key).row(row).placeholder("Engine");
        let mut st = SelectState::default();
        st.set_value(Some(ItemKey::num(9)));
        let _ = st.reconcile(engines.len(), |i| s.key_at(&engines, i));
        assert_eq!(st.value(), Some(ItemKey::num(9)));
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            s.draw(ui, a, &st, &engines);
        });
        let mut row0 = String::new();
        for x in 0..SCREEN.width {
            if let Some(c) = buf.cell(Position::new(x, 0)) {
                row0.push_str(c.symbol());
            }
        }
        assert!(row0.contains("sqlite"), "{row0}");
        // an item that vanishes takes the value with it
        let shrunk: Vec<Engine> = Vec::new();
        let _ = st.reconcile(shrunk.len(), |i| s.key_at(&shrunk, i));
        assert_eq!(st.value(), None);
    }

    /// A select whose `disabled` prop the test drives.
    #[derive(Default)]
    struct DisablableSelect {
        st: SelectState,
        disabled: bool,
    }

    impl App for DisablableSelect {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let items = ["alpha", "beta", "gamma"];
            let s: Select<'_, &str> = Select::new(SEL).disabled(self.disabled);
            s.update(cx, &mut self.st, &items).erase()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    /// §16.2 case 1 (`disabled_cannot_activate`): a disabled collection stays
    /// drawable from the caller's current item slice, but its update phase
    /// must not reconcile or seed durable state. The sibling contract is
    /// `choice::disabled_update_does_not_initialize_collection_state`.
    #[test]
    fn disabled_update_does_not_initialize_collection_state() {
        let app = DisablableSelect {
            st: SelectState::default(),
            disabled: true,
        };
        let mut rt = Runtime::new(app, Theme::junie());
        let _ = rt.handle(Input::Tick);
        assert_eq!(rt.app().st, SelectState::default());
    }

    /// The gate above skips work rather than deferring it: `reconcile` is
    /// stamped on `(len, keys)`, so however many disabled frames pass, the
    /// first enabled one reconciles the *current* list and seeds the cursor.
    /// A disabled select is therefore never left wedged.
    #[test]
    fn a_select_reconciles_on_the_first_enabled_frame_after_disabled_ones() {
        let app = DisablableSelect {
            st: SelectState::default(),
            disabled: true,
        };
        let mut rt = Runtime::new(app, Theme::junie());
        for _ in 0..3 {
            let _ = rt.handle(Input::Tick);
        }
        assert_eq!(
            rt.app().st,
            SelectState::default(),
            "three disabled frames accrued nothing"
        );
        rt.app_mut().disabled = false;
        let _ = rt.handle(Input::Tick);
        assert_eq!(
            rt.app().st.cursor(),
            Some(ItemKey::index(0)),
            "the first enabled frame seeds the cursor"
        );
        assert_eq!(
            rt.app().st.scroll().content_len(),
            3,
            "and the scroll length"
        );
    }

    /// Draw the closed field over the stub screen, optionally with an
    /// instance patch on `part`, and return what was painted.
    fn draw_field(part: Option<Part>) -> Buffer {
        let items = ["alpha", "beta"];
        let patch = [(
            part.unwrap_or(Part::FIELD),
            StylePatch::new().set_fg(Role::Warning),
        )];
        let mut s: Select<'_, &str> = Select::new(SEL).placeholder("Pick one");
        if part.is_some() {
            s = s.patch_part(&patch);
        }
        let st = SelectState::default();
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(SCREEN, &mut buf, |ui, a| {
            s.draw(ui, a, &st, &items);
        });
        buf
    }

    fn draw_selected(value: Option<ItemKey>) -> Buffer {
        let items = ["alpha", "beta"];
        let select: Select<'_, &str> = Select::new(SEL).placeholder("Pick one");
        let mut state = SelectState::default();
        state.set_value(value);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
            select.draw(ui, area, &state, &items);
        });
        buffer
    }

    fn disclosure(theme: Theme, open: bool, glyph: Slot<GlyphRole>) -> (String, Rect) {
        let area = Rect::new(0, 0, 20, 6);
        let patch = [(
            Part::MARKER,
            StylePatch {
                glyph,
                ..StylePatch::new()
            },
        )];
        let select: Select<'_, &str> = Select::new(SEL).patch_part(&patch);
        let state = SelectState {
            open,
            ..SelectState::default()
        };
        let mut runtime = Runtime::new(Stub::default(), theme);
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, area| {
            select.draw(ui, area, &state, &["alpha"]);
        });
        let marker = Position::new(area.right().saturating_sub(2), area.y);
        (
            buffer
                .cell(marker)
                .map_or_else(String::new, |cell| cell.symbol().to_owned()),
            runtime
                .area_of_part(SEL, PartRef::of(Part::FIELD))
                .unwrap_or(Rect::ZERO),
        )
    }

    #[test]
    fn select_disclosure_is_exact_for_both_themes_and_color_levels() {
        for theme in [Theme::junie(), Theme::paper()] {
            for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
                let theme = theme.clone().downgrade(level);
                assert_eq!(disclosure(theme.clone(), false, Slot::Inherit).0, "▾");
                assert_eq!(disclosure(theme, true, Slot::Inherit).0, "▴");
            }
        }
    }

    #[test]
    fn select_disclosure_set_inherit_and_clear_win_in_both_states() {
        for open in [false, true] {
            assert_eq!(
                disclosure(Theme::junie(), open, Slot::Set(GlyphRole::WarningMark)).0,
                Theme::junie().design.glyphs.get(GlyphRole::WarningMark)
            );
            assert_eq!(
                disclosure(Theme::junie(), open, Slot::Inherit).0,
                if open { "▴" } else { "▾" }
            );
            assert_eq!(disclosure(Theme::junie(), open, Slot::Clear).0, " ");
        }
    }

    #[test]
    fn select_pressed_mono_brackets_preserve_field_geometry() {
        let area = Rect::new(0, 0, 20, 6);
        let select: Select<'_, &str> = Select::new(SEL);
        let state = SelectState::default();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, area| {
            ui.reference(
                Some(crate::ReferenceTarget::new(
                    SEL,
                    crate::ReferenceState::FOCUSED | crate::ReferenceState::PRESSED,
                )),
                |ui| select.draw(ui, area, &state, &["alpha"]),
            );
        });

        assert_eq!(
            buffer
                .cell(Position::new(0, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("[")
        );
        assert_eq!(
            buffer
                .cell(Position::new(area.right().saturating_sub(2), 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("]")
        );
        assert_eq!(
            disclosure(Theme::junie(), false, Slot::Inherit).1,
            Rect::new(0, 0, 20, 1)
        );
    }

    #[test]
    fn selected_painting_comes_only_from_the_semantic_value() {
        assert_ne!(draw_selected(Some(ItemKey::index(1))), draw_selected(None));
    }

    /// §16.2 registry: every part a drawn select *styles* must be in
    /// [`Select::PARTS`], or a per-part patch reaches a part the component
    /// never declared. The closed field resolves `GUTTER` on every frame and
    /// `PLACEHOLDER` on every valueless one; the patch changing the render is
    /// how the conformance check observes that.
    #[test]
    fn the_parts_a_drawn_select_styles_are_declared() {
        let plain = draw_field(None);
        for part in [Part::GUTTER, Part::PLACEHOLDER] {
            assert_ne!(
                draw_field(Some(part)),
                plain,
                "a drawn Select does not style {part:?}"
            );
            assert!(
                <Select<'_, &str>>::PARTS.contains(&part),
                "Select styles {part:?} but PARTS omits it: {:?}",
                <Select<'_, &str>>::PARTS
            );
        }
    }
}
