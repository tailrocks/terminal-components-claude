#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(
    test,
    allow(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines,
        clippy::many_single_char_names,
        clippy::unreadable_literal,
        clippy::items_after_statements
    )
)]

pub(crate) mod action;
pub(crate) mod capture;
pub(crate) mod collection;
pub(crate) mod cursor;
pub(crate) mod diagnostics;
pub(crate) mod event;
pub(crate) mod field_control;
pub(crate) mod focus;
pub(crate) mod hit;
pub(crate) mod id;
pub(crate) mod intent;
pub(crate) mod keymap;
pub(crate) mod layer;
pub mod layout;
pub(crate) mod measure;
pub(crate) mod response;
pub(crate) mod runtime;
pub(crate) mod scroll;
pub(crate) mod secret;
pub mod text;
pub mod theme;
pub(crate) mod ui;
pub(crate) mod validate;

pub mod author;

// ── the application-author facade (Appendix B.3 item 2: one curated line each) ──

// identity
pub use id::{Id, ItemKey, Part, PartRef};
// runtime
#[cfg(feature = "crossterm")]
pub use runtime::session::{DefaultTerminal, TerminalSession, chain_panic_hook, run};
pub use runtime::{App, Runtime};
// phases
pub use ui::{Cx, FrameRead, LayoutFacts, Ui};
// events, intents, responses
pub use event::{Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
pub use intent::{FocusVia, Intent, IntentIter, Phase};
pub use response::{Activated, Flow, Invalidate, Response, StateFlags};
// keymaps, actions, diagnostics
pub use action::{Action, ActionKey};
pub use diagnostics::Diagnostic;
pub use keymap::{
    Binding, BindingState, Bindings, Hint, HintLayer, KeyMap, KeyPhase, binding_conflicts,
};
// focus, hit, capture, scroll
pub use capture::Capture;
pub use focus::{FocusEntry, FocusRing, FocusState, FocusVis, Focusability, ScopeId, ScopeMode};
pub use hit::{Axes, Headroom, Hit, Region, RegionKind, Registry};
pub use scroll::ScrollState;
// layers
pub use layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, LayerEvent, LayerId, LayerKind,
    LayerSpec, ScreenAlign, Side, backdrop_area, resolve_anchor,
};
// theme
pub use theme::{
    Align, ColorLevel, ColorTokens, Density, DesignTokens, Family, FgStep, GlyphRole, MeterRole,
    Modifier, Overlay, OverlayRule, Resolved, Role, Slot, StylePatch, Surface, SyntaxRole, Theme,
    ThemeBuilder, Variant,
};
// layout and measurement
pub use layout::{Insets, Maximized, RowAlign, SplitAxis, SplitModel, Track};
pub use measure::{Constraints, Measure, Size};
// text
pub use text::{CursorPos, EditAction, EditOutcome, Extend, Motion, Span, TextEditorCore};
// collections
pub use collection::{
    ByIndex, CellDecor, CellUi, CollectionCore, ColumnsUi, DefaultRow, EmptyState, KeyFn, KeySet,
    MAX_COLUMNS, Reconcile, Reconciliation, RowDecor, RowFn, RowTotal, RowUi, SelectMode, Status,
};
// fields and secrets
pub use field_control::FieldControl;
pub use secret::{Secret, SecretPolicy};
pub use validate::{FieldError, NoValidate, Validate};
// ratatui-core types named by the public surface (Adjudication M1)
pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};
pub use ratatui_core::terminal::Frame;
