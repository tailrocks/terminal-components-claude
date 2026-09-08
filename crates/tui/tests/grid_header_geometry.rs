//! Independent header and gap policies preserve default geometry.
use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, Grid, GridHeaderSizing, GridModel, GridSortIndicator,
    GridState, Id, ItemKey, Rect, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("header.grid");
struct Model;
impl GridModel for Model {
    fn row_count(&self) -> usize {
        1
    }
    fn row_key(&self, _: usize) -> ItemKey {
        ItemKey::num(0)
    }
    fn cell(&self, _: usize, _: usize) -> Option<CellRef<'_>> {
        Some(CellRef::new("x"))
    }
}
struct Page {
    state: GridState,
    columns: [Column<'static>; 2],
    gap: u16,
    sizing: GridHeaderSizing,
    indicator: GridSortIndicator,
    area: Rect,
}
impl Page {
    fn new() -> Self {
        let mut columns = [
            Column::new(ColumnKey::num(0), "ab"),
            Column::new(ColumnKey::num(1), "cd"),
        ];
        for c in &mut columns {
            c.min_width = 1;
            c.max_width = 4;
            c.sortable = true;
        }
        Self {
            state: GridState::default(),
            columns,
            gap: 2,
            sizing: GridHeaderSizing::Minimum {
                padding: 2,
                cap: 24,
            },
            indicator: GridSortIndicator::ActiveOnly,
            area: Rect::new(0, 0, 20, 4),
        }
    }
    fn grid(&self) -> Grid<'_> {
        Grid::new(ID, &self.columns)
            .column_gap(self.gap)
            .header_sizing(self.sizing)
            .sort_indicator(self.indicator)
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Grid::new(ID, &self.columns)
            .column_gap(self.gap)
            .header_sizing(self.sizing)
            .sort_indicator(self.indicator)
            .update(cx, &mut self.state, &Model)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.grid().draw(ui, self.area, &self.state, &Model);
    }
}
#[test]
fn source_header_padding_gap_and_active_indicator_share_hit_geometry() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 20, 4);
    assert_eq!(h.cell(2, 0).symbol(), "a");
    assert_eq!(h.cell(8, 0).symbol(), "c");
    assert_eq!(h.cell(5, 0).symbol(), " ");
    assert_eq!(h.cell(11, 0).symbol(), " ");
    let _ = h.click(8, 0);
    assert_ne!(h.cell(11, 0).symbol(), " ");
    assert_eq!(h.cell(5, 0).symbol(), " ");
    let _ = h.click(8, 1);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(0), ColumnKey::num(1)))
    );
}

#[test]
fn independent_gap_and_indicator_options_leave_header_sizing_explicit() {
    let mut page = Page::new();
    page.gap = 0;
    let mut h = Harness::new(page, Theme::junie(), 20, 4);
    assert_eq!(h.cell(6, 0).symbol(), "c");
    h.app_mut().gap = 1;
    h.draw();
    assert_eq!(h.cell(7, 0).symbol(), "c");
    h.app_mut().indicator = GridSortIndicator::Always;
    h.draw();
    assert_ne!(h.cell(5, 0).symbol(), " ");
    assert_ne!(h.cell(10, 0).symbol(), " ");
    h.app_mut().sizing = GridHeaderSizing::Content;
    h.app_mut().indicator = GridSortIndicator::ActiveOnly;
    h.draw();
    assert_eq!(h.cell(5, 0).symbol(), "c");
}
#[test]
fn header_minimum_is_capped_and_tiny_rectangles_remain_clipped() {
    let mut page = Page::new();
    page.columns[0].title = "abcdefghijklmnopqrstuvwxyz";
    page.area = Rect::new(0, 0, 60, 4);
    let mut h = Harness::new(page, Theme::junie(), 60, 4);
    // First column grows beyond max4 to cap24; second begins2+24+2.
    assert_eq!(h.cell(28, 0).symbol(), "c");
    for width in 0..=3 {
        for height in 0..=3 {
            h.app_mut().area = Rect::new(1, 1, width, height);
            let _ = h.resize(6, 6);
            h.draw();
            assert_eq!(h.cell(0, 0).symbol(), " ");
            assert_eq!(h.cell(5, 5).symbol(), " ");
        }
    }
}
