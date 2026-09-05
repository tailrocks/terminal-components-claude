//! Style patches and their merge laws (`COMPONENT_ARCHITECTURE.md` §11.3).

use ratatui_core::style::Modifier;

use super::glyph::GlyphRole;
use super::role::{Align, Role};
use crate::response::StateFlags;

/// One slot of a patch: say nothing, set, or clear.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub enum Slot<T> {
    /// Inherit whatever the lower layer says.
    #[default]
    Inherit,
    /// Set a value.
    Set(T),
    /// Clear: resolve to "no value" (the inherited surface colour).
    Clear,
}

impl<T: Copy> Slot<T> {
    /// `self` over `base`: `self` wins where it speaks.
    #[must_use]
    pub const fn over(self, base: Slot<T>) -> Slot<T> {
        match self {
            Slot::Inherit => base,
            o => o,
        }
    }

    /// The set value, if any.
    pub const fn get(self) -> Option<T> {
        match self {
            Slot::Set(v) => Some(v),
            _ => None,
        }
    }

    /// Whether the slot speaks.
    pub const fn speaks(self) -> bool {
        !matches!(self, Slot::Inherit)
    }
}

/// A role-level style delta. `const`-constructible.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct StylePatch {
    /// Foreground role.
    pub fg: Slot<Role>,
    /// Background role.
    pub bg: Slot<Role>,
    /// Underline colour role.
    pub underline: Slot<Role>,
    /// Modifiers to add.
    pub add: Modifier,
    /// Modifiers to remove.
    pub remove: Modifier,
    /// Glyph for the part.
    pub glyph: Slot<GlyphRole>,
    /// Size for the part.
    pub size: Slot<u16>,
    /// Text alignment.
    pub align: Slot<Align>,
}

impl StylePatch {
    /// The empty patch.
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

    /// Set the foreground role.
    #[must_use]
    pub const fn set_fg(mut self, r: Role) -> Self {
        self.fg = Slot::Set(r);
        self
    }

    /// Clear the foreground.
    #[must_use]
    pub const fn clear_fg(mut self) -> Self {
        self.fg = Slot::Clear;
        self
    }

    /// Set the background role.
    #[must_use]
    pub const fn set_bg(mut self, r: Role) -> Self {
        self.bg = Slot::Set(r);
        self
    }

    /// Clear the background.
    #[must_use]
    pub const fn clear_bg(mut self) -> Self {
        self.bg = Slot::Clear;
        self
    }

    /// Set the underline colour role.
    #[must_use]
    pub const fn set_underline(mut self, r: Role) -> Self {
        self.underline = Slot::Set(r);
        self
    }

    /// Add modifiers (and stop removing them).
    #[must_use]
    pub const fn add(mut self, m: Modifier) -> Self {
        self.add = self.add.union(m);
        self.remove = self.remove.difference(m);
        self
    }

    /// Remove modifiers (and stop adding them).
    #[must_use]
    pub const fn remove(mut self, m: Modifier) -> Self {
        self.remove = self.remove.union(m);
        self.add = self.add.difference(m);
        self
    }

    /// Set the glyph.
    #[must_use]
    pub const fn set_glyph(mut self, g: GlyphRole) -> Self {
        self.glyph = Slot::Set(g);
        self
    }

    /// Clear the glyph while retaining its reserved cell and geometry.
    ///
    /// This is distinct from omitting a glyph (`Inherit`): a component that
    /// owns the cell must paint the reserved cell blank for `Clear`.
    #[must_use]
    pub const fn clear_glyph(mut self) -> Self {
        self.glyph = Slot::Clear;
        self
    }

    /// Set the size.
    #[must_use]
    pub const fn set_size(mut self, n: u16) -> Self {
        self.size = Slot::Set(n);
        self
    }

    /// Set the alignment.
    #[must_use]
    pub const fn set_align(mut self, a: Align) -> Self {
        self.align = Slot::Set(a);
        self
    }

    /// `over` wins where it speaks. A later `remove` beats an earlier `add`
    /// and vice versa (modifier symmetry).
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

    /// Whether the patch says nothing at all.
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

/// A state rule: a patch applied when `when ⊆ live`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StateRule {
    /// The flags that must all be live.
    pub when: StateFlags,
    /// The patch.
    pub patch: StylePatch,
}

impl StateRule {
    /// Whether the rule applies to `live`.
    pub const fn matches(&self, live: StateFlags) -> bool {
        live.contains(self.when)
    }

    /// The specificity: the number of flags required.
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
    fn patch_clear_glyph_is_explicit_and_overrides_a_set() {
        let p = StylePatch::new()
            .set_glyph(GlyphRole::Chosen)
            .merge(StylePatch::new().clear_glyph());
        assert_eq!(p.glyph, Slot::Clear);
        assert!(p.glyph.speaks());
        assert_eq!(StylePatch::new().clear_glyph().glyph.get(), None);
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
