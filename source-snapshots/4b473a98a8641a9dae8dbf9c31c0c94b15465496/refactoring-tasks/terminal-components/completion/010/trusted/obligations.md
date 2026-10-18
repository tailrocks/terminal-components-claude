# TASK-010 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Preserve all twelve pointer_capture_eligibility tests. Successful publication of disabled/absent/decorative/empty/layer-blocked targets cancels held press/capture; aborted publication does not. Test truly overlapping later disabled top over enabled lower on same layer and across layers; neither owner may receive leaked activation. Re-enable never resurrects a gesture; new independent Press works.

Keep one runtime ring/dispatch/presentation authority; input and draw use the same controlled props and allocation. Pending input is delivered only against successfully published geometry. Ui::reference alone suppresses every registration while forcing only exact-target runtime presentation flags.

Probe Press, Release, Click, DoubleClick, DragStart, Drag, DragEnd and Secondary separately, disable mid-drag, release outside, remove actual part, move eligible part, resize and push/pop modal. Preserve hover observation, keyboard hover suppression, boundary wheel, ClickOnly and outside dismissal. Nested focus settling never redelivers physical input; one typing/cursor owner survives route resize.

## Fixed BF13 effective-chord owner

TASK-010 owns a crate-private effective-chord equivalence in event.rs and its keymap.rs consumers: binding_conflicts, component_conflicts, ordinary and typing-scope conflicts, and FocusedHints::contains_chord. Matching and equivalence remove SHIFT only for Char with the same exact character; non-character modifiers and character case remain distinct. Retain documented structural Chord Eq/Hash and existing binding identities, override keys and revision/cache behavior. Do not globally lowercase codes or redefine raw identity. W-010-10 exercises each conflict scope and focused hints using NONE/SHIFT-equivalent uppercase chars, different-case chars and Up modifier controls. Existing intent::claim_binding_chord has only the Form Enter production caller and remains its exact typed claim contract; no unsupported general claim-routing expansion is required. All production changes stay under the named event/keymap functions plus direct completion010 tests; later028 consumes the repaired owner through existing ancestry.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_010.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `pointer_capture_eligibility`, `pointer_publication`, `focus_traversal`, `focus_restoration`, `typing_owner`, `publication`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:focus-hit-capture

- family: focus-hit-capture
- reference_implementation: O:src/core/focus.rs;hit.rs;O:src/bin/showcase/app.rs:634
- main_implementation: M:crates/tui/src/focus.rs;hit.rs;capture.rs;runtime.rs
- architectural_target: Runtime-owned focus ring hit publication and capture
- visual_status: unverified
- interaction_status: field Press/FocusIn versus completed click differs
- api_refactor_status: implemented; interaction needs migration
- tests_available: M:focus_traversal.rs;focus_restoration.rs;pointer_capture_eligibility.rs;pointer_publication.rs
- tests_missing: CP-02 Down-only/release-outside; disabled barriers; reparent/resize freshness

### ARCH:A03

- id: A03
- area: component phase model
- oracle_state: mutable render owns state changes
- main_state: App update mutable; draw shared
- architectural_target: retained caller state and borrowed props; no business mutation in draw
- status: implemented structurally
- remaining_obligation: preserve read-only draw during parity restoration
- available_gates: draw_takes_shared_self;cache_types_are_derived_only;props_are_built_once
- missing_proof: production-view read-only mutation tests for each restored app
- evidence: 7b27732:crates/tui/src/runtime.rs:44;7b27732:COMPONENT_ARCHITECTURE.md:332

### ARCH:A05

- id: A05
- area: responses and routing
- oracle_state: Outcome/bool/tuple variants
- main_state: Response<A>;Intent;Phase;Bindings
- architectural_target: one typed action/repaint/flow contract; runtime dispatch
- status: implemented;parity unproven
- remaining_obligation: restore exact bubbling/activation without direct input dispatch duplication
- available_gates: activation_origin;conformance;compile_fail_cases_hold
- missing_proof: all app keyboard/mouse action equality
- evidence: 7b27732:crates/tui/src/lib.rs:71;7b27732:COMPONENT_ARCHITECTURE.md:379

### ARCH:A07

- id: A07
- area: focus scopes
- oracle_state: app rings barriers saved focus
- main_state: shared runtime scopes/reconciliation
- architectural_target: one runtime focus ring/traps; disabled entries retained but skipped
- status: implemented;behavior drift possible
- remaining_obligation: restore oracle startup/traversal/repeat click and overlay focus behavior
- available_gates: focus_traversal;focus_restoration;layer_focus_reparent
- missing_proof: oracle focus sequences per app
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:577;7b27732:crates/tui/tests/focus_traversal.rs:1

### ARCH:A08

- id: A08
- area: pointer capture and disabled owner
- oracle_state: per-app pressed/hovered/drag
- main_state: shared capture eligibility/publication; dynamic disable cancels held gesture on successful publication
- architectural_target: top owner absorbs disabled activation; live geometry/capture valid
- status: implemented;12 eligibility tests independently pass;app parity unproven
- remaining_obligation: preserve publication-owned cancellation and extend true overlapping-owner proof
- available_gates: pointer_capture_eligibility;pointer_publication;conformance
- missing_proof: true same-layer/cross-layer overlap intent proof plus oracle app trajectories
- evidence: 715ee0777e20a09e0f373b07024076bdc742ef1d;7b27732:crates/tui/src/runtime.rs:1725;architecture-adjudication.md ADJ-01

### ARCH:A11

- id: A11
- area: cursor and typing
- oracle_state: per-app editing heuristics
- main_state: TypingPolicy and cursor owner publication
- architectural_target: one explicit runtime typing owner with presented geometry
- status: implemented;component mismatch
- remaining_obligation: remove component focus-auto-edit drift while preserving runtime ownership
- available_gates: typing_owner;keyboard_editor;publication
- missing_proof: oracle input/textarea/code cursor trajectories
- evidence: 7b27732:crates/tui/src/runtime/typing.rs:1;7b27732:crates/tui/tests/typing_owner.rs:1

### ARCH:A22

- id: A22
- area: exact reference fixtures
- oracle_state: manual fake state renderers
- main_state: Ui::reference exact ReferenceTarget
- architectural_target: reference paints only; centralized inert registration and scoped restoration
- status: implemented
- remaining_obligation: use real components with honest semantic fixture state
- available_gates: reference_rendering_is_ui_scoped;legacy_forced_state_apis_are_absent
- missing_proof: mutation rejecting broad state forcing and registration leakage
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:8492;7b27732:crates/tui/src/lib.rs:69

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A01

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §3 / A,J; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Separate update and draw; props borrow configuration and state stays caller-owned
- Disposition: accepted; current retain_and_verify
- Remaining proof: Compile borrowed phase calls and external state ownership; no model data stored in props
- Gates: architecture;external examples
- Origin: docs/refactoring-plan/historical-obligations.tsv:2; global semantic anchor; supplemental clauses retained

### HIST:A03

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §3 / J; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Derived caches runtime-owned and keyed by Id/type plus size/theme/source generation
- Disposition: accepted; current retain_and_verify
- Remaining proof: Invalidate stale derived facts without hiding durable state in caches
- Gates: cache invalidation;resize;repeated draw
- Origin: docs/refactoring-plan/historical-obligations.tsv:4; global semantic anchor; supplemental clauses retained

### HIST:A04

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §6 / C,J; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Response flow, invalidation and typed semantic action are independent
- Disposition: accepted; current retain_and_verify
- Remaining proof: Boundary wheel consumes without repaint; actions retain identity
- Gates: response unit tests;scroll boundary
- Origin: docs/refactoring-plan/historical-obligations.tsv:5; global semantic anchor; supplemental clauses retained

### HIST:A05

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §6,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Only Response<()> may fold; preserve first present id and strongest invalidation
- Disposition: accepted; current retain_and_verify
- Remaining proof: Compile-fail action-bearing fold; unit merge laws
- Gates: trybuild;response laws
- Origin: docs/refactoring-plan/historical-obligations.tsv:6; global semantic anchor; supplemental clauses retained

### HIST:A07

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §7,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Id structural equality/order/hash and duplicate diagnostics; no claimed mathematical collision prevention
- Disposition: accepted; current retain_and_verify
- Remaining proof: Derived-id corpus and duplicate diagnostics
- Gates: id unit tests;external const patterns
- Origin: docs/refactoring-plan/historical-obligations.tsv:8; global semantic anchor; supplemental clauses retained

### HIST:A08

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: One runtime owns focus registration, traversal, restoration, hover, pressed/capture and cursor
- Disposition: accepted; current retain_and_verify
- Remaining proof: Remove app duplicates while reproducing oracle behavior
- Gates: architecture boundary;four-app interactions
- Origin: docs/refactoring-plan/historical-obligations.tsv:9; global semantic anchor; supplemental clauses retained

### HIST:A09

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Hit priority highest layer then latest registration; decoration does not receive control input
- Disposition: accepted; current retain_and_verify
- Remaining proof: Overlap/layer/decorative negative probes
- Gates: runtime hit tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:10; global semantic anchor; supplemental clauses retained

### HIST:A10

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§73; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Disabled top hit absorbs all activating pointer input without lower fallthrough
- Disposition: accepted; current proof_gap
- Remaining proof: Add direct overlapping enabled lower/disabled upper runtime proof
- Gates: disabled activation;overlap test
- Origin: docs/refactoring-plan/historical-obligations.tsv:11; global semantic anchor; supplemental clauses retained

### HIST:A11

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§73; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Disabled hover can be observed but presentation strips hovered/pressed; wheel may remain routable
- Disposition: accepted; current retain_and_verify
- Remaining proof: Separate activation eligibility from wheel/hover/ClickOnly behavior
- Gates: AC2;disabled conformance
- Origin: docs/refactoring-plan/historical-obligations.tsv:12; global semantic anchor; supplemental clauses retained

### HIST:A12

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§73;715ee0777e20a09e0f373b07024076bdc742ef1d; COMPONENT_ARCHITECTURE.md;crates/tui/src/{runtime,capture}.rs
- Requirement: Successfully published disabled/absent/decorative/empty/layer-blocked target cancels held capture and press; no Release/Click resurrection
- Disposition: accepted_implementation_closes_historical_gap; current implemented_retention_verification
- Remaining proof: Retain publication-boundary cancellation and independent timed feedback; do not create duplicate implementation task
- Gates: pointer_capture_eligibility suite; disable/re-enable; dropped-frame boundary; no lower retarget
- Origin: docs/refactoring-plan/historical-obligations.tsv:13; global semantic anchor; supplemental clauses retained

### HIST:A13

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Pointer capture preserves actual press anchor and owner; resize/removal/generation mismatch releases capture
- Disposition: accepted; current retain_and_verify
- Remaining proof: Drag outside/release; owner vanishes; nested capture rejected
- Gates: capture lifecycle tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:14; global semantic anchor; supplemental clauses retained

### HIST:A14

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Hover and wheel never steal focus; keyboard suppresses hover until pointer movement
- Disposition: accepted; current retain_and_verify
- Remaining proof: Pointer/key sequence asserts focus and hover separately
- Gates: conformance;app journeys
- Origin: docs/refactoring-plan/historical-obligations.tsv:15; global semantic anchor; supplemental clauses retained

### HIST:A16

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Focus settling never duplicates physical input; bounded nonconvergence diagnostic preserves pending notifications
- Disposition: accepted; current retain_and_verify
- Remaining proof: Transition delivery assertions through resize and settling
- Gates: focus-settle tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:17; global semantic anchor; supplemental clauses retained

### HIST:A17

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§9; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Esc first reaches focused editor, then application bubble/escape policy and layer dismissal
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Nested editing/modal/menu ladder and focus restoration
- Gates: Esc journey matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:18; global semantic anchor; supplemental clauses retained

### HIST:A45

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §72; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Ui::reference sole inert exact-target boundary; only focused/focus-visible/hovered/pressed may be forced
- Disposition: accepted_supersedes_local_forcing; current retain_and_verify
- Remaining proof: No component state_override/inherit_forced; semantic states caller-owned
- Gates: reference API absence;conformance
- Origin: docs/refactoring-plan/historical-obligations.tsv:46; global semantic anchor; supplemental clauses retained

### HIST:A46

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §72,§73; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Inert/reference suppression centrally covers all registrations, bindings, cursor, layout, scopes and callbacks
- Disposition: accepted; current retain_and_verify
- Remaining proof: Malicious callbacks and nested target restoration; background paints retained
- Gates: reference sink;external author
- Origin: docs/refactoring-plan/historical-obligations.tsv:47; global semantic anchor; supplemental clauses retained

### HIST:A49

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §68; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Explicit component Tab bindings precede traversal; raw ignored input alone reaches bubble
- Disposition: accepted; current retain_and_verify
- Remaining proof: Tab/BackTab editing and modal focused routing
- Gates: key precedence tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:50; global semantic anchor; supplemental clauses retained

### HIST:A60

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §52; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: register_focus_only is explicit no-hit API; ClickOnly no-op and area remains absent
- Disposition: accepted; current retain_and_verify
- Remaining proof: Zero area ring entry and no phantom click
- Gates: focus-only tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:61; global semantic anchor; supplemental clauses retained

### HIST:A74

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §69; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Pressed PartRef styles only actual Grid row/cell/action or ScrollRegion thumb
- Disposition: accepted; current retain_and_verify
- Remaining proof: Sibling and whole-container press isolation
- Gates: pressed-part tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:75; global semantic anchor; supplemental clauses retained

### HIST:A97

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §54 later timing; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Exactly one Bootstrap; first timer pass Tick then Settle; monotonic deadlines persist and input does not advance elapsed time
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: 2199/2200 ms and input-flood invariance; no unsolicited ticks
- Gates: elapsed_contract;runtime scheduler
- Origin: docs/refactoring-plan/historical-obligations.tsv:98; global semantic anchor; supplemental clauses retained

### HIST:A102

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md later runtime docs;amendments=2caff455,577bf533; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Registry/geometry published only after successful output; restored focus validated against publication
- Disposition: accepted_later_amendment; current retain_and_verify
- Remaining proof: Failed output/stale hit and queued input after route/resize
- Gates: publication tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:103; global semantic anchor; supplemental clauses retained

### HIST:A103

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md later runtime docs;amendments=b1c8c447,934c92dc; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Typing and cursor ownership explicit; declared caret offers resolved conditionally
- Disposition: accepted_later_amendment; current retain_and_verify
- Remaining proof: Focus/editor/caret visibility under overlays and first input
- Gates: typing/cursor tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:104; global semantic anchor; supplemental clauses retained

### HIST:A121

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §27.4(b):6293-6303; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Empty intent drain zero probes/alloc; historical two-intent wording superseded by one-intent differential
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Exactly one intent and one update pass: probes(500)-probes(20)==480; empty probes0; both paths allocations0; construction-cumulative counter preserved; normalized (ns500*20)/(ns20*500)<=1.25 under PERF_STRICT; raw ratio reported only; reject absolute500 and modulo480
- Gates: intents drain perf exact amended workload and differential
- Origin: docs/refactoring-plan/historical-obligations.tsv:122; global semantic anchor; supplemental clauses retained

### HIST:F23c

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f23c-event-freshness-resize-v03.md
- Requirement: Queued input observes current owner/geometry through resize
- Disposition: current_holla_done_later_apps_open; current current_main_mapping_required
- Remaining proof: Before/after first render and resize event sequences
- Gates: freshness PTY and runtime
- Origin: docs/refactoring-plan/historical-obligations.tsv:159; global semantic anchor; supplemental clauses retained

### HIST:F23d

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f23d-input-flood-fairness-v04.md
- Requirement: Input floods cannot starve work or advance simulated time incorrectly
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Bounded fairness and deterministic clock assertions
- Gates: input flood
- Origin: docs/refactoring-plan/historical-obligations.tsv:160; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-018

- Source: a1759b2a §§3.3,6,8.2,8.6,18.2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Uncaptured Move targets exactly one top live non-decorative part, changes hover without focus/activation; captured Move is Drag only.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Phase::Move added; menu hover cursor updates occur in update, not draw or Press substitution.
- Gates: Decorative overlap; live top layer; no-target case; visible-hover repaint; captured unbuttoned movement; enabled-menu cursor change.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:19; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-019

- Source: a1759b2a §§3.3,13.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Explicit effective Tab/ShiftTab/BackTab component bindings precede fallback ring traversal.
- Disposition: accepted; oracle product mapping controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Navigation-only Tab rule amended; Capture still runs first.
- Gates: Bound/unbound Tab and BackTab; remap/remove transitions; overlay and edit-mode focus paths match oracle.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-021

- Source: a1759b2a §§12.1,16.2,17,18.3,21,28;15ecde1e;0f018368; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Ui::reference makes complete subtree inert and targets only one owner/part with FOCUSED/FOCUS_VISIBLE/HOVERED/PRESSED.
- Disposition: accepted supersession; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Deletes all component state_override/inherit_forced, including FieldControl hooks; historical rejection of central scope is superseded.
- Gates: Callback/slot/nested component suppression of hits/ring/layout/cursor/bindings/layers; exact target and None; no semantic flag injection.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-047

- Source: 2caff455 §16.1;577bf533 §§3.3,21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Restored owner receives original retained key only after successful live publication proves opener enabled/present/admissible.
- Disposition: accepted supersession; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old before-next-draw exception withdrawn; closing FocusOut immediate; invalid opener uses survivor reconciliation; fresh programmatic focus distinct.
- Gates: Removed/disabled/trapped opener; aborted/dropped paint and Scene cannot acknowledge; exact retained-key and callback ordering.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:48; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-048

- Source: b1c8c447 §8.4;2f139259 §8.4; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Explicit typing fallback may own text/paste/editing/cursor without moving navigation focus; ambiguous targets choose none.
- Disposition: accepted extension; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Primary editor blocks fallback even idle/read-only; conditional cursor offers need same-owner/layer declaration; unselected valid offers silent.
- Gates: Unicode draft/caret while row focus retained; modal/aborted/model-change barriers; conflict diagnostics; raw cursor rejection preserved; zero-warm-allocation bound Scene.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:49; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-001

- Source: 2e453023 §§3–5;95ab6529 §21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Caller owns durable component state; update mutates through explicit state/model without Buffer; draw uses shared self/state/model and cannot commit or emit semantic actions.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Derived runtime cache mutation is not semantic state mutation.
- Gates: Compile-fail draw mutation plus state-before/after draw comparisons for every component and composition.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-004

- Source: 2e453023 §§6–8;587c53bd §25; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Stable kind-tagged Id/ItemKey derivation; identity and order semantics independent of debug labels; duplicate IDs diagnosed.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Structural Eq enables const patterns; corpus hash equality is not a universal collision proof.
- Gates: Const-pattern compile test; pair ordering/separator corpus; duplicate ID negative control.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-006

- Source: 2e453023 §§3,8;95ab6529 §21;587c53bd §25; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime owns ring/hits/capture/hover/press; last published geometry is authoritative; app cannot retain mutable geometry caches.
- Disposition: accepted_with_later_publication_amendment; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Candidate draw is not successful publication.
- Gates: Read-only app audit; aborted/dropped output; resize/removal; no stale input authority.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:7; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-008

- Source: 2e453023 §§8–9;95ab6529 §21;dc3e0fa1 §28 P3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Layer IDs allocated at open; hit priority is layer then latest registration; capture released on close; lifecycle addressed to decorative owners must also be consumed.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No exemption based solely on absence of focus/hit registration; update layer owner unconditionally.
- Gates: Decor-only LayerCancel/FocusOut delivery; closed Dialog next update; topmost-hit order permutation.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-010

- Source: 95ab6529 §21 items3,6;587c53bd §25; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Focused editor sees Escape before bubble/layer/screen; intent queue frozen and indexed by owner; mutable services independent of iterator borrow.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No single mutable Cx borrow through iteration; empty queue zero probes.
- Gates: Editor cancel-before-dismiss; compile borrow proof; zero probes on empty; owner-bucket complexity counter.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-011

- Source: 95ab6529 §21;587c53bd §25; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: No intent redelivery during bounded focus reconciliation; pending focus survives resize and pass-limit rollover to next handle.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Fifth focus request must not be silently dropped.
- Gates: Multi-hop focus chain with exactly-once events; resize then next handle; bounded termination.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:12; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-012

- Source: 95ab6529 §21 item4; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Response keeps typed action separate from consumed/changed/repaint; only unit-action Response supports bitwise merge.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Reject dropped actions from generic bitwise combination.
- Gates: Compile-fail action Response merge; boundary-wheel consumed/changed distinction; typed action trace.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-034

- Source: dc3e0fa1 §28 P5;70dacec1 §29 Q2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Reference-state projection must remain inert and cannot manufacture input/focus/capture authority.
- Disposition: accepted_invariant_old_API_superseded; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Component-local state_override/inherit_forced and private Fixture paired override are superseded by§72 Ui::reference; late-reader direct source join required.
- Gates: Exact four-runtime-bit reference projection; derived props unchanged; no registrations/lifecycle/cursor; live child escape rejected.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-040

- Source: 2caff455 §16.1;577bf533 §§3,21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Restore focus only after successful live publication validates historical opener; retain original key until settlement; invalid opener reconciles.
- Disposition: accepted_later_supersession; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Immediate opener FocusIn and stale-last-ring exception explicitly superseded.
- Gates: Removed/disabled/trappedout opener; abort/drop/Scene cannot acknowledge; close FocusOut immediate; original retained key delivered once.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:41; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-041

- Source: b1c8c447 §8 typingamendment;2f139259 §8; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Explicit fallback typing owner selected from complete geometry independently of navigation focus; conditional caret offers obey sameowner/layer declaration and final admission.
- Disposition: accepted_later_extension; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Focused-owner default and rawcursor rejection remain; fallback cannot use stale/aborted facts.
- Gates: Ambiguity diagnostic; idle/read-only primary block; modal rejection; contextual commands; Scene parity; warmzeroalloc.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-001

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;834aa58e; docs/audit/api-audit.md:19-38;904-916;1289
- Requirement: Uniform typed actions separate consumption and redraw
- Disposition: accepted direction via owner join; current unverified
- Remaining proof: No bool tuple or polled-result parallel protocol
- Gates: Typed action mapping; consumed without repaint; exact cardinality
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-003

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:40-87;998-1004
- Requirement: Stable ownership and item identity replace reverse owns/locate scans
- Disposition: accepted amended; current unverified
- Remaining proof: Avoid positional Path and stale frame geometry
- Gates: Insert/remove/reorder plus click/close/edit identity; collision-kind negative control
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:4; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-004

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;e81ca17b; docs/audit/api-audit.md:151-194;docs/audit/app-audit.md:132-178
- Requirement: Encapsulated hit/focus registration and layer ownership prevent inert bypass and manual reregistration
- Disposition: accepted amended; current unverified
- Remaining proof: Replace public registry mutation and duplicate placement algorithms
- Gates: Reverse registration order; nested pointer/keyboard isolation; dismissal focus restore
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-024

- Source: e81ca17b; docs/audit/architecture-research.md:132-154;344-454
- Requirement: HashedId/debug registry and whole-key reconciliation sketches
- Disposition: amended/superseded; current unverified
- Remaining proof: Structural identity plus cheap change-aware reconciliation
- Gates: Same-index kind collision; reorder action; 100k versus1k dispatch/draw
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:25; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-033

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B3/B5/B6/B15;732-735
- Requirement: Phase-call data; derived-only runtime caches; post-update Esc
- Disposition: J accepted; K amends grid model split; current unverified
- Remaining proof: Join latest model/cache/ordering contracts to actual callers
- Gates: Borrow-compiling mutation closure; shared draw; derived-cache structural and invalidation proof; editor cancel before dismiss
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-034

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B1/B2/B4/B8/B10;M11/M24;540-555;737-744
- Requirement: Compilable builders; independent frozen intent borrow; Field identity; command/action split; lossless Response fold
- Disposition: J accepted; later exact public API governs; current unverified
- Remaining proof: Eliminate compile and semantic-action-loss enabling conditions
- Gates: Complete external examples; compile-fail semantic fold; intent iteration plus mutable services; paste arena lifetime; no duplicate Field ID
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-035

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B7/B11/B12;M2-M5/M29;572-587
- Requirement: Bounded transition-only focus settle; owner capture; layer outside-hit and restore semantics
- Disposition: J accepted; later runtime ordering governs; current unverified
- Remaining proof: Verify order edges instead of replay or draw-order ownership assumptions
- Gates: No duplicate key activation; lower-layer outside hit; inert silent discard versus rejection; pre-draw restore key; close releases capture; duplicate layer draw diagnostic
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-041

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B13/B14;629-646;A4/A11
- Requirement: Noncolor pressed state; cache hints/hash; empty intent fast path; bounded diagnostics; no secret draw allocations
- Disposition: J accepted then container/Id/mono/cache amendments; current unverified
- Remaining proof: Reject review ownSmallVec; replace obsolete debug-Id assumptions; preserve current secret and mono contracts
- Gates: Zero unchanged-focus hint allocations; no lost cache generations; nonempty drain scale; bounded diagnostic drop count; actual pressed bracket cells; no raw secret disclosure
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-001

- Source: a156054d; docs/audit/interaction-audit.md B2-B3; STATE
- Requirement: Retained props plus caller-owned state; explicit update/draw; runtime resolves intents
- Disposition: accepted; immediate show rejected; current not independently tested
- Remaining proof: Preserve phase capability separation across all migrated apps
- Gates: draw shared-self; no semantic mutation on repeated draw; truthful handle outcome
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-002

- Source: e81ca17b; docs/audit/interaction-audit.md B1
- Requirement: Separated and kind-tagged Id plus stable ItemKey; typed part routing
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Stable logical identity under insert/remove/reorder; no owns/locate inversion
- Gates: collision-kind corpus; actual click/close key after reorder; negative controls
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:3; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-003

- Source: e81ca17b; docs/audit/interaction-audit.md B2
- Requirement: Response separates flow invalidation and semantic action
- Disposition: accepted and amended; current not independently tested
- Remaining proof: No tuple/bool/polled-result fallback; typed action mapping without value leakage
- Gates: compile-fail must_use and unit-only BitOr; consumed-without-repaint
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:4; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-004

- Source: e81ca17b; docs/audit/interaction-audit.md B3
- Requirement: Runtime owns routing services, not retained boxed component tree
- Disposition: accepted; boxed tree/full-tree dispatch rejected; current not independently tested
- Remaining proof: Apps own data and domain composition; direct owner/part dispatch
- Gates: borrowed external component compile; no full collection scan per event
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-005

- Source: e81ca17b; docs/audit/interaction-audit.md B4-B8
- Requirement: Focus scopes restoration explicit capture nested layers and cursor arbitration
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Top-layer hit wins regardless registration; focused owner cursor; resize delivers focus transition
- Gates: nested overlays; reversed draw order; capture release; multi-writer cursor; resize focus pair
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-008

- Source: e81ca17b; docs/audit/interaction-audit.md B10
- Requirement: Out-of-band repaint deadlines alongside Response invalidation
- Disposition: accepted; Layout retained as Paint-equivalent initially; current not independently tested
- Remaining proof: One-shot repaint and deterministic clock; preserve oracle animation phases
- Gates: virtual tick deadline sequences; no input-dependent drift
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-028

- Source: ba858131; docs/audit/modern-api-audit.md R7-R13
- Requirement: Runtime cursor write; semantic theme styles; modifier patch laws; typed glyph sets; integer layout vocabulary
- Disposition: accepted; current not independently tested
- Remaining proof: Do not introduce Stylize/literal RGB/constraint solver or second scrollbar state; preserve Junie symbols
- Gates: boundary checks; modifier removal; layout tiny sizes; typed glyph/border contract
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-032

- Source: e2be0ced; docs/reviews/slice2-architecture-review.md; STATE
- Requirement: Data moves to phase calls; derived Ui cache; frozen intent queue outlives Cx borrow; editor receives Esc before layer
- Disposition: accepted J amendments; current not independently tested
- Remaining proof: Keep borrowed mutation ergonomic and cache nonsemantic; never replay original input during focus settling
- Gates: external close/reorder fixture; cache derivation; Esc editor/modal sequence; exactly-once actions
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:33; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-043

- Source: bb92a657; docs/reviews/adjudication-p-prototype-decisions.md P3/P5/P6
- Requirement: Mono disabled text must remain visible; status-driven fixtures; forced-state composition inert; dismissal intents delivered
- Disposition: accepted and amended; current not independently tested
- Remaining proof: No black-on-black Faint fallback; no props mismatch; do not gate owner update on open state
- Gates: owner dismissal action; actual nonempty text; forced child registration inert; status affordance
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-059

- Source: 84993dfc; docs/reviews/adjudication-o-foundations-followups.md O4b
- Requirement: Intent cost uses zero empty probes plus480 nonempty probe delta per single update pass; raw500/20 time ratio only reported
- Disposition: accepted; absolute probe500 and raw1.25 ratio rejected; current not independently tested
- Remaining proof: Keep construction-cumulative counter; exactly one probe per drain not perframe absolute; legitimate additional pass must explain changed delta
- Gates: Zero allocations; empty0;500minus20=480; normalized(ns500*20)/(ns20*500)<=1.25 strict
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:60; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-062

- Source: bb92a657; docs/reviews/adjudication-p-prototype-decisions.md P3
- Requirement: Diagnose every undrained runtime-addressed Layer/Cancel/FocusIn/FocusOut owner regardless decorative registration
- Disposition: accepted with factual correction; current not independently tested
- Remaining proof: Unconditional owner update; size at opener prevents Fill flash; decorative pointer exemption stays; no redraw-based result polling
- Gates: Gated-shape diagnostic negativecontrol vs unconditional Dismissed action; decorative pointer no bucket; all within samehandle
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:63; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-54-CAUSE

- Source: ecc13378 COMPONENT_ARCHITECTURE §54.1–.3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Cx-only Bootstrap/Event/Tick/Settle; one Bootstrap; one Tick per delivery; earliest persistent deadline
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Ui/FrameRead update cause; session-only bootstrap; unsolicited idle ticks
- Gates: Retain exact six library tests listed in §54.8; test terminal/headless lifecycle and settle reruns
- Origin: docs/refactoring-plan/history-late-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-MOVE

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime delivers Phase::Move; enabled menu Move changes cursor; Click activates
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Press/Release/Drag masquerading as hover
- Gates: Exactly one uncaptured Move; capture produces Drag; no focus/activation; paint only visible hover change
- Origin: docs/refactoring-plan/history-late-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-68-PUBLISH

- Source: a1759b2a §68; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Focused non-forced owner publishes erased static/dynamic descriptors; latest contiguous additive table; no Any/box/unsafe
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Fresh descriptor Vec; dynamic hidden commands separate resolver
- Gates: Capture then explicit component including Tab, traversal, raw Key, ignored-only Bubble; hidden duplicates diagnosed
- Origin: docs/refactoring-plan/history-late-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-69-PRESS

- Source: a1759b2a §69; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Snapshot/capture retain owner and PartRef; Grid press row/cell/actions targeted; ScrollRegion only thumb
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Child press paints whole container pressed
- Gates: Exact target cells/regions; forced target cursor or thumb only; unrelated children unforced
- Origin: docs/refactoring-plan/history-late-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-72-BOUNDARY

- Source: a1759b2a §72; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Ui.reference sole inert subtree boundary, target None or exact id/optional part, only four runtime bits
- Disposition: accepted-superseding; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: §28P5/§29Q2/§39 component-local state_override/inherit_forced and composite broadcast
- Gates: AST legacy absence/reference scope; nested callback and slot side-effect suppression; semantic state remains real
- Origin: docs/refactoring-plan/history-late-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-73-HIT

- Source: 33eeb99e/03697429 §73D1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Hit max(layer,registration index), disabled absorbs activating pointer, never lower-owner search
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Disabled fallthrough
- Gates: Current AC1 only neighbors; add actual overlapping disabled top/enabled lower adversarial proof
- Origin: docs/refactoring-plan/history-late-obligations.tsv:47; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-73-EXCEPTIONS

- Source: 33eeb99e/03697429 §73D1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Disabled hover/Move inspectable but visual flags removed; wheel routable; ClickOnly active; below-layer outside first
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Blanket disabled routing suppression; breaking outside dismissal
- Gates: AC2 retains all exception cases
- Origin: docs/refactoring-plan/history-late-obligations.tsv:48; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-73-CAPTURE

- Source: §73 historical gap;715ee0777e20a09e0f373b07024076bdc742ef1d;main runtime.rs1725/capture.rs19; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Published disabled/absent/decorative/empty/layer-blocked target cancels capture and press; no release/click resurrection
- Disposition: implemented-retention-verification; current implemented; direct source verified; fresh focused suite owned by architecture adjudicator
- Remaining proof: Earlier assessment reading only direct delivery at runtime.rs1107 missed publication-time reconciliation
- Gates: Retain pointer_capture_eligibility suite: disable/re-enable, dropped output, no lower retarget and independent timed feedback; current implementation verified by source, fresh suite owned by architecture adjudicator
- Origin: docs/refactoring-plan/history-late-obligations.tsv:49; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-73-INERT

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §73D3 (current cumulative source; originating edge in section coverage index); COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Public registration APIs centrally no-op suppressed; reference suppresses focus/layer/layout/cursor/bindings; normal inert layers keep semantics
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Component-local guards as correctness authority
- Gates: Arbitrary callbacks/slots, nested reference restoration, external author API and inert-below tests
- Origin: docs/refactoring-plan/history-late-obligations.tsv:51; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM23

- Source: 7f7cc6ac §39;later §72; COMPONENT_ARCHITECTURE.md
- Requirement: Reference forcing substitutes runtime presentation flags without erasing caller semantic state; all registrations suppressed through central Ui scope
- Disposition: accepted_operator;local_APIs_superseded; current retain_verify
- Remaining proof: Exact component/item/part reference; preserve error/checked/readiness; absence vs empty reference explicit
- Gates: runtime-vs-derived disagreement; stale hover replacement; props survive; inert subtree
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM36

- Source: GAP-2;36468976 §47.1–3;later §73; COMPONENT_ARCHITECTURE.md;docs/audit/legacy-test-disposition.md
- Requirement: Disabled absorbs hover/pointer presentation structurally; recipe ordering/conflict tests alone miss disjoint slots and variant-over-family effects
- Disposition: accepted_amended_by73; current retain_verify
- Remaining proof: Use current §73 ownership and oracle reconciliation; test disabled overlay opacity and capture transition separately
- Gates: DISABLED vs DISABLED|HOVERED equality; overlapping target; owner becomes disabled during capture
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM59

- Source: 3adb6efe §52 Q3; COMPONENT_ARCHITECTURE.md;docs/reviews/laneB-grid-contract.md
- Requirement: Explicit register_focus_only adds zero-rect ring entry without hit target; ClickOnly no-op; disabled unreachable; read-only remains reachable
- Disposition: accepted; current retain_verify
- Remaining proof: Never weaken ordinary zero-area component rejection; area_of None; click ignored with UnaddressableId
- Gates: enabled/disabled/read-only traversal; no-hit diagnostic; tiny ordinary control negative
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:60; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM62

- Source: 3adb6efe §54;ecc13378 §54.1–2; COMPONENT_ARCHITECTURE.md;docs/reviews/laneC-app-tick.md
- Requirement: Runtime exactly one intent-free clock-free bootstrap; Cx-only Bootstrap Event Tick Settle; one timer delivery causes Tick once through settling
- Disposition: accepted_strengthened; current retain_verify
- Remaining proof: Same lifecycle terminal and headless; no draw-time cause branch; event before first draw still bootstraps once
- Gates: bootstrap_runs_once; tick_cause_once_when_focus_settles; headless lifecycle
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:63; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM63

- Source: 3adb6efe §54;ecc13378 §54.3;later runtime amendments; COMPONENT_ARCHITECTURE.md
- Requirement: Repaint deadlines persistent earliest-wins; unrelated inputs cannot erase/postpone; expired delivery clears only its deadline; no unsolicited idle ticks
- Disposition: accepted_later_monotonic_amendments; current retain_verify
- Remaining proof: Retain later monotonic/publication contracts; compare exact oracle simulation feedback timing
- Gates: unrelated key/mouse/paste/resize; min poll/deadline; headless controlled ticks; no-wall-time equality
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:64; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG11

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Applications own domain behavior, not routine focus/hover/press/hit/cursor/child routing
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Runtime and app ownership matrices
- Gates: Anti-copy/ownership scanner plus real application event flows
- Origin: docs/refactoring-plan/history-other-obligations.tsv:12; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG16

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Domain data, props, durable interaction, controlled values, frame geometry, theme and actions have separate owners
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-core; architecture state model
- Gates: Ownership/API review and compile contracts
- Origin: docs/refactoring-plan/history-other-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG20

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Nested/repeated component parts have stable collision-safe identity and readable debugging
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Id/Part/Scope; runtime target registry
- Gates: Duplicate-identity rejection and stable child-source tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG21

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Deterministic Tab/ShiftTab, disabled skip, roving composite cursor, nested scope, focus restoration and focus-visible
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-focus; app matrices
- Gates: Keyboard traversal and disappearing opener journey
- Origin: docs/refactoring-plan/history-other-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG22

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Hover never steals focus; keyboard suppresses stale hover; completed valid click matches keyboard semantic action
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-pointer; runtime and component response
- Gates: Down/up/outside/disabled and keyboard-mouse equivalence tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG23

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Pointer capture, drag selection, topmost wheel, nested scroll and scrollbar ownership
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-pointer/scroll; P03
- Gates: Captured drag/resize/remove-owner and nested-boundary scroll tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
