# TASK-029 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

Compare determinate/indeterminate/paused/done/failed progress, spinner phase/gap/inactive and exact oracle clock boundaries. Meter zero/full/no-ratio/series, line/block, readout/on-fill, stale/unknown and thresholds paint exact glyphs/roles at every capability.

Keep shared driver clock; Input::Tick does not advance elapsed time. MeterFillRest Color/ReferenceLift uses accepted ordered Canvas/Surface/Elevated/Field/Popover reference policy only; no generic RGB role inference. Authored tokens and explicit colors win before delayed dimming.

Probe every threshold immediately below/equal/above, fill rounding at tiny width, authored alias collisions, ten-frame ASCII timing and motion wrap. Historical seven-of-sixteen Meter changed digests are not new oracle evidence; preserve source history while comparing independent oracle captures.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_029.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `meter_defaults`, `meter_readout`, `meter_semantic_tokens`, `progress_paused_glyph`, `spinner_gap`, `feedback_clock`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:progress-bar

- family: progress-bar
- reference_implementation: O:src/widgets/progress.rs
- main_implementation: M:crates/tui/src/components/progress.rs
- architectural_target: ProgressBar semantic state and deterministic phase
- visual_status: unverified
- interaction_status: static display; update phase unverified
- api_refactor_status: implemented; verify
- tests_available: M:progress_paused_glyph.rs;progress.rs percent/sweep/done tests
- tests_missing: Oracle determinate/indeterminate/paused/done/failed widths animation phases all colors

### COMP:spinner

- family: spinner
- reference_implementation: O:src/widgets/progress.rs
- main_implementation: M:crates/tui/src/components/progress.rs
- architectural_target: Shared Spinner glyph/time recipes
- visual_status: unverified
- interaction_status: clock-driven phase unverified
- api_refactor_status: implemented; verify
- tests_available: M:spinner_gap.rs;status_spinner.rs;feedback_clock.rs
- tests_missing: Oracle frame vocabulary phase gap inactive states reduced animation where supported

### COMP:meter

- family: meter
- reference_implementation: O:app custom meter/progress painting;src/theme.rs
- main_implementation: M:crates/tui/src/components/meter.rs
- architectural_target: Reusable semantic MeterRole thresholds fill/rest/readout
- visual_status: unverified app-specific visuals
- interaction_status: static display
- api_refactor_status: implemented; app composition parity needed
- tests_available: M:meter_defaults.rs;meter_readout.rs;meter_semantic_tokens.rs
- tests_missing: Oracle occupancy/usage thresholds exact fill glyphs colors labels and zero/full values

### ARCH:A12

- id: A12
- area: time and feedback
- oracle_state: Instant and app-specific tick clocks
- main_state: Moment monotonic advance and FeedbackClock
- architectural_target: driver time separated from simulation; precise presented feedback
- status: implemented;flow parity unproven
- remaining_obligation: carry same timing boundaries through all restored app scenarios
- available_gates: monotonic;feedback_clock;activation_origin;app timing tests
- missing_proof: seeded equal-time and threshold PTY/action schedules
- evidence: 7b27732:crates/tui/src/runtime/time.rs:1;7b27732:crates/tui/src/event.rs:24

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A37

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md Meter appendix;amendments=4a9dcab1,d52ee38b,29ed8e5e; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: MeterFillRest explicit Color or ReferenceLift; ordered reference color comparison is narrow authored policy
- Disposition: accepted_later_amendment; current retain_and_verify
- Remaining proof: Canvas-first aliases and delayed dimming; explicit colors win
- Gates: Meter 16-frame preservation
- Origin: docs/refactoring-plan/historical-obligations.tsv:38; global semantic anchor; supplemental clauses retained

### HIST:EARLY-030

- Source: d52ee38b appendedMeter;29ed8e5e amendedMeter; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: MeterFillRest ReferenceLift applies exact ordered source-color policy; concrete custom/Paper colors stay authoritative; retain typed source surface for delayed dim.
- Disposition: accepted_later_amendment; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: RaisedSurface semantic-enum mapping superseded; ordered color comparisons are explicit source policy, not inferred paint-role provenance.
- Gates: Alias/capability matrix;16-frame existing preservation test unchanged; concrete+symbolic slot counts and dim provenance.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:31; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-055

- Source: 2e453023 §12.5 J10–J13; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Shared RowDecor/change slots, keyed tabs, meter threshold helper and modal stack/result routing replace duplicates without moving product policy into library.
- Disposition: accepted_with_later_meter_policy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Quota lifecycle remains application-owned; runtime feedback and typed actions replace ad hoc UI state.
- Gates: J10–J13 keyed row action, tab reorder, thresholdboundary, layerresult tests with pinned copy/cadence.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:56; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-016

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:659-671;749-760;821-879
- Requirement: Separate generic progress from quota lifecycle; stable Tabs and keyboard SplitPane
- Disposition: accepted direction; exact homes owner governed; current unverified
- Remaining proof: Domain quota/tab/PTY policy stays app-owned; all catalog families mapped
- Gates: Keyed reorder/close; keyboard and drag resize; supported statuses and glyph parity
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-METER-POLICY

- Source: d52ee38b ->29ed8e5e Meter amendment; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: MeterFillRest Color or ReferenceLift; compare resolved Canvas then Surface/Elevated then Field else Popover
- Disposition: accepted-superseding; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: RaisedSurface semantic-enum dispatch; generic raise ladder; painted-role RGB inference
- Gates: Aliased/reduced palettes exact order; typed HoverSurface provenance; Low Secondary/Stale Faint; Paper concrete rest
- Origin: docs/refactoring-plan/history-late-obligations.tsv:57; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-METER-PINS

- Source: c62ec8d3 meter_defaults diff; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Seven of16 preservation integers changed; prose says unchanged pending review
- Disposition: current-evidence-gap; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Current test success accepted as immutable product parity
- Gates: Reconstruct canonical before/after cells from pinned Sep10 oracle and reconcile provenance before baseline acceptance
- Origin: docs/refactoring-plan/history-late-obligations.tsv:59; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM39

- Source: GAP-5 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: Meter threshold tone reaches painted foreground at all ladder rungs
- Disposition: accepted; current proof_required
- Remaining proof: Exercise ratios around all thresholds and authored policy at every color capability
- Gates: actual run color at50/70/95 and boundaries; oracle cells
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:40; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM40

- Source: GAP-6 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: Meter Block background share/value on-fill color plus no-ratio value-only and explicit Series paths remain covered
- Disposition: accepted; current proof_required
- Remaining proof: Do not let single ratio line-mode matrix stand for unexecuted alternate painter branches
- Gates: Block fill/text inversion; ratio absent; explicit tone; Series
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:41; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM41

- Source: GAP-7 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: Library Stale/Unknown meter behavior survives while Warning/Exhausted/Refreshing domain lifecycle belongs in Jackin
- Disposition: accepted_boundary; current proof_required
- Remaining proof: Separate generic paint from app lifecycle and preserve all old six-tone observable cases
- Gates: Stale/Unknown library fixtures; app lifecycle markers and labels
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
