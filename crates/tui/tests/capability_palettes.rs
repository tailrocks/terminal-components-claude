//! Authored semantic output preserves Junie while generic RGB stays generic.
use junie_tui::theme::downgrade_color;
use junie_tui::{CapabilityPalettes, ColorLevel, Theme};
use ratatui_core::style::Color;

// Independent immutable Holla 794b095 src/theme.rs587-600 formula.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "exact pinned reference formula over bounded u8 channels"
)]
fn reference256(r: u8, g: u8, b: u8) -> Color {
    let step = |v: u8| (u32::from(v) * 5 + 127) / 255;
    let cube = 16 + 36 * step(r) + 6 * step(g) + step(b);
    let value = |i: u32| if i == 0 { 0 } else { 55 + i as i32 * 40 };
    let cr = value(step(r));
    let cg = value(step(g));
    let cb = value(step(b));
    let cube_error =
        (cr - i32::from(r)).pow(2) + (cg - i32::from(g)).pow(2) + (cb - i32::from(b)).pow(2);
    let avg = (i32::from(r) + i32::from(g) + i32::from(b)) / 3;
    let gi = ((avg - 8).max(0) / 10).min(23);
    let gray = 8 + gi * 10;
    let gray_error =
        (gray - i32::from(r)).pow(2) + (gray - i32::from(g)).pow(2) + (gray - i32::from(b)).pow(2);
    Color::Indexed(if gray_error < cube_error {
        (232 + gi) as u8
    } else {
        cube as u8
    })
}
#[expect(
    clippy::arithmetic_side_effects,
    reason = "sum of three u8 channels fits u32"
)]
fn reference_mono(r: u8, g: u8, b: u8) -> Color {
    match (u32::from(r) + u32::from(g) + u32::from(b)) / 3 {
        0..=40 => Color::Black,
        41..=110 => Color::DarkGray,
        111..=190 => Color::Gray,
        _ => Color::White,
    }
}

#[test]
fn all_73_junie_slots_match_reference_capability_output() {
    let original = Theme::junie();
    assert_eq!(original.color.colors().len(), 73);
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let expected = original
            .color
            .map_colors(&mut |color| match (level, color) {
                (ColorLevel::Ansi256, Color::Rgb(r, g, b)) => reference256(r, g, b),
                (ColorLevel::Mono, Color::Rgb(r, g, b)) => reference_mono(r, g, b),
                _ => downgrade_color(color, level),
            });
        assert_eq!(original.downgrade(level).color, expected, "{level:?}");
    }
    assert_eq!(
        downgrade_color(Color::Rgb(72, 224, 84), ColorLevel::Ansi256),
        Color::Indexed(77)
    );
    assert_eq!(
        original.downgrade(ColorLevel::Ansi256).color.accent,
        Color::Indexed(78)
    );
}

#[test]
fn repeated_and_chained_downgrade_keep_authored_tokens() {
    let original = Theme::junie();
    for first in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let narrowed = original.for_level(first);
        assert_eq!(narrowed, narrowed.downgrade(first));
        assert_eq!(narrowed, narrowed.for_level(ColorLevel::TrueColor));
        assert_eq!(
            narrowed.for_level(ColorLevel::Mono).color,
            original.for_level(ColorLevel::Mono).color
        );
    }
}

#[test]
fn every_four_step_capability_path_preserves_all_authored_mono_slots() {
    const LEVELS: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    let source = Theme::junie();
    let expected = source.for_level(ColorLevel::Mono).color;
    for a in LEVELS {
        for b in LEVELS {
            for c in LEVELS {
                for d in LEVELS {
                    for narrow_only in [false, true] {
                        let mut theme = source.clone();
                        for level in [a, b, c, d] {
                            theme = if narrow_only {
                                theme.for_level(level)
                            } else {
                                theme.downgrade(level)
                            };
                            assert_eq!(theme, theme.downgrade(theme.capability.color));
                        }
                        assert_eq!(
                            theme.for_level(ColorLevel::Mono).color,
                            expected,
                            "path {a:?}/{b:?}/{c:?}/{d:?}, narrow_only={narrow_only}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn mutations_at_each_intermediate_level_never_reattach_or_detach_other_slots() {
    for (prefix, changed) in [
        (0, Color::Rgb(95, 215, 135)),
        (1, Color::Indexed(77)),
        (2, Color::LightBlue),
    ] {
        let source = Theme::junie();
        let mut theme = source.clone();
        let path = [ColorLevel::Ansi256, ColorLevel::Ansi16, ColorLevel::Mono];
        for level in path.iter().take(prefix) {
            theme = theme.for_level(*level);
        }
        theme.color.accent = changed;
        let mut expected_accent = changed;
        for level in path.iter().skip(prefix) {
            expected_accent = downgrade_color(expected_accent, *level);
            theme = theme.for_level(*level);
        }
        let mut expected = source.for_level(ColorLevel::Mono).color;
        expected.accent = expected_accent;
        assert_eq!(theme.color, expected, "mutation after {prefix} projections");
    }
}

#[test]
fn direct_token_mutation_and_quantization_collision_never_reattach() {
    let original = Theme::junie();
    let mut custom = original.clone();
    // This RGB is exactly palette78. It differs from the authored source,
    // yet quantizes to the same authored256 output; provenance must survive.
    let changed = Color::Rgb(95, 215, 135);
    custom.color.accent = changed;
    let narrowed = custom.for_level(ColorLevel::Ansi256);
    assert_eq!(narrowed.color.accent, Color::Indexed(78));
    let authored = original.for_level(ColorLevel::Ansi256);
    assert_eq!(narrowed.color, authored.color);
    assert_ne!(
        narrowed.fingerprint(),
        authored.fingerprint(),
        "equal quantized colors retain different eligibility provenance"
    );
    assert_eq!(
        narrowed.for_level(ColorLevel::Mono).color.accent,
        downgrade_color(Color::Indexed(78), ColorLevel::Mono)
    );
    assert_ne!(
        narrowed.for_level(ColorLevel::Mono).color.accent,
        original.for_level(ColorLevel::Mono).color.accent
    );
    assert_eq!(
        original.for_level(ColorLevel::Ansi256).color.focus,
        Color::Indexed(78)
    );
    assert_eq!(custom.color.focus, original.color.focus);
    assert_eq!(original.color.accent, Color::Rgb(72, 224, 84));
    if let Some(palettes) = &narrowed.capability_palettes {
        let reset = narrowed
            .clone()
            .builder()
            .capability_palettes((**palettes).clone())
            .build();
        assert_eq!(
            reset.for_level(ColorLevel::Mono).color.accent,
            original.for_level(ColorLevel::Mono).color.accent
        );
    } else {
        assert!(narrowed.capability_palettes.is_some());
    }
}

#[test]
fn builder_dependants_fall_back_and_non_junie_stays_generic() {
    let changed = Theme::junie()
        .builder()
        .accent(Color::Rgb(150, 40, 220))
        .build();
    let generic = Theme::from_tokens(changed.color);
    for level in [ColorLevel::Ansi256, ColorLevel::Ansi16, ColorLevel::Mono] {
        let actual = changed.for_level(level);
        let expected = generic.for_level(level);
        assert_eq!(actual.color.accent, expected.color.accent);
        assert_eq!(actual.color.accent_hover, expected.color.accent_hover);
        assert_eq!(actual.color.accent_pressed, expected.color.accent_pressed);
        assert_eq!(actual.color.focus, expected.color.focus);
        assert_eq!(actual.color.focus_ring, expected.color.focus_ring);
    }
    let paper = Theme::paper();
    assert!(paper.capability_palettes.is_none());
    let custom = Theme::from_tokens(Theme::junie().color);
    assert!(custom.capability_palettes.is_none());
    for level in [ColorLevel::Ansi256, ColorLevel::Mono] {
        assert_eq!(
            paper.for_level(level).color,
            paper.color.map_colors(&mut |c| downgrade_color(c, level))
        );
        assert_eq!(
            custom.for_level(level).color,
            custom.color.map_colors(&mut |c| downgrade_color(c, level))
        );
    }
}

#[test]
fn explicit_palette_contents_fingerprint_and_missing_level_fallback() {
    let original = Theme::from_tokens(Theme::junie().color);
    let mut target = original
        .color
        .map_colors(&mut |c| downgrade_color(c, ColorLevel::Ansi256));
    target.accent = Color::Indexed(99);
    target.focus = Color::Indexed(100);
    let authored = original
        .clone()
        .builder()
        .capability_palettes(
            CapabilityPalettes::new(original.color).with(ColorLevel::Ansi256, target),
        )
        .build();
    assert_ne!(original.fingerprint(), authored.fingerprint());
    assert_eq!(
        authored.for_level(ColorLevel::Ansi256).color.accent,
        Color::Indexed(99)
    );
    // Identical source RGB in these two roles cannot alias their authored output.
    assert_eq!(
        authored.for_level(ColorLevel::Ansi256).color.focus,
        Color::Indexed(100)
    );
    assert_eq!(
        authored.for_level(ColorLevel::Mono).color,
        original.for_level(ColorLevel::Mono).color
    );
    let cleared = authored
        .clone()
        .builder()
        .clear_capability_palettes()
        .build();
    assert_eq!(cleared, original);
    let second = original
        .clone()
        .builder()
        .capability_palettes(
            CapabilityPalettes::new(original.color).with(ColorLevel::Ansi256, target),
        )
        .build();
    assert_eq!(authored.fingerprint(), second.fingerprint());
    target.accent = Color::Indexed(100);
    let different = original
        .clone()
        .builder()
        .capability_palettes(
            CapabilityPalettes::new(original.color).with(ColorLevel::Ansi256, target),
        )
        .build();
    assert_ne!(authored.fingerprint(), different.fingerprint());
}

#[test]
fn direct_mutation_after_narrowing_detaches_only_that_semantic_slot() {
    let mut theme = Theme::junie().for_level(ColorLevel::Ansi256);
    theme.color.focus = Color::Indexed(99);
    let mono = theme.for_level(ColorLevel::Mono);
    assert_eq!(
        mono.color.focus,
        downgrade_color(Color::Indexed(99), ColorLevel::Mono)
    );
    assert_eq!(mono.color.accent, Color::Gray);
}

#[test]
fn production_theme_replacement_invalidates_palette_derived_style_cache() {
    use junie_tui::{
        App, Cx, Family, Part, Rect, Response, Role, Runtime, StateFlags, StylePatch, Ui, Variant,
    };
    struct Page;
    impl App for Page {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let style = ui
                .style(
                    Family::BUTTON,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                )
                .style;
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", style);
        }
    }
    fn themed(index: u8) -> Theme {
        let base = Theme::from_tokens(Theme::junie().color);
        let mut colors = base.downgrade(ColorLevel::Ansi256).color;
        colors.accent = Color::Indexed(index);
        base.clone()
            .builder()
            .capability_palettes(
                CapabilityPalettes::new(base.color).with(ColorLevel::Ansi256, colors),
            )
            .build()
            .override_family(Family::BUTTON, |recipe| {
                recipe
                    .part(Part::LABEL)
                    .base(StylePatch::new().set_fg(Role::Accent));
            })
            .for_level(ColorLevel::Ansi256)
    }
    let mut runtime = Runtime::new(Page, themed(99));
    let area = Rect::new(0, 0, 1, 1);
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer);
    runtime.draw_buffer(area, &mut buffer);
    assert_eq!(buffer.cell((0, 0)).map(|c| c.fg), Some(Color::Indexed(99)));
    runtime.set_theme(themed(100));
    runtime.draw_buffer(area, &mut buffer);
    assert_eq!(buffer.cell((0, 0)).map(|c| c.fg), Some(Color::Indexed(100)));
}
