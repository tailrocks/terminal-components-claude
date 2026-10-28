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
            sql::cmp_values_nulls_last(left, right, direction == SortDir::Asc)
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
