//! Component-author API. Everything needed to build a component that
//! participates in theme resolution, focus, hover, press, dispatch, hit
//! testing, cursor output, scrolling, overlays, capture, testing and visual
//! capture — and nothing more (`COMPONENT_ARCHITECTURE.md` Appendix B.4,
//! Adjudication M1).
//!
//! Deliberately **not** here: `Runtime`, `run`, `TerminalSession`,
//! `Registry`, `FocusRing`, `FocusState`, `App` and the concrete
//! components. A component author drives none of those.

use core::fmt;

/// Borrowed replacement painter for one component part.
pub type PartPainter<'a> = dyn Fn(&mut Ui<'_>, Rect) + 'a;

/// Borrowed per-instance styling and painting overrides.
///
/// `PartStyle` is the component-author counterpart to the library's internal
/// override carrier. It keeps all override data borrowed and allocation-free:
/// [`global`](Self::global) applies one patch to every styled part,
/// [`part`](Self::part) applies matching patches in declaration order, and
/// [`slot`](Self::slot) replaces one part's painter while leaving layout,
/// registration and interaction with the component. [`style`](Self::style)
/// is the single resolution path and records the query in testing builds.
///
/// The type is `Copy` so a component can keep it in ordinary borrowed props and
/// consume it through fluent builders.
#[derive(Clone, Copy)]
pub struct PartStyle<'a> {
    pub(crate) patch: Option<&'a StylePatch>,
    pub(crate) parts: &'a [(Part, StylePatch)],
    pub(crate) slot: Option<(Part, &'a PartPainter<'a>)>,
}

impl fmt::Debug for PartStyle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PartStyle")
            .field("patch", &self.patch)
            .field("parts", &self.parts.len())
            .field("slot", &self.slot.map(|(part, _)| part))
            .finish()
    }
}

impl<'a> PartStyle<'a> {
    /// An empty override set.
    pub const fn new() -> Self {
        Self {
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    /// Apply `patch` to every part this instance resolves.
    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Apply the matching patches to their named parts in declaration order.
    #[must_use]
    pub const fn part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self
    }

    /// Apply `patch` to every part; compatibility spelling for component APIs.
    #[must_use]
    pub const fn patch(self, patch: &'a StylePatch) -> Self {
        self.global(patch)
    }

    /// Apply matching named-part patches; compatibility spelling for component APIs.
    #[must_use]
    pub const fn patch_part(self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.part(patches)
    }

    /// Replace the painter for `part` while preserving the component's geometry
    /// and interaction registrations.
    #[must_use]
    pub const fn slot(mut self, part: Part, painter: &'a PartPainter<'a>) -> Self {
        self.slot = Some((part, painter));
        self
    }

    /// Combine runtime-owned and component-derived state flags.
    #[must_use]
    pub const fn flags(runtime: StateFlags, derived: StateFlags) -> StateFlags {
        runtime.union(derived)
    }

    /// Return the merged patch for `part`, if any patch applies.
    pub fn part_patch(&self, part: Part) -> Option<StylePatch> {
        let mut merged = self.patch.copied();
        for (named, patch) in self.parts {
            if *named == part {
                merged = Some(match merged {
                    Some(base) => base.merge(*patch),
                    None => *patch,
                });
            }
        }
        merged
    }

    /// Return the replacement painter for `part`, if one was configured.
    pub fn slot_for(&self, part: Part) -> Option<&'a PartPainter<'a>> {
        match self.slot {
            Some((named, painter)) if named == part => Some(painter),
            _ => None,
        }
    }

    /// Resolve one part through theme resolution and this instance's patches.
    ///
    /// In testing builds this also records `(owner, family, variant, part)` so
    /// declared-part checks can verify the component's style coverage.
    pub fn style(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        let resolved = match self.part_patch(part) {
            Some(patch) => ui.style_patched(family, variant, part, flags, &patch),
            None => ui.style(family, variant, part, flags),
        };
        self.note(ui, owner, family, variant, part, resolved);
        resolved
    }

    /// Record a resolved part in testing builds; this is a no-op otherwise.
    pub fn note(
        &self,
        ui: &mut Ui<'_>,
        owner: Id,
        family: Family,
        variant: Variant,
        part: Part,
        resolved: Resolved,
    ) {
        #[cfg(feature = "testing")]
        ui.note_styled(owner, family, variant, part, resolved);
        #[cfg(not(feature = "testing"))]
        let _ = (ui, owner, family, variant, part, resolved);
    }
}

impl Default for PartStyle<'_> {
    fn default() -> Self {
        Self::new()
    }
}

// identity and parts — the NAMED items, never the module itself: re-exporting
// `crate::id` widens the surface unintentionally, and the `id!` macro is
// `#[macro_export]` and already reachable at the root (F21, MI-11).
pub use crate::id::{Id, ItemKey, Part, PartRef};
// phases and plumbing
pub use crate::event::{Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
pub use crate::intent::{FocusVia, Intent, IntentIter, Phase};
pub use crate::response::{Activated, Flow, Invalidate, Response, StateFlags};
pub use crate::ui::{Cx, FrameRead, LayoutFacts, ReferenceState, ReferenceTarget, Ui};
// registration services
pub use crate::capture::Capture;
pub use crate::focus::{FocusVis, Focusability, ScopeId, ScopeMode};
pub use crate::hit::{Axes, Headroom, Hit, RegionKind};
pub use crate::layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, LayerEvent, LayerId, LayerKind,
    LayerSize, LayerSpec, ScreenAlign, Side, backdrop_area, resolve_anchor,
};
pub use crate::scroll::ScrollState;
// theme resolution
pub use crate::theme::border;
pub use crate::theme::{
    Align, ColorLevel, Density, DesignTokens, FG_STEPS, Family, FgStep, GlyphRole,
    MONO_RULES_PER_FAMILY, MeterRole, MeterThresholds, Modifier, MonoRule, Overlay, OverlayRule,
    PartMetrics, Resolved, Role, SURFACE_LEVELS, Slot, StateRule, StylePatch, Surface, SyntaxRole,
    Theme, Variant,
};
// layout and measurement
pub use crate::layout::{self, Insets, RowAlign, SplitModel, Track};
pub use crate::measure::{Constraints, Measure, Size};
// text — curated: the core types are public author-facing editing storage;
// grapheme internals remain private (Appendix B.4).
pub use crate::text::{
    CursorPos, EditAction, EditOutcome, Extend, Motion, Span, TextBuffer, TextEditorCore, fuzzy,
    truncate, truncate_middle, width, wrap, wrapped_rows,
};
// collections
pub use crate::collection::{
    ByIndex, CellDecor, CellUi, CollectionCore, ColumnsUi, DefaultRow, EmptyState, KeyFn, KeySet,
    Reconcile, Reconciliation, RowDecor, RowFn, RowTotal, RowUi, SelectMode, Status,
};
// bindings and hints
pub use crate::action::{Action, ActionKey};
pub use crate::keymap::{
    Binding, BindingState, BindingTableId, Bindings, Hint, HintLayer, KeyMap, KeyPhase,
    binding_conflicts,
};
// errors and diagnostics
pub use crate::diagnostics::Diagnostic;
pub use crate::{FieldControl, FieldError, NoValidate, Secret, SecretPolicy, Validate};
// ratatui-core types a painter needs (`ratatui_core::` paths, never the umbrella crate)
pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};

/// Types needed only to drive the `Ui::raw()` / `RowUi::raw()` escape hatch.
/// The only re-export not forced by a signature. `raw::Span` is ratatui's
/// style-carrying span and is written qualified, always: `raw::Span`.
pub mod raw {
    pub use ratatui_core::text::{Line, Span, Text};
}

#[cfg(test)]
mod tests {
    use super::*;

    const GLOBAL: StylePatch = StylePatch::new().set_fg(Role::Accent);
    const LABEL_A: StylePatch = StylePatch::new().set_bg(Role::Danger);
    const LABEL_B: StylePatch = StylePatch::new().add(Modifier::BOLD);
    const PARTS: &[(Part, StylePatch)] = &[(Part::LABEL, LABEL_A), (Part::LABEL, LABEL_B)];

    #[test]
    fn part_style_merges_global_and_part_patches_in_order() {
        let styles = PartStyle::new().global(&GLOBAL).part(PARTS);
        let internal = crate::components::Overrides::new()
            .patch(&GLOBAL)
            .patch_part(PARTS);

        assert_eq!(
            styles.part_patch(Part::LABEL),
            Some(GLOBAL.merge(LABEL_A).merge(LABEL_B))
        );
        assert_eq!(
            styles.part_patch(Part::LABEL),
            internal.part_patch(Part::LABEL)
        );
        assert_eq!(
            styles.part_patch(Part::BODY),
            internal.part_patch(Part::BODY)
        );
        assert_eq!(styles.part_patch(Part::BODY), Some(GLOBAL));
        assert_eq!(PartStyle::new().part(PARTS).part_patch(Part::BODY), None);
    }

    #[test]
    fn part_style_flags_match_internal_override_union() {
        let runtime = StateFlags::FOCUSED | StateFlags::HOVERED;
        let derived = StateFlags::SELECTED | StateFlags::DISABLED;
        assert_eq!(PartStyle::flags(runtime, derived), runtime | derived);
    }

    #[test]
    fn part_style_slot_lookup_is_part_scoped() {
        fn paint(_: &mut Ui<'_>, _: Rect) {}

        let styles = PartStyle::new().slot(Part::LABEL, &paint);
        assert!(styles.slot_for(Part::LABEL).is_some());
        assert!(styles.slot_for(Part::BODY).is_none());
    }
}
