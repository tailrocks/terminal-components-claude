# TASK-019 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Prove F1–F13 from history-inline-obligations.tsv individually: configuration-only props; declaration-order traversal; hidden geometry absence with eligible draft retention; pure height/reveal; ID-only actions; draw purity/FocusOut commit; first action plus all invalidation; nested Select Esc; layer-agnostic body; commit-visible-validation-cross-validation-submit order; editing blocks accidental Enter submit; dirty on commit; secret redaction/clearing.

Preserve per-phase semantic items, private configured bridges and closed FieldKind. FormData::disabled occurs before one mutable value_and_options borrow; inherited OR configured disable applies. No dyn/public item-aware trait, public secret projection, owned values bundle or domain execution.

Test all kinds, sections/pairs, hidden/removal/type/sensitivity transitions, reordered stable IDs, server errors and first failing field reveal/focus. Under ADJ-12 validate every visible field exactly once in declaration order, retain each field error through the existing sensitivity-safe path, skip hidden validators, run cross-validation only after all visible fields pass, and focus/reveal only the first invalid field after the complete scan. Commit the active edit once before clearing stale errors and validation; preserve cancel draft cleanup and immediate choice dirty. Same Form works bare/Panel/Dialog.

## Fixed re-audit contract — opt-in Form radio navigation commit

Consume TASK-018's typed RadioGroupAction::Navigated(ItemKey) and Chose without duplicating keys, cursor reconciliation or item lookup. Ordinary Form defaults keep value changes activation-only. Expose `Form::radio_navigation_commits(self, enabled: bool) -> Self`, default false. The true policy makes source-qualified oracle Form compositions commit both typed events, including boundary-clamped navigation, through the existing private choice update_in_form path; pass the policy into that private bridge rather than duplicating key dispatch. This task owns choice.rs for that bounded signature/behavior change. Because slot.radio is private (main form.rs:386), applications must not reach into state or recreate navigation externally. Keep public scalar FieldControl unchanged; item-bearing choices remain direct per-phase bridges with one value_and_options borrow, inherited-disabled OR configured-disabled and the same configured control in both phases.

TASK-018 already makes the new action exhaustively compile-safe; this task owns policy configuration and precise FormAction::Committed emission only for an admitted controlled-value commit operation under that policy, including an admitted repeated-key commit whose value is equal (not only inequality). Reconcile/seed/removal/pointer Press never masquerade as a commit. Preserve dirty/error/validation ordering, ordinary caller semantics and source-qualified oracle callback behavior when an admitted arrow repeats the selected key. Prove direct Form default-versus-oracle-policy pairs for Up/Down/j/k, clamped selected!=cursor, Space/Enter, press then outside release, enabled/inherited-disabled/configured-disabled, readonly, empty options and reorder without input; compare value, stable cursor, ordered actions/callbacks and exact cells. App producer paths are Showcase Forms/Settings, TablePro Connections and Jackin Accounts/Config/choice modals wherever their admitted composition uses Form.

## Fixed ADJ-15 configured Select consumption

Use TASK-018's existing Select value owner and existing Chose through update_in_form; configured LabelSelect.navigation(Commit).open_keys(ConsumeUnhandled) is the sole source policy. The inherited-disabled reconstruction preserves both props. No new Form switch, SelectAction variant, app manual setter or duplicate key/value engine. A changed closed arrow emits one Chose, writes the controlled usize once and yields existing Committed/dirty semantics; a clamped/same-value choice retains Changed/closure flow but emits no Chose/Committed. Radio's separate ADJ-10 repeated-key commit contract is unchanged. Default Select remains cursor-only and generic Tab traverses; configured open Tab/BackTab is consumed by focused Select before Form traversal, while actual external focus-out still dismisses. Preserve one value_and_options borrow and readonly/inherited/configured disabled. TASK-018 owns select.rs policy implementation; this task proves the existing private bridge and only repairs its already-owned Form integration if necessary.

## Fixed F10 total ordered validation

Original F10 requires every visible field to validate; oracle TablePro ConnForm also validates both name and port and paints both errors when both fail. Main form.rs:1175–1217 currently returns on the first field error: that is an implementation divergence, not an accepted supersession. ADJ-12 fixes the existing Form implementation without a new policy API: commit the active edit once, clear stale errors, validate every visible field exactly once in declaration order, and store each error through existing safe_error/current-sensitivity handling. Remember the first invalid field only for final reveal/focus/Invalid after the complete scan. Hidden fields do not validate. Cross-validation runs once only when every visible field passes; submission follows only when cross-validation also passes. W-019-10 fixes first/middle/last and simultaneous invalid cases with later visible fields, plus all-valid/cross-invalid and all-valid/cross-valid cases. A one-field fixture cannot distinguish this contract from the current early return.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_019.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `conformance`, `dialog_ack_keyboard`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:form

- family: form
- reference_implementation: O:Showcase forms/settings;TablePro connection forms;Jackin forms
- main_implementation: M:crates/tui/src/components/form.rs
- architectural_target: Keyed FormData slots caller values declaration-owned traversal/validation
- visual_status: unverified app composition
- interaction_status: unverified edit/submit focus interactions
- api_refactor_status: implemented; app migration still requires parity
- tests_available: M:form.rs forty-six unit tests;conformance.rs
- tests_missing: Oracle form field order sizing submit/cancel errors and hidden fields; same production app paths

### ARCH:A20

- id: A20
- area: forms and declared fields
- oracle_state: three separate app form engines
- main_state: Form FormData Field FieldKind
- architectural_target: one field engine; app owns domain values; no values() extraction
- status: implemented;app parity incomplete
- remaining_obligation: migrate all oracle forms using exact shared field/scroll/error semantics
- available_gates: conformance;dialog_ack_keyboard;connection-form example
- missing_proof: oracle Jackin/TablePro/Holla form journeys
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1530;7b27732:crates/tui/src/components/form.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A80

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15 K1; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Form values caller-owned, declaration-order focus, conditional fields, sections and pairs
- Disposition: accepted; current retain_and_verify
- Remaining proof: All field kinds and real forms across sizes
- Gates: Form F1-F13
- Origin: docs/refactoring-plan/historical-obligations.tsv:81; global semantic anchor; supplemental clauses retained

### HIST:A81

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15 K1; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Hidden fields contribute no geometry/focus but retain eligible drafts; removed/shape/sensitivity changes erase stale drafts
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Stable Id reorder and dynamic ownership transitions
- Gates: Form sensitivity tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:82; global semantic anchor; supplemental clauses retained

### HIST:A82

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §15 K1; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Submit commits active edit then visible validation then cross-field validation; first error revealed/focused
- Disposition: accepted_clarified_by_ADJ-12; current first_implementation_failfast_incomplete_against_total_validation_contract
- Remaining proof: ADJ-12: commit active edit once; validate every visible field exactly once in declaration order and store all field errors through existing sensitivity-safe storage; cross-validation only if all per-field checks pass; reveal/focus/emit Invalid only for first failure; preserve oracle TablePro simultaneous name/port errors
- Gates: Exact callback and all-error-state logs; first/middle/last invalid; hidden skip; cross gating; two-invalid-field oracle frames
- Origin: docs/refactoring-plan/historical-obligations.tsv:83; global semantic anchor; supplemental clauses retained

### HIST:A83

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §24 M3; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Form options/value borrowed in one mutable access; standalone choices retain borrowed generic phase data
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: No option copy and phase agreement; compile borrow cases
- Gates: FormData examples
- Origin: docs/refactoring-plan/historical-obligations.tsv:84; global semantic anchor; supplemental clauses retained

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

### HIST:A125

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §70; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Form real Runtime Tick+draw zero allocations/bytes; style downgrade ceiling 1079 allocations
- Disposition: accepted; current retain_and_verify
- Remaining proof: Preserve workload binary isolation and exact assertions
- Gates: frame_form_update_draw;style_downgrade_theme_all_levels
- Origin: docs/refactoring-plan/historical-obligations.tsv:126; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-026

- Source: a1759b2a §§15.1,17 A9;91f0296a;3f26ab14; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Form has no cancel builder or public Reconcile implementation; enclosing owner controls cancellation and lifecycle.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Private field reconciliation replaces generic trait; actions remain declared, submit/enter remain explicit.
- Gates: Owner dismiss/cancel zeroization; no Form layer-event ownership; generic API absence; declaration order and form action ordering.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:27; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-027

- Source: 91f0296a §§15.1,17 A9;e49de3f8;c936d51f; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Form reconciliation reads sensitivity before visibility filtering and zeroizes removed/shape/sensitivity-transitioned drafts.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Visibility-only toggles preserve eligible drafts/cursor; ErrorState replaces raw stored public FieldError; reveal is explicit.
- Gates: Dynamic sensitive/plain/hidden/group changes; reorder; removed slots; current-owner detail access; no stale secret error detail.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:28; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-028

- Source: e49de3f8 §§15.1,17 A9;c936d51f §15;3f26ab14; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Public FormState errors are generic; direct secret String begin requires sensitive state; expose is crate-private and masks accept SecretPolicy.
- Disposition: accepted security boundary; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Only private Form draw sees current plain detail; zeroization remains best-effort safe Rust, never guaranteed erasure.
- Gates: Public error reads before update during dynamic sensitivity change; direct sensitive begin/cancel; Debug/display/mask policy; capacity release.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-014

- Source: 27bd918e §23 K1 and§15.1 F1–F13; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Form owns ordered composition, visibility, scrolling, validation/action ordering and error reveal, not domain values or layers.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Closed field kinds with richer Chooser escape; own Form layer rejected.
- Gates: All F1–F13 tests; hidden preserves drafts/no geometry; pure height; simultaneous actions choose declaration-first.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-015

- Source: 27bd918e §23 K1;87ab93d4 §24 M3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Controlled FormData channels borrow values/options per phase; value_and_options solves same-owner mutable value/shared options; no cloned values result bundle.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No generic FormData property closure capturing mutable draft; no secret return vector.
- Gates: Compile borrowing examples; no secret Clone/Eq/Serialize; ID/action-only submit results.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-016

- Source: 27bd918e §15.1 F9–F12; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Submit commits visible controls and validates locally then cross-field, focuses/reveals first error; Enter only when nested/editor input permits; dirty means committed change.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No validation/commit from draw; Cancel policy does not turn blur into Escape.
- Gates: Draft/committed distinction; failed submit; nested open Select; visible field order; external errors survive redraw.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-017

- Source: 87ab93d4 §24;70dacec1 §29.7;ecc13378 STATE839–843 and§67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Item-bearing choices use direct per-phase paths; Form drives choices through private bridges; inherited-disabled OR configured-disabled; public standalone API unchanged.
- Disposition: decided_with_future_trait_widening_open; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Stale architecture open wording does not mandate widening scalar FieldControl; STATE explicitly decides direct path.
- Gates: Borrowed keyed standalone choice and configured Form examples; dynamic inherited/configured disable; single value_and_options borrow; no unauthorized newtrait.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-048

- Source: e49de3f8 §15.1 F13;3f26ab14 §15.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Public FormState errors always generic; private Form draw consults current owner data for plain detail; hidden/inactive reconcile; owner zeroizes on cancel/dismiss.
- Disposition: accepted_security_tightening; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Form does not own LayerEvent lifecycle; stale state cannot determine current sensitivity.
- Gates: Plain→secret between updates; hidden/inactive errors; direct public error; cancellation/dismissal ownercalls; no rawsecret output.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:49; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-053

- Source: 2e453023 §12.5 J1–J5; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Library owns modal chrome/lifecycle, Form composition, choice/facts Dialog conveniences and binding-fed HelpOverlay; remove duplicate form/modal/help engines.
- Disposition: accepted_with_Field_Form_amendments; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Domain facts remain app-owned; DialogBody closedenum rejected.
- Gates: Map each J1–J5 oldcall to library composition plus exact behavior/geometry proof.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:54; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F1

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Props hold configuration only, never values or mutable screen/data borrows.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A01,A47,A80
- Gates: No data-bearing FieldKind/Form props; single configuration helper with per-phase data.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:2; refines A01,A47,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F2

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Tab order is visible focusable field declaration order; Form never duplicates traversal with cx.focus.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A80
- Gates: Tab and ShiftTab observe declared registration order.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:3; refines A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F3

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Hidden fields register no ring/region and contribute no measure while eligible draft/cursor survives.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A81
- Gates: Hide/show retains draft; hidden no geometry; later removal/type/sensitivity invalidation separately covered.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:4; refines A81; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F4

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Field height is pure in FieldSpec, DesignTokens and width; update owns reveal using last-frame area, first-frame absent area documented.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A22,A80
- Gates: Same measured/drawn height both phases and first-layout reveal case.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:5; refines A22,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F5

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: FormAction carries only IDs/action keys; caller owns values; no values bundle crosses submit.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A80,A85
- Gates: Exhaustive action payload proof and no secret clone/serialize channel.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:6; refines A80,A85; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F6

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Form draw commits/cancels/validates nothing; blur commit occurs through FocusOut update.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A02,A87
- Gates: Draw state equality and FocusOut-driven commit test.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:7; refines A02,A87; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F7

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: At most one semantic FormAction; declaration-first controls then action-row buttons; remaining outcomes preserve invalidation.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A04,A05,A80
- Gates: Simultaneous actions resolve first deterministically without losing invalidation.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:8; refines A04,A05,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F8

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Nested Select popup owns its layer lifecycle, independent of draw order; Esc reaches control before outer layer.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A17,A19,A80
- Gates: Nested popup closes before dialog; no deferred repaint or re-registration hack; apply later popover focus policy.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:9; refines A17,A19,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F9

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Form is layer-agnostic composition with decorative background, no own backdrop/frame/trap.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A09,A18,A80
- Gates: Same Form under bare rect/Panel/Dialog; background does not produce UndeliveredIntent.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:10; refines A09,A18,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F10

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Submit commits active edit, validates visible fields in order, then cross-field rules; first error stored/revealed/focused; success emits submit action.
- Disposition: accepted_clarified_by_ADJ-12; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: ADJ-12 resolves original every-visible wording against incomplete fail-fast implementation; A82
- Gates: Every visible validator exactly once in order; all safe per-field errors stored; cross only after all pass; first invalid focus/reveal/action; two simultaneous oracle TablePro errors; hidden skip and per-kind commit policy
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:11; refines A82; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F11

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Enter submits only when focused control is neither swallowing typing nor editing; submit chord comes from Action declaration.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A48,A82
- Gates: Editing/nested input prevents accidental submit; CtrlS action binding route and hint agree.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:12; refines A48,A82; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F12

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Dirty changes on committed mutation or toggle/choice, not draft keystroke.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A80,A87
- Gates: Draft then cancel stays clean; commit and immediate value choice become dirty.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:13; refines A80,A87; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F13

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: FormState/field drafts redact Debug and zeroize secret drafts before discard.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A85,A86
- Gates: Secret never appears in Debug/frame; close/cancel/removal lifecycle uses best-effort safe clearing.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:14; refines A85,A86; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-036

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:M1/M6-M10/M25/M28;515-538;560-568;A8-A13
- Requirement: Author API complete; one props constructor per configured instance; Form drives fields; borrowed extension data
- Disposition: J accepted direction; later K/M shapes govern; current unverified
- Remaining proof: Prevent update/draw configuration drift without reintroducing domain context
- Gates: Disabled behavior parity; external author example; no runtime debug-name side table; inline editor click precedence; external edit app request
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-013

- Source: 0100241f; docs/reviews/adjudication-k-form-grid.md K1
- Requirement: Form is reusable component with controlled per-phase FormData and value-free actions
- Disposition: accepted although source header remains proposed; current not independently tested
- Remaining proof: Visibility/draft retention; commit-before-validate; first-error focus; no values() leak
- Gates: heterogeneous form; hidden/show restored draft; submitted-invalid focus; debug redaction
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:14; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-014

- Source: fa8adb7b; docs/reviews/adjudication-m-small-items.md M3
- Requirement: Choice controls remain generic per-phase collections; closed FieldKind uses label aliases and FormData options/value_and_options
- Disposition: accepted with later item-channel follow-up; current not independently tested
- Remaining proof: Keep items out of props and handle mutable value/options in one borrow; join later Form adjudication
- Gates: borrow-checking external fixture; changing options without rebuilding props
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-016

- Source: e81ca17b; docs/audit/domain-boundary-audit.md3.3; modern-api R19
- Requirement: Secret redacts Debug/Display; actions carry identities; no ratatui Masked
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Follow later accepted disclosure policy; prevent raw secrets in debug/form results/capture
- Gates: redaction corpus; compile-fail clone/eq; explicit reveal/reference paths
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-021

- Source: e81ca17b; docs/audit/domain-boundary-audit.md4.1 J1-J13
- Requirement: Reusable form/choice/info/help/wizard/picker-chain/row-decoration facilities replace duplicated app plumbing
- Disposition: accepted dispositions; exact APIs require architecture join; current not independently tested
- Remaining proof: Every J item needs explicit disposition; do not promote file-system or account semantics to library
- Gates: showcase coverage for new public facilities; wizard rewind/drafts; custom rows; domain boundary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-064

- Source: fa8adb7b; docs/reviews/adjudication-m-small-items.md M3
- Requirement: Closed form label controls have positional value contract; richer keyed/custom-row control through Chooser
- Disposition: accepted historical contract; join laterForm amendments; current not independently tested
- Remaining proof: Do not call positional form choice reorder-stable; app remaps optionindex or chooses keyed external control; no props/state storage crossover
- Gates: Option reorder index mapping; chooser emits fieldId; state Clone+Eq redacting draft; no FieldKind reachable fromstate
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:65; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-065

- Source: 0100241f; docs/reviews/adjudication-k-form-grid.md K1.3/K1.7
- Requirement: Form declaration order drives Tab/action precedence; dirty from commit not keystroke; submit validatesvisible then crossfield and focusesfirsterror
- Disposition: accepted with later API amendments; current not independently tested
- Remaining proof: Do not copy K stale trapped-Select acceptance name; preserve firstframe areaNone no-op and draft retained whenhidden; secret zeroizeonclose
- Gates: Every declared field value coverage; commit-beforevalidation; one action; hidden validation skip; latestSelectfocus semantics
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:66; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-FORM

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Configured FieldKind controls keep private update_in_form/draw_in_form; effective disable inherited OR own
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Public form-specific component API; disabled state duplicated inconsistently
- Gates: Dynamic FormData.disabled before mutable value_and_options borrow; chooser same rule
- Origin: docs/refactoring-plan/history-late-obligations.tsv:30; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-70-FORMPERF

- Source: a1759b2a §70; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Real Runtime one borrowed field Form warm Tick update+draw zero allocation/bytes
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Per-pass Form placements Vec
- Gates: frame_form_update_draw remains concrete public path proof
- Origin: docs/refactoring-plan/history-late-obligations.tsv:41; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM71

- Source: ecc13378 state:832;architecture §30;later §67; REFACTORING_STATE.md;COMPONENT_ARCHITECTURE.md
- Requirement: Scalar FieldControl cannot carry per-phase items; item-bearing choices use direct per-phase paths and Form configured bridges; item-aware trait widening remains open
- Disposition: accepted_current_path;future_widening_deferred; current preserve_accepted_bridge
- Remaining proof: Do not mistake stale §30 unresolved shorthand for requirement to widen trait; retain configured options with current public behavior
- Gates: Form choice item/disabled forwarding and controlled value parity
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:72; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

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
