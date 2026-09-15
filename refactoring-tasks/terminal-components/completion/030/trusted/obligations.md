# TASK-030 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

HelpOverlay renders exact grouping, columns, scrolling and dismissal from the same effective owner/action resolver as menu and hints. Wizard stable keyed enabled steps preserve back/forward/rewind/completion data/focus across dynamic eligibility.

No help-only decorative bindings, parallel keymap, domain jobs or redesigned wizard chrome. Retain caller-owned state and runtime layers; repeated draw cannot navigate or mutate completion.

Test full/narrow columns, remap/remove/hidden actions, nested modal help, disabled/vanished step, back then forward retaining draft and failed completion. Exact key/menu/help equivalence and restored focus remain visible in production frames.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_030.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `derived_hintbar_metadata`, `conformance`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:help-overlay

- family: help-overlay
- reference_implementation: O:Showcase and app help dialogs/hints
- main_implementation: M:crates/tui/src/components/help.rs
- architectural_target: HelpOverlay from same effective binding metadata as hints
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; app composition proof required
- tests_available: M:help.rs sections/bounded columns;derived_hintbar_metadata.rs
- tests_missing: Oracle help text grouping/columns/scroll narrow resize and dismissal

### COMP:wizard

- family: wizard
- reference_implementation: O:application multi-step dialog flows
- main_implementation: M:crates/tui/src/components/wizard.rs
- architectural_target: Caller-owned Wizard steps and state; no execution/domain ownership
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; app composition proof required
- tests_available: M:wizard.rs rewind/keyed enabled-step tests;conformance.rs
- tests_missing: Oracle back/forward/disabled/completion retention and focus; no redesigned step chrome

### ARCH:A21

- id: A21
- area: keymaps hints menus
- oracle_state: hand-coded hint strings synthesized keys
- main_state: BindingTableId ActionKey DerivedHintBar
- architectural_target: same metadata drives menu/action/hints; overrides owner-scoped
- status: implemented;app audit needed
- remaining_obligation: reconcile product labels/chords without decorative binding bypass
- available_gates: derived_hintbar_metadata;descriptive_hints;menu_chord_case
- missing_proof: menu-vs-key full app action equivalence
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:8396;7b27732:crates/tui/src/keymap.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A48

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §68; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Stable ActionKey identities and owner/action remaps drive routing and hints through same resolver
- Disposition: accepted; current retain_and_verify
- Remaining proof: Remap/remove/latent/hidden/duplicate chord probes
- Gates: bindings;derived hints
- Origin: docs/refactoring-plan/historical-obligations.tsv:49; global semantic anchor; supplemental clauses retained

### HIST:A49

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §68; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Explicit component Tab bindings precede traversal; raw ignored input alone reaches bubble
- Disposition: accepted; current retain_and_verify
- Remaining proof: Tab/BackTab editing and modal focused routing
- Gates: key precedence tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:50; global semantic anchor; supplemental clauses retained

### HIST:A50

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §68; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Structural keymap/table revisions invalidate hint caches; warm hints and routing allocate zero
- Disposition: accepted; current retain_and_verify
- Remaining proof: Dynamic borrowed descriptors and owner changes
- Gates: frame_hintbar_derived
- Origin: docs/refactoring-plan/historical-obligations.tsv:51; global semantic anchor; supplemental clauses retained

### HIST:A68

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §58; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Steps skipped remains read-only inspectable/navigable terminal state; stable action keys
- Disposition: accepted; current retain_and_verify
- Remaining proof: Whole-disabled differs from skipped; boundary no-op
- Gates: Steps lifecycle tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:69; global semantic anchor; supplemental clauses retained

### HIST:F08d

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f08d-unassigned-modifiers-perform-plain-actions.md
- Requirement: Explicit shortcut modifiers and hints agree with current owner
- Disposition: current_holla_done_later_apps_open; current current_main_mapping_required
- Remaining proof: Unassigned modified keys must be classified from oracle, not silently plain
- Gates: All-app binding matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:142; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-019

- Source: a1759b2a §§3.3,13.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Explicit effective Tab/ShiftTab/BackTab component bindings precede fallback ring traversal.
- Disposition: accepted; oracle product mapping controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Navigation-only Tab rule amended; Capture still runs first.
- Gates: Bound/unbound Tab and BackTab; remap/remove transitions; overlay and edit-mode focus paths match oracle.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-020

- Source: a1759b2a §13.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Binding has ActionKey and Option<Chord>; owner-scoped add/remap/remove drives routing and hints from one resolver.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: HintBar selects first nonempty priority layer; cache includes BindingTableId and keymap_revision, not only focus/state/layer.
- Gates: Removed/invisible entries absent; stable priority/insertion; structural map/table replacement invalidates; unchanged frame zero allocations.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-046

- Source: f01703a6 §17 A4,example9; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: ActionKey::application uses0x4000–0x7FFF; component custom uses0x8000–0xFFFF.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Disjoint product/component command identity; delete example corrected to application key.
- Gates: Range/disjointness tests; app source scan and effective binding tests; no product-command collision with internal action.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:47; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-043

- Source: 2e453023 §§13–15;95ab6529 §21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Bindings, visible hints and semantic actions share declarations; menu action+Chord no synthesized keys/string-label dispatch; app domain exceptions remain app-owned.
- Disposition: accepted_subject_to_pinned_app_copy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old permission to change drifted hint text is not current parity approval.
- Gates: Key/mouse action equivalence; visiblebinding conflicts once per change; exact source footer wording/order/gating.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-053

- Source: 2e453023 §12.5 J1–J5; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Library owns modal chrome/lifecycle, Form composition, choice/facts Dialog conveniences and binding-fed HelpOverlay; remove duplicate form/modal/help engines.
- Disposition: accepted_with_Field_Form_amendments; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Domain facts remain app-owned; DialogBody closedenum rejected.
- Gates: Map each J1–J5 oldcall to library composition plus exact behavior/geometry proof.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:54; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-054

- Source: 2e453023 §12.5 J6–J9; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: File browser domain stays app composition; Wizard owns steps/rewind retention; PickerChain owns generic staged loading/error/back navigation; shared keyed row rendering replaces app reimplementations.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Steps remains displayonly; filesystem/1Password models and async lifecycle do not enter library. Source owner correction: J6 is Jackin JA-015/TASK-051, with JA-017/034/059 consumed by TASK-052/053/056; Holla TASK-042 cannot prove Jackin J6.
- Gates: J6–J9 each current consumer, state retention and source error/retry/back/cancel flows.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-012

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;834aa58e;e81ca17b; docs/audit/api-audit.md:465-472;1006-1010;docs/audit/app-audit.md:295-318
- Requirement: Data-driven commands unify key handling hints and menu actions
- Disposition: accepted amended; current unverified
- Remaining proof: No key resynthesis by label; preserve component-specific commit/cancel policies
- Gates: Remap updates hints; editing suppression; keyboard/mouse same domain action
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-007

- Source: e81ca17b; docs/audit/interaction-audit.md B9
- Requirement: Bindings and hints derive from one typed command vocabulary with Capture/Bubble phases
- Disposition: accepted and amended; current not independently tested
- Remaining proof: No label dispatch/key synthesis or generic domain chords; retain product Esc ladders
- Gates: binding/handled-key equivalence; typing suppression; remap relabels hint
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-021

- Source: e81ca17b; docs/audit/domain-boundary-audit.md4.1 J1-J13
- Requirement: Reusable form/choice/info/help/wizard/picker-chain/row-decoration facilities replace duplicated app plumbing
- Disposition: accepted dispositions; exact APIs require architecture join; current not independently tested
- Remaining proof: Every J item needs explicit disposition; do not promote file-system or account semantics to library
- Gates: showcase coverage for new public facilities; wizard rewind/drafts; custom rows; domain boundary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-58-LIFECYCLE

- Source: 14bca4a3 §58; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Skipped is READ_ONLY terminal lifecycle, remains reachable/activatable; only entire rail disabled suppresses
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Skipped-as-disabled; initial false Moved; wrapping
- Gates: Seed stable key; physical-row move clamp; boundary no mutation/repaint/action; keyed click/double-click
- Origin: docs/refactoring-plan/history-late-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-68-ID

- Source: a1759b2a §68; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Stable ActionKey per Binding; optional default/latent chord; Binding::command sole lookup; Intent::Binding routes identity
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Raw-key rematching for declared commands; index/synthetic tokens
- Gates: Duplicate action diagnostics; owner/action remap/removal; raw text remains available
- Origin: docs/refactoring-plan/history-late-obligations.tsv:32; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-68-MAP

- Source: a1759b2a §68; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Structural equality KeyMap snapshot in UiCore with reused clone_from and monotonic change revision
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Fingerprint substitute; component-local map revision; global chord override
- Gates: Unchanged map no revision/allocations; changed effective map invalidates hints/routes together
- Origin: docs/refactoring-plan/history-late-obligations.tsv:33; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-68-PUBLISH

- Source: a1759b2a §68; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Focused non-forced owner publishes erased static/dynamic descriptors; latest contiguous additive table; no Any/box/unsafe
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Fresh descriptor Vec; dynamic hidden commands separate resolver
- Gates: Capture then explicit component including Tab, traversal, raw Key, ignored-only Bubble; hidden duplicates diagnosed
- Origin: docs/refactoring-plan/history-late-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG24

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Generic terminal bindings remain configurable while domain chords stay in apps; hints reflect active actual actions
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: CommandMap/HintBar; F08d
- Gates: Advertised hint reaches same action with exact modifiers
- Origin: docs/refactoring-plan/history-other-obligations.tsv:25; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
