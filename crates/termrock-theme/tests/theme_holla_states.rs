use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Style;
use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Response};
use termrock_layout::Track;
use termrock_runtime::{App, Cx, RowUi, Runtime, Ui};
use termrock_test_support::{Harness, Scene};
use termrock_text::Span;
use termrock_theme::{
    Align, ColorLevel, Family, FgStep, Modifier, PaintStyle, Part, Role, StateFlags, StylePatch,
    Surface, Theme, Variant,
};

const OWNER: Id = Id::root("theme.holla.rows");
const AREA: Rect = Rect::new(0, 0, 16, 1);

fn row(theme: Theme, level: ColorLevel, surface: Surface, flags: StateFlags, dim: bool) -> Scene {
    let mut scene = Scene::new("theme_holla_states", theme, level, 16, 1);
    scene.draw(|ui, _| {
        ui.with_surface(surface, |ui| {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                flags,
                ItemKey::index(0),
                AREA,
            );
            row.gutter();
            row.label("hello");
        });
        if dim {
            ui.dim_layer(AREA, 1);
        }
    });
    scene
}

fn bg(scene: &Scene) -> ratatui_core::style::Color {
    scene
        .buffer()
        .cell((15, 0))
        .map_or(ratatui_core::style::Color::Reset, |c| c.bg)
}

#[test]
fn every_hover_plane_uses_reference_tokens_in_all_color_modes() {
    use ratatui_core::style::Color;
    let surfaces = [
        Surface::Canvas,
        Surface::Surface,
        Surface::Elevated,
        Surface::Overlay,
        Surface::Popover,
        Surface::Field,
        Surface::FieldHover,
    ];
    let cases = [
        (
            ColorLevel::TrueColor,
            [
                0x18_18_1b, 0x27_27_2a, 0x27_27_2a, 0x3f_3f_46, 0x3f_3f_46, 0x2f_2f_34, 0x3f_3f_46,
            ]
            .map(Color::from_u32),
        ),
        (
            ColorLevel::Ansi256,
            [233, 235, 235, 237, 237, 237, 237].map(Color::Indexed),
        ),
        (
            ColorLevel::Ansi16,
            [
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::DarkGray,
                Color::DarkGray,
                Color::DarkGray,
                Color::DarkGray,
            ],
        ),
        (
            ColorLevel::Mono,
            [
                Color::Black,
                Color::Black,
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::Black,
                Color::DarkGray,
            ],
        ),
    ];
    for (level, expected) in cases {
        for (surface, expected) in surfaces.into_iter().zip(expected) {
            let scene = row(Theme::junie(), level, surface, StateFlags::HOVERED, false);
            assert_eq!(bg(&scene), expected, "{level:?} {surface:?}");
            assert!(
                scene
                    .buffer()
                    .cell((2, 0))
                    .is_some_and(|c| !c.modifier.contains(Modifier::BOLD))
            );
        }
    }
}

#[test]
fn selection_hover_and_press_have_reference_precedence() {
    use ratatui_core::style::Color;
    let selected_focus = StateFlags::SELECTED | StateFlags::FOCUSED;
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let (canvas, tint, hover, white) = match level {
            ColorLevel::TrueColor => (
                Color::Rgb(0, 0, 0),
                Color::Rgb(15, 46, 19),
                Color::Rgb(24, 24, 27),
                Color::Rgb(255, 255, 255),
            ),
            ColorLevel::Ansi256 => (
                Color::Indexed(16),
                Color::Indexed(233),
                Color::Indexed(233),
                Color::Indexed(231),
            ),
            ColorLevel::Ansi16 => (Color::Black, Color::DarkGray, Color::Black, Color::White),
            ColorLevel::Mono => (Color::Black, Color::Black, Color::Black, Color::White),
            _ => continue,
        };
        for (flags, expected_bg, expected_fg, bold) in [
            (StateFlags::empty(), canvas, white, false),
            (StateFlags::SELECTED, canvas, white, false),
            (StateFlags::FOCUSED, canvas, white, true),
            (selected_focus, tint, white, true),
            (selected_focus | StateFlags::HOVERED, hover, white, true),
            (selected_focus | StateFlags::PRESSED, white, canvas, true),
            (
                selected_focus | StateFlags::HOVERED | StateFlags::PRESSED,
                white,
                canvas,
                true,
            ),
        ] {
            let scene = row(Theme::junie(), level, Surface::Canvas, flags, false);
            assert_eq!(bg(&scene), expected_bg, "{level:?} {flags:?}");
            let cell = scene.buffer().cell((2, 0));
            assert!(
                cell.is_some_and(
                    |c| c.fg == expected_fg && c.modifier.contains(Modifier::BOLD) == bold
                ),
                "{level:?} {flags:?}: {cell:?}"
            );
        }
        let disabled = row(
            Theme::junie(),
            level,
            Surface::Canvas,
            selected_focus | StateFlags::HOVERED | StateFlags::PRESSED | StateFlags::DISABLED,
            false,
        );
        assert_eq!(bg(&disabled), canvas);
        assert!(
            disabled
                .buffer()
                .cell((0, 0))
                .is_some_and(|c| c.symbol() == " ")
        );
        assert!(
            disabled
                .buffer()
                .cell((2, 0))
                .is_some_and(|c| !c.modifier.contains(Modifier::BOLD))
        );
    }
}

#[test]
fn junie_custom_tokens_bind_late_without_changing_other_theme_policies() {
    use ratatui_core::style::Color;
    let custom = [
        Color::Indexed(10),
        Color::Indexed(11),
        Color::Indexed(12),
        Color::Indexed(13),
        Color::Indexed(14),
    ];
    let theme = Theme::junie()
        .builder()
        .surfaces(custom)
        .field(Color::Indexed(15), Color::Indexed(16))
        .build();
    for (surface, expected) in [
        (Surface::Canvas, Color::Indexed(12)),
        (Surface::Surface, Color::Indexed(13)),
        (Surface::Elevated, Color::Indexed(13)),
        (Surface::Overlay, Color::Indexed(14)),
        (Surface::Popover, Color::Indexed(14)),
        (Surface::Field, Color::Indexed(16)),
        (Surface::FieldHover, Color::Indexed(14)),
    ] {
        assert_eq!(
            bg(&row(
                theme.clone(),
                ColorLevel::TrueColor,
                surface,
                StateFlags::HOVERED,
                false
            )),
            expected
        );
    }
    for theme in [Theme::paper(), Theme::from_tokens(Theme::junie().color)] {
        let expected = theme.bg(Surface::Surface);
        assert_eq!(
            bg(&row(
                theme,
                ColorLevel::TrueColor,
                Surface::Canvas,
                StateFlags::HOVERED,
                false
            )),
            expected
        );
    }
    assert_eq!(Theme::junie().raise(Surface::Canvas), Surface::Surface);
}

#[test]
fn dimming_hover_preserves_reference_neutral_planes() {
    use ratatui_core::style::Color;
    for (surface, expected) in [
        (Surface::Canvas, 0x18_18_1b),
        (Surface::Surface, 0x27_27_2a),
        (Surface::Elevated, 0x27_27_2a),
        (Surface::Overlay, 0x27_27_2a),
        (Surface::Popover, 0x27_27_2a),
        (Surface::Field, 0x18_18_1b),
        (Surface::FieldHover, 0x27_27_2a),
    ] {
        assert_eq!(
            bg(&row(
                Theme::junie(),
                ColorLevel::TrueColor,
                surface,
                StateFlags::HOVERED,
                true
            )),
            Color::from_u32(expected)
        );
    }
}

#[test]
fn dim_after_surface_scope_uses_custom_neutral_tokens_in_every_mode() {
    use ratatui_core::style::Color;
    let theme = Theme::junie()
        .builder()
        .surfaces([
            Color::Rgb(0, 0, 0),
            Color::Rgb(255, 0, 0),
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 255),
            Color::Rgb(255, 255, 255),
        ])
        .build();
    for (level, elevated, overlay) in [
        (
            ColorLevel::TrueColor,
            Color::Rgb(255, 255, 255),
            Color::Rgb(0, 0, 255),
        ),
        (ColorLevel::Ansi256, Color::Indexed(231), Color::Indexed(21)),
        (ColorLevel::Ansi16, Color::White, Color::LightBlue),
        (ColorLevel::Mono, Color::White, Color::Black),
    ] {
        for (surface, expected) in [
            (Surface::Canvas, elevated),
            (Surface::Surface, overlay),
            (Surface::Elevated, overlay),
            (Surface::Overlay, overlay),
            (Surface::Popover, overlay),
            (Surface::Field, elevated),
            (Surface::FieldHover, overlay),
        ] {
            let scene = row(theme.clone(), level, surface, StateFlags::HOVERED, true);
            assert_eq!(bg(&scene), expected, "{level:?} {surface:?}");
            assert_eq!(
                scene.buffer().cell((2, 0)).map(|cell| cell.bg),
                Some(expected),
                "label {level:?} {surface:?}"
            );
        }
    }
}

#[test]
fn foreground_only_painters_preserve_background_and_raw_overwrite_drops_it() {
    let mut scene = Scene::new(
        "theme_holla_paint_provenance",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui, _| {
        ui.with_surface(Surface::Canvas, |ui| {
            let hover = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::HOVERED,
                )
                .style;
            ui.fill(AREA, hover);
            let text = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_cell(Position::new(0, 0), "a", text);
            ui.paint_str(Rect::new(1, 0, 1, 1), "b", text);
            ui.paint_spans(Rect::new(2, 0, 1, 1), &[Span::new("c")], text);
            ui.paint_style(Rect::new(3, 0, 1, 1), text);
            ui.fill(Rect::new(4, 0, 1, 1), text);

            ui.paint_str(
                Rect::new(5, 0, 1, 1),
                "r",
                Style::new().bg(ratatui_core::style::Color::Red),
            );
        });
        ui.dim_layer(AREA, 1);
    });
    for x in 0..5 {
        assert!(
            scene
                .buffer()
                .cell((x, 0))
                .is_some_and(|c| c.bg == ratatui_core::style::Color::Rgb(24, 24, 27)),
            "painter {x}"
        );
    }
    assert!(
        scene
            .buffer()
            .cell((5, 0))
            .is_some_and(|c| c.bg == ratatui_core::style::Color::Rgb(0, 0, 0))
    );
    scene.draw(|ui, _| {
        let hover = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::HOVERED,
            )
            .style;
        ui.fill(AREA, hover);
        let (buffer, _) = ui.raw();
        if let Some(cell) = buffer.cell_mut((2, 0)) {
            cell.set_symbol("r")
                .set_style(Style::new().bg(ratatui_core::style::Color::Red));
        }
        ui.dim_layer(AREA, 1);
    });
    assert!(
        scene
            .buffer()
            .cell((2, 0))
            .is_some_and(|c| c.bg == ratatui_core::style::Color::Rgb(0, 0, 0))
    );
}

#[derive(Default)]
struct ListApp {
    state: termrock_navigation::ListState,
}
impl App for ListApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ =
            termrock_navigation::List::new(OWNER).update(cx, &mut self.state, &["alpha", "beta"]);
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        termrock_navigation::List::new(OWNER).draw(
            ui,
            Rect::new(0, 0, 16, 2),
            &self.state,
            &["alpha", "beta"],
        );
    }
}

#[test]
fn production_list_pointer_hover_does_not_move_keyboard_cursor() {
    use ratatui_core::style::Color;
    use termrock_runtime::stub;
    let mut runtime = Runtime::new(ListApp::default(), Theme::junie());
    let _ = runtime.initialize();
    let mut buffer = ratatui_core::buffer::Buffer::empty(Rect::new(0, 0, 16, 2));
    runtime
        .draw_buffer(Rect::new(0, 0, 16, 2), &mut buffer)
        .commit_presented();
    let cursor = runtime.app().state.cursor();
    let _ = stub::deliver(&mut runtime, stub::mouse(MouseKind::Move, 5, 1));
    runtime
        .draw_buffer(Rect::new(0, 0, 16, 2), &mut buffer)
        .commit_presented();
    assert_eq!(
        buffer.cell((5, 1)).map(|c| c.bg),
        Some(Color::Rgb(24, 24, 27))
    );
    assert_eq!(runtime.app().state.cursor(), cursor);
    let _ = stub::deliver(&mut runtime, stub::key(KeyCode::Char('x')));
    runtime
        .draw_buffer(Rect::new(0, 0, 16, 2), &mut buffer)
        .commit_presented();
    assert_eq!(buffer.cell((5, 1)).map(|c| c.bg), Some(Color::Rgb(0, 0, 0)));
}

#[test]
fn raw_background_immediately_after_resolution_has_no_semantic_provenance() {
    use ratatui_core::style::Color;
    let mut scene = Scene::new(
        "raw_after_hover",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    for background in [Color::Red, Color::Rgb(24, 24, 27)] {
        scene.draw(|ui, _| {
            let _ = ui.style(
                Family::LIST,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::HOVERED,
            );
            ui.fill(AREA, Style::new().bg(background));
            ui.dim_layer(AREA, 1);
        });
        assert_eq!(
            bg(&scene),
            Color::Rgb(0, 0, 0),
            "raw background {background:?}"
        );
    }
}

#[test]
fn delayed_resolved_style_keeps_its_own_surface_after_another_query() {
    let source = Family::custom("saved.surface");
    let theme = Theme::junie().define_family(source, |r| {
        r.part(Part::CONTAINER)
            .base(StylePatch::new().set_bg(Role::Surface(Surface::Overlay)));
    });
    let mut scene = Scene::new("delayed_style", theme, ColorLevel::TrueColor, 16, 1);
    scene.draw(|ui, _| {
        let saved = ui
            .style(
                source,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::empty(),
            )
            .style;
        let _ = ui.style(
            Family::LIST,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(AREA, saved);
        ui.dim_layer(AREA, 1);
    });
    assert_eq!(bg(&scene), ratatui_core::style::Color::Rgb(39, 39, 42));
}

#[test]
fn wide_cell_string_and_span_reset_the_same_continuation_provenance() {
    let mut buffers = Vec::new();
    for painter in 0..3 {
        let mut scene = Scene::new(
            "wide_provenance",
            Theme::junie(),
            ColorLevel::TrueColor,
            16,
            1,
        );
        scene.draw(|ui, _| {
            let hover = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::HOVERED,
                )
                .style;
            ui.fill(AREA, hover);
            match painter {
                0 => ui.paint_cell(Position::new(0, 0), "界", hover),
                1 => {
                    ui.paint_str(AREA, "界", hover);
                }
                _ => {
                    ui.paint_spans(AREA, &[Span::new("界")], hover);
                }
            }
            ui.dim_layer(AREA, 1);
        });
        buffers.push(scene.buffer().clone());
    }
    assert!(buffers.windows(2).all(|pair| pair.first() == pair.get(1)));
}

#[test]
fn wide_grapheme_does_not_partially_paint_at_the_right_clip_boundary() {
    let mut buffers = Vec::new();
    for cell in [true, false] {
        let mut scene = Scene::new("wide_clip", Theme::junie(), ColorLevel::TrueColor, 16, 1);
        scene.draw(|ui, _| {
            ui.with_area(Rect::new(15, 0, 1, 1), |ui| {
                if cell {
                    ui.paint_cell(Position::new(15, 0), "界", Style::new());
                } else {
                    ui.paint_str(AREA, "界", Style::new());
                }
            });
        });
        buffers.push(scene.buffer().clone());
    }
    assert!(buffers.windows(2).all(|pair| pair.first() == pair.get(1)));
}

#[test]
fn equal_palette_colors_never_confuse_raw_and_saved_semantic_channels() {
    use ratatui_core::style::Color;
    for (level, neutral) in [
        (ColorLevel::TrueColor, Color::Rgb(255, 255, 255)),
        (ColorLevel::Ansi256, Color::Indexed(231)),
        (ColorLevel::Ansi16, Color::White),
        (ColorLevel::Mono, Color::White),
    ] {
        let theme = Theme::junie()
            .builder()
            .surfaces([
                Color::Rgb(0, 0, 0),
                Color::Rgb(0, 0, 0),
                Color::Rgb(255, 255, 255),
                Color::Rgb(255, 255, 255),
                Color::Rgb(255, 255, 255),
            ])
            .build();
        let mut scene = Scene::new("carrier_collision", theme, level, 16, 1);
        scene.draw(|ui, _| {
            let saved = ui.with_surface(Surface::Surface, |ui| {
                ui.paint_patch(
                    &StylePatch::new()
                        .set_bg(Role::HoverSurface)
                        .set_fg(Role::Fg(FgStep::Primary)),
                )
            });
            let _ = ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(Surface::Canvas)));

            let styles = [
                saved,
                PaintStyle::from(Style::new().bg(neutral)),
                saved.bg(neutral),
                saved.patch(Style::new().bg(neutral)),
                PaintStyle::new().with_bg_from(saved),
                saved
                    .add_modifier(Modifier::BOLD)
                    .remove_modifier(Modifier::ITALIC),
                saved.fg(Color::Red),
                saved.patch(Style::new().fg(Color::Red)),
            ];
            for (x, style) in styles.into_iter().enumerate() {
                ui.fill(Rect::new(x as u16, 0, 1, 1), style);
            }
            ui.dim_layer(AREA, 1);
        });
        for x in [0, 4, 5, 6, 7] {
            assert_eq!(
                scene.buffer().cell((x, 0)).map(|c| c.bg),
                Some(neutral),
                "semantic {level:?} {x}"
            );
        }
        let canvas = match level {
            ColorLevel::TrueColor => Color::Rgb(0, 0, 0),
            ColorLevel::Ansi256 => Color::Indexed(16),
            _ => Color::Black,
        };
        for x in [1, 2, 3] {
            assert_eq!(
                scene.buffer().cell((x, 0)).map(|c| c.bg),
                Some(canvas),
                "raw {level:?} {x}"
            );
        }
    }
}

#[test]
fn background_only_and_modifier_only_paint_preserve_the_foreground_origin() {
    use ratatui_core::style::Color;
    let mut scene = Scene::new(
        "fg_inheritance",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui, _| {
        let ghost = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Ghost)));
        ui.paint_str(AREA, "abc", ghost);
        ui.paint_style(Rect::new(0, 0, 1, 1), Style::new().bg(Color::Red));
        ui.paint_style(
            Rect::new(1, 0, 1, 1),
            Style::new().add_modifier(Modifier::ITALIC),
        );

        ui.paint_style(
            Rect::new(2, 0, 1, 1),
            Style::new().fg(ghost.fg.unwrap_or(Color::Reset)),
        );
        ui.dim_layer(AREA, 1);
    });
    assert_eq!(
        scene
            .buffer()
            .cell((0, 0))
            .map(ratatui_core::buffer::Cell::symbol),
        Some(" ")
    );
    assert_eq!(
        scene
            .buffer()
            .cell((1, 0))
            .map(ratatui_core::buffer::Cell::symbol),
        Some(" ")
    );
    assert_eq!(
        scene
            .buffer()
            .cell((2, 0))
            .map(ratatui_core::buffer::Cell::symbol),
        Some("c")
    );
}

#[test]
fn aligned_production_cells_move_their_semantic_origin_with_the_glyphs() {
    use ratatui_core::style::Color;
    let mut scene = Scene::new(
        "aligned_origin",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui, _| {
        {
            let mut row = RowUi::new(
                ui,
                OWNER,
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::HOVERED,
                ItemKey::index(0),
                AREA,
            );
            let mut columns = row.columns(&[Track::Flex(1)]);
            columns
                .cell(0)
                .text("a界")
                .align(Align::Right)
                .patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Ghost)));
        }
        ui.dim_layer(AREA, 1);
    });
    for x in [13, 14] {
        let cell = scene.buffer().cell((x, 0));
        assert_eq!(
            cell.map(ratatui_core::buffer::Cell::symbol),
            Some(" "),
            "shifted Ghost must erase at {x}"
        );
        assert_eq!(
            cell.map(|c| c.bg),
            Some(Color::Rgb(24, 24, 27)),
            "lead background moved at {x}"
        );
    }
    assert_eq!(
        scene.buffer().cell((15, 0)).map(|c| c.bg),
        Some(Color::Rgb(0, 0, 0)),
        "wide reset has no inherited plane"
    );
}

#[test]
fn shared_grapheme_writer_matches_ratatui_at_clip_and_width_boundaries() {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::style::Color;
    for text in ["界x", "👩‍💻x", "a\u{301}z", "\u{200b}x", "\n\tx", "ab界", "x"] {
        for width in 0..6 {
            let area = Rect::new(2, 0, width, 1);
            let mut scene = Scene::new(
                "grapheme_reference",
                Theme::junie(),
                ColorLevel::TrueColor,
                16,
                1,
            );
            let raw = Style::new().fg(Color::Red).bg(Color::Blue);
            let mut before = Buffer::empty(AREA);
            scene.draw(|ui, _| {
                ui.fill(AREA, Style::new().bg(Color::Green));
                before = ui.raw().0.clone();
                ui.paint_str(area, text, raw);
            });
            before.set_stringn(area.x, area.y, text, usize::from(width), raw);
            assert_eq!(scene.buffer(), &before, "{text:?} width {width}");
        }
    }
}

struct NestedPaintOrigins {
    reverse: bool,
    raw: bool,
}
impl App for NestedPaintOrigins {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        use termrock_runtime::{Backdrop, LayerSpec};
        for (id, backdrop) in [
            (Id::root("origins.lower"), Backdrop::None),
            (
                Id::root("origins.upper"),
                Backdrop::Dim {
                    exclude_footer: false,
                },
            ),
        ] {
            if !cx.is_open(id) {
                cx.open_layer(id, LayerSpec::modal(id).backdrop(backdrop));
            }
        }
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        use ratatui_core::style::Color;
        let page = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas)),
        );
        ui.fill(ui.full(), page);
        ui.paint_str(AREA, "PPPPPPPPPPPPPPPP", page);
        let lower = |ui: &mut Ui<'_>, _: Rect| {
            let origin = ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Ghost))
                    .set_bg(Role::Surface(Surface::Overlay)),
            );
            ui.paint_str(Rect::new(1, 0, 1, 1), "X", origin);
            {
                let mut row = RowUi::new(
                    ui,
                    OWNER,
                    Family::LIST,
                    Variant::DEFAULT,
                    StateFlags::HOVERED,
                    ItemKey::index(0),
                    Rect::new(3, 0, 8, 1),
                );
                let mut columns = row.columns(&[Track::Flex(1)]);
                columns.cell(0).text("a界").align(Align::Right);
            }
            if self.raw {
                ui.paint_str(
                    Rect::new(1, 0, 1, 1),
                    "R",
                    Style::new()
                        .fg(origin.fg.unwrap_or(Color::Reset))
                        .bg(origin.bg.unwrap_or(Color::Reset)),
                );
            }
        };
        let upper = |ui: &mut Ui<'_>, _: Rect| {
            let origin = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary)));
            ui.paint_str(Rect::new(15, 0, 1, 1), "U", origin);
        };
        if self.reverse {
            ui.layer(Id::root("origins.upper"), upper);
            ui.layer(Id::root("origins.lower"), lower);
        } else {
            ui.layer(Id::root("origins.lower"), lower);
            ui.layer(Id::root("origins.upper"), upper);
        }
    }
}

#[test]
fn nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells() {
    use ratatui_core::style::Color;
    use ratatui_core::style::Modifier;
    for reverse in [false, true] {
        for raw in [false, true] {
            let h = Harness::new(NestedPaintOrigins { reverse, raw }, Theme::junie(), 16, 2);
            let ghost = Color::Rgb(38, 38, 38);
            let overlay = Color::Rgb(39, 39, 42);
            let page = h.cell(0, 0);
            let composed = h.cell(1, 0);
            let upper = h.cell(15, 0);

            // Frozen source Theme::backdrop keeps the glyph and maps Ghost over Overlay to
            // Ghost/Overlay, including the raw-painted glyph whose origin carries Overlay.
            assert_eq!(
                (
                    composed.symbol(),
                    composed.fg,
                    composed.bg,
                    composed.underline_color,
                    composed.modifier,
                ),
                if raw {
                    ("R", ghost, overlay, Color::Reset, Modifier::empty())
                } else {
                    ("X", ghost, overlay, Color::Reset, Modifier::empty())
                },
                "ordinary and raw painted glyphs survive the modal backdrop"
            );

            // The backdrop dims existing page text in place. The top layer paints after the
            // backdrop and remains bright. Check full cell appearance, not only glyphs.
            assert_eq!(
                (
                    page.symbol(),
                    page.fg,
                    page.bg,
                    page.underline_color,
                    page.modifier,
                ),
                (
                    "P",
                    Color::Rgb(138, 138, 138),
                    Color::Rgb(0, 0, 0),
                    Color::Reset,
                    Modifier::empty(),
                ),
                "unwritten page cell retains glyph with frozen dim style"
            );
            assert_eq!(
                (
                    upper.symbol(),
                    upper.fg,
                    upper.bg,
                    upper.underline_color,
                    upper.modifier,
                ),
                (
                    "U",
                    Color::Rgb(255, 255, 255),
                    Color::Reset,
                    Color::Reset,
                    Modifier::empty()
                ),
                "top layer remains undimmed"
            );
            assert_eq!(h.cursor(), None, "layers do not manufacture a cursor");
        }
    }
}
