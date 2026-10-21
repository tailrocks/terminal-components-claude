//! Frozen-ink pins for the navigation widget families (`LIST`, `TREE`,
//! `STEPS`).
//!
//! The PR #26 shared ink helpers (`Theme::gutter`, `Theme::gutter_symbol`,
//! `Theme::chosen_in`) were evaluated for adoption at the gutter paint sites
//! in `list.rs`, `tree.rs` and `steps.rs` (commit-scoped audit, 2026-10-21).
//! The frozen app output diverges from the helpers, so the recipe-driven
//! local paths stay. These tests pin the frozen decisions the audit
//! measured, so a theme change cannot move app pixels silently:
//!
//! - A focused row keeps the focus bar while disabled at the truecolor, 256
//!   and 16 tiers (`LIST` blanks it there), and every family blanks it at
//!   the mono tier. The helpers blank it for every family and tier, so no
//!   single default reproduces the frozen output.
//! - A checked row keeps the accent marker while focus is elsewhere (owner
//!   decision 2026-10-17, `InactiveSelection::Accent`).
//! - A selected row's fill keeps the accent tint only while focused;
//!   unfocused selected rows read as the plain container surface (the
//!   frozen reference shows a neutral inactive selection, unlike the PR #26
//!   reference default, which kept the accent fill).

use termrock_core::id::Part;
use termrock_core::response::StateFlags;
use termrock_theme::{ColorLevel, Family, GlyphRole, Slot, Surface, Theme, Variant};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const FAMILIES: [Family; 3] = [Family::LIST, Family::TREE, Family::STEPS];

fn resolved(
    theme: &Theme,
    family: Family,
    part: Part,
    flags: StateFlags,
) -> termrock_theme::Resolved {
    theme.resolve(family, Variant::DEFAULT, part, flags, Surface::Canvas)
}

#[test]
fn row_gutter_bar_policy_stays_frozen_per_family_and_tier() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let focused_disabled = StateFlags::FOCUSED | StateFlags::DISABLED;
        for family in FAMILIES {
            let r = resolved(&theme, family, Part::GUTTER, focused_disabled);
            if level == ColorLevel::Mono || family == Family::LIST {
                assert!(
                    !matches!(r.glyph, Slot::Set(GlyphRole::FocusBar)),
                    "{level:?} {family:?}: the mono tier and list rows blank the bar while disabled"
                );
            } else {
                assert_eq!(
                    r.glyph,
                    Slot::Set(GlyphRole::FocusBar),
                    "{level:?} {family:?}: focused rows keep the bar while disabled"
                );
            }
            let focused = resolved(&theme, family, Part::GUTTER, StateFlags::FOCUSED);
            assert_eq!(
                focused.glyph,
                Slot::Set(GlyphRole::FocusBar),
                "{level:?} {family:?}: a focused row draws the bar without focus visibility"
            );
            assert_eq!(
                focused.style.as_style().bg,
                None,
                "{level:?} {family:?}: the bar cell keeps the container background"
            );
        }
    }
}

#[test]
fn checked_markers_keep_the_accent_without_focus() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let unfocused = resolved(&theme, Family::LIST, Part::MARKER, StateFlags::CHECKED);
        let focused = resolved(
            &theme,
            Family::LIST,
            Part::MARKER,
            StateFlags::CHECKED | StateFlags::FOCUSED,
        );
        assert_eq!(
            unfocused.style.as_style().fg,
            Some(theme.color.accent),
            "{level:?}: the checked marker stays accent while focus is elsewhere"
        );
        assert_eq!(
            focused.style.as_style().fg,
            Some(theme.color.accent),
            "{level:?}: the checked marker is accent while focused"
        );
    }
}

#[test]
fn selected_row_fill_keeps_the_accent_tint_only_while_focused() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for family in FAMILIES {
            let rest = resolved(&theme, family, Part::CONTAINER, StateFlags::empty());
            let selected = resolved(&theme, family, Part::CONTAINER, StateFlags::SELECTED);
            let focused = resolved(
                &theme,
                family,
                Part::CONTAINER,
                StateFlags::SELECTED | StateFlags::FOCUSED,
            );
            assert_eq!(
                selected.style.as_style().bg,
                rest.style.as_style().bg,
                "{level:?} {family:?}: an unfocused selected row reads as the plain surface"
            );
            assert_eq!(
                focused.style.as_style().bg,
                Some(theme.color.accent_tint),
                "{level:?} {family:?}: a focused selected row takes the accent tint"
            );
        }
    }
}
