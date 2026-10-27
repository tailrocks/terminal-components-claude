//! Ported from the PR #26 reference theme (`src/theme.rs` at

use ratatui_core::style::{Color, Style};

use crate::Theme;
use crate::downgrade::{luminance, rgb_of};
use crate::glyph::GlyphRole;
use crate::response::StateFlags;
use crate::tokens::ColorLevel;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum InactiveSelection {
    #[default]
    Accent,

    Neutral,

    None,
}

impl InactiveSelection {
    pub const ALL: [InactiveSelection; 3] = [
        InactiveSelection::Accent,
        InactiveSelection::Neutral,
        InactiveSelection::None,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum FocusBarPolicy {
    #[default]
    RowBlanked,

    RowKept,

    FocusVisible,
}

impl FocusBarPolicy {
    pub const ALL: [FocusBarPolicy; 3] = [
        FocusBarPolicy::RowBlanked,
        FocusBarPolicy::RowKept,
        FocusBarPolicy::FocusVisible,
    ];

    #[must_use]
    pub fn shows_bar(self, flags: StateFlags, level: ColorLevel) -> bool {
        if !flags.contains(StateFlags::FOCUSED) {
            return false;
        }
        match self {
            Self::RowBlanked => !flags.contains(StateFlags::DISABLED),
            Self::RowKept => !(flags.contains(StateFlags::DISABLED) && level == ColorLevel::Mono),
            Self::FocusVisible => flags.contains(StateFlags::FOCUS_VISIBLE),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GutterCell {
    #[default]
    Inherit,

    Fill,
}

impl GutterCell {
    pub const ALL: [GutterCell; 2] = [GutterCell::Inherit, GutterCell::Fill];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum BackdropSelection {
    #[default]
    Neutral,

    Accent,
}

impl BackdropSelection {
    pub const ALL: [BackdropSelection; 2] = [BackdropSelection::Neutral, BackdropSelection::Accent];
}

impl Theme {
    #[must_use]
    pub fn gutter_symbol(&self, flags: StateFlags) -> &'static str {
        self.focus_bar_symbol(FocusBarPolicy::RowBlanked, flags)
    }

    #[must_use]
    pub fn focus_bar_symbol(&self, policy: FocusBarPolicy, flags: StateFlags) -> &'static str {
        if policy.shows_bar(flags, self.capability.color) {
            self.design.glyphs.get(GlyphRole::FocusBar)
        } else {
            " "
        }
    }

    #[must_use]
    pub fn gutter(&self, flags: StateFlags, bg: Color, on_accent: bool) -> Style {
        self.gutter_in(
            FocusBarPolicy::RowBlanked,
            flags,
            GutterCell::Fill,
            bg,
            on_accent,
        )
    }

    #[must_use]
    pub fn gutter_in(
        &self,
        policy: FocusBarPolicy,
        flags: StateFlags,
        cell: GutterCell,
        bg: Color,
        on_accent: bool,
    ) -> Style {
        if !policy.shows_bar(flags, self.capability.color) {
            return match cell {
                GutterCell::Inherit => Style::new(),
                GutterCell::Fill => Style::new().fg(bg).bg(bg),
            };
        }
        let fg = if on_accent {
            self.color.on_accent
        } else {
            self.legible_mark(self.color.focus, bg)
        };
        match cell {
            GutterCell::Inherit => Style::new().fg(fg),
            GutterCell::Fill => Style::new().fg(fg).bg(bg),
        }
    }

    #[must_use]
    pub fn chosen_in(
        &self,
        policy: InactiveSelection,
        under_modal: bool,
        flags: StateFlags,
    ) -> Color {
        self.chosen_in_with(policy, under_modal, BackdropSelection::Neutral, flags)
    }

    #[must_use]
    pub fn chosen_in_with(
        &self,
        policy: InactiveSelection,
        under_modal: bool,
        backdrop: BackdropSelection,
        flags: StateFlags,
    ) -> Color {
        if flags.contains(StateFlags::FOCUSED)
            || self.inactive_look_with(policy, under_modal, backdrop) == InactiveSelection::Accent
        {
            self.color.accent
        } else {
            self.color.fg[0]
        }
    }

    #[must_use]
    pub fn inactive_look(&self, policy: InactiveSelection, under_modal: bool) -> InactiveSelection {
        self.inactive_look_with(policy, under_modal, BackdropSelection::Neutral)
    }

    #[must_use]
    pub fn inactive_look_with(
        &self,
        policy: InactiveSelection,
        under_modal: bool,
        backdrop: BackdropSelection,
    ) -> InactiveSelection {
        let _ = self;
        match policy {
            InactiveSelection::Accent if under_modal && backdrop == BackdropSelection::Neutral => {
                InactiveSelection::Neutral
            }
            look => look,
        }
    }

    #[must_use]
    pub fn legible_mark(&self, fg: Color, bg: Color) -> Color {
        const MARK: f64 = 3.0;
        let reads = |ink: Color| contrast_ratio(ink, bg).is_none_or(|r| r >= MARK);
        if reads(fg) {
            return fg;
        }

        let strong: [Color; 0] = [];
        if let Some(ink) = bright(fg).into_iter().chain(strong).find(|&ink| reads(ink)) {
            return ink;
        }
        if reads(self.color.fg[0]) {
            return self.color.fg[0];
        }
        [self.color.fg[0], self.color.on_accent]
            .into_iter()
            .filter_map(|ink| contrast_ratio(ink, bg).map(|c| (ink, c)))
            .fold(
                (fg, 0.0),
                |best, cand| {
                    if cand.1 > best.1 { cand } else { best }
                },
            )
            .0
    }
}

fn bright(c: Color) -> Option<Color> {
    Some(match c {
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::Gray => Color::White,
        _ => return None,
    })
}

#[must_use]
pub fn contrast_ratio(a: Color, b: Color) -> Option<f64> {
    let (a, b) = (rgb_of(a)?, rgb_of(b)?);
    let (la, lb) = (luminance(a) + 0.05, luminance(b) + 0.05);
    Some(if la > lb { la / lb } else { lb / la })
}

pub const COLOR_LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

pub const FOCUS_BAR_POLICIES: [FocusBarPolicy; 3] = FocusBarPolicy::ALL;

pub const GUTTER_CELLS: [GutterCell; 2] = GutterCell::ALL;

pub const BACKDROP_SELECTIONS: [BackdropSelection; 2] = BackdropSelection::ALL;
