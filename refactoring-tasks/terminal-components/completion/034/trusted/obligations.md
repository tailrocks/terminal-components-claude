# TASK-034 trusted obligations

This file is part of the immutable task package, not candidate-writable configuration. It binds R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006. Source rows are copied without changing their action or assertion content from the showcase scenario register. Every copied field below is normative except its historical capture-status description, which is not a parity verdict.

## Observable outcome

Replace the five-metric/four-column impostor with the actual customer Grid model: 40→80→96 rows; eight typed columns; keyed selection, range/plain/TSV copy, widths and sorting; insert/duplicate/delete/undo/discard/refresh; pending edits and SQL dialog; four-tick commit, seats>500 error and successful retry. Restore Tasks/Checks tables and all editable-cell commit/cancel/validation/click behavior. Do not invent a live reference-column fixture absent from the oracle.

## Architecture and non-regression

One generic Grid implementation supplies row and cell modes; the live model must be the visible customer data and actual dispatch model. Delete customer paint-over and invisible/live metric scaffolding; no application-local table editor or scrollbar clone.

All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.

## Trust and expansion

The oracle commit is 02f5294bfdbf38004cc49130d0aff1d01f31434c; main architecture starts at 7b27732a8c3c131760ec3438f641cb3c11343a42. Accepted prerequisite receipts, taskfmt 52d9f1eb7721f409bc47beb9fced7997b5c13ede and the independently qualified proof executable are host-owned. Materialized actions, numeric oracle coordinates, scenario/checkpoint membership, palettes, clocks, fonts, profiles, semantic mappings and exact expected cells are sealed before this task starts. Candidate code cannot read expected artifacts or write the host catalog, receipts, refs, comparator or another run. The dispatcher/bootstrap is produced by TASK-001, runners by TASK-070, accounting by TASK-071 and architecture probes by TASK-072. These commands are prerequisite deliverables, not evidence that they already execute.

The host supplies the immutable full application expansion grammar and proof contract alongside this file. Expand all recorded sizes, colors, focus stops, targets, ticks, source-qualified test bodies and checkpoint boundaries exactly as the accepted baseline did. Preserve every original assertion in ROUTE seeds. Source-only helpers and SCAN/SWEEP membership are materialized before this task; the executor never expands against its own implementation. Capture full row-major schema-3 cells, blank/wide continuation cells, modifiers, cursor and semantic transitions after every listed checkpoint. Preview/state-only or exact-clock lanes remain honest; executable reachability and terminal lifecycle require an actual process and owned PTY. Both capture operations must produce an explicit complete result, including accepted non-applicability rather than omitted output.

Cross-cutting full-app rows may be closure-owned; earlier tasks still preserve the subset already closed in the host ledger. Final closure replays the union on one exact tree and requires zero application-unresolved identities. Future-owned diagnostic failures are reported as failures, never success.

## Exact scenario contracts

### APP:SC-BASE-tables

- **page:** tables
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Tasks/Checks columns; tones; clipping
- **source_refs:** O:pages/tables.rs:97
- **components:** Table,Panel,scrollbar

### APP:SC-BASE-editable

- **page:** editabletables
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Cell navigation table; preset branch error
- **source_refs:** O:pages/editable.rs:75
- **components:** Table,TextInput,Panel

### APP:SC-BASE-grid

- **page:** datagrid
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** 40 customers of96; eight typed columns; toolbar/footer; null/JSON/bool/currency
- **source_refs:** O:pages/grid.rs:309
- **components:** DataGrid,Panel,Button,Props

### APP:SC-TABLE-SORT

- **page:** tables
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(Tasks);s;s;s;click column header three times;Down;Enter;End;Home;PageDown;wheelH(Tasks,+3);wheel(Tasks,+3)
- **required_observable_proof:** Ascending/descending/unsorted stable order; column indicator; row choice; horizontal and vertical clamp
- **source_refs:** O:pages/tables.rs:130;O:app_tests.rs:279,309
- **components:** Table,sort,scroll

### APP:SC-EDITABLE-CELL

- **page:** editabletables
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(Tasks);assert initial column Task;Enter;End;type( ok);Enter;assert one committed edit;Enter;type(discard);Esc;Right to Owner;Enter;End;type( bad);Enter;assert Owner is a single handle;Esc;move Changes column;Enter;Ctrl+U;type(x);Enter;Esc;move Branch column;F2;Ctrl+U;type(bad name);Tab
- **required_observable_proof:** Task commit vs revert; Owner space validation; numeric/branch error keeps editor; Tab next editable cell and validation boundary; readonly columns skip
- **source_refs:** O:pages/editable.rs:14,116;O:app_tests.rs:323
- **components:** Table,TextInput,validation

### APP:SC-EDITABLE-MOUSE

- **page:** editabletables
- **sizes:** 120x40
- **action_checkpoints:** click editable task cell;click same cell;paste(Changed);Tab;BackTab;Esc;click sort header;assert120x40 table scrollbar absent;resize80x24;drag(vertical table scrollbar,top,bottom);resize120x40;wheelH(table,+2)
- **required_observable_proof:** Exact single/repeated-click edit behavior; retained cell identity through sort; original geometry; no scrollbar for14rows at120x40; real overflow drag at80x24; clamp and focus retained across resize
- **source_refs:** O:pages/editable.rs:116;O:src/widgets/table.rs
- **components:** Table,HitRegistry,text-edit

### APP:SC-GRID-NAV

- **page:** datagrid
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** focus(customer grid);Right;Down;Shift+Right;Shift+Down;y;Y;Space;Home;End;Ctrl+Home;Ctrl+End;s;s;s;S;wheelH(grid,+4);wheel(grid,+4)
- **required_observable_proof:** Cell/range/row selection; plain vs TSV copy; first/last cell; local sort identity; fixed columns and scroll edges
- **source_refs:** O:src/widgets/grid.rs:975;O:pages/grid.rs:325
- **components:** DataGrid,selection,sort,scroll

### APP:SC-GRID-EDIT

- **page:** datagrid
- **sizes:** 120x40
- **action_checkpoints:** focus(customer grid);move seats cell;Enter;Ctrl+U;type(invalid);Enter;Esc;Enter;Ctrl+U;type(501);Enter;Ctrl+S;ticks3;ticks1;Enter;Ctrl+U;type(50);Enter;Ctrl+S;ticks4
- **required_observable_proof:** Typed validation; pending count; four-tick loading; server seats>500 row error; successful retry clears pending and increments saved
- **source_refs:** O:pages/grid.rs:198,269,325
- **components:** DataGrid,Field,commit,clock

### APP:SC-GRID-QUEUE

- **page:** datagrid
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(customer grid);+;edit inserted name;Enter;Ctrl+d;-;u;p;Enter;assert Cancel leaves pending unchanged;p;Right;Enter;assert Copied N statements status;U;r;G;PageDown;G;PageDown;G;PageDown
- **required_observable_proof:** Insert/duplicate/delete/undo; SQL facts initially Cancel and explicit Copy SQL result/status; pending queue unchanged by either dialog action; discard/refresh; fetch40->80->96; exhausted boundary
- **source_refs:** O:pages/grid.rs:147,198;O:src/widgets/grid.rs:1116
- **components:** DataGrid,Dialog,Props,paging

### APP:SC-GRID-ACTIONS

- **page:** datagrid
- **sizes:** 120x40
- **action_checkpoints:** focus(customer grid);move JSON/bool/null cells;Enter on each;f;/;F;Ctrl+];click each toolbar action on fresh appropriate clean/pending state;drag column boundary;click header
- **required_observable_proof:** Viewer/filter status; bool toggle/null representation; no invented reference-column fixture; toolbar hitboxes; widths/sort preserve state
- **source_refs:** O:pages/grid.rs:198;O:src/widgets/grid.rs:1053,1145
- **components:** DataGrid,Toolbar,HitRegistry

## Correction boundary

If exact parity requires a shared-component fix, unsupported public extension, changed source authority or new trusted fixture, stop NEEDS_REPLAN and name the responsible owner. Do not duplicate mechanics inside this app or edit expected evidence. A completed task produces host-verified results and an accepted tree receipt, not a candidate-written approval.
