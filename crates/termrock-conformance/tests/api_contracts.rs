#[test]
fn public_api_contracts_reject_invalid_usage() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/api_contracts/*.rs");
}

#[test]
fn authored_monochrome_palette_and_generic_fallback_are_public() {
    use ratatui::style::Color;
    use termrock::theme::{CapabilityPalettes, ColorLevel, Theme};

    let base = Theme::from_tokens(Theme::junie().color);
    let mut mono = base.for_level(ColorLevel::Mono).color;
    mono.accent = Color::Gray;
    let authored = base
        .clone()
        .builder()
        .capability_palettes(CapabilityPalettes::new(base.color).with(ColorLevel::Mono, mono))
        .build();
    assert_eq!(
        authored.for_level(ColorLevel::Mono).color.accent,
        Color::Gray
    );

    let generic = Theme {
        color: base.color,
        design: base.design,
        recipes: base.recipes,
        capability: base.capability,
        capability_palettes: None,
    };
    assert!(generic.capability_palettes.is_none());
}

#[test]
fn mono_rules_can_select_chosen_glyph() {
    use termrock::theme::{
        ColorLevel, Family, GlyphRole, Part, Slot, StateFlags, StylePatch, Surface, Theme, Variant,
    };

    let family = Family::custom("segment");
    let theme = Theme::junie()
        .builder()
        .mono_rules(
            family,
            &[(
                Part::MARKER,
                StateFlags::ACTIVE,
                StylePatch::new().set_glyph(GlyphRole::Chosen),
            )],
        )
        .build()
        .downgrade(ColorLevel::Mono);

    let resolved = theme.resolve(
        family,
        Variant::DEFAULT,
        Part::MARKER,
        StateFlags::ACTIVE,
        Surface::Canvas,
    );
    assert_eq!(resolved.glyph, Slot::Set(GlyphRole::Chosen));
}
