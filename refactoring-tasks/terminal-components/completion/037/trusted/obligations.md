# TASK-037 trusted obligations

## Noninteractive Overview and public-author preservation

Oracle `src/bin/showcase/pages/overview.rs:11,174–188` has no live page control. Main `apps/showcase/src/pages/author.rs:18–82` is a genuine public-author facade consumer, but its Overview state/update/draw references at `overview.rs:165–175,197,437–460` change the oracle surface. R-001/AC-001/CHK-004 requires exact complete Overview geometry/cells at all declared sizes; R-002/AC-002/CHK-006 requires absence of the badge and its page hit/focus/state consumers, not a narrow-size concealment, zero-area draw or test-only product branch. Remove the unnecessary product-demo source/declaration under the narrow scope; preserve the public-author API and demonstrate actual runtime registration, draw, input, focus and state through an already accepted equivalent or a real outside-product consumer in the owned `tests/completion_037.rs`. This immediate R-003/AC-003/CHK-005 regression cannot wait for TASK-068's later external documentation/example closure. Main `tests/app_tests.rs` Author/Overview requirements and `tests/sidebar_contract.rs` static-Overview assertions require exact TASK-008 assertion-level disposition before modification; no wholesale deletion, weakened preservation, altered archived authority or future receipt dependency is permitted. Bind the no-phantom-stop/hit observation to the preserved full SC-FOCUS parent expansion without claiming ownership of other pages' unresolved focus cases.

## Fixed ADJ-15 Select consumer

ADJ-15 is binding for Oracle src/bin/showcase/pages/chips.rs53–61 sort and page-size Selects, plus its disabled engine fixture. Compose the shared configured Select with navigation(SelectNavigation::Commit).open_keys(SelectOpenKeys::ConsumeUnhandled) in both phases, or the same LabelSelect props through the existing Form bridge. Consume existing Chose only: changed values produce one domain callback, clamped/equal-value choice produces none while preserving Changed/closure flow. Do not manually set a second Select value or duplicate navigation. Preserve exact source open Tab/BackTab consumption, closed traversal, outer global-chord precedence, Esc, external focus-out, disabled/read-only and option rebuild behavior. Generic unrelated Select defaults remain unchanged. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

## Re-audit shared API consumer contract

ADJ-11: replace TASK-020's compile-only new-action placeholders with real LeadRequested match_all toggling and ClearRequested list clearing. Use one shared borrowed ChipBar with lead and checked/removable/error providers, scrollable(false), reserve_add(false), source-qualified add label, exact clipping/hits and no group focus on overflow. No app-local strip painter or fake keys. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005. Direct and full application traces must prove the actual source composition; an API declaration or compile-only adaptation does not close parity.

This file is part of the immutable task package, not candidate-writable configuration. It binds R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006. Source rows are copied without changing their action or assertion content from the showcase scenario register. Every copied field below is normative except its historical capture-status description, which is not a parity verdict.

## Observable outcome

Restore Overview swatches/typography/guidance; single/multi/range/disabled/empty Lists; Tree fold/disclosure/ancestor/selection details; Sidebar expanded/collapsed sections with disabled Billing and cursor/current distinction; Chips add/remove/all/any/overflow and Select chosen/cursor/cancel/disabled behavior.

## Architecture and non-regression

Use keyed List/Tree/NavList/ChipBar/Select and borrowed row/part override channels. Do not recreate the oracle's old application-local NavList; preserve exact appearance through accepted public composition and reusable fades.

All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.

## Trust and expansion

The oracle commit is 02f5294bfdbf38004cc49130d0aff1d01f31434c; main architecture starts at 7b27732a8c3c131760ec3438f641cb3c11343a42. Accepted prerequisite receipts, taskfmt 52d9f1eb7721f409bc47beb9fced7997b5c13ede and the independently qualified proof executable are host-owned. Materialized actions, numeric oracle coordinates, scenario/checkpoint membership, palettes, clocks, fonts, profiles, semantic mappings and exact expected cells are sealed before this task starts. Candidate code cannot read expected artifacts or write the host catalog, receipts, refs, comparator or another run. The dispatcher/bootstrap is produced by TASK-001, runners by TASK-070, accounting by TASK-071 and architecture probes by TASK-072. These commands are prerequisite deliverables, not evidence that they already execute.

The host supplies the immutable full application expansion grammar and proof contract alongside this file. Expand all recorded sizes, colors, focus stops, targets, ticks, source-qualified test bodies and checkpoint boundaries exactly as the accepted baseline did. Preserve every original assertion in ROUTE seeds. Source-only helpers and SCAN/SWEEP membership are materialized before this task; the executor never expands against its own implementation. Capture full row-major schema-3 cells, blank/wide continuation cells, modifiers, cursor and semantic transitions after every listed checkpoint. Preview/state-only or exact-clock lanes remain honest; executable reachability and terminal lifecycle require an actual process and owned PTY. Both capture operations must produce an explicit complete result, including accepted non-applicability rather than omitted output.

Cross-cutting full-app rows may be closure-owned; earlier tasks still preserve the subset already closed in the host ledger. Final closure replays the union on one exact tree and requires zero application-unresolved identities. Future-owned diagnostic failures are reported as failures, never success.

## Exact scenario contracts

### APP:SC-BASE-overview

- **page:** overview
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Complete shell; swatches; typography; badges; guidance
- **source_refs:** O:pages/overview.rs:76
- **components:** theme,text,Panel

### APP:SC-BASE-lists

- **page:** lists
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Single/multi/disabled/empty list presentation
- **source_refs:** O:pages/lists.rs:66
- **components:** List,Panel,scrollbar

### APP:SC-BASE-trees

- **page:** trees
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Tree hierarchy and selection detail
- **source_refs:** O:pages/trees.rs:34
- **components:** Tree,Panel,scrollbar

### APP:SC-BASE-sidebars

- **page:** sidebars
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Sections; labels/icons; disabled Billing; current vs cursor
- **source_refs:** O:pages/sidebars.rs:372
- **components:** NavList,Panel,Button

### APP:SC-BASE-chips

- **page:** chipsselects
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Filter chips; disabled engine; selects; segments/props/empty
- **source_refs:** O:pages/chips.rs:131
- **components:** ChipBar,Select,Segments,Props,EmptyState

### APP:SC-LIST-SELECT

- **page:** lists
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(Language);Down;Enter;End;Home;PageDown;focus(Files to include);Space;Shift+Down*3;a;a;click enabled row;click disabled row;focus(Search results);Enter
- **required_observable_proof:** Chosen vs cursor; multi/range/all/none; disabled skips; empty list boundary actions
- **source_refs:** O:pages/lists.rs:19,158;O:src/widgets/list.rs:125
- **components:** List,selection,scroll

### APP:SC-TREE-FOLD

- **page:** trees
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(Project tree);Left;Right;Down;Right;Enter;*;-;End;Home;click(src disclosure);click(src row label);wheel(tree,+3);assert collapsed-root layout has no scrollbar;focus(Project tree);*;assert expanded tree overflow;drag(scrollbar,top,bottom)
- **required_observable_proof:** Expansion vs activation; ancestor navigation; selection details; stable gutter; collapsed-fit vs expanded-overflow exact geometry; disabled/leaf distinctions as O
- **source_refs:** O:pages/trees.rs:105;O:src/widgets/tree.rs:367
- **components:** Tree,scrollbar,fade

### APP:SC-SIDEBAR-MODES

- **page:** sidebars
- **sizes:** 72x20,80x24,120x40
- **action_checkpoints:** focus(demo navigation);Down;Enter;End;Home;Down*5;click Billing;click Collapse;assert collapse button focused;focus(demo navigation);Down;Enter;click(expand control rendered as ›);wheel(nav,+3); at72x20 drag(nav scrollbar,top,bottom); at80x24/120x40 assert demo scrollbar absent and no scrollbar hit
- **required_observable_proof:** Cursor/current retained; disabled Billing skipped; click-collapse focus vs explicit navigation re-entry; collapsed › activation expands; icons/labels/borders; overflow13rows in10row viewport at72x20 vs no scrollbar in14/16row viewports
- **source_refs:** O:pages/sidebars.rs:123,173,453
- **components:** NavList,Button,scrollbar

### APP:SC-CHIP-FILTERS

- **page:** chipsselects
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(filters);Right;Space;Enter;x;+;X;+;click(match all);click chip remove;repeat+ until overflow;Left;Right
- **required_observable_proof:** Enabled flag vs removed chip; status-only edit request; candidate cycle; empty/overflow clipping; all/any lead
- **source_refs:** O:pages/chips.rs:76,270;O:src/widgets/chips.rs:91
- **components:** ChipBar,Segments,EmptyState

### APP:SC-CHIP-SELECTS

- **page:** chipsselects
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(Sort by);Enter;Down;Enter;Enter;Down;Esc;Left;Right;focus(Page size);Enter;Down;Space;click Engine;open Sort by;click outside
- **required_observable_proof:** Popup selected/cursor distinction; cancel; inline arrow selection; disabled engine; focus restoration/outside dismissal
- **source_refs:** O:pages/chips.rs:270;O:src/widgets/select.rs:76
- **components:** Select,overlay,focus

## Correction boundary

If exact parity requires a shared-component fix, unsupported public extension, changed source authority or new trusted fixture, stop NEEDS_REPLAN and name the responsible owner. Do not duplicate mechanics inside this app or edit expected evidence. A completed task produces host-verified results and an accepted tree receipt, not a candidate-written approval.
