//! Application-wide policy for ambient hover tooltips.

/// Default timing and width for tooltips that inherit the application policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tooltips {
    /// Whether inherited tooltips are enabled.
    pub enabled: bool,
    /// Delay in milliseconds before an inherited tooltip opens.
    pub delay_ms: u64,
    /// Maximum tooltip content width in terminal cells.
    pub max_width: u16,
}

impl Tooltips {
    /// Enabled tooltips with a 600 ms delay and a 48 cell content width.
    pub const DEFAULT: Self = Self {
        enabled: true,
        delay_ms: 600,
        max_width: 48,
    };

    /// Disabled tooltips, retaining the default delay and width.
    pub const OFF: Self = Self {
        enabled: false,
        delay_ms: 600,
        max_width: 48,
    };

    /// Set whether inherited tooltips are enabled.
    #[must_use]
    pub const fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }

    /// Set the delay in milliseconds before an inherited tooltip opens.
    #[must_use]
    pub const fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Set the maximum tooltip content width in terminal cells.
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
