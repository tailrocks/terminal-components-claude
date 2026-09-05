//! `TablePro`'s application-owned data adapter.
//!
//! The generic grid owns navigation and edit lifecycle. This module owns the
//! database-shaped rows, stable row keys, type-aware parsing, sorting, and
//! SQL preview generation. The split is deliberate: the library never knows
//! about databases, while the app never reaches into runtime internals.

use junie_tui::{
    CellDecor, CellRef, ColumnKey, EditIntent, FieldError, GridEditor, GridModel, ItemKey,
    RowDecor, RowTotal, SortDir, StateFlags,
};

use crate::db::{ColType, Table, Value};
use crate::sql;

/// Application-owned pending edits over a rectangular result set.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEdits {
    // Most result sets are rendered and discarded without an edit. Keep the
    // clean snapshot lazy so loading a large result does not clone every
    // database value solely to support a possible later undo/save.
    original: Option<Vec<Vec<Value>>>,
    current: Vec<Vec<Value>>,
    inserted: Vec<bool>,
    deleted: Vec<bool>,
}

impl PendingEdits {
    /// Start tracking `rows` without changing them.
    pub fn new(rows: Vec<Vec<Value>>) -> Self {
        let flags = vec![false; rows.len()];
        Self {
            original: None,
            current: rows,
            inserted: flags.clone(),
            deleted: flags,
        }
    }

    /// Number of rows currently held by the result.
    pub fn row_count(&self) -> usize {
        self.current.len()
    }

    /// Read one current cell.
    pub fn value(&self, row: usize, col: usize) -> Option<&Value> {
        self.current.get(row).and_then(|cells| cells.get(col))
    }

    fn original_value(&self, row: usize, col: usize) -> Option<&Value> {
        self.original
            .as_ref()
            .and_then(|rows| rows.get(row))
            .and_then(|cells| cells.get(col))
            .or_else(|| self.current.get(row).and_then(|cells| cells.get(col)))
    }

    fn snapshot(&mut self) {
        if self.original.is_none() {
            self.original = Some(self.current.clone());
        }
    }

    /// Set one cell. Returns whether the value changed.
    pub fn set(&mut self, row: usize, col: usize, value: Value) -> bool {
        let Some(cell) = self.value(row, col) else {
            return false;
        };
        if *cell == value {
            return false;
        }
        self.snapshot();
        let Some(cell) = self
            .current
            .get_mut(row)
            .and_then(|cells| cells.get_mut(col))
        else {
            return false;
        };
        *cell = value;
        true
    }

    /// Add a new row initialized to SQL NULLs and return its row index.
    pub fn insert_row(&mut self, columns: usize) -> usize {
        self.snapshot();
        let row = self.current.len();
        let values = vec![Value::Null; columns];
        self.current.push(values);
        self.inserted.push(true);
        self.deleted.push(false);
        row
    }

    /// Mark a row for deletion. Deleting an inserted row cancels that insert.
    pub fn delete_row(&mut self, row: usize) -> bool {
        if row >= self.deleted.len() {
            return false;
        }
        self.snapshot();
        let Some(deleted) = self.deleted.get_mut(row) else {
            return false;
        };
        if self.inserted.get(row).copied().unwrap_or(false) {
            if let Some(inserted) = self.inserted.get_mut(row) {
                *inserted = false;
            }
            if let Some(current) = self.current.get_mut(row)
                && let Some(original) = self.original.as_ref().and_then(|rows| rows.get(row))
            {
                current.clone_from(original);
            }
            return true;
        }
        if *deleted {
            return false;
        }
        *deleted = true;
        true
    }

    /// Whether the row was inserted by the user.
    pub fn is_inserted(&self, row: usize) -> bool {
        self.inserted.get(row).copied().unwrap_or(false)
    }

    /// Whether the row was marked for deletion.
    pub fn is_deleted(&self, row: usize) -> bool {
        self.deleted.get(row).copied().unwrap_or(false)
    }

    /// Whether one cell differs from its original value.
    pub fn is_dirty(&self, row: usize, col: usize) -> bool {
        if self.is_inserted(row) {
            return self
                .value(row, col)
                .is_some_and(|value| *value != Value::Null);
        }
        self.value(row, col)
            .zip(self.original_value(row, col))
            .is_some_and(|(current, original)| current != original)
    }

    /// Whether any cell or row lifecycle operation is pending.
    pub fn is_dirty_row(&self, row: usize) -> bool {
        self.is_inserted(row)
            || self.is_deleted(row)
            || self
                .current
                .get(row)
                .is_some_and(|cells| (0..cells.len()).any(|col| self.is_dirty(row, col)))
    }

    /// Number of changed cells, excluding row lifecycle markers.
    pub fn dirty_cell_count(&self) -> usize {
        self.current
            .iter()
            .enumerate()
            .map(|(row, cells)| {
                (0..cells.len())
                    .filter(|&col| self.is_dirty(row, col))
                    .count()
            })
            .sum()
    }

    /// Row indexes with pending cell or row changes.
    pub fn dirty_rows(&self) -> Vec<usize> {
        (0..self.current.len())
            .filter(|&row| self.is_dirty_row(row))
            .collect()
    }

    /// Discard pending edits and restore the loaded clean rows.
    pub fn clear(&mut self) {
        // `clear` is the discard transition, not a save transition. Restore
        // the immutable load snapshot and drop rows that only exist in the
        // pending edit set; callers can then safely rebuild their display
        // cache without silently committing user edits.
        if let Some(original) = self.original.take() {
            self.current = original;
        }
        self.inserted = vec![false; self.current.len()];
        self.deleted = vec![false; self.current.len()];
    }

    /// Reorder rows while preserving each row's pending state.
    pub fn reorder(&mut self, order: &[usize]) {
        if let Some(original) = self.original.as_mut() {
            let rows = core::mem::take(original);
            *original = reorder_owned(rows, order);
        }
        let rows = core::mem::take(&mut self.current);
        self.current = reorder_owned(rows, order);
        self.inserted = reorder_flags(&self.inserted, order);
        self.deleted = reorder_flags(&self.deleted, order);
    }
}

fn reorder_owned(rows: Vec<Vec<Value>>, order: &[usize]) -> Vec<Vec<Value>> {
    let mut rows: Vec<Option<Vec<Value>>> = rows.into_iter().map(Some).collect();
    order
        .iter()
        .filter_map(|&index| rows.get_mut(index).and_then(Option::take))
        .collect()
}

fn reorder_flags(flags: &[bool], order: &[usize]) -> Vec<bool> {
    order
        .iter()
        .filter_map(|&index| flags.get(index).copied())
        .collect()
}

/// Quote a database value for the deterministic SQL preview.
pub(crate) fn sql_literal(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_owned(),
        Value::Text(text) => format!("'{}'", text.replace('\'', "''")),
        Value::Int(value) => value.to_string(),
        Value::Num(value) => format!("{value:.2}"),
        Value::Bool(value) => value.to_string(),
        Value::Json(value) => format!("'{}'::jsonb", value.replace('\'', "''")),
    }
}

/// Build the statements `TablePro` would send for pending result edits.
pub(crate) fn preview_sql(
    table: &Table,
    columns: &[(String, ColType)],
    pending: &PendingEdits,
) -> Vec<String> {
    let pk = table.primary_key();
    let where_clause = |row: usize| {
        let mut terms = Vec::new();
        if pk.is_empty() {
            for (col, _) in columns.iter().enumerate() {
                let Some((name, _)) = columns.get(col) else {
                    continue;
                };
                let value = pending.original_value(row, col).unwrap_or(&Value::Null);
                terms.push(match value {
                    Value::Null => format!("{name} IS NULL"),
                    value => format!("{name} = {}", sql_literal(value)),
                });
            }
        } else {
            for key in &pk {
                let Some(col) = columns.iter().position(|(name, _)| name == &key.name) else {
                    continue;
                };
                let value = pending.original_value(row, col).unwrap_or(&Value::Null);
                terms.push(format!("{} = {}", key.name, sql_literal(value)));
            }
        }
        if terms.is_empty() {
            "1 = 0".to_owned()
        } else {
            terms.join(" AND ")
        }
    };

    let dirty_rows = pending.dirty_rows();
    let mut out = Vec::new();

    // Keep the save order deterministic and compatible with TablePro's old
    // commit queue: updates, then inserts, then deletes.
    for &row in &dirty_rows {
        if pending.is_inserted(row) || pending.is_deleted(row) {
            continue;
        }
        let mut sets = Vec::new();
        for col in 0..pending.current_row_width(row) {
            if pending.is_dirty(row, col)
                && let Some((name, _)) = columns.get(col)
                && let Some(value) = pending.value(row, col)
            {
                sets.push(format!("{name} = {}", sql_literal(value)));
            }
        }
        if !sets.is_empty() {
            out.push(format!(
                "UPDATE {} SET {} WHERE {};",
                table.qualified(),
                sets.join(", "),
                where_clause(row)
            ));
        }
    }

    for &row in &dirty_rows {
        if pending.is_inserted(row) && !pending.is_deleted(row) {
            let mut names = Vec::new();
            let mut values = Vec::new();
            for (col, (name, _)) in columns.iter().enumerate() {
                let value = pending.value(row, col).unwrap_or(&Value::Null);
                if !matches!(value, Value::Null) {
                    names.push(name.clone());
                    values.push(sql_literal(value));
                }
            }
            if names.is_empty() {
                out.push(format!("INSERT INTO {} DEFAULT VALUES;", table.qualified()));
            } else {
                out.push(format!(
                    "INSERT INTO {} ({}) VALUES ({});",
                    table.qualified(),
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }
    }

    for &row in &dirty_rows {
        if pending.is_deleted(row) && !pending.is_inserted(row) {
            out.push(format!(
                "DELETE FROM {} WHERE {};",
                table.qualified(),
                where_clause(row)
            ));
        }
    }
    out
}

impl PendingEdits {
    fn current_row_width(&self, row: usize) -> usize {
        self.current.get(row).map_or(0, Vec::len)
    }
}

/// A database result adapted to the generic keyed grid.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultGrid {
    types: Vec<ColType>,
    pending: PendingEdits,
    keys: Vec<ItemKey>,
    total: usize,
    editable: bool,
    source: Option<String>,
    read_only_reason: Option<String>,
    // Text and JSON cells borrow their already-owned value at read time;
    // only scalar values that need formatting allocate a cached string.
    display: Vec<Option<String>>,
    undo: Vec<PendingEdits>,
}

impl ResultGrid {
    /// An empty, read-only result adapter.
    pub fn empty() -> Self {
        Self::from_result(&sql::ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            total: 0,
            source: None,
            duration_ms: 0,
            editable: false,
        })
    }

    /// Adapt one SQL result without exposing database types to `junie-tui`.
    pub fn from_result(result: &sql::ResultSet) -> Self {
        let pending = PendingEdits::new(result.rows.clone());
        let keys = (0..result.rows.len())
            .map(|row| ItemKey::num((row as u64).saturating_add(1)))
            .collect();
        let read_only_reason = (!result.editable)
            .then(|| "Read-only result: select a primary-key column to edit".to_owned());
        let mut grid = Self {
            types: result.columns.iter().map(|(_, ty)| *ty).collect(),
            pending,
            keys,
            total: result.total,
            editable: result.editable,
            source: result.source.clone(),
            read_only_reason,
            display: Vec::new(),
            undo: Vec::new(),
        };
        grid.rebuild_display();
        grid
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

    /// Number of pending row inserts/deletes and cell updates.
    pub fn pending_total(&self) -> usize {
        self.pending
            .dirty_rows()
            .len()
            .saturating_add(self.pending.dirty_cell_count())
    }

    /// Insert a row with typed NULL/default values.
    pub fn insert_row(&mut self) -> Option<usize> {
        if !self.editable {
            return None;
        }
        self.undo.push(self.pending.clone());
        let row = self.pending.insert_row(self.types.len());
        self.keys
            .push(ItemKey::num((self.keys.len() as u64).saturating_add(1)));
        self.rebuild_display();
        Some(row)
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

    /// Restore all cells and row lifecycle markers to the clean baseline.
    pub fn discard(&mut self) {
        self.undo.push(self.pending.clone());
        self.pending.clear();
        self.rebuild_display();
    }

    /// Restore the previous pending state, if one exists.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.pending = previous;
        self.rebuild_display();
        true
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
        let mut order: Vec<usize> = (0..self.row_count()).collect();
        order.sort_by(|&a, &b| {
            let left = self.pending.value(a, column).unwrap_or(&Value::Null);
            let right = self.pending.value(b, column).unwrap_or(&Value::Null);
            let cmp = sql::cmp_values(left, right);
            if direction == SortDir::Asc {
                cmp
            } else {
                cmp.reverse()
            }
        });
        self.pending.reorder(&order);
        self.keys = order
            .iter()
            .filter_map(|&row| self.keys.get(row).copied())
            .collect();
        self.rebuild_display();
    }

    fn rebuild_display(&mut self) {
        self.display = self
            .pending
            .current
            .iter()
            .flat_map(|row| {
                row.iter().map(|value| match value {
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
        self.keys
            .get(row)
            .copied()
            .unwrap_or_else(|| ItemKey::index(row))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.display_cell(row, col).map(CellRef::new)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        RowDecor {
            flags: if self.pending.is_dirty_row(row) {
                StateFlags::DIRTY
            } else {
                StateFlags::empty()
            },
            ..RowDecor::default()
        }
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        CellDecor {
            dirty: self.pending.is_dirty(row, col),
            ..CellDecor::default()
        }
    }

    fn total(&self) -> RowTotal {
        RowTotal::Exact(self.total())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> Vec<(String, ColType)> {
        vec![
            ("id".to_owned(), ColType::Int),
            ("name".to_owned(), ColType::Text),
        ]
    }

    #[test]
    fn pending_update_uses_original_key_and_escapes_text() {
        let table = CatalogTable::orders();
        let mut pending = PendingEdits::new(vec![vec![
            Value::Int(7),
            Value::Text("O'Reilly".to_owned()),
        ]]);
        assert!(pending.set(0, 1, Value::Text("new value".to_owned())));
        let sql = preview_sql(&table, &columns(), &pending);
        assert_eq!(
            sql,
            vec!["UPDATE public.orders SET name = 'new value' WHERE id = 7;".to_owned()]
        );
    }

    #[test]
    fn grid_adapter_marks_dirty_cells_and_parses_typed_edits() {
        let result = sql::ResultSet {
            columns: columns(),
            rows: vec![vec![Value::Int(1), Value::Text("Ada".to_owned())]],
            total: 1,
            source: Some("public.orders".to_owned()),
            duration_ms: 1,
            editable: true,
        };
        let mut grid = ResultGrid::from_result(&result);
        assert_eq!(grid.row_key(0), ItemKey::num(1));
        assert!(grid.commit_cell(0, 0, "2").is_ok());
        assert!(grid.pending().is_dirty(0, 0));
        assert_eq!(grid.cell(0, 0).map(|cell| cell.text), Some("2"));
        assert!(grid.commit_cell(0, 0, "not an int").is_err());
        assert_eq!(grid.cell(0, 0).map(|cell| cell.text), Some("2"));
    }

    #[test]
    fn sort_preserves_logical_row_keys() {
        let result = sql::ResultSet {
            columns: vec![("id".to_owned(), ColType::Int)],
            rows: vec![vec![Value::Int(20)], vec![Value::Int(10)]],
            total: 2,
            source: None,
            duration_ms: 1,
            editable: false,
        };
        let mut grid = ResultGrid::from_result(&result);
        let first_key = grid.row_key(0);
        grid.sort(ColumnKey::num(1), SortDir::Asc);
        assert_eq!(grid.cell(0, 0).map(|cell| cell.text), Some("10"));
        assert_eq!(grid.row_key(1), first_key);
    }

    struct CatalogTable;

    impl CatalogTable {
        fn orders() -> Table {
            crate::db::Catalog::acme_prod()
                .find(Some("public"), "orders")
                .cloned()
                .unwrap_or_else(|| Table {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    kind: crate::db::ObjectKind::Table,
                    columns: vec![crate::db::Column {
                        name: "id".to_owned(),
                        ty: ColType::Int,
                        nullable: false,
                        default: None,
                        primary: true,
                        references: None,
                        enum_values: Vec::new(),
                        generated: false,
                    }],
                    indexes: Vec::new(),
                    constraints: Vec::new(),
                    triggers: Vec::new(),
                    row_count: 1,
                    comment: None,
                })
        }
    }
}
