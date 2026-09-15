# TASK-023 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

Menu reverse navigation uses one shared bounded cyclic-index operation for context menus, open MenuBar switching and closed MenuBar previous navigation. Previous from index zero in an enabled three-item collection reaches index two; unsigned wrapping modulo is not equivalent to signed Euclidean cycling. W-023-07 freezes three/five-item boundaries and disabled scanning without changing accepted forward navigation, typed action remapping or runtime layer ownership. Empty/all-disabled paths terminate without selecting or activating an unavailable item; each mode's existing source-qualified close/open response remains distinct.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

EARLY-AMEND-033 / W-023-06 fixes the reference target: a no-prompt Dialog targets its first enabled action, including a leading-disabled-action fixture; a prompt Dialog targets its actual input for focus, with editing supplied by real input state. Central Ui::reference injects only the four runtime-owned flags into that exact child/part and makes the whole callback subtree inert. Root chrome and sibling actions receive no injected flags. Assert actual changed/unchanged cells and absent registration output, not only target IDs. This task produces the concrete fixture proof consumed later by TASK-031; it does not wait for TASK-031 or add a local state-broadcast mechanism.

Use content-measured open Dialog body for prompt/confirm/info/error/destructive acknowledgement and facts; current draft controls eligibility. Menu/ContextMenu/MenuBar retain title/dropdown/submenu bounds, enabled Move cursor changes, chord rendering, typed actions, dynamic bindings and bounded reverse wrap for three/five-item and disabled-menu boundaries in W-023-07.

Compose established runtime layers and Form; no duplicate trap, backdrop, input dispatch or domain execution. Dialog body returns bare R and runs once at empty clip; callbacks cannot escape. Menu action and key route resolve the same effective owner/action metadata.

Probe nested Esc/outside/secondary/paste, popup over editor, disabled menu item and sibling submenu transitions; non-Move phases never masquerade as hover. Resize and close/reopen restore correct focus, not vanished opener. Actual facts/draft mutation must change next production frame and response.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_023.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `dialog_ack_keyboard`, `dialog_acknowledgement`, `dialog_navigation`, `menu_chord_case`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:dialog

- family: dialog
- reference_implementation: O:src/widgets/dialog.rs
- main_implementation: M:crates/tui/src/components/dialog.rs
- architectural_target: Content-measured Dialog body slots runtime layers caller form values
- visual_status: unverified full decision facts and dimensions
- interaction_status: unverified action eligibility and dismissal
- api_refactor_status: implemented; verify
- tests_available: M:dialog_ack_keyboard.rs;dialog_acknowledgement.rs;dialog_navigation.rs;dialog.rs units
- tests_missing: Prompt/confirm/error/info/destructive ack; exact current draft; outside Esc stacking focus

### COMP:menu-context-menubar

- family: menu-context-menubar
- reference_implementation: O:src/widgets/menu.rs
- main_implementation: M:crates/tui/src/components/menu.rs
- architectural_target: Typed MenuItem actions submenus ContextMenu and MenuBar layers
- visual_status: unverified
- interaction_status: unverified disabled/hover/keyboard chords
- api_refactor_status: implemented; verify
- tests_available: M:menu_chord_case.rs;menu.rs keyboard/closing/dynamic binding tests
- tests_missing: Oracle menu titles dropdown bounds submenu close/outside click paste priority

### ARCH:A10

- id: A10
- area: layers
- oracle_state: app backdrop loops popup placement
- main_state: LayerSpec Anchor Dismiss compositor
- architectural_target: single runtime layer stack/anchor resolver; open generic Dialog content
- status: implemented;parity unproven
- remaining_obligation: prove exact dimming/overlay capture/dismissal/restoration across app flows
- available_gates: overlay;dialog_navigation;dialog_acknowledgement
- missing_proof: all stacked oracle frames plus action outcomes
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:664;7b27732:crates/tui/src/layer.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A25

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §55; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Panel/Dialog body returns bare R; only Ui::layer returns Option<R>
- Disposition: accepted; current retain_and_verify
- Remaining proof: Public signatures and one body invocation
- Gates: signature gate;external examples
- Origin: docs/refactoring-plan/historical-obligations.tsv:26; global semantic anchor; supplemental clauses retained

### HIST:A72

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §67; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Menu hover changes cursor only on enabled Move; other pointer phases do not masquerade as hover
- Disposition: accepted; current retain_and_verify
- Remaining proof: Phase-by-phase cursor/action counts
- Gates: menu pointer tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:73; global semantic anchor; supplemental clauses retained

### HIST:F01

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f01-modal-first-paste-routing.md
- Requirement: Modal owner receives/consumes paste before hidden editors
- Disposition: deferred_not_selected_oracle_preserved; current historical_proposal_not_current_product_gate
- Remaining proof: Deferred proposal, not selected by immutable-oracle parity. Preserve oracle02f5294b TablePro app.rs237-254 Picker paste fallthrough; do not impose exhaustive modal-first ownership or apply this TablePro proposal to Holla. Any later change requires explicit authority
- Gates: Source-qualified disposition against exact current oracle; no proposed-behavior PASS claim
- Origin: docs/refactoring-plan/historical-obligations.tsv:132; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-014

- Source: 3adb6efe §§5,17 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Dialog body-slot draw returns R; one-slot containers use one body closure; SplitPane is the sole two-rectangle body exception.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Dialog Option<R> sketch corrected to R; full §56 contract belongs to late ledger.
- Gates: Public compile examples inspect exact closure return and both logical SplitPane rects; shared self and state preserved.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-035

- Source: 587c53bd §26; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime LayerSize Fill/Fixed and anchors own geometry; zeroFixed empty; Dialog's pure props+tokens measure equals draw; resize/reanchor applies sameframe.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Reject min_size sentinel, recentering in Dialog and runtime-held borrowed props callback.
- Gates: All anchors edges/narrow/zero; prompt height none0/field/+1ack; theme-sensitive measure; update/draw equality.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-053

- Source: 2e453023 §12.5 J1–J5; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Library owns modal chrome/lifecycle, Form composition, choice/facts Dialog conveniences and binding-fed HelpOverlay; remove duplicate form/modal/help engines.
- Disposition: accepted_with_Field_Form_amendments; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Domain facts remain app-owned; DialogBody closedenum rejected.
- Gates: Map each J1–J5 oldcall to library composition plus exact behavior/geometry proof.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:54; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-010

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:353-429;634-655;723-745
- Requirement: Completion picker select menus compose overlays/collections; dialog body open
- Disposition: accepted amended; current unverified
- Remaining proof: Functional scrolling and typed actions; retain nontrapping Select final policy
- Gates: Long option list; scrollbar drag; owner focus; nested Esc and click-outside
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-012

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.1
- Requirement: Open composed Dialog body on runtime layer; convenience constructors use same primitive
- Disposition: accepted; closed DialogBody rejected; current not independently tested
- Remaining proof: No body-kind switch or parallel convenience renderer; scroll/nested picker/ack state work
- Gates: convenience/composed frame equality; nested forms; typed ack before activation
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-017

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.2-2.3
- Requirement: Menus use typed action keys/chords; picker uses keyed borrowed rows filtered behavior and layer composition
- Disposition: accepted target; exact scope/controller APIs need later authority join; current not independently tested
- Remaining proof: Preserve submenu scope/back/loading/error/disabled/secondary product paths without string payloads
- Gates: key/mouse actions; async retry/back; no parallel index vectors
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-061

- Source: bb92a657; docs/reviews/adjudication-p-prototype-decisions.md P4
- Requirement: Dialog measured height includes prompt field row and acknowledgement echo row; default body_rows differs by constructor
- Disposition: accepted amendment toN formula; current not independently tested
- Remaining proof: No caller-owned field-height arithmetic; title row remains reserved even empty; same wrapped_rows algorithm
- Gates: Prompt/ack both themes; measured content terms equal drawn field height; oracle exact geometry
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:62; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-MOVE

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime delivers Phase::Move; enabled menu Move changes cursor; Click activates
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Press/Release/Drag masquerading as hover
- Gates: Exactly one uncaptured Move; capture produces Drag; no focus/activation; paint only visible hover change
- Origin: docs/refactoring-plan/history-late-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM69

- Source: 3adb6efe §55; COMPONENT_ARCHITECTURE.md
- Requirement: Panel/Dialog return bare R and invoke body exactly once even zero/tiny/collapsed; empty rect anchored inside request under container surface and empty clip
- Disposition: accepted; current retain_verify
- Remaining proof: Malicious closure cannot paint/register outside empty clip; Ui::layer alone optional; SplitPane separate contract
- Gates: typed sentinel; call count; nonzero-origin degenerate rect; no escaped paint/hit; AST return/signature gate
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:70; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG26

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Arbitrary composed dialog body plus ergonomic confirmation/prompt/facts conveniences use same primitives
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Dialog/Form/Layer
- Gates: Custom borrowed body and nested convenience composition tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:27; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
