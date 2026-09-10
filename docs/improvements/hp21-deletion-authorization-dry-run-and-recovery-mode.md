# HP21 — Deletion authorization, dry run and recovery mode

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Enforce immutable simulated target/mode/revision authorization, all mapped path/policy rules, Trash default, explicit permanent mode and truthful dry-run results. Model drift, aliases, protected descendants and backend failure before any simulated effect.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Actual canonical filesystem validation, Trash/permanent mutation, pathname-race probes and CLI bypass tests.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve deletion review, Trash default, explicit permanent mode, dry-run and path protections. Current gates protect fixture labels, not filesystem identity.

**Source evidence and mandatory scope:** [matrix HP21](../holla-parity-matrix.md#hp21--deletion-authorization-dry-run-and-recovery-mode) — `L20`, `L21`, `LD048`, `LD049`, `LD051`, `LD055`, `LD057`, `LD058`, `LD050`, `LD052`, `LD053`, `LD054`, `LD056`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Resolve paths, inherited policies, recovery mode and estimated effects before review. Keep Cancel default; permanent selection is explicit and resets authorization; broad operation adds typed target-bound phrase. Dry-run uses identical discovery/validation and emits Would remove, with no daemon stop, cleanup-target mutation or false Removed result; its audit-log write is explicit. Application-owned filesystem mutation has one shared boundary; external Cargo/Gradle/Docker effects disclose tool-native behavior.

**Architecture / reusable components:** Create immutable authorized cleanup specification carrying canonical parent/leaf identity, host/root/selection/mode/revision, category/process/age policy and typed dry-run outcome. Validate absolute lexical components and every LD053/LD054 deny rule, including ancestor containment of protected descendants. Reject symlink ancestors except exact macOS aliases; preserve selected-link-only semantics. Revalidate after sizing, never silently fallback from Trash to permanent. State the unprivileged-user threat model and remaining pathname race limit.

**Required deterministic fixture:** `parity-delete-safety` — every allowed/denied root, home/container/browser/cloud data, ancestor containing protected child, duplicate parent/child, Unicode/newline names, replaced symlink ancestor/leaf, missing/unreadable target, drift during sizing, Trash collision/exhaustion, mode change and dry-run.

**Acceptance / automated verification:** Assert no denied target or protected descendant mutates through any entry, canonical checks rerun at commit, leaf symlink target survives, retry only transient AlreadyExists (max three), no permanent fallback. Assert dry-run validates/logs exact would-results but triggers no process or cleanup-target effects; explicit audit logging is allowed. Gate invalidates on any material target/mode/policy change; alias/history/CLI cannot bypass authorization. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture selected path/mode facts, permanent toggle, typed gate, invalid/changed target, Trash backend failure and truthful dry-run result.

**Dependencies:** HP17/HP18/HP19/HP20/HP22/HP23; F01/F02/F05/F23. Safety boundary precedes wiring any new destructive action.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-delete-safety` (`delete_safety` in src/bin/holla/domain/parity.rs) · `hp21_deletion_is_authorized_validated_at_commit_and_never_falls_back` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: validation_denies_every_documented_rule`, `src/bin/holla/domain/cleanup.rs: execution_is_truthful_across_modes_dedup_and_failures`, `src/bin/holla/sim/fs.rs: trash_keeps_used_bytes_and_permanent_frees_them`, `src/bin/holla/sim/fs.rs: canonical_parent_reports_link_ancestors_and_keeps_the_leaf`. · row proofs in src/bin/holla/app_tests_rows.rs: `ld053_ld054_every_protected_root_and_user_path_is_denied_by_name`.

**What the journey asserts:**
- Clicking `target`, `linked`, `gone-later`, `locked` in `disk::TREE` gives "4 selected".
- `validate` denies `.Trash/old` ("Trash"), `Library/Containers/com.apple.Safari/Data/x` and `Library/Application Support/App/x` ("holds user data"), `Mobile Documents/com~apple~CloudDocs/x` ("iCloud"), `Library/Caches` ("itself is protected"), `Projects/safe/linked/node_modules` ("symbolic link"); `/tmp/holla-scratch/x` resolves to `/private/tmp/holla-scratch/x`; `Projects/safe/ünï\ncode/build` is `Ok`.
- `d` shows "Review · trash 4 items", "Threat model" with "re-resolved at commit", "Denied" with "none · every path passed", "Revision" with "voids this review".
- `gone-later` is removed after review; gate 2 phrase "TRASH 4 UNDER /Users/alex/Projects/safe ON mbp"; report outcomes: `/target` `Outcome::Trashed`, `/gone-later` `Outcome::Failed`, `/linked` `Outcome::Trashed` and `/Users/alex/work/other/node_modules/x` still exists, `/locked` `Outcome::Failed` with error containing "unreadable".
- `r.freed_now == 0`, `r.bytes >= 1000 MiB`, page shows "Cleanup report" (fixture `trash_collisions = 2` is absorbed by the retry).
- Second world: `m` shows "PERMANENT", `n` shows "Dry run", `d` shows "dry run" and "touch nothing"; gate 2 phrase "PERMANENTLY DELETE {n} UNDER /Users/alex ON mbp"; report `dry_run` is true, no item is `Outcome::Removed`, volume `used` unchanged, ops log has a line with `"dry_run":true` and `would_remove`.
- Unit `validation_denies_every_documented_rule`: relative, `/`, trailing slash, `//`, `..`, `/usr/bin/git`, `/usr/local`, `/Library`, `/Users`, home, Keychains, Safari, Containers (Safari, docker), `Library/Caches`, `Library/Logs` refused; `Library/Caches/com.app`, `/usr/local/lib/x`, `/Library/Caches/x`, missing `nothing-here` accepted; `link/node_modules` refused while `link` itself is Ok; `/tmp/junk` on `Os::Debian` refused; `protected_descendant(~/Library)` is `Some`.
- Unit `execution_is_truthful_across_modes_dedup_and_failures`: stale `revision + 1` is `Err` with an empty log; Trash failure "Trash full" gives `Failed` and the path still exists; `phrase()` equals "PERMANENTLY DELETE 1 UNDER /Users/alex ON mbp"; Trash and Permanent plans have different revisions.
- Unit `trash_keeps_used_bytes_and_permanent_frees_them`: `trash_collisions = 2` succeeds, `5` fails with "3 attempts" and the file survives; `Trash unavailable` leaves the file.

**Captures:** `shots/h_hp21_gate1` ("Review · trash 1 item", "Threat model", "Denied"), `shots/h_hp21_gate2` ("gate 2 of 2" with the phrase "TRASH 1 UNDER /Users/alex/Projects/safe ON mbp"), `shots/h_hp21_gate2_typed` (phrase typed into the input), `shots/h_hp21_report` ("Cleanup report", "trashed"); base matrix `shots/h_parity_delete_safety_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| L20 | destructive route needs two gates: "gate 1 of 2" then "gate 2 of 2" typed phrase (`gate2` helper); safe/mutating metadata flags are not asserted in `hp21_*`. | none |
| L21 | Cancel is default (`confirm` helper presses Right before Enter; `review.rs` `GATE_CANCEL` "Cancel"); description, "Recoverable" and "Threat model" facts; small-terminal truncation not asserted (`h_parity_delete_safety_80x24` only). | none |
| LD048 | tree entry (`hp21_*` "Review · trash 4 items"), Top files entry (`hp19_*` "gate 1 of 2"), insight entry (second world `d`); unit dedup records "duplicate" and "covered by its ancestor"; first-ten path list not asserted. | none |
| LD049 | `m` shows "PERMANENT"; gate phrase switches to "PERMANENTLY DELETE"; unit: mode change gives a different `revision`. | none |
| LD051 | unit `execute` is the single path for Trash/Permanent/dry run; external Cargo/Gradle/Docker disclosure not asserted here. | NSFileManager Trash, `remove_dir_all` |
| LD055 | `linked/node_modules` refused "symbolic link", `linked` leaf `Trashed` while `work/other/node_modules/x` exists; unit `canonical_parent` returns `["/Users/alex/work/a/to-b"]` for a link ancestor and `[]` for a link leaf. | real canonicalize at commit |
| LD057 | fixture `trash_collisions = 2` and `/target` is `Trashed`; unit "3 attempts" failure and "Trash unavailable" leave the file; `hp23_*` Linux: all items `Failed`, "nothing fell back". | NSFileManager, trash crate, FreeDesktop |
| LD058 | unit permanent `Removed` with `freed_now == 2 * BLOCK`; fs unit missing path "no such file", unreadable "permission denied"; journey `/locked` `Failed` "unreadable", vanished `gone-later` `Failed`. | `remove_file`, `remove_dir_all`, real metadata traversal |
| LD050 | `n` "Dry run"; "touch nothing"; report `dry_run`, no `Removed`, volume `used` unchanged, log `"dry_run":true` with `would_remove`; unit `WouldRemove == 2` and `fs.exists(nm)`. | none |
| LD052 | unit: relative, `/`, trailing slash, `//`, `..` refused; `ünï\ncode` and missing `nothing-here` accepted (journey `ünï\ncode/build` Ok). | none |
| LD053 | `ld053_ld054_every_protected_root_and_user_path_is_denied_by_name`: `/`, `/bin/x`, `/sbin/x`, `/etc/x`, `/private/etc/x`, `/System/x`, `/var/db/x`, `/private/var/db/x`, `/usr/x`, `/usr/local`, `/Library`, `/Applications`, `/Users`, `/home` and the home itself are each denied; unit `validation_denies_every_documented_rule` adds `/usr/bin/git` and the accepted `/usr/local/lib/x`, `/Library/Caches/x`. | none |
| LD054 | journey: Trash, Safari container, Application Support, iCloud, `Library/Caches` itself each denied with its reason; `ld053_ld054_every_protected_root_and_user_path_is_denied_by_name`: `.Trash/x`, `Library/Keychains/x`, `Library/Application Support/x`, `Library/Safari/x`, the Safari and docker containers, `Library/Caches` and `Library/Logs` roots denied while `Library/Caches/com.app`, `work/x/target` and `.npm/_cacache` pass. | none |
| LD056 | "Threat model" fact with "re-resolved at commit" on the gate; `cleanup.rs` module doc names the unprivileged single user. | pathname-race probe |

**Deferred remainder:**
- Actual canonical filesystem validation, Trash/permanent mutation, pathname-race probes and CLI bypass tests (Later section).
- Acceptance clauses not proven by the tests: drift during sizing (the journey removes `gone-later` between review and commit, not during sizing), Trash exhaustion at the UI level (unit only), alias/history/CLI bypass, first-ten path list and remaining count, Esc during execution, duplicate parent/child through the UI (unit only).

**Limits:**
- Denials and canonicalisation run against the simulated `Fs`; `/tmp` to `/private/tmp` is a fixture symlink, not `realpath`.
- Collision retries and the unreadable folder are fixture flags, not filesystem errno values.
