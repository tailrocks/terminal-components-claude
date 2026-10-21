use termrock::grid::{Column, ColumnKey, Table, TableAction, TableColumn, TableSort, TableState};
use termrock::{
    CellRef, Color, Cx, FgStep, GridAction, GridModel, ItemKey, Modifier, PaintStyle, Rect, Role,
    StylePatch, Ui, width,
};

use showcase_data::{TASKS, TaskStatus};

use super::{Page, PageStatus, PageUpdate, chrome_ink};
use termrock::feedback::{CardChrome, PageFrame, SectionStrip};

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Sort by header, hover rows, select with Enter, overflow scrolls";

const TASKS_H: u16 = 25;
const REVIEWERS_H: u16 = 8;
const CHECKS_H: u16 = 6;
const CARD_GAP: u16 = 1;
const SCROLL_ROOM: u16 = 2;
const COL_GAP: u16 = 2;
const TABLE_INSET: u16 = 3;
const TABLE_CHROME: u16 = 5;
const TASK_TABLE_ID: termrock::Id = termrock::id!("tables.tasks");
const TASK_TONE_STYLES: [(Role, StylePatch); 1] = [(
    Role::Danger,
    StylePatch::new().set_fg(Role::Custom(Color::Rgb(240, 93, 93))),
)];

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const ACCENT: u32 = 0x48e054;
const ACCENT_BG: u32 = 0x173f1c;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;

const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;

const CHECK_OFF: &str = "[ ]";
const CHECK_ON: &str = "[✓]";
const CHECK_MIXED: &str = "[–]";

struct Reviewer {
    name: &'static str,
    team: &'static str,
    open: &'static str,
    on: bool,
}

const REVIEWERS: [Reviewer; 4] = [
    Reviewer {
        name: "mira",
        team: "platform",
        open: "3",
        on: true,
    },
    Reviewer {
        name: "jonas",
        team: "payments",
        open: "0",
        on: false,
    },
    Reviewer {
        name: "ana",
        team: "platform",
        open: "5",
        on: true,
    },
    Reviewer {
        name: "kai",
        team: "infra",
        open: "1",
        on: false,
    },
];

struct Spec {
    title: &'static str,
    min: u16,
    flex: bool,
    right: bool,
}

const TASK_COLUMNS: [TableColumn<'static>; 7] = [
    TableColumn::new(Column {
        min_width: 5,
        max_width: 5,
        sortable: true,
        ..Column::new(ColumnKey::num(0), "ID")
    })
    .sort(TableSort::NumericDisplay),
    TableColumn::new(Column {
        min_width: 24,
        max_width: 80,
        ..Column::new(ColumnKey::num(1), "Task")
    })
    .flexible(),
    TableColumn::new(Column {
        min_width: 7,
        max_width: 7,
        sortable: true,
        ..Column::new(ColumnKey::num(2), "Owner")
    }),
    TableColumn::new(Column {
        min_width: 9,
        max_width: 9,
        sortable: true,
        ..Column::new(ColumnKey::num(3), "Status")
    }),
    TableColumn::new(Column {
        min_width: 20,
        max_width: 20,
        sortable: true,
        ..Column::new(ColumnKey::num(4), "Branch")
    }),
    TableColumn::new(Column {
        min_width: 9,
        max_width: 9,
        sortable: true,
        align: termrock::Align::Right,
        ..Column::new(ColumnKey::num(5), "Changes")
    })
    .sort(TableSort::NumericDisplay),
    TableColumn::new(Column {
        min_width: 9,
        max_width: 9,
        sortable: true,
        align: termrock::Align::Right,
        ..Column::new(ColumnKey::num(6), "Duration")
    })
    .sort(TableSort::NumericDisplay),
];

const REVIEWER_SPECS: [Spec; 3] = [
    Spec {
        title: "Reviewer",
        min: 12,
        flex: true,
        right: false,
    },
    Spec {
        title: "Team",
        min: 10,
        flex: false,
        right: false,
    },
    Spec {
        title: "Open reviews",
        min: 12,
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
    body: u16,
    widths: Vec<u16>,
    shown: usize,
}

fn table_geom(area: Rect, specs: &[Spec], rows: usize, check_w: u16) -> Option<TableGeom> {
    if area.height < 2 || area.width <= TABLE_CHROME.saturating_add(check_w) {
        return None;
    }
    let mut has_sb = rows > usize::from(area.height.saturating_sub(1));
    let mut cols_w = area
        .width
        .saturating_sub(TABLE_CHROME)
        .saturating_sub(check_w)
        .saturating_sub(u16::from(has_sb));
    let mins: Vec<u16> = specs.iter().map(|spec| spec.min).collect();
    let mut shown = columns_fit(&mins, cols_w);
    let has_bar = rows > 0 && area.height > 2 && shown < specs.len();
    let body = if has_bar {
        let body = area.height.saturating_sub(2);
        if !has_sb && rows > usize::from(body) {
            has_sb = true;
            cols_w = area
                .width
                .saturating_sub(TABLE_CHROME)
                .saturating_sub(check_w)
                .saturating_sub(1);
            shown = columns_fit(&mins, cols_w);
        }
        body
    } else {
        area.height.saturating_sub(1)
    };
    Some(TableGeom {
        cols_w,
        has_sb,
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

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Tables");
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

struct PaintedCell<'a> {
    text: &'a str,
    fg: u32,
}

struct RowLook<'a> {
    specs: &'a [Spec],
    widths: &'a [u16],
    cells: &'a [PaintedCell<'a>],
    check: Option<(u32, &'static str)>,
    bg: u32,
    bold: bool,
    keep: Option<f32>,
}

fn paint_table_row(ui: &mut Ui<'_>, area: Rect, look: RowLook<'_>) {
    if area.is_empty() {
        return;
    }
    let mut style = ink(faded(WHITE, look.bg, look.keep), look.bg);
    if look.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    ui.fill(area, style);
    paint_at(ui, area.x, area.y, 1, " ", ink(look.bg, look.bg));
    if let Some((fg, glyph)) = look.check {
        let mut mark = ink(faded(fg, look.bg, look.keep), look.bg);
        if look.bold {
            mark = mark.add_modifier(Modifier::BOLD);
        }
        paint_at(
            ui,
            area.x.saturating_add(TABLE_INSET),
            area.y,
            3,
            glyph,
            mark,
        );
    }
    let check_w = u16::from(look.check.is_some()).saturating_mul(4);
    let mut x = area.x.saturating_add(TABLE_INSET).saturating_add(check_w);
    for (index, width) in look.widths.iter().copied().enumerate() {
        let Some(cell) = look.cells.get(index) else {
            break;
        };
        let mut cell_style = ink(faded(cell.fg, look.bg, look.keep), look.bg);
        if look.bold {
            cell_style = cell_style.add_modifier(Modifier::BOLD);
        }
        let right = look.specs.get(index).is_some_and(|spec| spec.right);
        paint_at(
            ui,
            x,
            area.y,
            width,
            &fit(cell.text, width, right),
            cell_style,
        );
        x = x.saturating_add(width).saturating_add(COL_GAP);
    }
}

fn paint_header(
    ui: &mut Ui<'_>,
    area: Rect,
    specs: &[Spec],
    geom: &TableGeom,
    check: Option<&'static str>,
) {
    let check_w = u16::from(check.is_some()).saturating_mul(4);
    if let Some(glyph) = check {
        paint_at(
            ui,
            area.x.saturating_add(TABLE_INSET),
            area.y,
            3,
            glyph,
            ink(ACCENT, SURFACE),
        );
    }
    let mut x = area.x.saturating_add(TABLE_INSET).saturating_add(check_w);
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
                .saturating_add(check_w)
                .saturating_add(geom.cols_w)
                .saturating_add(1),
            area.y,
            1,
            "…",
            ink(FAINT, SURFACE),
        );
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

fn changes_text(changes: u32) -> String {
    changes.to_string()
}

fn duration_text(seconds: u32) -> String {
    if seconds >= 60 {
        format!("{}m {:02}s", seconds / 60, seconds % 60)
    } else {
        format!("{seconds}s")
    }
}

#[derive(Debug)]
struct TaskTableRow {
    id: u32,
    cells: [String; 7],
    tones: [Option<Role>; 7],
}

#[derive(Debug)]
struct TaskTableModel {
    rows: Vec<TaskTableRow>,
}

impl TaskTableModel {
    fn new() -> Self {
        let rows = TASKS
            .iter()
            .map(|task| {
                let status = status_text(task.status);
                let status_tone = match task.status {
                    TaskStatus::Running => Role::Fg(FgStep::Primary),
                    TaskStatus::Failed => Role::Danger,
                    TaskStatus::Paused => Role::Warning,
                    TaskStatus::Queued => Role::Fg(FgStep::Muted),
                    TaskStatus::Done => Role::Fg(FgStep::Secondary),
                };
                TaskTableRow {
                    id: task.id,
                    cells: [
                        format!("#{:04}", task.id),
                        task.name.to_owned(),
                        task.owner.to_owned(),
                        status.to_owned(),
                        task.branch.to_owned(),
                        changes_text(task.changes),
                        duration_text(task.duration_s),
                    ],
                    tones: [
                        Some(Role::Fg(FgStep::Muted)),
                        None,
                        None,
                        Some(status_tone),
                        Some(Role::Fg(FgStep::Muted)),
                        (task.changes == 0).then_some(Role::Fg(FgStep::Muted)),
                        None,
                    ],
                }
            })
            .collect();
        Self { rows }
    }
}

impl Default for TaskTableModel {
    fn default() -> Self {
        Self::new()
    }
}

impl GridModel for TaskTableModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::num(0), |task| ItemKey::num(u64::from(task.id)))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let task = self.rows.get(row)?;
        let text = task.cells.get(col)?;
        let cell = CellRef::new(text);
        Some(match task.tones.get(col).copied().flatten() {
            Some(tone) => cell.tone(tone),
            None => cell,
        })
    }
}

fn task_table() -> Table<'static> {
    Table::new(TASK_TABLE_ID, &TASK_COLUMNS).tone_styles(&TASK_TONE_STYLES)
}

fn draw_tasks(ui: &mut Ui<'_>, card: Rect, state: &TableState, model: &TaskTableModel) {
    let inner = card_inner(card);
    let table = task_table();
    let meta = table.position_label(state, model.row_count());
    CardChrome::new("Tasks")
        .meta(&meta)
        .ink(chrome_ink())
        .draw(ui, card);
    table.draw(ui, inner, state, model);
}

fn selection_meta() -> String {
    let selected = REVIEWERS.iter().filter(|row| row.on).count();
    let state = if selected == 0 {
        "none"
    } else if selected == REVIEWERS.len() {
        "all"
    } else {
        "mixed"
    };
    format!("{selected} selected · header {state}")
}

fn paint_reviewers(ui: &mut Ui<'_>, card: Rect) {
    CardChrome::new("Reviewers")
        .meta(&selection_meta())
        .ink(chrome_ink())
        .draw(ui, card);
    let inner = card_inner(card);
    let Some(geom) = table_geom(inner, &REVIEWER_SPECS, REVIEWERS.len(), 4) else {
        return;
    };
    let header_mark = if REVIEWERS.iter().all(|row| row.on) {
        CHECK_ON
    } else if REVIEWERS.iter().any(|row| row.on) {
        CHECK_MIXED
    } else {
        CHECK_OFF
    };
    paint_header(
        ui,
        Rect { height: 1, ..inner },
        &REVIEWER_SPECS,
        &geom,
        Some(header_mark),
    );
    for (index, row) in REVIEWERS.iter().enumerate().take(usize::from(geom.body)) {
        let cells = [
            PaintedCell {
                text: row.name,
                fg: WHITE,
            },
            PaintedCell {
                text: row.team,
                fg: MUTED,
            },
            PaintedCell {
                text: row.open,
                fg: WHITE,
            },
        ];
        paint_table_row(
            ui,
            Rect::new(
                inner.x,
                inner
                    .y
                    .saturating_add(1)
                    .saturating_add(u16::try_from(index).unwrap_or(u16::MAX)),
                inner.width.saturating_sub(u16::from(geom.has_sb)),
                1,
            ),
            RowLook {
                specs: &REVIEWER_SPECS,
                widths: &geom.widths,
                cells: &cells,
                check: Some((
                    if row.on { ACCENT } else { MUTED },
                    if row.on { CHECK_ON } else { CHECK_OFF },
                )),
                bg: if row.on { ACCENT_BG } else { SURFACE },
                bold: row.on,
                keep: row_fade(index, usize::from(geom.body), REVIEWERS.len(), 0),
            },
        );
    }
    if geom.has_sb && geom.body > 0 {
        paint_vertical_bar(
            ui,
            Rect::new(
                inner.right().saturating_sub(1),
                inner.y.saturating_add(1),
                1,
                geom.body,
            ),
            REVIEWERS.len(),
            geom.body,
        );
    }
}

fn paint_checks(ui: &mut Ui<'_>, card: Rect) {
    CardChrome::new("Checks")
        .meta("")
        .ink(chrome_ink())
        .draw(ui, card);
    let inner = card_inner(card);
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    let header = ink(MUTED, SURFACE);
    let check_x = inner.x.saturating_add(TABLE_INSET);
    let check_w = width("Check");
    paint_at(ui, check_x, inner.y, check_w, "Check", header);
    let result_w = width("Result");
    let result_x = card.right().saturating_sub(12);
    let gap_x = check_x.saturating_add(check_w);
    // Baseline keeps two unpainted cells before Result and two muted cells after it.
    let gap_w = result_x.saturating_sub(2).saturating_sub(gap_x);
    if gap_w > 0 {
        paint_at(
            ui,
            gap_x,
            inner.y,
            gap_w,
            &" ".repeat(usize::from(gap_w)),
            header,
        );
    }
    paint_at(ui, result_x, inner.y, result_w, "Result", header);
    paint_at(
        ui,
        result_x.saturating_add(result_w),
        inner.y,
        2,
        "  ",
        header,
    );
    if inner.height < 2 {
        return;
    }
    let title = "No checks have run yet";
    let body_y = inner.y.saturating_add(1);
    let body_h = inner.height.saturating_sub(1);
    let y = body_y.saturating_add(body_h / 2);
    let text_w = width(title).min(inner.width);
    paint_at(
        ui,
        inner
            .x
            .saturating_add(inner.width.saturating_sub(text_w) / 2),
        y,
        text_w,
        title,
        header,
    );
}

fn content_height() -> u16 {
    TASKS_H
        .saturating_add(REVIEWERS_H)
        .saturating_add(CHECKS_H)
        .saturating_add(CARD_GAP.saturating_mul(2))
}

fn paint_playground(
    ui: &mut Ui<'_>,
    area: Rect,
    task_state: &TableState,
    task_model: &TaskTableModel,
) {
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
    let mut y = view.y;
    let tasks = Rect::new(
        view.x,
        y,
        view.width,
        TASKS_H.min(view.bottom().saturating_sub(y)),
    );
    if tasks.height > 0 && y < view.bottom() {
        draw_tasks(
            ui,
            Rect::new(view.x, y, view.width, TASKS_H),
            task_state,
            task_model,
        );
    }
    y = y.saturating_add(TASKS_H).saturating_add(CARD_GAP);
    if y < view.bottom() {
        paint_reviewers(ui, Rect::new(view.x, y, view.width, REVIEWERS_H));
    }
    y = y.saturating_add(REVIEWERS_H).saturating_add(CARD_GAP);
    if y < view.bottom() {
        paint_checks(ui, Rect::new(view.x, y, view.width, CHECKS_H));
    }
    if overflows {
        paint_page_scrollbar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, view.height),
            content_h,
        );
    }
}

#[derive(Debug, Default)]
pub struct TablesPage {
    task_model: TaskTableModel,
    task_state: TableState,
}

impl TablesPage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Page for TablesPage {
    fn title(&self) -> &'static str {
        "Tables"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let response = task_table().update(cx, &mut self.task_state, &self.task_model);
        let status = match response.action_ref() {
            Some(TableAction::Grid(GridAction::Activated(ItemKey::Num(id)))) => TASKS
                .iter()
                .find(|task| u64::from(task.id) == *id)
                .map(|task| PageStatus(format!("Selected {}", task.name))),
            _ => None,
        };
        let mut update = PageUpdate::from(response.erase());
        update.status = status;
        update
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.with_area(area, |ui| self.draw_clipped(ui, area));
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[
            ("↑ ↓", "Move"),
            ("← →", "Columns"),
            ("s", "Sort column"),
            ("Enter", "Select"),
        ]
    }
}

impl TablesPage {
    fn draw_clipped(&self, ui: &mut Ui<'_>, area: Rect) {
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
                    &self.task_state,
                    &self.task_model,
                );
            });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CARD_GAP, Page, TASK_COLUMNS, TASK_TABLE_ID, TASKS, TableSort, TablesPage, TaskStatus,
        TaskTableModel, content_height, status_text, table_geom,
    };
    use crate::pages::tables::REVIEWER_SPECS;
    use termrock::{
        App, Color, Cx, FgStep, GridModel, ItemKey, KeyCode, Modifier, Rect, Response, Role,
        SortDir, Theme, Ui, UpdateCause,
    };
    use termrock_test_support::Harness;

    #[derive(Default)]
    struct Host {
        page: TablesPage,
        status: Option<String>,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(TASK_TABLE_ID);
            }
            let update = self.page.update(cx);
            if let Some(status) = update.status {
                self.status = Some(status.0);
            }
            update.response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, ui.full());
        }
    }

    #[test]
    fn task_columns_declare_baseline_minimums_and_numeric_sort_types() {
        assert_eq!(TASK_COLUMNS.len(), 7);
        assert_eq!(
            TASK_COLUMNS.map(|column| column.column.title),
            [
                "ID", "Task", "Owner", "Status", "Branch", "Changes", "Duration"
            ]
        );
        assert_eq!(
            TASK_COLUMNS.map(|column| column.column.min_width),
            [5, 24, 7, 9, 20, 9, 9]
        );
        assert!(TASK_COLUMNS[1].flexible);
        assert_eq!(TASK_COLUMNS[0].sort, TableSort::NumericDisplay);
        assert_eq!(TASK_COLUMNS[5].sort, TableSort::NumericDisplay);
        assert_eq!(TASK_COLUMNS[6].sort, TableSort::NumericDisplay);
        assert_eq!(TASK_COLUMNS[5].column.align, termrock::Align::Right);
        assert_eq!(TASK_COLUMNS[6].column.align, termrock::Align::Right);
        assert_eq!(content_height(), 25 + 8 + 6 + CARD_GAP * 2);
    }

    #[test]
    fn wide_task_columns_keep_branch_and_numeric_widths() {
        assert_eq!(super::duration_text(412), "6m 52s");
        assert_eq!(super::duration_text(0), "0s");
        assert_eq!(super::changes_text(14), "14");
    }

    #[test]
    fn reviewer_columns_keep_the_check_gutter() {
        let area = Rect::new(0, 0, 88, 5);
        let geom = table_geom(area, &REVIEWER_SPECS, 4, 4).expect("geom");
        assert!(!geom.has_sb);
        assert_eq!(geom.body, 4);
        assert_eq!(geom.widths, vec![53, 10, 12]);
    }

    #[test]
    fn task_table_model_exposes_source_order_keys_and_formatted_cells() {
        let model = TaskTableModel::new();
        assert_eq!(model.row_count(), 24);
        assert_eq!(model.row_key(0), ItemKey::num(1040));
        assert_eq!(model.row_key(23), ItemKey::num(1063));
        assert_eq!(model.cell(0, 0).expect("ID").text, "#1040");
        assert_eq!(
            model.cell(0, 1).expect("task").text,
            "Add rate limiting to auth endpoints"
        );
        assert_eq!(model.cell(0, 3).expect("status").text, "Done");
        assert_eq!(model.cell(0, 6).expect("duration").text, "6m 52s");
        assert_eq!(model.cell(2, 3).expect("failed").tone, Some(Role::Danger));
        assert_eq!(
            model.cell(3, 5).expect("zero changes").tone,
            Some(Role::Fg(FgStep::Muted))
        );
        for (index, task) in TASKS.iter().enumerate() {
            let expected_tone = match task.status {
                TaskStatus::Running => Role::Fg(FgStep::Primary),
                TaskStatus::Failed => Role::Danger,
                TaskStatus::Paused => Role::Warning,
                TaskStatus::Queued => Role::Fg(FgStep::Muted),
                TaskStatus::Done => Role::Fg(FgStep::Secondary),
            };
            let status = model.cell(index, 3).expect("status");
            assert_eq!(status.text, status_text(task.status));
            assert_eq!(status.tone, Some(expected_tone));
        }
    }

    #[test]
    fn stacked_playground_matches_the_120_column_body() {
        let mut h = Harness::new(Host::default(), Theme::junie(), 94, 36);
        let _ = h.tick();
        let text = h.text();
        assert!(text.contains("Tables"), "{text}");
        assert!(
            text.contains("Sort by header, hover rows, select with Enter, overflow scrolls"),
            "{text}"
        );
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("unsorted · 1–20 of 24"), "{text}");
        assert!(text.contains("2 selected · header mixed"), "{text}");
        assert!(text.contains("#1040"), "{text}");
        assert!(text.contains("Add rate limiting to auth endpoi…"), "{text}");
        assert!(text.contains("#1059"), "{text}");
        assert!(text.contains("[–]"), "{text}");
        assert!(text.contains("[✓]"), "{text}");
        assert!(text.contains("mira"), "{text}");
        assert!(!text.contains("No checks have run yet"), "{text}");
        assert!(!text.contains("#1060"), "{text}");

        let (x, y) = h.find("Playground").expect("tab");
        let tab = h.cell(x, y);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(tab.modifier.contains(Modifier::BOLD));

        let (fx, fy) = h.find("Failed").expect("failed status");
        assert_eq!(h.cell(fx, fy).fg, Color::Rgb(0xf0, 0x5d, 0x5d));

        let (mx, my) = h.find("[✓]").expect("selected reviewer");
        assert_eq!(h.cell(mx, my).bg, Color::Rgb(0x17, 0x3f, 0x1c));
        assert!(h.cell(mx, my).modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(mx, my).fg, Color::Rgb(0x48, 0xe0, 0x54));
    }

    #[test]
    fn activating_task_returns_its_domain_result() {
        let mut h = Harness::new(Host::default(), Theme::junie(), 120, 40);
        let _ = h.tick();
        let _ = h.key(KeyCode::Enter);
        assert_eq!(
            h.app().status.as_deref(),
            Some("Selected Add rate limiting to auth endpoints")
        );
    }

    #[test]
    fn clicking_owner_header_sorts_the_task_model() {
        let mut h = Harness::new(Host::default(), Theme::junie(), 120, 40);
        let _ = h.tick();
        let initial_row = h
            .find("Add rate limiting to auth endpoints")
            .expect("initial task row");
        let (x, y) = h.find("Owner").expect("owner header");
        let _ = h.click(x, y);
        assert_eq!(
            h.app().page.task_state.sort(),
            Some((termrock::grid::ColumnKey::num(2), SortDir::Asc))
        );
        let sorted_row = h
            .find("Add rate limiting to auth endpoints")
            .expect("sorted task row");
        assert_ne!(initial_row.1, sorted_row.1);
    }
}
