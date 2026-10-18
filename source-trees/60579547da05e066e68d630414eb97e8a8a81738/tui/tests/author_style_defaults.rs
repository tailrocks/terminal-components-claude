//! Author defaults keep semantic pairs before Mono policy and explicit overrides.
use junie_tui::theme::StyleDefaults;
use junie_tui::{
    ColorLevel, Family, FgStep, Modifier, MonoRule, Overlay, OverlayRule, Part, Rect, Role,
    StateFlags, StylePatch, Surface, Theme, Variant,
};
use junie_tui_testing::Scene;
use ratatui_core::style::Color;
const FAMILY: Family = Family::custom("author.pairs");
const TEXT: Part = Part::custom("critical.text");
const INVERSE: Part = Part::custom("inverse.brand");
const HIDDEN: Part = Part::custom("hidden.gutter");
const AREA: Rect = Rect::new(0, 0, 16, 1);
static MONO: [MonoRule; 2] = [
    (
        TEXT,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        INVERSE,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary)),
    ),
];
#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;

#[test]
fn authored_pairs_keep_truecolor_and_explicit_mono_readability_in_both_themes() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut scene = Scene::new("author_pairs", theme.clone(), level, 16, 1);
            let pairs = [
                (TEXT, Role::Accent, Role::Surface(Surface::Canvas)),
                (TEXT, Role::Danger, Role::Surface(Surface::Canvas)),
                (TEXT, Role::Warning, Role::Surface(Surface::Canvas)),
                (INVERSE, Role::OnAccent, Role::Accent),
                (
                    HIDDEN,
                    Role::Surface(Surface::Canvas),
                    Role::Surface(Surface::Canvas),
                ),
            ];
            scene.draw(|ui, _| {
                for (x, (part, fg, bg)) in pairs.into_iter().enumerate() {
                    let style = ui
                        .style_defaults(
                            FAMILY,
                            Variant::DEFAULT,
                            part,
                            StateFlags::empty(),
                            StyleDefaults::new(StylePatch::new().set_fg(fg).set_bg(bg)).mono(&MONO),
                            None,
                        )
                        .style;
                    ui.paint_str(Rect::new(x as u16, 0, 1, 1), "X", style);
                }
            });
            for x in 0..4 {
                let c = scene.buffer().cell((x, 0));
                assert!(
                    c.is_some_and(|c| c.fg != c.bg),
                    "{level:?} pair{x} remains readable"
                );
            }
            let gutter = scene.buffer().cell((4, 0));
            assert!(
                gutter.is_some_and(|c| c.fg == c.bg),
                "intentional hiddenpair stays equal"
            );
            if level == ColorLevel::TrueColor {
                for (x, fg, bg) in [
                    (0, theme.color.accent, theme.bg(Surface::Canvas)),
                    (1, theme.color.danger, theme.bg(Surface::Canvas)),
                    (2, theme.color.warning, theme.bg(Surface::Canvas)),
                    (3, theme.color.on_accent, theme.color.accent),
                ] {
                    assert_eq!(
                        scene.buffer().cell((x, 0)).map(|c| (c.fg, c.bg)),
                        Some((fg, bg))
                    );
                }
            }
            if level == ColorLevel::Mono {
                let light = theme.bg(Surface::Canvas) == Color::Rgb(251, 250, 248);
                let (canvas, text) = if light {
                    (Color::White, Color::Black)
                } else {
                    (Color::Black, Color::White)
                };
                assert_eq!(
                    scene.buffer().cell((0, 0)).map(|c| (c.fg, c.bg)),
                    Some((text, canvas))
                );
                assert_eq!(
                    scene.buffer().cell((3, 0)).map(|c| (c.fg, c.bg)),
                    Some((canvas, text))
                );
            }
        }
    }
}

#[test]
fn ordinary_unknown_family_is_unchanged_and_different_defaults_never_alias() {
    let mut scene = Scene::new(
        "default_identity",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui, _| {
        let ordinary = ui.style(
            FAMILY,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        let ghost = ui.style_defaults(
            FAMILY,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            StyleDefaults::new(
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Ghost))
                    .set_bg(Role::Surface(Surface::Overlay)),
            ),
            None,
        );
        let red = ui.style_defaults(
            FAMILY,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
            StyleDefaults::new(StylePatch::new().set_fg(Role::Custom(Color::Red))),
            None,
        );
        assert_eq!(red.style.fg, Some(Color::Red));
        assert_eq!(
            red.style.bg, None,
            "opted defaults have no neutral background"
        );
        let after = ui.style(
            FAMILY,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        assert_eq!(
            ordinary, after,
            "custom defaults do not mutate or alias normal cache"
        );
        assert_eq!(ordinary.style.bg, Some(Color::Rgb(0, 0, 0)));
        ui.paint_str(AREA, "G", ghost.style);
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
        scene.buffer().cell((0, 0)).map(|c| c.bg),
        Some(Color::Rgb(39, 39, 42))
    );
}

#[test]
fn declared_recipe_variant_state_theme_scope_and_local_override_author_defaults() {
    static SCOPE: [OverlayRule; 1] = [(
        FAMILY,
        Variant::PRIMARY,
        TEXT,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Custom(Color::White)),
    )];
    let family = Theme::junie().define_family(FAMILY, |r| {
        r.default_variant(Variant::PRIMARY);
        r.part(TEXT)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Green)));
    });
    let variant = family
        .clone()
        .define_variant(FAMILY, Variant::PRIMARY, |r| {
            r.part(TEXT)
                .base(StylePatch::new().set_fg(Role::Custom(Color::Blue)))
                .when(
                    StateFlags::ERROR,
                    StylePatch::new().set_fg(Role::Custom(Color::Yellow)),
                );
        });
    let global = variant.clone().override_family(FAMILY, |r| {
        r.part(TEXT)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Cyan)));
    });
    let themed = global
        .clone()
        .override_variant(FAMILY, Variant::PRIMARY, |r| {
            r.part(TEXT)
                .base(StylePatch::new().set_fg(Role::Custom(Color::Magenta)));
        });
    for (theme, flags, expected, scoped, local) in [
        (Theme::junie(), StateFlags::empty(), Color::Red, false, None),
        (family, StateFlags::empty(), Color::Green, false, None),
        (
            variant.clone(),
            StateFlags::empty(),
            Color::Blue,
            false,
            None,
        ),
        (variant, StateFlags::ERROR, Color::Yellow, false, None),
        (global, StateFlags::ERROR, Color::Cyan, false, None),
        (
            themed.clone(),
            StateFlags::ERROR,
            Color::Magenta,
            false,
            None,
        ),
        (themed.clone(), StateFlags::ERROR, Color::White, true, None),
        (
            themed,
            StateFlags::ERROR,
            Color::Black,
            true,
            Some(StylePatch::new().set_fg(Role::Custom(Color::Black))),
        ),
    ] {
        let mut scene = Scene::new("default_precedence", theme, ColorLevel::TrueColor, 16, 1);
        scene.draw(|ui, _| {
            let paint = |ui: &mut junie_tui::Ui<'_>| {
                let style = ui
                    .style_defaults(
                        FAMILY,
                        Variant::DEFAULT,
                        TEXT,
                        flags,
                        StyleDefaults::new(StylePatch::new().set_fg(Role::Custom(Color::Red))),
                        local.as_ref(),
                    )
                    .style;
                ui.paint_str(AREA, "X", style);
            };
            if scoped {
                ui.with_overlay(&Overlay::new(&SCOPE), paint);
            } else {
                paint(ui);
            }
        });
        assert_eq!(scene.buffer().cell((0, 0)).map(|c| c.fg), Some(expected));
    }
}

#[test]
fn theme_mono_manifest_replaces_authored_whole_set_then_explicit_overrides_win() {
    static THEME_MONO: [MonoRule; 1] = [(
        TEXT,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary)),
    )];
    static SCOPE: [OverlayRule; 1] = [(
        FAMILY,
        Variant::DEFAULT,
        TEXT,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary)),
    )];
    let replaced = Theme::junie()
        .builder()
        .mono_rules(FAMILY, &THEME_MONO)
        .build();
    let overridden = replaced.clone().override_family(FAMILY, |r| {
        r.part(TEXT).base(
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas)),
        );
    });
    for (theme, scoped, local, expected) in [
        (Theme::junie(), false, None, (Color::White, Color::Black)),
        (
            Theme::junie().builder().mono_rules(FAMILY, &[]).build(),
            false,
            None,
            (Color::Gray, Color::Black),
        ),
        (replaced, false, None, (Color::Black, Color::White)),
        (
            overridden.clone(),
            false,
            None,
            (Color::White, Color::Black),
        ),
        (overridden.clone(), true, None, (Color::Black, Color::White)),
        (
            overridden,
            true,
            Some(
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(Surface::Canvas)),
            ),
            (Color::White, Color::Black),
        ),
    ] {
        let mut scene = Scene::new("mono_precedence", theme, ColorLevel::Mono, 16, 1);
        scene.draw(|ui, _| {
            let paint = |ui: &mut junie_tui::Ui<'_>| {
                let style = ui
                    .style_defaults(
                        FAMILY,
                        Variant::DEFAULT,
                        TEXT,
                        StateFlags::empty(),
                        StyleDefaults::new(
                            StylePatch::new()
                                .set_fg(Role::Accent)
                                .set_bg(Role::Surface(Surface::Canvas)),
                        )
                        .mono(&MONO),
                        local.as_ref(),
                    )
                    .style;
                ui.paint_str(AREA, "X", style);
            };
            if scoped {
                ui.with_overlay(&Overlay::new(&SCOPE), paint);
            } else {
                paint(ui);
            }
        });
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| (c.fg, c.bg)),
            Some(expected)
        );
    }
}

#[test]
fn default_binding_and_paint_allocate_nothing_and_local_clear_preserves_inheritance() {
    use junie_tui_testing::perf::{allocs, bytes};
    let mut scene = Scene::new(
        "default_allocations",
        Theme::junie(),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui, _| {
        let before = (allocs(), bytes());
        let style = ui
            .style_defaults(
                FAMILY,
                Variant::DEFAULT,
                TEXT,
                StateFlags::empty(),
                StyleDefaults::new(
                    StylePatch::new()
                        .set_fg(Role::Accent)
                        .set_bg(Role::Surface(Surface::Overlay)),
                )
                .mono(&MONO),
                Some(&StylePatch::new().clear_fg()),
            )
            .style;
        assert_eq!(style.fg, None);
        ui.paint_str(AREA, "X", style.add_modifier(Modifier::BOLD));
        assert_eq!((allocs(), bytes()), before);
    });
}

#[test]
fn generic_mono_and_state_targeted_defaults_keep_order_without_cache_aliasing() {
    static STATES: [MonoRule; 2] = [
        (
            TEXT,
            StateFlags::empty(),
            StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
        ),
        (
            TEXT,
            StateFlags::ERROR,
            StylePatch::new()
                .set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary)),
        ),
    ];
    let theme = Theme::junie().builder().mono_rules(FAMILY, &[]).build();
    let mut generic = Scene::new("default_generic_mono", theme, ColorLevel::Mono, 16, 1);
    generic.draw(|ui, _| {
        let style = ui
            .style_defaults(
                FAMILY,
                Variant::DEFAULT,
                Part::CONTAINER,
                StateFlags::PRESSED,
                StyleDefaults::new(
                    StylePatch::new()
                        .set_fg(Role::Accent)
                        .set_bg(Role::Surface(Surface::Canvas)),
                ),
                None,
            )
            .style;
        ui.paint_str(AREA, "G", style);
    });
    assert_eq!(
        generic
            .buffer()
            .cell((0, 0))
            .map(|c| (c.fg, c.bg, c.modifier)),
        Some((Color::Black, Color::White, Modifier::BOLD)),
        "genericpress survives empty targetedmanifest"
    );
    let mut scene = Scene::new(
        "default_mono_identity",
        Theme::junie(),
        ColorLevel::Mono,
        16,
        1,
    );
    scene.draw(|ui, _| {
        let defaults = StyleDefaults::new(
            StylePatch::new()
                .set_fg(Role::Accent)
                .set_bg(Role::Surface(Surface::Canvas)),
        );
        let plain = ui
            .style_defaults(
                FAMILY,
                Variant::DEFAULT,
                TEXT,
                StateFlags::empty(),
                defaults.mono(&STATES),
                None,
            )
            .style;
        let error = ui
            .style_defaults(
                FAMILY,
                Variant::DEFAULT,
                TEXT,
                StateFlags::ERROR,
                defaults.mono(&STATES),
                None,
            )
            .style;
        let no_rules = ui
            .style_defaults(
                FAMILY,
                Variant::DEFAULT,
                TEXT,
                StateFlags::ERROR,
                defaults,
                None,
            )
            .style;
        // Paint after all three same-family/part queries: each value owns its pair.
        for (x, style) in [plain, error, no_rules].into_iter().enumerate() {
            ui.paint_str(Rect::new(x as u16, 0, 1, 1), "X", style);
        }
    });
    for (x, pair) in [
        (0, (Color::White, Color::Black)),
        (1, (Color::Black, Color::White)),
        (2, (Color::Gray, Color::Black)),
    ] {
        assert_eq!(
            scene.buffer().cell((x, 0)).map(|c| (c.fg, c.bg)),
            Some(pair)
        );
    }
}
