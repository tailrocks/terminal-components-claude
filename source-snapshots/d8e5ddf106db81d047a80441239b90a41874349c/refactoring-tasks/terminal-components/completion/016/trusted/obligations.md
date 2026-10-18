# TASK-016 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Keyboard FocusIn alone remains navigation; completed click enters TextInput/TextArea editing at exact grapheme caret. Press-only then release outside does not edit. Preserve first/repeated click, Enter/typing, horizontal/multiline scroll, selection, commit/cancel/blur/validation and externally controlled value synchronization.

Field supplies chrome only and no second focus stop. Shared TextEditorCore and sealed String/Secret FieldControl own lifecycle; no duplicate app editor. Secret has no Clone/Eq/Serialize, Debug redacts, synthetic mask preserves grapheme geometry and drafts clear on close/cancel/removal.

Exercise disabled versus readonly, masked CJK/emoji/combining hits, invalid TextInput commit ending the edit and recording its validation error without an invented commit/navigation veto, hide/show, type/sensitivity transition and focus loss. Secret sentinel must not occur in logs/copy/frame/error artifacts. Preserve compile-fail containment, safe best-effort zeroization and CP-SCROLL-FADE multiline states.

## Fixed re-audit contract — configured shared paste command

Expose `TextInput::paste(&self, state: &mut TextInputState, value: &mut String, text: &str) -> Response<TextAction>`. The public method and runtime Intent::Paste branch call the same private target-generic mutation implementation, ending in TextEditorCore::apply(EditAction::Paste); the private Form Secret target stays sealed and no new public Secret projection is introduced. Document and compile-test this exact public signature. Oracle input.rs:237–243 admits enabled paste by beginning an idle edit from the controlled value before insertion, unlike idle TextArea/CodeEditor paste. Set sensitivity before beginning/copying the draft; configured readonly/disabled (including inherited disable in the private bridge) rejects paste without beginning editing. Already editing paste replaces selection through the same core. Run live validation and admit repaint even for empty paste, as oracle on_paste returns Changed; TextAction::Changed denotes actual draft insertion/replacement while entering edit alone reports changed invalidation without inventing a commit. Preserve masks, secret lifetime, limits, grapheme cursor/scroll and commit-only controlled-value writes. A plain FocusIn alone still does not begin editing. Do not synthesize focus, deliver a hidden runtime Intent, insert text directly in the app, or add another editor engine.

This command supports the source-qualified Holla F10 MenuBar compatibility dispatch (oracle holla app.rs:480 checks true modals but not the menu before the Args edit paste), including a focused idle Args field. Application task owners select the explicit current state and consume that paste once; generic runtime modal/inert routing is unchanged. Matching runtime-command/programmatic-command probes compare draft/cursor/selection/error/value/action immediately, and scroll/hardware-cursor/frame outcomes after the same actual update/layout settlement. Main input.rs:1243–1248 owns scroll_into_view using the current allocation; the no-Cx helper cannot invent or cache substitute geometry. The first frame while the menu is still open must already match, including long paste and resize: input.rs:1420–1436 purely derives the effective horizontal window from the new cursor and current inner width even when the control is inert. Use that same shared projection; do not wait for reveal, mutate state in draw, register a hidden owner or add a second scrolling engine. Stored scroll reconciles later through the ordinary eligible update, and is observed separately from the first frame's effective window. Positive cases explicitly contrast idle TextInput paste (begins edit, including empty paste) with idle TextArea/CodeEditor paste (ignored); test idle paste then Escape snapshot restoration, existing selection replacement, and prior invalid state with empty/sanitized paste still executing live validation. Negatives include readonly, disabled/inherited-disabled, secret sentinel leakage, duplicate delivery, validation bypass, and a true modal mutating the underlying editor. CodeEditor has the parallel configured command owned by TASK-025; no input package edit is delegated to an application task.

## Fixed validation disposition

Oracle input.rs:169–172/203–215 ends editing and validates, then emits Committed or CommittedTab even when invalid. Main input.rs:745–753 likewise writes the controlled value before recording validation. Preserve that component lifecycle: an invalid TextInput commit is not vetoed, and the caller handles the committed-tab navigation according to its source contract. Form submission is the separate validation gate owned by TASK-019; Grid model-denied cell commit is a different editor contract and cannot be generalized to TextInput. Test required-empty commit, invalid Tab/BackTab, FocusOut CommitAndValidate, error display, and later Form submit rejection with exact value/phase/action/focus/callback order.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_016.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `keyboard_editor`, `input_placeholder`, `typing_owner`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:field-chrome

- family: field-chrome
- reference_implementation: O:src/widgets/input.rs;textarea.rs;select.rs field chrome
- main_implementation: M:crates/tui/src/components/field.rs;field_control.rs
- architectural_target: Field chrome only; one child Id and focus stop
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:field::a_reference_field_registers_no_control;conformance.rs
- tests_missing: Field label/help/error and child pointer geometry; no second focus stop

### COMP:text-input

- family: text-input
- reference_implementation: O:src/widgets/input.rs:250
- main_implementation: M:crates/tui/src/components/input.rs:1181
- architectural_target: Caller value/draft lifecycle with explicit navigation and edit modes
- visual_status: unverified editing style
- interaction_status: CP-02 FocusIn/Press behavior differs
- api_refactor_status: implemented; behavioral migration required
- tests_available: M:input.rs lifecycle/secret tests;input_placeholder.rs;keyboard_editor.rs;O:one_click_enters_editing_at_the_pointer_and_a_disabled_field_ignores_it
- tests_missing: CP-02 keyboard focus navigation; completed click cursor; masked grapheme hit mapping

### COMP:text-area

- family: text-area
- reference_implementation: O:src/widgets/textarea.rs:177
- main_implementation: M:crates/tui/src/components/textarea.rs:885
- architectural_target: Shared TextEditorCore caller value and multiline draft lifecycle
- visual_status: missing fade; edit state unverified
- interaction_status: CP-02 FocusIn auto-edit differs
- api_refactor_status: implemented; behavioral migration required
- tests_available: M:textarea.rs lifecycle/secret/readiness units;conformance.rs
- tests_missing: CP-02; multiline cursor/selection/blur; CP-SCROLL-FADE

### COMP:secret-validation

- family: secret-validation
- reference_implementation: O:input masked fields;grid validators;dialog prompt
- main_implementation: M:crates/tui/src/secret.rs;validate.rs;components/form.rs
- architectural_target: Non-clone secrets sealed text targets zeroized drafts typed validation
- visual_status: unverified masking/error parity
- interaction_status: unverified lifecycle
- api_refactor_status: implemented; retain architecture contract
- tests_available: M:secret compile-fail fixtures;input/textarea/form secret tests
- tests_missing: Public-boundary secret dynamic transitions with exact oracle masks; no secret artifacts

### ARCH:A15

- id: A15
- area: text editor and secrets
- oracle_state: TextBuffer clones fn callbacks
- main_state: TextEditorCore TextBuffer Secret Validate FieldControl
- architectural_target: one editor action grammar; borrowed validation; secret containment
- status: implemented;behavior drift
- remaining_obligation: preserve exact legacy editing interactions through shared core
- available_gates: keyboard_editor;input_placeholder;compile_fail_cases_hold;secret unit tests
- missing_proof: oracle selection/edit/blur/unicode/readonly sequences
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1450;7b27732:crates/tui/src/text/editor.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A84

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §67; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Inherited/configured disabled combine before value borrow; TextTarget sealed for String/Secret
- Disposition: accepted; current retain_and_verify
- Remaining proof: Dynamic disable, secret editing without Clone
- Gates: form bridges
- Origin: docs/refactoring-plan/historical-obligations.tsv:85; global semantic anchor; supplemental clauses retained

### HIST:A85

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Secret no Clone/Eq/Serialize and redacted Debug/display; synthetic mask never leaks content
- Disposition: accepted; current retain_and_verify
- Remaining proof: Leak sentinels across values/errors/capture/log/cancel
- Gates: secret compile-fail;security tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:86; global semantic anchor; supplemental clauses retained

### HIST:A86

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Owner clears secrets on close/cancel with best-effort safe-Rust zeroization; no guaranteed-erasure claim
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Ownership clearing and no exposed copies
- Gates: secret lifecycle tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:87; global semantic anchor; supplemental clauses retained

### HIST:A87

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Editing begin/commit/cancel/blur/validation uses one shared lifecycle across text controls
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Every input/paste/key/mouse Unicode sequence and draft policy
- Gates: editing matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:88; global semantic anchor; supplemental clauses retained

### HIST:F08c

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f08c-ctrl-shift-home-end.md
- Requirement: Ctrl+Shift+Home/End extend selection according to editor contract
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: Modifier matrix and selection anchor integrity
- Gates: Editor modified keys
- Origin: docs/refactoring-plan/historical-obligations.tsv:141; global semantic anchor; supplemental clauses retained

### HIST:F18

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f18-distinguish-read-only-from-disabled.md
- Requirement: Read-only interaction remains distinct from disabled
- Disposition: deferred_fixture_correction_not_selected_shared_contract_retained; current historical_proposal_not_current_product_gate
- Remaining proof: Keep modern disabled versus readonly API distinction; preserve Showcase TextAreas original Read-only transcript label with disabled dim inert unfocusable configuration; no relabel replacement or new navigation from deferred F18
- Gates: Separate generic readonly navigation/copy/nonmutation proof from exact source disabled fixture frames and no-focus observations
- Origin: docs/refactoring-plan/historical-obligations.tsv:152; global semantic anchor; supplemental clauses retained

### HIST:O07

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; docs/improvements-plan-reference.md
- Requirement: Read-only field API proposal
- Disposition: conditional_not_selected; current current_main_mapping_required
- Remaining proof: Accepted architecture invariants still apply; no unsolicited new product behavior
- Gates: scope disposition
- Origin: docs/refactoring-plan/historical-obligations.tsv:194; global semantic anchor; supplemental clauses retained

### HIST:P01

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; git commits 53b8212f;e4866ce4;92d91629;02f5294b
- Requirement: One click focuses and starts text editing at pointer; Grid cells click-to-edit
- Disposition: oracle_behavior_binding; current current_main_mapping_required
- Remaining proof: 53b8212f/e4866ce4 implementation must be expressed through main runtime
- Gates: all field click flows
- Origin: docs/refactoring-plan/historical-obligations.tsv:195; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-028

- Source: e49de3f8 §§15.1,17 A9;c936d51f §15;3f26ab14; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Public FormState errors are generic; direct secret String begin requires sensitive state; expose is crate-private and masks accept SecretPolicy.
- Disposition: accepted security boundary; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Only private Form draw sees current plain detail; zeroization remains best-effort safe Rust, never guaranteed erasure.
- Gates: Public error reads before update during dynamic sensitivity change; direct sensitive begin/cancel; Debug/display/mask policy; capacity release.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-044

- Source: 15371443 §29.9; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: TextArea covered FIELD part override can be proved by its actual recorded resolution when composed paint leaves digest unchanged.
- Disposition: accepted implementation evidence; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: ebfa8b8 closes earlier false negative; patch_part hook remains future optional ordering-hardening, not mandatory unfinished gate.
- Gates: Check selected part/owner resolution and theme immutability; no unrelated sibling comparison; include current text_area override case.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:45; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-013

- Source: 95ab6529 §21 item7;27bd918e §23; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Field owns chrome/height only, remains idless; child FieldControl owns its configured ID and input/ring registration.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Supersedes original Field own-focus proposal.
- Gates: Composed Field exactly one ring entry/control ID; field chrome inheritance and child update tests.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:14; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-047

- Source: 2e453023 §15;587c53bd §25 MA13;c936d51f §15; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Secrets nonClone/nonEq/nonSerialize and redacted; sensitive direct state before begin; expose crate-private; policy-aware mask never copies real tail.
- Disposition: accepted_security_tightening; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Zeroization explicitly best-effort safeRust, not guaranteed erased allocation memory.
- Gates: Compilefails; direct lifecycle dynamic sensitivity; Debug/errors/action/output wholebuffer secret sentinel; zeroize fresh exposeempty/capacityrelease.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:48; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F13

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: FormState/field drafts redact Debug and zeroize secret drafts before discard.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A85,A86
- Gates: Secret never appears in Debug/frame; close/cancel/removal lifecycle uses best-effort safe clearing.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:14; refines A85,A86; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-002

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=834aa58e; docs/audit/api-audit.md:1073-1098
- Requirement: Focus transition lifecycle removes commit/validation/data mutation from draw
- Disposition: accepted direction; current unverified
- Remaining proof: Fix phase capabilities and transition delivery across all families
- Gates: Headless update; shared-state draw twice; focus loss exactly once
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:3; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-014

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:528-549;842-857
- Requirement: Shared editor core and field decoration; closure validators and explicit controlled sync
- Disposition: accepted amended J/K; current unverified
- Remaining proof: No correlated public state; no draw-time validation; per-phase borrowing
- Gates: Commit/cancel/blur/paste Unicode; invalid retains draft; first-error focus
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-017

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=834aa58e;568e1ef6;9df371d5; docs/audit/api-audit.md:1102-1139;docs/audit/app-audit.md:11-20
- Requirement: Secret exposure assertions distinguish transient value from persisted suffix
- Disposition: amended explicitly; current unverified
- Remaining proof: Masked input enters pending workspace only after valid key and Save; m stays masked
- Gates: No full raw secret in frames/debug/actions; deliberate final-four persisted mask; cancel lifetime
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-034

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B1/B2/B4/B8/B10;M11/M24;540-555;737-744
- Requirement: Compilable builders; independent frozen intent borrow; Field identity; command/action split; lossless Response fold
- Disposition: J accepted; later exact public API governs; current unverified
- Remaining proof: Eliminate compile and semantic-action-loss enabling conditions
- Gates: Complete external examples; compile-fail semantic fold; intent iteration plus mutable services; paste arena lifetime; no duplicate Field ID
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-041

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B13/B14;629-646;A4/A11
- Requirement: Noncolor pressed state; cache hints/hash; empty intent fast path; bounded diagnostics; no secret draw allocations
- Disposition: J accepted then container/Id/mono/cache amendments; current unverified
- Remaining proof: Reject review ownSmallVec; replace obsolete debug-Id assumptions; preserve current secret and mono contracts
- Gates: Zero unchanged-focus hint allocations; no lost cache generations; nonempty drain scale; bounded diagnostic drop count; actual pressed bracket cells; no raw secret disclosure
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-015

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.4;3.4
- Requirement: Shared TextEditorCore; explicit begin/commit/cancel/blur; borrowed trait/closure extensions
- Disposition: accepted and amended; current not independently tested
- Remaining proof: No render-time commit; preserve oracle control-specific Esc semantics via supported policy; no fn-pointer-only hooks
- Gates: focus-loss transitions; edit snapshot; paste/unicode; completion splice/anchor lifecycle
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-016

- Source: e81ca17b; docs/audit/domain-boundary-audit.md3.3; modern-api R19
- Requirement: Secret redacts Debug/Display; actions carry identities; no ratatui Masked
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Follow later accepted disclosure policy; prevent raw secrets in debug/form results/capture
- Gates: redaction corpus; compile-fail clone/eq; explicit reveal/reference paths
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-SECRET

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Sealed private TextTarget only String/Secret enables direct non-Clone editing
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Cloned secret; public secret-target abstraction
- Gates: Non-Clone secret/direct edit tests plus later security amendments owned by early historian
- Origin: docs/refactoring-plan/history-late-obligations.tsv:31; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG18

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Focus-loss, validation, commit, cancellation and synchronization are explicit lifecycle/input transitions
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Runtime dispatch and field/Grid adapters
- Gates: No-paint transition tests; repeated paint leaves state unchanged
- Origin: docs/refactoring-plan/history-other-obligations.tsv:19; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG37

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Forms/editors share explicit edit lifecycle,controlled synchronization,paste,graphemes,selection,cursor and validation extension
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-form/editor; F08
- Gates: Event/paste/modifier/Unicode/begin-commit-cancel-blur tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:38; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG38

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Secret values do not leak through Debug/log/capture/copy; removed sensitive drafts zeroize
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Form/Input/Jackin credential rows
- Gates: Leak sentinels, removed slot zeroize and Cancel/Save tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:39; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG39

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Read-only permits appropriate navigation/copy while rejecting mutation; disabled is distinct
- Disposition: accepted_current_oracle_bounded; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Ordinary readonly navigation/copy and nonmutation remain accepted; F04 mid-edit permission-transition proposal is deferred, not an override of current oracle behavior. Prove direct-component versus app reachability separately
- Gates: Ordinary readonly mutation/navigation proof; separately retain F04 direct transition oracle observations and deferred disposition
- Origin: docs/refactoring-plan/history-other-obligations.tsv:40; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
