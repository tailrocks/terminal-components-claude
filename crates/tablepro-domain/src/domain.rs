#![allow(
    missing_docs,
    reason = "items promoted from pub(crate) during the crate split keep their original undocumented fixtures"
)]

use std::collections::{HashMap, HashSet};
use termrock::{
    CellDecor, CellRef, ColumnKey, EditIntent, FgStep, FieldError, GlyphRole, GridEditor,
    GridModel, ItemKey, Role, RowDecor, RowTotal, SortDir, StateFlags,
};

use tablepro_demo::db::{ColType, Column, Table, Value};
use tablepro_sql::sql;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IdentityType {
    Uuid,
    Text,
    Int,
    Numeric,
    Bool,
    Timestamp,
    Date,
    Json,
    Enum,
}

impl From<ColType> for IdentityType {
    fn from(value: ColType) -> Self {
        match value {
            ColType::Uuid => Self::Uuid,
            ColType::Text => Self::Text,
            ColType::Int => Self::Int,
            ColType::Numeric => Self::Numeric,
            ColType::Bool => Self::Bool,
            ColType::Timestamp => Self::Timestamp,
            ColType::Date => Self::Date,
            ColType::Json => Self::Json,
            ColType::Enum => Self::Enum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IdentityValue {
    Null,
    Text(String),
    Int(i64),
    Numeric(u64),
    Bool(bool),
    Json(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RowIdentity(Vec<(String, IdentityType, IdentityValue)>);

#[derive(Clone, PartialEq)]
struct PendingRow {
    key: ItemKey,
    current: Vec<Value>,
    original: Option<Vec<Value>>,
    inserted: bool,
    deleted: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub struct PendingEdits {
    rows: Vec<PendingRow>,

    next_key: Option<u64>,
}

impl PendingEdits {
    pub fn new(rows: Vec<Vec<Value>>) -> Self {
        let next_key = u64::try_from(rows.len())
            .ok()
            .and_then(|n| n.checked_add(1));
        Self {
            rows: rows
                .into_iter()
                .enumerate()
                .map(|(index, current)| PendingRow {
                    key: ItemKey::num((index as u64).saturating_add(1)),
                    current,
                    original: None,
                    inserted: false,
                    deleted: false,
                })
                .collect(),
            next_key,
        }
    }

    fn from_rows_with_keys(
        rows: Vec<Vec<Value>>,
        keys: Vec<ItemKey>,
        next_key: Option<u64>,
    ) -> Option<Self> {
        if rows.len() != keys.len() {
            return None;
        }
        let mut seen = HashSet::with_capacity(keys.len());
        if keys.iter().any(|key| !seen.insert(*key)) {
            return None;
        }
        Some(Self {
            rows: rows
                .into_iter()
                .zip(keys)
                .map(|(current, key)| PendingRow {
                    key,
                    current,
                    original: None,
                    inserted: false,
                    deleted: false,
                })
                .collect(),
            next_key,
        })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn value(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row)?.current.get(col)
    }

    fn original_value(&self, row: usize, col: usize) -> Option<&Value> {
        let record = self.rows.get(row)?;
        record.original.as_ref().unwrap_or(&record.current).get(col)
    }

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

    pub fn insert_row(&mut self, columns: usize) -> Option<usize> {
        self.insert_values(vec![Value::Null; columns])
    }

    fn insert_values(&mut self, values: Vec<Value>) -> Option<usize> {
        let key = self.next_key?;
        let row = self.rows.len();
        self.rows.push(PendingRow {
            key: ItemKey::num(key),
            current: values,
            original: None,
            inserted: true,
            deleted: false,
        });
        self.next_key = key.checked_add(1);
        Some(row)
    }

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

    fn toggle_delete(&mut self, row: usize) -> bool {
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

    pub fn is_inserted(&self, row: usize) -> bool {
        self.rows.get(row).is_some_and(|record| record.inserted)
    }

    pub fn is_deleted(&self, row: usize) -> bool {
        self.rows.get(row).is_some_and(|record| record.deleted)
    }

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

    pub fn is_dirty_row(&self, row: usize) -> bool {
        self.is_inserted(row)
            || self.is_deleted(row)
            || (0..self.current_row_width(row)).any(|col| self.is_dirty(row, col))
    }

    pub fn dirty_cell_count(&self) -> usize {
        (0..self.row_count())
            .map(|row| {
                (0..self.current_row_width(row))
                    .filter(|&col| self.is_dirty(row, col))
                    .count()
            })
            .sum()
    }

    pub fn dirty_rows(&self) -> Vec<usize> {
        (0..self.row_count())
            .filter(|&row| self.is_dirty_row(row))
            .collect()
    }

    pub fn clear(&mut self) {
        self.rows.retain(|record| !record.inserted);
        for record in &mut self.rows {
            if let Some(original) = record.original.take() {
                record.current = original;
            }
            record.deleted = false;
        }
    }

    fn commit(&mut self) -> (usize, usize) {
        let mut inserted = 0usize;
        let mut deleted = 0usize;
        self.rows.retain_mut(|record| {
            if record.deleted && !record.inserted {
                deleted = deleted.saturating_add(1);
                return false;
            }
            inserted = inserted.saturating_add(usize::from(record.inserted));
            record.original = None;
            record.inserted = false;
            record.deleted = false;
            true
        });
        (inserted, deleted)
    }

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

pub fn sql_literal(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_owned(),
        Value::Default => "DEFAULT".to_owned(),
        Value::Text(text) => format!("'{}'", text.replace('\'', "''")),
        Value::Int(value) => value.to_string(),
        Value::Num(value) => format!("{value:.2}"),
        Value::Bool(value) => value.to_string(),
        Value::Json(value) => format!("'{}'::jsonb", value.replace('\'', "''")),
    }
}

pub fn preview_sql(
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
                if !matches!(value, Value::Default) {
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
        self.rows.get(row).map_or(0, |record| record.current.len())
    }
}

#[derive(Clone, PartialEq)]
pub struct ResultGrid {
    types: Vec<ColType>,
    pending: PendingEdits,
    sort: Option<(ColumnKey, SortDir)>,
    total: usize,
    editable: bool,
    source: Option<String>,
    duration_ms: u32,
    read_only_reason: Option<String>,

    display: Vec<Option<String>>,
    undo: Vec<PendingEdits>,
}

fn unique_column_index(columns: &[(String, ColType)], name: &str) -> Option<usize> {
    let mut matches = columns
        .iter()
        .enumerate()
        .filter(|(_, (candidate, _))| candidate.eq_ignore_ascii_case(name));
    let index = matches.next()?.0;
    matches.next().is_none().then_some(index)
}

fn identity_layout<'a>(
    projected: &[(String, ColType)],
    target: &[&'a Column],
) -> Option<Vec<(usize, &'a Column)>> {
    let mut seen = HashSet::with_capacity(target.len());
    target
        .iter()
        .map(|column| {
            let index = unique_column_index(projected, &column.name)?;
            let (_, ty) = projected.get(index)?;
            (*ty == column.ty && seen.insert(index)).then_some((index, *column))
        })
        .collect()
}

fn is_complete_source_record(projected: &[(String, ColType)], table_columns: &[&Column]) -> bool {
    projected.len() == table_columns.len()
        && identity_layout(projected, table_columns)
            .is_some_and(|layout| layout.len() == table_columns.len())
}

fn identity_value(value: &Value, ty: ColType, allow_typed_null: bool) -> Option<IdentityValue> {
    match (value, ty) {
        (Value::Null, _) if allow_typed_null => Some(IdentityValue::Null),
        (
            Value::Text(value),
            ColType::Uuid | ColType::Text | ColType::Timestamp | ColType::Date | ColType::Enum,
        ) => Some(IdentityValue::Text(value.clone())),
        (Value::Int(value), ColType::Int) => Some(IdentityValue::Int(*value)),
        (Value::Num(value), ColType::Numeric) => Some(IdentityValue::Numeric(value.to_bits())),
        (Value::Bool(value), ColType::Bool) => Some(IdentityValue::Bool(*value)),
        (Value::Json(value), ColType::Json) => Some(IdentityValue::Json(value.clone())),
        _ => None,
    }
}

fn identity_from_row<'a>(
    layout: &[(usize, &'a Column)],
    mut value_at: impl FnMut(usize) -> Option<&'a Value>,
    allow_typed_null: bool,
) -> Option<RowIdentity> {
    layout
        .iter()
        .map(|(index, column)| {
            Some((
                column.name.to_ascii_lowercase(),
                IdentityType::from(column.ty),
                identity_value(value_at(*index)?, column.ty, allow_typed_null)?,
            ))
        })
        .collect::<Option<Vec<_>>>()
        .map(RowIdentity)
}

fn identities_for_grid(
    grid: &ResultGrid,
    columns: &[(String, ColType)],
    target: &[&Column],
    allow_typed_null: bool,
) -> Option<Vec<Option<RowIdentity>>> {
    let layout = identity_layout(columns, target)?;
    Some(
        (0..grid.row_count())
            .map(|row| {
                identity_from_row(
                    &layout,
                    |column| grid.pending.value(row, column),
                    allow_typed_null,
                )
            })
            .collect(),
    )
}

fn identities_for_result(
    result: &sql::ResultSet,
    target: &[&Column],
    allow_typed_null: bool,
) -> Option<Vec<Option<RowIdentity>>> {
    let layout = identity_layout(&result.columns, target)?;
    Some(
        result
            .rows
            .iter()
            .map(|row| identity_from_row(&layout, |column| row.get(column), allow_typed_null))
            .collect(),
    )
}

fn unique_identity_indices(
    identities: &[Option<RowIdentity>],
) -> HashMap<RowIdentity, Option<usize>> {
    let mut unique = HashMap::with_capacity(identities.len());
    for (row, identity) in identities.iter().enumerate() {
        let Some(identity) = identity else {
            continue;
        };
        if let Some(existing) = unique.get_mut(identity) {
            *existing = None;
        } else {
            unique.insert(identity.clone(), Some(row));
        }
    }
    unique
}

fn reuse_unique_row_keys(
    old_identities: &[Option<RowIdentity>],
    new_identities: &[Option<RowIdentity>],
    source: &ResultGrid,
    old_key_counts: &HashMap<ItemKey, usize>,
    row_keys: &mut [Option<ItemKey>],
    used_keys: &mut HashSet<ItemKey>,
) {
    let old_unique = unique_identity_indices(old_identities);
    let new_unique = unique_identity_indices(new_identities);
    for (identity, new_row) in new_unique {
        let Some(new_row) = new_row else {
            continue;
        };
        let Some(old_row) = old_unique.get(&identity).copied().flatten() else {
            continue;
        };
        let Some(slot) = row_keys.get_mut(new_row) else {
            continue;
        };
        if slot.is_some() {
            continue;
        }
        let old_key = source.row_key(old_row);
        if old_key_counts.get(&old_key) == Some(&1) && used_keys.insert(old_key) {
            *slot = Some(old_key);
        }
    }
}

fn allocate_row_key(
    next_key: &mut Option<u64>,
    used_keys: &mut HashSet<ItemKey>,
) -> Option<ItemKey> {
    let mut candidate = (*next_key)?;
    loop {
        let key = ItemKey::num(candidate);
        let next_candidate = candidate.checked_add(1);
        if used_keys.insert(key) {
            *next_key = next_candidate;
            return Some(key);
        }
        candidate = next_candidate?;
        *next_key = Some(candidate);
    }
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

    pub fn from_result(result: &sql::ResultSet) -> Self {
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
            duration_ms: result.duration_ms,
            read_only_reason,
            display: Vec::new(),
            undo: Vec::new(),
        };
        grid.rebuild_display();
        grid
    }

    /// Refresh a clean table result while carrying row keys across records whose identity is
    /// uniquely proven by a declared primary tuple or, when unavailable, an exact full row.
    /// Ambiguous rows receive fresh monotonic keys so filtering never aliases their selection.
    pub fn from_filter_refresh(
        result: &sql::ResultSet,
        source: &Self,
        source_columns: &[(String, ColType)],
        table: &Table,
    ) -> Option<Self> {
        if source.pending_total() != 0 {
            return None;
        }

        let mut row_keys = vec![None; result.rows.len()];
        let mut old_key_counts = HashMap::with_capacity(source.row_count());
        for row in 0..source.row_count() {
            let key = source.row_key(row);
            let count = old_key_counts.entry(key).or_insert(0usize);
            *count = count.saturating_add(1);
        }
        let mut used_keys = HashSet::with_capacity(result.rows.len());
        let same_source = table.qualified();
        if source.source() == Some(same_source.as_str())
            && result.source.as_deref() == Some(same_source.as_str())
        {
            let primary_columns: Vec<&Column> = table
                .columns
                .iter()
                .filter(|column| column.primary)
                .collect();
            // A NULL component cannot prove a primary-key identity, even in a unique result.
            if !primary_columns.is_empty()
                && let Some(old_identities) =
                    identities_for_grid(source, source_columns, &primary_columns, false)
                && let Some(new_identities) = identities_for_result(result, &primary_columns, false)
            {
                reuse_unique_row_keys(
                    &old_identities,
                    &new_identities,
                    source,
                    &old_key_counts,
                    &mut row_keys,
                    &mut used_keys,
                );
            }

            let table_columns: Vec<&Column> = table.columns.iter().collect();
            if row_keys.iter().any(Option::is_none)
                && is_complete_source_record(source_columns, &table_columns)
                && is_complete_source_record(&result.columns, &table_columns)
                // In a complete typed record, NULL is a known cell value and participates in
                // exact tuple equality when that tuple is unique in both results.
                && let Some(old_identities) =
                    identities_for_grid(source, source_columns, &table_columns, true)
                && let Some(new_identities) = identities_for_result(result, &table_columns, true)
            {
                reuse_unique_row_keys(
                    &old_identities,
                    &new_identities,
                    source,
                    &old_key_counts,
                    &mut row_keys,
                    &mut used_keys,
                );
            }
        }

        let mut next_key = source.pending.next_key;
        let mut keys = Vec::with_capacity(row_keys.len());
        for row_key in row_keys {
            let key = if let Some(key) = row_key {
                key
            } else {
                allocate_row_key(&mut next_key, &mut used_keys)?
            };
            keys.push(key);
        }

        let pending = PendingEdits::from_rows_with_keys(result.rows.clone(), keys, next_key)?;
        let sort = source.sort.and_then(|(old_key, direction)| {
            let old_column = usize::from(old_key.raw()).checked_sub(1)?;
            let name = source_columns.get(old_column)?.0.as_str();
            let new_column = unique_column_index(&result.columns, name)?;
            let key = u16::try_from(new_column.checked_add(1)?).ok()?;
            Some((ColumnKey::num(key), direction))
        });
        let mut grid = Self {
            types: result.columns.iter().map(|(_, ty)| *ty).collect(),
            pending,
            sort,
            total: result.total,
            editable: result.editable,
            source: result.source.clone(),
            duration_ms: result.duration_ms,
            read_only_reason: (!result.editable)
                .then(|| "Read-only result: select a primary-key column to edit".to_owned()),
            display: Vec::new(),
            undo: Vec::new(),
        };
        grid.rebuild_display();
        if let Some((column, direction)) = sort {
            grid.sort(column, direction);
        }
        Some(grid)
    }

    pub fn row_count(&self) -> usize {
        self.pending.row_count()
    }

    pub fn total(&self) -> usize {
        self.total.max(self.row_count())
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub const fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    pub fn pending(&self) -> &PendingEdits {
        &self.pending
    }

    pub fn pending_total(&self) -> usize {
        self.pending
            .rows
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

    pub fn duplicate_row(&mut self, row: usize, defaults: &[bool]) -> Option<usize> {
        if !self.editable || defaults.len() != self.types.len() {
            return None;
        }
        let source = self.pending.rows.get(row)?.current.clone();
        if source.len() != defaults.len() {
            return None;
        }
        let values = source
            .into_iter()
            .zip(defaults)
            .map(
                |(value, default)| {
                    if *default { Value::Default } else { value }
                },
            )
            .collect();
        let before = self.pending.clone();
        let inserted = self.pending.insert_values(values)?;
        self.undo.push(before);
        self.rebuild_display();
        Some(inserted)
    }

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

    pub fn discard(&mut self) {
        self.undo.push(self.pending.clone());
        self.pending.clear();
        self.restore_sort();
    }

    /// Applies staged changes to this in-memory result and makes them its new baseline.
    ///
    /// Existing row keys and current values survive. Deleted rows are removed, inserted
    /// rows become ordinary rows, and undo entries from before the commit are cleared.
    pub fn commit_pending(&mut self) -> bool {
        if !self.editable || self.pending_total() == 0 {
            return false;
        }
        let (inserted, deleted) = self.pending.commit();
        self.total = self.total.saturating_add(inserted).saturating_sub(deleted);
        self.undo.clear();
        self.restore_sort();
        true
    }

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

    pub fn dirty_cell_count(&self) -> usize {
        self.pending.dirty_cell_count()
    }

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
            let cmp = sql::cmp_values(left, right);
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
            .rows
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
            .rows
            .get(row)
            .map_or_else(|| ItemKey::index(row), |record| record.key)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.display_cell(row, col).map(CellRef::new)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        RowDecor {
            marker: if self.pending.is_deleted(row) {
                Some(GlyphRole::Deleted)
            } else if self.pending.is_inserted(row) {
                Some(GlyphRole::Inserted)
            } else {
                None
            },
            strike: self.pending.is_deleted(row),
            tone: self
                .pending
                .is_deleted(row)
                .then_some(Role::Fg(FgStep::Faint)),
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

    fn test_column(name: &str, ty: ColType, primary: bool) -> Column {
        Column {
            name: name.to_owned(),
            ty,
            nullable: !primary,
            default: None,
            primary,
            references: None,
            enum_values: Vec::new(),
            generated: false,
        }
    }

    fn test_table(columns: Vec<Column>) -> Table {
        Table {
            schema: "public".to_owned(),
            name: "records".to_owned(),
            kind: tablepro_demo::db::ObjectKind::Table,
            columns,
            indexes: Vec::new(),
            constraints: Vec::new(),
            triggers: Vec::new(),
            row_count: 0,
            comment: None,
        }
    }

    fn test_result(columns: Vec<(String, ColType)>, rows: Vec<Vec<Value>>) -> sql::ResultSet {
        sql::ResultSet {
            columns,
            total: rows.len(),
            rows,
            source: Some("public.records".to_owned()),
            duration_ms: 1,
            editable: true,
        }
    }

    #[test]
    fn exhausted_insert_is_atomic_and_undo_never_reuses_keys() {
        let mut grid = ResultGrid::empty();
        grid.editable = true;
        grid.pending.next_key = Some(u64::MAX);
        let Some(row) = grid.insert_row() else {
            unreachable!("last key");
        };
        assert_eq!(grid.row_key(row), ItemKey::num(u64::MAX));
        assert!(grid.undo());
        let before = grid.clone();
        assert_eq!(grid.insert_row(), None);
        assert_eq!(grid, before);
    }

    #[test]
    fn sorted_insert_update_delete_sql_uses_each_records_original_key() {
        let mut pending = PendingEdits::new(vec![
            vec![Value::Int(7), Value::Text("old".into())],
            vec![Value::Int(9), Value::Text("other".into())],
        ]);
        let Some(inserted) = pending.insert_row(2) else {
            unreachable!("key");
        };
        assert!(pending.set(inserted, 0, Value::Int(1)));
        pending.reorder(&[2, 1, 0]);
        assert!(pending.set(2, 0, Value::Int(8)));
        assert!(pending.delete_row(1));
        assert_eq!(
            preview_sql(&CatalogTable::orders(), &columns(), &pending),
            vec![
                "UPDATE public.orders SET id = 8 WHERE id = 7;",
                "INSERT INTO public.orders (id, name) VALUES (1, NULL);",
                "DELETE FROM public.orders WHERE id = 9;",
            ]
        );
        pending.clear();
        assert_eq!(pending.value(0, 0), Some(&Value::Int(9)));
        assert_eq!(pending.value(1, 0), Some(&Value::Int(7)));
        assert_eq!(pending.dirty_rows(), Vec::<usize>::new());
    }

    #[test]
    fn inserted_default_is_distinct_from_explicit_null_in_preview() {
        let mut grid = ResultGrid::from_result(&sql::ResultSet {
            columns: columns(),
            rows: Vec::new(),
            total: 0,
            source: Some("public.orders".to_owned()),
            duration_ms: 0,
            editable: true,
        });
        let before = grid.clone();
        assert!(grid.insert_row_with_defaults(&[true]).is_none());
        assert_eq!(grid, before);
        let Some(row) = grid.insert_row_with_defaults(&[true, false]) else {
            unreachable!("insert")
        };
        assert_eq!(grid.pending().value(row, 0), Some(&Value::Default));
        assert_eq!(grid.pending().value(row, 1), Some(&Value::Null));
        assert_eq!(
            preview_sql(&CatalogTable::orders(), &columns(), grid.pending()),
            vec!["INSERT INTO public.orders (name) VALUES (NULL);"]
        );
        assert!(grid.undo());
        assert!(grid.insert_row_with_defaults(&[true, true]).is_some());
        assert_eq!(
            preview_sql(&CatalogTable::orders(), &columns(), grid.pending()),
            vec!["INSERT INTO public.orders DEFAULT VALUES;"]
        );
    }

    #[test]
    fn duplicate_row_copies_values_and_defaults_key_columns() {
        let mut grid = ResultGrid::from_result(&sql::ResultSet {
            columns: columns(),
            rows: vec![vec![Value::Int(7), Value::Text("kept".to_owned())]],
            total: 1,
            source: Some("public.orders".to_owned()),
            duration_ms: 0,
            editable: true,
        });
        let Some(row) = grid.duplicate_row(0, &[true, false]) else {
            unreachable!("duplicate")
        };
        assert_eq!(row, 1);
        assert_eq!(grid.pending().value(row, 0), Some(&Value::Default));
        assert_eq!(
            grid.pending().value(row, 1),
            Some(&Value::Text("kept".to_owned()))
        );
        assert_eq!(
            grid.pending().value(0, 1),
            Some(&Value::Text("kept".to_owned()))
        );
        assert!(grid.undo());
        assert_eq!(grid.row_count(), 1);
    }

    #[test]
    fn delete_toggle_replaces_updates_and_undo_restores_full_row_snapshot() {
        let mut grid = ResultGrid::from_result(&sql::ResultSet {
            columns: columns(),
            rows: vec![vec![Value::Int(7), Value::Text("old".to_owned())]],
            total: 1,
            source: Some("public.orders".to_owned()),
            duration_ms: 0,
            editable: true,
        });
        let key = grid.row_key(0);
        assert!(grid.commit_cell(0, 1, "edited").is_ok());
        assert!(grid.toggle_delete(0));
        assert_eq!(grid.pending_total(), 1);
        assert_eq!(
            grid.pending().value(0, 1),
            Some(&Value::Text("old".to_owned()))
        );
        assert!(grid.undo());
        assert_eq!(grid.row_key(0), key);
        assert_eq!(
            grid.pending().value(0, 1),
            Some(&Value::Text("edited".to_owned()))
        );
        assert!(!grid.pending().is_deleted(0));
        assert!(grid.toggle_delete(0));
        assert!(grid.toggle_delete(0));
        assert_eq!(grid.pending_total(), 0);
        assert_eq!(grid.row_key(0), key);
    }

    fn columns() -> Vec<(String, ColType)> {
        vec![
            ("id".to_owned(), ColType::Int),
            ("name".to_owned(), ColType::Text),
        ]
    }

    #[test]
    fn preview_sql_orders_updates_inserts_deletes() -> Result<(), String> {
        let catalog = tablepro_demo::db::Catalog::acme_prod();
        let table = catalog.find(None, "orders").ok_or("orders missing")?;
        let columns = table
            .columns
            .iter()
            .map(|column| (column.name.clone(), column.ty))
            .collect::<Vec<_>>();
        let mut pending = PendingEdits::new(tablepro_demo::db::rows(table, 0, 3));
        let status = columns
            .iter()
            .position(|(name, _)| name == "status")
            .ok_or("status missing")?;
        let original = pending
            .value(1, status)
            .cloned()
            .ok_or("original status missing")?;
        assert!(pending.set(1, status, Value::Text("shipped".to_owned())));
        assert!(pending.delete_row(2));
        let sql = preview_sql(table, &columns, &pending);
        assert_eq!(sql.len(), 2);
        assert!(sql.first().is_some_and(|statement| {
            statement.starts_with("UPDATE public.orders SET status = 'shipped' WHERE id = '")
        }));
        assert!(sql.get(1).is_some_and(|statement| {
            statement.starts_with("DELETE FROM public.orders WHERE id = '")
        }));
        assert!(pending.set(1, status, original));
        let reverted = preview_sql(table, &columns, &pending);
        assert_eq!(reverted.len(), 1);
        assert_eq!(reverted.first(), sql.get(1));
        Ok(())
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

    #[test]
    fn filter_refresh_matches_composite_primary_keys_by_typed_columns() -> Result<(), String> {
        let table = test_table(vec![
            test_column("left_key", ColType::Text, true),
            test_column("right_key", ColType::Text, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("left_key".to_owned(), ColType::Text),
                ("right_key".to_owned(), ColType::Text),
                ("label".to_owned(), ColType::Text),
            ],
            vec![
                vec![
                    Value::Text("ab".to_owned()),
                    Value::Text("c".to_owned()),
                    Value::Text("first".to_owned()),
                ],
                vec![
                    Value::Text("a".to_owned()),
                    Value::Text("bc".to_owned()),
                    Value::Text("second".to_owned()),
                ],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(
            vec![
                ("label".to_owned(), ColType::Text),
                ("right_key".to_owned(), ColType::Text),
                ("left_key".to_owned(), ColType::Text),
            ],
            vec![vec![
                Value::Text("second".to_owned()),
                Value::Text("bc".to_owned()),
                Value::Text("a".to_owned()),
            ]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("a filtered table result remains available")?;
        assert_eq!(refreshed.row_count(), 1);
        assert_eq!(refreshed.row_key(0), ItemKey::num(2));
        Ok(())
    }

    #[test]
    fn filter_refresh_remaps_model_sort_by_column_name_after_projection_reorder()
    -> Result<(), String> {
        let table = test_table(vec![
            test_column("id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("id".to_owned(), ColType::Int),
                ("label".to_owned(), ColType::Text),
            ],
            vec![
                vec![Value::Int(1), Value::Text("zulu".to_owned())],
                vec![Value::Int(2), Value::Text("alpha".to_owned())],
                vec![Value::Int(3), Value::Text("mike".to_owned())],
            ],
        );
        let mut old_grid = ResultGrid::from_result(&old);
        old_grid.sort(ColumnKey::num(2), SortDir::Asc);
        let new = test_result(
            vec![
                ("label".to_owned(), ColType::Text),
                ("id".to_owned(), ColType::Int),
            ],
            vec![
                vec![Value::Text("mike".to_owned()), Value::Int(3)],
                vec![Value::Text("zulu".to_owned()), Value::Int(1)],
                vec![Value::Text("alpha".to_owned()), Value::Int(2)],
            ],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("the filtered projection keeps a valid model")?;
        assert_eq!(refreshed.cell(0, 0).map(|cell| cell.text), Some("alpha"));
        assert_eq!(refreshed.row_key(0), ItemKey::num(2));
        assert_eq!(refreshed.cell(1, 0).map(|cell| cell.text), Some("mike"));
        assert_eq!(refreshed.row_key(1), ItemKey::num(3));
        assert_eq!(refreshed.cell(2, 0).map(|cell| cell.text), Some("zulu"));
        assert_eq!(refreshed.row_key(2), ItemKey::num(1));
        assert_eq!(refreshed.sort, Some((ColumnKey::num(1), SortDir::Asc)));
        Ok(())
    }

    #[test]
    fn filter_refresh_does_not_match_a_malformed_value_variant_to_a_typed_key() -> Result<(), String>
    {
        let table = test_table(vec![test_column("id", ColType::Int, true)]);
        let old = test_result(
            vec![("id".to_owned(), ColType::Int)],
            vec![vec![Value::Text("7".to_owned())]],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(old.columns.clone(), vec![vec![Value::Int(7)]]);

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("malformed identity values do not make filtering fail")?;
        assert_eq!(refreshed.row_key(0), ItemKey::num(2));
        assert_ne!(refreshed.row_key(0), old_grid.row_key(0));
        Ok(())
    }

    #[test]
    fn filter_refresh_keeps_exact_numeric_identity_not_rounded_display() -> Result<(), String> {
        let table = test_table(vec![
            test_column("amount", ColType::Numeric, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("amount".to_owned(), ColType::Numeric),
                ("label".to_owned(), ColType::Text),
            ],
            vec![
                vec![Value::Num(1.231), Value::Text("first".to_owned())],
                vec![Value::Num(1.234), Value::Text("second".to_owned())],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        assert_eq!(old_grid.cell(0, 0).map(|cell| cell.text), Some("1.23"));
        assert_eq!(old_grid.cell(1, 0).map(|cell| cell.text), Some("1.23"));
        let new = test_result(
            old.columns.clone(),
            vec![vec![Value::Num(1.234), Value::Text("second".to_owned())]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("the new result has a valid identity")?;
        assert_eq!(refreshed.row_key(0), ItemKey::num(2));
        Ok(())
    }

    #[test]
    fn filter_refresh_preserves_unique_full_rows_but_never_guesses_duplicate_occurrences()
    -> Result<(), String> {
        let table = test_table(vec![
            test_column("code", ColType::Text, false),
            test_column("value", ColType::Int, false),
        ]);
        let old = test_result(
            vec![
                ("code".to_owned(), ColType::Text),
                ("value".to_owned(), ColType::Int),
            ],
            vec![
                vec![Value::Text("same".to_owned()), Value::Int(4)],
                vec![Value::Text("other".to_owned()), Value::Int(9)],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let unique = test_result(
            vec![
                ("value".to_owned(), ColType::Int),
                ("code".to_owned(), ColType::Text),
            ],
            vec![vec![Value::Int(9), Value::Text("other".to_owned())]],
        );
        let unique_grid = ResultGrid::from_filter_refresh(&unique, &old_grid, &old.columns, &table)
            .ok_or("a complete unique row matches without a primary key")?;
        assert_eq!(unique_grid.row_key(0), ItemKey::num(2));

        let duplicates = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Text("same".to_owned()), Value::Int(4)],
                vec![Value::Text("same".to_owned()), Value::Int(4)],
            ],
        );
        let duplicate_grid =
            ResultGrid::from_filter_refresh(&duplicates, &old_grid, &old.columns, &table)
                .ok_or("duplicate source rows remain present in the filtered result")?;
        assert_eq!(duplicate_grid.row_count(), 2);
        assert_eq!(duplicate_grid.row_key(0), ItemKey::num(3));
        assert_eq!(duplicate_grid.row_key(1), ItemKey::num(4));
        assert_ne!(duplicate_grid.row_key(0), duplicate_grid.row_key(1));
        Ok(())
    }

    #[test]
    fn filter_refresh_uses_full_rows_when_primary_values_are_null_or_duplicated()
    -> Result<(), String> {
        let table = test_table(vec![
            test_column("id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("id".to_owned(), ColType::Int),
                ("label".to_owned(), ColType::Text),
            ],
            vec![
                vec![Value::Int(1), Value::Text("first".to_owned())],
                vec![Value::Null, Value::Text("null-key".to_owned())],
                vec![Value::Int(7), Value::Text("duplicate-a".to_owned())],
                vec![Value::Int(7), Value::Text("duplicate-b".to_owned())],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Null, Value::Text("null-key".to_owned())],
                vec![Value::Int(7), Value::Text("duplicate-b".to_owned())],
            ],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("non-unique primary values do not reject filtering")?;
        assert_eq!(refreshed.row_key(0), ItemKey::num(2));
        assert_eq!(refreshed.row_key(1), ItemKey::num(4));
        Ok(())
    }

    #[test]
    fn filter_refresh_does_not_reuse_null_primary_key_for_changed_payload() -> Result<(), String> {
        let table = test_table(vec![
            test_column("id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("id".to_owned(), ColType::Int),
                ("label".to_owned(), ColType::Text),
            ],
            vec![vec![Value::Null, Value::Text("A".to_owned())]],
        );
        let old_grid = ResultGrid::from_result(&old);
        let selected_key = old_grid.row_key(0);
        let new = test_result(
            old.columns.clone(),
            vec![vec![Value::Null, Value::Text("B".to_owned())]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("a row with a null primary value remains filterable")?;
        assert_eq!(refreshed.row_count(), 1);
        assert_eq!(
            refreshed.pending().value(0, 1),
            Some(&Value::Text("B".to_owned()))
        );
        assert_ne!(refreshed.row_key(0), selected_key);
        assert!((0..refreshed.row_count()).all(|row| refreshed.row_key(row) != selected_key));
        Ok(())
    }

    #[test]
    fn filter_refresh_rejects_composite_primary_tuple_with_null_component() -> Result<(), String> {
        let table = test_table(vec![
            test_column("tenant_id", ColType::Int, true),
            test_column("record_id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("tenant_id".to_owned(), ColType::Int),
                ("record_id".to_owned(), ColType::Int),
                ("label".to_owned(), ColType::Text),
            ],
            vec![vec![
                Value::Int(7),
                Value::Null,
                Value::Text("A".to_owned()),
            ]],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(
            old.columns.clone(),
            vec![vec![
                Value::Int(7),
                Value::Null,
                Value::Text("B".to_owned()),
            ]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("a row with an incomplete composite primary tuple remains filterable")?;
        assert_ne!(refreshed.row_key(0), old_grid.row_key(0));
        Ok(())
    }

    #[test]
    fn filter_refresh_uses_unique_null_full_row_after_null_primary_ambiguity() -> Result<(), String>
    {
        let table = test_table(vec![
            test_column("id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("id".to_owned(), ColType::Int),
                ("label".to_owned(), ColType::Text),
            ],
            vec![
                vec![Value::Null, Value::Text("A".to_owned())],
                vec![Value::Null, Value::Text("B".to_owned())],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(
            old.columns.clone(),
            vec![vec![Value::Null, Value::Text("B".to_owned())]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("a unique complete record with null data remains filterable")?;
        assert_eq!(refreshed.row_key(0), old_grid.row_key(1));
        Ok(())
    }

    #[test]
    fn filter_refresh_matches_unique_no_primary_null_row_but_not_duplicate_occurrences()
    -> Result<(), String> {
        let table = test_table(vec![
            test_column("code", ColType::Text, false),
            test_column("note", ColType::Text, false),
        ]);
        let old = test_result(
            vec![
                ("code".to_owned(), ColType::Text),
                ("note".to_owned(), ColType::Text),
            ],
            vec![
                vec![Value::Text("survivor".to_owned()), Value::Null],
                vec![
                    Value::Text("other".to_owned()),
                    Value::Text("kept".to_owned()),
                ],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let reordered = test_result(
            vec![
                ("note".to_owned(), ColType::Text),
                ("code".to_owned(), ColType::Text),
            ],
            vec![vec![Value::Null, Value::Text("survivor".to_owned())]],
        );

        let refreshed =
            ResultGrid::from_filter_refresh(&reordered, &old_grid, &old.columns, &table)
                .ok_or("a unique complete no-primary-key row with null data remains filterable")?;
        assert_eq!(refreshed.row_key(0), old_grid.row_key(0));

        let duplicate_old = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Text("duplicate".to_owned()), Value::Null],
                vec![Value::Text("duplicate".to_owned()), Value::Null],
            ],
        );
        let duplicate_grid = ResultGrid::from_result(&duplicate_old);
        let duplicate_new = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Text("duplicate".to_owned()), Value::Null],
                vec![Value::Text("duplicate".to_owned()), Value::Null],
            ],
        );
        let refreshed_duplicates = ResultGrid::from_filter_refresh(
            &duplicate_new,
            &duplicate_grid,
            &duplicate_old.columns,
            &table,
        )
        .ok_or("ambiguous duplicate null rows remain filterable")?;
        assert_eq!(refreshed_duplicates.row_count(), 2);
        assert_eq!(refreshed_duplicates.row_key(0), ItemKey::num(3));
        assert_eq!(refreshed_duplicates.row_key(1), ItemKey::num(4));
        assert_ne!(
            refreshed_duplicates.row_key(0),
            refreshed_duplicates.row_key(1)
        );
        assert_ne!(refreshed_duplicates.row_key(0), duplicate_grid.row_key(0));
        assert_ne!(refreshed_duplicates.row_key(1), duplicate_grid.row_key(1));
        Ok(())
    }

    #[test]
    fn filter_refresh_keeps_rows_filterable_when_primary_column_is_not_projected()
    -> Result<(), String> {
        let table = test_table(vec![
            test_column("id", ColType::Int, true),
            test_column("label", ColType::Text, false),
        ]);
        let old = test_result(
            vec![("label".to_owned(), ColType::Text)],
            vec![
                vec![Value::Text("first".to_owned())],
                vec![Value::Text("second".to_owned())],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let new = test_result(
            old.columns.clone(),
            vec![vec![Value::Text("second".to_owned())]],
        );

        let refreshed = ResultGrid::from_filter_refresh(&new, &old_grid, &old.columns, &table)
            .ok_or("a missing identity projection does not block filtering")?;
        assert_eq!(refreshed.row_count(), 1);
        assert_eq!(refreshed.row_key(0), ItemKey::num(3));
        Ok(())
    }

    #[test]
    fn filter_refresh_keeps_monotonic_keys_across_shrink_widen_and_insert() -> Result<(), String> {
        let table = test_table(vec![test_column("id", ColType::Int, true)]);
        let old = test_result(
            vec![("id".to_owned(), ColType::Int)],
            vec![
                vec![Value::Int(1)],
                vec![Value::Int(2)],
                vec![Value::Int(3)],
            ],
        );
        let old_grid = ResultGrid::from_result(&old);
        let narrow = test_result(
            old.columns.clone(),
            vec![vec![Value::Int(1)], vec![Value::Int(3)]],
        );
        let narrow_grid = ResultGrid::from_filter_refresh(&narrow, &old_grid, &old.columns, &table)
            .ok_or("shrink result")?;
        assert_eq!(narrow_grid.row_key(0), ItemKey::num(1));
        assert_eq!(narrow_grid.row_key(1), ItemKey::num(3));

        let wide = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Int(1)],
                vec![Value::Int(2)],
                vec![Value::Int(3)],
                vec![Value::Int(4)],
            ],
        );
        let mut wide_grid =
            ResultGrid::from_filter_refresh(&wide, &narrow_grid, &narrow.columns, &table)
                .ok_or("widen result")?;
        assert_eq!(wide_grid.row_key(0), ItemKey::num(1));
        assert_eq!(wide_grid.row_key(1), ItemKey::num(4));
        assert_eq!(wide_grid.row_key(2), ItemKey::num(3));
        assert_eq!(wide_grid.row_key(3), ItemKey::num(5));
        assert_eq!(wide_grid.insert_row(), Some(4));
        assert_eq!(wide_grid.row_key(4), ItemKey::num(6));
        Ok(())
    }

    #[test]
    fn filter_refresh_allocator_skips_live_collisions_and_stops_at_key_overflow()
    -> Result<(), String> {
        let table = test_table(vec![test_column("id", ColType::Int, true)]);
        let old = test_result(
            vec![("id".to_owned(), ColType::Int)],
            vec![vec![Value::Int(1)]],
        );
        let mut old_grid = ResultGrid::from_result(&old);
        old_grid.pending.next_key = Some(1);
        let added = test_result(
            old.columns.clone(),
            vec![vec![Value::Int(1)], vec![Value::Int(2)]],
        );
        let refreshed = ResultGrid::from_filter_refresh(&added, &old_grid, &old.columns, &table)
            .ok_or("allocator advances beyond a live-key collision")?;
        assert_eq!(refreshed.row_key(0), ItemKey::num(1));
        assert_eq!(refreshed.row_key(1), ItemKey::num(2));

        old_grid.pending.next_key = Some(u64::MAX);
        let last_key = ResultGrid::from_filter_refresh(&added, &old_grid, &old.columns, &table)
            .ok_or("the final nonzero key can be assigned")?;
        assert_eq!(last_key.row_key(1), ItemKey::num(u64::MAX));
        let too_many = test_result(
            old.columns.clone(),
            vec![
                vec![Value::Int(1)],
                vec![Value::Int(2)],
                vec![Value::Int(3)],
            ],
        );
        assert!(
            ResultGrid::from_filter_refresh(&too_many, &old_grid, &old.columns, &table).is_none(),
            "key exhaustion does not publish a partial row-key assignment"
        );
        Ok(())
    }

    struct CatalogTable;

    impl CatalogTable {
        fn orders() -> Table {
            tablepro_demo::db::Catalog::acme_prod()
                .find(Some("public"), "orders")
                .cloned()
                .unwrap_or_else(|| Table {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    kind: tablepro_demo::db::ObjectKind::Table,
                    columns: vec![Column {
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
