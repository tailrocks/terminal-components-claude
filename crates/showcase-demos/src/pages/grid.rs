//! Data grid: typed cells, a pending-change queue, paging and local sort.

use termrock::{
    Align, CellDecor, CellRef, Column, ColumnKey, Cx, FgStep, FrameRead, GlyphRole, Grid,
    GridAction, GridColumnFit, GridGutter, GridModel, GridOverflowIndicator, GridSortIndicator,
    GridState, Id, ItemKey, NavUnit, Panel, PanelKind, Part, Rect, Role, RowTotal, StateFlags,
    StylePatch, Ui, id,
};

use super::{Page, PageStatus, PageUpdate, frame};

const GRID: Id = id!("grid.grid");
const CUSTOMERS: Id = id!("grid.customers");

const NAMES: &[&str] = &[
    "Northwind Traders",
    "Blue Yonder Airlines",
    "Contoso Pharmaceuticals",
    "Fabrikam Robotics",
    "Litware Analytics",
    "Tailspin Toys",
    "Wide World Importers",
    "Adventure Works",
    "Proseware Studio",
    "Woodgrove Bank",
    "Alpine Ski House",
    "Coho Winery",
    "Lucerne Publishing",
    "Margie's Travel",
    "Trey Research",
    "Humongous Insurance",
];
const PLANS: &[&str] = &["free", "pro", "team", "enterprise"];
const OWNERS: &[&str] = &["mira", "jonas", "ana", "kai"];
const SEATS: &[u32] = &[1, 3, 5, 12, 25, 40, 80, 150];
const ROWS: usize = 40;
const ESTIMATED_TOTAL: usize = 4_812;

/// Fixed widths sampled from the historical grid (p95 content, header floor):
/// the layout never depends on the viewport, only the visible window does.
const COLUMNS: [Column<'static>; 8] = [
    Column {
        key: ColumnKey::num(0),
        title: "id",
        subtitle: None,
        align: Align::Left,
        min_width: 9,
        max_width: 9,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: Some(GlyphRole::PrimaryKey),
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(1),
        title: "customer",
        subtitle: None,
        align: Align::Left,
        min_width: 25,
        max_width: 25,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(2),
        title: "plan",
        subtitle: None,
        align: Align::Left,
        min_width: 10,
        max_width: 10,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(3),
        title: "seats",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(4),
        title: "mrr",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(5),
        title: "active",
        subtitle: None,
        align: Align::Left,
        min_width: 8,
        max_width: 8,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(6),
        title: "renewed_at",
        subtitle: None,
        align: Align::Left,
        min_width: 12,
        max_width: 12,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(7),
        title: "notes",
        subtitle: None,
        align: Align::Left,
        min_width: 27,
        max_width: 27,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
];

#[derive(Clone, Debug)]
struct CustomerRow {
    id: String,
    customer: String,
    plan: &'static str,
    seats: String,
    mrr: String,
    active: &'static str,
    renewed: String,
    renewed_null: bool,
    notes: String,
    notes_null: bool,
}

fn row(index: usize) -> CustomerRow {
    let plan = PLANS[(index * 7 + 3) % PLANS.len()];
    let seats = SEATS[(index * 5 + 1) % SEATS.len()];
    let mrr = match plan {
        "free" => 0.0,
        "pro" => 29.0 * f64::from(seats),
        "team" => 24.0 * f64::from(seats),
        _ => 19.0 * f64::from(seats),
    };
    let suffix = if index >= NAMES.len() {
        format!(" {}", index / NAMES.len() + 1)
    } else {
        String::new()
    };
    let (renewed, renewed_null) = if plan == "free" {
        (String::from("NULL"), true)
    } else {
        (
            format!("2026-{:02}-{:02}", 1 + index % 12, 1 + (index * 3) % 28),
            false,
        )
    };
    let (notes, notes_null) = if index.is_multiple_of(4) {
        (
            format!(
                "{{\"owner\":\"{}\",\"seats\":{seats}}}",
                OWNERS[index % OWNERS.len()]
            ),
            false,
        )
    } else {
        (String::from("NULL"), true)
    };
    CustomerRow {
        id: (1001 + index as u64).to_string(),
        customer: format!("{}{suffix}", NAMES[index % NAMES.len()]),
        plan,
        seats: seats.to_string(),
        mrr: format!("{mrr:.2}"),
        active: if index % 5 == 3 { "false" } else { "true" },
        renewed,
        renewed_null,
        notes,
        notes_null,
    }
}

#[derive(Clone, Debug, Default)]
struct CustomerModel {
    rows: Vec<CustomerRow>,
}

impl CustomerModel {
    fn new() -> Self {
        Self {
            rows: (0..ROWS).map(row).collect(),
        }
    }

    fn row_index(&self, key: ItemKey) -> Option<usize> {
        (0..self.rows.len()).find(|index| self.row_key(*index) == key)
    }
}

impl GridModel for CustomerModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        if row < self.rows.len() {
            ItemKey::num(1001 + row as u64)
        } else {
            ItemKey::num(0)
        }
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = self.rows.get(row)?;
        let text = match col {
            0 => item.id.as_str(),
            1 => item.customer.as_str(),
            2 => item.plan,
            3 => item.seats.as_str(),
            4 => item.mrr.as_str(),
            5 => item.active,
            6 => item.renewed.as_str(),
            7 => item.notes.as_str(),
            _ => return None,
        };
        Some(CellRef::new(text))
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        let null = match self.rows.get(row) {
            Some(item) if col == 6 => item.renewed_null,
            Some(item) if col == 7 => item.notes_null,
            _ => false,
        };
        if null {
            CellDecor {
                tone: Some(Role::Fg(FgStep::Muted)),
                italic: true,
                ..CellDecor::default()
            }
        } else {
            CellDecor::default()
        }
    }

    fn total(&self) -> RowTotal {
        RowTotal::Estimated(ESTIMATED_TOTAL)
    }

    fn has_more(&self) -> bool {
        true
    }
}

fn grid() -> Grid<'static> {
    Grid::new(GRID, &COLUMNS)
        .nav(NavUnit::Cell)
        .column_gap(2)
        .gutter(GridGutter::Detailed {
            row_numbers: true,
            min_digits: 2,
        })
        .right_reserve(4)
        .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
        .sort_indicator(GridSortIndicator::ActiveOnly)
        .overflow_indicator(GridOverflowIndicator::Count)
        .fetch_label("Enter fetches more")
}

/// Page-local meta tone: the historical meta reads faint while the shared
/// `PANEL` detail stays secondary (T4). The title keeps the recipe's own
/// base (secondary) and focused (primary + bold) rules untouched.
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

/// The one customers-card constructor: the meta gate reads its geometry and
/// draw paints through it, so the two can never disagree.
fn customers_panel(meta: &str, focused: bool) -> Panel<'_> {
    Panel::new(CUSTOMERS)
        .kind(PanelKind::Card)
        .title("customers")
        .meta(meta)
        .meta_late(true)
        .focused(focused)
        .patch_part(PANEL_PARTS)
}

/// `rows a–b of n loaded · ~t total · cols a–b of n`, from draw-time panel
/// geometry: last-frame layout is unavailable on frame 1, which is what the
/// no-input default capture presents.
fn position_label(
    grid: &Grid<'_>,
    state: &GridState,
    model: &CustomerModel,
    grid_area: Rect,
) -> String {
    let rows = grid.rows_label_for(state, model, grid_area);
    match grid.cols_label_for(state, model, grid_area) {
        Some(cols) => format!("{rows} · {cols}"),
        None => rows,
    }
}

/// The data grid: typed cells over the shared [`Grid`] engine, with the
/// paging position composed into the card meta.
#[derive(Debug)]
pub struct GridPage {
    model: CustomerModel,
    state: GridState,
}

impl GridPage {
    pub fn new() -> Self {
        Self {
            model: CustomerModel::new(),
            state: GridState::default(),
        }
    }
}

impl Default for GridPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for GridPage {
    fn title(&self) -> &'static str {
        "Data grid"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let action = grid().update(cx, &mut self.state, &self.model);
        let mut status = None;
        if let Some(GridAction::Activated(key)) = action.action_ref()
            && let Some(row) = self.model.row_index(*key)
        {
            status = Some(PageStatus(format!("Row {} activated", row + 1)));
        }
        PageUpdate {
            response: action.erase(),
            status,
        }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Typed cells, a pending-change queue, paging and local sort",
            |ui, body| {
                let card = Rect {
                    height: body.height.min(30),
                    ..body
                };
                let focused = ui.state(GRID).contains(StateFlags::FOCUSED);
                let probe = customers_panel("", focused).inner(ui, card);
                let meta = position_label(&grid(), &self.state, &self.model, probe);
                customers_panel(&meta, focused).draw(ui, card, |ui, inner| {
                    grid().draw(ui, inner, &self.state, &self.model);
                });
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            &[("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            &[
                ("↑↓←→", "Cell"),
                ("Enter", "Edit"),
                ("s", "Sort"),
                ("Space", "Select row"),
                ("+ -", "Insert / delete"),
                ("u", "Undo"),
            ]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn typed_model_matches_the_historical_rows() {
        let model = CustomerModel::new();
        assert_eq!(model.row_count(), 40);
        assert_eq!(model.total(), RowTotal::Estimated(4_812));
        assert!(model.has_more());

        let text = |row: usize, col: usize| model.cell(row, col).unwrap().text.to_owned();
        assert_eq!(model.row_key(0), ItemKey::num(1001));
        assert_eq!(text(0, 0), "1001");
        assert_eq!(text(0, 1), "Northwind Traders");
        assert_eq!(text(0, 2), "enterprise");
        assert_eq!(text(0, 3), "3");
        assert_eq!(text(0, 4), "57.00");
        assert_eq!(text(0, 5), "true");
        assert_eq!(text(0, 6), "2026-01-01");
        assert_eq!(text(0, 7), "{\"owner\":\"mira\",\"seats\":3}");

        // Free plans have no renewal date; sparse rows carry no notes.
        assert_eq!(text(3, 2), "free");
        assert_eq!(text(3, 6), "NULL");
        assert!(model.cell_decor(3, 6).italic);
        assert_eq!(text(1, 7), "NULL");
        assert!(model.cell_decor(1, 7).italic);
        assert!(!model.cell_decor(0, 7).italic);

        // Second page of names carries a suffix; inactive rows read false.
        assert_eq!(text(16, 1), "Northwind Traders 2");
        assert_eq!(text(3, 5), "false");
        assert_eq!(model.row_key(39), ItemKey::num(1040));
        assert_eq!(model.row_index(ItemKey::num(1002)), Some(1));
        assert_eq!(model.cell(0, 8), None);
        assert_eq!(model.cell(40, 0), None);
    }
}

#[cfg(test)]
mod meta_tests {
    use super::*;

    fn harness() -> (Grid<'static>, GridState, CustomerModel) {
        (grid(), GridState::default(), CustomerModel::new())
    }

    #[test]
    fn paging_meta_matches_the_historical_labels() {
        let (grid, state, model) = harness();
        // 120x40: the card inner rect the panel hands the grid.
        let area = Rect::new(28, 7, 90, 27);
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total"
        );
        assert_eq!(
            grid.cols_label_for(&state, &model, area).as_deref(),
            Some("cols 1–6 of 8")
        );
        assert_eq!(
            position_label(&grid, &state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total · cols 1–6 of 8"
        );
    }

    #[test]
    fn narrow_viewports_clip_columns_and_rows() {
        let (grid, state, model) = harness();
        // 72x20: the card inner rect the panel hands the grid.
        let area = Rect::new(23, 7, 47, 11);
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 1–10 of 40 loaded · ~4,812 total"
        );
        assert_eq!(
            grid.cols_label_for(&state, &model, area).as_deref(),
            Some("cols 1–2 of 8")
        );
    }

    #[test]
    fn wide_viewports_drop_the_column_label() {
        let (grid, state, model) = harness();
        // 160x50: every column fits, so only the rows read out.
        let area = Rect::new(28, 7, 130, 27);
        assert_eq!(grid.cols_label_for(&state, &model, area), None,);
        assert_eq!(
            position_label(&grid, &state, &model, area),
            "rows 1–26 of 40 loaded · ~4,812 total"
        );
    }

    #[test]
    fn scrolled_state_moves_the_window_not_the_paint() {
        let (grid, mut state, model) = harness();
        let area = Rect::new(28, 7, 90, 27);
        state.scroll_mut().apply_layout(26, model.row_count());
        state.scroll_mut().jump_end();
        assert_eq!(
            grid.rows_label_for(&state, &model, area),
            "rows 15–40 of 40 loaded · ~4,812 total"
        );
    }
}

#[cfg(test)]
mod row_number_tests {
    use super::*;
    use termrock::{App, Family, FgStep, KeyCode, Response, StylePatch, Surface, Theme, Variant};
    use termrock_test_support::Harness;

    struct PageApp(GridPage);

    impl App for PageApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.0.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let full = ui.full();
            self.0.draw(ui, full);
        }
    }

    /// Row numbers must bind the GRID-owned faint address. Poison
    /// (VIEWPORT, GUTTER) and require the digits to stay faint: any
    /// row number still resolved from the viewport gutter fails.
    #[test]
    fn row_numbers_resolve_from_grid_overflow() {
        let mut theme = Theme::junie();
        theme
            .recipes
            .get_mut(Family::VIEWPORT)
            .parts
            .entry(Part::GUTTER)
            .base = StylePatch::new().set_fg(Role::Danger);
        let faint = theme.color.fg.get(FgStep::Faint.index()).copied();
        assert_ne!(Some(theme.color.danger), faint);
        let poisoned = theme.resolve(
            Family::VIEWPORT,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::empty(),
            Surface::Surface,
        );
        assert_eq!(poisoned.style.fg, Some(theme.color.danger));
        let owned = theme.resolve(
            Family::GRID,
            Variant::DEFAULT,
            Part::OVERFLOW,
            StateFlags::empty(),
            Surface::Surface,
        );
        assert_eq!(owned.style.fg, faint);

        let mut h = Harness::new(PageApp(GridPage::new()), theme, 120, 40);
        let _ = h.key(KeyCode::Null);
        // The cursor row wears the focused secondary number; the row below
        // must stay faint.
        let (x, y) = h.find("1002").expect("datagrid id column renders");
        let cell = h.cell(x - 2, y);
        assert_eq!(cell.symbol(), "2");
        assert_eq!(Some(cell.fg), faint, "row number must stay faint");
    }
}
