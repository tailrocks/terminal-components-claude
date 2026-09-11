# TASK-025 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

TASK-025 also owns the narrowly scoped shared multiline indentation primitive in text/editor.rs and text/buffer.rs, after TASK-013/016. TextEditorCore::apply remains the sole public mutation entry point; CodeEditor chooses the existing Tab policy and invokes that primitive. Indent/dedent operates only on the source-selected inclusive line interval, preserves source cursor/anchor rebasing, removes up to the configured count of leading ASCII spaces, and does not clone/rewrite the whole document. Buffer-level insertion/removal support stays non-public outside the editing core. Preserve all existing EditAction behavior, single-line guards, sensitive-buffer zeroization and non-code consumer semantics; W-025-07 covers forward/reversed/multiline/boundary selections and no-selection controls.

Freeze W-025-07/08 alongside existing source-mode and target-aware paste witnesses: range Tab/BackTab indentation, navigation versus editing movement, wheel persistence until cursor movement/resize, running/diagnostic precedence and narrow find-footer suffix/count. Reuse TextEditorCore commands and accepted viewport geometry, borrowed Highlighter/Segmenter and immutable draw; no alternate grammar or owned document clone. TASK-008 must source-dispose any exact starting test assertion that conflicts with these already oracle-owned modes while retaining compatible cache/runtime assertions. TASK-025 cannot edit protected test dispositions, preserve contradictory expected behavior, or redefine source outputs.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

Readonly pointer click moves caret and permits selection/copy without mutation. Editable completed click enters at exact pointer, keyboard focus alone does not. Integrate shared editor/output/completion primitives for highlight/diagnostics/gutter/current line, find query paste, selection and scrolling.

Keep borrowed Highlighter/Segmenter and update-owned revision caches; one text editor grammar and runtime cursor owner. No owned syntax clone, alternative test painter or app-specific editor engine.

Probe readonly first/repeated click, press-only/release-outside, remapped edit/find/completion, tabs/CJK/combining spans, diagnostic changes, wrap/scroll resize and disabled field. Measure highlight cache revision/width invalidation; cells and copied original text agree with oracle.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_025.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `keyboard_editor`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:code-editor

- family: code-editor
- reference_implementation: O:src/widgets/code.rs
- main_implementation: M:crates/tui/src/components/code.rs
- architectural_target: CodeEditorState and borrowed Highlighter Segmenter; shared editor and scroll
- visual_status: missing fade
- interaction_status: CP-02 read-only pointer caret missing
- api_refactor_status: implemented; behavioral parity incomplete
- tests_available: M:code.rs highlight/cache/find/readonly/cursor tests;keyboard_editor.rs
- tests_missing: Read-only click/selection; edit Down/Up; find query paste; diagnostics/gutter/tab/current line

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

### HIST:A87

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Editing begin/commit/cancel/blur/validation uses one shared lifecycle across text controls
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Every input/paste/key/mouse Unicode sequence and draft policy
- Gates: editing matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:88; global semantic anchor; supplemental clauses retained

### HIST:F20

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f20-find-in-read-only-output-through-existing-composition.md
- Requirement: Read-only output search through existing composed controls
- Disposition: source_Holla_find_preserved_Jackin_extension_deferred; current source_present_Holla_behavior_requires_main_restoration
- Remaining proof: Restore source Holla Files preview and Activity output query match navigation marks reveal follow and revision behavior; Files ModeFind paste edits outer filename query before preview find, Browse preview-find paste uses trimmed query; no Jackin Inspect snapshot or new cross-app controller search from Deferred F20; preserve separate existing generic CodeEditor find
- Gates: Exact Holla Files Browse-versus-Find paste recipient and Activity output query match/copy/follow/eviction journeys; shared match-to-source mapping; no invented Jackin search parity
- Origin: docs/refactoring-plan/historical-obligations.tsv:154; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-030

- Source: a1759b2a §18.2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: CompletionController keeps focus/binding ownership on editor while popup owns geometry, pointer, scroll and lifecycle.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Completion::update_for(editor_id,…) consumes reachable editor-addressed commands; no app hand-wired synthetic keys.
- Gates: Owner-separated bindings; editor focus retained; popup click/scroll/close; nested overlay and completion action traces.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:31; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-021

- Source: e8d053c9 §20.9 items9–11; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: CodeEditor cache uses edit counter and sorted-span cursor; Capsule never clones viewport per frame; grid load performs one owned conversion.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No hashing whole editor or copying ResultSet merely to draw.
- Gates: 2k-line editor <40 frame allocations; four-pane Capsule <200; grid500×12 load <8000; source-call evidence.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-023

- Source: a2ddd278; docs/audit/performance-audit.md6.3; STATE
- Requirement: Windowed incremental viewport and tree; remove full viewport clone; code edit-counter caches sorted span walks
- Disposition: accepted; current not independently tested
- Remaining proof: No buffer-size-scaled render/push or per-frame clone; semantic cache boundary retained
- Gates: 100k lines push/render; node-count-independent render; clone absence and byte limits
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
