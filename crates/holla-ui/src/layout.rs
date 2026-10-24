//! The width-band layout policy.
//!
//! A screen picks its layout from the room it has, not from the size a
//! capture happened to use. Every viewport falls into one band, and the bands
//! are the only thing a painter asks about the size of the terminal.

/// The narrowest viewport the shell renders.
pub const MIN_WIDTH: u16 = 72;
/// The shortest viewport the shell renders.
pub const MIN_HEIGHT: u16 = 20;

/// The layout band a viewport falls into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Below 80 columns: no room for a column of its own.
    Compact,
    /// 80 to 99 columns: one column, no wide parts.
    Narrow,
    /// 100 to 119 columns: one column with the mid parts.
    Mid,
    /// 120 columns and up: the wide card.
    Wide,
}

impl Band {
    /// The band a viewport of this width falls into.
    pub const fn of(width: u16) -> Self {
        match width {
            0..=79 => Self::Compact,
            80..=99 => Self::Narrow,
            100..=119 => Self::Mid,
            _ => Self::Wide,
        }
    }

    /// The band a rectangle falls into.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands_follow_the_width_not_the_captured_frame() {
        assert_eq!(Band::of(40), Band::Compact);
        assert_eq!(Band::of(72), Band::Compact);
        assert_eq!(Band::of(80), Band::Narrow);
        assert_eq!(Band::of(99), Band::Narrow);
        assert_eq!(Band::of(100), Band::Mid);
        assert_eq!(Band::of(119), Band::Mid);
        assert_eq!(Band::of(120), Band::Wide);
        assert_eq!(Band::of(160), Band::Wide);
    }

    #[test]
    fn every_width_belongs_to_exactly_one_band() {
        for width in 0..=200_u16 {
            let band = Band::of(width);
            let hits = [
                band.is_compact(),
                band.is_narrow(),
                band.is_mid(),
                band.is_wide(),
            ]
            .into_iter()
            .filter(|hit| *hit)
            .count();
            assert_eq!(hits, 1, "width {width} landed in {band:?}");
        }
    }
}
