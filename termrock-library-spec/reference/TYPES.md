# Shared type and model dictionary

These are proposed data shapes and semantics. Concrete implementation fields remain private unless the shape is explicitly a borrowed input record. This document resolves the names used by the component references; it is not generated Rust documentation or a claim that the APIs compile today.

## Core values

| Type | Contract |
|---|---|
| Id / ItemKey / ColumnKey / FieldKey / ActionKey | Distinct semantic identities; see F01. ActionKey is not a row index. |
| Revision | Caller-supplied accepted-source generation; monotonically advances for a source lifetime; overflow is checked. |
| Part / PartRef | Family-scoped standard/custom part name and optional item identity; e.g. LABEL, GUTTER, FILL. An unsupported part override is diagnosed. |
| Rect / Size / Constraints | Terminal cell rectangle, extent and min/max bounds; saturating/checked allocation, including nonzero origins. |
| Cx / Ui / MeasureCx | Update intents/runtime requests, constrained paint/layout publication, and immutable measurement environment respectively. |
| Response<A> | Owner Id, Flow, Invalidate, VisualState and at most one typed action. Metadata is recorded once by a shared response helper. |
| Flow / Invalidate | Bubble or Consumed; None, Paint or Layout. These axes are orthogonal. |
| VisualState | Runtime/props-derived flags: focused, focus-visible, hovered, pressed, activation-feedback, selected, checked, disabled, editing, invalid, busy/loading. Not all combinations apply. |
| ActivationOrigin / Activated / ValueChanged<T> | Keyboard/Pointer/Programmatic origin; marker action; controlled next-value request. |
| Input / Intent / InputToken | Normalized event; owner-resolved semantic input; opaque link to original bytes for host forwarding when needed. |
| Chord / Binding / BindingView / ScopeId | Typed key sequence, action metadata and effective scoped/remapped binding view. No string-to-key roundtrip. |
| UpdateCause | Boot, Input(Input), Tick, ModelChanged or Resize(Size); supplied Moment accompanies every cause. |
| Moment / AnimationSample / MotionPolicy | Monotonic elapsed time; explicit glyph phase or epoch+cadence sample; Full, Reduced, Paused. Drawing does not advance time. |
| FrameToken / PaintedFrame / UpdateReport | Versioned presented-frame receipt, canonical paint output plus published geometry, and dispatch invalidation/diagnostics. |
| RuntimeError / LayerError / ClockError / SessionError | Typed failures with safe descriptions; do not include secret values. Duplicate identity/stale geometry/nonmonotonic time fail closed. |

## Display and value policies

| Type | Values / meaning |
|---|---|
| ButtonVariant | Primary, Secondary, Subtle, Danger, Toggle, Quiet, Ghost. Source evidence determines each actual visual recipe. |
| ControlStatus | Ready, Busy, Loading, Error; status and disabled eligibility remain distinct. |
| Readiness<'a> | Ready; Empty { title, detail }; Loading { detail }; Partial { detail }; Error { message, retry: Option<ActionMeta> }. Partial keeps existing rows. |
| ValidationMessage / FieldError | Safe error code+display and stable field association. Arbitrary caller validators must not echo secrets. |
| EditPhase / BlurPolicy / ConflictPolicy | Navigation or Editing; Commit/Cancel/KeepDraft; default PreserveDraftAndReport for changed external revisions. |
| ConflictResolution | KeepDraft (rebase explicit caller intent) or Reload (explicitly replace draft); never implicit. |
| TextMode / Plain / SecretText | Sealed modes with associated Value=String or Secret. No external mode implementations required. |
| SecretPolicy / CopyPolicy | Explicit tail reveal/copy permission; ordinary copy, source-key request or forbidden/protected value. |
| Axis / Alignment / WrapMode | Horizontal/Vertical; Start/Center/End; None/Word/Character with explicit source mapping. |
| Surface / Role / Tone | Inherited semantic surface and named color/style roles; roles are resolved against theme and capability. |
| StylePatch / ModifierPatch / PatchSlot<T> | Semantic role overrides and modifier add/remove; Inherit/Set/Clear are distinct. |
| StyledText<'a> / StyleSpan | Borrowed logical text and style ranges; graphemes segmented before span paint. |
| Glyph / Badge<'a> | Theme-resolved glyph identity; borrowed badge text/tone, not a product object. |
| PanelKind / NavMode / StepsMode | Card/Framed; Full/Compact; Display/Navigable. |
| NavUnit / GridPresentation / DiffMode | Row/Cell navigation; Table/DataGrid visual recipe; Unified/Review requested diff presentation. |
| SelectionMode / SelectionRequest | None/Single/Multiple; keyed target+anchor+replace/toggle/range intent with source revision. |
| BranchActivation | Explicit leaf-only versus branch-activate policy; disclosure remains a distinct part. |
| LabelWidth / ColumnFit | Fixed or bounded auto-fit label/column budgets; fitting may sample only a declared bounded number of rows. |
| Fraction / Percent | Finite clamped0..1 ratio with checked construction; validated0..100 percent. None represents unknown, not zero. |
| ProgressValue / ProgressStatus | Determinate(Fraction) or Indeterminate; Active/Done/Error/Paused. |
| MeterVisual / MeterTone / MeterLevel | Line/Block; Normal, Level, Warning, Exhausted, Stale, Refreshing, Error, Unknown; Low/Medium/High. |
| SpinnerStatus | Running, Stopped or Failed; exact glyph for nonbaseline additions is separately approved. |
| DialogTone / DismissPolicy | Info/Confirm/Danger/Error; explicit Escape/outside policy. Product risk classification stays outside the type. |
| Anchor / LayerSize / Backdrop | Screen/rect/owner-part anchor, constrained measured size, and baseline dim/opaque/none composition policy. |
| Extent / ProtectedRange / FadePolicy / ScrollbarPolicy | Logical content length; source/visible rows protected from fades; baseline/none fade; overflow/always/never bars. |
| SplitRatio / SplitAreas | Validated preferred proportion, and two current body rectangles+seam; geometry never lives in durable state. |
| TabBehavior | Insert indentation, move focus or completion-accept policy; choose the documented context rather than globally consuming Tab. |

## Borrowed row records

Each interactive row record implements Keyed. Noninteractive headings/separators do not become fake selectable rows. Fields below describe the contract, not permission for uncontrolled internal mutation.

| Record | Required information |
|---|---|
| ChoiceItem<'a> | key, label, enabled, optional detail |
| ChipItem<'a> | key, label, checked, enabled, closable, optional tone |
| NavItem<'a> | key, section/kind, label, icon, badge, enabled |
| TreeNode<'a> | key, parent, label/payload, children keys, Leaf/Loaded/Unloaded/Loading/Error readiness |
| StepItem<'a> | key, label, detail, Queued/Running/Skipped/Blocked/Done/Failed status, optional action eligibility |
| TabItem<'a> | key, label, optional metadata/status, closable, enabled |
| PickerItem<'a> | key, label, detail, grouping, matching metadata, eligibility/explanation |
| CommandItem<'a> | stable row key, ActionKey, label, detail and effective binding metadata |
| MenuItem<'a> | Command(ActionMeta), Submenu { key, label, children }, or Separator; checked/radio markers where supplied |
| TopMenu<'a> | key, label, borrowed MenuItem slice |
| PropsRow<'a> / PropsValue<'a> | key, label, value; value is Plain/Styled/Empty/Redacted with safe display and optional safe copy identity |
| StatusGroup<'a> / StatusItem<'a> | left/center/right placement; keyed items with label/value/icon, priority, tone and optional action |
| Hint<'a> / HelpSection<'a> | effective chord, description, visibility/priority; grouped explanatory entries |
| PickerStage<'a> / WizardStep<'a> | stable stage key, title, current eligibility/readiness; content/results and state remain caller-bound |
| ActionMeta<'a> | ActionKey, label, effective Chord, enabled/visible flags, priority, optional semantic tone |
| FieldSpec<'a> | FieldKey, child Id, visible/enabled/required state, measured layout hint; values and child states live in FormControls |
| CompletionItem<'a> | key, label/kind/detail and revision-valid replacement text/range |
| GridColumn<'a> / CellValue<'a> | ColumnKey, title, width/sort/edit metadata; typed safe display value without SQL/database policy |
| DiffRow<'a> | stable line/hunk key, old/new line identity, Context/Add/Delete kind, borrowed source text and emphasis ranges |
| TextMark / TextSelection / TextRange | Stable source line identities, grapheme-safe byte positions and source revision |
| CellKey / GridSelection / CellEdit | Stable row+column identity; keyed source-revision selection; revision-valid proposed local edit |
| Diagnostic | Source range, severity and safe message; no syntax-server object |

## Model traits and extension adapters

TreeSource, GridModel, GridEditor, TextSource and DiffSource are defined in F07/F08. They borrow from caller-owned values and expose a revision. Provider clients, IO handles and a particular application's types are not part of these traits.

FormControls must support field-keyed update/commit/validate/measure/draw of real controls. The caller adapter applies typed child commits to its own model before Form validates Submit. Suggested operations:

```rust
trait FormControls {
    fn update_field(&mut self, key: FieldKey, cx: &mut Cx<'_>);
    fn commit_field(&mut self, key: FieldKey, cx: &mut Cx<'_>);
    fn validate_field(&self, key: FieldKey) -> Result<(), ValidationMessage>;
    fn measure_field(&self, key: FieldKey, cx: &MeasureCx<'_>, c: Constraints) -> Size;
    fn draw_field(&self, key: FieldKey, ui: &mut Ui<'_>, area: Rect);
}
```

Highlighter and Segmenter are borrowed pure callbacks returning revision-valid style spans or named text segments. MatchResult contains a rank and source-grapheme match ranges. They must not synchronously query external services during a draw/update call.

TerminalSource is a **new extension contract**:

```rust
trait TerminalSource {
    fn revision(&self) -> Revision;
    fn size(&self) -> Size;
    fn cell(&self, position: Position) -> Option<TerminalCell<'_>>;
    fn cursor(&self) -> Option<TerminalCursor>;
    fn history_line(&self, key: ItemKey) -> Option<TerminalLine<'_>>;
}
```

TerminalCell preserves a grapheme/symbol, width/continuation information, foreground/background and supported attributes. TerminalCursor includes position/visibility and supported shape. TerminalSelection names stable line/cell ranges and revision. TerminalInteraction chooses view selection versus host forwarding based on caller-provided mode policy. LinkKey is a stable identifier resolved and security-checked by the caller. InputToken refers to a caller-owned original event. Byte-for-byte forwarding is possible only when the host captured the raw representation before normalization; a token cannot reconstruct bytes already discarded by a decoder. The standard optional session adapter is not implicitly a lossless PTY relay. Additional protocol attributes are extensions, not silently normalized baseline evidence.

## Callback aliases

RowPainter<T>, ChoiceRowPainter, ChipRowPainter, NavRowPainter, TreeRowPainter, StepRowPainter, PickerRowPainter, TabPartPainter and GridCellPainter are borrowed Fn callbacks over typed row/cell data, immutable visual state, reserved Rect and constrained RowUi/CellUi/PartUi. SlotPainter replaces a declared part, not the entire widget. None receives a mutable domain model, global Buffer, runtime registry or an executor.

## Public state access

List/Tree/Nav/Radio/Chip/Tabs/Steps/Props expose read-only cursor and scroll observations and explicit keyed navigation commands. Picker/FilterList/PickerChain additionally expose safe query/stage observations. Plain text state exposes phase/caret/selection/draft; secret-specialized state exposes only safe phase/caret/selection and redacted status. Grid exposes CellKey/selection/edit-phase. Viewport/Diff/Code expose source-keyed reading/selection; SplitPane exposes preferred ratio/zoom. Constructors/defaults enforce valid initial state; no public field mutation bypasses invariants.
