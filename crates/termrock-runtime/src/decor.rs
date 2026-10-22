use crate::response::StateFlags;
use crate::theme::{GlyphRole, Role};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RowDecor<'a> {
    pub marker: Option<GlyphRole>,
    pub tone: Option<Role>,
    pub strike: bool,
    pub faint: bool,
    pub flags: StateFlags,
    pub message: Option<&'a str>,
    pub number: Option<usize>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CellDecor<'a> {
    pub tone: Option<Role>,
    pub italic: bool,
    pub error: Option<&'a str>,
    pub dirty: bool,
    pub suffix: Option<GlyphRole>,
}

impl RowDecor<'_> {
    pub const fn flags(&self) -> StateFlags {
        self.flags
    }
}

impl CellDecor<'_> {
    pub fn flags(&self) -> StateFlags {
        let mut f = StateFlags::empty();
        if self.error.is_some() {
            // The severity is `ERROR`; the value in the cell is `INVALID`.
            // Both are reported so a consumer can tell "this cell failed"
            // from "this cell holds a value that cannot be used".
            f |= StateFlags::ERROR | StateFlags::INVALID;
        }
        if self.dirty {
            f |= StateFlags::DIRTY;
        }
        f
    }
}
