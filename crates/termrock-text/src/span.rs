use ratatui_core::style::Modifier;

use crate::theme::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span<'a> {
    pub text: &'a str,
    pub role: Option<Role>,
    pub add: Modifier,
}

impl<'a> Span<'a> {
    pub const fn new(text: &'a str) -> Self {
        Span {
            text,
            role: None,
            add: Modifier::empty(),
        }
    }

    #[must_use]
    pub const fn role(mut self, r: Role) -> Self {
        self.role = Some(r);
        self
    }

    #[must_use]
    pub const fn modifier(mut self, m: Modifier) -> Self {
        self.add = self.add.union(m);
        self
    }

    #[must_use]
    pub const fn bold(self) -> Self {
        self.modifier(Modifier::BOLD)
    }

    #[must_use]
    pub const fn italic(self) -> Self {
        self.modifier(Modifier::ITALIC)
    }

    #[must_use]
    pub const fn underlined(self) -> Self {
        self.modifier(Modifier::UNDERLINED)
    }

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
