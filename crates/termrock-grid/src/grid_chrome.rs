//! The grid chrome: the column header band, the row-number gutter and the
//! column placement that both of them share.
//!
//! A host that draws a grid body paints the header band and the row-number
//! gutter itself. The arithmetic that measures the columns, places them in
//! the content area and decides which of them fit belongs here, so a host
//! passes the columns, the content and a policy, and the chrome lands on the
//! same cells.

use ratatui_core::layout::Rect;

use termrock_runtime::CellUi;
use termrock_runtime::ui::Ui;
use termrock_text::{truncate, truncate_middle, width};
use termrock_theme::{ColorLevel, FgStep, Modifier, Role, StylePatch};

use crate::grid::{GridCell, GridModel, SortDir};
use crate::grid_viewport::GridContentChrome;

/// How the grid chrome treats one column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridColumnKind {
    /// An ordinary column.
    Other,
    /// The primary key column: it keeps its key mark and its own ink.
    Primary,
    /// A foreign key column: its values are shortened from the middle.
    Foreign,
}

/// What the chrome knows about one column of a grid body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridColumnSpec<'a> {
    /// The column title.
    pub name: &'a str,
    /// How the chrome treats the column.
    pub kind: GridColumnKind,
    /// The column holds numbers, so its content is not padded.
    pub numeric: bool,
    /// The column is filtered.
    pub filtered: bool,
    /// The column is sorted, and in which direction.
    pub sorted: Option<SortDir>,
    /// The column keeps the extra width of an inline editor.
    pub editing: bool,
}

/// Where one column sits in the grid body's content area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridColumnPlacement {
    /// The column's offset from the content area's left edge.
    pub x: u16,
    /// The width the column takes.
    pub width: u16,
    /// Whether the column is the primary key column.
    pub primary: bool,
}

/// How much of one column the grid body may paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridColumnBudget {
    /// How the column is treated.
    pub kind: GridColumnKind,
    /// The width the column's content may take.
    pub text: u16,
    /// Whether the column ends in a follow-reference mark.
    pub arrow: bool,
}

/// The resolved chrome of a grid body.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridChrome {
    /// How many digits the row-number gutter holds.
    pub digits: u16,
    /// The width of the row-number gutter.
    pub gutter: u16,
    /// The width of the content area the columns are laid out in.
    pub content_width: u16,
    /// How many columns the grid has.
    pub columns: usize,
    /// How many columns fit completely.
    pub complete: usize,
    /// How many columns scrolled off the left edge.
    pub hidden_left: usize,
    /// The column that is kept visible.
    pub focus: Option<usize>,
    /// Where each visible column sits.
    pub places: Vec<GridColumnPlacement>,
    /// The cells the overflow count keeps between the content and the edge.
    pub overflow_reserve: u16,
}

impl GridChrome {
    /// The chrome the shared grid bars align to.
    pub fn content_chrome(&self) -> GridContentChrome {
        GridContentChrome {
            gutter: self.gutter,
            content_width: self.content_width,
            columns: self.columns,
            complete: self.complete,
            hidden_left: self.hidden_left,
        }
    }
}

/// The resolved chrome of a grid body and the budgets its cells follow.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridChromeResolution {
    /// Where the columns sit.
    pub chrome: GridChrome,
    /// How much room each column's content may take.
    pub budgets: Vec<GridColumnBudget>,
}

/// How the natural width of a column is measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridColumnWidthPolicy {
    /// How many rows of the model are sampled for the column width.
    pub sample_rows: usize,
    /// The width a primary column never falls below.
    pub primary_min_width: u16,
    /// The cells a primary column keeps for its key mark.
    pub primary_padding: u16,
    /// The cells a text column keeps past its content.
    pub text_padding: u16,
    /// The cells a numeric column keeps past its content.
    pub numeric_padding: u16,
    /// The cells the filter mark takes in the header.
    pub filter_mark_width: u16,
    /// The cells the sort mark takes in the header.
    pub sort_mark_width: u16,
    /// The cells the inline editor keeps past the content.
    pub editor_padding: u16,
    /// The narrowest a column may become.
    pub min_width: u16,
    /// The widest a column may become.
    pub max_width: u16,
}

/// How the columns of a grid body are placed in the content area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPlacementPolicy {
    /// The cells the row-number gutter keeps past its digits.
    pub gutter_extra: u16,
    /// The cells the grid body keeps to the right of the content area.
    pub trailing_reserve: u16,
    /// The cells the scroll-bar lane keeps to the right of the content area.
    pub scrollbar_reserve: u16,
    /// The cells between two columns.
    pub column_gap: u16,
    /// The room an overflowing column needs before it is shown at all.
    pub overflow_min_room: u16,
    /// The cells the follow-reference mark takes from an overflowing column.
    pub overflow_arrow_width: u16,
    /// The cells the overflow count keeps between the content and the edge.
    pub overflow_reserve: u16,
}

/// The colours and glyphs a grid's column header is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridHeaderPolicy {
    /// The role the muted column titles are drawn in.
    pub muted: Role,
    /// The role the overflow count is drawn in.
    pub overflow: Role,
    /// The role the primary key mark is drawn in.
    pub key: Role,
    /// The step the primary column title is drawn in.
    pub primary: FgStep,
    /// The step the primary column title is drawn in under a covered layer.
    pub primary_covered: FgStep,
    /// The glyph the primary column's key mark is drawn with.
    pub key_glyph: &'static str,
    /// The glyph a filtered column is marked with.
    pub filter_mark: char,
    /// The glyph an ascending column is marked with.
    pub sort_asc: char,
    /// The glyph a descending column is marked with.
    pub sort_desc: char,
}

/// How the column header of a grid body is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridHeaderOptions {
    /// The header sits under a covered layer, so the primary column is dimmed.
    pub covered: bool,
    /// The header is drawn calm, without the key emphasis.
    pub calm: bool,
}

/// The colours a grid body's row numbers are drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridRowNumberPolicy {
    /// The role an ordinary row number is drawn in.
    pub muted: Role,
    /// The role the number of the cursor row is drawn in.
    pub cursor: Role,
    /// The role the pending-change mark is drawn in.
    pub pending: Role,
    /// The role of the gutter pads on the faded edge row.
    pub pad_fade_edge: Role,
    /// The role of the gutter pads on the faded row next to the edge.
    pub pad_fade_soft: Role,
    /// The role of a row number on a faded edge row.
    pub text_fade_edge: Role,
    /// The role of a row number on a faded row next to the edge.
    pub text_fade_soft: Role,
}

/// The colours a grid body's cells are drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCellPolicy {
    /// The background the active cell keeps.
    pub active_bg: Role,
    /// The step the active cell's text is drawn in.
    pub active_fg: FgStep,
    /// The role the primary column's text is drawn in.
    pub primary_fg: Role,
}

/// The colours a grid body's rows are drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridRowPolicy {
    /// The role the primary column's content is drawn in.
    pub primary_fg: Role,
    /// The role the follow-reference mark is drawn in.
    pub arrow_fg: Role,
}

/// Measure the natural width of one column.
///
/// The width follows the column title, the widest sampled value and the
/// marks the header adds, and it never falls below the policy's minimum.
fn natural_width<M: GridModel + ?Sized>(
    spec: &GridColumnSpec<'_>,
    index: usize,
    model: &M,
    policy: &GridColumnWidthPolicy,
) -> u16 {
    let mut natural = width(spec.name);
    if spec.kind == GridColumnKind::Primary {
        natural = natural
            .saturating_add(policy.primary_padding)
            .max(policy.primary_min_width);
    } else {
        natural = natural.saturating_add(if spec.numeric {
            policy.numeric_padding
        } else {
            policy.text_padding
        });
    }
    let sample = model.row_count().min(policy.sample_rows);
    for row in 0..sample {
        if let Some(cell) = model.cell(row, index) {
            natural = natural.max(width(cell.text));
        }
    }
    if spec.filtered {
        natural = natural.saturating_add(policy.filter_mark_width);
        if spec.sorted.is_some() {
            natural = natural.saturating_add(policy.sort_mark_width);
        }
    }
    let natural = natural.clamp(policy.min_width, policy.max_width);
    if spec.editing && !spec.numeric {
        natural.saturating_add(policy.editor_padding)
    } else {
        natural
    }
}

/// The width of the row-number gutter.
fn gutter_width<M: GridModel + ?Sized>(model: &M, placement: &GridPlacementPolicy) -> (u16, u16) {
    let digits = width(&model.row_count().max(1).to_string());
    (digits, digits.saturating_add(placement.gutter_extra))
}

/// The width of the content area the columns are laid out in.
fn content_width(area_width: u16, gutter: u16, placement: &GridPlacementPolicy) -> u16 {
    area_width
        .saturating_sub(placement.trailing_reserve)
        .saturating_sub(gutter)
        .saturating_sub(placement.scrollbar_reserve)
}

/// Solve the chrome of a grid body that shows every column from the first.
///
/// The columns are measured, then placed in the content area from left to
/// right. The column that does not fit keeps the room that is left and ends
/// in a follow-reference mark; the columns after it are closed.
pub fn solve_grid_chrome<M: GridModel + ?Sized>(
    area_width: u16,
    columns: &[GridColumnSpec<'_>],
    model: &M,
    widths: &GridColumnWidthPolicy,
    placement: &GridPlacementPolicy,
) -> GridChromeResolution {
    let (digits, gutter) = gutter_width(model, placement);
    let avail = content_width(area_width, gutter, placement);
    let mut budgets = Vec::with_capacity(columns.len());
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut closed = false;
    let mut complete = 0usize;
    for (index, spec) in columns.iter().enumerate() {
        let kind = spec.kind;
        let natural = natural_width(spec, index, model, widths);
        if closed {
            budgets.push(GridColumnBudget {
                kind,
                text: 0,
                arrow: false,
            });
            continue;
        }
        let gap = if places.is_empty() {
            0
        } else {
            placement.column_gap
        };
        let needed = used.saturating_add(gap).saturating_add(natural);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            if room >= placement.overflow_min_room {
                let arrow = kind == GridColumnKind::Foreign;
                let text = room.saturating_sub(if arrow {
                    placement.overflow_arrow_width
                } else {
                    0
                });
                budgets.push(GridColumnBudget { kind, text, arrow });
                places.push(GridColumnPlacement {
                    x: used.saturating_add(gap),
                    width: room,
                    primary: kind == GridColumnKind::Primary,
                });
            } else {
                budgets.push(GridColumnBudget {
                    kind,
                    text: 0,
                    arrow: false,
                });
            }
            closed = true;
            continue;
        }
        budgets.push(GridColumnBudget {
            kind,
            text: natural,
            arrow: false,
        });
        places.push(GridColumnPlacement {
            x: used.saturating_add(gap),
            width: natural,
            primary: kind == GridColumnKind::Primary,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    GridChromeResolution {
        chrome: GridChrome {
            digits,
            gutter,
            content_width: avail,
            columns: columns.len(),
            complete,
            hidden_left: 0,
            focus: None,
            places,
            overflow_reserve: placement.overflow_reserve,
        },
        budgets,
    }
}

/// The columns that fit in the content area from `start`.
struct WindowFit {
    complete: usize,
    places: Vec<GridColumnPlacement>,
}

/// Place the columns from `start` in the content area.
fn layout_window(
    widths: &[u16],
    start: usize,
    avail: u16,
    placement: &GridPlacementPolicy,
) -> WindowFit {
    let mut places = Vec::new();
    let mut used = 0u16;
    let mut complete = 0usize;
    let mut closed = false;
    for width in widths.iter().copied().skip(start) {
        if closed {
            break;
        }
        let gap = if places.is_empty() {
            0
        } else {
            placement.column_gap
        };
        let needed = used.saturating_add(gap).saturating_add(width);
        if !places.is_empty() && needed > avail {
            let room = avail.saturating_sub(used).saturating_sub(gap);
            // Four cells sit on the overflow count. The filtered sort frame
            // leaves them blank.
            if room >= placement.overflow_min_room {
                places.push(GridColumnPlacement {
                    x: used.saturating_add(gap),
                    width: room,
                    primary: false,
                });
            }
            closed = true;
            continue;
        }
        places.push(GridColumnPlacement {
            x: used.saturating_add(gap),
            width,
            primary: false,
        });
        used = needed;
        complete = complete.saturating_add(1);
    }
    WindowFit { complete, places }
}

/// Whether the focused column fits completely when the window starts at
/// `start`.
fn focus_is_complete(
    widths: &[u16],
    start: usize,
    focus: usize,
    avail: u16,
    placement: &GridPlacementPolicy,
) -> bool {
    if start > focus {
        return false;
    }
    let fit = layout_window(widths, start, avail, placement);
    focus.saturating_sub(start) < fit.complete
}

/// The first column of the window that keeps the focused column complete.
fn window_start(
    widths: &[u16],
    focus: usize,
    avail: u16,
    placement: &GridPlacementPolicy,
) -> usize {
    if focus == 0 || widths.is_empty() {
        return 0;
    }
    let focus = focus.min(widths.len().saturating_sub(1));
    let mut start = focus;
    while start > 0 && focus_is_complete(widths, start.saturating_sub(1), focus, avail, placement) {
        start = start.saturating_sub(1);
    }
    start
}

/// Solve the chrome of a grid body that scrolls to keep `focus` visible.
///
/// Returns [`None`] when the focused column already fits, or when it cannot
/// be reached at all.
pub fn solve_scrolled_grid_chrome<M: GridModel + ?Sized>(
    area_width: u16,
    columns: &[GridColumnSpec<'_>],
    model: &M,
    widths: &GridColumnWidthPolicy,
    placement: &GridPlacementPolicy,
    focus: usize,
) -> Option<GridChromeResolution> {
    if focus == 0 || focus >= columns.len() {
        return None;
    }
    let (digits, gutter) = gutter_width(model, placement);
    let avail = content_width(area_width, gutter, placement);
    let natural: Vec<u16> = columns
        .iter()
        .enumerate()
        .map(|(index, spec)| natural_width(spec, index, model, widths))
        .collect();
    let start = window_start(&natural, focus, avail, placement);
    if start == 0 {
        return None;
    }
    let fit = layout_window(&natural, start, avail, placement);
    let budgets = columns
        .iter()
        .enumerate()
        .map(|(index, spec)| GridColumnBudget {
            kind: spec.kind,
            text: index
                .checked_sub(start)
                .and_then(|offset| fit.places.get(offset))
                .map_or(0, |place| place.width),
            arrow: false,
        })
        .collect();
    Some(GridChromeResolution {
        chrome: GridChrome {
            digits,
            gutter,
            content_width: avail,
            columns: columns.len(),
            complete: fit.complete,
            hidden_left: start,
            focus: Some(focus),
            places: fit.places,
            overflow_reserve: placement.overflow_reserve,
        },
        budgets,
    })
}

/// Paint the column header band of a grid body.
///
/// `area` is the header row. The band is drawn from the chrome the body
/// resolved, so the header and the columns of the body always agree.
pub fn paint_grid_header(
    ui: &mut Ui<'_>,
    area: Rect,
    columns: &[GridColumnSpec<'_>],
    chrome: &GridChrome,
    policy: &GridHeaderPolicy,
    options: &GridHeaderOptions,
) {
    if chrome.hidden_left > 0 {
        paint_scrolled_header(ui, area, columns, chrome, policy);
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
    let primary = if options.calm {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(policy.muted)
                    .remove(Modifier::BOLD),
            ),
        )
    } else {
        base.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(if options.covered {
                        policy.primary_covered
                    } else {
                        policy.primary
                    }))
                    .add(Modifier::BOLD),
            ),
        )
    };
    let muted = base.remove_modifier(Modifier::BOLD).patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(policy.muted)
                .remove(Modifier::BOLD),
        ),
    );
    ui.fill(
        Rect {
            x: area.x.saturating_add(chrome.gutter),
            y: area.y,
            width: chrome.content_width,
            height: 1,
        },
        base,
    );
    let key = if options.calm {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.key).remove(Modifier::BOLD)))
    } else {
        base.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.key).add(Modifier::BOLD)))
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
        let Some(spec) = columns.get(index) else {
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
                policy.key_glyph,
                key,
            );
            ui.paint_str(
                Rect {
                    x: origin.saturating_add(2),
                    y: area.y,
                    width: place.width.saturating_sub(2),
                    height: 1,
                },
                spec.name,
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
                spec.name,
                muted,
            );
        }
    }
    if chrome.columns > chrome.complete {
        let hidden = chrome.columns.saturating_sub(chrome.complete);
        let label = format!("{hidden}›");
        let label_w = width(&label);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(chrome.overflow_reserve);
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
            base.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.overflow))),
        );
    }
}

/// Paint the column header band of a grid body that scrolled to the left.
fn paint_scrolled_header(
    ui: &mut Ui<'_>,
    area: Rect,
    columns: &[GridColumnSpec<'_>],
    chrome: &GridChrome,
    policy: &GridHeaderPolicy,
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
    let muted = base.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.muted)));
    let sorted_style = base.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(policy.primary))
                .add(Modifier::BOLD),
        ),
    );
    let overflow = base.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.overflow)));
    let origin = area.x.saturating_add(chrome.gutter);
    if chrome.hidden_left > 0 {
        let label = format!("‹{}", chrome.hidden_left);
        ui.paint_str(
            Rect {
                x: area.x.saturating_add(1),
                y: area.y,
                width: width(&label).min(chrome.gutter.saturating_sub(1)),
                height: 1,
            },
            &label,
            overflow,
        );
    }
    for (offset, place) in chrome.places.iter().enumerate() {
        let index = chrome.hidden_left.saturating_add(offset);
        let Some(spec) = columns.get(index) else {
            continue;
        };
        paint_scrolled_title(
            ui,
            area,
            origin,
            place,
            spec,
            offset,
            index,
            columns,
            chrome,
            policy,
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
        let slot = width(&label).max(3);
        let end = area
            .x
            .saturating_add(chrome.gutter)
            .saturating_add(chrome.content_width)
            .saturating_add(chrome.overflow_reserve);
        ui.paint_str(
            Rect {
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

/// Paint one column title of a grid body that scrolled to the left.
#[allow(
    clippy::too_many_arguments,
    reason = "the title, its marks and the styles it lands on are one paint"
)]
fn paint_scrolled_title(
    ui: &mut Ui<'_>,
    area: Rect,
    origin: u16,
    place: &GridColumnPlacement,
    spec: &GridColumnSpec<'_>,
    offset: usize,
    index: usize,
    columns: &[GridColumnSpec<'_>],
    chrome: &GridChrome,
    policy: &GridHeaderPolicy,
    base: termrock_theme::PaintStyle,
    muted: termrock_theme::PaintStyle,
    sorted_style: termrock_theme::PaintStyle,
) {
    let sorted = spec.sorted.is_some();
    let editing = spec.editing;
    let focused = chrome.focus.is_some_and(|column| column == index);
    // End keeps the cursor column primary even when that column is not sorted
    // or wide.
    let style = if sorted || spec.filtered || editing || focused {
        sorted_style
    } else {
        muted
    };
    let mut title = spec.name.to_owned();
    if spec.filtered {
        title.push(' ');
        title.push(policy.filter_mark);
    }
    if let Some(direction) = spec.sorted {
        title.push(' ');
        title.push(match direction {
            SortDir::Asc => policy.sort_asc,
            SortDir::Desc => policy.sort_desc,
        });
    }
    let title = truncate(&title, place.width);
    let previous_filtered = offset > 0
        && columns
            .get(index.saturating_sub(1))
            .is_some_and(|previous| previous.filtered);
    let cell = Rect {
        x: origin.saturating_add(place.x),
        y: area.y,
        width: place.width,
        height: 1,
    };
    if spec.filtered {
        // The mark is accent. The gap after it stays the header surface, sort
        // included.
        ui.fill(cell, base);
        ui.fill(
            Rect {
                width: width(&title).saturating_add(1).min(place.width),
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
        // Editor chrome is wider than the header mark. The extra cells stay
        // the header surface.
        let fill_width = if editing {
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
        if editing {
            // The two-cell gap after the editor header is muted, unlike other
            // gaps.
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

/// The rows a grid body shows in its row-number gutter.
pub struct GridRowNumbers<'a> {
    /// How many rows the model has.
    pub rows: usize,
    /// The first row of the model the body shows.
    pub origin: usize,
    /// The row the cursor sits on.
    pub cursor: Option<usize>,
    /// Whether a row carries a pending change.
    pub dirty: &'a dyn Fn(usize) -> bool,
}

/// Paint the row-number gutter of a grid body.
///
/// `area` is the grid body area. The gutter starts under the header row, and
/// `origin` is the first row of the model the body shows.
pub fn paint_grid_row_numbers(
    ui: &mut Ui<'_>,
    area: Rect,
    chrome: &GridChrome,
    numbers: &GridRowNumbers<'_>,
    policy: &GridRowNumberPolicy,
) {
    let rows = numbers.rows;
    let origin = numbers.origin;
    let cursor = numbers.cursor;
    let dirty = numbers.dirty;
    let digits = usize::from(chrome.digits).max(1);
    let visible = usize::from(area.height.saturating_sub(1)).min(rows);
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let truecolor = ui.theme_ref().capability.color == ColorLevel::TrueColor;
    for row in 0..visible {
        let y = area.y.saturating_add(1).saturating_add(row as u16);
        let model_row = origin.saturating_add(row);
        let gutter = if cursor == Some(model_row) {
            plain.add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        let fade = visible.saturating_sub(row);
        // A two-row body is the whole viewport. Keep it solid, as the baseline does.
        let edge = visible > 2;
        // These two pads sit in the grid body. The edge fade already ran.
        // Repainting them in primary ink hides the bottom (and top) fade.
        let pad = if truecolor && edge && (fade == 1 || (origin > 0 && row == 0)) {
            gutter.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.pad_fade_edge)))
        } else if truecolor && edge && (fade == 2 || (origin > 0 && row == 1)) {
            gutter.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.pad_fade_soft)))
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
                policy.cursor
            } else {
                policy.muted
            }
        } else if edge && origin > 0 && row == 0 {
            policy.text_fade_edge
        } else if edge && origin > 0 && row == 1 {
            policy.text_fade_soft
        } else if edge && fade == 1 {
            policy.text_fade_edge
        } else if edge && fade == 2 {
            policy.text_fade_soft
        } else if cursor == Some(model_row) {
            policy.cursor
        } else {
            policy.muted
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
        if dirty(model_row) {
            let mut mark = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.pending)));
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

/// Paint one grid cell with the budget the chrome resolved for its column.
///
/// A column without a budget paints its value as it is; a column whose
/// budget is empty stays blank.
pub fn paint_grid_cell(
    cell: GridCell<'_>,
    ui: &mut CellUi<'_>,
    budget: Option<GridColumnBudget>,
    policy: &GridCellPolicy,
) {
    let Some(budget) = budget else {
        ui.text(cell.value.text);
        return;
    };
    if budget.text == 0 {
        return;
    }
    let shown = if budget.kind == GridColumnKind::Foreign {
        truncate_middle(cell.value.text, budget.text)
    } else if width(cell.value.text) > budget.text {
        truncate(cell.value.text, budget.text)
    } else {
        cell.value.text.to_owned()
    };
    ui.text(&shown);
    if cell
        .flags
        .contains(termrock_core::response::StateFlags::ACTIVE)
    {
        let mut patch = StylePatch::new()
            .set_bg(policy.active_bg)
            .set_fg(Role::Fg(policy.active_fg))
            .add(Modifier::BOLD);
        if ui.color_level() == ColorLevel::Mono {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if budget.kind == GridColumnKind::Primary {
        let mut patch = StylePatch::new().set_fg(policy.primary_fg);
        if cell
            .flags
            .contains(termrock_core::response::StateFlags::FOCUSED)
            && ui.color_level() == ColorLevel::Mono
        {
            patch = patch.add(Modifier::REVERSED);
        }
        ui.patch(&patch);
    } else if cell
        .flags
        .contains(termrock_core::response::StateFlags::FOCUSED)
    {
        ui.patch(&StylePatch::new().add(Modifier::BOLD));
    }
}

/// The rows a grid body presents over the rows the grid widget drew.
pub struct GridRowPaint<'a, M: GridModel + ?Sized> {
    /// The columns of the body.
    pub columns: &'a [GridColumnSpec<'a>],
    /// The budgets the chrome resolved for the columns.
    pub budgets: &'a [GridColumnBudget],
    /// The model the body shows.
    pub model: &'a M,
    /// The first row of the model the body shows.
    pub origin: usize,
    /// How many rows the body presents.
    pub visible: usize,
}

/// Paint the rows of a grid body over the rows the grid widget drew.
///
/// `origin` is the first row of the model the body shows, and `visible` is
/// how many rows the body presents. The rows follow the chrome the body
/// resolved, so they land under the same columns as the header.
pub fn paint_grid_rows<M: GridModel + ?Sized>(
    ui: &mut Ui<'_>,
    area: Rect,
    chrome: &GridChrome,
    rows: &GridRowPaint<'_, M>,
    policy: &GridRowPolicy,
) {
    let columns = rows.columns;
    let budgets = rows.budgets;
    let model = rows.model;
    let origin = rows.origin;
    let visible = rows.visible;
    let plain = ui.surface_style().remove_modifier(Modifier::BOLD);
    let primary = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.primary_fg)));
    let arrow_style = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.arrow_fg)));
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
            let kind = columns
                .get(index)
                .map_or(GridColumnKind::Other, |spec| spec.kind);
            let numeric = columns.get(index).is_some_and(|spec| spec.numeric);
            let budget_arrow = budgets.get(index).is_some_and(|budget| budget.arrow);
            let text_budget = if budget_arrow {
                place.width.saturating_sub(2)
            } else {
                place.width
            };
            let owned;
            let shown: &str = if kind == GridColumnKind::Foreign {
                owned = truncate_middle(&text, text_budget);
                &owned
            } else if width(&text) > text_budget {
                owned = truncate(&text, text_budget);
                &owned
            } else {
                &text
            };
            let style = if place.primary { primary } else { plain };
            let text_width = width(shown).min(text_budget);
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
