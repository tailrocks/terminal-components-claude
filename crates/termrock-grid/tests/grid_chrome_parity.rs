//! Frame equality between the pre-refactor app-owned grid chrome and the
//! shared grid chrome block.
//!
//! The legacy painters below are the TablePro grid chrome as it stood before
//! the header, the row-number gutter and the column placement moved into
//! `termrock-grid`. They are kept here as the oracle: the shared block must
//! resolve the same chrome and paint the same cells, styles and modifiers as
//! the app-owned path it replaced.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_grid::{
    CellRef, GridCell, GridColumnBudget, GridColumnKind, GridColumnPlacement, GridColumnSpec,
    GridModel, SortDir, solve_grid_chrome, solve_scrolled_grid_chrome,
};
use termrock_runtime::CellUi;
use termrock_runtime::ui::Ui;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, FgStep, Modifier, Role, StateFlags, StylePatch, Theme};

const CHROME_FG: Role = Role::Custom(Color::from_u32(0x008a_8a8a));
const PATH_FG: Role = Role::Custom(Color::from_u32(0x00b3_b3b3));
const FAINT: Role = Role::Custom(Color::from_u32(0x0073_7373));
const CELL_BG: Role = Role::Custom(Color::from_u32(0x002f_5aa8));
const PAD_FADE_EDGE: Role = Role::Custom(Color::from_u32(0x008c_8c8c));
const PAD_FADE_SOFT: Role = Role::Custom(Color::from_u32(0x00cc_cccc));
const TEXT_FADE_EDGE: Role = Role::Custom(Color::from_u32(0x003f_3f3f));
const TEXT_FADE_SOFT: Role = Role::Custom(Color::from_u32(0x005c_5c5c));
const ARROW_FG: Role = Role::Custom(Color::from_u32(0x008a_8a8a));

/// The columns the legacy chrome and the shared chrome agree on.
#[derive(Clone)]
struct Col {
    name: String,
    kind: GridColumnKind,
    numeric: bool,
}

impl Col {
    fn new(name: &str, kind: GridColumnKind, numeric: bool) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            numeric,
        }
    }

    fn specs(
        &self,
        filtered: &[String],
        sorted: Option<(usize, SortDir)>,
        index: usize,
    ) -> GridColumnSpec<'_> {
        GridColumnSpec {
            name: &self.name,
            kind: self.kind,
            numeric: self.numeric,
            filtered: filtered
                .iter()
                .any(|column| column.eq_ignore_ascii_case(&self.name)),
            sorted: sorted
                .filter(|(column, _)| *column == index)
                .map(|(_, dir)| dir),
            editing: false,
        }
    }
}

/// A model with a fixed set of cells.
struct Model {
    columns: Vec<Col>,
    rows: Vec<Vec<String>>,
}

impl Model {
    fn new(columns: Vec<Col>, rows: Vec<Vec<String>>) -> Self {
        Self { columns, rows }
    }

    fn specs(
        &self,
        filtered: &[String],
        sorted: Option<(usize, SortDir)>,
        editing: Option<usize>,
    ) -> Vec<GridColumnSpec<'_>> {
        self.columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let mut spec = column.specs(filtered, sorted, index);
                spec.editing = editing == Some(index);
                spec
            })
            .collect()
    }
}

impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> termrock_core::id::ItemKey {
        termrock_core::id::ItemKey::num(row as u64 + 1)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.rows
            .get(row)
            .and_then(|cells| cells.get(col))
            .map(|text| CellRef::new(text))
    }
}

/// The legacy budgets, as the app computed them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LegacyBudget {
    kind: GridColumnKind,
    text: u16,
    arrow: bool,
}

/// The legacy placement, as the app computed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LegacyPlace {
    x: u16,
    width: u16,
    primary: bool,
}

/// The legacy chrome, as the app computed it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LegacyChrome {
    digits: u16,
    gutter: u16,
    content_width: u16,
    complete: usize,
    columns: usize,
    hidden_left: usize,
    focus: Option<usize>,
    places: Vec<LegacyPlace>,
}

/// The legacy width policy of the app-owned budget solver.
const LEGACY_WIDTHS: termrock_grid::GridColumnWidthPolicy = termrock_grid::GridColumnWidthPolicy {
    sample_rows: 24,
    primary_min_width: 38,
    primary_padding: 2,
    text_padding: 0,
    numeric_padding: 0,
    filter_mark_width: 0,
    sort_mark_width: 0,
    editor_padding: 0,
    min_width: 3,
    max_width: 40,
};

/// The legacy width policy of the app-owned scrolled budget solver.
const LEGACY_WINDOW_WIDTHS: termrock_grid::GridColumnWidthPolicy =
    termrock_grid::GridColumnWidthPolicy {
        sample_rows: 64,
        primary_min_width: 38,
        primary_padding: 2,
        text_padding: 2,
        numeric_padding: 0,
        filter_mark_width: 3,
        sort_mark_width: 2,
        editor_padding: 2,
        min_width: 3,
        max_width: 40,
    };

/// The legacy placement policy of the app-owned budget solver.
const LEGACY_PLACEMENT: termrock_grid::GridPlacementPolicy = termrock_grid::GridPlacementPolicy {
    gutter_extra: 4,
    trailing_reserve: 1,
    scrollbar_reserve: 3,
    column_gap: 2,
    overflow_min_room: 4,
    overflow_arrow_width: 2,
    overflow_reserve: 4,
};

/// The legacy placement policy of the app-owned scrolled budget solver.
const LEGACY_WINDOW_PLACEMENT: termrock_grid::GridPlacementPolicy =
    termrock_grid::GridPlacementPolicy {
        overflow_min_room: 5,
        ..LEGACY_PLACEMENT
    };

fn legacy_column_natural_width(
    column: &Col,
    model: &Model,
    col: usize,
    filtered: &[String],
    sorted: Option<(usize, SortDir)>,
) -> u16 {
    let numeric = column.numeric;
    let mut natural = termrock_text::width(&column.name);
    if column.kind == GridColumnKind::Primary {
        natural = natural.saturating_add(2).max(38);
    } else if !numeric {
        natural = natural.saturating_add(2);
    }
    let sample = model.row_count().min(64);
    for row in 0..sample {
        if let Some(cell) = model.cell(row, col) {
            natural = natural.max(termrock_text::width(cell.text));
        }
    }
    if filtered
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&column.name))
    {
        natural = natural.saturating_add(3);
        if sorted.is_some_and(|(index, _)| index == col) {
            natural = natural.saturating_add(2);
        }
    }
    natural.clamp(3, 40)
}

fn legacy_column_budget_avail(area_width: u16, gutter: u16) -> u16 {
    area_width
        .saturating_sub(1)
        .saturating_sub(gutter)
        .saturating_sub(3)
}

struct LegacyWindowFit {
    complete: usize,
    places: Vec<LegacyPlace>,
}

fn legacy_layout_window(widths: &[u16], start: usize, avail: u16) -> LegacyWindowFit {
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
            if room >= 5 {
                places.push(LegacyPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary: false,
                });
            }
            closed = true;
            continue;
        }
        places.push(LegacyPlace {
            x: used.saturating_add(gap),
            width,
            primary: false,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    LegacyWindowFit { complete, places }
}

fn legacy_focus_is_complete(widths: &[u16], start: usize, focus: usize, avail: u16) -> bool {
    if start > focus {
        return false;
    }
    let fit = legacy_layout_window(widths, start, avail);
    focus.saturating_sub(start) < fit.complete
}

fn legacy_window_start(widths: &[u16], focus: usize, avail: u16) -> usize {
    if focus == 0 || widths.is_empty() {
        return 0;
    }
    let focus = focus.min(widths.len().saturating_sub(1));
    let mut start = focus;
    while start > 0 && legacy_focus_is_complete(widths, start.saturating_sub(1), focus, avail) {
        start = start.saturating_sub(1);
    }
    start
}

/// The budgets and the chrome as the app computed them.
fn legacy_install_grid_budgets(
    area_width: u16,
    model: &Model,
) -> (Vec<LegacyBudget>, LegacyChrome) {
    let row_count = model.row_count().max(1).to_string();
    let digits = termrock_text::width(&row_count);
    let gutter = digits.saturating_add(4);
    let avail = legacy_column_budget_avail(area_width, gutter);
    let mut budgets = Vec::with_capacity(model.columns.len());
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut closed = false;
    let mut complete = 0usize;
    let sample_rows = model.row_count().min(24);
    for (index, column) in model.columns.iter().enumerate() {
        let primary = column.kind == GridColumnKind::Primary;
        let kind = column.kind;
        let mut natural = termrock_text::width(&column.name);
        if primary {
            natural = natural.saturating_add(2).max(38);
        }
        for row in 0..sample_rows {
            if let Some(cell) = model.cell(row, index) {
                natural = natural.max(termrock_text::width(cell.text));
            }
        }
        natural = natural.clamp(3, 40);
        if closed {
            budgets.push(LegacyBudget {
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
                let arrow = kind == GridColumnKind::Foreign;
                let text = room.saturating_sub(if arrow { 2 } else { 0 });
                budgets.push(LegacyBudget { kind, text, arrow });
                places.push(LegacyPlace {
                    x: used.saturating_add(gap),
                    width: room,
                    primary,
                });
            } else {
                budgets.push(LegacyBudget {
                    kind,
                    text: 0,
                    arrow: false,
                });
            }
            closed = true;
            continue;
        }
        budgets.push(LegacyBudget {
            kind,
            text: natural,
            arrow: false,
        });
        places.push(LegacyPlace {
            x: used.saturating_add(gap),
            width: natural,
            primary,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    let chrome = LegacyChrome {
        digits,
        gutter,
        content_width: avail,
        complete,
        columns: model.columns.len(),
        hidden_left: 0,
        focus: None,
        places,
    };
    (budgets, chrome)
}

/// The budgets and the chrome as the app computed them for a scrolled body.
fn legacy_install_scrolled_budgets(
    area_width: u16,
    model: &Model,
    focus: usize,
    filtered: &[String],
    sorted: Option<(usize, SortDir)>,
    editing: Option<usize>,
) -> Option<(Vec<LegacyBudget>, LegacyChrome)> {
    if focus == 0 || focus >= model.columns.len() {
        return None;
    }
    let row_count = model.row_count().max(1).to_string();
    let digits = termrock_text::width(&row_count);
    let gutter = digits.saturating_add(4);
    let avail = legacy_column_budget_avail(area_width, gutter);
    let widths: Vec<u16> = model
        .columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let natural = legacy_column_natural_width(column, model, index, filtered, sorted);
            if editing == Some(index) && !column.numeric {
                natural.saturating_add(2)
            } else {
                natural
            }
        })
        .collect();
    let start = legacy_window_start(&widths, focus, avail);
    if start == 0 {
        return None;
    }
    let fit = legacy_layout_window(&widths, start, avail);
    let mut budgets = Vec::with_capacity(model.columns.len());
    for index in 0..model.columns.len() {
        let shown = index
            .checked_sub(start)
            .and_then(|offset| fit.places.get(offset));
        budgets.push(LegacyBudget {
            kind: model.columns[index].kind,
            text: shown.map_or(0, |place| place.width),
            arrow: false,
        });
    }
    let chrome = LegacyChrome {
        digits,
        gutter,
        content_width: avail,
        complete: fit.complete,
        columns: model.columns.len(),
        hidden_left: start,
        focus: Some(focus),
        places: fit.places,
    };
    Some((budgets, chrome))
}

fn legacy_sort_mark(direction: SortDir) -> char {
    match direction {
        SortDir::Asc => '▴',
        SortDir::Desc => '▾',
    }
}

/// The column header as the app painted it.
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "the legacy header is kept as the oracle, one paint"
)]
fn legacy_paint_grid_header(
    ui: &mut Ui<'_>,
    area: Rect,
    model: &Model,
    chrome: &LegacyChrome,
    filtered: &[String],
    sorted: Option<(usize, SortDir)>,
    editing: Option<usize>,
    covered: bool,
    calm_header: bool,
) {
    if chrome.hidden_left > 0 {
        legacy_paint_scrolled_header(ui, area, model, chrome, filtered, sorted, editing);
        return;
    }
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        Rect {
            x: area.x,
            y: area.y,
            width: chrome.gutter,
            height: 1,
        },
        base,
    );
    let primary = if calm_header {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(CHROME_FG).remove(Modifier::BOLD)))
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
    let muted = base
        .remove_modifier(Modifier::BOLD)
        .patch(ui.paint_patch(&StylePatch::new().set_fg(CHROME_FG).remove(Modifier::BOLD)));
    ui.fill(
        Rect {
            x: area.x.saturating_add(chrome.gutter),
            y: area.y,
            width: chrome.content_width,
            height: 1,
        },
        base,
    );
    let key = if calm_header {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(FAINT).remove(Modifier::BOLD)))
    } else {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(FAINT).add(Modifier::BOLD)))
    };
    for (index, place) in chrome.places.iter().enumerate() {
        let origin = area.x.saturating_add(chrome.gutter).saturating_add(place.x);
        let rect = Rect {
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
            Rect {
                width: fill_width,
                ..rect
            },
            if place.primary { primary } else { muted },
        );
        if place.primary {
            ui.fill(
                Rect {
                    x: origin.saturating_add(place.width),
                    width: 2,
                    ..rect
                },
                muted,
            );
        }
        let Some(column) = model.columns.get(index) else {
            continue;
        };
        if place.primary {
            ui.paint_str(
                Rect {
                    x: origin,
                    y: area.y,
                    width: 1,
                    height: 1,
                },
                "⚷",
                key,
            );
            ui.paint_str(
                Rect {
                    x: origin.saturating_add(2),
                    y: area.y,
                    width: place.width.saturating_sub(2),
                    height: 1,
                },
                &column.name,
                primary,
            );
        } else {
            ui.paint_str(
                Rect {
                    x: origin,
                    y: area.y,
                    width: place.width,
                    height: 1,
                },
                &column.name,
                muted,
            );
        }
    }
    if chrome.columns > chrome.complete {
        let hidden = chrome.columns.saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let label_w = termrock_text::width(&label);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        let label_x = end.saturating_sub(label_w);
        if label_x > area.x {
            ui.paint_str(
                Rect {
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
            Rect {
                x: label_x,
                y: area.y,
                width: label_w,
                height: 1,
            },
            &label,
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(PATH_FG))),
        );
    }
}

/// The scrolled column header as the app painted it.
fn legacy_paint_scrolled_header(
    ui: &mut Ui<'_>,
    area: Rect,
    model: &Model,
    chrome: &LegacyChrome,
    filtered: &[String],
    sorted: Option<(usize, SortDir)>,
    editing: Option<usize>,
) {
    let base = ui.surface_style().remove_modifier(Modifier::BOLD);
    ui.fill(
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        },
        base,
    );
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(CHROME_FG)));
    let sorted_style = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    );
    let overflow = base.patch(ui.paint_patch(&StylePatch::new().set_fg(PATH_FG)));
    let origin = area.x.saturating_add(chrome.gutter);
    if chrome.hidden_left > 0 {
        let label = format!("‹{}", chrome.hidden_left);
        ui.paint_str(
            Rect {
                x: area.x.saturating_add(1),
                y: area.y,
                width: termrock_text::width(&label).min(chrome.gutter.saturating_sub(1)),
                height: 1,
            },
            &label,
            overflow,
        );
    }
    for (offset, place) in chrome.places.iter().enumerate() {
        let index = chrome.hidden_left.saturating_add(offset);
        let Some(column) = model.columns.get(index) else {
            continue;
        };
        legacy_paint_scrolled_title(
            ui,
            area,
            origin,
            place,
            column,
            index,
            offset,
            model,
            filtered,
            sorted,
            editing,
            chrome,
            base,
            muted,
            sorted_style,
        );
    }
    if chrome.hidden_left.saturating_add(chrome.complete) < chrome.columns {
        let hidden = chrome
            .columns
            .saturating_sub(chrome.hidden_left)
            .saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let slot = termrock_text::width(&label).max(3);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(4);
        ui.paint_str(
            Rect {
                x: end.saturating_sub(slot),
                y: area.y,
                width: termrock_text::width(&label),
                height: 1,
            },
            &label,
            overflow,
        );
    }
}

/// One scrolled column title as the app painted it.
#[allow(
    clippy::too_many_arguments,
    reason = "the legacy title is kept as the oracle, one paint"
)]
fn legacy_paint_scrolled_title(
    ui: &mut Ui<'_>,
    area: Rect,
    origin: u16,
    place: &LegacyPlace,
    column: &Col,
    index: usize,
    offset: usize,
    model: &Model,
    filtered: &[String],
    sorted: Option<(usize, SortDir)>,
    editing: Option<usize>,
    chrome: &LegacyChrome,
    base: termrock_theme::PaintStyle,
    muted: termrock_theme::PaintStyle,
    sorted_style: termrock_theme::PaintStyle,
) {
    let is_filtered = filtered
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&column.name));
    let is_sorted = sorted.is_some_and(|(at, _)| at == index);
    let is_editing = editing == Some(index);
    let focused = chrome.focus.is_some_and(|at| at == index);
    let style = if is_sorted || is_filtered || is_editing || focused {
        sorted_style
    } else {
        muted
    };
    let mut title = column.name.clone();
    if is_filtered {
        title.push(' ');
        title.push('∇');
    }
    if let Some((_, direction)) = sorted.filter(|(at, _)| *at == index) {
        title.push(' ');
        title.push(legacy_sort_mark(direction));
    }
    let title = termrock_text::truncate(&title, place.width);
    let previous_filtered = offset > 0
        && model
            .columns
            .get(index.saturating_sub(1))
            .is_some_and(|previous| {
                filtered
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&previous.name))
            });
    let cell = Rect {
        x: origin.saturating_add(place.x),
        y: area.y,
        width: place.width,
        height: 1,
    };
    if is_filtered {
        ui.fill(cell, base);
        ui.fill(
            Rect {
                width: termrock_text::width(&title)
                    .saturating_add(1)
                    .min(place.width),
                ..cell
            },
            sorted_style,
        );
    } else {
        if previous_filtered && place.x >= 2 {
            ui.fill(
                Rect {
                    x: cell.x.saturating_sub(2),
                    width: 2,
                    ..cell
                },
                muted,
            );
        }
        let fill_width = if is_editing {
            place.width.saturating_sub(2)
        } else {
            place.width
        };
        ui.fill(
            Rect {
                width: fill_width,
                ..cell
            },
            style,
        );
        if is_editing {
            ui.fill(
                Rect {
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

/// The row-number gutter as the app painted it.
fn legacy_paint_row_numbers(
    ui: &mut Ui<'_>,
    area: Rect,
    model: &Model,
    chrome: &LegacyChrome,
    cursor: Option<usize>,
    origin: usize,
    dirty: &[usize],
) {
    let rows = model.row_count();
    let digits = usize::from(chrome.gutter.saturating_sub(4)).max(1);
    let visible = usize::from(area.height.saturating_sub(1)).min(rows);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let model_row = origin.saturating_add(row);
        let gutter = if cursor == Some(model_row) {
            plain.add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        let fade = visible.saturating_sub(row);
        let truecolor = ui.theme_ref().capability.color == ColorLevel::TrueColor;
        let pad = if truecolor && (fade == 1 || (origin > 0 && row == 0)) {
            gutter.patch(ui.paint_patch(&StylePatch::new().set_fg(PAD_FADE_EDGE)))
        } else if truecolor && (fade == 2 || (origin > 0 && row == 1)) {
            gutter.patch(ui.paint_patch(&StylePatch::new().set_fg(PAD_FADE_SOFT)))
        } else {
            gutter
        };
        for dx in 1..3 {
            ui.paint_str(
                Rect {
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
                PATH_FG
            } else {
                FAINT
            }
        } else if origin > 0 && row == 0 {
            TEXT_FADE_EDGE
        } else if origin > 0 && row == 1 {
            TEXT_FADE_SOFT
        } else if fade == 1 {
            TEXT_FADE_EDGE
        } else if fade == 2 {
            TEXT_FADE_SOFT
        } else if cursor == Some(model_row) {
            PATH_FG
        } else {
            FAINT
        };
        let text = format!("{:>width$}", model_row.saturating_add(1), width = digits);
        ui.paint_str(
            Rect {
                x: area.x.saturating_add(3),
                y,
                width: digits as u16,
                height: 1,
            },
            &text,
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(role))),
        );
        if dirty.contains(&model_row) {
            let mut mark = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
            if cursor == Some(model_row) {
                mark = mark.add_modifier(Modifier::BOLD);
            }
            ui.paint_str(
                Rect {
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

/// The rows as the app painted them over the grid body.
fn legacy_repaint_presented_rows(
    ui: &mut Ui<'_>,
    area: Rect,
    model: &Model,
    chrome: &LegacyChrome,
    budgets: &[LegacyBudget],
    origin: usize,
    visible: usize,
) {
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let primary = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(PATH_FG)));
    let arrow_style = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(ARROW_FG)));
    let x0 = area.x.saturating_add(chrome.gutter);
    for local in 0..visible {
        let Some(row) = origin.checked_add(local) else {
            continue;
        };
        if row >= model.row_count() {
            break;
        }
        let y = area.y.saturating_add(1).saturating_add(local as u16);
        ui.fill(
            Rect {
                x: x0,
                y,
                width: chrome.content_width.saturating_add(4),
                height: 1,
            },
            plain,
        );
        for (index, place) in chrome.places.iter().enumerate() {
            let Some(text) = model.cell(row, index).map(|cell| cell.text.to_owned()) else {
                continue;
            };
            let foreign = model
                .columns
                .get(index)
                .is_some_and(|column| column.kind == GridColumnKind::Foreign);
            let numeric = model
                .columns
                .get(index)
                .is_some_and(|column| column.numeric);
            let budget_arrow = budgets.get(index).is_some_and(|budget| budget.arrow);
            let text_budget = if budget_arrow {
                place.width.saturating_sub(2)
            } else {
                place.width
            };
            let owned;
            let shown: &str = if foreign {
                owned = termrock_text::truncate_middle(&text, text_budget);
                &owned
            } else if termrock_text::width(&text) > text_budget {
                owned = termrock_text::truncate(&text, text_budget);
                &owned
            } else {
                &text
            };
            let style = if place.primary { primary } else { plain };
            let text_width = termrock_text::width(shown).min(text_budget);
            let indent = if numeric {
                text_budget.saturating_sub(text_width)
            } else {
                0
            };
            ui.paint_str(
                Rect {
                    x: x0.saturating_add(place.x).saturating_add(indent),
                    y,
                    width: text_width,
                    height: 1,
                },
                shown,
                style,
            );
            if budget_arrow {
                ui.paint_str(
                    Rect {
                        x: x0
                            .saturating_add(place.x)
                            .saturating_add(place.width)
                            .saturating_sub(1),
                        y,
                        width: 1,
                        height: 1,
                    },
                    "→",
                    arrow_style,
                );
            }
        }
    }
}

/// The cell as the app painted it.
fn legacy_paint_result_cell(cell: GridCell<'_>, ui: &mut CellUi<'_>, budget: Option<LegacyBudget>) {
    let Some(budget) = budget else {
        ui.text(cell.value.text);
        return;
    };
    if budget.text == 0 {
        return;
    }
    let shown = if budget.kind == GridColumnKind::Foreign {
        termrock_text::truncate_middle(cell.value.text, budget.text)
    } else if termrock_text::width(cell.value.text) > budget.text {
        termrock_text::truncate(cell.value.text, budget.text)
    } else {
        cell.value.text.to_owned()
    };
    ui.text(&shown);
    if cell.flags.contains(StateFlags::ACTIVE) {
        let mut patch = StylePatch::new()
            .set_bg(CELL_BG)
            .set_fg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD);
        if ui.color_level() == ColorLevel::Mono {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if budget.kind == GridColumnKind::Primary {
        let mut patch = StylePatch::new().set_fg(PATH_FG);
        if cell.flags.contains(StateFlags::FOCUSED) && ui.color_level() == ColorLevel::Mono {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if cell.flags.contains(StateFlags::FOCUSED) {
        ui.patch(&StylePatch::new().add(Modifier::BOLD));
    }
}

/// The shared policies the app hands to the shared block.
fn shared_header_policy() -> termrock_grid::GridHeaderPolicy {
    termrock_grid::GridHeaderPolicy {
        muted: CHROME_FG,
        overflow: PATH_FG,
        key: FAINT,
        primary: FgStep::Primary,
        primary_covered: FgStep::Secondary,
        key_glyph: "⚷",
        filter_mark: '∇',
        sort_asc: '▴',
        sort_desc: '▾',
    }
}

fn shared_row_number_policy() -> termrock_grid::GridRowNumberPolicy {
    termrock_grid::GridRowNumberPolicy {
        muted: FAINT,
        cursor: PATH_FG,
        pending: Role::Warning,
        pad_fade_edge: PAD_FADE_EDGE,
        pad_fade_soft: PAD_FADE_SOFT,
        text_fade_edge: TEXT_FADE_EDGE,
        text_fade_soft: TEXT_FADE_SOFT,
    }
}

fn shared_row_policy() -> termrock_grid::GridRowPolicy {
    termrock_grid::GridRowPolicy {
        primary_fg: PATH_FG,
        arrow_fg: ARROW_FG,
    }
}

fn shared_cell_policy() -> termrock_grid::GridCellPolicy {
    termrock_grid::GridCellPolicy {
        active_bg: CELL_BG,
        active_fg: FgStep::Primary,
        primary_fg: PATH_FG,
    }
}

fn shared_chrome(chrome: &LegacyChrome) -> termrock_grid::GridChrome {
    termrock_grid::GridChrome {
        digits: chrome.digits,
        gutter: chrome.gutter,
        content_width: chrome.content_width,
        columns: chrome.columns,
        complete: chrome.complete,
        hidden_left: chrome.hidden_left,
        focus: chrome.focus,
        places: chrome
            .places
            .iter()
            .map(|place| GridColumnPlacement {
                x: place.x,
                width: place.width,
                primary: place.primary,
            })
            .collect(),
        overflow_reserve: 4,
    }
}

fn shared_budgets(budgets: &[LegacyBudget]) -> Vec<GridColumnBudget> {
    budgets
        .iter()
        .map(|budget| GridColumnBudget {
            kind: budget.kind,
            text: budget.text,
            arrow: budget.arrow,
        })
        .collect()
}

fn assert_same_cells(left: &Buffer, right: &Buffer, label: &str) {
    let area = *left.area();
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let Some(l) = left.cell((x, y)) else {
                continue;
            };
            let Some(r) = right.cell((x, y)) else {
                continue;
            };
            assert_eq!(l.symbol(), r.symbol(), "{label}: symbol at {x},{y}");
            assert_eq!(l.fg, r.fg, "{label}: fg at {x},{y}");
            assert_eq!(l.bg, r.bg, "{label}: bg at {x},{y}");
            assert_eq!(l.modifier, r.modifier, "{label}: modifier at {x},{y}");
        }
    }
}

fn scene(name: &'static str, width: u16, height: u16, color: ColorLevel) -> Scene {
    Scene::new(name, Theme::junie(), color, width, height)
}

fn wide_model() -> Model {
    Model::new(
        vec![
            Col::new("id", GridColumnKind::Primary, false),
            Col::new("order_number", GridColumnKind::Other, true),
            Col::new("customer_id", GridColumnKind::Foreign, false),
            Col::new("status", GridColumnKind::Other, false),
            Col::new("total", GridColumnKind::Other, true),
        ],
        (0..40)
            .map(|row| {
                vec![
                    format!("1d9f28fc-3e09-4488-bae7-d3aeb39b{row:04}"),
                    format!("{}", 10_000 + row),
                    format!("f92c5e53-3a5e-4d55-9d9e-{row:04}"),
                    format!("status {row}"),
                    format!("{row}.50"),
                ]
            })
            .collect(),
    )
}

fn narrow_model() -> Model {
    Model::new(
        vec![
            Col::new("id", GridColumnKind::Primary, false),
            Col::new("name", GridColumnKind::Other, false),
        ],
        (0..3)
            .map(|row| vec![format!("{row}"), format!("row {row}")])
            .collect(),
    )
}

#[test]
fn the_shared_chrome_resolves_the_legacy_budgets() {
    for model in [wide_model(), narrow_model()] {
        for area_width in [20u16, 40, 80, 120] {
            let (legacy_budgets, legacy_chrome) = legacy_install_grid_budgets(area_width, &model);
            let specs = model.specs(&[], None, None);
            let shared = solve_grid_chrome(
                area_width,
                &specs,
                &model,
                &LEGACY_WIDTHS,
                &LEGACY_PLACEMENT,
            );
            assert_eq!(
                shared.budgets,
                shared_budgets(&legacy_budgets),
                "budgets at width {area_width}"
            );
            assert_eq!(
                shared.chrome,
                shared_chrome(&legacy_chrome),
                "chrome at width {area_width}"
            );
        }
    }
}

#[test]
fn the_shared_scrolled_chrome_resolves_the_legacy_budgets() {
    let model = wide_model();
    let filtered = vec!["status".to_owned()];
    for focus in 1..model.columns.len() {
        for area_width in [20u16, 40, 80, 120] {
            let legacy = legacy_install_scrolled_budgets(
                area_width,
                &model,
                focus,
                &filtered,
                Some((3, SortDir::Desc)),
                Some(3),
            );
            let specs = model.specs(&filtered, Some((3, SortDir::Desc)), Some(3));
            let shared = solve_scrolled_grid_chrome(
                area_width,
                &specs,
                &model,
                &LEGACY_WINDOW_WIDTHS,
                &LEGACY_WINDOW_PLACEMENT,
                focus,
            );
            match (legacy, shared) {
                (None, None) => {}
                (Some((legacy_budgets, legacy_chrome)), Some(shared)) => {
                    assert_eq!(
                        shared.budgets,
                        shared_budgets(&legacy_budgets),
                        "budgets at width {area_width} focus {focus}"
                    );
                    assert_eq!(
                        shared.chrome,
                        shared_chrome(&legacy_chrome),
                        "chrome at width {area_width} focus {focus}"
                    );
                }
                (legacy, shared) => {
                    panic!(
                        "scrolled chrome at width {area_width} focus {focus}: legacy {legacy:?} shared {shared:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_shared_header_paints_the_legacy_cells() {
    for model in [wide_model(), narrow_model()] {
        for area_width in [20u16, 40, 80, 120] {
            for filtered in [Vec::new(), vec!["status".to_owned()]] {
                for sorted in [None, Some((3usize, SortDir::Desc))] {
                    for editing in [None, Some(3usize)] {
                        for covered in [false, true] {
                            for calm in [false, true] {
                                let area = Rect::new(0, 0, area_width, 12);
                                let (_, legacy_chrome) =
                                    legacy_install_grid_budgets(area_width, &model);
                                let specs = model.specs(&filtered, sorted, editing);
                                let mut legacy =
                                    scene("legacy", area_width, 12, ColorLevel::TrueColor);
                                legacy.draw(|ui, _| {
                                    legacy_paint_grid_header(
                                        ui,
                                        area,
                                        &model,
                                        &legacy_chrome,
                                        &filtered,
                                        sorted,
                                        editing,
                                        covered,
                                        calm,
                                    );
                                });
                                let mut shared =
                                    scene("shared", area_width, 12, ColorLevel::TrueColor);
                                shared.draw(|ui, _| {
                                    termrock_grid::paint_grid_header(
                                        ui,
                                        area,
                                        &specs,
                                        &shared_chrome(&legacy_chrome),
                                        &shared_header_policy(),
                                        &termrock_grid::GridHeaderOptions { covered, calm },
                                    );
                                });
                                assert_same_cells(
                                    legacy.buffer(),
                                    shared.buffer(),
                                    &format!(
                                        "header width {area_width} filtered {filtered:?} sorted {sorted:?} editing {editing:?} covered {covered} calm {calm}"
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn the_shared_scrolled_header_paints_the_legacy_cells() {
    let model = wide_model();
    let filtered = vec!["status".to_owned()];
    for area_width in [20u16, 40, 80] {
        for focus in 1..model.columns.len() {
            let Some((_, legacy_chrome)) = legacy_install_scrolled_budgets(
                area_width,
                &model,
                focus,
                &filtered,
                Some((3, SortDir::Desc)),
                Some(3),
            ) else {
                continue;
            };
            let area = Rect::new(0, 0, area_width, 12);
            let specs = model.specs(&filtered, Some((3, SortDir::Desc)), Some(3));
            let mut legacy = scene("legacy", area_width, 12, ColorLevel::TrueColor);
            legacy.draw(|ui, _| {
                legacy_paint_grid_header(
                    ui,
                    area,
                    &model,
                    &legacy_chrome,
                    &filtered,
                    Some((3, SortDir::Desc)),
                    Some(3),
                    false,
                    false,
                );
            });
            let mut shared = scene("shared", area_width, 12, ColorLevel::TrueColor);
            shared.draw(|ui, _| {
                termrock_grid::paint_grid_header(
                    ui,
                    area,
                    &specs,
                    &shared_chrome(&legacy_chrome),
                    &shared_header_policy(),
                    &termrock_grid::GridHeaderOptions {
                        covered: false,
                        calm: false,
                    },
                );
            });
            assert_same_cells(
                legacy.buffer(),
                shared.buffer(),
                &format!("scrolled header width {area_width} focus {focus}"),
            );
        }
    }
}

#[test]
fn the_shared_row_numbers_paint_the_legacy_cells() {
    let model = wide_model();
    for area_width in [20u16, 40, 80] {
        for origin in [0usize, 3] {
            for cursor in [None, Some(4usize)] {
                for dirty in [Vec::new(), vec![0usize, 2, 5]] {
                    for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                        let area = Rect::new(0, 0, area_width, 12);
                        let (_, legacy_chrome) = legacy_install_grid_budgets(area_width, &model);
                        let mut legacy = scene("legacy", area_width, 12, color);
                        legacy.draw(|ui, _| {
                            legacy_paint_row_numbers(
                                ui,
                                area,
                                &model,
                                &legacy_chrome,
                                cursor,
                                origin,
                                &dirty,
                            );
                        });
                        let mut shared = scene("shared", area_width, 12, color);
                        shared.draw(|ui, _| {
                            termrock_grid::paint_grid_row_numbers(
                                ui,
                                area,
                                &shared_chrome(&legacy_chrome),
                                &termrock_grid::GridRowNumbers {
                                    rows: model.row_count(),
                                    origin,
                                    cursor,
                                    dirty: &|row: usize| dirty.contains(&row),
                                },
                                &shared_row_number_policy(),
                            );
                        });
                        assert_same_cells(
                            legacy.buffer(),
                            shared.buffer(),
                            &format!(
                                "row numbers width {area_width} origin {origin} cursor {cursor:?} dirty {dirty:?}"
                            ),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_shared_rows_paint_the_legacy_cells() {
    let model = wide_model();
    for area_width in [20u16, 40, 80] {
        for origin in [0usize, 3] {
            for visible in [1usize, 5] {
                let area = Rect::new(0, 0, area_width, 12);
                let (legacy_budgets, legacy_chrome) =
                    legacy_install_grid_budgets(area_width, &model);
                let specs = model.specs(&[], None, None);
                let mut legacy = scene("legacy", area_width, 12, ColorLevel::TrueColor);
                legacy.draw(|ui, _| {
                    legacy_repaint_presented_rows(
                        ui,
                        area,
                        &model,
                        &legacy_chrome,
                        &legacy_budgets,
                        origin,
                        visible,
                    );
                });
                let mut shared = scene("shared", area_width, 12, ColorLevel::TrueColor);
                shared.draw(|ui, _| {
                    termrock_grid::paint_grid_rows(
                        ui,
                        area,
                        &shared_chrome(&legacy_chrome),
                        &termrock_grid::GridRowPaint {
                            columns: &specs,
                            budgets: &shared_budgets(&legacy_budgets),
                            model: &model,
                            origin,
                            visible,
                        },
                        &shared_row_policy(),
                    );
                });
                assert_same_cells(
                    legacy.buffer(),
                    shared.buffer(),
                    &format!("rows width {area_width} origin {origin} visible {visible}"),
                );
            }
        }
    }
}

#[test]
fn the_shared_cell_paints_the_legacy_cells() {
    let model = wide_model();
    let (legacy_budgets, _) = legacy_install_grid_budgets(80, &model);
    for (column, budget) in legacy_budgets.iter().enumerate() {
        for flags in [
            StateFlags::empty(),
            StateFlags::ACTIVE,
            StateFlags::FOCUSED,
            StateFlags::ACTIVE.union(StateFlags::FOCUSED),
        ] {
            for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                let cell_area = Rect::new(0, 0, 40, 1);
                let cell = GridCell {
                    row_key: termrock_core::id::ItemKey::num(1),
                    column_key: termrock_grid::ColumnKey::num(column as u16 + 1),
                    row: 0,
                    column,
                    value: CellRef::new("1d9f28fc-3e09-4488-bae7-d3aeb39b0000"),
                    flags,
                    style: termrock_theme::PaintStyle::new(),
                };
                let mut legacy = scene("legacy", 40, 3, color);
                legacy.draw(|ui, _| {
                    let mut painter =
                        termrock_runtime::CellUi::new(ui.reborrow(), cell_area, cell.style);
                    legacy_paint_result_cell(cell, &mut painter, Some(*budget));
                });
                let mut shared = scene("shared", 40, 3, color);
                shared.draw(|ui, _| {
                    let mut painter =
                        termrock_runtime::CellUi::new(ui.reborrow(), cell_area, cell.style);
                    termrock_grid::paint_grid_cell(
                        cell,
                        &mut painter,
                        Some(GridColumnBudget {
                            kind: budget.kind,
                            text: budget.text,
                            arrow: budget.arrow,
                        }),
                        &shared_cell_policy(),
                    );
                });
                assert_same_cells(
                    legacy.buffer(),
                    shared.buffer(),
                    &format!("cell column {column} flags {flags:?}"),
                );
            }
        }
    }
}
