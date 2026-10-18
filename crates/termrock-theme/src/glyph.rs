#![doc = r" Glyph roles and sets (`COMPONENT_ARCHITECTURE.md` §11.2, `DESIGN.md` markers table)."]

use ratatui_core::symbols::{line, scrollbar};

#[doc = r" Every glyph in the design system's table, one role each. Roles stay"]
#[doc = r" distinct even where Junie maps two to the same glyph, so a theme can"]
#[doc = r" separate them."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GlyphRole {
    #[doc = r" Keyboard focus bar."]
    FocusBar,
    #[doc = r" Current or chosen item."]
    Chosen,
    #[doc = r" Checked / selected / completed."]
    Checked,
    #[doc = r" Checkbox on."]
    CheckboxOn,
    #[doc = r" Checkbox off."]
    CheckboxOff,
    #[doc = r" Radio on."]
    RadioOn,
    #[doc = r" Radio off."]
    RadioOff,
    #[doc = r" Switch knob."]
    SwitchKnob,
    #[doc = r" Modified / pending."]
    Dirty,
    #[doc = r" Inserted row."]
    Inserted,
    #[doc = r" Deleted row."]
    Deleted,
    #[doc = r" Error / diagnostic."]
    Error,
    #[doc = r" Warning weight."]
    WarningMark,
    #[doc = r" Collapsed disclosure."]
    Collapsed,
    #[doc = r" Expanded disclosure."]
    Expanded,
    #[doc = r" Sort ascending."]
    SortAsc,
    #[doc = r" Sort descending."]
    SortDesc,
    #[doc = r" Filter applied."]
    Filtered,
    #[doc = r" Primary key."]
    PrimaryMark,
    #[doc = r" List bullet."]
    Bullet,
    #[doc = r" Follows a reference."]
    FollowRef,
    #[doc = r" More rows available."]
    MoreRows,
    #[doc = r" Hidden content to the left."]
    OverflowLeft,
    #[doc = r" Hidden content to the right."]
    OverflowRight,
    #[doc = r" Truncated / clipped."]
    Ellipsis,
    #[doc = r" Close / remove."]
    Close,
    #[doc = r" Path separator."]
    PathSep,
    #[doc = r" Production environment."]
    EnvProduction,
    #[doc = r" Staging environment."]
    EnvStaging,
    #[doc = r" Quiet rule."]
    RuleQuiet,
    #[doc = r" Active rule."]
    RuleActive,
    #[doc = r" Scrollbar track."]
    ScrollTrack,
    #[doc = r" Scrollbar thumb."]
    ScrollThumb,
    #[doc = r" Progress done."]
    ProgressDone,
    #[doc = r" Progress paused."]
    ProgressPaused,
    #[doc = r" New tab affordance."]
    NewTab,
    #[doc = r" Mono `PRESSED` left bracket."]
    PressLeft,
    #[doc = r" Mono `PRESSED` right bracket."]
    PressRight,
    #[doc = r" The mask a secret field paints per character (§11.2, D-11)."]
    #[doc = r""]
    #[doc = r" Distinct from [`GlyphRole::Dirty`] even though the Junie theme binds"]
    #[doc = r" both to `•`: `Dirty` is the *uncommitted changes* marker and §11.4's"]
    #[doc = r" mono rule already binds `MARKER + WARNING/DIRTY` to it, so overloading"]
    #[doc = r" it would make a theme that restyles the dirty marker also restyle"]
    #[doc = r" password masking."]
    SecretMask,
    #[doc = r" Closed select disclosure."]
    SelectClosed,
    #[doc = r" Open select disclosure."]
    SelectOpen,
    #[doc = r" Primary-key header glyph, distinct from the compact primary marker."]
    PrimaryKey,
}

impl GlyphRole {
    #[doc = r" Every role, in declaration order."]
    pub const ALL: [GlyphRole; 42] = [
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
        GlyphRole::PrimaryKey,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

#[doc = r" One `&'static str` per [`GlyphRole`]. `ScrollTrack`/`ScrollThumb` read"]
#[doc = r" from a typed `symbols::scrollbar::Set`; `RuleQuiet`/`RuleActive` from two"]
#[doc = r" typed `symbols::line::Set`s (§22 R‑11)."]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphSet {
    glyphs: [&'static str; 41],
    primary_key: &'static str,
    scroll: scrollbar::Set<'static>,
    rule_quiet: line::Set<'static>,
    rule_active: line::Set<'static>,
}

impl GlyphSet {
    #[doc = r" A set from the original 41-role table plus typed symbol sets."]
    #[doc = r" `PrimaryKey` defaults to `⚷`; use `set` to customize it independently."]
    pub const fn new(
        glyphs: [&'static str; 41],
        scroll: scrollbar::Set<'static>,
        rule_quiet: line::Set<'static>,
        rule_active: line::Set<'static>,
    ) -> Self {
        GlyphSet {
            glyphs,
            primary_key: "⚷",
            scroll,
            rule_quiet,
            rule_active,
        }
    }

    #[doc = r" The glyph for a role."]
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn get(&self, r: GlyphRole) -> &'static str {
        match r {
            GlyphRole::PrimaryKey => self.primary_key,
            GlyphRole::ScrollTrack => self.scroll.track,
            GlyphRole::ScrollThumb => self.scroll.thumb,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal,
            GlyphRole::RuleActive => self.rule_active.horizontal,
            other => self.glyphs[other.index()],
        }
    }

    #[doc = r" Replace the glyph for a role."]
    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn set(&mut self, r: GlyphRole, s: &'static str) {
        match r {
            GlyphRole::PrimaryKey => self.primary_key = s,
            GlyphRole::ScrollTrack => self.scroll.track = s,
            GlyphRole::ScrollThumb => self.scroll.thumb = s,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal = s,
            GlyphRole::RuleActive => self.rule_active.horizontal = s,
            other => self.glyphs[other.index()] = s,
        }
    }

    #[doc = r" Replace the typed scrollbar set."]
    #[doc = r""]
    #[doc = r" [`GlyphRole::ScrollTrack`] and [`GlyphRole::ScrollThumb`] read `track`"]
    #[doc = r" and `thumb` from it. `begin` and `end` remain available to consumers"]
    #[doc = r" that need cap glyphs; the shared `ScrollRegion` uses its historical"]
    #[doc = r" full-height track and deliberately does not paint cap rows (§11.2,"]
    #[doc = r" Adjudication O2)."]
    pub const fn set_scrollbar(&mut self, s: scrollbar::Set<'static>) {
        self.scroll = s;
    }

    #[doc = r" Replace the typed quiet-rule line set."]
    #[doc = r""]
    #[doc = r" [`GlyphRole::RuleQuiet`] reads `horizontal`; the vertical, the cross"]
    #[doc = r" and the eight corner/tee junctions are the seam glyphs (§22.2 item 12)"]
    #[doc = r" and are reachable only here."]
    pub const fn set_rule_quiet(&mut self, s: line::Set<'static>) {
        self.rule_quiet = s;
    }

    #[doc = r" Replace the typed active-rule line set."]
    #[doc = r""]
    #[doc = r" [`GlyphRole::RuleActive`] reads `horizontal`; the rest are seams, as"]
    #[doc = r" for [`GlyphSet::set_rule_quiet`]."]
    pub const fn set_rule_active(&mut self, s: line::Set<'static>) {
        self.rule_active = s;
    }

    #[doc = r" The typed scrollbar set."]
    pub fn scrollbar(&self) -> scrollbar::Set<'static> {
        self.scroll.clone()
    }

    #[doc = r" The typed line set for quiet rules and seams."]
    pub const fn rule_quiet(&self) -> line::Set<'static> {
        self.rule_quiet
    }

    #[doc = r" The typed line set for active rules."]
    pub const fn rule_active(&self) -> line::Set<'static> {
        self.rule_active
    }
}

#[doc = r" ASCII scrollbar: `|` track and caps, `#` thumb."]
#[doc = r""]
#[doc = r" Crate-private, like every other item of this module that is not re-exported"]
#[doc = r" by `theme`: the public entry point is"]
#[doc = r" [`ThemeBuilder::ascii_glyphs`](crate::theme::ThemeBuilder::ascii_glyphs),"]
#[doc = r" and a theme author who wants a bespoke set writes one and passes it to"]
#[doc = r" [`GlyphSet::set_scrollbar`]."]
#[doc = r""]
#[doc = r" The thumb must read **denser** than its track where colour is gone"]
#[doc = r" (`DESIGN.md:491`, `:560`): `|` is one stroke, `#` a crosshatch, which is"]
#[doc = r" the conventional ASCII thumb. `+` was rejected because it collides with"]
#[doc = r" [`border::ASCII`](crate::theme::border::ASCII)'s corners, `*` reads as a"]
#[doc = r" marker and `H` as a letter. Applied by"]
#[doc = r" [`ThemeBuilder::ascii_glyphs`](crate::theme::ThemeBuilder::ascii_glyphs)."]
pub(crate) const ASCII_SCROLLBAR: scrollbar::Set<'static> = scrollbar::Set {
    track: "|",
    thumb: "#",
    begin: "|",
    end: "|",
};

#[doc = r" ASCII quiet rule: `-` horizontal, `|` vertical, `+` at every junction."]
#[doc = r""]
#[doc = r" `-` is the direct one-stroke equivalent of `─` (`DESIGN.md:555`); the"]
#[doc = r" seams follow [`border::ASCII`](crate::theme::border::ASCII), so a rule and"]
#[doc = r" a frame meeting in one cell agree."]
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

#[doc = r" ASCII active rule: `=` horizontal, `|` vertical, `+` at every junction."]
#[doc = r""]
#[doc = r" An active rule must read **heavier** than a quiet one (`DESIGN.md:557`)."]
#[doc = r" `=` is two strokes against `-`'s one, which survives monochrome, where"]
#[doc = r" weight is the only channel left; `#` was rejected because it reads as"]
#[doc = r" hatch or fill rather than as a rule."]
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
        assert_eq!(GlyphRole::ALL.len(), 42);
        assert_eq!(GlyphRole::PrimaryKey as usize, 41);
        assert_eq!(GlyphRole::SecretMask as usize, 38);
        assert_eq!(GlyphRole::SelectClosed as usize, 39);
        assert_eq!(GlyphRole::SelectOpen as usize, 40);
    }

    #[doc = r" Adjudication O2: `borders_set(border::ASCII)` must leave **nothing** in"]
    #[doc = r" the box-drawing block, including the fields no `GlyphRole` names —"]
    #[doc = r" `scrollbar::Set`'s `begin`/`end` and `line::Set`'s seam junctions. This"]
    #[doc = r" is component-free, so it is not hostage to which painters exist; the"]
    #[doc = r" whole-frame render test can only approximate it."]
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

        let j = crate::theme::Theme::junie();
        assert_eq!(j.design.glyphs.scrollbar().begin, "│");
        assert_eq!(j.design.glyphs.rule_quiet().cross, "┼");
    }
}
