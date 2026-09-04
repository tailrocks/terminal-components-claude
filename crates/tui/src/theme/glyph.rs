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
    /// The mask a secret field paints per character (§11.2, D-11).
    ///
    /// Distinct from [`GlyphRole::Dirty`] even though the Junie theme binds
    /// both to `•`: `Dirty` is the *uncommitted changes* marker and §11.4's
    /// mono rule already binds `MARKER + WARNING/DIRTY` to it, so overloading
    /// it would make a theme that restyles the dirty marker also restyle
    /// password masking.
    SecretMask,
    /// Closed select disclosure.
    SelectClosed,
    /// Open select disclosure.
    SelectOpen,
}

impl GlyphRole {
    /// Every role, in declaration order.
    pub const ALL: [GlyphRole; 41] = [
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
        GlyphRole::SecretMask,
        GlyphRole::SelectClosed,
        GlyphRole::SelectOpen,
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
    glyphs: [&'static str; 41],
    scroll: scrollbar::Set<'static>,
    rule_quiet: line::Set<'static>,
    rule_active: line::Set<'static>,
}

impl GlyphSet {
    /// A set from a full table plus the typed symbol sets.
    pub const fn new(
        glyphs: [&'static str; 41],
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

    /// Replace the typed scrollbar set.
    ///
    /// [`GlyphRole::ScrollTrack`] and [`GlyphRole::ScrollThumb`] read `track`
    /// and `thumb` from it; `begin` and `end` — the caps a scroll region
    /// paints — are named by no [`GlyphRole`] and are reachable **only** here
    /// (§11.2, Adjudication O2).
    pub const fn set_scrollbar(&mut self, s: scrollbar::Set<'static>) {
        self.scroll = s;
    }

    /// Replace the typed quiet-rule line set.
    ///
    /// [`GlyphRole::RuleQuiet`] reads `horizontal`; the vertical, the cross
    /// and the eight corner/tee junctions are the seam glyphs (§22.2 item 12)
    /// and are reachable only here.
    pub const fn set_rule_quiet(&mut self, s: line::Set<'static>) {
        self.rule_quiet = s;
    }

    /// Replace the typed active-rule line set.
    ///
    /// [`GlyphRole::RuleActive`] reads `horizontal`; the rest are seams, as
    /// for [`GlyphSet::set_rule_quiet`].
    pub const fn set_rule_active(&mut self, s: line::Set<'static>) {
        self.rule_active = s;
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

/// ASCII scrollbar: `|` track and caps, `#` thumb.
///
/// Crate-private, like every other item of this module that is not re-exported
/// by `theme`: the public entry point is
/// [`ThemeBuilder::ascii_glyphs`](crate::theme::ThemeBuilder::ascii_glyphs),
/// and a theme author who wants a bespoke set writes one and passes it to
/// [`GlyphSet::set_scrollbar`].
///
/// The thumb must read **denser** than its track where colour is gone
/// (`DESIGN.md:491`, `:560`): `|` is one stroke, `#` a crosshatch, which is
/// the conventional ASCII thumb. `+` was rejected because it collides with
/// [`border::ASCII`](crate::theme::border::ASCII)'s corners, `*` reads as a
/// marker and `H` as a letter. Applied by
/// [`ThemeBuilder::ascii_glyphs`](crate::theme::ThemeBuilder::ascii_glyphs).
pub(crate) const ASCII_SCROLLBAR: scrollbar::Set<'static> = scrollbar::Set {
    track: "|",
    thumb: "#",
    begin: "|",
    end: "|",
};

/// ASCII quiet rule: `-` horizontal, `|` vertical, `+` at every junction.
///
/// `-` is the direct one-stroke equivalent of `─` (`DESIGN.md:555`); the
/// seams follow [`border::ASCII`](crate::theme::border::ASCII), so a rule and
/// a frame meeting in one cell agree.
pub(crate) const ASCII_RULE_QUIET: line::Set<'static> = line::Set {
    vertical: "|",
    horizontal: "-",
    top_right: "+",
    top_left: "+",
    bottom_right: "+",
    bottom_left: "+",
    vertical_left: "+",
    vertical_right: "+",
    horizontal_down: "+",
    horizontal_up: "+",
    cross: "+",
};

/// ASCII active rule: `=` horizontal, `|` vertical, `+` at every junction.
///
/// An active rule must read **heavier** than a quiet one (`DESIGN.md:557`).
/// `=` is two strokes against `-`'s one, which survives monochrome, where
/// weight is the only channel left; `#` was rejected because it reads as
/// hatch or fill rather than as a rule.
pub(crate) const ASCII_RULE_ACTIVE: line::Set<'static> = line::Set {
    vertical: "|",
    horizontal: "=",
    top_right: "+",
    top_left: "+",
    bottom_right: "+",
    bottom_left: "+",
    vertical_left: "+",
    vertical_right: "+",
    horizontal_down: "+",
    horizontal_up: "+",
    cross: "+",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_reads_and_writes_its_slot() {
        let mut g = GlyphSet::new(["x"; 41], scrollbar::VERTICAL, line::NORMAL, line::THICK);
        for r in GlyphRole::ALL {
            g.set(r, "y");
            assert_eq!(g.get(r), "y", "{r:?}");
        }
        assert_eq!(g.scrollbar().thumb, "y");
    }

    #[test]
    fn select_roles_are_appended_without_shifting_existing_discriminants() {
        assert_eq!(GlyphRole::ALL.len(), 41);
        assert_eq!(GlyphRole::SecretMask as usize, 38);
        assert_eq!(GlyphRole::SelectClosed as usize, 39);
        assert_eq!(GlyphRole::SelectOpen as usize, 40);
    }

    /// Adjudication O2: `borders_set(border::ASCII)` must leave **nothing** in
    /// the box-drawing block, including the fields no `GlyphRole` names —
    /// `scrollbar::Set`'s `begin`/`end` and `line::Set`'s seam junctions. This
    /// is component-free, so it is not hostage to which painters exist; the
    /// whole-frame render test can only approximate it.
    #[test]
    fn ascii_glyph_set_has_no_box_drawing() {
        let t = crate::theme::Theme::junie()
            .builder()
            .borders_set(crate::theme::border::ASCII)
            .build();
        let g = &t.design.glyphs;
        let check = |what: &str, s: &'static str| {
            for c in s.chars() {
                assert!(
                    !('\u{2500}'..='\u{257F}').contains(&c),
                    "{what} is {s:?}, which contains box drawing U+{:04X}",
                    c as u32
                );
            }
        };
        for r in GlyphRole::ALL {
            check(&format!("{r:?}"), g.get(r));
        }
        let sb = g.scrollbar();
        for (name, s) in [
            ("scrollbar.track", sb.track),
            ("scrollbar.thumb", sb.thumb),
            ("scrollbar.begin", sb.begin),
            ("scrollbar.end", sb.end),
        ] {
            check(name, s);
        }
        for (set_name, l) in [
            ("rule_quiet", g.rule_quiet()),
            ("rule_active", g.rule_active()),
        ] {
            for (name, s) in [
                ("vertical", l.vertical),
                ("horizontal", l.horizontal),
                ("top_right", l.top_right),
                ("top_left", l.top_left),
                ("bottom_right", l.bottom_right),
                ("bottom_left", l.bottom_left),
                ("vertical_left", l.vertical_left),
                ("vertical_right", l.vertical_right),
                ("horizontal_down", l.horizontal_down),
                ("horizontal_up", l.horizontal_up),
                ("cross", l.cross),
            ] {
                check(&format!("{set_name}.{name}"), s);
            }
        }
        // not vacuous: plain Junie does bind these to box drawing
        let j = crate::theme::Theme::junie();
        assert_eq!(j.design.glyphs.scrollbar().begin, "│");
        assert_eq!(j.design.glyphs.rule_quiet().cross, "┼");
    }
}
