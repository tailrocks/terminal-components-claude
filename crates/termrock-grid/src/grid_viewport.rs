//! The grid viewport: the region a grid body draws into and the chrome that
//! surrounds it.
//!
//! A host that owns a grid paints its own bands above the grid body (a mode
//! header, a filter bar) and its own rows below it (a pending-change bar, a
//! status line). The arithmetic that carves those regions out of the host
//! area belongs here, so a host passes a policy and the grid body, the bands
//! and the bars all land on the same cells.

use ratatui_core::layout::Rect;

use termrock_runtime::ui::Ui;
use termrock_theme::{ColorLevel, Modifier, StylePatch};

/// How a grid viewport is carved out of its host area.
///
/// Every band is optional. A host that draws no status line leaves
/// [`status_rows`](Self::status_rows) at zero; a host that draws no pending
/// bar leaves [`pending_rows`](Self::pending_rows) at zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridViewportPolicy {
    /// Rows above the grid body that the host paints itself.
    pub header_rows: u16,
    /// Rows reserved for the filter band, between the header band and the
    /// grid body. The filter bar sits on the band's first row.
    pub filter_rows: u16,
    /// Rows reserved below the grid body for a pending-change bar. The bar
    /// sits on the row above the status row; the rows between the bar and
    /// the horizontal bar stay blank.
    pub pending_rows: u16,
    /// Rows reserved below the pending bar for a status line.
    pub status_rows: u16,
    /// Columns reserved on the left of the status row for the host's own
    /// marker. The status text starts after them.
    pub status_inset: u16,
    /// Columns reserved to the right of the grid body.
    pub right_reserve: u16,
}

/// The resolved regions of a grid viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridViewport {
    /// The band above the filter band, painted by the host.
    pub header: Rect,
    /// The filter band.
    pub filter: Rect,
    /// The filter bar row, the filter band's first row.
    pub filter_bar: Rect,
    /// Where the grid body draws.
    pub grid: Rect,
    /// The horizontal bar row, directly below the grid body.
    pub horizontal: Rect,
    /// The pending-change bar row. [`Rect::ZERO`] when no rows are reserved.
    pub pending: Rect,
    /// The status line row. [`Rect::ZERO`] when no rows are reserved.
    pub status: Rect,
    /// The status text area: the status row without the host's own marker
    /// columns. [`Rect::ZERO`] when no rows are reserved.
    pub status_text: Rect,
}

/// Carve a grid viewport out of `area`.
///
/// The grid body takes the rows that are left once the header band, the
/// filter band, the pending rows, the status rows and the horizontal bar row
/// are taken out. The horizontal bar row always sits directly below the grid
/// body, and the pending bar row always sits directly above the status row.
pub fn solve_grid_viewport(area: Rect, policy: &GridViewportPolicy) -> GridViewport {
    let header_rows = policy.header_rows;
    let filter_rows = policy.filter_rows;
    let reserved = header_rows
        .saturating_add(filter_rows)
        .saturating_add(policy.pending_rows)
        .saturating_add(policy.status_rows)
        .saturating_add(1);
    let grid = Rect {
        x: area.x,
        y: area
            .y
            .saturating_add(header_rows)
            .saturating_add(filter_rows),
        width: area.width.saturating_sub(policy.right_reserve),
        height: area.height.saturating_sub(reserved),
    };
    let horizontal = Rect {
        x: grid.x,
        y: area
            .bottom()
            .saturating_sub(policy.pending_rows)
            .saturating_sub(policy.status_rows)
            .saturating_sub(1),
        width: grid.width,
        height: 1,
    };
    let pending = if policy.pending_rows == 0 {
        Rect::ZERO
    } else {
        Rect {
            x: area.x,
            y: area
                .bottom()
                .saturating_sub(1)
                .saturating_sub(policy.status_rows),
            width: area.width,
            height: 1,
        }
    };
    let status = if policy.status_rows == 0 {
        Rect::ZERO
    } else {
        Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        }
    };
    let status_text = if policy.status_rows == 0 {
        Rect::ZERO
    } else {
        Rect {
            x: status.x.saturating_add(policy.status_inset),
            width: status.width.saturating_sub(policy.status_inset),
            ..status
        }
    };
    let filter = Rect {
        x: area.x,
        y: area.y.saturating_add(header_rows),
        width: area.width,
        height: filter_rows,
    };
    let filter_bar = if policy.filter_rows == 0 {
        Rect::ZERO
    } else {
        Rect {
            height: 1,
            ..filter
        }
    };
    GridViewport {
        header: Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: header_rows,
        },
        filter,
        filter_bar,
        grid,
        horizontal,
        pending,
        status,
        status_text,
    }
}

/// The column chrome a grid body resolved.
///
/// The scroll bars align to the grid body's gutter and content width, so the
/// host hands the resolved chrome to the bars instead of recomputing the
/// geometry from the column widths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridContentChrome {
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
}

/// The colours and glyphs a grid's scroll bars are drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridBarPolicy {
    /// The role the track is drawn in.
    pub track: termrock_theme::Role,
    /// The role the thumb is drawn in.
    pub thumb: termrock_theme::Role,
    /// The glyph the track is drawn with.
    pub track_glyph: &'static str,
    /// The glyph the thumb is drawn with.
    pub thumb_glyph: &'static str,
    /// The role the cell before the track is drawn in at the far end of the
    /// track, where colour can carry the state.
    pub cover_far: termrock_theme::Role,
    /// The role the cell before the track is drawn in one cell in from either
    /// end of the track, where colour can carry the state.
    pub cover_near: termrock_theme::Role,
    /// Whether the far cover cell dims where colour cannot carry the state.
    pub cover_far_dim: bool,
}

/// The colours and glyphs a grid's focus bar is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridFocusPolicy {
    /// The role the focus bar is drawn in.
    pub role: termrock_theme::Role,
    /// Whether the focus bar is drawn bold.
    pub bold: bool,
    /// The glyph the focus bar is drawn with.
    pub glyph: &'static str,
}

/// Paint the vertical scroll bar of a grid body.
///
/// `area` is the grid body area. The track runs down the column just right
/// of the grid body's content, starting under the body's first row, and the
/// thumb covers the presented slice of the rows.
pub fn paint_grid_vertical_bar(
    ui: &mut Ui<'_>,
    area: Rect,
    chrome: &GridContentChrome,
    rows: usize,
    origin: usize,
    policy: &GridBarPolicy,
) {
    if area.height < 2 || area.width == 0 {
        return;
    }
    let track_len = usize::from(area.height.saturating_sub(1));
    let thumb_len = track_len
        .saturating_mul(track_len)
        .checked_div(rows.max(1))
        .unwrap_or(1)
        .max(1)
        .min(track_len);
    let max_off = rows.saturating_sub(track_len);
    let usable = track_len.saturating_sub(thumb_len);
    let thumb_at = if max_off == 0 {
        0
    } else {
        origin
            .saturating_mul(usable)
            .saturating_add(max_off / 2)
            .checked_div(max_off)
            .unwrap_or(0)
            .min(usable)
    };
    if chrome.columns == 0 {
        return;
    }
    // Stay on the content edge. A dropped preview must not pull the thumb
    // left.
    let x = area
        .x
        .saturating_add(chrome.gutter)
        .saturating_add(chrome.content_width)
        .saturating_add(4);
    let covered = x.saturating_sub(1);
    let plain = ui.surface_style();
    let track = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.track)));
    let thumb = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.thumb)));
    let truecolor = ui.theme_ref().capability.color == ColorLevel::TrueColor;
    for offset in 0..track_len {
        let y = area.y.saturating_add(1).saturating_add(offset as u16);
        let from_end = track_len.saturating_sub(offset).saturating_sub(1);
        let from_start = origin > 0 && (offset == 0 || offset == 1);
        let cover = if !truecolor {
            if from_end == 0 || (from_start && offset == 0) {
                if policy.cover_far_dim {
                    plain.add_modifier(Modifier::DIM)
                } else {
                    plain
                }
            } else {
                plain
            }
        } else if from_end == 0 || (from_start && offset == 0) {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.cover_far)))
        } else if from_end == 1 || (from_start && offset == 1) {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.cover_near)))
        } else {
            plain
        };
        ui.paint_str(
            Rect {
                x: covered,
                y,
                width: 1,
                height: 1,
            },
            " ",
            cover,
        );
        let on_thumb = offset >= thumb_at && offset < thumb_at.saturating_add(thumb_len);
        let glyph = if on_thumb {
            policy.thumb_glyph
        } else {
            policy.track_glyph
        };
        let style = if on_thumb { thumb } else { track };
        ui.paint_str(
            Rect {
                x,
                y,
                width: 1,
                height: 1,
            },
            glyph,
            style,
        );
    }
}

/// Paint the horizontal scroll bar of a grid body.
///
/// `row` is the bar's row. The track starts after the row-number gutter and
/// the thumb covers the presented slice of the columns.
pub fn paint_grid_horizontal_bar(
    ui: &mut Ui<'_>,
    row: Rect,
    chrome: &GridContentChrome,
    policy: &GridBarPolicy,
) {
    if chrome.content_width == 0 || chrome.columns == 0 {
        return;
    }
    let x = row.x.saturating_add(chrome.gutter);
    let track_w = chrome
        .content_width
        .saturating_add(4)
        .min(row.right().saturating_sub(x));
    if track_w == 0 {
        return;
    }
    let columns = chrome.columns.max(1);
    let thumb = usize::from(track_w)
        .saturating_mul(chrome.complete.max(1))
        .saturating_div(columns)
        .max(1)
        .min(usize::from(track_w));
    let thumb_at = if chrome.hidden_left == 0 {
        0
    } else {
        usize::from(track_w)
            .saturating_mul(chrome.hidden_left)
            .saturating_add(columns / 2)
            .saturating_div(columns)
    }
    .min(usize::from(track_w).saturating_sub(thumb));
    let track = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(policy.track)));
    let thumb_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(policy.thumb)));
    ui.paint_str(
        Rect {
            x,
            y: row.y,
            width: track_w,
            height: 1,
        },
        &policy.track_glyph.repeat(usize::from(track_w)),
        track,
    );
    ui.paint_str(
        Rect {
            x: x.saturating_add(thumb_at as u16),
            y: row.y,
            width: thumb as u16,
            height: 1,
        },
        &policy.thumb_glyph.repeat(thumb),
        thumb_style,
    );
}

/// Paint the keyboard-focus bar of a grid body.
///
/// `row` is the row the focus sits on, counted from the grid body's first
/// row. The bar is the grid body's own focus indicator, so it is the only
/// focus mark the body draws.
pub fn paint_grid_focus_bar(
    ui: &mut Ui<'_>,
    area: Rect,
    row: Option<u16>,
    policy: &GridFocusPolicy,
) {
    let Some(row) = row else {
        return;
    };
    let y = area.y.saturating_add(1).saturating_add(row);
    if y >= area.bottom() {
        return;
    }
    let mut patch = StylePatch::new().set_fg(policy.role);
    if policy.bold {
        patch = patch.add(Modifier::BOLD);
    }
    let style = ui.surface_style().patch(ui.paint_patch(&patch));
    ui.paint_str(
        Rect {
            x: area.x,
            y,
            width: 1,
            height: 1,
        },
        policy.glyph,
        style,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: u16, height: u16) -> Rect {
        Rect::new(0, 0, width, height)
    }

    #[test]
    fn a_query_result_viewport_reserves_the_scrollbar_lane_and_one_bar_row() {
        let viewport = solve_grid_viewport(
            area(40, 10),
            &GridViewportPolicy {
                right_reserve: 3,
                ..GridViewportPolicy::default()
            },
        );
        assert_eq!(viewport.grid, Rect::new(0, 0, 37, 9));
        assert_eq!(viewport.horizontal, Rect::new(0, 9, 37, 1));
        assert_eq!(viewport.pending, Rect::ZERO);
        assert_eq!(viewport.status, Rect::ZERO);
    }

    #[test]
    fn a_table_viewport_carves_the_filter_band_and_the_status_row() {
        let viewport = solve_grid_viewport(
            area(40, 20),
            &GridViewportPolicy {
                header_rows: 3,
                filter_rows: 2,
                status_rows: 1,
                status_inset: 1,
                ..GridViewportPolicy::default()
            },
        );
        assert_eq!(viewport.header, Rect::new(0, 0, 40, 3));
        assert_eq!(viewport.filter, Rect::new(0, 3, 40, 2));
        assert_eq!(viewport.grid, Rect::new(0, 5, 40, 13));
        assert_eq!(viewport.horizontal, Rect::new(0, 18, 40, 1));
        assert_eq!(viewport.pending, Rect::ZERO);
        assert_eq!(viewport.status, Rect::new(0, 19, 40, 1));
    }

    #[test]
    fn a_pending_viewport_leaves_one_blank_row_above_the_pending_bar() {
        let viewport = solve_grid_viewport(
            area(40, 20),
            &GridViewportPolicy {
                header_rows: 3,
                filter_rows: 2,
                pending_rows: 2,
                status_rows: 1,
                status_inset: 1,
                ..GridViewportPolicy::default()
            },
        );
        assert_eq!(viewport.grid, Rect::new(0, 5, 40, 11));
        assert_eq!(viewport.horizontal, Rect::new(0, 16, 40, 1));
        assert_eq!(viewport.pending, Rect::new(0, 18, 40, 1));
        assert_eq!(viewport.status, Rect::new(0, 19, 40, 1));
    }

    #[test]
    fn a_short_area_saturates_instead_of_wrapping() {
        let viewport = solve_grid_viewport(
            area(2, 1),
            &GridViewportPolicy {
                header_rows: 3,
                filter_rows: 2,
                pending_rows: 2,
                status_rows: 1,
                status_inset: 1,
                right_reserve: 3,
            },
        );
        assert_eq!(viewport.grid.height, 0);
        assert_eq!(viewport.grid.width, 0);
        assert_eq!(viewport.horizontal.height, 1);
        assert_eq!(viewport.pending, Rect::new(0, 0, 2, 1));
        assert_eq!(viewport.status, Rect::new(0, 0, 2, 1));
    }
}
