//! Frame equality between the pre-refactor app-owned grid chrome and the
//! shared grid viewport block.
//!
//! The legacy painters below are the TablePro grid chrome as it stood before
//! the viewport moved into `termrock-grid`. They are kept here as the oracle:
//! the shared block must resolve the same regions and paint the same cells,
//! styles and modifiers as the app-owned path it replaced.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_grid::{
    GridBarPolicy, GridContentChrome, GridFocusPolicy, GridViewportPolicy, paint_grid_focus_bar,
    paint_grid_horizontal_bar, paint_grid_vertical_bar, solve_grid_viewport,
};
use termrock_runtime::ui::Ui;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Modifier, Role, StylePatch, Theme};

const TRACK: Role = Role::Custom(Color::from_u32(0x0047_4747));
const THUMB: Role = Role::Custom(Color::from_u32(0x0080_8080));
const COVER_FAR: Role = Role::Custom(Color::from_u32(0x008c_8c8c));
const COVER_NEAR: Role = Role::Custom(Color::from_u32(0x00cc_cccc));

/// The app-owned chrome the legacy painters read.
struct LegacyChrome {
    gutter: u16,
    content_width: u16,
    columns: usize,
    complete: usize,
    hidden_left: usize,
    places: usize,
}

impl LegacyChrome {
    fn shared(&self) -> GridContentChrome {
        GridContentChrome {
            gutter: self.gutter,
            content_width: self.content_width,
            columns: self.columns,
            complete: self.complete,
            hidden_left: self.hidden_left,
        }
    }
}

fn vertical_policy() -> GridBarPolicy {
    GridBarPolicy {
        track: TRACK,
        thumb: THUMB,
        track_glyph: "┆",
        thumb_glyph: "┃",
        cover_far: COVER_FAR,
        cover_near: COVER_NEAR,
        cover_far_dim: true,
    }
}

fn horizontal_policy() -> GridBarPolicy {
    GridBarPolicy {
        track: TRACK,
        thumb: THUMB,
        track_glyph: "┄",
        thumb_glyph: "━",
        cover_far: COVER_FAR,
        cover_near: COVER_NEAR,
        cover_far_dim: true,
    }
}

fn focus_policy() -> GridFocusPolicy {
    GridFocusPolicy {
        role: Role::Accent,
        bold: true,
        glyph: "▎",
    }
}

/// The vertical scroll bar as the app painted it before the extraction.
fn legacy_vertical_bar(
    ui: &mut Ui<'_>,
    area: Rect,
    chrome: &LegacyChrome,
    rows: usize,
    origin: usize,
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
    if chrome.places == 0 {
        return;
    }
    let x = area
        .x
        .saturating_add(chrome.gutter)
        .saturating_add(chrome.content_width)
        .saturating_add(4);
    let covered = x.saturating_sub(1);
    let plain = ui.surface_style();
    let track = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(TRACK)));
    let thumb = plain.patch(ui.paint_patch(&StylePatch::new().set_fg(THUMB)));
    for offset in 0..track_len {
        let y = area.y.saturating_add(1).saturating_add(offset as u16);
        let from_end = track_len.saturating_sub(offset).saturating_sub(1);
        let from_start = origin > 0 && (offset == 0 || offset == 1);
        let truecolor = ui.theme_ref().capability.color == ColorLevel::TrueColor;
        let cover = if !truecolor {
            if from_end == 0 || (from_start && offset == 0) {
                plain.add_modifier(Modifier::DIM)
            } else {
                plain
            }
        } else if from_end == 0 || (from_start && offset == 0) {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(COVER_FAR)))
        } else if from_end == 1 || (from_start && offset == 1) {
            plain.patch(ui.paint_patch(&StylePatch::new().set_fg(COVER_NEAR)))
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
        let glyph = if on_thumb { "┃" } else { "┆" };
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

/// The horizontal scroll bar as the app painted it before the extraction.
fn legacy_horizontal_bar(ui: &mut Ui<'_>, row: Rect, chrome: &LegacyChrome) {
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
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "columns is at least 1 and the product fits a terminal width"
    )]
    let thumb = (usize::from(track_w) * chrome.complete.max(1) / columns)
        .max(1)
        .min(usize::from(track_w));
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "columns is at least 1 and the product fits a terminal width"
    )]
    let thumb_at = if chrome.hidden_left == 0 {
        0
    } else {
        (usize::from(track_w) * chrome.hidden_left + columns / 2) / columns
    }
    .min(usize::from(track_w).saturating_sub(thumb));
    let track = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(TRACK)));
    let thumb_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(THUMB)));
    ui.paint_str(
        Rect {
            x,
            y: row.y,
            width: track_w,
            height: 1,
        },
        &"┄".repeat(usize::from(track_w)),
        track,
    );
    ui.paint_str(
        Rect {
            x: x.saturating_add(thumb_at as u16),
            y: row.y,
            width: thumb as u16,
            height: 1,
        },
        &"━".repeat(thumb),
        thumb_style,
    );
}

/// The focus bar as the app painted it before the extraction.
fn legacy_focus_bar(ui: &mut Ui<'_>, area: Rect, row: u16) {
    let y = area.y.saturating_add(1).saturating_add(row);
    if y >= area.bottom() {
        return;
    }
    let style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD)));
    ui.paint_str(
        Rect {
            x: area.x,
            y,
            width: 1,
            height: 1,
        },
        "▎",
        style,
    );
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

#[test]
fn the_shared_viewport_resolves_the_legacy_result_grid_regions() {
    for (width, height) in [(40u16, 12u16), (80, 24), (120, 40), (3, 2), (1, 1)] {
        let area = Rect::new(0, 0, width, height);
        let legacy_grid = Rect {
            width: area.width.saturating_sub(3),
            height: area.height.saturating_sub(1),
            ..area
        };
        let legacy_bar = Rect {
            y: area.bottom().saturating_sub(1),
            width: area.width.saturating_sub(3),
            height: 1,
            ..area
        };
        let viewport = solve_grid_viewport(
            area,
            &GridViewportPolicy {
                right_reserve: 3,
                ..GridViewportPolicy::default()
            },
        );
        assert_eq!(
            viewport.grid, legacy_grid,
            "result grid body at {width}x{height}"
        );
        assert_eq!(
            viewport.horizontal, legacy_bar,
            "result grid bar at {width}x{height}"
        );
        assert_eq!(
            viewport.pending,
            Rect::ZERO,
            "result grid pending at {width}x{height}"
        );
        assert_eq!(
            viewport.status,
            Rect::ZERO,
            "result grid status at {width}x{height}"
        );
    }
}

#[test]
fn the_shared_viewport_resolves_the_legacy_table_grid_regions() {
    for (width, height) in [(40u16, 12u16), (80, 24), (120, 40), (3, 2), (1, 1)] {
        for filter_rows in [0u16, 2] {
            for pending_gap in [0u16, 2] {
                let area = Rect::new(0, 0, width, height);
                let legacy_grid = Rect {
                    y: area.y.saturating_add(3).saturating_add(filter_rows),
                    height: area
                        .height
                        .saturating_sub(5)
                        .saturating_sub(filter_rows)
                        .saturating_sub(pending_gap),
                    ..area
                };
                let legacy_status = Rect {
                    y: area.bottom().saturating_sub(1),
                    height: 1,
                    ..area
                };
                let legacy_bar = Rect {
                    y: legacy_status
                        .y
                        .saturating_sub(1)
                        .saturating_sub(pending_gap),
                    height: 1,
                    ..legacy_status
                };
                let legacy_pending = Rect {
                    y: legacy_status.y.saturating_sub(1),
                    height: 1,
                    ..legacy_status
                };
                let legacy_filter = Rect {
                    y: area.y.saturating_add(3),
                    height: 1,
                    ..area
                };
                let viewport = solve_grid_viewport(
                    area,
                    &GridViewportPolicy {
                        header_rows: 3,
                        filter_rows,
                        pending_rows: pending_gap,
                        status_rows: 1,
                        right_reserve: 0,
                    },
                );
                let label = format!("{width}x{height} filters {filter_rows} pending {pending_gap}");
                assert_eq!(viewport.grid, legacy_grid, "table grid body: {label}");
                assert_eq!(viewport.status, legacy_status, "table grid status: {label}");
                assert_eq!(viewport.horizontal, legacy_bar, "table grid bar: {label}");
                if pending_gap == 0 {
                    assert_eq!(viewport.pending, Rect::ZERO, "table grid pending: {label}");
                } else {
                    assert_eq!(
                        viewport.pending, legacy_pending,
                        "table grid pending: {label}"
                    );
                }
                if filter_rows == 0 {
                    assert_eq!(
                        viewport.filter,
                        Rect::new(area.x, area.y.saturating_add(3), area.width, 0)
                    );
                } else {
                    assert_eq!(
                        viewport.filter,
                        Rect {
                            height: filter_rows,
                            ..legacy_filter
                        },
                        "table grid filter: {label}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_shared_vertical_bar_paints_the_legacy_cells() {
    for (width, height) in [(40u16, 12u16), (80, 24), (20, 3)] {
        for rows in [1usize, 5, 40, 400] {
            for origin in [0usize, 3, 39] {
                for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                    let area = Rect::new(0, 0, width, height);
                    let chrome = LegacyChrome {
                        gutter: 4,
                        content_width: width.saturating_sub(11),
                        columns: 6,
                        complete: 4,
                        hidden_left: 0,
                        places: 6,
                    };
                    let mut legacy = scene("legacy", width, height, color);
                    legacy.draw(|ui, _| legacy_vertical_bar(ui, area, &chrome, rows, origin));
                    let mut shared = scene("shared", width, height, color);
                    shared.draw(|ui, _| {
                        paint_grid_vertical_bar(
                            ui,
                            area,
                            &chrome.shared(),
                            rows,
                            origin,
                            &vertical_policy(),
                        );
                    });
                    assert_same_cells(
                        legacy.buffer(),
                        shared.buffer(),
                        &format!("vertical bar {width}x{height} rows {rows} origin {origin}"),
                    );
                }
            }
        }
    }
}

#[test]
fn the_shared_horizontal_bar_paints_the_legacy_cells() {
    for width in [20u16, 40, 120] {
        for hidden_left in [0usize, 2, 9] {
            for complete in [1usize, 4, 6] {
                let area = Rect::new(0, 0, width, 10);
                let row = Rect {
                    y: 9,
                    height: 1,
                    ..area
                };
                let chrome = LegacyChrome {
                    gutter: 4,
                    content_width: width.saturating_sub(11),
                    columns: 6,
                    complete,
                    hidden_left,
                    places: 6,
                };
                let mut legacy = scene("legacy", width, 10, ColorLevel::TrueColor);
                legacy.draw(|ui, _| legacy_horizontal_bar(ui, row, &chrome));
                let mut shared = scene("shared", width, 10, ColorLevel::TrueColor);
                shared.draw(|ui, _| {
                    paint_grid_horizontal_bar(ui, row, &chrome.shared(), &horizontal_policy());
                });
                assert_same_cells(
                    legacy.buffer(),
                    shared.buffer(),
                    &format!("horizontal bar width {width} hidden {hidden_left}"),
                );
            }
        }
    }
}

#[test]
fn the_shared_focus_bar_paints_the_legacy_cells() {
    for (width, height) in [(40u16, 12u16), (80, 24), (20, 3)] {
        for row in [0u16, 1, 5, 40] {
            let area = Rect::new(0, 0, width, height);
            let mut legacy = scene("legacy", width, height, ColorLevel::TrueColor);
            legacy.draw(|ui, _| legacy_focus_bar(ui, area, row));
            let mut shared = scene("shared", width, height, ColorLevel::TrueColor);
            shared.draw(|ui, _| paint_grid_focus_bar(ui, area, Some(row), &focus_policy()));
            assert_same_cells(
                legacy.buffer(),
                shared.buffer(),
                &format!("focus bar {width}x{height} row {row}"),
            );
        }
    }
}
