use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};
use termrock_controls::Button;
use termrock_core::{Id, Part, Response, StateFlags, Variant};
use termrock_runtime::{App, Cx, Runtime, Ui};
use termrock_theme::{
    Align, ColorLevel, Family, GlyphRole, MonoRule, Overlay, OverlayRule, Role, Slot,
    StyleDefaults, StylePatch, Surface, Theme,
};

const FAMILY: Family = Family::custom("profile_contract.consumer");
const BUTTON: Id = Id::root("profile_contract.button");
const AREA: Rect = Rect::new(0, 0, 12, 2);

fn custom_consumer_theme() -> Theme {
    Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Fg(termrock_theme::FgStep::Primary))
                    .set_bg(Role::Accent),
            );
        })
        .define_variant_for_level(ColorLevel::Ansi256, FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Indexed(252)))
                    .set_bg(Role::Custom(Color::Indexed(31))),
            );
        })
}

fn background(theme: &Theme, family: Family, variant: Variant, surface: Surface) -> Option<Color> {
    theme
        .resolve(
            family,
            variant,
            Part::CONTAINER,
            StateFlags::empty(),
            surface,
        )
        .style
        .bg
}

#[test]
fn variant_profile_uses_resolved_default_and_keeps_other_variant_independent() {
    let theme = custom_consumer_theme().for_level(ColorLevel::Ansi256);
    assert_eq!(
        background(&theme, FAMILY, Variant::DEFAULT, Surface::Canvas),
        Some(Color::Indexed(31))
    );

    let other = Family::custom("profile_contract.other");
    let theme = theme
        .define_family(other, |recipe| {
            recipe.default_variant(Variant::SECONDARY);
        })
        .define_variant(other, Variant::SECONDARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::Accent));
        })
        .define_variant_for_level(ColorLevel::Ansi256, other, Variant::SECONDARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(44))));
        });
    let theme = theme.for_level(ColorLevel::Ansi256);
    assert_eq!(
        background(&theme, other, Variant::DEFAULT, Surface::Canvas),
        Some(Color::Indexed(44))
    );
    assert_eq!(
        background(&theme, FAMILY, Variant::DEFAULT, Surface::Canvas),
        Some(Color::Indexed(31))
    );
}

#[test]
fn surface_profile_matches_only_its_semantic_container() {
    let theme = Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::CurrentSurface));
        })
        .define_variant_for_level_on_surface(
            ColorLevel::Ansi256,
            FAMILY,
            Variant::PRIMARY,
            Surface::Surface,
            |recipe| {
                recipe
                    .part(Part::CONTAINER)
                    .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(33))));
            },
        )
        .for_level(ColorLevel::Ansi256);

    assert_eq!(
        background(&theme, FAMILY, Variant::DEFAULT, Surface::Canvas),
        Some(Color::Indexed(16))
    );
    assert_eq!(
        background(&theme, FAMILY, Variant::DEFAULT, Surface::Surface),
        Some(Color::Indexed(33))
    );
}

#[test]
fn semantic_token_mutation_disables_only_dependent_profile_channels_and_cache_rekeys() {
    let mut theme = custom_consumer_theme().for_level(ColorLevel::Ansi256);
    let mut cache = termrock_theme::resolve::StyleCache::default();
    let first = cache.resolve(
        &theme,
        FAMILY,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        &[],
        0,
        Surface::Canvas,
    );
    assert_eq!(first.style.bg, Some(Color::Indexed(31)));

    theme.color.accent = Color::Indexed(45);
    let next = cache.resolve(
        &theme,
        FAMILY,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        &[],
        0,
        Surface::Canvas,
    );
    assert_eq!(next.style.bg, Some(Color::Indexed(45)));
    assert!(
        cache.stats().1 >= 2,
        "token edits must invalidate cached profile eligibility"
    );
}

#[test]
fn same_valued_custom_role_remains_an_explicit_recipe_override() {
    let theme = custom_consumer_theme()
        .override_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(78))));
        })
        .for_level(ColorLevel::Ansi256);

    assert_eq!(
        background(&theme, FAMILY, Variant::PRIMARY, Surface::Canvas),
        Some(Color::Indexed(78)),
        "the authored Custom role must win even when it equals the semantic Accent token"
    );
}

#[test]
fn clear_and_remove_overrides_win_for_every_profile_channel() {
    let theme = Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::Accent)
                    .set_underline(Role::Accent)
                    .add(Modifier::BOLD)
                    .set_glyph(GlyphRole::Chosen)
                    .set_size(2)
                    .set_align(Align::Right),
            );
        })
        .define_variant_for_level(ColorLevel::Ansi256, FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Indexed(201)))
                    .set_bg(Role::Custom(Color::Indexed(202)))
                    .set_underline(Role::Custom(Color::Indexed(203)))
                    .add(Modifier::BOLD)
                    .set_glyph(GlyphRole::Error)
                    .set_size(3)
                    .set_align(Align::Center),
            );
        })
        .for_level(ColorLevel::Ansi256);
    let local = StylePatch {
        underline: Slot::Clear,
        glyph: Slot::Clear,
        size: Slot::Clear,
        align: Slot::Clear,
        ..StylePatch::new()
            .clear_fg()
            .clear_bg()
            .remove(Modifier::BOLD)
    };

    let resolved = termrock_theme::resolve::resolve_uncached(
        &theme,
        FAMILY,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        Surface::Canvas,
        &[],
        Some(&local),
    );

    let style = resolved.style.as_style();
    assert_eq!(style.fg, None);
    assert_eq!(style.bg, None);
    assert_eq!(style.underline_color, None);
    assert!(!style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(resolved.glyph, Slot::Clear);
    assert_eq!(resolved.size, None);
    assert_eq!(resolved.align, None);
}

#[test]
fn mono_profile_channels_survive_generic_fallback_but_authored_rules_win() {
    const MONO_RULES: [MonoRule; 1] = [(
        Part::CONTAINER,
        StateFlags::PRESSED,
        StylePatch::new().set_bg(Role::Custom(Color::Indexed(37))),
    )];
    let theme = Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::CurrentSurface));
        })
        .define_variant_for_level(ColorLevel::Mono, FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::Custom(Color::White)));
        })
        .for_level(ColorLevel::Mono);

    let profile_only = termrock_theme::resolve::bind_defaults(
        &theme,
        (FAMILY, Variant::DEFAULT, Part::CONTAINER),
        StateFlags::PRESSED,
        &[],
        Surface::Canvas,
        StyleDefaults::default(),
        None,
    );
    assert_eq!(profile_only.style.bg, Some(Color::White));

    let profile = termrock_theme::resolve::bind_defaults(
        &theme,
        (FAMILY, Variant::DEFAULT, Part::CONTAINER),
        StateFlags::PRESSED,
        &[],
        Surface::Canvas,
        StyleDefaults::default().mono(&MONO_RULES),
        None,
    );

    assert_eq!(
        profile.style.bg,
        termrock_theme::resolve::bind_role(
            &theme,
            Role::Custom(Color::Indexed(37)),
            Surface::Canvas,
        )
    );
}

#[test]
fn mono_authored_custom_defaults_keep_canonical_binding_without_profiles() {
    const MONO_RULES: [MonoRule; 1] = [(
        Part::CONTAINER,
        StateFlags::PRESSED,
        StylePatch::new().set_bg(Role::Custom(Color::Indexed(37))),
    )];
    let theme = Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::AccentPressed));
        })
        .for_level(ColorLevel::Mono);
    let expected = termrock_theme::resolve::bind_role(
        &theme,
        Role::Custom(Color::Indexed(37)),
        Surface::Canvas,
    );

    let defaults = StyleDefaults::default().mono(&MONO_RULES);
    let bound = termrock_theme::resolve::bind_defaults(
        &theme,
        (FAMILY, Variant::PRIMARY, Part::CONTAINER),
        StateFlags::PRESSED,
        &[],
        Surface::Canvas,
        defaults,
        None,
    );
    let with_explicit = termrock_theme::resolve::bind_defaults_with_explicit(
        &theme,
        (FAMILY, Variant::PRIMARY, Part::CONTAINER),
        StateFlags::PRESSED,
        &[],
        Surface::Canvas,
        defaults,
        None,
    );

    assert_eq!(bound.style.bg, expected);
    assert_eq!(with_explicit.resolved.style.bg, expected);
}

#[test]
fn same_valued_authored_mono_channels_take_ownership_over_exact_profile_channels() {
    const MONO_RULES: [MonoRule; 1] = [(
        Part::CONTAINER,
        StateFlags::PRESSED,
        StylePatch::new()
            .set_fg(Role::Custom(Color::DarkGray))
            .set_bg(Role::Custom(Color::Gray))
            .set_underline(Role::Custom(Color::White)),
    )];
    let theme = Theme::junie()
        .define_family(FAMILY, |recipe| {
            recipe.default_variant(Variant::PRIMARY);
        })
        .define_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Accent)
                    .set_bg(Role::Accent)
                    .set_underline(Role::Accent),
            );
        })
        .define_variant_for_level(ColorLevel::Mono, FAMILY, Variant::PRIMARY, |recipe| {
            recipe.part(Part::CONTAINER).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::DarkGray))
                    .set_bg(Role::Custom(Color::Gray))
                    .set_underline(Role::Custom(Color::White)),
            );
        })
        .for_level(ColorLevel::Mono);
    let defaults = StyleDefaults::default().mono(&MONO_RULES);
    let flags = StateFlags::PRESSED;
    let expected = (
        termrock_theme::resolve::bind_role(&theme, Role::Custom(Color::DarkGray), Surface::Canvas),
        termrock_theme::resolve::bind_role(&theme, Role::Custom(Color::Gray), Surface::Canvas),
        termrock_theme::resolve::bind_role(&theme, Role::Custom(Color::White), Surface::Canvas),
    );

    let resolved = termrock_theme::resolve::bind_defaults(
        &theme,
        (FAMILY, Variant::DEFAULT, Part::CONTAINER),
        flags,
        &[],
        Surface::Canvas,
        defaults,
        None,
    );
    let with_explicit = termrock_theme::resolve::bind_defaults_with_explicit(
        &theme,
        (FAMILY, Variant::DEFAULT, Part::CONTAINER),
        flags,
        &[],
        Surface::Canvas,
        defaults,
        None,
    );

    for style in [resolved.style, with_explicit.resolved.style] {
        assert_eq!((style.fg, style.bg, style.underline_color), expected);
    }
}

#[test]
fn ordinary_overrides_keep_family_variant_scope_and_instance_precedence() {
    static RULES: [OverlayRule; 1] = [(
        FAMILY,
        Variant::PRIMARY,
        Part::CONTAINER,
        StateFlags::empty(),
        StylePatch::new().set_bg(Role::Custom(Color::Indexed(35))),
    )];
    let theme = custom_consumer_theme()
        .define_variant_for_level_on_surface(
            ColorLevel::Ansi256,
            FAMILY,
            Variant::PRIMARY,
            Surface::Surface,
            |recipe| {
                recipe
                    .part(Part::CONTAINER)
                    .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(33))));
            },
        )
        .override_variant(FAMILY, Variant::PRIMARY, |recipe| {
            recipe
                .part(Part::CONTAINER)
                .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(34))));
        })
        .for_level(ColorLevel::Ansi256);
    assert_eq!(
        background(&theme, FAMILY, Variant::PRIMARY, Surface::Canvas),
        Some(Color::Indexed(34))
    );

    let overlay = Overlay::new(&RULES);
    let resolved = termrock_theme::resolve::resolve_uncached(
        &theme,
        FAMILY,
        Variant::PRIMARY,
        Part::CONTAINER,
        StateFlags::empty(),
        Surface::Surface,
        &[overlay],
        Some(&StylePatch::new().set_bg(Role::Custom(Color::Indexed(36)))),
    );
    assert_eq!(resolved.style.bg, Some(Color::Indexed(36)));

    let scoped = termrock_theme::resolve::resolve_uncached(
        &theme,
        FAMILY,
        Variant::PRIMARY,
        Part::CONTAINER,
        StateFlags::empty(),
        Surface::Surface,
        &[overlay],
        None,
    );
    assert_eq!(scoped.style.bg, Some(Color::Indexed(35)));
}

struct ProfileButton;

impl App for ProfileButton {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let _ = Button::new(BUTTON, "profile")
            .variant(Variant::PRIMARY)
            .draw(ui, AREA);
    }
}

#[test]
fn public_button_render_uses_cached_variant_profile() {
    let theme = Theme::junie()
        .define_variant_for_level(
            ColorLevel::Ansi256,
            Family::BUTTON,
            Variant::PRIMARY,
            |recipe| {
                recipe
                    .part(Part::CONTAINER)
                    .base(StylePatch::new().set_bg(Role::Custom(Color::Indexed(65))));
            },
        )
        .for_level(ColorLevel::Ansi256);
    let mut runtime = Runtime::new(ProfileButton, theme);
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(
        buffer
            .content
            .iter()
            .any(|cell| cell.bg == Color::Indexed(65))
    );
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(
        buffer
            .content
            .iter()
            .any(|cell| cell.bg == Color::Indexed(65)),
        "cached button styles must retain the variant profile"
    );
}
