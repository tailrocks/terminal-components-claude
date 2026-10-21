use termrock::{
    Align, CellRef, Color, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id,
    ItemKey, Modifier, NavUnit, PaintStyle, Rect, Role, RowDecor, Ui, width,
};

use super::{Page, PageUpdate, chrome_ink};
use termrock::feedback::{PageFrame, SectionStrip};

const METRICS: Id = termrock::Id::root("showcase_ui::pages::grid::grid.metrics");
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

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Typed cells, a pending-change queue, paging and local sort";

const GRID_H: u16 = 26;
const LEGEND_H: u16 = 3;
const SCHEMA_H: u16 = 12;
const COMMANDS_H: u16 = 26;
const CARD_GAP: u16 = 1;
const SCROLL_ROOM: u16 = 2;
const COL_GAP: u16 = 2;
const GUTTER_W: u16 = 6;
const COL_RESERVE: u16 = 4;
const LABEL_W: u16 = 14;

const LOADED: usize = 40;
const CONTENT_ROWS: usize = 41;
const ALL_COLS: usize = 9;
const WIDE_GRID: u16 = 70;
const PREVIEW_MIN: u16 = 6;
const SHOWN_ROWS: usize = 21;
const NARROW_TRACK: u16 = 75;
const STRING: u32 = 0x8bd5ca;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const LITERAL: u32 = 0xc6a0f6;

const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;

const NAMES: [&str; 16] = [
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
const PLANS: [&str; 4] = ["free", "pro", "team", "enterprise"];
const SEATS: [u32; 8] = [1, 3, 5, 12, 25, 40, 80, 150];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ColKind {
    Text,
    Primary,
    Literal,
    Nullable,
    Link,
    Json,
}

struct Col {
    title: &'static str,
    width: u16,
    right: bool,
    kind: ColKind,
}

const GRID_COLS: [Col; 9] = [
    Col {
        title: "id",
        width: 9,
        right: false,
        kind: ColKind::Primary,
    },
    Col {
        title: "customer",
        width: 25,
        right: false,
        kind: ColKind::Text,
    },
    Col {
        title: "plan",
        width: 10,
        right: false,
        kind: ColKind::Text,
    },
    Col {
        title: "seats",
        width: 7,
        right: true,
        kind: ColKind::Text,
    },
    Col {
        title: "mrr",
        width: 7,
        right: true,
        kind: ColKind::Text,
    },
    Col {
        title: "active",
        width: 8,
        right: false,
        kind: ColKind::Literal,
    },
    Col {
        title: "renewed_at",
        width: 12,
        right: false,
        kind: ColKind::Nullable,
    },
    Col {
        title: "account_id",
        width: 12,
        right: true,
        kind: ColKind::Link,
    },
    Col {
        title: "notes",
        width: 27,
        right: false,
        kind: ColKind::Json,
    },
];

const SCHEMA: [(&str, &str); 9] = [
    ("id", "Id · integer · primary key · not null · read-only"),
    ("customer", "Text · text · not null"),
    (
        "plan",
        "Enum · plan_tier · not null · one of free, pro, team, enterprise · ▾ picks from a fixed set",
    ),
    ("seats", "Number · integer · not null"),
    ("mrr", "Number · numeric(10,2) · read-only"),
    ("active", "Bool · boolean · not null"),
    ("renewed_at", "Timestamp · date"),
    ("account_id", "Number · integer · references accounts"),
    ("notes", "Text · jsonb"),
];

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn mix(fg: u32, bg: u32, keep: f32) -> u32 {
    let channel = |shift: u32| {
        let from = f32::from(((fg >> shift) & 0xff) as u8);
        let into = f32::from(((bg >> shift) & 0xff) as u8);
        (into + (from - into) * keep).round().clamp(0.0, 255.0) as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn faded(color: u32, bg: u32, keep: Option<f32>) -> u32 {
    keep.map_or(color, |keep| mix(color, bg, keep))
}

fn thumb(offset: usize, viewport: usize, content: usize, track: usize) -> (usize, usize) {
    if viewport == 0 || track == 0 || content <= viewport {
        return (0, track);
    }
    let len = (viewport.saturating_mul(track) / content).max(1).min(track);
    let max_off = content.saturating_sub(viewport);
    let start = offset
        .saturating_mul(track.saturating_sub(len))
        .saturating_add(max_off / 2)
        / max_off.max(1);
    (start.min(track.saturating_sub(len)), len)
}

fn row_fade(index: usize, viewport: usize) -> Option<f32> {
    if !(4..CONTENT_ROWS).contains(&viewport) {
        return None;
    }
    let depth = usize::from(viewport >= 12) + 1;
    if index + 1 == viewport {
        return Some(FADE_OUTER);
    }
    if depth == 2 && index + 2 == viewport {
        return Some(FADE_INNER);
    }
    None
}

fn fit(text: &str, columns: u16, right: bool) -> String {
    let shown = termrock::truncate(text, columns);
    let pad = usize::from(columns.saturating_sub(width(&shown)));
    if right {
        format!("{}{shown}", " ".repeat(pad))
    } else {
        format!("{shown}{}", " ".repeat(pad))
    }
}

fn paint_at(ui: &mut Ui<'_>, x: u16, y: u16, columns: u16, text: &str, style: PaintStyle) {
    if columns == 0 {
        return;
    }
    ui.paint_str(Rect::new(x, y, columns, 1), text, style);
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Data grid");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let style = ink(MUTED, CANVAS);
    if width(BLURB) <= meta.width {
        ui.paint_str(meta, BLURB, style);
        return;
    }
    if meta.width == 0 {
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, BLURB, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str, right_inset: u16) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 4 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(2).saturating_sub(right_inset);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    if !meta.is_empty() && text_w > 0 {
        let meta_w = width(meta);
        let title_w = width(title);
        let min_x = text_x.saturating_add(title_w.saturating_mul(2));
        let fitted = text_w >= meta_w;
        let right_x = text_x.saturating_add(text_w.saturating_sub(meta_w));
        if fitted && right_x >= min_x {
            ui.paint_str(
                Rect::new(right_x, card.y, meta_w, 1),
                meta,
                ink(FAINT, SURFACE),
            );
        } else if min_x < text_x.saturating_add(text_w) {
            let room = text_x.saturating_add(text_w).saturating_sub(min_x);
            ui.paint_str(Rect::new(min_x, card.y, room, 1), meta, ink(FAINT, SURFACE));
        }
    }
    if card.height > 1 && text_w > 0 {
        ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
            ink(RULE, SURFACE),
        );
    }
}

fn paint_vertical_bar(ui: &mut Ui<'_>, area: Rect, rows: usize, viewport: u16, bg: u32) {
    if area.is_empty() {
        return;
    }
    let (start, len) = thumb(0, usize::from(viewport), rows, usize::from(area.height));
    for row in 0..area.height {
        let on = usize::from(row) >= start && usize::from(row) < start.saturating_add(len);
        paint_at(
            ui,
            area.x,
            area.y.saturating_add(row),
            1,
            if on { "┃" } else { "┆" },
            if on { ink(THUMB, bg) } else { ink(TRACK, bg) },
        );
    }
}

fn paint_horizontal_bar(ui: &mut Ui<'_>, area: Rect, shown: usize, columns: usize, bg: u32) {
    if area.is_empty() {
        return;
    }
    let (start, len) = thumb(0, shown, columns, usize::from(area.width));
    for col in 0..area.width {
        let on = usize::from(col) >= start && usize::from(col) < start.saturating_add(len);
        paint_at(
            ui,
            area.x.saturating_add(col),
            area.y,
            1,
            if on { "━" } else { "┄" },
            if on { ink(THUMB, bg) } else { ink(TRACK, bg) },
        );
    }
}

fn paint_page_scrollbar(ui: &mut Ui<'_>, bar: Rect, content_h: u16) {
    let (start, len) = thumb(
        0,
        usize::from(bar.height),
        usize::from(content_h),
        usize::from(bar.height),
    );
    for row in 0..bar.height {
        let on = usize::from(row) >= start && usize::from(row) < start.saturating_add(len);
        paint_at(
            ui,
            bar.x,
            bar.y.saturating_add(row),
            1,
            if on { "┃" } else { "┆" },
            if on {
                ink(THUMB, CANVAS)
            } else {
                ink(TRACK, CANVAS)
            },
        );
    }
}

fn content_height() -> u16 {
    GRID_H
        .saturating_add(LEGEND_H)
        .saturating_add(SCHEMA_H)
        .saturating_add(COMMANDS_H)
        .saturating_add(CARD_GAP.saturating_mul(3))
}

fn customer_name(index: usize) -> String {
    let suffix = if index >= NAMES.len() {
        format!(" {}", index / NAMES.len() + 1)
    } else {
        String::new()
    };
    format!("{}{suffix}", NAMES[index % NAMES.len()])
}

fn plan_of(index: usize) -> &'static str {
    PLANS[(index * 7 + 3) % PLANS.len()]
}

fn seats_of(index: usize) -> u32 {
    SEATS[(index * 5 + 1) % SEATS.len()]
}

fn mrr_of(plan: &str, seats: u32) -> String {
    let rate = match plan {
        "free" => 0.0,
        "pro" => 29.0,
        "team" => 24.0,
        _ => 19.0,
    };
    format!("{:.2}", rate * f64::from(seats))
}

fn renewed_at(index: usize) -> String {
    if index % 4 == 3 {
        return "NULL".to_owned();
    }
    let month = (index % 12) + 1;
    let day = (index * 3) % 28 + 1;
    format!("2026-{month:02}-{day:02}")
}

fn notes_of(index: usize, seats: u32) -> String {
    if index.is_multiple_of(4) {
        format!("{{\"owner\":\"mira\",\"seats\":{seats}}}")
    } else {
        "NULL".to_owned()
    }
}

fn cell_text(index: usize, col: usize) -> String {
    let plan = plan_of(index);
    let seats = seats_of(index);
    match col {
        0 => (1001 + index).to_string(),
        1 => customer_name(index),
        2 => plan.to_owned(),
        3 => seats.to_string(),
        4 => mrr_of(plan, seats),
        5 => (index % 5 != 3).to_string(),
        6 => renewed_at(index),
        7 => (201 + (index % 4)).to_string(),
        _ => notes_of(index, seats),
    }
}

fn cell_style(col: &Col, text: &str) -> PaintStyle {
    let (fg, italic) = match col.kind {
        ColKind::Primary => (SECONDARY, false),
        ColKind::Literal => (LITERAL, true),
        ColKind::Nullable | ColKind::Json if text == "NULL" => (LITERAL, true),
        ColKind::Json => (STRING, false),
        ColKind::Text | ColKind::Link | ColKind::Nullable => (WHITE, false),
    };
    let mut style = ink(fg, SURFACE);
    if italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    style
}

struct ColBudget {
    cols_w: u16,
    complete: usize,
    preview: u16,
}

fn column_budget(cols_w: u16, preview_min: u16) -> ColBudget {
    let mut used = 0u16;
    let mut complete = 0usize;
    for col in &GRID_COLS {
        let need = if complete == 0 {
            col.width
        } else {
            col.width.saturating_add(COL_GAP)
        };
        if used.saturating_add(need) > cols_w {
            break;
        }
        used = used.saturating_add(need);
        complete += 1;
    }
    let preview = if complete < GRID_COLS.len() {
        let gap = u16::from(complete > 0).saturating_mul(COL_GAP);
        let rest = cols_w.saturating_sub(used).saturating_sub(gap);
        if rest >= preview_min { rest } else { 0 }
    } else {
        0
    };
    ColBudget {
        cols_w,
        complete,
        preview,
    }
}

fn paint_clipped(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    columns: u16,
    text: &str,
    style: PaintStyle,
    ellipsis: bool,
) {
    if columns == 0 {
        return;
    }
    if width(text) <= columns {
        paint_at(ui, x, y, columns, &fit(text, columns, false), style);
        return;
    }
    if ellipsis {
        paint_at(ui, x, y, columns, &termrock::truncate(text, columns), style);
        return;
    }
    ui.paint_str(Rect::new(x, y, columns, 1), text, style);
}

#[expect(clippy::too_many_arguments)]
fn paint_column(
    ui: &mut Ui<'_>,
    x: u16,
    y: u16,
    col: &Col,
    text: &str,
    columns: u16,
    ellipsis: bool,
    keep: Option<f32>,
) {
    if col.kind == ColKind::Link {
        let num_w = columns.saturating_sub(2);
        let style = ink(faded(WHITE, SURFACE, keep), SURFACE);
        paint_at(ui, x, y, num_w, &fit(text, num_w, true), style);
        if columns > 0 {
            paint_at(
                ui,
                x.saturating_add(columns).saturating_sub(1),
                y,
                1,
                "→",
                ink(faded(MUTED, SURFACE, keep), SURFACE),
            );
        }
        return;
    }
    let base = cell_style(col, text);
    let style = ink(faded(color_of(base), SURFACE, keep), SURFACE);
    let style = if base.as_style().add_modifier.contains(Modifier::ITALIC) {
        style.add_modifier(Modifier::ITALIC)
    } else {
        style
    };
    if col.right && width(text) <= columns {
        paint_at(ui, x, y, columns, &fit(text, columns, true), style);
        return;
    }
    paint_clipped(ui, x, y, columns, text, style, ellipsis);
}

fn color_of(style: PaintStyle) -> u32 {
    match style.as_style().fg {
        Some(termrock::Color::Rgb(r, g, b)) => {
            (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
        }
        _ => WHITE,
    }
}

fn paint_grid_header(ui: &mut Ui<'_>, origin_x: u16, y: u16, budget: &ColBudget, badge: bool) {
    let mut x = origin_x;
    for col in GRID_COLS.iter().take(budget.complete) {
        let title = if col.kind == ColKind::Primary {
            format!("  {}", col.title)
        } else {
            col.title.to_owned()
        };
        if col.right {
            paint_at(
                ui,
                x,
                y,
                col.width,
                &fit(&title, col.width, true),
                ink(MUTED, SURFACE),
            );
        } else {
            paint_clipped(ui, x, y, col.width, &title, ink(MUTED, SURFACE), false);
        }
        if col.kind == ColKind::Primary {
            paint_at(ui, x, y, 1, "⚷", ink(FAINT, SURFACE));
        }
        x = x.saturating_add(col.width).saturating_add(COL_GAP);
    }
    if budget.preview > 0
        && let Some(col) = GRID_COLS.get(budget.complete)
    {
        paint_clipped(
            ui,
            x,
            y,
            budget.preview,
            col.title,
            ink(MUTED, SURFACE),
            budget.preview >= PREVIEW_MIN,
        );
    }
    if badge {
        let hidden = ALL_COLS.saturating_sub(budget.complete);
        if hidden > 0 {
            let label = format!("{hidden}›");
            paint_at(
                ui,
                origin_x.saturating_add(budget.cols_w).saturating_add(1),
                y,
                width(&label),
                &label,
                ink(SECONDARY, SURFACE),
            );
        }
    }
}

fn paint_grid_row(
    ui: &mut Ui<'_>,
    area: Rect,
    index: usize,
    budget: &ColBudget,
    keep: Option<f32>,
) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, ink(faded(WHITE, SURFACE, keep), SURFACE));
    paint_at(
        ui,
        area.x,
        area.y,
        1,
        " ",
        ink(faded(SURFACE, SURFACE, keep), SURFACE),
    );
    paint_at(
        ui,
        area.x.saturating_add(3),
        area.y,
        2,
        &fit(&(index + 1).to_string(), 2, true),
        ink(faded(FAINT, SURFACE, keep), SURFACE),
    );
    let mut x = area.x.saturating_add(GUTTER_W);
    for (col_index, col) in GRID_COLS.iter().take(budget.complete).enumerate() {
        paint_column(
            ui,
            x,
            area.y,
            col,
            &cell_text(index, col_index),
            col.width,
            true,
            keep,
        );
        x = x.saturating_add(col.width).saturating_add(COL_GAP);
    }
    if budget.preview > 0
        && let Some(col) = GRID_COLS.get(budget.complete)
    {
        paint_column(
            ui,
            x,
            area.y,
            col,
            &cell_text(index, budget.complete),
            budget.preview,
            budget.preview >= PREVIEW_MIN,
            keep,
        );
    }
}

fn paint_customers(ui: &mut Ui<'_>, card: Rect, wide: bool, inside_bar: bool) {
    let right_inset = u16::from(wide).saturating_mul(2);
    let cols_w = if wide {
        card.width
            .saturating_sub(4)
            .saturating_sub(GUTTER_W)
            .saturating_sub(COL_RESERVE)
            .saturating_sub(1)
    } else {
        card.width.saturating_sub(2).saturating_sub(GUTTER_W)
    };
    let preview_min = if wide { PREVIEW_MIN } else { 1 };
    let budget = column_budget(cols_w, preview_min);
    let meta = format!(
        "rows 1–{SHOWN_ROWS} of {LOADED} loaded · ~4,812 total · cols 1–{} of {ALL_COLS}",
        budget.complete
    );
    paint_card_chrome(ui, card, "customers", &meta, right_inset);
    if card.height < 4 || budget.complete == 0 {
        return;
    }
    let origin_x = card.x.saturating_add(2).saturating_add(GUTTER_W);
    let header_y = card.y.saturating_add(2);
    paint_grid_header(ui, origin_x, header_y, &budget, wide);
    let data_rows = if wide && inside_bar {
        usize::from(card.height.saturating_sub(5)).min(LOADED)
    } else {
        usize::from(card.height.saturating_sub(3)).min(LOADED)
    };
    let row_w = if wide {
        card.width.saturating_sub(4).saturating_sub(1)
    } else {
        card.width.saturating_sub(2)
    };
    for index in 0..data_rows {
        let y = header_y
            .saturating_add(1)
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        paint_grid_row(
            ui,
            Rect::new(card.x.saturating_add(2), y, row_w, 1),
            index,
            &budget,
            row_fade(index, SHOWN_ROWS),
        );
    }
    if wide && data_rows > 0 {
        let inner_right = card.x.saturating_add(card.width).saturating_sub(2);
        paint_vertical_bar(
            ui,
            Rect::new(
                inner_right.saturating_sub(1),
                header_y.saturating_add(1),
                1,
                u16::try_from(data_rows).unwrap_or(u16::MAX),
            ),
            CONTENT_ROWS,
            u16::try_from(data_rows).unwrap_or(u16::MAX),
            SURFACE,
        );
    }
    if inside_bar && budget.complete < ALL_COLS && data_rows > 0 {
        let x = origin_x;
        let bar_w = cols_w.saturating_add(COL_RESERVE);
        paint_horizontal_bar(
            ui,
            Rect::new(
                x,
                header_y
                    .saturating_add(1)
                    .saturating_add(u16::try_from(data_rows).unwrap_or(u16::MAX)),
                bar_w,
                1,
            ),
            budget.complete,
            ALL_COLS,
            SURFACE,
        );
    }
}

fn paint_schema(ui: &mut Ui<'_>, card: Rect) {
    paint_card_chrome(ui, card, "Schema", "", 2);
    if card.height < 3 {
        return;
    }
    let x = card.x.saturating_add(2);
    let label_w = 12u16;
    let value_x = x.saturating_add(label_w);
    let value_w = card.width.saturating_sub(4).saturating_sub(label_w);
    for (index, (label, value)) in SCHEMA.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = card.y.saturating_add(2).saturating_add(offset);
        if y >= card.bottom() {
            break;
        }
        paint_at(ui, x, y, label_w, label, ink(MUTED, SURFACE));
        if value_w > 0 {
            paint_clipped(ui, value_x, y, value_w, value, ink(WHITE, SURFACE), true);
        }
    }
}

fn paint_legend(ui: &mut Ui<'_>, area: Rect) {
    let rows = [
        (
            "Change marks",
            "• modified   + inserted   − deleted   ! error",
        ),
        ("Pending", "no pending changes"),
    ];
    let x = area.x.saturating_add(2);
    for (index, (label, value)) in rows.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = area.y.saturating_add(offset);
        if y >= area.bottom() {
            break;
        }
        paint_at(ui, x, y, width(label), label, ink(MUTED, CANVAS));
        let value_x = x.saturating_add(LABEL_W);
        if value_x < area.right() {
            paint_at(
                ui,
                value_x,
                y,
                area.right().saturating_sub(value_x),
                value,
                ink(WHITE, CANVAS),
            );
        }
    }
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let content_h = content_height();
    let overflows = area.width > SCROLL_ROOM && content_h > area.height;
    let wide = area.width >= WIDE_GRID;
    let view = if overflows {
        Rect {
            width: area.width.saturating_sub(SCROLL_ROOM),
            ..area
        }
    } else {
        area
    };
    let narrow_bar = overflows && !wide && view.height > 1;
    let card_h = if narrow_bar {
        view.height.saturating_sub(1)
    } else {
        GRID_H.min(view.height)
    };
    let inside_bar = wide && card_h >= GRID_H;
    if card_h > 0 {
        paint_customers(
            ui,
            Rect::new(view.x, view.y, view.width, card_h),
            wide,
            inside_bar,
        );
    }
    if narrow_bar && card_h > 0 {
        let track = view.width.saturating_add(1);
        paint_horizontal_bar(
            ui,
            Rect::new(view.x, view.y.saturating_add(card_h), track, 1),
            usize::from(view.width),
            usize::from(NARROW_TRACK),
            CANVAS,
        );
    }
    let legend_y = view.y.saturating_add(GRID_H).saturating_add(CARD_GAP);
    if legend_y < view.bottom() {
        paint_legend(
            ui,
            Rect::new(
                view.x,
                legend_y,
                view.width,
                view.bottom().saturating_sub(legend_y).min(2),
            ),
        );
    }
    let schema_y = legend_y.saturating_add(LEGEND_H).saturating_add(CARD_GAP);
    if schema_y < view.bottom() {
        paint_schema(
            ui,
            Rect::new(
                view.x,
                schema_y,
                view.width,
                view.bottom().saturating_sub(schema_y).min(SCHEMA_H),
            ),
        );
    }
    if overflows {
        let bar_h = if narrow_bar {
            area.height.saturating_sub(1)
        } else {
            area.height
        };
        paint_page_scrollbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, bar_h),
            content_h,
        );
    }
}

#[derive(Debug, Default)]
pub struct GridPage {
    state: GridState,
    selected: Option<ItemKey>,
}

impl GridPage {
    pub fn new() -> Self {
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
        PageFrame::new(self.title())
            .meta("")
            .draw_with(ui, area, |ui, body| {
                paint_blurb(ui, body);
                SectionStrip::new(&SECTIONS)
                    .caption(SECTION_CAPTION)
                    .caption_min_width(SECTION_WIDE)
                    .ink(chrome_ink())
                    .draw(
                        ui,
                        Rect {
                            height: 2.min(body.height),
                            ..body
                        },
                    );
                let content = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                paint_playground(ui, content);
                if let Some(key) = self.selected {
                    let row = content.bottom().saturating_sub(1);
                    if row >= content.y && row < body.bottom() {
                        let selected = format!("selected metric: {key:?}");
                        ui.paint_str(
                            Rect::new(content.x, row, content.width, 1),
                            &selected,
                            ui.surface_style(),
                        );
                    }
                }
            });
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
