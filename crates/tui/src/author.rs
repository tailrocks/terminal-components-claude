//! Component-author API. Everything needed to build a component that
//! participates in theme resolution, focus, hover, press, dispatch, hit
//! testing, cursor output, scrolling, overlays, capture, testing and visual
//! capture — and nothing more (`COMPONENT_ARCHITECTURE.md` Appendix B.4,
//! Adjudication M1).
//!
//! Deliberately **not** here: `Runtime`, `run`, `TerminalSession`,
//! `Registry`, `FocusRing`, `FocusState`, `App` and the concrete
//! components. A component author drives none of those.

// identity and parts
pub use crate::id;
pub use crate::id::{Id, ItemKey, Part, PartRef};
// phases and plumbing
pub use crate::event::{Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
pub use crate::intent::{FocusVia, Intent, IntentIter, Phase};
pub use crate::response::{Activated, Flow, Invalidate, Response, StateFlags};
pub use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
// registration services
pub use crate::capture::Capture;
pub use crate::focus::{FocusVis, Focusability, ScopeId, ScopeMode};
pub use crate::hit::{Axes, Headroom, Hit, RegionKind};
pub use crate::layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, LayerEvent, LayerId, LayerKind,
    LayerSpec, ScreenAlign, Side, backdrop_area, resolve_anchor,
};
pub use crate::scroll::ScrollState;
// theme resolution
pub use crate::theme::border;
pub use crate::theme::{
    Align, ColorLevel, Density, DesignTokens, Family, FgStep, GlyphRole, MeterRole, Modifier,
    Overlay, OverlayRule, Resolved, Role, Slot, StateRule, StylePatch, Surface, SyntaxRole, Theme,
    Variant,
};
// layout and measurement
pub use crate::layout::{self, Insets, RowAlign, SplitModel, Track};
pub use crate::measure::{Constraints, Measure, Size};
// text
pub use crate::text::{
    CursorPos, EditAction, EditOutcome, Extend, Motion, Span, TextEditorCore, fuzzy, truncate,
    truncate_middle, width, wrap,
};
// collections
pub use crate::collection::{
    ByIndex, CellDecor, CellUi, CollectionCore, ColumnsUi, DefaultRow, EmptyState, KeyFn, KeySet,
    Reconcile, Reconciliation, RowDecor, RowFn, RowTotal, RowUi, SelectMode, Status,
};
// bindings and hints
pub use crate::action::{Action, ActionKey};
pub use crate::keymap::{
    Binding, BindingState, Bindings, Hint, HintLayer, KeyMap, KeyPhase, binding_conflicts,
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
