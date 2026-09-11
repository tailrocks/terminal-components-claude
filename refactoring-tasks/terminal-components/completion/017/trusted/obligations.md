# TASK-017 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

One resolver clamps/flips anchors and distinguishes Fill from Fixed zero. Test tiny/empty/offscreen anchors, changed body size/theme, reverse draw order, undrawn modal and duplicate layer. Modal traps and restores even empty; popover focus-out dismisses without restoring opener.

Layer lifecycle owns z-order/barriers; only size/anchor may change while open and no-op setters do not repaint. Keep capture origin/publication semantics; no Form/Dialog dependency or app backdrop loop.

Nested editor Esc precedes outer dismissal; modal-first paste and outside/secondary clicks never leak. Resize/reparent and opener removal yield exact focus/cursor and dimmed cells. Empty body remains clipped with single invocation; malicious callbacks cannot escape layer clip.

## Fixed re-audit contract — runtime capture versus explicit oracle application commands

Default runtime modal/inert capture remains authoritative: blocked controls receive no hidden Intent::Paste, key, pointer or wheel; no LayerSpec passthrough API, fake focus or alternate router is introduced. Source-qualified application compatibility is an explicit domain command before runtime dispatch, not leakage from a modal. Holla F10 MenuBar retains its editing Args paste through TASK-016's shared TextInput command; TablePro Picker retains editing query paste through TASK-025's shared CodeEditor command. True Holla modals and TablePro Dialog/Filter branches retain their oracle-specific paste owner.

TablePro's concrete route is W;Ctrl+T;i;type(SELECT );Ctrl+O (and separately Ctrl+G);paste(PARITY): oracle app.rs:236–254 falls through Picker to Workbench.on_paste:1194 and Query.on_paste in tabs.rs:1543. Picker query/focus stays unchanged while the already editing underlying query changes once. Component probes here prove no implicit runtime delivery; app owners prove these full-frame typed-command branches and default modal isolation. Reconciliation of this decision is not left to an executor.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_017.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `overlay`, `layer_focus_reparent`, `focus_restoration`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:layers-popups

- family: layers-popups
- reference_implementation: O:src/ui/popup.rs;widgets/dialog.rs;menu.rs;select.rs
- main_implementation: M:crates/tui/src/layer.rs;ui/layer_buf.rs;runtime.rs
- architectural_target: Runtime layer stack anchor size backdrop capture focus restoration
- visual_status: unverified
- interaction_status: unverified modal routing/dismissal
- api_refactor_status: implemented; verify
- tests_available: M:overlay.rs;layer_focus_reparent.rs;dialog_navigation.rs
- tests_missing: Modal-first paste; nested Esc/outside capture; relocation after resize; exact backdrops

### ARCH:A10

- id: A10
- area: layers
- oracle_state: app backdrop loops popup placement
- main_state: LayerSpec Anchor Dismiss compositor
- architectural_target: single runtime layer stack/anchor resolver; open generic Dialog content
- status: implemented;parity unproven
- remaining_obligation: prove exact dimming/overlay capture/dismissal/restoration across app flows
- available_gates: overlay;dialog_navigation;dialog_acknowledgement
- missing_proof: all stacked oracle frames plus action outcomes
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:664;7b27732:crates/tui/src/layer.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A17

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §8,§9; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Esc first reaches focused editor, then application bubble/escape policy and layer dismissal
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Nested editing/modal/menu ladder and focus restoration
- Gates: Esc journey matrix
- Origin: docs/refactoring-plan/historical-obligations.tsv:18; global semantic anchor; supplemental clauses retained

### HIST:A18

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §9,§29.8; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Modal traps/restores even empty; popover dismisses on focus-out without restoring opener
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Nonempty/zero-area forward/backward traps; same-owner reparenting
- Gates: modal and popover tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:19; global semantic anchor; supplemental clauses retained

### HIST:A19

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §9; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Layer lifecycle controls barriers/z-order independent of paint call order
- Disposition: accepted; current retain_and_verify
- Remaining proof: Reverse draw order, undrawn modal, duplicate layer, background input
- Gates: layer conformance
- Origin: docs/refactoring-plan/historical-obligations.tsv:20; global semantic anchor; supplemental clauses retained

### HIST:A20

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §26; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: LayerSize Fill differs from Fixed zero; one resolver clamps/flips placement
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: Empty/tiny/edge anchoring and dynamic size/error/theme updates
- Gates: layer-size tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:21; global semantic anchor; supplemental clauses retained

### HIST:A21

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §26; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Open layers allow size/anchor updates only; immutable kind/inert/focus lifetime facts
- Disposition: accepted; current retain_and_verify
- Remaining proof: No-op setters do not repaint; lifecycle immutable
- Gates: layer setters
- Origin: docs/refactoring-plan/historical-obligations.tsv:22; global semantic anchor; supplemental clauses retained

### HIST:A101

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §54; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: dim_layer zero identity; ordered semantic fade then erase; role collisions cannot drive generic reverse lookup
- Disposition: accepted; current compare_new_oracle
- Remaining proof: All roles/color levels and preserved footer/handoff
- Gates: dimming properties;frames
- Origin: docs/refactoring-plan/historical-obligations.tsv:102; global semantic anchor; supplemental clauses retained

### HIST:F01

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f01-modal-first-paste-routing.md
- Requirement: Modal owner receives/consumes paste before hidden editors
- Disposition: deferred_not_selected_oracle_preserved; current historical_proposal_not_current_product_gate
- Remaining proof: Deferred proposal, not selected by immutable-oracle parity. Preserve oracle02f5294b TablePro app.rs237-254 Picker paste fallthrough; do not impose exhaustive modal-first ownership or apply this TablePro proposal to Holla. Any later change requires explicit authority
- Gates: Source-qualified disposition against exact current oracle; no proposed-behavior PASS claim
- Origin: docs/refactoring-plan/historical-obligations.tsv:132; global semantic anchor; supplemental clauses retained

### HIST:O06

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; docs/improvements-plan-reference.md
- Requirement: Generic container/modal framework proposal
- Disposition: retired; current current_main_mapping_required
- Remaining proof: Use accepted runtime/layer/container architecture
- Gates: rejected design ledger
- Origin: docs/refactoring-plan/historical-obligations.tsv:191; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-010

- Source: a3fe79b1 §29.8;11924199 §29.8;15371443 §29.8; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: A popover losing focus dismisses without restoring its opener; modal traps have bidirectional nonempty wrapping proof.
- Disposition: accepted; implementation checkpoint not fresh gate; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: a3fe marked Dialog TRAPS_FOCUS and driver owed;11924199 reports implementation at5d17cc0;15371443 corrects runnable module paths.
- Gates: FocusOut popover/modal differential; both directions of caps/modal correspondence; empty and zero-area traps; Tab/BackTab wrap.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-008

- Source: 2e453023 §§8–9;95ab6529 §21;dc3e0fa1 §28 P3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Layer IDs allocated at open; hit priority is layer then latest registration; capture released on close; lifecycle addressed to decorative owners must also be consumed.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No exemption based solely on absence of focus/hit registration; update layer owner unconditionally.
- Gates: Decor-only LayerCancel/FocusOut delivery; closed Dialog next update; topmost-hit order permutation.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-009

- Source: 2e453023 §9;587c53bd §25 F3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Pooled layer composition copies only written cells; role provenance supports dimming only covered rect and excludes footer policy as selected.
- Disposition: accepted_subject_to_pinned_app_parity; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old uniform-footer visual-change permission is not current oracle authority.
- Gates: Sentinel unpainted cells; nested overlay permutation; typed source-role dim; exact pinned footer cells.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-035

- Source: 587c53bd §26; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime LayerSize Fill/Fixed and anchors own geometry; zeroFixed empty; Dialog's pure props+tokens measure equals draw; resize/reanchor applies sameframe.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Reject min_size sentinel, recentering in Dialog and runtime-held borrowed props callback.
- Gates: All anchors edges/narrow/zero; prompt height none0/field/+1ack; theme-sensitive measure; update/draw equality.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-039

- Source: 3ed377e3 §29.8; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Select popover does not trap; FocusOut producer dismisses without restoring opener; modal alone traps; one-stop wrapping tested.
- Disposition: accepted_historical_contract_requires_oracle_join; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Old Select Tab swallowing is not generic trap authority; current immutable parity may require source-selected app routing.
- Gates: Tab next target preserved; modal negative case; nonempty actualtrap; exact pinned-app Tab flow.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:40; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-055

- Source: 2e453023 §12.5 J10–J13; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Shared RowDecor/change slots, keyed tabs, meter threshold helper and modal stack/result routing replace duplicates without moving product policy into library.
- Disposition: accepted_with_later_meter_policy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Quota lifecycle remains application-owned; runtime feedback and typed actions replace ad hoc UI state.
- Gates: J10–J13 keyed row action, tab reorder, thresholdboundary, layerresult tests with pinned copy/cadence.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:56; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:INLINE-F8

- Source: 27bd918e3a8a0a7fdba14fb10643139340d6281f; COMPONENT_ARCHITECTURE.md
- Requirement: Nested Select popup owns its layer lifecycle, independent of draw order; Esc reaches control before outer layer.
- Disposition: accepted; later amendments apply; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: A17,A19,A80
- Gates: Nested popup closes before dialog; no deferred repaint or re-registration hack; apply later popover focus policy.
- Origin: docs/refactoring-plan/history-inline-obligations.tsv:9; refines A17,A19,A80; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-004

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc;e81ca17b; docs/audit/api-audit.md:151-194;docs/audit/app-audit.md:132-178
- Requirement: Encapsulated hit/focus registration and layer ownership prevent inert bypass and manual reregistration
- Disposition: accepted amended; current unverified
- Remaining proof: Replace public registry mutation and duplicate placement algorithms
- Gates: Reverse registration order; nested pointer/keyboard isolation; dismissal focus restore
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:5; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-035

- Source: 4e15e97e; docs/reviews/slice2-architecture-review.md:B7/B11/B12;M2-M5/M29;572-587
- Requirement: Bounded transition-only focus settle; owner capture; layer outside-hit and restore semantics
- Disposition: J accepted; later runtime ordering governs; current unverified
- Remaining proof: Verify order edges instead of replay or draw-order ownership assumptions
- Gates: No duplicate key activation; lower-layer outside hit; inert silent discard versus rejection; pre-draw restore key; close releases capture; duplicate layer draw diagnostic
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-005

- Source: e81ca17b; docs/audit/interaction-audit.md B4-B8
- Requirement: Focus scopes restoration explicit capture nested layers and cursor arbitration
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Top-layer hit wins regardless registration; focused owner cursor; resize delivers focus transition
- Gates: nested overlays; reversed draw order; capture release; multi-writer cursor; resize focus pair
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-041

- Source: d7faa2ce; docs/reviews/adjudication-n-layer-measure.md N1-N2
- Requirement: Explicit LayerSize Fill/Fixed; only runtime resolves placement; pure Ui resolve/metrics for measure
- Disposition: accepted; current not independently tested
- Remaining proof: No sentinel0 size; no component duplicate center/clamp; slots/theme glyphs affect measurement
- Gates: tiny viewport; resize/reanchor; wrapped rows; shared draw pure measure
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:42; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-047

- Source: ad94d12a; docs/reviews/adjudication-q-residuals.md; STATE
- Requirement: Select opens Popover without focus trap; OVERLAY distinct from TRAPS_FOCUS
- Disposition: accepted Q; contradictory trap proposed and under adjudication at handoff; current not independently tested
- Remaining proof: Join final §33/later ruling; never silently convert popover to modal to satisfy case14
- Gates: outside-focus dismissal; owner focus; pointer barrier; modal-only tab trapping
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:48; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-053

- Source: d7faa2ce; docs/reviews/adjudication-n-layer-measure.md N1
- Requirement: Fixed zero means empty Rect; size never grows into free space; only resize_layer/reanchor mutate open geometry
- Disposition: accepted; callback measurement and mutable LayerSpec closure rejected; current not independently tested
- Remaining proof: Preserve kind/inert/initial-focus after open; repaint only on geometry change; retain scope semantics for empty layer
- Gates: Fixed0; oversized/tiny viewport; identical resize no repaint; rejected outside mutable policy access
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:54; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-062

- Source: bb92a657; docs/reviews/adjudication-p-prototype-decisions.md P3
- Requirement: Diagnose every undrained runtime-addressed Layer/Cancel/FocusIn/FocusOut owner regardless decorative registration
- Disposition: accepted with factual correction; current not independently tested
- Remaining proof: Unconditional owner update; size at opener prevents Fill flash; decorative pointer exemption stays; no redraw-based result polling
- Gates: Gated-shape diagnostic negativecontrol vs unconditional Dismissed action; decorative pointer no bucket; all within samehandle
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:63; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-54-DIM

- Source: ecc13378 §54.5; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: dim_layer zero identity; semantic monotonic non-ladder Muted/Faint/Ghost/erase
- Disposition: accepted-architecture-parity-constrained; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Color-equality reverse lookup; constant Muted at every step
- Gates: All color levels; exact oracle cells constrain visible policy; no newly authorized visual changes
- Origin: docs/refactoring-plan/history-late-obligations.tsv:4; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM65

- Source: 3adb6efe §54;ecc13378 §54.5; COMPONENT_ARCHITECTURE.md
- Requirement: Ui dim_layer zero byte identity and total semantic Muted→Faint→Ghost→erase; no generic resolved-color equality reverse lookup
- Disposition: accepted;later Meter authored policy exception separate; current retain_verify
- Remaining proof: Preserve role provenance under collisions/all capability levels and exact oracle fade frames
- Gates: zero-step bytes; all foreground roles; collisions; step>=4 erase to actual backdrop
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:66; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG25

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: One layered overlay contract handles stack,z-order,modal barrier,trapping,restore,outside click,Esc,anchor,flip,clamp,cursor,hints,lifecycle
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-layers; overlay family matrix
- Gates: Nested overlay full input/paste/wheel/cursor journey
- Origin: docs/refactoring-plan/history-other-obligations.tsv:26; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
