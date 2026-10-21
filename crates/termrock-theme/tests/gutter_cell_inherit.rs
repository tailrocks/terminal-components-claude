//! Gutter-cell fill policy: inherit the container or stamp an explicit blank.
//!
//! Frozen evidence: `crates/termrock-controls/tests/frozen_ink_pins.rs` and
//! `crates/termrock-navigation/tests/frozen_ink_pins.rs` pin the bar cell's
//! `bg == None` (the container background shows through) and the audit notes
//! the helpers' explicit `fg=bg` stamp as the byte-difference that blocked
//! adoption (matrix, 2026-10-21). [`GutterCell::Inherit`] is the frozen path;
//! [`GutterCell::Fill`] reproduces the reference behaviour.

use ratatui_core::style::Style;
use termrock_core::response::StateFlags;
use termrock_theme::{
    COLOR_LEVELS, FOCUS_BAR_POLICIES, FocusBarPolicy, GUTTER_CELLS, GutterCell, Theme,
};

fn flags_set() -> [StateFlags; 6] {
    [
        StateFlags::empty(),
        StateFlags::FOCUSED,
        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
        StateFlags::FOCUSED | StateFlags::DISABLED,
        StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::DISABLED,
        StateFlags::DISABLED,
    ]
}

#[test]
fn inherit_stamps_nothing_where_no_bar_paints() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        for policy in FOCUS_BAR_POLICIES {
            for flags in flags_set() {
                if policy.shows_bar(flags, level) {
                    continue;
                }
                assert_eq!(
                    theme.gutter_in(policy, flags, GutterCell::Inherit, canvas, false),
                    Style::new(),
                    "{level:?} {policy:?} {flags:?}: an inherit cell stamps nothing"
                );
            }
        }
    }
}

#[test]
fn fill_keeps_the_reference_explicit_blank() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        for policy in FOCUS_BAR_POLICIES {
            for flags in flags_set() {
                if policy.shows_bar(flags, level) {
                    continue;
                }
                assert_eq!(
                    theme.gutter_in(policy, flags, GutterCell::Fill, canvas, false),
                    Style::new().fg(canvas).bg(canvas),
                    "{level:?} {policy:?} {flags:?}: the fill cell stamps the explicit blank"
                );
            }
        }
    }
}

#[test]
fn the_bar_cell_never_stamps_a_background_on_inherit() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        for policy in FOCUS_BAR_POLICIES {
            for flags in flags_set() {
                if !policy.shows_bar(flags, level) {
                    continue;
                }
                let inherit = theme.gutter_in(policy, flags, GutterCell::Inherit, canvas, false);
                let fill = theme.gutter_in(policy, flags, GutterCell::Fill, canvas, false);
                assert_eq!(
                    inherit.bg, None,
                    "{level:?} {policy:?} {flags:?}: the inherit bar keeps the container bg"
                );
                assert_eq!(
                    fill.fg, inherit.fg,
                    "{level:?} {policy:?} {flags:?}: both cells agree on the bar ink"
                );
                assert_eq!(
                    fill.bg,
                    Some(canvas),
                    "{level:?} {policy:?} {flags:?}: the fill bar stamps the host bg"
                );
                assert_eq!(
                    inherit.fg,
                    Some(theme.legible_mark(theme.color.focus, canvas)),
                    "{level:?} {policy:?} {flags:?}: the bar ink is the legible focus mark"
                );
            }
        }
    }
}

#[test]
fn on_accent_bars_use_the_on_accent_ink_in_both_cells() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let accent = theme.color.accent;
        let flags = StateFlags::FOCUSED;
        let policy = FocusBarPolicy::RowBlanked;
        for cell in GUTTER_CELLS {
            let style = theme.gutter_in(policy, flags, cell, accent, true);
            assert_eq!(
                style.fg,
                Some(theme.color.on_accent),
                "{level:?} {cell:?}: the on-accent bar keeps the reference ink"
            );
        }
    }
}

#[test]
fn the_legacy_gutter_keeps_the_reference_fill_behavior() {
    for level in COLOR_LEVELS {
        let theme = Theme::junie().for_level(level);
        let canvas = theme.color.surfaces[0];
        for flags in flags_set() {
            assert_eq!(
                theme.gutter(flags, canvas, false),
                theme.gutter_in(
                    FocusBarPolicy::RowBlanked,
                    flags,
                    GutterCell::Fill,
                    canvas,
                    false
                ),
                "{level:?} {flags:?}: the legacy helper is the Fill cell of its own rule"
            );
        }
    }
}
