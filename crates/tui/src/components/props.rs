//! `Props` — a two-column label / value list (`COMPONENT_ARCHITECTURE.md`
//! §12.4, §17.0 A7).

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
use crate::text::width;
use crate::theme::{Family, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};

/// What an interactive property surface reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropsAction {
    /// Copy the caller-owned value identified by this row key.
    Copy(ItemKey),
}

/// The const-constructible commands of [`PropsList`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropsCmd {
    /// Move to the previous row.
    Up,
    /// Move to the next row.
    Down,
    /// Move up one viewport.
    PageUp,
    /// Move down one viewport.
    PageDown,
    /// Move to the first row.
    Home,
    /// Move to the last row.
    End,
    /// Copy the current row when it is copyable.
    Copy,
}

const fn binding(
    action: &'static str,
    chord: Chord,
    cmd: PropsCmd,
    label: &'static str,
    visible: bool,
) -> Binding<PropsCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const PROPS_BINDINGS: [Binding<PropsCmd>; 12] = [
    binding(
        "props.up",
        Chord::key(KeyCode::Up),
        PropsCmd::Up,
        "Up",
        true,
    ),
    binding(
        "props.down",
        Chord::key(KeyCode::Down),
        PropsCmd::Down,
        "Down",
        true,
    ),
    binding(
        "props.up-vim",
        Chord::key(KeyCode::Char('k')),
        PropsCmd::Up,
        "Up",
        false,
    ),
    binding(
        "props.down-vim",
        Chord::key(KeyCode::Char('j')),
        PropsCmd::Down,
        "Down",
        false,
    ),
    binding(
        "props.page-up",
        Chord::key(KeyCode::PageUp),
        PropsCmd::PageUp,
        "Page up",
        false,
    ),
    binding(
        "props.page-down",
        Chord::key(KeyCode::PageDown),
        PropsCmd::PageDown,
        "Page down",
        false,
    ),
    binding(
        "props.home",
        Chord::key(KeyCode::Home),
        PropsCmd::Home,
        "First",
        false,
    ),
    binding(
        "props.home-vim",
        Chord::key(KeyCode::Char('g')),
        PropsCmd::Home,
        "First",
        false,
    ),
    binding(
        "props.end",
        Chord::key(KeyCode::End),
        PropsCmd::End,
        "Last",
        false,
    ),
    binding(
        "props.end-vim",
        Chord::key(KeyCode::Char('G')),
        PropsCmd::End,
        "Last",
        false,
    ),
    binding(
        "props.copy",
        Chord::key(KeyCode::Char('y')),
        PropsCmd::Copy,
        "Copy",
        true,
    ),
    binding(
        "props.copy-enter",
        Chord::key(KeyCode::Enter),
        PropsCmd::Copy,
        "Copy",
        false,
    ),
];

/// Caller-owned interaction state for [`PropsList`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PropsState {
    core: CollectionCore,
}

impl PropsState {
    /// The current row key.
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    /// The current row index from the last reconciliation.
    pub const fn cursor_index(&self) -> usize {
        self.core.cursor_index()
    }

    /// The vertical scroll state.
    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    /// Point the cursor at a keyed row.
    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for PropsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        self.core.reconcile(len, key)
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
    }
}

/// A keyed, scrollable two-column list over caller-borrowed items.
///
/// Items are passed to each phase. `.key` supplies durable identity and
/// `.row` paints the two columns through [`RowUi::label`] and
/// [`RowUi::meta`]. `.label_width` is the caller-supplied width of the label
/// column, normally the widest emitted label; the component adds the design's
/// two-cell gap before values. If omitted, labels use the full available row
/// after the gutter and no value column is guessed. `.copyable_item` is only
/// an authorization predicate;
/// [`PropsAction::Copy`] carries a key, never the value. Secret owners must
/// paint through [`crate::Secret::write_mask`] and leave that predicate false.
pub struct PropsList<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    copyable_item: Option<&'a dyn Fn(&T) -> bool>,
    label_width: Option<u16>,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: Overrides<'a>,
    _item: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for PropsList<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropsList")
            .field("id", &self.id)
            .field("copyable_item", &self.copyable_item.is_some())
            .finish_non_exhaustive()
    }
}

impl<T> PropsList<'_, T, ByIndex, DefaultRow> {
    /// A property list keyed by index and painted through `Display`.
    pub const fn new(id: Id) -> Self {
        PropsList {
            id,
            key: ByIndex,
            row: DefaultRow,
            copyable_item: None,
            label_width: None,
            patch: None,
            parts: &[],
            ov: Overrides::new(),
            _item: PhantomData,
        }
    }
}

impl<'a, T, K, R> PropsList<'a, T, K, R> {
    /// The parts this component styles or registers.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::GUTTER,
        Part::LABEL,
        Part::META,
        Part::TRACK,
        Part::THUMB,
    ];

    /// A stable key accessor.
    pub fn key<K2: Fn(&T) -> ItemKey>(self, key: K2) -> PropsList<'a, T, K2, R> {
        PropsList {
            id: self.id,
            key,
            row: self.row,
            copyable_item: self.copyable_item,
            label_width: self.label_width,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }

    /// A two-column row painter.
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, row: R2) -> PropsList<'a, T, K, R2> {
        PropsList {
            id: self.id,
            key: self.key,
            row,
            copyable_item: self.copyable_item,
            label_width: self.label_width,
            patch: self.patch,
            parts: self.parts,
            ov: self.ov,
            _item: PhantomData,
        }
    }

    /// Set the stable label-column width in terminal cells.
    ///
    /// Pass the widest label width for the rows supplied to both phases. The
    /// component never invokes the row renderer for measurement, so this
    /// keeps side effects single-shot and column positions stable while
    /// scrolling. Without this builder, the fallback is a label-only row.
    #[must_use]
    pub const fn label_width(mut self, width: u16) -> Self {
        self.label_width = Some(width);
        self
    }

    /// Authorize copy for selected items. Secret-bearing rows must return
    /// `false`; the component never reads or returns item values.
    #[must_use]
    pub fn copyable_item(mut self, copyable: &'a dyn Fn(&T) -> bool) -> Self {
        self.copyable_item = Some(copyable);
        self
    }

    /// Apply an instance patch to every part.
    #[must_use]
    pub fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.patch(patch);
        self
    }

    /// Apply instance patches to named parts.
    #[must_use]
    pub fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self.ov = self.ov.patch_part(patches);
        self
    }

    /// Replace `GUTTER`, `TRACK`, or `THUMB` painting without changing
    /// layout or hit regions. A `GUTTER` slot owns the reserved cell and
    /// bypasses the default `GUTTER` style; that part's patch applies only
    /// when the default painter is used.
    #[must_use]
    pub fn slot(mut self, part: Part, painter: SlotFn<'a>) -> Self {
        if matches!(part, Part::GUTTER | Part::TRACK | Part::THUMB) {
            self.ov = self.ov.slot(part, painter);
        }
        self
    }

    fn scrollbar(&self) -> ScrollRegion<'a> {
        let mut scrollbar = ScrollRegion::new(self.id)
            .inherit_family(Family::PROPS)
            .patch_part(self.parts);
        if let Some(patch) = self.patch {
            scrollbar = scrollbar.patch(patch);
        }
        if let Some(painter) = self.ov.slot_for(Part::TRACK) {
            scrollbar = scrollbar.slot(Part::TRACK, painter);
        }
        if let Some(painter) = self.ov.slot_for(Part::THUMB) {
            scrollbar = scrollbar.slot(Part::THUMB, painter);
        }
        scrollbar
    }

    fn can_copy(&self, item: &T) -> bool {
        self.copyable_item.is_some_and(|copyable| copyable(item))
    }
}

#[derive(Clone, Copy)]
struct PaintRow<'a> {
    id: Id,
    flags: StateFlags,
    key: ItemKey,
    show_gutter: bool,
    gutter: Option<SlotFn<'a>>,
    label_width: u16,
    fill_container: bool,
    ellipsis: bool,
    ov: Overrides<'a>,
}

fn paint_two_columns<T, R: RowFn<T>>(
    ui: &mut Ui<'_>,
    area: Rect,
    item: &T,
    row: &R,
    paint: PaintRow<'_>,
) {
    {
        let mut row_ui = RowUi::new_props(
            ui,
            paint.id,
            paint.flags,
            paint.key,
            area,
            paint.label_width,
            paint.ov.part_patch(Part::CONTAINER),
            paint.ov.part_patch(Part::GUTTER),
            paint.ov.part_patch(Part::META),
            paint.ov.part_patch(Part::LABEL),
            paint.fill_container,
            paint.ellipsis,
        );
        if paint.show_gutter {
            if paint.gutter.is_some() {
                row_ui.reserve_gutter();
            } else {
                row_ui.gutter();
            }
        }
        row.row(item, &mut row_ui);
    }
    if paint.show_gutter
        && let Some(gutter) = paint.gutter
    {
        gutter(ui, cell_at(area, area.x));
    }
}

fn row_flags(
    live: StateFlags,
    key: ItemKey,
    cursor: Option<ItemKey>,
    hovered: Option<PartRef>,
    pressed: Option<PartRef>,
) -> StateFlags {
    let row_part = PartRef::item(Part::ROW, key);
    let is_cursor = cursor == Some(key);
    let mut flags = StateFlags::empty();
    if is_cursor {
        flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
    }
    if hovered == Some(row_part) {
        flags |= StateFlags::HOVERED;
    }
    if pressed == Some(row_part)
        || (pressed.is_none() && is_cursor && live.contains(StateFlags::PRESSED))
    {
        flags |= StateFlags::PRESSED;
    }
    flags
}

impl<T, K: KeyFn<T>, R: RowFn<T>> PropsList<'_, T, K, R> {
    fn key_at(&self, items: &[T], index: usize) -> ItemKey {
        items
            .get(index)
            .map_or(ItemKey::index(index), |item| self.key.key(item, index))
    }

    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(index) = hint
            && index < items.len()
            && self.key_at(items, index) == key
        {
            return Some(index);
        }
        (0..items.len()).find(|&index| self.key_at(items, index) == key)
    }

    fn move_cursor(
        &self,
        st: &mut PropsState,
        items: &[T],
        target: usize,
        acc: &mut Acc<PropsAction>,
    ) {
        let Some(index) = (!items.is_empty()).then(|| target.min(items.len().saturating_sub(1)))
        else {
            acc.consumed();
            return;
        };
        let key = self.key_at(items, index);
        if st.core.cursor() == Some(key) {
            acc.consumed();
            return;
        }
        st.core.set_cursor(index, key);
        acc.changed();
    }

    fn copy_cursor(&self, st: &PropsState, items: &[T], acc: &mut Acc<PropsAction>) {
        let Some(key) = st.core.cursor() else {
            acc.consumed();
            return;
        };
        let Some(index) = self.index_of(items, key, Some(st.core.cursor_index())) else {
            acc.consumed();
            return;
        };
        if items.get(index).is_some_and(|item| self.can_copy(item)) {
            acc.action(PropsAction::Copy(key));
        } else {
            acc.consumed();
        }
    }

    /// Reconcile keyed rows, then drain navigation, copy, pointer, wheel and
    /// scrollbar intents.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut PropsState,
        items: &[T],
    ) -> Response<PropsAction> {
        let _ = st
            .core
            .reconcile(items.len(), |index| self.key_at(items, index));
        if st.core.cursor().is_none() && !items.is_empty() {
            st.core.set_cursor(0, self.key_at(items, 0));
        }
        let mut acc = Acc::new();
        let scroll = self
            .scrollbar()
            .update(cx, st.core.scroll_mut(), items.len());
        acc.fold(&scroll);
        let viewport = st.core.scroll().viewport_len().max(1);
        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(action) => match Binding::command(&PROPS_BINDINGS, action) {
                    Some(PropsCmd::Up) => self.move_cursor(
                        st,
                        items,
                        st.core.cursor_index().saturating_sub(1),
                        &mut acc,
                    ),
                    Some(PropsCmd::Down) => self.move_cursor(
                        st,
                        items,
                        st.core.cursor_index().saturating_add(1),
                        &mut acc,
                    ),
                    Some(PropsCmd::PageUp) => self.move_cursor(
                        st,
                        items,
                        st.core.cursor_index().saturating_sub(viewport),
                        &mut acc,
                    ),
                    Some(PropsCmd::PageDown) => self.move_cursor(
                        st,
                        items,
                        st.core.cursor_index().saturating_add(viewport),
                        &mut acc,
                    ),
                    Some(PropsCmd::Home) => self.move_cursor(st, items, 0, &mut acc),
                    Some(PropsCmd::End) => self.move_cursor(st, items, usize::MAX, &mut acc),
                    Some(PropsCmd::Copy) => self.copy_cursor(st, items, &mut acc),
                    None => {}
                },
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(key),
                        },
                    ..
                } => {
                    let hint = self.index_of(items, key, Some(st.core.cursor_index()));
                    let Some(index) = hint else {
                        acc.consumed();
                        continue;
                    };
                    match phase {
                        Phase::Press => self.move_cursor(st, items, index, &mut acc),
                        Phase::Click | Phase::DoubleClick => {
                            if st.core.cursor() != Some(key) {
                                st.core.set_cursor(index, key);
                                acc.changed();
                            }
                            self.copy_cursor(st, items, &mut acc);
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

    /// Draw only visible rows; the row renderer is never invoked offscreen.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &PropsState, items: &[T]) -> Rect {
        if area.is_empty() {
            return area;
        }
        if !ui.is_inert() {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let live = Overrides::flags(ui.state(self.id), StateFlags::empty());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, &PROPS_BINDINGS);
        }
        let root = self.ov.style(
            ui,
            self.id,
            Family::PROPS,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED),
        );
        ui.fill(area, root.style);
        let content = self
            .scrollbar()
            .draw(ui, area, st.core.scroll(), items.len());
        let view = ScrollRegion::view(st.core.scroll(), content, items.len());
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        // The caller supplies the widest label width when values are present.
        // No renderer pass guesses it. The fallback deliberately preserves a
        // label-only row instead of inventing a column split from the viewport.
        let label_width = self
            .label_width
            .unwrap_or_else(|| content.width.saturating_sub(1));
        for (visible, index) in view.visible_range().enumerate() {
            let Some(item) = items.get(index) else { break };
            let key = self.key.key(item, index);
            let row_part = PartRef::item(Part::ROW, key);
            let flags = row_flags(live, key, st.core.cursor(), hovered, pressed);
            let row_area = Rect {
                x: content.x,
                y: content
                    .y
                    .saturating_add(visible.min(usize::from(u16::MAX)) as u16),
                width: content.width,
                height: 1,
            };
            paint_two_columns(
                ui,
                row_area,
                item,
                &self.row,
                PaintRow {
                    id: self.id,
                    flags,
                    key,
                    show_gutter: true,
                    gutter: self.ov.slot_for(Part::GUTTER),
                    label_width,
                    fill_container: true,
                    ellipsis: true,
                    ov: self.ov,
                },
            );
            if !ui.is_inert() {
                ui.register_part(self.id, row_part, row_area);
            }
        }
        area
    }

    /// A conservative natural size for a property list.
    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (32, constraints.max.1),
        }
        .fit(constraints)
    }
}

impl<T, K, R> Bindings for PropsList<'_, T, K, R> {
    type Cmd = PropsCmd;

    fn bindings(&self, _state: BindingState) -> &'static [Binding<PropsCmd>] {
        &PROPS_BINDINGS
    }
}

fn paint_pair(row: &(&str, &str), ui: &mut RowUi<'_>) {
    ui.label(row.0);
    ui.meta(row.1);
}

/// Label / value rows: muted labels in a column sized to the widest, values
/// beside them. Static and interactive Props share the same painter.
///
/// ## Construction
/// `Props::new(rows)` over `&[(&str, &str)]`.
///
/// ## Ownership
/// Stateless; the rows are borrowed.
///
/// ## Configuration
/// `.patch_part`.
///
/// ## Variants
/// `Family::PROPS`, `DEFAULT` only.
///
/// ## States
/// None.
///
/// ## Actions
/// None.
///
/// ## Focus
/// Never a focus stop; registers nothing (it has no id).
///
/// ## Keyboard
/// None.
///
/// ## Mouse
/// None.
///
/// ## Layout
/// One row per pair; `measure` is `(widest label + 2 + widest value,
/// rows)`; `draw` returns the rows painted, clipped to `area`.
///
/// ## Parts
/// `META` (the label column), `LABEL` (the value column).
///
/// ## Overrides
/// `.patch_part` on both parts.
///
/// ## Identity
/// None.
///
/// ## Testing
/// `PropsCase` with no capabilities; `render::components::dialog::*` covers
/// it inside a body.
///
/// ## Invariants
/// Never writes outside `area`; never allocates; static and interactive rows
/// both pass through `paint_two_columns`.
pub struct Props<'a> {
    rows: &'a [(&'a str, &'a str)],
    ov: Overrides<'a>,
}

impl fmt::Debug for Props<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Props")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

impl<'a> Props<'a> {
    /// The parts this component styles.
    pub const PARTS: &'static [Part] = &[Part::META, Part::LABEL];

    /// Rows of `(label, value)`.
    pub const fn new(rows: &'a [(&'a str, &'a str)]) -> Self {
        Props {
            rows,
            ov: Overrides::new(),
        }
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(ps);
        self
    }

    fn label_width(&self) -> u16 {
        self.rows.iter().map(|(k, _)| width(k)).max().unwrap_or(0)
    }

    /// The draw phase.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        let mut painted = 0u16;
        for (index, (row_area, row)) in area.rows().zip(self.rows.iter()).enumerate() {
            paint_two_columns(
                ui,
                row_area,
                row,
                &paint_pair,
                PaintRow {
                    id: Id::root("tui.props"),
                    flags: StateFlags::empty(),
                    key: ItemKey::index(index),
                    show_gutter: false,
                    gutter: None,
                    label_width: self.label_width().min(area.width),
                    fill_container: false,
                    ellipsis: false,
                    ov: self.ov,
                },
            );
            painted = painted.saturating_add(1);
        }
        Rect {
            height: painted,
            ..area
        }
    }

    /// The natural size.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let vw = self.rows.iter().map(|(_, v)| width(v)).max().unwrap_or(0);
        let w = self.label_width().saturating_add(2).saturating_add(vw);
        let h = self.rows.len().min(usize::from(u16::MAX)) as u16;
        Size::exact(w, h).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::{Buffer, Cell as BufferCell};
    use ratatui_core::layout::Position;
    use ratatui_core::style::Color;

    use super::*;
    use crate::runtime::stub::{Stub, key};
    use crate::runtime::{App, Runtime};
    use crate::secret::{Secret, SecretPolicy};
    use crate::theme::{Role, Theme};
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("props.list.tests");
    const AREA: Rect = Rect::new(0, 0, 24, 3);

    #[derive(Clone, Copy)]
    struct Fact {
        key: ItemKey,
        label: &'static str,
        value: &'static str,
        copyable: bool,
    }

    const FACTS: [Fact; 4] = [
        Fact {
            key: ItemKey::num(1),
            label: "One",
            value: "first",
            copyable: true,
        },
        Fact {
            key: ItemKey::num(2),
            label: "Two",
            value: "second",
            copyable: true,
        },
        Fact {
            key: ItemKey::num(3),
            label: "Three",
            value: "third",
            copyable: false,
        },
        Fact {
            key: ItemKey::num(4),
            label: "Four",
            value: "fourth",
            copyable: true,
        },
    ];

    const REORDERED: [Fact; 4] = [FACTS[0], FACTS[2], FACTS[1], FACTS[3]];

    fn fact_key(fact: &Fact) -> ItemKey {
        fact.key
    }

    fn fact_row(fact: &Fact, row: &mut RowUi<'_>) {
        row.label(fact.label);
        row.meta(fact.value);
    }

    fn fact_copyable(fact: &Fact) -> bool {
        fact.copyable
    }

    type FactList<'a> = PropsList<'a, Fact, fn(&Fact) -> ItemKey, fn(&Fact, &mut RowUi<'_>)>;

    fn fact_list() -> FactList<'static> {
        PropsList::new(ID)
            .key(fact_key as fn(&Fact) -> ItemKey)
            .row(fact_row as fn(&Fact, &mut RowUi<'_>))
            .label_width(5)
            .copyable_item(&fact_copyable)
    }

    #[derive(Default)]
    struct FactApp {
        state: PropsState,
        reordered: bool,
        copied: Vec<ItemKey>,
    }

    impl FactApp {
        fn facts(&self) -> &'static [Fact] {
            if self.reordered { &REORDERED } else { &FACTS }
        }
    }

    impl App for FactApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let facts = if self.reordered { &REORDERED } else { &FACTS };
            let response = fact_list().update(cx, &mut self.state, facts);
            if let Some(PropsAction::Copy(key)) = response.action_ref() {
                self.copied.push(*key);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            fact_list().draw(ui, AREA, &self.state, self.facts());
        }
    }

    #[test]
    fn copy_resolves_the_cursor_by_stable_key_after_same_length_reorder() {
        let mut runtime = Runtime::new(FactApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Down));
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(2)));

        runtime.app_mut().reordered = true;
        let _ = runtime.handle(key(KeyCode::Char('y')));
        assert_eq!(runtime.app().copied, [ItemKey::num(2)]);
    }

    #[test]
    fn reconcile_tracks_a_key_after_an_in_place_reorder_is_invalidated() {
        let mut state = PropsState::default();
        state.set_cursor(1, ItemKey::num(2));
        let _ = state.reconcile(FACTS.len(), |index| FACTS[index].key);
        state.invalidate();
        assert_eq!(
            state.reconcile(REORDERED.len(), |index| REORDERED[index].key),
            Reconciliation::Unchanged
        );
        assert_eq!(state.cursor(), Some(ItemKey::num(2)));
        assert_eq!(state.cursor_index(), 2);
    }

    #[test]
    fn noncopyable_item_consumes_copy_without_an_action() {
        let mut runtime = Runtime::new(FactApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Down));
        let _ = runtime.handle(key(KeyCode::Down));
        let response = runtime.handle(key(KeyCode::Char('y')));
        assert!(response.is_consumed());
        assert!(runtime.app().copied.is_empty());
    }

    #[test]
    fn only_the_cursor_row_receives_focus_state() {
        let theme = Theme::junie().define_family(Family::PROPS, |recipe| {
            recipe.part(Part::META).when(
                StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
                StylePatch::new().set_bg(Role::Danger),
            );
        });
        let mut state = PropsState::default();
        state.set_cursor(1, ItemKey::num(2));
        let mut runtime = Runtime::new(Stub::default(), theme.clone());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            ui.reference(
                Some(ReferenceTarget::new(
                    ID,
                    ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE,
                )),
                |ui| {
                    fact_list().draw(ui, area, &state, &FACTS);
                },
            );
        });

        assert_ne!(
            buffer.cell(Position::new(1, 0)).map(|cell| cell.bg),
            Some(theme.color.danger)
        );
        assert_eq!(
            buffer.cell(Position::new(1, 1)).map(|cell| cell.bg),
            Some(theme.color.danger)
        );
    }

    struct SecretFact {
        secret: Secret,
    }

    fn secret_row(fact: &SecretFact, row: &mut RowUi<'_>) {
        row.label("Token");
        let mut value = row.part(Part::LABEL, 12);
        fact.secret
            .write_mask(&mut value, fact.secret.len(), SecretPolicy::default());
    }

    fn never_copy(_fact: &SecretFact) -> bool {
        false
    }

    #[test]
    fn secret_renderer_masks_and_copy_authorization_stays_false() {
        let facts = [SecretFact {
            secret: Secret::new("hunter2".to_owned()),
        }];
        let list = PropsList::new(ID)
            .row(secret_row as fn(&SecretFact, &mut RowUi<'_>))
            .label_width(5)
            .copyable_item(&never_copy);
        let mut state = PropsState::default();
        state.set_cursor(0, ItemKey::index(0));
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            list.draw(ui, area, &state, &facts);
        });
        let text: String = AREA
            .positions()
            .filter_map(|position| buffer.cell(position))
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(!text.contains("hunter2"));

        let mut acc = Acc::new();
        list.copy_cursor(&state, &facts, &mut acc);
        assert!(acc.finish(ID).action_ref().is_none());
    }

    #[test]
    fn draw_invokes_only_visible_rows() {
        let calls = Cell::new(0usize);
        let row = |_: &usize, _: &mut RowUi<'_>| calls.set(calls.get().saturating_add(1));
        let items = [0usize; 100];
        let state = PropsState::default();
        let area = Rect::new(0, 0, 20, 2);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, area| {
            PropsList::new(ID).row(row).draw(ui, area, &state, &items);
        });
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn static_props_preserves_bdfda5d_frame_bytes_and_styles() {
        const STATIC_AREA: Rect = Rect::new(0, 0, 16, 2);
        let rows = [("A", "one"), ("Longest", "two")];
        let patches = [
            (Part::META, StylePatch::new().set_fg(Role::Danger)),
            (Part::LABEL, StylePatch::new().set_bg(Role::Info)),
        ];
        let theme = Theme::junie();
        let mut runtime = Runtime::new(Stub::default(), theme.clone());
        let mut sentinel = BufferCell::EMPTY;
        sentinel.bg = Color::Rgb(1, 2, 3);
        let mut buffer = Buffer::filled(STATIC_AREA, sentinel);
        runtime.draw_scene(STATIC_AREA, &mut buffer, |ui, area| {
            Props::new(&rows).patch_part(&patches).draw(ui, area);
        });

        let line = |y| {
            (0..STATIC_AREA.width)
                .filter_map(|x| buffer.cell(Position::new(x, y)))
                .map(ratatui_core::buffer::Cell::symbol)
                .collect::<String>()
        };
        assert_eq!(line(0), "A        one    ");
        assert_eq!(line(1), "Longest  two    ");
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(|cell| cell.fg),
            Some(theme.color.danger)
        );
        assert_eq!(
            buffer.cell(Position::new(9, 0)).map(|cell| cell.bg),
            Some(theme.color.info)
        );
        assert_eq!(
            buffer.cell(Position::new(4, 0)).map(|cell| cell.bg),
            Some(Color::Rgb(1, 2, 3)),
            "static Props must leave the label/value gap untouched"
        );
        assert_eq!(
            buffer.cell(Position::new(15, 1)).map(|cell| cell.bg),
            Some(Color::Rgb(1, 2, 3)),
            "static Props must leave trailing cells untouched"
        );
    }

    #[test]
    fn static_props_clips_narrow_labels_without_row_ui_ellipsis() {
        const TINY: Rect = Rect::new(0, 0, 3, 1);
        let rows = [("Long", "")];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(TINY);
        runtime.draw_scene(TINY, &mut buffer, |ui, area| {
            Props::new(&rows).draw(ui, area);
        });
        let line = (0..TINY.width)
            .filter_map(|x| buffer.cell(Position::new(x, 0)))
            .map(BufferCell::symbol)
            .collect::<String>();
        assert_eq!(line, "Lon");
    }

    #[test]
    fn generic_props_rows_use_the_explicit_widest_label_column() {
        let state = PropsState::default();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            fact_list().draw(ui, area, &state, &FACTS[..3]);
        });

        assert_eq!(
            (8..13)
                .filter_map(|x| buffer.cell(Position::new(x, 0)))
                .map(ratatui_core::buffer::Cell::symbol)
                .collect::<String>(),
            "first"
        );
        assert_eq!(
            (8..13)
                .filter_map(|x| buffer.cell(Position::new(x, 2)))
                .map(ratatui_core::buffer::Cell::symbol)
                .collect::<String>(),
            "third",
            "the widest explicit label still leaves the value column intact"
        );
    }

    #[test]
    fn props_list_without_label_width_uses_a_stable_label_only_fallback() {
        let state = PropsState::default();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            PropsList::new(ID)
                .row(|_: &usize, row: &mut RowUi<'_>| {
                    row.label("Label");
                    row.meta("value");
                })
                .draw(ui, area, &state, &[0]);
        });
        assert_eq!(
            buffer.cell(Position::new(1, 0)).map(BufferCell::symbol),
            Some("L")
        );
        assert!(
            AREA.positions()
                .filter_map(|position| buffer.cell(position))
                .all(|cell| cell.symbol() != "v"),
            "unset label width must not invent a value-column split"
        );
    }

    #[test]
    fn track_and_thumb_slots_are_forwarded_independently() {
        let items = [0usize; 12];
        let state = PropsState::default();
        for part in [Part::TRACK, Part::THUMB] {
            let called = Cell::new(false);
            let slot = |_: &mut Ui<'_>, _: Rect| called.set(true);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(AREA);
            runtime.draw_scene(AREA, &mut buffer, |ui, area| {
                PropsList::new(ID)
                    .slot(part, &slot)
                    .draw(ui, area, &state, &items);
            });
            assert!(called.get(), "{part:?} slot was not forwarded");
        }
    }

    #[test]
    fn gutter_slot_owns_the_reserved_cell_and_default_gutter_patch_is_forwarded() {
        let items = [0usize];
        let state = PropsState::default();
        let patches = [(Part::GUTTER, StylePatch::new().set_bg(Role::Danger))];
        let slot = |ui: &mut Ui<'_>, area: Rect| {
            ui.paint_str(area, "!", ratatui_core::style::Style::default());
        };
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            PropsList::new(ID)
                .row(|_: &usize, row: &mut RowUi<'_>| row.label("L"))
                .patch_part(&patches)
                .slot(Part::GUTTER, &slot)
                .draw(ui, area, &state, &items);
        });
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(BufferCell::symbol),
            Some("!")
        );
        assert_eq!(
            buffer.cell(Position::new(1, 0)).map(BufferCell::symbol),
            Some("L"),
            "a custom gutter still reserves one column"
        );

        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            PropsList::new(ID)
                .row(|_: &usize, row: &mut RowUi<'_>| row.label("L"))
                .patch_part(&patches)
                .draw(ui, area, &state, &items);
        });
        assert_eq!(
            buffer.cell(Position::new(0, 0)).map(|cell| cell.bg),
            Some(Theme::junie().color.danger),
            "the default gutter receives its part patch"
        );
    }
}
