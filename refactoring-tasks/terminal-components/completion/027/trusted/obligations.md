# TASK-027 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

Restore oracle EmptyState geometry through the already scoped shared Empty helper and standalone component: intersect the allocation with the buffer, wrap the optional hint at max(width.saturating_sub(4),8), vertically center the complete title plus optional blank row plus wrapped hint block using saturating subtraction, truncate and horizontally center each line within the clipped width, and clip bottom rows. Error renders the centered '! '+title in error tone with only '!' bold; ordinary title is muted and hint faint against the caller background. W-027-07 proves these source-backed Empty/Error configurations; it does not invent Loading/Partial oracle behavior. Preserve existing modern readiness and per-state TITLE/HELP/ICON reachability, inherited owner/provenance and real paint sinks. Layout measurement and painting must consume one shared geometry result, not disagreeing wrap estimates or an app-local renderer.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

Button exact keyboard/completed-click and disabled/readiness geometry; Brand static lockup versus click-only branch; Panel title/meta/badge/container-focus clipped body; borrowed Props/PropsList rich wrapping/copy/secret masks; EmptyState inherited owner styles and exact clipped, wrapped, vertically centered source Empty/Error block; TooSmall four-line notice and recovery match oracle.

No second focus stop for decorative containers, no public secret copy, no obsolete Lockup/ScrollPanel clone. EmptyState already has inheritance: preserve rather than duplicate it. All static/action chrome uses semantic public components and source text belongs to caller.

Test each five collection owners Empty/Loading/Partial/Error with actual-cell sentinels, ignored ICON slot negative, panel empty callback once, Props keyed copy/reorder/secret lock, late position labels and too-small minimum−1/equal/+1. Both themes/all capabilities plus nonzero/zero clips; changing live source facts changes painted cells.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_027.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `empty_style_inheritance`, `independent_empty_carrier`, `props_rich`, `props_rich_perf`, `panel_badge`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:button

- family: button
- reference_implementation: O:src/widgets/button.rs
- main_implementation: M:crates/tui/src/components/button.rs
- architectural_target: Controlled Button typed command runtime states
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:button.rs units;showcase_buttons.rs;render_components.rs;conformance.rs
- tests_missing: CP-COMMON; exact click/keyboard activation and disabled/autofocus readiness

### COMP:brand

- family: brand
- reference_implementation: O:src/widgets/brand.rs
- main_implementation: M:crates/tui/src/components/brand.rs
- architectural_target: Static or click-only Brand with label/meta parts
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:brand.rs five units;render_components.rs;conformance.rs
- tests_missing: Oracle lockup padding/glyphs all modes clickable versus static

### COMP:panel

- family: panel
- reference_implementation: O:src/widgets/panel.rs:31
- main_implementation: M:crates/tui/src/components/panel.rs
- architectural_target: Pure Panel chrome and clipped body callback; no focus stop
- visual_status: unverified
- interaction_status: unverified decorative registration
- api_refactor_status: implemented; verify
- tests_available: M:panel_badge.rs;panel.rs body/clip/parts tests
- tests_missing: Oracle card/framed/title/meta/badge/container-focus widths and late position freshness

### COMP:props

- family: props
- reference_implementation: O:src/widgets/props.rs
- main_implementation: M:crates/tui/src/components/props.rs
- architectural_target: Borrowed PropsValue/PropsRow and keyed PropsList copy
- visual_status: missing fade
- interaction_status: unverified copy and secret behavior
- api_refactor_status: implemented rich borrowed API; verify
- tests_available: M:props_rich.rs;props_rich_perf.rs;props.rs keyed/copy/secret tests
- tests_missing: Oracle label/value wrapping source copy secret lock state protected-row fade

### COMP:empty-readiness

- family: empty-readiness
- reference_implementation: O:src/widgets/empty.rs;collection inline empty states
- main_implementation: M:crates/tui/src/components/empty.rs;collection/empty.rs
- architectural_target: Shared EmptyState with owner style inheritance and distinct semantic parts
- visual_status: unverified; old missing-inheritance finding repaired
- interaction_status: unverified retry/disabled contexts
- api_refactor_status: implemented; verify actual painted cells
- tests_available: M:empty_style_inheritance.rs;independent_empty_carrier.rs;empty.rs units
- tests_missing: All five collection owners Empty/Loading/Partial/Error sentinel styles and geometry

### COMP:too-small

- family: too-small
- reference_implementation: O:four app minimum-size notices
- main_implementation: M:crates/tui/src/components/too_small.rs
- architectural_target: Shared TooSmall component with exact source app contract
- visual_status: unverified
- interaction_status: resize recovery and quit unverified
- api_refactor_status: implemented; app adoption proof required
- tests_available: M:too_small.rs ten units;render_components.rs
- tests_missing: Each app minimum boundary -1/equal/+1; notice text exact; recovery restores state

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A24

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §5,§55; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Empty leaves paint/register nothing; containers still invoke body exactly once under empty clip
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Bare return sentinel, malicious body containment, tiny/zero dimensions
- Gates: container closure tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:25; global semantic anchor; supplemental clauses retained

### HIST:A25

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §55; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Panel/Dialog body returns bare R; only Ui::layer returns Option<R>
- Disposition: accepted; current retain_and_verify
- Remaining proof: Public signatures and one body invocation
- Gates: signature gate;external examples
- Origin: docs/refactoring-plan/historical-obligations.tsv:26; global semantic anchor; supplemental clauses retained

### HIST:A79

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §63; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: TooSmall owns notice hierarchy and four-line geometry with isolated theme overrides
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Tiny states and transition restoration across apps
- Gates: TooSmall conformance
- Origin: docs/refactoring-plan/historical-obligations.tsv:80; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-014

- Source: 3adb6efe §§5,17 A7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Dialog body-slot draw returns R; one-slot containers use one body closure; SplitPane is the sole two-rectangle body exception.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Dialog Option<R> sketch corrected to R; full §56 contract belongs to late ledger.
- Gates: Public compile examples inspect exact closure return and both logical SplitPane rects; shared self and state preserved.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:15; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-031

- Source: a1759b2a §§11.2,20.10 item24; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: TooSmall has dedicated appended family raw value34; preserve copy, blank rows, centering and faint ACTIONS.
- Disposition: accepted architecture; oracle pixels govern; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: New family must not renumber existing values; historical first-generation64 keys not blanket appearance approval.
- Gates: Family isolation; small/zero rect; exact quit tone and oracle minimum-size text; complete public conformance.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:32; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-009

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:258-349
- Requirement: Brand/chip/choice use shared presentation and roving/field mechanics
- Disposition: proposal homes; accepted broad dispositions; current unverified
- Remaining proof: Map each old family explicitly; overflowing chip bar remains reachable
- Gates: Full-width overflow; disabled/readonly distinctions; oracle marker/bracket geometry
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-015

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:515-524;553-562;675-719;764-794
- Requirement: Unify static/interactive props, priority strips and component hint metadata
- Disposition: accepted direction; current unverified
- Remaining proof: No second drop algorithm or independently painted shortcut text
- Gates: Priority drop oracle; copy/activate actions; last-row hint precedence; hover by item
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:16; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-018

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.5-2.6
- Requirement: DiffView consumes source abstraction; ScrollPanel absorbed by TextViewport; Panel gives contextual surface
- Disposition: accepted target; current not independently tested
- Remaining proof: No copied diff domain or separate read-only scroll engine; contextual roles preserve oracle
- Gates: unified/review and resize; wheel survives redraw; custom theme and inherited background
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:19; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-63-FAMILY

- Source: a1759b2a §63; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: TOO_SMALL family34; Primary title bold/Secondary detail/Muted help/Faint actions; exact four lines
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: PANEL alias/variant; implicit instance patch; unnamed family
- Gates: Bidirectional Panel isolation; q Quit faint cell; family completeness; twenty conformance cases
- Origin: docs/refactoring-plan/history-late-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM35

- Source: GAP-1;36468976 §47.4;later a1759b2a; COMPONENT_ARCHITECTURE.md;docs/audit/legacy-test-disposition.md
- Requirement: Brand clickable registration/hover/click preserved; plain lockup remains inert and default fixture may retain empty Caps
- Disposition: accepted; current retain_verify
- Remaining proof: Production click-only branch both positive and negative; no invented keyboard activation
- Gates: Brand click hit PartRef hover paint; plain no registration
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM69

- Source: 3adb6efe §55; COMPONENT_ARCHITECTURE.md
- Requirement: Panel/Dialog return bare R and invoke body exactly once even zero/tiny/collapsed; empty rect anchored inside request under container surface and empty clip
- Disposition: accepted; current retain_verify
- Remaining proof: Malicious closure cannot paint/register outside empty clip; Ui::layer alone optional; SplitPane separate contract
- Gates: typed sentinel; call count; nonzero-origin degenerate rect; no escaped paint/hit; AST return/signature gate
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:70; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
