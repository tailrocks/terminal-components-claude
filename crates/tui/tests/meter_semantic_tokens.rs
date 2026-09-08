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
    let indexed = [233, 235, 235, 237, 237, 234, 237];
    let mono = [
        Color::Black,
        Color::Black,
        Color::Black,
        Color::DarkGray,
        Color::DarkGray,
        Color::Black,
        Color::DarkGray,
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
                ColorLevel::Ansi16 => rgb.get(i).copied().map(|c| downgrade_color(c, level)),
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
    indexed.meter.fill_rest = MeterFillRest::RaisedSurface;
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
        MeterFillRest::RaisedSurface
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
