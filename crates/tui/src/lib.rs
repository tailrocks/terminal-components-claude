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
pub mod components;
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
pub(crate) mod text;
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
#[cfg(feature = "testing")]
pub use ui::StyledQuery;
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
    LayerSize, LayerSpec, ScreenAlign, Side, backdrop_area, resolve_anchor,
};
// theme
pub use theme::{
    Align, ColorLevel, ColorTokens, Density, DesignTokens, Family, FgStep, GlyphRole, MeterRole,
    Modifier, Overlay, OverlayRule, PartMetrics, Resolved, Role, Slot, StylePatch, Surface,
    SyntaxRole, Theme, ThemeBuilder, Variant,
};
// layout and measurement
pub use layout::{Insets, Maximized, RowAlign, SplitAxis, SplitModel, Track};
pub use measure::{Constraints, Measure, Size};
// text — `text` is `pub(crate)` (Appendix B.3 item 2): `grapheme_width`,
// `is_word_char` and `thousands` are internal, and the rest is curated here
pub use text::{
    CursorPos, EditAction, EditOutcome, Extend, Motion, Span, TextBuffer, TextEditorCore, fuzzy,
    truncate, truncate_middle, width, wrap, wrapped_rows,
};
// collections
pub use collection::{
    ByIndex, CellDecor, CellUi, CollectionCore, ColumnsUi, DefaultRow, EmptyState, KeyFn, KeySet,
    MAX_COLUMNS, Reconcile, Reconciliation, RowDecor, RowFn, RowTotal, RowUi, SelectMode, Status,
};
// components (Slice 4, append-only region, alphabetical)
pub use components::{
    BlurPolicy, Button, ButtonCmd, Dialog, DialogAction, DialogCmd, DialogState, EditPhase, Field,
    List, ListAction, ListCmd, ListState, Props, ScrollRegion, Tabs, TabsAction, TabsCmd,
    TabsState, TextAction, TextCmd, TextInput, TextInputState,
};
// components — work package 4B (fields, inputs, textarea, select, choice, chips),
// appended as its own line so the shared list above is never rewritten
pub use components::{
    Checkbox, ChipBar, ChipBarAction, ChipBarCmd, ChipBarState, ChoiceCmd, LabelChips, LabelRadio,
    LabelSelect, RadioGroup, RadioGroupAction, RadioGroupState, Select, SelectAction, SelectCmd,
    SelectState, TextArea, TextAreaState, Toggle,
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
