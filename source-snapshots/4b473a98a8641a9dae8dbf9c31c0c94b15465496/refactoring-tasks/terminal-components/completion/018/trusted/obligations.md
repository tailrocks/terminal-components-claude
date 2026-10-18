# TASK-018 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Checkbox/Toggle values remain caller-controlled; RadioGroup stable chosen key is distinct from cursor and only seeds missing navigation. Select closed/open markers preserve exact reserved position, strip false field SELECTED and show popup chosen once. Complete the real mono_pressed_choice_keeps_the_label_geometry proof.

Use shared fields/rows/runtime popup; no item-aware public FieldControl widening. Preserve source item ownership, effective inherited/configured disable and explicit glyph Set/Clear over defaults.

Compare narrow/full/zero clipping, checked/unchecked/pressed/hovered/disabled all-color cells and complete pointer containment. External value change/reorder/removal does not retarget established cursor. Select Esc/focus-out closes correctly; installed readiness slot differs from single-glyph override; no label truncation from brackets.

## Fixed re-audit contract — source-fixed choice geometry and typed navigation

ADJ-10 closes the historical unresolved Choice question without blessing candidate frames. In oracle choice.rs, a nonempty row reserves gutter x+0 and mark starting at x+1. Checkbox normal marks are [✓]/[ ], Radio marks (●)/( ), and Toggle tracks ──●/○──; width<4 uses the single-cell ✓/□, ●/○ and ●/○ respectively at the same x+1, clipped to the allocation. Labels begin at x+5 in every state; Checkbox truncates to width.saturating_sub(6), Radio and Toggle to width.saturating_sub(5). Radio's group label is x+2 on its separate first row. Toggle's on/off text starts at 6+the untruncated label's measured width only when offset+3<row.width. Width/height zero paints and registers nothing. No pressed/mono state inserts brackets inside the label run or reallocates its columns. Use only existing reserved cells for any explicitly requested adornment; default oracle application composition retains exact source cells. This fixes the named mono_pressed_choice_keeps_the_label_geometry proof, not its test name alone.

At widths 0,1,2,3,4,5,6 and a full-width overflow label, compare exact gutter/marker/label/trailing coordinates and all clipped/shadow cells for off/on, focused, hovered, pressed and disabled under each source-applicable color/theme. Disabled clears interactive emphasis and cannot choose; glyph Set/Clear/Inherit remain separately proved. Reject in-run bracket insertion, shifting a compact label, Toggle's wrong truncation budget, fabricated trailing text and lost neighbor cells.

Preserve RadioGroupState::cursor()/cursor_index() (main choice.rs:826), caller-controlled value(ItemKey) (:1062), and generic cursor/value separation. Add typed RadioGroupAction::Navigated(ItemKey) emitted only by admitted enabled shared navigation, including a clamped boundary Up/Down/j/k; Chose remains activation/click. Reconcile, missing-cursor seeding, source reorder/removal, external value synchronization and pointer Press must not fabricate Navigated or commit. Emit Navigated at admitted navigation binding branches, or pass explicit navigation origin to the helper: pointer Press currently calls the same move_cursor helper and must not inherit this action accidentally. The shared update algorithm remains the only cursor/key lookup engine. Ordinary callers may ignore Navigated; source-qualified oracle app callers commit its key exactly as they commit Chose, then pass that value consistently to both phases. This matters even when cursor does not move: oracle choice.rs:151–165 assigns selected=cursor on a clamped arrow.

Update exhaustive in-package consumers and conformance.rs:1434 without deleting original test identities. The narrow Form adaptation in this task erases an uncommitted Navigated action via Response::take_action before mapping, retaining flow/invalidation/id/state and default no-navigation-commit behavior; no misleading FormAction::Committed is emitted. TASK-019 supplies the separately configured oracle Form bridge using the same typed event.

Direct API and compile-positive probes cover both variants, ignored navigation retaining controlled value, app-style commit, clamped arrow with selected!=cursor, actual movement, pointer press without commit then completed click, readonly/disabled and inherited disable, empty items, reorder/removal with no input and source change during publication. Assert exact action key/value/cursor and callbacks, not only visual selection. Oracle application owners are Showcase Forms/Settings, TablePro Connections, and Jackin Accounts/Config/choice modals; each caller proves its full trajectory separately.

## Fixed ADJ-15 contract — Select owns optional commit navigation

Add public `SelectNavigation::{Cursor,Commit}` and `SelectOpenKeys::{FocusTraversal,ConsumeUnhandled}` with `navigation(self, policy: SelectNavigation) -> Self` and `open_keys(self, policy: SelectOpenKeys) -> Self`. Keep Cursor/FocusTraversal as defaults, including existing closed cursor-only extended keys and Tab/focus-out dismissal. Preserve SelectState's documented state-owned value and existing update signature. Commit is one policy of the shared engine, not a new action, app-manual setter or duplicate lookup. Copy both configured policies through every constructor/type-changing builder and with_inherited_disabled reconstruction used by Form.

Under Commit, closed plain Up/Left and Down/Right navigate from the committed value's current live index, update cursor and value, and emit existing Chose only when value differs. At a clamped boundary the admitted operation remains Changed with no action/callback. Open Up/k and Down/j move cursor only; Enter/Space commits/closes, Esc restores. Commit-policy same-value open/pointer choice closes and remains Changed without Chose; generic Cursor preserves existing Chose semantics. Empty closed directional input is Changed/no key; empty open Enter/Space closes with Changed/no Chose (Closed records closure). Pointer Press, reconcile, seeding, source changes and external synchronization never commit. Existing live-key reconciliation remains authoritative.

The exact source table is oracle select.rs70–128: Ctrl/Alt characters are ignored before either phase; remaining open Up/k/Down/j and Enter/Space/Esc follow their source guards; other admitted open keys consume under ConsumeUnhandled. Closed Up/Left/Down/Right require modifiers excluding SHIFT to be empty; closed Enter/Space open; closed k/j/Home/End/Page keys do not navigate under Commit. Do not treat all modified non-character keys as ignored. Freeze every combination of existing modifier bits and each named code in W-018-12 against the source, not a hand-picked plain-key example. ConsumeUnhandled only while open and enabled/writable publishes focused Tab/BackTab bindings before runtime traversal; closed or ineligible publishes none. Popover stays Dismiss::ALL and true external focus-out dismisses without restoration. No LayerSpec/runtime changes, app interception or fake focus.

TASK-019 consumes the existing update_in_form response: it already copies Chose(Index) to the controlled usize and Form maps Chose to Committed. No new SelectAction variant, Form policy or key engine is introduced; TASK-018's form/conformance adaptations remain solely Radio Navigated authority. Add only the two enum re-exports in components/mod.rs and lib.rs. Source app composition is navigation(Commit).open_keys(ConsumeUnhandled), through the same configured control in both phases. Defaults and all historical test identities remain independently proved.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_018.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `select`, `conformance`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:checkbox-toggle

- family: checkbox-toggle
- reference_implementation: O:src/widgets/choice.rs
- main_implementation: M:crates/tui/src/components/choice.rs
- architectural_target: Controlled Checkbox and Toggle values
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:choice.rs controlled-value/disabled units;O:tests/choice_containment.rs
- tests_missing: Controlled values never replaced by focus; exact pointer containment all sizes

### COMP:radio-group

- family: radio-group
- reference_implementation: O:src/widgets/choice.rs
- main_implementation: M:crates/tui/src/components/choice.rs
- architectural_target: Borrowed keyed RadioGroup separate cursor and chosen value
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:radio_group_separates_cursor_from_value;radio_choose_action_uses_the_items_stable_key
- tests_missing: Oracle choose/navigation and disabled row containment; CP-COMMON

### COMP:select

- family: select
- reference_implementation: O:src/widgets/select.rs
- main_implementation: M:crates/tui/src/components/select.rs
- architectural_target: Controlled keyed Select with runtime-owned popup
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:select.rs disclosure/popup/controlled-value units;empty_style_inheritance.rs
- tests_missing: Oracle popup open/close Escape focus-out and selected/cursor distinction; all-color geometry

### ARCH:A24

- id: A24
- area: Choice mono affordance
- oracle_state: oracle choice geometry
- main_state: unresolved named bracket contract
- architectural_target: preserve label geometry and oracle states; explicit shared policy
- status: partially refactored
- remaining_obligation: resolve original geometry obligation with oracle evidence
- available_gates: conformance;named test allowlist
- missing_proof: mono_pressed_choice_keeps_the_label_geometry
- evidence: 7b27732:xtask/named_tests_allow.txt:23

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A18

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §9,§29.8; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Modal traps/restores even empty; popover dismisses on focus-out without restoring opener
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Nonempty/zero-area forward/backward traps; same-owner reparenting
- Gates: modal and popover tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:19; global semantic anchor; supplemental clauses retained

### HIST:A42

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §29; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Pressed brackets must use reserved geometry without truncating labels; Choice-specific geometry unresolved
- Disposition: partly_implemented_unresolved_choice; current remaining_work
- Remaining proof: Discharge Choice mono proof without geometry drift or deferral
- Gates: Choice named proof;oracle frames
- Origin: docs/refactoring-plan/historical-obligations.tsv:43; global semantic anchor; supplemental clauses retained

### HIST:A65

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §50; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Radio caller value distinct from cursor; initial seeding only, commit requests caller change
- Disposition: accepted; current retain_and_verify
- Remaining proof: External value change does not retarget established cursor
- Gates: controlled radio tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:66; global semantic anchor; supplemental clauses retained

### HIST:A75

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §71; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Select closed/open glyph roles preserve discriminants, one-cell size and field placement; Set/Clear authoritative
- Disposition: accepted; current compare_new_oracle
- Remaining proof: No closed-field SELECTED contamination; popup chosen exactly once
- Gates: Select marker/mono frames
- Origin: docs/refactoring-plan/historical-obligations.tsv:76; global semantic anchor; supplemental clauses retained

### HIST:A83

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §24 M3; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Form options/value borrowed in one mutable access; standalone choices retain borrowed generic phase data
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: No option copy and phase agreement; compile borrow cases
- Gates: FormData examples
- Origin: docs/refactoring-plan/historical-obligations.tsv:84; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-010

- Source: a3fe79b1 §29.8;11924199 §29.8;15371443 §29.8; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: A popover losing focus dismisses without restoring its opener; modal traps have bidirectional nonempty wrapping proof.
- Disposition: accepted; implementation checkpoint not fresh gate; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: a3fe marked Dialog TRAPS_FOCUS and driver owed;11924199 reports implementation at5d17cc0;15371443 corrects runnable module paths.
- Gates: FocusOut popover/modal differential; both directions of caps/modal correspondence; empty and zero-area traps; Tab/BackTab wrap.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-015

- Source: 3adb6efe §§6,13,17 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: RadioGroup::value(ItemKey) is identical in both phases, seeds only missing cursor during update, and controls checked draw.
- Disposition: accepted; user-visible key semantics governed by oracle; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Draw-only controlled-value exception withdrawn; §29.7 RadioGroup API question closed by§50.
- Gates: Shared props constructor; initial/moved/missing cursor; controlled value changes; keyboard/mouse oracle scenario.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-033

- Source: a1759b2a §20.10 items28–31; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Semantic selection is caller/state-owned; Select disclosure has distinct SelectClosed/SelectOpen glyphs and exact field marker position.
- Disposition: accepted boundary; oracle output governs; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Historical16/28/177-key movements do not authorize new parity drift; no-prompt Dialog reference targets first enabled action only. TASK-023 proves prompt input versus no-prompt first-enabled action including disabled leading action; TASK-031 consumes accepted producer proof without reversing prerequisites.
- Gates: Exactly one semantic chosen/active row; closed/open Select glyph and FIELD.right()-2; prompt target versus action; no root/sibling runtime flags.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-043

- Source: 43a147f0 §§16.1,29;15371443 §§16.1,29.1,29.4; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Choice in-run pressed bracket remains unresolved; Brand/menu reserved-pad proofs do not close Choice.
- Disposition: unresolved historical obligation; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: 15371443 retracts nonexistent choice::mono_pressed_choice_keeps_the_label_geometry; approved shared helper excludes choice.rs.
- Gates: Task must reconcile intended geometry with immutable Choice/RadioGroup app states; proof must execute actual current implementation.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:44; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-017

- Source: 87ab93d4 §24;70dacec1 §29.7;ecc13378 STATE839–843 and§67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Item-bearing choices use direct per-phase paths; Form drives choices through private bridges; inherited-disabled OR configured-disabled; public standalone API unchanged.
- Disposition: decided_with_future_trait_widening_open; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Stale architecture open wording does not mandate widening scalar FieldControl; STATE explicitly decides direct path.
- Gates: Borrowed keyed standalone choice and configured Form examples; dynamic inherited/configured disable; single value_and_options borrow; no unauthorized newtrait.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-033

- Source: 70dacec1 §29 Q1;70dacec1 §§31–32; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Pressed brackets use existing reserved padding; Glyph Slot Clear reserves a blank cell and differs from Inherit; neutral custom family receives observable mono behavior.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: RowUi no-pad lists use container styling; unresolved Choice/Brand applicability cannot be guessed.
- Gates: Clear/Set/Inherit width and cells; label extent preserved; custom family paint not rule counter; Choice/Brand later source adjudication.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-039

- Source: 3ed377e3 §29.8; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Select popover does not trap; FocusOut producer dismisses without restoring opener; modal alone traps; one-stop wrapping tested.
- Disposition: accepted_historical_contract_requires_oracle_join; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old Select Tab swallowing is not generic trap authority; current immutable parity may require source-selected app routing.
- Gates: Tab next target preserved; modal negative case; nonempty actualtrap; exact pinned-app Tab flow.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:40; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-057

- Source: 70dacec1 §29.7;7b27732a §29.7 excerpt; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: RadioGroup caller value and ChipBar Add action were open in Q, later resolved§50; StatusBar hover primitive existed but lacked consumer, later resolved§51.
- Disposition: historical_open_items_superseded; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Do not revive fabricated ItemKey sentinel or claim hovered_part presence was implementation closure.
- Gates: Join middle-reader original§50–51; actual payloadless AddRequested/NEW and keyed hover consumer tests.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:58; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F8

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Nested Select popup owns its layer lifecycle, independent of draw order; Esc reaches control before outer layer.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A17,A19,A80
- Gates: Nested popup closes before dialog; no deferred repaint or re-registration hack; apply later popover focus policy.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:9; refines A17,A19,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-009

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:258-349
- Requirement: Brand/chip/choice use shared presentation and roving/field mechanics
- Disposition: proposal homes; accepted broad dispositions; current unverified
- Remaining proof: Map each old family explicitly; overflowing chip bar remains reachable
- Gates: Full-width overflow; disabled/readonly distinctions; oracle marker/bracket geometry
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-010

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:353-429;634-655;723-745
- Requirement: Completion picker select menus compose overlays/collections; dialog body open
- Disposition: accepted amended; current unverified
- Remaining proof: Functional scrolling and typed actions; retain nontrapping Select final policy
- Gates: Long option list; scrollbar drag; owner focus; nested Esc and click-outside
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-041

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B13/B14;629-646;A4/A11
- Requirement: Noncolor pressed state; cache hints/hash; empty intent fast path; bounded diagnostics; no secret draw allocations
- Disposition: J accepted then container/Id/mono/cache amendments; current unverified
- Remaining proof: Reject review ownSmallVec; replace obsolete debug-Id assumptions; preserve current secret and mono contracts
- Gates: Zero unchanged-focus hint allocations; no lost cache generations; nonempty drain scale; bounded diagnostic drop count; actual pressed bracket cells; no raw secret disclosure
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-014

- Source: fa8adb7b; docs/reviews/adjudication-m-small-items.md M3
- Requirement: Choice controls remain generic per-phase collections; closed FieldKind uses label aliases and FormData options/value_and_options
- Disposition: accepted with later item-channel follow-up; current not independently tested
- Remaining proof: Keep items out of props and handle mutable value/options in one borrow; join later Form adjudication
- Gates: borrow-checking external fixture; changing options without rebuilding props
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-044

- Source: e22ae190; docs/reviews/adjudication-q-residuals.md Q1
- Requirement: Pressed brackets use existing reserved cells; shared implementation; label methods never steal content columns
- Disposition: accepted; Choice/Brand later remain open; current not independently tested
- Remaining proof: Finish surviving Choice/Brand questions with oracle geometry; do not generalize brackets into RowUi labels
- Gates: full-width label/close-cell equality; Tabs bracket-off negative control
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:45; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-047

- Source: ad94d12a; docs/reviews/adjudication-q-residuals.md; STATE
- Requirement: Select opens Popover without focus trap; OVERLAY distinct from TRAPS_FOCUS
- Disposition: accepted Q; contradictory trap proposed and under adjudication at handoff; current not independently tested
- Remaining proof: Join final §33/later ruling; never silently convert popover to modal to satisfy case14
- Gates: outside-focus dismissal; owner focus; pointer barrier; modal-only tab trapping
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:48; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-064

- Source: fa8adb7b; docs/reviews/adjudication-m-small-items.md M3
- Requirement: Closed form label controls have positional value contract; richer keyed/custom-row control through Chooser
- Disposition: accepted historical contract; join laterForm amendments; current not independently tested
- Remaining proof: Do not call positional form choice reorder-stable; app remaps optionindex or chooses keyed external control; no props/state storage crossover
- Gates: Option reorder index mapping; chooser emits fieldId; state Clone+Eq redacting draft; no FieldKind reachable fromstate
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:65; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-FORM

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Configured FieldKind controls keep private update_in_form/draw_in_form; effective disable inherited OR own
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Public form-specific component API; disabled state duplicated inconsistently
- Gates: Dynamic FormData.disabled before mutable value_and_options borrow; chooser same rule
- Origin: docs/refactoring-plan/history-late-obligations.tsv:30; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-71-DISCLOSURE

- Source: a1759b2a §71; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Append SelectClosed/Open39/40; inherited marker only; exact field.right-2; strip field SELECTED, retain live pressed
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Chosen replacing disclosure; moving existing role discriminants; overriding explicit Set/Clear
- Gates: One-cell down/up; popup exactly one chosen marker; field and popup mono states distinct
- Origin: docs/refactoring-plan/history-late-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM04

- Source: 70dacec1 §32.2;15371443 §32.2; COMPONENT_ARCHITECTURE.md
- Requirement: Approved reserved-pad brackets have one shared implementation; role mentions/delegation are allowed; Choice in-run bracket remains separate unresolved geometry
- Disposition: accepted;Choice_unresolved; current partial_explicit_exception
- Remaining proof: Keep helper and byte-stable pads; record oracle-backed Choice disposition without blanket exemption
- Gates: bracket helper AST gate; pressed vs focus mono geometry
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM56

- Source: 3adb6efe §50.3; COMPONENT_ARCHITECTURE.md
- Requirement: RadioGroup immutable value prop in both phases; state owns navigation only; missing cursor may seed from value, existing cursor not retargeted by external value
- Disposition: accepted; current retain_verify
- Remaining proof: Caller rebuild commits Chose; missing/vanished value and reordered keys preserve ownership
- Gates: external value changes vs established cursor; chosen paint; no hidden mutable value
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:57; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM71

- Source: ecc13378 state:832;architecture §30;later §67; REFACTORING_STATE.md;COMPONENT_ARCHITECTURE.md
- Requirement: Scalar FieldControl cannot carry per-phase items; item-bearing choices use direct per-phase paths and Form configured bridges; item-aware trait widening remains open
- Disposition: accepted_current_path;future_widening_deferred; current preserve_accepted_bridge
- Remaining proof: Do not mistake stale §30 unresolved shorthand for requirement to widen trait; retain configured options with current public behavior
- Gates: Form choice item/disabled forwarding and controlled value parity
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:72; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG34

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collection cursor differs from chosen value; relevant single/multiple/range selection remains explicit
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; Choice/Picker/Grid
- Gates: Independent cursor/value/check/anchor mutation tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
