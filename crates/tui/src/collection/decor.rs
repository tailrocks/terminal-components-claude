//! Owner-supplied decoration (`COMPONENT_ARCHITECTURE.md` §12.2, §21 item 22).

use crate::response::StateFlags;
use crate::theme::{GlyphRole, Role};

/// Decoration for one row, supplied by the owner, never derived inside the
/// component. Not `#[non_exhaustive]`: adapters build literals.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RowDecor<'a> {
    /// A marker glyph.
    pub marker: Option<GlyphRole>,
    /// A foreground role.
    pub tone: Option<Role>,
    /// Strike the label.
    pub strike: bool,
    /// Faint the label.
    pub faint: bool,
    /// Extra state flags to wear.
    pub flags: StateFlags,
    /// A message shown beside the row.
    pub message: Option<&'a str>,
}

/// Decoration for one cell.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CellDecor<'a> {
    /// A foreground role.
    pub tone: Option<Role>,
    /// Italic.
    pub italic: bool,
    /// An error message.
    pub error: Option<&'a str>,
    /// Modified.
    pub dirty: bool,
    /// A trailing glyph.
    pub suffix: Option<GlyphRole>,
}

impl RowDecor<'_> {
    /// The flags this decoration adds: `ERROR` for a message with an error
    /// tone is the owner's call; `DIRTY`/`WARNING` come from `flags`.
    pub const fn flags(&self) -> StateFlags {
        self.flags
    }
}

impl CellDecor<'_> {
    /// The flags this decoration adds.
    pub fn flags(&self) -> StateFlags {
        let mut f = StateFlags::empty();
        if self.error.is_some() {
            f |= StateFlags::ERROR;
        }
        if self.dirty {
            f |= StateFlags::DIRTY;
        }
        f
    }
}
