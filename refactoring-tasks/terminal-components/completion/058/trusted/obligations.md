# TASK-058 trusted obligations

## Fixed ADJ-15 Select consumer

ADJ-15 is binding for Oracle src/bin/tablepro/connections.rs143,165 engine/group; on_form_key573+ dispatches focused Select at622/636. Compose the shared configured Select with navigation(SelectNavigation::Commit).open_keys(SelectOpenKeys::ConsumeUnhandled) in both phases, or the same LabelSelect props through the existing Form bridge. Consume existing Chose only: changed values produce one domain callback, clamped/equal-value choice produces none while preserving Changed/closure flow. Do not manually set a second Select value or duplicate navigation. Preserve exact source open Tab/BackTab consumption, closed traversal, outer global-chord precedence, Esc, external focus-out, disabled/read-only and option rebuild behavior. Generic unrelated Select defaults remain unchanged. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

## Re-audit total validation contract

ADJ-12 binds original F10 and oracle connections.rs:246–250,1131,1145–1153. A submitted form with both invalid name and invalid port must show both field errors while focusing the first; all visible field validators run once in declaration order and cross-validation is gated on their complete success. Consume TASK-019's shared error collection rather than app-local duplicate validation. Preserve exact source field text, errors, tab state, focus and blocked submission through R-001/AC-001/CHK-004 and R-003/AC-003/CHK-005.

## Re-audit shared API consumer contract

ADJ-10: connection RadioGroup/Form consumers preserve oracle navigation commits using the shared typed event or Form::radio_navigation_commits(true), including exact boundary behavior and controlled props in both phases. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005. Direct and full application traces must prove the actual source composition; an API declaration or compile-only adaptation does not close parity.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/tablepro/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-005/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore Connections grouped tree/filter/detail, separate Connect/Edit/Duplicate/Delete/Retry controls, Basic/Advanced form, secret/paste, validation, test outcomes, save/cancel/save-connect, empty list and live identity-strip actions.

Use public Tree/TextInput, Form/Select/choice/TextArea, Buttons, Progress and StatusBar segments through runtime focus/hit ownership. Keep connection records, test jobs, validation policy and in-memory demo effects in TablePro.

- Replace the broad CONNECTION_DETAILS connect hit region with individually identified controls matching oracle connections.rs:509; a Surface label is not a mounted form.
- Preserve 12-tick connection completion and 10-tick test completion, engine-specific port defaults, SSL/SSH disclosure and exact password representation.
- Production is a fixture name. Do not add live database, SSH, keychain, clipboard or persistent filesystem operations; connection-strip actions remain real routed controls.

## Exact membership and staging

Primary complete scenario IDs: TP-001, TP-002, TP-003, TP-004, TP-006, TP-007, TP-008, TP-009, TP-010, TP-011, TP-012, TP-013, TP-014, TP-015.

Legacy preservation edges protect previously closed work only. New-behavior acceptance additionally requires every exact flow contribution; no empty preservation intersection is sufficient.

The baseline expansion algorithms and finite axes in tablepro.md and tablepro-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Expand ALL at 80x24,100x30,120x40,160x50 and all listed boundaries. C/W/L/T/Q/QL presets are exact event sequences, not candidate state injection. Preserve exact first/subsequent query and connection/test tick counts. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires TP-005, TP-016 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

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

## Regression and trust closure

Run every required source-qualified TablePro and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
