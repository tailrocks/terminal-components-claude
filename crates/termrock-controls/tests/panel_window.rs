use std::cell::Cell;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};
use termrock_controls::{Panel, PanelKind, PanelWindowError};
use termrock_core::id::{Id, Part, PartRef};
use termrock_runtime::author::VerticalWindow;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Role, StylePatch, Surface, Theme};

const ID: Id = Id::root("panel.window.contract");
const SCREEN: Rect = Rect::new(0, 0, 42, 18);
const LOGICAL_AREA: Rect = Rect::new(4, 2, 29, 9);
const BODY_ROWS: [&str; 8] = [
    "Body row 0: wide 界",
    "Body row 1: combining e\u{301}",
    "Body row 2: unchanged source",
    "Body row 3: middle row",
    "Body row 4: clipped logical body",
    "Body row 5: row after middle",
    "Body row 6: near end",
    "Body row 7: final source row",
];
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn sentinel_patch() -> StylePatch {
    StylePatch::new()
        .set_fg(Role::Custom(Color::Rgb(19, 29, 39)))
        .set_bg(Role::Custom(Color::Rgb(49, 59, 69)))
        .add(Modifier::UNDERLINED)
}

fn panel_patches() -> [(Part, StylePatch); 5] {
    [
        (
            Part::CONTAINER,
            StylePatch::new()
                .set_bg(Role::Custom(Color::Rgb(31, 41, 51)))
                .add(Modifier::DIM),
        ),
        (
            Part::BORDER,
            StylePatch::new()
                .set_fg(Role::Custom(Color::Rgb(71, 81, 91)))
                .add(Modifier::BOLD),
        ),
        (
            Part::GUTTER,
            StylePatch::new()
                .set_fg(Role::Custom(Color::Rgb(101, 111, 121)))
                .add(Modifier::ITALIC),
        ),
        (
            Part::TITLE,
            StylePatch::new().set_fg(Role::Custom(Color::Rgb(131, 141, 151))),
        ),
        (
            Part::DETAIL,
            StylePatch::new()
                .set_bg(Role::Custom(Color::Rgb(161, 171, 181)))
                .add(Modifier::UNDERLINED),
        ),
    ]
}

fn panel<'a>(kind: PanelKind, patches: &'a [(Part, StylePatch)]) -> Panel<'a> {
    Panel::new(ID)
        .kind(kind)
        .title("Panel 界")
        .meta("Source e\u{301}")
        .badge("LIVE")
        .patch_part(patches)
}

fn render_sentinel(theme: Theme, level: ColorLevel) -> Scene {
    let mut scene = Scene::new(
        "panel-window-sentinel",
        theme,
        level,
        SCREEN.width,
        SCREEN.height,
    );
    let patch = sentinel_patch();
    scene.draw(|ui, area| ui.fill(area, ui.paint_patch(&patch)));
    scene
}

fn paint_full_body(ui: &mut termrock_runtime::Ui<'_>, inner: Rect) {
    for (index, row) in inner.rows().enumerate() {
        let text = BODY_ROWS.get(index).copied().unwrap_or("extra body row");
        ui.paint_str(row, text, ui.surface_style());
    }
}

fn paint_window_body(ui: &mut termrock_runtime::Ui<'_>, window: Option<VerticalWindow>) {
    let Some(window) = window else {
        return;
    };
    let visible = window.visible_body();
    for body_row in window.visible_source_range() {
        let Some(y) = window.physical_row_for_source(body_row) else {
            continue;
        };
        let text = BODY_ROWS.get(body_row).copied().unwrap_or("extra body row");
        ui.paint_str(
            Rect::new(visible.x, y, visible.width, 1),
            text,
            ui.surface_style(),
        );
    }
}

fn render_full(theme: Theme, level: ColorLevel, kind: PanelKind) -> Scene {
    let mut scene = Scene::new(
        "panel-window-reference",
        theme,
        level,
        SCREEN.width,
        SCREEN.height,
    );
    let sentinel = sentinel_patch();
    let patches = panel_patches();
    scene.draw(|ui, screen| {
        ui.fill(screen, ui.paint_patch(&sentinel));
        panel(kind, &patches).draw(ui, LOGICAL_AREA, paint_full_body);
    });
    scene
}

fn render_window(
    theme: Theme,
    level: ColorLevel,
    kind: PanelKind,
    window: VerticalWindow,
    seen_body_windows: &Cell<usize>,
    seen_surface: &Cell<Option<Surface>>,
) -> Scene {
    let mut scene = Scene::new("panel-window", theme, level, SCREEN.width, SCREEN.height);
    let sentinel = sentinel_patch();
    let patches = panel_patches();
    scene.draw(|ui, screen| {
        ui.fill(screen, ui.paint_patch(&sentinel));
        panel(kind, &patches)
            .draw_window(ui, LOGICAL_AREA, window, |ui, body| {
                seen_body_windows.set(seen_body_windows.get() + 1);
                seen_surface.set(Some(ui.surface()));
                paint_window_body(ui, body);
            })
            .expect("valid partial panel window");
    });
    scene
}

fn assert_source_crop(
    reference: &Scene,
    actual: &Scene,
    sentinel: &Scene,
    logical_area: Rect,
    window: VerticalWindow,
) {
    let visible = window.visible_body();
    for y in 0..SCREEN.height {
        for x in 0..SCREEN.width {
            if visible.contains(Position::new(x, y)) {
                let source_row = window
                    .source_row_for_physical(y)
                    .expect("visible physical row maps to panel source row");
                let source_y = logical_area
                    .y
                    .saturating_add(u16::try_from(source_row).unwrap_or(u16::MAX));
                assert_eq!(
                    actual.buffer().cell((x, y)),
                    reference.buffer().cell((x, source_y)),
                    "window differs from full panel source row {source_row} at ({x},{y})"
                );
            } else {
                assert_eq!(
                    actual.buffer().cell((x, y)),
                    sentinel.buffer().cell((x, y)),
                    "window painted outside its visible body at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn partial_windows_match_full_panel_source_rows_and_keep_body_surface() {
    let windows = [
        VerticalWindow::new(Rect::new(4, 12, 29, 2), 9, 0).expect("top border and title"),
        VerticalWindow::new(Rect::new(4, 7, 29, 3), 9, 3).expect("middle rows"),
        VerticalWindow::new(Rect::new(4, 13, 29, 2), 9, 7).expect("bottom rows"),
    ];

    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            let sentinel = render_sentinel(theme.clone(), level);
            for kind in [PanelKind::Card, PanelKind::Framed] {
                let reference = render_full(theme.clone(), level, kind);
                for window in windows {
                    let calls = Cell::new(0);
                    let surface = Cell::new(None);
                    let actual =
                        render_window(theme.clone(), level, kind, window, &calls, &surface);
                    assert_eq!(calls.get(), 1, "body callback must run exactly once");
                    assert_eq!(
                        surface.get(),
                        Some(if kind == PanelKind::Card {
                            Surface::Surface
                        } else {
                            Surface::Canvas
                        })
                    );
                    assert_source_crop(&reference, &actual, &sentinel, LOGICAL_AREA, window);
                }
            }
        }
    }
}

#[test]
fn full_window_is_cell_identical_to_legacy_draw() {
    let patches = panel_patches();
    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            for kind in [PanelKind::Card, PanelKind::Framed] {
                let mut legacy = Scene::new(
                    "panel-legacy",
                    theme.clone(),
                    level,
                    SCREEN.width,
                    SCREEN.height,
                );
                let mut windowed = Scene::new(
                    "panel-full-window",
                    theme.clone(),
                    level,
                    SCREEN.width,
                    SCREEN.height,
                );
                legacy.draw(|ui, _| {
                    panel(kind, &patches).draw(ui, LOGICAL_AREA, paint_full_body);
                });
                windowed.draw(|ui, _| {
                    panel(kind, &patches)
                        .draw_window(
                            ui,
                            LOGICAL_AREA,
                            VerticalWindow::full(LOGICAL_AREA),
                            |ui, window| {
                                assert_eq!(
                                    window,
                                    Some(VerticalWindow::full(
                                        panel(kind, &patches).inner(ui, LOGICAL_AREA)
                                    ))
                                );
                                paint_window_body(ui, window);
                            },
                        )
                        .expect("full window has no unsupported slot");
                });
                for y in 0..SCREEN.height {
                    for x in 0..SCREEN.width {
                        assert_eq!(
                            windowed.buffer().cell((x, y)),
                            legacy.buffer().cell((x, y)),
                            "full draw differs at ({x},{y}) for {kind:?}/{level:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn partial_legacy_slots_and_invalid_windows_fail_before_effects() {
    let area = Rect::new(4, 2, 29, 9);
    let window = VerticalWindow::new(Rect::new(4, 10, 29, 2), 9, 0).expect("valid window");
    let slot_calls = Cell::new(0);
    let slot = |_: &mut termrock_runtime::Ui<'_>, _: Rect| {
        slot_calls.set(slot_calls.get() + 1);
    };

    let render_rejected = |panel: Panel<'_>, window| {
        let mut scene = Scene::new(
            "panel-window-rejected",
            Theme::junie(),
            ColorLevel::TrueColor,
            SCREEN.width,
            SCREEN.height,
        );
        let sentinel = sentinel_patch();
        let body_calls = Cell::new(0);
        scene.draw(|ui, screen| {
            ui.fill(screen, ui.paint_patch(&sentinel));
            let result = panel.draw_window(ui, area, window, |_, _| {
                body_calls.set(body_calls.get() + 1);
            });
            assert_eq!(
                result,
                Err(PanelWindowError::UnsupportedLegacySlot(Part::BADGE))
            );
        });
        assert_eq!(body_calls.get(), 0, "rejected draw must not call the body");
        assert_eq!(slot_calls.get(), 0, "preflight must not run the slot");
        assert_eq!(
            scene
                .registry()
                .expect("draw has a registry")
                .area_of_part(ID, PartRef::of(Part::CONTAINER)),
            None,
            "preflight must not register panel decoration"
        );
        let sentinel_scene = render_sentinel(Theme::junie(), ColorLevel::TrueColor);
        assert_eq!(
            scene.buffer(),
            sentinel_scene.buffer(),
            "preflight painted cells"
        );
    };

    render_rejected(Panel::new(ID).slot(Part::BADGE, &slot), window);

    let invalid_windows = [
        VerticalWindow::new(Rect::new(5, 10, 29, 2), 9, 0)
            .expect("structurally valid window with mismatched x"),
        VerticalWindow::new(Rect::new(4, 10, 28, 2), 9, 0)
            .expect("structurally valid window with mismatched width"),
        VerticalWindow::new(Rect::new(4, 10, 29, 2), 8, 0)
            .expect("structurally valid window with mismatched logical row count"),
        VerticalWindow::full(Rect::new(4, 10, 29, 2)),
    ];
    for (index, invalid_window) in invalid_windows.into_iter().enumerate() {
        let mut invalid = Scene::new(
            "panel-invalid-window",
            Theme::junie(),
            ColorLevel::TrueColor,
            SCREEN.width,
            SCREEN.height,
        );
        let sentinel = sentinel_patch();
        let body_calls = Cell::new(0);
        invalid.draw(|ui, screen| {
            ui.fill(screen, ui.paint_patch(&sentinel));
            let error = Panel::new(ID).draw_window(ui, area, invalid_window, |_, _| {
                body_calls.set(body_calls.get() + 1);
            });
            assert_eq!(error, Err(PanelWindowError::InvalidWindow));
        });
        assert_eq!(body_calls.get(), 0, "invalid case {index} called the body");
        assert_eq!(
            invalid
                .registry()
                .expect("draw has a registry")
                .area_of_part(ID, PartRef::of(Part::CONTAINER)),
            None,
            "invalid case {index} registered panel decoration"
        );
        let sentinel_scene = render_sentinel(Theme::junie(), ColorLevel::TrueColor);
        assert_eq!(invalid.buffer(), sentinel_scene.buffer());
    }
}

#[test]
fn full_window_keeps_legacy_slots_and_calls_body_once() {
    let slot_calls = Cell::new(0);
    let body_calls = Cell::new(0);
    let slot = |_: &mut termrock_runtime::Ui<'_>, _: Rect| {
        slot_calls.set(slot_calls.get() + 1);
    };
    let mut scene = Scene::new(
        "panel-window-full-slot",
        Theme::junie(),
        ColorLevel::TrueColor,
        SCREEN.width,
        SCREEN.height,
    );
    scene.draw(|ui, _| {
        Panel::new(ID)
            .title("Full")
            .badge("EDIT")
            .slot(Part::BADGE, &slot)
            .draw_window(
                ui,
                LOGICAL_AREA,
                VerticalWindow::full(LOGICAL_AREA),
                |ui, window| {
                    body_calls.set(body_calls.get() + 1);
                    assert!(window.is_some());
                    ui.fill(
                        window.expect("nonempty panel body").visible_body(),
                        ui.surface_style(),
                    );
                },
            )
            .expect("full windows use legacy drawing");
    });
    assert_eq!(
        slot_calls.get(),
        1,
        "full draw must preserve the badge slot"
    );
    assert_eq!(body_calls.get(), 1, "legacy body must run exactly once");
}

#[test]
fn overflowing_nonempty_full_window_fails_before_effects() {
    let area = Rect {
        x: 4,
        y: u16::MAX,
        width: 8,
        height: 2,
    };
    let window = VerticalWindow::full(area);
    let slot_calls = Cell::new(0);
    let body_calls = Cell::new(0);
    let slot = |_: &mut termrock_runtime::Ui<'_>, _: Rect| {
        slot_calls.set(slot_calls.get() + 1);
    };
    let mut scene = Scene::new(
        "panel-window-overflow",
        Theme::junie(),
        ColorLevel::TrueColor,
        SCREEN.width,
        SCREEN.height,
    );
    let sentinel = sentinel_patch();
    scene.draw(|ui, screen| {
        ui.fill(screen, ui.paint_patch(&sentinel));
        let result = Panel::new(ID)
            .title("Overflow")
            .slot(Part::GUTTER, &slot)
            .draw_window(ui, area, window, |_, _| {
                body_calls.set(body_calls.get() + 1);
            });
        assert_eq!(result, Err(PanelWindowError::InvalidWindow));
    });

    assert_eq!(body_calls.get(), 0, "overflow preflight must skip the body");
    assert_eq!(
        slot_calls.get(),
        0,
        "overflow preflight must skip legacy slots"
    );
    assert_eq!(
        scene
            .registry()
            .expect("draw has a registry")
            .area_of_part(ID, PartRef::of(Part::CONTAINER)),
        None,
        "overflow preflight must not register panel decoration"
    );
    let sentinel_scene = render_sentinel(Theme::junie(), ColorLevel::TrueColor);
    assert_eq!(scene.buffer(), sentinel_scene.buffer());
}

#[test]
fn zero_sized_full_window_keeps_legacy_body_behavior() {
    let area = Rect::new(4, 3, 0, 2);
    let slot_calls = Cell::new(0);
    let body_calls = Cell::new(0);
    let slot = |_: &mut termrock_runtime::Ui<'_>, _: Rect| {
        slot_calls.set(slot_calls.get() + 1);
    };
    let mut scene = Scene::new(
        "panel-window-zero-width",
        Theme::junie(),
        ColorLevel::TrueColor,
        SCREEN.width,
        SCREEN.height,
    );
    scene.draw(|ui, _| {
        let result = Panel::new(ID)
            .title("Zero")
            .slot(Part::GUTTER, &slot)
            .draw_window(ui, area, VerticalWindow::full(area), |_, window| {
                body_calls.set(body_calls.get() + 1);
                assert_eq!(window, None, "an empty body has no visible row window");
            });
        assert_eq!(result, Ok(()));
    });
    assert_eq!(body_calls.get(), 1, "legacy draw invokes its body once");
    assert_eq!(slot_calls.get(), 0, "empty geometry paints no gutter slot");
}
