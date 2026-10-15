# Termrock public API map

Source of truth: `junie-tui` root exports at target 2e07d6f7 (crates/tui/src/lib.rs).
Destination = the restored PR #17 crate structure at the integration head.
Every item keeps its name; access paths move from the flat `junie_tui::` root to the
facade sections (`termrock::core`, `termrock::controls`, ...) or the owning crate.

| Public item | Old path (junie_tui) | Destination |
|---|---|---|
| `Action` | `action::` | `termrock::core::action` |
| `ActionKey` | `action::` | `termrock::core::action` |
| `Activated` | `response::` | `termrock::core::response` |
| `ActivationFeedback` | `runtime::` | `termrock::runtime::runtime` |
| `ActivationKey` | `runtime::` | `termrock::runtime::runtime` |
| `Align` | `theme::` | `termrock::theme::theme` |
| `Anchor` | `layer::` | `termrock::runtime::layer` |
| `App` | `runtime::` | `termrock::runtime::runtime` |
| `AsItem` | `components::` | `termrock::controls` |
| `Axes` | `hit::` | `termrock::runtime::hit` |
| `Axis` | `event::` | `termrock::core::event` |
| `Backdrop` | `layer::` | `termrock::runtime::layer` |
| `BadgeFn` | `components::` | `termrock::controls` |
| `Binding` | `keymap::` | `termrock::runtime::keymap` |
| `BindingState` | `keymap::` | `termrock::runtime::keymap` |
| `BindingTableId` | `keymap::` | `termrock::runtime::keymap` |
| `Bindings` | `keymap::` | `termrock::runtime::keymap` |
| `BlurPolicy` | `components::` | `termrock::controls` |
| `BorderSet` | `theme::` | `termrock::theme::theme` |
| `Brand` | `components::` | `termrock::controls` |
| `Buffer` | `ratatui_core::buffer::` | `termrock::author (ratatui_core re-export)` |
| `Button` | `components::` | `termrock::controls` |
| `ButtonCmd` | `components::` | `termrock::controls` |
| `ByIndex` | `collection::` | `termrock::collections` |
| `CapabilityPalettes` | `theme::` | `termrock::theme::theme` |
| `Capture` | `capture` | `termrock::runtime::capture` |
| `Cell` | `ratatui_core::buffer::` | `termrock::author (ratatui_core re-export)` |
| `CellAction` | `components::` | `termrock::controls` |
| `CellDecor` | `collection::` | `termrock::collections` |
| `CellPos` | `components::` | `termrock::controls` |
| `CellRef` | `components::` | `termrock::controls` |
| `CellUi` | `collection::` | `termrock::collections` |
| `Checkbox` | `components::` | `termrock::controls` |
| `ChipBar` | `components::` | `termrock::controls` |
| `ChipBarAction` | `components::` | `termrock::controls` |
| `ChipBarCmd` | `components::` | `termrock::controls` |
| `ChipBarState` | `components::` | `termrock::controls` |
| `ChoiceCmd` | `components::` | `termrock::controls` |
| `Chord` | `event::` | `termrock::core::event` |
| `ChordCase` | `keymap::` | `termrock::runtime::keymap` |
| `ClockError` | `runtime::` | `termrock::runtime::runtime` |
| `CodeAction` | `components::` | `termrock::controls` |
| `CodeCmd` | `components::` | `termrock::controls` |
| `CodeDiagnostic` | `components::` | `termrock::controls` |
| `CodeEditor` | `components::` | `termrock::controls` |
| `CodeEditorState` | `components::` | `termrock::controls` |
| `CodeSeverity` | `components::` | `termrock::controls` |
| `CollectionCore` | `collection::` | `termrock::collections` |
| `Color` | `ratatui_core::style::` | `termrock::author (ratatui_core re-export)` |
| `ColorLevel` | `theme::` | `termrock::theme::theme` |
| `ColorTokens` | `theme::` | `termrock::theme::theme` |
| `Column` | `components::` | `termrock::controls` |
| `ColumnKey` | `components::` | `termrock::controls` |
| `ColumnsUi` | `collection::` | `termrock::collections` |
| `CommandPalette` | `components::` | `termrock::controls` |
| `Completion` | `components::` | `termrock::controls` |
| `CompletionAction` | `components::` | `termrock::controls` |
| `CompletionCmd` | `components::` | `termrock::controls` |
| `CompletionController` | `components::` | `termrock::controls` |
| `CompletionState` | `components::` | `termrock::controls` |
| `Constraints` | `measure::` | `termrock::layout::measure` |
| `ContextMenu` | `components::` | `termrock::controls` |
| `CrossAlign` | `layer::` | `termrock::runtime::layer` |
| `CursorPos` | `text::` | `termrock::text::text` |
| `Cx` | `ui::` | `termrock::runtime::ui` |
| `DefaultRow` | `collection::` | `termrock::collections` |
| `DefaultTerminal` | `runtime::session::` | `termrock::session (termrock-runtime runtime::session)` |
| `Density` | `theme::` | `termrock::theme::theme` |
| `DerivedHintBar` | `components::` | `termrock::controls` |
| `DesignTokens` | `theme::` | `termrock::theme::theme` |
| `Diagnostic` | `diagnostics` | `termrock::core::diagnostics` |
| `Dialog` | `components::` | `termrock::controls` |
| `DialogAction` | `components::` | `termrock::controls` |
| `DialogCmd` | `components::` | `termrock::controls` |
| `DialogState` | `components::` | `termrock::controls` |
| `DiffLineKind` | `components::` | `termrock::controls` |
| `DiffMode` | `components::` | `termrock::controls` |
| `DiffRow` | `components::` | `termrock::controls` |
| `DiffSource` | `components::` | `termrock::controls` |
| `DiffView` | `components::` | `termrock::controls` |
| `DiffViewState` | `components::` | `termrock::controls` |
| `Dismiss` | `layer::` | `termrock::runtime::layer` |
| `DismissReason` | `layer::` | `termrock::runtime::layer` |
| `EditAction` | `text::` | `termrock::text::text` |
| `EditIntent` | `components::` | `termrock::controls` |
| `EditOutcome` | `text::` | `termrock::text::text` |
| `EditPhase` | `components::` | `termrock::controls` |
| `Emphasis` | `components::` | `termrock::controls` |
| `Empty` | `components::` | `termrock::controls` |
| `EmptyState` | `collection::` | `termrock::collections` |
| `EnterPolicy` | `components::` | `termrock::controls` |
| `Extend` | `text::` | `termrock::text::text` |
| `FG_STEPS` | `theme::` | `termrock::theme::theme` |
| `Family` | `theme::` | `termrock::theme::theme` |
| `FeedbackClock` | `runtime::` | `termrock::runtime::runtime` |
| `FeedbackClockError` | `runtime::` | `termrock::runtime::runtime` |
| `FgStep` | `theme::` | `termrock::theme::theme` |
| `Field` | `components::` | `termrock::controls` |
| `FieldControl` | `field_control` | `termrock::runtime::field_control` |
| `FieldError` | `validate::` | `termrock::core::validate` |
| `FieldKind` | `components::` | `termrock::controls` |
| `FieldMut` | `components::` | `termrock::controls` |
| `FieldRef` | `components::` | `termrock::controls` |
| `FieldSpan` | `components::` | `termrock::controls` |
| `FieldSpec` | `components::` | `termrock::controls` |
| `FilterList` | `components::` | `termrock::controls` |
| `FilterListAction` | `components::` | `termrock::controls` |
| `FilterListCmd` | `components::` | `termrock::controls` |
| `FilterListState` | `components::` | `termrock::controls` |
| `FilterPolicy` | `components::` | `termrock::controls` |
| `Flow` | `response::` | `termrock::core::response` |
| `FocusEntry` | `focus::` | `termrock::runtime::focus` |
| `FocusRing` | `focus::` | `termrock::runtime::focus` |
| `FocusState` | `focus::` | `termrock::runtime::focus` |
| `FocusVia` | `intent::` | `termrock::core::intent` |
| `FocusVis` | `focus::` | `termrock::runtime::focus` |
| `Focusability` | `focus::` | `termrock::runtime::focus` |
| `Form` | `components::` | `termrock::controls` |
| `FormAction` | `components::` | `termrock::controls` |
| `FormData` | `components::` | `termrock::controls` |
| `FormState` | `components::` | `termrock::controls` |
| `Frame` | `ratatui_core::terminal` | `termrock::author (ratatui_core re-export)` |
| `FrameRead` | `ui::` | `termrock::runtime::ui` |
| `FuzzyBoundary` | `text::` | `termrock::text::text` |
| `GRID_MAX_COLUMNS` | `components::` | `termrock::controls` |
| `GlyphRole` | `theme::` | `termrock::theme::theme` |
| `Grid` | `components::` | `termrock::controls` |
| `GridAction` | `components::` | `termrock::controls` |
| `GridCell` | `components::` | `termrock::controls` |
| `GridCmd` | `components::` | `termrock::controls` |
| `GridColumnFit` | `components::` | `termrock::controls` |
| `GridCursorError` | `components::` | `termrock::controls` |
| `GridEditor` | `components::` | `termrock::controls` |
| `GridGutter` | `components::` | `termrock::controls` |
| `GridHeaderSizing` | `components::` | `termrock::controls` |
| `GridModel` | `components::` | `termrock::controls` |
| `GridSortIndicator` | `components::` | `termrock::controls` |
| `GridState` | `components::` | `termrock::controls` |
| `Group` | `components::` | `termrock::controls` |
| `GroupKey` | `components::` | `termrock::controls` |
| `Headroom` | `hit::` | `termrock::runtime::hit` |
| `HelpAction` | `components::` | `termrock::controls` |
| `HelpCmd` | `components::` | `termrock::controls` |
| `HelpOverlay` | `components::` | `termrock::controls` |
| `HelpOverlayState` | `components::` | `termrock::controls` |
| `HelpSection` | `components::` | `termrock::controls` |
| `Highlighter` | `components::` | `termrock::controls` |
| `Hint` | `keymap::` | `termrock::runtime::keymap` |
| `HintBar` | `components::` | `termrock::controls` |
| `HintKey` | `keymap::` | `termrock::runtime::keymap` |
| `HintLayer` | `keymap::` | `termrock::runtime::keymap` |
| `Hit` | `hit::` | `termrock::runtime::hit` |
| `Id` | `id::` | `termrock::core::id` |
| `Input` | `event::` | `termrock::core::event` |
| `Insets` | `layout::` | `termrock::layout::layout` |
| `Intent` | `intent::` | `termrock::core::intent` |
| `IntentIter` | `intent::` | `termrock::core::intent` |
| `Invalidate` | `response::` | `termrock::core::response` |
| `Item` | `components::` | `termrock::controls` |
| `ItemKey` | `id::` | `termrock::core::id` |
| `ItemRow` | `components::` | `termrock::controls` |
| `ItemRowLayout` | `components::` | `termrock::controls` |
| `Key` | `event::` | `termrock::core::event` |
| `KeyCode` | `event::` | `termrock::core::event` |
| `KeyFn` | `collection::` | `termrock::collections` |
| `KeyHint` | `components::` | `termrock::controls` |
| `KeyMap` | `keymap::` | `termrock::runtime::keymap` |
| `KeyModifiers` | `event::` | `termrock::core::event` |
| `KeyPhase` | `keymap::` | `termrock::runtime::keymap` |
| `KeySet` | `collection::` | `termrock::collections` |
| `LabelChips` | `components::` | `termrock::controls` |
| `LabelRadio` | `components::` | `termrock::controls` |
| `LabelSelect` | `components::` | `termrock::controls` |
| `LayerEvent` | `layer::` | `termrock::runtime::layer` |
| `LayerId` | `layer::` | `termrock::runtime::layer` |
| `LayerKind` | `layer::` | `termrock::runtime::layer` |
| `LayerSize` | `layer::` | `termrock::runtime::layer` |
| `LayerSpec` | `layer::` | `termrock::runtime::layer` |
| `LayoutFacts` | `ui::` | `termrock::runtime::ui` |
| `List` | `components::` | `termrock::controls` |
| `ListAction` | `components::` | `termrock::controls` |
| `ListCmd` | `components::` | `termrock::controls` |
| `ListState` | `components::` | `termrock::controls` |
| `MAX_COLUMNS` | `collection::` | `termrock::collections` |
| `MAX_ITEMS` | `components::` | `termrock::controls` |
| `MONO_RULES_PER_FAMILY` | `theme::` | `termrock::theme::theme` |
| `Maximized` | `layout::` | `termrock::layout::layout` |
| `Measure` | `measure::` | `termrock::layout::measure` |
| `MediaKeyCode` | `keys::` | `termrock::core::keys` |
| `Menu` | `components::` | `termrock::controls` |
| `MenuAction` | `components::` | `termrock::controls` |
| `MenuBar` | `components::` | `termrock::controls` |
| `MenuCmd` | `components::` | `termrock::controls` |
| `MenuItem` | `components::` | `termrock::controls` |
| `MenuState` | `components::` | `termrock::controls` |
| `Meter` | `components::` | `termrock::controls` |
| `MeterFillRest` | `theme::` | `termrock::theme::theme` |
| `MeterRole` | `theme::` | `termrock::theme::theme` |
| `MeterThresholds` | `theme::` | `termrock::theme::theme` |
| `MeterTone` | `components::` | `termrock::controls` |
| `MeterVisual` | `components::` | `termrock::controls` |
| `Modifier` | `theme::` | `termrock::theme::theme` |
| `ModifierKeyCode` | `keys::` | `termrock::core::keys` |
| `Moment` | `runtime::` | `termrock::runtime::runtime` |
| `MonoRule` | `theme::` | `termrock::theme::theme` |
| `Motion` | `text::` | `termrock::text::text` |
| `Mouse` | `event::` | `termrock::core::event` |
| `MouseKind` | `event::` | `termrock::core::event` |
| `NavList` | `components::` | `termrock::controls` |
| `NavListAction` | `components::` | `termrock::controls` |
| `NavListCmd` | `components::` | `termrock::controls` |
| `NavListState` | `components::` | `termrock::controls` |
| `NavMode` | `components::` | `termrock::controls` |
| `NavUnit` | `components::` | `termrock::controls` |
| `NoValidate` | `validate::` | `termrock::core::validate` |
| `NodeKind` | `components::` | `termrock::controls` |
| `Overlay` | `theme::` | `termrock::theme::theme` |
| `OverlayRule` | `theme::` | `termrock::theme::theme` |
| `PaintStyle` | `theme::` | `termrock::theme::theme` |
| `PaintedFrame` | `runtime::` | `termrock::runtime::runtime` |
| `Panel` | `components::` | `termrock::controls` |
| `PanelKind` | `components::` | `termrock::controls` |
| `Part` | `id::` | `termrock::core::id` |
| `PartMetrics` | `theme::` | `termrock::theme::theme` |
| `PartRef` | `id::` | `termrock::core::id` |
| `PendingInput` | `runtime::` | `termrock::runtime::runtime` |
| `Phase` | `intent::` | `termrock::core::intent` |
| `Picker` | `components::` | `termrock::controls` |
| `PickerAction` | `components::` | `termrock::controls` |
| `PickerChain` | `components::` | `termrock::controls` |
| `PickerChainAction` | `components::` | `termrock::controls` |
| `PickerChainCmd` | `components::` | `termrock::controls` |
| `PickerChainState` | `components::` | `termrock::controls` |
| `PickerStage` | `components::` | `termrock::controls` |
| `PickerState` | `components::` | `termrock::controls` |
| `Position` | `ratatui_core::layout::` | `termrock::author (ratatui_core re-export)` |
| `ProgressBar` | `components::` | `termrock::controls` |
| `ProjectedFrame` | `runtime::` | `termrock::runtime::runtime` |
| `ProjectedText` | `components::` | `termrock::controls` |
| `Props` | `components::` | `termrock::controls` |
| `PropsAction` | `components::` | `termrock::controls` |
| `PropsCmd` | `components::` | `termrock::controls` |
| `PropsList` | `components::` | `termrock::controls` |
| `PropsRow` | `components::` | `termrock::controls` |
| `PropsState` | `components::` | `termrock::controls` |
| `PropsValue` | `components::` | `termrock::controls` |
| `RadioField` | `components::` | `termrock::controls` |
| `RadioGroup` | `components::` | `termrock::controls` |
| `RadioGroupAction` | `components::` | `termrock::controls` |
| `RadioGroupState` | `components::` | `termrock::controls` |
| `Reconcile` | `collection::` | `termrock::collections` |
| `Reconciliation` | `collection::` | `termrock::collections` |
| `Rect` | `ratatui_core::layout::` | `termrock::author (ratatui_core re-export)` |
| `ReferenceState` | `ui::` | `termrock::runtime::ui` |
| `ReferenceTarget` | `ui::` | `termrock::runtime::ui` |
| `Region` | `hit::` | `termrock::runtime::hit` |
| `RegionKind` | `hit::` | `termrock::runtime::hit` |
| `Registry` | `hit::` | `termrock::runtime::hit` |
| `RenderModel` | `runtime::` | `termrock::runtime::runtime` |
| `RenderSnapshot` | `runtime::` | `termrock::runtime::runtime` |
| `RenderSnapshotError` | `runtime::` | `termrock::runtime::runtime` |
| `Resolved` | `theme::` | `termrock::theme::theme` |
| `Response` | `response::` | `termrock::core::response` |
| `Role` | `theme::` | `termrock::theme::theme` |
| `RowAlign` | `layout::` | `termrock::layout::layout` |
| `RowDecor` | `collection::` | `termrock::collections` |
| `RowFn` | `collection::` | `termrock::collections` |
| `RowTotal` | `collection::` | `termrock::collections` |
| `RowUi` | `collection::` | `termrock::collections` |
| `Runtime` | `runtime::` | `termrock::runtime::runtime` |
| `SURFACE_LEVELS` | `theme::` | `termrock::theme::theme` |
| `ScopeId` | `focus::` | `termrock::runtime::focus` |
| `ScopeKey` | `components::` | `termrock::controls` |
| `ScopeMode` | `focus::` | `termrock::runtime::focus` |
| `ScreenAlign` | `layer::` | `termrock::runtime::layer` |
| `ScrollRegion` | `components::` | `termrock::controls` |
| `ScrollState` | `scroll` | `termrock::collections::scroll` |
| `Secret` | `secret::` | `termrock::text::secret` |
| `SecretPolicy` | `secret::` | `termrock::text::secret` |
| `Segmenter` | `components::` | `termrock::controls` |
| `Select` | `components::` | `termrock::controls` |
| `SelectAction` | `components::` | `termrock::controls` |
| `SelectCmd` | `components::` | `termrock::controls` |
| `SelectField` | `components::` | `termrock::controls` |
| `SelectMode` | `collection::` | `termrock::collections` |
| `SelectState` | `components::` | `termrock::controls` |
| `Side` | `layer::` | `termrock::runtime::layer` |
| `SimulationMoment` | `runtime::` | `termrock::runtime::runtime` |
| `Size` | `measure::` | `termrock::layout::measure` |
| `Slot` | `theme::` | `termrock::theme::theme` |
| `SortDir` | `components::` | `termrock::controls` |
| `Span` | `text::` | `termrock::text::text` |
| `Spinner` | `components::` | `termrock::controls` |
| `SplitAction` | `components::` | `termrock::controls` |
| `SplitAxis` | `layout::` | `termrock::layout::layout` |
| `SplitCmd` | `components::` | `termrock::controls` |
| `SplitModel` | `layout::` | `termrock::layout::layout` |
| `SplitPane` | `components::` | `termrock::controls` |
| `SplitPaneState` | `components::` | `termrock::controls` |
| `StateFlags` | `response::` | `termrock::core::response` |
| `Status` | `collection::` | `termrock::collections` |
| `StatusAction` | `components::` | `termrock::controls` |
| `StatusBar` | `components::` | `termrock::controls` |
| `StatusItem` | `components::` | `termrock::controls` |
| `StepState` | `components::` | `termrock::controls` |
| `Steps` | `components::` | `termrock::controls` |
| `StepsAction` | `components::` | `termrock::controls` |
| `StepsCmd` | `components::` | `termrock::controls` |
| `StepsState` | `components::` | `termrock::controls` |
| `Style` | `ratatui_core::style::` | `termrock::author (ratatui_core re-export)` |
| `StyleDefaults` | `theme::` | `termrock::theme::theme` |
| `StylePatch` | `theme::` | `termrock::theme::theme` |
| `StyledQuery` | `ui` | `termrock::runtime::ui` |
| `Surface` | `theme::` | `termrock::theme::theme` |
| `SyntaxRole` | `theme::` | `termrock::theme::theme` |
| `TabBehavior` | `components::` | `termrock::controls` |
| `Tabs` | `components::` | `termrock::controls` |
| `TabsAction` | `components::` | `termrock::controls` |
| `TabsCmd` | `components::` | `termrock::controls` |
| `TabsState` | `components::` | `termrock::controls` |
| `TerminalSession` | `runtime::session::` | `termrock::session (termrock-runtime runtime::session)` |
| `TextAction` | `components::` | `termrock::controls` |
| `TextArea` | `components::` | `termrock::controls` |
| `TextAreaState` | `components::` | `termrock::controls` |
| `TextBuffer` | `text::` | `termrock::text::text` |
| `TextCmd` | `components::` | `termrock::controls` |
| `TextEditorCore` | `text::` | `termrock::text::text` |
| `TextInput` | `components::` | `termrock::controls` |
| `TextInputState` | `components::` | `termrock::controls` |
| `TextViewport` | `components::` | `termrock::controls` |
| `Theme` | `theme::` | `termrock::theme::theme` |
| `ThemeBuilder` | `theme::` | `termrock::theme::theme` |
| `Toggle` | `components::` | `termrock::controls` |
| `TooSmall` | `components::` | `termrock::controls` |
| `Track` | `layout::` | `termrock::layout::layout` |
| `Tree` | `components::` | `termrock::controls` |
| `TreeAction` | `components::` | `termrock::controls` |
| `TreeBranchActivation` | `components::` | `termrock::controls` |
| `TreeBranchClick` | `components::` | `termrock::controls` |
| `TreeCmd` | `components::` | `termrock::controls` |
| `TreeNode` | `components::` | `termrock::controls` |
| `TreeState` | `components::` | `termrock::controls` |
| `TypingPolicy` | `runtime::` | `termrock::runtime::runtime` |
| `Ui` | `ui::` | `termrock::runtime::ui` |
| `UpdateCause` | `runtime::` | `termrock::runtime::runtime` |
| `Validate` | `validate::` | `termrock::core::validate` |
| `Variant` | `theme::` | `termrock::theme::theme` |
| `ViewportAction` | `components::` | `termrock::controls` |
| `ViewportCmd` | `components::` | `termrock::controls` |
| `ViewportLine` | `components::` | `termrock::controls` |
| `ViewportState` | `components::` | `termrock::controls` |
| `ViewportWorkProbe` | `components::` | `termrock::controls` |
| `ViewportWorkSnapshot` | `components::` | `termrock::controls` |
| `WidthSample` | `components::` | `termrock::controls` |
| `WidthSampleError` | `components::` | `termrock::controls` |
| `Wizard` | `components::` | `termrock::controls` |
| `WizardAction` | `components::` | `termrock::controls` |
| `WizardCmd` | `components::` | `termrock::controls` |
| `WizardState` | `components::` | `termrock::controls` |
| `WizardStep` | `components::` | `termrock::controls` |
| `backdrop_area` | `layer::` | `termrock::runtime::layer` |
| `binding_conflicts` | `keymap::` | `termrock::runtime::keymap` |
| `chain_panic_hook` | `runtime::session::` | `termrock::session (termrock-runtime runtime::session)` |
| `fuzzy` | `text::` | `termrock::text::text` |
| `fuzzy_with_boundary` | `text::` | `termrock::text::text` |
| `line::Set` | `ratatui_core::symbols::` | `termrock::author (ratatui_core re-export)` |
| `resolve_anchor` | `layer::` | `termrock::runtime::layer` |
| `run` | `runtime::session::` | `termrock::session (termrock-runtime runtime::session)` |
| `run_with_feedback_clock` | `runtime::session::` | `termrock::session (termrock-runtime runtime::session)` |
| `scrollbar::Set` | `ratatui_core::symbols::` | `termrock::author (ratatui_core re-export)` |
| `truncate` | `text::` | `termrock::text::text` |
| `truncate_middle` | `text::` | `termrock::text::text` |
| `width` | `text::` | `termrock::text::text` |
| `wrap` | `text::` | `termrock::text::text` |
| `wrapped_rows` | `text::` | `termrock::text::text` |

## Additions from PR #16/#17 restored on top of the target API

| Item | Origin | Destination |
|---|---|---|
| `TerminalView, TerminalSource, TerminalCell, TerminalCursor` | source-only (PR #17) | `termrock::terminal` |
| `TerminalViewState, TerminalSelection, TerminalInteraction, TerminalAction, InputToken, LinkKey` | source-only (PR #17) | `termrock::terminal` |
| `TerminalSession, DefaultTerminal, chain_panic_hook, run, run_with_feedback_clock` | source-only (PR #17) | `termrock::session` |
| `Revision` | source-only export; implemented in this integration | `termrock::core (termrock_core::id::Revision)` |
| `Item, AsItem, ItemRow, ItemRowLayout, ItemColumns` | source-only (PR #17) | `termrock::navigation::item` |
| `GridOverflowIndicator` | source-only (PR #17); restored with Count and Ellipsis behavior | `termrock::grid::grid::GridOverflowIndicator` |
| `CellWriter` | source-only (PR #17) abstraction; CellUi implements it | `termrock::text::secret::CellWriter` |
| `InheritedFormState` | target form API; canonical home is the controls crate | `termrock::controls::choice::InheritedFormState` |
| `LabelChips` | target type alias, kept | `termrock::navigation::chip::LabelChips` |
| `EnterPolicy` | target type, kept | `termrock::forms::form::EnterPolicy` |
| `Acc, SlotFn, PartStyle, cell_at, first_row, overlay_chrome, paint_pressed_bracket, shift` | target shared component helpers | `termrock::runtime root (support module)` |
| `RowFn` | both trees; canonical home is the runtime rowui module | `termrock::runtime::rowui::RowFn` |

## Facade-only losses (audited)

None. Every root export of `junie-tui` has a destination above; the facade keeps a
`termrock::author` curated module for the ratatui re-exports and the author framework.
