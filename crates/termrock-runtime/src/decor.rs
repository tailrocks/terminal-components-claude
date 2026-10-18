#![doc = r" Owner-supplied decoration (`COMPONENT_ARCHITECTURE.md` §12.2, §21 item 22)."]

use crate::response::StateFlags;
use crate::theme::{GlyphRole, Role};

#[doc = r" Decoration for one row, supplied by the owner, never derived inside the"]
#[doc = r" component. Not `#[non_exhaustive]`: adapters build literals."]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RowDecor<'a> {
    #[doc = r" A marker glyph."]
    pub marker: Option<GlyphRole>,
    #[doc = r" A foreground role."]
    pub tone: Option<Role>,
    #[doc = r" Strike the label."]
    pub strike: bool,
    #[doc = r" Faint the label."]
    pub faint: bool,
    #[doc = r" Extra state flags to wear."]
    pub flags: StateFlags,
    #[doc = r" A message shown beside the row."]
    pub message: Option<&'a str>,
    #[doc = r" One-based display number for the row-number gutter. `None` falls back"]
    #[doc = r" to the on-screen position, so a sorted view supplies its source order"]
    #[doc = r" here instead of on the model trait."]
    pub number: Option<usize>,
}

#[doc = r" Decoration for one cell."]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CellDecor<'a> {
    #[doc = r" A foreground role."]
    pub tone: Option<Role>,
    #[doc = r" Italic."]
    pub italic: bool,
    #[doc = r" An error message."]
    pub error: Option<&'a str>,
    #[doc = r" Modified."]
    pub dirty: bool,
    #[doc = r" A trailing glyph."]
    pub suffix: Option<GlyphRole>,
}

impl RowDecor<'_> {
    #[doc = r" The flags this decoration adds: `ERROR` for a message with an error"]
    #[doc = r" tone is the owner's call; `DIRTY`/`WARNING` come from `flags`."]
    pub const fn flags(&self) -> StateFlags {
        self.flags
    }
}

impl CellDecor<'_> {
    #[doc = r" The flags this decoration adds."]
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
