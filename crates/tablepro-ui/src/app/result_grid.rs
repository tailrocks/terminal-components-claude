//! The result grid view: column budgets, chrome, scroll, header, numbers, status.
//!
//! The view owns the per-frame grid paint state. A frame starts with
//! [`ResultGridView::begin_frame`], paints the body with
//! [`ResultGridView::draw`], then paints the scrollbar and the status row
//! from the stored snapshot. Nothing grid-related lives in thread-locals.

use std::cell::{Cell, RefCell};

use tablepro_demo::db::ColType;
use tablepro_domain::domain::ResultGrid;
use termrock::author::PaintStyle;
use termrock::{
    ActionKey, CellAction, CellUi, FgStep, FrameRead, Grid, GridCell, GridColumnFit, GridGutter,
    GridModel, GridSortIndicator, Id, ItemKey, Modifier, Part, Role, RowDecor, RowTotal, SortDir,
    StateFlags, StylePatch, Ui, middle_parts, truncate, truncate_parts, width,
};

use super::{
    ID_HEADER_PREFIX, LEGACY_CELL_BG, LEGACY_FAINT, LEGACY_SCROLL_THUMB, LEGACY_SCROLL_TRACK,
    TAG_BUTTON_BG, TAG_BUTTON_FG, TAG_CHROME_FG, TAG_PATH_FG, grouped, paint_action_button,
    paint_quiet_gutter, sort_mark,
};

#[derive(Default)]
pub(crate) struct ResultGridView {
    frame: RefCell<Frame>,
}

#[derive(Default)]
struct Frame {
    filter: Vec<String>,
    sort: Option<(usize, SortDir)>,
    edit: Option<usize>,
    chrome: Option<GridChrome>,
    scroll: Option<GridScrollSnapshot>,
}

impl ResultGridView {
    pub(crate) fn begin_frame(
        &self,
        filter: Vec<String>,
        sort: Option<(usize, SortDir)>,
        edit: Option<usize>,
    ) {
        let mut frame = self.frame.borrow_mut();
        frame.filter = filter;
        frame.sort = sort;
        frame.edit = edit;
        frame.scroll = None;
    }

    /// Paint the grid body, focus bar, row numbers, and header.
    ///
    /// The origin is whatever the grid painted, so numbers, header, and
    /// scrollbar always agree with the body. There is no overpaint pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &self,
        ui: &mut Ui<'_>,
        area: termrock::Rect,
        id: Id,
        columns: &[(String, ColType)],
        specs: &[termrock::Column<'_>],
        state: &termrock::GridState,
        model: &ResultGrid,
        covered: bool,
        calm_header: bool,
    ) {
        let frame = self.frame.borrow();
        let (mut budgets, mut chrome) = install_grid_budgets(area.width, columns, model);
        if let Some(scrolled) = install_scrolled_budgets(
            area.width,
            columns,
            model,
            state,
            &frame.filter,
            frame.sort,
            frame.edit,
        ) {
            (budgets, chrome) = scrolled;
        }
        let seen = Cell::new(None);
        let render = |cell: GridCell<'_>, ui: &mut CellUi<'_>| {
            paint_result_cell(cell, ui, &budgets, &seen);
        };
        let grid = result_grid(id, specs).cell(&render);
        grid.draw(
            ui,
            area,
            state,
            &PaintedGrid {
                inner: model,
                budgets: &budgets,
            },
        );
        paint_grid_focus_bar(ui, area, state, model);
        let origin = seen.get().unwrap_or(0);
        let cursor = state.cursor().and_then(|(key, _)| {
            (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
        });
        let focused = cursor.and_then(|row| row.checked_sub(origin));
        paint_row_numbers(ui, area, model, cursor, origin, &chrome);
        paint_grid_header(
            ui,
            area,
            columns,
            covered,
            calm_header,
            &frame.filter,
            frame.sort,
            frame.edit,
            &chrome,
        );
        drop(frame);
        let mut frame = self.frame.borrow_mut();
        frame.chrome = Some(chrome);
        frame.scroll = Some((area, model.row_count(), origin, focused));
    }

    pub(crate) fn paint_scrollbar(&self, ui: &mut Ui<'_>) {
        let frame = self.frame.borrow();
        let (Some(chrome), Some((grid_area, rows, origin, focused))) =
            (frame.chrome.as_ref(), frame.scroll)
        else {
            return;
        };
        termrock::grid::paint_grid_vertical_bar(
            ui,
            grid_area,
            &chrome.content_chrome(),
            rows,
            origin,
            focused,
            &GRID_VERTICAL_BAR,
        );
    }

    pub(crate) fn paint_horizontal(&self, ui: &mut Ui<'_>, row: termrock::Rect) {
        let frame = self.frame.borrow();
        if let Some(chrome) = frame.chrome.as_ref() {
            paint_horizontal_bar(ui, row, chrome);
        }
    }

    /// Paint the status row, pending bar, and horizontal bar for a table view.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_status(
        &self,
        ui: &mut Ui<'_>,
        viewport: &termrock::grid::GridViewport,
        columns: usize,
        model: &ResultGrid,
        enabled: usize,
        shown: usize,
        sort_prefix: Option<String>,
    ) {
        let frame = self.frame.borrow();
        let loaded = model.row_count();
        let origin = frame.scroll.map_or(0, |(_, _, origin, _)| origin);
        let last = origin.saturating_add(shown).min(loaded);
        let first = if loaded == 0 {
            1
        } else {
            origin.saturating_add(1).min(loaded)
        };
        let cols = grid_column_status(frame.chrome.as_ref(), columns);
        let total = if enabled == 0 {
            grouped(model.total())
        } else {
            format!("~{}", grouped(model.total()))
        };
        let mut status = format!(
            "rows {}–{} of {} loaded · {total} total · {cols}",
            grouped(first),
            grouped(last),
            grouped(loaded),
        );
        if enabled > 0 {
            status = format!("filtered ({enabled}) · {status}");
        }
        if let Some(prefix) = sort_prefix {
            status = format!("{prefix} · {status}");
        }
        if !viewport.status_text_fits(width(&status))
            && let Some((head, _)) = status.rsplit_once(" · ")
        {
            // The column window is the clause that drops once a sort prefix lands.
            status = head.to_owned();
        }
        if let Some(chrome) = frame.chrome.as_ref() {
            paint_horizontal_bar(ui, viewport.horizontal, chrome);
        }
        if viewport.pending != termrock::Rect::ZERO {
            paint_pending_bar(ui, viewport.pending, model);
        }
        let style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
        ui.paint_str(viewport.status_text, &status, style);
    }
}
const GRID_PART_PATCH: [(Part, StylePatch); 2] = [
    (Part::OVERFLOW, StylePatch::new().set_fg(TAG_PATH_FG)),
    (Part::ACTIONS, StylePatch::new().set_fg(TAG_CHROME_FG)),
];
/// The vertical scroll bar of a grid body.
const GRID_VERTICAL_BAR: termrock::grid::GridBarPolicy = termrock::grid::GridBarPolicy {
    track: LEGACY_SCROLL_TRACK,
    thumb: LEGACY_SCROLL_THUMB,
    track_glyph: "┆",
    thumb_glyph: "┃",
    cover_far: Role::Custom(termrock::Color::from_u32(0x008c_8c8c)),
    cover_near: Role::Custom(termrock::Color::from_u32(0x00cc_cccc)),
    cover_far_dim: true,
};
/// The horizontal scroll bar of a grid body.
const GRID_HORIZONTAL_BAR: termrock::grid::GridBarPolicy = termrock::grid::GridBarPolicy {
    track: LEGACY_SCROLL_TRACK,
    thumb: LEGACY_SCROLL_THUMB,
    track_glyph: "┄",
    thumb_glyph: "━",
    cover_far: Role::Custom(termrock::Color::from_u32(0x008c_8c8c)),
    cover_near: Role::Custom(termrock::Color::from_u32(0x00cc_cccc)),
    cover_far_dim: true,
};
/// The keyboard-focus bar of a grid body.
const GRID_FOCUS_BAR: termrock::grid::GridFocusPolicy = termrock::grid::GridFocusPolicy {
    role: Role::Accent,
    bold: true,
    glyph: "▎",
};
/// The viewport of a query result grid: the scroll bar lane sits right of
/// the grid body and the horizontal bar takes the grid body's last row.
const FOLLOW_REF: CellAction = CellAction::new(ActionKey::application("tablepro.follow-ref"));
const FOLLOW_REFS: [CellAction; 1] = [FOLLOW_REF];
#[derive(Clone, Copy)]
struct ColBudget {
    kind: u8,
    text: u16,
    arrow: bool,
}

#[derive(Clone, Copy)]
struct ColPlace {
    x: u16,
    width: u16,
    primary: bool,
}

#[derive(Clone)]
struct GridChrome {
    gutter: u16,
    content_width: u16,
    complete: usize,
    columns: usize,
    hidden_left: usize,
    focus: Option<usize>,
    places: Vec<ColPlace>,
}

impl GridChrome {
    /// The chrome the shared grid bars align to.
    fn content_chrome(&self) -> termrock::grid::GridContentChrome {
        termrock::grid::GridContentChrome {
            gutter: self.gutter,
            content_width: self.content_width,
            columns: self.columns,
            complete: self.complete,
            hidden_left: self.hidden_left,
        }
    }
}

const KIND_OTHER: u8 = 0;
const KIND_PRIMARY: u8 = 1;
const KIND_FOREIGN: u8 = 2;

type GridScrollSnapshot = (termrock::Rect, usize, usize, Option<usize>);
fn digit_count(mut n: usize) -> u16 {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

fn format_row_number<'a>(buf: &'a mut [u8; 16], num: usize, width: usize) -> &'a str {
    let mut n = num;
    let mut idx = buf.len();
    if n == 0 {
        idx -= 1;
        buf[idx] = b'0';
    } else {
        while n > 0 {
            idx -= 1;
            buf[idx] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    let digits = buf.len() - idx;
    let pad = width.saturating_sub(digits);
    let start = idx.saturating_sub(pad);
    buf[start..idx].fill(b' ');
    std::str::from_utf8(&buf[start..]).unwrap_or("")
}

fn format_hidden_label<'a>(buf: &'a mut [u8; 16], num: usize) -> &'a str {
    let mut n = num;
    let arrow = "›".as_bytes();
    let mut idx = buf.len();
    for &b in arrow.iter().rev() {
        idx -= 1;
        buf[idx] = b;
    }
    if n == 0 {
        idx -= 1;
        buf[idx] = b'0';
    } else {
        while n > 0 {
            idx -= 1;
            buf[idx] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    std::str::from_utf8(&buf[idx..]).unwrap_or("")
}

fn paint_cell_text(ui: &mut CellUi<'_>, text: &str, budget: ColBudget) {
    if budget.kind == KIND_FOREIGN {
        if width(text) <= budget.text {
            ui.text(text);
        } else {
            let [head, ellipsis, tail] = middle_parts(text, budget.text);
            if !head.is_empty() {
                ui.text(head);
            }
            if !ellipsis.is_empty() {
                ui.text(ellipsis);
            }
            if !tail.is_empty() {
                ui.text(tail);
            }
        }
    } else if width(text) <= budget.text {
        ui.text(text);
    } else {
        let (head, ellipsis) = truncate_parts(text, budget.text);
        if !head.is_empty() {
            ui.text(head);
        }
        if !ellipsis.is_empty() {
            ui.text(ellipsis);
        }
    }
}

fn install_grid_budgets(
    area_width: u16,
    columns: &[(String, ColType)],
    model: &ResultGrid,
) -> (Vec<ColBudget>, GridChrome) {
    let digits = digit_count(model.row_count().max(1));
    let gutter = digits.saturating_add(4);
    let avail = area_width
        .saturating_sub(1)
        .saturating_sub(gutter)
        .saturating_sub(3);
    let mut budgets = Vec::with_capacity(columns.len());
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut closed = false;
    let mut complete = 0usize;
    let sample_rows = model.row_count().min(24);
    for (index, (name, ty)) in columns.iter().enumerate() {
        let primary = name == "id";
        let foreign = *ty == ColType::Uuid && !primary;
        let kind = if primary {
            KIND_PRIMARY
        } else if foreign {
            KIND_FOREIGN
        } else {
            KIND_OTHER
        };
        let mut natural = width(name);
        if primary {
            natural = natural.saturating_add(2).max(38);
        }
        for row in 0..sample_rows {
            if let Some(cell) = GridModel::cell(model, row, index) {
                natural = natural.max(width(cell.text));
            }
        }
        natural = natural.clamp(3, 40);
        if closed {
            budgets.push(ColBudget {
                kind,
                text: 0,
                arrow: false,
            });
            continue;
        }
        let gap = if places.is_empty() { 0 } else { 2 };
        let needed = used.saturating_add(gap).saturating_add(natural);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            if room >= 4 {
                // A reference mark needs a cell of its own. Width 6 is the
                // whole value, the way the baseline grid skips the arrow.
                let arrow = foreign && room > 6;
                let text = room.saturating_sub(if arrow { 2 } else { 0 });
                budgets.push(ColBudget { kind, text, arrow });
                places.push(ColPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary,
                });
            } else {
                budgets.push(ColBudget {
                    kind,
                    text: 0,
                    arrow: false,
                });
            }
            closed = true;
            continue;
        }
        let arrow = foreign && natural > 6;
        budgets.push(ColBudget {
            kind,
            // The follow arrow occupies the last two cells, so the text
            // budget is what truncate_middle actually paints.
            text: natural.saturating_sub(if arrow { 2 } else { 0 }),
            arrow,
        });
        places.push(ColPlace {
            x: used.saturating_add(gap),
            width: natural,
            primary,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    let chrome = GridChrome {
        gutter,
        content_width: avail,
        complete,
        columns: columns.len(),
        hidden_left: 0,
        focus: None,
        places,
    };
    (budgets, chrome)
}
struct PaintedGrid<'a> {
    inner: &'a ResultGrid,
    budgets: &'a [ColBudget],
}

impl GridModel for PaintedGrid<'_> {
    fn row_count(&self) -> usize {
        self.inner.row_count()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        GridModel::row_key(self.inner, row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<termrock::CellRef<'_>> {
        GridModel::cell(self.inner, row, col)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        GridModel::row_decor(self.inner, row)
    }

    fn cell_decor(&self, row: usize, col: usize) -> termrock::CellDecor<'_> {
        GridModel::cell_decor(self.inner, row, col)
    }

    fn total(&self) -> RowTotal {
        GridModel::total(self.inner)
    }

    fn read_only_reason(&self) -> Option<&str> {
        GridModel::read_only_reason(self.inner)
    }

    fn actions(&self, row: usize, col: usize) -> &[CellAction] {
        let arrow = self.budgets.get(col).is_some_and(|budget| budget.arrow);
        if arrow && GridModel::cell(self.inner, row, col).is_some_and(|cell| !cell.text.is_empty())
        {
            &FOLLOW_REFS
        } else {
            &[]
        }
    }
}
fn paint_result_cell(
    cell: GridCell<'_>,
    ui: &mut CellUi<'_>,
    budgets: &[ColBudget],
    seen: &Cell<Option<usize>>,
) {
    seen.set(Some(
        seen.get().map_or(cell.row, |current| current.min(cell.row)),
    ));
    let budget = budgets.get(cell.column).copied();
    let Some(budget) = budget else {
        ui.text(cell.value.text);
        return;
    };
    if budget.text == 0 {
        return;
    }
    paint_cell_text(ui, cell.value.text, budget);
    if cell.flags.contains(StateFlags::ACTIVE) {
        let mut patch = StylePatch::new()
            .set_bg(LEGACY_CELL_BG)
            .set_fg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD);
        if ui.color_level() == termrock::ColorLevel::Mono {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if budget.kind == KIND_PRIMARY {
        let mut patch = StylePatch::new().set_fg(TAG_PATH_FG);
        if cell.flags.contains(StateFlags::FOCUSED)
            && ui.color_level() == termrock::ColorLevel::Mono
        {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if cell.flags.contains(StateFlags::FOCUSED) {
        ui.patch(&StylePatch::new().add(Modifier::BOLD));
    }
}
fn paint_grid_focus_bar(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    state: &termrock::GridState,
    model: &ResultGrid,
) {
    let Some((key, _)) = state.cursor() else {
        return;
    };
    let Some(row) = (0..model.row_count()).find(|row| GridModel::row_key(model, *row) == key)
    else {
        return;
    };
    termrock::grid::paint_grid_focus_bar(ui, area, Some(row as u16), &GRID_FOCUS_BAR);
}
fn column_is_filtered(filter: &[String], name: &str) -> bool {
    filter
        .iter()
        .any(|column| column.eq_ignore_ascii_case(name))
}

fn column_is_sorted(sort: Option<(usize, SortDir)>, index: usize) -> bool {
    sort.is_some_and(|(column, _)| column == index)
}
fn grid_column_status(chrome: Option<&GridChrome>, columns: usize) -> String {
    let Some(chrome) = chrome else {
        return format!("cols 1–2 of {}", grouped(columns));
    };
    if chrome.hidden_left == 0 {
        return format!("cols 1–2 of {}", grouped(chrome.columns));
    }
    let first = chrome.hidden_left.saturating_add(1);
    let last = chrome.hidden_left.saturating_add(chrome.complete.max(1));
    format!(
        "cols {}–{} of {}",
        grouped(first),
        grouped(last),
        grouped(chrome.columns)
    )
}
fn column_natural_width(
    name: &str,
    ty: ColType,
    model: &ResultGrid,
    col: usize,
    filter: &[String],
    sort: Option<(usize, SortDir)>,
) -> u16 {
    let numeric = matches!(ty, ColType::Int | ColType::Numeric);
    let mut natural = width(name);
    if name == "id" {
        natural = natural.saturating_add(2).max(38);
    } else if !numeric {
        natural = natural.saturating_add(2);
    }
    let sample = model.row_count().min(64);
    for row in 0..sample {
        if let Some(cell) = GridModel::cell(model, row, col) {
            natural = natural.max(width(cell.text));
        }
    }
    if column_is_filtered(filter, name) {
        // " ∇" plus the gap the header keeps after the mark.
        natural = natural.saturating_add(3);
        if column_is_sorted(sort, col) {
            // " ▴" / " ▾" follows the filter mark. Unfiltered sorts already fit in the sample.
            natural = natural.saturating_add(2);
        }
    }
    natural.clamp(3, 40)
}
fn column_budget_avail(area_width: u16, gutter: u16) -> u16 {
    area_width
        .saturating_sub(1)
        .saturating_sub(gutter)
        .saturating_sub(3)
}
struct WindowFit {
    complete: usize,
    places: Vec<ColPlace>,
}
fn layout_window(widths: &[u16], start: usize, avail: u16) -> WindowFit {
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut complete = 0usize;
    let mut closed = false;
    for width in widths.iter().copied().skip(start) {
        if closed {
            break;
        }
        let gap = if places.is_empty() { 0 } else { 2 };
        let needed = used.saturating_add(gap).saturating_add(width);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            // Four cells sit on the overflow count. The filtered sort frame leaves them blank.
            if room >= 5 {
                places.push(ColPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary: false,
                });
            }
            closed = true;
            continue;
        }
        places.push(ColPlace {
            x: used.saturating_add(gap),
            width,
            primary: false,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    WindowFit { complete, places }
}
fn focus_is_complete(widths: &[u16], start: usize, focus: usize, avail: u16) -> bool {
    if start > focus {
        return false;
    }
    let fit = layout_window(widths, start, avail);
    focus.saturating_sub(start) < fit.complete
}
fn window_start(widths: &[u16], focus: usize, avail: u16) -> usize {
    if focus == 0 || widths.is_empty() {
        return 0;
    }
    let focus = focus.min(widths.len().saturating_sub(1));
    let mut start = focus;
    while start > 0 && focus_is_complete(widths, start.saturating_sub(1), focus, avail) {
        start = start.saturating_sub(1);
    }
    start
}
fn install_scrolled_budgets(
    area_width: u16,
    columns: &[(String, ColType)],
    model: &ResultGrid,
    state: &termrock::GridState,
    filter: &[String],
    sort: Option<(usize, SortDir)>,
    edit: Option<usize>,
) -> Option<(Vec<ColBudget>, GridChrome)> {
    let (_, column) = state.cursor()?;
    let focus = usize::from(column.raw()).checked_sub(1)?;
    if focus == 0 || focus >= columns.len() {
        return None;
    }
    let gutter = digit_count(model.row_count().max(1)).saturating_add(4);
    let avail = column_budget_avail(area_width, gutter);
    let wide = edit;
    let widths: Vec<u16> = columns
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| {
            let natural = column_natural_width(name, *ty, model, index, filter, sort);
            // The inline editor keeps two cells of field chrome past the sample.
            // A committed edit keeps that width on the focused text column.
            if wide == Some(index) && !matches!(ty, ColType::Int | ColType::Numeric) {
                natural.saturating_add(2)
            } else {
                natural
            }
        })
        .collect();
    let start = window_start(&widths, focus, avail);
    if start == 0 {
        return None;
    }
    let fit = layout_window(&widths, start, avail);
    let mut budgets = Vec::with_capacity(columns.len());
    for index in 0..columns.len() {
        let shown = index
            .checked_sub(start)
            .and_then(|offset| fit.places.get(offset));
        let foreign = columns
            .get(index)
            .is_some_and(|(name, ty)| *ty == ColType::Uuid && name != "id");
        let shown_width = shown.map_or(0, |place| place.width);
        let arrow = foreign && shown_width > 6;
        budgets.push(ColBudget {
            kind: if columns.get(index).is_some_and(|(name, _)| name == "id") {
                KIND_PRIMARY
            } else if foreign {
                KIND_FOREIGN
            } else {
                KIND_OTHER
            },
            text: shown_width.saturating_sub(if arrow { 2 } else { 0 }),
            arrow,
        });
    }
    let chrome = GridChrome {
        gutter,
        content_width: avail,
        complete: fit.complete,
        columns: columns.len(),
        hidden_left: start,
        focus: Some(focus),
        places: fit.places,
    };
    Some((budgets, chrome))
}
struct ScrolledTitle<'a> {
    origin: u16,
    place: ColPlace,
    name: &'a str,
    index: usize,
    offset: usize,
    columns: &'a [(String, ColType)],
    sort_at: Option<(usize, SortDir)>,
    focused: bool,
    base: PaintStyle,
    muted: PaintStyle,
    sorted_style: PaintStyle,
}
fn paint_scrolled_title(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    column: &ScrolledTitle<'_>,
    filter: &[String],
    edit: Option<usize>,
) {
    let &ScrolledTitle {
        origin,
        place,
        name,
        index,
        offset,
        columns,
        sort_at,
        focused,
        base,
        muted,
        sorted_style,
    } = column;
    let filtered = column_is_filtered(filter, name);
    let sorted = sort_at.is_some_and(|(column, _)| column == index);
    let editing = edit.is_some_and(|column| column == index);
    // End keeps the cursor column primary even when that column is not sorted or wide.
    let style = if sorted || filtered || editing || focused {
        sorted_style
    } else {
        muted
    };
    let mut title = name.to_owned();
    if filtered {
        title.push(' ');
        title.push('∇');
    }
    if let Some((_, direction)) = sort_at.filter(|(column, _)| *column == index) {
        title.push(' ');
        title.push(sort_mark(direction));
    }
    let title = truncate(&title, place.width);
    let previous_filtered = offset > 0
        && columns
            .get(index.saturating_sub(1))
            .is_some_and(|(name, _)| column_is_filtered(filter, name));
    let cell = termrock::Rect {
        x: origin.saturating_add(place.x),
        y: area.y,
        width: place.width,
        height: 1,
    };
    if filtered {
        // The mark is accent. The gap after it stays the header surface, sort included.
        ui.fill(cell, base);
        ui.fill(
            termrock::Rect {
                width: width(&title).saturating_add(1).min(place.width),
                ..cell
            },
            sorted_style,
        );
    } else {
        if previous_filtered && place.x >= 2 {
            ui.fill(
                termrock::Rect {
                    x: cell.x.saturating_sub(2),
                    width: 2,
                    ..cell
                },
                muted,
            );
        }
        // Editor chrome is wider than the header mark. The extra cells stay the header surface.
        let fill_width = if editing {
            place.width.saturating_sub(2)
        } else {
            place.width
        };
        ui.fill(
            termrock::Rect {
                width: fill_width,
                ..cell
            },
            style,
        );
        if editing {
            // The two-cell gap after the editor header is muted, unlike other gaps.
            ui.fill(
                termrock::Rect {
                    x: cell.x.saturating_add(place.width),
                    width: 2,
                    ..cell
                },
                muted,
            );
        }
    }
    ui.paint_str(cell, &title, style);
}
fn paint_scrolled_header(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
    chrome: &GridChrome,
    filter: &[String],
    sort: Option<(usize, SortDir)>,
    edit: Option<usize>,
) {
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        base,
    );
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let sorted_style = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let overflow = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let origin = area.x.saturating_add(chrome.gutter);
    if chrome.hidden_left > 0 {
        let label = format!("‹{}", chrome.hidden_left);
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(1),
                y: area.y,
                width: width(&label).min(chrome.gutter.saturating_sub(1)),
                height: 1,
            },
            &label,
            overflow,
        );
    }
    let sort_at = sort;
    for (offset, place) in chrome.places.iter().enumerate() {
        let index = chrome.hidden_left.saturating_add(offset);
        let Some((name, _)) = columns.get(index) else {
            continue;
        };
        paint_scrolled_title(
            ui,
            area,
            &ScrolledTitle {
                origin,
                place: *place,
                name,
                index,
                offset,
                columns,
                sort_at,
                focused: chrome.focus.is_some_and(|column| column == index),
                base,
                muted,
                sorted_style,
            },
            filter,
            edit,
        );
    }
    if chrome.hidden_left.saturating_add(chrome.complete) < chrome.columns {
        let hidden = chrome
            .columns
            .saturating_sub(chrome.hidden_left)
            .saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let slot = width(&label).max(3);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        ui.paint_str(
            termrock::Rect {
                x: end.saturating_sub(slot),
                y: area.y,
                width: width(&label),
                height: 1,
            },
            &label,
            overflow,
        );
    }
}
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "header gutter, primary key, and muted titles share one pass"
)]
fn paint_grid_header(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    columns: &[(String, ColType)],
    covered: bool,
    calm_header: bool,
    filter: &[String],
    sort: Option<(usize, SortDir)>,
    edit: Option<usize>,
    chrome: &GridChrome,
) {
    if chrome.hidden_left > 0 {
        paint_scrolled_header(ui, area, columns, chrome, filter, sort, edit);
        return;
    }
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        termrock::Rect {
            x: area.x,
            y: area.y,
            width: chrome.gutter,
            height: 1,
        },
        base,
    );
    let primary = if calm_header {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(TAG_CHROME_FG)
                    .remove(Modifier::BOLD),
            ),
        )
    } else {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(if covered {
                        FgStep::Secondary
                    } else {
                        FgStep::Primary
                    }))
                    .add(Modifier::BOLD),
            ),
        )
    };
    let muted = base.remove_modifier(Modifier::BOLD).patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_CHROME_FG)
                .remove(Modifier::BOLD),
        ),
    );
    ui.fill(
        termrock::Rect {
            x: area.x.saturating_add(chrome.gutter),
            y: area.y,
            width: chrome.content_width,
            height: 1,
        },
        base,
    );
    let key = if calm_header {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(LEGACY_FAINT)
                    .remove(Modifier::BOLD),
            ),
        )
    } else {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(LEGACY_FAINT).add(Modifier::BOLD)))
    };
    for (index, place) in chrome.places.iter().enumerate() {
        let origin = area.x.saturating_add(chrome.gutter).saturating_add(place.x);
        let rect = termrock::Rect {
            x: origin,
            y: area.y,
            width: place.width,
            height: 1,
        };
        let fill_width = if place.primary {
            place.width.saturating_sub(2)
        } else {
            place.width
        };
        ui.fill(
            termrock::Rect {
                width: fill_width,
                ..rect
            },
            if place.primary { primary } else { muted },
        );
        if place.primary {
            ui.fill(
                termrock::Rect {
                    x: origin.saturating_add(place.width),
                    width: 2,
                    ..rect
                },
                muted,
            );
        }
        let Some((name, _)) = columns.get(index) else {
            continue;
        };
        if place.primary {
            ui.paint_str(
                termrock::Rect {
                    x: origin,
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                "⚷",
                key,
            );
            ui.paint_str(
                termrock::Rect {
                    x: origin.saturating_add(2),
                    y: area.y,
                    width: place.width.saturating_sub(2),
                    height: 1,
                },
                name,
                primary,
            );
        } else {
            let (head, ellipsis) = truncate_parts(name, place.width);
            let rect = termrock::Rect {
                x: origin,
                y: area.y,
                width: place.width,
                height: 1,
            };
            if ellipsis.is_empty() {
                ui.paint_str(rect, head, muted);
            } else {
                let used = ui.paint_str(rect, head, muted);
                ui.paint_str(
                    termrock::Rect {
                        x: origin.saturating_add(used),
                        width: place.width.saturating_sub(used),
                        ..rect
                    },
                    ellipsis,
                    muted,
                );
            }
        }
    }
    if chrome.columns > chrome.complete {
        let hidden = chrome.columns.saturating_sub(chrome.complete);
        let mut label_buf = [0u8; 16];
        let label = format_hidden_label(&mut label_buf, hidden);
        let label_w = width(label);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        let label_x = end.saturating_sub(label_w);
        if label_x > area.x {
            ui.paint_str(
                termrock::Rect {
                    x: label_x.saturating_sub(1),
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                " ",
                base,
            );
        }
        ui.paint_str(
            termrock::Rect {
                x: label_x,
                y: area.y,
                width: label_w,
                height: 1,
            },
            label,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG))),
        );
    }
}
pub(crate) fn wide_header_column(
    columns: &[(String, ColType)],
    state: &termrock::GridState,
    model: &ResultGrid,
) -> Option<usize> {
    if let Some(index) = state
        .edit_cell()
        .and_then(|(_, column)| usize::from(column.raw()).checked_sub(1))
    {
        return Some(index);
    }
    if model.pending_total() == 0 {
        return None;
    }
    let (_, column) = state.cursor()?;
    let index = usize::from(column.raw()).checked_sub(1)?;
    if index == 0 {
        return None;
    }
    let numeric = columns
        .get(index)
        .is_some_and(|(_, ty)| matches!(ty, ColType::Int | ColType::Numeric));
    if numeric { None } else { Some(index) }
}
fn paint_row_numbers(
    ui: &mut Ui<'_>,
    area: termrock::Rect,
    model: &ResultGrid,
    cursor: Option<usize>,
    origin: usize,
    chrome: &GridChrome,
) {
    let rows = model.row_count();
    let digits = usize::from(chrome.gutter.saturating_sub(4)).max(1);
    let visible = usize::from(area.height.saturating_sub(1)).min(rows);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let mut num_buf = [0u8; 16];
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let model_row = origin.saturating_add(row);
        let gutter = if cursor == Some(model_row) {
            plain.add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        let fade = visible.saturating_sub(row);
        let truecolor = ui.theme().capability.color == termrock::ColorLevel::TrueColor;
        // Baseline scroll edges: under four rows stay solid, under twelve
        // fade only the outer row, and twelve or more fade two rows.
        let depth = if visible >= 12 {
            2
        } else if visible >= 4 {
            1
        } else {
            0
        };
        let far = depth >= 1 && (fade == 1 || (origin > 0 && row == 0));
        let near = depth >= 2 && (fade == 2 || (origin > 0 && row == 1));
        // These two pads sit in the grid body. The edge fade already ran.
        // Repainting them in primary ink hides the bottom (and top) fade.
        let pad = if truecolor && far {
            gutter.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x008c_8c8c))),
            ))
        } else if truecolor && near {
            gutter.patch(ui.paint_patch(
                &StylePatch::new().set_fg(Role::Custom(termrock::Color::from_u32(0x00cc_cccc))),
            ))
        } else {
            gutter
        };
        for dx in 1..3 {
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(dx),
                    y,
                    width: 1,
                    height: 1,
                },
                " ",
                pad,
            );
        }
        let role = if !truecolor {
            if cursor == Some(model_row) {
                TAG_PATH_FG
            } else {
                LEGACY_FAINT
            }
        } else if far && (row == 0 || fade == 1) {
            Role::Custom(termrock::Color::from_u32(0x003f_3f3f))
        } else if near {
            Role::Custom(termrock::Color::from_u32(0x005c_5c5c))
        } else if cursor == Some(model_row) {
            TAG_PATH_FG
        } else {
            LEGACY_FAINT
        };
        let text = format_row_number(&mut num_buf, model_row.saturating_add(1), digits);
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(3),
                y,
                width: digits as u16,
                height: 1,
            },
            text,
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(role))),
        );
        if model.pending().is_dirty_row(model_row) {
            let mut mark = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
            if cursor == Some(model_row) {
                mark = mark.add_modifier(Modifier::BOLD);
            }
            ui.paint_str(
                termrock::Rect {
                    x: area.x.saturating_add(2),
                    y,
                    width: 1,
                    height: 1,
                },
                "•",
                mark,
            );
        }
    }
}
fn pending_detail(model: &ResultGrid) -> String {
    let pending = model.pending();
    let mut updates = 0usize;
    let mut inserts = 0usize;
    let mut deletes = 0usize;
    for row in 0..pending.row_count() {
        if pending.is_inserted(row) && !pending.is_deleted(row) {
            inserts = inserts.saturating_add(1);
        } else if pending.is_deleted(row) && !pending.is_inserted(row) {
            deletes = deletes.saturating_add(1);
        } else if pending.is_dirty_row(row) {
            updates = updates.saturating_add(1);
        }
    }
    let mut parts = Vec::new();
    if updates > 0 {
        parts.push(format!(
            "{updates} update{}",
            if updates == 1 { "" } else { "s" }
        ));
    }
    if inserts > 0 {
        parts.push(format!(
            "{inserts} insert{}",
            if inserts == 1 { "" } else { "s" }
        ));
    }
    if deletes > 0 {
        parts.push(format!(
            "{deletes} delete{}",
            if deletes == 1 { "" } else { "s" }
        ));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("· {}", parts.join(" · "))
    }
}
fn paint_pending_bar(ui: &mut Ui<'_>, area: termrock::Rect, model: &ResultGrid) {
    let base = ui.surface_style();
    let warning = base.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_CHROME_FG)));
    let label = format!("• {} pending", model.pending_total());
    let label_width = width(&label);
    ui.paint_str(
        termrock::Rect {
            x: area.x.saturating_add(1),
            width: label_width,
            ..area
        },
        &label,
        warning,
    );
    let detail = pending_detail(model);
    if !detail.is_empty() {
        ui.paint_str(
            termrock::Rect {
                x: area.x.saturating_add(2).saturating_add(label_width),
                width: width(&detail),
                ..area
            },
            &detail,
            muted,
        );
    }
    let save = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(TAG_BUTTON_FG)
                .set_bg(TAG_BUTTON_BG),
        ),
    );
    let subtle = base.patch(ui.paint_patch(&StylePatch::new().set_fg(TAG_PATH_FG)));
    let save_w = width("Save").saturating_add(2);
    let discard_w = width("Discard").saturating_add(2);
    let preview_w = width("Preview SQL").saturating_add(2);
    let mut x = area.right().saturating_sub(save_w);
    paint_action_button(ui, x, area.y, "Save", save, true);
    x = x.saturating_sub(1).saturating_sub(discard_w);
    paint_action_button(ui, x, area.y, "Discard", subtle, false);
    x = x.saturating_sub(1).saturating_sub(preview_w);
    paint_action_button(ui, x, area.y, "Preview SQL", subtle, false);
}
fn paint_horizontal_bar(ui: &mut Ui<'_>, row: termrock::Rect, chrome: &GridChrome) {
    termrock::grid::paint_grid_horizontal_bar(
        ui,
        row,
        &chrome.content_chrome(),
        &GRID_HORIZONTAL_BAR,
    );
}
pub(crate) fn result_grid<'a>(id: Id, columns: &'a [termrock::Column<'a>]) -> Grid<'a> {
    let grid = Grid::new(id, columns)
        .blur(termrock::BlurPolicy::Keep)
        .nav(termrock::NavUnit::Cell)
        .select_mode(termrock::SelectMode::Multi)
        .gutter(GridGutter::Detailed {
            row_numbers: true,
            min_digits: 1,
        })
        .column_fit(GridColumnFit::CompleteWithPreview { min_width: 4 })
        .header_sizing(termrock::GridHeaderSizing::Minimum {
            padding: 2,
            cap: 24,
        })
        .column_gap(2)
        .right_reserve(3)
        .sort_indicator(GridSortIndicator::ActiveOnly)
        .patch_part(&GRID_PART_PATCH)
        .slot(Part::GUTTER, &paint_quiet_gutter);
    if columns.first().is_some_and(|column| column.title == "id") {
        grid.header_prefixes(&ID_HEADER_PREFIX)
    } else {
        grid
    }
}
