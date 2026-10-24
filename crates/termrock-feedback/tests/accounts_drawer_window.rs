use core::ops::Range;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use termrock_feedback::{
    AccountsDrawer, DetailInk, DetailLine, DetailSpan, DrawerButton, DrawerInk, DrawerLine,
    DrawerMeter, DrawerTone, PaneChrome, paint_accounts_drawer,
};
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, FgStep, PaintStyle, Surface, Theme};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const FACT_SPANS: [DetailSpan<'static>; 4] = [
    DetailSpan {
        text: "scope:",
        ink: DetailInk::Muted,
    },
    DetailSpan {
        text: " Warning",
        ink: DetailInk::Warning,
    },
    DetailSpan {
        text: " · stale",
        ink: DetailInk::Danger,
    },
    DetailSpan {
        text: " · active",
        ink: DetailInk::Info,
    },
];
const FACT_LINE: DetailLine<'static> = DetailLine { spans: &FACT_SPANS };
const BUTTONS: [DrawerButton<'static>; 2] = [
    DrawerButton {
        label: "Refresh",
        danger: false,
    },
    DrawerButton {
        label: "Remove",
        danger: true,
    },
];
const LINES: [DrawerLine<'static>; 11] = [
    DrawerLine::Title {
        title: "Workspace profile",
        meta: "ACTIVE",
    },
    DrawerLine::Rule,
    DrawerLine::Field {
        label: "Account",
        value: "user@example.test",
        ink: DrawerInk::Plain,
    },
    DrawerLine::Value {
        text: "expired soon",
        ink: DrawerInk::Warning,
    },
    DrawerLine::Heading {
        label: "Quota",
        meta: "WEEK",
    },
    DrawerLine::Meter(DrawerMeter {
        label: "Requests",
        readout: "95%",
        fill_cols: 5,
        track_cols: 7,
        tone: DrawerTone::Warning,
        mark: "▲",
        mark_bold: true,
        note: "resets soon",
    }),
    DrawerLine::Meter(DrawerMeter {
        label: "Tokens",
        readout: "FULL",
        fill_cols: 4,
        track_cols: 8,
        tone: DrawerTone::Danger,
        mark: "!",
        mark_bold: false,
        note: "Exhausted",
    }),
    DrawerLine::Spans(&FACT_LINE),
    DrawerLine::Text("wide界 and combining e\u{301} stay in their columns"),
    DrawerLine::Buttons(&BUTTONS),
    DrawerLine::Blank,
];

const PRECHANGE_WHOLE_DRAWER: [[u64; 4]; 6] = [
    [
        0x40d4_e637_e19c_f30c,
        0xf41f_bfc0_8fb5_cbd2,
        0xc247_e72b_15dc_c146,
        0xa51a_f065_b4fb_da82,
    ],
    [
        0x2502_fcbf_0244_c148,
        0x9683_fe6d_6bf6_b2b0,
        0xdc18_9ef5_965c_4c99,
        0xa757_7118_d9dd_4a5f,
    ],
    [
        0x0c48_d953_4ace_fa5a,
        0x61a5_401f_c055_7f9e,
        0xa60f_e253_2c3c_bf02,
        0x5ba0_30a3_1397_8f7e,
    ],
    [
        0xbde1_04b3_10f0_e01a,
        0x3d77_791f_f7a3_cdb0,
        0xaa96_71a4_4377_03fc,
        0xc647_35d0_da3f_0291,
    ],
    [
        0x9b98_5ff1_afcf_6611,
        0x035e_d999_e9f9_58d9,
        0x53f0_fa36_6230_da2d,
        0x7ef2_2a21_6617_f604,
    ],
    [
        0x1090_404b_00a5_39b6,
        0xc0d4_9fd3_5c6c_0d54,
        0x0ed5_0799_c7d6_d3e2,
        0x61a8_0bfb_ffa7_f637,
    ],
];

fn page(chrome: PaneChrome) -> AccountsDrawer<'static> {
    AccountsDrawer {
        brand: "TERMS",
        menus: &[],
        crumb: "Accounts / Profile",
        status: "READY",
        cluster: termrock_feedback::settings_page::SettingsCluster {
            note: "",
            mark: "",
            value: "",
        },
        title: "Accounts",
        summary: "1 account",
        rows: &[],
        lines: &LINES,
        hints: &[],
        hint_overflow: "",
        chrome,
    }
}

fn custom_theme() -> Theme {
    let mut tokens = Theme::junie().color;
    tokens.accent = ratatui_core::style::Color::Rgb(0x13, 0x57, 0x9b);
    tokens.on_accent = ratatui_core::style::Color::Rgb(0xf0, 0xe1, 0xd2);
    tokens.border_strong = ratatui_core::style::Color::Rgb(0x4a, 0x8c, 0xb1);
    for (index, color) in tokens.surfaces.iter_mut().enumerate() {
        *color = ratatui_core::style::Color::Rgb(
            0x10 + index as u8,
            0x20 + index as u8,
            0x30 + index as u8,
        );
    }
    for (index, color) in tokens.fg.iter_mut().enumerate() {
        *color = ratatui_core::style::Color::Rgb(
            0xa0 + index as u8,
            0xb0 + index as u8,
            0xc0 + index as u8,
        );
    }
    Theme::from_tokens(tokens)
}

fn digest(scene: &Scene) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for (index, cell) in scene.buffer().content.iter().enumerate() {
        let record = format!("{index}|{cell:?}");
        for byte in record.bytes().chain([0xff]) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
    }
    hash
}

fn baseline_scene(theme: Theme, level: ColorLevel, chrome: PaneChrome) -> Scene {
    let mut scene = Scene::new("accounts-drawer-prechange", theme, level, 112, 36);
    let area = Rect::new(3, 2, 100, 30);
    scene.draw(|ui, _| paint_accounts_drawer(ui, area, &page(chrome)));
    scene
}

#[test]
fn whole_drawer_matches_prechange_full_cells_across_themes_and_tiers() {
    let themes = [
        ("junie", Theme::junie()),
        ("paper", Theme::paper()),
        ("custom", custom_theme()),
    ];
    for (theme_index, (_, theme)) in themes.into_iter().enumerate() {
        for (chrome_index, chrome) in [PaneChrome::Drawer, PaneChrome::Detail]
            .into_iter()
            .enumerate()
        {
            for (level_index, level) in LEVELS.into_iter().enumerate() {
                let scene = baseline_scene(theme.clone(), level, chrome);
                assert_eq!(
                    digest(&scene),
                    PRECHANGE_WHOLE_DRAWER[theme_index * 2 + chrome_index][level_index],
                    "theme index {theme_index}, chrome index {chrome_index}, level {level:?}"
                );
            }
        }
    }

    let mut nested = Scene::new(
        "accounts-drawer-prechange-nested-clip",
        Theme::junie(),
        ColorLevel::TrueColor,
        112,
        36,
    );
    let area = Rect::new(3, 2, 100, 30);
    let parent_clip = Rect::new(54, 8, 20, 10);
    nested.draw(|ui, _| {
        ui.with_area(parent_clip, |ui| {
            paint_accounts_drawer(ui, area, &page(PaneChrome::Drawer))
        })
    });
    assert_eq!(digest(&nested), 0x5ffd_35a4_5e65_162e);
}

fn drawer_layout() -> Rect {
    Rect::new(3, 2, 100, 30)
}

fn content_layout(area: Rect) -> Rect {
    let list_x = area.x.saturating_add(1);
    let list_right = list_x.saturating_add(39).saturating_sub(1);
    let content_x = list_right.saturating_add(3);
    Rect::new(
        content_x,
        area.y,
        area.right().saturating_sub(content_x),
        area.height,
    )
}

fn body_y(area: Rect) -> u16 {
    area.y.saturating_add(2)
}

fn primary_color(ui: &termrock::Ui<'_>) -> Color {
    ui.theme_ref().color.fg[FgStep::Primary.index()]
}

fn drawer_background_styles(ui: &termrock::Ui<'_>) -> (PaintStyle, PaintStyle) {
    let theme = ui.theme_ref();
    let clear = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM;
    (
        PaintStyle::new()
            .fg(primary_color(ui))
            .bg(theme.bg(Surface::Surface))
            .remove_modifier(clear),
        PaintStyle::new()
            .fg(primary_color(ui))
            .bg(theme.bg(Surface::Canvas))
            .remove_modifier(clear),
    )
}

fn fill_expected_drawer_background(ui: &mut termrock::Ui<'_>, layout: Rect, clip: Rect) {
    let (chrome, canvas) = drawer_background_styles(ui);
    let base_width = layout.width.saturating_sub(1);
    let base = Rect::new(layout.x, clip.y, base_width, clip.height).intersection(clip);
    ui.fill(base, chrome);
    if layout.width > 0 {
        let edge =
            Rect::new(layout.right().saturating_sub(1), clip.y, 1, clip.height).intersection(clip);
        ui.fill(edge, canvas);
    }
}

fn draw_window(
    theme: Theme,
    level: ColorLevel,
    chrome: PaneChrome,
    layout: Rect,
    clip: Rect,
    first_source_row: usize,
    parent_clip: Option<Rect>,
) -> (Scene, Option<Range<usize>>) {
    const SENTINEL_FG: Color = Color::Rgb(0xa1, 0x33, 0x7f);
    const SENTINEL_BG: Color = Color::Rgb(0x23, 0x6b, 0x8d);
    const SENTINEL: &str = "§";

    let mut scene = Scene::new(
        "accounts-drawer-window",
        theme,
        level,
        layout.right().saturating_add(8).max(112),
        layout.bottom().saturating_add(4).max(36),
    );
    let mut visible = None;
    scene.draw_over(
        |buffer| {
            for cell in &mut buffer.content {
                cell.set_symbol(SENTINEL).set_style(
                    Style::new()
                        .fg(SENTINEL_FG)
                        .bg(SENTINEL_BG)
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
        },
        |ui, _| {
            let paint = |ui: &mut termrock::Ui<'_>, visible: &mut Option<Range<usize>>| {
                fill_expected_drawer_background(ui, layout, clip);
                *visible = termrock_feedback::paint_accounts_drawer_lines_window(
                    ui,
                    layout,
                    clip,
                    first_source_row,
                    &LINES,
                    chrome,
                );
            };
            if let Some(parent_clip) = parent_clip {
                ui.with_area(parent_clip, |ui| paint(ui, &mut visible));
            } else {
                paint(ui, &mut visible);
            }
        },
    );
    (scene, visible)
}

fn assert_clip_matches_full(full: &Scene, actual: &Scene, clip: Rect) {
    for y in clip.y..clip.bottom() {
        for x in clip.x..clip.right() {
            let pos = Position::new(x, y);
            assert_eq!(
                format!("{:?}", full.buffer().cell(pos)),
                format!("{:?}", actual.buffer().cell(pos)),
                "different full-cell output at {pos:?}"
            );
        }
    }
}

fn assert_sentinels_outside_clip(scene: &Scene, clip: Rect) {
    const SENTINEL_FG: Color = Color::Rgb(0xa1, 0x33, 0x7f);
    const SENTINEL_BG: Color = Color::Rgb(0x23, 0x6b, 0x8d);
    let area = *scene.buffer().area();
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            if clip.contains(Position::new(x, y)) {
                continue;
            }
            let cell = scene.buffer().cell(Position::new(x, y)).unwrap();
            assert_eq!(cell.symbol(), "§", "outside sentinel at ({x}, {y})");
            assert_eq!(cell.fg, SENTINEL_FG, "outside sentinel fg at ({x}, {y})");
            assert_eq!(cell.bg, SENTINEL_BG, "outside sentinel bg at ({x}, {y})");
            assert_eq!(
                cell.modifier,
                Modifier::UNDERLINED,
                "outside sentinel modifiers at ({x}, {y})"
            );
        }
    }
}

fn effective_clip(requested: Rect, parent: Option<Rect>) -> Rect {
    parent.map_or(requested, |parent| requested.intersection(parent))
}

#[test]
fn window_rows_match_existing_full_painter_for_every_line_kind_and_tier() {
    let themes = [Theme::junie(), Theme::paper(), custom_theme()];
    let area = drawer_layout();
    let layout = content_layout(area);
    let clip = Rect::new(
        layout.x,
        body_y(area),
        layout.width.saturating_sub(1),
        u16::try_from(LINES.len()).unwrap(),
    );

    for theme in themes {
        for level in LEVELS {
            for chrome in [PaneChrome::Drawer, PaneChrome::Detail] {
                let expected = baseline_scene(theme.clone(), level, chrome);
                let (actual, rows) =
                    draw_window(theme.clone(), level, chrome, layout, clip, 0, None);
                assert_eq!(rows, Some(0..LINES.len()));
                assert_clip_matches_full(&expected, &actual, clip);
                assert_sentinels_outside_clip(&actual, clip);
            }
        }
    }
}

#[test]
fn window_offsets_preserve_original_anchors_and_advance_through_parent_clips() {
    let area = drawer_layout();
    let layout = content_layout(area);
    let body = Rect::new(
        layout.x,
        body_y(area),
        layout.width.saturating_sub(1),
        u16::try_from(LINES.len()).unwrap(),
    );
    let expected = baseline_scene(Theme::junie(), ColorLevel::TrueColor, PaneChrome::Drawer);

    let title_clip = Rect::new(
        layout.x + 35,
        body.y,
        layout.right().saturating_sub(layout.x + 35),
        1,
    );
    let (title, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        title_clip,
        0,
        None,
    );
    assert_eq!(rows, Some(0..1));
    assert_clip_matches_full(&expected, &title, title_clip);
    assert_eq!(
        title
            .buffer()
            .cell(Position::new(layout.x + 49, body.y))
            .unwrap()
            .symbol(),
        "A",
        "right-aligned title metadata retains the original right anchor"
    );

    let (offset, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        Rect::new(layout.x + 4, body.y + 3, 25, 3),
        3,
        None,
    );
    let offset_clip = Rect::new(layout.x + 4, body.y + 3, 25, 3);
    assert_eq!(rows, Some(3..6));
    assert_clip_matches_full(&expected, &offset, offset_clip);
    assert_sentinels_outside_clip(&offset, offset_clip);

    let parent_clip = Rect::new(layout.x + 3, body.y + 3, 20, 4);
    let (parent, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        body,
        0,
        Some(parent_clip),
    );
    assert_eq!(rows, Some(3..7));
    assert_clip_matches_full(&expected, &parent, parent_clip);
    assert_sentinels_outside_clip(&parent, effective_clip(body, Some(parent_clip)));
}

#[test]
fn horizontal_clips_keep_grapheme_columns_and_omit_partial_wide_clusters() {
    let area = drawer_layout();
    let layout = content_layout(area);
    let row_y = body_y(area) + 8;
    let expected = baseline_scene(Theme::junie(), ColorLevel::TrueColor, PaneChrome::Drawer);

    let left = Rect::new(layout.x + 5, row_y, 24, 1);
    let (actual, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        left,
        8,
        None,
    );
    assert_eq!(rows, Some(8..9));
    for y in left.y..left.bottom() {
        for x in left.x..left.right() {
            if x == left.x {
                continue;
            }
            let pos = Position::new(x, y);
            assert_eq!(
                format!("{:?}", expected.buffer().cell(pos)),
                format!("{:?}", actual.buffer().cell(pos)),
                "left crop shifted a complete grapheme at {pos:?}"
            );
        }
    }
    assert_eq!(
        actual
            .buffer()
            .cell(Position::new(left.x, row_y))
            .unwrap()
            .symbol(),
        " ",
        "a wide cluster cut by the left edge is omitted"
    );
    assert_eq!(
        actual
            .buffer()
            .cell(Position::new(left.x + 2, row_y))
            .unwrap()
            .symbol(),
        "a",
        "text after the omitted wide cluster remains at its original column"
    );
    assert_sentinels_outside_clip(&actual, left);

    let right = Rect::new(layout.x, row_y, 5, 1);
    let (actual, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        right,
        8,
        None,
    );
    assert_eq!(rows, Some(8..9));
    for x in right.x..right.right().saturating_sub(1) {
        let pos = Position::new(x, row_y);
        assert_eq!(
            format!("{:?}", expected.buffer().cell(pos)),
            format!("{:?}", actual.buffer().cell(pos)),
            "right crop changed a complete grapheme at {pos:?}"
        );
    }
    assert_eq!(
        actual
            .buffer()
            .cell(Position::new(right.right() - 1, row_y))
            .unwrap()
            .symbol(),
        " ",
        "a wide cluster crossing the right edge is omitted whole"
    );
    assert_sentinels_outside_clip(&actual, right);

    let combining_x = layout.x + 21;
    let combining = Rect::new(combining_x, row_y, 1, 1);
    let (actual, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        combining,
        8,
        None,
    );
    assert_eq!(rows, Some(8..9));
    assert_eq!(
        actual
            .buffer()
            .cell(Position::new(combining_x, row_y))
            .unwrap()
            .symbol(),
        "e\u{301}",
        "the base and combining mark stay one grapheme at the clip boundary"
    );
    assert_clip_matches_full(&expected, &actual, combining);
    assert_sentinels_outside_clip(&actual, combining);
}

#[test]
fn empty_tiny_and_invalid_windows_are_bounded_and_side_effect_safe() {
    let area = drawer_layout();
    let layout = content_layout(area);
    let empty = Rect::new(layout.x + 3, body_y(area), 0, 0);
    let (empty_scene, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        empty,
        0,
        None,
    );
    assert_eq!(rows, Some(0..0));
    assert_sentinels_outside_clip(&empty_scene, empty);

    let tiny = Rect::new(layout.x + 4, body_y(area) + 8, 1, 1);
    let (tiny_scene, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        tiny,
        8,
        None,
    );
    assert_eq!(rows, Some(8..9));
    let cell = tiny_scene
        .buffer()
        .cell(Position::new(tiny.x, tiny.y))
        .unwrap();
    assert_eq!(cell.symbol(), " ", "a partial wide grapheme cannot leak");
    assert_eq!(cell.fg, Theme::junie().color.fg[FgStep::Primary.index()]);
    assert_eq!(cell.bg, Theme::junie().bg(Surface::Surface));
    assert_sentinels_outside_clip(&tiny_scene, tiny);

    let mut invalid = Scene::new(
        "invalid-window",
        Theme::junie(),
        ColorLevel::TrueColor,
        112,
        36,
    );
    // `Rect::new` clamps overflowing dimensions, so use the public fields to
    // exercise malformed rectangles supplied by callers that construct Rects
    // directly.
    let invalid_layout = Rect {
        x: u16::MAX - 1,
        y: 0,
        width: 4,
        height: 2,
    };
    let cases = [
        (layout, Rect::new(layout.right(), body_y(area), 1, 1), 0),
        (
            layout,
            Rect::new(layout.x, body_y(area), 1, 1),
            LINES.len().saturating_add(1),
        ),
        (invalid_layout, Rect::new(u16::MAX - 1, 0, 4, 2), 0),
        (
            layout,
            Rect {
                x: layout.x,
                y: u16::MAX - 1,
                width: 1,
                height: 3,
            },
            0,
        ),
    ];
    invalid.draw_over(
        |buffer| {
            for cell in &mut buffer.content {
                cell.set_symbol("§").set_style(
                    Style::new()
                        .fg(Color::Rgb(0xa1, 0x33, 0x7f))
                        .bg(Color::Rgb(0x23, 0x6b, 0x8d))
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
        },
        |ui, _| {
            for (layout, clip, first_source_row) in cases {
                assert_eq!(
                    termrock_feedback::paint_accounts_drawer_lines_window(
                        ui,
                        layout,
                        clip,
                        first_source_row,
                        &LINES,
                        PaneChrome::Drawer,
                    ),
                    None,
                    "invalid window {:?} into {:?} at {first_source_row}",
                    clip,
                    layout
                );
            }
        },
    );
    assert_sentinels_outside_clip(&invalid, Rect::new(0, 0, 0, 0));
}

#[test]
fn warning_and_exhausted_meter_cells_retain_fill_track_mark_and_status_inks() {
    const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
    const EXHAUSTED: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
    const TRACK: Color = Color::Rgb(0x26, 0x26, 0x2a);

    let wide_area = Rect::new(3, 2, 150, 30);
    let layout = content_layout(wide_area);
    let clip = Rect::new(layout.x, body_y(wide_area) + 5, layout.width - 1, 2);
    let expected = {
        let mut scene = Scene::new(
            "wide-meter-baseline",
            Theme::junie(),
            ColorLevel::TrueColor,
            160,
            36,
        );
        scene.draw(|ui, _| {
            paint_accounts_drawer(ui, wide_area, &page(PaneChrome::Drawer));
        });
        scene
    };
    let (actual, rows) = draw_window(
        Theme::junie(),
        ColorLevel::TrueColor,
        PaneChrome::Drawer,
        layout,
        clip,
        5,
        None,
    );
    assert_eq!(rows, Some(5..7));
    assert_clip_matches_full(&expected, &actual, clip);
    let secondary = Theme::junie().color.fg[FgStep::Secondary.index()];
    let chrome = Theme::junie().bg(Surface::Surface);
    let bar_x = layout.x + 22;
    let warning_y = body_y(wide_area) + 5;
    for x in bar_x..bar_x + 5 {
        let cell = actual.buffer().cell(Position::new(x, warning_y)).unwrap();
        assert_eq!(cell.fg, Color::Rgb(0, 0, 0));
        assert_eq!(cell.bg, WARNING);
        assert_eq!(cell.modifier, Modifier::BOLD | Modifier::ITALIC);
    }
    for x in bar_x + 5..bar_x + 12 {
        let cell = actual.buffer().cell(Position::new(x, warning_y)).unwrap();
        assert_eq!(cell.fg, secondary);
        assert_eq!(cell.bg, TRACK);
        assert_eq!(cell.modifier, Modifier::ITALIC);
    }
    let warning_mark = actual
        .buffer()
        .cell(Position::new(bar_x + 34, warning_y))
        .unwrap();
    assert_eq!(warning_mark.symbol(), "▲");
    assert_eq!(warning_mark.fg, WARNING);
    assert_eq!(warning_mark.bg, chrome);
    assert_eq!(warning_mark.modifier, Modifier::BOLD);
    let warning_note = actual
        .buffer()
        .cell(Position::new(bar_x + 38, warning_y))
        .unwrap();
    assert_eq!(warning_note.symbol(), "r");
    assert_eq!(
        warning_note.fg,
        Theme::junie().color.fg[FgStep::Muted.index()]
    );
    assert_eq!(warning_note.bg, chrome);
    assert_eq!(warning_note.modifier, Modifier::empty());

    let exhausted_y = warning_y + 1;
    for x in bar_x..bar_x + 4 {
        let cell = actual.buffer().cell(Position::new(x, exhausted_y)).unwrap();
        assert_eq!(cell.bg, EXHAUSTED);
        assert_eq!(cell.fg, Color::Rgb(0, 0, 0));
        assert_eq!(cell.modifier, Modifier::BOLD | Modifier::ITALIC);
    }
    for x in bar_x + 4..bar_x + 12 {
        let cell = actual.buffer().cell(Position::new(x, exhausted_y)).unwrap();
        assert_eq!(cell.bg, TRACK);
        assert_eq!(cell.fg, secondary);
        assert_eq!(cell.modifier, Modifier::ITALIC);
    }
    let exhausted_mark = actual
        .buffer()
        .cell(Position::new(bar_x + 34, exhausted_y))
        .unwrap();
    assert_eq!(exhausted_mark.symbol(), "!");
    assert_eq!(exhausted_mark.fg, EXHAUSTED);
    assert_eq!(exhausted_mark.bg, chrome);
    assert_eq!(exhausted_mark.modifier, Modifier::empty());
    let exhausted_note = actual
        .buffer()
        .cell(Position::new(bar_x + 38, exhausted_y))
        .unwrap();
    assert_eq!(exhausted_note.symbol(), "E");
    assert_eq!(
        exhausted_note.fg,
        Theme::junie().color.fg[FgStep::Muted.index()]
    );
    assert_eq!(exhausted_note.bg, chrome);
    assert_eq!(exhausted_note.modifier, Modifier::empty());
}
