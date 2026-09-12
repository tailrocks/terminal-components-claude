# HP22 — Cleanup ownership, outcomes and operation log

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Keep simulated execution/report ownership outside modal lifetime. Model mixed per-path outcomes, quit-wait, in-memory operation logs, log failure, restart and honest Trash-versus-physical capacity.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real cleanup worker ownership, durable JSONL logging and physical capacity/OS integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cleanup progress, mixed outcomes, estimates, per-item operation logs and ownership. Current history records only simulated successful removal and falsely frees Trash space immediately.

**Source evidence and mandatory scope:** [matrix HP22](../parity/holla-parity-matrix.md#hp22--cleanup-ownership-outcomes-and-operation-log) — `LD016`, `LD059`, `LD060`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep cleanup in an activity independent of modal lifetime. Before irreversible commit allow cancellation; after commit keep execution/report ownership until settled and make quit wait explicit. Report removed/trashed/would-remove/failed/skipped separately, with counts, path/reason detail and log health. Same-volume Trash preserves filesystem used bytes until emptied; estimates and measured capacity change remain separate.

**Architecture / reusable components:** Typed result carries mode, dry-run, selected/executed identities, estimated size and observed outcome through worker, UI and log. Preserve JSONL v1 per requested item (including duplicates/protected/process-skipped) at XDG cache or ~/.cache/holla/ops.log: timestamp_ms/mode/path/size/outcome/error. Log failure is visible and does not invent rollback of completed deletion. Keep worker handle/report outside confirmation enum.

**Required deterministic fixture:** `parity-cleanup-results` — mixed success/fail/skip, duplicate, permission change, vanished target, process skip, dry-run, log write failure, receiver failure, result arrives during quit dialog, Stay/Leave and Trash versus permanent capacity.

**Acceptance / automated verification:** Assert every requested path gets one correct audit outcome; no success-only record after failure, correct first-six/overflow/full-log summary, persistent restart readability and JSON escaping. Assert no lost worker/report on modal replacement; quit completion is acknowledged. Apply only successful effects and rescan; reflect APFS clone/purgeable/hardlink estimation limits honestly. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture running/quit-wait/finished, mixed report with log failure, dry-run history and Trash estimated-versus-physical capacity.

**Dependencies:** HP14/HP15/HP18/HP20/HP21/HP23; F10–F15/F21/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-cleanup-results` (`cleanup_results` in src/bin/holla/domain/parity.rs) · `hp22_reports_ownership_outcomes_and_the_operation_log_are_exact` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: log_records_round_trip_through_jsonl`, `src/bin/holla/domain/cleanup.rs: execution_is_truthful_across_modes_dedup_and_failures`, `src/bin/holla/sim/fs.rs: trash_keeps_used_bytes_and_permanent_frees_them`. · row proofs in src/bin/holla/app_tests_rows.rs: `hp22_quit_waits_for_a_running_cleanup_and_its_report_lands_first` (in app_tests_parity.rs) proves the quit-wait ownership.

**What the journey asserts:**
- Before anything runs `ops_log.lines.len() == 1`; `Show cleanup history` shows "legacy/node_modules" and "1 · 1 operation log records"; the prior record is a table row containing "legacy/node_modules", "trash" and "trashed".
- Clicking `big` in `disk::TREE` then `d` shows "APFS clones may overcount" and "purgeable space excluded".
- Gate 2 phrase "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"; report `count(Outcome::Trashed) == 1`, `freed_now == 0`, `bytes == 6000 * 1024 * 1024 + BLOCK` (two clones plus the directory block).
- Report page reads "returns when the Trash is emptied" and "every record written".
- `ops_log.lines.len() == 2`, `persisted.ops_log.len() == 2`, `persisted.ops_log[0]` still contains "legacy/node_modules", `persisted.ops_log[1]` contains `"outcome":"trashed"` and `/big`.
- History reopened: "2 · 2 operation log records", a row with "Projects/safe/big", "trashed" and "GiB", and "Report 1".
- Volume `used > 0` while `/Users/alex/Projects/safe/big` no longer exists.
- Unit `execution_is_truthful_across_modes_dedup_and_failures`: eight requested items give `log.lines.len() == 8` (duplicates included) with `would_remove`, "duplicate", "covered by its ancestor", "Xcode is running", `"outcome":"failed"`, "contains protected"; `summary()` contains "2 would be removed"; `details()` is `(6, 0)`; `write_failure = "EROFS"` gives `Removed == 1`, `log_failures == 1`, "1 log write failures" and the file is gone (no rollback); `rec.json()` equals the exact escaped `{"v":1,...}` string.
- Unit `log_records_round_trip_through_jsonl`: two records parse back equal (including `"Trash unavailable: \"devbox\" has no backend"`); `{"v":2,...}` and `not json` parse to `None`.

**Captures:** `shots/h_hp22_history` (history page with "operation log records" and the prior "trashed" row); the report state is captured as `shots/h_hp21_report` and `shots/h_hp23_linux_report` ("Cleanup report" with trashed and failed rows); base matrix `shots/h_parity_cleanup_results_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| LD016 | gate: "APFS clones may overcount", "purgeable space excluded"; report `freed_now == 0`, `bytes == 6000 MiB + BLOCK`, "returns when the Trash is emptied"; volume `used > 0` after the path is gone; unit `trash` keeps `used` and `empty_trash()` frees it. | physical capacity measurement |
| LD059 | "Cleanup report", `count(Trashed) == 1`, "every record written"; unit `summary()` "2 would be removed", `details()` six rows and zero overflow, `log_failures == 1` "1 log write failures". Ownership: `hp22_quit_waits_for_a_running_cleanup_and_its_report_lands_first`: the committed plan becomes a world-owned job (`World.cleanup_job`, `cleanup::Execution` stepping one item per tick) after the gate closes (`modals.is_empty()`), the report page reads "Cleanup running" and "running · 0 of 1 items", Ctrl+Q says "A cleanup is running (0 of 1 items)" with "Leave when it settles", Leave sets `quitting` without dropping the job ("leaving when the cleanup settles"), the item lands on the next tick and the report plus the persisted log (2 records) are complete before `quit`. Receiver disconnect is not modeled. | real worker thread, join and receiver failures |
| LD060 | one JSONL line per requested item (8 of 8 including duplicate, protected, guarded, failed, dry run); exact `rec.json()` escaping; parse round trip and `v:2` rejection; `persisted.ops_log` append only with the prior record intact; `OpsLog.path` `~/.cache/holla/ops.log` in the unit; XDG choice not asserted. | durable `ops.log` I/O and `$XDG_CACHE_HOME` resolution |

**Deferred remainder:**
- Real cleanup worker ownership, durable JSONL logging and physical capacity/OS integration (Later section).
- Acceptance clauses not proven by the tests: a result arriving while the quit dialog is still open (the job steps only on ticks, so the dialog and the landing are asserted in sequence, not concurrently), receiver failure report, permission change and process skip through the UI (unit only for the guard), log write failure through the UI (unit only, `w.ops_log.write_failure` is `None` in the fixture), rescan after cleanup, restart readability beyond `persisted.ops_log` in memory.

**Limits:**
- The log is an in-memory `OpsLog` with a `persisted` mirror; no file is written or reopened across a process restart.
- Trash and volume accounting are simulated; no APFS clone or purgeable measurement occurs.
