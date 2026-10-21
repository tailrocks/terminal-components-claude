use ratatui_core::symbols::{line, scrollbar};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GlyphRole {
    FocusBar,
    Chosen,
    Checked,
    CheckboxOn,
    CheckboxOff,
    RadioOn,
    RadioOff,
    SwitchKnob,
    Dirty,
    Inserted,
    Deleted,
    Error,
    WarningMark,
    Collapsed,
    Expanded,
    SortAsc,
    SortDesc,
    Filtered,
    PrimaryMark,
    Bullet,
    FollowRef,
    MoreRows,
    OverflowLeft,
    OverflowRight,
    Ellipsis,
    Close,
    PathSep,
    EnvProduction,
    EnvStaging,
    RuleQuiet,
    RuleActive,
    ScrollTrack,
    ScrollThumb,
    ProgressDone,
    ProgressPaused,
    NewTab,
    PressLeft,
    PressRight,
    SecretMask,
    SelectClosed,
    SelectOpen,
    PrimaryKey,
    DotOn,
    DotOff,
    SwitchOff,
}

impl GlyphRole {
    pub const ALL: [GlyphRole; 45] = [
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
        GlyphRole::DotOn,
        GlyphRole::DotOff,
        GlyphRole::SwitchOff,
    ];

    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphSet {
    glyphs: [&'static str; 41],
    primary_key: &'static str,
    dot_on: &'static str,
    dot_off: &'static str,
    switch_off: &'static str,
    scroll: scrollbar::Set<'static>,
    rule_quiet: line::Set<'static>,
    rule_active: line::Set<'static>,
}

impl GlyphSet {
    pub const fn new(
        glyphs: [&'static str; 41],
        scroll: scrollbar::Set<'static>,
        rule_quiet: line::Set<'static>,
        rule_active: line::Set<'static>,
    ) -> Self {
        GlyphSet {
            glyphs,
            primary_key: "⚷",
            dot_on: "●",
            dot_off: "○",
            switch_off: "○",
            scroll,
            rule_quiet,
            rule_active,
        }
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn get(&self, r: GlyphRole) -> &'static str {
        match r {
            GlyphRole::PrimaryKey => self.primary_key,
            GlyphRole::DotOn => self.dot_on,
            GlyphRole::DotOff => self.dot_off,
            GlyphRole::SwitchOff => self.switch_off,
            GlyphRole::ScrollTrack => self.scroll.track,
            GlyphRole::ScrollThumb => self.scroll.thumb,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal,
            GlyphRole::RuleActive => self.rule_active.horizontal,
            other => self.glyphs[other.index()],
        }
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "the index is a variant discriminant below the array length"
    )]
    pub const fn set(&mut self, r: GlyphRole, s: &'static str) {
        match r {
            GlyphRole::PrimaryKey => self.primary_key = s,
            GlyphRole::DotOn => self.dot_on = s,
            GlyphRole::DotOff => self.dot_off = s,
            GlyphRole::SwitchOff => self.switch_off = s,
            GlyphRole::ScrollTrack => self.scroll.track = s,
            GlyphRole::ScrollThumb => self.scroll.thumb = s,
            GlyphRole::RuleQuiet => self.rule_quiet.horizontal = s,
            GlyphRole::RuleActive => self.rule_active.horizontal = s,
            other => self.glyphs[other.index()] = s,
        }
    }

    pub const fn set_scrollbar(&mut self, s: scrollbar::Set<'static>) {
        self.scroll = s;
    }

    pub const fn set_rule_quiet(&mut self, s: line::Set<'static>) {
        self.rule_quiet = s;
    }

    pub const fn set_rule_active(&mut self, s: line::Set<'static>) {
        self.rule_active = s;
    }

    pub fn scrollbar(&self) -> scrollbar::Set<'static> {
        self.scroll.clone()
    }

    pub const fn rule_quiet(&self) -> line::Set<'static> {
        self.rule_quiet
    }

    pub const fn rule_active(&self) -> line::Set<'static> {
        self.rule_active
    }
}

pub(crate) const ASCII_SCROLLBAR: scrollbar::Set<'static> = scrollbar::Set {
    track: "|",
    thumb: "#",
    begin: "|",
    end: "|",
};

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
        assert_eq!(GlyphRole::ALL.len(), 45);
        assert_eq!(GlyphRole::PrimaryKey as usize, 41);
        assert_eq!(GlyphRole::SecretMask as usize, 38);
        assert_eq!(GlyphRole::SelectClosed as usize, 39);
        assert_eq!(GlyphRole::SelectOpen as usize, 40);
        assert_eq!(GlyphRole::DotOn as usize, 42);
        assert_eq!(GlyphRole::DotOff as usize, 43);
        assert_eq!(GlyphRole::SwitchOff as usize, 44);
    }

    #[test]
    fn dot_roles_are_one_cell_and_distinct_from_radio_pair_glyphs() {
        let glyphs = crate::theme::Theme::junie().design.glyphs;
        assert_eq!(glyphs.get(GlyphRole::DotOn), "●");
        assert_eq!(glyphs.get(GlyphRole::DotOff), "○");
        assert_eq!(glyphs.get(GlyphRole::RadioOn), "(●)");
        assert_eq!(glyphs.get(GlyphRole::RadioOff), "(○)");
    }

    #[test]
    fn ascii_builder_defines_dot_roles_without_changing_radio_roles() {
        let glyphs = crate::theme::Theme::junie()
            .builder()
            .ascii_glyphs()
            .build()
            .design
            .glyphs;
        assert_eq!(glyphs.get(GlyphRole::DotOn), "*");
        assert_eq!(glyphs.get(GlyphRole::DotOff), "o");
        assert_eq!(glyphs.get(GlyphRole::RadioOn), "(●)");
        assert_eq!(glyphs.get(GlyphRole::RadioOff), "(○)");
    }

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
        assert_eq!(j.design.glyphs.scrollbar().begin, "┆");
        assert_eq!(j.design.glyphs.rule_quiet().cross, "┼");
    }
}
