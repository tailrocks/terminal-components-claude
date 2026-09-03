//! Measurement (`COMPONENT_ARCHITECTURE.md` §10).

use crate::ui::Ui;

/// A measured size: the minimum a component can use and what it prefers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Size {
    /// `(width, height)` below which the component degrades.
    pub min: (u16, u16),
    /// `(width, height)` the component would like.
    pub preferred: (u16, u16),
}

/// The space offered to `measure`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Constraints {
    /// The largest `(width, height)` available.
    pub max: (u16, u16),
    /// The width is fixed at `max.0`.
    pub tight_w: bool,
    /// The height is fixed at `max.1`.
    pub tight_h: bool,
}

impl Constraints {
    /// Loose constraints up to `w × h`.
    pub const fn loose(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: false,
            tight_h: false,
        }
    }

    /// Both axes fixed.
    pub const fn tight(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: true,
            tight_h: true,
        }
    }
}

impl Size {
    /// A size whose minimum and preferred are equal.
    pub const fn exact(w: u16, h: u16) -> Self {
        Size {
            min: (w, h),
            preferred: (w, h),
        }
    }

    /// Clip both pairs to `c.max`; tight axes report exactly `c.max`.
    #[must_use]
    pub fn fit(self, c: Constraints) -> Size {
        let clip = |(w, h): (u16, u16)| {
            (
                if c.tight_w { c.max.0 } else { w.min(c.max.0) },
                if c.tight_h { c.max.1 } else { h.min(c.max.1) },
            )
        };
        Size {
            min: clip(self.min),
            preferred: clip(self.preferred),
        }
    }
}

/// Optional measurement for components that can size themselves.
pub trait Measure {
    /// Measure against the design tokens and the offered constraints.
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_reports_min_and_preferred() {
        let s = Size {
            min: (10, 1),
            preferred: (40, 3),
        };
        let f = s.fit(Constraints::loose(20, 2));
        assert_eq!(
            f,
            Size {
                min: (10, 1),
                preferred: (20, 2)
            }
        );
        let t = s.fit(Constraints::tight(30, 5));
        assert_eq!(t, Size::exact(30, 5));
        let c = Constraints {
            max: (12, 9),
            tight_w: true,
            tight_h: false,
        };
        assert_eq!(
            s.fit(c),
            Size {
                min: (12, 1),
                preferred: (12, 3)
            }
        );
    }
}
