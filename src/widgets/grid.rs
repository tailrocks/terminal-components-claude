//! Data grid: dense tabular data with a cursor cell, row/range selection,
//! typed cell rendering, and a pending-change queue with dirty / inserted /
//! deleted / error states. Sorting and filtering are *requests* to the
//! owner (server-side), never local reordering, except when the owner
//! opts into local sort for fully loaded data.
//!
//! Row anatomy: `▎` focus · `✓` selection · change slot (`•` `+` `−` `!`) ·
//! row number · cells. Current cell is reversed; editing shows the cursor.

use std::collections::{BTreeSet, HashMap};

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::focus::BlurAction;
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::scroll::{HScroll, ScrollState, ScrollbarVisibility};
use crate::core::text::TextBuffer;
use crate::theme::Theme;
use crate::ui::columns::{ColumnSizing, HeaderAction, HeaderLabels};
use crate::ui::ctx::{RenderCtx, VisualState, fill};
use crate::ui::text::{fit, fit_right, truncate, truncate_middle, wider_than, width, width_capped};
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::option_editor::{
    Affordance, ChoiceEditor, ChoiceResult, ColumnEditor, EditorConfig,
};
use crate::widgets::scrollbar;
use crate::widgets::table::SortDir;

// ------------------------------------------------------------------ values

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    /// Server default (inserted rows).
    Default,
    Text(String),
    Int(i64),
    Num(f64),
    Bool(bool),
    Json(String),
}

impl CellValue {
    pub fn text(&self) -> String {
        self.text_ref().into_owned()
    }
    /// [`Self::text`] without copying text and JSON values: rendering
    /// compares and measures values on every frame and never needs a copy.
    pub fn text_ref(&self) -> std::borrow::Cow<'_, str> {
        match self {
            CellValue::Null => "NULL".into(),
            CellValue::Default => "DEFAULT".into(),
            CellValue::Text(s) | CellValue::Json(s) => s.as_str().into(),
            CellValue::Int(i) => i.to_string().into(),
            CellValue::Num(n) => format!("{n:.2}").into(),
            CellValue::Bool(b) => b.to_string().into(),
        }
    }
    /// Text to put in the editor (empty for NULL/DEFAULT).
    pub fn edit_text(&self) -> String {
        match self {
            CellValue::Null | CellValue::Default => String::new(),
            v => v.text(),
        }
    }
}

/// Rendering behaviour; the owner maps its own types onto these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Text,
    Id,
    Number,
    Bool,
    Timestamp,
    Json,
    Enum,
}

impl CellKind {
    fn default_width(self) -> (u16, u16) {
        match self {
            CellKind::Id => (9, 36),
            CellKind::Text => (6, 40),
            CellKind::Number => (4, 22),
            CellKind::Bool => (5, 5),
            CellKind::Timestamp => (10, 29),
            CellKind::Json => (8, 40),
            CellKind::Enum => (6, 16),
        }
    }
    fn right_aligned(self) -> bool {
        self == CellKind::Number
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSpec {
    pub name: String,
    pub kind: CellKind,
    pub primary: bool,
    pub nullable: bool,
    pub read_only: bool,
    pub references: Option<String>,
    pub enum_values: Vec<String>,
    pub sortable: bool,
    pub min_width: u16,
    pub max_width: u16,
    /// Muted type label shown under the name when `type_row` is on.
    pub type_label: String,
    /// How the column's cells are edited: typed text (default) or a value
    /// from a fixed set ([`ColumnEditor`]).
    pub editor: ColumnEditor,
}

impl ColumnSpec {
    pub fn new(name: &str, kind: CellKind) -> Self {
        let (min_width, max_width) = kind.default_width();
        Self {
            name: name.to_owned(),
            kind,
            primary: false,
            nullable: true,
            read_only: false,
            references: None,
            enum_values: vec![],
            sortable: kind != CellKind::Json,
            min_width,
            max_width,
            type_label: String::new(),
            editor: ColumnEditor::Text,
        }
    }
    /// Edit the column with a constrained `editor`.
    pub fn editor(mut self, editor: ColumnEditor) -> Self {
        self.editor = editor;
        self
    }
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }
    pub fn nullable(mut self, n: bool) -> Self {
        self.nullable = n;
        self
    }
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }
    pub fn references(mut self, t: &str) -> Self {
        self.references = Some(t.to_owned());
        self
    }
    pub fn type_label(mut self, l: &str) -> Self {
        self.type_label = l.to_owned();
        self
    }
    pub fn enum_values(mut self, v: &[&str]) -> Self {
        self.enum_values = v.iter().map(|s| (*s).to_owned()).collect();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowTotal {
    Exact(usize),
    Estimated(usize),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridRows {
    pub rows: Vec<Vec<CellValue>>,
    pub total: RowTotal,
    /// More rows can be fetched (the result was capped at this many).
    pub more: bool,
}

// ------------------------------------------------------------- pending edits

#[derive(Debug, Clone, PartialEq)]
pub enum UndoAction {
    Cell {
        row: usize,
        col: usize,
        before: Option<CellValue>,
    },
    Delete {
        row: usize,
        was_deleted: bool,
    },
    Insert {
        row: usize,
    },
}

/// Nothing reaches the server until the owner commits. Row keys are source
/// row indices, so sorting (a permutation) never invalidates them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PendingChanges {
    pub cells: HashMap<(usize, usize), CellValue>,
    pub inserted: BTreeSet<usize>,
    pub deleted: BTreeSet<usize>,
}

impl PendingChanges {
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty() && self.inserted.is_empty() && self.deleted.is_empty()
    }
    pub fn dirty_rows(&self) -> BTreeSet<usize> {
        self.cells
            .keys()
            .map(|(r, _)| *r)
            .filter(|r| !self.inserted.contains(r))
            .collect()
    }
    /// (updates, inserts, deletes)
    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.dirty_rows().len(),
            self.inserted.len(),
            self.deleted.len(),
        )
    }
    pub fn total(&self) -> usize {
        let (u, i, d) = self.counts();
        u + i + d
    }
    pub fn is_dirty(&self, row: usize, col: usize) -> bool {
        self.cells.contains_key(&(row, col))
    }
    pub fn value(&self, row: usize, col: usize) -> Option<&CellValue> {
        self.cells.get(&(row, col))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    Clean,
    Modified,
    Inserted,
    Deleted,
    Error,
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub row: usize,
    pub col: usize,
    pub buffer: TextBuffer,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridEvent {
    CellChanged {
        row: usize,
        col: usize,
    },
    RowInserted(usize),
    RowDeleted(usize),
    SortRequested(Option<(usize, SortDir)>),
    FetchMore,
    Refresh,
    CommitRequested,
    DiscardRequested,
    PreviewSql,
    Copy(String),
    FollowReference {
        row: usize,
        col: usize,
    },
    OpenViewer {
        row: usize,
        col: usize,
    },
    FilterOnCell {
        col: usize,
        value: CellValue,
    },
    OpenFilters,
    ClearFilters,
    Activated(usize),
    LeaveForward,
    LeaveBackward,
    /// Enter (or F2) on a cell that cannot be edited: a read-only column,
    /// a row marked for deletion, or a grid that is not editable and says
    /// why ([`DataGrid::read_only_reason`]). The owner shows `reason`
    /// (a status line), so the key is never silently dropped.
    ReadOnly {
        reason: String,
    },
}

/// Validate text typed into a cell for a column. The owner can replace
/// this with engine-aware parsing ([`DataGrid::validator`]); a closure, so
/// it can capture state (a schema, an enum list).
#[derive(Clone)]
pub struct Validator(pub std::rc::Rc<ValidateFn>);

/// The function a [`Validator`] wraps.
pub type ValidateFn = dyn Fn(&ColumnSpec, &str) -> Result<CellValue, String>;

impl Validator {
    pub fn new(f: impl Fn(&ColumnSpec, &str) -> Result<CellValue, String> + 'static) -> Self {
        Self(std::rc::Rc::new(f))
    }
}

impl Default for Validator {
    /// [`default_validator`].
    fn default() -> Self {
        Self::new(default_validator)
    }
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validator")
    }
}

pub fn default_validator(col: &ColumnSpec, text: &str) -> Result<CellValue, String> {
    let t = text.trim();
    if t.eq_ignore_ascii_case("null") {
        return if col.nullable {
            Ok(CellValue::Null)
        } else {
            Err(format!("{} is NOT NULL", col.name))
        };
    }
    match col.kind {
        CellKind::Number => {
            if let Ok(i) = t.parse::<i64>() {
                Ok(CellValue::Int(i))
            } else {
                t.parse::<f64>()
                    .map(CellValue::Num)
                    .map_err(|_| "Must be a number".to_owned())
            }
        }
        CellKind::Bool => match t.to_ascii_lowercase().as_str() {
            "true" | "t" | "1" | "yes" => Ok(CellValue::Bool(true)),
            "false" | "f" | "0" | "no" => Ok(CellValue::Bool(false)),
            _ => Err("Must be true or false".into()),
        },
        CellKind::Json => {
            if (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
            {
                Ok(CellValue::Json(t.into()))
            } else {
                Err("Must be a JSON object or array".into())
            }
        }
        CellKind::Enum if !col.enum_values.is_empty() => {
            if col.enum_values.iter().any(|v| v == t) {
                Ok(CellValue::Text(t.into()))
            } else {
                Err(format!("Must be one of: {}", col.enum_values.join(", ")))
            }
        }
        CellKind::Id => {
            if t.len() == 36 && t.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                Ok(CellValue::Text(t.into()))
            } else {
                Err("Must be a UUID".into())
            }
        }
        CellKind::Timestamp => {
            if t.len() >= 10 && t.as_bytes()[4] == b'-' && t.as_bytes()[7] == b'-' {
                Ok(CellValue::Text(t.into()))
            } else {
                Err("Use YYYY-MM-DD".into())
            }
        }
        _ => Ok(CellValue::Text(t.into())),
    }
}

// ------------------------------------------------------------------ grid

/// A pointer drag on a [`DataGrid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridDrag {
    /// A range selection from the pressed cell.
    Cells(WidgetId),
    /// Mark painting from the pressed row number at display row `from`.
    Rows {
        id: WidgetId,
        from: usize,
        mark: bool,
    },
}

#[derive(Debug, Clone)]
pub struct DataGrid {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    pub id: WidgetId,
    pub columns: Vec<ColumnSpec>,
    rows: Vec<Vec<CellValue>>,
    order: Vec<usize>,
    pub total: RowTotal,
    pub more: bool,
    pub sort: Option<(usize, SortDir)>,
    /// Sort locally instead of asking the owner (only for fully loaded data).
    pub local_sort: bool,
    pub filtered_cols: BTreeSet<usize>,
    pub cursor: (usize, usize),
    /// Range selection anchor (display row, col).
    anchor: Option<(usize, usize)>,
    pub selected_rows: BTreeSet<usize>,
    pub pending: PendingChanges,
    undo: Vec<UndoAction>,
    pub scroll: ScrollState,
    pub hscroll: ScrollState,
    pub edit: Option<EditState>,
    pub editable: bool,
    /// What a cell edit in progress does when the grid loses focus
    /// ([`Self::on_blur`]). Default [`BlurAction::Commit`].
    pub blur_action: BlurAction,
    pub read_only_reason: Option<String>,
    pub loading: bool,
    pub cell_errors: HashMap<(usize, usize), String>,
    pub row_errors: HashMap<usize, String>,
    pub empty: crate::widgets::empty::EmptyState,
    pub validator: Validator,
    pub row_numbers: bool,
    pub area: Rect,
    body: Rect,
    /// The pointer drag in progress, kept by the pressed id so it goes on
    /// after edge autoscroll moved its origin out of view.
    drag: Option<GridDrag>,
    widths: Vec<u16>,
    /// User column widths from dragging a header seam or autofit
    /// ([`ColumnSizing`]); `sizing.resizable` turns the seams off.
    pub sizing: ColumnSizing,
    /// Labels of the header context menu ([`Self::header_menu`]).
    pub header_labels: HeaderLabels,
    col_rects: Vec<Rect>,
    show_bar: bool,
    bar: [Button; 3],
    /// The theme's strings as of the last render: the empty state and bar
    /// labels still reading them follow a change
    /// ([`crate::theme::Strings::follow`]).
    strings: crate::theme::Strings,
    /// How a cut cell reveals its full value on hover (see
    /// [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// Horizontal scrolling policy; `None` follows [`RenderCtx::hscroll`].
    /// The grid always moves by whole columns and its bar counts columns.
    pub horizontal: Option<HScroll>,
    /// The `‹n` / `n›` header badges counting columns hidden on either
    /// side (clickable, one column per click). They complement the bar:
    /// the bar shows where, the badges how many.
    pub overflow_badges: bool,
    /// The policy resolved at the last render (events use it).
    h_cfg: HScroll,
    /// The horizontal scrollbar track as last drawn.
    htrack: Rect,
    /// Which keys move the cursor (default [`NavKeys::DEFAULT`]). `g`/`G`
    /// go to the first/last row, Home/End to the row's first/last cell,
    /// Ctrl+Home/End to the first/last row; Ctrl+←/→ page sideways.
    pub nav: NavKeys,
    /// The grid's action keys (default [`GridAction::default_keymap`]).
    pub keymap: KeyMap<GridAction>,
    /// Typing a prefix jumps to the first row whose cell in
    /// [`Self::type_ahead_column`] starts with it (macOS table type-select).
    /// Off by default: plain letters keep their vim and action bindings.
    pub type_ahead: bool,
    /// The column type-ahead matches; `None` (default) is the cursor's
    /// column.
    pub type_ahead_column: Option<usize>,
    typeahead: crate::core::typeahead::TypeAhead,
    /// How many leading rows [`Self::set_rows`] measures to size columns
    /// (the 95th percentile of their widths, clamped to each column's
    /// min/max). Default [`DEFAULT_WIDTH_SAMPLE_ROWS`]; widths are measured
    /// once per dataset, never per frame.
    pub width_sample_rows: usize,
    /// What a press on the bare vertical track does; `None` follows
    /// [`RenderCtx::track_click`](crate::ui::ctx::RenderCtx::track_click).
    pub track_click: Option<crate::core::scroll::TrackClick>,
    /// The vertical track policy resolved at the last render.
    v_click: crate::core::scroll::TrackClick,
    /// The constrained editor open on a cell, if any.
    pub choice: Option<GridChoice>,
    /// Behaviour of every constrained column editor
    /// ([`EditorConfig::default`]).
    pub editor_config: EditorConfig,
}

/// A constrained editor open on one grid cell ([`DataGrid::choice`]).
#[derive(Debug, Clone)]
pub struct GridChoice {
    /// Source row.
    pub row: usize,
    pub col: usize,
    pub editor: ChoiceEditor,
}

/// A [`DataGrid`] action a key can be bound to. Enter/F2 (edit or
/// activate), Space (mark the row) and Esc (clear the selection) are fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAction {
    /// Set the cell to NULL, or mark the selected rows deleted. Default Del
    /// and Backspace.
    ClearCell,
    /// Mark the cursor row deleted (again: restore it). Default `-`.
    ToggleDelete,
    /// Insert a row. Default `+`.
    InsertRow,
    /// Duplicate the cursor row. Default Ctrl+D and Alt+D.
    DuplicateRow,
    /// Copy the cell, range or marked rows. Default Cmd+C where the
    /// terminal delivers Cmd, and `y` (a hint names the first deliverable).
    Copy,
    /// Copy with the column names. Default `Y`.
    CopyWithHeader,
    /// Ask the app to save the pending changes. Default Ctrl+S.
    Commit,
    /// Sort by the cursor column (asc, desc, off). Default `s`.
    Sort,
    /// Drop the sort. Default `S`.
    ClearSort,
    /// Filter on the cursor cell's value (an empty grid opens the
    /// filters). Default `f`.
    FilterOnCell,
    /// Open the filter editor. Default `/`.
    OpenFilters,
    /// Clear every filter. Default `F`.
    ClearFilters,
    /// Reload. Default `r` and F5.
    Refresh,
    /// Undo the last pending change. Default `u`.
    Undo,
    /// Ask the app to discard the pending changes. Default `U`.
    Discard,
    /// Ask the app to preview the SQL. Default `p`.
    PreviewSql,
    /// Follow the cell's foreign key. Default Ctrl+].
    FollowReference,
    /// Mark every row (Select All), as Space marks one. Default Cmd+A
    /// where the terminal delivers Cmd, and Ctrl+A
    /// ([`crate::core::keymap::SELECT_ALL`]).
    SelectAll,
}

impl Actions for GridAction {
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::key(KeyCode::Delete), GridAction::ClearCell),
            (KeyBinding::key(KeyCode::Backspace), GridAction::ClearCell),
            (KeyBinding::char('-'), GridAction::ToggleDelete),
            (KeyBinding::char('+'), GridAction::InsertRow),
            (KeyBinding::ctrl('d'), GridAction::DuplicateRow),
            (KeyBinding::alt('d'), GridAction::DuplicateRow),
            (KeyBinding::cmd('c'), GridAction::Copy),
            (KeyBinding::char('y'), GridAction::Copy),
            (KeyBinding::char('Y'), GridAction::CopyWithHeader),
            (KeyBinding::ctrl('s'), GridAction::Commit),
            (KeyBinding::char('s'), GridAction::Sort),
            (KeyBinding::char('S'), GridAction::ClearSort),
            (KeyBinding::char('f'), GridAction::FilterOnCell),
            (KeyBinding::char('/'), GridAction::OpenFilters),
            (KeyBinding::char('F'), GridAction::ClearFilters),
            (KeyBinding::char('r'), GridAction::Refresh),
            (KeyBinding::key(KeyCode::F(5)), GridAction::Refresh),
            (KeyBinding::char('u'), GridAction::Undo),
            (KeyBinding::char('U'), GridAction::Discard),
            (KeyBinding::char('p'), GridAction::PreviewSql),
            (KeyBinding::ctrl(']'), GridAction::FollowReference),
            (crate::core::keymap::SELECT_ALL[0], GridAction::SelectAll),
            (crate::core::keymap::SELECT_ALL[1], GridAction::SelectAll),
        ]
    }
}

/// Rows sampled for column widths by default (see
/// [`DataGrid::width_sample_rows`]).
pub const DEFAULT_WIDTH_SAMPLE_ROWS: usize = 200;

impl DataGrid {
    pub fn new(id: WidgetId, columns: Vec<ColumnSpec>) -> Self {
        let widths = columns.iter().map(|c| c.min_width).collect();
        Self {
            track_click: None,
            v_click: crate::core::scroll::TrackClick::Page,
            id,
            columns,
            rows: vec![],
            order: vec![],
            total: RowTotal::Unknown,
            more: false,
            sort: None,
            local_sort: false,
            filtered_cols: BTreeSet::new(),
            cursor: (0, 0),
            anchor: None,
            selected_rows: BTreeSet::new(),
            pending: PendingChanges::default(),
            undo: vec![],
            scroll: ScrollState::default(),
            hscroll: ScrollState::default(),
            edit: None,
            editable: true,
            blur_action: BlurAction::Commit,
            read_only_reason: None,
            loading: false,
            cell_errors: HashMap::new(),
            row_errors: HashMap::new(),
            empty: crate::widgets::empty::EmptyState::new(crate::theme::Strings::DEFAULT.no_rows),
            validator: Validator::default(),
            row_numbers: true,
            area: Rect::ZERO,
            track: Rect::ZERO,
            body: Rect::ZERO,
            drag: None,
            widths,
            sizing: ColumnSizing::default(),
            header_labels: HeaderLabels::DEFAULT,
            col_rects: vec![],
            show_bar: true,
            bar: [
                Button::subtle(
                    id.sub("preview"),
                    crate::theme::Strings::DEFAULT.preview_sql,
                ),
                Button::subtle(id.sub("discard"), crate::theme::Strings::DEFAULT.discard),
                Button::primary(id.sub("save"), crate::theme::Strings::DEFAULT.save_changes),
            ],
            strings: crate::theme::Strings::DEFAULT,
            reveal: None,
            horizontal: None,
            overflow_badges: true,
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            width_sample_rows: DEFAULT_WIDTH_SAMPLE_ROWS,
            nav: NavKeys::DEFAULT,
            keymap: GridAction::default_keymap(),
            type_ahead: false,
            type_ahead_column: None,
            typeahead: Default::default(),
            choice: None,
            editor_config: EditorConfig::default(),
        }
    }

    /// See [`Self::editor_config`].
    pub fn editor_config(mut self, cfg: EditorConfig) -> Self {
        self.editor_config = cfg;
        self
    }

    /// The id of the popup a constrained editor opens.
    pub fn choice_id(&self) -> WidgetId {
        self.id.sub("choice")
    }

    /// The `▾` of a constrained cell (on the cursor cell and on hover).
    pub fn open_id(&self, display: usize, col: usize) -> WidgetId {
        self.cell_id(display, col).sub("open")
    }

    fn locate_open(&self, id: WidgetId) -> Option<(usize, usize)> {
        for d in self.scroll.visible_range() {
            for c in 0..self.columns.len() {
                if self.open_id(d, c) == id {
                    return Some((d, c));
                }
            }
        }
        None
    }

    /// The cell at (source row, column) holds text outside its column's
    /// options (imported data).
    pub fn out_of_set(&self, src: usize, col: usize) -> bool {
        let Some(spec) = self.columns.get(col) else {
            return false;
        };
        match self.value(src, col) {
            CellValue::Text(t) => spec.editor.out_of_set(t, &self.editor_config),
            _ => false,
        }
    }

    /// Store a chosen value through the grid's validator and pending
    /// queue.
    fn apply_choice(&mut self, src: usize, col: usize, value: String) -> Result<GridEvent, String> {
        let parsed = if value.is_empty() {
            Ok(CellValue::Text(String::new()))
        } else {
            (self.validator.0)(&self.columns[col], &value)
        }?;
        self.record_cell(src, col, parsed);
        Ok(GridEvent::CellChanged { row: src, col })
    }

    /// The next (previous) writable column of the cursor row, or the
    /// event that leaves the grid.
    fn next_writable(&self, backward: bool) -> Result<usize, GridEvent> {
        let n = self.columns.len();
        let mut col = self.cursor.1;
        loop {
            if backward {
                if col == 0 {
                    return Err(GridEvent::LeaveBackward);
                }
                col -= 1;
            } else {
                col += 1;
                if col >= n {
                    return Err(GridEvent::LeaveForward);
                }
            }
            if !self.columns[col].read_only && !self.columns[col].primary {
                return Ok(col);
            }
        }
    }

    /// Resolve what an open constrained editor reported.
    fn on_choice(&mut self, r: Option<ChoiceResult>) -> Option<GridEvent> {
        let (value, tab) = match r? {
            ChoiceResult::Cancel => {
                self.choice = None;
                return None;
            }
            ChoiceResult::Commit(v) => (v, None),
            ChoiceResult::CommitAndTab { value, backward } => (value, Some(backward)),
        };
        let (src, col) = self.choice.as_ref().map(|c| (c.row, c.col))?;
        match self.apply_choice(src, col, value) {
            Ok(ev) => {
                self.choice = None;
                if let Some(backward) = tab {
                    match self.next_writable(backward) {
                        Ok(c) => {
                            self.set_cursor(self.cursor.0, c, false);
                            let _ = self.begin_edit();
                        }
                        Err(leave) => return Some(leave),
                    }
                }
                Some(ev)
            }
            Err(msg) => {
                if let Some(c) = self.choice.as_mut() {
                    c.editor.error = Some(msg);
                }
                None
            }
        }
    }

    /// See [`Self::type_ahead`].
    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    /// See [`Self::type_ahead_column`].
    pub fn type_ahead_column(mut self, col: Option<usize>) -> Self {
        self.type_ahead_column = col;
        self
    }

    /// Override the app-wide horizontal scrolling policy.
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    /// Move the default text still reading the last strings to `s`.
    fn follow_strings(&mut self, s: crate::theme::Strings) {
        use crate::theme::Strings;
        let old = std::mem::replace(&mut self.strings, s);
        if old == s {
            return;
        }
        Strings::follow(&mut self.empty.title, old.no_rows, s.no_rows);
        let pairs = [
            (old.preview_sql, s.preview_sql),
            (old.discard, s.discard),
            (old.save_changes, s.save_changes),
        ];
        for (b, (from, to)) in self.bar.iter_mut().zip(pairs) {
            Strings::follow(&mut b.label, from, to);
        }
    }

    /// Show or hide the `‹n` / `n›` hidden-column badges.
    pub fn overflow_badges(mut self, on: bool) -> Self {
        self.overflow_badges = on;
        self
    }

    /// The pending-changes bar's labels: preview, discard, save. Default
    /// the theme's `Preview SQL`, `Discard`, `Save`
    /// ([`crate::theme::Strings`]); labels set here no longer follow it.
    pub fn bar_labels(mut self, labels: [&str; 3]) -> Self {
        for (b, l) in self.bar.iter_mut().zip(labels) {
            b.label = l.to_owned();
        }
        self
    }

    /// What an empty grid shows.
    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.empty = state;
        self
    }

    /// Replace the cell validator (default [`default_validator`]).
    pub fn validator(
        mut self,
        f: impl Fn(&ColumnSpec, &str) -> Result<CellValue, String> + 'static,
    ) -> Self {
        self.validator = Validator::new(f);
        self
    }

    pub fn editable(mut self, on: bool) -> Self {
        self.editable = on;
        self
    }

    /// The keys that duplicate the cursor row (default Ctrl+D and Alt+D):
    /// replaces the [`GridAction::DuplicateRow`] bindings in
    /// [`Self::keymap`]. An empty list turns the shortcut off;
    /// [`Self::duplicate_row`] stays.
    pub fn duplicate_keys(mut self, keys: &[Key]) -> Self {
        self.keymap.retain(|(_, a)| *a != GridAction::DuplicateRow);
        self.keymap.extend(
            keys.iter()
                .map(|k| (KeyBinding::new(k.code, k.mods), GridAction::DuplicateRow)),
        );
        self
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<GridAction>) -> Self {
        self.keymap = keymap;
        self
    }

    /// How many leading rows size the columns (default
    /// [`DEFAULT_WIDTH_SAMPLE_ROWS`]). Takes effect on the next
    /// [`Self::set_rows`].
    pub fn width_sample_rows(mut self, rows: usize) -> Self {
        self.width_sample_rows = rows;
        self
    }

    /// Header seams resize columns by drag (double-click autofits).
    /// Default on.
    pub fn resizable_columns(mut self, on: bool) -> Self {
        self.sizing.resizable = on;
        self
    }

    /// Hit width of a header seam in cells (default
    /// [`crate::ui::columns::DEFAULT_SEAM_W`]).
    pub fn seam_hit_w(mut self, cells: u16) -> Self {
        self.sizing.seam_w = cells.max(1);
        self
    }

    /// The width column `c` draws at: its user width, else the measured one.
    fn col_w(&self, c: usize) -> u16 {
        self.sizing
            .width(c, self.widths.get(c).copied().unwrap_or(0))
    }

    /// Fit column `c` to its header and the values of the visible rows
    /// (no wider than the grid), as a user width.
    pub fn autofit(&mut self, c: usize) {
        let Some(col) = self.columns.get(c) else {
            return;
        };
        let cap = (self.area.width.saturating_sub(4) as usize).max(8);
        let sorted = self.sort.is_some_and(|(sc, _)| sc == c);
        let mut need = width(&col.name)
            + if col.primary { 2 } else { 0 }
            + if self.filtered_cols.contains(&c) {
                2
            } else {
                0
            }
            + if sorted { 2 } else { 0 };
        for d in self.scroll.visible_range() {
            if let Some(&src) = self.order.get(d) {
                need = need.max(width_capped(&self.value(src, c).text_ref(), cap));
            }
        }
        self.sizing.set(c, need.min(cap) as u16);
    }

    /// The header context menu of column `c`: sort, autofit and reset
    /// widths, each paired with the action [`Self::on_header_action`] runs.
    pub fn header_menu(&self, c: usize) -> Vec<(HeaderAction, crate::widgets::menu::MenuItem)> {
        let sortable = self.columns.get(c).is_some_and(|col| col.sortable);
        let sorted = self.sort.and_then(|(sc, d)| (sc == c).then_some(d));
        crate::ui::columns::header_menu(
            &self.header_labels,
            sortable,
            sorted,
            self.sizing.resizable,
            !self.sizing.is_default(),
        )
    }

    /// Run a header menu command on column `c`.
    pub fn on_header_action(
        &mut self,
        c: usize,
        action: HeaderAction,
    ) -> (Outcome, Option<GridEvent>) {
        match action {
            HeaderAction::SortAsc => self.request_sort(Some((c, SortDir::Asc))),
            HeaderAction::SortDesc => self.request_sort(Some((c, SortDir::Desc))),
            HeaderAction::ClearSort => self.request_sort(None),
            HeaderAction::Autofit => {
                self.autofit(c);
                (Outcome::Changed, None)
            }
            HeaderAction::ResetWidths => {
                self.sizing.reset();
                (Outcome::Changed, None)
            }
        }
    }

    // ---- data -------------------------------------------------------

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn rows(&self) -> &[Vec<CellValue>] {
        &self.rows
    }
    pub fn source_row(&self, display: usize) -> usize {
        self.order.get(display).copied().unwrap_or(display)
    }
    /// Effective value (pending edit or stored).
    pub fn value(&self, src: usize, col: usize) -> &CellValue {
        self.pending.value(src, col).unwrap_or_else(|| {
            self.rows
                .get(src)
                .and_then(|r| r.get(col))
                .unwrap_or(&CellValue::Null)
        })
    }

    /// Replace all rows: resets pending edits, cursor and widths.
    pub fn set_rows(&mut self, data: GridRows) {
        self.rows = data.rows;
        self.total = data.total;
        self.more = data.more;
        self.order = (0..self.rows.len()).collect();
        self.pending = PendingChanges::default();
        self.undo.clear();
        self.cell_errors.clear();
        self.row_errors.clear();
        self.selected_rows.clear();
        self.anchor = None;
        self.edit = None;
        self.cursor.0 = self.cursor.0.min(self.rows.len().saturating_sub(1));
        self.sample_widths();
        if self.local_sort
            && let Some((c, d)) = self.sort
        {
            self.apply_local_sort(c, d);
        }
        self.scroll.set_content(self.content_rows());
        self.scroll.ensure_visible(self.cursor.0);
    }

    /// Fetch-more result: keeps widths, cursor and pending changes.
    pub fn append_rows(&mut self, data: GridRows) {
        let start = self.rows.len();
        self.rows.extend(data.rows);
        self.order.extend(start..self.rows.len());
        self.total = data.total;
        self.more = data.more;
        self.loading = false;
        self.scroll.set_content(self.content_rows());
    }

    pub fn set_loading(&mut self, on: bool) {
        self.loading = on;
    }

    fn content_rows(&self) -> usize {
        self.rows.len() + usize::from(self.more)
    }

    fn sample_widths(&mut self) {
        self.widths = self
            .columns
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                let header = width(&c.name) as u16 + if c.primary { 2 } else { 0 } + 2;
                // a header never ellipsises below its own name (up to a sane cap)
                let max = c.max_width.max(header.min(24));
                // the result is clamped to `max`, so a value only needs
                // measuring up to it: a long one costs its first cells
                let mut ws: Vec<usize> = self
                    .rows
                    .iter()
                    .take(self.width_sample_rows)
                    .map(|r| {
                        r.get(ci)
                            .map_or(0, |v| width_capped(&v.text_ref(), max as usize))
                    })
                    .collect();
                ws.sort_unstable();
                let p95 = ws.get(ws.len() * 95 / 100).copied().unwrap_or(0) as u16;
                p95.max(header).clamp(c.min_width.min(max), max)
            })
            .collect();
    }

    fn apply_local_sort(&mut self, col: usize, dir: SortDir) {
        let rows = &self.rows;
        self.order.sort_by(|&a, &b| {
            let va = &rows[a][col];
            let vb = &rows[b][col];
            let o = cmp_cells(va, vb).then_with(|| a.cmp(&b));
            if dir == SortDir::Asc { o } else { o.reverse() }
        });
    }

    pub fn row_state(&self, src: usize) -> RowState {
        self.row_state_with(src, |src| self.pending.cells.keys().any(|(r, _)| *r == src))
    }

    /// [`Self::row_state`] with the "has an edited cell" test supplied: a
    /// frame collects the edited rows once instead of scanning every
    /// pending cell for every visible row.
    fn row_state_with(&self, src: usize, edited: impl Fn(usize) -> bool) -> RowState {
        if self.row_errors.contains_key(&src) {
            RowState::Error
        } else if self.pending.deleted.contains(&src) {
            RowState::Deleted
        } else if self.pending.inserted.contains(&src) {
            RowState::Inserted
        } else if edited(src) {
            RowState::Modified
        } else {
            RowState::Clean
        }
    }

    // ---- editing ----------------------------------------------------

    pub fn is_editing(&self) -> bool {
        if self.choice.is_some() {
            return true;
        }
        self.edit.is_some()
    }
    pub fn edit_error(&self) -> Option<&str> {
        if let Some(c) = &self.choice {
            return c.editor.error.as_deref();
        }
        self.edit.as_ref().and_then(|e| e.error.as_deref())
    }

    fn cursor_src(&self) -> Option<usize> {
        self.order.get(self.cursor.0).copied()
    }

    /// Why the cursor cell cannot be edited, or `None` when it can (or
    /// when the grid is not editable and gives no reason).
    pub fn read_only_reason_at_cursor(&self) -> Option<String> {
        let src = self.cursor_src()?;
        if !self.editable {
            return self.read_only_reason.clone();
        }
        let spec = self.columns.get(self.cursor.1)?;
        if self.pending.deleted.contains(&src) {
            Some(self.strings.grid_row_deleted.to_owned())
        } else if spec.read_only {
            Some(crate::theme::Strings::fill(
                self.strings.grid_read_only,
                &[&spec.name],
            ))
        } else {
            None
        }
    }

    /// Whether Enter would edit the cursor cell: hints offer "Edit" only
    /// then.
    pub fn cell_editable(&self) -> bool {
        self.editable
            && !self.on_more_row()
            && self.cursor_src().is_some()
            && self.read_only_reason_at_cursor().is_none()
    }

    pub fn begin_edit(&mut self) -> (Outcome, Option<GridEvent>) {
        let Some(src) = self.cursor_src() else {
            return (Outcome::Consumed, None);
        };
        if !self.editable {
            return (Outcome::Consumed, None);
        }
        let col = self.cursor.1;
        if let Some(reason) = self.read_only_reason_at_cursor() {
            return (Outcome::Changed, Some(GridEvent::ReadOnly { reason }));
        }
        let spec = &self.columns[col];
        let v = self.value(src, col).clone();
        // a constrained column: a checkbox flips, the others open a popup
        if spec.editor.is_constrained() {
            let raw = v.edit_text();
            if let Some(next) = spec.editor.toggled(&raw) {
                return match self.apply_choice(src, col, next) {
                    Ok(ev) => (Outcome::Changed, Some(ev)),
                    Err(_) => (Outcome::Consumed, None),
                };
            }
            let opened =
                ChoiceEditor::open(self.choice_id(), &spec.editor, &raw, &self.editor_config);
            self.choice = opened.map(|editor| GridChoice {
                row: src,
                col,
                editor,
            });
            return (Outcome::Changed, None);
        }
        match spec.kind {
            CellKind::Bool => {
                let next = match v {
                    CellValue::Bool(true) => CellValue::Bool(false),
                    CellValue::Bool(false) if spec.nullable => CellValue::Null,
                    _ => CellValue::Bool(true),
                };
                self.record_cell(src, col, next);
                return (
                    Outcome::Changed,
                    Some(GridEvent::CellChanged { row: src, col }),
                );
            }
            CellKind::Json => {
                return (
                    Outcome::Changed,
                    Some(GridEvent::OpenViewer { row: src, col }),
                );
            }
            CellKind::Text
                if width(&v.text()) > 2 * self.widths.get(col).copied().unwrap_or(20) as usize =>
            {
                return (
                    Outcome::Changed,
                    Some(GridEvent::OpenViewer { row: src, col }),
                );
            }
            _ => {}
        }
        self.edit = Some(EditState {
            row: src,
            col,
            buffer: TextBuffer::single(v.edit_text()),
            error: None,
        });
        (Outcome::Changed, None)
    }

    pub fn commit_edit(&mut self) -> Option<GridEvent> {
        let e = self.edit.take()?;
        let spec = &self.columns[e.col];
        let text = e.buffer.text().to_owned();
        let parsed = if text.is_empty() {
            match spec.kind {
                CellKind::Text | CellKind::Json | CellKind::Enum => {
                    Ok(CellValue::Text(String::new()))
                }
                _ if spec.nullable => Ok(CellValue::Null),
                _ => Err("Empty: use Delete for NULL".to_owned()),
            }
        } else {
            (self.validator.0)(spec, &text)
        };
        match parsed {
            Ok(v) => {
                self.record_cell(e.row, e.col, v);
                Some(GridEvent::CellChanged {
                    row: e.row,
                    col: e.col,
                })
            }
            Err(msg) => {
                self.edit = Some(EditState {
                    error: Some(msg),
                    ..e
                });
                None
            }
        }
    }

    pub fn cancel_edit(&mut self) -> bool {
        self.edit.take().is_some()
    }

    /// Focus left the grid (see [`crate::core::focus::Focus::take_changes`]).
    /// A cell edit in progress ends per [`Self::blur_action`]: a commit
    /// reports `CellChanged` like Enter (an invalid value stays open with
    /// its error), a cancel drops the edit.
    pub fn on_blur(&mut self) -> (Outcome, Option<GridEvent>) {
        if let Some(c) = &self.choice {
            let r = match self.editor_config.blur {
                BlurAction::Commit => match c.editor.blur_value(&self.editor_config) {
                    Some(v) => ChoiceResult::Commit(v),
                    None => ChoiceResult::Cancel,
                },
                BlurAction::Cancel => ChoiceResult::Cancel,
                BlurAction::Keep => return (Outcome::Ignored, None),
            };
            return (Outcome::Changed, self.on_choice(Some(r)));
        }
        if self.edit.is_none() {
            return (Outcome::Ignored, None);
        }
        match self.blur_action {
            BlurAction::Commit => (Outcome::Changed, self.commit_edit()),
            BlurAction::Cancel => {
                self.cancel_edit();
                (Outcome::Changed, None)
            }
            BlurAction::Keep => (Outcome::Ignored, None),
        }
    }

    /// Set a pending value; reverting to the stored value clears the change.
    pub fn record_cell(&mut self, src: usize, col: usize, value: CellValue) {
        let before = self.pending.cells.get(&(src, col)).cloned();
        let stored = self
            .rows
            .get(src)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(CellValue::Null);
        if value == stored && !self.pending.inserted.contains(&src) {
            self.pending.cells.remove(&(src, col));
        } else {
            self.pending.cells.insert((src, col), value);
        }
        self.cell_errors.remove(&(src, col));
        self.undo.push(UndoAction::Cell {
            row: src,
            col,
            before,
        });
    }

    pub fn toggle_delete(&mut self, src: usize) -> Option<GridEvent> {
        if !self.editable {
            return None;
        }
        if self.pending.inserted.contains(&src) {
            // deleting an inserted row removes it entirely
            self.remove_inserted(src);
            return Some(GridEvent::RowDeleted(src));
        }
        let was = self.pending.deleted.contains(&src);
        if was {
            self.pending.deleted.remove(&src);
        } else {
            self.pending.deleted.insert(src);
            self.pending.cells.retain(|(r, _), _| *r != src);
        }
        self.undo.push(UndoAction::Delete {
            row: src,
            was_deleted: was,
        });
        Some(GridEvent::RowDeleted(src))
    }

    fn remove_inserted(&mut self, src: usize) {
        self.rows.remove(src);
        self.order = (0..self.rows.len()).collect();
        self.pending.inserted.remove(&src);
        self.pending.cells.retain(|(r, _), _| *r != src);
        let shift = |s: &mut BTreeSet<usize>| {
            *s = s.iter().map(|&r| if r > src { r - 1 } else { r }).collect();
        };
        shift(&mut self.pending.inserted);
        shift(&mut self.pending.deleted);
        self.pending.cells = self
            .pending
            .cells
            .drain()
            .map(|((r, c), v)| ((if r > src { r - 1 } else { r }, c), v))
            .collect();
        self.scroll.set_content(self.content_rows());
        self.cursor.0 = self.cursor.0.min(self.rows.len().saturating_sub(1));
    }

    pub fn insert_row(&mut self) -> Option<GridEvent> {
        if !self.editable {
            return None;
        }
        let row: Vec<CellValue> = self
            .columns
            .iter()
            .map(|c| {
                if c.primary || c.read_only {
                    CellValue::Default
                } else {
                    CellValue::Null
                }
            })
            .collect();
        self.rows.push(row);
        let src = self.rows.len() - 1;
        self.order.push(src);
        self.pending.inserted.insert(src);
        self.undo.push(UndoAction::Insert { row: src });
        self.scroll.set_content(self.content_rows());
        // cursor to the new row, first writable column
        let disp = self.order.iter().position(|&r| r == src).unwrap_or(0);
        let col = self
            .columns
            .iter()
            .position(|c| !c.primary && !c.read_only)
            .unwrap_or(0);
        self.set_cursor(disp, col, false);
        Some(GridEvent::RowInserted(src))
    }

    /// Duplicate the current row, resetting generated columns to defaults.
    /// [`Self::duplicate_keys`] invoke this; read-only grids refuse it.
    pub fn duplicate_row(&mut self) -> Option<GridEvent> {
        let src = self.cursor_src()?;
        if !self.editable {
            return None;
        }
        let mut row: Vec<CellValue> = (0..self.columns.len())
            .map(|c| self.value(src, c).clone())
            .collect();
        for (ci, c) in self.columns.iter().enumerate() {
            if c.primary || c.read_only {
                row[ci] = CellValue::Default;
            }
        }
        self.rows.push(row);
        let new = self.rows.len() - 1;
        self.order.push(new);
        self.pending.inserted.insert(new);
        self.undo.push(UndoAction::Insert { row: new });
        self.scroll.set_content(self.content_rows());
        let disp = self.order.len() - 1;
        self.set_cursor(disp, self.cursor.1, false);
        Some(GridEvent::RowInserted(new))
    }

    pub fn undo(&mut self) -> bool {
        let Some(a) = self.undo.pop() else {
            return false;
        };
        match a {
            UndoAction::Cell { row, col, before } => match before {
                Some(v) => {
                    self.pending.cells.insert((row, col), v);
                }
                None => {
                    self.pending.cells.remove(&(row, col));
                }
            },
            UndoAction::Delete { row, was_deleted } => {
                if was_deleted {
                    self.pending.deleted.insert(row);
                } else {
                    self.pending.deleted.remove(&row);
                }
            }
            UndoAction::Insert { row } => {
                if self.pending.inserted.contains(&row) {
                    self.remove_inserted(row);
                }
            }
        }
        true
    }

    /// The owner reports the commit outcome.
    pub fn apply_commit_result(&mut self, result: Result<(), (usize, String)>) {
        match result {
            Ok(()) => {
                // fold pending values into stored rows; drop deleted rows
                for ((r, c), v) in self.pending.cells.drain() {
                    if let Some(row) = self.rows.get_mut(r)
                        && let Some(cell) = row.get_mut(c)
                    {
                        *cell = v;
                    }
                }
                let deleted = std::mem::take(&mut self.pending.deleted);
                let mut keep: Vec<Vec<CellValue>> = Vec::new();
                for (i, row) in self.rows.drain(..).enumerate() {
                    if !deleted.contains(&i) {
                        keep.push(row);
                    }
                }
                self.rows = keep;
                self.order = (0..self.rows.len()).collect();
                self.pending = PendingChanges::default();
                self.undo.clear();
                self.row_errors.clear();
                self.cell_errors.clear();
                self.scroll.set_content(self.content_rows());
                self.cursor.0 = self.cursor.0.min(self.rows.len().saturating_sub(1));
            }
            Err((row, msg)) => {
                self.row_errors.insert(row, msg);
            }
        }
    }

    pub fn discard(&mut self) {
        let inserted: Vec<usize> = self.pending.inserted.iter().copied().collect();
        for src in inserted.into_iter().rev() {
            self.remove_inserted(src);
        }
        self.pending = PendingChanges::default();
        self.undo.clear();
        self.cell_errors.clear();
        self.row_errors.clear();
    }

    // ---- navigation ------------------------------------------------

    fn set_cursor(&mut self, row: usize, col: usize, extend: bool) {
        let n = self.content_rows();
        let row = row.min(n.saturating_sub(1));
        let col = col.min(self.columns.len().saturating_sub(1));
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = (row, col);
        self.scroll.ensure_visible(row);
        self.ensure_col_visible();
    }

    fn ensure_col_visible(&mut self) {
        let n = self.columns.len();
        if self.hscroll.viewport_len == 0 {
            return;
        }
        if self.cursor.1 < self.hscroll.offset {
            self.hscroll.offset = self.cursor.1;
        } else if self.cursor.1 >= self.hscroll.offset + self.hscroll.viewport_len {
            self.hscroll.offset = self.cursor.1 + 1 - self.hscroll.viewport_len;
        }
        self.hscroll.offset = self
            .hscroll
            .offset
            .min(n.saturating_sub(self.hscroll.viewport_len));
    }

    /// Is the display cell inside the current range selection?
    fn in_range(&self, row: usize, col: usize) -> bool {
        let Some(a) = self.anchor else {
            return false;
        };
        let (r0, r1) = (a.0.min(self.cursor.0), a.0.max(self.cursor.0));
        let (c0, c1) = (a.1.min(self.cursor.1), a.1.max(self.cursor.1));
        row >= r0 && row <= r1 && col >= c0 && col <= c1
    }

    fn on_more_row(&self) -> bool {
        self.more && self.cursor.0 == self.rows.len()
    }

    fn copy_text(&self, with_header: bool) -> String {
        let mut out = String::new();
        if let Some(a) = self.anchor {
            let (r0, r1) = (a.0.min(self.cursor.0), a.0.max(self.cursor.0));
            let (c0, c1) = (a.1.min(self.cursor.1), a.1.max(self.cursor.1));
            if with_header {
                out.push_str(
                    &self.columns[c0..=c1]
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join("\t"),
                );
                out.push('\n');
            }
            for r in r0..=r1 {
                if let Some(&src) = self.order.get(r) {
                    let line: Vec<String> = (c0..=c1).map(|c| self.value(src, c).text()).collect();
                    out.push_str(&line.join("\t"));
                    out.push('\n');
                }
            }
        } else if !self.selected_rows.is_empty() || with_header {
            let rows: Vec<usize> = if self.selected_rows.is_empty() {
                self.cursor_src().into_iter().collect()
            } else {
                self.selected_rows.iter().copied().collect()
            };
            if with_header {
                out.push_str(
                    &self
                        .columns
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join("\t"),
                );
                out.push('\n');
            }
            for src in rows {
                let line: Vec<String> = (0..self.columns.len())
                    .map(|c| self.value(src, c).text())
                    .collect();
                out.push_str(&line.join("\t"));
                out.push('\n');
            }
        } else if let Some(src) = self.cursor_src() {
            out = self.value(src, self.cursor.1).text();
        }
        out
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<GridEvent>) {
        if let Some(c) = self.choice.as_mut() {
            let (out, r) = c.editor.on_key(key, &self.editor_config);
            return (out, self.on_choice(r));
        }
        // Alt+↓ opens a constrained cell's popup (plain ↓ moves)
        if key.code == KeyCode::Down
            && key.alt()
            && self.editable
            && self
                .columns
                .get(self.cursor.1)
                .is_some_and(|c| c.editor.has_popup())
        {
            return self.begin_edit();
        }
        if let Some(e) = self.edit.as_mut() {
            return match edit_key(key, false) {
                EditAction::Commit => match self.commit_edit() {
                    Some(ev) => (Outcome::Changed, Some(ev)),
                    None => (Outcome::Changed, None),
                },
                EditAction::Cancel => {
                    self.cancel_edit();
                    (Outcome::Changed, None)
                }
                EditAction::Tab { backward } => {
                    let ev = self.commit_edit();
                    if self.edit.is_some() {
                        return (Outcome::Changed, None);
                    }
                    // next writable cell
                    let n = self.columns.len();
                    let mut col = self.cursor.1;
                    loop {
                        if backward {
                            if col == 0 {
                                return (Outcome::Changed, Some(GridEvent::LeaveBackward));
                            }
                            col -= 1;
                        } else {
                            col += 1;
                            if col >= n {
                                return (Outcome::Changed, Some(GridEvent::LeaveForward));
                            }
                        }
                        if !self.columns[col].read_only && !self.columns[col].primary {
                            break;
                        }
                    }
                    self.set_cursor(self.cursor.0, col, false);
                    let _ = self.begin_edit();
                    (Outcome::Changed, ev)
                }
                EditAction::Apply(f) => {
                    f(&mut e.buffer);
                    (Outcome::Changed, None)
                }
                action @ (EditAction::Undo
                | EditAction::Redo
                | EditAction::Copy
                | EditAction::Cut
                | EditAction::Paste
                | EditAction::SelectAll) => {
                    // a cell edit is short-lived: clipboard keys work, its
                    // undo is Esc
                    crate::widgets::field_common::apply_edit(
                        &mut e.buffer,
                        &mut crate::core::edit::UndoStack::new(),
                        action,
                        key.code,
                        false,
                    );
                    (Outcome::Changed, None)
                }
                EditAction::Insert(c) => {
                    e.buffer.insert_char(c);
                    e.error = None;
                    (Outcome::Changed, None)
                }
                EditAction::None => (Outcome::Consumed, None),
            };
        }
        // pending bar buttons are separate focus stops; grid keys only here
        let (r, c) = self.cursor;
        let shift = key.shift();
        let ctrl = key.ctrl();
        let n_rows = self.content_rows();
        if n_rows == 0 {
            // an empty grid: only the actions that do not need a row (the
            // filter-on-cell key opens the filters instead)
            return match lookup(&self.keymap, key) {
                Some(GridAction::InsertRow) if self.editable => {
                    (Outcome::Changed, self.insert_row())
                }
                Some(GridAction::Refresh) => (Outcome::Changed, Some(GridEvent::Refresh)),
                Some(GridAction::FilterOnCell) => (Outcome::Changed, Some(GridEvent::OpenFilters)),
                Some(GridAction::ClearFilters) => (Outcome::Changed, Some(GridEvent::ClearFilters)),
                _ => (Outcome::Ignored, None),
            };
        }
        if let KeyCode::Char(ch) = key.code
            && self.type_ahead
            && !ch.is_whitespace()
            && !key.chord()
        {
            let col = self.type_ahead_column.unwrap_or(c);
            let prefix = self.typeahead.feed(ch).to_owned();
            let texts: Vec<String> = self
                .order
                .iter()
                .map(|&src| {
                    self.rows[src]
                        .get(col)
                        .map(CellValue::text)
                        .unwrap_or_default()
                })
                .collect();
            let found = crate::core::typeahead::TypeAhead::find(
                &prefix,
                texts.iter().map(String::as_str),
                r.min(texts.len().saturating_sub(1)),
                |_| true,
            );
            return match found {
                Some(d) => {
                    self.set_cursor(d, c, false);
                    (Outcome::Changed, None)
                }
                None => (Outcome::Consumed, None),
            };
        }
        if let Some(action) = lookup(&self.keymap, key) {
            return self.on_action(action);
        }
        if let Some(nav) = self.nav.action(key) {
            // `g`/`G` jump to the first/last row; Home/End stay in the row
            // (spreadsheet convention) and Ctrl+Home/End go to the ends
            let letter = matches!(key.code, KeyCode::Char(_));
            let page = self.scroll.viewport_len.max(1);
            match nav {
                Nav::Up => self.set_cursor(r.saturating_sub(1), c, shift),
                Nav::Down => self.set_cursor(r + 1, c, shift),
                Nav::Left if ctrl => {
                    self.hscroll
                        .scroll_by(-(self.hscroll.viewport_len.max(1) as isize));
                }
                Nav::Right if ctrl => {
                    self.hscroll
                        .scroll_by(self.hscroll.viewport_len.max(1) as isize);
                }
                Nav::Left => self.set_cursor(r, c.saturating_sub(1), shift),
                Nav::Right => self.set_cursor(r, c + 1, shift),
                Nav::Home if letter => self.set_cursor(0, c, false),
                Nav::End if letter => self.set_cursor(usize::MAX, c, false),
                Nav::Home if ctrl => self.set_cursor(0, c, shift),
                Nav::End if ctrl => self.set_cursor(usize::MAX, c, shift),
                Nav::Home => self.set_cursor(r, 0, shift),
                Nav::End => self.set_cursor(r, usize::MAX, shift),
                Nav::PageUp => self.set_cursor(r.saturating_sub(page), c, shift),
                Nav::PageDown => self.set_cursor(r + page, c, shift),
            }
            return (Outcome::Changed, None);
        }
        match key.code {
            KeyCode::Enter | KeyCode::F(2) => {
                if self.on_more_row() {
                    return (Outcome::Changed, Some(GridEvent::FetchMore));
                }
                if !self.editable {
                    // a grid that says why it is read-only says so on
                    // Enter; one without a reason is a list (activation)
                    if let Some(reason) = self.read_only_reason_at_cursor() {
                        return (Outcome::Changed, Some(GridEvent::ReadOnly { reason }));
                    }
                    return (
                        Outcome::Changed,
                        self.cursor_src().map(GridEvent::Activated),
                    );
                }
                self.begin_edit()
            }
            KeyCode::Char(' ') => {
                if self.on_more_row() {
                    return (Outcome::Changed, Some(GridEvent::FetchMore));
                }
                if let Some(src) = self.cursor_src()
                    && !self.selected_rows.remove(&src)
                {
                    self.selected_rows.insert(src);
                }
                (Outcome::Changed, None)
            }
            KeyCode::Esc => {
                if self.anchor.is_some() || !self.selected_rows.is_empty() {
                    self.anchor = None;
                    self.selected_rows.clear();
                    (Outcome::Changed, None)
                } else {
                    (Outcome::Ignored, None)
                }
            }
            _ => (Outcome::Ignored, None),
        }
    }

    fn on_action(&mut self, action: GridAction) -> (Outcome, Option<GridEvent>) {
        let c = self.cursor.1;
        match action {
            GridAction::ClearCell => {
                if !self.editable {
                    return (Outcome::Consumed, None);
                }
                if !self.selected_rows.is_empty() {
                    let rows: Vec<usize> = self.selected_rows.iter().copied().collect();
                    let mut last = None;
                    for src in rows {
                        last = self.toggle_delete(src);
                    }
                    return (Outcome::Changed, last);
                }
                let Some(src) = self.cursor_src() else {
                    return (Outcome::Consumed, None);
                };
                let spec = &self.columns[c];
                if spec.read_only {
                    return (Outcome::Consumed, None);
                }
                if spec.nullable {
                    self.record_cell(src, c, CellValue::Null);
                    (
                        Outcome::Changed,
                        Some(GridEvent::CellChanged { row: src, col: c }),
                    )
                } else {
                    self.cell_errors
                        .insert((src, c), format!("{} is NOT NULL", spec.name));
                    (Outcome::Changed, None)
                }
            }
            GridAction::ToggleDelete => {
                let Some(src) = self.cursor_src() else {
                    return (Outcome::Consumed, None);
                };
                (Outcome::Changed, self.toggle_delete(src))
            }
            GridAction::InsertRow => (Outcome::Changed, self.insert_row()),
            GridAction::DuplicateRow => (Outcome::Changed, self.duplicate_row()),
            GridAction::Copy => (
                Outcome::Changed,
                Some(GridEvent::Copy(self.copy_text(false))),
            ),
            GridAction::CopyWithHeader => (
                Outcome::Changed,
                Some(GridEvent::Copy(self.copy_text(true))),
            ),
            GridAction::Commit => (Outcome::Changed, Some(GridEvent::CommitRequested)),
            GridAction::Sort => {
                if !self.columns[c].sortable {
                    return (Outcome::Consumed, None);
                }
                let next = match self.sort {
                    Some((sc, SortDir::Asc)) if sc == c => Some((c, SortDir::Desc)),
                    Some((sc, SortDir::Desc)) if sc == c => None,
                    _ => Some((c, SortDir::Asc)),
                };
                self.request_sort(next)
            }
            GridAction::ClearSort => self.request_sort(None),
            GridAction::FilterOnCell => {
                let value = self
                    .cursor_src()
                    .map(|src| self.value(src, c).clone())
                    .unwrap_or(CellValue::Null);
                (
                    Outcome::Changed,
                    Some(GridEvent::FilterOnCell { col: c, value }),
                )
            }
            GridAction::OpenFilters => (Outcome::Changed, Some(GridEvent::OpenFilters)),
            GridAction::ClearFilters => (Outcome::Changed, Some(GridEvent::ClearFilters)),
            GridAction::Refresh => (Outcome::Changed, Some(GridEvent::Refresh)),
            GridAction::Undo => {
                self.undo();
                (Outcome::Changed, None)
            }
            GridAction::Discard => (Outcome::Changed, Some(GridEvent::DiscardRequested)),
            GridAction::SelectAll => {
                let before = self.selected_rows.len();
                self.selected_rows.extend(self.order.iter().copied());
                if self.selected_rows.len() != before {
                    (Outcome::Changed, None)
                } else {
                    (Outcome::Consumed, None)
                }
            }
            GridAction::PreviewSql => (Outcome::Changed, Some(GridEvent::PreviewSql)),
            GridAction::FollowReference => {
                if self.columns[c].references.is_some()
                    && let Some(src) = self.cursor_src()
                {
                    return (
                        Outcome::Changed,
                        Some(GridEvent::FollowReference { row: src, col: c }),
                    );
                }
                (Outcome::Consumed, None)
            }
        }
    }

    fn request_sort(&mut self, next: Option<(usize, SortDir)>) -> (Outcome, Option<GridEvent>) {
        self.sort = next;
        if self.local_sort {
            match next {
                Some((c, d)) => self.apply_local_sort(c, d),
                None => self.order = (0..self.rows.len()).collect(),
            }
            (Outcome::Changed, None)
        } else {
            (Outcome::Changed, Some(GridEvent::SortRequested(next)))
        }
    }

    // ---- ids / hit-testing -----------------------------------------

    pub fn header_id(&self, c: usize) -> WidgetId {
        self.id.sub("header").child(c)
    }
    pub fn cell_id(&self, display: usize, c: usize) -> WidgetId {
        self.id.child(display).child(c)
    }
    /// The resize seam after column `c` in the header.
    pub fn seam_id(&self, c: usize) -> WidgetId {
        self.id.sub("seam").child(c)
    }
    pub fn locate_seam(&self, id: WidgetId) -> Option<usize> {
        (0..self.columns.len()).find(|&c| self.seam_id(c) == id)
    }
    /// The column whose header menu a secondary click on `id` opens (its
    /// header or its seam).
    pub fn locate_header_menu(&self, id: WidgetId) -> Option<usize> {
        self.locate_header(id).or_else(|| self.locate_seam(id))
    }
    pub fn rownum_id(&self, display: usize) -> WidgetId {
        self.id.sub("rownum").child(display)
    }
    pub fn more_id(&self) -> WidgetId {
        self.id.sub("more")
    }
    pub fn left_id(&self) -> WidgetId {
        self.id.sub("hl")
    }
    pub fn right_id(&self) -> WidgetId {
        self.id.sub("hr")
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        self.choice.as_ref().is_some_and(|c| c.editor.owns(id))
            || self.locate_open(id).is_some()
            || id == self.id
            || scrollbar::is_bar_of(id, self.id)
            || id == self.more_id()
            || id == self.left_id()
            || id == self.right_id()
            || self.locate(id).is_some()
            || self.locate_header(id).is_some()
            || self.locate_seam(id).is_some()
            || self.locate_rownum(id).is_some()
            || self.bar.iter().any(|b| b.id == id)
            || matches!(self.drag, Some(GridDrag::Cells(d) | GridDrag::Rows { id: d, .. }) if d == id)
    }

    pub fn locate(&self, id: WidgetId) -> Option<(usize, usize)> {
        for d in self.scroll.visible_range() {
            for c in self.hscroll.offset
                ..(self.hscroll.offset + self.hscroll.viewport_len).min(self.columns.len())
            {
                if self.cell_id(d, c) == id {
                    return Some((d, c));
                }
            }
        }
        None
    }
    pub fn locate_header(&self, id: WidgetId) -> Option<usize> {
        (0..self.columns.len()).find(|&c| self.header_id(c) == id)
    }
    pub fn locate_rownum(&self, id: WidgetId) -> Option<usize> {
        self.scroll
            .visible_range()
            .find(|&d| self.rownum_id(d) == id)
    }

    /// Button down on `id` with the click count it starts: a press on a
    /// header seam starts a resize (keeping the grab offset); the second
    /// press of a double click autofits the column.
    pub fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        let Some(c) = self.locate_seam(id) else {
            return Outcome::Ignored;
        };
        if !self.sizing.resizable {
            return Outcome::Consumed;
        }
        if count >= 2 {
            self.sizing.release();
            self.autofit(c);
            return Outcome::Changed;
        }
        let w = self.col_w(c);
        self.sizing.press(c, w, pos.x);
        Outcome::Changed
    }

    pub fn on_click(&mut self, id: WidgetId, pos: Position) -> (Outcome, Option<GridEvent>) {
        if self.locate_seam(id).is_some() {
            // the release of a seam press or drag: the resize is done
            self.sizing.release();
            return (Outcome::Consumed, None);
        }
        if let Some(c) = self.choice.as_mut() {
            if c.editor.owns(id) {
                let (out, r) = c.editor.on_click(id, &self.editor_config);
                return (out, self.on_choice(r));
            }
            // a click anywhere else ends the popup without a choice
            self.choice = None;
        }
        if let Some((d, c)) = self.locate_open(id) {
            self.set_cursor(d, c, false);
            return self.begin_edit();
        }
        if id == self.bar[0].id && self.bar[0].on_click() {
            return (Outcome::Changed, Some(GridEvent::PreviewSql));
        }
        if id == self.bar[1].id && self.bar[1].on_click() {
            return (Outcome::Changed, Some(GridEvent::DiscardRequested));
        }
        if id == self.bar[2].id && self.bar[2].on_click() {
            return (Outcome::Changed, Some(GridEvent::CommitRequested));
        }
        if id == self.more_id() {
            self.set_cursor(self.rows.len(), self.cursor.1, false);
            return (Outcome::Changed, Some(GridEvent::FetchMore));
        }
        if id == self.left_id() {
            self.hscroll.scroll_by(-1);
            return (Outcome::Changed, None);
        }
        if id == self.right_id() {
            self.hscroll.scroll_by(1);
            return (Outcome::Changed, None);
        }
        if id == scrollbar::id_for(self.id) {
            return (self.on_scrollbar(pos), None);
        }
        if id == scrollbar::h_id_for(self.id) {
            return (self.on_hscrollbar(pos), None);
        }
        if let Some(c) = self.locate_header(id) {
            if self.edit.is_some() {
                self.commit_edit();
                if self.edit.is_some() {
                    return (Outcome::Changed, None);
                }
            }
            if !self.columns[c].sortable {
                return (Outcome::Consumed, None);
            }
            let next = match self.sort {
                Some((sc, SortDir::Asc)) if sc == c => Some((c, SortDir::Desc)),
                Some((sc, SortDir::Desc)) if sc == c => None,
                _ => Some((c, SortDir::Asc)),
            };
            return self.request_sort(next);
        }
        if let Some(d) = self.locate_rownum(id) {
            // the release that ends a mark-painting drag back on its
            // origin keeps the painted marks
            if let Some(GridDrag::Rows { id: from_id, .. }) = self.drag.take()
                && from_id == id
            {
                return (Outcome::Changed, None);
            }
            if let Some(&src) = self.order.get(d) {
                self.set_cursor(d, self.cursor.1, false);
                if !self.selected_rows.remove(&src) {
                    self.selected_rows.insert(src);
                }
            }
            return (Outcome::Changed, None);
        }
        if let Some((d, c)) = self.locate(id) {
            if let Some(e) = &self.edit {
                if self.order.get(d) == Some(&e.row) && e.col == c {
                    // click inside the editor places the cursor
                    if let Some(rect) = self.col_rects.get(c.saturating_sub(self.hscroll.offset)) {
                        let cw = rect.width.saturating_sub(1) as usize;
                        let off = e
                            .buffer
                            .cursor_pos()
                            .col
                            .saturating_sub(cw.saturating_sub(1));
                        let col = pos.x.saturating_sub(rect.x) as usize + off;
                        self.edit
                            .as_mut()
                            .unwrap()
                            .buffer
                            .set_cursor_line_col(0, col);
                    }
                    return (Outcome::Changed, None);
                }
                self.commit_edit();
                if self.edit.is_some() {
                    return (Outcome::Changed, None);
                }
            }
            let same = self.cursor == (d, c) && self.anchor.is_none();
            // click on the trailing → of a reference cell follows it
            if let Some(rect) = self.col_rects.get(c.saturating_sub(self.hscroll.offset))
                && self.columns[c].references.is_some()
                && pos.x + 1 == rect.right()
                && let Some(&src) = self.order.get(d)
            {
                return (
                    Outcome::Changed,
                    Some(GridEvent::FollowReference { row: src, col: c }),
                );
            }
            self.set_cursor(d, c, false);
            if same && self.editable {
                return self.begin_edit();
            }
            if same && !self.editable {
                return (
                    Outcome::Changed,
                    self.cursor_src().map(GridEvent::Activated),
                );
            }
            return (Outcome::Changed, None);
        }
        (Outcome::Ignored, None)
    }

    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if self.locate_seam(pressed).is_some() {
            return match self.sizing.drag_to(pos.x) {
                Some(_) => Outcome::Changed,
                None => Outcome::Consumed,
            };
        }
        if pressed == scrollbar::id_for(self.id) {
            return self.on_scrollbar_drag(pos);
        }
        if pressed == scrollbar::h_id_for(self.id) {
            return self.on_hscrollbar_drag(pos);
        }
        // a drag from a row number paints the rows' marks
        let row_origin = match self.drag {
            Some(GridDrag::Rows { id, .. }) if id == pressed => None,
            _ => self.locate_rownum(pressed),
        };
        if row_origin.is_some()
            || matches!(self.drag, Some(GridDrag::Rows { id, .. }) if id == pressed)
        {
            return self.drag_rows(pressed, row_origin, pos);
        }
        // range selection from a cell: the cell under the pointer, or past
        // an edge the row or column beyond it (as many as the pointer is
        // past the edge), so a held drag autoscrolls
        let from_cell = matches!(self.drag, Some(GridDrag::Cells(id)) if id == pressed)
            || self.locate(pressed).is_some();
        if !from_cell || self.body.height == 0 {
            return Outcome::Ignored;
        }
        self.drag = Some(GridDrag::Cells(pressed));
        let (d, c) = (self.drag_row(pos.y), self.drag_col(pos.x));
        if (d, c) != self.cursor {
            self.set_cursor(d, c, true);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    /// The display row a drag at `y` names: past the top or bottom of the
    /// body, the row as many rows beyond the visible ones.
    fn drag_row(&self, y: u16) -> usize {
        let d = self.scroll.offset as isize + (y as isize - self.body.y as isize);
        (d.max(0) as usize).min(self.content_rows().saturating_sub(1))
    }

    /// The column a drag at `x` names: left of the cells the one before
    /// the first visible, right of them the one after the last.
    fn drag_col(&self, x: u16) -> usize {
        let (Some(first), Some(last)) = (self.col_rects.first(), self.col_rects.last()) else {
            return self.cursor.1;
        };
        let off = self.hscroll.offset;
        let c = if x < first.x {
            off.saturating_sub(1)
        } else if x >= last.right() + 2 {
            off + self.col_rects.len()
        } else {
            off + self
                .col_rects
                .iter()
                .position(|r| x >= r.x && x < r.right() + 2)
                .unwrap_or(0)
        };
        c.min(self.columns.len().saturating_sub(1))
    }

    /// A drag that started on a row number: every row between it and the
    /// pointer takes the mark the origin's click would give it.
    fn drag_rows(&mut self, pressed: WidgetId, origin: Option<usize>, pos: Position) -> Outcome {
        let d = self.drag_row(pos.y);
        let (from, mark) = match (self.drag, origin) {
            (Some(GridDrag::Rows { from, mark, .. }), _) => (from, mark),
            (_, Some(from)) => {
                if d == from {
                    return Outcome::Ignored;
                }
                let src = self.order.get(from).copied();
                (from, !src.is_some_and(|s| self.selected_rows.contains(&s)))
            }
            _ => return Outcome::Ignored,
        };
        self.drag = Some(GridDrag::Rows {
            id: pressed,
            from,
            mark,
        });
        for r in from.min(d)..=from.max(d) {
            if let Some(&src) = self.order.get(r) {
                if mark {
                    self.selected_rows.insert(src);
                } else {
                    self.selected_rows.remove(&src);
                }
            }
        }
        self.set_cursor(d, self.cursor.1, false);
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32, horizontal: bool) -> Outcome {
        if let Some(c) = self.choice.as_mut()
            && !horizontal
        {
            return c.editor.on_wheel(delta);
        }
        if horizontal && !self.h_cfg.enabled {
            return Outcome::Ignored;
        }
        let moved = if horizontal {
            self.hscroll.scroll_by(delta.signum() as isize)
        } else {
            self.scroll.scroll_by(delta as isize)
        };
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    fn track(&self) -> Rect {
        if self.track.is_empty() {
            Rect::new(
                self.area.right().saturating_sub(1),
                self.body.y,
                1,
                self.body.height,
            )
        } else {
            self.track
        }
    }

    /// The pointer went down on the scrollbar (or a completed click): a
    /// press on the thumb grabs it, a press on the track jumps to it.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        if scrollbar::press_with(self.track(), pos, &mut self.scroll, self.v_click) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        if scrollbar::drag(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer went down on the horizontal scrollbar: the thumb is
    /// grabbed, or the track click moves by columns.
    pub fn on_hscrollbar(&mut self, pos: Position) -> Outcome {
        if self.htrack.is_empty() {
            return Outcome::Ignored;
        }
        if scrollbar::press_h(self.htrack, pos, &mut self.hscroll, self.h_cfg.track_click) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the horizontal scrollbar after a press.
    pub fn on_hscrollbar_drag(&mut self, pos: Position) -> Outcome {
        if self.htrack.is_empty() {
            return Outcome::Ignored;
        }
        if scrollbar::drag_h(self.htrack, pos, &mut self.hscroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(c) = self.choice.as_mut() {
            return c.editor.on_paste(text);
        }
        match self.edit.as_mut() {
            Some(e) => {
                e.buffer.insert_str(text);
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }

    // ---- labels ----------------------------------------------------

    /// `rows 120–150 of 1,203,338 · cols 3–9 of 14`
    pub fn position_label(&self) -> String {
        match self.cols_label() {
            Some(c) => format!("{} · {c}", self.rows_label()),
            None => self.rows_label(),
        }
    }

    /// `rows 120–150 of 1,203,338`
    pub fn rows_label(&self) -> String {
        use crate::ui::text::thousands;
        if self.rows.is_empty() {
            return "0 rows".into();
        }
        let r = self.scroll.visible_range();
        let (a, b) = (r.start + 1, r.end.min(self.rows.len()));
        let total = match self.total {
            RowTotal::Exact(n) if self.more => format!(
                "{} loaded · {} total",
                thousands(self.rows.len()),
                thousands(n)
            ),
            RowTotal::Estimated(n) if self.more => format!(
                "{} loaded · ~{} total",
                thousands(self.rows.len()),
                thousands(n)
            ),
            RowTotal::Exact(n) => thousands(n),
            RowTotal::Estimated(n) => format!("~{}", thousands(n)),
            RowTotal::Unknown => thousands(self.rows.len()),
        };
        format!("rows {}–{} of {total}", thousands(a), thousands(b))
    }

    /// `cols 3–9 of 14` when columns overflow horizontally.
    pub fn cols_label(&self) -> Option<String> {
        if self.rows.is_empty() || !self.hscroll.overflows() {
            return None;
        }
        let c0 = self.hscroll.offset + 1;
        let c1 = (self.hscroll.offset + self.hscroll.viewport_len).min(self.columns.len());
        Some(format!("cols {c0}–{c1} of {}", self.columns.len()))
    }

    pub fn pending_label(&self) -> Option<String> {
        if self.pending.is_empty() {
            return None;
        }
        let (u, i, d) = self.pending.counts();
        let mut parts = vec![];
        if u > 0 {
            parts.push(format!("{u} update{}", if u == 1 { "" } else { "s" }));
        }
        if i > 0 {
            parts.push(format!("{i} insert{}", if i == 1 { "" } else { "s" }));
        }
        if d > 0 {
            parts.push(format!("{d} delete{}", if d == 1 { "" } else { "s" }));
        }
        Some(parts.join(" · "))
    }

    // ---- rendering -------------------------------------------------

    fn layout_columns(&mut self, area: Rect) {
        let n = self.columns.len();
        let gap = 2u16;
        let mut fit = 0;
        let mut used = 0u16;
        for i in self.hscroll.offset..n {
            let w = self.col_w(i);
            let need = if fit == 0 { w } else { w + gap };
            if used + need > area.width {
                break;
            }
            used += need;
            fit += 1;
        }
        let fit = fit.max(1).min(n.saturating_sub(self.hscroll.offset).max(1));
        self.hscroll.content_len = n;
        self.hscroll.viewport_len = fit;
        self.hscroll.clamp();
        self.col_rects.clear();
        let mut x = area.x;
        for i in self.hscroll.offset..(self.hscroll.offset + fit).min(n) {
            let w = self.col_w(i).min(area.right().saturating_sub(x));
            self.col_rects.push(Rect::new(x, area.y, w, area.height));
            x += w + gap;
        }
        // the next column shows clipped rather than leaving the pane blank;
        // it is not part of the viewport, so moving onto it scrolls
        let next = self.hscroll.offset + fit;
        if next < n && x < area.right() {
            let rest = area.right() - x;
            if rest >= 6 {
                self.col_rects.push(Rect::new(x, area.y, rest, area.height));
            }
        }
    }

    /// Widen a column so its header keeps room for the sort/filter marks.
    fn fit_header_marks(&mut self) {
        for ci in 0..self.columns.len() {
            let sorted = self.sort.is_some_and(|(c, _)| c == ci);
            let filtered = self.filtered_cols.contains(&ci);
            if !sorted && !filtered {
                continue;
            }
            let col = &self.columns[ci];
            let need = width(&col.name)
                + if col.primary { 2 } else { 0 }
                + if filtered { 2 } else { 0 }
                + if sorted { 2 } else { 0 }
                + 1;
            let need = (need as u16).min(col.max_width.max(col.min_width));
            if let Some(w) = self.widths.get_mut(ci) {
                *w = (*w).max(need);
            }
        }
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        // a drag ends with its press
        if let Some(GridDrag::Cells(id) | GridDrag::Rows { id, .. }) = self.drag
            && ctx.interaction.pressed != Some(id)
        {
            self.drag = None;
        }
        self.follow_strings(ctx.theme.strings);
        self.v_click = self.track_click.unwrap_or(ctx.track_click);
        self.fit_header_marks();
        let area = area.intersection(*buf.area());
        if area.is_empty() || area.height < 2 {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        // a cell edit that lost focus is ended by `on_blur`, never here
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);

        let bar_h = if self.show_bar && !self.pending.is_empty() {
            2
        } else {
            0
        };
        let grid_area = Rect::new(
            area.x,
            area.y,
            area.width,
            area.height.saturating_sub(bar_h),
        );
        let mut body = Rect::new(
            grid_area.x,
            grid_area.y + 1,
            grid_area.width,
            grid_area.height.saturating_sub(1),
        );
        let h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        self.h_cfg = h_cfg;
        // the viewport is settled once both bars are known: setting a
        // provisional taller one first would clamp the offset and drop
        // the cursor row off the bottom
        let rows_len = self.content_rows();
        let mut has_sb = body.height > 0 && rows_len > body.height as usize;
        // gutter: bar(1) marker(1) change(1) rownum(w) space(1)
        let num_w = if self.row_numbers {
            (self.rows.len().max(1).to_string().len() as u16).max(2)
        } else {
            0
        };
        let gutter_w = 3 + num_w + if self.row_numbers { 1 } else { 0 };
        let cols_for = |has_sb: bool| {
            Rect::new(
                grid_area.x + gutter_w,
                grid_area.y,
                grid_area
                    .width
                    .saturating_sub(gutter_w + 4 + u16::from(has_sb)),
                grid_area.height,
            )
        };
        self.layout_columns(cols_for(has_sb));
        // the horizontal bar takes the body's bottom row; losing that row
        // can make the rows overflow, which costs a column for the
        // vertical bar, so the columns are laid out again
        let has_rows = !self.rows.is_empty() || self.more;
        let has_hb = has_rows && body.height > 1 && h_cfg.shows_bar(&self.hscroll);
        if has_hb {
            body.height -= 1;
            if !has_sb && rows_len > body.height as usize {
                has_sb = true;
                self.layout_columns(cols_for(true));
            }
        }
        let cols_area = cols_for(has_sb);
        self.scroll.set_content(rows_len);
        self.scroll.set_viewport(body.height as usize);
        self.body = body;
        if has_rows {
            ctx.autoscroll(body);
        }

        // header
        let header_y = grid_area.y;
        fill(
            buf,
            Rect::new(grid_area.x, header_y, grid_area.width, 1),
            Style::new().bg(bg),
        );
        if self.overflow_badges && self.hscroll.offset > 0 {
            let lbl = format!("{}{}", t.glyphs.overflow_left, self.hscroll.offset);
            let st = ctx.title_style(self.left_id(), bg);
            buf.set_string(grid_area.x + 1, header_y, &lbl, st);
            ctx.clickable(
                self.left_id(),
                Rect::new(
                    grid_area.x + 1,
                    header_y,
                    crate::ui::text::width(&lbl) as u16,
                    1,
                ),
            );
        }
        for k in 0..self.col_rects.len() {
            let rect = &self.col_rects[k];
            let ci = self.hscroll.offset + k;
            let col = &self.columns[ci];
            let hid = self.header_id(ci);
            let sorted = self.sort.map(|(c, _)| c == ci).unwrap_or(false);
            let filtered = self.filtered_cols.contains(&ci);
            let active = sorted || filtered || ci == self.cursor.1 && focused;
            let st = header_style(ctx, hid, bg, col.sortable, active);
            let mut suffix = String::new();
            if filtered {
                suffix.push(' ');
                suffix.push_str(t.glyphs.filter);
            }
            match self.sort {
                Some((c, SortDir::Asc)) if c == ci => {
                    suffix.push(' ');
                    suffix.push_str(t.glyphs.sort_asc);
                }
                Some((c, SortDir::Desc)) if c == ci => {
                    suffix.push(' ');
                    suffix.push_str(t.glyphs.sort_desc);
                }
                _ => {}
            }
            // the key glyph is drawn over the prefix's first cell, faint
            let prefix = if col.primary { "  " } else { "" };
            let avail = (rect.width as usize).saturating_sub(width(&suffix) + width(prefix));
            let shown = truncate(&col.name, avail.max(1));
            let cut = shown != col.name;
            let title = format!("{prefix}{shown}{suffix}");
            let text = if col.kind.right_aligned() {
                fit_right(&title, rect.width as usize)
            } else {
                fit(&title, rect.width as usize)
            };
            buf.set_string(rect.x, header_y, &text, st);
            if col.primary {
                let px = if col.kind.right_aligned() {
                    rect.right().saturating_sub(width(&title) as u16)
                } else {
                    rect.x
                };
                buf.set_string(px, header_y, t.glyphs.key, st.fg(t.text_faint));
            }
            let head = Rect::new(rect.x, header_y, rect.width, 1);
            if col.sortable {
                ctx.clickable(hid, head);
            } else if cut && !ctx.inert {
                // a cut title reveals on hover even when it does not sort
                ctx.hits.register_hover(hid, head);
            }
            if cut {
                crate::ui::reveal::truncated(ctx, self.reveal, hid, head, &col.name);
            }
            if self.sizing.resizable {
                let sid = self.seam_id(ci);
                let hit = self.sizing.seam_rect(rect.right(), header_y, 2, grid_area);
                let held = self.sizing.dragging() == Some(ci);
                crate::ui::columns::render_seam(buf, ctx, sid, hit, rect.right(), bg, held);
            }
        }
        let hidden_right = self
            .columns
            .len()
            .saturating_sub(self.hscroll.offset + self.hscroll.viewport_len);
        if self.overflow_badges && hidden_right > 0 {
            let lbl = format!("{hidden_right}{}", t.glyphs.overflow_right);
            let x = cols_area.right() + 1;
            let st = ctx.title_style(self.right_id(), bg);
            buf.set_string(x, header_y, &lbl, st);
            ctx.clickable(
                self.right_id(),
                Rect::new(x, header_y, crate::ui::text::width(&lbl) as u16, 1),
            );
        }

        if self.rows.is_empty() && !self.more {
            if self.loading {
                crate::widgets::progress::render_spinner(
                    Rect::new(body.x + 3, body.y + body.height / 2, body.width, 1),
                    buf,
                    ctx,
                    t.strings.loading_rows,
                    bg,
                );
            } else {
                crate::widgets::empty::render(body, buf, t, &self.empty, bg);
            }
            return;
        }

        // rows
        let edited_rows: std::collections::HashSet<usize> =
            self.pending.cells.keys().map(|&(r, _)| r).collect();
        let editing_here = self
            .edit
            .as_ref()
            .filter(|_| focused)
            .map(|e| (e.row, e.col));
        let choice_here = self.choice.as_ref().map(|c| (c.row, c.col));
        let mut choice_anchor: Option<Rect> = None;
        let mut open_marks: Vec<(WidgetId, Rect)> = Vec::new();
        // stands in for a cell error on a value outside its options
        let flagged_msg = String::new();
        for (i, d) in self.scroll.visible_range().enumerate() {
            let y = body.y + i as u16;
            let row_rect = Rect::new(
                grid_area.x,
                y,
                grid_area.width.saturating_sub(u16::from(has_sb)),
                1,
            );
            if d >= self.rows.len() {
                // fetch-more virtual row
                let mid = self.more_id();
                let mut s = ctx.child_state(self.id, mid, d == self.cursor.0);
                s.focused = focused && d == self.cursor.0;
                let st = t.row(s, bg);
                fill(buf, row_rect, st);
                buf.set_string(
                    row_rect.x,
                    y,
                    t.gutter_symbol(s),
                    t.gutter(s, st.bg.unwrap_or(bg), false),
                );
                let text = if self.loading {
                    format!(
                        "{} {}",
                        t.glyphs.spinner_frame(ctx.interaction.tick),
                        t.strings.fetching
                    )
                } else {
                    format!(
                        "{} {}",
                        t.glyphs.arrows[1],
                        crate::theme::Strings::fill(
                            t.strings.fetch_more,
                            &[&crate::ui::text::thousands(self.rows.len())]
                        )
                    )
                };
                buf.set_string(row_rect.x + gutter_w, y, &text, st.fg(t.text_muted));
                ctx.clickable(mid, row_rect);
                continue;
            }
            let src = self.order[d];
            let rid = self.rownum_id(d);
            let mut s = ctx.child_state(self.id, rid, d == self.cursor.0);
            let visible_cols = self.hscroll.offset..self.hscroll.offset + self.hscroll.viewport_len;
            let hovered_cell = visible_cols
                .clone()
                .find(|&c| ctx.interaction.hovered(self.cell_id(d, c)));
            // cell hover and press count as the row's
            s.hovered |= hovered_cell.is_some();
            s.pressed |= ctx
                .interaction
                .pressed_any(visible_cols.map(|c| self.cell_id(d, c)));
            s.focused = focused && d == self.cursor.0;
            s.selected = self.selected_rows.contains(&src);
            let state = self.row_state_with(src, |r| edited_rows.contains(&r));
            let mut row_style = t.row(s, bg);
            if state == RowState::Deleted {
                row_style = row_style
                    .fg(t.text_faint)
                    .add_modifier(Modifier::CROSSED_OUT);
            }
            fill(buf, row_rect, row_style);
            buf.set_string(
                row_rect.x,
                y,
                t.gutter_symbol(s),
                t.gutter(s, row_style.bg.unwrap_or(bg), false),
            );
            if s.selected {
                buf.set_string(
                    row_rect.x + 1,
                    y,
                    t.glyphs.tick,
                    row_style
                        .fg(if s.focused {
                            t.accent
                        } else {
                            t.text_secondary
                        })
                        .remove_modifier(Modifier::CROSSED_OUT),
                );
            }
            let (glyph, gs) = match state {
                RowState::Clean => (" ", row_style),
                RowState::Modified => (t.glyphs.dirty, row_style.fg(t.warning)),
                RowState::Inserted => ("+", row_style.fg(t.text_secondary)),
                RowState::Deleted => (t.glyphs.removed, row_style.fg(t.text_muted)),
                RowState::Error => ("!", row_style.fg(t.error).add_modifier(Modifier::BOLD)),
            };
            buf.set_string(
                row_rect.x + 2,
                y,
                glyph,
                gs.remove_modifier(Modifier::CROSSED_OUT),
            );
            if self.row_numbers {
                let n = fit_right(&(src + 1).to_string(), num_w as usize);
                let ns = row_style
                    .fg(if s.focused {
                        t.text_secondary
                    } else {
                        t.text_faint
                    })
                    .remove_modifier(Modifier::BOLD | Modifier::CROSSED_OUT);
                buf.set_string(row_rect.x + 3, y, &n, ns);
                ctx.clickable(rid, Rect::new(row_rect.x, y, gutter_w, 1));
            }
            for k in 0..self.col_rects.len() {
                let rect = &self.col_rects[k];
                let ci = self.hscroll.offset + k;
                let col = &self.columns[ci];
                let cell_rect = Rect::new(rect.x, y, rect.width, 1);
                let is_cursor = focused && (d, ci) == self.cursor;
                if editing_here == Some((src, ci)) {
                    let e = self.edit.as_ref().unwrap();
                    let es = t.field_style(VisualState {
                        editing: true,
                        ..Default::default()
                    });
                    fill(buf, cell_rect, es);
                    let cw = rect.width.saturating_sub(1) as usize;
                    let cur = e.buffer.cursor_pos().col;
                    let off = cur.saturating_sub(cw.saturating_sub(1));
                    let text = e.buffer.text();
                    let shown = crate::ui::text::slice_cells(text, off, cw);
                    let mut ts = es
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                    if e.error.is_some() {
                        ts = ts.underline_color(t.error);
                    }
                    buf.set_string(rect.x, y, &shown, ts);
                    if cw > 0 {
                        ctx.set_cursor(Position::new(rect.x + (cur - off) as u16, y));
                    }
                    if e.error.is_some() {
                        buf.set_string(
                            rect.right().saturating_sub(1),
                            y,
                            "!",
                            es.fg(t.error).add_modifier(Modifier::BOLD),
                        );
                    }
                } else {
                    let v = self.value(src, ci);
                    let dirty = self.pending.is_dirty(src, ci) && state != RowState::Inserted;
                    let cfg = &self.editor_config;
                    let in_choice = choice_here == Some((src, ci));
                    if in_choice {
                        choice_anchor = Some(cell_rect);
                    }
                    let is_cursor = is_cursor || in_choice;
                    // a constrained cell shows its option label; a value
                    // outside the options is flagged like an error
                    let flagged = cfg.flag_out_of_set && self.out_of_set(src, ci);
                    let err = self.cell_errors.get(&(src, ci));
                    let err = err.or(flagged.then_some(&flagged_msg));
                    let mut st = row_style;
                    let affordance = self.editable
                        && !col.read_only
                        && col.editor.has_popup()
                        && state != RowState::Deleted
                        && rect.width > 3
                        && match cfg.affordance {
                            Affordance::Never => false,
                            Affordance::Always => true,
                            Affordance::Focused => is_cursor || hovered_cell == Some(ci),
                        };
                    let text_w = if affordance {
                        rect.width - 2
                    } else {
                        rect.width
                    } as usize;
                    let text = match v {
                        CellValue::Text(raw) if col.editor.is_constrained() => {
                            truncate(&col.editor.display(raw, cfg), text_w)
                        }
                        _ => cell_text_with(v, col.kind, text_w, &t.glyphs),
                    };
                    // tone
                    match v {
                        CellValue::Null | CellValue::Default => {
                            st = st.fg(t.text_muted).add_modifier(Modifier::ITALIC)
                        }
                        CellValue::Text(s) if s.is_empty() => st = st.fg(t.text_faint),
                        _ => {}
                    }
                    if col.primary && !s.focused {
                        st = st.fg(t.text_secondary);
                    }
                    if self.in_range(d, ci) && !is_cursor {
                        st = st.patch(t.selection());
                    }
                    if dirty {
                        // changed values read in the warning tone; underline stays the editing token
                        st = st.fg(t.warning).remove_modifier(Modifier::ITALIC);
                    }
                    if err.is_some() {
                        st = st.fg(t.error).remove_modifier(Modifier::ITALIC);
                    }
                    if is_cursor {
                        st = t.cell_cursor(err.is_some(), s.pressed);
                        if state == RowState::Deleted {
                            st = st.add_modifier(Modifier::CROSSED_OUT);
                        }
                    } else if hovered_cell == Some(ci)
                        && self.editable
                        && !col.read_only
                        && state != RowState::Deleted
                    {
                        st = st
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(t.border_strong);
                    }
                    let shown = if col.kind.right_aligned() {
                        fit_right(&text, text_w)
                    } else {
                        fit(&text, text_w)
                    };
                    if affordance {
                        fill(buf, cell_rect, st);
                    }
                    buf.set_string(rect.x, y, &shown, st);
                    if affordance {
                        let mark = Rect::new(rect.right() - 1, y, 1, 1);
                        let ms = if is_cursor {
                            st
                        } else {
                            st.fg(t.text_secondary)
                        };
                        buf.set_string(mark.x, y, t.glyphs.disclosure_open, ms);
                        open_marks.push((self.open_id(d, ci), mark));
                    }
                    // a cut value (long text, JSON) shows in full on hover,
                    // with its real line breaks
                    let full = v.text_ref();
                    if text != *full {
                        crate::ui::reveal::truncated(
                            ctx,
                            self.reveal,
                            self.cell_id(d, ci),
                            Rect::new(rect.x, y, rect.width, 1),
                            &full,
                        );
                    }
                    if col.references.is_some()
                        && !matches!(v, CellValue::Null | CellValue::Default)
                        && rect.width > 6
                    {
                        buf.set_string(
                            rect.right().saturating_sub(1),
                            y,
                            t.glyphs.arrow,
                            st.fg(if is_cursor { t.canvas } else { t.text_muted }),
                        );
                    }
                    if err.is_some() && !is_cursor && !affordance {
                        buf.set_string(
                            rect.right().saturating_sub(1),
                            y,
                            "!",
                            st.add_modifier(Modifier::BOLD),
                        );
                    }
                }
                ctx.clickable(self.cell_id(d, ci), cell_rect);
            }
        }
        // a cell's `▾` sits above the cell: a click on it opens the editor
        for (id, rect) in open_marks {
            ctx.clickable(id, rect);
        }
        if let (Some(anchor), Some(c)) = (choice_anchor, self.choice.as_mut()) {
            c.editor.render(anchor, buf, ctx, &self.editor_config);
        }
        // the scroll region also sits above the rows: a wheel burst handled
        // before the next frame (see `runtime::Coalesce`) then routes to the
        // widget, not to a row whose display index it already scrolled away
        ctx.scrollable(self.id, area);
        self.track = Rect::ZERO;
        if has_sb {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    body.y,
                    (area.right() - 1).saturating_sub(area.x),
                    body.height,
                ),
                &self.scroll,
            );
            let sb = Rect::new(area.right() - 1, body.y, 1, body.height);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        self.htrack = Rect::ZERO;
        if has_hb {
            // under the scrolling columns only: the row-number gutter is
            // frozen, and the corner under the vertical bar stays blank
            let y = body.bottom();
            let right = area.right() - u16::from(has_sb);
            fill(
                buf,
                Rect::new(area.x, y, area.width, 1),
                Style::new().bg(bg),
            );
            let hb = Rect::new(cols_area.x, y, right.saturating_sub(cols_area.x), 1);
            self.htrack = hb;
            scrollbar::render_horizontal_with(
                hb,
                buf,
                ctx,
                self.id,
                &self.hscroll,
                focused,
                h_cfg.bar == ScrollbarVisibility::Always,
            );
        }

        // pending bar
        if bar_h > 0 {
            let by = area.bottom() - 1;
            let bar_area = Rect::new(area.x, by, area.width, 1);
            fill(buf, bar_area, Style::new().bg(bg));
            let (u, i, d) = self.pending.counts();
            let count = u + i + d;
            let text = format!("{} {count} pending", t.glyphs.dirty);
            buf.set_string(area.x + 1, by, &text, t.primary().fg(t.warning).bg(bg));
            // the row under the cursor explains its own rejection; otherwise the breakdown
            let cursor_src = self.order.get(self.cursor.0).copied();
            let dot = t.glyphs.separator.trim_start();
            let (detail, ds) = match cursor_src.and_then(|r| self.row_errors.get(&r)) {
                Some(msg) => (format!("{dot}{msg}"), t.error_fg().bg(bg)),
                None => (
                    self.pending_label()
                        .map(|d| format!("{dot}{d}"))
                        .unwrap_or_default(),
                    t.muted().bg(bg),
                ),
            };
            let bw: u16 = self.bar.iter().map(|b| b.width() + 1).sum();
            let room = area.width.saturating_sub(width(&text) as u16 + 4 + bw) as usize;
            buf.set_string(
                area.x + 2 + width(&text) as u16,
                by,
                truncate(&detail, room),
                ds,
            );
            let widths: Vec<u16> = self.bar.iter().map(|b| b.width()).collect();
            let rects = row_layout_right(
                Rect::new(area.x, by, area.width.saturating_sub(1), 1),
                &widths,
                1,
            );
            for (b, r) in self.bar.iter_mut().zip(rects) {
                b.render(r, buf, ctx, bg);
            }
        }
    }

    pub fn bar_ids(&self) -> [WidgetId; 3] {
        [self.bar[0].id, self.bar[1].id, self.bar[2].id]
    }

    /// Keyboard activation of a pending-bar button (they are focus stops).
    pub fn on_bar_key(&mut self, id: WidgetId, key: &Key) -> (Outcome, Option<GridEvent>) {
        for (i, b) in self.bar.iter_mut().enumerate() {
            if b.id == id {
                let (o, act) = b.on_key(key);
                if act {
                    let ev = match i {
                        0 => GridEvent::PreviewSql,
                        1 => GridEvent::DiscardRequested,
                        _ => GridEvent::CommitRequested,
                    };
                    return (o, Some(ev));
                }
                return (o, None);
            }
        }
        (Outcome::Ignored, None)
    }
}

fn cmp_cells(a: &CellValue, b: &CellValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let num = |v: &CellValue| match v {
        CellValue::Int(i) => Some(*i as f64),
        CellValue::Num(n) => Some(*n),
        _ => None,
    };
    match (a, b) {
        (CellValue::Null, CellValue::Null) => Ordering::Equal,
        (CellValue::Null, _) => Ordering::Greater,
        (_, CellValue::Null) => Ordering::Less,
        _ => match (num(a), num(b)) {
            (Some(x), Some(y)) => x.total_cmp(&y),
            _ => a.text().to_lowercase().cmp(&b.text().to_lowercase()),
        },
    }
}

/// Display text for a cell within `w` cells, with the default glyphs.
pub fn cell_text(v: &CellValue, kind: CellKind, w: usize) -> String {
    cell_text_with(v, kind, w, &crate::theme::Glyphs::UNICODE)
}

/// [`cell_text`] drawing its marks (cut JSON, line breaks, tabs, control
/// characters) with `g`.
pub fn cell_text_with(v: &CellValue, kind: CellKind, w: usize, g: &crate::theme::Glyphs) -> String {
    match v {
        CellValue::Null => "NULL".into(),
        CellValue::Default => "DEFAULT".into(),
        CellValue::Text(s) if s.is_empty() => "''".into(),
        CellValue::Json(j) => {
            if w < 8 {
                // the first non-blank character of the one-line form
                if j.trim_start().starts_with('[') {
                    format!("[{}]", g.ellipsis)
                } else {
                    format!("{{{}}}", g.ellipsis)
                }
            } else {
                truncate(&shown_prefix(w, |limit| one_line_json(j, limit)), w)
            }
        }
        CellValue::Text(s) => {
            if kind == CellKind::Id {
                // the middle cut keeps the end too: sanitise all of it
                truncate_middle(&sanitized(s, usize::MAX, g).0, w)
            } else {
                truncate(&shown_prefix(w, |limit| sanitized(s, limit, g)), w)
            }
        }
        other => truncate(&other.text_ref(), w),
    }
}

/// Characters of a text cell shown at most (control characters mapped to
/// visible marks); a longer value is cut there before it is measured.
const TEXT_CELL_CHARS: usize = 10_000;

/// The cell form of `s` (`↵`, `⇥`, `·` for line breaks, tabs and other
/// control characters; at most [`TEXT_CELL_CHARS`] characters), built
/// until it holds `limit` bytes. Returns it and whether it is complete.
fn sanitized(s: &str, limit: usize, g: &crate::theme::Glyphs) -> (String, bool) {
    let mut out = String::new();
    let mut chars = s.chars();
    for _ in 0..TEXT_CELL_CHARS {
        if out.len() >= limit {
            return (out, false);
        }
        let Some(c) = chars.next() else {
            return (out, true);
        };
        match c {
            '\n' => out.push_str(g.newline),
            '\t' => out.push_str(g.tab),
            c if c.is_control() => out.push_str(g.control),
            c => out.push(c),
        }
    }
    (out, true)
}

/// JSON on one line (whitespace runs collapsed to one space), built until
/// it holds `limit` bytes. Returns it and whether it is complete.
fn one_line_json(j: &str, limit: usize) -> (String, bool) {
    let mut out = String::new();
    for word in j.split_whitespace() {
        if out.len() >= limit {
            return (out, false);
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    (out, true)
}

/// Display text for `w` cells built from a prefix of a long value:
/// `make(limit)` returns at least `limit` bytes of the text (or all of it)
/// and whether that is all. The prefix grows until it is complete or wider
/// than `w` plus a margin, so truncating it to `w` gives what truncating
/// the whole text would, at a cost bounded by `w` instead of the value's
/// length (see [`wider_than`]).
fn shown_prefix(w: usize, make: impl Fn(usize) -> (String, bool)) -> String {
    let mut limit = w.saturating_mul(4).saturating_add(64);
    loop {
        let (text, complete) = make(limit);
        if complete || wider_than(&text, w.saturating_add(2)) {
            return text;
        }
        limit = limit.saturating_mul(2);
    }
}

impl Theme {
    /// Convenience for owners that render matching glyphs elsewhere.
    pub fn change_glyph(&self, state: RowState) -> (&'static str, Color) {
        match state {
            RowState::Clean => (" ", self.text_faint),
            RowState::Modified => (self.glyphs.dirty, self.warning),
            RowState::Inserted => ("+", self.text_secondary),
            RowState::Deleted => (self.glyphs.removed, self.text_muted),
            RowState::Error => ("!", self.error),
        }
    }
}

/// A column header: sortable headers walk the title ladder; the sorted,
/// filtered or cursor column reads in primary ink, the rest muted.
pub(crate) fn header_style(
    ctx: &RenderCtx,
    id: WidgetId,
    bg: Color,
    sortable: bool,
    active: bool,
) -> Style {
    let t = ctx.theme;
    let base = if sortable {
        ctx.title_style(id, bg)
    } else {
        Style::new().bg(bg)
    };
    let lifted = sortable && (ctx.interaction.hovered(id) || ctx.interaction.pressed(id));
    let ink = if active || lifted {
        t.text_primary
    } else {
        t.text_muted
    };
    let st = base.fg(t.legible(ink, base.bg.unwrap_or(bg)));
    // where muted ink must lift to stay readable (a card plane), weight
    // keeps the active column apart
    if active {
        st.add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        st
    }
}

impl crate::core::focus::BlurTarget for DataGrid {
    fn blur_id(&self) -> WidgetId {
        self.id
    }
    fn blur(&mut self) -> Outcome {
        self.on_blur().0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::ColorLevel;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn render_editor(grid: &mut DataGrid, level: ColorLevel) -> (Buffer, Position) {
        let area = Rect::new(0, 0, 80, 8);
        let mut buf = Buffer::empty(area);
        let theme = Theme::for_level(level);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(grid.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        grid.render(area, &mut buf, &mut ctx, theme.surface);
        let cursor = ctx.cursor.unwrap();
        (buf, cursor)
    }

    fn grid() -> DataGrid {
        let cols = vec![
            ColumnSpec::new("id", CellKind::Id).primary(),
            ColumnSpec::new("name", CellKind::Text),
            ColumnSpec::new("qty", CellKind::Number).nullable(false),
            ColumnSpec::new("active", CellKind::Bool),
        ];
        let mut g = DataGrid::new(WidgetId::of("g"), cols);
        let rows = (0..10)
            .map(|i| {
                vec![
                    CellValue::Text(format!("id-{i}")),
                    CellValue::Text(format!("row {i}")),
                    CellValue::Int(i),
                    CellValue::Bool(i % 2 == 0),
                ]
            })
            .collect();
        g.set_rows(GridRows {
            rows,
            total: RowTotal::Exact(10),
            more: false,
        });
        g.scroll.set_viewport(5);
        g
    }

    #[test]
    fn bar_labels_override_the_theme_and_default_labels_follow_it() {
        let de = crate::theme::Strings {
            preview_sql: "Vorschau",
            ..crate::theme::Strings::DEFAULT
        };
        let labels = |g: &DataGrid| g.bar.iter().map(|b| b.label.clone()).collect::<Vec<_>>();
        let mut g = grid().bar_labels(["SQL", "Undo all", "Write"]);
        g.follow_strings(de);
        assert_eq!(labels(&g), ["SQL", "Undo all", "Write"]);
        let mut g = grid();
        g.follow_strings(de);
        assert_eq!(labels(&g), ["Vorschau", "Discard", "Save"]);
        g.follow_strings(crate::theme::Strings::DEFAULT);
        assert_eq!(labels(&g), ["Preview SQL", "Discard", "Save"]);
    }

    #[test]
    fn rendering_without_focus_never_ends_the_cell_edit() {
        let mut g = grid();
        g.set_cursor(0, 1, false);
        g.begin_edit();
        g.edit.as_mut().unwrap().buffer.set_text("renamed");
        render_at(&mut g, Rect::new(0, 0, 80, 12), Interaction::default());
        assert!(g.is_editing(), "render is not a blur");
        assert!(
            g.pending.is_empty(),
            "nothing committed behind the owner's back"
        );
    }

    #[test]
    fn blur_commits_cancels_or_keeps_the_cell_edit() {
        let edited = |action| {
            let mut g = grid();
            g.blur_action = action;
            assert_eq!(g.on_blur(), (Outcome::Ignored, None));
            g.set_cursor(0, 1, false);
            g.begin_edit();
            g.edit.as_mut().unwrap().buffer.set_text("renamed");
            g
        };
        let mut commit = edited(BlurAction::Commit);
        assert_eq!(
            commit.on_blur(),
            (
                Outcome::Changed,
                Some(GridEvent::CellChanged { row: 0, col: 1 })
            )
        );
        assert!(!commit.is_editing());
        assert!(!commit.pending.is_empty());
        let mut cancel = edited(BlurAction::Cancel);
        assert_eq!(cancel.on_blur(), (Outcome::Changed, None));
        assert!(!cancel.is_editing());
        assert!(cancel.pending.is_empty());
        let mut keep = edited(BlurAction::Keep);
        assert_eq!(keep.on_blur(), (Outcome::Ignored, None));
        assert!(keep.is_editing());
        // an invalid value stays open with its error
        let mut bad = grid();
        bad.set_cursor(0, 2, false);
        bad.begin_edit();
        bad.edit.as_mut().unwrap().buffer.set_text("not a number");
        assert_eq!(bad.on_blur(), (Outcome::Changed, None));
        assert!(bad.edit_error().is_some());
    }

    #[test]
    fn clicking_another_cell_keeps_invalid_editor_and_cursor() {
        let mut g = grid();
        g.hscroll.set_viewport(g.columns.len());
        g.set_cursor(0, 2, false);
        g.begin_edit();
        g.edit.as_mut().unwrap().buffer.set_text("not a number");
        g.on_click(g.cell_id(1, 2), Position::ORIGIN);
        assert_eq!(g.cursor, (0, 2));
        assert_eq!(g.edit.as_ref().unwrap().buffer.text(), "not a number");
        assert!(g.edit_error().is_some());
        g.on_click(g.cell_id(1, 2), Position::ORIGIN);
        assert_eq!(g.edit.as_ref().unwrap().buffer.text(), "not a number");
        assert!(g.pending.is_empty());
    }

    #[test]
    fn invalid_edit_blocks_header_sort_request_and_preserves_draft() {
        let mut g = grid();
        g.set_cursor(0, 2, false);
        g.begin_edit();
        g.edit.as_mut().unwrap().buffer.set_text("not a number");
        let (_, event) = g.on_click(g.header_id(2), Position::ORIGIN);
        assert_eq!(event, None);
        assert_eq!(g.sort, None);
        assert_eq!(g.edit.as_ref().unwrap().buffer.text(), "not a number");
        assert!(g.edit_error().is_some());
    }

    #[test]
    fn unicode_edit_window_keeps_cells_bounded_and_clicks_at_display_position() {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let mut g = grid();
            g.set_cursor(0, 1, false);
            g.begin_edit();
            let (before, _) = render_editor(&mut g, level);
            g.edit.as_mut().unwrap().buffer.select_all();
            let text = format!("{}終", "日本語👩‍💻cafe\u{301}".repeat(8));
            g.on_paste(&text);
            let (after, cursor) = render_editor(&mut g, level);
            let rect = g.col_rects[1 - g.hscroll.offset];
            let cell = Rect::new(rect.x, cursor.y, rect.width, 1);
            assert!(cell.contains(cursor));
            assert_eq!(after[(cursor.x - 2, cursor.y)].symbol(), "終");
            let expected = g
                .edit
                .as_ref()
                .unwrap()
                .buffer
                .offset_at(0, crate::ui::text::width(&text) - 2);
            g.on_click(g.cell_id(0, 1), Position::new(cursor.x - 2, cursor.y));
            assert_eq!(g.edit.as_ref().unwrap().buffer.cursor_offset(), expected);
            g.on_key(&Key {
                code: KeyCode::Home,
                mods: ratatui::crossterm::event::KeyModifiers::NONE,
            });
            let (home, _) = render_editor(&mut g, level);
            for x in 0..80 {
                if !cell.contains(Position::new(x, cursor.y)) {
                    assert_eq!(
                        after[(x, cursor.y)].symbol(),
                        before[(x, cursor.y)].symbol()
                    );
                    assert_eq!(home[(x, cursor.y)].symbol(), before[(x, cursor.y)].symbol());
                }
            }
        }
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
    }

    fn wide_grid() -> DataGrid {
        let cols = (0..11)
            .map(|i| ColumnSpec::new(&format!("column_{i}"), CellKind::Text))
            .collect();
        let mut g = DataGrid::new(WidgetId::of("wide"), cols);
        let rows = (0..3)
            .map(|r| {
                (0..11)
                    .map(|c| CellValue::Text(format!("r{r}c{c}-9e3d2906-68f5")))
                    .collect()
            })
            .collect();
        g.set_rows(GridRows {
            rows,
            total: RowTotal::Exact(3),
            more: false,
        });
        g
    }

    fn render_at(g: &mut DataGrid, area: Rect, interaction: Interaction) -> (Buffer, HitRegistry) {
        let mut buf = Buffer::empty(area);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        {
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            g.render(area, &mut buf, &mut ctx, theme.surface);
        }
        (buf, hits)
    }

    fn row_text(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    }

    /// The thumb's first column on the grid's horizontal bar.
    fn thumb_x(buf: &Buffer, y: u16) -> Option<u16> {
        (0..buf.area.width).find(|&x| buf[(x, y)].symbol() == scrollbar::H_THUMB)
    }

    /// The first cell on header row `y` whose hit is `id`.
    fn hit_x(hits: &HitRegistry, y: u16, w: u16, id: WidgetId) -> Option<u16> {
        (0..w).find(|&x| hits.hit(Position::new(x, y)) == Some(id))
    }

    #[test]
    fn dragging_a_header_seam_resizes_the_column_cell_by_cell() {
        let mut g = wide_grid();
        let area = Rect::new(0, 0, 100, 8);
        let (_, hits) = render_at(&mut g, area, Interaction::default());
        let before = g.col_rects[0].width;
        let sid = g.seam_id(0);
        // the seam sits in the gap right after the first column
        let x = hit_x(&hits, area.y, area.width, sid).expect("a seam after column 0");
        assert_eq!(x, g.col_rects[0].right());
        assert_eq!(
            g.on_press(sid, Position::new(x + 1, 0), 1),
            Outcome::Changed
        );
        // the first move does not jump: the grab offset is kept
        g.on_drag(sid, Position::new(x + 1, 0));
        render_at(&mut g, area, Interaction::default());
        assert_eq!(g.col_rects[0].width, before);
        g.on_drag(sid, Position::new(x + 6, 0));
        let (buf, _) = render_at(&mut g, area, Interaction::default());
        assert_eq!(g.col_rects[0].width, before + 5);
        assert_eq!(g.col_rects[1].x, x + 5 + 2, "the next column follows");
        // held, the seam shows the heavy rule; the release ends the drag
        let glyphs = Theme::junie().glyphs;
        assert_eq!(buf[(x + 5, 0)].symbol(), glyphs.scroll_thumb);
        g.on_click(sid, Position::new(x + 6, 0));
        assert_eq!(g.on_drag(sid, Position::new(x + 9, 0)), Outcome::Consumed);
        let (rest, _) = render_at(&mut g, area, Interaction::default());
        let hover = Interaction {
            hover: Some(sid),
            ..Default::default()
        };
        let (hovered, _) = render_at(&mut g, area, hover);
        assert_eq!(rest[(x + 5, 0)].symbol(), " ", "invisible at rest");
        assert_eq!(hovered[(x + 5, 0)].symbol(), glyphs.scroll_track);
        // off: no seam is registered
        let mut g = wide_grid().resizable_columns(false);
        let (_, hits) = render_at(&mut g, area, Interaction::default());
        assert_eq!(hit_x(&hits, 0, area.width, g.seam_id(0)), None);
    }

    #[test]
    fn double_clicking_a_seam_autofits_and_the_header_menu_resets() {
        let cols = vec![
            ColumnSpec::new("id", CellKind::Number),
            ColumnSpec::new("name", CellKind::Text),
        ];
        let mut g = DataGrid::new(WidgetId::of("fit"), cols);
        g.set_rows(GridRows {
            rows: vec![vec![CellValue::Int(1), CellValue::Text("Ada".into())]],
            total: RowTotal::Exact(1),
            more: false,
        });
        let area = Rect::new(0, 0, 60, 6);
        render_at(&mut g, area, Interaction::default());
        let sid = g.seam_id(1);
        let w0 = g.col_rects[1].width;
        g.on_press(sid, Position::new(0, 0), 1);
        g.on_drag(sid, Position::new(20, 0));
        render_at(&mut g, area, Interaction::default());
        assert_eq!(g.col_rects[1].width, w0 + 20, "dragged wider");
        // a double click on the seam fits the header and the values
        g.on_press(sid, Position::new(20, 0), 2);
        render_at(&mut g, area, Interaction::default());
        assert_eq!(g.col_rects[1].width, 4, "\"name\" is the widest text");
        // the header menu sorts, autofits and resets
        let menu = g.header_menu(1);
        let labels: Vec<_> = menu.iter().map(|(_, m)| m.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Sort Ascending",
                "Sort Descending",
                "Clear Sort",
                "Size Column to Fit",
                "Reset Column Widths"
            ]
        );
        assert!(menu[2].1.disabled, "nothing to clear");
        assert!(!menu[4].1.disabled, "a user width exists");
        g.on_header_action(1, HeaderAction::ResetWidths);
        assert!(g.sizing.is_default());
        g.on_header_action(1, HeaderAction::SortDesc);
        assert_eq!(g.sort, Some((1, SortDir::Desc)));
        assert_eq!(g.locate_header_menu(g.header_id(1)), Some(1));
        assert_eq!(g.locate_header_menu(sid), Some(1));
    }

    #[test]
    fn a_cut_header_title_reveals_on_hover() {
        let long = "customer_lifetime_value_in_reporting_currency";
        let cols = vec![ColumnSpec::new(long, CellKind::Number)];
        let mut g = DataGrid::new(WidgetId::of("cut"), cols);
        g.set_rows(GridRows {
            rows: vec![vec![CellValue::Int(1)]],
            total: RowTotal::Exact(1),
            more: false,
        });
        let area = Rect::new(0, 0, 80, 8);
        let (buf, _) = render_at(&mut g, area, Interaction::default());
        assert!(!row_text(&buf, 0).contains(long), "the title is cut");
        let theme = Theme::junie();
        let mut buf = Buffer::empty(area);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let hover = Interaction {
            hover: Some(g.header_id(0)),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
        g.render(area, &mut buf, &mut ctx, theme.surface);
        ctx.flush_overlays(&mut buf);
        let all: String = (0..area.height).map(|y| row_text(&buf, y)).collect();
        assert!(all.contains(long), "{all}");
    }

    #[test]
    fn dragging_the_vertical_thumb_keeps_the_grabbed_row_under_the_pointer() {
        let mut g = grid();
        let area = Rect::new(0, 0, 60, 9);
        let (_, hits) = render_at(&mut g, area, Interaction::default());
        let bar = scrollbar::id_for(g.id);
        let track = hits.area_of(bar).expect("vertical bar");
        let (start, len) = g.scroll.thumb(track.height as usize);
        assert!(len >= 2 && start == 0, "{start} {len}");
        // grab the thumb on its second row, then pull it down two rows: a
        // drag is a drag, not a fresh press that would leave it in place
        g.on_drag(bar, Position::new(track.x, track.y + 1));
        g.on_drag(bar, Position::new(track.x, track.y + 3));
        let (moved, _) = g.scroll.thumb(track.height as usize);
        assert_eq!(moved, 2, "offset {}", g.scroll.offset);
    }

    #[test]
    fn wide_columns_show_a_horizontal_bar_that_follows_wheel_and_drag() {
        let mut g = wide_grid();
        let area = Rect::new(0, 0, 60, 8);
        let (buf, hits) = render_at(&mut g, area, Interaction::default());
        assert!(g.hscroll.overflows());
        let y = area.bottom() - 1;
        let bar = row_text(&buf, y);
        assert!(
            bar.contains(scrollbar::H_THUMB),
            "bar on the bottom row: {bar:?}"
        );
        assert!(bar.contains(scrollbar::H_TRACK));
        assert_eq!(
            thumb_x(&buf, y),
            Some(g.htrack.x),
            "the thumb starts at the left"
        );
        assert_eq!(
            hits.hit(Position::new(g.htrack.x + 1, y)),
            Some(scrollbar::h_id_for(g.id))
        );
        assert!(g.owns(scrollbar::h_id_for(g.id)));
        assert!(
            !row_text(&buf, 1).contains(scrollbar::H_THUMB),
            "rows are not drawn over"
        );

        // a wheel to the right moves one column and the thumb with it
        assert_eq!(g.on_wheel(3, true), Outcome::Changed);
        assert_eq!(g.hscroll.offset, 1);
        let (buf, _) = render_at(&mut g, area, Interaction::default());
        assert!(thumb_x(&buf, y) > Some(g.htrack.x));

        // dragging the thumb to the right end shows the last columns
        let grab = Position::new(thumb_x(&buf, y).unwrap(), y);
        let (out, _) = g.on_click(scrollbar::h_id_for(g.id), grab);
        assert_eq!(out, Outcome::Consumed, "pressing the thumb only grabs it");
        assert_eq!(
            g.on_drag(scrollbar::h_id_for(g.id), Position::new(area.right(), y)),
            Outcome::Changed
        );
        assert_eq!(g.hscroll.offset, g.hscroll.max_offset());
        let (buf, _) = render_at(&mut g, area, Interaction::default());
        assert!(
            row_text(&buf, 0).contains("column_10"),
            "last column in view"
        );
        assert_eq!(
            g.cols_label().as_deref().map(|l| l.ends_with("of 11")),
            Some(true)
        );
    }

    #[test]
    fn horizontal_policy_and_badges_are_configurable() {
        let area = Rect::new(0, 0, 60, 8);
        let mut g = wide_grid()
            .horizontal(HScroll::default().bar(ScrollbarVisibility::Never))
            .overflow_badges(false);
        let (buf, hits) = render_at(&mut g, area, Interaction::default());
        assert!(!row_text(&buf, area.bottom() - 1).contains(scrollbar::H_THUMB));
        assert_eq!(hits.area_of(scrollbar::h_id_for(g.id)), None);
        assert!(!row_text(&buf, 0).contains('›'), "no hidden-column badge");
        assert_eq!(g.on_wheel(1, true), Outcome::Changed, "still scrolls");

        let mut off = wide_grid().horizontal(HScroll::off());
        render_at(&mut off, area, Interaction::default());
        assert_eq!(off.on_wheel(1, true), Outcome::Ignored);

        // content that fits draws no bar by default
        let mut fits = grid();
        let (buf, _) = render_at(&mut fits, Rect::new(0, 0, 80, 8), Interaction::default());
        assert!(!row_text(&buf, 7).contains(scrollbar::H_THUMB));
    }

    #[test]
    fn dirty_back_to_original_clears_change() {
        let mut g = grid();
        g.record_cell(1, 1, CellValue::Text("x".into()));
        assert_eq!(g.pending.total(), 1);
        assert_eq!(g.row_state(1), RowState::Modified);
        g.record_cell(1, 1, CellValue::Text("row 1".into()));
        assert!(g.pending.is_empty());
        assert_eq!(g.row_state(1), RowState::Clean);
    }

    #[test]
    fn delete_removes_update_and_undo_restores() {
        let mut g = grid();
        g.record_cell(2, 1, CellValue::Text("x".into()));
        g.toggle_delete(2);
        assert_eq!(g.row_state(2), RowState::Deleted);
        assert!(!g.pending.is_dirty(2, 1));
        g.undo();
        assert_eq!(g.row_state(2), RowState::Clean);
    }

    #[test]
    fn insert_then_undo_shifts_nothing_else() {
        let mut g = grid();
        g.insert_row();
        assert_eq!(g.len(), 11);
        assert_eq!(g.row_state(10), RowState::Inserted);
        assert_eq!(g.cursor, (10, 1));
        g.undo();
        assert_eq!(g.len(), 10);
        assert!(g.pending.is_empty());
    }

    #[test]
    fn edit_commit_validates_by_kind() {
        let mut g = grid();
        g.cursor = (0, 2);
        g.begin_edit();
        assert!(g.is_editing());
        g.edit.as_mut().unwrap().buffer.insert_str("abc");
        assert!(g.commit_edit().is_none());
        assert_eq!(g.edit_error(), Some("Must be a number"));
        g.cancel_edit();
        g.cursor = (0, 3);
        g.begin_edit(); // bool cycles without an editor
        assert!(!g.is_editing());
        assert_eq!(g.value(0, 3), &CellValue::Bool(false));
    }

    #[test]
    fn a_validator_can_capture_state() {
        let allowed = ["red".to_owned(), "blue".to_owned()];
        let mut g = grid().validator(move |_, text| {
            if allowed.iter().any(|a| a == text) {
                Ok(CellValue::Text(text.to_owned()))
            } else {
                Err(format!("One of {}", allowed.join(", ")))
            }
        });
        g.cursor = (0, 2);
        g.begin_edit();
        g.edit.as_mut().unwrap().buffer.insert_str("green");
        assert!(g.commit_edit().is_none());
        assert_eq!(g.edit_error(), Some("One of red, blue"));
    }

    #[test]
    fn duplicate_keys_are_configurable() {
        let ctrl_d = Key {
            code: KeyCode::Char('d'),
            mods: KeyModifiers::CONTROL,
        };
        let alt_d = Key {
            code: KeyCode::Char('d'),
            mods: KeyModifiers::ALT,
        };
        // an app that gives Ctrl+D another meaning keeps only Alt+D
        let mut g = grid().editable(true).duplicate_keys(&[alt_d]);
        assert_eq!(g.on_key(&ctrl_d).1, None);
        assert_eq!(g.rows().len(), 10);
        assert_eq!(g.on_key(&alt_d).1, Some(GridEvent::RowInserted(10)));
        // an empty list turns the shortcut off
        let mut g = grid().editable(true).duplicate_keys(&[]);
        assert_eq!(g.on_key(&alt_d).1, None);
        assert_eq!(g.rows().len(), 10);
    }

    #[test]
    fn enter_on_a_read_only_cell_says_why_and_hints_drop_edit() {
        let mut g = grid().editable(true);
        g.columns[0].read_only = true;
        assert!(!g.cell_editable(), "the cursor starts on the read-only id");
        let (o, ev) = g.on_key(&key(KeyCode::Enter));
        assert_eq!(o, Outcome::Changed);
        assert_eq!(
            ev,
            Some(GridEvent::ReadOnly {
                reason: "id is read-only".into()
            })
        );
        assert!(!g.is_editing());
        g.on_key(&key(KeyCode::Right));
        assert!(g.cell_editable());
        // a grid that is not editable and says why reports that reason
        let mut ro = grid();
        ro.editable = false;
        ro.read_only_reason = Some("Views cannot be edited".into());
        assert!(!ro.cell_editable());
        let (_, ev) = ro.on_key(&key(KeyCode::Enter));
        assert_eq!(
            ev,
            Some(GridEvent::ReadOnly {
                reason: "Views cannot be edited".into()
            })
        );
        // without a reason it stays a list: Enter activates the row
        ro.read_only_reason = None;
        let (_, ev) = ro.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(GridEvent::Activated(0)));
    }

    #[test]
    fn duplicate_chords_share_defaults_undo_and_read_only_guards() {
        for mods in [KeyModifiers::ALT, KeyModifiers::CONTROL] {
            let mut g = grid().editable(true);
            let duplicate = Key {
                code: KeyCode::Char('d'),
                mods,
            };
            assert_eq!(g.on_key(&duplicate).1, Some(GridEvent::RowInserted(10)));
            assert_eq!(g.value(10, 0), &CellValue::Default);
            assert_eq!(g.value(10, 1), &CellValue::Text("row 0".into()));
            assert!(g.undo());
            assert_eq!(g.rows().len(), 10);
            g.editable = false;
            assert_eq!(g.on_key(&duplicate).1, None);
            assert_eq!(g.rows().len(), 10);
            assert_eq!(g.pending.total(), 0);
        }
    }

    #[test]
    fn keys_navigate_select_and_sort_request() {
        let mut g = grid();
        g.on_key(&key(KeyCode::Down));
        g.on_key(&key(KeyCode::Right));
        assert_eq!(g.cursor, (1, 1));
        g.on_key(&key(KeyCode::Char(' ')));
        assert!(g.selected_rows.contains(&1));
        let (_, ev) = g.on_key(&key(KeyCode::Char('s')));
        assert_eq!(ev, Some(GridEvent::SortRequested(Some((1, SortDir::Asc)))));
        let (_, ev) = g.on_key(&key(KeyCode::Char('s')));
        assert_eq!(ev, Some(GridEvent::SortRequested(Some((1, SortDir::Desc)))));
        let (_, ev) = g.on_key(&key(KeyCode::Char('s')));
        assert_eq!(ev, Some(GridEvent::SortRequested(None)));
        g.local_sort = true;
        g.on_key(&key(KeyCode::Char('s')));
        g.on_key(&key(KeyCode::Char('s'))); // desc by name
        assert_eq!(g.source_row(0), 9);
        // dirty key survives sorting
        g.record_cell(9, 1, CellValue::Text("z".into()));
        assert!(g.pending.is_dirty(9, 1));
        assert_eq!(g.row_state(g.source_row(0)), RowState::Modified);
    }

    #[test]
    fn range_selection_and_copy() {
        let mut g = grid();
        g.on_key(&Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::SHIFT,
        });
        g.on_key(&Key {
            code: KeyCode::Right,
            mods: ratatui::crossterm::event::KeyModifiers::SHIFT,
        });
        assert!(g.in_range(0, 0) && g.in_range(1, 1));
        let text = g.copy_text(false);
        assert_eq!(text.lines().count(), 2);
        assert!(text.starts_with("id-0\trow 0"));
    }

    #[test]
    fn position_label_variants() {
        let mut g = grid();
        assert_eq!(g.position_label(), "rows 1–5 of 10");
        g.more = true;
        g.total = RowTotal::Estimated(1_203_338);
        assert_eq!(
            g.position_label(),
            "rows 1–5 of 10 loaded · ~1,203,338 total"
        );
        g.set_rows(GridRows {
            rows: vec![],
            total: RowTotal::Exact(0),
            more: false,
        });
        assert_eq!(g.position_label(), "0 rows");
    }

    #[test]
    fn fetch_more_row_is_reachable() {
        let mut g = grid();
        g.more = true;
        g.scroll.set_content(g.content_rows());
        g.on_key(&key(KeyCode::Char('G')));
        assert!(g.on_more_row());
        let (_, ev) = g.on_key(&key(KeyCode::Enter));
        assert_eq!(ev, Some(GridEvent::FetchMore));
    }

    #[test]
    fn commit_result_folds_and_drops() {
        let mut g = grid();
        g.record_cell(0, 1, CellValue::Text("new".into()));
        g.toggle_delete(3);
        g.apply_commit_result(Ok(()));
        assert_eq!(g.len(), 9);
        assert_eq!(g.value(0, 1), &CellValue::Text("new".into()));
        assert!(g.pending.is_empty());
        g.record_cell(0, 1, CellValue::Text("bad".into()));
        g.apply_commit_result(Err((0, "boom".into())));
        assert_eq!(g.row_state(0), RowState::Error);
    }

    /// `cell_text` before bounded measurement, as the reference.
    fn cell_text_reference(v: &CellValue, kind: CellKind, w: usize) -> String {
        match v {
            CellValue::Null => "NULL".into(),
            CellValue::Default => "DEFAULT".into(),
            CellValue::Text(s) if s.is_empty() => "''".into(),
            CellValue::Json(j) => {
                let one: String = j.split_whitespace().collect::<Vec<_>>().join(" ");
                if w < 8 {
                    if one.starts_with('[') {
                        "[…]".into()
                    } else {
                        "{…}".into()
                    }
                } else {
                    truncate(&one, w)
                }
            }
            CellValue::Text(s) => {
                let sanitized: String = s
                    .chars()
                    .take(10_000)
                    .map(|c| match c {
                        '\n' => '↵',
                        '\t' => '⇥',
                        c if c.is_control() => '·',
                        c => c,
                    })
                    .collect();
                if kind == CellKind::Id {
                    truncate_middle(&sanitized, w)
                } else {
                    truncate(&sanitized, w)
                }
            }
            other => truncate(&other.text(), w),
        }
    }

    #[test]
    fn cell_text_from_a_prefix_matches_the_whole_value() {
        let mut values = vec![
            CellValue::Null,
            CellValue::Default,
            CellValue::Int(-42),
            CellValue::Num(2.5049),
            CellValue::Bool(true),
            CellValue::Text(String::new()),
            CellValue::Json("  [ 1,\n 2 ]".into()),
            CellValue::Json("\n\t{\"a\":   {\"b\": [1, 2, 3]}}\n".into()),
        ];
        for seed in [
            "plain words",
            "multi\nline\ttext\u{7}",
            "日本語のテキスト",
            "👩\u{200D}👩\u{200D}👧 ⌚\u{FE0E}",
            "e\u{301}",
        ] {
            for n in [1, 9, 3_000] {
                values.push(CellValue::Text(seed.repeat(n)));
                values.push(CellValue::Json(format!(
                    "{{\"k\": \"{}\"}}",
                    seed.repeat(n)
                )));
            }
        }
        // longer than the 10,000-character cap of a text cell
        values.push(CellValue::Text("ab\n".repeat(6_000)));
        for v in &values {
            for kind in [CellKind::Text, CellKind::Id, CellKind::Json] {
                for w in (0..40).chain([80, 400]) {
                    assert_eq!(
                        cell_text(v, kind, w),
                        cell_text_reference(v, kind, w),
                        "{kind:?} {w} {v:?}"
                    );
                }
            }
        }
    }

    /// Guard (P2): a cut cell reads a prefix bounded by its width, however
    /// long the value is.
    #[test]
    fn cell_text_reads_a_prefix_bounded_by_the_cell_width() {
        let long = "word ".repeat(400_000);
        let asked = std::cell::Cell::new(0);
        let shown = shown_prefix(40, |limit| {
            asked.set(asked.get().max(limit));
            sanitized(&long, limit, &crate::theme::Glyphs::UNICODE)
        });
        assert!(asked.get() <= 4 * 40 + 64, "read {} bytes", asked.get());
        assert_eq!(truncate(&shown, 40), truncate(&long, 40));
        let json = format!("[{}]", "1, ".repeat(400_000));
        asked.set(0);
        shown_prefix(40, |limit| {
            asked.set(asked.get().max(limit));
            one_line_json(&json, limit)
        });
        assert!(asked.get() <= 4 * 40 + 64, "read {} bytes", asked.get());
    }

    #[test]
    fn width_sampling_is_configurable_and_reads_only_the_sample() {
        let cols = vec![ColumnSpec::new("note", CellKind::Text)];
        let rows = |n: usize, wide_from: usize| GridRows {
            rows: (0..n)
                .map(|r| {
                    vec![CellValue::Text(if r >= wide_from {
                        "x".repeat(70_000)
                    } else {
                        "short".into()
                    })]
                })
                .collect(),
            total: RowTotal::Exact(n),
            more: false,
        };
        let mut g = DataGrid::new(WidgetId::of("s"), cols.clone());
        assert_eq!(g.width_sample_rows, DEFAULT_WIDTH_SAMPLE_ROWS);
        g.set_rows(rows(400, 200));
        let narrow = g.widths[0];
        let mut g = DataGrid::new(WidgetId::of("s"), cols).width_sample_rows(400);
        g.set_rows(rows(400, 200));
        assert!(
            g.widths[0] > narrow,
            "the wider sample sees the long values"
        );
        // the long values clamp to the column max
        assert_eq!(g.widths[0], g.columns[0].max_width.max(8));
    }

    #[test]
    fn hovering_a_cut_cell_reveals_the_whole_value() {
        let long = "a note far longer than any text column is wide by default";
        let mut g = grid();
        let mut rows: Vec<Vec<CellValue>> = (0..3)
            .map(|i| {
                vec![
                    CellValue::Text(format!("id-{i}")),
                    CellValue::Text(format!("row {i}")),
                    CellValue::Int(i),
                    CellValue::Bool(true),
                ]
            })
            .collect();
        rows[0][1] = CellValue::Text(long.into());
        g.set_rows(GridRows {
            rows,
            total: RowTotal::Exact(3),
            more: false,
        });
        let theme = Theme::junie();
        let mut draw = |hover| {
            let mut buf = Buffer::empty(Rect::new(0, 0, 100, 8));
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            g.render(Rect::new(0, 0, 60, 8), &mut buf, &mut ctx, theme.surface);
            ctx.flush_overlays(&mut buf);
            (0..8)
                .map(|y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(!draw(None).contains(long));
        let shown = draw(Some(grid().cell_id(0, 1)));
        assert!(shown.contains(long), "{shown}");
        assert!(!draw(Some(grid().cell_id(1, 1))).contains(long));
    }

    /// The interaction of a press held on `id`.
    fn holding(id: WidgetId) -> Interaction {
        Interaction {
            pressed: Some(id),
            ..Interaction::default()
        }
    }

    #[test]
    fn a_range_drag_held_past_the_bottom_edge_keeps_scrolling_and_extending() {
        let mut g = grid();
        let area = Rect::new(0, 0, 60, 6);
        render_at(&mut g, area, Interaction::default());
        let origin = g.cell_id(0, 1);
        g.on_click(origin, Position::new(g.col_rects[1].x, g.body.y));
        let (_, hits) = render_at(&mut g, area, holding(origin));
        assert_eq!(
            hits.autoscroll_at(Position::new(g.body.x + 8, g.body.y)),
            Some(g.body),
            "the rows are the autoscroll body"
        );
        // the pointer rests one row below the body: every replayed drag
        // (the app's autoscroll tick) moves on by a row, with no motion
        let below = Position::new(g.col_rects[1].x, g.body.bottom());
        let mut last = g.cursor.0;
        for _ in 0..5 {
            assert_eq!(g.on_drag(origin, below), Outcome::Changed);
            render_at(&mut g, area, holding(origin));
            assert!(g.cursor.0 > last, "cursor {:?}", g.cursor);
            last = g.cursor.0;
        }
        assert!(g.scroll.offset > 0, "the rows scrolled");
        assert_eq!(g.anchor, Some((0, 1)), "the range keeps its anchor");
        assert_eq!(g.cursor.1, 1);
        // the press ends: a later stray drag from the origin id is no drag
        render_at(&mut g, area, Interaction::default());
        assert_eq!(g.on_drag(origin, below), Outcome::Ignored);
    }

    #[test]
    fn a_drag_from_a_row_number_paints_the_marks_and_autoscrolls() {
        let mut g = grid();
        let area = Rect::new(0, 0, 60, 6);
        render_at(&mut g, area, Interaction::default());
        let origin = g.rownum_id(0);
        render_at(&mut g, area, holding(origin));
        let gutter_x = g.body.x + 1;
        assert_eq!(
            g.on_drag(origin, Position::new(gutter_x, g.body.y + 2)),
            Outcome::Changed
        );
        assert_eq!(
            g.selected_rows.iter().copied().collect::<Vec<_>>(),
            [0, 1, 2]
        );
        // back on the origin and released there: the click keeps the
        // painted marks instead of toggling the origin off
        g.on_drag(origin, Position::new(gutter_x, g.body.y));
        let (out, _) = g.on_click(origin, Position::new(gutter_x, g.body.y));
        assert_eq!(out, Outcome::Changed);
        assert!(g.selected_rows.contains(&0), "{:?}", g.selected_rows);
        // a fresh press and drag held below the body marks rows as they
        // scroll in
        render_at(&mut g, area, Interaction::default());
        let origin = g.rownum_id(3);
        render_at(&mut g, area, holding(origin));
        for _ in 0..4 {
            g.on_drag(origin, Position::new(gutter_x, g.body.bottom()));
            render_at(&mut g, area, holding(origin));
        }
        assert!(g.scroll.offset > 0);
        assert!(
            (3..8).all(|r| g.selected_rows.contains(&r)),
            "{:?}",
            g.selected_rows
        );
    }
}

#[cfg(test)]
#[path = "grid_choice_tests.rs"]
mod choice_tests;

/// Select All marks every row of a grid, as Space marks one.
#[cfg(test)]
mod select_all_tests {
    use super::*;

    #[test]
    fn cmd_or_ctrl_a_marks_every_row() {
        use ratatui::crossterm::event::KeyModifiers;
        for mods in [KeyModifiers::SUPER, KeyModifiers::CONTROL] {
            let mut g = DataGrid::new(
                WidgetId::of("g"),
                vec![ColumnSpec::new("name", CellKind::Text)],
            )
            .type_ahead(true);
            g.set_rows(GridRows {
                rows: (0..4)
                    .map(|i| vec![CellValue::Text(format!("a{i}"))])
                    .collect(),
                total: RowTotal::Exact(4),
                more: false,
            });
            let (out, _) = g.on_key(&Key {
                code: KeyCode::Char('a'),
                mods,
            });
            assert_eq!(out, Outcome::Changed);
            assert_eq!(g.selected_rows.len(), 4);
            assert_eq!(g.cursor.0, 0, "never a type-ahead `a`");
        }
    }
}

/// Cmd+C copies the cell, range or marked rows, as `y` does; the hint
/// names it only once the terminal delivers Cmd.
#[cfg(test)]
mod cmd_copy_tests {
    use super::*;
    use crate::core::keymap::hint_key;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn cmd_c_copies_the_marked_rows_and_the_hint_falls_back_to_y() {
        let mut g = DataGrid::new(
            WidgetId::of("g"),
            vec![ColumnSpec::new("name", CellKind::Text)],
        );
        g.set_rows(GridRows {
            rows: (0..3)
                .map(|i| vec![CellValue::Text(format!("a{i}"))])
                .collect(),
            total: RowTotal::Exact(3),
            more: false,
        });
        let cmd = |c| Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::SUPER,
        };
        g.on_key(&cmd('a'));
        let (out, ev) = g.on_key(&cmd('c'));
        assert_eq!(out, Outcome::Changed);
        match ev {
            Some(GridEvent::Copy(text)) => assert!(text.contains("a2"), "{text}"),
            other => panic!("Cmd+C copies: {other:?}"),
        }
        assert_eq!(hint_key(&g.keymap, GridAction::Copy), Some("y"));
        crate::core::keyboard::set_super_delivered(true);
        let label = KeyBinding::cmd('c').label();
        assert_eq!(hint_key(&g.keymap, GridAction::Copy), Some(label.as_str()));
    }
}
