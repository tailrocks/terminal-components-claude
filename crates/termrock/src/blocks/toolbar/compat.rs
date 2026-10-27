use termrock_core::id::{Id, PartRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconLook {
    #[default]
    Close,

    Title,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IconButtonTip {
    pub text: String,

    pub shortcut: Option<String>,
}

impl IconButtonTip {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            shortcut: None,
        }
    }

    #[must_use]
    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_owned());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconButton {
    pub id: Id,

    pub glyph: Option<String>,

    pub look: IconLook,

    pub pad: u16,

    pub danger_hover: bool,

    pub tip: Option<IconButtonTip>,

    pub part: Option<(Id, PartRef)>,
}

impl IconButton {
    pub const DEFAULT_PAD: u16 = 1;

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

    pub fn new(id: Id, glyph: &str) -> Self {
        Self {
            glyph: Some(glyph.to_owned()),
            look: IconLook::Title,
            ..Self::close(id)
        }
    }

    #[must_use]
    pub fn with_id(&self, id: Id) -> Self {
        Self {
            id,
            part: None,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn with_glyph(&self, glyph: &str) -> Self {
        Self {
            glyph: Some(glyph.to_owned()),
            ..self.clone()
        }
    }

    #[must_use]
    pub fn pad(mut self, cells: u16) -> Self {
        self.pad = cells;
        self
    }

    #[must_use]
    pub fn plate(self, cells: u16) -> Self {
        self.pad(cells.saturating_sub(1) / 2)
    }

    #[must_use]
    pub fn danger_hover(mut self, enabled: bool) -> Self {
        self.danger_hover = enabled;
        self
    }

    #[must_use]
    pub fn tip(mut self, tip: IconButtonTip) -> Self {
        self.tip = Some(tip);
        self
    }
}
