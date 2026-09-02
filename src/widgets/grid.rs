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
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::core::text::TextBuffer;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, VisualState, fill};
use crate::ui::text::{fit, fit_right, truncate, truncate_middle, width};
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::field_common::{EditAction, edit_key};
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
        match self {
            CellValue::Null => "NULL".into(),
            CellValue::Default => "DEFAULT".into(),
            CellValue::Text(s) | CellValue::Json(s) => s.clone(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Num(n) => format!("{n:.2}"),
            CellValue::Bool(b) => b.to_string(),
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
        }
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
    CellChanged { row: usize, col: usize },
    RowInserted(usize),
    RowDeleted(usize),
    SortRequested(Option<(usize, SortDir)>),
    FetchMore,
    Refresh,
    CommitRequested,
    DiscardRequested,
    PreviewSql,
    Copy(String),
    FollowReference { row: usize, col: usize },
    OpenViewer { row: usize, col: usize },
    FilterOnCell { col: usize, value: CellValue },
    OpenFilters,
    ClearFilters,
    Activated(usize),
    LeaveForward,
    LeaveBackward,
}

/// Validate text typed into a cell for a column. The owner can replace
/// this with engine-aware parsing.
pub type Validator = fn(&ColumnSpec, &str) -> Result<CellValue, String>;

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

#[derive(Debug, Clone)]
pub struct DataGrid {
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
    pub read_only_reason: Option<String>,
    pub loading: bool,
    pub cell_errors: HashMap<(usize, usize), String>,
    pub row_errors: HashMap<usize, String>,
    pub empty: crate::widgets::empty::EmptyState,
    pub validator: Validator,
    pub row_numbers: bool,
    pub area: Rect,
    body: Rect,
    widths: Vec<u16>,
    col_rects: Vec<Rect>,
    show_bar: bool,
    bar: [Button; 3],
}

const HEADER_SORT_ASC: &str = " ▴";
const HEADER_SORT_DESC: &str = " ▾";

impl DataGrid {
    pub fn new(id: WidgetId, columns: Vec<ColumnSpec>) -> Self {
        let widths = columns.iter().map(|c| c.min_width).collect();
        Self {
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
            read_only_reason: None,
            loading: false,
            cell_errors: HashMap::new(),
            row_errors: HashMap::new(),
            empty: crate::widgets::empty::EmptyState::new("No rows"),
            validator: default_validator,
            row_numbers: true,
            area: Rect::ZERO,
            body: Rect::ZERO,
            widths,
            col_rects: vec![],
            show_bar: true,
            bar: [
                Button::subtle(id.sub("preview"), "Preview SQL"),
                Button::subtle(id.sub("discard"), "Discard"),
                Button::primary(id.sub("save"), "Save"),
            ],
        }
    }

    pub fn editable(mut self, on: bool) -> Self {
        self.editable = on;
        self
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
                let mut ws: Vec<usize> = self
                    .rows
                    .iter()
                    .take(200)
                    .map(|r| width(&r.get(ci).map(CellValue::text).unwrap_or_default()))
                    .collect();
                ws.sort_unstable();
                let p95 = ws.get(ws.len() * 95 / 100).copied().unwrap_or(0) as u16;
                let header = width(&c.name) as u16 + if c.primary { 2 } else { 0 } + 2;
                // a header never ellipsises below its own name (up to a sane cap)
                let max = c.max_width.max(header.min(24));
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
        if self.row_errors.contains_key(&src) {
            RowState::Error
        } else if self.pending.deleted.contains(&src) {
            RowState::Deleted
        } else if self.pending.inserted.contains(&src) {
            RowState::Inserted
        } else if self.pending.cells.keys().any(|(r, _)| *r == src) {
            RowState::Modified
        } else {
            RowState::Clean
        }
    }

    // ---- editing ----------------------------------------------------

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }
    pub fn edit_error(&self) -> Option<&str> {
        self.edit.as_ref().and_then(|e| e.error.as_deref())
    }

    fn cursor_src(&self) -> Option<usize> {
        self.order.get(self.cursor.0).copied()
    }

    pub fn begin_edit(&mut self) -> (Outcome, Option<GridEvent>) {
        let Some(src) = self.cursor_src() else {
            return (Outcome::Consumed, None);
        };
        if !self.editable {
            return (Outcome::Consumed, None);
        }
        let col = self.cursor.1;
        let spec = &self.columns[col];
        if spec.read_only || self.pending.deleted.contains(&src) {
            return (Outcome::Consumed, None);
        }
        let v = self.value(src, col).clone();
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
            (self.validator)(spec, &text)
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
            return match key.code {
                KeyCode::Char('+') if self.editable => (Outcome::Changed, self.insert_row()),
                KeyCode::Char('r') | KeyCode::F(5) => (Outcome::Changed, Some(GridEvent::Refresh)),
                KeyCode::Char('f') if key.plain() => {
                    (Outcome::Changed, Some(GridEvent::OpenFilters))
                }
                KeyCode::Char('F') => (Outcome::Changed, Some(GridEvent::ClearFilters)),
                _ => (Outcome::Ignored, None),
            };
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                self.set_cursor(r.saturating_sub(1), c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                self.set_cursor(r + 1, c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::Left if ctrl => {
                self.hscroll
                    .scroll_by(-(self.hscroll.viewport_len.max(1) as isize));
                (Outcome::Changed, None)
            }
            KeyCode::Right if ctrl => {
                self.hscroll
                    .scroll_by(self.hscroll.viewport_len.max(1) as isize);
                (Outcome::Changed, None)
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                self.set_cursor(r, c.saturating_sub(1), shift);
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                self.set_cursor(r, c + 1, shift);
                (Outcome::Changed, None)
            }
            KeyCode::Home if ctrl => {
                self.set_cursor(0, c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::End if ctrl => {
                self.set_cursor(usize::MAX, c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::Home => {
                self.set_cursor(r, 0, shift);
                (Outcome::Changed, None)
            }
            KeyCode::End => {
                self.set_cursor(r, usize::MAX, shift);
                (Outcome::Changed, None)
            }
            KeyCode::PageUp => {
                self.set_cursor(r.saturating_sub(self.scroll.viewport_len.max(1)), c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::PageDown => {
                self.set_cursor(r + self.scroll.viewport_len.max(1), c, shift);
                (Outcome::Changed, None)
            }
            KeyCode::Char('g') => {
                self.set_cursor(0, c, false);
                (Outcome::Changed, None)
            }
            KeyCode::Char('G') => {
                self.set_cursor(usize::MAX, c, false);
                (Outcome::Changed, None)
            }
            KeyCode::Enter | KeyCode::F(2) => {
                if self.on_more_row() {
                    return (Outcome::Changed, Some(GridEvent::FetchMore));
                }
                if !self.editable {
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
            KeyCode::Delete | KeyCode::Backspace => {
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
            KeyCode::Char('-') => {
                let Some(src) = self.cursor_src() else {
                    return (Outcome::Consumed, None);
                };
                (Outcome::Changed, self.toggle_delete(src))
            }
            KeyCode::Char('+') => (Outcome::Changed, self.insert_row()),
            KeyCode::Char('d') if ctrl => (Outcome::Changed, self.duplicate_row()),
            KeyCode::Char('y') => (
                Outcome::Changed,
                Some(GridEvent::Copy(self.copy_text(false))),
            ),
            KeyCode::Char('Y') => (
                Outcome::Changed,
                Some(GridEvent::Copy(self.copy_text(true))),
            ),
            KeyCode::Char('s') if ctrl => (Outcome::Changed, Some(GridEvent::CommitRequested)),
            KeyCode::Char('s') => {
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
            KeyCode::Char('S') => self.request_sort(None),
            KeyCode::Char('f') => {
                let value = self
                    .cursor_src()
                    .map(|src| self.value(src, c).clone())
                    .unwrap_or(CellValue::Null);
                (
                    Outcome::Changed,
                    Some(GridEvent::FilterOnCell { col: c, value }),
                )
            }
            KeyCode::Char('/') => (Outcome::Changed, Some(GridEvent::OpenFilters)),
            KeyCode::Char('F') => (Outcome::Changed, Some(GridEvent::ClearFilters)),
            KeyCode::Char('r') | KeyCode::F(5) => (Outcome::Changed, Some(GridEvent::Refresh)),
            KeyCode::Char('u') if key.plain() => {
                self.undo();
                (Outcome::Changed, None)
            }
            KeyCode::Char('U') => (Outcome::Changed, Some(GridEvent::DiscardRequested)),
            KeyCode::Char('p') if key.plain() => (Outcome::Changed, Some(GridEvent::PreviewSql)),
            KeyCode::Char(']') if ctrl => {
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
            _ => (Outcome::Ignored, None),
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
        id == self.id
            || id == scrollbar::id_for(self.id)
            || id == self.more_id()
            || id == self.left_id()
            || id == self.right_id()
            || self.locate(id).is_some()
            || self.locate_header(id).is_some()
            || self.locate_rownum(id).is_some()
            || self.bar.iter().any(|b| b.id == id)
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

    pub fn on_click(&mut self, id: WidgetId, pos: Position) -> (Outcome, Option<GridEvent>) {
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
        if let Some(c) = self.locate_header(id) {
            if self.edit.is_some() {
                self.commit_edit();
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
                        let col = pos.x.saturating_sub(rect.x) as usize;
                        self.edit
                            .as_mut()
                            .unwrap()
                            .buffer
                            .set_cursor_line_col(0, col);
                    }
                    return (Outcome::Changed, None);
                }
                self.commit_edit();
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
        if pressed == scrollbar::id_for(self.id) {
            return self.on_scrollbar(pos);
        }
        // range selection: find the cell under the pointer
        for d in self.scroll.visible_range() {
            let y = self.body.y + (d - self.scroll.offset) as u16;
            if y != pos.y {
                continue;
            }
            for (k, r) in self.col_rects.iter().enumerate() {
                if pos.x >= r.x && pos.x < r.right() + 2 {
                    let c = self.hscroll.offset + k;
                    if self.locate(pressed).is_some() && (d, c) != self.cursor {
                        self.set_cursor(d, c, true);
                        return Outcome::Changed;
                    }
                }
            }
        }
        Outcome::Ignored
    }

    pub fn on_wheel(&mut self, delta: i32, horizontal: bool) -> Outcome {
        if horizontal {
            self.hscroll.scroll_by(delta.signum() as isize);
        } else {
            self.scroll.scroll_by(delta as isize);
        }
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.body.y,
            1,
            self.body.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
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
            let w = self.widths[i];
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
            let w = self.widths[i].min(area.right().saturating_sub(x));
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

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.fit_header_marks();
        let area = area.intersection(*buf.area());
        if area.is_empty() || area.height < 2 {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        if !focused && self.edit.is_some() {
            self.commit_edit();
        }
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
        let body = Rect::new(
            grid_area.x,
            grid_area.y + 1,
            grid_area.width,
            grid_area.height.saturating_sub(1),
        );
        self.body = body;
        self.scroll.set_content(self.content_rows());
        self.scroll.set_viewport(body.height as usize);
        let has_sb = self.scroll.overflows();
        // gutter: bar(1) marker(1) change(1) rownum(w) space(1)
        let num_w = if self.row_numbers {
            (self.rows.len().max(1).to_string().len() as u16).max(2)
        } else {
            0
        };
        let gutter_w = 3 + num_w + if self.row_numbers { 1 } else { 0 };
        let cols_area = Rect::new(
            grid_area.x + gutter_w,
            grid_area.y,
            grid_area
                .width
                .saturating_sub(gutter_w + 4 + u16::from(has_sb)),
            grid_area.height,
        );
        self.layout_columns(cols_area);

        // header
        let header_y = grid_area.y;
        fill(
            buf,
            Rect::new(grid_area.x, header_y, grid_area.width, 1),
            Style::new().bg(bg),
        );
        if self.hscroll.offset > 0 {
            let lbl = format!("‹{}", self.hscroll.offset);
            let hovered = ctx.interaction.hovered(self.left_id());
            let st = if hovered {
                t.primary().bg(t.lift(bg))
            } else {
                t.faint().bg(bg)
            };
            buf.set_string(grid_area.x + 1, header_y, &lbl, st);
            ctx.clickable(
                self.left_id(),
                Rect::new(grid_area.x + 1, header_y, lbl.len() as u16, 1),
            );
        }
        for (k, rect) in self.col_rects.clone().iter().enumerate() {
            let ci = self.hscroll.offset + k;
            let col = &self.columns[ci];
            let hid = self.header_id(ci);
            let hovered = ctx.interaction.hovered(hid);
            let sorted = self.sort.map(|(c, _)| c == ci).unwrap_or(false);
            let filtered = self.filtered_cols.contains(&ci);
            let mut st = if sorted || filtered || ci == self.cursor.1 && focused {
                t.primary().bg(bg)
            } else {
                t.muted().bg(bg)
            };
            if hovered && col.sortable {
                st = st
                    .fg(t.text_primary)
                    .add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.border_strong);
            }
            let mut suffix = String::new();
            if filtered {
                suffix.push_str(" ∇");
            }
            match self.sort {
                Some((c, SortDir::Asc)) if c == ci => suffix.push_str(HEADER_SORT_ASC),
                Some((c, SortDir::Desc)) if c == ci => suffix.push_str(HEADER_SORT_DESC),
                _ => {}
            }
            let prefix = if col.primary { "▪ " } else { "" };
            let avail = (rect.width as usize).saturating_sub(width(&suffix) + width(prefix));
            let title = format!("{prefix}{}{suffix}", truncate(&col.name, avail.max(1)));
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
                buf.set_string(px, header_y, "⚷", st.fg(t.text_faint));
            }
            if col.sortable {
                ctx.clickable(hid, Rect::new(rect.x, header_y, rect.width, 1));
            }
        }
        let hidden_right = self
            .columns
            .len()
            .saturating_sub(self.hscroll.offset + self.hscroll.viewport_len);
        if hidden_right > 0 {
            let lbl = format!("{hidden_right}›");
            let x = cols_area.right() + 1;
            let hovered = ctx.interaction.hovered(self.right_id());
            let st = if hovered {
                t.primary().bg(t.lift(bg))
            } else {
                t.faint().bg(bg)
            };
            buf.set_string(x, header_y, &lbl, st);
            ctx.clickable(self.right_id(), Rect::new(x, header_y, lbl.len() as u16, 1));
        }

        if self.rows.is_empty() && !self.more {
            if self.loading {
                crate::widgets::progress::render_spinner(
                    Rect::new(body.x + 3, body.y + body.height / 2, body.width, 1),
                    buf,
                    ctx,
                    "Loading rows…",
                    bg,
                );
            } else {
                crate::widgets::empty::render(body, buf, t, &self.empty, bg);
            }
            return;
        }

        // rows
        let editing_here = self
            .edit
            .as_ref()
            .filter(|_| focused)
            .map(|e| (e.row, e.col));
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
                let mut s = ctx.state(mid);
                s.focused = focused && d == self.cursor.0;
                let st = t.row(s, bg);
                fill(buf, row_rect, st);
                buf.set_string(row_rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                let text = if self.loading {
                    format!(
                        "{} fetching…",
                        crate::widgets::progress::spinner_frame(ctx.interaction.tick)
                    )
                } else {
                    format!(
                        "↓ {} loaded · Enter fetches more",
                        crate::ui::text::thousands(self.rows.len())
                    )
                };
                buf.set_string(row_rect.x + gutter_w, y, &text, st.fg(t.text_muted));
                ctx.clickable(mid, row_rect);
                continue;
            }
            let src = self.order[d];
            let rid = self.rownum_id(d);
            let mut s = ctx.state(rid);
            let hovered_cell = (self.hscroll.offset
                ..self.hscroll.offset + self.hscroll.viewport_len)
                .find(|&c| ctx.interaction.hovered(self.cell_id(d, c)));
            if hovered_cell.is_some() {
                s.hovered = true;
            }
            s.focused = focused && d == self.cursor.0;
            s.selected = self.selected_rows.contains(&src);
            let state = self.row_state(src);
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
                "▎",
                t.gutter(s, row_style.bg.unwrap_or(bg), false),
            );
            if s.selected {
                buf.set_string(
                    row_rect.x + 1,
                    y,
                    "✓",
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
                RowState::Modified => ("•", row_style.fg(t.warning)),
                RowState::Inserted => ("+", row_style.fg(t.text_secondary)),
                RowState::Deleted => ("−", row_style.fg(t.text_muted)),
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
            for (k, rect) in self.col_rects.clone().iter().enumerate() {
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
                    let mut shown: String = text.chars().skip(off).take(cw).collect();
                    if off > 0 {
                        shown.replace_range(..shown.chars().next().map_or(0, char::len_utf8), "…");
                    }
                    let mut ts = es
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                    if e.error.is_some() {
                        ts = ts.underline_color(t.error);
                    }
                    buf.set_string(rect.x, y, &shown, ts);
                    ctx.set_cursor(Position::new(rect.x + (cur - off) as u16, y));
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
                    let err = self.cell_errors.get(&(src, ci));
                    let mut st = row_style;
                    let text = cell_text(v, col.kind, rect.width as usize);
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
                        st = st.bg(t.popover);
                    }
                    if dirty {
                        // changed values read in the warning tone; underline stays the editing token
                        st = st.fg(t.warning).remove_modifier(Modifier::ITALIC);
                    }
                    if err.is_some() {
                        st = st.fg(t.error).remove_modifier(Modifier::ITALIC);
                    }
                    if is_cursor {
                        st = Style::new()
                            .fg(if state == RowState::Deleted {
                                t.text_muted
                            } else {
                                t.canvas
                            })
                            .bg(if err.is_some() {
                                t.error
                            } else {
                                t.text_primary
                            })
                            .add_modifier(Modifier::BOLD);
                        if err.is_some() {
                            st = st.fg(t.text_primary);
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
                        fit_right(&text, rect.width as usize)
                    } else {
                        fit(&text, rect.width as usize)
                    };
                    buf.set_string(rect.x, y, &shown, st);
                    if col.references.is_some()
                        && !matches!(v, CellValue::Null | CellValue::Default)
                        && rect.width > 6
                    {
                        buf.set_string(
                            rect.right().saturating_sub(1),
                            y,
                            "→",
                            st.fg(if is_cursor { t.canvas } else { t.text_muted }),
                        );
                    }
                    if err.is_some() && !is_cursor {
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
        if has_sb {
            let sb = Rect::new(area.right() - 1, body.y, 1, body.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }

        // pending bar
        if bar_h > 0 {
            let by = area.bottom() - 1;
            let bar_area = Rect::new(area.x, by, area.width, 1);
            fill(buf, bar_area, Style::new().bg(bg));
            let (u, i, d) = self.pending.counts();
            let count = u + i + d;
            let text = format!("• {count} pending");
            buf.set_string(area.x + 1, by, &text, t.primary().fg(t.warning).bg(bg));
            // the row under the cursor explains its own rejection; otherwise the breakdown
            let cursor_src = self.order.get(self.cursor.0).copied();
            let (detail, ds) = match cursor_src.and_then(|r| self.row_errors.get(&r)) {
                Some(msg) => (format!("· {msg}"), t.error_fg().bg(bg)),
                None => (
                    self.pending_label()
                        .map(|d| format!("· {d}"))
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

/// Display text for a cell within `w` cells.
pub fn cell_text(v: &CellValue, kind: CellKind, w: usize) -> String {
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

impl Theme {
    /// Convenience for owners that render matching glyphs elsewhere.
    pub fn change_glyph(&self, state: RowState) -> (&'static str, Color) {
        match state {
            RowState::Clean => (" ", self.text_faint),
            RowState::Modified => ("•", self.warning),
            RowState::Inserted => ("+", self.text_secondary),
            RowState::Deleted => ("−", self.text_muted),
            RowState::Error => ("!", self.error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
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
}
