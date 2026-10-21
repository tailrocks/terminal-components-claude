use ratatui_core::symbols::{line, scrollbar};
use termrock_theme::{GlyphRole, GlyphSet, Theme};

#[test]
fn external_legacy_constructor_and_all_appended_roles_remain_safe() {
    // Keep an external consumer using the exact pre-extension constructor shape.
    let mut glyphs = GlyphSet::new(
        ["legacy"; 41],
        scrollbar::VERTICAL,
        line::NORMAL,
        line::THICK,
    );
    assert_eq!(GlyphRole::PrimaryKey as usize, 41);
    assert_eq!(GlyphRole::DotOn as usize, 42);
    assert_eq!(GlyphRole::DotOff as usize, 43);
    assert_eq!(glyphs.get(GlyphRole::PrimaryKey), "⚷");
    assert_eq!(glyphs.get(GlyphRole::DotOn), "●");
    assert_eq!(glyphs.get(GlyphRole::DotOff), "○");

    for role in GlyphRole::ALL {
        glyphs.set(role, "probe");
        assert_eq!(glyphs.get(role), "probe", "{role:?}");
    }
}

#[test]
fn built_in_dots_are_one_cell_and_radio_roles_remain_three_cell_pairs() {
    let glyphs = &Theme::junie().design.glyphs;
    assert_eq!(glyphs.get(GlyphRole::DotOn), "●");
    assert_eq!(glyphs.get(GlyphRole::DotOff), "○");
    assert_eq!(glyphs.get(GlyphRole::RadioOn), "(●)");
    assert_eq!(glyphs.get(GlyphRole::RadioOff), "(○)");
}
