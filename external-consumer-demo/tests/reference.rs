//! Public-facade integration tests for the borrowed reference adapter API.

use core::cell::{Cell, RefCell};
use core::ops::Range;
use std::rc::Rc;

#[cfg(feature = "testing")]
use external_consumer_demo::reference::INPUT_ID;
use external_consumer_demo::reference::{BUTTON_ID, ReferenceGallery};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::author::{Buffer, Constraints, Rect, Size};
use termrock::{
    App, Cx, Id, ReferenceCase, ReferenceGrid, ReferenceSpecimen, ReferenceState, ReferenceTarget,
    Response, Runtime, Theme, Ui,
};

const SCREEN: Rect = Rect::new(0, 0, 80, 16);

fn present<A: App>(runtime: &mut Runtime<A>, area: Rect) -> Buffer {
    let mut terminal =
        Terminal::new(TestBackend::new(area.width, area.height)).expect("test terminal starts");
    let prepared = runtime.prepare(area).expect("reference frame prepares");
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("prepared reference frame matches the terminal");
        })
        .expect("test backend draws the reference frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("reference frame is acknowledged");
    terminal.backend().buffer().clone()
}

struct GridApp<'g, 's> {
    grid: &'g ReferenceGrid<'s>,
    rows: Range<usize>,
    header: bool,
    area: Rect,
    drawn: Rc<RefCell<Vec<usize>>>,
}

impl App for GridApp<'_, '_> {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        *self.drawn.borrow_mut() =
            self.grid
                .draw_rows(ui, self.rows.clone(), self.header, self.area);
    }
}

fn draw_rows(
    grid: &ReferenceGrid<'_>,
    rows: Range<usize>,
    header: bool,
    area: Rect,
) -> (Vec<usize>, Buffer) {
    draw_rows_in(grid, rows, header, area, area)
}

fn draw_rows_in(
    grid: &ReferenceGrid<'_>,
    rows: Range<usize>,
    header: bool,
    frame_area: Rect,
    grid_area: Rect,
) -> (Vec<usize>, Buffer) {
    let drawn = Rc::new(RefCell::new(Vec::new()));
    let mut runtime = Runtime::new(
        GridApp {
            grid,
            rows,
            header,
            area: grid_area,
            drawn: Rc::clone(&drawn),
        },
        Theme::junie(),
    );
    let buffer = present(&mut runtime, frame_area);
    (drawn.borrow().clone(), buffer)
}

fn text(buffer: &Buffer) -> String {
    let area = *buffer.area();
    let mut out = String::new();
    for y in area.y..area.y.saturating_add(area.height) {
        for x in area.x..area.x.saturating_add(area.width) {
            if let Some(cell) = buffer.cell(termrock::Position::new(x, y)) {
                out.push_str(cell.symbol());
            }
        }
        out.push('\n');
    }
    out
}

struct Tall<'a> {
    calls: &'a Cell<usize>,
    last_area: &'a Cell<Rect>,
}

impl ReferenceSpecimen for Tall<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        matches!(case, ReferenceCase::Rest | ReferenceCase::Focused)
    }

    fn specimen_height(&self, case: ReferenceCase) -> u16 {
        match case {
            ReferenceCase::Rest => 2,
            ReferenceCase::Focused => 3,
            _ => 0,
        }
    }

    fn target(&self, case: ReferenceCase) -> Option<ReferenceTarget> {
        (case == ReferenceCase::Focused).then(|| {
            ReferenceTarget::new(
                Id::root("external-reference.tall"),
                ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE,
            )
        })
    }

    fn draw(&self, _case: ReferenceCase, ui: &mut Ui<'_>, area: Rect) {
        self.calls.set(self.calls.get() + 1);
        self.last_area.set(area);
        let _ = ui.paint_str(area, "T", ui.surface_style());
    }
}

struct Flat<'a> {
    calls: &'a Cell<usize>,
}

impl ReferenceSpecimen for Flat<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        matches!(case, ReferenceCase::Rest | ReferenceCase::Focused)
    }

    fn specimen_height(&self, _case: ReferenceCase) -> u16 {
        0
    }

    fn draw(&self, _case: ReferenceCase, ui: &mut Ui<'_>, area: Rect) {
        self.calls.set(self.calls.get() + 1);
        let _ = ui.paint_str(area, "F", ui.surface_style());
    }
}

struct Never<'a> {
    calls: &'a Cell<usize>,
}

struct LocalBorrowed<'a> {
    label: &'a str,
    calls: Rc<Cell<usize>>,
}

impl ReferenceSpecimen for LocalBorrowed<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        case == ReferenceCase::Rest
    }

    fn specimen_height(&self, _case: ReferenceCase) -> u16 {
        0
    }

    fn draw(&self, _case: ReferenceCase, ui: &mut Ui<'_>, area: Rect) {
        self.calls.set(self.calls.get() + 1);
        let _ = ui.paint_str(area, self.label, ui.surface_style());
    }
}

impl ReferenceSpecimen for Never<'_> {
    fn supports(&self, _case: ReferenceCase) -> bool {
        false
    }

    fn specimen_height(&self, _case: ReferenceCase) -> u16 {
        u16::MAX
    }

    fn draw(&self, _case: ReferenceCase, _ui: &mut Ui<'_>, _area: Rect) {
        self.calls.set(self.calls.get() + 1);
    }
}

#[test]
fn historical_cases_and_slugs_remain_exact() {
    let expected = [
        ("rest", "rest"),
        ("hover", "hover"),
        ("pressed", "pressed"),
        ("focused", "focused"),
        ("focused_hover", "focus + hover"),
        ("selected", "selected"),
        ("disabled", "disabled"),
        ("busy", "busy"),
        ("error", "error"),
        ("editing", "editing"),
        ("empty", "empty"),
        ("loading", "loading"),
        ("overflow", "overflow"),
        ("narrow", "narrow"),
    ];
    assert_eq!(ReferenceCase::ALL.len(), expected.len());
    for (case, (slug, label)) in ReferenceCase::ALL.into_iter().zip(expected) {
        assert_eq!(case.slug(), slug);
        assert_eq!(case.label(), label);
        assert_eq!(ReferenceCase::from_slug(slug), Some(case));
    }
    assert_eq!(ReferenceCase::from_slug("Focused"), None);
}

#[test]
fn borrowed_native_adapters_return_exact_synthetic_targets() {
    let gallery = ReferenceGallery::new();
    let specimen = gallery.button_adapter();
    assert_eq!(
        specimen.target(ReferenceCase::FocusedHover),
        Some(ReferenceTarget::new(
            BUTTON_ID,
            ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE | ReferenceState::HOVERED,
        ))
    );
    assert_eq!(
        specimen.target(ReferenceCase::Pressed),
        Some(ReferenceTarget::new(
            BUTTON_ID,
            ReferenceState::HOVERED | ReferenceState::PRESSED,
        ))
    );
    assert_eq!(specimen.target(ReferenceCase::Selected), None);
}

#[test]
fn external_gallery_paints_two_borrowed_native_adapters_inertly() {
    let area = Rect::new(0, 0, 80, 16);
    let mut runtime = Runtime::new(ReferenceGallery::new(), Theme::junie());
    let buffer = present(&mut runtime, area);

    // These are production observations. Exact hit and overlay publication
    // introspection is covered by the explicit testing-feature probe below.
    assert_eq!(runtime.focus(), None);
    assert_eq!(runtime.cursor_position(), None);

    let rendered = text(&buffer);
    assert!(rendered.contains("Button"), "{rendered:?}");
    assert!(rendered.contains("Text input"), "{rendered:?}");
    assert!(rendered.contains("focus + hover"), "{rendered:?}");
    assert!(rendered.contains("–"), "{rendered:?}");
}

#[cfg(feature = "testing")]
#[test]
fn testing_feature_confirms_reference_has_no_live_publication() {
    let area = Rect::new(0, 0, 80, 16);
    let mut runtime = Runtime::new(ReferenceGallery::new(), Theme::junie());
    let _buffer = present(&mut runtime, area);

    assert_eq!(runtime.region_count(), 0);
    assert_eq!(runtime.area_of(BUTTON_ID), None);
    assert_eq!(runtime.area_of(INPUT_ID), None);
    assert_eq!(runtime.focus(), None);
    assert_eq!(runtime.cursor_position(), None);
    assert_eq!(runtime.open_spec(BUTTON_ID), None);
    assert_eq!(runtime.open_spec(INPUT_ID), None);
}

#[test]
fn empty_columns_all_unsupported_and_zero_heights_still_measure_one_row() {
    let cases = [ReferenceCase::Rest, ReferenceCase::Empty];
    let empty = ReferenceGrid::new().rows(&cases);
    assert_eq!(empty.row_heights(), vec![1, 1]);
    assert_eq!(empty.size(), Size::exact(15, 3));

    let calls = Cell::new(0);
    let all_unsupported = ReferenceGrid::new()
        .rows(&cases)
        .column("unsupported", Never { calls: &calls });
    assert_eq!(all_unsupported.row_heights(), vec![1, 1]);
    assert_eq!(all_unsupported.size().preferred.1, 3);
    assert_eq!(calls.get(), 0);

    let zero_height = ReferenceGrid::new()
        .row(ReferenceCase::Rest)
        .column("zero", Flat { calls: &calls });
    assert_eq!(zero_height.row_heights(), vec![1]);
    assert_eq!(zero_height.size().preferred.1, 2);
}

#[test]
fn grid_accepts_a_non_static_borrowed_non_send_adapter() {
    let label = String::from("local adapter");
    let calls = Rc::new(Cell::new(0));
    let specimen = LocalBorrowed {
        label: &label,
        calls: Rc::clone(&calls),
    };
    let grid = ReferenceGrid::new()
        .row(ReferenceCase::Rest)
        .column("borrowed", specimen);
    assert_eq!(grid.row_heights(), vec![1]);

    let (drawn, buffer) = draw_rows(&grid, 0..1, false, Rect::new(0, 0, 40, 1));
    assert_eq!(drawn, vec![0]);
    assert!(text(&buffer).contains("local adapter"));
    assert_eq!(calls.get(), 1);
}

#[test]
fn grid_uses_max_supported_height_and_clips_requested_ranges() {
    let tall_calls = Cell::new(0);
    let tall_area = Cell::new(Rect::ZERO);
    let flat_calls = Cell::new(0);
    let cases = [
        ReferenceCase::Rest,
        ReferenceCase::Focused,
        ReferenceCase::Disabled,
    ];
    let grid = ReferenceGrid::new()
        .rows(&cases)
        .column(
            "tall",
            Tall {
                calls: &tall_calls,
                last_area: &tall_area,
            },
        )
        .column("flat", Flat { calls: &flat_calls });
    assert_eq!(grid.row_heights(), vec![2, 3, 1]);
    assert_eq!(grid.size().preferred.1, 7);

    let (drawn, _) = draw_rows(&grid, 1..usize::MAX, false, Rect::new(0, 0, 80, 3));
    assert_eq!(drawn, vec![1], "returned indices are original row indices");
    assert_eq!(tall_calls.get(), 1);
    assert_eq!(flat_calls.get(), 1);

    let tall_calls = Cell::new(0);
    let tall_area = Cell::new(Rect::ZERO);
    let flat_calls = Cell::new(0);
    let grid = ReferenceGrid::new()
        .rows(&cases)
        .column(
            "tall",
            Tall {
                calls: &tall_calls,
                last_area: &tall_area,
            },
        )
        .column("flat", Flat { calls: &flat_calls });
    let (drawn, _) = draw_rows(&grid, 0..3, true, Rect::new(0, 0, 80, 4));
    assert_eq!(
        drawn,
        vec![0],
        "the first partial row and following rows are omitted"
    );
    assert_eq!(tall_calls.get(), 1);
    assert_eq!(flat_calls.get(), 1);

    let reversed = Range { start: 3, end: 1 };
    let (drawn, _) = draw_rows(&grid, reversed, false, SCREEN);
    assert!(drawn.is_empty(), "reversed ranges are empty");
    let (drawn, _) = draw_rows(&grid, 100..usize::MAX, false, SCREEN);
    assert!(
        drawn.is_empty(),
        "ranges beyond the original row list clamp empty"
    );
}

#[test]
fn header_is_reserved_and_zero_columns_still_paint_row_labels() {
    let calls = Cell::new(0);
    let grid = ReferenceGrid::new()
        .row(ReferenceCase::Rest)
        .column("only", Never { calls: &calls });
    let (drawn, buffer) = draw_rows(&grid, 0..1, true, Rect::new(0, 0, 30, 1));
    assert!(drawn.is_empty(), "the header consumes its own row");
    assert!(text(&buffer).contains("only"));
    assert_eq!(calls.get(), 0);

    let empty = ReferenceGrid::new().row(ReferenceCase::Focused);
    let (drawn, buffer) = draw_rows(&empty, 0..1, false, Rect::new(0, 0, 20, 1));
    assert_eq!(drawn, vec![0]);
    assert!(text(&buffer).contains("focused"));
}

#[test]
fn unsupported_marker_empty_area_and_cell_clipping_are_bounded() {
    let calls = Cell::new(0);
    let grid = ReferenceGrid::new()
        .row(ReferenceCase::Empty)
        .column("none", Never { calls: &calls });
    let (drawn, buffer) = draw_rows(&grid, 0..1, false, Rect::new(0, 0, 40, 1));
    assert_eq!(drawn, vec![0]);
    assert!(text(&buffer).contains("–"));
    assert_eq!(calls.get(), 0);

    let calls = Cell::new(0);
    let area_seen = Cell::new(Rect::ZERO);
    let clipped = ReferenceGrid::new().row(ReferenceCase::Rest).column(
        "clip",
        Tall {
            calls: &calls,
            last_area: &area_seen,
        },
    );
    let (drawn, _) = draw_rows_in(&clipped, 0..1, false, SCREEN, Rect::new(0, 0, 0, 1));
    assert!(drawn.is_empty());
    let (drawn, _) = draw_rows_in(&clipped, 0..1, false, SCREEN, Rect::new(0, 0, 20, 0));
    assert!(drawn.is_empty());
    assert_eq!(
        calls.get(),
        0,
        "empty rectangles never call supported adapters"
    );

    let (drawn, _) = draw_rows(&clipped, 0..1, false, Rect::new(0, 0, 16, 2));
    assert_eq!(drawn, vec![0]);
    assert_eq!(calls.get(), 1);
    assert_eq!(area_seen.get().x, 15);
    assert_eq!(area_seen.get().width, 1);
    assert_eq!(area_seen.get().height, 2);
}

struct Greedy<'a> {
    calls: &'a Cell<usize>,
    last_area: &'a Cell<Rect>,
}

impl ReferenceSpecimen for Greedy<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        matches!(case, ReferenceCase::Rest | ReferenceCase::Focused)
    }

    fn specimen_height(&self, case: ReferenceCase) -> u16 {
        match case {
            ReferenceCase::Rest => 2,
            ReferenceCase::Focused => 3,
            _ => 0,
        }
    }

    fn draw(&self, _case: ReferenceCase, ui: &mut Ui<'_>, area: Rect) {
        self.calls.set(self.calls.get() + 1);
        self.last_area.set(area);
        // Deliberately ignore the adapter area. The grid must narrow Ui itself.
        let _ = ui.paint_str(ui.full(), &"G".repeat(80), ui.surface_style());
    }
}

#[test]
fn grid_clips_before_row_fitting_and_contains_adversarial_painting() {
    let neighbor_calls = Cell::new(0);
    let greedy_calls = Cell::new(0);
    let greedy_area = Cell::new(Rect::ZERO);
    let cases = [ReferenceCase::Rest, ReferenceCase::Focused];
    let grid = ReferenceGrid::new()
        .rows(&cases)
        .label_width(4)
        .column_width(8)
        .column(
            "neighbor",
            Flat {
                calls: &neighbor_calls,
            },
        )
        .column(
            "greedy",
            Greedy {
                calls: &greedy_calls,
                last_area: &greedy_area,
            },
        );

    let viewport = Rect::new(0, 0, 20, 4);
    let overflowing_grid = Rect::new(2, 1, 40, 20);
    let (drawn, buffer) = draw_rows_in(&grid, 0..usize::MAX, true, viewport, overflowing_grid);

    assert_eq!(
        drawn,
        vec![0],
        "fit rows against the grid intersected with the viewport"
    );
    assert_eq!(neighbor_calls.get(), 1);
    assert_eq!(greedy_calls.get(), 1);
    assert_eq!(greedy_area.get(), Rect::new(14, 2, 6, 2));
    assert_eq!(
        buffer.area(),
        &viewport,
        "the backend buffer remains bounded to the viewport"
    );
    assert_eq!(
        buffer
            .cell(termrock::Position::new(6, 2))
            .expect("neighbor cell is in the viewport")
            .symbol(),
        "F",
        "the later hostile adapter cannot overwrite its neighbor"
    );
    assert_eq!(
        buffer
            .cell(termrock::Position::new(14, 2))
            .expect("hostile cell is in the viewport")
            .symbol(),
        "G",
        "the adapter still paints within its own narrowed cell"
    );
}

struct MeasureApp<'g, 's> {
    grid: &'g ReferenceGrid<'s>,
    constraints: Constraints,
    measured: &'g Cell<Size>,
}

impl App for MeasureApp<'_, '_> {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.measured.set(self.grid.measure(ui, self.constraints));
    }
}

#[test]
fn measure_obeys_constraints_on_the_production_render_path() {
    let calls = Cell::new(0);
    let grid = ReferenceGrid::new()
        .rows(&[ReferenceCase::Rest, ReferenceCase::Focused])
        .column("tall", Never { calls: &calls });
    let measured = Cell::new(Size::default());
    let mut runtime = Runtime::new(
        MeasureApp {
            grid: &grid,
            constraints: Constraints::loose(8, 2),
            measured: &measured,
        },
        Theme::junie(),
    );
    let _buffer = present(&mut runtime, SCREEN);
    assert_eq!(measured.get(), Size::exact(8, 2));
}
