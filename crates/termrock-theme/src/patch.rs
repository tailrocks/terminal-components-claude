use ratatui_core::style::Modifier;

use super::glyph::GlyphRole;
use super::role::{Align, Role};
use crate::response::StateFlags;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub enum Slot<T> {
    #[default]
    Inherit,
    Set(T),
    Clear,
}

impl<T: Copy> Slot<T> {
    #[must_use]
    pub const fn over(self, base: Slot<T>) -> Slot<T> {
        match self {
            Slot::Inherit => base,
            o => o,
        }
    }

    pub const fn get(self) -> Option<T> {
        match self {
            Slot::Set(v) => Some(v),
            _ => None,
        }
    }

    pub const fn speaks(self) -> bool {
        !matches!(self, Slot::Inherit)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct StylePatch {
    pub fg: Slot<Role>,
    pub bg: Slot<Role>,
    pub underline: Slot<Role>,
    pub add: Modifier,
    pub remove: Modifier,
    pub glyph: Slot<GlyphRole>,
    pub size: Slot<u16>,
    pub align: Slot<Align>,
}

impl StylePatch {
    pub const fn new() -> Self {
        StylePatch {
            fg: Slot::Inherit,
            bg: Slot::Inherit,
            underline: Slot::Inherit,
            add: Modifier::empty(),
            remove: Modifier::empty(),
            glyph: Slot::Inherit,
            size: Slot::Inherit,
            align: Slot::Inherit,
        }
    }

    #[must_use]
    pub const fn set_fg(mut self, r: Role) -> Self {
        self.fg = Slot::Set(r);
        self
    }

    #[must_use]
    pub const fn clear_fg(mut self) -> Self {
        self.fg = Slot::Clear;
        self
    }

    #[must_use]
    pub const fn set_bg(mut self, r: Role) -> Self {
        self.bg = Slot::Set(r);
        self
    }

    #[must_use]
    pub const fn clear_bg(mut self) -> Self {
        self.bg = Slot::Clear;
        self
    }

    #[must_use]
    pub const fn set_underline(mut self, r: Role) -> Self {
        self.underline = Slot::Set(r);
        self
    }

    #[must_use]
    pub const fn add(mut self, m: Modifier) -> Self {
        self.add = self.add.union(m);
        self.remove = self.remove.difference(m);
        self
    }

    #[must_use]
    pub const fn remove(mut self, m: Modifier) -> Self {
        self.remove = self.remove.union(m);
        self.add = self.add.difference(m);
        self
    }

    #[must_use]
    pub const fn set_glyph(mut self, g: GlyphRole) -> Self {
        self.glyph = Slot::Set(g);
        self
    }

    #[must_use]
    pub const fn set_size(mut self, n: u16) -> Self {
        self.size = Slot::Set(n);
        self
    }

    #[must_use]
    pub const fn set_align(mut self, a: Align) -> Self {
        self.align = Slot::Set(a);
        self
    }

    #[must_use]
    pub const fn merge(self, over: StylePatch) -> StylePatch {
        StylePatch {
            fg: over.fg.over(self.fg),
            bg: over.bg.over(self.bg),
            underline: over.underline.over(self.underline),
            add: self.add.difference(over.remove).union(over.add),
            remove: self.remove.difference(over.add).union(over.remove),
            glyph: over.glyph.over(self.glyph),
            size: over.size.over(self.size),
            align: over.align.over(self.align),
        }
    }

    pub const fn is_empty(&self) -> bool {
        !self.fg.speaks()
            && !self.bg.speaks()
            && !self.underline.speaks()
            && self.add.is_empty()
            && self.remove.is_empty()
            && !self.glyph.speaks()
            && !self.size.speaks()
            && !self.align.speaks()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StateRule {
    pub when: StateFlags,
    pub patch: StylePatch,
}

impl StateRule {
    pub const fn matches(&self, live: StateFlags) -> bool {
        live.contains(self.when)
    }

    pub const fn specificity(&self) -> u32 {
        self.when.bits().count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::role::FgStep;

    const A: StylePatch = StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD);
    const B: StylePatch = StylePatch::new()
        .set_bg(Role::Danger)
        .remove(Modifier::BOLD);
    const C: StylePatch = StylePatch::new()
        .set_fg(Role::Fg(FgStep::Muted))
        .add(Modifier::ITALIC);

    #[test]
    fn slot_over_prefers_the_speaking_side() {
        assert_eq!(Slot::Set(1).over(Slot::Set(2)), Slot::Set(1));
        assert_eq!(Slot::Inherit.over(Slot::Set(2)), Slot::Set(2));
        assert_eq!(Slot::<u8>::Clear.over(Slot::Set(2)), Slot::Clear);
        assert_eq!(Slot::Set(1).over(Slot::Clear), Slot::Set(1));
        assert_eq!(Slot::<u8>::Inherit.over(Slot::Inherit), Slot::Inherit);
    }

    #[test]
    fn patch_merge_identity() {
        assert_eq!(A.merge(StylePatch::default()), A);
        assert_eq!(StylePatch::default().merge(A), A);
    }

    #[test]
    fn patch_merge_absorption() {
        assert_eq!(A.merge(A), A);
        assert_eq!(A.merge(B).merge(B), A.merge(B));
    }

    #[test]
    fn patch_merge_is_associative() {
        assert_eq!(A.merge(B).merge(C), A.merge(B.merge(C)));
        assert_eq!(C.merge(A).merge(B), C.merge(A.merge(B)));
    }

    #[test]
    fn patch_clear_resolves_to_inherited_surface_fg() {
        let p = A.merge(StylePatch::new().clear_fg());
        assert_eq!(p.fg, Slot::Clear);
        assert_eq!(p.fg.get(), None);
        assert!(p.fg.speaks());
    }

    #[test]
    fn modifier_add_then_remove_is_symmetric() {
        let add = StylePatch::new().add(Modifier::BOLD);
        let rem = StylePatch::new().remove(Modifier::BOLD);
        let r1 = add.merge(rem);
        assert!(r1.add.is_empty() && r1.remove.contains(Modifier::BOLD));
        let r2 = rem.merge(add);
        assert!(r2.remove.is_empty() && r2.add.contains(Modifier::BOLD));
        assert_eq!(
            StylePatch::new()
                .add(Modifier::BOLD)
                .remove(Modifier::BOLD)
                .add,
            Modifier::empty()
        );
        assert!(StylePatch::new().is_empty());
        assert!(!add.is_empty());
    }

    #[test]
    fn state_rule_matches_only_when_when_is_a_subset() {
        let r = StateRule {
            when: StateFlags::HOVERED | StateFlags::PRESSED,
            patch: A,
        };
        assert!(r.matches(StateFlags::HOVERED | StateFlags::PRESSED | StateFlags::FOCUSED));
        assert!(!r.matches(StateFlags::HOVERED));
        assert_eq!(r.specificity(), 2);
    }
}
