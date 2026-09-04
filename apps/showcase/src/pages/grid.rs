//! A second grid consumer: compact service-health metrics.

use tui_next::{Align, CellRef, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id, ItemKey, NavUnit, Rect, Response, Role, RowDecor, Ui, id};

use super::{Page, frame, lines};

const METRICS: Id = id!("grid.metrics");
const COLUMNS: [Column<'static>; 4] = [
    Column { key: ColumnKey::num(0), title: "Metric", subtitle: None, align: Align::Left, min_width: 14, max_width: 26, sortable: false, editable: false, sticky: true, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(1), title: "Current", subtitle: None, align: Align::Right, min_width: 10, max_width: 14, sortable: false, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(2), title: "Target", subtitle: None, align: Align::Right, min_width: 10, max_width: 14, sortable: false, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(3), title: "Trend", subtitle: None, align: Align::Left, min_width: 9, max_width: 14, sortable: false, editable: false, sticky: false, prefix_glyph: None, badge: None },
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
    Metric { id: 1, name: "P95 latency", current: "182 ms", target: "< 250 ms", trend: "↓ improving", healthy: true },
    Metric { id: 2, name: "Error rate", current: "0.42%", target: "< 1.0%", trend: "→ steady", healthy: true },
    Metric { id: 3, name: "Queue depth", current: "1,284", target: "< 1,000", trend: "↑ watch", healthy: false },
    Metric { id: 4, name: "Cache hit rate", current: "96.7%", target: "> 95%", trend: "→ steady", healthy: true },
    Metric { id: 5, name: "Deploy age", current: "4 h", target: "< 24 h", trend: "↓ fresh", healthy: true },
];

#[derive(Debug, Default)]
struct MetricModel;

impl GridModel for MetricModel {
    fn row_count(&self) -> usize { VALUES.len() }
    fn row_key(&self, row: usize) -> ItemKey { VALUES.get(row).map_or(ItemKey::num(0), |item| ItemKey::num(item.id)) }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = VALUES.get(row)?;
        let text = match col { 0 => item.name, 1 => item.current, 2 => item.target, 3 => item.trend, _ => return None };
        Some(CellRef::new(text))
    }
    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let mut result = RowDecor::default();
        if VALUES.get(row).is_some_and(|item| !item.healthy) { result.tone = Some(Role::Warning); }
        result
    }
}

fn metrics() -> Grid<'static> { Grid::new(METRICS, &COLUMNS).nav(NavUnit::Row) }

/// A read-only health dashboard demonstrates row identity and typed cell data.
#[derive(Debug, Default)]
pub(crate) struct GridPage {
    state: GridState,
    selected: Option<ItemKey>,
}

impl GridPage { pub(crate) fn new() -> Self { Self::default() } }

impl Page for GridPage {
    fn title(&self) -> &'static str { "Data grid" }
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let action = metrics().update(cx, &mut self.state, &MetricModel);
        if let Some(GridAction::Activated(key)) = action.action_ref() { self.selected = Some(*key); }
        action.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "typed cells · row navigation · stable keys", |ui, body| {
            let grid_area = Rect { height: body.height.saturating_sub(2), ..body };
            metrics().draw(ui, grid_area, &self.state, &MetricModel);
            let selected = self.selected.map_or_else(|| "none".to_owned(), |key| format!("{key:?}"));
            let _ = ui.paint_str(Rect { y: grid_area.bottom(), height: 1, ..body }, &format!("selected metric: {selected}"), ui.surface_style());
            lines(ui, Rect { y: grid_area.bottom().saturating_add(1), height: 1, ..body }, &["The model supplies borrowed cells; Grid owns cursor, selection and viewport state."]);
        });
    }
}
