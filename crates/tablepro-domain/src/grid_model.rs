//! Application-owned grid metadata and the public adapter facade.

use crate::domain::{ResultGrid, preview_sql};

use tablepro_demo::db::{ColType, Table};

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
        let rows: Vec<Vec<tablepro_demo::db::Value>> = table
            .columns
            .iter()
            .map(|column| {
                vec![
                    tablepro_demo::db::Value::Text(column.name.clone()),
                    tablepro_demo::db::Value::Text(column.ty.sql().to_owned()),
                    tablepro_demo::db::Value::Bool(column.nullable),
                    tablepro_demo::db::Value::Bool(column.primary),
                    tablepro_demo::db::Value::Text(column.default.clone().unwrap_or_default()),
                    tablepro_demo::db::Value::Text(
                        column
                            .references
                            .as_ref()
                            .map(|(target, name)| format!("{target}.{name}"))
                            .unwrap_or_default(),
                    ),
                ]
            })
            .collect();
        let result = tablepro_sql::sql::ResultSet {
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

impl termrock::GridModel for StructureModel {
    fn row_count(&self) -> usize {
        termrock::GridModel::row_count(&self.grid)
    }

    fn row_key(&self, row: usize) -> termrock::ItemKey {
        termrock::GridModel::row_key(&self.grid, row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<termrock::CellRef<'_>> {
        termrock::GridModel::cell(&self.grid, row, col)
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.read_only_reason()
    }
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
