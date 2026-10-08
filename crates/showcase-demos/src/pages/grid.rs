//! Data grid: typed cells, a pending-change queue, paging and local sort.

use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};

use termrock::{
    Action, ActionKey, Align, CellDecor, CellRef, Column, ColumnKey, Cx, Dialog, DialogAction,
    DialogState, EditIntent, FgStep, FieldError, FrameRead, GlyphRole, Grid, GridAction,
    GridColumnFit, GridEditor, GridGutter, GridModel, GridOverflowIndicator, GridSortIndicator,
    GridState, Id, ItemKey, NavUnit, Panel, PanelKind, Part, Props, PropsRow, Rect, Response, Role,
    RowDecor, RowTotal, SortDir, StateFlags, StylePatch, Ui, UpdateCause, id,
};

use super::forms::SUBMIT;
use super::{ModalFooter, Page, PageStatus, PageUpdate, frame};

const GRID: Id = id!("grid.grid");
const CUSTOMERS: Id = id!("grid.customers");
const PREVIEW: Id = id!("grid.preview");

/// Cycle the cursor column's sort Asc → Desc → cleared (tag `s`).
pub const SORT: ActionKey = ActionKey::application("showcase.grid.sort");
/// Clear any local sort (tag `S`).
pub const SORT_CLEAR: ActionKey = ActionKey::application("showcase.grid.sort_clear");
/// Open the pending-changes facts dialog (tag `p`).
pub const PREVIEW_SQL: ActionKey = ActionKey::application("showcase.grid.preview");
/// Undo the last queue mutation, silently (tag `u`).
pub const UNDO: ActionKey = ActionKey::application("showcase.grid.undo");
/// Discard the whole queue (tag `U`).
pub const DISCARD: ActionKey = ActionKey::application("showcase.grid.discard");
/// Insert a row (tag `+`).
pub const INSERT_ROW: ActionKey = ActionKey::application("showcase.grid.insert");
/// Queue the cursor row for deletion (tag `-`).
pub const DELETE_ROW: ActionKey = ActionKey::application("showcase.grid.delete");

const NAMES: &[&str] = &[
    "Northwind Traders",
    "Blue Yonder Airlines",
    "Contoso Pharmaceuticals",
    "Fabrikam Robotics",
    "Litware Analytics",
    "Tailspin Toys",
    "Wide World Importers",
    "Adventure Works",
    "Proseware Studio",
    "Woodgrove Bank",
    "Alpine Ski House",
    "Coho Winery",
    "Lucerne Publishing",
    "Margie's Travel",
    "Trey Research",
    "Humongous Insurance",
];
const PLANS: &[&str] = &["free", "pro", "team", "enterprise"];
const OWNERS: &[&str] = &["mira", "jonas", "ana", "kai"];
const SEATS: &[u32] = &[1, 3, 5, 12, 25, 40, 80, 150];
const PAGE: usize = 40;
const ALL: usize = 96;
const ESTIMATED_TOTAL: usize = 4_812;

const COL_CUSTOMER: usize = 1;
const COL_PLAN: usize = 2;
const COL_SEATS: usize = 3;
const COL_ACTIVE: usize = 5;
const COL_RENEWED: usize = 6;
const COL_NOTES: usize = 7;

const COLUMN_NAMES: [&str; 8] = [
    "id",
    "customer",
    "plan",
    "seats",
    "mrr",
    "active",
    "renewed_at",
    "notes",
];

/// Fixed widths sampled from the historical grid (p95 content, header floor):
/// the layout never depends on the viewport, only the visible window does.
const COLUMNS: [Column<'static>; 8] = [
    Column {
        key: ColumnKey::num(0),
        title: "id",
        subtitle: None,
        align: Align::Left,
        min_width: 9,
        max_width: 9,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: Some(GlyphRole::PrimaryKey),
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(1),
        title: "customer",
        subtitle: None,
        align: Align::Left,
        min_width: 25,
        max_width: 25,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(2),
        title: "plan",
        subtitle: None,
        align: Align::Left,
        min_width: 10,
        max_width: 10,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(3),
        title: "seats",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(4),
        title: "mrr",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(5),
        title: "active",
        subtitle: None,
        align: Align::Left,
        min_width: 8,
        max_width: 8,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(6),
        title: "renewed_at",
        subtitle: None,
        align: Align::Left,
        min_width: 12,
        max_width: 12,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(7),
        title: "notes",
        subtitle: None,
        align: Align::Left,
        min_width: 27,
        max_width: 27,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
];

/// One stored cell: ghost cells (`NULL`, `DEFAULT`) read muted italic.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CellVal {
    text: String,
    ghost: bool,
}

impl CellVal {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ghost: false,
        }
    }

    fn ghost(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ghost: true,
        }
    }

    fn is_null(&self) -> bool {
        self.ghost && self.text == "NULL"
    }
}

/// One clean customer record; pending edits overlay it by id and never
/// rewrite it until the commit folds.
#[derive(Clone, Debug)]
struct Record {
    id: u64,
    cells: [CellVal; 8],
}

fn row(index: usize) -> Record {
    let plan = PLANS[(index * 7 + 3) % PLANS.len()];
    let seats = SEATS[(index * 5 + 1) % SEATS.len()];
    let mrr = match plan {
        "free" => 0.0,
        "pro" => 29.0 * f64::from(seats),
        "team" => 24.0 * f64::from(seats),
        _ => 19.0 * f64::from(seats),
    };
    let suffix = if index >= NAMES.len() {
        format!(" {}", index / NAMES.len() + 1)
    } else {
        String::new()
    };
    let renewed = if plan == "free" {
        CellVal::ghost("NULL")
    } else {
        CellVal::plain(format!(
            "2026-{:02}-{:02}",
            1 + index % 12,
            1 + (index * 3) % 28
        ))
    };
    let notes = if index.is_multiple_of(4) {
        CellVal::plain(format!(
            "{{\"owner\":\"{}\",\"seats\":{seats}}}",
            OWNERS[index % OWNERS.len()]
        ))
    } else {
        CellVal::ghost("NULL")
    };
    Record {
        id: 1001 + index as u64,
        cells: [
            CellVal::plain((1001 + index).to_string()),
            CellVal::plain(format!("{}{suffix}", NAMES[index % NAMES.len()])),
            CellVal::plain(plan),
            CellVal::plain(seats.to_string()),
            CellVal::plain(format!("{mrr:.2}")),
            CellVal::plain(if index % 5 == 3 { "false" } else { "true" }),
            renewed,
            notes,
        ],
    }
}

/// Undoable queue state: records, order and pending cells. Row errors ride
/// outside it (tag `undo` never clears a rejection) and so do paging and
/// the applied sort.
#[derive(Clone, Debug)]
struct Snapshot {
    records: HashMap<u64, Record>,
    order: Vec<u64>,
    insertion: Vec<u64>,
    cells: HashMap<(u64, usize), CellVal>,
    inserted: BTreeSet<u64>,
    deleted: BTreeSet<u64>,
    next_key: u64,
}

/// The 8-column typed customer model with its pending-change queue. Rows
/// are keyed by stable id, so the local sort (a permutation of `order`)
/// never invalidates pending cells, and inserts/deletes never shift keys.
#[derive(Clone, Debug, Default)]
struct CustomerModel {
    records: HashMap<u64, Record>,
    order: Vec<u64>,
    insertion: Vec<u64>,
    cells: HashMap<(u64, usize), CellVal>,
    inserted: BTreeSet<u64>,
    deleted: BTreeSet<u64>,
    row_errors: HashMap<u64, String>,
    undo: Vec<Snapshot>,
    next_key: u64,
    loaded: usize,
    sort: Option<(usize, SortDir)>,
}

impl CustomerModel {
    fn new() -> Self {
        let mut model = Self {
            next_key: 1001 + ALL as u64,
            ..Self::default()
        };
        model.load_page(0, PAGE);
        model
    }

    fn load_page(&mut self, from: usize, to: usize) {
        for index in from..to.min(ALL) {
            let record = row(index);
            self.order.push(record.id);
            self.insertion.push(record.id);
            self.records.insert(record.id, record);
        }
        self.loaded = self.loaded.max(to.min(ALL));
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            records: self.records.clone(),
            order: self.order.clone(),
            insertion: self.insertion.clone(),
            cells: self.cells.clone(),
            inserted: self.inserted.clone(),
            deleted: self.deleted.clone(),
            next_key: self.next_key,
        }
    }

    fn restore(&mut self, snap: Snapshot) {
        self.records = snap.records;
        self.order = snap.order;
        self.insertion = snap.insertion;
        self.cells = snap.cells;
        self.inserted = snap.inserted;
        self.deleted = snap.deleted;
        self.next_key = snap.next_key;
    }

    fn row_index(&self, key: ItemKey) -> Option<usize> {
        self.order.iter().position(|id| ItemKey::num(*id) == key)
    }

    fn id_at(&self, row: usize) -> Option<u64> {
        self.order.get(row).copied()
    }

    fn clean(&self, id: u64, col: usize) -> Option<&CellVal> {
        self.records.get(&id)?.cells.get(col)
    }

    /// Set a pending value; reverting to the stored value clears the change
    /// (tag `record_cell`).
    fn record_cell(&mut self, id: u64, col: usize, value: CellVal) {
        let Some(stored) = self.clean(id, col).cloned() else {
            return;
        };
        let before = self.cells.get(&(id, col)).cloned();
        let after = if value == stored && !self.inserted.contains(&id) {
            None
        } else {
            Some(value)
        };
        if before == after {
            return;
        }
        let snap = self.snapshot();
        match after {
            Some(v) => {
                self.cells.insert((id, col), v);
            }
            None => {
                self.cells.remove(&(id, col));
            }
        }
        self.undo.push(snap);
    }

    fn toggle_delete(&mut self, id: u64) {
        if !self.records.contains_key(&id) {
            return;
        }
        let snap = self.snapshot();
        if self.inserted.remove(&id) {
            self.records.remove(&id);
            self.order.retain(|kept| *kept != id);
            self.insertion.retain(|kept| *kept != id);
            self.cells.retain(|(kept, _), _| *kept != id);
            self.row_errors.remove(&id);
        } else if self.deleted.remove(&id) {
            // re-queued rows come back clean of cell edits, like the tag.
        } else {
            self.deleted.insert(id);
            self.cells.retain(|(kept, _), _| *kept != id);
        }
        self.undo.push(snap);
    }

    fn insert_row(&mut self) -> u64 {
        let snap = self.snapshot();
        let id = self.next_key;
        self.next_key = self.next_key.saturating_add(1).max(id);
        let record = Record {
            id,
            cells: [
                CellVal::ghost("DEFAULT"),
                CellVal::ghost("NULL"),
                CellVal::ghost("NULL"),
                CellVal::ghost("NULL"),
                CellVal::ghost("DEFAULT"),
                CellVal::plain("true"),
                CellVal::ghost("NULL"),
                CellVal::ghost("NULL"),
            ],
        };
        self.order.push(id);
        self.insertion.push(id);
        self.records.insert(id, record);
        self.inserted.insert(id);
        self.undo.push(snap);
        id
    }

    fn discard(&mut self) {
        for id in std::mem::take(&mut self.inserted) {
            self.records.remove(&id);
            self.order.retain(|kept| kept != &id);
            self.insertion.retain(|kept| kept != &id);
            self.cells.retain(|(kept, _), _| kept != &id);
        }
        self.cells.clear();
        self.deleted.clear();
        self.undo.clear();
        self.row_errors.clear();
    }

    fn undo(&mut self) -> bool {
        let Some(snap) = self.undo.pop() else {
            return false;
        };
        self.restore(snap);
        true
    }

    /// Fold the queue into the clean records and drop deleted rows.
    fn commit_ok(&mut self) -> usize {
        let n = self.pending_total();
        for ((id, col), value) in std::mem::take(&mut self.cells) {
            if let Some(record) = self.records.get_mut(&id)
                && let Some(cell) = record.cells.get_mut(col)
            {
                *cell = value;
            }
        }
        for id in std::mem::take(&mut self.deleted) {
            self.records.remove(&id);
            self.order.retain(|kept| kept != &id);
            self.insertion.retain(|kept| kept != &id);
        }
        self.inserted.clear();
        self.undo.clear();
        self.row_errors.clear();
        n
    }

    fn commit_err(&mut self, id: u64, message: String) {
        self.row_errors.insert(id, message);
    }

    /// (updated rows, inserted rows, deleted rows).
    fn counts(&self) -> (usize, usize, usize) {
        let updates = self
            .cells
            .keys()
            .map(|(id, _)| *id)
            .filter(|id| !self.inserted.contains(id))
            .collect::<BTreeSet<_>>()
            .len();
        (updates, self.inserted.len(), self.deleted.len())
    }

    fn pending_total(&self) -> usize {
        let (u, i, d) = self.counts();
        u + i + d
    }

    fn pending_label(&self) -> String {
        let (u, i, d) = self.counts();
        let mut parts = Vec::new();
        if u > 0 {
            parts.push(format!("{u} update{}", if u == 1 { "" } else { "s" }));
        }
        if i > 0 {
            parts.push(format!("{i} insert{}", if i == 1 { "" } else { "s" }));
        }
        if d > 0 {
            parts.push(format!("{d} delete{}", if d == 1 { "" } else { "s" }));
        }
        parts.join(" · ")
    }

    fn fetch_more(&mut self) -> Option<(usize, usize)> {
        if self.loaded >= ALL {
            return None;
        }
        let from = self.loaded;
        let to = (from + PAGE).min(ALL);
        self.load_page(from, to);
        Some((from + 1, self.loaded))
    }

    fn sort(&self) -> Option<(usize, SortDir)> {
        self.sort
    }

    /// Locally permute the display order over the *stored* values (tag
    /// `apply_local_sort` reads `rows`, never pending).
    fn apply_sort(&mut self, col: usize, dir: SortDir) {
        if col >= COLUMN_NAMES.len() {
            return;
        }
        self.sort = Some((col, dir));
        let records = &self.records;
        self.order.sort_by(|&a, &b| {
            let ra = records.get(&a);
            let rb = records.get(&b);
            let o = match (ra, rb) {
                (Some(ra), Some(rb)) => {
                    cmp_cells(&ra.cells[col], &rb.cells[col], col).then_with(|| a.cmp(&b))
                }
                _ => a.cmp(&b),
            };
            if dir == SortDir::Asc { o } else { o.reverse() }
        });
    }

    fn clear_sort(&mut self) {
        self.sort = None;
        self.order.clone_from(&self.insertion);
    }

    fn statements(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut cells: Vec<((usize, usize), &(u64, usize), &CellVal)> = self
            .cells
            .iter()
            .filter_map(|(key @ (id, col), value)| {
                self.row_index(ItemKey::num(*id))
                    .map(|row| ((row, *col), key, value))
            })
            .collect();
        cells.sort_by_key(|(pos, _, _)| *pos);
        for (_, (id, col), value) in &cells {
            if self.inserted.contains(id) {
                continue;
            }
            out.push(format!(
                "UPDATE customers SET {} = {} WHERE id = {};",
                COLUMN_NAMES[*col],
                literal(*col, value),
                id
            ));
        }
        for id in &self.inserted {
            let mut cols = Vec::new();
            let mut vals = Vec::new();
            for col in 0..COLUMN_NAMES.len() {
                if let Some(value) = self.cells.get(&(*id, col)) {
                    cols.push(COLUMN_NAMES[col]);
                    vals.push(literal(col, value));
                }
            }
            if cols.is_empty() {
                out.push("INSERT INTO customers DEFAULT VALUES;".to_owned());
            } else {
                out.push(format!(
                    "INSERT INTO customers ({}) VALUES ({});",
                    cols.join(", "),
                    vals.join(", ")
                ));
            }
        }
        for id in &self.deleted {
            out.push(format!("DELETE FROM customers WHERE id = {id};"));
        }
        out
    }
}

fn literal(col: usize, value: &CellVal) -> String {
    if value.is_null() {
        return "NULL".to_owned();
    }
    match col {
        0 | 3 | 4 | 5 => value.text.clone(),
        _ => format!("'{}'", value.text.replace('\'', "''")),
    }
}

fn numeric(text: &str) -> Option<f64> {
    text.parse::<i64>()
        .map(|n| n as f64)
        .ok()
        .or_else(|| text.parse::<f64>().ok().filter(|n| n.is_finite()))
}

/// Tag `cmp_cells`: nulls sort last, numbers numerically, the rest as
/// lowercase text.
fn cmp_cells(a: &CellVal, b: &CellVal, col: usize) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.is_null(), b.is_null()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }
    if matches!(col, 0 | 3 | 4) {
        if let (Some(x), Some(y)) = (numeric(&a.text), numeric(&b.text)) {
            return x.total_cmp(&y);
        }
    }
    a.text.to_lowercase().cmp(&b.text.to_lowercase())
}

impl GridModel for CustomerModel {
    fn row_count(&self) -> usize {
        self.order.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.id_at(row).map_or(ItemKey::num(0), ItemKey::num)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let id = self.id_at(row)?;
        let clean = self.clean(id, col)?;
        let value = self.cells.get(&(id, col)).unwrap_or(clean);
        Some(CellRef::new(value.text.as_str()))
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        let Some(id) = self.id_at(row) else {
            return CellDecor::default();
        };
        if self.deleted.contains(&id) {
            return CellDecor {
                tone: Some(Role::Fg(FgStep::Muted)),
                ..CellDecor::default()
            };
        }
        let dirty = self.cells.contains_key(&(id, col)) && !self.inserted.contains(&id);
        if dirty {
            // Changed values read warning; ghost italics drop (tag rows).
            return CellDecor {
                tone: Some(Role::Warning),
                dirty: true,
                ..CellDecor::default()
            };
        }
        let ghost = self.clean(id, col).is_some_and(|cell| cell.ghost)
            || self.cells.get(&(id, col)).is_some_and(|cell| cell.ghost);
        if ghost {
            CellDecor {
                tone: Some(Role::Fg(FgStep::Muted)),
                italic: true,
                ..CellDecor::default()
            }
        } else {
            CellDecor::default()
        }
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let Some(id) = self.id_at(row) else {
            return RowDecor::default();
        };
        let number = self
            .insertion
            .iter()
            .position(|kept| *kept == id)
            .map(|n| n + 1);
        let mut decor = RowDecor {
            number,
            ..RowDecor::default()
        };
        if self.row_errors.contains_key(&id) {
            decor.marker = Some(GlyphRole::Error);
            decor.tone = Some(Role::Danger);
        } else if self.deleted.contains(&id) {
            decor.marker = Some(GlyphRole::Deleted);
            decor.tone = Some(Role::Fg(FgStep::Muted));
            decor.strike = true;
        } else if self.inserted.contains(&id) {
            decor.marker = Some(GlyphRole::Inserted);
            decor.tone = Some(Role::Fg(FgStep::Secondary));
        } else if self.cells.keys().any(|(kept, _)| *kept == id) {
            decor.marker = Some(GlyphRole::Dirty);
            decor.tone = Some(Role::Warning);
        }
        decor
    }

    fn total(&self) -> RowTotal {
        if self.loaded >= ALL {
            RowTotal::Exact(ALL)
        } else {
            RowTotal::Estimated(ESTIMATED_TOTAL)
        }
    }

    fn has_more(&self) -> bool {
        self.loaded < ALL
    }

    fn pending_count(&self) -> usize {
        self.pending_total()
    }

    fn pending_breakdown(&self) -> String {
        self.pending_label()
    }

    fn row_error(&self, row: usize) -> Option<&str> {
        self.id_at(row)
            .and_then(|id| self.row_errors.get(&id).map(String::as_str))
    }
}

fn validate_cell(col: usize, text: &str) -> Result<CellVal, FieldError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return match col {
            COL_CUSTOMER | COL_PLAN | COL_NOTES => Ok(CellVal::plain("")),
            COL_RENEWED => Ok(CellVal::ghost("NULL")),
            _ => Err(FieldError::new("Empty: use Delete for NULL")),
        };
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return match col {
            COL_RENEWED | COL_NOTES => Ok(CellVal::ghost("NULL")),
            _ => Err(FieldError::new(format!(
                "{} is NOT NULL",
                COLUMN_NAMES[col]
            ))),
        };
    }
    match col {
        COL_SEATS => {
            if numeric(trimmed).is_some() {
                Ok(CellVal::plain(trimmed))
            } else {
                Err(FieldError::new("Must be a number"))
            }
        }
        COL_ACTIVE => match trimmed.to_ascii_lowercase().as_str() {
            "true" | "t" | "1" | "yes" => Ok(CellVal::plain("true")),
            "false" | "f" | "0" | "no" => Ok(CellVal::plain("false")),
            _ => Err(FieldError::new("Must be true or false")),
        },
        COL_NOTES => {
            let json = (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'));
            if json {
                Ok(CellVal::plain(trimmed))
            } else {
                Err(FieldError::new("Must be a JSON object or array"))
            }
        }
        COL_PLAN => {
            if PLANS.contains(&trimmed) {
                Ok(CellVal::plain(trimmed))
            } else {
                Err(FieldError::new(format!(
                    "Must be one of: {}",
                    PLANS.join(", ")
                )))
            }
        }
        COL_RENEWED => {
            let bytes = trimmed.as_bytes();
            if trimmed.len() >= 10 && bytes[4] == b'-' && bytes[7] == b'-' {
                Ok(CellVal::plain(trimmed))
            } else {
                Err(FieldError::new("Use YYYY-MM-DD"))
            }
        }
        COL_CUSTOMER => Ok(CellVal::plain(trimmed)),
        _ => Err(FieldError::new("Cell is read-only")),
    }
}

impl GridEditor for CustomerModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        let Some(id) = self.id_at(row) else {
            return EditIntent::Refuse {
                reason: "Unknown customer row",
            };
        };
        if self.deleted.contains(&id) {
            return EditIntent::Refuse {
                reason: "Row is queued for deletion",
            };
        }
        match col {
            COL_ACTIVE => EditIntent::Cycle,
            COL_NOTES => EditIntent::External,
            COL_CUSTOMER | COL_PLAN | COL_SEATS | COL_RENEWED => {
                let initial = self
                    .cells
                    .get(&(id, col))
                    .or_else(|| self.clean(id, col))
                    .map_or("", |cell| if cell.ghost { "" } else { cell.text.as_str() });
                EditIntent::Inline { initial }
            }
            _ => EditIntent::Refuse {
                reason: "Cell is read-only",
            },
        }
    }

    fn apply_cycle(&mut self, row: usize, col: usize) {
        if col != COL_ACTIVE {
            return;
        }
        let Some(id) = self.id_at(row) else {
            return;
        };
        if self.deleted.contains(&id) {
            return;
        }
        let next = self
            .cells
            .get(&(id, col))
            .or_else(|| self.clean(id, col))
            .is_some_and(|cell| cell.text == "true");
        self.record_cell(id, col, CellVal::plain(if next { "false" } else { "true" }));
    }

    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError> {
        let Some(id) = self.id_at(row) else {
            return Err(FieldError::new("Unknown customer row"));
        };
        if self.deleted.contains(&id) {
            return Err(FieldError::new("Row is queued for deletion"));
        }
        let value = validate_cell(col, text)?;
        self.record_cell(id, col, value);
        Ok(())
    }

    fn is_editable(&self, row: usize, col: usize) -> bool {
        matches!(
            col,
            COL_CUSTOMER | COL_PLAN | COL_SEATS | COL_ACTIVE | COL_RENEWED | COL_NOTES
        ) && self
            .id_at(row)
            .is_some_and(|id| !self.deleted.contains(&id))
    }
}

fn grid() -> Grid<'static> {
    Grid::new(GRID, &COLUMNS)
        .nav(NavUnit::Cell)
        .column_gap(2)
        .gutter(GridGutter::Detailed {
            row_numbers: true,
            min_digits: 2,
        })
        .right_reserve(4)
        .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
        .sort_indicator(GridSortIndicator::ActiveOnly)
        .overflow_indicator(GridOverflowIndicator::Count)
        .fetch_label("Enter fetches more")
}

/// Page-local meta tone: the historical meta reads faint while the shared
/// `PANEL` detail stays secondary (T4). The title keeps the recipe's own
/// base (secondary) and focused (primary + bold) rules untouched.
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

/// The one customers-card constructor: the meta gate reads its geometry and
/// draw paints through it, so the two can never disagree.
fn customers_panel(meta: &str, focused: bool) -> Panel<'_> {
    Panel::new(CUSTOMERS)
        .kind(PanelKind::Card)
        .title("customers")
        .meta(meta)
        .meta_late(true)
        .focused(focused)
        .patch_part(PANEL_PARTS)
}

/// `rows a–b of n loaded · ~t total · cols a–b of n`, from draw-time panel
/// geometry: last-frame layout is unavailable on frame 1, which is what the
/// no-input default capture presents.
fn position_label(
    grid: &Grid<'_>,
    state: &GridState,
    model: &CustomerModel,
    grid_area: Rect,
) -> String {
    let rows = grid.rows_label_for(state, model, grid_area);
    match grid.cols_label_for(state, model, grid_area) {
        Some(cols) => format!("{rows} · {cols}"),
        None => rows,
    }
}

const PREVIEW_ACTIONS: [Action<'static>; 2] = [
    Action::secondary(ActionKey::CANCEL, "Cancel"),
    Action::primary(ActionKey::CONFIRM, "Copy SQL"),
];

/// The open facts preview: statements plus their rendered prop values, all
/// stored so draw can borrow them.
#[derive(Clone, Debug)]
struct Preview {
    statements: Vec<String>,
    count: String,
    rows: String,
}

/// The data grid: typed cells over the shared [`Grid`] engine, with the
/// paging position composed into the card meta and the pending queue
/// driving the engine's bar, the save flow and the facts preview.
pub struct GridPage {
    model: CustomerModel,
    state: GridState,
    commit_ticks: u8,
    queued_status: Option<String>,
    preview: Option<Preview>,
    preview_state: DialogState,
    grid_focused: Cell<bool>,
}

impl GridPage {
    pub fn new() -> Self {
        Self {
            model: CustomerModel::new(),
            state: GridState::default(),
            commit_ticks: 0,
            queued_status: None,
            preview: None,
            preview_state: DialogState::default(),
            grid_focused: Cell::new(false),
        }
    }

    /// Build the facts dialog over locally-borrowed prop/code slices.
    fn with_preview_dialog<R>(preview: &Option<Preview>, f: impl FnOnce(Dialog<'_>) -> R) -> R {
        match preview {
            Some(found) => {
                let code: Vec<&str> = found.statements.iter().map(String::as_str).collect();
                let props = [
                    ("Statements", found.count.as_str()),
                    ("Rows", found.rows.as_str()),
                    ("Target", "customers"),
                ];
                f(Dialog::facts(PREVIEW, "Pending changes", &props)
                    .code(&code)
                    .width(66)
                    .actions(&PREVIEW_ACTIONS)
                    .cancel(ActionKey::CANCEL))
            }
            None => f(Dialog::facts(PREVIEW, "Pending changes", &[])
                .actions(&PREVIEW_ACTIONS)
                .cancel(ActionKey::CANCEL)),
        }
    }

    fn open_preview(&mut self, cx: &mut Cx<'_>) {
        if self.preview.is_some() {
            return;
        }
        let statements = self.model.statements();
        let (changed, inserted, deleted) = self.model.counts();
        self.preview = Some(Preview {
            count: statements.len().to_string(),
            rows: format!("{changed} changed · {inserted} inserted · {deleted} deleted"),
            statements,
        });
        Self::with_preview_dialog(&self.preview, |dialog| {
            cx.open_layer(PREVIEW, dialog.layer(cx));
        });
    }

    fn close_preview(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(PREVIEW) {
            cx.close_layer(PREVIEW, None);
        }
        self.preview = None;
    }

    /// `ctrl-s` / bar Save: arm the 4-tick saving window, or say the queue
    /// is empty.
    fn start_commit(&mut self) -> String {
        if self.model.pending_total() == 0 {
            "Nothing to save".to_owned()
        } else {
            self.commit_ticks = 4;
            "Saving…".to_owned()
        }
    }

    /// The "server" rejects seat counts above the plan limit (tag
    /// `finish_commit`); anything else folds into the clean records.
    fn finish_commit(&mut self) -> String {
        let bad = self
            .model
            .cells
            .iter()
            .find(|((_, col), value)| {
                *col == COL_SEATS
                    && !value.ghost
                    && value.text.parse::<i64>().is_ok_and(|n| n > 500)
            })
            .map(|((id, _), _)| *id);
        match bad {
            Some(id) => {
                self.model
                    .commit_err(id, "seats above the plan limit (500)".to_owned());
                "Save failed · the row is marked".to_owned()
            }
            None => {
                let n = self.model.commit_ok();
                format!("Saved {n} changes")
            }
        }
    }

    /// Queue-key guard: the grid owns these chords only while it holds
    /// focus, outside an inline edit and with no preview open. The Bubble
    /// pass fires whenever nothing consumed the key, so typing (consumed
    /// by the editor) never reaches here, but nav, bar buttons and the
    /// dialog all bubble through and must be refused.
    fn queue_key(&self) -> bool {
        !self.state.is_editing() && self.preview.is_none() && self.grid_focused.get()
    }
}

impl Default for GridPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for GridPage {
    fn title(&self) -> &'static str {
        "Data grid"
    }

    fn command(&mut self, cx: &mut Cx<'_>, action: ActionKey) -> Response<()> {
        if action != SORT
            && action != SORT_CLEAR
            && action != PREVIEW_SQL
            && action != UNDO
            && action != DISCARD
            && action != INSERT_ROW
            && action != DELETE_ROW
            && action != SUBMIT
        {
            return Response::ignored();
        }
        if !self.queue_key() {
            return Response::ignored();
        }
        if action == SORT {
            let Some((_, key)) = self.state.cursor() else {
                return Response::consumed();
            };
            let col = usize::from(key.raw());
            if COLUMNS.get(col).is_none_or(|column| !column.sortable) {
                return Response::consumed();
            }
            let next = match self.state.sort() {
                Some((current, SortDir::Asc)) if current == key => Some((key, SortDir::Desc)),
                Some((current, SortDir::Desc)) if current == key => None,
                _ => Some((key, SortDir::Asc)),
            };
            self.state.set_sort(next);
            match next {
                Some((_, dir)) => self.model.apply_sort(col, dir),
                None => self.model.clear_sort(),
            }
            return Response::changed();
        }
        if action == SORT_CLEAR {
            self.state.set_sort(None);
            self.model.clear_sort();
            return Response::changed();
        }
        if action == UNDO {
            self.model.undo();
            return Response::changed();
        }
        if action == INSERT_ROW {
            let id = self.model.insert_row();
            let _ = grid().move_cursor_to(
                &mut self.state,
                &self.model,
                ItemKey::num(id),
                ColumnKey::num(1),
            );
            self.queued_status = Some("Row inserted · fill it in, then Save".to_owned());
            return Response::changed();
        }
        if action == DELETE_ROW {
            let Some((key, _)) = self.state.cursor() else {
                return Response::consumed();
            };
            let Some(row) = self.model.row_index(key) else {
                return Response::consumed();
            };
            let Some(id) = self.model.id_at(row) else {
                return Response::consumed();
            };
            self.model.toggle_delete(id);
            self.queued_status = Some("Row queued for deletion · u undoes".to_owned());
            return Response::changed();
        }
        if action == DISCARD {
            self.model.discard();
            self.queued_status = Some("Changes discarded".to_owned());
            return Response::changed();
        }
        if action == PREVIEW_SQL {
            self.open_preview(cx);
            return Response::changed();
        }
        if action == SUBMIT {
            let status = self.start_commit();
            self.queued_status = Some(status);
            return Response::changed();
        }
        Response::ignored()
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut status = self.queued_status.take().map(PageStatus);
        // The tag's `animating()` keeps 80 ms ticks flowing while a commit
        // is in flight; without the repaint request the runtime delivers
        // no Tick and the saving window never closes.
        if self.commit_ticks > 0 {
            cx.request_repaint_after(std::time::Duration::from_millis(80));
            if cx.update_cause() == UpdateCause::Tick {
                self.commit_ticks -= 1;
                if self.commit_ticks == 0 {
                    status = Some(PageStatus(self.finish_commit()));
                }
            }
        }
        let before = self.model.pending_total();
        let action = grid().update_editable(cx, &mut self.state, &mut self.model);
        let act = action.action_ref().cloned();
        let mut response = action.erase();
        let mut explicit = status.is_some();
        match act {
            Some(GridAction::Sort(key, dir)) => {
                let col = usize::from(key.raw());
                // Header clicks cycle Asc → Desc → Asc in the engine; the
                // tag clears on the third click, so an Asc arriving over an
                // applied Desc clears instead of re-applying.
                if dir == SortDir::Asc && self.model.sort() == Some((col, SortDir::Desc)) {
                    self.state.set_sort(None);
                    self.model.clear_sort();
                } else if COLUMNS.get(col).is_some_and(|column| column.sortable) {
                    self.model.apply_sort(col, dir);
                }
            }
            Some(GridAction::Copy(text)) => {
                status = Some(PageStatus(format!("Copied {} chars", text.len())));
                explicit = true;
            }
            Some(GridAction::EditRequested(key, column)) => {
                let row = self.model.row_index(key).map_or(0, |row| row + 1);
                let name = COLUMNS
                    .get(usize::from(column.raw()))
                    .map_or("cell", |spec| spec.title);
                status = Some(PageStatus(format!(
                    "Would open the viewer for {name} on row {row}"
                )));
                explicit = true;
            }
            Some(GridAction::FetchMore) => {
                if let Some((from, to)) = self.model.fetch_more() {
                    status = Some(PageStatus(format!("Fetched rows {from}–{to}")));
                    explicit = true;
                }
            }
            Some(GridAction::CommitRequested) => {
                status = Some(PageStatus(self.start_commit()));
                explicit = true;
            }
            Some(GridAction::DiscardRequested) => {
                self.model.discard();
                status = Some(PageStatus("Changes discarded".to_owned()));
                explicit = true;
            }
            Some(GridAction::PreviewRequested) => {
                self.open_preview(cx);
                explicit = true;
            }
            Some(
                GridAction::Moved
                | GridAction::Activated(_)
                | GridAction::CellAction(..)
                | GridAction::LeaveForward
                | GridAction::LeaveBackward,
            )
            | None => {}
        }
        // Update the facts layer unconditionally: a dismissed layer is
        // removed by the runtime before the app update, and the dialog
        // drains that dismissal on the following frame (dialogs precedent).
        let dialog = Self::with_preview_dialog(&self.preview, |dialog| {
            dialog.update(cx, &mut self.preview_state)
        });
        let dact = dialog.action_ref().cloned();
        response |= dialog.erase();
        if self.preview.is_some() {
            match dact {
                Some(DialogAction::Action(key)) if key == ActionKey::CONFIRM => {
                    let n = self.preview.as_ref().map_or(0, |p| p.statements.len());
                    status = Some(PageStatus(format!("Copied {n} statements")));
                    explicit = true;
                    self.close_preview(cx);
                }
                Some(DialogAction::Action(_) | DialogAction::Dismissed(_)) => {
                    self.close_preview(cx);
                }
                None => {}
            }
        }
        if !explicit && self.model.pending_total() != before {
            status = Some(PageStatus(format!(
                "{} pending",
                self.model.pending_total()
            )));
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Typed cells, a pending-change queue, paging and local sort",
            |ui, body| {
                let card = Rect {
                    height: body.height.min(30),
                    ..body
                };
                let focused = ui.state(GRID).contains(StateFlags::FOCUSED);
                let probe = customers_panel("", focused).inner(ui, card);
                let meta = position_label(&grid(), &self.state, &self.model, probe);
                customers_panel(&meta, focused).draw(ui, card, |ui, inner| {
                    grid().draw(ui, inner, &self.state, &self.model);
                });
            },
        );
        self.grid_focused
            .set(ui.state(GRID).contains(StateFlags::FOCUSED));
        ui.layer(PREVIEW, |ui, layer| {
            Self::with_preview_dialog(&self.preview, |dialog| {
                dialog.draw(ui, layer, &self.preview_state, |ui, body| {
                    if let Some(found) = &self.preview {
                        // Tag `Prop::new` values read `Tone::Normal`
                        // (primary); the pairs default stays secondary.
                        let props = [
                            PropsRow::new(ItemKey::num(0), "Statements", found.count.as_str())
                                .tone(Role::Fg(FgStep::Primary)),
                            PropsRow::new(ItemKey::num(1), "Rows", found.rows.as_str())
                                .tone(Role::Fg(FgStep::Primary)),
                            PropsRow::new(ItemKey::num(2), "Target", "customers")
                                .tone(Role::Fg(FgStep::Primary)),
                        ];
                        Props::rich(&props).draw(ui, body);
                    }
                })
            })
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            &[("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else if ui.state(grid().preview_id()).contains(StateFlags::FOCUSED)
            || ui.state(grid().discard_id()).contains(StateFlags::FOCUSED)
            || ui.state(grid().save_id()).contains(StateFlags::FOCUSED)
        {
            // The footer appends its own Tab / Next (tag `draw_footer`).
            &[("Enter", "Activate")]
        } else {
            &[
                ("↑↓←→", "Cell"),
                ("Enter", "Edit"),
                ("s", "Sort"),
                ("Space", "Select row"),
                ("+ -", "Insert / delete"),
                ("u", "Undo"),
            ]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }

    fn modal_footer(&self, ui: &Ui<'_>) -> Option<ModalFooter> {
        // Facts bodies never quick-answer (tag arms y / n on Text only).
        ui.is_open(PREVIEW).then_some(ModalFooter {
            editing: false,
            quick_answer: false,
        })
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;

    fn text(model: &CustomerModel, row: usize, col: usize) -> String {
        model.cell(row, col).unwrap().text.to_owned()
    }

    #[test]
    fn typed_model_matches_the_historical_rows() {
        let model = CustomerModel::new();
        assert_eq!(model.row_count(), 40);
        assert_eq!(model.total(), RowTotal::Estimated(4_812));
        assert!(model.has_more());

        assert_eq!(model.row_key(0), ItemKey::num(1001));
        assert_eq!(text(&model, 0, 0), "1001");
        assert_eq!(text(&model, 0, 1), "Northwind Traders");
        assert_eq!(text(&model, 0, 2), "enterprise");
        assert_eq!(text(&model, 0, 3), "3");
        assert_eq!(text(&model, 0, 4), "57.00");
        assert_eq!(text(&model, 0, 5), "true");
        assert_eq!(text(&model, 0, 6), "2026-01-01");
        assert_eq!(text(&model, 0, 7), "{\"owner\":\"mira\",\"seats\":3}");

        // Free plans have no renewal date; sparse rows carry no notes.
        assert_eq!(text(&model, 3, 2), "free");
        assert_eq!(text(&model, 3, 6), "NULL");
        assert!(model.cell_decor(3, 6).italic);
        assert_eq!(text(&model, 1, 7), "NULL");
        assert!(model.cell_decor(1, 7).italic);
        assert!(!model.cell_decor(0, 7).italic);

        // Second page of names carries a suffix; inactive rows read false.
        assert_eq!(text(&model, 16, 1), "Northwind Traders 2");
        assert_eq!(text(&model, 3, 5), "false");
        assert_eq!(model.row_key(39), ItemKey::num(1040));
        assert_eq!(model.row_index(ItemKey::num(1002)), Some(1));
        assert_eq!(model.cell(0, 8), None);
        assert_eq!(model.cell(40, 0), None);
    }

    #[test]
    fn pending_queue_marks_rows_and_counts() {
        let mut model = CustomerModel::new();
        assert_eq!(model.pending_total(), 0);
        assert_eq!(model.pending_count(), 0);
        assert!(model.row_decor(0).marker.is_none());

        model.commit_cell(0, COL_SEATS, "600").unwrap();
        model.commit_cell(1, COL_SEATS, "12").unwrap();
        assert_eq!(text(&model, 0, 3), "600");
        assert_eq!(text(&model, 1, 3), "12");
        // mrr is computed, never re-derived from pending seats.
        assert_eq!(text(&model, 0, 4), "57.00");
        assert_eq!(model.counts(), (2, 0, 0));
        assert_eq!(model.pending_total(), 2);
        assert_eq!(model.pending_breakdown(), "2 updates");
        assert_eq!(model.row_decor(0).marker, Some(GlyphRole::Dirty));
        assert_eq!(model.row_decor(1).marker, Some(GlyphRole::Dirty));
        assert!(model.row_decor(2).marker.is_none());
        assert_eq!(
            model.cell_decor(0, 3).tone,
            Some(Role::Warning),
            "dirty cells read warning"
        );
        assert!(!model.cell_decor(0, 3).italic);

        // Reverting to the stored value clears the change.
        model.commit_cell(1, COL_SEATS, "80").unwrap();
        assert_eq!(model.counts(), (1, 0, 0));
        assert!(model.row_decor(1).marker.is_none());
    }

    #[test]
    fn statements_match_the_tag_preview() {
        let mut model = CustomerModel::new();
        model.commit_cell(0, COL_SEATS, "600").unwrap();
        model.commit_cell(1, COL_SEATS, "12").unwrap();
        assert_eq!(
            model.statements(),
            vec![
                "UPDATE customers SET seats = 600 WHERE id = 1001;".to_owned(),
                "UPDATE customers SET seats = 12 WHERE id = 1002;".to_owned(),
            ]
        );
    }

    #[test]
    fn bool_cycles_and_json_goes_external() {
        let mut model = CustomerModel::new();
        assert!(matches!(
            model.edit_intent(0, COL_ACTIVE),
            EditIntent::Cycle
        ));
        assert!(matches!(
            model.edit_intent(0, COL_NOTES),
            EditIntent::External
        ));
        assert!(matches!(
            model.edit_intent(0, COL_SEATS),
            EditIntent::Inline { .. }
        ));
        assert!(!model.is_editable(0, 0));
        assert!(!model.is_editable(0, 4));

        model.apply_cycle(0, COL_ACTIVE);
        assert_eq!(text(&model, 0, 5), "false");
        assert_eq!(model.pending_total(), 1);
        model.apply_cycle(0, COL_ACTIVE);
        assert_eq!(text(&model, 0, 5), "true");
        assert_eq!(model.pending_total(), 0, "a full toggle reverts");
    }

    #[test]
    fn validation_matches_the_tag_rules() {
        let mut model = CustomerModel::new();
        assert!(model.commit_cell(0, COL_SEATS, "many").is_err());
        assert!(model.commit_cell(0, COL_SEATS, "12.5").is_ok());
        assert!(model.commit_cell(0, COL_PLAN, "startup").is_err());
        assert!(model.commit_cell(0, COL_PLAN, "pro").is_ok());
        assert!(model.commit_cell(0, COL_RENEWED, "tomorrow").is_err());
        assert!(model.commit_cell(0, COL_RENEWED, "2026-02-02").is_ok());
        assert!(model.commit_cell(0, COL_NOTES, "nope").is_err());
        assert!(model.commit_cell(0, COL_CUSTOMER, "NULL").is_err());
        assert!(model.commit_cell(0, COL_RENEWED, "NULL").is_ok());
        assert_eq!(text(&model, 0, 6), "NULL");
    }

    #[test]
    fn insert_delete_undo_discard_follow_the_tag() {
        let mut model = CustomerModel::new();
        let id = model.insert_row();
        assert_eq!(model.counts(), (0, 1, 0));
        assert_eq!(
            model
                .row_decor(model.row_index(ItemKey::num(id)).unwrap())
                .marker,
            Some(GlyphRole::Inserted)
        );
        assert_eq!(
            model.statements(),
            vec!["INSERT INTO customers DEFAULT VALUES;".to_owned()]
        );

        // Deleting an inserted row removes it entirely.
        model.toggle_delete(id);
        assert_eq!(model.row_count(), 40);
        assert_eq!(model.pending_total(), 0);

        // Deleting a stored row queues it, struck; undo restores.
        model.toggle_delete(1001);
        assert_eq!(model.counts(), (0, 0, 1));
        let row = model.row_index(ItemKey::num(1001)).unwrap();
        let decor = model.row_decor(row);
        assert_eq!(decor.marker, Some(GlyphRole::Deleted));
        assert!(decor.strike);
        assert!(model.undo());
        assert_eq!(model.pending_total(), 0);

        model.commit_cell(0, COL_SEATS, "600").unwrap();
        model.discard();
        assert_eq!(model.pending_total(), 0);
        assert_eq!(text(&model, 0, 3), "3");
        assert!(!model.undo(), "discard clears the undo stack");
    }

    #[test]
    fn commit_folds_and_rejects_like_the_server() {
        let mut model = CustomerModel::new();
        model.commit_cell(1, COL_SEATS, "12").unwrap();
        assert_eq!(model.commit_ok(), 1);
        assert_eq!(model.pending_total(), 0);
        assert_eq!(text(&model, 1, 3), "12");

        model.commit_cell(0, COL_SEATS, "600").unwrap();
        model.commit_err(1001, "seats above the plan limit (500)".to_owned());
        assert_eq!(
            model.row_decor(0).marker,
            Some(GlyphRole::Error),
            "the rejected row reads `!`"
        );
        assert_eq!(model.row_error(0), Some("seats above the plan limit (500)"));
        // The surviving queue stays pending under the error.
        assert_eq!(model.pending_total(), 1);
    }

    #[test]
    fn local_sort_permutes_display_but_not_identity() {
        let mut model = CustomerModel::new();
        model.commit_cell(0, COL_SEATS, "600").unwrap();
        model.apply_sort(COL_SEATS, SortDir::Asc);
        assert_eq!(model.sort(), Some((COL_SEATS, SortDir::Asc)));
        // Smallest stored seats first; pending values never steer the order.
        assert_eq!(text(&model, 0, 3), "1");
        // The edited row keeps its key and its pending cell through the sort.
        let row = model.row_index(ItemKey::num(1001)).unwrap();
        assert_eq!(text(&model, row, 3), "600");
        assert_eq!(
            model.row_decor(row).marker,
            Some(GlyphRole::Dirty),
            "pending survives the permutation"
        );
        // Gutter numbers keep source order under sort.
        assert_eq!(
            model.row_decor(row).number,
            Some(1),
            "row 1001 is still source row 1"
        );
        model.clear_sort();
        assert_eq!(model.sort(), None);
        assert_eq!(model.row_key(0), ItemKey::num(1001));
    }

    #[test]
    fn fetch_more_pages_like_the_tag() {
        let mut model = CustomerModel::new();
        assert_eq!(model.fetch_more(), Some((41, 80)));
        assert_eq!(model.row_count(), 80);
        assert_eq!(model.total(), RowTotal::Estimated(4_812));
        assert!(model.has_more());
        assert_eq!(model.fetch_more(), Some((81, 96)));
        assert_eq!(model.row_count(), 96);
        assert_eq!(model.total(), RowTotal::Exact(96));
        assert!(!model.has_more());
        assert_eq!(model.fetch_more(), None);
    }
}

#[cfg(test)]
mod meta_tests {
    use super::*;

    fn harness() -> (Grid<'static>, GridState, CustomerModel) {
        (grid(), GridState::default(), CustomerModel::new())
    }

    #[test]
    fn paging_meta_matches_the_historical_labels() {
        let (grid, state, model) = harness();
        // 120x40: the card inner rect the panel hands the grid.
        let area = Rect::new(28, 7, 90, 27);
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total"
        );
        assert_eq!(
            grid.cols_label_for(&state, &model, area).as_deref(),
            Some("cols 1–6 of 8")
        );
        assert_eq!(
            position_label(&grid, &state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total · cols 1–6 of 8"
        );
    }

    #[test]
    fn narrow_viewports_clip_columns_and_rows() {
        let (grid, state, model) = harness();
        // 72x20: the card inner rect the panel hands the grid.
        let area = Rect::new(23, 7, 47, 11);
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 1–10 of 40 loaded · ~4,812 total"
        );
        assert_eq!(
            grid.cols_label_for(&state, &model, area).as_deref(),
            Some("cols 1–2 of 8")
        );
    }

    #[test]
    fn wide_viewports_drop_the_column_label() {
        let (grid, state, model) = harness();
        // 160x50: every column fits, so only the rows read out.
        let area = Rect::new(28, 7, 130, 27);
        assert_eq!(grid.cols_label_for(&state, &model, area), None,);
        assert_eq!(
            position_label(&grid, &state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total"
        );
    }

    #[test]
    fn scrolled_state_moves_the_window_not_the_paint() {
        let (grid, mut state, model) = harness();
        let area = Rect::new(28, 7, 90, 27);
        state.scroll_mut().apply_layout(26, model.row_count());
        state.scroll_mut().jump_end();
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 15–40 of 40 loaded · ~4,812 total"
        );
    }

    #[test]
    fn pending_bar_steals_two_viewport_rows() {
        let (grid, state, mut model) = harness();
        model.commit_cell(0, COL_SEATS, "600").unwrap();
        let area = Rect::new(28, 7, 90, 27);
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 1–24 of 40 loaded · ~4,812 total"
        );
    }
}
