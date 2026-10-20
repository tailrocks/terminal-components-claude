use termrock::{Color, Cx, Modifier, PaintStyle, Rect, Response, Ui, width};

use showcase_data::{TASKS, TaskRow, TaskStatus};

use super::{Page, PageUpdate, frame};

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Navigation is reversed cell; editing is a cursor. They never look alike.";
const TITLE: &str = "Editable tables";

const ROW_COUNT: usize = 14;
const TASKS_H: u16 = 18;
const LEGEND_H: u16 = 4;
const COLUMNS_H: u16 = 9;
const EVENTS_H: u16 = 9;
const CARD_GAP: u16 = 1;
const SCROLL_ROOM: u16 = 2;
const COL_GAP: u16 = 2;
const TABLE_INSET: u16 = 3;
const TABLE_CHROME: u16 = 5;

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
const FAILED: u32 = 0xf05d5d;
const PAUSED: u32 = 0xf59e09;

const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;

const LEGEND: [(&str, &str); 4] = [
    ("reversed", "cell cursor (navigation)"),
    ("▁", "editing cursor + accent underline"),
    ("!", "validation error"),
    ("▾", "Owner: pick from a list (Enter, Space, Alt+↓)"),
];

const COLUMN_FACTS: [(&str, &str); 6] = [
    ("ID", "left · read-only"),
    ("Task", "left · editable · text"),
    ("Owner", "left · editable · ▾ Select"),
    ("Status", "left · read-only"),
    ("Branch", "left · editable · text"),
    ("Changes", "right · editable · text"),
];

const EVENT_FACTS: [(&str, &str); 6] = [
    ("Enter / F2 / click twice", "Edit · Enter again Committed"),
    ("Esc", "Cancel the edit · Cancelled"),
    (
        "Tab at the last cell",
        "LeaveForward · Shift+Tab LeaveBackward",
    ),
    ("Enter on a row (Basic table)", "Activated"),
    ("Space (Selection)", "SelectionChanged"),
    (
        "s or a header click",
        "Sort ▴ ascending, again ▾ descending",
    ),
];

struct Spec {
    title: &'static str,
    min: u16,
    flex: bool,
    right: bool,
}

const TASK_SPECS: [Spec; 6] = [
    Spec {
        title: "ID",
        min: 5,
        flex: false,
        right: false,
    },
    Spec {
        title: "Task",
        min: 24,
        flex: true,
        right: false,
    },
    Spec {
        title: "Owner",
        min: 8,
        flex: false,
        right: false,
    },
    Spec {
        title: "Status",
        min: 9,
        flex: false,
        right: false,
    },
    Spec {
        title: "Branch",
        min: 22,
        flex: false,
        right: false,
    },
    Spec {
        title: "Changes",
        min: 8,
        flex: false,
        right: true,
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

fn columns_fit(mins: &[u16], avail: u16) -> usize {
    let mut used = 0u16;
    let mut fit = 0usize;
    for &min in mins {
        let need = if fit == 0 {
            min
        } else {
            min.saturating_add(COL_GAP)
        };
        if used.saturating_add(need) > avail {
            break;
        }
        used = used.saturating_add(need);
        fit += 1;
    }
    fit
}

fn grow(specs: &[Spec], avail: u16) -> Vec<u16> {
    if specs.is_empty() {
        return Vec::new();
    }
    let gaps = COL_GAP.saturating_mul(u16::try_from(specs.len() - 1).unwrap_or(u16::MAX));
    let budget = avail.saturating_sub(gaps);
    let mut widths: Vec<u16> = specs.iter().map(|spec| spec.min).collect();
    let used = widths.iter().copied().fold(0u16, u16::saturating_add);
    let extra = budget.saturating_sub(used);
    let flex: Vec<usize> = specs
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.flex)
        .map(|(index, _)| index)
        .collect();
    if flex.is_empty() || extra == 0 {
        return widths;
    }
    let share = extra / u16::try_from(flex.len()).unwrap_or(1);
    let mut remainder = extra % u16::try_from(flex.len()).unwrap_or(1);
    for index in flex {
        widths[index] = widths[index].saturating_add(share);
        if remainder > 0 {
            widths[index] = widths[index].saturating_add(1);
            remainder -= 1;
        }
    }
    widths
}

struct TableGeom {
    cols_w: u16,
    has_sb: bool,
    has_bar: bool,
    body: u16,
    widths: Vec<u16>,
    shown: usize,
}

fn table_geom(area: Rect, specs: &[Spec], rows: usize) -> Option<TableGeom> {
    if area.height < 2 || area.width <= TABLE_CHROME {
        return None;
    }
    let mut has_sb = rows > usize::from(area.height.saturating_sub(1));
    let mut cols_w = area
        .width
        .saturating_sub(TABLE_CHROME)
        .saturating_sub(u16::from(has_sb));
    let mins: Vec<u16> = specs.iter().map(|spec| spec.min).collect();
    let mut shown = columns_fit(&mins, cols_w);
    let has_bar = rows > 0 && area.height > 2 && shown < specs.len();
    let body = if has_bar {
        let body = area.height.saturating_sub(2);
        if !has_sb && rows > usize::from(body) {
            has_sb = true;
            cols_w = area.width.saturating_sub(TABLE_CHROME).saturating_sub(1);
            shown = columns_fit(&mins, cols_w);
        }
        body
    } else {
        area.height.saturating_sub(1)
    };
    Some(TableGeom {
        cols_w,
        has_sb,
        has_bar,
        body,
        widths: grow(&specs[..shown], cols_w),
        shown,
    })
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

fn row_fade(index: usize, viewport: usize, content: usize, offset: usize) -> Option<f32> {
    if viewport < 4 || content <= viewport {
        return None;
    }
    let depth = usize::from(viewport >= 12) + 1;
    let down = offset.saturating_add(viewport) < content;
    if down && index + 1 == viewport {
        return Some(FADE_OUTER);
    }
    if down && depth == 2 && index + 2 == viewport {
        return Some(FADE_INNER);
    }
    if offset > 0 && index == 0 {
        return Some(FADE_OUTER);
    }
    if offset > 0 && depth == 2 && index == 1 {
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
    let title_w = width(TITLE);
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

fn paint_vertical_bar(ui: &mut Ui<'_>, area: Rect, rows: usize, viewport: u16) {
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
            if on {
                ink(THUMB, SURFACE)
            } else {
                ink(TRACK, SURFACE)
            },
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

struct PaintedCell<'a> {
    text: &'a str,
    fg: u32,
}

struct RowLook<'a> {
    specs: &'a [Spec],
    widths: &'a [u16],
    cells: &'a [PaintedCell<'a>],
    bg: u32,
    keep: Option<f32>,
}

fn paint_table_row(ui: &mut Ui<'_>, area: Rect, look: RowLook<'_>) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, ink(faded(WHITE, look.bg, look.keep), look.bg));
    paint_at(ui, area.x, area.y, 1, " ", ink(look.bg, look.bg));
    let mut x = area.x.saturating_add(TABLE_INSET);
    for (index, col_w) in look.widths.iter().copied().enumerate() {
        let Some(cell) = look.cells.get(index) else {
            break;
        };
        let right = look.specs.get(index).is_some_and(|spec| spec.right);
        paint_at(
            ui,
            x,
            area.y,
            col_w,
            &fit(cell.text, col_w, right),
            ink(faded(cell.fg, look.bg, look.keep), look.bg),
        );
        x = x.saturating_add(col_w).saturating_add(COL_GAP);
    }
}

fn paint_header(ui: &mut Ui<'_>, area: Rect, specs: &[Spec], geom: &TableGeom) {
    let mut x = area.x.saturating_add(TABLE_INSET);
    for (spec, col_w) in specs.iter().zip(geom.widths.iter().copied()) {
        paint_at(
            ui,
            x,
            area.y,
            col_w,
            &fit(spec.title, col_w, spec.right),
            ink(MUTED, SURFACE),
        );
        x = x.saturating_add(col_w).saturating_add(COL_GAP);
    }
    if geom.shown < specs.len() {
        paint_at(
            ui,
            area.x
                .saturating_add(TABLE_INSET)
                .saturating_add(geom.cols_w)
                .saturating_add(1),
            area.y,
            1,
            "…",
            ink(FAINT, SURFACE),
        );
    }
}

fn paint_scrollbars(ui: &mut Ui<'_>, area: Rect, geom: &TableGeom, rows: usize, columns: usize) {
    if geom.has_sb && geom.body > 0 {
        paint_vertical_bar(
            ui,
            Rect::new(
                area.right().saturating_sub(1),
                area.y.saturating_add(1),
                1,
                geom.body,
            ),
            rows,
            geom.body,
        );
    }
    if geom.has_bar {
        let x = area.x.saturating_add(TABLE_INSET);
        let right = area.right().saturating_sub(u16::from(geom.has_sb));
        paint_horizontal_bar(
            ui,
            Rect::new(
                x,
                area.y.saturating_add(1).saturating_add(geom.body),
                right.saturating_sub(x),
                1,
            ),
            geom.shown,
            columns,
        );
    }
}

fn status_fg(status: TaskStatus) -> u32 {
    match status {
        TaskStatus::Running => WHITE,
        TaskStatus::Failed => FAILED,
        TaskStatus::Paused => PAUSED,
        TaskStatus::Queued => MUTED,
        TaskStatus::Done => SECONDARY,
    }
}

fn status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Running => "▸ Running",
        TaskStatus::Failed => "Failed",
        TaskStatus::Paused => "Paused",
        TaskStatus::Queued => "Queued",
        TaskStatus::Done => "Done",
    }
}

fn branch_text(task: &TaskRow) -> &'static str {
    if task.id == 1042 {
        "fix/checkout flake"
    } else {
        task.branch
    }
}

fn task_field(task: &TaskRow, index: usize, col_w: u16) -> (String, u32) {
    match index {
        0 => (format!("#{}", task.id), MUTED),
        1 => (termrock::truncate(task.name, col_w), WHITE),
        2 => (termrock::truncate(task.owner, col_w), WHITE),
        3 => (status_text(task.status).to_owned(), status_fg(task.status)),
        4 => (
            termrock::truncate(branch_text(task), col_w),
            if task.id == 1042 { FAILED } else { MUTED },
        ),
        5 => (task.changes.to_string(), WHITE),
        _ => (String::new(), WHITE),
    }
}

fn tasks_meta(body: u16, rows: usize) -> String {
    if rows > usize::from(body) {
        format!("0 edits · 1–{body} of {rows}")
    } else {
        "0 edits".to_owned()
    }
}

fn content_height() -> u16 {
    TASKS_H
        .saturating_add(LEGEND_H)
        .saturating_add(COLUMNS_H)
        .saturating_add(EVENTS_H)
        .saturating_add(CARD_GAP.saturating_mul(3))
}

fn lines_for(text: &str, columns: u16) -> u16 {
    let columns = columns.max(1);
    let text_w = width(text);
    if text_w == 0 {
        return 1;
    }
    text_w.saturating_add(columns.saturating_sub(1)) / columns
}

fn fact_lines(facts: &[(&str, &str)], area_w: u16) -> u16 {
    let value_w = area_w.saturating_sub(label_column(facts)).max(1);
    facts.iter().fold(0u16, |total, (_, value)| {
        total.saturating_add(lines_for(value, value_w))
    })
}

fn content_height_for(content_w: u16) -> u16 {
    let legend = fact_lines(&LEGEND, content_w).max(LEGEND_H);
    let inner = content_w.saturating_sub(4);
    let columns = fact_lines(&COLUMN_FACTS, inner)
        .saturating_add(3)
        .max(COLUMNS_H);
    let events = fact_lines(&EVENT_FACTS, inner)
        .saturating_add(3)
        .max(EVENTS_H);
    TASKS_H
        .saturating_add(legend)
        .saturating_add(columns)
        .saturating_add(events)
        .saturating_add(CARD_GAP.saturating_mul(3))
        .max(content_height())
}

fn columns_shown(card_w: u16) -> usize {
    table_geom(
        card_inner(Rect::new(0, 0, card_w, TASKS_H)),
        &TASK_SPECS,
        ROW_COUNT,
    )
    .map(|geom| geom.shown)
    .unwrap_or(0)
}

fn layout_width(area: Rect) -> u16 {
    let full = area.width;
    let shrunk = full.saturating_sub(SCROLL_ROOM);
    if full <= SCROLL_ROOM || content_height_for(full) <= area.height {
        return full;
    }
    if columns_shown(shrunk) == columns_shown(full) {
        shrunk
    } else {
        full
    }
}

struct ScrollSplit {
    view: Rect,
    vertical: bool,
    horizontal: bool,
}

fn scroll_split(area: Rect, content_w: u16, content_h: u16) -> ScrollSplit {
    let (mut vertical, mut horizontal) = (false, false);
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        let view_h = area.height.saturating_sub(u16::from(horizontal));
        let next_v = content_h > view_h;
        let next_h = content_w > view_w;
        if (next_v, next_h) == (vertical, horizontal) {
            break;
        }
        vertical = next_v;
        horizontal = next_h;
    }
    let vertical = vertical && area.width > SCROLL_ROOM;
    let horizontal = horizontal && area.height > 1;
    let view = Rect {
        width: area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 }),
        height: area.height.saturating_sub(u16::from(horizontal)),
        ..area
    };
    ScrollSplit {
        view,
        vertical,
        horizontal,
    }
}

fn paint_page_scrollbars(
    ui: &mut Ui<'_>,
    area: Rect,
    split: &ScrollSplit,
    content_w: u16,
    content_h: u16,
) {
    if split.vertical {
        let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, split.view.height);
        let (start, len) = thumb(
            0,
            usize::from(split.view.height),
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
    if split.horizontal {
        let width = area.width.saturating_sub(u16::from(split.vertical));
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), width, 1);
        let (start, len) = thumb(
            0,
            usize::from(split.view.width),
            usize::from(content_w),
            usize::from(bar.width),
        );
        for col in 0..bar.width {
            let on = usize::from(col) >= start && usize::from(col) < start.saturating_add(len);
            paint_at(
                ui,
                bar.x.saturating_add(col),
                bar.y,
                1,
                if on { "━" } else { "┄" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
}

fn paint_tasks(ui: &mut Ui<'_>, card: Rect) {
    let rows: Vec<&TaskRow> = TASKS.iter().take(ROW_COUNT).collect();
    let inner = card_inner(card);
    let Some(geom) = table_geom(inner, &TASK_SPECS, rows.len()) else {
        paint_card_chrome(ui, card, "Tasks", "0 edits");
        return;
    };
    let meta = tasks_meta(geom.body, rows.len());
    paint_card_chrome(ui, card, "Tasks", &meta);
    paint_header(ui, Rect { height: 1, ..inner }, &TASK_SPECS, &geom);
    let row_w = inner.width.saturating_sub(u16::from(geom.has_sb));
    for (index, task) in rows
        .iter()
        .copied()
        .enumerate()
        .take(usize::from(geom.body))
    {
        let owned: Vec<(String, u32)> = (0..geom.shown)
            .map(|col| task_field(task, col, geom.widths[col]))
            .collect();
        let cells: Vec<PaintedCell<'_>> = owned
            .iter()
            .map(|(text, fg)| PaintedCell { text, fg: *fg })
            .collect();
        let keep = row_fade(index, usize::from(geom.body), rows.len(), 0);
        let y = inner
            .y
            .saturating_add(1)
            .saturating_add(u16::try_from(index).unwrap_or(u16::MAX));
        paint_table_row(
            ui,
            Rect::new(inner.x, y, row_w, 1),
            RowLook {
                specs: &TASK_SPECS,
                widths: &geom.widths,
                cells: &cells,
                bg: SURFACE,
                keep,
            },
        );
        if task.id == 1042 && geom.shown > 4 {
            let mut x = inner.x.saturating_add(TABLE_INSET);
            for col_w in geom.widths.iter().take(4) {
                x = x.saturating_add(*col_w).saturating_add(COL_GAP);
            }
            let bang_x = x.saturating_add(geom.widths[4].saturating_sub(1));
            paint_at(
                ui,
                bang_x,
                y,
                1,
                "!",
                ink(faded(FAILED, SURFACE, keep), SURFACE).add_modifier(Modifier::BOLD),
            );
        }
    }
    paint_scrollbars(ui, inner, &geom, rows.len(), TASK_SPECS.len());
}

fn label_column(facts: &[(&str, &str)]) -> u16 {
    facts
        .iter()
        .map(|(label, _)| width(label).saturating_add(2))
        .max()
        .unwrap_or(2)
}

fn paint_facts(ui: &mut Ui<'_>, area: Rect, facts: &[(&str, &str)], bg: u32) {
    if area.is_empty() {
        return;
    }
    let label_w = label_column(facts);
    for (index, (label, value)) in facts.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = area.y.saturating_add(offset);
        if y >= area.bottom() {
            break;
        }
        paint_at(ui, area.x, y, width(label), label, ink(MUTED, bg));
        let value_x = area.x.saturating_add(label_w);
        let value_w = area.right().saturating_sub(value_x);
        paint_at(ui, value_x, y, value_w, value, ink(WHITE, bg));
    }
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let content_w = layout_width(area);
    let content_h = content_height_for(content_w);
    let split = scroll_split(area, content_w, content_h);
    let content = Rect::new(area.x, area.y, content_w, content_h);
    ui.with_area(split.view, |ui| paint_playground_body(ui, content));
    paint_page_scrollbars(ui, area, &split, content_w, content_h);
}

fn paint_playground_body(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut y = area.y;
    if y < area.bottom() {
        paint_tasks(ui, Rect::new(area.x, y, area.width, TASKS_H));
    }
    y = y.saturating_add(TASKS_H).saturating_add(CARD_GAP);
    if y < area.bottom() {
        paint_facts(
            ui,
            Rect::new(area.x, y, area.width, LEGEND_H),
            &LEGEND,
            CANVAS,
        );
    }
    y = y.saturating_add(LEGEND_H).saturating_add(CARD_GAP);
    if y < area.bottom() {
        let card = Rect::new(area.x, y, area.width, COLUMNS_H);
        paint_card_chrome(ui, card, "Columns", "");
        paint_facts(ui, card_inner(card), &COLUMN_FACTS, SURFACE);
    }
    y = y.saturating_add(COLUMNS_H).saturating_add(CARD_GAP);
    if y < area.bottom() {
        let card = Rect::new(area.x, y, area.width, EVENTS_H);
        paint_card_chrome(ui, card, "Events", "");
        paint_facts(ui, card_inner(card), &EVENT_FACTS, SURFACE);
    }
}

#[derive(Debug, Default)]
pub struct EditablePage;

impl EditablePage {
    pub fn new() -> Self {
        Self
    }
}

impl Page for EditablePage {
    fn title(&self) -> &'static str {
        TITLE
    }

    fn update(&mut self, _cx: &mut Cx<'_>) -> PageUpdate {
        Response::ignored().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.with_area(area, |ui| self.draw_clipped(ui, area));
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[
            ("↑ ↓ ← →", "Cell"),
            ("Enter", "Edit"),
            ("s", "Sort"),
            ("click twice", "Edit"),
        ]
    }
}

impl EditablePage {
    fn draw_clipped(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, body);
            paint_section_strip(
                ui,
                Rect {
                    height: 2.min(body.height),
                    ..body
                },
            );
            let top = body.y.saturating_add(3);
            if top >= body.bottom() {
                return;
            }
            paint_playground(
                ui,
                Rect {
                    y: top,
                    height: body.bottom().saturating_sub(top),
                    ..body
                },
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{Page, ROW_COUNT, content_height};
    use crate::pages::editable::EditablePage;
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            EditablePage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_matches_the_120_column_body() {
        assert_eq!(content_height(), 18 + 4 + 9 + 9 + 3);
        assert_eq!(ROW_COUNT, 14);
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Editable tables"), "{text}");
        assert!(text.contains(super::BLURB), "{text}");
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("0 edits · 1–13 of 14"), "{text}");
        assert!(text.contains("Add rate limiting to auth end…"), "{text}");
        assert!(text.contains("fix/checkout flake"), "{text}");
        assert!(text.contains("left · editable · ▾ Select"), "{text}");
        assert!(!text.contains("#1053"), "{text}");

        let (x, y) = h.find("Playground").expect("tab");
        let tab = h.cell(x, y);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(tab.modifier.contains(Modifier::BOLD));

        let (fx, fy) = h.find("Failed").expect("failed status");
        assert_eq!(h.cell(fx, fy).fg, Color::Rgb(0xf0, 0x5d, 0x5d));

        let (bx, by) = h.find("flake").expect("branch error");
        let mut bang = None;
        for col in bx..94 {
            if h.cell(col, by).symbol() == "!" {
                bang = Some(col);
                break;
            }
        }
        let bang_x = bang.expect("error marker");
        assert!(h.cell(bang_x, by).modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(bang_x, by).fg, Color::Rgb(0xf0, 0x5d, 0x5d));
    }
}
