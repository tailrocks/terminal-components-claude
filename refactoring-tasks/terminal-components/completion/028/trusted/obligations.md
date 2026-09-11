# TASK-028 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

Use one StatusBar with an additive finite layout policy and explicit gap/edge configuration, never an application drop loop or second painter. Preserve the existing modern retained-left policy and theme-derived spacing by default. The source StatusBar configuration uses gap=3, edge=1, gaps between surviving items/groups, later-item removal within equal-priority groups, center-before-right-before-left cross-group ties, and final-left truncation. The source Segments configuration uses gap=2, edge=1 and a trailing gap in the width budget for every surviving item, first-left equal-priority removal, last-right equal-priority removal, right-before-left cross-group ties, and permits dropping all left items. Source Segments has no center group; do not infer its behavior for a center group from another source. Keyed Segments' visual/hit padding is outside its unpadded width budget, including the one-cell leading shift; preserve caller background and exact source hit geometry. Reuse borrowed StatusItem projection, keyed routing, fixed buffers, slots and inline Meter. W-028-01/06 independently prove both configured source policies and unchanged modern default; source-qualified final-survivor clauses apply only to StatusBar, never universally to Segments.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

StatusBar and Segments preserve their distinct source-qualified spacing, width budgets and declaration priority/drop order under D-BRANCH-SOURCE; final-left survivor ellipsis applies only to StatusBar; only keyed hovered label lifts and blank/unkeyed/key input clears hover. HintBar action/chord priority and overflow marker; KeyHint modifier fragments, remaps and multi-key clipping match handled actions.

Retain one StatusBar for old segments/statusbar, pure draw-time caller projection with fixed buffers and no mutable mirror cache. Binding metadata and structural revisions drive actual routing and hints, warm caches allocate zero.

Probe active/inactive/modal/edit owners, remap/remove/hidden/latent/duplicate chord, long mandatory label at tiny width, offsets and live inactive state update before pure repaint. Compare glyph/foreground/background/modifiers, not just distinct hashes; no hint points to unhandled action.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_028.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `status_bar_hover`, `derived_hintbar_metadata`, `descriptive_hints`, `status_spinner`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:statusbar-segments

- family: statusbar-segments
- reference_implementation: O:src/widgets/statusbar.rs;segments.rs
- main_implementation: M:crates/tui/src/components/status.rs
- architectural_target: Single reusable StatusBar Group StatusItem; old segments absorbed
- visual_status: unverified
- interaction_status: unverified keyed click/hover
- api_refactor_status: implemented; verify
- tests_available: M:status_bar_hover.rs;status_spinner.rs;status.rs priority/truncate/group tests
- tests_missing: Oracle left/center/right drop order tonal messages exact failure/warning glyphs

### COMP:hintbar

- family: hintbar
- reference_implementation: O:src/widgets/hintbar.rs
- main_implementation: M:crates/tui/src/components/hintbar.rs
- architectural_target: HintBar and DerivedHintBar consume effective metadata
- visual_status: unverified
- interaction_status: unverified active layer hint ownership
- api_refactor_status: implemented; verify
- tests_available: M:derived_hintbar_metadata.rs;descriptive_hints.rs;hintbar.rs tests
- tests_missing: Oracle action text/chords priority width dropping focus/edit/modal context

### COMP:keyhint

- family: keyhint
- reference_implementation: O:src/widgets/keyhint.rs
- main_implementation: M:crates/tui/src/components/keyhint.rs
- architectural_target: KeyHint from typed effective Chord and semantic action parts
- visual_status: unverified
- interaction_status: displayed binding versus handled binding unverified
- api_refactor_status: implemented; verify
- tests_available: M:keyhint.rs slot/chord/no-allocation/width units
- tests_missing: Oracle modifier glyphs multi-key width fragment clipping remaps and hidden actions

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

### HIST:A50

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §68; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Structural keymap/table revisions invalidate hint caches; warm hints and routing allocate zero
- Disposition: accepted; current retain_and_verify
- Remaining proof: Dynamic borrowed descriptors and owner changes
- Gates: frame_hintbar_derived
- Origin: docs/refactoring-plan/historical-obligations.tsv:51; global semantic anchor; supplemental clauses retained

### HIST:A66

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §51; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Status hover keyed per live label; unkeyed/static/blank regions clear it
- Disposition: accepted; current retain_and_verify
- Remaining proof: Sibling styles unchanged and keyboard suppression
- Gates: status hover tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:67; global semantic anchor; supplemental clauses retained

### HIST:A99

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §54; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Status projection pure and dynamic text in app-owned fixed buffers; no duplicated mutable Jx cache
- Disposition: accepted; current retain_and_verify
- Remaining proof: Current facts/tones/priorities/action keys after inactive updates
- Gates: status tests;allocations
- Origin: docs/refactoring-plan/historical-obligations.tsv:100; global semantic anchor; supplemental clauses retained

### HIST:F08d

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f08d-unassigned-modifiers-perform-plain-actions.md
- Requirement: Explicit shortcut modifiers and hints agree with current owner
- Disposition: current_holla_done_later_apps_open; current current_main_mapping_required
- Remaining proof: Unassigned modified keys must be classified from oracle, not silently plain
- Gates: All-app binding matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:142; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-020

- Source: a1759b2a §13.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Binding has ActionKey and Option<Chord>; owner-scoped add/remap/remove drives routing and hints from one resolver.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: HintBar selects first nonempty priority layer; cache includes BindingTableId and keymap_revision, not only focus/state/layer.
- Gates: Removed/invisible entries absent; stable priority/insertion; structural map/table replacement invalidates; unchanged frame zero allocations.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-041

- Source: 82158df5 §21 item17–19; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: HintBar receives required HintLayer; optional selection belongs to HintBar::resolve/DerivedHintBar; Cx::record feeds runtime/harness records.
- Disposition: accepted API correction; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No current Screen::hints hook or AppActionRecord/Harness::actions promise.
- Gates: Public examples compile; effective hint selection and instrumentation records validated independently.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-043

- Source: 2e453023 §§13–15;95ab6529 §21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Bindings, visible hints and semantic actions share declarations; menu action+Chord no synthesized keys/string-label dispatch; app domain exceptions remain app-owned.
- Disposition: accepted_subject_to_pinned_app_copy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old permission to change drifted hint text is not current parity approval.
- Gates: Key/mouse action equivalence; visiblebinding conflicts once per change; exact source footer wording/order/gating.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-057

- Source: 70dacec1 §29.7;7b27732a §29.7 excerpt; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: RadioGroup caller value and ChipBar Add action were open in Q, later resolved§50; StatusBar hover primitive existed but lacked consumer, later resolved§51.
- Disposition: historical_open_items_superseded; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Do not revive fabricated ItemKey sentinel or claim hovered_part presence was implementation closure.
- Gates: Join middle-reader original§50–51; actual payloadless AddRequested/NEW and keyed hover consumer tests.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:58; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-012

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;834aa58e;e81ca17b; docs/audit/api-audit.md:465-472;1006-1010;docs/audit/app-audit.md:295-318
- Requirement: Data-driven commands unify key handling hints and menu actions
- Disposition: accepted amended; current unverified
- Remaining proof: No key resynthesis by label; preserve component-specific commit/cancel policies
- Gates: Remap updates hints; editing suppression; keyboard/mouse same domain action
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-015

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:515-524;553-562;675-719;764-794
- Requirement: Unify static/interactive props, priority strips and component hint metadata
- Disposition: accepted direction; current unverified
- Remaining proof: No second drop algorithm or independently painted shortcut text
- Gates: Priority drop oracle; copy/activate actions; last-row hint precedence; hover by item
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-041

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B13/B14;629-646;A4/A11
- Requirement: Noncolor pressed state; cache hints/hash; empty intent fast path; bounded diagnostics; no secret draw allocations
- Disposition: J accepted then container/Id/mono/cache amendments; current unverified
- Remaining proof: Reject review ownSmallVec; replace obsolete debug-Id assumptions; preserve current secret and mono contracts
- Gates: Zero unchanged-focus hint allocations; no lost cache generations; nonempty drain scale; bounded diagnostic drop count; actual pressed bracket cells; no raw secret disclosure
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-007

- Source: e81ca17b; docs/audit/interaction-audit.md B9
- Requirement: Bindings and hints derive from one typed command vocabulary with Capture/Bubble phases
- Disposition: accepted and amended; current not independently tested
- Remaining proof: No label dispatch/key synthesis or generic domain chords; retain product Esc ladders
- Gates: binding/handled-key equivalence; typing suppression; remap relabels hint
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-019

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.7-2.8
- Requirement: Merge StatusBar and segments priority-strip behavior; integrate component hints
- Disposition: accepted; hover later explicit; current not independently tested
- Remaining proof: Keep precise drop order strongest left survivor and individual hover; no manual duplicate priority loops
- Gates: narrow widths; left/center/right; item hit key; hover; overlay hint precedence
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-54-STATUS

- Source: ecc13378 §54.6; laneC-app-tick Q1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Pure draw-time status projection into app-owned fixed byte buffers; preserve facts/tone/priority/key
- Disposition: accepted-amended; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Proposal allowing temporary formatting allocation; mutable Jx cache; seventh Screen method; owned StatusItem strings
- Gates: status_projection_allocates_zero_per_frame; pure_repaint_never_reuses_stale_status_items
- Origin: docs/refactoring-plan/history-late-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-68-HINTS

- Source: a1759b2a §68; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: HintBar top>mode>focused>screen>global first nonempty; stable priority reused Vec; full cache key
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Allocating sort/fresh Vec; duplicate hints; shell chord match
- Gates: frame_hintbar_derived warmed draw and component route allocate zero
- Origin: docs/refactoring-plan/history-late-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM38

- Source: GAP-4 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: HintBar truncation draws overflow marker and wide row does not
- Disposition: accepted_UI_invariant; current proof_required
- Remaining proof: Compare rendered ellipsis and drop order/width at narrow and full widths
- Gates: painted marker present/absent plus fitting counts
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:39; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM42

- Source: GAP-8;36468976 §47.4; docs/audit/legacy-test-disposition.md;COMPONENT_ARCHITECTURE.md
- Requirement: StatusBar groups start/end at exact side padding and keep declared order; prove geometry from painted buffer
- Disposition: accepted; current retain_verify
- Remaining proof: Left x+1 right width-1 center between; oracle narrow and offset-origin composition
- Gates: painted coordinates; deliberate wrong alignment fails
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:43; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM43

- Source: GAP-9 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: Last StatusBar survivor is actually ellipsis-truncated to fit, not merely retained by keep-mask
- Disposition: accepted; current proof_required
- Remaining proof: Paint tight width with long mandatory identity and verify adjacent content not overwritten
- Gates: survivor width bound and ellipsis cells
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM45

- Source: GAP-12;§51 3adb6efe; docs/audit/legacy-test-disposition.md;COMPONENT_ARCHITECTURE.md
- Requirement: StatusBar keyed hovered label alone lifts; keyboard suppresses hover; blank/unkeyed pointer restores every label to base
- Disposition: accepted; current retain_verify
- Remaining proof: Real move/key sequence and unkeyed/blank negative; central reference for forced component-wide sample
- Gates: status_bar_hover integration plus no-hover target proof
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:46; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM66

- Source: 3adb6efe proposal;ecc13378 §54.6; COMPONENT_ARCHITECTURE.md;docs/reviews/laneC-app-tick.md
- Requirement: Status strip inherent pure draw-time projection with fixed app-owned byte buffers; preserve facts priorities tones stable keys; no Jx cache/String per frame/StatusItem ownership widening
- Disposition: accepted_tightened_from_allocating_proposal; current proof_traceability_gap
- Remaining proof: Provide zero-allocation projection evidence and pure-repaint fresh-state proof for each contributing screen
- Gates: status_projection pure priority tone key; allocates_zero_per_frame; pure_repaint_never_stale
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:67; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG24

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Generic terminal bindings remain configurable while domain chords stay in apps; hints reflect active actual actions
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: CommandMap/HintBar; F08d
- Gates: Advertised hint reaches same action with exact modifiers
- Origin: docs/refactoring-plan/history-other-obligations.tsv:25; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
