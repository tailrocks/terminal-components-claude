# TASK-061 trusted obligations

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore SQL CodeEditor navigation/edit/selection/find/undo/completion, statement/selection/batch execution, result anchors and retention, errors/diagnostics, cancellation, EXPLAIN tree/raw/ANALYZE and editor/result split interaction.

Use shared public CodeEditor, completion/layers, Grid/Tree/Code result bodies and Split. Keep parser/classifier/executor, SQL candidates and stable result/tab identity in TablePro; background jobs cannot overwrite another target.

- Replace the fixed three-row TextInput and literal Explain/error messages in main app.rs:1815–1824 with fully interactive shared Editor and typed result views.
- Preserve seven ticks for first-statement completion and five for each subsequent statement, first-error batch stop, result anchors, pinned retention and no late completion after cancel.
- Ctrl+Y editing/global arbitration and statement-at-cursor versus selection are oracle behavior, not editor convenience changes. Exact diagnostic clearing, popup capture and separator constraints need action checkpoints.

## Exact membership and staging

Primary complete scenario IDs: TP-005, TP-016, TP-017, TP-018, TP-019, TP-020, TP-021, TP-022, TP-023, TP-024, TP-026, TP-045, TP-047, TP-048, TP-049, TP-051, TP-052, TP-053, TP-054, TP-055, TP-056, TP-057.

Legacy preservation edges protect previously closed work only. New-behavior acceptance additionally requires every exact flow contribution; no empty preservation intersection is sufficient.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires complete primary IDs TP-005, TP-016, TP-017, TP-018, TP-019, TP-020, TP-021, TP-022, TP-023, TP-024, TP-026, TP-045, TP-047, TP-048, TP-049, TP-051, TP-052, TP-053, TP-054, TP-055, TP-056, TP-057 and flow contributions for TP-046, TP-050 through the exact tables. Nonempty whole-frame and fixed semantic evidence is mandatory; full parents remain with the audited last producer.

## Source scenario clauses

### TP-005

- surface: Connect success stages
- sizes: ALL
- fixture: C
- actions_and_checkpoints: Down*8;Enter;cp(t0);ticks(3);cp(t3);ticks(4);cp(t7);ticks(4);cp(t11);ticks(1);cp(connected)
- assertions: SSH/authentication progress, spinner and successful Workbench transition
- oracle_source: connections.rs:384;app_tests.rs:125
- dependencies: progress/runtime/tree

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

### TP-026

- surface: No tabs
- sizes: ALL
- fixture: W
- actions_and_checkpoints: Ctrl+W until tab count zero;cp(empty);Ctrl+T;cp(new)
- assertions: No open tabs empty copy; explorer and new-query paths remain reachable
- oracle_source: workbench.rs:501,1277
- dependencies: empty/tabs

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

## Regression and trust closure

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
