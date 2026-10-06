//! `TablePro`'s application-owned data adapter.
//!
//! Adapts `tablepro-domain` pending edits and result sets to Termrock's `GridModel`
//! and `GridEditor` traits.

use termrock::{
    CellDecor, CellRef, ColumnKey, EditIntent, FgStep, FieldError, GlyphRole, GridEditor,
    GridModel, ItemKey, Role, RowDecor, RowTotal, SortDir,
};

use tablepro_domain::{ColType, PendingEdits, ResultSet, Value, cmp_values};

pub use tablepro_sql::preview_sql;

/// A database result adapted to the generic keyed grid.
#[derive(Clone, PartialEq)]
pub struct ResultGrid {
    types: Vec<ColType>,
    pending: PendingEdits,
    sort: Option<(ColumnKey, SortDir)>,
    total: usize,
    editable: bool,
    source: Option<String>,
    read_only_reason: Option<String>,
    display: Vec<Option<String>>,
    undo: Vec<PendingEdits>,
    pub estimated: bool,
}

impl core::fmt::Debug for ResultGrid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ResultGrid")
            .field("types", &self.types)
            .field("rows", &self.row_count())
            .field("pending_operations", &self.pending_total())
            .field("total", &self.total)
            .field("editable", &self.editable)
            .field("has_source", &self.source.is_some())
            .field("has_read_only_reason", &self.read_only_reason.is_some())
            .field("undo_depth", &self.undo.len())
            .field("sort", &self.sort)
            .finish_non_exhaustive()
    }
}

impl ResultGrid {
    /// An empty, read-only result adapter.
    pub fn empty() -> Self {
        Self::from_result(&ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            total: 0,
            source: None,
            duration_ms: 0,
            editable: false,
        })
    }

    /// Adapt one SQL result without exposing database types to `termrock`.
    pub fn from_result(result: &ResultSet) -> Self {
        let pending = PendingEdits::new(result.rows.clone());
        let read_only_reason = (!result.editable)
            .then(|| "Read-only result: select a primary-key column to edit".to_owned());
        let mut grid = Self {
            types: result.columns.iter().map(|(_, ty)| *ty).collect(),
            pending,
            sort: None,
            total: result.total,
            editable: result.editable,
            source: result.source.clone(),
            read_only_reason,
            display: Vec::new(),
            undo: Vec::new(),
            estimated: false,
        };
        grid.rebuild_display();
        grid
    }

    /// Set whether row total is estimated.
    pub fn set_estimated(&mut self, estimated: bool) {
        self.estimated = estimated;
    }

    /// Number of loaded rows.
    pub fn row_count(&self) -> usize {
        self.pending.row_count()
    }

    /// Total rows represented by the query, including rows beyond the cap.
    pub fn total(&self) -> usize {
        self.total.max(self.row_count())
    }

    /// Whether cells may be edited through `Grid::update_editable`.
    pub fn is_editable(&self) -> bool {
        self.editable
    }

    /// The source relation, when the result came from one table.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Current pending-edit state.
    pub fn pending(&self) -> &PendingEdits {
        &self.pending
    }

    /// Number of pending row updates, inserts and deletes.
    pub fn pending_total(&self) -> usize {
        self.pending
            .rows()
            .iter()
            .enumerate()
            .map(|(row, cells)| {
                if self.pending.is_inserted(row) {
                    1
                } else {
                    usize::from(self.pending.is_deleted(row)).saturating_add(usize::from(
                        (0..cells.current.len()).any(|col| self.pending.is_dirty(row, col)),
                    ))
                }
            })
            .sum()
    }

    /// Tuple of (updates, inserts, deletes) counts.
    pub fn pending_counts(&self) -> (usize, usize, usize) {
        let mut u: usize = 0;
        let mut i: usize = 0;
        let mut d: usize = 0;
        for (row, cells) in self.pending.rows().iter().enumerate() {
            if self.pending.is_inserted(row) {
                i = i.saturating_add(1);
            } else {
                if self.pending.is_deleted(row) {
                    d = d.saturating_add(1);
                }
                if (0..cells.current.len()).any(|col| self.pending.is_dirty(row, col)) {
                    u = u.saturating_add(1);
                }
            }
        }
        (u, i, d)
    }

    /// Insert a row with typed NULL/default values.
    pub fn insert_row(&mut self) -> Option<usize> {
        if !self.editable {
            return None;
        }
        let before = self.pending.clone();
        let row = self.pending.insert_row(self.types.len())?;
        self.undo.push(before);
        self.rebuild_display();
        Some(row)
    }

    /// Insert schema-derived DEFAULT/NULL values as one undoable operation.
    pub fn insert_row_with_defaults(&mut self, defaults: &[bool]) -> Option<usize> {
        if !self.editable || defaults.len() != self.types.len() {
            return None;
        }
        let before = self.pending.clone();
        let values = defaults
            .iter()
            .map(|default| {
                if *default {
                    Value::Default
                } else {
                    Value::Null
                }
            })
            .collect();
        let row = self.pending.insert_values(values)?;
        self.undo.push(before);
        self.rebuild_display();
        Some(row)
    }

    /// Duplicate an existing row, resetting primary/generated columns to DEFAULT.
    pub fn duplicate_row(&mut self, src: usize, defaults: &[bool]) -> Option<usize> {
        if !self.editable {
            return None;
        }
        let before = self.pending.clone();
        let mut values = Vec::with_capacity(self.types.len());
        for col in 0..self.types.len() {
            if defaults.get(col).copied().unwrap_or(false) {
                values.push(Value::Default);
            } else if let Some(val) = self.pending.value(src, col) {
                values.push(val.clone());
            } else {
                values.push(Value::Null);
            }
        }
        let row = self.pending.insert_values(values)?;
        self.undo.push(before);
        self.rebuild_display();
        Some(row)
    }

    /// Toggle a row deletion; marking deletion replaces existing cell updates.
    pub fn toggle_delete(&mut self, row: usize) -> bool {
        if !self.editable {
            return false;
        }
        let before = self.pending.clone();
        if !self.pending.toggle_delete(row) {
            return false;
        }
        self.undo.push(before);
        self.rebuild_display();
        true
    }

    /// Mark a row deleted, preserving it for undo/save preview.
    pub fn delete_row(&mut self, row: usize) -> bool {
        if !self.editable {
            return false;
        }
        let before = self.pending.clone();
        let changed = self.pending.delete_row(row);
        if changed {
            self.undo.push(before);
            self.rebuild_display();
        }
        changed
    }

    /// Restore all cells and row lifecycle markers to clean baseline.
    pub fn discard(&mut self) {
        self.undo.push(self.pending.clone());
        self.pending.clear();
        self.restore_sort();
    }

    /// Restore previous pending state, if one exists.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        let next_key = self
            .pending
            .next_key
            .zip(previous.next_key)
            .map(|(a, b)| a.max(b));
        self.pending = previous;
        self.pending.next_key = next_key;
        self.restore_sort();
        true
    }

    /// Commit pending edits to baseline, clearing undo history.
    pub fn commit(&mut self) {
        self.pending.commit();
        self.undo.clear();
        self.rebuild_display();
    }

    /// Expose a deterministic position label for app status bars.
    pub fn position_label(&self, first_row: usize, visible_rows: usize) -> String {
        if self.row_count() == 0 {
            return "0 rows".to_owned();
        }
        let first = first_row.saturating_add(1).min(self.row_count());
        let last = first
            .saturating_add(visible_rows.saturating_sub(1))
            .min(self.row_count());
        format!("rows {first}–{last} of {}", self.total())
    }

    /// Number of changed cells.
    pub fn dirty_cell_count(&self) -> usize {
        self.pending.dirty_cell_count()
    }

    /// Sort by a grid column key, retaining keyed row identity and edits.
    pub fn sort(&mut self, key: ColumnKey, direction: SortDir) {
        let Some(column) = usize::from(key.raw()).checked_sub(1) else {
            return;
        };
        if column >= self.types.len() {
            return;
        }
        self.sort = Some((key, direction));
        let mut order: Vec<usize> = (0..self.row_count()).collect();
        order.sort_by(|&a, &b| {
            let left = self.pending.value(a, column).unwrap_or(&Value::Null);
            let right = self.pending.value(b, column).unwrap_or(&Value::Null);
            let cmp = cmp_values(left, right);
            if direction == SortDir::Asc {
                cmp
            } else {
                cmp.reverse()
            }
        });
        self.pending.reorder(&order);
        self.rebuild_display();
    }

    fn restore_sort(&mut self) {
        if let Some((key, direction)) = self.sort {
            self.sort(key, direction);
        } else {
            self.rebuild_display();
        }
    }

    fn rebuild_display(&mut self) {
        self.display = self
            .pending
            .rows()
            .iter()
            .flat_map(|row| {
                row.current.iter().map(|value| match value {
                    Value::Text(_) | Value::Json(_) => None,
                    value => Some(value.display()),
                })
            })
            .collect();
    }

    fn display_cell(&self, row: usize, col: usize) -> Option<&str> {
        match self.pending.value(row, col)? {
            Value::Text(value) | Value::Json(value) => Some(value.as_str()),
            _ => self
                .types
                .len()
                .checked_mul(row)
                .and_then(|offset| offset.checked_add(col))
                .and_then(|index| self.display.get(index))
                .and_then(Option::as_deref),
        }
    }

    fn parse_value(&self, col: usize, text: &str) -> Result<Value, FieldError> {
        let Some(ty) = self.types.get(col).copied() else {
            return Err(FieldError::coded("Unknown result column", "column"));
        };
        if text.trim().eq_ignore_ascii_case("NULL") {
            return Ok(Value::Null);
        }
        match ty {
            ColType::Int => text
                .trim()
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| FieldError::coded("Enter a whole number", "integer")),
            ColType::Numeric => text
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .map(Value::Num)
                .ok_or_else(|| FieldError::coded("Enter a finite number", "numeric")),
            ColType::Bool => match text.trim().to_ascii_lowercase().as_str() {
                "true" | "t" | "1" => Ok(Value::Bool(true)),
                "false" | "f" | "0" => Ok(Value::Bool(false)),
                _ => Err(FieldError::coded("Enter true or false", "boolean")),
            },
            ColType::Json => Ok(Value::Json(text.to_owned())),
            ColType::Uuid | ColType::Text | ColType::Timestamp | ColType::Date | ColType::Enum => {
                Ok(Value::Text(text.to_owned()))
            }
        }
    }
}

impl GridModel for ResultGrid {
    fn row_count(&self) -> usize {
        self.row_count()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.pending
            .rows()
            .get(row)
            .map_or_else(|| ItemKey::index(row), |record| ItemKey::num(record.key))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.display_cell(row, col).map(CellRef::new)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let is_del = self.pending.is_deleted(row);
        let is_ins = self.pending.is_inserted(row);
        let is_mod = self.pending.is_dirty_row(row);
        let marker = if is_del {
            Some(GlyphRole::Deleted)
        } else if is_ins {
            Some(GlyphRole::Inserted)
        } else if is_mod {
            Some(GlyphRole::Dirty)
        } else {
            None
        };
        let tone = if is_del {
            Some(Role::Fg(FgStep::Faint))
        } else if is_ins {
            Some(Role::Fg(FgStep::Secondary))
        } else if is_mod {
            Some(Role::Warning)
        } else {
            None
        };
        RowDecor {
            marker,
            strike: is_del,
            tone,
            ..RowDecor::default()
        }
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        let dirty = self.pending.is_dirty(row, col);
        let mut italic = false;
        let mut tone = None;
        if !dirty {
            match self.pending.value(row, col) {
                Some(Value::Null | Value::Default) => {
                    tone = Some(Role::Fg(FgStep::Muted));
                    italic = true;
                }
                Some(Value::Text(s)) if s.is_empty() => {
                    tone = Some(Role::Fg(FgStep::Faint));
                }
                _ => {}
            }
        }
        CellDecor {
            dirty,
            italic,
            tone,
            ..CellDecor::default()
        }
    }

    fn total(&self) -> RowTotal {
        if self.estimated {
            RowTotal::Estimated(self.total())
        } else {
            RowTotal::Exact(self.total())
        }
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.read_only_reason.as_deref()
    }
}

impl GridEditor for ResultGrid {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        if !self.editable || self.pending.is_deleted(row) || self.pending.value(row, col).is_none()
        {
            return EditIntent::Refuse {
                reason: self
                    .read_only_reason
                    .as_deref()
                    .unwrap_or("Cell is read-only"),
            };
        }
        self.display_cell(row, col).map_or(
            EditIntent::Refuse {
                reason: "Missing cell",
            },
            |initial| EditIntent::Inline { initial },
        )
    }

    fn apply_cycle(&mut self, row: usize, col: usize) {
        if !self.editable
            || self.pending.is_deleted(row)
            || self.pending.value(row, col).is_none()
            || self.types.get(col) != Some(&ColType::Bool)
        {
            return;
        }
        let next = match self.pending.value(row, col) {
            Some(Value::Bool(true)) => Value::Bool(false),
            Some(Value::Bool(false)) => Value::Null,
            _ => Value::Bool(true),
        };
        let before = self.pending.clone();
        if self.pending.set(row, col, next) {
            self.undo.push(before);
            self.rebuild_display();
        }
    }

    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError> {
        let value = self.parse_value(col, text)?;
        let before = self.pending.clone();
        if !self.pending.set(row, col, value) {
            return Ok(());
        }
        self.undo.push(before);
        self.rebuild_display();
        Ok(())
    }

    fn is_editable(&self, row: usize, col: usize) -> bool {
        self.editable && !self.pending.is_deleted(row) && self.pending.value(row, col).is_some()
    }
}
