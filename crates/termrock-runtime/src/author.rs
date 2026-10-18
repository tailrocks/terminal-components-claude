#![doc = r" Component-author API. Everything needed to build a component that"]
#![doc = r" participates in theme resolution, focus, hover, press, dispatch, hit"]
#![doc = r" testing, cursor output, scrolling, overlays, capture, testing and visual"]
#![doc = r" capture — and nothing more (`COMPONENT_ARCHITECTURE.md` Appendix B.4,"]
#![doc = r" Adjudication M1)."]
#![doc = r""]
#![doc = r" Deliberately **not** here: `Runtime`, `run`, `TerminalSession`,"]
#![doc = r" `Registry`, `FocusRing`, `FocusState`, `App` and the concrete"]
#![doc = r" components. A component author drives none of those."]

use core::fmt;

#[doc = r" Borrowed replacement painter for one component part."]
pub type PartPainter<'a> = dyn Fn(&mut Ui<'_>, Rect) + 'a;

#[doc = r" Borrowed per-instance styling and painting overrides."]
#[doc = r""]
#[doc = r" `PartStyle` is the shared component-author styling carrier. It keeps all"]
#[doc = r" override data borrowed and allocation-free:"]
#[doc = r" [`global`](Self::global) applies one patch to every styled part,"]
#[doc = r" [`part`](Self::part) applies matching patches in declaration order, and"]
#[doc = r" [`slot`](Self::slot) replaces one part's painter while leaving layout,"]
#[doc = r" registration and interaction with the component. [`style`](Self::style)"]
#[doc = r" is the single resolution path and records the query in testing builds."]
#[doc = r""]
#[doc = r" The type is `Copy` so a component can keep it in ordinary borrowed props and"]
#[doc = r" consume it through fluent builders."]
#[derive(Clone, Copy)]
pub struct PartStyle<'a> {
    pub patch: Option<&'a StylePatch>,
    pub parts: &'a [(Part, StylePatch)],
    pub slot: Option<(Part, &'a PartPainter<'a>)>,
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
    #[doc = r" An empty override set."]
    pub const fn new() -> Self {
        Self {
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    #[doc = r" Apply `patch` to every part this instance resolves."]
    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    #[doc = r" Apply the matching patches to their named parts in declaration order."]
    #[must_use]
    pub const fn part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self
    }

    #[doc = r" Replace the painter for `part` while preserving the component's geometry"]
    #[doc = r" and interaction registrations."]
    #[must_use]
    pub const fn slot(mut self, part: Part, painter: &'a PartPainter<'a>) -> Self {
        self.slot = Some((part, painter));
        self
    }

    #[doc = r" Combine runtime-owned and component-derived state flags."]
    #[must_use]
    pub const fn flags(runtime: StateFlags, derived: StateFlags) -> StateFlags {
        runtime.union(derived)
    }

    #[doc = r" Return the merged patch for `part`, if any patch applies."]
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

    #[doc = r" Return the replacement painter for `part`, if one was configured."]
    pub fn slot_for(&self, part: Part) -> Option<&'a PartPainter<'a>> {
        match self.slot {
            Some((named, painter)) if named == part => Some(painter),
            _ => None,
        }
    }

    #[doc = r" Resolve one part through theme resolution and this instance's patches."]
    #[doc = r""]
    #[doc = r" In testing builds this also records `(owner, family, variant, part)` so"]
    #[doc = r" declared-part checks can verify the component's style coverage."]
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

    #[doc = r" Record a resolved part in testing builds; this is a no-op otherwise."]
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

pub use crate::id::{Id, ItemKey, Part, PartRef};

pub use crate::event::{Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
pub use crate::intent::{FocusVia, Intent, IntentIter, Phase};
pub use crate::response::{Activated, Flow, Invalidate, Response, StateFlags};
pub use crate::ui::{Cx, FrameRead, LayoutFacts, ReferenceState, ReferenceTarget, Ui};

pub use crate::capture::Capture;
pub use crate::focus::{FocusVis, Focusability, ScopeId, ScopeMode};
pub use crate::hit::{Axes, Headroom, Hit, RegionKind};
pub use crate::layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, LayerEvent, LayerId, LayerKind,
    LayerSize, LayerSpec, ScreenAlign, Side, backdrop_area, resolve_anchor,
};
pub use crate::scroll::ScrollState;

pub use crate::theme::border;
pub use crate::theme::{
    Align, CapabilityPalettes, ColorLevel, Density, DesignTokens, FG_STEPS, Family, FgStep,
    GlyphRole, MONO_RULES_PER_FAMILY, MeterFillRest, MeterRole, MeterThresholds, Modifier,
    MonoRule, Overlay, OverlayRule, PaintStyle, PartMetrics, Resolved, Role, SURFACE_LEVELS, Slot,
    StateRule, StyleDefaults, StylePatch, Surface, SyntaxRole, Theme, Variant,
};

pub use crate::layout::{self, Insets, RowAlign, SplitModel, Track};
pub use crate::measure::{Constraints, Size};
pub use crate::ui::measure::Measure;

pub use crate::text::{
    CursorPos, EditAction, EditOutcome, Extend, FuzzyBoundary, Motion, Span, TextBuffer,
    TextEditorCore, fuzzy, fuzzy_with_boundary, truncate, truncate_middle, width, wrap,
    wrapped_rows,
};

pub use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, KeySet, Reconcile, Reconciliation,
};

pub use crate::action::{Action, ActionKey};
pub use crate::keymap::{
    Binding, BindingState, BindingTableId, Bindings, Hint, HintLayer, KeyMap, KeyPhase,
    binding_conflicts,
};

pub use crate::diagnostics::Diagnostic;
pub use crate::field_control::FieldControl;
pub use crate::secret::{Secret, SecretPolicy};
pub use termrock_core::validate::{FieldError, NoValidate, Validate};

pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};

#[doc = r" Types needed only to drive the `Ui::raw()` / `RowUi::raw()` escape hatch."]
#[doc = r" The only re-export not forced by a signature. `raw::Span` is ratatui's"]
#[doc = r" style-carrying span and is written qualified, always: `raw::Span`."]
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
        let internal = PartStyle::new().global(&GLOBAL).part(PARTS);

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
