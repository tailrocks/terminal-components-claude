#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Size {
    pub min: (u16, u16),
    pub preferred: (u16, u16),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Constraints {
    pub max: (u16, u16),
    pub tight_w: bool,
    pub tight_h: bool,
}

impl Constraints {
    pub const fn loose(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: false,
            tight_h: false,
        }
    }

    pub const fn tight(w: u16, h: u16) -> Self {
        Constraints {
            max: (w, h),
            tight_w: true,
            tight_h: true,
        }
    }
}

impl Size {
    pub const fn exact(w: u16, h: u16) -> Self {
        Size {
            min: (w, h),
            preferred: (w, h),
        }
    }

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
