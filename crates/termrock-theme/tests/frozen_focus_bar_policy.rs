//! The focus-bar policy table, measured against the frozen widget output.
//!
//! Truth table: `crates/termrock-controls/tests/frozen_ink_pins.rs` and
//! `crates/termrock-navigation/tests/frozen_ink_pins.rs` (commit-scoped
//! audit, 2026-10-21). The audit found no single baked-in rule reproducing
//! every family, so the rule is a [`FocusBarPolicy`] parameter:
//!
//! - `BUTTON` draws the bar only while focused AND focus-visible, and a
//!   focus-visible bar survives the disabled state (truecolor/256/16/mono).
//! - `CHOICE`/`TREE`/`STEPS` rows draw the bar while focused even when
//!   disabled at the truecolor, 256 and 16 tiers; the mono tier blanks it.
//! - `LIST` rows blank the bar while disabled at every tier.
//! - No family needs focus visibility for a focused row, and no family draws
//!   a bar without focus.

use termrock_core::response::StateFlags;
use termrock_theme::{COLOR_LEVELS, ColorLevel, FocusBarPolicy, GlyphRole, Theme};

fn focused() -> StateFlags {
    StateFlags::FOCUSED
}

fn focused_visible() -> StateFlags {
    StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE
}

fn focused_disabled() -> StateFlags {
    StateFlags::FOCUSED | StateFlags::DISABLED
}

fn focused_visible_disabled() -> StateFlags {
    StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::DISABLED
}

#[test]
fn button_bar_requires_focus_visibility_and_survives_disabled() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let bar = theme.design.glyphs.get(GlyphRole::FocusBar);
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::FocusVisible, focused()),
            " ",
            "{level:?}: focus alone must not draw the button bar"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::FocusVisible, focused_visible()),
            bar,
            "{level:?}: focus-visible focus draws the bar"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::FocusVisible, focused_visible_disabled()),
            bar,
            "{level:?}: a focus-visible bar survives the disabled state"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::FocusVisible, focused_disabled()),
            " ",
            "{level:?}: without focus visibility the bar stays blank"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::FocusVisible, StateFlags::empty()),
            " ",
            "{level:?}: an idle button shows no bar"
        );
    }
}

#[test]
fn choice_tree_steps_rows_keep_the_bar_while_disabled_off_mono() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let bar = theme.design.glyphs.get(GlyphRole::FocusBar);
        let keep_disabled = if level == ColorLevel::Mono { " " } else { bar };
        let policy = FocusBarPolicy::RowKept;
        assert_eq!(
            theme.focus_bar_symbol(policy, focused()),
            bar,
            "{level:?} {policy:?}: a focused row draws the bar without focus visibility"
        );
        assert_eq!(
            theme.focus_bar_symbol(policy, focused_disabled()),
            keep_disabled,
            "{level:?} {policy:?}: only the mono tier blanks a disabled row's bar"
        );
        assert_eq!(
            theme.focus_bar_symbol(policy, StateFlags::empty()),
            " ",
            "{level:?} {policy:?}: an idle row shows no bar"
        );
    }
}

#[test]
fn list_rows_blank_the_bar_while_disabled_at_every_tier() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let bar = theme.design.glyphs.get(GlyphRole::FocusBar);
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::RowBlanked, focused()),
            bar,
            "{level:?}: a focused list row draws the bar"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::RowBlanked, focused_disabled()),
            " ",
            "{level:?}: a disabled list row blanks the bar"
        );
        assert_eq!(
            theme.focus_bar_symbol(FocusBarPolicy::RowBlanked, StateFlags::empty()),
            " ",
            "{level:?}: an idle list row shows no bar"
        );
    }
}

#[test]
fn the_pre_policy_helpers_keep_their_baked_in_rule() {
    // `gutter_symbol` keeps the reference rule (focused and enabled), which is
    // exactly the `RowBlanked` policy; the legacy `gutter` style follows it.
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        for flags in [
            StateFlags::empty(),
            focused(),
            focused_disabled(),
            focused_visible(),
            focused_visible_disabled(),
        ] {
            let blank_or_bar = theme.gutter_symbol(flags);
            assert_eq!(
                blank_or_bar,
                theme.focus_bar_symbol(FocusBarPolicy::RowBlanked, flags),
                "{level:?} {flags:?}: the legacy helper follows the RowBlanked rule"
            );
            let expected = if blank_or_bar == " " {
                ratatui_core::style::Style::new().fg(canvas).bg(canvas)
            } else {
                ratatui_core::style::Style::new()
                    .fg(theme.legible_mark(theme.color.focus, canvas))
                    .bg(canvas)
            };
            assert_eq!(
                theme.gutter(flags, canvas, false),
                expected,
                "{level:?} {flags:?}: the legacy gutter style matches its own rule"
            );
        }
    }
}
