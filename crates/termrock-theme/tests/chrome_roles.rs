use ratatui_core::style::Color;
use termrock_runtime::Rect;
use termrock_test_support::Scene;
use termrock_theme::{
    ColorLevel, ColorTokens, Family, Part, Role, StateFlags, StylePatch, Surface, Theme, Variant,
    downgrade_color,
};

const CHROME_ROLES: [Role; 7] = [
    Role::StatusBarBg,
    Role::StatusBarFg,
    Role::StatusAccent,
    Role::ChromeRule,
    Role::ScrollTrack,
    Role::ScrollThumb,
    Role::TagInk,
];

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn token_color(tokens: &ColorTokens, role: Role) -> Color {
    match role {
        Role::StatusBarBg => tokens.status_bar_bg,
        Role::StatusBarFg => tokens.status_bar_fg,
        Role::StatusAccent => tokens.status_accent,
        Role::ChromeRule => tokens.chrome_rule,
        Role::ScrollTrack => tokens.scroll_track,
        Role::ScrollThumb => tokens.scroll_thumb,
        Role::TagInk => tokens.tag_ink,
        _ => Color::Reset,
    }
}

fn set_token_color(tokens: &mut ColorTokens, role: Role, color: Color) {
    match role {
        Role::StatusBarBg => tokens.status_bar_bg = color,
        Role::StatusBarFg => tokens.status_bar_fg = color,
        Role::StatusAccent => tokens.status_accent = color,
        Role::ChromeRule => tokens.chrome_rule = color,
        Role::ScrollTrack => tokens.scroll_track = color,
        Role::ScrollThumb => tokens.scroll_thumb = color,
        Role::TagInk => tokens.tag_ink = color,
        _ => (),
    }
}

fn without_chrome(tokens: ColorTokens) -> ColorTokens {
    ColorTokens {
        status_bar_bg: Color::Reset,
        status_bar_fg: Color::Reset,
        status_accent: Color::Reset,
        chrome_rule: Color::Reset,
        scroll_track: Color::Reset,
        scroll_thumb: Color::Reset,
        tag_ink: Color::Reset,
        ..tokens
    }
}

fn resolve_fg(theme: Theme, role: Role) -> Option<Color> {
    theme
        .override_family(Family::BUTTON, |recipe| {
            recipe
                .part(Part::LABEL)
                .base(StylePatch::new().set_fg(role));
        })
        .resolve(
            Family::BUTTON,
            Variant::PRIMARY,
            Part::LABEL,
            StateFlags::empty(),
            Surface::Canvas,
        )
        .style
        .as_style()
        .fg
}

#[test]
fn every_chrome_role_binds_and_resolves_for_both_themes_at_each_level() {
    for source in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            let theme = source.clone().for_level(level);
            for role in CHROME_ROLES {
                let expected = token_color(&theme.color, role);
                let expected_binding = (expected != Color::Reset).then_some(expected);
                assert_eq!(
                    termrock_theme::resolve::bind_role(&theme, role, Surface::Canvas),
                    expected_binding,
                    "{role:?} at {level:?}"
                );
                assert_eq!(
                    resolve_fg(theme.clone(), role),
                    expected_binding,
                    "resolved {role:?} at {level:?}"
                );
            }
        }
    }
}

#[test]
fn reset_chrome_tokens_derive_and_explicit_custom_tokens_survive() {
    let source = Theme::junie().color;
    let mut tokens = source;
    for role in CHROME_ROLES {
        set_token_color(&mut tokens, role, Color::Reset);
    }

    let derived = Theme::from_tokens(tokens);
    for (role, expected) in [
        (Role::StatusBarBg, source.surfaces[1]),
        (Role::StatusBarFg, source.fg[0]),
        (Role::StatusAccent, source.accent),
        (Role::ChromeRule, source.border_subtle),
        (Role::ScrollTrack, source.border_subtle),
        (Role::ScrollThumb, source.fg[2]),
        (Role::TagInk, source.fg[2]),
    ] {
        assert_eq!(token_color(&derived.color, role), expected, "{role:?}");
    }

    let custom = [
        Color::Rgb(1, 2, 3),
        Color::Rgb(4, 5, 6),
        Color::Rgb(7, 8, 9),
        Color::Rgb(10, 11, 12),
        Color::Rgb(13, 14, 15),
        Color::Rgb(16, 17, 18),
        Color::Rgb(19, 20, 21),
    ];
    let mut customized = tokens;
    for (role, color) in CHROME_ROLES.into_iter().zip(custom) {
        set_token_color(&mut customized, role, color);
    }
    let customized = Theme::from_tokens(customized);
    for (role, color) in CHROME_ROLES.into_iter().zip(custom) {
        assert_eq!(token_color(&customized.color, role), color, "{role:?}");
    }
    assert_eq!(
        resolve_fg(customized, Role::StatusAccent),
        Some(Color::Rgb(7, 8, 9))
    );
}

#[test]
fn each_chrome_channel_projects_independently_after_every_intermediate_mutation() {
    let custom_colors = [
        Color::Rgb(31, 71, 191),
        Color::Rgb(47, 89, 173),
        Color::Rgb(61, 107, 157),
        Color::Rgb(73, 127, 139),
        Color::Rgb(89, 149, 113),
        Color::Rgb(101, 167, 97),
        Color::Rgb(113, 181, 79),
    ];

    for (role, custom) in CHROME_ROLES.into_iter().zip(custom_colors) {
        for mutation_step in 0..LEVELS.len() {
            let source = Theme::from_tokens(Theme::junie().color);
            let mut changed = source.clone();
            let mut baseline = source;

            for level in LEVELS.iter().take(mutation_step) {
                changed = changed.for_level(*level);
                baseline = baseline.for_level(*level);
            }
            set_token_color(&mut changed.color, role, custom);

            let mut expected_custom = custom;
            for level in LEVELS.iter().skip(mutation_step) {
                expected_custom = downgrade_color(expected_custom, *level);
                changed = changed.for_level(*level);
                baseline = baseline.for_level(*level);

                assert_eq!(
                    token_color(&changed.color, role),
                    expected_custom,
                    "{role:?}, mutation step {mutation_step}, level {level:?}"
                );
                assert_eq!(
                    without_chrome(changed.color),
                    without_chrome(baseline.color),
                    "old channels changed for {role:?}, mutation step {mutation_step}, level {level:?}"
                );
            }
        }
    }
}

#[test]
fn rendered_cells_use_each_chrome_role_in_both_themes_at_every_level() {
    const FOREGROUND_ROLES: [Role; 6] = [
        Role::StatusBarFg,
        Role::StatusAccent,
        Role::ChromeRule,
        Role::ScrollTrack,
        Role::ScrollThumb,
        Role::TagInk,
    ];

    for source in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            let projected = source.clone().for_level(level);
            let expected_bg = projected.color.status_bar_bg;
            let mut scene = Scene::new("chrome-roles", source.clone(), level, 6, 1);
            scene.draw(|ui, _| {
                for (column, role) in FOREGROUND_ROLES.into_iter().enumerate() {
                    let Ok(column) = u16::try_from(column) else {
                        continue;
                    };
                    let style =
                        ui.paint_patch(&StylePatch::new().set_fg(role).set_bg(Role::StatusBarBg));
                    ui.paint_str(Rect::new(column, 0, 1, 1), "X", style);
                }
            });

            for (column, role) in FOREGROUND_ROLES.into_iter().enumerate() {
                let Ok(column) = u16::try_from(column) else {
                    continue;
                };
                assert_eq!(
                    scene
                        .buffer()
                        .cell((column, 0))
                        .map(|cell| (cell.fg, cell.bg)),
                    Some((token_color(&projected.color, role), expected_bg)),
                    "{role:?} at {level:?}"
                );
            }
        }
    }
}
