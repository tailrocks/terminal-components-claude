//! The role-carrying span (`COMPONENT_ARCHITECTURE.md` §22.2 item 16, Adjudication M1).
//!
//! Stores a [`Role`], not a resolved `Style`, so a viewport re-themes
//! without rebuilding and `Ui::dim_layer` can walk roles. ratatui's
//! style-carrying span is reachable as `author::raw::Span` for the
//! `raw()` escape hatch only.

use ratatui_core::style::Modifier;

use crate::theme::Role;

/// Borrowed text with a colour role and modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span<'a> {
    /// The text.
    pub text: &'a str,
    /// The foreground role; `None` inherits the part's style.
    pub role: Option<Role>,
    /// Modifiers added over the part's style.
    pub add: Modifier,
}

impl<'a> Span<'a> {
    /// Plain text inheriting the part's style.
    pub const fn new(text: &'a str) -> Self {
        Span {
            text,
            role: None,
            add: Modifier::empty(),
        }
    }

    /// Text in a role.
    #[must_use]
    pub const fn role(mut self, r: Role) -> Self {
        self.role = Some(r);
        self
    }

    /// Add modifiers.
    #[must_use]
    pub const fn modifier(mut self, m: Modifier) -> Self {
        self.add = self.add.union(m);
        self
    }

    /// Bold.
    #[must_use]
    pub const fn bold(self) -> Self {
        self.modifier(Modifier::BOLD)
    }

    /// Italic.
    #[must_use]
    pub const fn italic(self) -> Self {
        self.modifier(Modifier::ITALIC)
    }

    /// Underlined.
    #[must_use]
    pub const fn underlined(self) -> Self {
        self.modifier(Modifier::UNDERLINED)
    }

    /// Dim.
    #[must_use]
    pub const fn dim(self) -> Self {
        self.modifier(Modifier::DIM)
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(text: &'a str) -> Self {
        Span::new(text)
    }
}
