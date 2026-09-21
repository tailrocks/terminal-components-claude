//! Finite colour, size, and origin axes (CP-COMMON).

use junie_tui::ColorLevel;

/// Colour capability levels. CP-COMMON requires truecolor/ANSI256/ANSI16/mono.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorSpec {
    /// 24-bit / truecolor.
    TrueColor,
    /// 256-colour palette.
    Ansi256,
    /// 16 ANSI colours.
    Ansi16,
    /// Monochrome / none / nocolor observation alias.
    Mono,
}

impl ColorSpec {
    /// Full capability axis: every Direct frame expands across all four.
    pub const AXIS: [Self; 4] = [Self::TrueColor, Self::Ansi256, Self::Ansi16, Self::Mono];

    /// Stable identity token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "256",
            Self::Ansi16 => "16",
            Self::Mono => "mono",
        }
    }

    /// Production theme capability.
    #[must_use]
    pub const fn level(self) -> ColorLevel {
        match self {
            Self::TrueColor => ColorLevel::TrueColor,
            Self::Ansi256 => ColorLevel::Ansi256,
            Self::Ansi16 => ColorLevel::Ansi16,
            Self::Mono => ColorLevel::Mono,
        }
    }

    /// Parse a colour token, including frozen-suite aliases.
    #[must_use]
    pub fn from_token(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "truecolor" | "24bit" => Some(Self::TrueColor),
            "256" | "ansi256" => Some(Self::Ansi256),
            "16" | "ansi16" => Some(Self::Ansi16),
            "none" | "mono" | "nocolor" => Some(Self::Mono),
            _ => None,
        }
    }
}

/// Component allocation size. CP-COMMON requires 120x40 and 40x10.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TerminalSize {
    /// Columns.
    pub cols: u16,
    /// Rows.
    pub rows: u16,
}

impl TerminalSize {
    /// CP-COMMON component allocation axis.
    pub const AXIS: [Self; 2] = [
        Self {
            cols: 120,
            rows: 40,
        },
        Self { cols: 40, rows: 10 },
    ];

    /// Construct a size.
    #[must_use]
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }

    /// Identity token `120x40`.
    #[must_use]
    pub fn token(self) -> String {
        format!("{}x{}", self.cols, self.rows)
    }

    /// Parse `120x40`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let (cols, rows) = value.split_once('x')?;
        Some(Self {
            cols: cols.parse().ok()?,
            rows: rows.parse().ok()?,
        })
    }

    /// Frame rectangle at the origin.
    #[must_use]
    pub const fn rect(self) -> junie_tui::Rect {
        junie_tui::Rect::new(0, 0, self.cols, self.rows)
    }
}

/// Widget placement origin inside the frame. CP-COMMON requires zero and
/// nonzero-origin clipping proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Origin {
    /// Column offset.
    pub x: u16,
    /// Row offset.
    pub y: u16,
}

impl Origin {
    /// Origin axis: zero and one fixed nonzero offset.
    pub const AXIS: [Self; 2] = [Self { x: 0, y: 0 }, Self { x: 7, y: 4 }];

    /// Identity token `o0-0`.
    #[must_use]
    pub fn token(self) -> String {
        format!("o{}-{}", self.x, self.y)
    }
}
