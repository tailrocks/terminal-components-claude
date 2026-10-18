# TASK-063 trusted obligations

The complete TP-039–042 parents preserve the source Filter Enter/Tab distinction, ignored upper-field paste followed by real typed upper input, and source chip lead-pointer/group-keyboard routing. Consume TASK-060's nonempty contributions without replacing their source no-ops with the deferred F08a proposal. The separately named TP-040 early-Apply and TP-042 group-Enter variants are required, not optional diagnostics.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore History scope/search/failed-only/list/detail/scroll/copy/open/rerun, one reusable history tab, quick switcher four scopes and six target classes, help and tab-list search/delete/focus restoration.

Compose public TextInput/List/Code/Buttons/Picker/Dialog with the same action resolver used by hints and keymaps. Keep history records, ranking, SQL rerun/safety and tab identity application-owned.

- Replace the six-line Query history painter with actual query/list/detail/action controls; a model-only history test cannot prove the surface.
- Quick switcher no-match is not query-as-command; preserve exact Ctrl+P alias, clear-then-close, Alt+Enter and mouse target semantics.
- Tab-list deletion retains oracle close semantics while preserving internal target identity. Help and picker scrolling/capture restore the prior real focus owner.

## Exact membership and staging

Primary complete scenario IDs: TP-025, TP-027, TP-028, TP-029, TP-030, TP-031, TP-032, TP-033, TP-034, TP-035, TP-036, TP-037, TP-038, TP-039, TP-040, TP-041, TP-042, TP-043, TP-044, TP-046, TP-050, TP-063, TP-065, TP-066, TP-067, TP-068, TP-069, TP-070, TP-071, TP-072, TP-073, TP-074, TP-077, TP-079, TP-080.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires its complete local primary scenarios; no additional earlier-owner flow identities are assigned. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### TP-025

- surface: Tab list picker
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+T repeated 18 times;Ctrl+G;cp(picker);type(Query 2);cp(filtered);Enter;cp(switched);Ctrl+G;Delete;cp(deleted);Esc;cp(restored)
- assertions: Search, delete, active markers, focus restore; preserve oracle close semantics
- oracle_source: app.rs:1283,1444,1546
- dependencies: picker/tabs

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

### TP-046

- surface: Query editor selection paste undo
- sizes: ALL
- fixture: Q(SELECT * FROM orders LIMIT 5;)
- actions_and_checkpoints: i;Home;Shift+End;cp(selection);paste(SELECT id FROM customers LIMIT 2;);cp(replaced);Ctrl+Z;cp(undo);Ctrl+Y;cp(history_global_chord);Esc;cp(nav)
- assertions: Exact editing key arbitration, selection, undo/redo and global chord behavior
- oracle_source: tabs.rs:1333;app.rs:584
- dependencies: editor/action-routing

### TP-050

- surface: Query batch and stop error
- sizes: ALL
- fixture: QL(SELECT id FROM orders LIMIT 2;\nSELECT nope FROM orders;\nSELECT id FROM customers LIMIT 2;)
- actions_and_checkpoints: Alt+R;cp(batch_start);ticks(7);cp(first_complete);ticks(5);cp(second_failed);ticks(10);cp(stopped);Ctrl+Y;cp(history)
- assertions: Worst-risk gate, statement anchors, first-error stop, no third result/history
- oracle_source: tabs.rs:1028,1099;app.rs:818
- dependencies: query/domain/history

### TP-063

- surface: Safe-mode save refusal
- sizes: ALL
- fixture: T
- actions_and_checkpoints: cell(1,currency);Enter;Ctrl+L;paste(PARITY);Enter;Ctrl+L;End;Enter;Ctrl+S;cp(refused);p;cp(preview_still_available);Esc;cp(pending_kept)
- assertions: Read-only save message and preserved draft/pending state; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:987
- dependencies: domain/grid/dialog

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

### TP-077

- surface: Input routing paste and no-op
- sizes: ALL
- fixture: T
- actions_and_checkpoints: F10;cp(noop);open help;paste(SELECT 1);cp(no_leak);Esc;Ctrl+D;Alt+D;cp(structure_no_duplicate);Ctrl+D;cell(1,currency);Enter;Ctrl+L;type(q?z[]);cp(literal_input);Esc
- assertions: Unbound keys ignored; editing literals not global actions; no background paste; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:229,491;app_tests.rs:266
- dependencies: action-routing/editor/layer

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

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
