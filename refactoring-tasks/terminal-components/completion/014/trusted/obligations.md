# TASK-014 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Implement oracle src/ui/fade.rs arithmetic once: height below four none; four through eleven one outer row at 55% contrast; twelve and above add inner row at 80%. Majority-background ties, componentwise rounding, reversed/different-background exclusion, hardware-cursor/current protected rows and non-RGB outer DIM match oracle. Exclude headers/footers/bar; paint after content and before bar.

Compose ScrollRegion directly, never revive Ui::scroll_region or collection-to-component inversion. Preserve one thumb track/grab and nearest eligible owner; boundary wheel consumes without chaining, repaint or focus shift. Hidden bars reserve no column and expose no pointer part while wheel/reveal stay active.

Test heights 3/4/11/12, fits/top/middle/bottom, resize thresholds, both themes/all capabilities, selected/hovered/marked protected cells and clipping. Hide during capture, scroll full endpoints, retain grab offset, and reject a per-app duplicate or fade changing glyph/hit/selection geometry.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_014.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `scrollbar_full_track`, `nav_list_scrolling`, `nav_list_scrollbar_visibility`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:scroll-state-region

- family: scroll-state-region
- reference_implementation: O:src/core/scroll.rs;widgets/scrollbar.rs;ui/fade.rs
- main_implementation: M:crates/tui/src/scroll.rs;components/scroll_region.rs
- architectural_target: Single scroll model and owner-keyed ScrollRegion plus reusable fade painter
- visual_status: missing CP-01 fades
- interaction_status: shared truthful boundary/drag present; oracle proof needed
- api_refactor_status: implemented; fade missing
- tests_available: M:scrollbar_full_track.rs;scroll_region::thumb_drag_preserves_the_grab_offset;O:core/scroll.rs tests
- tests_missing: CP-SCROLL-FADE; exact track endpoints/grab offset; no boundary chaining/focus movement

### ARCH:A09

- id: A09
- area: wheel and scroll
- oracle_state: public state and copied scrollbars
- main_state: private ScrollState shared ScrollRegion
- architectural_target: nearest eligible owner consumes boundary wheel; one thumb/capture model
- status: refactored;oracle fade missing
- remaining_obligation: add reusable fade semantics and preserve hidden-bar/wheel/reveal contract
- available_gates: scrollbar_full_track;nav_list_scrolling;nav_list_scrollbar_visibility
- missing_proof: oracle top/middle/bottom fade/cell/interaction equality
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:8791;02f5294:src/ui/fade.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A15

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Nested scroll owner consumes at boundary without chaining or repaint
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Preserve exact oracle routing and model boundaries
- Gates: nested wheel;boundary equality
- Origin: docs/refactoring-plan/historical-obligations.tsv:16; global semantic anchor; supplemental clauses retained

### HIST:A74

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §69; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Pressed PartRef styles only actual Grid row/cell/action or ScrollRegion thumb
- Disposition: accepted; current retain_and_verify
- Remaining proof: Sibling and whole-container press isolation
- Gates: pressed-part tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:75; global semantic anchor; supplemental clauses retained

### HIST:A105

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md ScrollRegion appendix;amendment=da4b1364; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Hidden scrollbar retains wheel/reveal, full content area, no bar paint/hit and releases drag capture
- Disposition: accepted_later_amendment; current retain_and_verify
- Remaining proof: Hide mid-drag and pointer-to-hidden-part refusal
- Gates: scrollbar_visible tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:106; global semantic anchor; supplemental clauses retained

### HIST:P02

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; git commits 53b8212f;e4866ce4;92d91629;02f5294b
- Requirement: Scrollable content fades toward hidden rows
- Disposition: oracle_behavior_binding; current current_main_mapping_required
- Remaining proof: 92d91629 edge fade cells/styles at top/middle/bottom and resize
- Gates: scroll fade matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:196; global semantic anchor; supplemental clauses retained

### HIST:P03

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; git commits 53b8212f;e4866ce4;92d91629;02f5294b
- Requirement: Truthful model-boundary scrolling and every container uses shared bounds
- Disposition: oracle_behavior_binding; current current_main_mapping_required
- Remaining proof: 02f5294b exact extent/offset/viewport/reveal/clamp behavior
- Gates: scroll model and app journey matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:197; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-002

- Source: afc60678 §§12.2,25.11,26.3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Owners compose ScrollRegion update/draw directly; Ui constructs no components.
- Disposition: accepted rejection; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Ui::scroll_region is struck, not deferred; seven legacy scrollbar copies remain successor-family migration obligations.
- Gates: Public-author composition compiles; no Ui-to-components dependency or legacy on_scrollbar copies.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:3; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-037

- Source: afc60678 §12.2 and§35; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Ui::scroll_region is struck, not an open implementation task; owner calls ScrollRegion update/draw and scrollbar parts share owner identity.
- Disposition: rejected_API_accepted_replacement; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Ui→components dependency inversion forbidden; seven on_scrollbar copies were library widgets, not app functions.
- Gates: No public convenience; direct composition; exact TRACK/THUMB/CONTAINER; no separate scrollbar ID.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:38; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-038

- Source: da4b1364 appendedScrollRegion; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: scrollbar_visible false preserves wheel/reveal, returns fullarea/min1×1, removes bar paint/hits and releases active thumb capture.
- Disposition: accepted_later_amendment; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Overflow alone no longer reserves column; NavList forwards property.
- Gates: Visible↔hidden mid-drag; stale bar-target consumed without scrolling; wheel works; measured/content rect exact.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:39; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-005

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:89-114;196-207
- Requirement: Scroll state invariants and Unicode text offsets need structural safe interfaces
- Disposition: audit finding; exact fix not accepted by this source; current unverified
- Remaining proof: Validate surviving issue class including casefold-to-original mapping
- Gates: Grapheme/cell corpus; Turkish dotted I; boundary wheel; no out-of-bounds
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-006

- Source: e81ca17b; docs/audit/interaction-audit.md B6
- Requirement: Innermost axis-aware wheel consumes boundary without bubbling or moving focus
- Disposition: accepted; current not independently tested
- Remaining proof: Wheel preserves cursor/selection; next navigation reveals cursor; modal blocks underlying scroll
- Gates: nested wheel actions at top/middle/bottom and each axis
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:7; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-69-PRESS

- Source: a1759b2a §69; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Snapshot/capture retain owner and PartRef; Grid press row/cell/actions targeted; ScrollRegion only thumb
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Child press paints whole container pressed
- Gates: Exact target cells/regions; forced target cursor or thumb only; unrelated children unforced
- Origin: docs/refactoring-plan/history-late-obligations.tsv:37; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-SCROLL

- Source: da4b1364 unnumbered amendment; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Scrollbar_visible default true, false full area1x1 min/no chrome/no pointer parts; wheel/reveal active; release hidden active thumb
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Discarded setter argument; content-width reservation when hidden
- Gates: Direct ScrollRegion contract plus NavList forwarding; hidden-bar pointer consumed no scrolling; hide-mid-drag release
- Origin: docs/refactoring-plan/history-late-obligations.tsv:60; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM13

- Source: afc60678 §35.1; COMPONENT_ARCHITECTURE.md
- Requirement: Ui::scroll_region convenience rejected; components compose ScrollRegion directly without collection-to-component layering inversion
- Disposition: rejected_api; current preserve_absence
- Remaining proof: No convenience revival or duplicate app scrollbar infrastructure
- Gates: public API and dependency boundary scan
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:14; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG23

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Pointer capture, drag selection, topmost wheel, nested scroll and scrollbar ownership
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-pointer/scroll; P03
- Gates: Captured drag/resize/remove-owner and nested-boundary scroll tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
