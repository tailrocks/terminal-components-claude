//! Application-owned grid metadata and the public adapter facade.

use crate::domain::{ResultGrid, preview_sql};

use crate::db::{ColType, Table};

/// Column metadata used by table grids and filter forms.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    dead_code,
    reason = "column metadata remains available to the private grid adapter"
)]
pub(crate) struct ColumnInfo {
    /// Column name.
    pub(crate) name: String,
    /// Database type.
    pub(crate) ty: ColType,
    /// Whether the database accepts null.
    pub(crate) nullable: bool,
    /// Whether this is part of the primary key.
    pub(crate) primary: bool,
    /// Optional foreign-key target.
    pub(crate) references: Option<(String, String)>,
    /// Optional default expression.
    pub(crate) default: Option<String>,
    /// Enum choices.
    pub(crate) enum_values: Vec<&'static str>,
}

#[expect(
    dead_code,
    reason = "column metadata remains available to the private grid adapter"
)]
impl ColumnInfo {
    /// Convert one catalog column to app metadata.
    pub(crate) fn from_column(column: &crate::db::Column) -> Self {
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
#[expect(
    dead_code,
    reason = "column metadata remains available to the private grid adapter"
)]
pub(crate) fn columns(table: &Table) -> Vec<ColumnInfo> {
    table.columns.iter().map(ColumnInfo::from_column).collect()
}

/// Column labels/types for the generic grid.
pub(crate) fn grid_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

/// Compact pending-change marker for the status bar.
#[expect(
    dead_code,
    reason = "pending marker remains available to the private grid adapter"
)]
pub(crate) fn pending_label(grid: &ResultGrid) -> String {
    match grid.pending_total() {
        0 => String::new(),
        n => format!("• {n} pending"),
    }
}

/// Renderable SQL preview rows.
pub fn preview_for(table: &Table, grid: &ResultGrid) -> Vec<String> {
    preview_sql(table, &grid_columns(table), grid.pending())
}
