use crate::domain::{ResultGrid, preview_sql};

use tablepro_demo::db::{ColType, Table};

pub type TableGridModel = ResultGrid;

pub type ResultGridModel = ResultGrid;

#[derive(Debug, Clone)]
pub struct StructureModel {
    grid: ResultGrid,
}

impl StructureModel {
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

pub fn grid_columns(table: &Table) -> Vec<(String, ColType)> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column.ty))
        .collect()
}

pub fn pending_label(grid: &ResultGrid) -> String {
    match grid.pending_total() {
        0 => String::new(),
        n => format!("• {n} pending"),
    }
}

pub fn preview_for(table: &Table, grid: &ResultGrid) -> Vec<String> {
    preview_sql(table, &grid_columns(table), grid.pending())
}
