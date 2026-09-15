# TASK-060 trusted obligations

## Fixed ADJ-15 Select consumer

ADJ-15 is binding for Oracle src/bin/tablepro/app.rs1408–1409,1692,2072 Filter column/operator and dependent option rebuild;1665–1698 traverses Tab only while neither editor nor Select is open. Compose the shared configured Select with navigation(SelectNavigation::Commit).open_keys(SelectOpenKeys::ConsumeUnhandled) in both phases, or the same LabelSelect props through the existing Form bridge. Consume existing Chose only: changed values produce one domain callback, clamped/equal-value choice produces none while preserving Changed/closure flow. Do not manually set a second Select value or duplicate navigation. Preserve exact source open Tab/BackTab consumption, closed traversal, outer global-chord precedence, Esc, external focus-out, disabled/read-only and option rebuild behavior. Generic unrelated Select defaults remain unchanged. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

## Re-audit shared API consumer contract

ADJ-11: filter chips use one borrowed ChipBar with lead and checked/removable/error providers, scrollable(false), reserve_add(false), and source-qualified add label. LeadRequested toggles match_all; ClearRequested clears the domain filters; keyed actions resolve live identities. Preserve exact first-overflow early return and focus/hit behavior, not app-local strip painting. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 through this task's exact nonempty frame and semantic contributions. It does not promote the complete parent trajectories from their later TASK-063 owner or require future receipts. Complete app closure separately reruns the intact source trajectories; an API declaration or compile-only adaptation does not close parity.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore live typed Grid traversal/selection/sort/edit/validation/insert/duplicate/delete/undo, pending SQL preview, JSON/reference/fetch-more behavior, filter forms/operator/chip semantics and all four read-only Structure sections.

Use the generic public Grid with app-owned mutation/cell model, controlled Tabs, Select/Form/ChipBar and Code viewer. Keep SQL generation, catalog metadata, row identity and filter semantics in TablePro.

- Replace decorative legacy filter painting and inverted Data/Structure label selection with actual controlled components that paint the same state dispatch updates.
- Preserve nullability, key/read-only cells, escaped SQL and mixed pending-operation order; generic widgets must not acquire SQL/table/connection knowledge.
- Structure read-only mutation no-ops and return-to-Data preservation are required. Save/discard authorization is completed by TASK-062; this task does not reopen already closed checkpoints.
- TP-039–042 preserve the source Filter event policy: Enter applies the Filter, Tab commits a field without applying, lower editing Value receives paste and upper editing and consumes paste without insertion (oracle app.rs:237–246,1657–1785). F08a is deferred; retain a reusable TextInput with the narrow app-owned delivery policy, not a shared paste defect, fake focus or app-local insertion engine. TP-040 proves both the ignored upper paste and subsequent actual typed upper value, plus the separate original lower-Enter early Apply.
- TP-042's borrowed ChipBar exposes the source group focus and completed lead pointer action; group Enter edits the first chip, Space toggles, Delete removes and uppercase X clears. No lead/individual-chip focus stop or clear-all button may be invented to satisfy an invalid selector. The source pointer handoff is explicit even when Grid consumes Tab for navigation.

## Exact membership and staging

Primary complete scenario IDs: none; this bounded slice closes its explicit nonempty flow contributions.

Legacy preservation edges protect previously closed work only. New-behavior acceptance additionally requires every exact flow contribution; no empty preservation intersection is sufficient.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires complete primary IDs (none) and flow contributions for TP-027, TP-028, TP-029, TP-030, TP-031, TP-032, TP-033, TP-037, TP-038, TP-039, TP-040, TP-041, TP-042, TP-043, TP-044, TP-074, TP-079, TP-020, TP-021 through the exact tables. Nonempty whole-frame and fixed semantic evidence is mandatory; full parents remain with the audited last producer.

## Source scenario clauses

### TP-020

- surface: Explorer open keyboard
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Down*5;Enter;cp(orders);Down;Right*2;End;cp(grid_moved)
- assertions: Orders tab selection, breadcrumb, primary grid focus and cursor
- oracle_source: app_tests.rs:165
- dependencies: tree/grid/tabs

### TP-021

- surface: Explorer preview promotion
- sizes: ALL
- fixture: W
- actions_and_checkpoints: click(orders explorer object);cp(preview);click(customers explorer object);cp(replaced_preview);click(customers explorer object);cp(promoted);click(orders explorer object);cp(new_preview)
- assertions: Single-click preview reuse; second same-object click promotes; pinned tab persists
- oracle_source: workbench.rs:428,968
- dependencies: tree/tabs/identity

### TP-027

- surface: Grid traversal and scroll
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Down;Right;PageDown;cp(middle);Ctrl+End;cp(bottom_right);Down;Right;cp(clamped);Ctrl+Home;cp(top_left);Home;End;cp(column_edges)
- assertions: Row/column cursor, scroll offsets, sticky headers, scrollbar, fades and no overscroll
- oracle_source: src/widgets/grid.rs:995;workbench.rs:778
- dependencies: grid/scroll

### TP-028

- surface: Grid mouse scroll drag
- sizes: ALL
- fixture: T
- actions_and_checkpoints: hover(order_number header);cp(header_hover);wheelDown(grid)*12;cp(vscroll);wheelRight(grid)*12;cp(hscroll);drag(vertical scrollbar thumb,bottom);cp(bottom);drag(horizontal scrollbar thumb,right);cp(right)
- assertions: Mouse wheel axis and thumb hit regions preserve cursor/selection
- oracle_source: workbench.rs:1138,1163;src/widgets/grid.rs:1240
- dependencies: grid/scrollbar/hit

### TP-029

- surface: Grid selection and copy
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Down;Space;Down;Space;cp(selected);Shift+Down*3;cp(range);y;cp(copy);Y;cp(copy_headers);Esc;cp(cleared)
- assertions: Multi-row/range selected tones, copy character status and Esc clearing
- oracle_source: src/widgets/grid.rs:995,1065,1124
- dependencies: grid/selection

### TP-030

- surface: Grid sort tri-state
- sizes: ALL
- fixture: T
- actions_and_checkpoints: focus(grid);Right; s;cp(ascending);s;cp(descending);s;cp(cleared);click(order_number header);cp(mouse_sorted);S;cp(clear_sort)
- assertions: Stable row data reorder, header markers and exact statuses
- oracle_source: app_tests.rs:193;workbench.rs:778
- dependencies: grid/model

### TP-031

- surface: Grid inline edit commit cancel
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;cp(editing);paste(PARITY-订单);cp(draft);Esc;cp(cancelled);Enter;Ctrl+L;paste(PARITY-订单);Tab;cp(committed_next);BackTab;cp(previous_source_focus); separate fresh T at each ALL size: cell(1,currency);click-cell(1,currency);cp(current_cell_click_edit);Esc;Left;click-cell(1,currency);cp(different_cell_click_move);click-cell(1,currency);cp(repeated_cell_click_edit);Esc
- assertions: Short currency cell edit snapshot/cancel, UTF-8 cursor and dirty bytes; Tab/BackTab exact source next-cell/focus behavior; explicit current/different/repeated completed-click outcomes; original long notes behavior retained in TP-037
- oracle_source: src/widgets/grid.rs:930;app_tests.rs:609;src/widgets/grid.rs:1213-1235,1300-1345
- dependencies: grid/input/identity

### TP-032

- surface: Grid typed cells and nullability
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,id);Enter;cp(primary_key);cell(1,total_amount);Enter;paste(not-a-number);Enter;cp(invalid_number);Esc;cell(1,status);Delete;cp(null_request)
- assertions: Read-only/key treatment and type/nullability feedback exactly oracle
- oracle_source: src/widgets/grid.rs:530,1085;tabs.rs:388
- dependencies: grid/validation

### TP-033

- surface: Grid insert duplicate delete undo
- sizes: ALL
- fixture: T
- actions_and_checkpoints: +;cp(inserted);Alt+D;cp(duplicated);-;cp(deleted);u;cp(undo_delete);u;cp(undo_duplicate);u;cp(undo_insert)
- assertions: Default/auto cells, counts, stable row identity and undo sequence
- oracle_source: app_tests.rs:266;src/widgets/grid.rs:659,724,1122
- dependencies: grid/model

### TP-037

- surface: Grid viewer and foreign key
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,organization_id);Ctrl+];cp(reference_target);Ctrl+O;type(events);Enter;cell(1,properties);Enter;cp(json_viewer);PageDown;cp(viewer_scroll);Esc;cp(focus_restored); separate fresh T at each ALL size: cell(1,notes);cp(notes_keyboard_selected);click-cell(1,notes);cp(notes_actual_click);Enter;cp(notes_actual_enter);paste(PARITY-1);cp(notes_paste_no_mutation);Esc
- assertions: Reference table filtered by id; formatted JSON/code viewer and scrolling; long notes actual click/Enter branch preserved: at120x40 registered partial hit is unroutable and click inert then Enter opens viewer; each other size freezes its exact locate/clip outcome; notes paste never invents inline pending edits
- oracle_source: workbench.rs:818;app.rs:385,2485;src/widgets/grid.rs:570-578,1213-1235,1300-1345;db.rs:489-503,1010
- dependencies: grid/dialog/code

### TP-038

- surface: Grid fetch-more cap
- sizes: ALL
- fixture: T
- actions_and_checkpoints: G;cp(fetch_row);Enter;cp(fetch_status);Down;cp(bottom_boundary)
- assertions: Demo cap message, loading clears, no phantom rows or loop
- oracle_source: workbench.rs:799;src/widgets/grid.rs:1049
- dependencies: grid/pagination

### TP-039

- surface: Filter create edit cancel
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+F;cp(filter);focus(Value);Enter;type(1);Tab;click(Add filter);cp(chip);click(chip label);cp(edit);focus(Value);Enter;Ctrl+L;paste(2);Tab;click(Cancel);cp(original);click(chip label);focus(Value);Enter;Ctrl+L;paste(2);Tab;click(Update filter);cp(updated)
- assertions: Column default from cursor, update identity, cancellation and rows; field Tab commits without applying, button Add/Cancel/Update executes while Filter is still open; field Enter applies immediately as separately retained TP-040 source branch
- oracle_source: app.rs:1372,1657,1747;app.rs:237-246,1657-1785
- dependencies: filter/select/input/chips

### TP-040

- surface: Filter operators types
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+F;select(Column,total_amount);select(Operator,between);cp(between);focus(Value);Enter;Ctrl+L;paste(10);Tab;focus(and);Enter;Ctrl+L;paste(20);cp(upper_paste_ignored);type(20);cp(upper_typed);Tab;click(Add filter);cp(ranged);Ctrl+F;select(Column,status);select(Operator,is NULL);cp(no_value);click(Add filter);cp(null_filter); separate fresh T: Ctrl+F;select(Column,total_amount);select(Operator,between);focus(Value);replace(10);cp(lower_enter_applied_early)
- assertions: Operator ranking and second/no-value fields; upper-field bracketed paste consumed without insertion while lower draft remains10; typed20 produces upper bound; SQL/preview and range exact; separate lower replace(10) Enter applies early with empty upper bound and closes Filter
- oracle_source: tabs.rs:42;app.rs:2345;app.rs:237-246,1657-1785
- dependencies: select/form/filter

### TP-041

- surface: Filter invalid and empty
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+F;select(Column,id);focus(Value);Enter;Ctrl+L;paste(not-number);Tab;click(Add filter);cp(invalid_value_oracle);Esc;Ctrl+F;focus(Value);Enter;Ctrl+L;paste(999999999);Tab;click(Add filter);cp(no_rows);F;cp(reset)
- assertions: Exact source coercion: nonempty not-number is accepted into the filter rather than rejected by a new validator; no-results and clear restore; field Tab retains modal until explicit Add
- oracle_source: app.rs:1747;tabs.rs:478;app.rs:237-246,1657-1785
- dependencies: filter/empty

### TP-042

- surface: Filter chip controls ALL ANY
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,status);f;cp(cell_prefill);click(Add filter);Ctrl+F;select(Column,id);focus(Value);Enter;Ctrl+L;paste(1);Tab;click(Add filter);cp(all);cp(chip_focus_inventory);click(chips lead);cp(any);Enter;cp(chip_editor_not_lead);Esc;Space;cp(disabled);Delete;cp(removed);X;cp(cleared)
- assertions: Prefill and boolean composition; only ChipBar group is a focus stop, not lead/individual chip/clear-all; completed lead click toggles match_all and focuses group, group Enter edits first chip rather than activating lead, Space toggles, Delete removes, uppercase X clears; no invented clear-all button
- oracle_source: workbench.rs:702;tabs.rs:164,478;workbench.rs:729-756,1061-1080;src/widgets/chips.rs:76-77,88-145,245
- dependencies: chips/filter

### TP-043

- surface: Structure sections
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+D;cp(columns);focus(structure tabs);Right;Enter;cp(indexes);Right;Enter;cp(foreign_keys);Right;Enter;cp(ddl);PageDown;End;cp(ddl_bottom);Ctrl+D;cp(data)
- assertions: All four modes, code layout and preserved data cursor/state
- oracle_source: tabs.rs:535,759;app_tests.rs:245
- dependencies: tabs/table/code

### TP-044

- surface: Structure mouse and read-only
- sizes: ALL
- fixture: T
- actions_and_checkpoints: click(Structure mode);click(Indexes structure tab);hover(Name header);click(Name header);cp(sorted);wheelDown(structure)*8;cp(scrolled);+;-;Alt+D;cp(read_only_noop);click(Data mode);cp(data)
- assertions: Correct active tab marker, sorted schema grid, no row mutations
- oracle_source: workbench.rs:702,1040;app_tests.rs:266
- dependencies: tabs/table/grid

### TP-074

- surface: Minimum size and resize recovery
- sizes: 71x20,72x19,72x20,80x24,100x30,120x40,160x50
- fixture: T
- actions_and_checkpoints: cp(initial);resize(71,20);cp(too_narrow);Ctrl+T;cp(noop);resize(72,19);cp(too_short);resize(72,20);cp(minimum);resize(160,50);cp(recovered);resize(80,24);cp(drawer)
- assertions: Exact Need/Have copy, no clipped writes, preserved state and valid focus
- oracle_source: app.rs:491,2126;app_tests.rs:681
- dependencies: runtime/layout/focus

### TP-079

- surface: Grid view and typed catalog variants
- sizes: ALL
- fixture: W
- actions_and_checkpoints: for catalog tables customers,orders,events and each catalog view: Ctrl+O;type(exact object);Enter;cp(open);Home;End;Ctrl+End;cp(edge);Enter;cp(edit_or_readonly);Esc
- assertions: Typed cells, empty/null/JSON/bool/long text/reference and read-only views retain oracle metadata
- oracle_source: db.rs:397;tabs.rs:388;src/widgets/grid.rs:530
- dependencies: grid/domain

## Regression and trust closure

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
