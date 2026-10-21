use core::fmt;

use crate::theme::PaintStyle;
use ratatui_core::layout::Rect;

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

    /// Returns a revision that changes whenever the ordered row-key identity changes.
    /// Models without a cheap revision use an identity comparison in reusable adapters.
    fn revision(&self) -> Option<u64> {
        None
    }
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
            sort: None,
        }
    }
}

impl GridState {
    pub(crate) fn invalidate_model(&mut self) {
        self.core.invalidate();
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

    /// Applies additional cell style patches for semantic tones, after the
    /// resolved cell style and the tone's default foreground.
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
        let Some(target) = target else {
            return g;
        };
        if self.columns.get(target).is_some_and(|c| c.sticky) {
            return g;
        }
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
        if count == 0 {
            return g;
        }

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

        // Match the grid's reveal contract: once the cursor is visible, move the
        // left edge back only while every remaining column still fits. This
        // avoids leaving a needless blank viewport edge after a wider layout.
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

    fn toggle_row<M: GridModel + ?Sized>(
        &self,
        st: &mut GridState,
        model: &M,
        row: usize,
        acc: &mut Acc<GridAction>,
    ) {
        if row >= model.row_count() {
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
            st.core.invalidate();
        }

        let outcome = st.core.reconcile_with(len, |i| model.row_key(i), |_| true);
        if outcome != Reconciliation::Unchanged {
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
                                let all =
                                    (0..len).all(|i| st.core.checked().contains(model.row_key(i)));
                                if all {
                                    st.core.checked_mut().none();
                                } else {
                                    st.core.checked_mut().all();
                                }
                                acc.changed();
                            }
                        }
                        Some(GridCmd::Activate) => {
                            if len == 0 || column_count == 0 {
                                acc.consumed();
                            } else if matches!(self.nav, NavUnit::Row) {
                                acc.action(GridAction::Activated(model.row_key(row)));
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
                    match phase {
                        Phase::Press => self.move_to(st, model, row, col, false, acc),
                        Phase::DoubleClick => {
                            if matches!(self.nav, NavUnit::Row) {
                                acc.action(GridAction::Activated(row_key));
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
        let key = model.row_key(row);
        let Some(ck) = self.col_key(col) else { return };
        st.editor.set_error(None);
        if model.cell(row, col).is_none()
            || !self.columns.get(col).is_some_and(|c| c.editable)
            || !model.is_editable(row, col)
        {
            if cause == EditCause::Activate {
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
        // The detailed gutter has one trailing cell after the row number.
        // It belongs to the row surface even though it has no glyph or hit
        // target of its own.
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
            if cflags.contains(StateFlags::ACTIVE)
                && (!actions.is_empty() || cell.text.is_empty())
            {
                // Actions and empty values need a full-cell cursor surface.
                // Nonempty ordinary values keep the focused row surface in
                // unused padding and paint their independent cursor style
                // on the visible text below.
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
            // Deletion is row data, so its strike-through survives the active
            // cursor's independent style unless CELL explicitly removes it.
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
                // The active text style is independent from the focused row,
                // so apply authored cell overrides after painting that text.
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
            // Keep the reference grid's edge fade in the shared component.
            // The scroll helper checks for real overflow, skips cursor/reversed
            // cells, and blends only cells on the container background.
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};

    use super::*;
    use crate::event::MouseKind;
    use crate::runtime::stub::{key, mouse};
    use crate::runtime::{App, Runtime, UpdateCause};
    use crate::theme::Theme;
    use crate::validate::FieldError;

    const ID: Id = Id::root("grid.tests");
    const AREA: Rect = Rect::new(0, 0, 30, 6);
    const CALLBACK_ID: Id = Id::root("grid.tests.callback_padding");
    const CELL_ACTION: ActionKey = ActionKey::custom("follow");
    const CELL_ACTIONS: [CellAction; 1] = [CellAction::new(CELL_ACTION)];

    fn columns() -> [Column<'static>; 2] {
        [
            Column {
                key: ColumnKey::num(1),
                title: "Name",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: true,
                editable: true,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
            Column {
                key: ColumnKey::num(2),
                title: "Value",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: false,
                editable: true,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
        ]
    }

    struct NumericCellModel;

    impl GridModel for NumericCellModel {
        fn row_count(&self) -> usize {
            1
        }

        fn row_key(&self, _row: usize) -> ItemKey {
            ItemKey::num(7)
        }

        fn cell(&self, _row: usize, _column: usize) -> Option<CellRef<'_>> {
            Some(CellRef::new("7"))
        }
    }

    fn paint_identity_cell(cell: GridCell<'_>, ui: &mut CellUi<'_>) {
        ui.text(cell.value.text);
    }

    struct CallbackPaddingApp {
        state: GridState,
        callback: bool,
        dim_after: bool,
    }

    impl App for CallbackPaddingApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(CALLBACK_ID);
            }
            let mut column = Column::new(ColumnKey::num(7), "number");
            column.min_width = 10;
            column.max_width = 10;
            column.align = Align::Right;
            Grid::new(CALLBACK_ID, &[column])
                .update(cx, &mut self.state, &NumericCellModel)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let mut column = Column::new(ColumnKey::num(7), "number");
            column.min_width = 10;
            column.max_width = 10;
            column.align = Align::Right;
            let columns = [column];
            let grid = Grid::new(CALLBACK_ID, &columns);
            if self.callback {
                grid.cell(&paint_identity_cell).draw(
                    ui,
                    Rect::new(0, 0, 20, 4),
                    &self.state,
                    &NumericCellModel,
                );
            } else {
                grid.draw(ui, Rect::new(0, 0, 20, 4), &self.state, &NumericCellModel);
            }
            if self.dim_after {
                ui.dim_layer(Rect::new(0, 0, 20, 4), 2);
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    enum Mode {
        #[default]
        Inline,
        Cycle,
        External,
        Refuse,
    }

    #[derive(Default)]
    struct Model {
        rows: Vec<(ItemKey, [&'static str; 2])>,
        mode: Mode,
        cycles: usize,
        commits: Vec<String>,
        fail_commit: bool,
        locked: bool,
        reason: Option<&'static str>,
        cell_action: bool,
        hole: Option<(usize, usize)>,
        strike: bool,
    }

    impl Model {
        fn two() -> Self {
            Model {
                rows: vec![
                    (ItemKey::num(10), ["alpha", "1"]),
                    (ItemKey::num(20), ["beta", "2"]),
                ],
                ..Model::default()
            }
        }
    }

    #[test]
    fn keyed_cursor_errors_preserve_complete_state_and_typed_reason() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        let model = Model::two();
        let mut state = GridState {
            edit: Some((ItemKey::num(10), ColumnKey::num(1))),
            ..GridState::default()
        };
        state.editor.begin("retained draft");
        let before = state.clone();
        for (row, col, expected) in [
            (99, 1, GridCursorError::UnknownRow(ItemKey::num(99))),
            (10, 99, GridCursorError::UnknownColumn(ColumnKey::num(99))),
            (20, 2, GridCursorError::Editing),
        ] {
            assert_eq!(
                grid.move_cursor_to(&mut state, &model, ItemKey::num(row), ColumnKey::num(col)),
                Err(expected)
            );
            assert_eq!(state, before);
        }
    }

    #[test]
    fn keyed_cursor_clears_range_but_preserves_checked_rows_and_sticky_window() {
        let mut columns = columns().to_vec();
        columns[0].sticky = true;
        columns.push(Column::new(ColumnKey::num(3), "last"));
        let grid = Grid::new(ID, &columns);
        let model = Model::two();
        let mut state = GridState {
            anchor: Some(RangeAnchor::Cell(ItemKey::num(20), ColumnKey::num(2))),
            col_offset: 1,
            ..GridState::default()
        };
        state.core.checked_mut().insert(ItemKey::num(20));
        assert_eq!(
            grid.move_cursor_to(&mut state, &model, ItemKey::num(10), ColumnKey::num(1)),
            Ok(())
        );
        assert_eq!(state.anchor, None);
        assert!(state.core.checked().contains(ItemKey::num(20)));
        assert_eq!(state.col_offset, 1);
        assert_eq!(state.core.scroll().pending_reveal(), Some(0));
    }

    impl GridModel for Model {
        fn row_count(&self) -> usize {
            self.rows.len()
        }

        fn row_key(&self, row: usize) -> ItemKey {
            self.rows.get(row).map_or(ItemKey::index(row), |r| r.0)
        }

        fn row_decor(&self, row: usize) -> RowDecor<'_> {
            RowDecor {
                strike: self.strike && row == 0,
                ..RowDecor::default()
            }
        }

        fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
            if self.hole == Some((row, col)) {
                return None;
            }
            self.rows
                .get(row)
                .and_then(|r| r.1.get(col))
                .copied()
                .map(CellRef::new)
        }

        fn read_only_reason(&self) -> Option<&str> {
            self.reason
        }

        fn actions(&self, row: usize, col: usize) -> &[CellAction] {
            if self.cell_action && row == 0 && col == 0 {
                &CELL_ACTIONS
            } else {
                &[]
            }
        }
    }

    impl GridEditor for Model {
        fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
            match self.mode {
                Mode::Inline => EditIntent::Inline { initial: "alpha" },
                Mode::Cycle => EditIntent::Cycle,
                Mode::External => EditIntent::External,
                Mode::Refuse => EditIntent::Refuse { reason: "locked" },
            }
        }

        fn apply_cycle(&mut self, _row: usize, _col: usize) {
            self.cycles = self.cycles.saturating_add(1);
        }

        fn commit_cell(&mut self, _row: usize, _col: usize, text: &str) -> Result<(), FieldError> {
            if self.fail_commit {
                Err(FieldError::coded("rejected", "grid-test"))
            } else {
                self.commits.push(text.to_owned());
                Ok(())
            }
        }

        fn is_editable(&self, _row: usize, _col: usize) -> bool {
            !self.locked
        }
    }

    #[derive(Default)]
    struct RaggedModel {
        second_present: Cell<bool>,
        absent_decor: Cell<usize>,
        absent_actions: Cell<usize>,
        editor_hooks: Cell<usize>,
    }

    impl GridModel for RaggedModel {
        fn row_count(&self) -> usize {
            1
        }

        fn row_key(&self, _row: usize) -> ItemKey {
            ItemKey::num(1)
        }

        fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
            match col {
                0 => Some(CellRef::new("present")),
                1 if self.second_present.get() => Some(CellRef::new("action")),
                _ => None,
            }
        }

        fn cell_decor(&self, _row: usize, col: usize) -> CellDecor<'_> {
            if col == 1 {
                self.absent_decor
                    .set(self.absent_decor.get().saturating_add(1));
            }
            CellDecor::default()
        }

        fn actions(&self, _row: usize, col: usize) -> &[CellAction] {
            if col == 1 && !self.second_present.get() {
                self.absent_actions
                    .set(self.absent_actions.get().saturating_add(1));
            }
            if col == 1 && self.second_present.get() {
                &CELL_ACTIONS
            } else {
                &[]
            }
        }
    }

    impl GridEditor for RaggedModel {
        fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            EditIntent::Cycle
        }

        fn apply_cycle(&mut self, _row: usize, _col: usize) {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
        }

        fn commit_cell(&mut self, _row: usize, _col: usize, _text: &str) -> Result<(), FieldError> {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            Ok(())
        }

        fn is_editable(&self, _row: usize, _col: usize) -> bool {
            self.editor_hooks
                .set(self.editor_hooks.get().saturating_add(1));
            true
        }
    }

    struct RaggedApp {
        state: GridState,
        model: RaggedModel,
        actions: Vec<GridAction>,
    }

    impl App for RaggedApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut columns = columns();
            if let Some(column) = columns.get_mut(1) {
                column.sortable = true;
            }
            let mut response =
                Grid::new(ID, &columns).update_editable(cx, &mut self.state, &mut self.model);
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let mut columns = columns();
            if let Some(column) = columns.get_mut(1) {
                column.sortable = true;
            }
            Grid::new(ID, &columns).draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct DisplayOnlyModel;

    impl GridModel for DisplayOnlyModel {
        fn row_count(&self) -> usize {
            1
        }

        fn row_key(&self, _row: usize) -> ItemKey {
            ItemKey::num(1)
        }

        fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
            (col == 0).then_some(CellRef::new("display only"))
        }
    }

    struct DisplayOnlyApp {
        state: GridState,
        model: DisplayOnlyModel,
    }

    impl App for DisplayOnlyApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Grid::new(ID, &columns())
                .update(cx, &mut self.state, &self.model)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns()).draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct GridApp {
        state: GridState,
        model: Model,
        editable: bool,
        actions: Vec<GridAction>,
        part_defaults: Vec<(Part, StylePatch)>,
        gutter: GridGutter,
    }

    impl GridApp {
        fn new(model: Model, editable: bool) -> Self {
            GridApp {
                state: GridState::default(),
                model,
                editable,
                actions: Vec::new(),
                part_defaults: Vec::new(),
                gutter: GridGutter::Compact,
            }
        }
    }

    impl App for GridApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let columns = columns();
            let grid = Grid::new(ID, &columns)
                .part_defaults(&self.part_defaults)
                .gutter(self.gutter)
                .select_mode(SelectMode::Multi);
            let mut response = if self.editable {
                grid.update_editable(cx, &mut self.state, &mut self.model)
            } else {
                grid.update(cx, &mut self.state, &self.model)
            };
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns())
                .part_defaults(&self.part_defaults)
                .gutter(self.gutter)
                .select_mode(SelectMode::Multi)
                .draw(ui, AREA, &self.state, &self.model);
        }
    }

    struct AreaGridApp {
        state: GridState,
        model: Model,
        area: Rect,
    }

    impl App for AreaGridApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            Grid::new(ID, &columns())
                .update(cx, &mut self.state, &self.model)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(ID, &columns()).draw(ui, self.area, &self.state, &self.model);
        }
    }

    fn runtime(model: Model, editable: bool) -> (Runtime<GridApp>, Buffer) {
        let mut runtime = Runtime::new(GridApp::new(model, editable), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        (runtime, buffer)
    }

    #[test]
    fn overflowing_grid_fades_only_real_scroll_edges() {
        let area = Rect::new(0, 0, 30, 8);
        let mut model = Model::two();
        model
            .rows
            .extend((2..12).map(|row| (ItemKey::num(10 + row), ["alpha", "1"])));
        let mut state = GridState::default();
        state.core.scroll_mut().set_content(model.row_count());
        state.core.scroll_mut().set_viewport(4);
        state.core.scroll_mut().scroll_to(4);
        let app = AreaGridApp { state, model, area };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let alpha_foreground = |y: u16| {
            buffer
                .content()
                .iter()
                .enumerate()
                .find(|(index, cell)| {
                    index / usize::from(area.width) == usize::from(y) && cell.symbol() == "a"
                })
                .map(|(_, cell)| cell.fg)
                .expect("visible alpha cell")
        };
        assert_eq!(
            alpha_foreground(6),
            ratatui_core::style::Color::Rgb(255, 255, 255)
        );
        assert_eq!(
            alpha_foreground(7),
            ratatui_core::style::Color::Rgb(140, 140, 140)
        );

        let fit_app = AreaGridApp {
            state: GridState::default(),
            model: Model::two(),
            area,
        };
        let mut fit_runtime = Runtime::new(fit_app, Theme::junie());
        let _ = fit_runtime.initialize();
        let mut fit_buffer = Buffer::empty(area);
        fit_runtime
            .draw_buffer(area, &mut fit_buffer)
            .commit_presented();
        assert!(
            fit_buffer
                .content()
                .iter()
                .enumerate()
                .filter(|(index, cell)| {
                    index / usize::from(area.width) > 0 && cell.symbol() == "a"
                })
                .all(|(_, cell)| cell.fg == ratatui_core::style::Color::Rgb(255, 255, 255))
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "columns \"Name\" and \"Value\" share ColumnKey::num(1)")]
    fn duplicate_column_keys_are_rejected_by_name() {
        let mut columns = columns();
        if let Some(column) = columns.get_mut(1) {
            column.key = ColumnKey::num(1);
        }
        Grid::new(ID, &columns).assert_distinct_column_keys();
    }

    #[test]
    fn right_header_overflow_reserves_space_before_truncating_the_title() {
        let columns = [
            Column {
                key: ColumnKey::num(1),
                title: "total_amount_and_more",
                subtitle: None,
                align: Align::Left,
                min_width: 18,
                max_width: 18,
                sortable: false,
                editable: false,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
            Column {
                key: ColumnKey::num(2),
                title: "second",
                subtitle: None,
                align: Align::Left,
                min_width: 8,
                max_width: 8,
                sortable: false,
                editable: false,
                sticky: false,
                prefix_glyph: None,
                badge: None,
            },
        ];
        let model = Model::two();
        let area = Rect::new(0, 0, 20, 5);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).draw(ui, area, &GridState::default(), &model);
            })
            .commit_presented();

        assert_eq!(
            buffer
                .cell(Position::new(17, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("…")
        );
        assert_eq!(
            buffer
                .cell(Position::new(18, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("1")
        );
        assert_eq!(
            buffer
                .cell(Position::new(19, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("›")
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "share ColumnKey::num(1)")]
    fn draw_rejects_duplicate_column_keys() {
        let mut columns = columns();
        if let Some(column) = columns.get_mut(1) {
            column.key = ColumnKey::num(1);
        }
        let model = Model::two();
        let screen = AREA;
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime
            .draw_scene(screen, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).draw(ui, screen, &GridState::default(), &model);
            })
            .commit_presented();
    }

    #[test]
    #[should_panic(expected = "ColumnKey::num is 0..=0x7FFF")]
    fn column_key_num_rejects_the_hashed_range() {
        let _ = ColumnKey::num(core::hint::black_box(0x8000));
    }

    #[test]
    fn column_key_num_keeps_every_number_it_accepts() {
        for n in [0_u16, 1, 255, 0x7FFE, COLUMN_KEY_MAX] {
            assert_eq!(ColumnKey::num(n).raw(), n, "num must not fold {n}");
            assert!(ColumnKey::num(n).raw() <= COLUMN_KEY_MAX);
        }
    }

    #[test]
    fn numbered_and_named_column_keys_never_meet() {
        for name in ["id", "name", "schema.table", "", "\u{1f600}"] {
            let named = ColumnKey::of(name);
            assert!(named.raw() > COLUMN_KEY_MAX, "{name:?} left the high half");
            for n in [0_u16, 1, 255, 0x7FFF] {
                assert_ne!(named, ColumnKey::num(n));
            }
        }
    }

    #[test]
    fn column_key_of_uses_the_shared_custom_fold() {
        for name in ["id", "name", "schema.table", ""] {
            assert_eq!(ColumnKey::of(name).raw(), custom_hash16(name));
            assert_eq!(ColumnKey::of(name).raw(), Part::custom(name).raw());
        }
        assert_ne!(ColumnKey::of("id"), ColumnKey::of("name"));
        assert_eq!(ColumnKey::of("id"), ColumnKey::of("id"));
    }

    #[test]
    fn column_key_debug_names_its_range() {
        assert_eq!(format!("{:?}", ColumnKey::num(2)), "ColumnKey::num(2)");
        assert_eq!(
            format!("{:?}", ColumnKey::of("id")),
            format!("ColumnKey::of(#{:04x})", ColumnKey::of("id").raw())
        );
    }

    #[test]
    fn an_absent_column_index_has_no_key_rather_than_a_sentinel() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        assert_eq!(grid.col_key(0), Some(ColumnKey::num(1)));
        assert_eq!(grid.col_key(1), Some(ColumnKey::num(2)));
        assert_eq!(grid.col_key(2), None);
        assert_eq!(grid.col_key(usize::MAX), None);

        const { assert!(u16::MAX > COLUMN_KEY_MAX) };
    }

    #[test]
    fn parts_and_slot_contract_are_exact() {
        assert_eq!(
            Grid::PARTS,
            &[
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
            ]
        );
        let addressable = [
            Part::HEADER,
            Part::EMPTY,
            Part::ACTIONS,
            Part::TRACK,
            Part::THUMB,
        ];
        assert!(addressable.iter().all(|part| Grid::PARTS.contains(part)));
    }

    #[test]
    fn grid_has_no_whole_surface_status_configuration() {
        let columns = columns();
        let debug = format!("{:?}", Grid::new(ID, &columns));
        assert!(!debug.contains("status"));
        assert_eq!(Grid::PARTS.len(), 15);
        assert!(Grid::new(ID, &columns).header_prefixes.is_empty());
    }

    #[test]
    fn zero_rows_render_explicit_loading_and_error_empty_states() {
        fn render(empty: EmptyState<'_>) -> String {
            let columns = columns();
            let model = Model::default();
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(AREA);
            runtime
                .draw_scene(AREA, &mut buffer, |ui, _| {
                    Grid::new(ID, &columns).empty(empty).draw(
                        ui,
                        AREA,
                        &GridState::default(),
                        &model,
                    );
                })
                .commit_presented();
            buffer
                .content()
                .iter()
                .map(ratatui_core::buffer::Cell::symbol)
                .collect()
        }

        assert!(
            render(EmptyState::Loading {
                label: "Loading rows"
            })
            .contains("Loading rows")
        );
        assert!(
            render(EmptyState::Error {
                message: "Load failed",
                detail: Some("Retry later"),
            })
            .contains("Load failed")
        );
    }

    fn focused_grid_buffer(theme: Theme) -> Buffer {
        focused_grid_buffer_with_model(theme, Model::two())
    }

    fn focused_grid_buffer_with_model(theme: Theme, model: Model) -> Buffer {
        focused_grid_buffer_with_defaults(theme, model, Vec::new())
    }

    fn focused_grid_buffer_with_defaults(
        theme: Theme,
        model: Model,
        part_defaults: Vec<(Part, StylePatch)>,
    ) -> Buffer {
        let mut app = GridApp::new(model, false);
        app.part_defaults = part_defaults;
        let mut runtime = Runtime::new(app, theme);
        let _ = runtime.initialize();
        runtime.set_focus(Some(ID));
        let _ = runtime.settle();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        buffer
    }

    #[test]
    fn identity_cell_callback_preserves_right_aligned_padding_style() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Ansi256,
            crate::theme::ColorLevel::Ansi16,
            crate::theme::ColorLevel::Mono,
        ] {
            let theme = Theme::junie().for_level(level);
            let mut buffers = Vec::new();
            for callback in [false, true] {
                let mut runtime = Runtime::new(
                    CallbackPaddingApp {
                        state: GridState::default(),
                        callback,
                        dim_after: false,
                    },
                    theme.clone(),
                );
                let _ = runtime.initialize();
                runtime.set_focus(Some(CALLBACK_ID));
                let _ = runtime.settle();
                let area = Rect::new(0, 0, 20, 4);
                let mut buffer = Buffer::empty(area);
                runtime.draw_buffer(area, &mut buffer).commit_presented();
                buffers.push(buffer);
            }

            let [default, callback] = buffers.as_slice() else {
                panic!("both grid renderings were captured");
            };
            for x in 0..15 {
                let expected = default.cell(Position::new(x, 1)).expect("default cell");
                let actual = callback.cell(Position::new(x, 1)).expect("callback cell");
                assert_eq!(
                    (
                        actual.symbol(),
                        actual.fg,
                        actual.bg,
                        actual.modifier,
                        actual.underline_color
                    ),
                    (
                        expected.symbol(),
                        expected.fg,
                        expected.bg,
                        expected.modifier,
                        expected.underline_color
                    ),
                    "identity callback changed the aligned cell or its padding at {x} in {level:?}"
                );
            }
        }
    }

    fn focused_detailed_grid_buffer(theme: Theme, model: Model) -> Buffer {
        let mut app = GridApp::new(model, false);
        app.gutter = GridGutter::Detailed {
            row_numbers: true,
            min_digits: 1,
        };
        let mut runtime = Runtime::new(app, theme);
        let _ = runtime.initialize();
        runtime.set_focus(Some(ID));
        let _ = runtime.settle();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        buffer
    }

    #[test]
    fn focused_grid_row_style_inherits_into_non_cursor_cells_at_every_color_level() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Ansi256,
            crate::theme::ColorLevel::Ansi16,
            crate::theme::ColorLevel::Mono,
        ] {
            let theme = Theme::junie()
                .override_family(Family::GRID, |recipe| {
                    recipe.part(Part::ROW).when(
                        StateFlags::FOCUSED,
                        StylePatch::new()
                            .add(Modifier::ITALIC)
                            .set_underline(Role::Fg(FgStep::Faint)),
                    );
                })
                .for_level(level);
            let mut model = Model::two();
            model.hole = Some((0, 1));
            let buffer = focused_grid_buffer_with_model(theme, model);
            let cursor = buffer
                .cell(Position::new(2, 1))
                .expect("cursor cell exists");
            let non_cursor = buffer
                .cell(Position::new(11, 1))
                .expect("non-cursor cell exists");
            let row_gutter = buffer
                .cell(Position::new(1, 1))
                .expect("row gutter cell exists");

            assert!(
                non_cursor.modifier.contains(Modifier::ITALIC),
                "the focused row style reaches ordinary cells at {level:?}"
            );
            assert!(
                non_cursor.modifier.contains(Modifier::BOLD),
                "the focused Grid row uses the baseline row weight at {level:?}"
            );
            assert!(
                !cursor.modifier.contains(Modifier::ITALIC),
                "the ACTIVE cursor cell replaces its inherited row style at {level:?}"
            );
            assert_eq!(
                non_cursor.underline_color, row_gutter.underline_color,
                "semantic row underline provenance survives CELL binding at {level:?}"
            );
            assert_eq!(
                cursor.underline_color,
                ratatui_core::style::Color::Reset,
                "the ACTIVE cursor cell keeps its independent underline style at {level:?}"
            );
        }
    }

    #[test]
    fn detailed_cursor_keeps_cell_padding_and_gutter_tail_on_focused_row_style() {
        let buffer = focused_detailed_grid_buffer(Theme::junie(), Model::two());
        let number = buffer
            .cell(Position::new(3, 1))
            .expect("detailed row number exists");
        let gutter_tail = buffer
            .cell(Position::new(4, 1))
            .expect("detailed gutter tail exists");
        let active_text = buffer
            .cell(Position::new(5, 1))
            .expect("active cell text exists");
        let active_padding = buffer
            .cell(Position::new(10, 1))
            .expect("active cell padding exists");

        assert!(!number.modifier.contains(Modifier::BOLD));
        assert!(gutter_tail.modifier.contains(Modifier::BOLD));
        assert!(active_padding.modifier.contains(Modifier::BOLD));
        assert_ne!(active_text.bg, active_padding.bg);
    }

    #[test]
    fn active_empty_cell_replaces_focused_row_style() {
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe.part(Part::ROW).when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Red))
                    .add(Modifier::ITALIC),
            );
        });
        let mut model = Model::two();
        model.hole = Some((0, 0));
        let buffer = focused_grid_buffer_with_model(theme, model);
        let active = buffer
            .cell(Position::new(2, 1))
            .expect("empty active cell exists");
        assert!(!active.modifier.contains(Modifier::ITALIC));
        assert_ne!(active.fg, ratatui_core::style::Color::Red);
    }

    #[test]
    fn active_empty_string_cell_replaces_focused_row_style() {
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe.part(Part::ROW).when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Red))
                    .add(Modifier::ITALIC),
            );
        });
        let model = Model {
            rows: vec![(ItemKey::num(10), ["", "1"])],
            ..Model::default()
        };
        let buffer = focused_grid_buffer_with_model(theme, model);
        let active = buffer
            .cell(Position::new(2, 1))
            .expect("empty-string active cell exists");
        assert!(!active.modifier.contains(Modifier::ITALIC));
        assert_ne!(active.fg, ratatui_core::style::Color::Red);
    }

    #[test]
    fn caller_part_default_clears_reach_cells_and_empty_cells() {
        let cell_clear_theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe.part(Part::ROW).when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Red))
                    .set_bg(Role::Custom(ratatui_core::style::Color::Blue)),
            );
        });
        let row_clear_theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Red))
                    .set_bg(Role::Custom(ratatui_core::style::Color::Blue)),
            );
        });
        for (theme, defaults) in [
            (
                cell_clear_theme,
                vec![(Part::CELL, StylePatch::new().clear_fg().clear_bg())],
            ),
            (
                row_clear_theme,
                vec![(Part::ROW, StylePatch::new().clear_fg().clear_bg())],
            ),
        ] {
            let mut model = Model::two();
            model.hole = Some((0, 1));
            let buffer = focused_grid_buffer_with_defaults(theme, model, defaults);
            let cell = buffer
                .cell(Position::new(11, 1))
                .expect("non-cursor cell exists");
            assert_eq!(cell.fg, ratatui_core::style::Color::Reset);
            assert_eq!(cell.bg, ratatui_core::style::Color::Reset);
        }
    }

    #[test]
    fn part_default_clear_does_not_override_a_theme_recipe_value() {
        let cell_theme = Theme::junie().define_family(Family::GRID, |recipe| {
            recipe
                .part(Part::CELL)
                .base(StylePatch::new().set_fg(Role::Custom(ratatui_core::style::Color::Green)));
        });
        let row_theme = Theme::junie().define_family(Family::GRID, |recipe| {
            recipe
                .part(Part::ROW)
                .base(StylePatch::new().set_fg(Role::Custom(ratatui_core::style::Color::Red)));
        });
        for (theme, defaults, expected) in [
            (
                cell_theme,
                vec![(Part::CELL, StylePatch::new().clear_fg())],
                ratatui_core::style::Color::Green,
            ),
            (
                row_theme,
                vec![(Part::ROW, StylePatch::new().clear_fg())],
                ratatui_core::style::Color::Red,
            ),
        ] {
            let mut model = Model::two();
            model.hole = Some((0, 1));
            let buffer = focused_grid_buffer_with_defaults(theme, model, defaults);
            assert_eq!(
                buffer
                    .cell(Position::new(11, 1))
                    .expect("non-cursor cell exists")
                    .fg,
                expected
            );
        }
    }

    #[test]
    fn deleted_row_strike_survives_active_cursor_style_unless_cell_removes_it() {
        let mut model = Model::two();
        model.strike = true;
        let buffer = focused_grid_buffer_with_model(Theme::junie(), model);
        for position in [Position::new(2, 1), Position::new(11, 1)] {
            assert!(
                buffer
                    .cell(position)
                    .expect("grid cell exists")
                    .modifier
                    .contains(Modifier::CROSSED_OUT),
                "deleted row decoration is preserved in active and ordinary cells"
            );
        }

        let mut model = Model::two();
        model.strike = true;
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe
                .part(Part::CELL)
                .base(StylePatch::new().remove(Modifier::CROSSED_OUT));
        });
        let buffer = focused_grid_buffer_with_model(theme, model);
        assert!(
            !buffer
                .cell(Position::new(2, 1))
                .expect("cursor cell exists")
                .modifier
                .contains(Modifier::CROSSED_OUT),
            "an explicit CELL removal overrides row strike for the active cell"
        );
        assert!(
            !buffer
                .cell(Position::new(11, 1))
                .expect("ordinary cell exists")
                .modifier
                .contains(Modifier::CROSSED_OUT),
            "an explicit CELL removal overrides inherited row strike"
        );
    }

    #[test]
    fn empty_focused_row_cells_inherit_row_style() {
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe
                .part(Part::ROW)
                .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD));
        });
        let mut model = Model::two();
        model.hole = Some((0, 1));
        let mut runtime = Runtime::new(GridApp::new(model, false), theme);
        let _ = runtime.initialize();
        runtime.set_focus(Some(ID));
        let _ = runtime.settle();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let hole = buffer
            .cell(Position::new(11, 1))
            .expect("empty cell exists");

        assert_eq!(hole.symbol(), " ");
        assert!(hole.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn explicit_cell_style_overrides_inherited_row_channels_and_modifiers() {
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe
                .part(Part::ROW)
                .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD));
            recipe.part(Part::CELL).base(
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Green))
                    .set_bg(Role::Custom(ratatui_core::style::Color::Blue))
                    .remove(Modifier::BOLD),
            );
        });
        let buffer = focused_grid_buffer(theme);
        let non_cursor = buffer
            .cell(Position::new(11, 1))
            .expect("non-cursor cell exists");

        assert_eq!(non_cursor.fg, ratatui_core::style::Color::Green);
        assert_eq!(non_cursor.bg, ratatui_core::style::Color::Blue);
        assert!(!non_cursor.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn explicit_cell_clear_resets_inherited_channels_in_data_and_empty_cells() {
        let mut cell_clear = StylePatch::new().clear_fg().clear_bg();
        cell_clear.underline = Slot::Clear;
        let theme = Theme::junie().override_family(Family::GRID, |recipe| {
            recipe.part(Part::ROW).when(
                StateFlags::FOCUSED,
                StylePatch::new()
                    .set_fg(Role::Custom(ratatui_core::style::Color::Red))
                    .set_bg(Role::Custom(ratatui_core::style::Color::Blue))
                    .set_underline(Role::Custom(ratatui_core::style::Color::Yellow)),
            );
            recipe.part(Part::CELL).base(cell_clear);
        });
        let mut model = Model::two();
        model.hole = Some((0, 1));
        let mut runtime = Runtime::new(GridApp::new(model, false), theme);
        let _ = runtime.initialize();
        runtime.set_focus(Some(ID));
        let _ = runtime.settle();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        for position in [Position::new(2, 1), Position::new(11, 1)] {
            let cell = buffer.cell(position).expect("grid cell exists");
            assert_eq!(cell.fg, ratatui_core::style::Color::Reset);
            assert_eq!(cell.bg, ratatui_core::style::Color::Reset);
            assert_eq!(cell.underline_color, ratatui_core::style::Color::Reset);
        }
        let gutter = buffer.cell(Position::new(0, 1)).expect("gutter exists");
        assert_eq!(gutter.fg, ratatui_core::style::Color::Red);
        assert_eq!(gutter.bg, ratatui_core::style::Color::Blue);
    }

    #[test]
    fn inherited_row_channels_keep_explicit_cell_clear_provenance() {
        let parent = PaintStyle::bound(
            ratatui_core::style::Style::new()
                .fg(ratatui_core::style::Color::Red)
                .bg(ratatui_core::style::Color::Blue)
                .add_modifier(Modifier::BOLD),
            Some(Role::Danger),
            Some(Role::Info),
            crate::theme::Surface::Canvas,
        );
        let defaults = inherited_style_defaults(parent, StylePatch::new());
        let clear = defaults.base.merge(
            StylePatch::new()
                .clear_fg()
                .clear_bg()
                .remove(Modifier::BOLD),
        );

        assert_eq!(clear.fg, Slot::Clear);
        assert_eq!(clear.bg, Slot::Clear);
        assert!(clear.remove.contains(Modifier::BOLD));
    }

    #[test]
    fn slot_surface_equals_the_documented_invariant_r_set() {
        fn render(slot: Option<Part>) -> Buffer {
            let screen = Rect::new(0, 0, 20, 10);
            let replace = |ui: &mut Ui<'_>, area: Rect| {
                ui.paint_str(area, "#", ui.surface_style());
            };
            let columns = columns();
            let mut model = Model::two();
            model.cell_action = true;
            model
                .rows
                .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
            let empty = Model::default();
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(screen);
            runtime
                .draw_scene(screen, &mut buffer, |ui, _| {
                    let mut full = Grid::new(ID, &columns).actions_slot(&replace);
                    let mut blank = Grid::new(ID.sub("empty"), &columns);
                    if let Some(part) = slot {
                        full = full.slot(part, &replace);
                        blank = blank.slot(part, &replace);
                    }
                    full.draw(ui, Rect::new(0, 0, 12, 6), &GridState::default(), &model);
                    blank.draw(ui, Rect::new(0, 7, 12, 3), &GridState::default(), &empty);
                })
                .commit_presented();
            buffer
        }

        let baseline = render(None);
        let addressable = [
            Part::HEADER,
            Part::EMPTY,
            Part::ACTIONS,
            Part::TRACK,
            Part::THUMB,
        ];
        for &part in Grid::PARTS {
            assert_eq!(
                render(Some(part)) != baseline,
                addressable.contains(&part),
                "slot truth disagrees for {part:?}"
            );
        }
    }

    #[test]
    fn state_accessors_expose_only_keyed_owned_state() {
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 1, ColumnKey::num(2));
        state.core.checked_mut().insert(ItemKey::num(20));
        state.col_offset = 3;
        state.edit = Some((ItemKey::num(20), ColumnKey::num(2)));
        state
            .editor
            .set_error(Some(FieldError::coded("bad", "typed")));

        assert_eq!(state.cursor(), Some((ItemKey::num(20), ColumnKey::num(2))));
        assert!(state.selected_rows().contains(ItemKey::num(20)));
        assert!(state.is_editing());
        assert_eq!(state.edit_error().and_then(|e| e.code), Some("typed"));
        assert_eq!(state.col_offset(), 3);
    }

    #[test]
    fn grid_edit_errors_resolve_plain_sensitivity_but_keep_secret_errors_redacted() {
        const DETAIL: &str = "validation detail";

        let mut state = GridState::default();
        state
            .editor
            .set_error(Some(FieldError::coded(DETAIL, "typed")));
        assert_eq!(
            state.edit_error().map(|error| error.message.as_ref()),
            Some(DETAIL)
        );
        assert_eq!(
            state.edit_error().and_then(|error| error.code),
            Some("typed")
        );

        state.editor.set_sensitive(true);
        state
            .editor
            .set_error(Some(FieldError::coded(DETAIL, "secret")));
        assert_eq!(
            state.edit_error().map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(state.edit_error().and_then(|error| error.code).is_none());
    }

    #[test]
    fn grid_model_only_supports_read_only_update_and_draw() {
        let app = DisplayOnlyApp {
            state: GridState::default(),
            model: DisplayOnlyModel,
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));

        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
    }

    #[test]
    fn read_only_f2_never_activates_but_enter_still_does() {
        let (mut runtime, _) = runtime(Model::two(), false);

        let f2 = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(2)));
        assert!(f2.is_consumed());
        assert!(runtime.app().actions.is_empty());

        let enter = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert!(enter.is_consumed());
        assert_eq!(
            runtime.app().actions,
            [GridAction::Activated(ItemKey::num(10))]
        );
    }

    #[test]
    fn explicit_edit_has_no_activation_fallback_for_a_locked_cell() {
        let mut model = Model::two();
        model.locked = true;
        let (mut runtime, _) = runtime(model, true);

        let f2 = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::F(2)));
        assert!(f2.is_consumed());
        assert!(!runtime.app().state.is_editing());
        assert!(runtime.app().actions.is_empty());

        let enter = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert!(enter.is_consumed());
        assert!(!runtime.app().state.is_editing());
        assert_eq!(
            runtime.app().actions,
            [GridAction::Activated(ItemKey::num(10))]
        );
    }

    #[test]
    fn ragged_holes_remain_navigable_without_cell_hooks() {
        let app = RaggedApp {
            state: GridState::default(),
            model: RaggedModel::default(),
            actions: Vec::new(),
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));

        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
        assert_eq!(
            runtime.app().actions,
            [GridAction::Moved, GridAction::Activated(ItemKey::num(1))]
        );
        assert_eq!(runtime.app().model.absent_decor.get(), 0);
        assert_eq!(runtime.app().model.absent_actions.get(), 0);
        assert_eq!(runtime.app().model.editor_hooks.get(), 0);

        let header = runtime
            .area_of_part(
                ID,
                PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(2))),
            )
            .unwrap_or(Rect::ZERO);
        assert!(!header.is_empty());
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, header.x, header.y));
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, header.x, header.y));
        assert_eq!(
            runtime.app().actions.last(),
            Some(&GridAction::Sort(ColumnKey::num(2), SortDir::Asc))
        );

        let pointer_app = RaggedApp {
            state: GridState::default(),
            model: RaggedModel::default(),
            actions: Vec::new(),
        };
        let mut pointer = Runtime::new(pointer_app, Theme::junie());
        let _ = pointer.initialize();
        pointer.draw_buffer(AREA, &mut buffer).commit_presented();
        pointer.draw_buffer(AREA, &mut buffer).commit_presented();
        let hole = pointer
            .area_of_part(ID, PartRef::item(Part::CELL, ItemKey::num(1)))
            .unwrap_or(Rect::ZERO);
        assert!(!hole.is_empty());
        assert_eq!(hole.x, 11);
        let _ = crate::runtime::stub::deliver(&mut pointer, mouse(MouseKind::Down, hole.x, hole.y));
        assert_eq!(
            pointer.app().state.cursor(),
            Some((ItemKey::num(1), ColumnKey::num(2)))
        );
        assert_eq!(pointer.app().model.absent_decor.get(), 0);
        assert_eq!(pointer.app().model.absent_actions.get(), 0);
        assert_eq!(pointer.app().model.editor_hooks.get(), 0);

        let columns = columns();
        let grid = Grid::new(ID, &columns);
        assert_eq!(
            grid.copy_tsv(&runtime.app().state, &runtime.app().model, (0, 1)),
            "\n"
        );
    }

    #[test]
    fn ragged_hole_resolves_and_fills_cell_patch() {
        let patch = [(Part::CELL, StylePatch::new().add(Modifier::BOLD))];
        let columns = columns();
        let model = RaggedModel::default();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).patch_part(&patch).draw(
                    ui,
                    AREA,
                    &GridState::default(),
                    &model,
                );
            })
            .commit_presented();

        assert!(
            buffer
                .cell(Position::new(11, 1))
                .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD))
        );
        assert_eq!(model.absent_decor.get(), 0);
        assert_eq!(model.absent_actions.get(), 0);
    }

    #[test]
    fn stale_cell_action_intent_rechecks_cell_presence() {
        let model = RaggedModel {
            second_present: Cell::new(true),
            ..RaggedModel::default()
        };
        let app = RaggedApp {
            state: GridState::default(),
            model,
            actions: Vec::new(),
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
        let affordance = runtime
            .area_of_part(ID, PartRef::item(Part::ACTIONS, ItemKey::num(1)))
            .unwrap_or(Rect::ZERO);
        assert!(!affordance.is_empty());

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Down, affordance.x, affordance.y),
        );
        runtime.app().model.second_present.set(false);
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Up, affordance.x, affordance.y),
        );

        assert!(
            runtime
                .app()
                .actions
                .iter()
                .all(|action| !matches!(action, GridAction::CellAction(..)))
        );
        assert_eq!(runtime.app().model.absent_actions.get(), 0);
    }

    #[test]
    fn cell_alignment_distinguishes_inheritance_from_explicit_left() {
        struct AlignmentModel(Option<Align>);

        impl GridModel for AlignmentModel {
            fn row_count(&self) -> usize {
                1
            }

            fn row_key(&self, _row: usize) -> ItemKey {
                ItemKey::num(1)
            }

            fn cell(&self, _row: usize, _col: usize) -> Option<CellRef<'_>> {
                Some(match self.0 {
                    Some(align) => CellRef::new("x").align(align),
                    None => CellRef::new("x"),
                })
            }
        }

        fn render(column_align: Align, cell_align: Option<Align>) -> Buffer {
            let screen = Rect::new(0, 0, 12, 3);
            let columns = [Column {
                align: column_align,
                min_width: 8,
                max_width: 8,
                ..Column::new(ColumnKey::num(1), "value")
            }];
            let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(screen);
            runtime
                .draw_scene(screen, &mut buffer, |ui, _| {
                    Grid::new(ID, &columns).draw(
                        ui,
                        screen,
                        &GridState::default(),
                        &AlignmentModel(cell_align),
                    );
                })
                .commit_presented();
            buffer
        }

        let inherited = render(Align::Right, None);
        let explicit_left = render(Align::Right, Some(Align::Left));
        let explicit_center = render(Align::Left, Some(Align::Center));
        let explicit_right = render(Align::Left, Some(Align::Right));
        assert_eq!(CellRef::new("x").align, None);
        assert_eq!(
            CellRef::new("x").align(Align::Left).align,
            Some(Align::Left)
        );
        assert_eq!(
            inherited
                .cell(Position::new(9, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_left
                .cell(Position::new(2, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_center
                .cell(Position::new(5, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
        assert_eq!(
            explicit_right
                .cell(Position::new(9, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some("x")
        );
    }

    #[test]
    fn model_cell_calls_never_exceed_max_columns() {
        struct BoundedModel {
            largest_column: Cell<usize>,
            out_of_bounds_calls: Cell<usize>,
        }

        impl BoundedModel {
            fn record(&self, col: usize) {
                self.largest_column.set(self.largest_column.get().max(col));
                if col >= GRID_MAX_COLUMNS {
                    self.out_of_bounds_calls
                        .set(self.out_of_bounds_calls.get().saturating_add(1));
                }
            }
        }

        impl GridModel for BoundedModel {
            fn row_count(&self) -> usize {
                1
            }

            fn row_key(&self, _row: usize) -> ItemKey {
                ItemKey::num(1)
            }

            fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
                self.record(col);
                Some(CellRef::new("x"))
            }

            fn cell_decor(&self, _row: usize, col: usize) -> CellDecor<'_> {
                self.record(col);
                CellDecor::default()
            }

            fn actions(&self, _row: usize, col: usize) -> &[CellAction] {
                self.record(col);
                &[]
            }
        }

        let mut columns = [Column::new(ColumnKey::num(0), "x"); GRID_MAX_COLUMNS + 2];
        for (index, column) in columns.iter_mut().enumerate() {
            column.key = ColumnKey::num(index as u16);
        }
        let model = BoundedModel {
            largest_column: Cell::new(0),
            out_of_bounds_calls: Cell::new(0),
        };
        let screen = Rect::new(0, 0, 200, 3);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime
            .draw_scene(screen, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).draw(ui, screen, &GridState::default(), &model);
            })
            .commit_presented();
        let mut state = GridState::default();
        state.set_cursor(
            0,
            ItemKey::num(1),
            GRID_MAX_COLUMNS - 1,
            ColumnKey::num(GRID_MAX_COLUMNS.saturating_sub(1) as u16),
        );
        let _ = Grid::new(ID, &columns).copy_tsv(
            &state,
            &model,
            (0, GRID_MAX_COLUMNS.saturating_sub(1)),
        );

        assert_eq!(model.largest_column.get(), GRID_MAX_COLUMNS - 1);
        assert_eq!(model.out_of_bounds_calls.get(), 0);
    }

    #[test]
    fn nonzero_origin_tiny_grid_never_places_a_column_outside_its_area() {
        let screen = Rect::new(0, 0, 12, 10);
        let area = Rect::new(5, 5, 2, 2);
        let columns = columns();
        let model = Model::two();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime
            .draw_scene(screen, &mut buffer, |ui, _| {
                Grid::new(ID, &columns).draw(ui, area, &GridState::default(), &model);
            })
            .commit_presented();

        for y in 0..screen.height {
            for x in 0..screen.width {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(
                        buffer
                            .cell(Position::new(x, y))
                            .map(ratatui_core::buffer::Cell::symbol),
                        Some(" "),
                        "grid painted outside {area:?} at ({x}, {y})"
                    );
                }
            }
        }
        let geometry = Grid::new(ID, &columns).geometry(
            Rect::new(5, 6, 2, 1),
            &GridState::default(),
            &model,
            0..1,
        );
        assert!(geometry.shown.iter().all(|shown| !shown));
    }

    #[test]
    fn oversized_first_column_is_clipped_to_positive_available_width() {
        let screen = Rect::new(0, 0, 14, 10);
        let area = Rect::new(5, 5, 4, 3);
        let body = Rect::new(5, 6, 4, 2);
        let columns = columns();
        let model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(0, ItemKey::num(10), 1, ColumnKey::num(2));
        let grid = Grid::new(ID, &columns);
        let geometry = grid.geometry(body, &state, &model, 0..2);
        assert_eq!(geometry.cell(0, body.y), Rect::new(7, 6, 2, 1));
        assert!(geometry.cell(1, body.y).is_empty());
        assert_eq!(geometry.column_at(8), Some(0));
        assert_eq!(geometry.column_at(body.right()), None);

        let app = AreaGridApp { state, model, area };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(screen);
        runtime.draw_buffer(screen, &mut buffer).commit_presented();
        runtime.draw_buffer(screen, &mut buffer).commit_presented();
        for y in 0..screen.height {
            for x in 0..screen.width {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(
                        buffer
                            .cell(Position::new(x, y))
                            .map(ratatui_core::buffer::Cell::symbol),
                        Some(" "),
                        "grid painted outside {area:?} at ({x}, {y})"
                    );
                }
            }
        }
        let cell = runtime
            .area_of_part(ID, PartRef::item(Part::CELL, ItemKey::num(10)))
            .unwrap_or(Rect::ZERO);
        assert_eq!(cell, Rect::new(7, 6, 2, 1));
        let header = runtime
            .area_of_part(
                ID,
                PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(1))),
            )
            .unwrap_or(Rect::ZERO);
        assert_eq!(header, Rect::new(7, 5, 2, 1));

        let _ = runtime.initialize();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Down, cell.right() - 1, cell.y),
        );
        assert_eq!(
            runtime.app().state.cursor(),
            Some((ItemKey::num(10), ColumnKey::num(1)))
        );
    }

    #[test]
    fn oversized_sticky_column_is_clipped_and_hides_nonsticky_columns_truthfully() {
        let area = Rect::new(0, 0, 10, 3);
        let mut columns = columns();
        if let Some(sticky) = columns.get_mut(0) {
            sticky.sticky = true;
            sticky.min_width = 9;
            sticky.max_width = 9;
        }
        if let Some(nonsticky) = columns.get_mut(1) {
            nonsticky.sortable = true;
        }
        let model = Model::two();
        let state = GridState::default();
        let grid = Grid::new(ID, &columns);
        let geometry = grid.geometry(Rect::new(0, 1, 10, 2), &state, &model, 0..2);
        assert_eq!(geometry.shown.first().copied(), Some(true));
        assert_eq!(geometry.shown.get(1).copied(), Some(false));
        assert_eq!(geometry.hidden_right, 1);
        assert_eq!(geometry.cell(0, 1), Rect::new(2, 1, 8, 1));
        assert!(geometry.cell(1, 1).is_empty());
        assert_eq!(geometry.column_at(area.right()), None);

        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                grid.draw(ui, area, &state, &model);
            })
            .commit_presented();
        let mut label = None;
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                grid.draw(ui, area, &state, &model);
                label = grid.cols_label(ui, &state, &model);
            })
            .commit_presented();
        assert_eq!(label.as_deref(), Some("cols 1–1 of 2"));
        assert!(
            runtime
                .area_of_part(
                    ID,
                    PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(2))),
                )
                .is_none()
        );
        let overflow = Theme::junie().design.glyphs.get(GlyphRole::OverflowRight);
        assert!(
            buffer
                .content()
                .iter()
                .any(|cell| cell.symbol() == overflow)
        );
    }

    #[test]
    fn variable_ragged_rows_survive_tiny_layout_measure_and_tsv() {
        struct VariableModel {
            rows: [&'static [&'static str]; 5],
            cell_calls: Cell<usize>,
        }

        impl GridModel for VariableModel {
            fn row_count(&self) -> usize {
                self.rows.len()
            }

            fn row_key(&self, row: usize) -> ItemKey {
                ItemKey::index(row)
            }

            fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
                self.cell_calls.set(self.cell_calls.get().saturating_add(1));
                self.rows
                    .get(row)
                    .and_then(|cells| cells.get(col))
                    .copied()
                    .map(CellRef::new)
            }
        }

        let mut columns = [Column::new(ColumnKey::num(0), "x"); 4];
        for (index, column) in columns.iter_mut().enumerate() {
            column.key = ColumnKey::num(index as u16);
        }
        let model = VariableModel {
            rows: [
                &[],
                &["a"],
                &["b", "c"],
                &["d", "e", "f"],
                &["g", "h", "i", "j"],
            ],
            cell_calls: Cell::new(0),
        };
        let grid = Grid::new(ID, &columns);
        let screen = Rect::new(0, 0, 12, 10);
        for width in 0..=3 {
            for height in 0..=3 {
                let area = Rect::new(5, 5, width, height);
                let mut runtime =
                    Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
                let mut buffer = Buffer::empty(screen);
                let before_measure = model.cell_calls.get();
                runtime
                    .draw_scene(screen, &mut buffer, |ui, _| {
                        let _ = grid.measure(ui, Constraints::loose(width, height));
                        assert_eq!(model.cell_calls.get(), before_measure);
                        grid.draw(ui, area, &GridState::default(), &model);
                    })
                    .commit_presented();
                for y in 0..screen.height {
                    for x in 0..screen.width {
                        if !area.contains(Position::new(x, y)) {
                            assert_eq!(
                                buffer
                                    .cell(Position::new(x, y))
                                    .map(ratatui_core::buffer::Cell::symbol),
                                Some(" "),
                                "{width}x{height} grid painted outside at ({x}, {y})"
                            );
                        }
                    }
                }
            }
        }

        let mut state = GridState::default();
        state.set_cursor(4, ItemKey::index(4), 3, ColumnKey::num(3));
        state.anchor = Some(RangeAnchor::Cell(ItemKey::index(0), ColumnKey::num(0)));
        assert_eq!(
            grid.copy_tsv(&state, &model, (4, 3)),
            "\t\t\t\na\t\t\t\nb\tc\t\t\nd\te\tf\t\ng\th\ti\tj\n"
        );
    }

    #[test]
    fn sort_is_a_permutation_and_edits_stay_bound_to_the_source_row() {
        let columns = columns();
        let grid = Grid::new(ID, &columns).select_mode(SelectMode::Multi);
        let mut model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 0, ColumnKey::num(1));
        state.core.checked_mut().insert(ItemKey::num(10));
        state.anchor = Some(RangeAnchor::Cell(ItemKey::num(10), ColumnKey::num(1)));
        state.edit = Some((ItemKey::num(20), ColumnKey::num(1)));
        let _ = state.reconcile(model.row_count(), |i| model.row_key(i));

        model.rows.reverse();
        if state
            .core
            .cursor()
            .is_some_and(|key| model.row_key(state.core.cursor_index()) != key)
        {
            state.invalidate();
        }
        let _ = state.reconcile(model.row_count(), |i| model.row_key(i));

        assert_eq!(state.cursor(), Some((ItemKey::num(20), ColumnKey::num(1))));
        assert!(state.selected_rows().contains(ItemKey::num(10)));
        assert_eq!(state.edit, Some((ItemKey::num(20), ColumnKey::num(1))));
        assert_eq!(grid.range(&state, &model, (0, 0)), Some(((0, 1), (0, 0))));
    }

    #[test]
    fn range_copy_is_tsv() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        let model = Model::two();
        let mut state = GridState::default();
        state.set_cursor(1, ItemKey::num(20), 1, ColumnKey::num(2));
        state.anchor = Some(RangeAnchor::Cell(ItemKey::num(10), ColumnKey::num(1)));
        assert_eq!(grid.copy_tsv(&state, &model, (1, 1)), "alpha\t1\nbeta\t2\n");
    }

    #[test]
    fn sortable_header_emits_keyed_requests_without_reordering_the_model() {
        let (mut rt, _) = runtime(Model::two(), false);
        let part = PartRef::item(Part::HEADER, column_item_key(ColumnKey::num(1)));
        let rect = rt.area_of_part(ID, part).unwrap_or(Rect::ZERO);
        assert!(!rect.is_empty());
        let x = rect.x.saturating_add(rect.width / 2);
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Down, x, rect.y));
        let _ = crate::runtime::stub::deliver(&mut rt, mouse(MouseKind::Up, x, rect.y));
        assert_eq!(
            rt.app().actions,
            [GridAction::Sort(ColumnKey::num(1), SortDir::Asc)]
        );
        assert_eq!(
            rt.app().model.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            [ItemKey::num(10), ItemKey::num(20)]
        );

        let (mut descending, _) = runtime(Model::two(), false);
        descending.app_mut().state.sort = Some((ColumnKey::num(1), SortDir::Asc));
        let rect = descending.area_of_part(ID, part).unwrap_or(Rect::ZERO);
        let x = rect.x.saturating_add(rect.width / 2);
        let _ = crate::runtime::stub::deliver(&mut descending, mouse(MouseKind::Down, x, rect.y));
        let _ = crate::runtime::stub::deliver(&mut descending, mouse(MouseKind::Up, x, rect.y));
        assert_eq!(
            descending.app().actions,
            [GridAction::Sort(ColumnKey::num(1), SortDir::Desc)]
        );
    }

    #[test]
    fn edit_intent_inline_cycle_external_refuse() {
        let (mut inline, _) = runtime(Model::two(), true);
        let _ = crate::runtime::stub::deliver(&mut inline, key(KeyCode::Enter));
        assert!(inline.app().state.is_editing());

        let mut cycle_model = Model::two();
        cycle_model.mode = Mode::Cycle;
        let (mut cycle, _) = runtime(cycle_model, true);
        let _ = crate::runtime::stub::deliver(&mut cycle, key(KeyCode::Enter));
        assert_eq!(cycle.app().model.cycles, 1);

        let mut external_model = Model::two();
        external_model.mode = Mode::External;
        let (mut external, _) = runtime(external_model, true);
        let _ = crate::runtime::stub::deliver(&mut external, key(KeyCode::Enter));
        assert_eq!(
            external.app().actions,
            [GridAction::EditRequested(
                ItemKey::num(10),
                ColumnKey::num(1)
            )]
        );

        let mut refuse_model = Model::two();
        refuse_model.mode = Mode::Refuse;
        let (mut refuse, mut refuse_buffer) = runtime(refuse_model, true);
        let _ = crate::runtime::stub::deliver(&mut refuse, key(KeyCode::Enter));
        refuse
            .draw_buffer(AREA, &mut refuse_buffer)
            .commit_presented();
        assert_eq!(
            refuse.app().state.edit_error().map(ToString::to_string),
            Some("locked".to_owned())
        );
        assert!(!refuse.app().state.is_editing());
        let refused_text = refuse_buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect::<String>();
        assert!(refused_text.contains("locked"));
    }

    #[test]
    fn update_editable_commits_through_the_editor() {
        let (mut runtime, _) = runtime(Model::two(), true);
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        runtime
            .draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('x')));
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert_eq!(runtime.app().model.commits.len(), 1);
        assert!(!runtime.app().state.is_editing());

        runtime.app_mut().model.fail_commit = true;
        runtime
            .draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        runtime
            .draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert!(runtime.app().state.is_editing());
        assert_eq!(
            runtime.app().state.edit_error().and_then(|e| e.code),
            Some("grid-test")
        );

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Esc));
        assert!(!runtime.app().state.is_editing());
        assert!(runtime.app().state.edit_error().is_none());
    }

    #[test]
    fn rejected_inline_edit_uses_the_grid_error_underline() {
        let levels = [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Ansi256,
            crate::theme::ColorLevel::Ansi16,
            crate::theme::ColorLevel::Mono,
        ];
        for level in levels {
            let theme = Theme::junie().for_level(level);
            let (valid, error, recovered) = inline_edit_underline_samples(theme.clone());
            let valid_expected = theme
                .resolve(
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::TEXT,
                    StateFlags::FOCUSED | StateFlags::EDITING,
                    crate::theme::Surface::Canvas,
                )
                .style
                .as_style()
                .underline_color;
            let error_expected = theme
                .resolve(
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::TEXT,
                    StateFlags::FOCUSED | StateFlags::EDITING | StateFlags::ERROR,
                    crate::theme::Surface::Canvas,
                )
                .style
                .as_style()
                .underline_color;
            assert_eq!(Some(valid), valid_expected, "valid draft at {level:?}");
            assert_eq!(Some(error), error_expected, "rejected draft at {level:?}");
            assert_eq!(
                Some(recovered),
                valid_expected,
                "corrected draft at {level:?}"
            );
        }
    }

    #[test]
    fn inline_editor_error_underline_respects_theme_override() {
        let theme = Theme::junie().define_family(Family::GRID, |recipe| {
            recipe.part(Part::TEXT).when(
                StateFlags::ERROR,
                StylePatch::new().set_underline(Role::Warning),
            );
        });
        let (_, error, _) = inline_edit_underline_samples(theme.clone());
        let expected = theme
            .resolve(
                Family::GRID,
                Variant::DEFAULT,
                Part::TEXT,
                StateFlags::FOCUSED | StateFlags::EDITING | StateFlags::ERROR,
                crate::theme::Surface::Canvas,
            )
            .style
            .as_style()
            .underline_color;
        assert_eq!(Some(error), expected);
    }

    #[test]
    fn inline_editor_theme_clear_resets_the_input_underline() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Ansi256,
            crate::theme::ColorLevel::Ansi16,
            crate::theme::ColorLevel::Mono,
        ] {
            let theme = Theme::junie()
                .for_level(level)
                .define_family(Family::INPUT, |recipe| {
                    recipe
                        .part(Part::TEXT)
                        .base(StylePatch::new().set_underline(Role::Warning));
                })
                .define_family(Family::GRID, |recipe| {
                    let mut clear = StylePatch::new();
                    clear.underline = crate::theme::Slot::Clear;
                    recipe.part(Part::TEXT).when(StateFlags::ERROR, clear);
                });
            let (_, error, _) = inline_edit_underline_samples(theme);
            assert_eq!(error, ratatui_core::style::Color::Reset, "at {level:?}");
        }
    }

    #[test]
    fn inline_editor_theme_clear_resets_prepaint_without_input_underline() {
        for level in [
            crate::theme::ColorLevel::TrueColor,
            crate::theme::ColorLevel::Ansi256,
            crate::theme::ColorLevel::Ansi16,
            crate::theme::ColorLevel::Mono,
        ] {
            let theme = Theme::junie()
                .for_level(level)
                .define_family(Family::GRID, |recipe| {
                    recipe.part(Part::CELL).base(
                        StylePatch::new()
                            .set_underline(Role::Custom(ratatui_core::style::Color::Yellow))
                            .add(Modifier::UNDERLINED),
                    );
                    let mut clear = StylePatch::new();
                    clear.underline = crate::theme::Slot::Clear;
                    recipe.part(Part::TEXT).base(clear);
                });
            let (mut runtime, mut buffer) = runtime(Model::two(), true);
            runtime.set_theme(theme);
            let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
            runtime.draw_buffer(AREA, &mut buffer).commit_presented();
            let underlined = buffer
                .content()
                .iter()
                .filter(|cell| cell.modifier.contains(Modifier::UNDERLINED))
                .collect::<Vec<_>>();
            assert!(
                !underlined.is_empty(),
                "the host cell has underline styling at {level:?}"
            );
            let rows = buffer.content().chunks(AREA.width as usize);
            let alpha = rows
                .clone()
                .flat_map(|row| row.windows(5))
                .find(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == "alpha")
                .expect("edited value is visible");
            assert!(
                alpha
                    .iter()
                    .all(|cell| cell.underline_color == ratatui_core::style::Color::Reset),
                "host Clear must reset the edited cell's prepainted underline without an input default at {level:?}"
            );
            assert!(
                rows.clone().flat_map(|row| row.windows(4)).any(|cells| {
                    cells.iter().map(|cell| cell.symbol()).collect::<String>() == "beta"
                        && cells
                            .iter()
                            .all(|cell| cell.modifier.contains(Modifier::UNDERLINED))
                }),
                "the non-edited cell retains its own underline at {level:?}"
            );
        }
    }

    fn inline_edit_underline_samples(
        theme: Theme,
    ) -> (
        ratatui_core::style::Color,
        ratatui_core::style::Color,
        ratatui_core::style::Color,
    ) {
        let mut model = Model::two();
        model.fail_commit = true;
        let (mut runtime, _) = runtime(model, true);
        runtime.set_theme(theme);
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        let mut valid_buffer = Buffer::empty(AREA);
        runtime
            .draw_buffer(AREA, &mut valid_buffer)
            .commit_presented();
        let valid = inline_underline(&valid_buffer);

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('x')));
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        assert!(runtime.app().state.edit_error().is_some());
        let mut error_buffer = Buffer::empty(AREA);
        runtime
            .draw_buffer(AREA, &mut error_buffer)
            .commit_presented();
        let error = inline_underline(&error_buffer);

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('y')));
        assert!(runtime.app().state.edit_error().is_none());
        let mut recovered_buffer = Buffer::empty(AREA);
        runtime
            .draw_buffer(AREA, &mut recovered_buffer)
            .commit_presented();
        let recovered = inline_underline(&recovered_buffer);
        (valid, error, recovered)
    }

    fn inline_underline(buffer: &Buffer) -> ratatui_core::style::Color {
        buffer
            .content()
            .iter()
            .find(|cell| cell.modifier.contains(Modifier::UNDERLINED))
            .expect("inline draft remains visibly underlined")
            .underline_color
    }

    #[test]
    fn click_inside_an_active_inline_edit_goes_to_the_editor() {
        let (mut runtime, _) = runtime(Model::two(), true);
        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Enter));
        runtime
            .draw_buffer(AREA, &mut Buffer::empty(AREA))
            .commit_presented();
        let editor = runtime.area_of(ID.part(Part::TEXT)).unwrap_or(Rect::ZERO);
        assert!(!editor.is_empty());
        let before = runtime.app().actions.len();
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Down, editor.x, editor.y));
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Up, editor.x, editor.y));
        assert_eq!(runtime.app().actions.len(), before);
        assert!(runtime.app().state.is_editing());
    }

    #[test]
    fn read_only_reason_is_rendered_from_a_grid_model() {
        let mut model = Model::two();
        model.reason = Some("read only by source");
        let (_, buffer) = runtime(model, false);
        let text: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("read only by source"));
    }

    #[test]
    fn cell_actions_affordance_is_painted_for_a_read_only_model() {
        let mut model = Model::two();
        model.cell_action = true;
        let (mut runtime, buffer) = runtime(model, false);
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::FollowRef);
        assert!(buffer.content().iter().any(|cell| cell.symbol() == glyph));
        let affordance = runtime
            .area_of_part(ID, PartRef::item(Part::ACTIONS, ItemKey::num(10)))
            .unwrap_or(Rect::ZERO);
        assert!(!affordance.is_empty());
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Down, affordance.x, affordance.y),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            mouse(MouseKind::Up, affordance.x, affordance.y),
        );
        assert_eq!(
            runtime.app().actions,
            [GridAction::CellAction(
                ItemKey::num(10),
                ColumnKey::num(1),
                CELL_ACTION
            )]
        );
    }

    #[test]
    fn nested_scroll_overrides_and_actions_slot_change_the_declared_parts() {
        let replace = |ui: &mut Ui<'_>, area: Rect| {
            ui.paint_str(area, "#", ui.surface_style());
        };
        let patch = StylePatch::new().add(Modifier::BOLD);
        let parts = [(Part::TRACK, patch), (Part::THUMB, patch)];
        let mut model = Model::two();
        model.cell_action = true;
        model
            .rows
            .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
        let columns = columns();
        let grid = Grid::new(ID, &columns)
            .actions_slot(&replace)
            .patch_part(&parts)
            .slot(Part::ACTIONS, &replace);
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, _| {
                grid.draw(ui, AREA, &GridState::default(), &model);
            })
            .commit_presented();

        let hashes = buffer
            .content()
            .iter()
            .filter(|cell| cell.symbol() == "#")
            .count();
        assert!(hashes >= 2, "ACTIONS slot must reach cell and action row");
        for part in [Part::TRACK, Part::THUMB] {
            let area = runtime
                .area_of_part(ID, PartRef::of(part))
                .unwrap_or(Rect::ZERO);
            assert!(!area.is_empty(), "{part:?} was not registered");
            assert!(
                buffer
                    .cell(Position::new(area.x, area.y))
                    .is_some_and(|cell| cell.modifier.contains(Modifier::BOLD)),
                "{part:?} dropped its forwarded patch"
            );
        }
    }

    #[test]
    fn reference_grid_makes_its_nested_scroll_region_inert() {
        let columns = columns();
        let grid = Grid::new(ID, &columns);
        let mut model = Model::two();
        model
            .rows
            .extend((0..20).map(|i| (ItemKey::num(100 + i), ["overflow", "row"])));
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, _| {
                ui.reference(
                    Some(crate::ReferenceTarget::new(
                        ID,
                        crate::ReferenceState::FOCUSED,
                    )),
                    |ui| grid.draw(ui, AREA, &GridState::default(), &model),
                );
            })
            .commit_presented();
        assert!(runtime.area_of(ID).is_none());
        assert!(runtime.area_of_part(ID, PartRef::of(Part::TRACK)).is_none());
    }

    #[test]
    fn reference_focus_and_press_cannot_fabricate_row_selection() {
        let columns = columns();
        let grid = Grid::new(ID, &columns).select_mode(SelectMode::Multi);
        let model = Model::two();
        let mut runtime = Runtime::new(crate::runtime::stub::Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let target = crate::ReferenceTarget::new(
            ID,
            crate::ReferenceState::FOCUSED | crate::ReferenceState::PRESSED,
        )
        .part(PartRef::item(Part::ROW, ItemKey::num(10)));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, _| {
                ui.reference(Some(target), |ui| {
                    grid.draw(ui, AREA, &GridState::default(), &model);
                });
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(1, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(" ")
        );

        let mut selected = GridState::default();
        selected.core.checked_mut().insert(ItemKey::num(10));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, _| {
                ui.reference(Some(target), |ui| grid.draw(ui, AREA, &selected, &model));
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(1, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Checked))
        );
    }

    #[test]
    fn column_reveal_updates_the_exact_horizontal_offset_reader() {
        let mut many = [Column::new(ColumnKey::num(0), "0"); 6];
        for (i, column) in many.iter_mut().enumerate() {
            column.key = ColumnKey::num(i as u16);
            column.title = "column";
        }
        let grid = Grid::new(ID, &many);
        let mut state = GridState::default();
        grid.reveal_column(&mut state, 5);
        assert_eq!(state.col_offset(), 5);
    }

    #[test]
    fn sortable_headers_use_typed_theme_glyphs() {
        let (_, buffer) = runtime(Model::two(), false);
        let glyph = Theme::junie().design.glyphs.get(GlyphRole::SortAsc);
        assert!(buffer.content().iter().any(|cell| cell.symbol() == glyph));
        assert!(buffer.cell(Position::new(0, 0)).is_some());
    }
}
