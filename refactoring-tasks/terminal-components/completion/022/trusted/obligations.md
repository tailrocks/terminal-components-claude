# TASK-022 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

Retain accepted generic Grid behavior by default and add one finite source-qualified gesture policy consumed by the same shared keyed update/editor path: modern default, DataGrid, DataTable row mode and DataTable cell mode. DataGrid different-cell completed click moves; current unranged globally noneditable Grid emits Activated, while globally editable Grid invokes begin_edit: column read_only or pending-deleted row returns Consumed with no action/edit/mutation, otherwise the source inline/cycle/external result applies. DataTable row mode completed click selects and activates the clicked source row, including a different row. DataTable cell mode different-cell click moves; a current editable column begins editing, but a current noneditable column produces no activation. Existing editor-click placement and failed commit behavior remain source-qualified. W-022-01 applies specifically to DataGrid; W-022-09 proves the four policies without application-side suppression or synthesized action. Freeze gesture-start keyed identity across Press/Click/DoubleClick, reorder and release-outside; do not let press-driven cursor movement manufacture a same-current-cell click. Preserve W-022-08 partial-column locate guards and W-022-07 accepted readonly mutation guards; this policy neither restores F04 unchecked writes nor invents app-level viewers.

Registered geometry is not sufficient click applicability: source-witnesses.md W-022-08 binds oracle partial-cell routing at all four source sizes. Reusable Grid owns this source-qualified guard and preserves ordinary valid current-cell activation; TASK-060 must not duplicate or suppress generic hit routing in an application shim. The exact source notes fixture remains a viewer/nonactivation witness, not an invented inline-edit state.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

Use the explicit generic-default/DataGrid/DataTable-row/DataTable-cell policy in D-BRANCH-SOURCE and W-022-09; specifically DataGrid different-cell click moves; current unranged globally noneditable Grid activates, while globally editable Grid enters the source begin_edit branch, which refuses read_only columns or pending-deleted rows without activation, while DataTable row and cell modes retain their distinct source outcomes. Press alone and release outside do not activate. Preserve every twenty-two TablePro capability through generic model/editor interfaces: row/cell modes, sorting requests, reference action, row-number selection, validation, fetch-more, scrolling and editor placement.

Only GridEditor/update_editable mutates caller model; three-method GridModel remains readonly. App adapter owns SQL/NULL/default/pending/undo/comparator/permutation. Ragged None cells never call decoration/action/editor hooks; state exposes owned semantics not derived windows.

Test initially current/different/ranged/locked/readonly cells, inline/cycle/external/refused editing, active editor caret, validation preventing navigation and model reorder between publication and action. Exact header/gutter/density/width/scrollbar/fade cells survive both themes/all capabilities and resize; custom cell sentinels reach paint.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_022.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `grid_model_contract`, `grid_keyed_cursor`, `grid_cell_renderer`, `grid_header_geometry`, `grid_column_fit`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:grid

- family: grid
- reference_implementation: O:src/widgets/grid.rs
- main_implementation: M:crates/tui/src/components/grid.rs
- architectural_target: Borrowed GridModel/GridEditor keyed Grid with constrained GridCell painter
- visual_status: missing fade; geometry parity unverified
- interaction_status: CP-03 current-cell completed click differs
- api_refactor_status: implemented broad API; activation parity incomplete
- tests_available: M:grid_* tests;custom GridCell;ragged models;keyed cursor;readonly compile-fail
- tests_missing: CP-GRID-EDIT; sort identity; reference action; validation error; scroll/fetch/resize exact cells

### COMP:table

- family: table
- reference_implementation: O:src/widgets/table.rs
- main_implementation: M:crates/tui/src/components/grid.rs NavUnit::Row
- architectural_target: DataTable removed; Grid row mode and caller-owned table adapter
- visual_status: missing fade; table density/header/gutters unverified
- interaction_status: unverified row activation/sort/edit behavior
- api_refactor_status: absorbed into Grid; app adoption must preserve table semantics
- tests_available: M:grid_header_geometry.rs;grid_detailed_gutter.rs;grid_column_fit.rs;TablePro app tests
- tests_missing: Oracle simple versus rich rows sorting current selection metadata widths all modes

### ARCH:A19

- id: A19
- area: Grid domain split
- oracle_state: DB data/pending/undo inside widget plus DataTable
- main_state: GridModel shared;GridEditor mutable;TablePro adapter
- architectural_target: generic grid; database policies stay TablePro; no editable bool
- status: implemented;behavior defects
- remaining_obligation: preserve all 22 TablePro capabilities and oracle single-click edit
- available_gates: grid_model_public_surface_is_exact;grid_model_contract;TablePro model tests
- missing_proof: oracle pending/undo/row/column/header/edit scenarios
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1314;7b27732:apps/tablepro/src/grid_model.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A53

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §19; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: No permanent legacy facade or duplicate DataTable/ScrollPanel; preserve their capabilities through Grid/Viewport
- Disposition: accepted_rejected_alternatives; current retain_and_verify
- Remaining proof: Remove duplicate mechanisms only after oracle coverage
- Gates: API inventory;behavior proofs
- Origin: docs/refactoring-plan/historical-obligations.tsv:54; global semantic anchor; supplemental clauses retained

### HIST:A54

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §14,§52; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Library owns generic Grid mechanics; adapters own SQL/domain ordering/null/default/undo/effects
- Disposition: accepted; current retain_and_verify
- Remaining proof: Domain scanner and actual adapter workflows
- Gates: domain boundary;22-capability map
- Origin: docs/refactoring-plan/historical-obligations.tsv:55; global semantic anchor; supplemental clauses retained

### HIST:A55

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §23 K2; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Grid update borrows read-only model; update_editable requires mutable editor; shared navigation path
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Read-only compile-fail, one commit/error, no duplicate navigation
- Gates: Grid external consumers
- Origin: docs/refactoring-plan/historical-obligations.tsv:56; global semantic anchor; supplemental clauses retained

### HIST:A56

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §52; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Grid cursor/selection/edit state exposes only owned values, not geometry-derived full column window
- Disposition: accepted; current retain_and_verify
- Remaining proof: Public API and stable-key reorder coverage
- Gates: Grid API tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:57; global semantic anchor; supplemental clauses retained

### HIST:A57

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §61; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Grid columns sole schema; Option cell holes never call invalid decoration/action/editor hooks
- Disposition: accepted; current retain_and_verify
- Remaining proof: Three-method external model; ragged copy/navigation/edit
- Gates: ragged grid tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:58; global semantic anchor; supplemental clauses retained

### HIST:A58

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §64:8310-8315 historical gesture text; current UI authority 02f5294bfdbf38004cc49130d0aff1d01f31434c:src/widgets/grid.rs:1225-1235;1294-1346 supersedes the universal single-move/double-activate product rule; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Grid gesture metadata and logical row identity retained; DataGrid-policy current-cell completed click calls begin_edit when globally editable (read_only column or pending-deleted row refuses without action/edit/mutation), or emits Activated when globally noneditable; different cell moves cursor without activation; modern-default and DataTable policies remain distinct; no synthetic semantic selection/edit flags
- Disposition: accepted_with_oracle_update; current compare_new_oracle
- Remaining proof: Freeze routed current-cell versus different-cell, editable versus read-only, active-range-anchor and partial/non-routable geometry cases. With no active edit, no anchor and no trailing-reference interception, one completed current-cell DataGrid-policy click invokes begin_edit when globally editable; a read_only column or pending-deleted row returns Consumed with no action/edit/mutation. Only globally noneditable Grid emits GridEvent::Activated(source_row); another cell moves cursor without activation. A registered but locate-excluded partial cell remains ignored. Retain keyboard and source-key identity proof; do not restore universal double-click prerequisite or F04 unchecked mutation
- Gates: source-qualified DataGrid current/different cell x global editability x read_only-column/pending-deleted-row action/state matrix; partial-cell no-op and anchor/reference/editor priority branches; exact four-size geometry and keyed reorder witness
- Origin: docs/refactoring-plan/historical-obligations.tsv:59; global semantic anchor; supplemental clauses retained

### HIST:A59

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §65; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Grid loading/error is empty/local fetch/decor state; no whole-grid status broadcasting
- Disposition: accepted; current retain_and_verify
- Remaining proof: Loaded cells preserve presentation while fetch row changes
- Gates: empty/fetch/error states
- Origin: docs/refactoring-plan/historical-obligations.tsv:60; global semantic anchor; supplemental clauses retained

### HIST:A74

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §69; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Pressed PartRef styles only actual Grid row/cell/action or ScrollRegion thumb
- Disposition: accepted; current retain_and_verify
- Remaining proof: Sibling and whole-container press isolation
- Gates: pressed-part tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:75; global semantic anchor; supplemental clauses retained

### HIST:F04

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f04-reconcile-datagrid-read-only-transitions-before-mutation.md
- Requirement: Recheck grid/column eligibility at every edit mutation and commit boundary
- Disposition: deferred_not_selected_oracle_preserved; current historical_proposal_not_current_product_gate
- Remaining proof: Deferred proposal, not selected by immutable-oracle parity. Archive legacy direct DataGrid defect evidence at oracle02f5294b grid.rs542,593-619,1417-1424; do not restore unchecked mutation in modern Grid APIs. Retain accepted main entry-point/readonly guards. TablePro editable assignments tabs.rs395/2069 initialize new grids; no app mid-edit permission transition established. Any app compatibility exception requires an actual source-qualified reachable trajectory
- Gates: Source-qualified disposition against exact current oracle; no proposed-behavior PASS claim
- Origin: docs/refactoring-plan/historical-obligations.tsv:135; global semantic anchor; supplemental clauses retained

### HIST:F06

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f06-preserve-datagrid-cursor-identity-through-local-sort.md
- Requirement: Grid display sort preserves logical cursor/edit/selection identity
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: Sort changes order without retargeting row state
- Gates: Grid sort identity
- Origin: docs/refactoring-plan/historical-obligations.tsv:137; global semantic anchor; supplemental clauses retained

### HIST:F07

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f07-checked-row-schema-ingestion.md
- Requirement: Checked schema/row ingestion handles ragged or invalid data consistently
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: No panic or invalid cell dispatch on malformed input
- Gates: Grid ingestion
- Origin: docs/refactoring-plan/historical-obligations.tsv:138; global semantic anchor; supplemental clauses retained

### HIST:F18

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f18-distinguish-read-only-from-disabled.md
- Requirement: Read-only interaction remains distinct from disabled
- Disposition: deferred_fixture_correction_not_selected_shared_contract_retained; current historical_proposal_not_current_product_gate
- Remaining proof: Keep modern disabled versus readonly API distinction; preserve Showcase TextAreas original Read-only transcript label with disabled dim inert unfocusable configuration; no relabel replacement or new navigation from deferred F18
- Gates: Separate generic readonly navigation/copy/nonmutation proof from exact source disabled fixture frames and no-focus observations
- Origin: docs/refactoring-plan/historical-obligations.tsv:152; global semantic anchor; supplemental clauses retained

### HIST:O04

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; docs/improvements-plan-reference.md
- Requirement: Matrix paste
- Disposition: conditional_not_selected; current current_main_mapping_required
- Remaining proof: No extra product feature unless required to reproduce oracle
- Gates: scope disposition
- Origin: docs/refactoring-plan/historical-obligations.tsv:192; global semantic anchor; supplemental clauses retained

### HIST:P01

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; git commits 53b8212f;e4866ce4;92d91629;02f5294b
- Requirement: One click focuses and starts text editing at pointer; Grid cells click-to-edit
- Disposition: oracle_behavior_binding; current current_main_mapping_required
- Remaining proof: 53b8212f/e4866ce4 implementation must be expressed through main runtime
- Gates: all field click flows
- Origin: docs/refactoring-plan/historical-obligations.tsv:195; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-005

- Source: 8ee99771 §16.5; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid capability rejection checks syntax, not whitespace-sensitive text; a missing due Grid source cannot count as implementation proof.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: syn rejects editable signatures including generic/multiline forms and GridCellActions in inline modules; historically missing Grid returned vacuous success.
- Gates: Negative syntax fixtures, source-presence guard, shared GridModel update and separate GridEditor path compile checks.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-023

- Source: a1759b2a §§12.3,20.9 item11,23; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: GridModel has three required methods; cell returns Option<CellRef> for ragged holes; CellRef align is Option<Align>.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Associated Row type removed; None align inherits Column; None cell differs from an empty present string.
- Gates: Read-only downstream model compiles; ragged holes versus empty cells; explicit/inherited alignment; preformatted load allocation gate.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-051

- Source: aacb7cb3 §17 A7;30924105 §17 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid supports optional detailed gutter, trailing reserve, part defaults and keyed semantic header prefixes through reusable geometry.
- Disposition: accepted optional API; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Compact default retains ROW two-cell paint;CHANGE/ROW_NUMBER parts34/35;ICON prefixes are real paint surfaces, never whole-grid Status.
- Gates: Compact parity;clipped four-slot replacement;source numbering;reserve;ColumnKey reorder;ICON explicit override precedence;HEADER slot replaces whole header.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:52; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-054

- Source: c3b51b96 §20.10 item35; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Empty List/Grid/FilterList/Picker regions inherit owning EMPTY style; NavList hover must lift only hovered row.
- Disposition: accepted reusable ownership; oracle output controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: EmptyState::draw_inherited and hovered-difference masks remove blank/gap/header tint bug class;historical20-key classification style-only.
- Gates: Canonical cells including blanks;empty full rect;hover header/gap/other-row invariance;exact text/geometry unchanged.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-018

- Source: 27bd918e §23 K2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: GridModel-only path supports navigation/read-only reason/cell actions; mutation is reachable only through update_editable with GridEditor.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Remove editable bool and separate GridCellActions trait; do not make base model mutable.
- Gates: Compile-fail readonly mutation; successful and rejected edit commit; readonly affordance click emits action.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:19; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-019

- Source: 2e453023 §12.3; e8d053c9 AppendixA Slice6;27bd918e §23; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Keep all22 DOM grid capabilities through library+TablePro adapter; SQL values, pending changes, undo, validation and preview belong to adapter.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Generic extraction is not capability deletion.
- Gates: 22-row original DOM mapping joined to current code and behavior tests.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-058

- Source: aacb7cb3 A7;30924105 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid detailed gutter adds real optional GUTTER/MARKER/CHANGE/ROW_NUMBER, rightreserve and partdefaults; keyed header ICON prefix optional and subordinate to explicit overrides.
- Disposition: accepted_later_extension; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Original ninepart rendering preserved bydefault; ICON not wholegrid readiness/status.
- Gates: Clipped each guttercell; stable source rownumber; reserved contentrect; HEADERslot replaces all; percolumn key reorder.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:59; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-013

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:476-511;798-817
- Requirement: One generic grid; SQL values pending undo validation commit belong in TablePro
- Disposition: accepted amended K; current unverified
- Remaining proof: Read-only model needs no editor trait; preserve all domain operations
- Gates: Edit/cycle/validate/sort/filter/FK/preview/save/undo plus readonly compile fixture
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:14; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-009

- Source: e81ca17b; docs/audit/domain-boundary-audit.md1.5-1.6
- Requirement: One generic Grid; database values pending changes SQL validation commit undo in TablePro
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Retain all22 listed TablePro capabilities through generic actions/model hooks
- Gates: edit/cycle/external viewer; sort/filter/FK; pending/undo/discard/preview/save; no SQL vocabulary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-010

- Source: f2d30b65; docs/reviews/adjudication-k-form-grid.md; STATE
- Requirement: Grid read-only update requires GridModel and shared model; editable update requires GridEditor and mutable model
- Disposition: accepted; earlier universal editable bound rejected; current not independently tested
- Remaining proof: No fake editor adapter for read-only model; actions/read_only_reason on GridModel
- Gates: external read-only model compiles without GridEditor; editable lifecycle tests
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-011

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.12
- Requirement: Delete DataTable and absorb it into generic Grid
- Disposition: accepted; current not independently tested
- Remaining proof: Preserve six Structure sections and distinct oracle sort/hover behavior through model/configuration
- Gates: Structure header sort and all sections; no duplicate table engine
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:12; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-61-SCHEMA

- Source: a1759b2a §61; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid columns sole schema; GridModel requires row_count/row_key/cell; cell Option for structural holes
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Associated Row; col_count; deprecated shims
- Gates: External three-method compile witness; AST exact trait gate
- Origin: docs/refactoring-plan/history-late-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-61-RAGGED

- Source: a1759b2a §61; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Missing cells draw/copy empty with rectangular cursor but never decor/actions/editor hooks
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Sentinel text for missing cells; invalid hook calls
- Gates: Panic-on-invalid ragged hooks across draw/measure/copy/navigation/sort/edit; MAX_COLUMNS bounds; empty TSV fields
- Origin: docs/refactoring-plan/history-late-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-61-ALIGN

- Source: a1759b2a §61; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: CellRef.align Option<Align>; None inherits Column, Some explicit
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Align::Left inheritance sentinel
- Gates: All inherited/explicit alignment cases including explicit Left overriding right column
- Origin: docs/refactoring-plan/history-late-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-61-NUMBER

- Source: c62ec8d3 source diff; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Detailed gutter source number belongs in RowDecor.number; frozen GridModel trait restored
- Disposition: implemented-correction; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Temporary GridModel::row_number extra method
- Gates: Retain exact trait witness and keyed sorted source-number display proof
- Origin: docs/refactoring-plan/history-late-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-64-GESTURE

- Source: a1759b2a §64; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Conformance PointerGesture default Click; Grid DoubleClick; single click Moved, double/Enter Activated key
- Disposition: accepted_mechanism_product_gesture_superseded_by_current_oracle; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: For the DataGrid policy only, oracle02f5294b grid.rs1322-1344/542-578 supersedes the historical product gesture: current-cell completed click calls begin_edit when globally editable, whose read_only-column/pending-deleted-row guards refuse with Consumed and no action/edit/mutation; globally noneditable Grid activates; different-cell click moves. Retain gesture metadata, keyed identity/selection separation and accepted modern default; DataTable row/cell policies are independently source-qualified, not overridden by this DataGrid rule
- Gates: DataGrid current/different cell x globally editable/noneditable x read_only-column/pending-deleted-row isolated trajectories, keyboard, cancellation and reorder; retain modern-default/DataTable policy controls and actual gesture metadata
- Origin: docs/refactoring-plan/history-late-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-64-CLIP

- Source: a1759b2a §64; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid zero remaining column width means no show/register; intersect cell rects; reject clipped column_at
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Oversized min widths escaping body
- Gates: Nonzero 2x2/3x2 buffers; exact region containment; update/draw geometry agreement
- Origin: docs/refactoring-plan/history-late-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-65-LOCAL

- Source: a1759b2a §65; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Grid empty Loading/Error and row/cell decor readiness; no global status propagation
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Grid.status(Status); whole-grid BUSY/ERROR; invent ICON for global readiness
- Gates: Zero-row affordances; real decorated ERROR mono underline; ready output unchanged
- Origin: docs/refactoring-plan/history-late-obligations.tsv:25; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-65-PARTS

- Source: 30924105 §65; aacb7cb3 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Detailed gutter adds GUTTER/MARKER/CHANGE/ROW_NUMBER and optional per-column header ICON; default compact picture unchanged
- Disposition: amended; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Earlier exact-nine-parts and blanket no-ICON restrictions superseded; status ban retained
- Gates: Default compatibility; optional clipped slots; Header slot replaces whole; prefix defaults below explicit overrides
- Origin: docs/refactoring-plan/history-late-obligations.tsv:26; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-69-PRESS

- Source: a1759b2a §69; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Snapshot/capture retain owner and PartRef; Grid press row/cell/actions targeted; ScrollRegion only thumb
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Child press paints whole container pressed
- Gates: Exact target cells/regions; forced target cursor or thumb only; unrelated children unforced
- Origin: docs/refactoring-plan/history-late-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM57

- Source: 3adb6efe §52 Q1; COMPONENT_ARCHITECTURE.md;docs/reviews/laneB-grid-contract.md
- Requirement: Grid emits keyed sort request; adapter owns domain comparator/permutation including NULLs-last and numeric ordering
- Disposition: accepted; current retain_verify
- Remaining proof: No rendered-text comparator/local-sort switch/order vector in Grid; row/cell/edit identities survive model reorder
- Gates: server-sort dispatch; local result order1,2,10+NULL; keyed pending edits/range/checks
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:58; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM58

- Source: 3adb6efe §52 Q2; COMPONENT_ARCHITECTURE.md;docs/reviews/laneB-grid-contract.md
- Requirement: GridState readers only owned semantics: keyed cursor borrowed KeySet editing typed FieldError and column offset; no derived row/column windows
- Disposition: accepted; current retain_verify
- Remaining proof: Preserve FieldError code and edit lifecycle; geometry queries need Grid/model/area not hidden state cache
- Gates: external API compile; typed error code; offset reveal; state reorder
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:59; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG19

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: External insertion/removal/reorder preserves logical focus/selection/edit target; disappearing owner reconciles predictably
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-identity; F02/F03/F05/F06
- Gates: Stable-key mutation/reorder/disappearance property tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG33

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collections support borrowed domain content, custom rows/cells, metadata and relevant empty/loading/error states
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; ItemSource/AsItem/row adapters
- Gates: Non-owned consumer fixtures and visible empty/loading states
- Origin: docs/refactoring-plan/history-other-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG34

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collection cursor differs from chosen value; relevant single/multiple/range selection remains explicit
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; Choice/Picker/Grid
- Gates: Independent cursor/value/check/anchor mutation tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG36

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Grid mechanics generic; SQL,FK,nullable schema,commit queues and database validation stay in TablePro adapters
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-Grid; TablePro rows
- Gates: Generic non-database consumer and TablePro capability journeys
- Origin: docs/refactoring-plan/history-other-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG39

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Read-only permits appropriate navigation/copy while rejecting mutation; disabled is distinct
- Disposition: accepted_current_oracle_bounded; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Ordinary readonly navigation/copy and nonmutation remain accepted; F04 mid-edit permission-transition proposal is deferred, not an override of current oracle behavior. Prove direct-component versus app reachability separately
- Gates: Ordinary readonly mutation/navigation proof; separately retain F04 direct transition oracle observations and deferred disposition
- Origin: docs/refactoring-plan/history-other-obligations.tsv:40; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
