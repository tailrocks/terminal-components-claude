pub mod core {
    pub use termrock_core::*;
}
pub mod layout {
    pub use termrock_layout::*;
}
pub mod theme {
    pub use termrock_theme::*;
}
pub mod text {
    pub use termrock_text::*;
}
pub mod render {
    pub use termrock_render::*;
}
pub mod collections {
    pub use termrock_collections::*;
}
pub mod runtime {
    pub use termrock_runtime::*;
}
pub mod slots;
pub mod controls {
    pub use termrock_controls::*;
}
pub mod fields {
    pub use termrock_fields::*;
}
pub mod feedback {
    pub use termrock_feedback::*;
}
pub mod navigation {
    pub use termrock_navigation::*;
}
pub mod viewport {
    pub use termrock_viewport::*;
}
pub mod overlays {
    pub use termrock_overlays::*;
}
pub mod grid {
    pub use termrock_grid::*;
}
pub mod editors {
    pub use termrock_editors::*;
}
pub mod forms {
    pub use termrock_forms::*;
}
pub mod terminal {
    pub use termrock_terminal::*;
}
pub mod session {
    pub use termrock_session::*;
}
pub mod shell {
    pub use termrock_shell::*;
}
pub mod blocks;
pub mod composition;
pub mod author {
    pub use ratatui_core::buffer::{Buffer, Cell};
    pub use ratatui_core::layout::{Position, Rect};
    pub use ratatui_core::style::{Color, Style};
    pub use termrock_core::event::{
        Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind,
    };
    pub use termrock_core::id::{Id, ItemKey, Part, PartRef, Revision};
    pub use termrock_core::intent::{FocusVia, Intent, IntentIter, Phase};
    pub use termrock_core::response::{Activated, Flow, Invalidate, Response, StateFlags};
    pub use termrock_layout::{Constraints, Size};
    pub use termrock_runtime::Measure;
    pub use termrock_runtime::author::*;
    pub use termrock_runtime::focus::{FocusVis, Focusability, ScopeId, ScopeMode};
    pub use termrock_runtime::{Cx, FrameRead, LayoutFacts, ReferenceState, ReferenceTarget, Ui};
    pub use termrock_theme::PaintStyle;
    pub use termrock_theme::{
        Align, Family, FgStep, GlyphRole, Modifier, Overlay, OverlayRule, PartMetrics, Resolved,
        Role, Slot, StyleDefaults, StylePatch, Surface, SyntaxRole, Theme, Variant,
    };
}

pub use termrock_core::validate::FieldError;

pub use termrock_core::custom_hash16;
pub use termrock_core::id;
pub use termrock_core::id::{Id, ItemKey, Part, PartRef, Revision};

pub use blocks::{
    AppShell, AppShellExt, AppShellSlots, Chrome, EmptyLeaf, MasterDetail, MasterDetailAction,
    MasterDetailError, MasterDetailPolicy, MasterDetailState, Regions, ShellSlots, SplitBlock,
    SplitBlockError,
};
pub use slots::{DialogSlotExt, PickerSlotExt, PropsSlotExt};
pub use termrock_runtime::ui::LayerSlot;
pub use termrock_runtime::{
    Acc, ActivationFeedback, ActivationKey, App, ClockError, FeedbackClock, FeedbackClockError,
    Moment, PaintedFrame, PendingInput, PreparedFrame, PreparedFrameError, RenderSnapshot,
    RenderSnapshotError, Runtime, SimulationMoment, TypingPolicy, UpdateCause,
};
pub use termrock_runtime::{Cx, FrameRead, LayoutFacts, ReferenceState, ReferenceTarget, Ui};
#[cfg(feature = "testing")]
pub use termrock_runtime::{ProjectedFrame, RenderModel, StyledQuery};
pub use termrock_session::{
    DefaultTerminal, TerminalSession, chain_panic_hook, run, run_with_feedback_clock,
};

pub use termrock_core::action::{Action, ActionKey};
pub use termrock_core::diagnostics::Diagnostic;
pub use termrock_core::edit_entry::EditEntry;
pub use termrock_core::event::{Axis, Chord, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
pub use termrock_core::intent::{FocusVia, Intent, IntentIter, Phase};
pub use termrock_core::keys::{MediaKeyCode, ModifierKeyCode};
pub use termrock_core::response::{Activated, Flow, Invalidate, Response, StateFlags};

pub use termrock_collections::ScrollState;
pub use termrock_runtime::capture::Capture;
pub use termrock_runtime::focus::{
    FocusEntry, FocusRing, FocusState, FocusVis, Focusability, ScopeId, ScopeMode,
};
pub use termrock_runtime::hit::{Axes, Headroom, Hit, Region, RegionKind, Registry};
pub use termrock_runtime::{
    Binding, BindingState, BindingTableId, Bindings, ChordCase, Hint, HintKey, HintLayer, KeyMap,
    KeyPhase, binding_conflicts,
};

pub use termrock_runtime::layer::{
    Anchor, Backdrop, CrossAlign, Dismiss, DismissReason, FocusContinuationOutcome,
    LayerCompletion, LayerEvent, LayerId, LayerKind, LayerOpening, LayerSize, LayerSpec,
    ScreenAlign, Side, backdrop_area, resolve_anchor,
};

pub use termrock_theme::{
    Align, BorderSet, CapabilityPalettes, ColorLevel, ColorTokens, CursorStyle, Density,
    DesignTokens, FG_STEPS, Family, FgStep, GlyphRole, MONO_RULES_PER_FAMILY, MeterFillRest,
    MeterRole, MeterThresholds, Modifier, MonoRule, Overlay, OverlayRule, PaintStyle, PartMetrics,
    Resolved, Role, SURFACE_LEVELS, Slot, StyleDefaults, StylePatch, Surface, SyntaxRole, Theme,
    ThemeBuilder, Variant,
};

pub use termrock_layout::{Constraints, Size};
pub use termrock_layout::{Insets, Maximized, RowAlign, SplitAxis, SplitModel, Track};
pub use termrock_runtime::Measure;

pub use termrock_text::{
    CursorPos, EditAction, EditOutcome, Extend, FuzzyBoundary, Motion, Span, TextBuffer,
    TextEditorCore, fuzzy, fuzzy_with_boundary, truncate, truncate_middle, width, wrap,
    wrapped_rows,
};
pub use termrock_text::{Secret, SecretPolicy};

pub use termrock_collections::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, KeySet, Reconcile, Reconciliation, SelectMode,
};
pub use termrock_runtime::{
    CellDecor, CellUi, ColumnsUi, EmptyState, MAX_COLUMNS, RowDecor, RowFn, RowTotal, RowUi,
    ScrollbarGutter, Status,
};

pub use termrock_controls::{
    Brand, BusyMarkerPolicy, Button, ButtonCmd, Checkbox, ChoiceCmd, Empty, LabelRadio,
    MasterDetailCmd, MasterDetailLayout, MasterDetailMode, Panel, PanelKind, Props, PropsAction,
    PropsCmd, PropsList, PropsRow, PropsState, PropsValue, RadioField, RadioGroup,
    RadioGroupAction, RadioGroupState, RadioNavigation, ResetPolicy, SplitAction, SplitCmd,
    SplitPane, SplitPaneState, Toggle, TooSmall,
};

pub use termrock_fields::{
    BlurPolicy, EditPhase, ErrorState, Field, NoValidate, Note, TextAction, TextArea,
    TextAreaState, TextCmd, TextInput, TextInputState, Validate, discard_error, redacted_text,
};
pub use termrock_runtime::field_control::FieldControl;

pub use termrock_feedback::{
    DerivedHintBar, Emphasis, Group, HintBar, KeyHint, MAX_ITEMS, Meter, MeterTone, MeterVisual,
    ProgressBar, Spinner, StatusAction, StatusBar, StatusItem, TextSection,
};

pub use termrock_navigation::{
    AsItem, BadgeFn, ChipBar, ChipBarAction, ChipBarCmd, ChipBarState, FilterList,
    FilterListAction, FilterListCmd, FilterListState, FilterPolicy, Item, ItemRow, ItemRowLayout,
    List, ListAction, ListCmd, ListState, NavList, NavListAction, NavListCmd, NavListState,
    NavMode, NodeKind, SelectionPolicy, StepState, Steps, StepsAction, StepsCmd, StepsState, Tabs,
    TabsAction, TabsCmd, TabsState, Tree, TreeAction, TreeBranchActivation, TreeBranchClick,
    TreeCmd, TreeNode, TreeState,
};

pub use termrock_viewport::{
    CellPos, ProjectedText, ScrollRegion, TextViewport, ViewportAction, ViewportCmd, ViewportLine,
    ViewportState,
};
#[cfg(feature = "testing")]
pub use termrock_viewport::{ViewportWorkProbe, ViewportWorkSnapshot};

pub use termrock_overlays::{
    CommandPalette, Completion, CompletionAction, CompletionCmd, CompletionController,
    CompletionState, ContextMenu, Dialog, DialogAction, DialogCmd, DialogState, HelpAction,
    HelpCmd, HelpOverlay, HelpOverlayState, HelpSection, LabelSelect, Menu, MenuAction, MenuBar,
    MenuBarHoverScope, MenuCmd, MenuItem, MenuState, Picker, PickerAction, PickerChain,
    PickerChainAction, PickerChainCmd, PickerChainState, PickerStage, PickerState, ScopeKey,
    Select, SelectAction, SelectCmd, SelectField, SelectState,
};

pub use termrock_grid::{
    CellAction, CellRef, Column, ColumnKey, EditIntent, GRID_MAX_COLUMNS, Grid, GridAction,
    GridCell, GridCmd, GridColumnFit, GridCursorError, GridEditor, GridGutter, GridHeaderSizing,
    GridModel, GridOverflowIndicator, GridSortIndicator, GridState, NavUnit, SortDir, WidthSample,
    WidthSampleError,
};

pub use termrock_editors::{
    CodeAction, CodeCmd, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity,
    CompletingCodeEditor, CompletingCodeEditorState, DiffLineKind, DiffMode, DiffRow, DiffSource,
    DiffView, DiffViewState, Highlighter, Segmenter, TabBehavior,
};

pub use termrock_forms::{
    EnterPolicy, FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec, Form, FormAction, FormData,
    FormState, GroupKey, Wizard, WizardAction, WizardCmd, WizardState, WizardStep,
};

pub use termrock_render::{TerminalCell, TerminalCursor, TerminalSource};
pub use termrock_terminal::{
    InputToken, LinkKey, TerminalAction, TerminalInteraction, TerminalSelection, TerminalView,
    TerminalViewState,
};

pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Style};
pub use ratatui_core::symbols::{line::Set as LineSet, scrollbar::Set as ScrollbarSet};
pub use ratatui_core::terminal::Frame;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facade_reexports() {
        let id = id!("facade_sanity");
        assert_ne!(id.hash(), 0);
        let rev = Revision::zero();
        assert_eq!(rev.0, 0);
        assert_eq!(CursorStyle::default(), CursorStyle::Bar);
    }
}
