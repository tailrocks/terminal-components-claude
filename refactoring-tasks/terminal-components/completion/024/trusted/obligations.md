# TASK-024 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

### Fixed branch-source repair

Completion must measure separate maximum semantic label and detail display widths, not the maximum combined width of any single row. The source presentation requests clamp(max_label+max_detail+8,24,48) before runtime placement/clipping and uses source-qualified detail visibility, first-frame and resize geometry; W-024-07 binds unequal-row maxima and popup lifecycle. Picker/FilterList add a shared typed keyless Submit outcome under an explicit query-submission capability, disabled by default to preserve existing generic callers. With that capability enabled, a source-searchable, nonblank query and no eligible current item produces Submit through the same normal/alternate Enter command path; an eligible item keeps its keyed normal/alternate outcome. Submit carries no fabricated key/index; the caller reads its own query state. Keep ordinary disabled and hidden-query guards, and encode separately the actual oracle readiness/eligibility fallback cases in W-024-08 rather than silently adopting deferred F01. TASK-042 enables the capability for Files jump and consumes Submit as selected=None. No application raw-Enter handler, fake item, second filter/editor or domain work in the component. Capability-off retains existing generic Ready/Loading/Error/Partial behavior. Modern Partial remains consumed with no action for both capability states; no source PickerStatus::Partial exists, so enabling source Submit does not extend that state. TASK-024 may change apps/showcase/src/pages/pickers.rs only to add the exhaustive no-op Submit match arm required by the additive PickerAction variant; leave the capability off, result/detail/level/layer state and every existing product branch unchanged. This compile-only consumer repair does not close or alter Showcase app parity.

The concrete `source-witnesses.md` companion is normative for R-001/R-002/R-003. Bind its source-state cases and real production mutants in the protected context before dispatch; execute them through their stated CHK-004/CHK-006/CHK-005 mappings as applicable. This is additional source-bounded proof, not permission to omit any clause below or to treat a proposed/deferred behavior as oracle authority.

FilterList/Picker consume semantic AsItem independently of RowFn; filter/paste/grapheme queries preserve source indexes. CommandPalette unavailable/destructive actions retain exact stable target. PickerChain back/error/retry retains stage query and breadcrumb; Completion uses insert text or label fallback, exact editor splice and independent label/detail width maxima. Opt-in typed keyless query submission follows W-024-08 without fabricated item identity.

Caller owns stages/data/jobs; component owns reusable projection/navigation only. CompletionController routes Arrow/Tab/Enter/Esc/remap/paste between editor and popup; runtime layers own placement/capture. No fake semantic selection or renderer-defined identity.

Test non-Clone/non-Display borrowed sources, empty/loading/partial/error, nonsearchable rows, unavailable action, target reorder before release, stage back/retry and narrowing resize. Match original-grapheme indexes, exact inserted bytes, popup fade/caret and editor unchanged on canceled completion.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each declared override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Specific regression target

Add `crates/tui/tests/completion_024.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `picker_nonsearchable`, `picker_projection`, `picker_width`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:filter-list

- family: filter-list
- reference_implementation: O:src/widgets/picker.rs filtering/results
- main_implementation: M:crates/tui/src/components/filter_list.rs
- architectural_target: Borrowed filtered index projection separate from caller data
- visual_status: missing fade through picker composition
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:filter_list.rs units;empty_style_inheritance.rs;picker_projection.rs
- tests_missing: Filtering eligibility/paste/grapheme query; exact empty/loading/error rows and fades

### COMP:picker-command-palette

- family: picker-command-palette
- reference_implementation: O:src/widgets/picker.rs
- main_implementation: M:crates/tui/src/components/picker.rs
- architectural_target: Borrowed AsItem keyed result rows and query; CommandPalette composition
- visual_status: missing fade; width/status geometry unverified
- interaction_status: eligibility/query/modal routing unverified
- api_refactor_status: implemented; oracle changes need proof
- tests_available: M:picker_nonsearchable.rs;picker_projection.rs;picker_width.rs;picker.rs units
- tests_missing: Oracle query paste/grapheme modifiers; unavailable action; destructive stable target; fade

### COMP:picker-chain

- family: picker-chain
- reference_implementation: O:Holla chained action/resource pickers;Jackin picker journeys
- main_implementation: M:crates/tui/src/components/picker_chain.rs
- architectural_target: Caller-owned stages and typed Back/Retry without domain execution
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; app composition parity needed
- tests_available: M:picker_chain.rs breadcrumb/root-owned-keyed-part tests;conformance.rs
- tests_missing: Oracle stage/back/query retention unavailable/error/retry and focus restoration

### COMP:completion

- family: completion
- reference_implementation: O:src/widgets/completion.rs
- main_implementation: M:crates/tui/src/components/completion.rs
- architectural_target: Borrowed completion rows and editor-owned CompletionController
- visual_status: missing fade
- interaction_status: unverified popup/editor input ownership
- api_refactor_status: implemented; verify
- tests_available: M:completion.rs editor binding and splice tests;conformance.rs
- tests_missing: Arrow/Tab/Enter/Esc/remap/paste exact editor state; popup geometry/fade under resize

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A70

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §67; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Picker/FilterList/Completion consume semantic AsItem; renderer does not define filtering or identity
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Borrowed non-Clone non-Display model and RowFn-only compile failure
- Gates: semantic item tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:71; global semantic anchor; supplemental clauses retained

### HIST:A71

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §67; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Completion insertion uses insert text, falling back to label; matches index original graphemes
- Disposition: accepted; current retain_and_verify
- Remaining proof: Distinct insert/label and Unicode transformed search
- Gates: completion insert tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:72; global semantic anchor; supplemental clauses retained

### HIST:A123

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §69; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Picker 100k borrowed non-Clone/non-Display render: 19 visible rows/38 calls, zero bytes; strict ratio<=1.5
- Disposition: accepted; current retain_and_verify
- Remaining proof: Warm 1k versus100k equivalent viewport
- Gates: picker_100k_borrowed_domain_render
- Origin: docs/refactoring-plan/historical-obligations.tsv:124; global semantic anchor; supplemental clauses retained

### HIST:F08b

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f08b-picker-grapheme-editing-query-paste.md
- Requirement: Picker Unicode query editing and actual paste preserve grapheme boundaries
- Disposition: current_holla_done_later_tablepro_open; current current_main_mapping_required
- Remaining proof: Owner-route paste and UTF-8 edit/caret/copy contract
- Gates: Picker query input
- Origin: docs/refactoring-plan/historical-obligations.tsv:140; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-030

- Source: a1759b2a §18.2; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: CompletionController keeps focus/binding ownership on editor while popup owns geometry, pointer, scroll and lifecycle.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Completion::update_for(editor_id,…) consumes reachable editor-addressed commands; no app hand-wired synthetic keys.
- Gates: Owner-separated bindings; editor focus retained; popup click/scroll/close; nested overlay and completion action traces.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:31; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-054

- Source: c3b51b96 §20.10 item35; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Empty List/Grid/FilterList/Picker regions inherit owning EMPTY style; NavList hover must lift only hovered row.
- Disposition: accepted reusable ownership; oracle output controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: EmptyState::draw_inherited and hovered-difference masks remove blank/gap/header tint bug class;historical20-key classification style-only.
- Gates: Canonical cells including blanks;empty full rect;hover header/gap/other-row invariance;exact text/geometry unchanged.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-054

- Source: 2e453023 §12.5 J6–J9; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: File browser domain stays app composition; Wizard owns steps/rewind retention; PickerChain owns generic staged loading/error/back navigation; shared keyed row rendering replaces app reimplementations.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Steps remains displayonly; filesystem/1Password models and async lifecycle do not enter library. Source owner correction: J6 is Jackin JA-015/TASK-051, with JA-017/034/059 consumed by TASK-052/053/056; Holla TASK-042 cannot prove Jackin J6.
- Gates: J6–J9 each current consumer, state retention and source error/retry/back/cancel flows.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-010

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:353-429;634-655;723-745
- Requirement: Completion picker select menus compose overlays/collections; dialog body open
- Disposition: accepted amended; current unverified
- Remaining proof: Functional scrolling and typed actions; retain nontrapping Select final policy
- Gates: Long option list; scrollbar drag; owner focus; nested Esc and click-outside
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-017

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.2-2.3
- Requirement: Menus use typed action keys/chords; picker uses keyed borrowed rows filtered behavior and layer composition
- Disposition: accepted target; exact scope/controller APIs need later authority join; current not independently tested
- Remaining proof: Preserve submenu scope/back/loading/error/disabled/secondary product paths without string payloads
- Gates: key/mouse actions; async retry/back; no parallel index vectors
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-021

- Source: e81ca17b; docs/audit/domain-boundary-audit.md4.1 J1-J13
- Requirement: Reusable form/choice/info/help/wizard/picker-chain/row-decoration facilities replace duplicated app plumbing
- Disposition: accepted dispositions; exact APIs require architecture join; current not independently tested
- Remaining proof: Every J item needs explicit disposition; do not promote file-system or account semantics to library
- Gates: showcase coverage for new public facilities; wizard rewind/drafts; custom rows; domain boundary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:22; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-67-SEMANTICS

- Source: a1759b2a §67; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Item/AsItem carries key,label,glyph,matched,detail,insert,tag,group,disabled; picker family requires AsItem, paint-only RowFn
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Display requirement; painter-derived filter; K parameter/key builder
- Gates: Domain AsItem no Display/Clone; compile-fail RowFn-only; semantic key/filter/grapheme matches; insert-vs-label
- Origin: docs/refactoring-plan/history-late-obligations.tsv:28; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-69-PERF

- Source: a1759b2a §69; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Borrowed-domain picker warm80x24,19 visible rows38 AsItem reads,zero allocations/bytes
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: 100k-dependent preparation in measured draw; Display/Clone requirement
- Gates: 1k/100k deterministic accessor equality; PERF_STRICT ratio<=1.5
- Origin: docs/refactoring-plan/history-late-obligations.tsv:38; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG33

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collections support borrowed domain content, custom rows/cells, metadata and relevant empty/loading/error states
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; ItemSource/AsItem/row adapters
- Gates: Non-owned consumer fixtures and visible empty/loading states
- Origin: docs/refactoring-plan/history-other-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
