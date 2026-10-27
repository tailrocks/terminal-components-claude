use core::fmt;
use std::time::Duration;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;
use termrock_core::typeahead::TypeAhead;

use super::{Acc, PartStyle, SlotFn};
use crate::action::ActionKey;
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef, custom_hash16};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::text::width;
use crate::theme::{
    Align, Family, FgStep, GlyphRole, Modifier, Role, Slot, StyleDefaults, StylePatch, Variant,
};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::collection::{
    CollectionCore, KeySet, Reconcile, Reconciliation, SelectMode,
};
use termrock_feedback::progress::Spinner;
use termrock_fields::input::{BlurPolicy, TextAction, TextInput, TextInputState};
use termrock_runtime::decor::{CellDecor, RowDecor};
use termrock_runtime::empty::{EmptyState, RowTotal};
use termrock_runtime::rowui::CellUi;
use termrock_runtime::scroll_region::ScrollRegion;

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnKey(u16);

const COLUMN_KEY_MAX: u16 = 0x7FFF;

impl ColumnKey {
    pub const fn num(n: u16) -> Self {
        assert!(
            n <= COLUMN_KEY_MAX,
            "ColumnKey::num is 0..=0x7FFF; 0x8000.. belongs to ColumnKey::of"
        );
        ColumnKey(n)
    }

    pub const fn of(name: &str) -> Self {
        ColumnKey(custom_hash16(name))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl fmt::Debug for ColumnKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > COLUMN_KEY_MAX {
            write!(f, "ColumnKey::of(#{:04x})", self.0)
        } else {
            write!(f, "ColumnKey::num({})", self.0)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NavUnit {
    Row,

    #[default]
    Cell,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CellRef<'a> {
    pub text: &'a str,

    pub tone: Option<Role>,

    pub align: Option<Align>,
}

impl<'a> CellRef<'a> {
    pub const fn new(text: &'a str) -> Self {
        CellRef {
            text,
            tone: None,
            align: None,
        }
    }

    #[must_use]
    pub const fn align(mut self, a: Align) -> Self {
        self.align = Some(a);
        self
    }

    #[must_use]
    pub const fn tone(mut self, r: Role) -> Self {
        self.tone = Some(r);
        self
    }
}

impl Default for CellRef<'_> {
    fn default() -> Self {
        CellRef::new("")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridCell<'a> {
    pub row_key: ItemKey,

    pub column_key: ColumnKey,

    pub row: usize,

    pub column: usize,

    pub value: CellRef<'a>,

    pub flags: StateFlags,

    pub style: PaintStyle,
}

type CellRenderer<'a> = &'a dyn Fn(GridCell<'_>, &mut CellUi<'_>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CellAction {
    pub key: ActionKey,

    pub glyph: GlyphRole,

    pub chord: Option<Chord>,
}

impl CellAction {
    pub const fn new(key: ActionKey) -> Self {
        CellAction {
            key,
            glyph: GlyphRole::FollowRef,
            chord: None,
        }
    }

    #[must_use]
    pub const fn glyph(mut self, g: GlyphRole) -> Self {
        self.glyph = g;
        self
    }

    #[must_use]
    pub const fn chord(mut self, c: Chord) -> Self {
        self.chord = Some(c);
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditIntent<'a> {
    Inline { initial: &'a str },

    Cycle,

    External,

    Refuse { reason: &'a str },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Column<'a> {
    pub key: ColumnKey,

    pub title: &'a str,

    pub subtitle: Option<&'a str>,

    pub align: Align,

    pub min_width: u16,

    pub max_width: u16,

    pub sortable: bool,

    pub editable: bool,

    pub sticky: bool,

    pub prefix_glyph: Option<GlyphRole>,

    pub badge: Option<&'a str>,
}

impl<'a> Column<'a> {
    pub const fn new(key: ColumnKey, title: &'a str) -> Self {
        Column {
            key,
            title,
            subtitle: None,
            align: Align::Left,
            min_width: 3,
            max_width: 40,
            sortable: false,
            editable: false,
            sticky: false,
            prefix_glyph: None,
            badge: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortDir {
    Asc,

    Desc,
}

pub trait GridModel {
    fn row_count(&self) -> usize;

    fn row_key(&self, row: usize) -> ItemKey;

    fn row_enabled(&self, _row: usize) -> bool {
        true
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>>;

    fn row_decor(&self, _row: usize) -> RowDecor<'_> {
        RowDecor::default()
    }

    fn cell_decor(&self, _row: usize, _col: usize) -> CellDecor<'_> {
        CellDecor::default()
    }

    fn total(&self) -> RowTotal {
        RowTotal::Unknown
    }

    fn has_more(&self) -> bool {
        false
    }

    fn read_only_reason(&self) -> Option<&str> {
        None
    }

    fn actions(&self, _row: usize, _col: usize) -> &[CellAction] {
        &[]
    }

    fn revision(&self) -> Option<u64> {
        None
    }
}

fn find_type_ahead_row<M: GridModel + ?Sized>(
    prefix: &str,
    len: usize,
    from: usize,
    search_column: usize,
    model: &M,
) -> Option<usize> {
    TypeAhead::find_indexed(
        prefix,
        len,
        from,
        |row| model.cell(row, search_column).map_or("", |cell| cell.text),
        |row| model.row_enabled(row),
    )
}

pub trait GridEditor: GridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_>;

    fn apply_cycle(&mut self, row: usize, col: usize);

    fn commit_cell(
        &mut self,
        row: usize,
        col: usize,
        text: &str,
    ) -> Result<(), crate::validate::FieldError>;

    fn is_editable(&self, row: usize, col: usize) -> bool;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GridAction {
    Moved,

    Activated(ItemKey),

    Sort(ColumnKey, SortDir),

    FetchMore,

    Copy(String),

    EditRequested(ItemKey, ColumnKey),

    CellAction(ItemKey, ColumnKey, ActionKey),

    LeaveForward,

    LeaveBackward,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridCmd {
    Up,

    Down,

    Left,

    Right,

    PageUp,

    PageDown,

    RowStart,

    RowEnd,

    First,

    Last,

    ExtendUp,

    ExtendDown,

    ExtendLeft,

    ExtendRight,

    ToggleRow,

    ToggleAll,

    Activate,

    BeginEdit,

    Copy,

    SortCurrent,
}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: GridCmd,
    label: &'static str,
    visible: bool,
) -> Binding<GridCmd> {
    Binding {
        action: ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BINDINGS: [Binding<GridCmd>; 25] = [
    b("grid.up", Chord::key(KeyCode::Up), GridCmd::Up, "Up", true),
    b(
        "grid.down",
        Chord::key(KeyCode::Down),
        GridCmd::Down,
        "Down",
        true,
    ),
    b(
        "grid.left",
        Chord::key(KeyCode::Left),
        GridCmd::Left,
        "Left",
        true,
    ),
    b(
        "grid.right",
        Chord::key(KeyCode::Right),
        GridCmd::Right,
        "Right",
        true,
    ),
    b(
        "grid.up-vim",
        Chord::key(KeyCode::Char('k')),
        GridCmd::Up,
        "Up",
        false,
    ),
    b(
        "grid.down-vim",
        Chord::key(KeyCode::Char('j')),
        GridCmd::Down,
        "Down",
        false,
    ),
    b(
        "grid.left-vim",
        Chord::key(KeyCode::Char('h')),
        GridCmd::Left,
        "Left",
        false,
    ),
    b(
        "grid.right-vim",
        Chord::key(KeyCode::Char('l')),
        GridCmd::Right,
        "Right",
        false,
    ),
    b(
        "grid.page-up",
        Chord::key(KeyCode::PageUp),
        GridCmd::PageUp,
        "Page up",
        false,
    ),
    b(
        "grid.page-down",
        Chord::key(KeyCode::PageDown),
        GridCmd::PageDown,
        "Page down",
        false,
    ),
    b(
        "grid.row-start",
        Chord::key(KeyCode::Home),
        GridCmd::RowStart,
        "Row start",
        false,
    ),
    b(
        "grid.row-end",
        Chord::key(KeyCode::End),
        GridCmd::RowEnd,
        "Row end",
        false,
    ),
    b(
        "grid.first",
        Chord::with(KeyCode::Home, CTRL),
        GridCmd::First,
        "First row",
        false,
    ),
    b(
        "grid.last",
        Chord::with(KeyCode::End, CTRL),
        GridCmd::Last,
        "Last row",
        false,
    ),
    b(
        "grid.first-vim",
        Chord::key(KeyCode::Char('g')),
        GridCmd::First,
        "First row",
        false,
    ),
    b(
        "grid.last-vim",
        Chord::key(KeyCode::Char('G')),
        GridCmd::Last,
        "Last row",
        false,
    ),
    b(
        "grid.extend-up",
        Chord::with(KeyCode::Up, SHIFT),
        GridCmd::ExtendUp,
        "Extend up",
        false,
    ),
    b(
        "grid.extend-down",
        Chord::with(KeyCode::Down, SHIFT),
        GridCmd::ExtendDown,
        "Extend down",
        false,
    ),
    b(
        "grid.extend-left",
        Chord::with(KeyCode::Left, SHIFT),
        GridCmd::ExtendLeft,
        "Extend left",
        false,
    ),
    b(
        "grid.extend-right",
        Chord::with(KeyCode::Right, SHIFT),
        GridCmd::ExtendRight,
        "Extend right",
        false,
    ),
    b(
        "grid.toggle-row",
        Chord::key(KeyCode::Char(' ')),
        GridCmd::ToggleRow,
        "Select row",
        true,
    ),
    b(
        "grid.toggle-all",
        Chord::with(KeyCode::Char('a'), CTRL),
        GridCmd::ToggleAll,
        "All",
        false,
    ),
    b(
        "grid.activate",
        Chord::key(KeyCode::Enter),
        GridCmd::Activate,
        "Open",
        true,
    ),
    b(
        "grid.begin-edit",
        Chord::key(KeyCode::F(2)),
        GridCmd::BeginEdit,
        "Edit",
        false,
    ),
    b(
        "grid.copy",
        Chord::with(KeyCode::Char('c'), CTRL),
        GridCmd::Copy,
        "Copy",
        false,
    ),
];
const SORT_BINDING: Binding<GridCmd> = b(
    "grid.sort-current",
    Chord::key(KeyCode::Char('s')),
    GridCmd::SortCurrent,
    "Sort",
    false,
);

const fn with_sort_binding(base: [Binding<GridCmd>; 25]) -> [Binding<GridCmd>; 26] {
    let mut combined = [SORT_BINDING; 26];
    let mut index = 0;
    while index < base.len() {
        combined[index] = base[index];
        index += 1;
    }
    combined[25] = SORT_BINDING;
    combined
}

const TABLE_BINDINGS: [Binding<GridCmd>; 26] = with_sort_binding(BINDINGS);

pub const GRID_MAX_COLUMNS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridCursorError {
    UnknownRow(ItemKey),

    UnknownColumn(ColumnKey),

    Editing,
}

impl fmt::Display for GridCursorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownRow(_) => "grid row key is absent",
            Self::UnknownColumn(_) => "grid column key is absent",
            Self::Editing => "grid inline edit is active",
        })
    }
}

impl core::error::Error for GridCursorError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidthSample {
    rows: usize,
    percentile: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthSampleError {
    RowsOutOfRange,

    PercentileOutOfRange,
}

impl fmt::Display for WidthSampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RowsOutOfRange => "width sample rows must be between 1 and 200",
            Self::PercentileOutOfRange => "width percentile must be between 0 and 100",
        })
    }
}
impl std::error::Error for WidthSampleError {}

impl WidthSample {
    pub const fn new(rows: usize, percentile: u8) -> Result<Self, WidthSampleError> {
        if rows == 0 || rows > 200 {
            return Err(WidthSampleError::RowsOutOfRange);
        }
        if percentile > 100 {
            return Err(WidthSampleError::PercentileOutOfRange);
        }
        Ok(Self { rows, percentile })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SampledWidth {
    key: ColumnKey,
    natural: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridGutter {
    #[default]
    Compact,

    Detailed {
        row_numbers: bool,

        min_digits: u16,
    },
}
impl GridGutter {
    fn widths(self, rows: usize) -> (u16, u16) {
        match self {
            Self::Compact => (2, 0),
            Self::Detailed {
                row_numbers: false, ..
            } => (3, 0),
            Self::Detailed { min_digits, .. } => {
                let digits = width(Num::new(rows.max(1)).as_str()).max(min_digits);
                (digits.saturating_add(4), digits)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridColumnFit {
    #[default]
    Whole,

    CompleteWithPreview {
        min_width: u16,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridHeaderSizing {
    #[default]
    Content,

    Minimum {
        padding: u16,

        cap: u16,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridSortIndicator {
    #[default]
    Always,

    ActiveOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridOverflowIndicator {
    #[default]
    Count,

    Ellipsis,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum TypeAheadColumn {
    #[default]
    Cursor,
    Index(usize),
    Key(ColumnKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TypeAheadConfig {
    column: TypeAheadColumn,
    identity: Option<ColumnKey>,
    timeout: Duration,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GridState {
    sampled_widths: [Option<SampledWidth>; GRID_MAX_COLUMNS],
    sampled_len: usize,

    fetch_row: Option<FetchRow>,

    core: CollectionCore,

    col: Option<ColumnKey>,
    col_index: usize,

    anchor: Option<RangeAnchor>,

    col_offset: usize,
    pending_column: Option<ColumnKey>,

    edit: Option<(ItemKey, ColumnKey)>,

    editor: TextInputState,

    type_ahead: TypeAhead,
    type_ahead_config: Option<TypeAheadConfig>,

    sort: Option<(ColumnKey, SortDir)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RangeAnchor {
    Cell(ItemKey, ColumnKey),
    Fetch(usize, ColumnKey),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FetchRow {
    boundary: usize,
}

impl Default for GridState {
    fn default() -> Self {
        let mut editor = TextInputState::default();
        editor.set_sensitive(false);
        Self {
            sampled_widths: [None; GRID_MAX_COLUMNS],
            sampled_len: 0,
            fetch_row: None,
            core: CollectionCore::default(),
            col: None,
            col_index: 0,
            anchor: None,
            col_offset: 0,
            pending_column: None,
            edit: None,
            editor,
            type_ahead: TypeAhead::default(),
            type_ahead_config: None,
            sort: None,
        }
    }
}

impl GridState {
    pub(crate) fn invalidate_model(&mut self) {
        self.core.invalidate();
        self.type_ahead.reset();
    }

    pub(crate) fn set_sort(&mut self, sort: Option<(ColumnKey, SortDir)>) {
        self.sort = sort;
    }

    pub fn sampled_column_width(&self, key: ColumnKey) -> Option<u16> {
        self.sampled_width(key).map(|sample| sample.natural)
    }

    pub fn clear_sampled_widths(&mut self) {
        self.sampled_widths.fill(None);
        self.sampled_len = 0;
    }

    fn sampled_width(&self, key: ColumnKey) -> Option<SampledWidth> {
        self.sampled_widths
            .iter()
            .take(self.sampled_len)
            .flatten()
            .find(|sample| sample.key == key)
            .copied()
    }

    pub const fn on_fetch_row(&self) -> bool {
        self.fetch_row.is_some()
    }

    pub const fn cursor(&self) -> Option<(ItemKey, ColumnKey)> {
        if self.on_fetch_row() {
            return None;
        }
        match (self.core.cursor(), self.col) {
            (Some(row), Some(col)) => Some((row, col)),
            _ => None,
        }
    }

    pub const fn selected_rows(&self) -> &KeySet {
        self.core.checked()
    }

    pub const fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    pub const fn edit_cell(&self) -> Option<(ItemKey, ColumnKey)> {
        self.edit
    }

    pub fn edit_draft(&self) -> Option<&str> {
        self.edit.and_then(|_| self.editor.draft_text())
    }

    pub const fn edit_error(&self) -> Option<&crate::validate::FieldError> {
        self.editor.error()
    }

    pub fn cancel_edit(&mut self) -> bool {
        let changed = self.edit.is_some() || self.editor.error().is_some();
        if changed {
            self.cancel_editor();
        }
        changed
    }

    pub fn set_scroll_reveal(&mut self, row: usize) {
        self.core.scroll_mut().ensure_visible_on_next_layout(row);
    }

    pub const fn col_offset(&self) -> usize {
        self.col_offset
    }

    pub(crate) fn set_col_offset(&mut self, offset: usize) {
        self.col_offset = offset;
    }

    pub fn visible_rows(&self, total: usize) -> core::ops::Range<usize> {
        let scroll = self.core.scroll();
        let start = scroll.offset().min(total);
        start..start.saturating_add(scroll.viewport_len()).min(total)
    }

    pub const fn rows_overflow(&self) -> bool {
        self.core.scroll().overflows()
    }

    fn set_cursor(&mut self, row: usize, key: ItemKey, col_index: usize, col: ColumnKey) {
        self.fetch_row = None;
        self.core.set_cursor(row, key);
        self.col_index = col_index;
        self.col = Some(col);
    }

    fn cancel_editor(&mut self) {
        self.edit = None;
        self.editor.cancel();
        self.editor.set_error(None);
    }
}

impl Reconcile for GridState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let r = self.core.reconcile(len, &key);
        if r != Reconciliation::Unchanged {
            self.type_ahead.reset();
        }
        if let Some(RangeAnchor::Cell(a, _)) = self.anchor
            && !(0..len).any(|i| key(i) == a)
        {
            self.anchor = None;
        }
        if let Some((e, _)) = self.edit
            && !(0..len).any(|i| key(i) == e)
        {
            self.cancel_editor();
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
        self.type_ahead.reset();
    }
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    width: [u16; GRID_MAX_COLUMNS],

    x: [u16; GRID_MAX_COLUMNS],

    shown: [bool; GRID_MAX_COLUMNS],
    complete: [bool; GRID_MAX_COLUMNS],
    offset: usize,

    n: usize,

    hidden_left: usize,
    hidden_right: usize,

    body: Rect,

    content_x: u16,
    gutter_width: u16,
    number_width: u16,
    columns_area: Rect,
}

#[derive(Clone, Copy)]
struct GutterRow<'a> {
    data: Option<(ItemKey, usize)>,
    decor: RowDecor<'a>,
    flags: StateFlags,
    style: PaintStyle,
}

struct RowPaint<'a, M: ?Sized> {
    content: Rect,
    geometry: &'a Geometry,
    state: &'a GridState,
    model: &'a M,
    cursor: (usize, usize),
    range: Option<((usize, usize), (usize, usize))>,
    live: StateFlags,
}

impl Geometry {
    const fn empty(body: Rect) -> Self {
        Geometry {
            width: [0; GRID_MAX_COLUMNS],
            x: [0; GRID_MAX_COLUMNS],
            shown: [false; GRID_MAX_COLUMNS],
            complete: [false; GRID_MAX_COLUMNS],
            offset: 0,
            n: 0,
            hidden_left: 0,
            hidden_right: 0,
            body,
            content_x: body.x,
            gutter_width: 0,
            number_width: 0,
            columns_area: body,
        }
    }

    fn cell(&self, i: usize, y: u16) -> Rect {
        if !self.shown.get(i).copied().unwrap_or(false) {
            return Rect::ZERO;
        }
        Rect {
            x: self.x.get(i).copied().unwrap_or(0),
            y,
            width: self.width.get(i).copied().unwrap_or(0),
            height: 1,
        }
        .intersection(Rect {
            y,
            height: 1,
            ..self.columns_area
        })
    }

    fn register_cell_part(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        part: PartRef,
        column: usize,
        area: Rect,
    ) {
        if self.complete.get(column).copied().unwrap_or(false) {
            ui.register_part(owner, part, area);
        } else {
            ui.register_decor(owner, part, area);
        }
    }

    fn gutter_part(&self, part: Part, y: u16) -> Rect {
        let (offset, width) = match part {
            Part::GUTTER => (0, 1),
            Part::MARKER => (1, 1),
            Part::CHANGE => (2, 1),
            Part::ROW_NUMBER => (3, self.number_width),
            _ => (0, 0),
        };
        Rect {
            x: self.body.x.saturating_add(offset),
            y,
            width,
            height: 1,
        }
        .intersection(Rect {
            x: self.body.x,
            y,
            width: self.gutter_width,
            height: 1,
        })
    }

    fn gutter_row(&self, y: u16) -> Rect {
        Rect {
            x: self.body.x,
            y,
            width: self.gutter_width,
            height: 1,
        }
    }

    fn column_at(&self, x: u16) -> Option<usize> {
        (0..self.n).find(|&i| {
            let r = self.cell(i, self.body.y);
            self.complete.get(i).copied().unwrap_or(false)
                && r.width > 0
                && x >= r.x
                && x < r.x.saturating_add(r.width)
        })
    }
}

pub struct Grid<'a> {
    id: Id,
    columns: &'a [Column<'a>],
    nav: NavUnit,
    row_nav_scrolls_columns: bool,
    row_nav_scroll_limit: Option<usize>,
    sort_current_column: bool,
    column_gap: u16,
    gutter: GridGutter,
    right_reserve: u16,
    part_defaults: &'a [(Part, StylePatch)],
    header_prefixes: &'a [(ColumnKey, GlyphRole, Role)],
    column_fit: GridColumnFit,
    header_sizing: GridHeaderSizing,
    sort_indicator: GridSortIndicator,
    cell_tone_styles: &'a [(Role, StylePatch)],
    overflow_indicator: GridOverflowIndicator,
    disabled: bool,
    type_ahead: bool,
    type_ahead_column: TypeAheadColumn,
    type_ahead_timeout: Duration,
    fetch_on_activate: bool,
    fetch_label: &'a str,
    fetch_glyph: GlyphRole,
    fetch_activity: Option<usize>,
    select_mode: SelectMode,
    empty: Option<EmptyState<'a>>,
    actions: Option<SlotFn<'a>>,
    cell: Option<CellRenderer<'a>>,
    blur: BlurPolicy,
    ov: PartStyle<'a>,
}

impl fmt::Debug for Grid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Grid")
            .field("id", &self.id)
            .field("columns", &self.columns.len())
            .field("nav", &self.nav)
            .field("select_mode", &self.select_mode)
            .field("empty", &self.empty)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> Grid<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::HEADER,
        Part::ROW,
        Part::CELL,
        Part::TEXT,
        Part::TRACK,
        Part::THUMB,
        Part::OVERFLOW,
        Part::EMPTY,
        Part::ACTIONS,
        Part::GUTTER,
        Part::MARKER,
        Part::CHANGE,
        Part::ROW_NUMBER,
        Part::ICON,
    ];

    pub const fn new(id: Id, columns: &'a [Column<'a>]) -> Self {
        Grid {
            id,
            columns,
            nav: NavUnit::Cell,
            row_nav_scrolls_columns: false,
            row_nav_scroll_limit: None,
            sort_current_column: false,
            column_gap: 1,
            gutter: GridGutter::Compact,
            right_reserve: 0,
            part_defaults: &[],
            header_prefixes: &[],
            column_fit: GridColumnFit::Whole,
            header_sizing: GridHeaderSizing::Content,
            sort_indicator: GridSortIndicator::Always,
            cell_tone_styles: &[],
            overflow_indicator: GridOverflowIndicator::Count,
            disabled: false,
            type_ahead: false,
            type_ahead_column: TypeAheadColumn::Cursor,
            type_ahead_timeout: TypeAhead::DEFAULT_TIMEOUT,
            fetch_on_activate: false,
            fetch_label: "more",
            fetch_glyph: GlyphRole::MoreRows,
            fetch_activity: None,
            select_mode: SelectMode::Single,
            empty: None,
            actions: None,
            cell: None,
            blur: BlurPolicy::CommitAndValidate,
            ov: PartStyle::new(),
        }
    }

    pub fn sample_column_widths<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        policy: WidthSample,
    ) {
        self.assert_distinct_column_keys();
        let len = model.row_count().min(policy.rows);
        let mut sampled = [None; GRID_MAX_COLUMNS];
        let mut widths = [0u16; 200];
        for (i, (column, sampled_slot)) in self
            .columns
            .iter()
            .take(self.column_count())
            .zip(sampled.iter_mut())
            .enumerate()
        {
            for (row, slot) in widths.iter_mut().take(len).enumerate() {
                *slot = if let Some(cell) = model.cell(row, i) {
                    width(cell.text)
                } else {
                    0
                };
            }
            let Some(values) = widths.get_mut(..len) else {
                return;
            };
            values.sort_unstable();
            let index = len.saturating_mul(usize::from(policy.percentile)) / 100;
            let natural = values
                .get(index.min(len.saturating_sub(1)))
                .copied()
                .unwrap_or(0);
            *sampled_slot = Some(SampledWidth {
                key: column.key,
                natural,
            });
        }
        st.sampled_widths = sampled;
        st.sampled_len = self.column_count();
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub const fn editor_id(&self) -> Id {
        self.id.part(Part::TEXT)
    }

    #[must_use]
    pub const fn nav(mut self, u: NavUnit) -> Self {
        self.nav = u;
        self
    }

    #[must_use]
    pub(crate) const fn row_nav_scrolls_columns(mut self, enabled: bool) -> Self {
        self.row_nav_scrolls_columns = enabled;
        self
    }

    pub(crate) const fn row_nav_scroll_limit(mut self, limit: usize) -> Self {
        self.row_nav_scroll_limit = Some(limit);
        self
    }

    #[must_use]
    pub(crate) const fn sort_current_column(mut self, enabled: bool) -> Self {
        self.sort_current_column = enabled;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub const fn type_ahead(mut self, enabled: bool) -> Self {
        self.type_ahead = enabled;
        self
    }

    #[must_use]
    pub const fn type_ahead_column(mut self, column: Option<usize>) -> Self {
        self.type_ahead_column = match column {
            Some(index) => TypeAheadColumn::Index(index),
            None => TypeAheadColumn::Cursor,
        };
        self
    }

    #[must_use]
    pub const fn type_ahead_column_key(mut self, key: Option<ColumnKey>) -> Self {
        self.type_ahead_column = match key {
            Some(key) => TypeAheadColumn::Key(key),
            None => TypeAheadColumn::Cursor,
        };
        self
    }

    #[must_use]
    pub const fn type_ahead_timeout(mut self, timeout: Duration) -> Self {
        self.type_ahead_timeout = timeout;
        self
    }

    #[must_use]
    pub const fn fetch_on_activate(mut self, enabled: bool) -> Self {
        self.fetch_on_activate = enabled;
        self
    }

    #[must_use]
    pub const fn fetch_label(mut self, label: &'a str) -> Self {
        self.fetch_label = label;
        self
    }

    #[must_use]
    pub const fn fetch_glyph(mut self, glyph: GlyphRole) -> Self {
        self.fetch_glyph = glyph;
        self
    }

    #[must_use]
    pub const fn fetch_activity(mut self, frame: Option<usize>) -> Self {
        self.fetch_activity = frame;
        self
    }

    #[must_use]
    pub const fn part_defaults(mut self, defaults: &'a [(Part, StylePatch)]) -> Self {
        self.part_defaults = defaults;
        self
    }

    #[must_use]
    pub const fn gutter(mut self, gutter: GridGutter) -> Self {
        self.gutter = gutter;
        self
    }

    #[must_use]
    pub const fn header_prefixes(mut self, prefixes: &'a [(ColumnKey, GlyphRole, Role)]) -> Self {
        self.header_prefixes = prefixes;
        self
    }

    fn header_prefix(&self, key: ColumnKey) -> Option<(GlyphRole, Role)> {
        self.header_prefixes
            .iter()
            .rev()
            .find_map(|(column, glyph, tone)| (*column == key).then_some((*glyph, *tone)))
    }

    #[must_use]
    pub const fn right_reserve(mut self, cells: u16) -> Self {
        self.right_reserve = cells;
        self
    }

    #[must_use]
    pub const fn column_gap(mut self, gap: u16) -> Self {
        self.column_gap = gap;
        self
    }

    #[must_use]
    pub const fn column_fit(mut self, policy: GridColumnFit) -> Self {
        self.column_fit = policy;
        self
    }

    #[must_use]
    pub const fn header_sizing(mut self, policy: GridHeaderSizing) -> Self {
        self.header_sizing = policy;
        self
    }

    #[must_use]
    pub const fn sort_indicator(mut self, policy: GridSortIndicator) -> Self {
        self.sort_indicator = policy;
        self
    }

    #[must_use]
    pub const fn cell_tone_styles(mut self, styles: &'a [(Role, StylePatch)]) -> Self {
        self.cell_tone_styles = styles;
        self
    }

    #[must_use]
    pub const fn overflow_indicator(mut self, policy: GridOverflowIndicator) -> Self {
        self.overflow_indicator = policy;
        self
    }

    #[must_use]
    pub const fn select_mode(mut self, m: SelectMode) -> Self {
        self.select_mode = m;
        self
    }

    #[must_use]
    pub const fn cell(mut self, render: &'a dyn Fn(GridCell<'_>, &mut CellUi<'_>)) -> Self {
        self.cell = Some(render);
        self
    }

    #[must_use]
    pub const fn blur(mut self, policy: BlurPolicy) -> Self {
        self.blur = policy;
        self
    }

    #[must_use]
    pub const fn empty(mut self, e: EmptyState<'a>) -> Self {
        self.empty = Some(e);
        self
    }

    #[must_use]
    pub const fn actions_slot(mut self, f: SlotFn<'a>) -> Self {
        self.actions = Some(f);
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

    fn bar(&self) -> ScrollRegion<'a> {
        let mut r = ScrollRegion::new(self.id).patch_part(self.ov.parts);
        if let Some(p) = self.ov.patch {
            r = r.patch(p);
        }
        if let Some((part, f)) = self.ov.slot {
            r = r.slot(part, f);
        }
        r
    }

    fn column_count(&self) -> usize {
        self.columns.len().min(GRID_MAX_COLUMNS)
    }

    fn sync_type_ahead(&self, state: &mut GridState) {
        if !self.type_ahead || self.disabled {
            state.type_ahead = TypeAhead::default();
            state.type_ahead_config = None;
            return;
        }
        let identity = match self.type_ahead_column {
            TypeAheadColumn::Cursor => self.col_key(self.cursor_col(state)),
            TypeAheadColumn::Index(index) => self.col_key(index),
            TypeAheadColumn::Key(key) => self.col_index(key).map(|_| key),
        };
        let config = TypeAheadConfig {
            column: self.type_ahead_column,
            identity,
            timeout: self.type_ahead_timeout,
        };
        if state.type_ahead_config != Some(config) {
            state.type_ahead = TypeAhead::with_timeout(self.type_ahead_timeout);
            state.type_ahead_config = Some(config);
        }
    }

    fn type_ahead_column_index(&self, state: &GridState) -> Option<usize> {
        match self.type_ahead_column {
            TypeAheadColumn::Cursor => {
                let column = self.cursor_col(state);
                (column < self.column_count()).then_some(column)
            }
            TypeAheadColumn::Index(column) => (column < self.column_count()).then_some(column),
            TypeAheadColumn::Key(key) => self.col_index(key),
        }
    }

    fn type_ahead_key<M: GridModel + ?Sized>(
        &self,
        cx: &Cx<'_>,
        state: &mut GridState,
        model: &M,
        key: crate::event::Key,
        len: usize,
        acc: &mut Acc<GridAction>,
    ) {
        if !self.type_ahead || state.is_editing() {
            return;
        }
        let Some(character) = TypeAhead::char_of(&key) else {
            return;
        };
        let Some(search_column) = self.type_ahead_column_index(state) else {
            state.type_ahead.reset();
            acc.consumed();
            return;
        };
        let from = if state.on_fetch_row() || len == 0 {
            0
        } else {
            state.core.cursor_index().min(len.saturating_sub(1))
        };
        let found = {
            let prefix = state
                .type_ahead
                .feed_at_duration(character, cx.now().as_duration());
            find_type_ahead_row(prefix, len, from, search_column, model)
        };
        if let Some(row) = found {
            let cursor_column = self.cursor_col(state);
            self.move_to(state, model, row, cursor_column, false, acc);
        }
        acc.consumed();
    }

    fn assert_distinct_column_keys(&self) {
        #[cfg(debug_assertions)]
        {
            let shown = self.columns.get(..self.column_count()).unwrap_or(&[]);
            for (i, a) in shown.iter().enumerate() {
                for b in shown.get(i.saturating_add(1)..).unwrap_or(&[]) {
                    assert!(
                        a.key != b.key,
                        "grid {:?}: columns {:?} and {:?} share {:?}",
                        self.id,
                        a.title,
                        b.title,
                        a.key
                    );
                }
            }
        }
    }

    fn col_index(&self, key: ColumnKey) -> Option<usize> {
        self.columns
            .get(..self.column_count())
            .and_then(|cs| cs.iter().position(|c| c.key == key))
    }

    fn col_key(&self, i: usize) -> Option<ColumnKey> {
        self.columns.get(i).map(|c| c.key)
    }

    fn cursor_col(&self, st: &GridState) -> usize {
        let column_count = self.column_count();
        st.col
            .and_then(|k| self.col_index(k))
            .unwrap_or_else(|| st.col_index.min(column_count.saturating_sub(1)))
    }

    fn chrome(&self, area: Rect, reason: Option<&str>) -> (Rect, Rect, Rect, Rect) {
        let header = Rect {
            height: u16::from(!self.columns.is_empty()),
            ..area
        };
        let rest = Rect {
            y: area.y.saturating_add(header.height),
            height: area.height.saturating_sub(header.height),
            ..area
        };
        let note = Rect {
            height: u16::from(reason.is_some()).min(rest.height),
            ..rest
        };
        let rest = Rect {
            y: rest.y.saturating_add(note.height),
            height: rest.height.saturating_sub(note.height),
            ..rest
        };
        let bar = Rect {
            y: rest.bottom().saturating_sub(1),
            height: u16::from(self.actions.is_some()).min(rest.height),
            ..rest
        };
        let body = Rect {
            height: rest.height.saturating_sub(bar.height),
            ..rest
        };
        (header, note, body, bar)
    }

    fn column_widths<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
        rows: core::ops::Range<usize>,
    ) -> [u16; GRID_MAX_COLUMNS] {
        let mut widths = [0; GRID_MAX_COLUMNS];
        for (i, c) in self.columns.iter().enumerate().take(self.column_count()) {
            let mut w = width(c.title).saturating_add(if self.header_prefix(c.key).is_some() {
                2
            } else {
                0
            });
            let mut has_actions = false;
            if let Some(b) = c.badge {
                w = w.saturating_add(width(b)).saturating_add(1);
            }
            if let Some(s) = c.subtitle {
                w = w.max(width(s));
            }
            let header_width = match self.header_sizing {
                GridHeaderSizing::Content => w,
                GridHeaderSizing::Minimum { padding, .. } => w.saturating_add(padding),
            };
            let sample = st.sampled_width(c.key);
            if let Some(sample) = sample {
                w = w.max(sample.natural);
            }

            for r in rows.clone() {
                if r >= model.row_count() {
                    break;
                }
                if let Some(cell) = model.cell(r, i) {
                    if sample.is_none() {
                        w = w.max(width(cell.text));
                    }
                    has_actions |= !model.actions(r, i).is_empty();
                }
            }
            if c.prefix_glyph.is_some() {
                w = w.saturating_add(2);
            }
            if has_actions {
                w = w.saturating_add(2);
            }
            if let Some(slot) = widths.get_mut(i) {
                *slot = match self.header_sizing {
                    GridHeaderSizing::Content => {
                        w.clamp(c.min_width.max(1), c.max_width.max(c.min_width).max(1))
                    }
                    GridHeaderSizing::Minimum { cap, .. } => {
                        let max = c.max_width.max(header_width.min(cap)).max(1);
                        w.max(header_width).clamp(c.min_width.min(max).max(1), max)
                    }
                };
            }
        }
        widths
    }

    fn input_content<M: GridModel + ?Sized>(&self, area: Rect, model: &M) -> Rect {
        let (_, _, mut body, _) = self.chrome(area, model.read_only_reason());
        let total = model
            .row_count()
            .saturating_add(usize::from(model.has_more()));
        if (self.column_fit != GridColumnFit::Whole
            || self.gutter != GridGutter::Compact
            || self.right_reserve > 0)
            && total > usize::from(body.height)
        {
            body.width = body.width.saturating_sub(1);
        }
        body
    }

    fn geometry<M: GridModel + ?Sized>(
        &self,
        body: Rect,
        st: &GridState,
        model: &M,
        rows: core::ops::Range<usize>,
    ) -> Geometry {
        if body.is_empty() {
            return Geometry::empty(body);
        }
        let (gutter_width, number_width) = self.gutter.widths(model.row_count());
        if body.width <= gutter_width.saturating_add(self.right_reserve) {
            return self.place_columns(
                body,
                [0; GRID_MAX_COLUMNS],
                st.col_offset,
                gutter_width,
                number_width,
            );
        }
        let widths = self.column_widths(st, model, rows);
        let mut g = self.place_columns(body, widths, st.col_offset, gutter_width, number_width);
        if self.column_fit == GridColumnFit::Whole {
            return g;
        }
        let target = st
            .pending_column
            .or_else(|| {
                (self.cursor_col(st) != st.col_index)
                    .then_some(st.col)
                    .flatten()
            })
            .and_then(|key| self.col_index(key));
        if let Some(target) = target
            && !self.columns.get(target).is_some_and(|c| c.sticky)
        {
            let ordinal = self
                .columns
                .iter()
                .take(target)
                .filter(|c| !c.sticky)
                .count();
            let count = self
                .columns
                .iter()
                .enumerate()
                .take(g.n)
                .filter(|(i, c)| !c.sticky && g.complete.get(*i).copied().unwrap_or(false))
                .count();
            if count > 0 {
                let offset = if ordinal < g.offset {
                    ordinal
                } else if ordinal >= g.offset.saturating_add(count) {
                    ordinal.saturating_add(1).saturating_sub(count)
                } else {
                    g.offset
                };
                if offset != g.offset {
                    g = self.place_columns(body, widths, offset, gutter_width, number_width);
                }
            }
        }

        while g.offset > 0 {
            let previous = self.place_columns(
                body,
                widths,
                g.offset.saturating_sub(1),
                gutter_width,
                number_width,
            );
            if previous.hidden_right == 0 {
                g = previous;
            } else {
                break;
            }
        }

        g
    }

    fn apply_column_geometry(&self, st: &mut GridState, g: &Geometry) {
        if self.column_fit == GridColumnFit::Whole || g.body.is_empty() {
            return;
        }
        st.col_offset = g.offset;
        st.col_index = self.cursor_col(st);
        let target_complete = st
            .pending_column
            .and_then(|key| self.col_index(key))
            .is_some_and(|column| g.complete.get(column).copied().unwrap_or(false));
        let movable_viewport =
            self.columns.iter().enumerate().any(|(column, spec)| {
                !spec.sticky && g.complete.get(column).copied().unwrap_or(false)
            });
        if target_complete || movable_viewport {
            st.pending_column = None;
        }
    }

    fn place_columns(
        &self,
        body: Rect,
        widths: [u16; GRID_MAX_COLUMNS],
        offset: usize,
        gutter_width: u16,
        number_width: u16,
    ) -> Geometry {
        let mut g = Geometry::empty(body);
        g.offset = offset;
        g.n = self.column_count();
        g.gutter_width = gutter_width.min(body.width);
        g.number_width = number_width;
        g.content_x = body.x.saturating_add(g.gutter_width);
        g.columns_area = Rect {
            x: g.content_x,
            width: body
                .width
                .saturating_sub(gutter_width)
                .saturating_sub(self.right_reserve),
            ..body
        };
        if g.n == 0 || body.is_empty() {
            return g;
        }
        let avail = g.columns_area.width;
        if avail == 0 {
            g.hidden_right = g.n;
            return g;
        }
        g.width = widths;

        let gap = self.column_gap;
        let mut x = g.content_x;
        let mut used = 0u16;
        for i in 0..g.n {
            if !self.columns.get(i).is_some_and(|c| c.sticky) {
                continue;
            }
            if used >= avail {
                g.hidden_right = g.hidden_right.saturating_add(1);
                continue;
            }
            let w = g.width.get(i).copied().unwrap_or(0);
            if let (Some(px), Some(sh)) = (g.x.get_mut(i), g.shown.get_mut(i)) {
                *px = x;
                *sh = true;
            }
            if let Some(complete) = g.complete.get_mut(i) {
                *complete = true;
            }
            x = x.saturating_add(w).saturating_add(gap);
            used = used.saturating_add(w).saturating_add(gap);
        }
        let first = offset;
        let mut seen = 0usize;
        let mut last_shown = 0usize;
        let mut window_closed = false;
        for i in 0..g.n {
            if self.columns.get(i).is_some_and(|c| c.sticky) {
                continue;
            }
            if seen < first {
                seen = seen.saturating_add(1);
                g.hidden_left = g.hidden_left.saturating_add(1);
                continue;
            }
            let w = g.width.get(i).copied().unwrap_or(0);
            if window_closed || used >= avail || (used.saturating_add(w) > avail && last_shown > 0)
            {
                g.hidden_right = g.hidden_right.saturating_add(1);
                if let GridColumnFit::CompleteWithPreview { min_width } = self.column_fit {
                    if !window_closed
                        && avail.saturating_sub(used) >= min_width.max(1)
                        && let (Some(px), Some(sh)) = (g.x.get_mut(i), g.shown.get_mut(i))
                    {
                        *px = x;
                        *sh = true;
                    }
                    window_closed = true;
                }
                continue;
            }
            if let (Some(px), Some(sh)) = (g.x.get_mut(i), g.shown.get_mut(i)) {
                *px = x;
                *sh = true;
            }
            if let Some(complete) = g.complete.get_mut(i) {
                *complete = true;
            }
            last_shown = last_shown.saturating_add(1);
            x = x.saturating_add(w).saturating_add(gap);
            used = used.saturating_add(w).saturating_add(gap);
        }
        g
    }
}

fn paint_aligned(ui: &mut Ui<'_>, area: Rect, text: &str, align: Align, style: PaintStyle) {
    if area.is_empty() || text.is_empty() {
        return;
    }
    let w = width(text);
    if w > area.width {
        let head = Rect {
            width: area.width.saturating_sub(1),
            ..area
        };
        let used = ui.paint_str(head, text, style);
        let tail = Rect {
            x: area.x.saturating_add(used),
            width: area.width.saturating_sub(used),
            ..area
        };
        ui.glyph(tail, GlyphRole::Ellipsis, style);
        return;
    }
    let pad = area.width.saturating_sub(w);
    let off = match align {
        Align::Left => 0,
        Align::Center => pad / 2,
        Align::Right => pad,
    };
    let at = Rect {
        x: area.x.saturating_add(off),
        width: area.width.saturating_sub(off),
        ..area
    };
    ui.paint_str(at, text, style);
}

fn apply_style_delta(ui: &Ui<'_>, base: PaintStyle, delta: StylePatch) -> PaintStyle {
    if delta.is_empty() {
        return base;
    }
    let top = crate::theme::resolve::bind(ui.theme_ref(), delta, None, ui.surface()).style;
    base.patch(top)
}

fn preserve_default_clears(
    defaults: StylePatch,
    resolved: PaintStyle,
    mut explicit: StylePatch,
) -> StylePatch {
    if defaults.fg == Slot::Clear && resolved.fg.is_none() && explicit.fg == Slot::Inherit {
        explicit.fg = Slot::Clear;
    }
    if defaults.bg == Slot::Clear && resolved.bg.is_none() && explicit.bg == Slot::Inherit {
        explicit.bg = Slot::Clear;
    }
    if defaults.underline == Slot::Clear
        && resolved.underline_color.is_none()
        && explicit.underline == Slot::Inherit
    {
        explicit.underline = Slot::Clear;
    }
    explicit
}

fn active_cell_style(
    ui: &Ui<'_>,
    row_style: PaintStyle,
    cell: crate::theme::ResolvedStyle,
    strike: bool,
) -> PaintStyle {
    let mut active_style = ui
        .surface_style()
        .patch(cell.resolved.style)
        .remove_modifier(
            row_style
                .as_style()
                .add_modifier
                .difference(cell.resolved.style.as_style().add_modifier),
        );
    if row_style.as_style().underline_color.is_some()
        && cell.resolved.style.as_style().underline_color.is_none()
    {
        active_style = active_style.underline_color(ratatui_core::style::Color::Reset);
    }
    if strike && !cell.explicit.remove.contains(Modifier::CROSSED_OUT) {
        active_style = active_style.add_modifier(Modifier::CROSSED_OUT);
    }
    active_style
}

fn inherited_style_defaults(style: PaintStyle, explicit: StylePatch) -> StyleDefaults<'static> {
    let raw = style.as_style();
    let mut base = StylePatch::new();
    if let Some((role, _)) = style.fg_role {
        base = base.set_fg(role);
    } else if let Some(color) = raw.fg {
        base = base.set_fg(Role::Custom(color));
    } else if matches!(explicit.fg, Slot::Clear) {
        base = base.clear_fg();
    }
    if let Some((role, _)) = style.bg_role {
        base = base.set_bg(role);
    } else if let Some(color) = raw.bg {
        base = base.set_bg(Role::Custom(color));
    } else if matches!(explicit.bg, Slot::Clear) {
        base = base.clear_bg();
    }
    if let Some((role, _)) = style.underline_role() {
        base = base.set_underline(role);
    } else if let Some(color) = raw.underline_color {
        base = base.set_underline(Role::Custom(color));
    } else if matches!(explicit.underline, Slot::Clear) {
        base.underline = Slot::Clear;
    }
    base = base.add(raw.add_modifier).remove(raw.sub_modifier);
    StyleDefaults::new(base)
}

fn push_grouped(out: &mut String, n: usize) {
    let mut digits = [0u8; 20];
    let mut len = 0usize;
    let mut v = n;
    loop {
        if let Some(d) = digits.get_mut(len) {
            *d = b'0'.saturating_add((v % 10) as u8);
        }
        len = len.saturating_add(1);
        v /= 10;
        if v == 0 || len == digits.len() {
            break;
        }
    }
    for i in (0..len).rev() {
        if let Some(d) = digits.get(i) {
            out.push(char::from(*d));
        }
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Num {
    buf: [u8; 20],
    len: usize,
}

impl Num {
    fn new(n: usize) -> Self {
        let mut digits = [0u8; 20];
        let mut len = 0usize;
        let mut v = n;
        loop {
            if let Some(d) = digits.get_mut(len) {
                *d = b'0'.saturating_add((v % 10) as u8);
            }
            len = len.saturating_add(1);
            v /= 10;
            if v == 0 || len == digits.len() {
                break;
            }
        }
        let mut buf = [0u8; 20];
        for i in 0..len {
            if let (Some(dst), Some(src)) = (
                buf.get_mut(i),
                digits.get(len.saturating_sub(1).saturating_sub(i)),
            ) {
                *dst = *src;
            }
        }
        Num { buf, len }
    }

    fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EditCause {
    Activate,

    Explicit,
}

#[derive(Clone, Copy, Debug)]
struct EditRequest {
    row: usize,
    col: usize,
    cause: EditCause,
}

#[derive(Clone, Copy, Debug, Default)]
struct Pending {
    edit: Option<EditRequest>,
}

enum Begin {
    Inline(String),
    Cycle,
    External,
    Refuse(String),
}

impl Grid<'_> {
    fn scroll_for_view(st: &GridState, viewport: usize) -> crate::scroll::ScrollState {
        let mut scroll = *st.core.scroll();
        if viewport != scroll.viewport_len()
            && let Some(fetch) = st.fetch_row
        {
            scroll.ensure_visible_on_next_layout(fetch.boundary);
        }
        scroll
    }

    fn window(st: &GridState, content: Rect, len: usize) -> core::ops::Range<usize> {
        let scroll = Self::scroll_for_view(st, usize::from(content.height));
        let view = ScrollRegion::view(&scroll, content, len);
        let r = view.visible_range();
        r.start.min(len)..r.end.min(len)
    }

    fn row_index<M: GridModel + ?Sized>(model: &M, key: ItemKey, hint: usize) -> Option<usize> {
        let len = model.row_count();
        if hint < len && model.row_key(hint) == key {
            return Some(hint);
        }
        (0..len).find(|&i| model.row_key(i) == key)
    }

    pub fn rows_label<M: GridModel + ?Sized>(
        &self,
        f: &impl FrameRead,
        st: &GridState,
        model: &M,
    ) -> String {
        let len = model.row_count();
        let rows = f
            .layout(self.id)
            .map_or_else(|| st.core.scroll().viewport_len(), |l| l.viewport_len);
        let content = Rect {
            height: rows.min(usize::from(u16::MAX)) as u16,
            ..Rect::ZERO
        };
        let w = Self::window(st, content, len);
        let mut s = String::new();
        s.push_str("rows ");
        push_grouped(&mut s, w.start.saturating_add(1).min(len));
        s.push('\u{2013}');
        push_grouped(&mut s, w.end);
        s.push_str(" of ");
        push_grouped(&mut s, len);
        match model.total() {
            RowTotal::Exact(t) | RowTotal::Estimated(t) if t > len => {
                s.push_str(" loaded \u{b7} ");
                push_grouped(&mut s, t);
                s.push_str(" total");
            }
            _ => {}
        }
        s
    }

    pub fn cols_label<M: GridModel + ?Sized>(
        &self,
        f: &impl FrameRead,
        st: &GridState,
        model: &M,
    ) -> Option<String> {
        let area = f.area(self.id)?;
        let len = model.row_count();
        let (_, _, body, _) = self.chrome(area, model.read_only_reason());
        let rows = Self::window(st, body, len);
        let g = self.geometry(body, st, model, rows);
        if g.hidden_left == 0 && g.hidden_right == 0 {
            return None;
        }
        let mut shown = (0..g.n).filter(|&i| g.shown.get(i).copied().unwrap_or(false));
        let first = shown.clone().next()?;
        let last = shown.next_back()?;
        let mut s = String::new();
        s.push_str("cols ");
        push_grouped(&mut s, first.saturating_add(1));
        s.push('\u{2013}');
        push_grouped(&mut s, last.saturating_add(1));
        s.push_str(" of ");
        push_grouped(&mut s, g.n);
        Some(s)
    }

    fn range<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
        cursor: (usize, usize),
    ) -> Option<((usize, usize), (usize, usize))> {
        let (ar, ac) = match st.anchor? {
            RangeAnchor::Cell(key, col) => (Self::row_index(model, key, cursor.0)?, col),
            RangeAnchor::Fetch(boundary, col) => (boundary, col),
        };
        let (r0, r1) = (ar.min(cursor.0), ar.max(cursor.0));
        let (c0, c1) = match self.nav {
            NavUnit::Row => (0, self.column_count().saturating_sub(1)),
            NavUnit::Cell => {
                let acol = self.col_index(ac)?;
                (acol.min(cursor.1), acol.max(cursor.1))
            }
        };
        Some(((r0, r1), (c0, c1)))
    }

    fn copy_tsv<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
        cursor: (usize, usize),
    ) -> String {
        let len = model.row_count();
        let cols = self.column_count();
        let mut out = String::new();
        if len == 0 || cols == 0 {
            return out;
        }
        let last_col = cols.saturating_sub(1);
        let last_row = len.saturating_sub(1);
        let row_to = |out: &mut String, r: usize, c0: usize, c1: usize| {
            for c in c0..=c1.min(last_col) {
                if c > c0 {
                    out.push('\t');
                }
                if let Some(cell) = model.cell(r, c) {
                    out.push_str(cell.text);
                }
            }
            out.push('\n');
        };
        if let Some(((r0, r1), (c0, c1))) = self.range(st, model, cursor) {
            for r in r0..=r1.min(last_row) {
                row_to(&mut out, r, c0, c1);
            }
        } else if !st.core.checked().is_empty() {
            for r in 0..len {
                if st.core.checked().contains(model.row_key(r)) {
                    row_to(&mut out, r, 0, last_col);
                }
            }
        } else if !st.on_fetch_row() {
            row_to(&mut out, cursor.0.min(last_row), cursor.1, cursor.1);
        }
        out
    }

    pub fn move_cursor_to<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: ItemKey,
        col: ColumnKey,
    ) -> Result<(), GridCursorError> {
        let row_index = Self::row_index(model, row, st.core.cursor_index())
            .ok_or(GridCursorError::UnknownRow(row))?;
        let col_index = self
            .col_index(col)
            .ok_or(GridCursorError::UnknownColumn(col))?;
        if st.is_editing() {
            return Err(GridCursorError::Editing);
        }
        st.type_ahead.reset();
        self.move_to(st, model, row_index, col_index, false, &mut Acc::new());
        st.core
            .scroll_mut()
            .ensure_visible_on_next_layout(row_index);
        Ok(())
    }

    fn cursor_anchor<M: GridModel + ?Sized>(
        &self,
        st: &GridState,
        model: &M,
    ) -> Option<RangeAnchor> {
        let col = self.col_key(self.cursor_col(st))?;
        if let Some(fetch) = st.fetch_row {
            return Some(RangeAnchor::Fetch(fetch.boundary, col));
        }
        let row = st
            .core
            .cursor_index()
            .min(model.row_count().saturating_sub(1));
        (row < model.row_count()).then(|| RangeAnchor::Cell(model.row_key(row), col))
    }

    fn move_to<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: usize,
        col: usize,
        extend: bool,
        acc: &mut Acc<GridAction>,
    ) {
        let len = model.row_count();
        let column_count = self.column_count();
        if self.fetch_on_activate && model.has_more() && row >= len {
            if st.is_editing() {
                acc.consumed();
                return;
            }
            if !extend {
                st.anchor = None;
            } else if st.anchor.is_none() {
                st.anchor = self.cursor_anchor(st, model);
            }
            st.fetch_row = Some(FetchRow { boundary: len });
            st.col_index = col.min(column_count.saturating_sub(1));
            st.col = self.col_key(st.col_index);
            st.core.scroll_mut().ensure_visible_on_next_layout(len);
            self.reveal_column(st, st.col_index);
            acc.action(GridAction::Moved);
            return;
        }
        if len == 0 || column_count == 0 {
            return;
        }
        let row = row.min(len.saturating_sub(1));
        let col = col.min(column_count.saturating_sub(1));
        let key = model.row_key(row);
        let Some(ck) = self.col_key(col) else { return };
        if !st.is_editing() {
            st.editor.set_error(None);
        }
        if extend {
            if st.anchor.is_none() {
                st.anchor = self.cursor_anchor(st, model);
            }
        } else {
            st.anchor = None;
        }
        st.set_cursor(row, key, col, ck);
        self.reveal_column(st, col);
        acc.action(GridAction::Moved);
    }

    fn reveal_column(&self, st: &mut GridState, col: usize) {
        if self.columns.get(col).is_some_and(|c| c.sticky) {
            if self.column_fit != GridColumnFit::Whole {
                st.pending_column = None;
            }
            return;
        }
        if self.column_fit != GridColumnFit::Whole {
            st.pending_column = self.col_key(col);
            return;
        }
        let scroll_index = self
            .columns
            .get(..col)
            .map_or(0, |cs| cs.iter().filter(|c| !c.sticky).count());
        st.col_offset = scroll_index;
    }

    fn prune_checked_rows<M: GridModel + ?Sized>(&self, st: &mut GridState, model: &M) -> bool {
        let len = model.row_count();
        let checked = st.core.checked_mut();
        if checked.is_empty() {
            return false;
        }

        if matches!(checked, KeySet::AllExcept(_)) {
            let mut dropped = false;
            for row in 0..len {
                if !model.row_enabled(row) {
                    let key = model.row_key(row);
                    if checked.contains(key) {
                        checked.remove(key);
                        dropped = true;
                    }
                }
            }
            dropped
        } else {
            let mut enabled_keys = Vec::with_capacity(len);
            for row in 0..len {
                if model.row_enabled(row) {
                    enabled_keys.push(model.row_key(row));
                }
            }
            enabled_keys.sort_unstable();
            checked.retain(|key| enabled_keys.binary_search(&key).is_ok()) > 0
        }
    }

    fn toggle_row<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: usize,
        acc: &mut Acc<GridAction>,
    ) {
        if row >= model.row_count() || !model.row_enabled(row) {
            acc.consumed();
            return;
        }
        let key = model.row_key(row);
        match self.select_mode {
            SelectMode::None => acc.consumed(),
            SelectMode::Single => {
                let was = st.core.checked().contains(key);
                st.core.checked_mut().none();
                if !was {
                    st.core.checked_mut().insert(key);
                }
                acc.changed();
            }
            SelectMode::Multi | SelectMode::Range => {
                st.core.checked_mut().toggle(key);
                acc.changed();
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the keymap dispatch and the three pointer surfaces in one drain loop"
    )]
    fn navigate<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &M,
        acc: &mut Acc<GridAction>,
    ) -> Pending {
        let len = model.row_count();
        let column_count = self.column_count();
        if let Some(RangeAnchor::Fetch(boundary, col)) = st.anchor {
            st.anchor = if boundary < len {
                Some(RangeAnchor::Cell(model.row_key(boundary), col))
            } else if model.has_more() {
                Some(RangeAnchor::Fetch(boundary.min(len), col))
            } else if len > 0 {
                Some(RangeAnchor::Cell(model.row_key(len.saturating_sub(1)), col))
            } else {
                None
            };
        }

        if st.core.cursor().is_some_and(|key| {
            st.core.cursor_index() >= len || model.row_key(st.core.cursor_index()) != key
        }) {
            st.type_ahead.reset();
            st.core.invalidate();
        }

        let outcome = st.core.reconcile_with(len, |i| model.row_key(i), |_| true);
        if self.prune_checked_rows(st, model) {
            acc.changed();
        }
        if outcome != Reconciliation::Unchanged {
            st.type_ahead.reset();
            if let Some(RangeAnchor::Cell(a, _)) = st.anchor
                && Self::row_index(model, a, st.core.cursor_index()).is_none()
            {
                st.anchor = None;
            }
            if let Some((e, _)) = st.edit
                && Self::row_index(model, e, st.core.cursor_index()).is_none()
            {
                st.edit = None;
                st.editor.cancel();
            }
        }
        if st.core.cursor().is_none() && len > 0 {
            st.core.set_cursor(0, model.row_key(0));
        }
        if let Some(fetch) = st.fetch_row {
            if len > fetch.boundary || !model.has_more() || !self.fetch_on_activate {
                st.fetch_row = None;
                if len > 0 {
                    let row = fetch.boundary.min(len.saturating_sub(1));
                    st.core.set_cursor(row, model.row_key(row));
                }
            } else {
                st.fetch_row = Some(FetchRow { boundary: len });
            }
        } else if self.fetch_on_activate && len == 0 && model.has_more() {
            st.fetch_row = Some(FetchRow { boundary: 0 });
        }
        let previous_col = st.col;
        if column_count == 0 {
            st.col = None;
            st.col_index = 0;
            st.col_offset = 0;
            st.anchor = None;
            st.cancel_editor();
        } else if st.col.is_none() {
            st.col = self.col_key(0);
            st.col_index = 0;
        } else if st.col.is_some_and(|key| self.col_index(key).is_none()) {
            st.type_ahead.reset();
            st.col_index = st.col_index.min(column_count.saturating_sub(1));
            st.col = self.col_key(st.col_index);
            st.anchor = None;
            st.cancel_editor();
        }
        if st
            .pending_column
            .is_some_and(|key| self.col_index(key).is_none())
        {
            st.pending_column = self.col_key(self.cursor_col(st));
        }

        if st.core.scroll().pending_reveal().is_some() {
            let row = st.fetch_row.map_or(st.core.cursor_index(), |f| f.boundary);
            st.core.scroll_mut().ensure_visible_on_next_layout(row);
            let col = self.cursor_col(st);
            if col != st.col_index || (previous_col.is_some() && previous_col != st.col) {
                st.col_index = col;
                self.reveal_column(st, col);
            }
        }
        let mut pending = Pending::default();
        let total = len.saturating_add(usize::from(model.has_more()));
        if let Some(layout) = cx.layout(self.id) {
            let scroll = Self::scroll_for_view(st, layout.viewport_len);
            *st.core.scroll_mut() = scroll;
        }
        let bar = self.bar().update(cx, st.core.scroll_mut(), total);
        acc.fold(&bar);
        let viewport = st.core.scroll().viewport_len().max(1);
        let area = cx.area(self.id);
        let content = area.map(|area| self.input_content(area, model));
        let geometry = content.map(|body| {
            let rows = Self::window(st, body, total);
            let geometry = self.geometry(body, st, model, rows);
            self.apply_column_geometry(st, &geometry);
            geometry
        });
        for it in cx.intents(self.id) {
            match it {
                Intent::Binding(action) => {
                    st.type_ahead.reset();
                    let row = st.fetch_row.map_or_else(
                        || st.core.cursor_index().min(len.saturating_sub(1)),
                        |f| f.boundary,
                    );
                    let col = self.cursor_col(st);
                    let command = Binding::command(
                        if self.sort_current_column {
                            &TABLE_BINDINGS
                        } else {
                            &BINDINGS
                        },
                        action,
                    );
                    match command {
                        Some(GridCmd::Up) => {
                            self.move_to(st, model, row.saturating_sub(1), col, false, acc);
                        }
                        Some(GridCmd::Down) => {
                            if row.saturating_add(1) >= len
                                && model.has_more()
                                && !self.fetch_on_activate
                            {
                                acc.action(GridAction::FetchMore);
                            } else {
                                self.move_to(st, model, row.saturating_add(1), col, false, acc);
                            }
                        }
                        Some(GridCmd::Left) if st.on_fetch_row() => {
                            self.move_to(st, model, row, col.saturating_sub(1), false, acc);
                        }
                        Some(GridCmd::Right) if st.on_fetch_row() => {
                            self.move_to(st, model, row, col.saturating_add(1), false, acc);
                        }
                        Some(GridCmd::Left)
                            if matches!(self.nav, NavUnit::Row) && self.row_nav_scrolls_columns =>
                        {
                            if st.col_offset > 0 {
                                st.col_offset = st.col_offset.saturating_sub(1);
                                acc.changed();
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::Right)
                            if matches!(self.nav, NavUnit::Row) && self.row_nav_scrolls_columns =>
                        {
                            if st.col_offset
                                < self
                                    .row_nav_scroll_limit
                                    .unwrap_or_else(|| column_count.saturating_sub(1))
                            {
                                st.col_offset = st.col_offset.saturating_add(1);
                                acc.changed();
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::Left) => match self.nav {
                            NavUnit::Row => acc.consumed(),
                            NavUnit::Cell if col > 0 => {
                                self.move_to(st, model, row, col.saturating_sub(1), false, acc);
                            }
                            NavUnit::Cell if row > 0 => self.move_to(
                                st,
                                model,
                                row.saturating_sub(1),
                                column_count.saturating_sub(1),
                                false,
                                acc,
                            ),
                            NavUnit::Cell => acc.action(GridAction::LeaveBackward),
                        },
                        Some(GridCmd::Right) => match self.nav {
                            NavUnit::Row => acc.consumed(),
                            NavUnit::Cell if col.saturating_add(1) < column_count => {
                                self.move_to(st, model, row, col.saturating_add(1), false, acc);
                            }
                            NavUnit::Cell if row.saturating_add(1) < len => {
                                self.move_to(st, model, row.saturating_add(1), 0, false, acc);
                            }
                            NavUnit::Cell => acc.action(GridAction::LeaveForward),
                        },
                        Some(GridCmd::PageUp) => {
                            self.move_to(st, model, row.saturating_sub(viewport), col, false, acc);
                        }
                        Some(GridCmd::PageDown) => {
                            self.move_to(st, model, row.saturating_add(viewport), col, false, acc);
                        }
                        Some(GridCmd::RowStart) => self.move_to(st, model, row, 0, false, acc),
                        Some(GridCmd::RowEnd) => {
                            self.move_to(
                                st,
                                model,
                                row,
                                column_count.saturating_sub(1),
                                false,
                                acc,
                            );
                        }
                        Some(GridCmd::First) => self.move_to(st, model, 0, col, false, acc),
                        Some(GridCmd::Last) => {
                            self.move_to(
                                st,
                                model,
                                len.saturating_sub(1).saturating_add(usize::from(
                                    self.fetch_on_activate && model.has_more(),
                                )),
                                col,
                                false,
                                acc,
                            );
                        }
                        Some(GridCmd::ExtendUp) => {
                            self.move_to(st, model, row.saturating_sub(1), col, true, acc);
                        }
                        Some(GridCmd::ExtendDown) => {
                            self.move_to(st, model, row.saturating_add(1), col, true, acc);
                        }
                        Some(GridCmd::ExtendLeft) => {
                            self.move_to(st, model, row, col.saturating_sub(1), true, acc);
                        }
                        Some(GridCmd::ExtendRight) => {
                            self.move_to(st, model, row, col.saturating_add(1), true, acc);
                        }
                        Some(GridCmd::ToggleRow | GridCmd::Activate | GridCmd::BeginEdit)
                            if st.on_fetch_row() =>
                        {
                            acc.action(GridAction::FetchMore);
                        }
                        Some(GridCmd::ToggleRow) => self.toggle_row(st, model, row, acc),
                        Some(GridCmd::ToggleAll) => {
                            if matches!(self.select_mode, SelectMode::None) {
                                acc.consumed();
                            } else {
                                let all = (0..len).all(|i| {
                                    !model.row_enabled(i)
                                        || st.core.checked().contains(model.row_key(i))
                                });
                                if all {
                                    st.core.checked_mut().none();
                                } else {
                                    st.core.checked_mut().all();
                                    for i in 0..len {
                                        if !model.row_enabled(i) {
                                            st.core.checked_mut().remove(model.row_key(i));
                                        }
                                    }
                                }
                                acc.changed();
                            }
                        }
                        Some(GridCmd::Activate) => {
                            if len == 0 || column_count == 0 {
                                acc.consumed();
                            } else if matches!(self.nav, NavUnit::Row) {
                                if model.row_enabled(row) {
                                    acc.action(GridAction::Activated(model.row_key(row)));
                                } else {
                                    acc.consumed();
                                }
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Activate,
                                });
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::BeginEdit) => {
                            if len == 0 || column_count == 0 {
                                acc.consumed();
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Explicit,
                                });
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::Copy) => {
                            if len > 0 {
                                let tsv = self.copy_tsv(st, model, (row, col));
                                acc.action(GridAction::Copy(tsv));
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::SortCurrent) if self.sort_current_column => {
                            let key = self.col_key(col);
                            if let Some(key) = key.filter(|_| {
                                self.columns.get(col).is_some_and(|column| column.sortable)
                            }) {
                                let dir = match st.sort {
                                    Some((current, SortDir::Asc)) if current == key => {
                                        SortDir::Desc
                                    }
                                    _ => SortDir::Asc,
                                };
                                st.sort = Some((key, dir));
                                acc.action(GridAction::Sort(key, dir));
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(GridCmd::SortCurrent) => {}
                        None => {}
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::HEADER,
                            item: Some(header_key),
                        },
                    ..
                } => {
                    let col = (0..column_count).find(|&i| {
                        self.columns.get(i).is_some_and(|column| {
                            column.sortable && column_item_key(column.key) == header_key
                        })
                    });
                    if matches!(phase, Phase::Click | Phase::DoubleClick) && col.is_some() {
                        st.type_ahead.reset();
                    }
                    match (phase, col.and_then(|i| self.col_key(i))) {
                        (Phase::Click | Phase::DoubleClick, Some(key)) => {
                            let dir = match st.sort {
                                Some((current, SortDir::Asc)) if current == key => SortDir::Desc,
                                _ => SortDir::Asc,
                            };
                            st.sort = Some((key, dir));
                            acc.action(GridAction::Sort(key, dir));
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part:
                        PartRef {
                            part: Part::ROW_NUMBER,
                            item: Some(key),
                        },
                    ..
                } => {
                    if let Some(row) = Self::row_index(model, key, st.core.cursor_index()) {
                        st.type_ahead.reset();
                        self.move_to(st, model, row, self.cursor_col(st), false, acc);
                        self.toggle_row(st, model, row, acc);
                    } else {
                        acc.consumed();
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::ACTIONS,
                            item: Some(row_key),
                        },
                    pos,
                    ..
                } => {
                    let hit = geometry
                        .as_ref()
                        .and_then(|geometry| geometry.column_at(pos.x))
                        .and_then(|column| {
                            Some((
                                Self::row_index(model, row_key, st.core.cursor_index())?,
                                column,
                            ))
                        });
                    match (phase, hit) {
                        (Phase::Click, Some((row, column))) => {
                            let action = model
                                .cell(row, column)
                                .and_then(|_| model.actions(row, column).first());
                            match (self.col_key(column), action) {
                                (Some(column_key), Some(action)) => {
                                    acc.action(GridAction::CellAction(
                                        row_key, column_key, action.key,
                                    ));
                                }
                                _ => acc.consumed(),
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase,
                    part:
                        PartRef {
                            part: Part::CELL,
                            item: Some(row_key),
                        },
                    pos,
                    ..
                } => {
                    let Some(row) = Self::row_index(model, row_key, st.core.cursor_index()) else {
                        acc.consumed();
                        continue;
                    };
                    let hit = geometry
                        .as_ref()
                        .and_then(|geometry| geometry.column_at(pos.x));
                    if hit.is_none() && self.column_fit != GridColumnFit::Whole {
                        acc.consumed();
                        continue;
                    }
                    let col = hit.unwrap_or_else(|| self.cursor_col(st));
                    if matches!(phase, Phase::Press | Phase::DoubleClick) {
                        st.type_ahead.reset();
                    }
                    match phase {
                        Phase::Press => self.move_to(st, model, row, col, false, acc),
                        Phase::DoubleClick => {
                            if matches!(self.nav, NavUnit::Row) {
                                if model.row_enabled(row) {
                                    acc.action(GridAction::Activated(row_key));
                                } else {
                                    acc.consumed();
                                }
                            } else {
                                pending.edit = Some(EditRequest {
                                    row,
                                    col,
                                    cause: EditCause::Activate,
                                });
                                acc.changed();
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click,
                    part:
                        PartRef {
                            part: Part::ROW,
                            item: None,
                        },
                    ..
                } => {
                    st.type_ahead.reset();
                    if self.fetch_on_activate {
                        if st.is_editing() || !model.has_more() {
                            acc.consumed();
                            continue;
                        }
                        self.move_to(st, model, len, self.cursor_col(st), false, acc);
                    }
                    acc.action(GridAction::FetchMore);
                }
                Intent::Pointer { .. } => acc.consumed(),
                Intent::FocusOut { to } if to != Some(self.id) => st.type_ahead.reset(),
                Intent::Key(key) => self.type_ahead_key(cx, st, model, key, len, acc),
                _ => {}
            }
        }
        if self.column_fit != GridColumnFit::Whole
            && let Some(body) = content
        {
            let geometry = self.geometry(body, st, model, Self::window(st, body, total));
            self.apply_column_geometry(st, &geometry);
        }
        pending
    }

    pub fn update<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &M,
    ) -> Response<GridAction> {
        self.assert_distinct_column_keys();
        self.sync_type_ahead(st);
        if self.disabled {
            for _ in cx.intents(self.id) {}
            for _ in cx.intents(self.editor_id()) {}
            return Response::ignored();
        }
        let mut acc = Acc::<GridAction>::new();
        let pending = self.navigate(cx, st, model, &mut acc);

        if let Some(EditRequest {
            row,
            cause: EditCause::Activate,
            ..
        }) = pending.edit
            && row < model.row_count()
            && model.row_enabled(row)
        {
            acc.action(GridAction::Activated(model.row_key(row)));
        }
        acc.finish(self.id)
    }

    pub fn update_editable<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
    ) -> Response<GridAction> {
        self.assert_distinct_column_keys();
        self.sync_type_ahead(st);
        if self.disabled {
            for _ in cx.intents(self.id) {}
            for _ in cx.intents(self.editor_id()) {}
            return Response::ignored();
        }
        let mut acc = Acc::<GridAction>::new();

        if st.edit.is_none() {
            for _ in cx.intents(self.editor_id()) {}
        }
        self.drive_editor(cx, st, model, &mut acc);
        let pending = self.navigate(cx, st, model, &mut acc);
        if let Some(request) = pending.edit
            && st.edit.is_none()
            && request.row < model.row_count()
        {
            self.begin_edit(cx, st, model, request, &mut acc);
        }
        acc.finish(self.id)
    }

    fn begin_edit<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
        request: EditRequest,
        acc: &mut Acc<GridAction>,
    ) {
        let EditRequest { row, col, cause } = request;
        st.type_ahead.reset();
        let key = model.row_key(row);
        let Some(ck) = self.col_key(col) else { return };
        st.editor.set_error(None);
        if model.cell(row, col).is_none()
            || !self.columns.get(col).is_some_and(|c| c.editable)
            || !model.is_editable(row, col)
        {
            if cause == EditCause::Activate && model.row_enabled(row) {
                acc.action(GridAction::Activated(key));
            }
            return;
        }
        let what = match model.edit_intent(row, col) {
            EditIntent::Inline { initial } => Begin::Inline(initial.to_owned()),
            EditIntent::Cycle => Begin::Cycle,
            EditIntent::External => Begin::External,
            EditIntent::Refuse { reason } => Begin::Refuse(reason.to_owned()),
        };
        match what {
            Begin::Inline(initial) => {
                st.editor.set_error(None);
                st.editor.begin(&initial);
                st.edit = Some((key, ck));
                cx.focus(self.editor_id());
                acc.changed();
            }
            Begin::Cycle => {
                model.apply_cycle(row, col);
                acc.changed();
            }

            Begin::External => acc.action(GridAction::EditRequested(key, ck)),
            Begin::Refuse(reason) => {
                st.editor
                    .set_error(Some(crate::validate::FieldError::new(reason)));
                acc.changed();
            }
        }
    }

    fn drive_editor<M: GridEditor + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut GridState,
        model: &mut M,
        acc: &mut Acc<GridAction>,
    ) {
        let Some((rk, ck)) = st.edit else { return };
        let (Some(col), Some(row)) = (
            self.col_index(ck),
            Self::row_index(model, rk, st.core.cursor_index()),
        ) else {
            st.cancel_editor();
            return;
        };

        let Some(cell) = model.cell(row, col) else {
            st.cancel_editor();
            return;
        };
        let mut value = cell.text.to_owned();
        let pointer_enabled = self.column_fit == GridColumnFit::Whole
            || cx.area(self.id).is_some_and(|area| {
                let body = self.input_content(area, model);
                let total = model
                    .row_count()
                    .saturating_add(usize::from(model.has_more()));
                let geometry = self.geometry(body, st, model, Self::window(st, body, total));
                geometry.complete.get(col).copied().unwrap_or(false)
            });
        let mut r = TextInput::new(self.editor_id())
            .embedded_cell()
            .text_style_target(Family::GRID, Variant::DEFAULT, Part::TEXT)
            .blur(self.blur)
            .pointer_enabled(pointer_enabled)
            .update(cx, &mut st.editor, &mut value);
        let action = r.take_action();
        let erased = r.erase();
        acc.fold(&erased);
        match action {
            Some(TextAction::Committed) => match model.commit_cell(row, col, &value) {
                Ok(()) => {
                    st.edit = None;
                    cx.focus(self.id);
                    acc.changed();
                }
                Err(e) => {
                    st.editor.begin(&value);
                    st.editor.set_error(Some(e));
                    acc.changed();
                }
            },
            Some(TextAction::Cancelled) => {
                st.cancel_editor();
                cx.focus(self.id);
                acc.changed();
            }
            _ => {}
        }
    }
}

impl Grid<'_> {
    fn right_overflow_rect(head: Rect, hidden_right: usize) -> Rect {
        let count = Num::new(hidden_right);
        let indicator_width = width(count.as_str()).saturating_add(1).min(head.width);
        Rect {
            x: head.right().saturating_sub(indicator_width),
            width: indicator_width,
            ..head
        }
    }

    fn register_sort_headers(&self, ui: &mut Ui<'_>, head: Rect, geometry: &Geometry) {
        if ui.is_inert() {
            return;
        }
        for index in 0..geometry.n {
            let rect = geometry.cell(index, head.y);
            if rect.is_empty() {
                continue;
            }
            if let Some(column) = self.columns.get(index)
                && column.sortable
            {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::HEADER, column_item_key(column.key)),
                    rect,
                );
            }
        }
    }

    fn draw_header_overflow(
        &self,
        ui: &mut Ui<'_>,
        head: Rect,
        geometry: &Geometry,
        live: StateFlags,
    ) {
        if geometry.hidden_left == 0 && geometry.hidden_right == 0 {
            return;
        }
        let style = self
            .ov
            .style(
                ui,
                self.id,
                Family::GRID,
                Variant::DEFAULT,
                Part::OVERFLOW,
                live,
            )
            .style;
        if geometry.hidden_left > 0 {
            let count = Num::new(geometry.hidden_left);
            let at = Rect {
                width: head.width.min(2),
                ..head
            };
            let used = ui.glyph(at, GlyphRole::OverflowLeft, style);
            ui.paint_str(
                Rect {
                    x: at.x.saturating_add(used),
                    width: at.width.saturating_sub(used),
                    ..at
                },
                count.as_str(),
                style,
            );
        }
        if geometry.hidden_right > 0 {
            let count = Num::new(geometry.hidden_right);
            let at = Self::right_overflow_rect(head, geometry.hidden_right);
            let used = ui.paint_str(at, count.as_str(), style);
            ui.glyph(
                Rect {
                    x: at.x.saturating_add(used),
                    width: at.width.saturating_sub(used),
                    ..at
                },
                GlyphRole::OverflowRight,
                style,
            );
        }
    }

    fn paint_header_title(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        col: &Column<'_>,
        flags: StateFlags,
        style: PaintStyle,
    ) {
        let Some((glyph, tone)) = self.header_prefix(col.key) else {
            paint_aligned(ui, area, col.title, col.align, style);
            return;
        };
        let group_width = width(col.title).saturating_add(2).min(area.width);
        let spare = area.width.saturating_sub(group_width);
        let offset = match col.align {
            Align::Left => 0,
            Align::Center => spare / 2,
            Align::Right => spare,
        };
        let group = Rect {
            x: area.x.saturating_add(offset),
            y: area.y,
            width: group_width,
            height: area.height,
        };
        let icon = Rect {
            width: group.width.min(1),
            ..group
        };
        let local = self.ov.part_patch(Part::ICON);
        let resolved = ui.style_defaults(
            Family::GRID,
            Variant::DEFAULT,
            Part::ICON,
            flags,
            StyleDefaults::new(StylePatch::new().set_fg(tone).set_glyph(glyph)),
            local.as_ref(),
        );
        self.ov.note(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::ICON,
            resolved,
        );
        if let Some(slot) = self.ov.slot_for(Part::ICON) {
            ui.with_area(icon, |ui| slot(ui, icon));
        } else if let Some(glyph) = resolved.glyph.get() {
            ui.glyph(icon, glyph, resolved.over(style));
        }
        let title = Rect {
            x: group.x.saturating_add(2),
            width: group.width.saturating_sub(2),
            ..group
        };
        paint_aligned(ui, title, col.title, Align::Left, style);
    }

    fn draw_header(
        &self,
        ui: &mut Ui<'_>,
        head: Rect,
        g: &Geometry,
        st: &GridState,
        live: StateFlags,
    ) {
        if head.is_empty() {
            return;
        }
        self.register_sort_headers(ui, head, g);
        if let Some(f) = self.ov.slot_for(Part::HEADER) {
            f(ui, head);
            return;
        }
        let hs = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::HEADER,
            live,
        );
        ui.fill(head, hs.style);
        let right_overflow =
            (g.hidden_right > 0).then(|| Self::right_overflow_rect(head, g.hidden_right));
        for i in 0..g.n {
            let raw_rect = g.cell(i, head.y);
            let Some(col) = self.columns.get(i) else {
                break;
            };
            let rect = right_overflow.map_or(raw_rect, |overflow| Rect {
                width: raw_rect.right().min(overflow.x).saturating_sub(raw_rect.x),
                ..raw_rect
            });
            if rect.width == 0 {
                continue;
            }
            let show_sort = col.sortable
                && (self.sort_indicator == GridSortIndicator::Always
                    || st.sort.is_some_and(|(key, _)| key == col.key));
            let sort_width = u16::from(show_sort).min(rect.width);
            let title = Rect {
                width: rect
                    .width
                    .saturating_sub(
                        col.badge
                            .map_or(0, |b| width(b).saturating_add(1).min(rect.width)),
                    )
                    .saturating_sub(sort_width),
                ..rect
            };
            self.paint_header_title(ui, title, col, live, hs.style);
            if let Some(badge) = col.badge {
                let bw = width(badge).min(rect.width);
                let at = Rect {
                    x: rect.right().saturating_sub(sort_width).saturating_sub(bw),
                    width: bw,
                    ..rect
                };
                paint_aligned(ui, at, badge, Align::Right, hs.style);
            }
            if show_sort {
                let glyph = match st.sort {
                    Some((key, SortDir::Desc)) if key == col.key => GlyphRole::SortDesc,
                    _ => GlyphRole::SortAsc,
                };
                ui.glyph(
                    Rect {
                        x: rect.right().saturating_sub(1),
                        width: sort_width,
                        ..rect
                    },
                    glyph,
                    hs.style,
                );
            }
        }
        self.draw_header_overflow(ui, head, g, live);
    }

    fn gutter_style(
        &self,
        ui: &mut Ui<'_>,
        part: Part,
        flags: StateFlags,
        mut base: StylePatch,
    ) -> crate::theme::Resolved {
        for (named, patch) in self.part_defaults {
            if *named == part {
                base = base.merge(*patch);
            }
        }
        let local = self.ov.part_patch(part);
        let resolved = ui.style_defaults(
            Family::GRID,
            Variant::DEFAULT,
            part,
            flags,
            StyleDefaults::new(base),
            local.as_ref(),
        );
        self.ov
            .note(ui, self.id, Family::GRID, Variant::DEFAULT, part, resolved);
        resolved
    }

    fn cell_style(
        &self,
        ui: &mut Ui<'_>,
        flags: StateFlags,
        defaults: StyleDefaults<'_>,
    ) -> crate::theme::ResolvedStyle {
        let mut base = defaults.base;
        for (part, patch) in self.part_defaults {
            if *part == Part::CELL {
                base = base.merge(*patch);
            }
        }
        let defaults = StyleDefaults { base, ..defaults };
        let local = self.ov.part_patch(Part::CELL);
        let mut resolved = ui.style_defaults_with_explicit(
            Family::GRID,
            Variant::DEFAULT,
            Part::CELL,
            flags,
            defaults,
            local.as_ref(),
        );
        resolved.explicit =
            preserve_default_clears(base, resolved.resolved.style, resolved.explicit);
        self.ov.note(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::CELL,
            resolved.resolved,
        );
        resolved
    }

    fn draw_detailed_gutter(
        &self,
        ui: &mut Ui<'_>,
        geometry: &Geometry,
        y: u16,
        row: GutterRow<'_>,
    ) {
        let GutterRow {
            data,
            decor,
            flags,
            style: inherited,
        } = row;
        let inherited = ui.surface_style().patch(inherited);

        ui.fill(geometry.gutter_row(y), inherited);
        let focused = flags.contains(StateFlags::FOCUSED);
        let checked = flags.contains(StateFlags::CHECKED);
        let mut focus = StylePatch::new().set_glyph(GlyphRole::FocusBar);
        if focused {
            focus = focus.set_fg(Role::Focus);
        }
        let mut check = StylePatch::new()
            .set_fg(if focused {
                Role::Accent
            } else {
                Role::Fg(FgStep::Secondary)
            })
            .remove(Modifier::CROSSED_OUT);
        if checked {
            check = check.set_glyph(GlyphRole::Checked);
        }
        let mut change = StylePatch::new()
            .set_fg(decor.tone.unwrap_or(Role::Fg(FgStep::Secondary)))
            .remove(Modifier::CROSSED_OUT);
        if let Some(glyph) = decor.marker {
            change = change.set_glyph(glyph);
        }
        let number_defaults = StylePatch::new()
            .set_fg(Role::Fg(if focused {
                FgStep::Secondary
            } else {
                FgStep::Faint
            }))
            .remove(Modifier::BOLD | Modifier::CROSSED_OUT);
        let value = data.map(|(_, number)| Num::new(number));
        for (part, defaults) in [
            (Part::GUTTER, focus),
            (Part::MARKER, check),
            (Part::CHANGE, change),
            (Part::ROW_NUMBER, number_defaults),
        ] {
            if data.is_none() && part != Part::GUTTER {
                continue;
            }
            let area = geometry.gutter_part(part, y);
            if area.is_empty() {
                continue;
            }
            let resolved = self.gutter_style(ui, part, flags, defaults);

            let base = if part == Part::GUTTER && !focused {
                inherited.with_fg_from_bg(inherited)
            } else {
                inherited
            };
            let style = resolved.over(base);
            ui.fill(area, style);
            if let Some(slot) = self.ov.slot_for(part) {
                ui.with_area(area, |ui| slot(ui, area));
            } else if let Some(glyph) = resolved.glyph.get() {
                ui.glyph(area, glyph, style);
            } else if part == Part::ROW_NUMBER
                && let Some(value) = &value
            {
                paint_aligned(
                    ui,
                    area,
                    value.as_str(),
                    resolved.align.unwrap_or(Align::Right),
                    style,
                );
            }
        }
        if geometry.number_width > 0
            && !ui.is_inert()
            && let Some((key, _)) = data
        {
            ui.register_part(
                self.id,
                PartRef::item(Part::ROW_NUMBER, key),
                geometry.gutter_row(y),
            );
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one pass over the visible cells: fill, gutter, marker, cells, affordances and the inline editor"
    )]
    fn draw_row<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        y: u16,
        row: usize,
        paint: &RowPaint<'_, M>,
    ) {
        let RowPaint {
            content,
            geometry,
            state,
            model,
            cursor,
            range,
            live,
        } = paint;
        let key = model.row_key(row);
        let decor = model.row_decor(row);
        let checked = state.core.checked().contains(key);
        let is_cursor = !state.on_fetch_row() && row == cursor.0;
        let pressed = ui.pressed_part(self.id);
        let mut rflags = decor.flags();
        if is_cursor {
            rflags |= *live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
        }
        if checked {
            rflags |= StateFlags::CHECKED | StateFlags::SELECTED;
        }
        if live.contains(StateFlags::DISABLED) {
            rflags |= StateFlags::DISABLED;
        }
        if !self.disabled && pressed == Some(PartRef::item(Part::ROW, key)) {
            rflags |= StateFlags::PRESSED;
        }
        if !self.disabled && self.gutter != GridGutter::Compact {
            let hovered = ui.hovered_part(self.id);
            if matches!(hovered, Some(PartRef { part: Part::ROW_NUMBER | Part::CELL, item: Some(hovered_key) }) if hovered_key == key)
            {
                rflags |= StateFlags::HOVERED;
            }
            if pressed == Some(PartRef::item(Part::ROW_NUMBER, key)) {
                rflags |= StateFlags::PRESSED;
            }
        }
        let band = Rect {
            x: content.x,
            y,
            width: content.width,
            height: 1,
        };
        let row_defaults = self
            .part_defaults
            .iter()
            .filter(|(part, _)| *part == Part::ROW)
            .fold(StylePatch::new(), |base, (_, patch)| base.merge(*patch));
        let row_defaults = StyleDefaults::new(row_defaults);
        let row_local = self.ov.part_patch(Part::ROW);
        let mut row_resolved = ui.style_defaults_with_explicit(
            Family::GRID,
            Variant::DEFAULT,
            Part::ROW,
            rflags,
            row_defaults,
            row_local.as_ref(),
        );
        row_resolved.explicit = preserve_default_clears(
            row_defaults.base,
            row_resolved.resolved.style,
            row_resolved.explicit,
        );
        self.ov.note(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::ROW,
            row_resolved.resolved,
        );
        let mut row_delta = StylePatch::new();
        if decor.strike {
            row_delta = row_delta.add(Modifier::CROSSED_OUT);
        }
        if decor.faint {
            row_delta = row_delta.add(Modifier::DIM);
        }
        let row_style = apply_style_delta(ui, row_resolved.resolved.style, row_delta);
        let cell_defaults = inherited_style_defaults(row_style, row_resolved.explicit);
        ui.fill(band, row_style);
        if !row_resolved.explicit.is_empty() {
            termrock_runtime::author::patch_style(ui, band, row_style, row_resolved.explicit);
        }
        if self.gutter == GridGutter::Compact {
            if is_cursor && live.contains(StateFlags::FOCUSED) {
                ui.glyph(super::cell_at(band, band.x), GlyphRole::FocusBar, row_style);
            }
            let marker_cell = super::cell_at(band, band.x.saturating_add(1));
            if checked {
                ui.glyph(marker_cell, GlyphRole::Checked, row_style);
            } else if let Some(m) = decor.marker {
                ui.glyph(marker_cell, m, row_style);
            }
        } else {
            self.draw_detailed_gutter(
                ui,
                geometry,
                y,
                GutterRow {
                    data: Some((key, decor.number.unwrap_or_else(|| row.saturating_add(1)))),
                    decor,
                    flags: rflags,
                    style: row_style,
                },
            );
        }
        let inert = ui.is_inert();
        for i in 0..geometry.n {
            let rect = geometry.cell(i, y);
            if rect.width == 0 {
                continue;
            }
            let cell = model.cell(row, i);

            let editing = cell.is_some()
                && self
                    .col_key(i)
                    .is_some_and(|col| state.edit == Some((key, col)));
            let refused_error = (!editing && is_cursor && i == cursor.1)
                .then(|| state.edit_error())
                .flatten();
            let mut cflags = rflags.difference(StateFlags::PRESSED);
            if is_cursor && i == cursor.1 && live.contains(StateFlags::FOCUSED) {
                cflags |= StateFlags::ACTIVE;
            }
            if range
                .is_some_and(|((r0, r1), (c0, c1))| row >= r0 && row <= r1 && i >= c0 && i <= c1)
            {
                cflags |= StateFlags::SELECTED;
            }
            if editing {
                cflags |= StateFlags::EDITING;
            }
            if refused_error.is_some() {
                cflags |= StateFlags::ERROR;
            }
            if !self.disabled && pressed == Some(PartRef::item(Part::CELL, key)) {
                cflags |= StateFlags::PRESSED;
            }
            let cell_defaults = if cflags.contains(StateFlags::ACTIVE) {
                StyleDefaults::default()
            } else {
                cell_defaults
            };
            let Some(cell) = cell else {
                let cs = self.cell_style(ui, cflags, cell_defaults);
                let style = if cflags.contains(StateFlags::ACTIVE) {
                    active_cell_style(ui, row_style, cs, decor.strike)
                } else {
                    cs.resolved.style
                };
                ui.fill(rect, style);
                if !cs.explicit.is_empty() {
                    termrock_runtime::author::patch_style(ui, rect, cs.resolved.style, cs.explicit);
                }
                if !inert {
                    geometry.register_cell_part(
                        ui,
                        self.id,
                        PartRef::item(Part::CELL, key),
                        i,
                        rect,
                    );
                }
                continue;
            };
            let cdecor = model.cell_decor(row, i);
            let actions = model.actions(row, i);
            cflags |= cdecor.flags();
            let cs = self.cell_style(ui, cflags, cell_defaults);
            if cflags.contains(StateFlags::ACTIVE) && (!actions.is_empty() || cell.text.is_empty())
            {
                ui.fill(rect, active_cell_style(ui, row_style, cs, decor.strike));
            }
            if !cs.explicit.is_empty() {
                termrock_runtime::author::patch_style(ui, rect, cs.resolved.style, cs.explicit);
            }
            let mut cell_delta = StylePatch::new();
            if let Some(role) = cdecor.tone.or(cell.tone) {
                cell_delta = cell_delta.set_fg(role);
                if let Some((_, patch)) =
                    self.cell_tone_styles.iter().find(|(tone, _)| *tone == role)
                {
                    cell_delta = cell_delta.merge(*patch);
                }
            }
            if cdecor.italic {
                cell_delta = cell_delta.add(Modifier::ITALIC);
            }

            if decor.strike && !cs.explicit.remove.contains(Modifier::CROSSED_OUT) {
                cell_delta = cell_delta.add(Modifier::CROSSED_OUT);
            }
            let cell_style = if cflags.contains(StateFlags::ACTIVE) {
                active_cell_style(ui, row_style, cs, decor.strike)
            } else {
                cs.resolved.style
            };
            let style = apply_style_delta(ui, cell_style, cell_delta);
            let mut text_rect = rect;
            if let Some(gl) = self.columns.get(i).and_then(|c| c.prefix_glyph) {
                let used = ui.glyph(rect, gl, style);
                text_rect = Rect {
                    x: rect.x.saturating_add(used).saturating_add(1),
                    width: rect.width.saturating_sub(used).saturating_sub(1),
                    ..rect
                };
            }
            let affordance = if actions.is_empty() {
                Rect::ZERO
            } else {
                let w = text_rect.width.min(1);
                let r = Rect {
                    x: text_rect.right().saturating_sub(w),
                    width: w,
                    ..text_rect
                };
                text_rect = Rect {
                    width: text_rect
                        .width
                        .saturating_sub(w.saturating_add(1).min(text_rect.width)),
                    ..text_rect
                };
                r
            };
            let align = cell
                .align
                .or_else(|| self.columns.get(i).map(|column| column.align))
                .unwrap_or(Align::Left);
            let text = refused_error.map_or(cell.text, |error| error.message.as_ref());
            if let Some((render, column_key)) = self
                .cell
                .filter(|_| !editing && refused_error.is_none())
                .zip(self.col_key(i))
            {
                let padding_style = (cflags.contains(StateFlags::ACTIVE) && actions.is_empty())
                    .then(|| ui.surface_style().patch(row_style));
                let mut painter = CellUi::new(ui.reborrow(), text_rect, style);
                if let Some(padding_style) = padding_style {
                    painter.padding_style(padding_style);
                }
                painter.align(align);
                render(
                    GridCell {
                        row_key: key,
                        column_key,
                        row,
                        column: i,
                        value: cell,
                        flags: cflags,
                        style,
                    },
                    &mut painter,
                );
            } else {
                paint_aligned(ui, text_rect, text, align, style);
            }
            if cflags.contains(StateFlags::ACTIVE) && !cs.explicit.is_empty() {
                termrock_runtime::author::patch_style(ui, rect, cs.resolved.style, cs.explicit);
            }
            if !inert {
                geometry.register_cell_part(ui, self.id, PartRef::item(Part::CELL, key), i, rect);
            }
            if let Some(a) = actions.first() {
                if let Some(f) = self.ov.slot_for(Part::ACTIONS) {
                    f(ui, affordance);
                } else {
                    let mut action_flags = cflags.difference(StateFlags::PRESSED);
                    if !self.disabled && pressed == Some(PartRef::item(Part::ACTIONS, key)) {
                        action_flags |= StateFlags::PRESSED;
                    }
                    let as_ = self.ov.style(
                        ui,
                        self.id,
                        Family::GRID,
                        Variant::DEFAULT,
                        Part::ACTIONS,
                        action_flags,
                    );
                    ui.glyph(affordance, a.glyph, as_.style);
                }
                if !inert {
                    geometry.register_cell_part(
                        ui,
                        self.id,
                        PartRef::item(Part::ACTIONS, key),
                        i,
                        affordance,
                    );
                }
            }
            if editing {
                TextInput::new(self.id.part(Part::TEXT))
                    .embedded_cell()
                    .text_style_target(Family::GRID, Variant::DEFAULT, Part::TEXT)
                    .pointer_enabled(geometry.complete.get(i).copied().unwrap_or(false))
                    .disabled(self.disabled)
                    .value(cell.text)
                    .draw(ui, rect, &state.editor);
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the chrome, the scroll region, the body and the four conditional surfaces in one pass"
    )]
    pub fn draw<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &GridState,
        model: &M,
    ) -> Rect {
        self.assert_distinct_column_keys();
        if area.is_empty() {
            return area;
        }
        let len = model.row_count();
        let total = len.saturating_add(usize::from(model.has_more()));
        let inert = ui.is_inert();
        if !inert {
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
        let reason = model.read_only_reason();
        let derived = if reason.is_some() {
            StateFlags::READ_ONLY
        } else {
            StateFlags::empty()
        };
        let mut live = PartStyle::flags(ui.state(self.id), derived);
        if self.disabled {
            live |= StateFlags::DISABLED;
            live.remove(
                StateFlags::HOVERED
                    | StateFlags::PRESSED
                    | StateFlags::FOCUSED
                    | StateFlags::FOCUS_VISIBLE,
            );
        }
        if !inert {
            ui.publish_bindings(
                self.id,
                live,
                if self.sort_current_column {
                    &TABLE_BINDINGS
                } else {
                    &BINDINGS
                },
            );
            if self.type_ahead && !self.disabled && !st.is_editing() {
                ui.publish_type_ahead(self.id, live);
            }
        }
        let container = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::CONTAINER,
            live.difference(StateFlags::FOCUSED | StateFlags::PRESSED | StateFlags::SELECTED),
        );
        ui.fill(area, container.style);
        let (header, note, body, bar) = self.chrome(area, reason);
        let scroll = Self::scroll_for_view(st, usize::from(body.height));
        let content = self.bar().draw(ui, body, &scroll, total);
        let scroll_view = ScrollRegion::view(&scroll, content, total);
        let rows = Self::window(st, content, total);
        let g = self.geometry(content, st, model, rows.clone());
        let head = Rect {
            x: content.x,
            width: content.width,
            ..header
        };
        self.draw_header(ui, head, &g, st, live);
        if let Some(r) = reason
            && note.height > 0
        {
            let ns = self.ov.style(
                ui,
                self.id,
                Family::GRID,
                Variant::DEFAULT,
                Part::HEADER,
                live | StateFlags::READ_ONLY,
            );
            ui.fill(note, ns.style);
            paint_aligned(
                ui,
                Rect {
                    x: note.x.saturating_add(2),
                    width: note.width.saturating_sub(2),
                    ..note
                },
                r,
                Align::Left,
                ns.style,
            );
        }
        if len == 0 && !(self.fetch_on_activate && model.has_more()) {
            let empty = self.empty.unwrap_or(EmptyState::Empty {
                title: "Nothing here yet",
                hint: None,
            });
            let mid = Rect {
                y: content.y.saturating_add(content.height / 2),
                height: content.height.saturating_sub(content.height / 2),
                ..content
            };
            if let Some(f) = self.ov.slot_for(Part::EMPTY) {
                f(ui, mid);
            } else {
                let inherited = self
                    .ov
                    .style(
                        ui,
                        self.id,
                        Family::GRID,
                        Variant::DEFAULT,
                        Part::EMPTY,
                        live,
                    )
                    .style;
                empty.draw_inherited(ui, mid, 0, inherited);
            }
            self.draw_actions(ui, bar, live);
            return area;
        }
        let cursor = (
            st.fetch_row.map_or_else(
                || st.core.cursor_index().min(len.saturating_sub(1)),
                |f| f.boundary,
            ),
            self.cursor_col(st),
        );
        let range = self.range(st, model, cursor);
        let row_paint = RowPaint {
            content,
            geometry: &g,
            state: st,
            model,
            cursor,
            range,
            live,
        };
        for (offset, row) in rows.clone().enumerate() {
            let y = content
                .y
                .saturating_add(offset.min(usize::from(u16::MAX)) as u16);
            if y >= content.bottom() {
                break;
            }
            if row >= len {
                let more = Rect {
                    x: content.x,
                    y,
                    width: content.width,
                    height: 1,
                };
                let mut flags = live;
                if self.fetch_on_activate {
                    if !st.on_fetch_row() {
                        flags.remove(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
                    }
                    if ui.hovered_part(self.id) != Some(PartRef::of(Part::ROW)) {
                        flags.remove(StateFlags::HOVERED);
                    }
                    if ui.pressed_part(self.id) != Some(PartRef::of(Part::ROW)) {
                        flags.remove(StateFlags::PRESSED);
                    }
                }
                let ms = self.ov.style(
                    ui,
                    self.id,
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::ROW,
                    flags,
                );
                ui.fill(more, ms.style);
                if self.gutter != GridGutter::Compact {
                    self.draw_detailed_gutter(
                        ui,
                        &g,
                        y,
                        GutterRow {
                            data: None,
                            decor: RowDecor::default(),
                            flags,
                            style: ms.style,
                        },
                    );
                }
                let content = Rect {
                    x: g.content_x,
                    width: more.right().saturating_sub(g.content_x),
                    ..more
                };
                if let Some(frame) = self.fetch_activity {
                    Spinner::new(self.id.sub("fetch_activity"))
                        .frame(frame)
                        .label(self.fetch_label)
                        .gap(1)
                        .draw(ui, content);
                } else {
                    let reserved = width(ui.glyph_str(self.fetch_glyph)).min(content.width);
                    let glyph = match ms.glyph {
                        crate::theme::Slot::Inherit => Some(self.fetch_glyph),
                        crate::theme::Slot::Set(glyph) => Some(glyph),
                        crate::theme::Slot::Clear => None,
                    };
                    let used = glyph.map_or(reserved, |glyph| ui.glyph(content, glyph, ms.style));
                    ui.paint_str(
                        Rect {
                            x: content.x.saturating_add(used).saturating_add(1),
                            width: content.width.saturating_sub(used).saturating_sub(1),
                            ..content
                        },
                        self.fetch_label,
                        ms.style,
                    );
                }
                if !inert {
                    ui.register_part(self.id, PartRef::of(Part::ROW), more);
                }
                continue;
            }
            self.draw_row(ui, y, row, &row_paint);
        }
        if !inert {
            ui.scroll_edges(content, &scroll_view);
        }
        self.draw_actions(ui, bar, live);
        area
    }

    fn draw_actions(&self, ui: &mut Ui<'_>, bar: Rect, live: StateFlags) {
        let Some(f) = self.actions else { return };
        if bar.is_empty() {
            return;
        }
        let s = self.ov.style(
            ui,
            self.id,
            Family::GRID,
            Variant::DEFAULT,
            Part::ACTIONS,
            live,
        );
        ui.fill(bar, s.style);
        self.ov.slot_for(Part::ACTIONS).unwrap_or(f)(ui, bar);
    }

    pub fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        let mut w = self.gutter.widths(0).0.saturating_add(self.right_reserve);
        for col in self.columns.iter().take(self.column_count()) {
            w = w
                .saturating_add(col.min_width.max(width(col.title)))
                .saturating_add(1);
        }
        Size {
            min: (12, 3),
            preferred: (w, c.max.1),
        }
        .fit(c)
    }
}

const fn column_item_key(key: ColumnKey) -> ItemKey {
    ItemKey::num(key.raw() as u64)
}

impl Bindings for Grid<'_> {
    type Cmd = GridCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<GridCmd>] {
        &BINDINGS
    }
}
