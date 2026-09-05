//! Application-owned grid metadata and the public adapter facade.

use crate::domain::{ResultGrid, preview_sql};

use crate::db::{ColType, Table};

/// The read/write result adapter used by a table data grid.
pub type TableGridModel = ResultGrid;

/// The read-only result adapter used by query and table-result views.
pub type ResultGridModel = ResultGrid;

/// Read-only structure metadata exposed through the public grid contract.
#[derive(Debug, Clone)]
pub struct StructureModel {
    grid: ResultGrid,
}

impl StructureModel {
    /// Build the six-column structure model for one catalog table.
    #[must_use]
    pub fn from_table(table: &Table) -> Self {
        let rows: Vec<Vec<crate::db::Value>> = table
            .columns
            .iter()
            .map(|column| {
                vec![
                    crate::db::Value::Text(column.name.clone()),
                    crate::db::Value::Text(column.ty.sql().to_owned()),
                    crate::db::Value::Bool(column.nullable),
                    crate::db::Value::Bool(column.primary),
                    crate::db::Value::Text(column.default.clone().unwrap_or_default()),
                    crate::db::Value::Text(
                        column
                            .references
                            .as_ref()
                            .map(|(target, name)| format!("{target}.{name}"))
                            .unwrap_or_default(),
                    ),
                ]
            })
            .collect();
        let result = crate::sql::ResultSet {
            columns: vec![
                ("name".to_owned(), ColType::Text),
                ("type".to_owned(), ColType::Text),
                ("nullable".to_owned(), ColType::Bool),
                ("primary".to_owned(), ColType::Bool),
                ("default".to_owned(), ColType::Text),
                ("references".to_owned(), ColType::Text),
            ],
            total: rows.len(),
            rows,
            source: Some(table.qualified()),
            duration_ms: 0,
            editable: false,
        };
        Self {
            grid: ResultGrid::from_result(&result),
        }
    }

    /// Explain why structure cells cannot be edited.
    #[must_use]
    pub fn read_only_reason(&self) -> Option<&str> {
        Some("Structure metadata is read-only")
    }
}

impl junie_tui::GridModel for StructureModel {
    fn row_count(&self) -> usize {
        junie_tui::GridModel::row_count(&self.grid)
    }

    fn row_key(&self, row: usize) -> junie_tui::ItemKey {
        junie_tui::GridModel::row_key(&self.grid, row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<junie_tui::CellRef<'_>> {
        junie_tui::GridModel::cell(&self.grid, row, col)
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.read_only_reason()
    }
}

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
