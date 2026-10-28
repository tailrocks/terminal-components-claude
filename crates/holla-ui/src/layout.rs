pub const MIN_WIDTH: u16 = 72;

pub const MIN_HEIGHT: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Compact,

    Narrow,

    Mid,

    Wide,
}

impl Band {
    pub const fn of(width: u16) -> Self {
        match width {
            0..=79 => Self::Compact,
            80..=99 => Self::Narrow,
            100..=119 => Self::Mid,
            _ => Self::Wide,
        }
    }

    pub const fn of_area(area: termrock::Rect) -> Self {
        Self::of(area.width)
    }

    pub const fn is_compact(self) -> bool {
        matches!(self, Self::Compact)
    }

    pub const fn is_narrow(self) -> bool {
        matches!(self, Self::Narrow)
    }

    pub const fn is_mid(self) -> bool {
        matches!(self, Self::Mid)
    }

    pub const fn is_wide(self) -> bool {
        matches!(self, Self::Wide)
    }
}
