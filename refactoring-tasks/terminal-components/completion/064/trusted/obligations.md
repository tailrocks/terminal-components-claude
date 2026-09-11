# TASK-064 trusted obligations

Closure reruns the intact repaired TP-039–042 parents and all TASK-060 semantic/whole-frame contribution checkpoints, including upper paste no-op, typed upper value, lower-Enter early Apply, chip focus inventory and group Enter editing. Preserve source input routing and deferred F08a disposition; no invented lead/clear-all focus target or whole-app receipt may substitute for these observations.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Prove all 80 TP scenarios and every frozen fixture, geometry, color/NO_COLOR and checkpoint expansion pass together on one integrated TablePro tree, including exact timing, lifecycle and compatible stable-identity/safety tests.

Independently prove every declared surface has mounted controls and dispatched actions, exact oracle rendering uses public components and SQL/safety/domain decisions remain in TablePro.

- This is validation-only. Source fixes return to their responsible production owner; no expected-output update, partial fixture acceptance or verification rule change is allowed.
- The TablePro unresolved-failure set must be empty. Execute every required TablePro target and compatible current identity/generation/read-only assertion, not only static digest matrices.
- Flash 139/140ms and status 4999/5001ms are exact controlled-clock observations; real CLI PTY proves lifecycle, encoding and reachable terminal states without equating wall sleeps with precise phases.

## Exact membership and staging

Primary complete scenario IDs: TP-075, TP-076, TP-078.

Closure additionally requires every TP source scenario below, every finite variant and all checkpoints, regardless of earlier primary ownership. No TablePro unresolved failure remains. All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires TP-005, TP-016, TP-026, TP-046, TP-050, TP-027, TP-028, TP-029, TP-030, TP-031, TP-032, TP-033, TP-035, TP-036, TP-037, TP-038, TP-039, TP-040, TP-041, TP-042, TP-043, TP-044, TP-063, TP-065, TP-073, TP-074, TP-077, TP-079 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### TP-001

- surface: CLI
- sizes: ALL
- fixture: C
- actions_and_checkpoints: launch;cp(initial);--help subprocess;--color bad subprocess;--connect missing subprocess
- assertions: Help text and exit statuses exact; initial focus/filter/tree/detail/strip/footer exact
- oracle_source: main.rs:24;connections.rs:918
- dependencies: CLI/runtime/panel/tree/input

### TP-002

- surface: Connections navigation
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Down*8;cp(production);Home;End;Up;Tab;BackTab;cp(traversal)
- assertions: Grouped order, selection distinct from focus; detail changes; full focus ring matches
- oracle_source: app_tests.rs:125;connections.rs:440
- dependencies: tree/focus/footer

### TP-003

- surface: Connection filter
- sizes: ALL
- fixture: C
- actions_and_checkpoints: /;type(Production);cp(filtered);Esc;cp(cancelled);/;type(zzzz_no_match);cp(empty);Esc;Down;cp(restored)
- assertions: Text editing cancellation and no-match details/disabled actions match
- oracle_source: connections.rs:440,327
- dependencies: input/tree/empty

### TP-004

- surface: Connection hover and actions
- sizes: ALL
- fixture: C
- actions_and_checkpoints: hover(Local PostgreSQL);cp(hover);Down;cp(keyboard_suppresses_hover);hover(Connect);down;cp(pressed);up;cp(connecting)
- assertions: Hover/press/focus and activation hit regions exact
- oracle_source: app.rs:211,1867;connections.rs:738
- dependencies: button/hit/focus

### TP-005

- surface: Connect success stages
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Down*8;Enter;cp(t0);ticks(3);cp(t3);ticks(4);cp(t7);ticks(4);cp(t11);ticks(1);cp(connected)
- assertions: SSH/authentication progress, spinner and successful Workbench transition
- oracle_source: connections.rs:384;app_tests.rs:125
- dependencies: progress/runtime/tree

### TP-006

- surface: Connection failures retry
- sizes: ALL
- fixture: C
- actions_and_checkpoints: /;type(Staging);Enter;Down;Enter;ticks(12);cp(auth_failed);click(Reconnect);cp(retry);ticks(12);cp(failed_again)
- assertions: Authentication failure detail and retry behavior exact
- oracle_source: connections.rs:384,509
- dependencies: error/button/runtime

### TP-007

- surface: Connection unreachable
- sizes: ALL
- fixture: C
- actions_and_checkpoints: /;type(Analytics);Enter;Down;Enter;ticks(12);cp(unreachable);click(Reconnect);ticks(12);cp(retry_failed)
- assertions: Host timeout message, detail and retry preserve selection
- oracle_source: connections.rs:384;app_tests.rs:148
- dependencies: error/button

### TP-008

- surface: Connection new invalid
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;cp(new_basic);Ctrl+S;cp(invalid);focus(Port);replace(65536);Ctrl+S;cp(invalid_port);focus(Port);replace(0);Ctrl+S;cp(zero_port)
- assertions: Required and exact port error, focus and disabled/submission behavior
- oracle_source: connections.rs:89,246,573
- dependencies: form/validation

### TP-009

- surface: Connection basic choices
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;focus(Name);replace(Parity fixture);select(Engine,MySQL);cp(mysql_port);select(Engine,SQLite);cp(sqlite_port);select(Engine,PostgreSQL);select(Group,Acme);focus(Environment);End;Space;focus(Safe Mode);End;Space;cp(choices)
- assertions: Engine port default; group/environment/safety selection and focus
- oracle_source: connections.rs:108,573
- dependencies: select/radio/form

### TP-010

- surface: Connection password and paste
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;focus(Password);Enter;paste(päss秘密);cp(password_field);Esc;focus(Prompt for password);Space;cp(prompt_toggle)
- assertions: Password field representation, grapheme cursor and checkbox exact; fixture text is synthetic
- oracle_source: connections.rs:108,860
- dependencies: input/checkbox

### TP-011

- surface: Connection advanced
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;focus(BASIC/ADVANCED tabs);Right;Enter;cp(advanced);focus(SSL);Space;focus(SSH);Space;cp(ssh_enabled);focus(SSH host);replace(bastion.fixture);focus(SSH user);replace(operator);focus(Startup);Enter;paste(SET search_path TO public;);Esc;focus(Local only);Space;cp(advanced_filled)
- assertions: Disclosure focus order, toggles, SSH fields and startup textarea
- oracle_source: connections.rs:62,573,1117
- dependencies: tabs/form/textarea/toggle

### TP-012

- surface: Connection test success and failure
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;focus(Host);replace(localhost);click(Test connection);cp(testing);ticks(10);cp(success);focus(Host);replace(analytics.acme.io);click(Test connection);ticks(10);cp(test_error)
- assertions: 10-tick deterministic test; matching outcome text and focus
- oracle_source: connections.rs:384,573
- dependencies: form/progress/error

### TP-013

- surface: Connection save edit cancel
- sizes: ALL
- fixture: C
- actions_and_checkpoints: click(Edit);cp(edit);focus(Name);replace(Local parity);click(Cancel);cp(cancelled);click(Edit);focus(Name);replace(Local parity);Ctrl+S;cp(saved)
- assertions: Cancel restores original; save edits selected record instead of creating duplicate
- oracle_source: connections.rs:509,689
- dependencies: form/input/button

### TP-014

- surface: Connection duplicate delete
- sizes: ALL
- fixture: C
- actions_and_checkpoints: click(Duplicate);cp(copy);click(Delete);cp(delete_prompt);Esc;cp(kept);click(Delete);focus(Delete dialog action);Enter;cp(deleted)
- assertions: Copy naming/ordering, destructive facts/action focus and deletion selection
- oracle_source: connections.rs:509,551
- dependencies: dialog/button/tree

### TP-015

- surface: Connection empty list
- sizes: 120x40
- fixture: C
- actions_and_checkpoints: repeat for six original connections: select first connection;click(Delete);focus(Delete dialog action);Enter;cp(after_each_delete);cp(empty);Ctrl+N;cp(new_from_empty)
- assertions: Final empty list no stale detail/hit/focus; new remains reachable
- oracle_source: connections.rs:551,918
- dependencies: empty/tree/focus

### TP-016

- surface: Connection save and connect
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Ctrl+N;focus(Name);replace(Parity local);focus(Host);replace(localhost);focus(Database);replace(acme_dev);focus(User);replace(postgres);click(Save & Connect);cp(connecting);ticks(12);cp(workbench)
- assertions: Saved connection identity and workbench state exact; no external DB effect
- oracle_source: connections.rs:689
- dependencies: form/runtime

### TP-017

- surface: Workbench initial and focus ring
- sizes: ALL
- fixture: W
- actions_and_checkpoints: cp(initial);Tab through complete oracle focus cycle with cp(each);BackTab through reverse cycle with cp(each)
- assertions: All visible and zero-area focus stops; borders and footer hints
- oracle_source: workbench.rs:99,1208;app.rs:558
- dependencies: focus/panel/footer

### TP-018

- surface: Explorer lazy expand
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Right;cp(expanded_public_navigation);End;cp(lazy_audit_selected);Right;cp(expanding);ticks(1);cp(loading);ticks(2);cp(loaded);Left;cp(collapsed);Home;End;PageUp;PageDown;cp(boundary)
- assertions: Lazy hierarchy, icons, metadata, load frames and scroll boundaries
- oracle_source: workbench.rs:133,200,332,604
- dependencies: tree/scrollbar

### TP-019

- surface: Explorer filter refresh schema
- sizes: ALL
- fixture: W
- actions_and_checkpoints: /;type(orders);cp(filtered);Esc;cp(cancelled);/;type(zzzz_no_match);cp(empty);Esc;0;r;cp(refreshed);activate(schema audit);cp(schema_changed)
- assertions: Filter restoration, refresh status and schema activation
- oracle_source: workbench.rs:604;app.rs:779
- dependencies: input/tree

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

### TP-022

- surface: Explorer drawer
- sizes: 80x24,100x30,101x30,102x30
- fixture: W
- actions_and_checkpoints: cp(explorer);Ctrl+T;cp(editor);0;cp(reopened);Tab until query editor;cp(closed);Ctrl+B;cp(hidden);Ctrl+B;cp(shown)
- assertions: Drawer threshold terminal width 102; preserved hidden focus stop and full-width body
- oracle_source: workbench.rs:1208;app_tests.rs:732
- dependencies: layout/focus

### TP-023

- surface: Workbench maximize
- sizes: ALL
- fixture: Q(SELECT * FROM orders LIMIT 5)
- actions_and_checkpoints: z;cp(editor_max);Esc;Ctrl+R;ticks(7);focus(result grid);z;cp(results_max);Esc;cp(restored);0;z;cp(explorer_max_case)
- assertions: Pane-dependent maximize, explorer visibility and Esc restoration
- oracle_source: app.rs:708,779
- dependencies: split/layout/focus

### TP-024

- surface: Tab cycling overflow
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+T repeated 18 times;cp(overflow);[;cp(previous);];cp(next);focus(tab strip);Home;End;Left;Right;cp(strip_navigation);Ctrl+W;cp(closed)
- assertions: Overflow glyphs, active/cursor, stable state and close focus
- oracle_source: app_tests.rs:573;app.rs:625,729
- dependencies: tabs/identity/focus

### TP-025

- surface: Tab list picker
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+T repeated 18 times;Ctrl+G;cp(picker);type(Query 2);cp(filtered);Enter;cp(switched);Ctrl+G;Delete;cp(deleted);Esc;cp(restored)
- assertions: Search, delete, active markers, focus restore; preserve oracle close semantics
- oracle_source: app.rs:1283,1444,1546
- dependencies: picker/tabs

### TP-026

- surface: No tabs
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+W until tab count zero;cp(empty);Ctrl+T;cp(new)
- assertions: No open tabs empty copy; explorer and new-query paths remain reachable
- oracle_source: workbench.rs:501,1277
- dependencies: empty/tabs

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

### TP-034

- surface: Grid pending save preview
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY-1);Enter;p;cp(sql_preview);Esc;Ctrl+S;cp(save_gate);focus(acknowledgement);Enter;type(orders);Enter;cp(armed);focus(Save);Enter;cp(saving);ticks(4);cp(still_pending);ticks(1);cp(saved);Ctrl+Y;cp(history_entry)
- assertions: Exact UPDATE public.orders SET currency = 'PARITY-1', facts/token, pending count clears and RowEdits history; original long notes behavior retained TP-037
- oracle_source: app_tests.rs:609;app.rs:987,1113,1155
- dependencies: dialog/grid/history

### TP-035

- surface: Grid mixed SQL and delete save
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY-1);Enter;+;Alt+D;-;p;cp(mixed_preview);Esc;Ctrl+S;cp(delete_review);Esc;cp(pending_preserved)
- assertions: Update/insert/delete ordering, escaping, destructive title and Cancel; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: model.rs:12,959;app.rs:987
- dependencies: grid/dialog/domain

### TP-036

- surface: Grid discard refresh
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY-1);Enter;F5;cp(refresh_gate);Esc;cp(kept);U;cp(discard_gate);focus(Discard);Enter;cp(discarded);F5;cp(refreshed)
- assertions: Dirty refresh gating, discard restoration, focus and footer; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: workbench.rs:778;app.rs:1797
- dependencies: grid/dialog

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

### TP-045

- surface: Query editor multiline and find
- sizes: ALL
- fixture: Q(SELECT id, order_number\nFROM orders\nWHERE id > 10;)
- actions_and_checkpoints: cp(nav);i;End;Enter;paste(-- 文本 é 👩‍💻);cp(edit);Esc;Ctrl+F;type(orders);cp(find);Enter;cp(match);Esc;cp(find_closed);Ctrl+Up;cp(split_up);Ctrl+Down;cp(split_down)
- assertions: SQL colors, gutters, statements, grapheme cursor, find highlighting and split
- oracle_source: tabs.rs:972,1333;app.rs:695,759
- dependencies: editor/split

### TP-046

- surface: Query editor selection paste undo
- sizes: ALL
- fixture: Q(SELECT * FROM orders LIMIT 5;)
- actions_and_checkpoints: i;Home;Shift+End;cp(selection);paste(SELECT id FROM customers LIMIT 2;);cp(replaced);Ctrl+Z;cp(undo);Ctrl+Y;cp(history_global_chord);Esc;cp(nav)
- assertions: Exact editing key arbitration, selection, undo/redo and global chord behavior
- oracle_source: tabs.rs:1333;app.rs:584
- dependencies: editor/action-routing

### TP-047

- surface: Completion auto manual accept
- sizes: ALL
- fixture: Q()
- actions_and_checkpoints: i;type(SELECT * FROM ord);cp(auto);Down;cp(selected);Enter;cp(accepted);type( WHERE );Ctrl+Space;cp(manual);Esc;cp(dismissed);Esc;cp(nav)
- assertions: Typed token replacement, contextual candidates, selected row/details and cursor
- oracle_source: app_tests.rs:307;tabs.rs:1233,1333;model.rs:567
- dependencies: editor/completion

### TP-048

- surface: Completion aliases schemas mouse bounds
- sizes: ALL
- fixture: Q()
- actions_and_checkpoints: i;type(SELECT o. FROM orders o);Home;Right*9;Ctrl+Space;cp(alias);hover(completion candidate);cp(hover);click(completion candidate);cp(accepted);Esc;Ctrl+F;cp(find_no_popup_leak)
- assertions: Alias detail, anchor geometry, clipping/scroll and popup isolation
- oracle_source: model.rs:478,567;tabs.rs:1444
- dependencies: completion/layer/editor

### TP-049

- surface: Query run statement and empty
- sizes: ALL
- fixture: Q()
- actions_and_checkpoints: Ctrl+R;cp(nothing);i;type(SELECT * FROM orders LIMIT 5;);Esc;Ctrl+R;cp(running0);ticks(3);cp(running3);Ctrl+R;cp(already_running);ticks(4);cp(rows)
- assertions: Empty/duplicate-run statuses, running range, 7-tick completion and result metadata
- oracle_source: app.rs:818;tabs.rs:1053,1099
- dependencies: editor/result/runtime

### TP-050

- surface: Query batch and stop error
- sizes: ALL
- fixture: QL(SELECT id FROM orders LIMIT 2;\nSELECT nope FROM orders;\nSELECT id FROM customers LIMIT 2;)
- actions_and_checkpoints: Alt+R;cp(batch_start);ticks(7);cp(first_complete);ticks(5);cp(second_failed);ticks(10);cp(stopped);Ctrl+Y;cp(history)
- assertions: Worst-risk gate, statement anchors, first-error stop, no third result/history
- oracle_source: tabs.rs:1028,1099;app.rs:818
- dependencies: query/domain/history

### TP-051

- surface: Query cursor or selection execution
- sizes: ALL
- fixture: QL(SELECT id FROM orders LIMIT 2;\nSELECT id FROM customers LIMIT 3;)
- actions_and_checkpoints: i;Home;Ctrl+Home;Ctrl+R;cp(first_running);ticks(7);cp(first_rows);Esc;Down;Home;Shift+End;Ctrl+R;ticks(7);cp(selection_rows)
- assertions: Statement-at-cursor vs nonempty selection and result anchors exact
- oracle_source: tabs.rs:1028
- dependencies: editor/query

### TP-052

- surface: Query execution errors
- sizes: ALL
- fixture: QL(SELECT nope FROM orders)
- actions_and_checkpoints: Ctrl+R;ticks(7);cp(unknown_column);i;replace(SELEC broken);Esc;Ctrl+R;ticks(7);cp(parse_error);i;type(x);cp(diagnostic_cleared)
- assertions: Specific error SQLSTATE/position, underline and result body; editing clears diagnostics
- oracle_source: app_tests.rs:351;tabs.rs:1099
- dependencies: editor/error/result

### TP-053

- surface: Query cancellation
- sizes: ALL
- fixture: Q(SELECT * FROM orders)
- actions_and_checkpoints: Ctrl+R;ticks(3);Esc;cp(cancelled);Ctrl+R;ticks(2);Ctrl+C;cp(ctrl_c_cancelled);ticks(10);cp(no_late_result)
- assertions: Cancel result anchor/duration and status; no quit or later completion
- oracle_source: app_tests.rs:366;tabs.rs:1080
- dependencies: runtime/query

### TP-054

- surface: Query results pinned and close
- sizes: ALL
- fixture: Q(SELECT * FROM orders LIMIT 5)
- actions_and_checkpoints: Ctrl+R;ticks(7);focus(result tabs);p;cp(pinned);Ctrl+R;ticks(7);cp(retained);focus(result tabs);Left;Enter;cp(active_first);.;cp(unpinned);x;cp(closed)
- assertions: Pinned retention/replacement, active-result identity and tab focus
- oracle_source: tabs.rs:1053,1179,1198,1216,1333
- dependencies: tabs/result/identity

### TP-055

- surface: Query explain tree raw analyze
- sizes: ALL
- fixture: Q(SELECT * FROM orders WHERE id > 10 ORDER BY id LIMIT 5)
- actions_and_checkpoints: Ctrl+X;ticks(7);cp(plan_tree);focus(plan tree);Right;Down;cp(expanded);r;cp(raw);PageDown;cp(raw_scroll);r;cp(tree_return);Alt+X;ticks(7);cp(analyze)
- assertions: Plan tree cost/timing badges, raw JSON, analyze flag and split
- oracle_source: app_tests.rs:384;tabs.rs:1333,1553
- dependencies: tree/code/query

### TP-056

- surface: Query result mouse and split drag
- sizes: ALL
- fixture: Q(SELECT * FROM orders LIMIT 25)
- actions_and_checkpoints: Ctrl+R;ticks(7);hover(result header);cp(hover);click(result cell);wheelDown(result grid)*6;cp(scroll);drag(editor/result separator,8 rows down);cp(resized);drag(separator,top boundary);cp(clamped)
- assertions: Grid readonly behavior; separator constraints and cursor/hit reflow
- oracle_source: tabs.rs:1444,1503,1518
- dependencies: split/grid/hit

### TP-057

- surface: Query affected and zero rows
- sizes: ALL
- fixture: QL(UPDATE orders SET status = 'paid' WHERE id = 1)
- actions_and_checkpoints: Ctrl+R;ticks(7);cp(affected);Ctrl+T;i;type(SELECT * FROM orders WHERE id = 999999999);Esc;Ctrl+R;ticks(7);cp(zero_rows)
- assertions: Affected-row copy and empty result retain headers/status as oracle
- oracle_source: tabs.rs:1553;sql.rs:888
- dependencies: result/empty

### TP-058

- surface: Safety six modes matrix
- sizes: ALL
- fixture: W
- actions_and_checkpoints: for each mode fresh W: Ctrl+L;Home;Down*mode_index;Enter;cp(mode);Q(SELECT id FROM orders LIMIT 1);Ctrl+R;cp(read_gate);if modal: Esc; otherwise ticks(7);Q(UPDATE orders SET status = 'paid' WHERE id = 1);Ctrl+R;cp(write_gate);if modal: Esc; otherwise ticks(7);Q(DELETE FROM orders);Ctrl+R;cp(danger_gate)
- assertions: Six levels × read/scoped-write/unscoped-delete: deny/run/confirm/deliberate matches sql::gate; snapshot every gate
- oracle_source: db.rs:35;sql.rs:735;app.rs:818,1341
- dependencies: picker/dialog/domain

### TP-059

- surface: Safety token keyboard
- sizes: ALL
- fixture: Q(DELETE FROM orders)
- actions_and_checkpoints: Ctrl+R;cp(gate);Enter;type(wrong);Enter;cp(disabled);focus(Execute);Enter;cp(still_open);focus(acknowledgement);replace(orders);cp(armed);focus(Execute);Enter;cp(running);ticks(7);cp(executed)
- assertions: Exact token validation, disabled action focus, target/risk and execution once
- oracle_source: app_tests.rs:438;app.rs:818
- dependencies: dialog/input/button

### TP-060

- surface: Safety token mouse paste
- sizes: ALL
- fixture: Q(DELETE FROM orders)
- actions_and_checkpoints: Ctrl+R;click(acknowledgement);cp(edit);paste(orders);cp(armed);hover(Execute);down;cp(pressed);up;cp(running);ticks(7);cp(executed)
- assertions: Mouse focus/edit/confirm route; password-like token field not skipped
- oracle_source: app_tests.rs:459
- dependencies: dialog/input/hit

### TP-061

- surface: Safety cancel and backdrop
- sizes: ALL
- fixture: Q(DELETE FROM orders)
- actions_and_checkpoints: Ctrl+R;cp(gate);click(background grid);cp(backdrop);Ctrl+T;cp(no_new_tab);Esc;cp(cancelled);ticks(20);cp(no_execution)
- assertions: Overlay prevents lower-layer actions; cancellation drops pending execution
- oracle_source: app.rs:1444,1867,1797
- dependencies: layer/focus/domain

### TP-062

- surface: Safe mode picker persist
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+L;cp(picker);Home;Down*5;Enter;cp(readonly);Ctrl+L;Esc;cp(kept);click(connection strip);cp(connection_screen_oracle);/;type(Production);Enter;Down;Enter;ticks(12);cp(persisted)
- assertions: Current arrow/tag, wrapped descriptions, focus restoration, saved record
- oracle_source: app_tests.rs:648;app.rs:1341,1546
- dependencies: picker/identity

### TP-063

- surface: Safe-mode save refusal
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY);Enter;Ctrl+L;End;Enter;Ctrl+S;cp(refused);p;cp(preview_still_available);Esc;cp(pending_kept)
- assertions: Read-only save message and preserved draft/pending state; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:987
- dependencies: domain/grid/dialog

### TP-064

- surface: Dirty tab close and quit
- sizes: ALL
- fixture: Q(SELECT 1)
- actions_and_checkpoints: i;End;type( );Esc;Ctrl+W;cp(close_gate);Esc;cp(kept);q;cp(quit_gate);Esc;cp(kept_again);Ctrl+C;cp(ctrl_c_gate);focus(Quit);Enter;assert_exit
- assertions: Dirty query count, destructive copy, focus and terminal restoration
- oracle_source: app.rs:274,625;workbench.rs:959
- dependencies: dialog/runtime

### TP-065

- surface: Multiple dirty ownership
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY);Enter;Ctrl+T;i;type(SELECT 1);Esc;q;cp(mixed_dirty);Esc;[;cp(original_grid);];cp(original_query)
- assertions: Pending rows + unsaved queries count; per-tab cursor/draft preserved; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:274;workbench.rs:501
- dependencies: identity/domain/dialog

### TP-066

- surface: History search scope failed
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+Y;cp(history);/;type(orders error);cp(search);Esc;cp(cancelled);s;cp(failed);c;cp(all_connections);/;type(zzzz_no_match);cp(empty);Esc;Home;End;PageUp;PageDown;cp(scroll)
- assertions: Multi-term AND, scopes, error colors, detail and list scroll/fades
- oracle_source: model.rs:396;workbench.rs:861;tabs.rs:2272
- dependencies: list/input/history

### TP-067

- surface: History open rerun copy
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+Y;Down;cp(detail);y;cp(copy);Enter;cp(opened);Ctrl+Y;cp(reused_history);r;cp(rerun_or_gate);if modal: Esc; otherwise ticks(7);Ctrl+Y;click(Copy);cp(mouse_copy);click(Open);cp(mouse_open)
- assertions: Selected SQL, new-query focus, rerun safety and one history tab
- oracle_source: app_tests.rs:556;workbench.rs:861
- dependencies: history/button/query

### TP-068

- surface: History detail mouse scroll
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+Y;click(history row 3);cp(selected);wheelDown(history list)*8;cp(list_scroll);click(SQL detail);wheelDown(SQL detail)*8;cp(detail_scroll);drag(history scrollbar,bottom);cp(bottom)
- assertions: Independent list/detail scroll and focused borders
- oracle_source: workbench.rs:1100,1163;tabs.rs:2386
- dependencies: list/code/scroll

### TP-069

- surface: Quick switcher search and scopes
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+O;cp(open);type(ord);cp(ranked);Tab;cp(scope1);Tab;cp(scope2);Tab;cp(scope3);Tab;cp(scope0);Esc;cp(clear);Esc;cp(closed);Ctrl+P;cp(alias)
- assertions: Fuzzy highlights, four scopes, clear-then-close and alias Ctrl+P
- oracle_source: app.rs:1216,1237,1444;model.rs:824
- dependencies: picker/search

### TP-070

- surface: Quick switcher target classes
- sizes: ALL
- fixture: W
- actions_and_checkpoints: for each target table orders,view active_customers,schema audit,database acme_prod,open Query 1,recent query: fresh W;create target if open-tab;Ctrl+O;type(exact target);cp(candidate);Enter;cp(opened)
- assertions: Table/view/schema/database/open-tab/recent-query actions and focus, exact names validated from catalog/index
- oracle_source: app.rs:1546;model.rs:829
- dependencies: picker/domain/identity

### TP-071

- surface: Quick switcher empty alt mouse
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+O;type(zzzz_no_match);cp(empty);Enter;cp(stays_open);Esc;type(orders);Alt+Enter;cp(opened_alt);Ctrl+O;type(customers);hover(first candidate);cp(hover);click(first candidate);cp(opened_mouse); separate fresh W for each picker chord Ctrl+O,Ctrl+G: Ctrl+T;i;type(SELECT );picker chord;cp(picker_over_editing_query);paste(PARITY);cp(paste_fallthrough);Esc;cp(editor_revealed)
- assertions: No query-as-command; Alt target oracle semantics; click and hover; exact oracle Picker paste fallthrough appends PARITY to underlying editing Query while picker query remains empty; Dialog/Filter routing stays separately qualified
- oracle_source: app.rs:236-254,638-645,1232-1234,1337-1338,1444,1546,1867;workbench.rs:1194;tabs.rs:1543
- dependencies: picker/layer/hit

### TP-072

- surface: Help overlay scrolling
- sizes: ALL
- fixture: C and W
- actions_and_checkpoints: ?;cp(help);Tab through full dialog focus cycle;PageDown;cp(scrolled);End;cp(bottom);wheelUp(dialog);cp(wheel);Esc;cp(restored)
- assertions: Help text/glyphs, scroll, focus trap and prior focus restore
- oracle_source: app.rs:1195,1444,1867
- dependencies: dialog/code/focus

### TP-073

- surface: Overlay resize and nested selects
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+F;select-open(Column);cp(nested);resize(80,24);cp(narrow);wheelDown(select)*8;cp(scroll);Esc;cp(select_closed);Esc;cp(filter_closed);resize(160,50);cp(restored)
- assertions: Topmost escape/paste/mouse/wheel capture, clipping and focus restoration
- oracle_source: app.rs:1657,1867,2345
- dependencies: select/layer/layout

### TP-074

- surface: Minimum size and resize recovery
- sizes: 71x20,72x19,72x20,80x24,100x30,120x40,160x50
- fixture: T
- actions_and_checkpoints: cp(initial);resize(71,20);cp(too_narrow);Ctrl+T;cp(noop);resize(72,19);cp(too_short);resize(72,20);cp(minimum);resize(160,50);cp(recovered);resize(80,24);cp(drawer)
- assertions: Exact Need/Have copy, no clipped writes, preserved state and valid focus
- oracle_source: app.rs:491,2126;app_tests.rs:681
- dependencies: runtime/layout/focus

### TP-075

- surface: Color capability and NO_COLOR
- sizes: ALL
- fixture: C and T and Q(SELECT * FROM orders LIMIT 5)
- actions_and_checkpoints: cp(initial);focus(primary);hover(primary);cp(hover);activate editing;select text;cp(selected);open overlay;cp(overlay)
- assertions: All five color profiles, fallback emphasis/reverse video, exact glyph/background/foreground and cursor
- oracle_source: main.rs:24;app.rs:211;tools/audit_flows.sh
- dependencies: theme/style/runtime

### TP-076

- surface: Transient flash status timing
- sizes: 120x40
- fixture: T
- actions_and_checkpoints: activate sortable header;cp(t0);advance_clock(139ms);cp(flash139);advance_clock(1ms);Tick;cp(flash140);advance_clock(to4999ms);cp(status4999);advance_clock(to5001ms);Tick;cp(status_expired)
- assertions: Press flash and status expiry exact; controlled-clock fixture required, no masking
- oracle_source: app.rs:211,317
- dependencies: runtime/clock/button/footer

### TP-077

- surface: Input routing paste and no-op
- sizes: ALL
- fixture: T
- actions_and_checkpoints: F10;cp(noop);open help;paste(SELECT 1);cp(no_leak);Esc;Ctrl+D;Alt+D;cp(structure_no_duplicate);Ctrl+D;cell(1,currency);Enter;Ctrl+L;type(q?z[]);cp(literal_input);Esc
- assertions: Unbound keys ignored; editing literals not global actions; no background paste; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:229,491;app_tests.rs:266
- dependencies: action-routing/editor/layer

### TP-078

- surface: Terminal lifecycle
- sizes: 120x40
- fixture: C
- actions_and_checkpoints: q;assert_exit;fresh C;Ctrl+C;assert_exit;fresh W;resize(60,15);q;assert_exit
- assertions: Exit status, cursor visibility, raw/alternate-screen/mouse/bracketed-paste teardown match runtime contract
- oracle_source: main.rs:57;app.rs:229,491
- dependencies: runtime

### TP-079

- surface: Grid view and typed catalog variants
- sizes: ALL
- fixture: W
- actions_and_checkpoints: for catalog tables customers,orders,events and each catalog view: Ctrl+O;type(exact object);Enter;cp(open);Home;End;Ctrl+End;cp(edge);Enter;cp(edit_or_readonly);Esc
- assertions: Typed cells, empty/null/JSON/bool/long text/reference and read-only views retain oracle metadata
- oracle_source: db.rs:397;tabs.rs:388;src/widgets/grid.rs:530
- dependencies: grid/domain

### TP-080

- surface: Connection strip segments
- sizes: ALL
- fixture: W
- actions_and_checkpoints: hover(Safe Mode strip);cp(hover_safe);click(Safe Mode strip);cp(safety);Esc;click(scope strip);cp(scope_switcher);Esc;click(help strip);cp(help);Esc;click(connection strip);cp(connections)
- assertions: Each identity segment owns its action, hitbox, tooltip/status and restored focus
- oracle_source: app.rs:1867,2197
- dependencies: segments/picker/dialog

## Regression and trust closure

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; none may remain for this app. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
