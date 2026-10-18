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
            f |= StateFlags::ERROR;
        }
        if self.dirty {
            f |= StateFlags::DIRTY;
        }
        f
    }
}
