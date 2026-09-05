//! `Props` — a two-column label / value list (`COMPONENT_ARCHITECTURE.md`
//! §12.4, §17.0 A7).

use core::fmt;

use super::{Acc, PartStyle, SlotFn, cell_at, paint_pressed_bracket};
use crate::collection::{CellUi, Reconcile, Reconciliation};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::Response;
use crate::response::StateFlags;
use crate::scroll::ScrollState;
use crate::secret::{Secret, SecretPolicy};
use crate::text::measure::graphemes;
use crate::text::width;
use crate::theme::{Family, GlyphRole, Role, Slot, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

/// A borrowed value displayed by an interactive property row.
///
/// `Secret` values are rendered through [`Secret::write_mask`] and can never
/// become a [`PropsAction::Copy`] action. The component does not clone or
/// expose either value variant.
#[derive(Clone, Copy)]
pub enum PropsValue<'a> {
    /// Ordinary borrowed text.
    Text(&'a str),
    /// A borrowed secret, rendered as a mask with a synthetic tail.
    Secret(&'a Secret),
}

impl fmt::Debug for PropsValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropsValue::Text(value) => f.debug_tuple("Text").field(value).finish(),
            PropsValue::Secret(_) => f.write_str("Secret([redacted])"),
        }
    }
}

impl PartialEq for PropsValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PropsValue::Text(left), PropsValue::Text(right)) => left == right,
            (PropsValue::Secret(left), PropsValue::Secret(right)) => core::ptr::eq(left, right),
            _ => false,
        }
    }
}

impl Eq for PropsValue<'_> {}

impl<'a> PropsValue<'a> {
    /// Whether this value is secret and therefore masked and non-copyable.
    pub const fn is_secret(self) -> bool {
        matches!(self, PropsValue::Secret(_))
    }

    /// The plain text, if this is not a secret value.
    pub const fn text(self) -> Option<&'a str> {
        match self {
            PropsValue::Text(value) => Some(value),
            PropsValue::Secret(_) => None,
        }
    }
}

/// One keyed, borrowed property row.
///
/// The key is the identity carried by [`PropsAction`]. `tone` is a semantic
/// [`Role`], never a concrete colour. `wrap` affects the value's visual row
/// count; `copyable` is forced off for secret values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PropsRow<'a> {
    /// Stable item identity.
    pub key: ItemKey,
    /// Borrowed label text.
    pub label: &'a str,
    /// Borrowed display value.
    pub value: PropsValue<'a>,
    /// Optional semantic foreground role for the value.
    pub tone: Option<Role>,
    /// Word-wrap the value into additional visual rows.
    pub wrap: bool,
    /// Whether keyboard or pointer copy is allowed.
    pub copyable: bool,
}

impl<'a> PropsRow<'a> {
    /// Construct a plain, non-copyable row.
    pub const fn new(key: ItemKey, label: &'a str, value: &'a str) -> Self {
        Self::value(key, label, PropsValue::Text(value))
    }

    /// Construct a row whose value is masked and never copyable.
    pub const fn secret(key: ItemKey, label: &'a str, value: &'a Secret) -> Self {
        Self::value(key, label, PropsValue::Secret(value))
    }

    /// Construct a row from an explicit borrowed value.
    pub const fn value(key: ItemKey, label: &'a str, value: PropsValue<'a>) -> Self {
        PropsRow {
            key,
            label,
            value,
            tone: None,
            wrap: false,
            copyable: false,
        }
    }

    /// Set the value's semantic foreground role.
    #[must_use]
    pub const fn tone(mut self, tone: Role) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Alias for [`Self::tone`] when the caller is emphasizing semantic role.
    #[must_use]
    pub const fn role(self, role: Role) -> Self {
        self.tone(role)
    }

    /// Enable word wrapping for the value.
    #[must_use]
    pub const fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// Set wrapping explicitly.
    #[must_use]
    pub const fn wrap_if(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    /// Allow copy for a plain value; secret rows remain non-copyable.
    #[must_use]
    pub const fn copyable(mut self) -> Self {
        self.copyable = !self.value.is_secret();
        self
    }

    /// Set copyability explicitly, still refusing secret rows.
    #[must_use]
    pub const fn copyable_if(mut self, yes: bool) -> Self {
        self.copyable = yes && !self.value.is_secret();
        self
    }

    /// Whether this row can produce a copy action.
    pub const fn can_copy(&self) -> bool {
        self.copyable && !self.value.is_secret()
    }
}

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
    core: crate::collection::CollectionCore,
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

    /// Invalidate the keyed collection after in-place row changes.
    pub fn invalidate(&mut self) {
        self.core.invalidate();
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

/// A keyed, scrollable two-column property list over borrowed rows.
///
/// ## Construction
///
/// `PropsList::new(id)`; rows are borrowed and passed to `update` and `draw`
/// for each phase. Use [`PropsRow::new`] for ordinary values and
/// [`PropsRow::secret`] for values that must remain masked and non-copyable.
///
/// ## Ownership
///
/// The caller owns rows and [`PropsState`]. The runtime owns focus, hover,
/// pressed state, pointer routing, and scrollbar capture.
///
/// ## Configuration
///
/// `.variant(Variant::DEFAULT)`, `.patch`, `.patch_part`, and `.slot`; the
/// default variant is `Variant::DEFAULT`.
///
/// ## Variants
///
/// `Family::PROPS`; `.variant` selects the theme variant and defaults to
/// `Variant::DEFAULT`.
///
/// ## States
///
/// The list wears `FOCUSED` and `FOCUS_VISIBLE`; keyed rows add `HOVERED` and
/// `PRESSED` from runtime pointer state. Cursor and scroll state live in
/// [`PropsState`]. It derives no disabled, selected, editing, or readiness
/// state.
///
/// ## Actions
///
/// Navigation changes state. Copy emits [`PropsAction::Copy`] with only the
/// row's [`ItemKey`]; the caller resolves the borrowed value and owns the
/// clipboard effect. Secret rows and non-copyable rows consume copy without
/// emitting an action.
///
/// ## Focus
///
/// One `Focusability::Focusable` stop when the list is not inert. It does not
/// swallow typing and has no autofocus or focus trap.
///
/// ## Keyboard
///
/// `Up`/`Down` and `k`/`j` move one row; `PageUp`/`PageDown`, `Home`/`g`, and
/// `End`/`G` navigate by range; `y` and `Enter` request copy of the cursor row.
///
/// ## Mouse
///
/// `PartRef::item(Part::ROW, key)` selects on press and selects plus copies on
/// click or double-click. Wheel, `TRACK`, and `THUMB` intents are handled by
/// the embedded scrollbar.
///
/// ## Layout
///
/// Labels occupy the widest label column; values use the remaining column and
/// may wrap into multiple visual rows. The scrollbar reserves its own column;
/// `measure` prefers `(32, available height)` with a minimum of `(12, 1)`, and
/// `draw` returns `area`. An empty area registers nothing.
///
/// ## Parts
///
/// `CONTAINER` (row fill), `META` (labels), `LABEL` (values), `ROW` (keyed row
/// hit regions), `TRACK`, and `THUMB` (scrollbar).
///
/// ## Overrides
///
/// `.patch` and `.patch_part` apply to named style resolutions. `.slot` is
/// honoured for `TRACK` and `THUMB`; row, label, value, and container painters
/// are not slot-addressable.
///
/// ## Identity
///
/// Rows are keyed by [`PropsRow::key`]. Keep keys stable across reorder; the
/// list has no separate positional identity.
///
/// ## Testing
///
/// Module tests cover keyed metadata, secret masking and redaction, and copy
/// through keyboard and mouse runtime intents.
///
/// ## Invariants
///
/// Secret values never appear in painted text or debug output and cannot emit
/// `Copy`. Reconciliation is keyed; `draw` never mutates rows or
/// [`PropsState`].
pub struct PropsList<'a> {
    id: Id,
    variant: Variant,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

impl fmt::Debug for PropsList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropsList")
            .field("id", &self.id)
            .field("variant", &self.variant)
            .field("patch", &self.patch.is_some())
            .field("parts", &self.parts.len())
            .finish_non_exhaustive()
    }
}

impl<'a> PropsList<'a> {
    /// The parts this component styles or registers.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::META,
        Part::LABEL,
        Part::ROW,
        Part::TRACK,
        Part::THUMB,
    ];

    /// Construct a property list with the default variant.
    pub const fn new(id: Id) -> Self {
        PropsList {
            id,
            variant: Variant::DEFAULT,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    /// Select the theme variant.
    #[must_use]
    pub const fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// An instance patch over every part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self.ov = self.ov.part(patches);
        self
    }

    /// Replace one part's painting.
    #[must_use]
    pub const fn slot(mut self, part: Part, painter: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, painter);
        self
    }

    fn scrollbar(&self) -> super::ScrollRegion<'a> {
        let mut scrollbar = super::ScrollRegion::new(self.id)
            .inherit_family(Family::PROPS)
            .patch_part(self.parts);
        if let Some(patch) = self.patch {
            scrollbar = scrollbar.patch(patch);
        }
        if let Some(painter) = self.ov.slot_for(Part::TRACK) {
            scrollbar = scrollbar.slot(Part::TRACK, painter);
        } else if let Some(painter) = self.ov.slot_for(Part::THUMB) {
            scrollbar = scrollbar.slot(Part::THUMB, painter);
        }
        scrollbar
    }

    fn label_width(rows: &[PropsRow<'_>]) -> u16 {
        rows.iter().map(|row| width(row.label)).max().unwrap_or(0)
    }

    fn value_width(rows: &[PropsRow<'_>], width: u16) -> u16 {
        width
            .saturating_sub(Self::label_width(rows))
            .saturating_sub(2)
    }

    fn row_height(row: &PropsRow<'_>, value_width: u16) -> u16 {
        match (row.wrap, row.value) {
            (true, PropsValue::Text(value)) => wrapped_rows(value, value_width),
            _ => 1,
        }
    }

    fn content_len(rows: &[PropsRow<'_>], width: u16) -> usize {
        let value_width = Self::value_width(rows, width);
        rows.iter()
            .map(|row| usize::from(Self::row_height(row, value_width)))
            .sum()
    }

    fn visual_start(rows: &[PropsRow<'_>], index: usize, width: u16) -> usize {
        let value_width = Self::value_width(rows, width);
        rows.iter()
            .take(index)
            .map(|row| usize::from(Self::row_height(row, value_width)))
            .sum()
    }

    fn row_at_visual(rows: &[PropsRow<'_>], visual: usize, width: u16) -> Option<(usize, usize)> {
        let value_width = Self::value_width(rows, width);
        let mut start = 0usize;
        for (index, row) in rows.iter().enumerate() {
            let end = start.saturating_add(usize::from(Self::row_height(row, value_width)));
            if visual < end {
                return Some((index, start));
            }
            start = end;
        }
        None
    }

    fn reveal(&self, cx: &Cx<'_>, st: &mut PropsState, rows: &[PropsRow<'_>], index: usize) {
        let width = cx
            .area(self.id)
            .map_or(1, |area| area.width.saturating_sub(1));
        st.core
            .scroll_mut()
            .ensure_visible_on_next_layout(Self::visual_start(rows, index, width));
    }

    fn move_cursor(
        &self,
        cx: &Cx<'_>,
        st: &mut PropsState,
        rows: &[PropsRow<'_>],
        index: usize,
        acc: &mut Acc<PropsAction>,
    ) {
        let Some(row) = rows.get(index.min(rows.len().saturating_sub(1))) else {
            acc.consumed();
            return;
        };
        let index = index.min(rows.len().saturating_sub(1));
        st.core.set_cursor(index, row.key);
        self.reveal(cx, st, rows, index);
        acc.changed();
    }

    fn copy_cursor(st: &PropsState, rows: &[PropsRow<'_>], acc: &mut Acc<PropsAction>) {
        let Some(cursor) = st.core.cursor() else {
            acc.consumed();
            return;
        };
        let Some(row) = rows.iter().find(|row| row.key == cursor) else {
            acc.consumed();
            return;
        };
        if row.can_copy() {
            acc.action(PropsAction::Copy(row.key));
        } else {
            acc.consumed();
        }
    }

    /// The update phase: reconcile rows, then drain binding, pointer and
    /// scrollbar intents from the modern runtime.
    #[expect(
        clippy::too_many_lines,
        reason = "the keyed navigation and pointer dispatch are one intent drain"
    )]
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        st: &mut PropsState,
        rows: &[PropsRow<'_>],
    ) -> Response<PropsAction> {
        let _ = st.core.reconcile_with(
            rows.len(),
            |index| rows.get(index).map_or(ItemKey::index(index), |row| row.key),
            |_| true,
        );
        if st.core.cursor().is_none()
            && let Some(row) = rows.first()
        {
            st.core.set_cursor(0, row.key);
        }

        let width = cx
            .area(self.id)
            .map_or(1, |area| area.width.saturating_sub(1));
        let content_len = Self::content_len(rows, width);
        let mut acc = Acc::<PropsAction>::new();
        let scroll = self
            .scrollbar()
            .update(cx, st.core.scroll_mut(), content_len);
        acc.fold(&scroll);
        let viewport = st.core.scroll().viewport_len().max(1);

        for intent in cx.intents(self.id) {
            match intent {
                Intent::Binding(action) => match Binding::command(&PROPS_BINDINGS, action) {
                    Some(PropsCmd::Up) => self.move_cursor(
                        cx,
                        st,
                        rows,
                        st.core.cursor_index().saturating_sub(1),
                        &mut acc,
                    ),
                    Some(PropsCmd::Down) => self.move_cursor(
                        cx,
                        st,
                        rows,
                        st.core.cursor_index().saturating_add(1),
                        &mut acc,
                    ),
                    Some(PropsCmd::PageUp) => self.move_cursor(
                        cx,
                        st,
                        rows,
                        st.core.cursor_index().saturating_sub(viewport),
                        &mut acc,
                    ),
                    Some(PropsCmd::PageDown) => self.move_cursor(
                        cx,
                        st,
                        rows,
                        st.core.cursor_index().saturating_add(viewport),
                        &mut acc,
                    ),
                    Some(PropsCmd::Home) => self.move_cursor(cx, st, rows, 0, &mut acc),
                    Some(PropsCmd::End) => {
                        self.move_cursor(cx, st, rows, usize::MAX, &mut acc);
                    }
                    Some(PropsCmd::Copy) => Self::copy_cursor(st, rows, &mut acc),
                    None => {}
                },
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: Some(key),
                        },
                    pos,
                    ..
                } => {
                    let Some(area) = cx.area(self.id) else {
                        acc.consumed();
                        continue;
                    };
                    let width = area.width.saturating_sub(1);
                    let content_len = Self::content_len(rows, width);
                    let view = super::ScrollRegion::view(st.core.scroll(), area, content_len);
                    let visual = view
                        .offset()
                        .saturating_add(usize::from(pos.y.saturating_sub(area.y)));
                    let Some((index, _)) = Self::row_at_visual(rows, visual, width) else {
                        acc.consumed();
                        continue;
                    };
                    let Some(row) = rows.get(index) else {
                        acc.consumed();
                        continue;
                    };
                    if row.key != key {
                        acc.consumed();
                        continue;
                    }
                    match phase {
                        Phase::Press => {
                            self.move_cursor(cx, st, rows, index, &mut acc);
                        }
                        Phase::Click | Phase::DoubleClick => {
                            self.move_cursor(cx, st, rows, index, &mut acc);
                            Self::copy_cursor(st, rows, &mut acc);
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
    pub fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &PropsState,
        rows: &[PropsRow<'_>],
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        if !ui.is_inert() {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, &PROPS_BINDINGS);
        }

        let total = Self::content_len(rows, area.width.saturating_sub(1));
        let content = self.scrollbar().draw(ui, area, st.core.scroll(), total);
        if content.is_empty() {
            return area;
        }
        let view = super::ScrollRegion::view(st.core.scroll(), content, total);
        let value_width = Self::value_width(rows, content.width);
        let label_width = Self::label_width(rows).min(content.width);
        let hover = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let cursor = st.cursor();
        let mut visual = 0usize;

        for row in rows {
            let height = Self::row_height(row, value_width);
            let start = visual;
            let end = start.saturating_add(usize::from(height));
            visual = end;
            if end <= view.offset() {
                continue;
            }
            if start >= view.offset().saturating_add(usize::from(content.height)) {
                break;
            }
            let delta = start.abs_diff(view.offset());
            let y = if start >= view.offset() {
                content
                    .y
                    .saturating_add(delta.min(usize::from(u16::MAX)) as u16)
            } else {
                content
                    .y
                    .saturating_sub(delta.min(usize::from(u16::MAX)) as u16)
            };
            let row_area = Rect {
                x: content.x,
                y,
                width: content.width,
                height,
            };
            let row_part = PartRef::item(Part::ROW, row.key);
            let mut flags = StateFlags::empty();
            if cursor == Some(row.key) {
                flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
            }
            if hover == Some(row_part) {
                flags |= StateFlags::HOVERED;
            }
            if pressed == Some(row_part) {
                flags |= StateFlags::PRESSED;
            }
            let visible_area = row_area.intersection(content);
            let container = self.ov.style(
                ui,
                self.id,
                Family::PROPS,
                self.variant,
                Part::CONTAINER,
                flags,
            );
            ui.fill(visible_area, container.style);

            let label = self
                .ov
                .style(ui, self.id, Family::PROPS, self.variant, Part::META, flags);
            let label_area = Rect {
                x: content.x,
                y,
                width: label_width,
                height: 1,
            };
            ui.paint_str(label_area, row.label, label.style);

            let value_area = Rect {
                x: content.x.saturating_add(label_width).saturating_add(2),
                y,
                width: value_width,
                height,
            };
            let value = self
                .ov
                .style(ui, self.id, Family::PROPS, self.variant, Part::LABEL, flags);
            paint_value(ui, value_area, row, value.style);
            if matches!(value.glyph, Slot::Set(GlyphRole::PressLeft)) {
                paint_pressed_bracket(
                    ui,
                    cell_at(row_area, content.x.saturating_add(label_width)),
                    cell_at(
                        row_area,
                        content.x.saturating_add(label_width).saturating_add(1),
                    ),
                    value.style,
                );
            }

            if !ui.is_inert() && !visible_area.is_empty() {
                ui.register_part(self.id, row_part, visible_area);
            }
        }
        area
    }

    /// A conservative natural size for a property list.
    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (32, c.max.1),
        }
        .fit(c)
    }
}

impl Bindings for PropsList<'_> {
    type Cmd = PropsCmd;

    fn bindings(&self, _state: BindingState) -> &'static [Binding<PropsCmd>] {
        &PROPS_BINDINGS
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WrapPiece<'a> {
    Text(&'a str),
    Space,
    Break,
}

fn walk_wrap<'a>(s: &'a str, width: u16, f: &mut dyn FnMut(WrapPiece<'a>)) {
    let width = width.max(1);
    for (paragraph, line) in s.split('\n').enumerate() {
        if paragraph > 0 {
            f(WrapPiece::Break);
        }
        let mut line_width = 0u16;
        for word in line.split(' ') {
            let word_width = width_of(word);
            if line_width == 0 {
                if word_width <= width {
                    f(WrapPiece::Text(word));
                    line_width = word_width;
                } else {
                    hard_wrap(word, width, &mut line_width, f);
                }
            } else if line_width.saturating_add(1).saturating_add(word_width) <= width {
                f(WrapPiece::Space);
                f(WrapPiece::Text(word));
                line_width = line_width.saturating_add(1).saturating_add(word_width);
            } else {
                f(WrapPiece::Break);
                line_width = 0;
                if word_width <= width {
                    f(WrapPiece::Text(word));
                    line_width = word_width;
                } else {
                    hard_wrap(word, width, &mut line_width, f);
                }
            }
        }
    }
}

fn hard_wrap<'a>(
    word: &'a str,
    width: u16,
    line_width: &mut u16,
    f: &mut dyn FnMut(WrapPiece<'a>),
) {
    for (_, grapheme) in graphemes(word) {
        let grapheme_width = width_of(grapheme);
        if line_width.saturating_add(grapheme_width) > width {
            f(WrapPiece::Break);
            *line_width = 0;
        }
        f(WrapPiece::Text(grapheme));
        *line_width = line_width.saturating_add(grapheme_width);
    }
}

fn width_of(s: &str) -> u16 {
    width(s)
}

fn wrapped_rows(s: &str, width: u16) -> u16 {
    let mut rows = 1u16;
    walk_wrap(s, width, &mut |piece| {
        if piece == WrapPiece::Break {
            rows = rows.saturating_add(1);
        }
    });
    rows
}

fn paint_value(ui: &mut Ui<'_>, area: Rect, row: &PropsRow<'_>, style: Style) {
    match row.value {
        PropsValue::Text(value) if row.wrap => {
            let mut x = area.x;
            let mut y = area.y;
            let width = area.width.max(1);
            walk_wrap(value, width, &mut |piece| match piece {
                WrapPiece::Break => {
                    y = y.saturating_add(1);
                    x = area.x;
                }
                WrapPiece::Text(text) => paint_piece(ui, area, &mut x, y, text, style, row.tone),
                WrapPiece::Space => paint_piece(ui, area, &mut x, y, " ", style, row.tone),
            });
        }
        PropsValue::Text(value) => {
            let mut x = area.x;
            paint_piece(ui, area, &mut x, area.y, value, style, row.tone);
        }
        PropsValue::Secret(secret) => {
            let mut cell = CellUi::new(ui.reborrow(), Rect { height: 1, ..area }, style);
            if let Some(tone) = row.tone {
                cell.tone(tone);
            }
            secret.write_mask(&mut cell, secret.len(), SecretPolicy::default());
        }
    }
}

fn paint_piece(
    ui: &mut Ui<'_>,
    area: Rect,
    x: &mut u16,
    y: u16,
    text: &str,
    style: Style,
    tone: Option<Role>,
) {
    if y >= area.bottom() || *x >= area.right() || text.is_empty() {
        return;
    }
    let remaining = area.right().saturating_sub(*x);
    if remaining == 0 {
        return;
    }
    let used = width(text).min(remaining);
    let mut cell = CellUi::new(
        ui.reborrow(),
        Rect {
            x: *x,
            y,
            width: used,
            height: 1,
        },
        style,
    );
    if let Some(tone) = tone {
        cell.tone(tone);
    }
    cell.text(text);
    *x = (*x).saturating_add(used);
}

/// Label / value rows: muted labels in a column sized to the widest, values
/// beside them.
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
/// Never writes outside `area`; never allocates.
pub struct Props<'a> {
    rows: &'a [(&'a str, &'a str)],
    ov: PartStyle<'a>,
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
            ov: PartStyle::new(),
        }
    }

    /// Per-part instance patches.
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
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
        let lw = self.label_width().min(area.width);
        let ov = self.ov;
        let owner = Id::root("tui.props");
        let key_style = ov
            .style(
                ui,
                owner,
                Family::PROPS,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style;
        let value_style = ov
            .style(
                ui,
                owner,
                Family::PROPS,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        let mut painted = 0u16;
        for (row, (k, v)) in area.rows().zip(self.rows.iter()) {
            ui.paint_str(row, k, key_style);
            let value = Rect {
                x: row.x.saturating_add(lw).saturating_add(2),
                width: row.width.saturating_sub(lw).saturating_sub(2),
                ..row
            };
            ui.paint_str(value, v, value_style);
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

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{Stub, key, mouse};
    use crate::runtime::{App, Runtime};
    use crate::secret::Secret;
    use crate::theme::{GlyphRole, Theme};
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("props.tests");
    const AREA: Rect = Rect::new(0, 0, 40, 4);
    const FIRST_KEY: ItemKey = ItemKey::num(11);
    const SECOND_KEY: ItemKey = ItemKey::num(22);

    #[test]
    fn rows_preserve_keyed_borrowed_semantic_metadata() {
        let value = String::from("ready");
        let row = PropsRow::new(FIRST_KEY, "Status", value.as_str())
            .role(Role::Success)
            .wrap()
            .copyable();

        assert_eq!(row.key, FIRST_KEY);
        assert_eq!(row.label, "Status");
        assert_eq!(row.value.text(), Some("ready"));
        assert!(
            row.value
                .text()
                .is_some_and(|text| core::ptr::eq(text.as_ptr(), value.as_ptr()))
        );
        assert_eq!(row.tone, Some(Role::Success));
        assert!(row.wrap);
        assert!(row.copyable);
        assert!(row.can_copy());
    }

    fn painted_text(buffer: &Buffer, area: Rect) -> String {
        let mut text = String::new();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(cell) = buffer.cell(Position::new(x, y)) {
                    text.push_str(cell.symbol());
                }
            }
        }
        text
    }

    #[test]
    fn secret_rows_are_masked_redacted_and_non_copyable() {
        let secret = Secret::new(String::from("hunter2"));
        let row = PropsRow::secret(SECOND_KEY, "Token", &secret).copyable();
        assert!(!row.copyable);
        assert!(!row.can_copy());
        assert!(!format!("{row:?}").contains("hunter2"));

        let rows = [row];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = PropsState::default();
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            PropsList::new(ID).draw(ui, area, &state, &rows);
        });

        let text = painted_text(&buffer, AREA);
        assert!(!text.contains("hunter2"));
        assert!(text.contains(Theme::junie().design.glyphs.get(GlyphRole::SecretMask)));
    }

    #[derive(Default)]
    struct ActionApp {
        state: PropsState,
        actions: Vec<PropsAction>,
    }

    const ACTION_ROWS: [PropsRow<'static>; 2] = [
        PropsRow::new(FIRST_KEY, "Name", "alice").copyable(),
        PropsRow::new(SECOND_KEY, "Mode", "safe"),
    ];

    impl App for ActionApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let response = PropsList::new(ID).update(cx, &mut self.state, &ACTION_ROWS);
            if let Some(action) = response.action_ref() {
                self.actions.push(*action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            PropsList::new(ID).draw(ui, AREA, &self.state, &ACTION_ROWS);
        }
    }

    #[test]
    fn copy_is_keyed_and_reached_by_keyboard_and_mouse_runtime_intents() {
        let mut runtime = Runtime::new(ActionApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);

        let _ = runtime.handle(key(KeyCode::Char('y')));
        assert_eq!(runtime.app().actions, [PropsAction::Copy(FIRST_KEY)]);

        let _ = runtime.handle(key(KeyCode::Down));
        let _ = runtime.handle(key(KeyCode::Char('y')));
        assert_eq!(
            runtime.app().actions,
            [PropsAction::Copy(FIRST_KEY)],
            "a non-copyable row must consume copy without emitting an action"
        );

        runtime.draw_buffer(AREA, &mut buffer);
        let row = runtime
            .area_of_part(ID, PartRef::item(Part::ROW, FIRST_KEY))
            .expect("the keyed row must register a mouse hit region");
        let x = row.x.saturating_add(row.width / 2);
        let _ = runtime.handle(mouse(MouseKind::Down, x, row.y));
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(mouse(MouseKind::Up, x, row.y));

        assert_eq!(
            runtime.app().actions,
            [PropsAction::Copy(FIRST_KEY), PropsAction::Copy(FIRST_KEY)]
        );
    }

    #[test]
    fn reconcile_tracks_a_stable_key_across_reorder() {
        let keys = [FIRST_KEY, SECOND_KEY];
        let mut state = PropsState::default();
        state.set_cursor(0, FIRST_KEY);
        assert_eq!(
            state.reconcile(keys.len(), |index| keys[index]),
            Reconciliation::Unchanged
        );

        let reordered = [SECOND_KEY, FIRST_KEY];
        assert_eq!(
            state.reconcile(reordered.len(), |index| reordered[index]),
            Reconciliation::Unchanged
        );
        assert_eq!(state.cursor(), Some(FIRST_KEY));
        assert_eq!(state.cursor_index(), 1);
    }

    #[derive(Default)]
    struct ReorderApp {
        state: PropsState,
        reordered: bool,
        copied: Option<ItemKey>,
    }

    const REORDER_ROWS: [PropsRow<'static>; 4] = [
        PropsRow::new(ItemKey::num(1), "One", "1").copyable(),
        PropsRow::new(ItemKey::num(2), "Two", "2").copyable(),
        PropsRow::new(ItemKey::num(3), "Three", "3").copyable(),
        PropsRow::new(ItemKey::num(4), "Four", "4").copyable(),
    ];
    const SAME_ENDS_REORDERED: [PropsRow<'static>; 4] = [
        REORDER_ROWS[0],
        REORDER_ROWS[2],
        REORDER_ROWS[1],
        REORDER_ROWS[3],
    ];

    impl ReorderApp {
        fn rows(&self) -> &'static [PropsRow<'static>] {
            if self.reordered {
                &SAME_ENDS_REORDERED
            } else {
                &REORDER_ROWS
            }
        }
    }

    impl App for ReorderApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let rows = self.rows();
            let response = PropsList::new(ID).update(cx, &mut self.state, rows);
            if let Some(PropsAction::Copy(key)) = response.action_ref() {
                self.copied = Some(*key);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            PropsList::new(ID).draw(ui, AREA, &self.state, self.rows());
        }
    }

    #[test]
    fn copy_resolves_the_cursor_by_key_after_same_length_reorder() {
        let mut runtime = Runtime::new(ReorderApp::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer);
        let _ = runtime.handle(key(KeyCode::Down));
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(2)));

        runtime.app_mut().reordered = true;
        let _ = runtime.handle(key(KeyCode::Char('y')));
        assert_eq!(runtime.app().copied, Some(ItemKey::num(2)));
    }

    #[test]
    fn only_the_cursor_row_receives_focus_flags() {
        let theme = Theme::junie().define_family(Family::PROPS, |recipe| {
            recipe.part(Part::LABEL).when(
                StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
                StylePatch::new().set_bg(Role::Danger),
            );
        });
        let rows = [
            PropsRow::new(FIRST_KEY, "First", "plain"),
            PropsRow::new(SECOND_KEY, "Second", "cursor"),
        ];
        let mut state = PropsState::default();
        state.set_cursor(1, SECOND_KEY);
        let mut runtime = Runtime::new(Stub::default(), theme.clone());
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_scene(AREA, &mut buffer, |ui, area| {
            ui.reference(
                Some(ReferenceTarget::new(
                    ID,
                    ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE,
                )),
                |ui| {
                    PropsList::new(ID).draw(ui, area, &state, &rows);
                },
            );
        });

        let value_x = 8;
        assert_ne!(
            buffer.cell(Position::new(value_x, 0)).map(|cell| cell.bg),
            Some(theme.color.danger)
        );
        assert_eq!(
            buffer.cell(Position::new(value_x, 1)).map(|cell| cell.bg),
            Some(theme.color.danger)
        );
    }

    #[test]
    fn mono_pressed_brackets_the_existing_column_gap() {
        let rows = [PropsRow::new(FIRST_KEY, "Name", "value")];
        let render = |target| {
            let mut state = PropsState::default();
            state.set_cursor(0, FIRST_KEY);
            let theme = Theme::junie().downgrade(crate::ColorLevel::Mono);
            let mut runtime = Runtime::new(Stub::default(), theme);
            let mut buffer = Buffer::empty(AREA);
            runtime.draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(Some(target), |ui| {
                    PropsList::new(ID).draw(ui, area, &state, &rows);
                });
            });
            buffer
        };

        let focused = render(ReferenceTarget::new(ID, ReferenceState::FOCUSED));
        let pressed = render(
            ReferenceTarget::new(ID, ReferenceState::PRESSED)
                .part(PartRef::item(Part::ROW, FIRST_KEY)),
        );
        let left = Theme::junie().design.glyphs.get(GlyphRole::PressLeft);
        let right = Theme::junie().design.glyphs.get(GlyphRole::PressRight);

        assert_eq!(
            focused.cell(Position::new(4, 0)).map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_eq!(
            focused.cell(Position::new(5, 0)).map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_eq!(
            pressed.cell(Position::new(4, 0)).map(|cell| cell.symbol()),
            Some(left)
        );
        assert_eq!(
            pressed.cell(Position::new(5, 0)).map(|cell| cell.symbol()),
            Some(right)
        );
    }

    #[test]
    fn track_and_thumb_slots_are_forwarded_independently() {
        let rows = [PropsRow::new(FIRST_KEY, "Name", "value"); 12];
        for part in [Part::TRACK, Part::THUMB] {
            let called = Cell::new(false);
            let painter = |_: &mut Ui<'_>, _: Rect| called.set(true);
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(AREA);
            let state = PropsState::default();
            runtime.draw_scene(AREA, &mut buffer, |ui, area| {
                PropsList::new(ID)
                    .slot(part, &painter)
                    .draw(ui, area, &state, &rows);
            });
            assert!(called.get(), "{part:?} slot was not forwarded");
        }
    }
}
