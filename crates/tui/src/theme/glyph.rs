//! Glyph roles and sets (`COMPONENT_ARCHITECTURE.md` §11.2, `DESIGN.md` markers table).

use ratatui_core::symbols::{line, scrollbar};

/// Every glyph in the design system's table, one role each. Roles stay
/// distinct even where Junie maps two to the same glyph, so a theme can
/// separate them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GlyphRole {
    /// Keyboard focus bar.
    FocusBar,
    /// Current or chosen item.
    Chosen,
    /// Checked / selected / completed.
    Checked,
    /// Checkbox on.
    CheckboxOn,
    /// Checkbox off.
    CheckboxOff,
    /// Radio on.
    RadioOn,
    /// Radio off.
    RadioOff,
    /// Switch knob.
    SwitchKnob,
    /// Modified / pending.
    Dirty,
    /// Inserted row.
    Inserted,
    /// Deleted row.
    Deleted,
    /// Error / diagnostic.
    Error,
    /// Warning weight.
    WarningMark,
    /// Collapsed disclosure.
    Collapsed,
    /// Expanded disclosure.
    Expanded,
    /// Sort ascending.
    SortAsc,
    /// Sort descending.
    SortDesc,
    /// Filter applied.
    Filtered,
    /// Primary key.
    PrimaryMark,
    /// List bullet.
    Bullet,
    /// Follows a reference.
    FollowRef,
    /// More rows available.
    MoreRows,
    /// Hidden content to the left.
    OverflowLeft,
    /// Hidden content to the right.
    OverflowRight,
    /// Truncated / clipped.
    Ellipsis,
    /// Close / remove.
    Close,
    /// Path separator.
    PathSep,
    /// Production environment.
    EnvProduction,
    /// Staging environment.
    EnvStaging,
    /// Quiet rule.
    RuleQuiet,
    /// Active rule.
    RuleActive,
    /// Scrollbar track.
    ScrollTrack,
    /// Scrollbar thumb.
    ScrollThumb,
    /// Progress done.
    ProgressDone,
    /// Progress paused.
    ProgressPaused,
    /// New tab affordance.
    NewTab,
    /// Mono `PRESSED` left bracket.
    PressLeft,
    /// Mono `PRESSED` right bracket.
    PressRight,
}

impl GlyphRole {
    /// Every role, in declaration order.
    pub const ALL: [GlyphRole; 38] = [
        GlyphRole::FocusBar,
        GlyphRole::Chosen,
        GlyphRole::Checked,
        GlyphRole::CheckboxOn,
        GlyphRole::CheckboxOff,
        GlyphRole::RadioOn,
        GlyphRole::RadioOff,
        GlyphRole::SwitchKnob,
        GlyphRole::Dirty,
        GlyphRole::Inserted,
        GlyphRole::Deleted,
        GlyphRole::Error,
        GlyphRole::WarningMark,
        GlyphRole::Collapsed,
        GlyphRole::Expanded,
        GlyphRole::SortAsc,
        GlyphRole::SortDesc,
        GlyphRole::Filtered,
        GlyphRole::PrimaryMark,
        GlyphRole::Bullet,
        GlyphRole::FollowRef,
        GlyphRole::MoreRows,
        GlyphRole::OverflowLeft,
        GlyphRole::OverflowRight,
        GlyphRole::Ellipsis,
        GlyphRole::Close,
        GlyphRole::PathSep,
        GlyphRole::EnvProduction,
        GlyphRole::EnvStaging,
        GlyphRole::RuleQuiet,
        GlyphRole::RuleActive,
        GlyphRole::ScrollTrack,
        GlyphRole::ScrollThumb,
        GlyphRole::ProgressDone,
        GlyphRole::ProgressPaused,
        GlyphRole::NewTab,
        GlyphRole::PressLeft,
        GlyphRole::PressRight,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

/// One `&'static str` per [`GlyphRole`]. `ScrollTrack`/`ScrollThumb` read
/// from a typed `symbols::scrollbar::Set`; `RuleQuiet`/`RuleActive` from two
/// typed `symbols::line::Set`s (§22 R‑11).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphSet {
    glyphs: [&'static str; 38],
    scroll: scrollbar::Set<'static>,
    rule_quiet: line::Set<'static>,
    rule_active: line::Set<'static>,
}

impl GlyphSet {
    /// A set from a full table plus the typed symbol sets.
    pub const fn new(
        glyphs: [&'static str; 38],
        scroll: scrollbar::Set<'static>,
        rule_quiet: line::Set<'static>,
        rule_active: line::Set<'static>,
    ) -> Self {
        GlyphSet {
            glyphs,
            scroll,
            rule_quiet,
            rule_active,
        }
    }

    /// The glyph for a role.
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn get(&self, r: GlyphRole) -> &'static str {
        match r {
            GlyphRole::ScrollTrack => self.scroll.track,
            GlyphRole::ScrollThumb => self.scroll.thumb,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal,
            GlyphRole::RuleActive => self.rule_active.horizontal,
            other => self.glyphs[other.index()],
        }
    }

    /// Replace the glyph for a role.
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn set(&mut self, r: GlyphRole, s: &'static str) {
        match r {
            GlyphRole::ScrollTrack => self.scroll.track = s,
            GlyphRole::ScrollThumb => self.scroll.thumb = s,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal = s,
            GlyphRole::RuleActive => self.rule_active.horizontal = s,
            other => self.glyphs[other.index()] = s,
        }
    }

    /// The typed scrollbar set.
    pub fn scrollbar(&self) -> scrollbar::Set<'static> {
        self.scroll.clone()
    }

    /// The typed line set for quiet rules and seams.
    pub const fn rule_quiet(&self) -> line::Set<'static> {
        self.rule_quiet
    }

    /// The typed line set for active rules.
    pub const fn rule_active(&self) -> line::Set<'static> {
        self.rule_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_reads_and_writes_its_slot() {
        let mut g = GlyphSet::new(["x"; 38], scrollbar::VERTICAL, line::NORMAL, line::THICK);
        for r in GlyphRole::ALL {
            g.set(r, "y");
            assert_eq!(g.get(r), "y", "{r:?}");
        }
        assert_eq!(g.scrollbar().thumb, "y");
    }
}
