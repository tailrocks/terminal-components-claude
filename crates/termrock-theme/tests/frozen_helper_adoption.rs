//! Adoption example: the upgraded helpers reproduce the frozen widget ink.
//!
//! For every frozen paint site the audit measured (migration matrix,
//! 2026-10-21; truth tables in `crates/termrock-controls/tests/
//! frozen_ink_pins.rs` and `crates/termrock-navigation/tests/
//! frozen_ink_pins.rs`), this renders the same gutter cell twice — once
//! through the frozen recipe path (`Theme::resolve`) and once through the
//! upgraded helpers (`Theme::focus_bar_symbol` + `Theme::gutter_in` with
//! `GutterCell::Inherit`) — into a one-cell buffer and asserts the cells
//! equal. Marker ink goes through `Theme::chosen_in_with` with
//! `BackdropSelection::Accent`. A controls/navigation paint site can now
//! adopt the helpers byte-identically at these states.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_core::id::Part;
use termrock_core::response::StateFlags;
use termrock_theme::{
    BackdropSelection, ColorLevel, FocusBarPolicy, GlyphRole, InactiveSelection, Slot, Surface,
    Theme, Variant,
};

/// One painted gutter cell: glyph plus style, both rendering routes.
fn painted_cell(
    theme: &Theme,
    family: termrock_theme::Family,
    flags: StateFlags,
) -> (String, ratatui_core::style::Style) {
    let resolved = theme.resolve(
        family,
        Variant::DEFAULT,
        Part::GUTTER,
        flags,
        Surface::Canvas,
    );
    let glyph = match resolved.glyph {
        Slot::Set(GlyphRole::FocusBar) => theme.design.glyphs.get(GlyphRole::FocusBar),
        _ => " ",
    };
    // The frozen painter shows the bar ink only when the bar paints; a blank
    // gutter inherits the container cell untouched.
    let style = if glyph == " " {
        ratatui_core::style::Style::new()
    } else {
        *resolved.style.as_style()
    };
    (glyph.to_string(), style)
}

fn helper_cell(
    theme: &Theme,
    policy: FocusBarPolicy,
    flags: StateFlags,
) -> (String, ratatui_core::style::Style) {
    let glyph = theme.focus_bar_symbol(policy, flags);
    let style = theme.gutter_in(
        policy,
        flags,
        termrock_theme::GutterCell::Inherit,
        theme.color.surfaces[0],
        false,
    );
    (glyph.to_string(), style)
}

fn assert_cells_equal(
    theme: &Theme,
    family: termrock_theme::Family,
    policy: FocusBarPolicy,
    flags: StateFlags,
    level: ColorLevel,
) {
    let (recipe_glyph, recipe_style) = painted_cell(theme, family, flags);
    let (helper_glyph, helper_style) = helper_cell(theme, policy, flags);
    let mut recipe_buf = Buffer::empty(Rect::new(0, 0, 1, 1));
    let mut helper_buf = Buffer::empty(Rect::new(0, 0, 1, 1));
    recipe_buf[(0, 0)]
        .set_symbol(&recipe_glyph)
        .set_style(recipe_style);
    helper_buf[(0, 0)]
        .set_symbol(&helper_glyph)
        .set_style(helper_style);
    assert_eq!(
        recipe_buf[(0, 0)],
        helper_buf[(0, 0)],
        "{level:?} {family:?} {policy:?} {flags:?}: the adopted helper cell must equal the frozen recipe cell"
    );
}

#[test]
fn button_adopts_the_focus_visible_policy_byte_identically() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        for flags in [
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::DISABLED,
            StateFlags::FOCUSED,
            StateFlags::FOCUSED | StateFlags::DISABLED,
            StateFlags::empty(),
        ] {
            assert_cells_equal(
                &theme,
                termrock_theme::Family::BUTTON,
                FocusBarPolicy::FocusVisible,
                flags,
                level,
            );
        }
    }
}

#[test]
fn kept_rows_adopt_row_kept_and_list_adopts_row_blanked() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        for family in [
            termrock_theme::Family::CHOICE,
            termrock_theme::Family::TREE,
            termrock_theme::Family::STEPS,
        ] {
            for flags in [
                StateFlags::FOCUSED,
                StateFlags::FOCUSED | StateFlags::DISABLED,
                StateFlags::empty(),
            ] {
                assert_cells_equal(&theme, family, FocusBarPolicy::RowKept, flags, level);
            }
        }
        for flags in [
            StateFlags::FOCUSED,
            StateFlags::FOCUSED | StateFlags::DISABLED,
            StateFlags::empty(),
        ] {
            assert_cells_equal(
                &theme,
                termrock_theme::Family::LIST,
                FocusBarPolicy::RowBlanked,
                flags,
                level,
            );
        }
    }
}

#[test]
fn checked_marker_ink_adopts_chosen_in_with_accent_backdrop() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        for family in [termrock_theme::Family::CHOICE, termrock_theme::Family::LIST] {
            for flags in [
                StateFlags::CHECKED,
                StateFlags::CHECKED | StateFlags::FOCUSED,
            ] {
                let resolved = theme.resolve(
                    family,
                    Variant::DEFAULT,
                    Part::MARKER,
                    flags,
                    Surface::Canvas,
                );
                assert_eq!(
                    resolved.style.as_style().fg,
                    Some(theme.chosen_in_with(
                        InactiveSelection::Accent,
                        true,
                        BackdropSelection::Accent,
                        flags
                    )),
                    "{level:?} {family:?} {flags:?}: the adopted marker ink equals the frozen ink, backdrop included"
                );
            }
        }
    }
}
