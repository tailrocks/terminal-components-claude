//! Application-owned grid metadata and the public adapter facade.

use tablepro_domain::{ColType, Table, Value};
use tablepro_sql::preview_sql;
use termrock::{CellDecor, CellRef, GridModel, ItemKey};

use crate::domain::ResultGrid;

/// Read/write result adapter used by table data grid.
pub type TableGridModel = ResultGrid;

/// Read-only result adapter used by query and table-result views.
pub type ResultGridModel = ResultGrid;

/// Read-only structure metadata exposed through public grid contract.
#[derive(Debug, Clone)]
pub struct StructureModel {
    grid: ResultGrid,
}

impl StructureModel {
    /// Build six-column structure model for one catalog table.
    pub fn from_table(table: &Table) -> Self {
        let rows: Vec<Vec<Value>> = table
            .columns
            .iter()
            .map(|column| {
                vec![
                    Value::Text(column.name.clone()),
                    Value::Text(column.ty.sql().to_owned()),
                    Value::Bool(column.nullable),
                    Value::Bool(column.primary),
                    Value::Text(column.default.clone().unwrap_or_default()),
                    Value::Text(
                        column
                            .references
                            .as_ref()
                            .map(|(target, name)| format!("{target}.{name}"))
                            .unwrap_or_default(),
                    ),
                ]
            })
            .collect();
        let result = tablepro_domain::ResultSet {
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
    pub fn read_only_reason(&self) -> Option<&str> {
        Some("Structure metadata is read-only")
    }
}

impl GridModel for StructureModel {
    fn row_count(&self) -> usize {
        GridModel::row_count(&self.grid)
    }

    fn row_key(&self, row: usize) -> ItemKey {
        GridModel::row_key(&self.grid, row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        GridModel::cell(&self.grid, row, col)
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        GridModel::cell_decor(&self.grid, row, col)
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.read_only_reason()
    }
}

/// Column labels/types for generic grid.
pub fn grid_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

/// Compact pending-change marker for status bar.
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
