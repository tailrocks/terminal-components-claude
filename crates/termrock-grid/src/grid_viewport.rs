use ratatui_core::layout::Rect;

use termrock_runtime::ui::Ui;
use termrock_theme::{ColorLevel, Modifier, StylePatch};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridViewportPolicy {
    pub header_rows: u16,

    pub filter_rows: u16,

    pub pending_rows: u16,

    pub status_rows: u16,

    pub status_inset: u16,

    pub right_reserve: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridViewport {
    pub header: Rect,

    pub filter: Rect,

    pub filter_bar: Rect,

    pub grid: Rect,

    pub horizontal: Rect,

    pub pending: Rect,

    pub status: Rect,

    pub status_text: Rect,
}

impl GridViewport {
    pub fn has_grid_body(&self) -> bool {
        !self.grid.is_empty()
    }

    pub fn status_text_fits(&self, text_width: u16) -> bool {
        text_width <= self.status_text.width
    }
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridContentChrome {
    pub gutter: u16,

    pub content_width: u16,

    pub columns: usize,

    pub complete: usize,

    pub hidden_left: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridBarPolicy {
    pub track: termrock_theme::Role,

    pub thumb: termrock_theme::Role,

    pub track_glyph: &'static str,

    pub thumb_glyph: &'static str,

    pub cover_far: termrock_theme::Role,

    pub cover_near: termrock_theme::Role,

    pub cover_far_dim: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridFocusPolicy {
    pub role: termrock_theme::Role,

    pub bold: bool,

    pub glyph: &'static str,
}

pub fn paint_grid_vertical_bar(
    ui: &mut Ui<'_>,
    area: Rect,
    chrome: &GridContentChrome,
    rows: usize,
    origin: usize,
    focused: Option<usize>,
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

        let depth = if track_len >= 12 {
            2
        } else if track_len >= 4 {
            1
        } else {
            0
        };
        let far = depth >= 1 && (from_end == 0 || (from_start && offset == 0));
        let near = depth >= 2 && (from_end == 1 || (from_start && offset == 1));
        let cover = if !truecolor {
            if far {
                if policy.cover_far_dim {
                    plain.add_modifier(Modifier::DIM)
                } else {
                    plain
                }
            } else {
                plain
            }
        } else if far {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.cover_far)))
        } else if near {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(policy.cover_near)))
        } else {
            if focused == Some(offset) {
                plain.add_modifier(Modifier::BOLD)
            } else {
                plain
            }
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
