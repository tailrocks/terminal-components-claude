//! Explicit source-width sampling never scans hidden rows during draw.
use junie_tui::{CellRef, Column, ColumnKey, Grid, GridModel, GridState, Id, ItemKey, WidthSample};
const ID: Id = Id::root("sample.grid");
struct Model {
    widths: Vec<usize>,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.widths.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::index(row)
    }
    fn cell(&self, row: usize, _: usize) -> Option<CellRef<'_>> {
        self.widths
            .get(row)
            .map(|w| CellRef::new(&"abcdefghijklmnopqrstuvwxyz"[..*w]))
    }
}
#[test]
fn pinned_p95_index_and_first_200_sample_are_explicit() -> Result<(), junie_tui::WidthSampleError> {
    let model = Model {
        widths: (0..201)
            .map(|i| {
                if i < 190 {
                    3
                } else if i < 200 {
                    8
                } else {
                    26
                }
            })
            .collect(),
    };
    let columns = [Column::new(ColumnKey::num(7), "name")];
    let mut state = GridState::default();
    let sample = WidthSample::new(200, 95)?;
    Grid::new(ID, &columns).sample_column_widths(&mut state, &model, sample);
    assert_eq!(state.sampled_column_width(ColumnKey::num(7)), Some(8));
    state.clear_sampled_widths();
    assert_eq!(state.sampled_column_width(ColumnKey::num(7)), None);
    Ok(())
}
#[test]
fn invalid_sampling_parameters_are_rejected() {
    assert!(WidthSample::new(0, 95).is_err());
    assert!(WidthSample::new(201, 95).is_err());
    assert!(WidthSample::new(200, 101).is_err());
    assert!(WidthSample::new(1, 0).is_ok());
    assert!(WidthSample::new(200, 100).is_ok());
}

struct AuditModel {
    rows: Vec<Vec<String>>,
    calls: std::cell::RefCell<Vec<(usize, usize)>>,
}
impl GridModel for AuditModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::index(row)
    }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.calls.borrow_mut().push((row, col));
        self.rows.get(row)?.get(col).map(|text| CellRef::new(text))
    }
}
struct Page {
    model: AuditModel,
    columns: Vec<Column<'static>>,
    state: GridState,
    area: junie_tui::Rect,
}
impl junie_tui::App for Page {
    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> junie_tui::Response<()> {
        Grid::new(ID, &self.columns)
            .update(cx, &mut self.state, &self.model)
            .erase()
    }
    fn draw(&self, ui: &mut junie_tui::Ui<'_>) {
        Grid::new(ID, &self.columns).draw(ui, self.area, &self.state, &self.model);
    }
}
#[test]
fn cached_widths_survive_reorder_append_and_draw_only_reads_visible_rows()
-> Result<(), junie_tui::WidthSampleError> {
    let mut page = Page {
        model: AuditModel {
            rows: (0..201)
                .map(|i| {
                    vec![
                        "x".repeat(if i < 190 {
                            3
                        } else if i < 200 {
                            8
                        } else {
                            26
                        }),
                        "y".into(),
                    ]
                })
                .collect(),
            calls: std::cell::RefCell::new(Vec::new()),
        },
        columns: vec![
            Column::new(ColumnKey::num(0), "a"),
            Column::new(ColumnKey::num(1), "b"),
        ],
        state: GridState::default(),
        area: junie_tui::Rect::new(0, 0, 40, 5),
    };
    for c in &mut page.columns {
        c.min_width = 1;
        c.max_width = 40;
    }
    Grid::new(ID, &page.columns).sample_column_widths(
        &mut page.state,
        &page.model,
        WidthSample::new(200, 95)?,
    );
    assert_eq!(page.model.calls.borrow().len(), 400);
    page.model.calls.borrow_mut().clear();
    let mut h = junie_tui_testing::Harness::new(page, junie_tui::Theme::junie(), 40, 5);
    assert_eq!(h.cell(11, 0).symbol(), "b");
    assert!(h.app().model.calls.borrow().iter().all(|(row, _)| *row < 4));
    h.app_mut().model.rows.reverse();
    h.app_mut()
        .model
        .rows
        .push(vec!["abcdefghijklmnopqrstuvwxyz".into(), "z".into()]);
    h.app().model.calls.borrow_mut().clear();
    h.draw();
    assert_eq!(h.cell(11, 0).symbol(), "b");
    assert!(h.app().model.calls.borrow().iter().all(|(row, _)| *row < 4));
    h.app_mut().columns.swap(0, 1);
    h.draw();
    assert_eq!(h.cell(4, 0).symbol(), "a");
    assert_eq!(
        h.app().state.sampled_column_width(ColumnKey::num(0)),
        Some(8)
    );
    for width in 0..=3 {
        for height in 0..=3 {
            h.app_mut().area = junie_tui::Rect::new(1, 1, width, height);
            let _ = h.resize(6, 6);
            h.draw();
            assert_eq!(h.cell(0, 0).symbol(), " ");
            assert_eq!(h.cell(5, 5).symbol(), " ");
            assert_eq!(
                h.app().state.sampled_column_width(ColumnKey::num(0)),
                Some(8)
            );
        }
    }
    Ok(())
}

#[test]
fn percentile_endpoints_empty_rows_and_replacement_are_defined()
-> Result<(), junie_tui::WidthSampleError> {
    let columns = [Column::new(ColumnKey::num(0), "a")];
    let grid = Grid::new(ID, &columns);
    let model = Model {
        widths: vec![1, 2, 3, 4],
    };
    let mut state = GridState::default();
    for (percentile, expected) in [(0, 1), (50, 3), (100, 4)] {
        grid.sample_column_widths(&mut state, &model, WidthSample::new(200, percentile)?);
        assert_eq!(
            state.sampled_column_width(ColumnKey::num(0)),
            Some(expected)
        );
    }
    grid.sample_column_widths(
        &mut state,
        &Model { widths: vec![] },
        WidthSample::new(200, 95)?,
    );
    assert_eq!(state.sampled_column_width(ColumnKey::num(0)), Some(0));
    let replacement = [Column::new(ColumnKey::num(9), "new")];
    Grid::new(ID, &replacement).sample_column_widths(
        &mut state,
        &model,
        WidthSample::new(200, 95)?,
    );
    assert_eq!(state.sampled_column_width(ColumnKey::num(0)), None);
    assert_eq!(state.sampled_column_width(ColumnKey::num(9)), Some(4));
    Ok(())
}
#[test]
fn ragged_unicode_cells_use_terminal_width_and_column_cap()
-> Result<(), junie_tui::WidthSampleError> {
    let model = AuditModel {
        rows: vec![vec!["e\u{301}中".into()], vec![]],
        calls: std::cell::RefCell::new(Vec::new()),
    };
    let columns: Vec<_> = (0..65)
        .map(|i| Column::new(ColumnKey::num(i), "x"))
        .collect();
    let mut state = GridState::default();
    Grid::new(ID, &columns).sample_column_widths(&mut state, &model, WidthSample::new(2, 100)?);
    assert_eq!(state.sampled_column_width(ColumnKey::num(0)), Some(3));
    assert_eq!(state.sampled_column_width(ColumnKey::num(63)), Some(0));
    assert_eq!(state.sampled_column_width(ColumnKey::num(64)), None);
    assert_eq!(model.calls.borrow().len(), 128);
    Ok(())
}
