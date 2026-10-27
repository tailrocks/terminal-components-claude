use termrock_core::id::{Id, PartRef};

/// Historical icon-button look retained as Toolbar More configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconLook {
    /// Use the Close-style More appearance.
    #[default]
    Close,
    /// Use the Title-style More appearance.
    Title,
}

/// Owned tip text and optional display-only shortcut.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IconButtonTip {
    /// Main tip text.
    pub text: String,
    /// Optional display-only shortcut.
    pub shortcut: Option<String>,
}

impl IconButtonTip {
    /// Create tip data with no shortcut.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            shortcut: None,
        }
    }

    /// Add display-only shortcut text.
    #[must_use]
    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_owned());
        self
    }
}

/// Data-only historical icon-button configuration for a Toolbar's More control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconButton {
    /// Historical control identifier. Toolbar retains its own native owner.
    pub id: Id,
    /// Optional glyph; `None` selects the Toolbar theme fallback.
    pub glyph: Option<String>,
    /// More-control appearance.
    pub look: IconLook,
    /// Horizontal padding on each side of the glyph.
    pub pad: u16,
    /// Whether pointer hover and press use the danger appearance.
    pub danger_hover: bool,
    /// Optional tip data; `None` restores the Toolbar default tip.
    pub tip: Option<IconButtonTip>,
    /// Optional historical owner and part metadata, retained as public data.
    pub part: Option<(Id, PartRef)>,
}

impl IconButton {
    /// Default horizontal padding on each side of the glyph.
    pub const DEFAULT_PAD: u16 = 1;

    /// Create the default Close-style button configuration.
    pub fn close(id: Id) -> Self {
        Self {
            id,
            glyph: None,
            look: IconLook::Close,
            pad: Self::DEFAULT_PAD,
            danger_hover: false,
            tip: None,
            part: None,
        }
    }

    /// Create a Title-style button configuration with an owned glyph.
    pub fn new(id: Id, glyph: &str) -> Self {
        Self {
            glyph: Some(glyph.to_owned()),
            look: IconLook::Title,
            ..Self::close(id)
        }
    }

    /// Clone this configuration with a new id and no part metadata.
    #[must_use]
    pub fn with_id(&self, id: Id) -> Self {
        Self {
            id,
            part: None,
            ..self.clone()
        }
    }

    /// Clone this configuration with an owned glyph.
    #[must_use]
    pub fn with_glyph(&self, glyph: &str) -> Self {
        Self {
            glyph: Some(glyph.to_owned()),
            ..self.clone()
        }
    }

    /// Set horizontal padding on each side of the glyph.
    #[must_use]
    pub fn pad(mut self, cells: u16) -> Self {
        self.pad = cells;
        self
    }

    /// Set total plate width, rounded down to symmetric cell padding.
    #[must_use]
    pub fn plate(self, cells: u16) -> Self {
        self.pad(cells.saturating_sub(1) / 2)
    }

    /// Set whether pointer hover and press use the danger appearance.
    #[must_use]
    pub fn danger_hover(mut self, enabled: bool) -> Self {
        self.danger_hover = enabled;
        self
    }

    /// Set owned tooltip data.
    #[must_use]
    pub fn tip(mut self, tip: IconButtonTip) -> Self {
        self.tip = Some(tip);
        self
    }
}
