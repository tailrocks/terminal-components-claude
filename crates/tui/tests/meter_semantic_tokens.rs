//! Typed meter token policies preserve source surfaces and custom colors.
use junie_tui::theme::downgrade_color;
use junie_tui::{ColorLevel, Id, Meter, MeterFillRest, MeterVisual, Surface, Theme};
use junie_tui_testing::Scene;
use ratatui_core::style::Color;
const ID: Id = Id::root("meter.semantic");
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];
#[test]
fn junie_block_rest_matches_pinned_lift_on_every_surface_and_capability() {
    let rgb = [
        Color::Rgb(24, 24, 27),
        Color::Rgb(39, 39, 42),
        Color::Rgb(39, 39, 42),
        Color::Rgb(63, 63, 70),
        Color::Rgb(63, 63, 70),
        Color::Rgb(35, 35, 40),
        Color::Rgb(63, 63, 70),
    ];
    let indexed = [233, 235, 235, 237, 237, 234, 234];
    let ansi16 = [
        Color::Black,
        Color::Black,
        Color::Black,
        Color::DarkGray,
        Color::DarkGray,
        Color::Black,
        Color::DarkGray,
    ];
    let mono = [
        Color::Black,
        Color::Black,
        Color::Black,
        Color::Black,
        Color::DarkGray,
        Color::Black,
        Color::Black,
    ];
    for (i, surface) in [
        Surface::Canvas,
        Surface::Surface,
        Surface::Elevated,
        Surface::Overlay,
        Surface::Popover,
        Surface::Field,
        Surface::FieldHover,
    ]
    .into_iter()
    .enumerate()
    {
        for level in LEVELS {
            let expected = match level {
                ColorLevel::TrueColor => rgb.get(i).copied(),
                ColorLevel::Ansi256 => indexed.get(i).copied().map(Color::Indexed),
                ColorLevel::Ansi16 => ansi16.get(i).copied(),
                ColorLevel::Mono => mono.get(i).copied(),
                _ => None,
            };
            let mut scene = Scene::new("meter_lift", Theme::junie(), level, 12, 1);
            scene.draw(|ui, area| {
                ui.with_surface(surface, |ui| {
                    Meter::new(ID)
                        .ratio(0.0)
                        .visual(MeterVisual::Block)
                        .draw(ui, area);
                });
            });
            assert_eq!(
                scene.buffer().cell((10, 0)).map(|c| c.bg),
                expected,
                "{surface:?} {level:?}"
            );
        }
    }
}
#[test]
fn explicit_custom_rest_and_paper_remain_color_tokens() {
    let explicit = Color::Rgb(17, 177, 92);
    for base in [Theme::junie(), Theme::paper()] {
        let mut custom = base;
        custom.color.meter.fill_rest = MeterFillRest::Color(explicit);
        for level in LEVELS {
            let narrowed = custom.for_level(level);
            assert_eq!(
                narrowed.color.meter.fill_rest,
                MeterFillRest::Color(downgrade_color(explicit, level))
            );
            let mut scene = Scene::new("meter_custom_rest", custom.clone(), level, 12, 1);
            scene.draw(|ui, area| {
                ui.with_surface(Surface::Popover, |ui| {
                    Meter::new(ID)
                        .ratio(0.0)
                        .visual(MeterVisual::Block)
                        .draw(ui, area);
                });
            });
            assert_eq!(
                scene.buffer().cell((10, 0)).map(|c| c.bg),
                Some(downgrade_color(explicit, level))
            );
        }
    }
    assert!(matches!(
        Theme::paper().color.meter.fill_rest,
        MeterFillRest::Color(_)
    ));
}
#[test]
fn symbolic_slots_never_shift_palette_projection_or_mutation_guards() {
    let source = Theme::junie();
    assert_eq!(source.color.semantic_colors().len(), 73);
    assert_eq!(source.color.colors().len(), 72);
    for a in LEVELS {
        for b in LEVELS {
            for c in LEVELS {
                let chain = source
                    .downgrade(a)
                    .downgrade(b)
                    .downgrade(c)
                    .downgrade(ColorLevel::Mono);
                assert_eq!(chain.color, source.downgrade(ColorLevel::Mono).color);
                let mut changed = source.downgrade(a);
                changed.color.meter.fill_rest = MeterFillRest::Color(Color::Red);
                let mut generic = changed.clone();
                generic.capability_palettes = None;
                let expected_rest = generic
                    .downgrade(b)
                    .downgrade(c)
                    .downgrade(ColorLevel::Mono)
                    .color
                    .meter
                    .fill_rest;
                let changed = changed
                    .downgrade(b)
                    .downgrade(c)
                    .downgrade(ColorLevel::Mono);
                let mut expected = source.downgrade(ColorLevel::Mono).color;
                expected.meter.fill_rest = expected_rest;
                assert_eq!(changed.color, expected);
            }
        }
    }
}

#[test]
fn authored_policy_transitions_use_typed_slot_guards() {
    let mut source = Theme::junie();
    source.capability_palettes = None;
    source.color.meter.fill_rest = MeterFillRest::Color(Color::Red);
    let mut indexed = source
        .color
        .map_colors(&mut |c| downgrade_color(c, ColorLevel::Ansi256));
    indexed.meter.fill_rest = MeterFillRest::ReferenceLift;
    let mut mono = source
        .color
        .map_colors(&mut |c| downgrade_color(c, ColorLevel::Mono));
    mono.meter.fill_rest = MeterFillRest::Color(Color::Blue);
    let palette = junie_tui::CapabilityPalettes::new(source.color)
        .with(ColorLevel::Ansi256, indexed)
        .with(ColorLevel::Mono, mono);
    let source = source.builder().capability_palettes(palette).build();
    let projected = source.downgrade(ColorLevel::Ansi256);
    assert_eq!(
        projected.color.meter.fill_rest,
        MeterFillRest::ReferenceLift
    );
    assert_eq!(
        projected.downgrade(ColorLevel::Mono).color.meter.fill_rest,
        MeterFillRest::Color(Color::Blue)
    );
    let mut explicit = projected.clone();
    explicit.color.meter.fill_rest = MeterFillRest::Color(Color::Reset);
    assert_ne!(explicit.fingerprint(), projected.fingerprint());
    assert_eq!(
        explicit
            .downgrade(ColorLevel::Ansi16)
            .downgrade(ColorLevel::Mono)
            .color
            .meter
            .fill_rest,
        MeterFillRest::Color(Color::Reset)
    );
}

#[test]
fn low_and_stale_tokens_match_reference_roles_but_custom_colors_remain_authoritative() {
    for level in LEVELS {
        let reference = Theme::junie().for_level(level);
        assert_eq!(reference.color.meter.low, reference.color.fg[1]);
        assert_eq!(reference.color.meter.stale, reference.color.fg[3]);
        let mut custom = Theme::junie();
        custom.color.meter.low = Color::Rgb(12, 123, 210);
        custom.color.meter.stale = Color::Rgb(210, 100, 13);
        let narrowed = custom.for_level(level);
        assert_eq!(
            narrowed.color.meter.low,
            downgrade_color(Color::Rgb(12, 123, 210), level)
        );
        assert_eq!(
            narrowed.color.meter.stale,
            downgrade_color(Color::Rgb(210, 100, 13), level)
        );
    }
}

#[test]
fn delayed_fill_rest_keeps_source_surface_provenance_after_scope_exit() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let expected = theme.bg(Surface::Elevated);
        let mut scene = Scene::new("meter_delayed_rest", theme, level, 4, 1);
        scene.draw(|ui, area| {
            let saved = ui.with_surface(Surface::Field, |ui| {
                ui.paint_patch(
                    &junie_tui::StylePatch::new()
                        .set_bg(junie_tui::Role::Meter(junie_tui::MeterRole::FillRest)),
                )
            });
            let _ = ui.with_surface(Surface::Popover, |ui| {
                ui.paint_patch(
                    &junie_tui::StylePatch::new().set_bg(junie_tui::Role::CurrentSurface),
                )
            });
            ui.paint_str(area, "X", saved);
            ui.dim_layer(area, 1);
        });
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| c.bg),
            Some(expected),
            "{level:?}"
        );
    }
}

#[test]
fn quantized_aliases_follow_pinned_ordered_lift() {
    for (level, surface, expected) in [
        (
            ColorLevel::Ansi256,
            Surface::FieldHover,
            Color::Indexed(234),
        ),
        (ColorLevel::Mono, Surface::Overlay, Color::Black),
        (ColorLevel::Mono, Surface::FieldHover, Color::Black),
        (ColorLevel::Ansi16, Surface::Surface, Color::Black),
    ] {
        let mut scene = Scene::new("meter_alias", Theme::junie(), level, 4, 1);
        scene.draw(|ui, area| {
            ui.with_surface(surface, |ui| {
                Meter::new(ID)
                    .ratio(0.0)
                    .visual(MeterVisual::Block)
                    .draw(ui, area);
            });
        });
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| c.bg),
            Some(expected),
            "{level:?}/{surface:?}"
        );
    }
}

// Direct transcription of immutable Holla theme.rs362–371 for custom palettes.
fn pinned_lift(theme: &Theme, surface: Surface) -> Color {
    let bg = theme.bg(surface);
    if bg == theme.bg(Surface::Canvas) {
        theme.bg(Surface::Elevated)
    } else if bg == theme.bg(Surface::Surface) || bg == theme.bg(Surface::Elevated) {
        theme.bg(Surface::Overlay)
    } else if bg == theme.bg(Surface::Field) {
        theme.bg(Surface::FieldHover)
    } else {
        theme.bg(Surface::Popover)
    }
}
const SURFACES: [Surface; 7] = [
    Surface::Canvas,
    Surface::Surface,
    Surface::Elevated,
    Surface::Overlay,
    Surface::Popover,
    Surface::Field,
    Surface::FieldHover,
];
#[test]
fn custom_aliases_use_ordered_policy_and_explicit_rest_stays_literal() {
    for colors in [
        [
            Color::Red,
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Red,
            Color::Magenta,
        ],
        [
            Color::Black,
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Red,
            Color::Red,
        ],
        [
            Color::Black,
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
            Color::Magenta,
        ],
    ] {
        let mut source = Theme::paper();
        source.capability_palettes = None;
        let [
            canvas,
            surface,
            elevated,
            overlay,
            popover,
            field,
            field_hover,
        ] = colors;
        source.color.surfaces = [canvas, surface, elevated, overlay, popover];
        source.color.field = field;
        source.color.field_hover = field_hover;
        source.color.meter.fill_rest = MeterFillRest::ReferenceLift;
        for level in LEVELS {
            for surface in SURFACES {
                for explicit in [false, true] {
                    let mut theme = source.for_level(level);
                    let expected = if explicit {
                        Color::Cyan
                    } else {
                        pinned_lift(&theme, surface)
                    };
                    if explicit {
                        theme.color.meter.fill_rest = MeterFillRest::Color(expected);
                    }
                    let mut scene = Scene::new("meter_custom_alias", theme, level, 4, 1);
                    scene.draw(|ui, area| {
                        ui.with_surface(surface, |ui| {
                            Meter::new(ID)
                                .ratio(0.0)
                                .visual(MeterVisual::Block)
                                .draw(ui, area);
                        });
                    });
                    assert!(
                        scene
                            .buffer()
                            .content
                            .iter()
                            .all(|cell| cell.bg == expected),
                        "{level:?}/{surface:?}/{explicit}"
                    );
                }
            }
        }
    }
}
#[test]
fn delayed_reference_lift_retains_both_channels_and_source_plane_under_dim() {
    for level in LEVELS {
        for surface in SURFACES {
            for dim in 0..=3 {
                let theme = Theme::junie().for_level(level);
                let original = pinned_lift(&theme, surface);
                let neutral = theme.bg(match surface {
                    Surface::Canvas | Surface::Field => Surface::Elevated,
                    _ => Surface::Overlay,
                });
                let mut scene = Scene::new("meter_alias_dim", theme, level, 4, 1);
                scene.draw(|ui, area| {
                    let role = junie_tui::Role::Meter(junie_tui::MeterRole::FillRest);
                    let saved = ui.with_surface(surface, |ui| {
                        ui.paint_patch(&junie_tui::StylePatch::new().set_fg(role).set_bg(role))
                    });
                    ui.with_surface(Surface::Popover, |ui| ui.paint_str(area, "XYZ", saved));
                    ui.dim_layer(area, dim);
                });
                let cell = scene.buffer().cell((0, 0));
                assert_eq!(
                    cell.map(|c| (c.symbol(), c.fg, c.bg)),
                    Some(("X", original, if dim == 0 { original } else { neutral })),
                    "{level:?}/{surface:?}/{dim}"
                );
            }
        }
    }
}
