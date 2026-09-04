//! Application-owned grid metadata and the public adapter facade.

pub use crate::domain::{PendingEdits, ResultGrid, preview_sql, sql_literal};

use crate::db::{ColType, Table};

/// Column metadata used by table grids and filter forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    /// Column name.
    pub name: String,
    /// Database type.
    pub ty: ColType,
    /// Whether the database accepts null.
    pub nullable: bool,
    /// Whether this is part of the primary key.
    pub primary: bool,
    /// Optional foreign-key target.
    pub references: Option<(String, String)>,
    /// Optional default expression.
    pub default: Option<String>,
    /// Enum choices.
    pub enum_values: Vec<&'static str>,
}

impl ColumnInfo {
    /// Convert one catalog column to app metadata.
    pub fn from_column(column: &crate::db::Column) -> Self {
        Self {
            name: column.name.clone(),
            ty: column.ty,
            nullable: column.nullable,
            primary: column.primary,
            references: column.references.clone(),
            default: column.default.clone(),
            enum_values: column.enum_values.clone(),
        }
    }
}

/// Derive all metadata needed by a table editor.
pub fn columns(table: &Table) -> Vec<ColumnInfo> {
    table.columns.iter().map(ColumnInfo::from_column).collect()
}

/// Column labels/types for the generic grid.
pub fn grid_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

/// Compact pending-change marker for the status bar.
pub fn pending_label(grid: &ResultGrid) -> String {
    match grid.pending_total() {
        0 => String::new(),
        n => format!("• {n} pending"),
    }
}

/// Renderable SQL preview rows.
pub fn preview_for(table: &Table, grid: &ResultGrid) -> Vec<String> {
    preview_sql(table, &grid_columns(table), grid.pending())
}
