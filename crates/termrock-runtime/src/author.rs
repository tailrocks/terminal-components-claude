use core::fmt;

mod window;
pub use window::{VerticalWindow, VerticalWindowError};

/// Publish low-priority bindings for a real focus scope. These bindings are
/// considered only after the focused control has declined the same key.
pub fn publish_scope_bindings<C: Copy + 'static>(
    ui: &mut crate::ui::Ui<'_>,
    scope: crate::focus::ScopeId,
    owner: crate::id::Id,
    flags: crate::response::StateFlags,
    table: &'static [crate::keymap::Binding<C>],
) {
    ui.publish_scope_bindings(scope, owner, flags, table);
}

mod composition;
pub use composition::{ClosureLeaf, CompositionLeaf, composition_leaf};

/// Apply only explicit style channels to an existing component-author surface.
/// Symbols, inherited channels, hit regions, and cursor state are preserved.
pub fn patch_style(
    ui: &mut crate::ui::Ui<'_>,
    area: ratatui_core::layout::Rect,
    style: crate::theme::PaintStyle,
    explicit: crate::theme::StylePatch,
) {
    ui.patch_style(area, style, explicit);
}

pub type PartPainter<'a> = dyn Fn(&mut Ui<'_>, Rect) + 'a;

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
    pub const fn new() -> Self {
        Self {
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    #[must_use]
    pub const fn global(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    #[must_use]
    pub const fn part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.parts = patches;
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, painter: &'a PartPainter<'a>) -> Self {
        self.slot = Some((part, painter));
        self
    }

    #[must_use]
    pub const fn flags(runtime: StateFlags, derived: StateFlags) -> StateFlags {
        runtime.union(derived)
    }

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

    pub fn slot_for(&self, part: Part) -> Option<&'a PartPainter<'a>> {
        match self.slot {
            Some((named, painter)) if named == part => Some(painter),
            _ => None,
        }
    }

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

pub trait LayerSlotAuthor {
    fn render_component<R>(&mut self, render: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> R;
}

/// Low-level adapter for a nested component slot clipped to its parent area.
///
/// This belongs to the component-author API: ordinary consumers should obtain
/// slots from [`Ui::layer_slot`](crate::ui::Ui::layer_slot).
pub trait NestedLayerSlotAuthor {
    fn with_nested_layer_slot<R>(
        &mut self,
        area: Rect,
        render: impl FnOnce(&mut crate::ui::LayerSlot<'_, '_>) -> R,
    ) -> R;
}

pub use crate::capture::Capture;
pub use crate::focus::{FocusVis, Focusability, ScopeId, ScopeMode};
pub use crate::hit::{Axes, Headroom, Hit, RegionKind};
pub use crate::layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, FocusContinuationOutcome,
    LayerCompletion, LayerEvent, LayerId, LayerKind, LayerOpening, LayerSize, LayerSpec,
    ScreenAlign, Side, backdrop_area, resolve_anchor,
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
    Binding, BindingExtensionError, BindingState, BindingTableId, Bindings, Hint, HintLayer,
    KeyMap, KeyPhase, binding_conflicts,
};

/// Explicitly extends an existing same-owner component binding table for this frame.
///
/// Unlike [`Ui::publish_bindings`], this does not replace the prior table. Only bindings
/// selected by `include` are added. Duplicate actions and ambiguous default chords are
/// rejected and diagnosed; key-map remapping conflicts remain visible through runtime
/// binding-conflict diagnostics.
pub fn extend_bindings<C: Copy + 'static>(
    ui: &mut Ui<'_>,
    owner: Id,
    flags: StateFlags,
    table: &'static [Binding<C>],
    include: impl Fn(C) -> bool,
) -> Result<(), BindingExtensionError> {
    ui.extend_bindings(owner, flags, table, include)
}

/// Publishes an explicitly selected subset of a static component binding table.
///
/// This keeps the replacement semantics of [`Ui::publish_bindings`]; use
/// [`extend_bindings`] when multiple controls intentionally share one owner.
pub fn publish_bindings_filtered<C: Copy + 'static>(
    ui: &mut Ui<'_>,
    owner: Id,
    flags: StateFlags,
    table: &'static [Binding<C>],
    include: impl Fn(C) -> bool,
) {
    ui.publish_bindings_filtered(owner, flags, table, include);
}

pub use crate::diagnostics::Diagnostic;
pub use crate::field_control::FieldControl;
pub use crate::secret::{Secret, SecretPolicy};
pub use termrock_core::validate::{FieldError, NoValidate, Validate};

pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};

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
