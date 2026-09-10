//! Natural samples stay frozen; visible action constraints remain live.
use junie_tui::*;
use junie_tui_testing::Harness;
const ID: Id = Id::root("sampling");
const ACTIONS: [CellAction; 1] = [CellAction::new(ActionKey::application("follow"))];
struct Model {
    actions: bool,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        1
    }
    fn row_key(&self, _: usize) -> ItemKey {
        ItemKey::num(1)
    }
    fn cell(&self, _: usize, col: usize) -> Option<CellRef<'_>> {
        Some(CellRef::new(if col == 0 { "abc" } else { "z" }))
    }
    fn actions(&self, _: usize, col: usize) -> &[CellAction] {
        if self.actions && col == 0 {
            &ACTIONS
        } else {
            &[]
        }
    }
}
struct Page {
    model: Model,
    columns: Vec<Column<'static>>,
    state: GridState,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Grid::new(ID, &self.columns)
            .update(cx, &mut self.state, &self.model)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &self.columns).draw(ui, ui.full(), &self.state, &self.model);
    }
}
fn page() -> Result<Page, WidthSampleError> {
    let mut columns = vec![
        Column::new(ColumnKey::num(0), "a"),
        Column::new(ColumnKey::num(1), "b"),
    ];
    for c in &mut columns {
        c.min_width = 1;
        c.max_width = 40;
    }
    let mut p = Page {
        model: Model { actions: false },
        columns,
        state: GridState::default(),
    };
    Grid::new(ID, &p.columns).sample_column_widths(
        &mut p.state,
        &p.model,
        WidthSample::new(200, 95)?,
    );
    Ok(p)
}
#[test]
fn actions_remain_live_constraints_after_sampling() -> Result<(), WidthSampleError> {
    let mut h = Harness::new(page()?, Theme::junie(), 40, 5);
    assert!(h.row(1).contains("abc"));
    h.app_mut().model.actions = true;
    h.draw();
    assert!(
        h.row(1).contains("abc"),
        "sampled natural width must retain full text when live action reserve appears: {:?}",
        h.row(1)
    );
    assert_eq!(h.cell(8, 1).symbol(), "z");
    h.app_mut().model.actions = false;
    h.draw();
    assert_eq!(h.cell(6, 1).symbol(), "z");
    assert_eq!(
        h.app().state.sampled_column_width(ColumnKey::num(0)),
        Some(3)
    );
    Ok(())
}
#[test]
fn custom_constraints_and_missing_keys_stay_live() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = Harness::new(page()?, Theme::junie(), 40, 5);
    h.app_mut()
        .columns
        .first_mut()
        .ok_or("missing first column")?
        .min_width = 10;
    h.app_mut()
        .columns
        .first_mut()
        .ok_or("missing first column")?
        .max_width = 10;
    h.draw();
    assert_eq!(h.cell(13, 0).symbol(), "b");
    h.app_mut()
        .columns
        .first_mut()
        .ok_or("missing first column")?
        .key = ColumnKey::num(9);
    h.draw();
    assert_eq!(h.app().state.sampled_column_width(ColumnKey::num(9)), None);
    assert_eq!(h.cell(13, 0).symbol(), "b");
    Ok(())
}
