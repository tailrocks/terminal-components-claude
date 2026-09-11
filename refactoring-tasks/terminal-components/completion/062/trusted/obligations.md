# TASK-062 trusted obligations

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore mounted safety/save/discard/dirty-state overlays, six safety modes and exact facts/token gates, pending SQL acknowledgement, nested select capture, paste/key no-leak behavior and current-target execution ownership.

Use shared runtime Layer/Dialog/Form/Select/Button and retain TablePro stable target identities, generations, per-tab drafts and captured destructive scope. Keep risk classifier, safety policy and SQL execution application-owned.

- Immutable oracle controls user-visible confirmation paths. Retain main identity/generation checks internally without adding new prompts or changing oracle close/delete semantics.
- Execute only a still-valid captured target once; invalidation follows the existing oracle cancellation/status behavior. Add internal stale-target proofs separately from UI traces.
- An overlay enum or Surface label without a draw/update/dismissal path is not implemented. Exact acknowledgement, disabled Execute, backdrop capture and nested Escape/focus restoration must be real.

## Exact membership and staging

Primary complete scenario IDs: TP-058, TP-059, TP-060, TP-061, TP-062, TP-064.

Legacy preservation edges protect previously closed work only. New-behavior acceptance additionally requires every exact flow contribution; no empty preservation intersection is sufficient.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires TP-035, TP-036, TP-063, TP-065, TP-073, TP-077 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

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

### TP-073

- surface: Overlay resize and nested selects
- sizes: ALL
- fixture: T
- actions_and_checkpoints: Ctrl+F;select-open(Column);cp(nested);resize(80,24);cp(narrow);wheelDown(select)*8;cp(scroll);Esc;cp(select_closed);Esc;cp(filter_closed);resize(160,50);cp(restored)
- assertions: Topmost escape/paste/mouse/wheel capture, clipping and focus restoration
- oracle_source: app.rs:1657,1867,2345
- dependencies: select/layer/layout

### TP-077

- surface: Input routing paste and no-op
- sizes: ALL
- fixture: T
- actions_and_checkpoints: F10;cp(noop);open help;paste(SELECT 1);cp(no_leak);Esc;Ctrl+D;Alt+D;cp(structure_no_duplicate);Ctrl+D;cell(1,currency);Enter;Ctrl+L;type(q?z[]);cp(literal_input);Esc
- assertions: Unbound keys ignored; editing literals not global actions; no background paste; edit setup uses short currency source, not long notes viewer (retained TP-037)
- oracle_source: app.rs:229,491;app_tests.rs:266
- dependencies: action-routing/editor/layer

## Regression and trust closure

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
