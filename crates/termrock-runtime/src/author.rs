use core::fmt;

mod canvas;
pub use canvas::{
    put_cell, put_fill, put_fill_in, put_frame, put_frame_in, put_scroll_edges, put_text,
    put_text_in, put_text_w, text_width,
};

mod surface;
pub use surface::{CellView, PaintSurface};

mod window;
pub use window::{VerticalWindow, VerticalWindowError};

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

pub fn patch_style(
    ui: &mut crate::ui::Ui<'_>,
    area: ratatui_core::layout::Rect,
    style: crate::theme::PaintStyle,
    explicit: crate::theme::StylePatch,
) {
    ui.patch_style(area, style, explicit);
}

pub fn with_part_hit_precedence<R>(
    ui: &mut Ui<'_>,
    part: PartRef,
    render: impl FnOnce(&mut Ui<'_>) -> R,
) -> R {
    ui.with_part_hit_precedence(part, render)
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

pub fn extend_bindings<C: Copy + 'static>(
    ui: &mut Ui<'_>,
    owner: Id,
    flags: StateFlags,
    table: &'static [Binding<C>],
    include: impl Fn(C) -> bool,
) -> Result<(), BindingExtensionError> {
    ui.extend_bindings(owner, flags, table, include)
}

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
