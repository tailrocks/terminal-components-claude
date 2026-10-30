//! Frozen-ink pins for the controls widget families.
//!
//! The PR #26 shared ink helpers (`Theme::gutter`, `Theme::gutter_symbol`,
//! `Theme::chosen_in`) were evaluated for adoption at the render sites in
//! `button.rs` and `choice.rs` (commit-scoped audit, 2026-10-21). The frozen
//! app output diverges from the helpers, so the recipe-driven local paths
//! stay. These tests pin the frozen decisions the audit measured, so a theme
//! change cannot move app pixels silently:
//!
//! - `BUTTON` draws the focus bar only while focused AND focus-visible, and
//!   keeps it through both states even while disabled.
//!   `Theme::gutter_symbol` has no focus-visibility gate, so adopting it
//!   would add a bar in mouse-initiated focus.
//! - `CHOICE`/`TREE`/`STEPS` rows keep the bar while focused even when
//!   disabled, at the truecolor, 256 and 16 tiers; `LIST` blanks it there,
//!   and every family blanks it at the mono tier. The helpers blank it for
//!   every family and tier, so no single default reproduces the frozen
//!   output.
//! - A checked box, on radio or on toggle keeps the accent mark while focus
//!   is elsewhere (owner decision 2026-10-17, `InactiveSelection::Accent`),
//!   and keeps it under a modal backdrop. `Theme::chosen_in(Accent, ..)`
//!   drops to primary ink under a modal, so the recipes stay.
//! - A chosen option's row fill keeps the accent tint only while focused;
//!   unfocused chosen rows fall back to the plain container surface.

use termrock_core::id::Part;
use termrock_core::response::StateFlags;
use termrock_theme::{ColorLevel, Family, GlyphRole, Slot, Surface, Theme, Variant};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

fn resolved(
    theme: &Theme,
    family: Family,
    part: Part,
    flags: StateFlags,
) -> termrock_theme::Resolved {
    theme.resolve(family, Variant::DEFAULT, part, flags, Surface::Canvas)
}

#[test]
fn button_gutter_requires_focus_visibility_and_stays_blank_while_disabled() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let focused = resolved(&theme, Family::BUTTON, Part::GUTTER, StateFlags::FOCUSED);
        assert!(
            !matches!(focused.glyph, Slot::Set(GlyphRole::FocusBar)),
            "{level:?}: the bar needs focus visibility, not focus alone"
        );
        let visible = resolved(
            &theme,
            Family::BUTTON,
            Part::GUTTER,
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
        );
        assert_eq!(
            visible.glyph,
            Slot::Set(GlyphRole::FocusBar),
            "{level:?}: focus-visible focus draws the bar"
        );
        assert_eq!(
            visible.style.as_style().bg,
            None,
            "{level:?}: the bar cell keeps the container background"
        );
        let blanked = resolved(
            &theme,
            Family::BUTTON,
            Part::GUTTER,
            StateFlags::FOCUSED | StateFlags::DISABLED,
        );
        assert!(
            !matches!(blanked.glyph, Slot::Set(GlyphRole::FocusBar)),
            "{level:?}: without focus visibility the bar stays blank"
        );
        let disabled = resolved(
            &theme,
            Family::BUTTON,
            Part::GUTTER,
            StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::DISABLED,
        );
        assert_eq!(
            disabled.glyph,
            Slot::Set(GlyphRole::FocusBar),
            "{level:?}: a focus-visible bar survives the disabled state"
        );
    }
}

#[test]
fn row_gutter_bar_policy_stays_frozen_per_family_and_tier() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let focused_disabled = StateFlags::FOCUSED | StateFlags::DISABLED;
        for family in [Family::CHOICE, Family::TREE, Family::STEPS, Family::LIST] {
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
        }
    }
}

#[test]
fn checked_marks_keep_the_accent_without_focus_and_under_a_backdrop() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        let unfocused = resolved(&theme, Family::CHOICE, Part::MARKER, StateFlags::CHECKED);
        let focused = resolved(
            &theme,
            Family::CHOICE,
            Part::MARKER,
            StateFlags::CHECKED | StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE,
        );
        assert_eq!(
            unfocused.style.as_style().fg,
            Some(theme.color.accent),
            "{level:?}: the checked mark stays accent while focus is elsewhere"
        );
        assert_eq!(
            focused.style.as_style().fg,
            Some(theme.color.accent),
            "{level:?}: the checked mark is accent while focused"
        );
    }
}

#[test]
fn chosen_option_fill_keeps_the_accent_tint_only_while_focused() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for family in [Family::CHOICE, Family::LIST, Family::TREE, Family::STEPS] {
            let rest = resolved(&theme, family, Part::CONTAINER, StateFlags::empty());
            let chosen = resolved(&theme, family, Part::CONTAINER, StateFlags::SELECTED);
            let focused = resolved(
                &theme,
                family,
                Part::CONTAINER,
                StateFlags::SELECTED | StateFlags::FOCUSED,
            );
            assert_eq!(
                chosen.style.as_style().bg,
                rest.style.as_style().bg,
                "{level:?} {family:?}: an unfocused chosen row reads as the plain surface"
            );
            assert_eq!(
                focused.style.as_style().bg,
                Some(theme.color.accent_tint),
                "{level:?} {family:?}: a focused chosen row takes the accent tint"
            );
        }
    }
}
