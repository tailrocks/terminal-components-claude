use termrock::{
    Align, CellRef, Color, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id,
    ItemKey, Modifier, NavUnit, PaintStyle, Rect, Role, RowDecor, Ui, width,
};

use super::{Page, PageUpdate, frame};

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

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
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

struct Col {
    title: &'static str,
    width: u16,
    right: bool,
    primary: bool,
    literal: bool,
}

const GRID_COLS: [Col; 6] = [
    Col {
        title: "id",
        width: 9,
        right: false,
        primary: true,
        literal: false,
    },
    Col {
        title: "customer",
        width: 25,
        right: false,
        primary: false,
        literal: false,
    },
    Col {
        title: "plan",
        width: 10,
        right: false,
        primary: false,
        literal: false,
    },
    Col {
        title: "seats",
        width: 7,
        right: true,
        primary: false,
        literal: false,
    },
    Col {
        title: "mrr",
        width: 7,
        right: true,
        primary: false,
        literal: false,
    },
    Col {
        title: "active",
        width: 8,
        right: false,
        primary: false,
        literal: true,
    },
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

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Data grid");
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    ui.paint_str(
        Rect::new(
            body.x.saturating_add(title_w).saturating_add(2),
            body.y.saturating_sub(2),
            body.width.saturating_sub(title_w).saturating_sub(3),
            1,
        ),
        BLURB,
        ink(MUTED, CANVAS),
    );
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 4 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
    if !meta.is_empty() {
        let meta_w = width(meta);
        if meta_w < text_w {
            ui.paint_str(
                Rect::new(
                    text_x.saturating_add(text_w.saturating_sub(meta_w)),
                    card.y,
                    meta_w,
                    1,
                ),
                meta,
                ink(FAINT, SURFACE),
            );
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

fn card_inner(card: Rect) -> Rect {
    Rect {
        x: card.x.saturating_add(2),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(4),
        height: card.height.saturating_sub(3),
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

fn paint_horizontal_bar(ui: &mut Ui<'_>, area: Rect, shown: usize, columns: usize) {
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
            if on {
                ink(THUMB, SURFACE)
            } else {
                ink(TRACK, SURFACE)
            },
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

fn columns_fit(avail: u16) -> usize {
    let mut used = 0u16;
    let mut fit = 0usize;
    for col in &GRID_COLS {
        let need = if fit == 0 {
            col.width
        } else {
            col.width.saturating_add(COL_GAP)
        };
        if used.saturating_add(need) > avail {
            break;
        }
        used = used.saturating_add(need);
        fit += 1;
    }
    fit
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

fn cell_text(index: usize, col: usize) -> String {
    let plan = plan_of(index);
    let seats = seats_of(index);
    match col {
        0 => (1001 + index).to_string(),
        1 => customer_name(index),
        2 => plan.to_owned(),
        3 => seats.to_string(),
        4 => mrr_of(plan, seats),
        _ => (index % 5 != 3).to_string(),
    }
}

fn cell_color(col: &Col) -> u32 {
    if col.literal {
        LITERAL
    } else if col.primary {
        SECONDARY
    } else {
        WHITE
    }
}

fn paint_grid_header(ui: &mut Ui<'_>, inner: Rect, cols_w: u16, shown: usize) {
    let mut x = inner.x.saturating_add(GUTTER_W);
    for col in GRID_COLS.iter().take(shown) {
        let title = if col.primary {
            format!("  {}", col.title)
        } else {
            col.title.to_owned()
        };
        paint_at(
            ui,
            x,
            inner.y,
            col.width,
            &fit(&title, col.width, col.right),
            ink(MUTED, SURFACE),
        );
        if col.primary {
            paint_at(ui, x, inner.y, 1, "⚷", ink(FAINT, SURFACE));
        }
        x = x.saturating_add(col.width).saturating_add(COL_GAP);
    }
    let hidden = ALL_COLS.saturating_sub(shown);
    if hidden > 0 {
        let badge = format!("{hidden}›");
        paint_at(
            ui,
            inner
                .x
                .saturating_add(GUTTER_W)
                .saturating_add(cols_w)
                .saturating_add(1),
            inner.y,
            width(&badge),
            &badge,
            ink(SECONDARY, SURFACE),
        );
    }
}

fn paint_grid_row(ui: &mut Ui<'_>, area: Rect, index: usize, shown: usize, keep: Option<f32>) {
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
    for (col_index, col) in GRID_COLS.iter().take(shown).enumerate() {
        let mut style = ink(faded(cell_color(col), SURFACE, keep), SURFACE);
        if col.literal {
            style = style.add_modifier(Modifier::ITALIC);
        }
        paint_at(
            ui,
            x,
            area.y,
            col.width,
            &fit(&cell_text(index, col_index), col.width, col.right),
            style,
        );
        x = x.saturating_add(col.width).saturating_add(COL_GAP);
    }
}

fn paint_customers(ui: &mut Ui<'_>, card: Rect) {
    let inner_probe = card_inner(card);
    let has_room =
        inner_probe.height > 2 && inner_probe.width > GUTTER_W.saturating_add(COL_RESERVE);
    let mut cols_w = inner_probe
        .width
        .saturating_sub(GUTTER_W)
        .saturating_sub(COL_RESERVE)
        .saturating_sub(1);
    let mut shown = columns_fit(cols_w);
    let has_sb = has_room && CONTENT_ROWS > usize::from(inner_probe.height.saturating_sub(2));
    if !has_sb {
        cols_w = inner_probe
            .width
            .saturating_sub(GUTTER_W)
            .saturating_sub(COL_RESERVE);
        shown = columns_fit(cols_w);
    }
    let body_rows = inner_probe.height.saturating_sub(2);
    let shown_rows = usize::from(body_rows).min(LOADED);
    let meta = format!(
        "rows 1–{shown_rows} of {LOADED} loaded · ~4,812 total · cols 1–{shown} of {ALL_COLS}"
    );
    paint_card_chrome(ui, card, "customers", &meta);
    let inner = card_inner(card);
    if inner.height < 2 || shown == 0 {
        return;
    }
    paint_grid_header(ui, inner, cols_w, shown);
    let row_w = inner.width.saturating_sub(u16::from(has_sb));
    for index in 0..usize::from(body_rows).min(LOADED) {
        let y = inner
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        paint_grid_row(
            ui,
            Rect::new(inner.x, y, row_w, 1),
            index,
            shown,
            row_fade(index, usize::from(body_rows)),
        );
    }
    if has_sb && body_rows > 0 {
        paint_vertical_bar(
            ui,
            Rect::new(
                inner.right().saturating_sub(1),
                inner.y.saturating_add(1),
                1,
                body_rows,
            ),
            CONTENT_ROWS,
            body_rows,
            SURFACE,
        );
    }
    if shown < ALL_COLS && body_rows > 0 {
        let x = inner.x.saturating_add(GUTTER_W);
        let right = inner.right().saturating_sub(u16::from(has_sb));
        paint_horizontal_bar(
            ui,
            Rect::new(
                x,
                inner.y.saturating_add(1).saturating_add(body_rows),
                right.saturating_sub(x),
                1,
            ),
            shown,
            ALL_COLS,
        );
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
    let view = if overflows {
        Rect {
            width: area.width.saturating_sub(SCROLL_ROOM),
            ..area
        }
    } else {
        area
    };
    let card_h = GRID_H.min(view.height);
    if card_h > 0 {
        paint_customers(ui, Rect::new(view.x, view.y, view.width, card_h));
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
    if overflows {
        paint_page_scrollbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
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
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, body);
            paint_section_strip(
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
