#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tooltips {
    pub enabled: bool,

    pub delay_ms: u64,

    pub max_width: u16,
}

impl Tooltips {
    pub const DEFAULT: Self = Self {
        enabled: true,
        delay_ms: 600,
        max_width: 48,
    };

    pub const OFF: Self = Self {
        enabled: false,
        delay_ms: 600,
        max_width: 48,
    };

    #[must_use]
    pub const fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }

    #[must_use]
    pub const fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    #[must_use]
    pub const fn max_width(mut self, cells: u16) -> Self {
        self.max_width = cells;
        self
    }
}

impl Default for Tooltips {
    fn default() -> Self {
        Self::DEFAULT
    }
}
