use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock_feedback::{
    DetailInk, DetailLine, DetailSpan, DrawerButton, DrawerInk, DrawerLine, DrawerMeter,
    DrawerProjectionError, DrawerProjectionSource, DrawerTone, PaneChrome,
    paint_accounts_drawer_lines_window, project_accounts_drawer_line,
};
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, FgStep, PaintStyle, Surface, Theme};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const SPANS: [DetailSpan<'static>; 4] = [
    DetailSpan {
        text: "state:",
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
        text: " · ready",
        ink: DetailInk::Info,
    },
];
const SPAN_LINE: DetailLine<'static> = DetailLine { spans: &SPANS };
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
        text: "quota warning",
        ink: DrawerInk::Warning,
    },
    DrawerLine::Heading {
        label: "Usage",
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
    DrawerLine::Spans(&SPAN_LINE),
    DrawerLine::Text("界 e\u{301} 👩‍💻 literal tail"),
    DrawerLine::Buttons(&BUTTONS),
    DrawerLine::Blank,
];

fn custom_theme() -> Theme {
    let mut tokens = Theme::junie().color;
    tokens.accent = Color::Rgb(0x13, 0x57, 0x9b);
    tokens.on_accent = Color::Rgb(0xf0, 0xe1, 0xd2);
    tokens.border_strong = Color::Rgb(0x4a, 0x8c, 0xb1);
    for (index, color) in tokens.surfaces.iter_mut().enumerate() {
        *color = Color::Rgb(0x10 + index as u8, 0x20 + index as u8, 0x30 + index as u8);
    }
    for (index, color) in tokens.fg.iter_mut().enumerate() {
        *color = Color::Rgb(0xa0 + index as u8, 0xb0 + index as u8, 0xc0 + index as u8);
    }
    Theme::from_tokens(tokens)
}

fn row_layout() -> Rect {
    Rect::new(13, 4, 92, 1)
}

fn draw_source(ui: &mut termrock::Ui<'_>, layout: Rect, line: &DrawerLine<'_>) {
    let theme = ui.theme_ref();
    let clear = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM;
    ui.fill(
        layout,
        PaintStyle::new()
            .fg(theme.color.fg[FgStep::Primary.index()])
            .bg(theme.bg(Surface::Surface))
            .remove_modifier(clear),
    );
    let _ = paint_accounts_drawer_lines_window(
        ui,
        layout,
        layout,
        0,
        core::slice::from_ref(line),
        PaneChrome::Drawer,
    );
}

fn draw_projection(ui: &mut termrock::Ui<'_>, layout: Rect, line: &DrawerLine<'_>) {
    let theme = ui.theme_ref();
    let clear = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM;
    ui.fill(
        layout,
        PaintStyle::new()
            .fg(theme.color.fg[FgStep::Primary.index()])
            .bg(theme.bg(Surface::Surface))
            .remove_modifier(clear),
    );
    let projection =
        project_accounts_drawer_line(theme, layout, layout.x, line, PaneChrome::Drawer)
            .expect("fixture line should have a grapheme-safe projection");
    assert!(core::ptr::eq(projection.source(), line));
    for range in projection.ranges() {
        let Some(style) = range.style() else {
            continue;
        };
        let bytes = range.bytes();
        let columns = range.columns();
        let x = projection
            .origin_x()
            .checked_add(u16::try_from(columns.start).expect("fixture column fits"))
            .expect("fixture x fits");
        let width = u16::try_from(columns.end - columns.start).expect("fixture width fits");
        ui.paint_str(
            Rect::new(x, layout.y, width, 1),
            &projection.text()[bytes],
            style,
        );
    }
}

fn draw_scene(line: &DrawerLine<'_>, theme: Theme, level: ColorLevel, projected: bool) -> Scene {
    let mut scene = Scene::new("accounts-drawer-projection", theme, level, 120, 12);
    let layout = row_layout();
    scene.draw_over(
        |buffer| {
            for cell in &mut buffer.content {
                cell.set_symbol("§").set_style(
                    ratatui_core::style::Style::new()
                        .fg(Color::Rgb(0xa1, 0x33, 0x7f))
                        .bg(Color::Rgb(0x23, 0x6b, 0x8d))
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
        },
        |ui, _| {
            if projected {
                draw_projection(ui, layout, line);
            } else {
                draw_source(ui, layout, line);
            }
        },
    );
    scene
}

fn draw_clipped_scene(
    line: &DrawerLine<'_>,
    theme: Theme,
    level: ColorLevel,
    projected: bool,
    clip: Rect,
) -> Scene {
    let mut scene = Scene::new("accounts-drawer-projection-clip", theme, level, 120, 12);
    let layout = row_layout();
    scene.draw_over(
        |buffer| {
            for cell in &mut buffer.content {
                cell.set_symbol("§").set_style(
                    ratatui_core::style::Style::new()
                        .fg(Color::Rgb(0xa1, 0x33, 0x7f))
                        .bg(Color::Rgb(0x23, 0x6b, 0x8d))
                        .add_modifier(Modifier::UNDERLINED),
                );
            }
        },
        |ui, _| {
            let theme = ui.theme_ref();
            let clear = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM;
            ui.fill(
                clip,
                PaintStyle::new()
                    .fg(theme.color.fg[FgStep::Primary.index()])
                    .bg(theme.bg(Surface::Surface))
                    .remove_modifier(clear),
            );
            ui.with_area(clip, |ui| {
                if projected {
                    draw_projection(ui, layout, line);
                } else {
                    let _ = paint_accounts_drawer_lines_window(
                        ui,
                        layout,
                        clip,
                        0,
                        core::slice::from_ref(line),
                        PaneChrome::Drawer,
                    );
                }
            });
        },
    );
    scene
}

#[test]
fn projection_matches_window_painter_full_cells_for_every_line_and_theme_tier() {
    let themes = [Theme::junie(), Theme::paper(), custom_theme()];
    for (theme_index, theme) in themes.into_iter().enumerate() {
        for level in LEVELS {
            for (line_index, line) in LINES.iter().enumerate() {
                let source = draw_scene(line, theme.clone(), level, false);
                let projected = draw_scene(line, theme.clone(), level, true);
                for y in 0..source.area().height {
                    for x in 0..source.area().width {
                        let pos = Position::new(x, y);
                        assert_eq!(
                            format!("{:?}", source.buffer().cell(pos)),
                            format!("{:?}", projected.buffer().cell(pos)),
                            "theme {theme_index}, level {level:?}, line {line_index}, cell {pos:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn logical_projection_keeps_offsets_while_ui_and_window_clips_match_at_grapheme_edges() {
    let line = &LINES[8];
    let layout = row_layout();
    // Begin after the wide CJK glyph and end after a complete visible word.
    let clip = Rect::new(layout.x + 3, layout.y, 12, 1);
    for level in LEVELS {
        let source = draw_clipped_scene(line, Theme::paper(), level, false, clip);
        let projected = draw_clipped_scene(line, Theme::paper(), level, true, clip);
        for y in 0..source.area().height {
            for x in 0..source.area().width {
                let pos = Position::new(x, y);
                assert_eq!(
                    format!("{:?}", source.buffer().cell(pos)),
                    format!("{:?}", projected.buffer().cell(pos)),
                    "level {level:?}, cell {pos:?}"
                );
                if !clip.contains(pos) {
                    assert_eq!(
                        source.buffer().cell(pos).unwrap().symbol(),
                        "§",
                        "outside clip at {pos:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn projection_keeps_columns_styles_semantic_segments_and_unicode_source() {
    let layout = row_layout();
    let field = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &LINES[2],
        PaneChrome::Drawer,
    )
    .unwrap();
    assert_eq!(&field.text()[..9], "  Account");
    assert_eq!(&field.text()[22..], "user@example.test");
    assert_eq!(
        field.ranges()[0].source(),
        DrawerProjectionSource::LayoutGap
    );
    let value_ranges: Vec<_> = field
        .ranges()
        .iter()
        .filter(|range| range.source() == DrawerProjectionSource::FieldValue)
        .collect();
    assert!(!value_ranges.is_empty());
    assert_eq!(
        value_ranges.first().unwrap().source_bytes().unwrap().start,
        0
    );
    assert_eq!(
        value_ranges.last().unwrap().source_bytes().unwrap().end,
        "user@example.test".len()
    );
    assert!(value_ranges.windows(2).all(|pair| {
        pair[0].source_bytes().unwrap().end == pair[1].source_bytes().unwrap().start
    }));

    let warning = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &LINES[5],
        PaneChrome::Drawer,
    )
    .unwrap();
    let readout = warning
        .ranges()
        .iter()
        .find(|range| range.source() == DrawerProjectionSource::MeterReadout)
        .unwrap();
    let readout_style = readout.style().unwrap();
    assert_eq!(
        readout_style.as_style().bg,
        Some(Color::Rgb(0xf5, 0x9e, 0x09))
    );
    assert_eq!(
        readout_style.as_style().add_modifier,
        Modifier::BOLD | Modifier::ITALIC
    );
    assert!(
        warning
            .ranges()
            .iter()
            .any(|range| range.source() == DrawerProjectionSource::MeterPadding)
    );
    assert!(
        warning
            .ranges()
            .iter()
            .any(|range| range.source() == DrawerProjectionSource::MeterTrack)
    );
    assert!(
        warning
            .ranges()
            .iter()
            .any(|range| range.source() == DrawerProjectionSource::MeterMark)
    );

    let exhausted = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &LINES[6],
        PaneChrome::Drawer,
    )
    .unwrap();
    let exhausted_readout = exhausted
        .ranges()
        .iter()
        .find(|range| range.source() == DrawerProjectionSource::MeterReadout)
        .unwrap();
    assert_eq!(
        exhausted_readout.style().unwrap().as_style().bg,
        Some(Color::Rgb(0xf0, 0x5d, 0x5d))
    );
    assert!(
        exhausted
            .ranges()
            .iter()
            .any(|range| range.source() == DrawerProjectionSource::MeterNote)
    );

    let status = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &LINES[7],
        PaneChrome::Drawer,
    )
    .unwrap();
    assert_eq!(status.text(), "state: Warning · stale · ready");
    assert!(status.ranges().iter().any(|range| {
        range.source() == DrawerProjectionSource::Span(1)
            && range.style().unwrap().as_style().fg == Some(Color::Rgb(0xf5, 0x9e, 0x09))
    }));

    let unicode = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &LINES[8],
        PaneChrome::Drawer,
    )
    .unwrap();
    assert_eq!(unicode.text(), "界 e\u{301} 👩‍💻 literal tail");
    assert!(
        unicode
            .ranges()
            .iter()
            .any(|range| range.source() == DrawerProjectionSource::Text)
    );

    let blank = DrawerLine::Blank;
    let empty = project_accounts_drawer_line(
        &Theme::junie(),
        layout,
        layout.x,
        &blank,
        PaneChrome::Drawer,
    )
    .unwrap();
    assert_eq!(empty.text(), "");
    assert!(empty.ranges().is_empty());
}

#[test]
fn projection_resolves_ordered_ascii_overpaint_to_the_later_source() {
    let long = "0123456789012345678901234567890123456789";
    let line = DrawerLine::Meter(DrawerMeter {
        label: "overlap",
        readout: long,
        fill_cols: 0,
        track_cols: 0,
        tone: DrawerTone::Warning,
        mark: "!",
        mark_bold: true,
        note: "tail",
    });
    let scene_source = draw_scene(&line, Theme::junie(), ColorLevel::TrueColor, false);
    let scene_projected = draw_scene(&line, Theme::junie(), ColorLevel::TrueColor, true);
    for y in 0..scene_source.area().height {
        for x in 0..scene_source.area().width {
            let pos = Position::new(x, y);
            assert_eq!(
                format!("{:?}", scene_source.buffer().cell(pos)),
                format!("{:?}", scene_projected.buffer().cell(pos)),
                "overpaint mismatch at {pos:?}"
            );
        }
    }
    let layout = row_layout();
    let projection =
        project_accounts_drawer_line(&Theme::junie(), layout, layout.x, &line, PaneChrome::Drawer)
            .unwrap();
    let mark = projection
        .ranges()
        .iter()
        .find(|range| range.source() == DrawerProjectionSource::MeterMark)
        .unwrap();
    let note_ranges: Vec<_> = projection
        .ranges()
        .iter()
        .filter(|range| range.source() == DrawerProjectionSource::MeterNote)
        .collect();
    assert_eq!(mark.columns(), 56..57);
    assert_eq!(&projection.text()[mark.bytes()], "!");
    assert_eq!(note_ranges.first().unwrap().columns().start, 60);
    assert_eq!(note_ranges.last().unwrap().columns().end, 64);
    assert_eq!(
        note_ranges
            .iter()
            .map(|range| &projection.text()[range.bytes()])
            .collect::<String>(),
        "tail"
    );
}

#[test]
fn projection_reports_unrepresentable_wide_overlaps_and_cross_emission_joins() {
    let layout = row_layout();
    let partial_wide = DrawerLine::Field {
        label: "1234567890123456789界",
        value: "value",
        ink: DrawerInk::Plain,
    };
    assert!(matches!(
        project_accounts_drawer_line(
            &Theme::junie(),
            layout,
            layout.x,
            &partial_wide,
            PaneChrome::Drawer,
        ),
        Err(DrawerProjectionError::PartialGraphemeOverlap)
    ));

    let flag_spans = [
        DetailSpan {
            text: "🇺",
            ink: DetailInk::Plain,
        },
        DetailSpan {
            text: "🇸",
            ink: DetailInk::Danger,
        },
    ];
    let joined = DrawerLine::Spans(&DetailLine { spans: &flag_spans });
    assert!(matches!(
        project_accounts_drawer_line(
            &Theme::junie(),
            layout,
            layout.x,
            &joined,
            PaneChrome::Drawer,
        ),
        Err(DrawerProjectionError::GraphemeJoinAcrossEmissions)
    ));

    let oversized_meta = DrawerLine::Title {
        title: "row",
        meta: "metadata wider than the row",
    };
    assert!(matches!(
        project_accounts_drawer_line(
            &Theme::junie(),
            Rect::new(layout.x, layout.y, 12, 1),
            layout.x,
            &oversized_meta,
            PaneChrome::Drawer,
        ),
        Err(DrawerProjectionError::BeforeOrigin)
    ));

    let isolated_combining = DrawerLine::Text("\u{301}");
    assert!(matches!(
        project_accounts_drawer_line(
            &Theme::junie(),
            layout,
            layout.x,
            &isolated_combining,
            PaneChrome::Drawer,
        ),
        Err(DrawerProjectionError::UnrenderableGrapheme)
    ));
}

#[test]
fn projection_preserves_saturating_right_anchor_coordinates() {
    let origin = u16::MAX - 10;
    let layout = Rect::new(origin, 0, 20, 1);
    let line = DrawerLine::Title {
        title: "T",
        meta: "A",
    };
    let projection =
        project_accounts_drawer_line(&Theme::junie(), layout, origin, &line, PaneChrome::Drawer)
            .unwrap();
    assert_eq!(projection.origin_x(), origin);
    assert_eq!(projection.text(), " ▎T   A");
    let meta = projection
        .ranges()
        .iter()
        .find(|range| range.source() == DrawerProjectionSource::TitleMeta)
        .unwrap();
    assert_eq!(meta.columns(), 6..7);
    assert_eq!(&projection.text()[meta.bytes()], "A");
}
