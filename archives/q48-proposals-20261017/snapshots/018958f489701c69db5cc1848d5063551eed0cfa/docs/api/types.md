# Termrock shared types and models

This is the canonical type dictionary for the target public API. It defines semantics and ownership; it is not generated Rust documentation and does not claim that these types exist in the current source. Exact spelling may change during P1 qualification, but each type must keep the contract below.

Identity types are owned by the [identity foundation](../foundations/identity.md). Input/event types are owned by [input and actions](../foundations/input-actions.md). Runtime frame, focus, capture, and time types are owned by [runtime](../foundations/runtime.md). Geometry and surfaces are owned by [layout](../foundations/layout.md), while overlay ownership is defined by [layers](../foundations/layers.md). Theme roles and patches are owned by [theme](../foundations/theme.md). This page records how those types cross component boundaries.

## Core values

| Type | Contract |
| --- | --- |
| Id | Stable control identity. Namespaces and child identities use length-delimited composition; duplicate live IDs fail closed. |
| ItemKey | Stable identity for a dynamic item/row. It survives reorder, filtering, rename, and refresh. It is never an implicit display index. |
| ColumnKey | Stable identity for a Grid column. It is distinct from ItemKey and column position. |
| FieldKey | Stable identity for a Form field. It is distinct from child Id and display label. |
| ActionKey | Stable semantic command identity. It is not a row index, label, or key chord. |
| Revision | Caller-supplied accepted-source generation. It advances monotonically for a source lifetime; stale edits/ranges fail closed and overflow is checked. |
| Part / PartRef | Family-scoped standard/custom part name plus optional item identity. Unsupported overrides are diagnosed. |
| Rect / Size / Constraints | Terminal-cell geometry, extent, and min/max bounds. Helpers handle zero size, nonzero origins, Unicode widths, and exhausted space safely. |
| Position | Cell or source position with explicit coordinate space. Screen coordinates are never confused with source grapheme offsets. |
| Cx | Update context for normalized intents and constrained runtime requests. It does not expose mutable runtime registries or application effects. |
| Ui | Draw context for clipped paint, geometry publication, cursor intent, layers, and constrained part/row/cell painting. It does not expose semantic mutation. |
| MeasureCx | Immutable measurement environment containing constraints, theme metrics, capability, and relevant time-independent policy. |
| Theme / ColorLevel / ColorTokens | Semantic recipes and capability selection for TrueColor, Ansi256, Ansi16, and Mono. Theme::termrock is the baseline-preserving default; Paper is an extension sentinel. |
| Response<A> | See the canonical target shape below: `id` is the semantic owner identity and `state` is derived `VisualState`. |
| Flow / Invalidate | Independent axes: Bubble/Consumed; None/Paint/Layout. |
| VisualState | Derived flags such as focused, focus-visible, hovered, pressed, activation-feedback, selected, checked, disabled, editing, invalid, busy, and loading. Inapplicable combinations are not fabricated. |
| ActivationOrigin / Activated / ValueChanged<T> | Keyboard, pointer, or programmatic origin; activation marker; controlled next-value request. |
| Input / Intent / InputToken | Normalized terminal event, owner-resolved semantic intent, and opaque caller token for raw forwarding when the host captured the original bytes. |
| Chord / Binding / BindingView / ScopeId | Typed key sequence, command metadata, effective scoped/remapped binding, and input scope. String round trips are not the source of truth. |
| UpdateCause | Boot, Input(Input), Tick, ModelChanged, or Resize(Size). Each cause carries a supplied Moment. |
| Moment / AnimationSample / MotionPolicy | Monotonic elapsed time, explicit animation epoch/phase, and Full, Reduced, or Paused motion. Draw never advances time. |
| FrameToken / PaintedFrame / UpdateReport | Presented-frame receipt, canonical paint plus published geometry/cursor facts, and dispatch invalidation/diagnostics. |
| RuntimeError / LayerError / ClockError / SessionError | Typed safe failures. Descriptions never contain secret values. Duplicate identity, stale geometry, and nonmonotonic time fail closed. |

### Typed response

This is the one canonical target shape for a semantic update result. The field names are part of the target contract; concrete fields may remain private and be exposed through narrow typed observations.

```rust
struct Response<A> {
    id: Id,
    flow: Flow,
    invalidate: Invalidate,
    state: VisualState,
    action: Option<A>,
}
```

`id` identifies the semantic owner of this response; it is not a second owner field. `state` is a derived visual-state observation, not durable component state. `action` carries at most one caller-facing typed action. The shared runtime response helper records metadata once.

## Display and value policies

| Type | Values and meaning |
| --- | --- |
| ButtonVariant | Primary, Secondary, Subtle, Danger, Toggle, Quiet, Ghost; source recipes determine exact visual metrics. |
| ControlStatus | Ready, Busy, Loading, or Error. Status and disabled eligibility remain separate. |
| Readiness<'a> | Ready, Empty { title, detail }, Loading { detail }, Partial { detail }, or Error { message, retry }; partial retains current rows. |
| ValidationMessage / FieldError | Safe error code/display and stable field association. Validators must not echo secrets. |
| EditPhase / BlurPolicy / ConflictPolicy | Navigation or editing; commit/cancel/keep-draft behavior; default conflict policy preserves draft and reports. |
| ConflictResolution | Explicit KeepDraft or Reload. Neither source nor draft is silently discarded. |
| TextMode / Plain / SecretText | Sealed mode markers whose value is String or Secret. No external mode implementations are required. |
| SecretPolicy / CopyPolicy | Explicit tail reveal/copy permission; ordinary copy, source-key request, or protected/forbidden value. |
| Axis / Alignment / WrapMode | Horizontal/vertical, start/center/end, and none/word/character with explicit source mapping. |
| Surface / Role / Tone | Inherited semantic surface and theme role/tone. RGB literals do not identify a surface. |
| StylePatch / ModifierPatch / PatchSlot<T> | Semantic role overrides and modifier add/remove; Inherit, Set, and Clear are distinct. |
| StyledText<'a> / StyleSpan | Borrowed logical text and style ranges. Segment graphemes before applying spans. |
| Glyph / Badge<'a> | Theme-resolved glyph identity and borrowed badge text/tone. |
| PanelKind / NavMode / StepsMode | Card/framed; full/compact; display/navigable. |
| NavUnit / GridPresentation / DiffMode | Row/cell navigation; table/data-grid recipe; unified/review diff presentation. |
| SelectionMode / SelectionRequest | None/single/multiple and keyed target/anchor/replace/toggle/range intent carrying revision. |
| BranchActivation | Explicit leaf-only or branch-activate policy; disclosure remains a separate part. |
| LabelWidth / ColumnFit | Fixed or bounded auto-fit budgets. Fitting samples only declared bounded rows. |
| Fraction / Percent | Checked finite clamped 0..1 ratio and validated 0..100 percent. None means unknown, not zero. |
| ProgressValue / ProgressStatus | Determinate fraction or indeterminate; active/done/error/paused. |
| MeterVisual / MeterTone / MeterLevel | Line/block; normal, level, warning, exhausted, stale, refreshing, error, unknown; low/medium/high. |
| SpinnerStatus | Running, stopped, or failed. Any nonbaseline glyph requires extension approval. |
| DialogTone / DismissPolicy | Info/confirm/danger/error and explicit Escape/outside policy. Product risk stays outside the type. |
| Anchor / LayerSize / Backdrop | Screen/rect/owner-part anchor, constrained measured size, and baseline dim/opaque/none composition. |
| Extent / ProtectedRange / FadePolicy / ScrollbarPolicy | Logical content length, source/visible rows protected from fades, baseline/none fade, and overflow/always/never bars. |
| SplitRatio / SplitAreas | Validated preferred proportion and current body rectangles/seam. Geometry never lives in durable state. |
| TabBehavior | Insert indentation, move focus, or accept completion according to the documented context; never consume Tab globally. |

## Secret and validation types

Secret is an owned value with controlled exposure and clearing. Secret-bearing state is not generally Clone, ordinary Debug, or serializable, and it has no plain draft getter. SecretPolicy defaults to no copy and no tail reveal; both require explicit caller policy. ValidationMessage and FieldError carry safe display text and stable field identity. Validator is deterministic caller code and must never echo secret input. See the [secret-aware foundation](../foundations/secret-validation.md) for zeroization and negative-proof requirements.

## Borrowed row and item records

Every interactive row record implements Keyed. Headings and separators remain noninteractive records. The fields below are the minimum caller-provided data; components may borrow richer records through documented adapters.

| Record | Required information |
| --- | --- |
| ChoiceItem<'a> | Key, label, enabled flag, optional detail. |
| ChipItem<'a> | Key, label, checked, enabled, closable, optional tone. |
| NavItem<'a> | Key, section/kind, label, icon, badge, enabled. |
| TreeNode<'a> | Key, parent, label/payload, child keys, and leaf/loaded/unloaded/loading/error readiness. |
| StepItem<'a> | Key, label, detail, queued/running/skipped/blocked/done/failed status, optional action eligibility. |
| TabItem<'a> | Key, label, optional metadata/status, closable, enabled. |
| PickerItem<'a> | Key, label, detail, grouping, matching metadata, eligibility, and explanation. |
| CommandItem<'a> | Stable row key, ActionKey, label, detail, and effective binding metadata. |
| MenuItem<'a> | Command action, keyed submenu, or separator; checked/radio markers where supplied. |
| TopMenu<'a> | Key, label, and borrowed MenuItem slice. |
| PropsRow<'a> / PropsValue<'a> | Key, label, and plain/styled/empty/redacted value with optional safe copy identity. |
| StatusGroup<'a> / StatusItem<'a> | Left/center/right placement; keyed label/value/icon, priority, tone, and optional action. |
| Hint<'a> / HelpSection<'a> | Effective chord, description, visibility/priority, and grouped explanatory entries. |
| PickerStage<'a> / WizardStep<'a> | Stable stage key, title, eligibility/readiness; results and state remain caller-bound. |
| ActionMeta<'a> | ActionKey, label, effective Chord, enabled/visible flags, priority, optional tone. |
| FieldSpec<'a> | FieldKey, child Id, visible/enabled/required state, and measured layout hint. Values and child states live in FormControls. |
| CompletionItem<'a> | Key, label/kind/detail, and revision-valid replacement text/range. |
| GridColumn<'a> / CellValue<'a> | ColumnKey, title, width/sort/edit metadata, typed safe display value without database policy. |
| DiffRow<'a> | Stable line/hunk key, old/new line identity, context/add/delete kind, borrowed source text, emphasis ranges. |
| TextMark / TextSelection / TextRange | Stable source line identity, grapheme-safe byte positions, source revision. |
| CellKey / GridSelection / CellEdit | Stable row+column identity, keyed revisioned selection, and revision-valid proposed local edit. |
| Diagnostic | Source range, severity, and safe message; no syntax-server object or live provider. |

## Model traits and adapters

TreeSource, GridModel, GridEditor, TextSource, and DiffSource borrow caller-owned values and expose a revision. They do not contain provider clients, IO handles, or application types.

### FormControls

Form is a caller adapter, not an untyped field database. It keys child operations by FieldKey, applies child commits to the caller model, and validates current values before emitting FormAction::Submit:

    trait FormControls {
        fn update_field(&mut self, key: FieldKey, cx: &mut Cx<'_>);
        fn commit_field(&mut self, key: FieldKey, cx: &mut Cx<'_>);
        fn validate_field(&self, key: FieldKey) -> Result<(), ValidationMessage>;
        fn measure_field(
            &self,
            key: FieldKey,
            cx: &MeasureCx<'_>,
            constraints: Constraints,
        ) -> Size;
        fn draw_field(&self, key: FieldKey, ui: &mut Ui<'_>, area: Rect);
    }

### Text and pure projections

TextSource returns revision-tagged source lines. Highlighter and Segmenter are borrowed pure callbacks returning revision-valid spans or named segments. MatchResult contains a rank and source-grapheme ranges. They do not synchronously query external services during update, measure, or draw.

    trait TextSource {
        fn revision(&self) -> Revision;
        fn line_count(&self) -> usize;
        fn line(&self, index: usize) -> Option<TextLine<'_>>;
    }

    struct TextLine<'a> {
        key: ItemKey,
        text: &'a str,
        spans: &'a [StyleSpan],
    }

The shared editor core owns grapheme motion, word motion, deletion, selection, undo/history, paste, and source/display coordinates. TextInput/TextArea/CodeEditor select policy; they do not fork these mechanics. Tabs expand for display only, and copy follows source text and explicit copy policy. Double-width continuation cells and combining marks are measured using the documented Unicode policy.

## Terminal cell adapter

TerminalView is a generic prepared-cell presentation component. It consumes caller-provided terminal cells and may display styles, continuation cells, cursor, and selection while emitting typed interaction, copy, or link requests:

    trait TerminalSource {
        fn revision(&self) -> Revision;
        fn size(&self) -> Size;
        fn cell(&self, position: Position) -> Option<TerminalCell<'_>>;
        fn cursor(&self) -> Option<TerminalCursor>;
        fn history_line(&self, key: ItemKey) -> Option<TerminalLine<'_>>;
    }

TerminalCell preserves grapheme/symbol, width/continuation information, foreground/background, and supported attributes. TerminalCursor carries position, visibility, and supported shape. TerminalSelection names stable line/cell ranges with a source revision. TerminalInteraction chooses view selection or host forwarding using caller policy. LinkKey is resolved and security-checked by the caller. InputToken can forward bytes only when the host captured the original representation before normalization.

Termrock does not become a terminal emulator, PTY runtime, shell, daemon, agent-session manager, or terminal escape parser. The optional session adapter owns raw-mode/crossterm integration and cleanup at the terminal edge; core TerminalView remains a borrowed-cell consumer.

## Callback aliases

RowPainter<T>, ChoiceRowPainter, ChipRowPainter, NavRowPainter, TreeRowPainter, StepRowPainter, PickerRowPainter, TabPartPainter, and GridCellPainter are borrowed Fn callbacks over typed row/cell data, immutable visual state, a reserved rectangle, and constrained RowUi/CellUi/PartUi. SlotPainter replaces one declared part, not the whole component.

No callback receives a mutable domain model, global buffer, runtime registry, or executor. Callback invocation is immediate; callbacks do not escape borrowed lifetimes or start background tasks. The [authoring contract](authoring.md) defines clipping, patch propagation, and side-effect checks.

## Public state observations

State fields remain private. Expose narrow observations and commands instead:

- List, Tree, NavList, RadioGroup, ChipBar, Tabs, Steps, and Props expose read-only cursor/current key and scroll observations plus explicit keyed navigation.
- Picker, FilterList, and PickerChain additionally expose safe query/stage observations.
- Plain text exposes phase, caret, selection, and (where safe) draft observation; secret text exposes only safe phase/caret/selection and redacted status.
- Grid exposes CellKey, selection, and edit phase; column identity is ColumnKey.
- TextViewport, DiffView, and CodeEditor expose source-keyed reading/selection observations.
- SplitPane exposes preferred ratio/zoom; current rectangles remain runtime geometry.

Constructors and defaults enforce valid initial state. No public field mutation may bypass identity, revision, geometry, or secret invariants.
