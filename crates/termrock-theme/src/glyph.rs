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
    more_titles: &'static str,
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
            more_titles: "»",
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

    pub const fn more_titles(&self) -> &'static str {
        self.more_titles
    }

    pub const fn set_more_titles(&mut self, glyph: &'static str) {
        self.more_titles = glyph;
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
