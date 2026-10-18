# TASK-015 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Thread borrowed explicit part overrides through RowUi::part, ColumnsUi and CellUi, not only label_patch. Preserve component container/label forwarding while explicit row/cell/custom-part patches override only their intended cells. Distinguish Clear reserved blank from Inherit, label base inheritance and glyph roles.

Caller keyed sources own data; collection caches own only derived identity/projection. Keep row-author provenance distinct from same-ID component composition through callbacks; no fabricated owner ID. No Display/Clone/static requirement for borrowed renderers or full-dataset clone in measured work.

Sentinel actual-cell tests cross inherited/family/variant/subtree/instance/part/cell/slot precedence, siblings, clipped columns and mono. Reorder/insert/delete duplicate labels and mutate revision between publication/input. Test non-Clone borrowed model, all column limits and visible-only callback counts at 100k rows.

## Fixed re-audit contract — shared styled-run consumer

RowUi::spans, CellUi and any styled column path delegate cross-fragment segmentation and painting to the TASK-013 logical-grapheme source/painter contract. Explicit part/cell overrides layer over the first-byte fragment provenance without splitting a cluster or changing geometry. Named direct cases use differently styled e/combining-acute and emoji-ZWJ fragments across borrowed row/column callbacks, clipped wide cells and custom part patches; compare exact cells, source boundaries and unchanged sibling/hit geometry. Reject a local per-span painter and a patch applied only to a discarded zero-width suffix.

## Fixed branch re-audit contract — formatted row consumer

TASK-013's single-pass streaming painter is mandatory for RowUi::label_fmt and the default Display row path as well as styled spans. CellUi forwards the same path; no local CellWriter per-fragment paint loop survives. Preserve public fmt::Arguments/Display semantics, one original Display invocation per actual visible-row callback, borrowed non-static models and all row/column part/cell overrides. The public consumer does not allocate its own scratch, pre-format a row or invoke Display again to measure it; the existing private Ui/FrameState owner from TASK-013 handles reusable cluster capacity.

Use a custom Display writing e then acute, every UTF-8 split of family-ZWJ/RI/CJK/control fixtures, empty writes, ephemeral fragment buffers and a+256/4096 acute+z. Compare actual List/default-row and custom RowUi/CellUi column cells against the same unsplit logical input at widths0..120/nonzero origins, including a wide cluster that does not fit followed by a narrow suffix. First-byte style/provenance, clipping and wide-shadow cleanup match; the suffix must not jump back into a rejected cluster's cell. Count original Display calls and formatter side effects on full/clipped/empty callback paths. A fmt::Error preserves its written prefix; unwind clears scratch. Tests that omit callback invocation because the allocation is empty must report that fact rather than claiming one invocation occurred.

Retain all inherited allocation gates; add actual cold/warm/fresh-Runtime scratch/output/total counters using TASK-013's exact finite fixtures and ceilings. A 25-byte family-ZWJ or long combining cell can allocate output storage even when scratch is warm zero. Reject per-fragment segmentation, two formatting passes, arbitrary cluster caps, whole-row materialization, post-clip scratch growth, quadratic repeated UTF-8 validation and per-row scratch allocation disguised by prewarming. TASK-015 closes these actual public consumer cases now; no later app/021/031 receipt is its prerequisite.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_015.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `item_row_columns`, `rowui_glyph_contract`, `list_row_renderer`, `tree_row_renderer`, `grid_cell_renderer`, `perf_collections`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:collection-core

- family: collection-core
- reference_implementation: O:src/widgets/list.rs;tree.rs;table.rs;grid.rs independent owned collections
- main_implementation: M:crates/tui/src/collection/*
- architectural_target: Borrowed keyed sources; shared reconciliation; constrained RowUi CellUi
- visual_status: CP-06 explicit row-part override channel incomplete
- interaction_status: source identity/reorder proof incomplete
- api_refactor_status: implemented core; row override migration required
- tests_available: M:perf_collections.rs;list_row_renderer.rs;tree_row_renderer.rs;grid_cell_renderer.rs
- tests_missing: CP-06 custom part/column actual-cell precedence; every dynamic caller stable keys; visible-only callbacks; source mutation invalidation

### COMP:identity

- family: identity
- reference_implementation: O:src/core/id.rs
- main_implementation: M:crates/tui/src/id.rs;collection/key.rs
- architectural_target: Stable Id ItemKey ColumnKey and typed PartRef
- visual_status: unverified
- interaction_status: unverified under reorder
- api_refactor_status: implemented; positional custom author example remains
- tests_available: M:grid_keyed_cursor.rs; component keyed tests
- tests_missing: External author insert/delete/reorder duplicate labels and stable semantic identity

### ARCH:A04

- id: A04
- area: identity
- oracle_state: WidgetId positional children and locate
- main_state: Id ItemKey Part PartRef
- architectural_target: stable keys across filtering/reordering/removal; no reverse hit scans
- status: implemented;parity unproven
- remaining_obligation: prove item/focus/selection identity on all restored app models
- available_gates: grid_keyed_cursor;tree_state_replacement;tablepro row_identity;no_owns_or_locate_in_applications
- missing_proof: golden dynamic collection scenarios
- evidence: 7b27732:crates/tui/src/lib.rs:51;7b27732:COMPONENT_ARCHITECTURE.md:499

### ARCH:A17

- id: A17
- area: collections
- oracle_state: owned cloned items positional callbacks
- main_state: borrowed CollectionCore RowUi KeySet
- architectural_target: borrowed keyed O(visible) rendering selection reconciliation
- status: implemented
- remaining_obligation: keep reusable models during application migration
- available_gates: perf_collections;list_boundary;tree_row_renderer;picker_projection
- missing_proof: model-read and allocation probes for restored large data
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1179;7b27732:crates/tui/src/collection/mod.rs:1

### ARCH:A18

- id: A18
- area: row/column instance overrides
- oracle_state: inconsistent component-specific painting
- main_state: RowUi/ColumnsUi retain label patch only
- architectural_target: all declared row parts honor instance/part/slot precedence
- status: partially refactored
- remaining_obligation: thread borrowed PartStyle to part/columns painters and consumers
- available_gates: item_row_columns;rowui_glyph_contract;overrides
- missing_proof: independent part-patch precedence red-proof across consumers
- evidence: 7b27732:crates/tui/src/collection/rowui.rs:29;7b27732:crates/tui/src/collection/rowui.rs:314

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A06

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §7 / B,J,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Stable namespaced ItemKey identity independent of visible index
- Disposition: accepted; current retain_and_verify
- Remaining proof: Reorder/insert/delete/filter/sort must not retarget selection/edit/close
- Gates: identity property tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:7; global semantic anchor; supplemental clauses retained

### HIST:A28

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §10,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: RowUi column limit is explicit and cannot silently remove required oracle content
- Disposition: accepted_with_limit; current verify_scope
- Remaining proof: Account every consumer exceeding or approaching limit
- Gates: external columns fixture
- Origin: docs/refactoring-plan/historical-obligations.tsv:29; global semantic anchor; supplemental clauses retained

### HIST:A30

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Span rendering inherits label base style without intermediate string/vector allocations
- Disposition: accepted; current retain_and_verify
- Remaining proof: Styled-span segmentation agrees with complete logical text
- Gates: span differential;500x3 zero alloc
- Origin: docs/refactoring-plan/historical-obligations.tsv:31; global semantic anchor; supplemental clauses retained

### HIST:A44

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §33,§50; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Row-owned parts separate from component-owned container/label forwarding; explicit row patch wins
- Disposition: accepted; current remaining_work
- Remaining proof: Repair actual part forwarding and independent style isolation
- Gates: item_row_columns;explicit_columns_styles_win
- Origin: docs/refactoring-plan/historical-obligations.tsv:45; global semantic anchor; supplemental clauses retained

### HIST:A129

- Source: db5c41a6:GOAL2.md;7b27732a8c3c131760ec3438f641cb3c11343a42:docs/audit/consolidation/disposition-ledger.md:166-167,175,254-255,259,468,476-477; GOAL2.md;docs/audit/consolidation/disposition-ledger.md
- Requirement: Rejected stale item-row, manager, Grid/progress patches do not waive their underlying capabilities
- Disposition: accepted_disposition; current remaining_work_by_matrix
- Remaining proof: Reimplement accepted behavior with current API and oracle evidence
- Gates: component/app task ownership
- Origin: docs/refactoring-plan/historical-obligations.tsv:130; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-007

- Source: 6ec29171 §§16.1,29.7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Regression proof exercises Clear geometry, keyed strip rendering, both cell boundaries, and actual ASCII glyph sets.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Clear reserves a blank cell; same buggy enumeration or non-overflowing fixture is insufficient; no-box-drawing is weaker than all-ASCII.
- Gates: Clear/Inherit differential; overflowing reorder; left/right out-of-area rejection; full typed GlyphSet ASCII scan.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-017

- Source: a1759b2a §20.10 item23;3adb6efe §29.7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: ChipBar checked membership uses canonical Checked in its reserved cell; component patches preserve caller META and owned geometry.
- Disposition: accepted; oracle output controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: CHECKED replaces synthesized SELECTED; automatic CONTAINER/LABEL patches do not overwrite caller-owned row styling.
- Gates: Set/Inherit/Clear marker; trailing META; close/pad/overflow positions; no CheckboxOn clipping or label truncation.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-002

- Source: 95ab6529 §21 items1–2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Configuration props contain no live item slice; item/model borrows are supplied per phase; generic key/row builders preserve inference through distinct defaults.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Replaces initial retained slice examples that caused E0502.
- Gates: Borrowed non-static rows; mutate model between phases; captured closures; standalone generic Select/List examples compile.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:3; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-033

- Source: 70dacec1 §29 Q1;70dacec1 §§31–32; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Pressed brackets use existing reserved padding; Glyph Slot Clear reserves a blank cell and differs from Inherit; neutral custom family receives observable mono behavior.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: RowUi no-pad lists use container styling; unresolved Choice/Brand applicability cannot be guessed.
- Gates: Clear/Set/Inherit width and cells; label extent preserved; custom family paint not rule counter; Choice/Brand later source adjudication.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-055

- Source: 2e453023 §12.5 J10–J13; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Shared RowDecor/change slots, keyed tabs, meter threshold helper and modal stack/result routing replace duplicates without moving product policy into library.
- Disposition: accepted_with_later_meter_policy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Quota lifecycle remains application-owned; runtime feedback and typed actions replace ad hoc UI state.
- Gates: J10–J13 keyed row action, tab reorder, thresholdboundary, layerresult tests with pinned copy/cadence.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:56; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-060

- Source: 69fcdcad §10;587c53bd §17examples9–10; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: TrackAuto without measurement takes1cell besideFlex or equal remainder withoutFlex; measuredvariants use natural sizes; zeroalloc distribute_into serves RowUi columns with16track cap.
- Disposition: accepted_historical_geometry_contract; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Do not change sourceobservable layout silently under PLANNING parity; contract names cannot prove examples use measuredvalues.
- Gates: Auto/Flex/fixed/multipleAuto/tiny matrix; measured themedglyph widths; example9two-rowProps notclipped; columns cap and zeroalloc.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:61; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-033

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B3/B5/B6/B15;732-735
- Requirement: Phase-call data; derived-only runtime caches; post-update Esc
- Disposition: J accepted; K amends grid model split; current unverified
- Remaining proof: Join latest model/cache/ordering contracts to actual callers
- Gates: Borrow-compiling mutation closure; shared draw; derived-cache structural and invalidation proof; editor cancel before dismiss
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-002

- Source: e81ca17b; docs/audit/interaction-audit.md B1
- Requirement: Separated and kind-tagged Id plus stable ItemKey; typed part routing
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Stable logical identity under insert/remove/reorder; no owns/locate inversion
- Gates: collision-kind corpus; actual click/close key after reorder; negative controls
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:3; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-021

- Source: e81ca17b; docs/audit/domain-boundary-audit.md4.1 J1-J13
- Requirement: Reusable form/choice/info/help/wizard/picker-chain/row-decoration facilities replace duplicated app plumbing
- Disposition: accepted dispositions; exact APIs require architecture join; current not independently tested
- Remaining proof: Every J item needs explicit disposition; do not promote file-system or account semantics to library
- Gates: showcase coverage for new public facilities; wizard rewind/drafts; custom rows; domain boundary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-022

- Source: a156054d; docs/audit/performance-audit.md6 R1-R7
- Requirement: Borrow data, window drawing, cheap reconciliation, sorted rules, nonallocating hot paths
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Join exact later thresholds; cheap generation heuristic cannot hide arbitrary in-place data changes
- Gates: 100k vs1k scaling; explicit invalidation; real action dispatch; zero allocation style path
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-032

- Source: e2be0ced; docs/reviews/slice2-architecture-review.md; STATE
- Requirement: Data moves to phase calls; derived Ui cache; frozen intent queue outlives Cx borrow; editor receives Esc before layer
- Disposition: accepted J amendments; current not independently tested
- Remaining proof: Keep borrowed mutation ergonomic and cache nonsemantic; never replay original input during focus settling
- Gates: external close/reorder fixture; cache derivation; Esc editor/modal sequence; exactly-once actions
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:33; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-044

- Source: e22ae190; docs/reviews/adjudication-q-residuals.md Q1
- Requirement: Pressed brackets use existing reserved cells; shared implementation; label methods never steal content columns
- Disposition: accepted; Choice/Brand later remain open; current not independently tested
- Remaining proof: Finish surviving Choice/Brand questions with oracle geometry; do not generalize brackets into RowUi labels
- Gates: full-width label/close-cell equality; Tabs bracket-off negative control
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:45; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-045

- Source: e22ae190; docs/reviews/adjudication-q-residuals.md Q1/A4
- Requirement: Resolved and PartMetrics glyph use Slot Inherit/Set/Clear; cell-owning RowUi honors distinct outcomes
- Disposition: accepted then incompletely implemented at early checkpoint; current not independently tested
- Remaining proof: Test each method separately; current Q says fixed, but verify pinned source
- Gates: Inherit/Set/Clear caller matrix; reserved geometry; existing examples07/08
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:46; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-57-PARTS

- Source: 14bca4a3 §57; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: NavList patches CONTAINER/GUTTER/MARKER/ICON/LABEL/HEADER/BADGE; slots GUTTER/MARKER/ICON/HEADER/BADGE; RowUi carries only CONTAINER/LABEL
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Generic row-part patch leakage; implicit BADGE row ownership
- Gates: Badge budget and slot replacement; full/collapsed grouping exact separators/text
- Origin: docs/refactoring-plan/history-late-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-58-PARTS

- Source: 14bca4a3 §58; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Steps exact parts/slots; lifecycle META direct-owned; optional row META row-owned; scroll all overrides
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Forwarding arbitrary RowUi patches
- Gates: Owner isolation; lifecycle metadata space; track/thumb forwarding
- Origin: docs/refactoring-plan/history-late-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM03

- Source: 70dacec1 §32.3; COMPONENT_ARCHITECTURE.md
- Requirement: RowUi cell-owning glyph methods distinguish Clear blank reserved width from Inherit no automatic glyph
- Disposition: accepted; current retain_verify
- Remaining proof: Pin marker/gutter/part plus wide glyph geometry; label methods retain their separate contract
- Gates: Slot Set/Clear/Inherit visual width and adjacent-cell proof
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:4; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM04

- Source: 70dacec1 §32.2;15371443 §32.2; COMPONENT_ARCHITECTURE.md
- Requirement: Approved reserved-pad brackets have one shared implementation; role mentions/delegation are allowed; Choice in-run bracket remains separate unresolved geometry
- Disposition: accepted;Choice_unresolved; current partial_explicit_exception
- Remaining proof: Keep helper and byte-stable pads; record oracle-backed Choice disposition without blanket exemption
- Gates: bracket helper AST gate; pressed vs focus mono geometry
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM50

- Source: de817f26 §45.1/7;§50 followup; COMPONENT_ARCHITECTURE.md
- Requirement: Nested same-id components forward component patches/part patches/slots; Chip RowUi forwards only container/label merged patch and explicit label_patched wins
- Disposition: accepted; current retain_and_close_gaps
- Remaining proof: No whole Overrides/generic callback propagation into arbitrary row-owned parts; distinguish patch source from resolution owner
- Gates: component instance sibling isolation; META/CELL/custom unaffected; nested TRACK/THUMB substitution
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:51; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG14

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Borrowed inputs, clear ownership, no unnecessary static bound/cloning/generics
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-core; collection API rows
- Gates: Borrowed non-Clone/non-Display domain fixtures
- Origin: docs/refactoring-plan/history-other-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG20

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Nested/repeated component parts have stable collision-safe identity and readable debugging
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Id/Part/Scope; runtime target registry
- Gates: Duplicate-identity rejection and stable child-source tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG33

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collections support borrowed domain content, custom rows/cells, metadata and relevant empty/loading/error states
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; ItemSource/AsItem/row adapters
- Gates: Non-owned consumer fixtures and visible empty/loading states
- Origin: docs/refactoring-plan/history-other-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
