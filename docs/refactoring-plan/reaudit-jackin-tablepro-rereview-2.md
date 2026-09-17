# Independent witness: Jackin/TablePro post-rereview corrections

> Historical planning evidence. Not current execution authority. Do not replay
> its commands, branch/pin/model/merge/container instructions. Reconcile this
> record against the current readiness report and current contracts.

2026-09-15. Fresh independent reviewer; did not author `reaudit-jackin-tablepro-rereview.md` repairs or prior JT planning edits. Read `docs/sources/PLANNING_GOAL.md` scope via assigned finding JT-JT01-JT06 and verify-and-stop bounds. Only this report and the closed finding row were written; no canonical scenario, task package, product source, or baseline was edited by this reviewer.

Oracle pin: `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Architecture main: `7b27732a8c3c131760ec3438f641cb3c11343a42`.

## Verdict

**ACCEPTED.** Post-rereview JTR-01/JTR-02 structural repairs are present in current canonical TablePro grammar and scenarios; JT-01–JT-06 canonical scenario bytes remain source-consistent; TablePro donor rows in TASK-005/058–064 match canonical; six counterexample/positive probes and the author seven-probe harness pass independently. This closes the bounded post-rereview witness gate assigned in `reaudit-jackin-tablepro-rereview.md`. It is not approval of all 150 parents, all 84 contributions, full PTY/color-profile parity, or coordinator source-obligation projection.

## JTR-01 — cell versus click-cell separation

Read `tablepro.md:71,79–83`. `cell(row,column)` is keyboard selection/reveal only with explicit Ctrl+Home/Down/Right and forbids hidden pointer events. `click-cell(row,column)` is a separately named completed-click macro with frozen HitRegistry/locate/clip geometry and explicit nonactivation when `Grid::locate` is `None`. Line 83 binds TP-031 current/different/repeated currency clicks and forbids treating registered rectangles as routable targets.

Canonical scenarios encode the split:

- `tablepro-scenarios.tsv:32` TP-031 uses `cell(1,currency)` for keyboard inline edit and a separate fresh-`T` suffix with `click-cell(1,currency)` checkpoints `current_cell_click_edit`, `different_cell_click_move`, `repeated_cell_click_edit`.
- TP-034/035/036/063/065/077 use `cell(1,currency)` with `Ctrl+L` for inline/save/dirty setup; notes inline claims are not retained there.
- `tablepro-scenarios.tsv:38` TP-037 alone retains `cell(1,notes)` / `click-cell(1,notes)` with explicit viewer/nonactivation checkpoints.

Independent harness `/tmp/jt-rereview.lBNmr0` (`cargo test --offline --lib audit::`): **6 passed, 0 failed**. Counterexamples prove currency reveal+click+Enter does not leave pending paste mutation and JSON reveal+click+Enter closes the viewer before its suffix checkpoint. Positive tests prove select-only currency edit/preview/save through four-pending/fifth-saved ticks at 80×24, 100×30, 120×40, 160×50 and current/different/repeated currency clicks at all four sizes.

## JTR-02 — currency inline versus long notes viewer branch

Read `tablepro-scenarios.tsv:32–38,64,66,78` and `tablepro.md:83`. Inline literal/save/dirty trajectories use short currency with explicit `Ctrl+L`, matching oracle `app_tests.rs:609` pending-edit setup. Long `orders.notes` behavior is preserved only in TP-037 with checkpoints `notes_keyboard_selected`, `notes_actual_click`, `notes_actual_enter`, `notes_paste_no_mutation`; prose documents registered-but-unroutable partial hit at 120×40 and size-specific guard freezing elsewhere.

TASK-022 owns the shared partial-hit contract without an application shim (`source-witnesses.md` W-022-08; `obligations.md` partial-hit rows). TASK-060 obligations mirror TP-031/037 action strings with click-cell checkpoints (`completion/060/trusted/obligations.md:104,134`).

Harness notes probe: registered hit present, locate absent, completed click inert, Enter opens viewer, paste leaves pending empty — consistent with `grid.rs:570–578,1213–1235,1300–1345`.

## JT-01–JT-06 canonical bytes (initial repair layer)

| ID | Canonical evidence read | Disposition |
| --- | --- | --- |
| JT-01 | `tablepro-scenarios.tsv:35` TP-034 `cp(saving);ticks(4);cp(still_pending);ticks(1);cp(saved)` before History | Present |
| JT-02 | `jackin-scenarios.tsv:25` JA-024 replay then `K(Esc,4,Enter);checkpoint(Environments-reentered)` | Present; donors 005/051–057 match canonical action |
| JT-03 | `jackin-scenarios.tsv:61` JA-060 replay, Inspect reopen, compact/advanced continuation | Present in canonical scenarios and `completion/056/trusted/obligations.md:76` |
| JT-04 | `tablepro-scenarios.tsv:19` TP-018 `End;cp(lazy_audit_selected);Right;cp(expanding);ticks(1);cp(loading);ticks(2);cp(loaded)` | Present; donors 005/058–064 match |
| JT-05 | `tablepro-scenarios.tsv:72` TP-071 separate fresh-`W` picker-chord paste fallthrough variants | Present; donors match |
| JT-06 | `tablepro.md:79–81` explicit keyboard reveal inside `cell()`; no hidden click | Present; superseded in prose by JTR-01 split |

Author harness `/tmp/jt-reaudit.4mpJ2V`: **5** TablePro `audit::` tests and **2** Jackin `audit_` tests passed independently (lazy load, commit timing, picker paste, notes reveal, Jackin replay continuations).

## Donor synchronization and supplements

Mechanical compare of action columns (preset prefix normalized):

- **TablePro:** TP-031/034/035/036/037/063/065/077 — canonical `app-flow-stage-audit.tsv` equals TASK-005 and TASK-058–064 stage-audit rows; canonical `app-flow-contributions.tsv` equals TASK-060/064 rows for inspected TP-031/037.
- **Jackin:** JA-024 matches across 005/051–057. JA-060 full copy-discard suffix is in canonical `jackin-scenarios.tsv:61` and `app-flow-stage-audit.tsv:61` and TASK-056 obligations; TASK-051–057 stage-audit rows retain the JT-03 reopen/continuation repair but omit the later routing extension suffix — coordinator stage mirror sync remains separate from this bounded cell/click-cell witness.
- **TASK-004 baseline:** `completion/004/trusted/app-flow-stage-audit.tsv` retains pre-repair notes-based TP rows by design as historical baseline producer; not a regression in canonical authority.
- **`jt-branch-corrections.md` (051–064):** present; branch-diff supplements (connections, filter, picker-paste, journeys) do not contradict the cell/click-cell grammar or currency/notes split.

`python3 docs/refactoring-plan/evidence/sync-app-scenarios.py --write` → `written:true`, `changed_files:[]`.

## Checked byte inventory

Digests identify bytes read at witness time; they supersede the pre-handoff table in `reaudit-jackin-tablepro-rereview.md` where files evolved afterward.

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `docs/refactoring-plan/tablepro.md` | 107 | `9b6b1719748f0391db63343b544ee0f7bcdbd341c5b3eb574ae83f945357a317` |
| `docs/refactoring-plan/tablepro-scenarios.tsv` | 81 | `b1eace66e823213048292fb4906fa000722488a184db647d674a9fdcf657a531` |
| `docs/refactoring-plan/app-flow-stage-audit.tsv` | 151 | `64211a2a0fdd270a50e54d82c31fe32b53bbad9d955c3762d86e3730a291ed9d` |
| `docs/refactoring-plan/app-flow-contributions.tsv` | 43 | `69495b477bede24ee34d85f50c6583d567370b81a6c45a340d360de870640f48` |
| `refactoring-tasks/terminal-components/completion/022/trusted/source-witnesses.md` | 20 | `fc2a5d32ef47fc58f4a032448f4678e6f31fc9cbcf32f744983303b2904bdf30` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |

## Scope limits

No claim of executed full 150-parent expanded campaign, complete application frames, PTY/compiler-qualified observers, or coordinator-wide source-field mirror regeneration. JA-060 copy-discard stage-audit propagation into TASK-005/051–057 and TASK-004 baseline retirement remain coordinator-owned follow-ups outside this accepted cell/click-cell witness.
