use core::fmt;

use super::{Acc, PartStyle, SlotFn, cell_at, paint_pressed_bracket};
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
use crate::theme::PaintStyle;
use crate::theme::{Family, GlyphRole, Role, Slot, StyleDefaults, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, Ui};
use ratatui_core::layout::Rect;
use termrock_collections::collection::{Reconcile, Reconciliation};
use termrock_runtime::rowui::CellUi;
use termrock_runtime::rowui::RowUi;
use termrock_theme::CursorStyle;

#[derive(Clone, Copy)]
pub enum PropsValue<'a> {
    Text(&'a str),
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
    pub const fn is_secret(self) -> bool {
        matches!(self, PropsValue::Secret(_))
    }

    pub const fn text(self) -> Option<&'a str> {
        match self {
            PropsValue::Text(value) => Some(value),
            PropsValue::Secret(_) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PropsRow<'a> {
    pub key: ItemKey,
    pub label: &'a str,
    pub value: PropsValue<'a>,
    pub tone: Option<Role>,
    pub wrap: bool,
    pub copyable: bool,
}

impl<'a> PropsRow<'a> {
    pub const fn new(key: ItemKey, label: &'a str, value: &'a str) -> Self {
        Self::value(key, label, PropsValue::Text(value))
    }

    pub const fn secret(key: ItemKey, label: &'a str, value: &'a Secret) -> Self {
        Self::value(key, label, PropsValue::Secret(value))
    }

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

    #[must_use]
    pub const fn tone(mut self, tone: Role) -> Self {
        self.tone = Some(tone);
        self
    }

    #[must_use]
    pub const fn role(self, role: Role) -> Self {
        self.tone(role)
    }

    #[must_use]
    pub const fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    #[must_use]
    pub const fn wrap_if(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    #[must_use]
    pub const fn copyable(mut self) -> Self {
        self.copyable = !self.value.is_secret();
        self
    }

    #[must_use]
    pub const fn copyable_if(mut self, yes: bool) -> Self {
        self.copyable = yes && !self.value.is_secret();
        self
    }

    pub const fn can_copy(&self) -> bool {
        self.copyable && !self.value.is_secret()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropsAction {
    Copy(ItemKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropsCmd {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
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

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PropsState {
    core: termrock_collections::collection::CollectionCore,
}

/// Layout policy for keyed property rows.
///
/// `Columns` preserves the original compact two-column presentation. `Sheet`
/// reserves a focus gutter, aligns labels using a bounded share of the row,
/// and exposes the focused row's copy hint when that row is copyable.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PropsPresentation {
    #[default]
    Columns,
    Sheet,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PropsGeometry {
    label_width: u16,
    value_width: u16,
}

impl PropsState {
    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn cursor_index(&self) -> usize {
        self.core.cursor_index()
    }

    pub const fn scroll(&self) -> &ScrollState {
        self.core.scroll()
    }

    pub fn set_cursor(&mut self, index: usize, key: ItemKey) {
        self.core.set_cursor(index, key);
    }

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

pub struct PropsList<'a> {
    id: Id,
    presentation: PropsPresentation,
    variant: Variant,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    ov: PartStyle<'a>,
}

impl fmt::Debug for PropsList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropsList")
            .field("id", &self.id)
            .field("presentation", &self.presentation)
            .field("variant", &self.variant)
            .field("patch", &self.patch.is_some())
            .field("parts", &self.parts.len())
            .finish_non_exhaustive()
    }
}

impl<'a> PropsList<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::META,
        Part::LABEL,
        Part::ROW,
        Part::GUTTER,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const fn new(id: Id) -> Self {
        PropsList {
            id,
            presentation: PropsPresentation::Columns,
            variant: Variant::DEFAULT,
            patch: None,
            parts: &[],
            ov: PartStyle::new(),
        }
    }

    /// Use the aligned property-sheet row contract used by read-only fact
    /// lists. Existing `PropsList::new` callers keep the compact layout.
    #[must_use]
    pub const fn sheet(mut self) -> Self {
        self.presentation = PropsPresentation::Sheet;
        self
    }

    #[must_use]
    pub const fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self.ov = self.ov.global(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self.ov = self.ov.part(patches);
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, painter: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, painter);
        self
    }

    fn scrollbar(&self) -> super::ScrollRegion<'a> {
        let mut scrollbar = super::ScrollRegion::new(self.id)
            .inherit_family(Family::PROPS)
            .patch_part(self.parts)
            .preserve_underlay(self.presentation == PropsPresentation::Sheet);
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

    fn style_defaults(
        &self,
        ui: &mut Ui<'_>,
        part: Part,
        flags: StateFlags,
        defaults: StyleDefaults<'_>,
    ) -> PaintStyle {
        let local = self.ov.part_patch(part);
        let resolved = ui.style_defaults(
            Family::PROPS,
            self.variant,
            part,
            flags,
            defaults,
            local.as_ref(),
        );
        self.ov
            .note(ui, self.id, Family::PROPS, self.variant, part, resolved);
        resolved.style
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

    fn sheet_row_height(row: &PropsRow<'_>, value_width: u16, active_key: Option<ItemKey>) -> u16 {
        if row.wrap && active_key == Some(row.key) {
            match row.value {
                PropsValue::Text(value) => wrapped_rows(value, value_width),
                PropsValue::Secret(_) => 1,
            }
        } else {
            1
        }
    }

    fn content_len(rows: &[PropsRow<'_>], width: u16) -> usize {
        let value_width = Self::value_width(rows, width);
        rows.iter()
            .map(|row| usize::from(Self::row_height(row, value_width)))
            .sum()
    }

    fn sheet_geometry(rows: &[PropsRow<'_>], width: u16) -> PropsGeometry {
        // Frozen Sheet::column() first tries every full measured label plus
        // its two-cell gap. Only when that leaves less than twelve value cells
        // does it apply the configured 40% cap. Keep the gutter outside this
        // label-column calculation, as the source implementation does.
        let label_area = width.saturating_sub(2);
        let full_label_column = Self::label_width(rows).saturating_add(2);
        let label_column = if rows.is_empty() {
            0
        } else if label_area.saturating_sub(full_label_column) >= 12 {
            full_label_column
        } else {
            let cap = u32::from(label_area).saturating_mul(40) / 100;
            let cap = cap.min(u32::from(u16::MAX)) as u16;
            rows.iter()
                .map(|row| width_of(row.label).saturating_add(2))
                .filter(|candidate| *candidate <= cap)
                .max()
                .unwrap_or(cap.max(1))
        }
        .max(if rows.is_empty() { 0 } else { 3 });
        let label_width = label_column.saturating_sub(2);
        let value_width = width
            .saturating_sub(2)
            .saturating_sub(label_width)
            .saturating_sub(2)
            .saturating_sub(1);
        PropsGeometry {
            label_width,
            value_width,
        }
    }

    fn sheet_value_width(
        geometry: PropsGeometry,
        row: &PropsRow<'_>,
        active_key: Option<ItemKey>,
    ) -> u16 {
        geometry
            .value_width
            .saturating_sub(if active_key == Some(row.key) && row.can_copy() {
                8
            } else {
                0
            })
    }

    fn resolved_content_len(
        rows: &[PropsRow<'_>],
        area: Rect,
        cursor: Option<ItemKey>,
        focused: bool,
        presentation: PropsPresentation,
    ) -> usize {
        if presentation == PropsPresentation::Columns {
            return Self::content_len(rows, area.width.saturating_sub(1));
        }
        let active_key = focused.then_some(cursor).flatten();
        let geometry = Self::sheet_geometry(rows, area.width);
        let full = rows
            .iter()
            .map(|row| {
                usize::from(Self::sheet_row_height(
                    row,
                    Self::sheet_value_width(geometry, row, active_key),
                    active_key,
                ))
            })
            .sum::<usize>();
        if full <= usize::from(area.height) || area.width == 0 {
            return full;
        }
        let geometry = Self::sheet_geometry(rows, area.width.saturating_sub(1));
        rows.iter()
            .map(|row| {
                usize::from(Self::sheet_row_height(
                    row,
                    Self::sheet_value_width(geometry, row, active_key),
                    active_key,
                ))
            })
            .sum()
    }

    fn row_at_visual_width(
        rows: &[PropsRow<'_>],
        visual: usize,
        geometry: PropsGeometry,
        active_key: Option<ItemKey>,
        presentation: PropsPresentation,
    ) -> Option<(usize, usize)> {
        let mut start = 0usize;
        for (index, row) in rows.iter().enumerate() {
            let height = match presentation {
                PropsPresentation::Columns => Self::row_height(row, geometry.value_width),
                PropsPresentation::Sheet => Self::sheet_row_height(
                    row,
                    Self::sheet_value_width(geometry, row, active_key),
                    active_key,
                ),
            };
            let end = start.saturating_add(usize::from(height));
            if visual < end {
                return Some((index, start));
            }
            start = end;
        }
        None
    }

    fn reveal(&self, cx: &Cx<'_>, st: &mut PropsState, rows: &[PropsRow<'_>], index: usize) {
        let area = cx.area(self.id).unwrap_or_default();
        let focused = cx.state(self.id).contains(StateFlags::FOCUSED);
        let total = Self::resolved_content_len(rows, area, st.cursor(), focused, self.presentation);
        let width = if total > usize::from(area.height) {
            area.width.saturating_sub(1)
        } else {
            area.width
        };
        let geometry = match self.presentation {
            PropsPresentation::Columns => PropsGeometry {
                label_width: 0,
                value_width: Self::value_width(rows, width),
            },
            PropsPresentation::Sheet => Self::sheet_geometry(rows, width),
        };
        let active_key = match self.presentation {
            PropsPresentation::Columns => None,
            PropsPresentation::Sheet if focused => st.cursor(),
            PropsPresentation::Sheet => None,
        };
        let row_start = rows
            .iter()
            .take(index)
            .map(|row| match self.presentation {
                PropsPresentation::Columns => Self::row_height(row, geometry.value_width),
                PropsPresentation::Sheet => Self::sheet_row_height(
                    row,
                    Self::sheet_value_width(geometry, row, active_key),
                    active_key,
                ),
            })
            .map(usize::from)
            .sum();
        st.core
            .scroll_mut()
            .ensure_visible_on_next_layout(row_start);
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
        let cursor_before = st.core.cursor();
        let index_before = st.core.cursor_index();
        let _ = st.core.reconcile_with(
            rows.len(),
            |index| rows.get(index).map_or(ItemKey::index(index), |row| row.key),
            |_| true,
        );
        // The collection's cheap stamp catches length and endpoint changes.
        // A live property list can also reorder interior items while retaining
        // those endpoints, so resolve the stable cursor key against current
        // rows before routing input or publishing its next layout.
        if let Some(key) = cursor_before
            && let Some(index) = rows.iter().position(|row| row.key == key)
            && index != index_before
        {
            st.core.set_cursor(index, key);
        } else if cursor_before.is_some_and(|key| !rows.iter().any(|row| row.key == key)) {
            // The current key disappeared without changing the collection's
            // endpoint stamp; invalidate it so reconciliation selects a live
            // neighbor rather than leaving an invisible cursor behind.
            st.core.invalidate();
            let _ = st.core.reconcile_with(
                rows.len(),
                |index| rows.get(index).map_or(ItemKey::index(index), |row| row.key),
                |_| true,
            );
        }
        if st.core.cursor().is_none()
            && let Some(row) = rows.first()
        {
            st.core.set_cursor(0, row.key);
        }

        let area = cx.area(self.id).unwrap_or_default();
        let content_len = Self::resolved_content_len(
            rows,
            area,
            st.cursor(),
            cx.state(self.id).contains(StateFlags::FOCUSED),
            self.presentation,
        );
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
                    let content_len = Self::resolved_content_len(
                        rows,
                        area,
                        st.cursor(),
                        cx.state(self.id).contains(StateFlags::FOCUSED),
                        self.presentation,
                    );
                    let view = super::ScrollRegion::view(st.core.scroll(), area, content_len);
                    let visual = view
                        .offset()
                        .saturating_add(usize::from(pos.y.saturating_sub(area.y)));
                    let content_width = if content_len > usize::from(area.height) {
                        area.width.saturating_sub(1)
                    } else {
                        area.width
                    };
                    let active_key = cx
                        .state(self.id)
                        .contains(StateFlags::FOCUSED)
                        .then_some(st.cursor())
                        .flatten();
                    let geometry = match self.presentation {
                        PropsPresentation::Columns => PropsGeometry {
                            label_width: 0,
                            value_width: Self::value_width(rows, content_width),
                        },
                        PropsPresentation::Sheet => Self::sheet_geometry(rows, content_width),
                    };
                    let Some((index, _)) = Self::row_at_visual_width(
                        rows,
                        visual,
                        geometry,
                        active_key,
                        self.presentation,
                    ) else {
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

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over scrolling, state styling, value painting and row registration"
    )]
    pub fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &PropsState,
        rows: &[PropsRow<'_>],
    ) -> Rect {
        if area.is_empty() || (self.presentation == PropsPresentation::Sheet && rows.is_empty()) {
            return area;
        }
        if !ui.is_inert() && !(self.presentation == PropsPresentation::Sheet && rows.is_empty()) {
            ui.register_control(self.id, area, Focusability::Focusable);
        }
        let live = PartStyle::flags(ui.state(self.id), StateFlags::empty());
        if !ui.is_inert() {
            ui.publish_bindings(self.id, live, &PROPS_BINDINGS);
        }

        let total = Self::resolved_content_len(
            rows,
            area,
            st.cursor(),
            ui.state(self.id).contains(StateFlags::FOCUSED),
            self.presentation,
        );
        let content = self.scrollbar().draw(ui, area, st.core.scroll(), total);
        if content.is_empty() {
            return area;
        }
        let view = super::ScrollRegion::view(st.core.scroll(), content, total);
        let cursor = st.cursor();
        let focused_cursor = ui
            .state(self.id)
            .contains(StateFlags::FOCUSED)
            .then_some(cursor)
            .flatten();
        let geometry = match self.presentation {
            PropsPresentation::Columns => PropsGeometry {
                label_width: Self::label_width(rows).min(content.width),
                value_width: Self::value_width(rows, content.width),
            },
            PropsPresentation::Sheet => Self::sheet_geometry(rows, content.width),
        };
        let label_width = geometry.label_width;
        let hover = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let mut visual = 0usize;

        for row in rows {
            let value_width = match self.presentation {
                PropsPresentation::Columns => geometry.value_width,
                PropsPresentation::Sheet => Self::sheet_value_width(geometry, row, focused_cursor),
            };
            let height = match self.presentation {
                PropsPresentation::Columns => Self::row_height(row, value_width),
                PropsPresentation::Sheet => {
                    Self::sheet_row_height(row, value_width, focused_cursor)
                }
            };
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
            let container_part = self.ov.part_patch(Part::CONTAINER);
            let container_defaults = ui.style_defaults_with_explicit(
                Family::PROPS,
                self.variant,
                Part::CONTAINER,
                flags,
                StyleDefaults::new(StylePatch::new().clear_fg()),
                container_part.as_ref(),
            );
            let container_has_explicit_fg = container_defaults.explicit.fg.speaks();
            if self.presentation == PropsPresentation::Sheet {
                {
                    let mut row_ui = RowUi::new_with_author_style_and_container_patch(
                        ui,
                        self.id,
                        Family::PROPS,
                        self.variant,
                        flags,
                        row.key,
                        visible_area,
                        self.ov,
                        if container_has_explicit_fg {
                            StylePatch::new()
                        } else {
                            StylePatch::new().clear_fg()
                        },
                    )
                    .with_cursor_style(CursorStyle::Bar)
                    .with_author_parts(Self::PARTS);
                    row_ui.gutter();
                    row_ui.leading_part(Part::CELL, 1, 0, |_| {});
                }
            } else {
                let container = self.ov.style(
                    ui,
                    self.id,
                    Family::PROPS,
                    self.variant,
                    Part::CONTAINER,
                    flags,
                );
                ui.fill(visible_area, container.style);
            }

            let label = self
                .ov
                .style(ui, self.id, Family::PROPS, self.variant, Part::META, flags);
            let label_origin =
                content
                    .x
                    .saturating_add(if self.presentation == PropsPresentation::Sheet {
                        2
                    } else {
                        0
                    });
            let label_area = Rect {
                x: label_origin,
                y,
                width: match self.presentation {
                    PropsPresentation::Columns => label_width,
                    PropsPresentation::Sheet => label_width,
                },
                height: 1,
            };
            let label_style = if self.presentation == PropsPresentation::Sheet {
                let defaults = StylePatch::new()
                    .set_fg(Role::Fg(termrock_theme::FgStep::Muted))
                    .remove(ratatui_core::style::Modifier::BOLD);
                self.style_defaults(ui, Part::META, flags, StyleDefaults::new(defaults))
            } else {
                label.style
            };
            ui.paint_str(
                label_area.intersection(visible_area),
                row.label,
                label_style,
            );

            let value_area = Rect {
                x: label_origin.saturating_add(label_width).saturating_add(2),
                y,
                width: value_width,
                height,
            };
            let value_defaults = row
                .tone
                .map_or_else(StylePatch::new, |tone| StylePatch::new().set_fg(tone));
            let authored_value =
                self.ov
                    .style(ui, self.id, Family::PROPS, self.variant, Part::LABEL, flags);
            let value_style =
                self.style_defaults(ui, Part::LABEL, flags, StyleDefaults::new(value_defaults));
            ui.with_area(visible_area, |ui| {
                paint_value(ui, value_area, row, value_style, None);
            });
            if self.presentation == PropsPresentation::Sheet
                && flags.contains(StateFlags::FOCUSED)
                && row.can_copy()
            {
                let hint_style = self.style_defaults(
                    ui,
                    Part::META,
                    flags,
                    StyleDefaults::new(
                        StylePatch::new()
                            .set_fg(Role::Fg(termrock_theme::FgStep::Faint))
                            .remove(ratatui_core::style::Modifier::BOLD),
                    ),
                );
                let hint_x = row_area.right().saturating_sub(7).max(visible_area.x);
                let hint = Rect {
                    x: hint_x,
                    y,
                    width: row_area.right().saturating_sub(hint_x).min(6),
                    height: 1,
                };
                ui.paint_str(hint.intersection(visible_area), "y copy", hint_style);
            }
            if matches!(authored_value.glyph, Slot::Set(GlyphRole::PressLeft)) {
                paint_pressed_bracket(
                    ui,
                    cell_at(row_area, label_origin.saturating_add(label_width)),
                    cell_at(
                        row_area,
                        label_origin.saturating_add(label_width).saturating_add(1),
                    ),
                    value_style,
                );
            }

            if !ui.is_inert() && !visible_area.is_empty() {
                ui.register_part(self.id, row_part, visible_area);
            }
        }
        area
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (12, 1),
            preferred: (32, c.max.1),
        }
        .fit(c)
    }

    /// Measure this list from its live rows and keyed state. The same row
    /// geometry used by update and draw determines wrapping and copy-hint
    /// reservation; `measure` remains available for legacy callers.
    pub fn measure_rows(
        &self,
        ui: &Ui<'_>,
        c: Constraints,
        state: &PropsState,
        rows: &[PropsRow<'_>],
    ) -> Size {
        if self.presentation != PropsPresentation::Sheet {
            return self.measure(ui, c);
        }
        if rows.is_empty() || c.max.0 == 0 {
            return Size::exact(0, 0).fit(c);
        }
        let focused = ui.state(self.id).contains(StateFlags::FOCUSED);
        let hint = focused
            && state
                .cursor()
                .and_then(|key| rows.iter().find(|row| row.key == key))
                .is_some_and(PropsRow::can_copy);
        let natural_label = Self::label_width(rows);
        let natural_value = rows
            .iter()
            .map(|row| match row.value {
                PropsValue::Text(value) => width(value),
                PropsValue::Secret(secret) => {
                    let policy = SecretPolicy::default();
                    let glyph_width = width(ui.design().glyphs.get(policy.mask));
                    (secret.len().min(usize::from(u16::MAX)) as u16).saturating_mul(glyph_width)
                }
            })
            .max()
            .unwrap_or(0);
        let natural_width = 2u16
            .saturating_add(natural_label.saturating_add(2).max(3))
            .saturating_add(2)
            .saturating_add(natural_value)
            .saturating_add(if hint { 8 } else { 0 })
            .saturating_add(1);
        let chosen_width = if c.tight_w {
            c.max.0
        } else {
            natural_width.min(c.max.0)
        };
        let active_key = focused.then_some(state.cursor()).flatten();
        let mut geometry = Self::sheet_geometry(rows, chosen_width);
        let mut height = rows
            .iter()
            .map(|row| {
                usize::from(Self::sheet_row_height(
                    row,
                    Self::sheet_value_width(geometry, row, active_key),
                    active_key,
                ))
            })
            .sum::<usize>();
        let chosen_width = if height > usize::from(c.max.1) && chosen_width > 0 {
            let narrower = chosen_width.saturating_sub(1);
            geometry = Self::sheet_geometry(rows, narrower);
            height = rows
                .iter()
                .map(|row| {
                    usize::from(Self::sheet_row_height(
                        row,
                        Self::sheet_value_width(geometry, row, active_key),
                        active_key,
                    ))
                })
                .sum::<usize>();
            narrower
        } else {
            chosen_width
        };
        Size::exact(chosen_width, height.min(usize::from(u16::MAX)) as u16).fit(c)
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

fn paint_value(
    ui: &mut Ui<'_>,
    area: Rect,
    row: &PropsRow<'_>,
    style: PaintStyle,
    tone: Option<Role>,
) {
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
                WrapPiece::Text(text) => paint_piece(ui, area, &mut x, y, text, style, tone),
                WrapPiece::Space => paint_piece(ui, area, &mut x, y, " ", style, tone),
            });
        }
        PropsValue::Text(value) => {
            let mut x = area.x;
            paint_piece(ui, area, &mut x, area.y, value, style, tone);
        }
        PropsValue::Secret(secret) => {
            let mut cell = CellUi::new(ui.reborrow(), Rect { height: 1, ..area }, style);
            if let Some(tone) = tone {
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
    style: PaintStyle,
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

pub struct Props<'a> {
    rows: PropsRows<'a>,
    ov: PartStyle<'a>,
}

#[derive(Clone, Copy)]
enum PropsRows<'a> {
    Pairs(&'a [(&'a str, &'a str)]),
    Rich(&'a [PropsRow<'a>]),
}

impl PropsRows<'_> {
    fn len(self) -> usize {
        match self {
            Self::Pairs(rows) => rows.len(),
            Self::Rich(rows) => rows.len(),
        }
    }
}

impl fmt::Debug for Props<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Props")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

impl<'a> Props<'a> {
    pub const PARTS: &'static [Part] = &[Part::META, Part::LABEL];

    pub const fn new(rows: &'a [(&'a str, &'a str)]) -> Self {
        Props {
            rows: PropsRows::Pairs(rows),
            ov: PartStyle::new(),
        }
    }

    pub const fn rich(rows: &'a [PropsRow<'a>]) -> Self {
        Props {
            rows: PropsRows::Rich(rows),
            ov: PartStyle::new(),
        }
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    fn label_width(&self) -> u16 {
        match self.rows {
            PropsRows::Pairs(rows) => rows.iter().map(|(k, _)| width(k)).max().unwrap_or(0),
            PropsRows::Rich(rows) => PropsList::label_width(rows),
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        let rows = match self.rows {
            PropsRows::Pairs(rows) => rows,
            PropsRows::Rich(rows) => return self.draw_rich(ui, area, rows),
        };
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
        for (row, (k, v)) in area.rows().zip(rows.iter()) {
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

    fn draw_rich(&self, ui: &mut Ui<'_>, area: Rect, rows: &[PropsRow<'_>]) -> Rect {
        if area.is_empty() || rows.is_empty() {
            return Rect { height: 0, ..area };
        }
        let owner = Id::root("tui.props");
        let label_width = PropsList::label_width(rows).min(area.width);
        let value_width = PropsList::value_width(rows, area.width);
        let key_style = self
            .ov
            .style(
                ui,
                owner,
                Family::PROPS,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style;
        let mut painted = 0u16;
        for row in rows {
            if painted >= area.height {
                break;
            }
            let height =
                PropsList::row_height(row, value_width).min(area.height.saturating_sub(painted));
            let y = area.y.saturating_add(painted);
            ui.paint_str(
                Rect {
                    x: area.x,
                    y,
                    width: label_width,
                    height: 1,
                },
                row.label,
                key_style,
            );
            let value_style = if let Some(tone) = row.tone {
                let local = self.ov.part_patch(Part::LABEL);
                let resolved = ui.style_defaults(
                    Family::PROPS,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                    StyleDefaults::new(StylePatch::new().set_fg(tone)),
                    local.as_ref(),
                );
                self.ov.note(
                    ui,
                    owner,
                    Family::PROPS,
                    Variant::DEFAULT,
                    Part::LABEL,
                    resolved,
                );
                resolved.style
            } else {
                self.ov
                    .style(
                        ui,
                        owner,
                        Family::PROPS,
                        Variant::DEFAULT,
                        Part::LABEL,
                        StateFlags::empty(),
                    )
                    .style
            };

            let resolved_row = PropsRow { tone: None, ..*row };
            paint_value(
                ui,
                Rect {
                    x: area.x.saturating_add(label_width).saturating_add(2),
                    y,
                    width: value_width,
                    height,
                },
                &resolved_row,
                value_style,
                None,
            );
            painted = painted.saturating_add(height);
        }
        Rect {
            height: painted,
            ..area
        }
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        match self.rows {
            PropsRows::Pairs(rows) => {
                let vw = rows.iter().map(|(_, v)| width(v)).max().unwrap_or(0);
                let w = self.label_width().saturating_add(2).saturating_add(vw);
                let h = rows.len().min(usize::from(u16::MAX)) as u16;
                Size::exact(w, h).fit(c)
            }
            PropsRows::Rich(rows) => {
                if rows.is_empty() || c.max.0 == 0 {
                    return Size::exact(0, 0).fit(c);
                }
                let value_width = rows
                    .iter()
                    .map(|row| match row.value {
                        PropsValue::Text(value) if row.wrap => {
                            value.split('\n').map(width).max().unwrap_or(0)
                        }
                        PropsValue::Text(value) => width(value),
                        PropsValue::Secret(secret) => {
                            let policy = SecretPolicy::default();
                            let glyph_width = width(ui.design().glyphs.get(policy.mask));
                            let masks = secret.len().min(usize::from(u16::MAX)) as u16;
                            let tail = if secret.is_empty() {
                                0
                            } else {
                                policy.synthetic_tail.min(8) as u16
                            };
                            masks.saturating_mul(glyph_width).saturating_add(tail)
                        }
                    })
                    .max()
                    .unwrap_or(0);
                let natural = self
                    .label_width()
                    .saturating_add(2)
                    .saturating_add(value_width);
                let chosen = if c.tight_w {
                    c.max.0
                } else {
                    natural.min(c.max.0)
                };
                let height = PropsList::content_len(rows, chosen).min(usize::from(u16::MAX)) as u16;
                Size::exact(chosen, height).fit(c)
            }
        }
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
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                PropsList::new(ID).draw(ui, area, &state, &rows);
            })
            .commit_presented();

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
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert_eq!(runtime.app().actions, [PropsAction::Copy(FIRST_KEY)]);

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Down));
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert_eq!(
            runtime.app().actions,
            [PropsAction::Copy(FIRST_KEY)],
            "a non-copyable row must consume copy without emitting an action"
        );

        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let row = runtime
            .area_of_part(ID, PartRef::item(Part::ROW, FIRST_KEY))
            .expect("the keyed row must register a mouse hit region");
        let x = row.x.saturating_add(row.width / 2);
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, x, row.y));
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, x, row.y));

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
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Down));
        assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(2)));

        runtime.app_mut().reordered = true;
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
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
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(ReferenceTarget::new(
                        ID,
                        ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE,
                    )),
                    |ui| {
                        PropsList::new(ID).draw(ui, area, &state, &rows);
                    },
                );
            })
            .commit_presented();

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
            runtime
                .draw_scene(AREA, &mut buffer, |ui, area| {
                    ui.reference(Some(target), |ui| {
                        PropsList::new(ID).draw(ui, area, &state, &rows);
                    });
                })
                .commit_presented();
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
            focused
                .cell(Position::new(4, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(" ")
        );
        assert_eq!(
            focused
                .cell(Position::new(5, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(" ")
        );
        assert_eq!(
            pressed
                .cell(Position::new(4, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(left)
        );
        assert_eq!(
            pressed
                .cell(Position::new(5, 0))
                .map(ratatui_core::buffer::Cell::symbol),
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
            runtime
                .draw_scene(AREA, &mut buffer, |ui, area| {
                    PropsList::new(ID)
                        .slot(part, &painter)
                        .draw(ui, area, &state, &rows);
                })
                .commit_presented();
            assert!(called.get(), "{part:?} slot was not forwarded");
        }
    }
}
