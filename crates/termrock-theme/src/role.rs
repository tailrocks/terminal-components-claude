use ratatui_core::style::Color;

pub const SURFACE_LEVELS: usize = 5;
pub const FG_STEPS: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Surface {
    #[default]
    Canvas,
    Surface,
    Elevated,
    Overlay,
    Popover,
    Field,
    FieldHover,
}

impl Surface {
    pub const fn level(self) -> Option<usize> {
        match self {
            Surface::Canvas => Some(0),
            Surface::Surface => Some(1),
            Surface::Elevated => Some(2),
            Surface::Overlay => Some(3),
            Surface::Popover => Some(4),
            Surface::Field | Surface::FieldHover => None,
        }
    }

    pub const fn from_level(i: usize) -> Surface {
        match i {
            0 => Surface::Canvas,
            1 => Surface::Surface,
            2 => Surface::Elevated,
            3 => Surface::Overlay,
            _ => Surface::Popover,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FgStep {
    Primary,
    Secondary,
    Muted,
    Faint,
    Ghost,
}

impl FgStep {
    pub const fn index(self) -> usize {
        match self {
            FgStep::Primary => 0,
            FgStep::Secondary => 1,
            FgStep::Muted => 2,
            FgStep::Faint => 3,
            FgStep::Ghost => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SyntaxRole {
    Keyword,
    Ident,
    Str,
    Number,
    Operator,
    Punct,
    Comment,
    Plain,
    TypeName,
    Function,
    Constant,
    Invalid,
    Deprecated,
    MatchBg,
    MatchCurrentBg,
    BracketMatch,
    DiagError,
    DiagWarning,
    DiagInfo,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MeterRole {
    Low,
    Medium,
    High,
    Track,
    FillRest,
    Stale,
    Unknown,
    Series(u8),
}

/// Semantic color roles used by theme recipes and local style patches.
///
/// Downstream exhaustive matches must handle variants added by newer releases.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    CurrentSurface,
    RaisedSurface,
    HoverSurface,
    Surface(Surface),
    Fg(FgStep),
    OnAccent,
    OnDanger,
    OnSurfaceInverse,
    BorderSubtle,
    BorderStrong,
    Accent,
    AccentHover,
    AccentPressed,
    AccentTint,
    Focus,
    FocusRing,
    SelectionBg,
    SelectionFg,
    HighlightBg,
    HighlightFg,
    HighlightDangerBg,
    HighlightDangerFg,
    BackdropFg,
    BackdropBg,
    Danger,
    DangerSoft,
    DangerTint,
    Warning,
    WarningTint,
    Success,
    Info,
    DisabledFg,
    DisabledBg,
    ReadOnlyFg,
    /// The background of an application status strip.
    StatusBarBg,
    /// The primary ink of an application status strip.
    StatusBarFg,
    /// The accent ink of an application status strip.
    StatusAccent,
    /// The rule that separates application chrome from its content.
    ChromeRule,
    /// The track of a scroll bar.
    ScrollTrack,
    /// The thumb of a scroll bar.
    ScrollThumb,
    /// The ink of a small tag or label chip.
    TagInk,
    Syntax(SyntaxRole),
    Meter(MeterRole),
    Custom(Color),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

impl Role {
    pub const fn painted(self, surface: Surface) -> Self {
        match self {
            Self::HoverSurface => Self::Surface(match surface {
                Surface::Canvas | Surface::Field => Surface::Elevated,
                _ => Surface::Overlay,
            }),
            other => other,
        }
    }
}
