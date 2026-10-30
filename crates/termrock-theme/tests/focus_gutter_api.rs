//! Theme-level port of the PR #26 reference `tests/focus_gutter.rs`
//! (f4867e9b5f8a08a3bb61ce80d644198a29f51984).
//!
//! The reference drives whole widgets (Button, ListBox, TextInput); the parity
//! port pins the shared theme API those widgets must adopt
//! (`gutter_symbol`, `gutter`, `chosen_in`, `inactive_look`, `legible_mark`).
//! Widget-level adoption is tracked as a handoff: controls currently paint
//! gutters through the recipe system (`Part::GUTTER`) and apps hand-paint
//! `▎`.

use termrock_theme::{
    COLOR_LEVELS, GlyphRole, InactiveSelection, StateFlags, Theme, contrast_ratio,
};

fn focused() -> StateFlags {
    StateFlags::FOCUSED
}

fn focused_disabled() -> StateFlags {
    StateFlags::FOCUSED | StateFlags::DISABLED
}

fn idle() -> StateFlags {
    StateFlags::empty()
}

#[test]
fn the_focus_gutter_is_the_bar_only_while_focused_and_enabled() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let bar = theme.design.glyphs.get(GlyphRole::FocusBar);
        assert_eq!(bar, "▎", "{level:?}: the focus bar glyph is the phosphor ▎");
        assert_eq!(theme.gutter_symbol(focused()), bar, "{level:?}");
        assert_eq!(theme.gutter_symbol(idle()), " ", "{level:?}");
        assert_eq!(theme.gutter_symbol(focused_disabled()), " ", "{level:?}");
    }
}

#[test]
fn the_gutter_cell_blanks_without_focus_and_holds_the_bar_with_it() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        let blank = theme.gutter(idle(), canvas, false);
        assert_eq!(
            blank.fg,
            Some(canvas),
            "{level:?}: blank cell keeps the plane"
        );
        assert_eq!(blank.bg, Some(canvas), "{level:?}");

        let bar = theme.gutter(focused(), canvas, false);
        assert_eq!(bar.bg, Some(canvas), "{level:?}");
        assert_eq!(bar.fg, Some(theme.legible_mark(theme.color.focus, canvas)));
        let ratio = contrast_ratio(bar.fg.unwrap(), canvas).unwrap_or(0.0);
        assert!(
            ratio >= 3.0,
            "{level:?}: focus bar ink {bar:?} reads at {ratio:.2} on the canvas"
        );
    }
}

#[test]
fn the_gutter_bar_sits_on_accent_fills_in_on_accent_ink() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let accent = theme.color.accent;
        let bar = theme.gutter(focused(), accent, true);
        assert_eq!(bar.fg, Some(theme.color.on_accent), "{level:?}");
        assert_eq!(bar.bg, Some(accent), "{level:?}");
    }
}

#[test]
fn the_inactive_selection_keeps_accent_until_a_modal_backdrop() {
    // Owner decision 2026-10-17: selection keeps its accent in-layer, both
    // kinds go neutral under a modal or popup.
    assert_eq!(
        Theme::junie().inactive_look(InactiveSelection::Accent, false),
        InactiveSelection::Accent
    );
    assert_eq!(
        Theme::junie().inactive_look(InactiveSelection::Accent, true),
        InactiveSelection::Neutral
    );
    for policy in [InactiveSelection::Neutral, InactiveSelection::None] {
        assert_eq!(
            Theme::junie().inactive_look(policy, true),
            policy,
            "{policy:?}: only the Accent policy reacts to a backdrop"
        );
    }
    assert_eq!(InactiveSelection::default(), InactiveSelection::Accent);
}

#[test]
fn chosen_ink_follows_focus_and_the_inactive_selection_policy() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let primary = theme.color.fg[0];
        assert_eq!(
            theme.chosen_in(InactiveSelection::Accent, false, focused()),
            theme.color.accent,
            "{level:?}: focus keeps the accent"
        );
        assert_eq!(
            theme.chosen_in(InactiveSelection::Accent, false, idle()),
            theme.color.accent,
            "{level:?}: in-layer selection keeps the accent"
        );
        assert_eq!(
            theme.chosen_in(InactiveSelection::Neutral, false, idle()),
            primary,
            "{level:?}: the neutral policy drops to primary ink"
        );
        assert_eq!(
            theme.chosen_in(InactiveSelection::Accent, true, idle()),
            primary,
            "{level:?}: a modal backdrop neutralizes the accent selection"
        );
    }
}

#[test]
fn the_focus_bar_ink_stays_legible_on_every_plane_at_every_level() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        for plane in theme.color.surfaces {
            let fg = theme.legible_mark(theme.color.focus, plane);
            let ratio = contrast_ratio(fg, plane).unwrap_or(0.0);
            assert!(
                ratio >= 3.0,
                "{level:?}: mark ink {fg:?} on {plane:?} reads at {ratio:.2}"
            );
        }
    }
}
