//! Exact Junie row cells from Holla 794b095 src/theme.rs:329–372 and DESIGN.
//! Expectations are reference values, not values obtained from the candidate resolver.

use junie_tui::{
    App, ColorLevel, Cx, Family, Id, ItemKey, KeyCode, List, ListState, Modifier, MouseKind, Rect,
    Response, RowUi, StateFlags, Surface, Theme, Ui, Variant,
};
use junie_tui_testing::{Harness, Scene};
use ratatui_core::style::Color;

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

fn bg(scene: &Scene) -> Color {
    scene.buffer().cell((15, 0)).map_or(Color::Reset, |c| c.bg)
}

#[test]
fn every_hover_plane_uses_reference_tokens_in_all_color_modes() {
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
                0x18_18_1b, 0x27_27_2a, 0x27_27_2a, 0x3f_3f_46, 0x3f_3f_46, 0x23_23_28, 0x3f_3f_46,
            ]
            .map(Color::from_u32),
        ),
        (
            ColorLevel::Ansi256,
            [234, 235, 235, 238, 238, 235, 238].map(Color::Indexed),
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
        (ColorLevel::Mono, [Color::Black; 7]),
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
                Color::Indexed(234),
                Color::Indexed(234),
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
    use junie_tui::{Part, Span};
    use ratatui_core::{layout::Position, style::Style};
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
            // An explicit background starts new provenance, never inherits
            // the prior hover role just because a text style has no bg role.
            ui.paint_str(Rect::new(5, 0, 1, 1), "r", Style::new().bg(Color::Red));
        });
        ui.dim_layer(AREA, 1);
    });
    for x in 0..5 {
        assert!(
            scene
                .buffer()
                .cell((x, 0))
                .is_some_and(|c| c.bg == Color::Rgb(24, 24, 27)),
            "painter {x}"
        );
    }
    assert!(
        scene
            .buffer()
            .cell((5, 0))
            .is_some_and(|c| c.bg == Color::Rgb(0, 0, 0))
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
            cell.set_symbol("r").set_style(Style::new().bg(Color::Red));
        }
        ui.dim_layer(AREA, 1);
    });
    assert!(
        scene
            .buffer()
            .cell((2, 0))
            .is_some_and(|c| c.bg == Color::Rgb(0, 0, 0))
    );
}

#[derive(Default)]
struct ListApp {
    state: ListState,
}
impl App for ListApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = List::new(OWNER).update(cx, &mut self.state, &["alpha", "beta"]);
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        List::new(OWNER).draw(ui, Rect::new(0, 0, 16, 2), &self.state, &["alpha", "beta"]);
    }
}

#[test]
fn production_list_pointer_hover_does_not_move_keyboard_cursor() {
    let mut h = Harness::new(ListApp::default(), Theme::junie(), 16, 2);
    let cursor = h.app().state.cursor();
    let _ = h.mouse(MouseKind::Move, 5, 1);
    assert_eq!(h.cell(5, 1).bg, Color::Rgb(24, 24, 27));
    assert_eq!(h.app().state.cursor(), cursor);
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.cell(5, 1).bg, Color::Rgb(0, 0, 0));
}
