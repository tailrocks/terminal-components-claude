use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};
use termrock::feedback::{Notice, NoticeKind};
use termrock::layout::Constraints;
use termrock::theme::{ColorLevel, FgStep, GlyphRole, Role, StylePatch, Theme};
use termrock_test_support::Scene;

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn cell(scene: &Scene, x: u16, y: u16) -> &ratatui_core::buffer::Cell {
    scene
        .buffer()
        .cell((x, y))
        .expect("the requested cell is inside the scene")
}

fn draw_notice(
    notice: Notice<'_>,
    theme: Theme,
    level: ColorLevel,
    width: u16,
    height: u16,
) -> Scene {
    let mut scene = Scene::new("notice-contract", theme, level, width, height);
    scene.draw(|ui, area| notice.draw(ui, area));
    scene
}

fn expected_marker(theme: &Theme, kind: NoticeKind) -> &'static str {
    if kind == NoticeKind::ERROR {
        theme.design.glyphs.get(GlyphRole::Error)
    } else if kind == NoticeKind::WARNING {
        theme.design.glyphs.get(GlyphRole::WarningMark)
    } else if kind == NoticeKind::INFO {
        "i"
    } else {
        assert_eq!(kind, NoticeKind::SUCCESS);
        "✓"
    }
}

#[test]
fn four_kinds_keep_distinct_marks_and_semantic_ink_across_builtin_themes_and_tiers() {
    let kinds = [
        (NoticeKind::ERROR, Notice::error("message")),
        (NoticeKind::WARNING, Notice::warning("message")),
        (NoticeKind::INFO, Notice::info("message")),
        (NoticeKind::SUCCESS, Notice::success("message")),
    ];

    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            let projected = theme.downgrade(level);
            let mut marks = Vec::new();
            for (kind, notice) in kinds {
                let scene = draw_notice(notice, theme.clone(), level, 24, 1);
                let marker = cell(&scene, 0, 0);
                let message = cell(&scene, 2, 0);
                let expected_marker = expected_marker(&projected, kind);
                marks.push(marker.symbol().to_owned());
                assert_eq!(marker.symbol(), expected_marker, "{kind:?} {level:?}");

                let (marker_fg, message_fg) = if kind == NoticeKind::ERROR {
                    (projected.color.danger, projected.color.danger)
                } else if kind == NoticeKind::WARNING {
                    (projected.color.warning, projected.color.warning)
                } else if kind == NoticeKind::INFO {
                    (projected.color.info, projected.color.fg[0])
                } else {
                    assert_eq!(kind, NoticeKind::SUCCESS);
                    (projected.color.success, projected.color.fg[0])
                };
                assert_eq!(marker.fg, marker_fg, "marker {kind:?} {level:?}");
                assert_eq!(message.fg, message_fg, "message {kind:?} {level:?}");
                assert_eq!(
                    marker.modifier.contains(Modifier::BOLD),
                    kind == NoticeKind::ERROR,
                    "only the error marker is bold: {kind:?} {level:?}"
                );
                assert!(!message.modifier.contains(Modifier::BOLD));
            }
            let channels = draw_notice(
                Notice::error("message").detail("detail").hint("hint"),
                theme.clone(),
                level,
                24,
                3,
            );
            assert_eq!(cell(&channels, 2, 1).fg, projected.color.fg[1]);
            assert_eq!(cell(&channels, 2, 2).fg, projected.color.fg[2]);
            marks.sort();
            marks.dedup();
            assert_eq!(marks.len(), 4, "{theme:?} {level:?}");
        }
    }
}

#[test]
fn error_notice_uses_three_rows_and_measurement_matches_its_width() {
    let notice = Notice::error("Authentication failed")
        .detail("The replica rejected the credentials")
        .hint("Press Enter to retry");
    let mut scene = Scene::new(
        "notice-three-rows",
        Theme::junie(),
        ColorLevel::TrueColor,
        50,
        3,
    );
    let mut measured = 0;
    let mut measured_size = (0, 0);
    scene.draw(|ui, area| {
        measured = notice.height_for_width(ui, area.width);
        let size = notice.measure(ui, Constraints::tight(area.width, area.height));
        measured_size = size.preferred;
        notice.draw(ui, area);
    });

    assert_eq!(measured, 3);
    assert_eq!(measured_size, (50, 3));
    let rendered = scene.text();
    let rows = rendered
        .split('\n')
        .take(3)
        .map(str::trim_end)
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        [
            "! Authentication failed",
            "  The replica rejected the credentials",
            "  Press Enter to retry",
        ]
    );
}

#[test]
fn empty_multiline_combining_wide_and_joined_graphemes_keep_text_layout() {
    let message = "alpha 界e\u{301} 👩\u{200D}💻\n\nomega";
    let notice = Notice::new(message).message_lines(4);
    let mut scene = Scene::new(
        "notice-unicode",
        Theme::junie(),
        ColorLevel::TrueColor,
        12,
        5,
    );
    let mut measured = 0;
    scene.draw(|ui, area| {
        measured = notice.height_for_width(ui, area.width);
        notice.draw(ui, area);
    });
    assert_eq!(measured, 4);
    let rendered = scene.text();
    let rows = rendered.split('\n').map(str::trim_end).collect::<Vec<_>>();
    assert_eq!(rows[0], "! alpha 界e\u{301}");
    assert_eq!(rows[1], "  👩\u{200D}💻");
    assert_eq!(rows[2], "");
    assert_eq!(rows[3], "  omega");

    let empty = draw_notice(
        Notice::new(""),
        Theme::junie(),
        ColorLevel::TrueColor,
        10,
        1,
    );
    assert_eq!(cell(&empty, 0, 0).symbol(), "!");

    let blank_lines = Notice::new("a\n\n").message_lines(4);
    let mut trailing = Scene::new(
        "notice-trailing-lines",
        Theme::junie(),
        ColorLevel::TrueColor,
        10,
        3,
    );
    let mut trailing_height = 0;
    trailing.draw(|ui, area| {
        trailing_height = blank_lines.height_for_width(ui, area.width);
        blank_lines.draw(ui, area);
    });
    assert_eq!(trailing_height, 3);
    let rendered = trailing.text();
    let rows = rendered
        .split('\n')
        .take(3)
        .map(str::trim_end)
        .collect::<Vec<_>>();
    assert_eq!(rows, ["! a", "", ""]);

    let narrow_wide = draw_notice(
        Notice::new("界").message_lines(1),
        Theme::junie(),
        ColorLevel::TrueColor,
        3,
        1,
    );
    assert_eq!(cell(&narrow_wide, 2, 0).symbol(), "…");

    let wide_then_text = draw_notice(
        Notice::new("界x").message_lines(2),
        Theme::junie(),
        ColorLevel::TrueColor,
        3,
        2,
    );
    assert_eq!(cell(&wide_then_text, 2, 0).symbol(), "…");
    assert_eq!(cell(&wide_then_text, 2, 1).symbol(), "x");
}

#[test]
fn zero_through_two_column_widths_preserve_height_and_never_paint_outside() {
    let notice = Notice::new("message").detail("detail").hint("hint");
    for width in 0..=2 {
        let mut scene = Scene::new("notice-narrow", Theme::junie(), ColorLevel::TrueColor, 8, 5);
        let area = Rect::new(2, 1, width, 3);
        let mut measured = 0;
        let mut constrained = 0;
        scene.draw(|ui, _| {
            measured = notice.height_for_width(ui, width);
            constrained = notice.measure(ui, Constraints::tight(width, 3)).preferred.1;
            notice.draw(ui, area);
        });
        assert_eq!(measured, 3, "width={width}");
        assert_eq!(constrained, 3, "width={width}");
        if width > 0 {
            assert_eq!(cell(&scene, 2, 1).symbol(), "!", "width={width}");
        }
        assert_eq!(cell(&scene, 1, 1).symbol(), " ");
        assert_eq!(cell(&scene, 4, 1).symbol(), " ");
    }
}

#[test]
fn offset_area_clips_paint_and_marks_the_last_visible_cut_row() {
    let notice = Notice::new("first\nsecond\nthird").message_lines(20);
    let area = Rect::new(3, 2, 10, 2);
    let patch = StylePatch::new()
        .set_fg(Role::Custom(Color::Rgb(19, 29, 39)))
        .set_bg(Role::Custom(Color::Rgb(49, 59, 69)))
        .add(Modifier::UNDERLINED);

    let mut expected = Scene::new(
        "notice-sentinel",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        7,
    );
    expected.draw(|ui, frame| ui.fill(frame, ui.paint_patch(&patch)));

    let mut actual = Scene::new(
        "notice-clipped",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        7,
    );
    actual.draw(|ui, frame| {
        ui.fill(frame, ui.paint_patch(&patch));
        notice.draw(ui, area);
    });

    for y in 0..7 {
        for x in 0..16 {
            let inside = x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
            if !inside {
                assert_eq!(
                    actual.buffer().cell((x, y)),
                    expected.buffer().cell((x, y)),
                    "outside cell changed at ({x},{y})"
                );
            }
        }
    }
    assert_eq!(cell(&actual, 3, 2).symbol(), "!");
    assert!(
        actual
            .text()
            .split('\n')
            .nth(3)
            .unwrap_or("")
            .contains("second…")
    );
}

#[test]
fn caps_hint_truncation_and_cut_marker_option_share_one_height_contract() {
    let marked = Notice::new("first\nsecond")
        .message_lines(0)
        .detail("suppressed\ntext")
        .max_lines(0)
        .hint("abcdefghi")
        .mark_cut(true);
    let plain = marked.mark_cut(false);

    let marked_scene = draw_notice(marked, Theme::junie(), ColorLevel::TrueColor, 6, 2);
    let plain_scene = draw_notice(plain, Theme::junie(), ColorLevel::TrueColor, 6, 2);
    let mut marked_height = 0;
    let mut plain_height = 0;
    let mut measured_scene = Scene::new(
        "notice-cut-measurement",
        Theme::junie(),
        ColorLevel::TrueColor,
        6,
        2,
    );
    measured_scene.draw(|ui, _| {
        marked_height = marked.height_for_width(ui, 6);
        plain_height = plain.height_for_width(ui, 6);
    });
    assert_eq!(marked_height, plain_height);
    assert_eq!(marked_scene.text().split('\n').count(), 2);
    assert_eq!(plain_scene.text().split('\n').count(), 2);
    assert!(
        marked_scene.text().contains("fir…"),
        "{}",
        marked_scene.text()
    );
    assert!(
        marked_scene.text().contains("abc…"),
        "{}",
        marked_scene.text()
    );
    assert!(!plain_scene.text().contains('…'), "{}", plain_scene.text());
    assert!(!marked_scene.text().contains("suppressed"));

    let very_long = "x\n".repeat(65_540);
    let huge_caps = Notice::new(&very_long)
        .message_lines(usize::MAX)
        .detail("tail\nmore")
        .max_lines(usize::MAX);
    let mut scene = Scene::new(
        "notice-saturating-height",
        Theme::junie(),
        ColorLevel::TrueColor,
        8,
        1,
    );
    let mut measured = 0;
    scene.draw(|ui, _| measured = huge_caps.height_for_width(ui, 4));
    assert_eq!(measured, u16::MAX);
}

#[test]
fn local_style_channels_override_independently_and_clear_explicitly() {
    let marker_fg = Color::Rgb(170, 80, 20);
    let marker_bg = Color::Rgb(30, 80, 170);
    let message_fg = Color::Rgb(20, 170, 80);
    let notice = Notice::error("Failure")
        .marker("+")
        .marker_patch(
            StylePatch::new()
                .set_fg(Role::Custom(marker_fg))
                .set_bg(Role::Custom(marker_bg))
                .remove(Modifier::BOLD),
        )
        .message_patch(
            StylePatch::new()
                .set_fg(Role::Custom(message_fg))
                .add(Modifier::ITALIC),
        )
        .detail("Details")
        .hint("Try again");
    let scene = draw_notice(notice, Theme::junie(), ColorLevel::TrueColor, 30, 3);
    let marker = cell(&scene, 0, 0);
    let message = cell(&scene, 2, 0);
    assert_eq!(marker.symbol(), "+");
    assert_eq!(marker.fg, marker_fg);
    assert_eq!(marker.bg, marker_bg);
    assert!(!marker.modifier.contains(Modifier::BOLD));
    assert_eq!(message.fg, message_fg);
    assert!(message.modifier.contains(Modifier::ITALIC));
    assert_ne!(message.bg, marker_bg);

    let detail_tone = Notice::new("message")
        .detail("details")
        .detail_tone(FgStep::Faint)
        .hint("try")
        .hint_patch(StylePatch::new().set_fg(Role::Custom(Color::Rgb(80, 20, 170))));
    let scene = draw_notice(detail_tone, Theme::junie(), ColorLevel::TrueColor, 30, 3);
    assert_eq!(
        cell(&scene, 2, 1).fg,
        Theme::junie().color.fg[FgStep::Faint.index()]
    );
    assert_eq!(cell(&scene, 2, 2).fg, Color::Rgb(80, 20, 170));

    let cleared = Notice::new("message")
        .detail("details")
        .detail_patch(StylePatch::new().clear_fg());
    let scene = draw_notice(cleared, Theme::junie(), ColorLevel::TrueColor, 20, 2);
    assert_eq!(cell(&scene, 2, 1).fg, Color::Reset);
}
