use showcase_demos::pages::frame;
use termrock::{
    App, Color, ColorLevel, Cx, Family, Modifier, Part, Rect, Response, Role, StateFlags,
    StylePatch, Surface, Theme, Ui, Variant,
};
use termrock_test_support::Harness;

struct HeadingApp {
    area: Rect,
    surface: Surface,
}

impl App for HeadingApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.with_surface(self.surface, |ui| {
            frame(ui, self.area, "Heading", "Description", |ui, body| {
                ui.paint_str(body, "BODY", ui.surface_style());
            });
        });
    }
}

#[test]
fn shared_heading_uses_semantic_parts_over_its_surface_without_changing_geometry() {
    let cases = [
        (
            ColorLevel::TrueColor,
            Color::Rgb(138, 138, 138),
            Color::Rgb(255, 255, 255),
        ),
        (
            ColorLevel::Ansi256,
            Color::Indexed(245),
            Color::Indexed(231),
        ),
        (ColorLevel::Ansi16, Color::Gray, Color::White),
        (ColorLevel::Mono, Color::Gray, Color::White),
    ];

    for (level, expected_meta, expected_title) in cases {
        let theme = Theme::junie().for_level(level);
        let expected_surface = theme.bg(Surface::Elevated);
        let harness = Harness::new(
            HeadingApp {
                area: Rect::new(2, 1, 30, 5),
                surface: Surface::Elevated,
            },
            theme,
            40,
            10,
        );

        let title = harness.cell(2, 1);
        assert_eq!(title.symbol(), "H", "{level:?} title starts in the header");
        assert_eq!(title.fg, expected_title, "{level:?} uses PANEL/TITLE ink");
        assert_eq!(
            title.bg, expected_surface,
            "{level:?} title keeps the surface"
        );

        let meta = harness.cell(11, 1);
        assert_eq!(
            meta.symbol(),
            "D",
            "{level:?} meta keeps its title-row offset"
        );
        assert_eq!(meta.fg, expected_meta, "{level:?} uses LIST/META ink");
        assert_eq!(
            meta.bg, expected_surface,
            "{level:?} meta keeps the surface"
        );

        assert!(harness.row(1).starts_with("  Heading  Description"));
        for blank_x in [9, 10, 22, 30] {
            let blank = harness.cell(blank_x, 1);
            assert_eq!(blank.symbol(), " ", "{level:?} heading gap at {blank_x}");
            assert_eq!(blank.fg, expected_title, "{level:?} gap keeps surface ink");
            assert_eq!(blank.bg, expected_surface, "{level:?} gap keeps surface bg");
        }
        let body = harness.cell(2, 3);
        assert_eq!(body.symbol(), "B", "body still starts two rows later");
        assert_eq!(body.fg, expected_title, "body keeps the surface ink");
        assert_eq!(body.bg, expected_surface, "body keeps the surface bg");
        assert!(harness.row(3).starts_with("  BODY"));
        assert_eq!(
            harness.cell(1, 1).symbol(),
            " ",
            "area left edge stays untouched"
        );
        assert_eq!(
            harness.cell(1, 1).bg,
            Color::Reset,
            "outside keeps its original style"
        );
        assert_eq!(
            harness.cell(32, 1).bg,
            Color::Reset,
            "area right edge stays untouched"
        );
    }
}

#[test]
fn shared_heading_uses_paper_theme_roles_instead_of_junie_inks() {
    let harness = Harness::new(
        HeadingApp {
            area: Rect::new(2, 1, 30, 5),
            surface: Surface::Elevated,
        },
        Theme::paper(),
        40,
        10,
    );

    let title = harness.cell(2, 1);
    assert_eq!(title.fg, Color::Rgb(27, 26, 23));
    assert_eq!(title.bg, Color::Rgb(232, 229, 223));

    let meta = harness.cell(11, 1);
    assert_eq!(meta.fg, Color::Rgb(119, 113, 106));
    assert_eq!(meta.bg, Color::Rgb(232, 229, 223));
}

#[test]
fn shared_heading_preserves_explicit_part_colors_reset_underline_and_modifiers() {
    const TITLE_FG: Color = Color::Rgb(1, 2, 3);
    const TITLE_BG: Color = Color::Rgb(4, 5, 6);
    const TITLE_UNDERLINE: Color = Color::Rgb(7, 8, 9);
    const META_FG: Color = Color::Rgb(10, 11, 12);
    const META_UNDERLINE: Color = Color::Rgb(13, 14, 15);

    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie()
            .for_level(level)
            .override_family(Family::PANEL, |recipe| {
                recipe.part(Part::TITLE).base(
                    StylePatch::new()
                        .set_fg(Role::Custom(TITLE_FG))
                        .set_bg(Role::Custom(TITLE_BG))
                        .set_underline(Role::Custom(TITLE_UNDERLINE))
                        .add(Modifier::BOLD),
                );
            })
            .override_family(Family::LIST, |recipe| {
                recipe.part(Part::META).base(
                    StylePatch::new()
                        .set_fg(Role::Custom(META_FG))
                        .set_bg(Role::Custom(Color::Reset))
                        .set_underline(Role::Custom(META_UNDERLINE))
                        .add(Modifier::ITALIC),
                );
            });
        let inherited_surface = theme.bg(Surface::Elevated);
        let expected_title = theme.resolve(
            Family::PANEL,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::empty(),
            Surface::Elevated,
        );
        let expected_meta = theme.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            Surface::Elevated,
        );
        let surface_fg = theme.color.fg.first().copied().unwrap_or(Color::Reset);
        let harness = Harness::new(
            HeadingApp {
                area: Rect::new(2, 1, 30, 5),
                surface: Surface::Elevated,
            },
            theme.clone(),
            40,
            10,
        );

        let title = harness.cell(2, 1);
        assert_eq!(
            title.fg,
            expected_title.style.fg.unwrap_or(surface_fg),
            "{level:?}"
        );
        assert_eq!(
            title.bg,
            expected_title.style.bg.unwrap_or(inherited_surface),
            "{level:?}"
        );
        assert_eq!(
            title.underline_color,
            expected_title.style.underline_color.unwrap_or(Color::Reset),
            "{level:?}"
        );
        assert!(title.modifier.contains(Modifier::BOLD), "{level:?}");

        let meta = harness.cell(11, 1);
        assert_eq!(
            meta.fg,
            expected_meta.style.fg.unwrap_or(surface_fg),
            "{level:?}"
        );
        assert_eq!(
            meta.bg,
            expected_meta.style.bg.unwrap_or(inherited_surface),
            "{level:?} Reset keeps the inherited surface"
        );
        assert_eq!(
            meta.underline_color,
            expected_meta.style.underline_color.unwrap_or(Color::Reset),
            "{level:?}"
        );
        assert!(meta.modifier.contains(Modifier::ITALIC), "{level:?}");
    }
}
