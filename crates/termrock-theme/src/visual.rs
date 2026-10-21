//! Shared focus/selection ink API.
//!
//! Ported from the PR #26 reference theme (`src/theme.rs` at
//! f4867e9b5f8a08a3bb61ce80d644198a29f51984): `gutter_symbol`, `gutter`,
//! `chosen_in`, `inactive_look`, `legible_mark`. The reference reads a
//! `VisualState` struct; the parity architecture carries widget state in
//! [`StateFlags`], so those inputs arrive as flags plus explicit arguments.
//! Defaults reproduce the reference output exactly; consumers that paint
//! through the recipe system (controls `Part::GUTTER`) keep doing so until
//! they adopt these helpers.

use ratatui_core::style::{Color, Style};

use crate::Theme;
use crate::downgrade::{luminance, rgb_of};
use crate::glyph::GlyphRole;
use crate::response::StateFlags;
use crate::tokens::ColorLevel;

/// Rendering policy for a widget's chosen item while its layer does not hold
/// keyboard focus.
///
/// Owner decision 2026-10-17 (`InactiveSelection::Accent`): the selected item
/// of a list, tree, table, sidebar or menu keeps its accent fill, and a
/// checked box, the on radio or toggle keeps its accent mark, when focus moves
/// elsewhere in the same layer. Under a modal or popup both selection kinds go
/// neutral. The reference default (neutral inactive selection) stays available
/// as [`InactiveSelection::Neutral`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum InactiveSelection {
    /// Keep the accent ink while focus is elsewhere in the same layer.
    #[default]
    Accent,
    /// Drop to the neutral primary ink when focus leaves.
    Neutral,
    /// Render no chosen-item distinction without focus.
    None,
}

impl InactiveSelection {
    pub const ALL: [InactiveSelection; 3] = [
        InactiveSelection::Accent,
        InactiveSelection::Neutral,
        InactiveSelection::None,
    ];
}

/// When a widget family paints the keyboard-focus bar in its gutter.
///
/// One shared policy for every gutter paint site. The frozen families pin
/// three distinct rules (see `frozen_ink_pins.rs` in `termrock-controls` and
/// `termrock-navigation`), so the rule is a parameter here and never a
/// per-widget fork. Tier-sensitive rules read the theme's own capability
/// level (set by [`Theme::for_level`](crate::Theme::for_level)), matching the
/// resolver.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum FocusBarPolicy {
    /// Row families that blank the bar while disabled (`LIST`): the bar needs
    /// focus and an enabled widget, at every tier. This is also the rule the
    /// pre-policy helpers baked in, so it is the default.
    #[default]
    RowBlanked,
    /// Row families that keep the bar while disabled (`CHOICE`, `TREE`,
    /// `STEPS`): focus alone draws the bar at every tier; only the mono tier
    /// blanks a disabled row's bar.
    RowKept,
    /// macOS-style controls (`BUTTON`): the bar needs focus AND focus
    /// visibility, so mouse-initiated focus shows no bar. A disabled control
    /// keeps a focus-visible bar.
    FocusVisible,
}

impl FocusBarPolicy {
    pub const ALL: [FocusBarPolicy; 3] = [
        FocusBarPolicy::RowBlanked,
        FocusBarPolicy::RowKept,
        FocusBarPolicy::FocusVisible,
    ];

    /// Whether the focus bar paints for `flags` at colour capability `level`.
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

/// How a gutter cell that is not showing the focus bar paints the plane
/// behind it.
///
/// The frozen widget path inherits the container cell (nothing stamped), so
/// blank cells stay byte-identical to the container surface; the reference
/// helper stamped an explicit blank fill in the host background instead.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GutterCell {
    /// Stamp nothing: the cell reads as whatever the container painted.
    #[default]
    Inherit,
    /// Stamp an explicit blank fill in the host background colour.
    Fill,
}

impl GutterCell {
    pub const ALL: [GutterCell; 2] = [GutterCell::Inherit, GutterCell::Fill];
}

/// How chosen-item ink reacts to a modal or popup backdrop.
///
/// The reference default sends inactive selections neutral under a backdrop.
/// The frozen checked markers (checkbox on, radio on, toggle on) keep their
/// accent ink under a backdrop, so that behaviour stays selectable; the
/// default is not flipped.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum BackdropSelection {
    /// A modal or popup backdrop neutralizes an accent selection.
    #[default]
    Neutral,
    /// Checked and chosen marks keep their accent ink under a backdrop.
    Accent,
}

impl BackdropSelection {
    pub const ALL: [BackdropSelection; 2] = [BackdropSelection::Neutral, BackdropSelection::Accent];
}

impl Theme {
    /// The glyph painted in an item's gutter column for the given state.
    ///
    /// The single keyboard-focus indicator: the phosphor bar while the widget
    /// is focused and enabled, a blank reserved cell otherwise.
    #[must_use]
    pub fn gutter_symbol(&self, flags: StateFlags) -> &'static str {
        self.focus_bar_symbol(FocusBarPolicy::RowBlanked, flags)
    }

    /// The gutter glyph under an explicit [`FocusBarPolicy`].
    ///
    /// Tier-sensitive policies read the theme's own capability level.
    #[must_use]
    pub fn focus_bar_symbol(&self, policy: FocusBarPolicy, flags: StateFlags) -> &'static str {
        if policy.shows_bar(flags, self.capability.color) {
            self.design.glyphs.get(GlyphRole::FocusBar)
        } else {
            " "
        }
    }

    /// Style for the gutter cell: the focus bar in legible focus ink, or a
    /// blank cell in the host background.
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

    /// Style for the gutter cell under an explicit [`FocusBarPolicy`] and
    /// [`GutterCell`].
    ///
    /// [`GutterCell::Inherit`] stamps only what the bar needs (the bar's fg
    /// ink, never a background), so the cell stays byte-identical to the
    /// container surface whenever no bar paints — the frozen widget path.
    /// [`GutterCell::Fill`] reproduces the reference behaviour of an explicit
    /// blank in `bg`.
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

    /// Ink for a chosen item (current page, chosen option, selected row).
    ///
    /// Focused widgets and widgets whose inactive look resolves to
    /// [`InactiveSelection::Accent`] use the accent; the rest use primary
    /// text ink.
    #[must_use]
    pub fn chosen_in(
        &self,
        policy: InactiveSelection,
        under_modal: bool,
        flags: StateFlags,
    ) -> Color {
        self.chosen_in_with(policy, under_modal, BackdropSelection::Neutral, flags)
    }

    /// Ink for a chosen item with an explicit [`BackdropSelection`].
    ///
    /// [`BackdropSelection::Accent`] reproduces the frozen checked markers,
    /// which keep the accent under a modal or popup backdrop.
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

    /// Resolve the effective inactive-selection look.
    ///
    /// An [`InactiveSelection::Accent`] policy drops to
    /// [`InactiveSelection::Neutral`] while a modal or popup backdrop is up;
    /// every other policy passes through.
    #[must_use]
    pub fn inactive_look(&self, policy: InactiveSelection, under_modal: bool) -> InactiveSelection {
        self.inactive_look_with(policy, under_modal, BackdropSelection::Neutral)
    }

    /// Resolve the effective inactive-selection look under an explicit
    /// [`BackdropSelection`].
    ///
    /// Only the [`InactiveSelection::Accent`] policy reacts to a backdrop, and
    /// only with [`BackdropSelection::Neutral`]; every other combination
    /// passes through.
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

    /// Keep a small mark (focus bar, checkbox tick, bullet) legible on `bg`.
    ///
    /// Marks may sit on accent fills and reduced color levels, so the ink may
    /// need to move: try the input, a brighter variant of it, the strong
    /// danger/success inks, primary text, and finally whichever of primary or
    /// on-accent ink reads best.
    #[must_use]
    pub fn legible_mark(&self, fg: Color, bg: Color) -> Color {
        const MARK: f64 = 3.0;
        let reads = |ink: Color| contrast_ratio(ink, bg).is_none_or(|r| r >= MARK);
        if reads(fg) {
            return fg;
        }
        // The parity token set has no error_strong/success_strong inks, so the
        // reference's strong-ink branch is inert here until those tokens land.
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

/// The brighter named-ansi variant of a color, if one exists.
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

/// WCAG 2.x contrast ratio over the terminal approximation of a color.
///
/// Colors without a known RGB ( [`Color::Reset`] ) read as `None`: a pair the
/// terminal cannot resolve never passes a legibility mark.
#[must_use]
pub fn contrast_ratio(a: Color, b: Color) -> Option<f64> {
    let (a, b) = (rgb_of(a)?, rgb_of(b)?);
    let (la, lb) = (luminance(a) + 0.05, luminance(b) + 0.05);
    Some(if la > lb { la / lb } else { lb / la })
}

/// Every color level, for exhaustive per-tier tests.
pub const COLOR_LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

/// Every focus-bar policy, for exhaustive per-policy tests.
pub const FOCUS_BAR_POLICIES: [FocusBarPolicy; 3] = FocusBarPolicy::ALL;

/// Every gutter-cell fill mode, for exhaustive tests.
pub const GUTTER_CELLS: [GutterCell; 2] = GutterCell::ALL;

/// Every backdrop-selection policy, for exhaustive tests.
pub const BACKDROP_SELECTIONS: [BackdropSelection; 2] = BackdropSelection::ALL;
