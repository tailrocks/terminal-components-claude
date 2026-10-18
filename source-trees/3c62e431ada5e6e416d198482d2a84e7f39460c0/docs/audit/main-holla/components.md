# Component/theme contract audit

Read-only source audit. Main base c12cad8728755cd2d03eefdd8e02891143fca86d; Holla reference 794b095c196562d38f1b6f7ce379c128af2a023d. No candidate source changed. No builds, tests, visual inspection or live interactions executed by this audit. JSON inventory records source-discovered test names, not executed test coverage. Full historical obligation authority remains the history worker's responsibility; the user attachment supersedes holla-first/tree-replacement and historical model restrictions.

## Source-proven shared gaps

C1 — Inert collection EMPTY customization. `crates/tui/src/components/list.rs:864`, `filter_list.rs:787`, `select.rs:1037`, `grid.rs:2531`, `tree.rs:1603` resolve the owning family's EMPTY part, discard the result, then call `EmptyState::draw`. `collection/empty.rs:127` independently resolves EMPTY/TITLE, HELP and ICON. Owner-specific instance/part/state patches therefore do not reach painted empty content. This is the explicit meaningless-style-call bug class: instrumentation sees a styled part although no cell consumes it. Shared repair: provide a readiness painter accepting resolved owning-part styling and clearly specified child-part precedence; all five callers consume that painter. Preserve EmptyState's own semantic child styling and surface inheritance under a documented precedence. Tests must set unmistakable fg/bg/modifier sentinels on each owning EMPTY part for Empty/Loading/Partial/Error, assert actual cells change, and assert adjacent instances do not. Mutate away consumption to demonstrate gate failure. Existing Button-focused overrides tests cannot prove this contract.

C2 — Backend-free dependency graph incomplete. `crates/tui/Cargo.toml:22` declares `crossterm = []`; `:28` unconditionally depends on ratatui-crossterm. Root `Cargo.toml:30` enables that dependency's default backend features. `lib.rs:51` gates session exports, which is useful but does not remove backend dependency. Repair: optional backend dependency activated only by crossterm feature, then isolated downstream no-default-features cargo metadata/tree/check, outside workspace feature unification. Verify no backend package in resolved compilation graph; preserve normal feature build. This finding is source-proven dependency inclusion; cross-target failure has not been reproduced.

C3 — Public custom-author example perpetuates positional durable state. `examples/12_author_component.rs:57` stores cursor/selected as usize; documentation at :29 defines ItemKey::index(i). This example teaches a dynamic identity failure unless labels are explicitly immutable. `collection/key.rs:23` offers ByIndex as default and documents instability. Generic keyed collections already have key callbacks. Repair: keyed borrowed items in public author example and durable ItemKey state, or enforce immutable positional scope in a distinct static-only API; user contract requires no durable logical selection keyed only by display position. Test insert/reorder/remove with duplicate labels and stable keys through the public example. Audit app call sites separately; this is not a claim that every ByIndex call is wrong.

C4 — Grid lacks unrestricted custom cell painter. `GridModel` at `grid.rs:287` supplies borrowed CellRef, row/cell decoration, readiness and actions; no RowFn/CellUi callback in Grid. `draw_row` at :2236 owns painting. Lists, tabs, choices, navigation, trees, steps, selects, chips expose true Fn renderers. The user explicitly requires custom row/cell renderer proof. Existing decoration capability is useful and must remain, but does not demonstrate arbitrary cell content authoring. Shared repair if accepted by consolidated historical contract: borrowed Fn hook with a constrained CellUi, stable row+column identities, authoritative cell rect, and existing semantic flags; preserve default painter. Verify callback changes content/style only inside assigned cell, borrowing non-static model data, no per-cell allocation, no update-side draw callbacks, and sorting/reordering safety. Treat final interface as dependent on historical adjudication, not authorized replacement yet.

C5 — Pinned Holla DESIGN changes must forward-port. Exact diff has 15 added/2 removed lines: status warning `▲` and bold error `!` (:371); failed `✗` and unavailable/blocked `⚠` vocabulary (:573); tonal entity strip exception (:806); dialog content-driven width preserving complete decision facts (:1082). Main DESIGN lacks these additions. Reuse shared status semantics; generic Tabs needs either representable per-item tone/width geometry or public-author entity strip, not forbidden compatibility overpaint. Capture exact reference warning glyph widths before deciding glyph-role changes. All color modes need semantic glyph/modifier assertions, not RGB-only snapshots.

## Existing architecture to preserve and prove

`theme/resolve.rs:1` specifies family base -> variant -> specificity-merged state rules -> theme override -> outer-to-inner scopes -> instance patch -> semantic role binding against surface/capability. `accumulate` implements family/variant/state ordering and inserts mono fallback before author overrides. `author.rs` PartStyle centralizes borrowed global/part patches and replacement slots; `PartStyle::style` resolves actual override and registers testing facts. Ui author API exports identity, capture, registration, layers, text, themes and response plumbing without private module access. `examples/12_author_component.rs` is a real separately compiled example target, but compilation is not yet evidence from this audit.

Collections borrow slices or model access; List/Select/RadioGroup/ChipBar/Tabs/NavList/Tree/Steps expose Fn row hooks with stable-key callback. Tree has visible-only renderer tests. Grid splits read-only GridModel from mutating GridEditor; preserving that boundary avoids SQL/domain leakage. Interior mutability remains possible under shared Rust references: draw-purity requires adversarial effect/state-invariance tests, not documentation declaring mutation unrepresentable.

`tests/overrides.rs` covers global family, global variant, subtree, instance and part slot, chiefly Button instances. It proves narrower scope than every advertised part across every family. `tests/rowui_glyph_contract.rs` tests marker glyph/spacing. `tests/grid_model_contract.rs` tests three-method read-only model and shared update. Every named test must be enumerated and actually run by gate owner. Matrix: Junie and Paper independently; truecolor, ANSI256, ANSI16, mono; base/focused/focus-visible/selected/current/hovered/pressed/disabled/busy/editing/error/inactive plus meaningful combinations. Sentinel overrides must hit actual painted cells. Verify each slot preserves registrations and geometry.

## Family inventory and migration destinations

Reference `src/widgets/table.rs` and `grid.rs` map to generic Grid plus app-side adapters, `segments.rs` maps to public-author segmented control, `scrollbar.rs` to ScrollRegion, `splitter.rs` to SplitPane, `statusbar.rs` to StatusBar, `chips.rs` to ChipBar. Other matching names map to modern component modules. Reference field_common behavior maps shared field/text contracts. Holla app-only entity strips/rings/activity/control surfaces must be inventoried by Holla worker and composed through public author API; they are not silently absent because the generic module inventory ends here.

| Module | Public family/types | Borrowed slice found | Row hook | Slot hook |
|---|---|---|---|---|
| brand | Brand | True | False | True |
| button | ButtonCmd, Button | True | False | True |
| chip | ChipBarAction, ChipBarCmd, LabelChips, ChipBarState, ChipBar | True | True | True |
| choice | RadioGroupAction, ChoiceCmd, Checkbox, Toggle, LabelRadio, RadioGroupState, RadioGroup | True | True | True |
| code | Highlighter, Segmenter, CodeSeverity, TabBehavior, CodeDiagnostic, CodeAction, CodeCmd, CodeEditorState, CodeEditor | True | False | True |
| completion | CompletionAction, CompletionCmd, CompletionState, CompletionController, Completion | True | True | True |
| dialog | DialogAction, DialogCmd, DialogState, Dialog | True | False | False |
| diff | DiffLineKind, DiffRow, DiffSource, DiffMode, DiffViewState, DiffView | True | False | True |
| empty | Empty | True | False | True |
| field | Field | True | False | False |
| filter_list | FilterListAction, FilterListCmd, FilterListState, FilterList | True | True | True |
| form | FieldSpan, GroupKey, FieldKind, FieldSpec, FieldMut, FieldRef, FormData, EnterPolicy, FormAction, FormState, Form | True | False | False |
| grid | ColumnKey, NavUnit, CellRef, CellAction, EditIntent, Column, SortDir, GridModel, GridEditor, GridAction, GridCmd, GridState, Grid | True | False | True |
| help | HelpSection, HelpOverlayState, HelpAction, HelpCmd, HelpOverlay | True | False | True |
| hintbar | HintBar, DerivedHintBar | True | False | True |
| input | BlurPolicy, EditPhase, TextAction, TextCmd, TextInputState, TextInput | True | False | True |
| keyhint | KeyHint | True | False | True |
| list | ListAction, ListCmd, ListState, List | True | True | True |
| menu | MenuItem, Menu, MenuState, MenuAction, MenuCmd, ContextMenu, MenuBar | True | False | True |
| meter | MeterTone, MeterVisual, Meter | True | False | True |
| nav_list | BadgeFn, NavMode, NavListAction, NavListCmd, NavListState, NavList | True | True | True |
| panel | PanelKind, Panel | True | False | True |
| picker | Item, AsItem, ItemRow, ScopeKey, PickerAction, PickerState, Picker, CommandPalette | True | True | True |
| picker_chain | PickerStage, PickerChainAction, PickerChainCmd, PickerChainState, PickerChain | True | False | True |
| progress | ProgressBar, Spinner | True | False | True |
| props | PropsValue, PropsRow, PropsAction, PropsCmd, PropsState, PropsList, Props | True | False | True |
| scroll_region | ScrollRegion | True | False | True |
| select | SelectAction, SelectCmd, LabelSelect, SelectState, Select | True | True | True |
| split | SplitAction, SplitCmd, SplitPaneState, SplitPane | True | False | True |
| status | Group, Emphasis, StatusItem, StatusAction, StatusBar | True | False | True |
| steps | StepState, StepsAction, StepsCmd, StepsState, Steps | True | True | True |
| tabs | TabsAction, TabsCmd, TabsState, Tabs | True | True | True |
| textarea | TextAreaState, TextArea | True | False | True |
| too_small | TooSmall | True | False | True |
| tree | NodeKind, TreeNode, TreeAction, TreeCmd, TreeState, Tree | True | True | True |
| viewport | ViewportWorkProbe, ViewportWorkSnapshot, ViewportLine, CellPos, ViewportAction, ViewportCmd, ViewportState, TextViewport | True | False | True |
| wizard | WizardStep, WizardAction, WizardCmd, WizardState, Wizard | True | False | True |

The booleans are syntactic inventory signals only: false does not mean incapable (e.g. Grid borrows its model in draw/update), true does not prove behavior. JSON includes 37 modules and 406 source-discovered unit-test names, explicitly not an execution count.

## Acceptance slices

1. C1 failing actual-cell sentinel cases -> shared readiness styling contract -> all five callers -> independent all-color review.
2. C2 dependency isolation -> external public author/custom-theme examples -> default/all/isolated feature and MSRV verification.
3. C3 keyed example + caller identity audit -> insert/delete/sort/filter/reorder tests, with duplicate labels.
4. Resolve C4 with historical obligation map before interface edits -> external custom cell painter proof and hot-path allocation measurement.
5. C5 DESIGN forward-port + reusable status/entity-dialog capabilities -> pinned reference actual-cell and live-flow comparisons. Do not bless candidate snapshots as reference.
6. Every family: active and inactive states, clipping at zero/one-cell extents, disabled pointer barrier, themed parts, borrowing, draw purity, actual runtime actions, exact reference geometry where family existed. App-specific scenarios remain separate obligations rather than substituting component tests.
