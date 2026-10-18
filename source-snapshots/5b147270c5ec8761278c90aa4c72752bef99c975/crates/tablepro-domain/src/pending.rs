//! Application-owned pending edits over a rectangular result set.

use crate::value::Value;

/// One row owns its identity, baseline and lifecycle through every transition.
#[derive(Clone, PartialEq)]
pub struct PendingRow {
    pub key: u64,
    pub current: Vec<Value>,
    pub original: Option<Vec<Value>>,
    pub inserted: bool,
    pub deleted: bool,
}

impl core::fmt::Debug for PendingRow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PendingRow")
            .field("key", &self.key)
            .field("columns", &self.current.len())
            .field("has_baseline", &self.original.is_some())
            .field("inserted", &self.inserted)
            .field("deleted", &self.deleted)
            .finish()
    }
}

/// Application-owned pending edits over a rectangular result set.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEdits {
    rows: Vec<PendingRow>,
    pub next_key: Option<u64>,
}

impl PendingEdits {
    /// Start tracking `rows` without changing them.
    pub fn new(rows: Vec<Vec<Value>>) -> Self {
        let next_key = u64::try_from(rows.len())
            .ok()
            .and_then(|n| n.checked_add(1));
        Self {
            rows: rows
                .into_iter()
                .enumerate()
                .map(|(index, current)| PendingRow {
                    key: (index as u64).saturating_add(1),
                    current,
                    original: None,
                    inserted: false,
                    deleted: false,
                })
                .collect(),
            next_key,
        }
    }

    /// Number of rows currently held by the result.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Borrow the underlying pending rows.
    pub fn rows(&self) -> &[PendingRow] {
        &self.rows
    }

    /// Stable key of one row.
    pub fn row_key(&self, row: usize) -> u64 {
        self.rows.get(row).map_or(0, |record| record.key)
    }

    /// Read one current cell.
    pub fn value(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row)?.current.get(col)
    }

    /// Read the baseline clean value for one cell.
    pub fn original_value(&self, row: usize, col: usize) -> Option<&Value> {
        let record = self.rows.get(row)?;
        record.original.as_ref().unwrap_or(&record.current).get(col)
    }

    /// Set one cell. Returns whether the value changed.
    pub fn set(&mut self, row: usize, col: usize, value: Value) -> bool {
        let Some(record) = self.rows.get_mut(row) else {
            return false;
        };
        let Some(cell) = record.current.get(col) else {
            return false;
        };
        if *cell == value {
            return false;
        }
        if !record.inserted && record.original.is_none() {
            record.original = Some(record.current.clone());
        }
        if let Some(cell) = record.current.get_mut(col) {
            *cell = value;
        }
        true
    }

    /// Width in columns of the specified row.
    pub fn current_row_width(&self, row: usize) -> usize {
        self.rows.get(row).map_or(0, |record| record.current.len())
    }

    /// Add a NULL row, refusing without mutation if stable keys are exhausted.
    pub fn insert_row(&mut self, columns: usize) -> Option<usize> {
        self.insert_values(vec![Value::Null; columns])
    }

    /// Insert specific values.
    pub fn insert_values(&mut self, values: Vec<Value>) -> Option<usize> {
        let key = self.next_key?;
        let row = self.rows.len();
        self.rows.push(PendingRow {
            key,
            current: values,
            original: None,
            inserted: true,
            deleted: false,
        });
        self.next_key = key.checked_add(1);
        Some(row)
    }

    /// Mark an existing row deleted, or remove a cancelled insertion.
    pub fn delete_row(&mut self, row: usize) -> bool {
        let Some(record) = self.rows.get_mut(row) else {
            return false;
        };
        if record.inserted {
            self.rows.remove(row);
            return true;
        }
        if record.deleted {
            return false;
        }
        record.deleted = true;
        true
    }

    /// Toggle row deletion.
    pub fn toggle_delete(&mut self, row: usize) -> bool {
        let Some(record) = self.rows.get_mut(row) else {
            return false;
        };
        if record.inserted {
            self.rows.remove(row);
        } else {
            if !record.deleted
                && let Some(original) = record.original.take()
            {
                record.current = original;
            }
            record.deleted = !record.deleted;
        }
        true
    }

    /// Discard edits for one row.
    pub fn discard_row(&mut self, row: usize) -> bool {
        let Some(record) = self.rows.get_mut(row) else {
            return false;
        };
        if record.inserted {
            self.rows.remove(row);
            return true;
        }
        if let Some(orig) = record.original.take() {
            record.current = orig;
        }
        record.deleted = false;
        true
    }

    /// Whether the row was inserted by the user.
    pub fn is_inserted(&self, row: usize) -> bool {
        self.rows.get(row).is_some_and(|record| record.inserted)
    }

    /// Whether the row was marked for deletion.
    pub fn is_deleted(&self, row: usize) -> bool {
        self.rows.get(row).is_some_and(|record| record.deleted)
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
            || (0..self.current_row_width(row)).any(|col| self.is_dirty(row, col))
    }

    /// Number of changed cells, excluding row lifecycle markers.
    pub fn dirty_cell_count(&self) -> usize {
        (0..self.row_count())
            .map(|row| {
                (0..self.current_row_width(row))
                    .filter(|&col| self.is_dirty(row, col))
                    .count()
            })
            .sum()
    }

    /// Row indexes with pending cell or row changes.
    pub fn dirty_rows(&self) -> Vec<usize> {
        (0..self.row_count())
            .filter(|&row| self.is_dirty_row(row))
            .collect()
    }

    /// Discard pending edits and restore the loaded clean rows.
    pub fn clear(&mut self) {
        self.rows.retain(|record| !record.inserted);
        for record in &mut self.rows {
            if let Some(original) = record.original.take() {
                record.current = original;
            }
            record.deleted = false;
        }
    }

    /// Commit pending changes into the clean baseline.
    pub fn commit(&mut self) {
        self.rows.retain(|record| !record.deleted);
        for record in &mut self.rows {
            record.original = None;
            record.inserted = false;
            record.deleted = false;
        }
    }

    /// Reorder whole records; invalid permutations leave the result unchanged.
    pub fn reorder(&mut self, order: &[usize]) {
        let mut seen = vec![false; self.rows.len()];
        if order.len() != self.rows.len()
            || order.iter().any(|&index| {
                let Some(slot) = seen.get_mut(index) else {
                    return true;
                };
                core::mem::replace(slot, true)
            })
        {
            return;
        }
        let mut records: Vec<_> = core::mem::take(&mut self.rows)
            .into_iter()
            .map(Some)
            .collect();
        self.rows = order
            .iter()
            .filter_map(|&index| records.get_mut(index).and_then(Option::take))
            .collect();
    }
}
