# TASK-012 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Exercise fixed/flex/Auto and measured Auto allocation, insets, wrapping, truncation, zero/one-cell extents and nonzero origins. SplitPane alone returns two logical body rectangles excluding seam; horizontal/vertical minima, drag grab offset, collapse, maximize and resize retain oracle ratios and targets.

Measure performs no state mutation, registration or allocation and follows the same style accumulation as paint. Container callbacks run once even under empty clipping; empty leaves paint/register nothing. No public bypass exposes duplicated split geometry.

Negative cases deliberately attempt paint/registration from an empty body and stale resize hits. Compare first-frame layout facts and scroll-position labels, half-open boundaries, offset seam hitboxes and 1000 repeated measures with unchanged durable state and allocation counters.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_012.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `split_seam_geometry`, `grid_column_fit`, `panel_badge`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:layout-measure

- family: layout-measure
- reference_implementation: O:src/ui/layout.rs;popup.rs
- main_implementation: M:crates/tui/src/layout.rs;measure.rs;ui/cx.rs
- architectural_target: Pure layout/measure and current LayoutFacts
- visual_status: unverified
- interaction_status: resize metadata freshness unverified
- api_refactor_status: implemented; verify
- tests_available: M:split_seam_geometry.rs;grid_column_fit.rs;component tiny clipping tests
- tests_missing: Nonzero origin and 0/1 extents; resize threshold transitions; first-frame position labels

### COMP:split-pane

- family: split-pane
- reference_implementation: O:src/widgets/splitter.rs
- main_implementation: M:crates/tui/src/components/split.rs
- architectural_target: SplitPaneState geometry capture resize/maximize callback
- visual_status: unverified
- interaction_status: unverified drag and keyboard focus
- api_refactor_status: implemented; verify
- tests_available: M:split_seam_geometry.rs;split.rs minima/grab/maximization tests
- tests_missing: Oracle seam hitbox pointer grab ratio focus/minima/maximized/resize states

### ARCH:A14

- id: A14
- area: layout and measure
- oracle_state: ad hoc app rows/columns and raw backgrounds
- main_state: layout Track Insets Measure Surface
- architectural_target: shared measure/layout same geometry in draw and update
- status: implemented;parity unproven
- remaining_obligation: correct geometry at responsible primitive/component and test tiny/resizes
- available_gates: grid_column_fit;panel_badge;split_seam_geometry;viewport
- missing_proof: oracle component and app rect/state coverage
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:748;7b27732:crates/tui/src/layout.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A20

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §26; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: LayerSize Fill differs from Fixed zero; one resolver clamps/flips placement
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Empty/tiny/edge anchoring and dynamic size/error/theme updates
- Gates: layer-size tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:21; global semantic anchor; supplemental clauses retained

### HIST:A22

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §26; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Measure queries styles without mutation/allocation; shares accumulation path with paint resolution
- Disposition: accepted; current retain_and_verify
- Remaining proof: Differential family/variant/part/state tests; 1000 measures no effects
- Gates: measurement differential;10k zero alloc
- Origin: docs/refactoring-plan/historical-obligations.tsv:23; global semantic anchor; supplemental clauses retained

### HIST:A23

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §5,§22; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: All paint/clip/hit/focus/scroll/cursor geometry derives from same allocation
- Disposition: accepted; current retain_and_verify
- Remaining proof: Tiny nonzero-origin containment and queued event freshness
- Gates: geometry conformance;PTY resize
- Origin: docs/refactoring-plan/historical-obligations.tsv:24; global semantic anchor; supplemental clauses retained

### HIST:A24

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §5,§55; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Empty leaves paint/register nothing; containers still invoke body exactly once under empty clip
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Bare return sentinel, malicious body containment, tiny/zero dimensions
- Gates: container closure tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:25; global semantic anchor; supplemental clauses retained

### HIST:A26

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §56; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: SplitPane sole two-rect body exception; logical order and seam excluded; no public bypass geometry
- Disposition: accepted; current retain_and_verify
- Remaining proof: Horizontal/vertical/collapse/maximize/empty/seam tests
- Gates: split conformance
- Origin: docs/refactoring-plan/historical-obligations.tsv:27; global semantic anchor; supplemental clauses retained

### HIST:A27

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §10,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Layout Auto/Flex/measure semantics explicit; allocation-free distribution
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Measured/unmeasured Auto and fixed/flex clipping
- Gates: layout tests;10k allocations
- Origin: docs/refactoring-plan/historical-obligations.tsv:28; global semantic anchor; supplemental clauses retained

### HIST:F17

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f17-empty-allocations-emit-nothing.md
- Requirement: Empty allocations paint/register nothing
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: Zero/tiny nested containment with sentinel surroundings
- Gates: Component containment
- Origin: docs/refactoring-plan/historical-obligations.tsv:151; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-001

- Source: 9486b654 §26.2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Sizing resolution records no painted parts; painted-part evidence must come from actual instrumented paint paths.
- Disposition: accepted correction; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Strikes false Ui::style/style_patched automatic styled-part recording claim; later parts instrumentation must prove its actual behavior.
- Gates: Downstream author paint fixture resolves and paints declared parts; sizing-only resolution produces no styled-part evidence.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-014

- Source: 3adb6efe §§5,17 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Dialog body-slot draw returns R; one-slot containers use one body closure; SplitPane is the sole two-rectangle body exception.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Dialog Option<R> sketch corrected to R; full §56 contract belongs to late ledger.
- Gates: Public compile examples inspect exact closure return and both logical SplitPane rects; shared self and state preserved.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-007

- Source: 2e453023 §5 R5;587c53bd §25; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Every component is safe at 0×0 through3×3, clips both edges, registers no stale geometry, and avoids underflow/ragged-row indexing.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Layer traps remain live even when body cannot paint.
- Gates: Full tiny matrix; sentinel outside rect; translated left-edge checks; zero-size-to-live transitions.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-036

- Source: 587c53bd §26; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Ui::resolve/Theme::metrics are shared sizing paths without painting side effects; Ui::style/with_part records actual painting family/part and resolved role.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No hardcoded Button in Harness resolved query; no cache mutation disguised as measured semantic state.
- Gates: Record actual queried family/variant; metrics surface-independent; one with_part resolve; role record only actualpaint.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-060

- Source: 69fcdcad §10;587c53bd §17examples9–10; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: TrackAuto without measurement takes1cell besideFlex or equal remainder withoutFlex; measuredvariants use natural sizes; zeroalloc distribute_into serves RowUi columns with16track cap.
- Disposition: accepted_historical_geometry_contract; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Do not change sourceobservable layout silently under PLANNING parity; contract names cannot prove examples use measuredvalues.
- Gates: Auto/Flex/fixed/multipleAuto/tiny matrix; measured themedglyph widths; example9two-rowProps notclipped; columns cap and zeroalloc.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:61; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-008

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;834aa58e; docs/audit/api-audit.md:170-180;1067-1069;1166-1205
- Requirement: Shared measurement and safe clipped geometry avoid tiny-rect underflow and stale interactions
- Disposition: accepted amended; current unverified
- Remaining proof: Preserve minima/fallback behavior through accepted integer layout
- Gates: 0/1-cell rects; resize disappear; no out-of-clip hit/pixels; focus survives by contract
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-016

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:659-671;749-760;821-879
- Requirement: Separate generic progress from quota lifecycle; stable Tabs and keyboard SplitPane
- Disposition: accepted direction; exact homes owner governed; current unverified
- Remaining proof: Domain quota/tab/PTY policy stays app-owned; all catalog families mapped
- Gates: Keyed reorder/close; keyboard and drag resize; supported statuses and glyph parity
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-020

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.9-2.11;4.2
- Requirement: Stable Tabs and SplitPane/Viewport reusable mechanics; dirty-close preview/pinning pane-tree/PTY semantics app-owned
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Remove per-render tabs/viewport reconstruction; keep domain tab policies and pane simulation
- Gates: logical close after reorder; strip window; zoom/split/drag; caret override without clone
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-028

- Source: ba858131; docs/audit/modern-api-audit.md R7-R13
- Requirement: Runtime cursor write; semantic theme styles; modifier patch laws; typed glyph sets; integer layout vocabulary
- Disposition: accepted; current not independently tested
- Remaining proof: Do not introduce Stylize/literal RGB/constraint solver or second scrollbar state; preserve Junie symbols
- Gates: boundary checks; modifier removal; layout tiny sizes; typed glyph/border contract
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:29; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-041

- Source: d7faa2ce; docs/reviews/adjudication-n-layer-measure.md N1-N2
- Requirement: Explicit LayerSize Fill/Fixed; only runtime resolves placement; pure Ui resolve/metrics for measure
- Disposition: accepted; current not independently tested
- Remaining proof: No sentinel0 size; no component duplicate center/clamp; slots/theme glyphs affect measurement
- Gates: tiny viewport; resize/reanchor; wrapped rows; shared draw pure measure
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-054

- Source: d7faa2ce; docs/reviews/adjudication-n-layer-measure.md N1/N2
- Requirement: Measurement resolves theme/surface/overlay without styled-part recording or cache mutation; metrics omit overlays/surface for update
- Disposition: accepted; measure signature unchanged; current not independently tested
- Remaining proof: Measurement cannot make invisible part appear conformance-covered or evict painting cache; instance patch excluded by N contract
- Gates: Cache stats and styled-parts unchanged after measure; glyph size differential; later overrides authority join
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-56-TRAVERSE

- Source: 3adb6efe §56; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: SplitPane one FnOnce Ui/Rect/Rect body returning bare R exactly once under with_area for every area
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Tuple-return geometry; separate closures; public SplitAreas
- Gates: Normal/empty/tiny/collapsed/maximized sentinel; nonzero anchor; malicious body clipping; seam preservation; AST signature
- Origin: docs/refactoring-plan/history-late-obligations.tsv:7; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG15

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Typed semantic modes/errors, private geometry/caches, normal interactions cannot panic
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: Public API inventory and negative probes
- Gates: Fallible ingestion, tiny geometry and panic guards
- Origin: docs/refactoring-plan/history-other-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG31

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Consistent minimum/preferred sizing,insets,clipping,truncation/wrap,overflow,resize and parent-surface composition
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-layout; all family allocation matrices
- Gates: Nonzero-origin/tiny/empty/overflow/resize sentinels
- Origin: docs/refactoring-plan/history-other-obligations.tsv:32; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG32

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Empty rectangles cannot panic,underflow,paint/register outside allocation or leave stale hits
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: F17; A-containment
- Gates: Zero-size render/update/query/hit/cursor assertions
- Origin: docs/refactoring-plan/history-other-obligations.tsv:33; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
