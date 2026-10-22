use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock::author::VerticalWindow;
use termrock::feedback::Notice;
use termrock::theme::{ColorLevel, FgStep, Role, StylePatch, Theme};
use termrock_test_support::Scene;

const SCREEN: Rect = Rect::new(0, 0, 38, 18);
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn sentinel() -> StylePatch {
    StylePatch::new()
        .set_fg(Role::Custom(Color::Rgb(19, 29, 39)))
        .set_bg(Role::Custom(Color::Rgb(49, 59, 69)))
        .add(Modifier::UNDERLINED)
}

fn notice(mark_cut: bool) -> Notice<'static> {
    Notice::warning(
        "Wide 界 graphemes and e\u{301} words wrap across rows\nsecond paragraph stays logical",
    )
    .detail("Detail 一 uses its own wrapped row group")
    .hint("Hint 界 remains on the original final row")
    .message_lines(6)
    .max_lines(3)
    .mark_cut(mark_cut)
    .marker("界")
    .marker_patch(
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(201, 31, 61)))
            .set_bg(Role::Custom(Color::Rgb(41, 51, 61)))
            .add(Modifier::BOLD),
    )
    .message_patch(
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(81, 91, 101)))
            .add(Modifier::ITALIC),
    )
    .detail_tone(FgStep::Muted)
    .detail_patch(
        StylePatch::new()
            .set_bg(Role::Custom(Color::Rgb(11, 121, 131)))
            .add(Modifier::DIM),
    )
    .hint_patch(
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(71, 81, 91)))
            .add(Modifier::UNDERLINED),
    )
}

fn render_reference(
    theme: Theme,
    level: ColorLevel,
    notice: Notice<'_>,
    logical_area: Rect,
) -> Scene {
    let mut scene = Scene::new(
        "notice-window-reference",
        theme,
        level,
        SCREEN.width,
        SCREEN.height,
    );
    let sentinel = sentinel();
    scene.draw(|ui, frame| {
        ui.fill(frame, ui.paint_patch(&sentinel));
        notice.draw(ui, logical_area);
    });
    scene
}

fn render_window(
    theme: Theme,
    level: ColorLevel,
    notice: Notice<'_>,
    window: VerticalWindow,
) -> Scene {
    let mut scene = Scene::new("notice-window", theme, level, SCREEN.width, SCREEN.height);
    let sentinel = sentinel();
    scene.draw(|ui, frame| {
        ui.fill(frame, ui.paint_patch(&sentinel));
        notice.draw_window(ui, window);
    });
    scene
}

fn render_sentinel(theme: Theme, level: ColorLevel) -> Scene {
    let mut scene = Scene::new(
        "notice-window-sentinel",
        theme,
        level,
        SCREEN.width,
        SCREEN.height,
    );
    let sentinel = sentinel();
    scene.draw(|ui, frame| ui.fill(frame, ui.paint_patch(&sentinel)));
    scene
}

fn assert_window_is_source_crop(
    reference: &Scene,
    actual: &Scene,
    sentinel: &Scene,
    logical_area: Rect,
    window: VerticalWindow,
) {
    let visible = window.visible_body();
    for y in 0..SCREEN.height {
        for x in 0..SCREEN.width {
            let expected = if visible.contains(Position::new(x, y)) {
                let source_row = window
                    .source_row_for_physical(y)
                    .expect("every visible row maps to a source row");
                let source_y = logical_area
                    .y
                    .saturating_add(u16::try_from(source_row).unwrap_or(u16::MAX));
                reference.buffer().cell((x, source_y))
            } else {
                None
            };
            if let Some(expected) = expected {
                assert_eq!(
                    actual.buffer().cell((x, y)),
                    Some(expected),
                    "visible crop differs at ({x},{y})"
                );
            } else {
                assert_eq!(
                    actual.buffer().cell((x, y)),
                    sentinel.buffer().cell((x, y)),
                    "paint escaped window at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn partial_windows_equal_full_logical_rows_without_new_edge_cuts() {
    let logical_area = Rect::new(4, 2, 19, 7);
    let windows = [
        VerticalWindow::new(Rect::new(4, 10, 19, 2), 7, 0).expect("top window"),
        VerticalWindow::new(Rect::new(4, 6, 19, 2), 7, 2).expect("middle window"),
        VerticalWindow::new(Rect::new(4, 12, 19, 2), 7, 5).expect("cap-edge window"),
    ];

    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            for mark_cut in [true, false] {
                let notice = notice(mark_cut);
                let reference = render_reference(theme.clone(), level, notice, logical_area);
                let sentinel = render_sentinel(theme.clone(), level);
                for window in windows {
                    let actual = render_window(theme.clone(), level, notice, window);
                    assert_window_is_source_crop(
                        &reference,
                        &actual,
                        &sentinel,
                        logical_area,
                        window,
                    );
                }
            }
        }
    }
}

#[test]
fn full_window_matches_ordinary_draw_and_preserves_height_cut_metadata() {
    let area = Rect::new(3, 3, 23, 5);
    for level in LEVELS {
        let notice = notice(true);
        let reference = render_reference(Theme::junie(), level, notice, area);
        let mut full = Scene::new(
            "notice-window-full",
            Theme::junie(),
            level,
            SCREEN.width,
            SCREEN.height,
        );
        let sentinel = sentinel();
        full.draw(|ui, frame| {
            ui.fill(frame, ui.paint_patch(&sentinel));
            notice.draw_window(ui, VerticalWindow::full(area));
        });
        for y in 0..SCREEN.height {
            for x in 0..SCREEN.width {
                assert_eq!(
                    full.buffer().cell((x, y)),
                    reference.buffer().cell((x, y)),
                    "full window diverged at ({x},{y}) for {level:?}"
                );
            }
        }
    }
}
