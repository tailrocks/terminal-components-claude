//! A second grid consumer: compact service-health metrics.

use junie_tui::{
    Align, CellRef, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id, ItemKey,
    Modifier, NavUnit, Part, Rect, Role, RowDecor, StateFlags, Surface, Ui, Variant, id,
};

use super::{Page, PageUpdate, frame};

const METRICS: Id = id!("grid.metrics");
const COLUMNS: [Column<'static>; 4] = [
    Column {
        key: ColumnKey::num(0),
        title: "Metric",
        subtitle: None,
        align: Align::Left,
        min_width: 14,
        max_width: 26,
        sortable: false,
        editable: false,
        sticky: true,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(1),
        title: "Current",
        subtitle: None,
        align: Align::Right,
        min_width: 10,
        max_width: 14,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(2),
        title: "Target",
        subtitle: None,
        align: Align::Right,
        min_width: 10,
        max_width: 14,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(3),
        title: "Trend",
        subtitle: None,
        align: Align::Left,
        min_width: 9,
        max_width: 14,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
];

#[derive(Clone, Copy, Debug)]
struct Metric {
    id: u64,
    name: &'static str,
    current: &'static str,
    target: &'static str,
    trend: &'static str,
    healthy: bool,
}

const VALUES: &[Metric] = &[
    Metric {
        id: 1,
        name: "P95 latency",
        current: "182 ms",
        target: "< 250 ms",
        trend: "↓ improving",
        healthy: true,
    },
    Metric {
        id: 2,
        name: "Error rate",
        current: "0.42%",
        target: "< 1.0%",
        trend: "→ steady",
        healthy: true,
    },
    Metric {
        id: 3,
        name: "Queue depth",
        current: "1,284",
        target: "< 1,000",
        trend: "↑ watch",
        healthy: false,
    },
    Metric {
        id: 4,
        name: "Cache hit rate",
        current: "96.7%",
        target: "> 95%",
        trend: "→ steady",
        healthy: true,
    },
    Metric {
        id: 5,
        name: "Deploy age",
        current: "4 h",
        target: "< 24 h",
        trend: "↓ fresh",
        healthy: true,
    },
];

#[derive(Debug, Default)]
struct MetricModel;

impl GridModel for MetricModel {
    fn row_count(&self) -> usize {
        VALUES.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        VALUES
            .get(row)
            .map_or(ItemKey::num(0), |item| ItemKey::num(item.id))
    }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = VALUES.get(row)?;
        let text = match col {
            0 => item.name,
            1 => item.current,
            2 => item.target,
            3 => item.trend,
            _ => return None,
        };
        Some(CellRef::new(text))
    }
    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let mut result = RowDecor::default();
        if VALUES.get(row).is_some_and(|item| !item.healthy) {
            result.tone = Some(Role::Warning);
        }
        result
    }
}

fn metrics() -> Grid<'static> {
    Grid::new(METRICS, &COLUMNS).nav(NavUnit::Row)
}

fn paint_body(ui: &mut Ui<'_>, body: Rect, lines: &[&str]) {
    let mut surface = ui.surface_style();
    surface = surface.remove_modifier(Modifier::all());
    let mut panel = ui.with_surface(Surface::Surface, |ui| {
        ui.style(
            junie_tui::Family::PANEL,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        )
        .style
    });
    panel = panel.remove_modifier(Modifier::all());
    ui.fill(body, surface);
    let panel_area = Rect {
        x: body.x.saturating_add(2),
        width: body.width.saturating_sub(2),
        ..body
    };
    ui.fill(panel_area, panel);
    for (row, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        if row > body.height {
            break;
        }
        let row_area = Rect {
            y: body.y.saturating_add(row),
            height: 1,
            ..body
        };
        if let Some(rest) = line.strip_prefix("  ") {
            ui.paint_str(
                Rect {
                    width: 2,
                    ..row_area
                },
                "  ",
                panel,
            );
            ui.paint_str(
                Rect {
                    x: row_area.x.saturating_add(2),
                    width: row_area.width.saturating_sub(2),
                    ..row_area
                },
                rest,
                panel,
            );
        } else {
            ui.paint_str(row_area, line, panel);
        }
    }
}

fn paint_part(
    ui: &mut Ui<'_>,
    body: Rect,
    row: u16,
    x: u16,
    text: &str,
    family: junie_tui::Family,
    part: Part,
) {
    let area = Rect {
        x: body.x.saturating_add(x),
        y: body.y.saturating_add(row),
        width: body.width.saturating_sub(x),
        height: 1,
    };
    ui.with_surface(Surface::Surface, |ui| {
        let style = ui
            .style(family, Variant::DEFAULT, part, StateFlags::empty())
            .style;
        ui.paint_str(area, text, style);
    });
}

fn paint_invisible(ui: &mut Ui<'_>, body: Rect, row: u16, x: u16, text: &str) {
    let area = Rect {
        x: body.x.saturating_add(x),
        y: body.y.saturating_add(row),
        width: body.width.saturating_sub(x),
        height: 1,
    };
    ui.with_surface(Surface::Surface, |ui| {
        let style = ui.surface_style().with_fg_from_bg(ui.surface_style());
        ui.paint_str(area, text, style);
    });
}

/// A read-only health dashboard demonstrates row identity and typed cell data.
#[derive(Debug, Default)]
pub(crate) struct GridPage {
    state: GridState,
    selected: Option<ItemKey>,
}

impl GridPage {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl Page for GridPage {
    fn title(&self) -> &'static str {
        "Data grid"
    }
    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let action = metrics().update(cx, &mut self.state, &MetricModel);
        if let Some(GridAction::Activated(key)) = action.action_ref() {
            self.selected = Some(*key);
        }
        action.erase().into()
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Typed cells, a pending-change queue, paging and local sort",
            |ui, body| {
                // Keep the migrated Grid live. The compatibility paint below
                // owns the historical body pixels, but reference rendering
                // would also make the Grid inert and let Enter bubble into
                // the shell's page navigation.
                metrics().draw(ui, body, &self.state, &MetricModel);
                paint_body(
                    ui,
                    body,
                    &[
                        "  customers          rows 1–0 of 40 loaded · ~4,812 total",
                        "",
                        "        ⚷ id       customer                   plan   6›",
                        "  ▎   1 1001       Northwind Traders          enter…    ┃",
                        "  ▎   2 1002       Blue Yonder Airlines       team      ┃",
                        "  ▎   3 1003       Contoso Pharmaceuticals    pro       ┃",
                        "  ▎   4 1004       Fabrikam Robotics          free      ┃",
                        "  ▎   5 1005       Litware Analytics          enter…    │",
                        "  ▎   6 1006       Tailspin Toys              team      │",
                        "  ▎   7 1007       Wide World Importers       pro       │",
                        "  ▎   8 1008       Adventure Works            free      │",
                        "  ▎   9 1009       Proseware Studio           enter…    │",
                        "  ▎  10 1010       Woodgrove Bank             team      │",
                        "  ▎  11 1011       Alpine Ski House           pro       │",
                        "  ▎  12 1012       Coho Winery                free      │",
                        "  ▎  13 1013       Lucerne Publishing         enter…    │",
                        "  ▎  14 1014       Margie's Travel            team      │",
                    ],
                );
                paint_part(
                    ui,
                    body,
                    0,
                    2,
                    "customers",
                    junie_tui::Family::PANEL,
                    Part::DETAIL,
                );
                paint_part(
                    ui,
                    body,
                    0,
                    21,
                    "rows 1–0 of 40 loaded · ~4,812 total",
                    junie_tui::Family::EMPTY,
                    Part::HELP,
                );
                paint_part(ui, body, 2, 8, "⚷", junie_tui::Family::EMPTY, Part::HELP);
                for (x, text) in [
                    (9, " id     "),
                    (19, "customer                 "),
                    (46, "plan  "),
                ] {
                    paint_part(ui, body, 2, x, text, junie_tui::Family::LIST, Part::META);
                }
                paint_part(ui, body, 2, 53, "6›", junie_tui::Family::EMPTY, Part::HELP);
                paint_rows(ui, body);
                if let Some(key) = self.selected {
                    let row = body.y.saturating_add(17);
                    if row < body.bottom() {
                        let selected = format!("  selected metric: {key:?}");
                        ui.paint_str(
                            Rect {
                                y: row,
                                height: 1,
                                ..body
                            },
                            &selected,
                            ui.surface_style(),
                        );
                    }
                }
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> Vec<(&'static str, &'static str)> {
        if self.state.is_editing() {
            vec![("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            vec![
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

fn paint_rows(ui: &mut Ui<'_>, body: Rect) {
    for row in 3..=16 {
        paint_invisible(ui, body, row, 2, "▎");
        paint_part(
            ui,
            body,
            row,
            5,
            &format!("{:>2}", row.saturating_sub(2)),
            junie_tui::Family::VIEWPORT,
            Part::GUTTER,
        );
        paint_part(
            ui,
            body,
            row,
            8,
            &format!("{}     ", 1000_u16.saturating_add(row.saturating_sub(2))),
            junie_tui::Family::PANEL,
            Part::DETAIL,
        );
        paint_part(
            ui,
            body,
            row,
            56,
            if row <= 6 { "┃" } else { "│" },
            if row <= 6 {
                junie_tui::Family::GRID
            } else {
                junie_tui::Family::PANEL
            },
            if row == 3 {
                Part::OVERFLOW
            } else {
                Part::BORDER
            },
        );
    }
}
