//! Cell rendering primitives, paint style application, and component-author styling.

pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};

pub use termrock_core::id::Part;
pub use termrock_core::response::StateFlags;
pub use termrock_theme::{Modifier, PaintStyle, Role, StyleDefaults, StylePatch};

/// Borrowed per-instance styling and painting overrides.
#[derive(Clone, Copy, Debug)]
pub struct PartStyle<'a> {
    pub patch: Option<&'a StylePatch>,
    pub parts: &'a [(Part, StylePatch)],
}

impl<'a> PartStyle<'a> {
    /// An empty override set.
    pub const fn new() -> Self {
        Self {
            patch: None,
            parts: &[],
        }
    }

    /// Apply `patch` to every part this instance resolves.
    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Apply per-part patches.
    #[must_use]
    pub const fn part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self
    }

    /// Retrieve the patch for `part`.
    pub fn part_patch(&self, part: Part) -> Option<StylePatch> {
        let mut patch = self.patch.copied();
        for &(p, ref pt) in self.parts {
            if p == part {
                patch = Some(patch.map_or(*pt, |base| base.merge(*pt)));
            }
        }
        patch
    }

    /// Combine runtime and derived state flags.
    pub const fn flags(runtime: StateFlags, derived: StateFlags) -> StateFlags {
        StateFlags::from_bits_truncate(runtime.bits() | derived.bits())
    }
}

impl Default for PartStyle<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A prepared terminal cell for `TerminalView`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCell<'a> {
    pub symbol: &'a str,
    pub width: u8,
    pub continuation: bool,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifier: Modifier,
}

impl<'a> TerminalCell<'a> {
    pub fn new(symbol: &'a str) -> Self {
        Self {
            symbol,
            width: 1,
            continuation: false,
            fg: None,
            bg: None,
            modifier: Modifier::empty(),
        }
    }

    pub fn empty() -> Self {
        Self::new(" ")
    }
}

/// Terminal cursor description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCursor {
    pub pos: Position,
    pub visible: bool,
}

/// Caller-provided prepared-cell terminal source trait.
pub trait TerminalSource {
    fn revision(&self) -> termrock_core::Revision;
    fn size(&self) -> ratatui_core::layout::Size;
    fn cell(&self, position: Position) -> Option<TerminalCell<'_>>;
    fn cursor(&self) -> Option<TerminalCursor>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_style_and_terminal_cell() {
        let ps = PartStyle::new();
        assert!(ps.patch.is_none());
        assert!(ps.parts.is_empty());

        let cell = TerminalCell::empty();
        assert_eq!(cell.symbol, " ");
        assert_eq!(cell.width, 1);
        assert!(!cell.continuation);
    }
}
