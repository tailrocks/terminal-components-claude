# HP11 — Gradle tasks, daemon and recursive cleanup

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model installed/wrapper/missing states, tasks, recursive fixture candidates, daemon-stop prerequisites, Trash/dry-run and partial outcomes.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real Gradle commands, process probes/stops, filesystem traversal and cleanup.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve installed-Gradle clean/build/test and recursive .gradle/build cleanup with daemon stop. One wrapper-clean fixture does not cover these.

**Source evidence and mandatory scope:** [matrix HP11](../holla-parity-matrix.md#hp11--gradle-tasks-daemon-and-recursive-cleanup) — `OP44`, `OP45`, `OP46`, `OP47`, `OP48`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep Gradle command actions for build.gradle/build.gradle.kts with installed gradle. Wrapper preference may extend the new model but must retain legacy routes. Separate ordinary tool clean from depth-five Trash cleanup and global cache insights. Resolve exact candidates, review rebuild cost and show daemon stop as an explicit prerequisite; stop failure/unknown state must prevent unsafe cache deletion.

**Architecture / reusable components:** Share bounded candidate traversal with HP12 and deletion policy with HP20/HP21. Activity owns daemon state and cleanup report. Reuse tasks, candidates, Plan and facts; avoid standalone daemon widget.

**Required deterministic fixture:** `parity-gradle` — installed/wrapper-only/missing tool, both build files, clean/build/test success/failure, mixed build/.gradle candidates depth 5/6, symlink/node_modules exclusion, daemon active/stop failed, no candidates.

**Acceptance / automated verification:** Assert exact commands/cwd, selected directories only, no traversal through links, ignored node_modules, safe empty result, mixed failure report. Dry-run must not stop a daemon or mutate cleanup targets; explicit audit logging remains allowed. Distinguish tool-native clean from Trash and prerequisite failure from completed cleanup. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture three task choices, daemon-stop barrier, Trash candidate plan, dry-run and failed prerequisite.

**Dependencies:** HP07/HP12/HP14/HP18/HP20/HP21/HP22; F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-gradle` (fixture fn `gradle` in src/bin/holla/domain/parity.rs) · `hp11_gradle_wrapper_daemon_and_recursive_cleanup_are_bounded_and_truthful` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: artifacts_need_an_indicator_and_never_nest` (the shared `walk_candidates` walker).

**What the journey asserts:**
- Ids `gradle.build`, `gradlew.build`, `gradle.clean`, `gradlew.clean`, `gradle.test`, `gradlew.test` and `gradle.clean-all` exist; `gradlew.build` has argv `./gradlew build @/Users/alex/work/android`.
- `./gradlew build` settles `ActivityState::Failed` and `world.gradle.daemon_running` stays true.
- `gradle clean` has argv `gradle clean @/Users/alex/work/android`, settles `Succeeded`, and `/Users/alex/work/android/build` no longer exists.
- The `gradle.clean-all` label contains `49.0 MiB`; its page text contains `Cleanup`.
- Candidate rows contain `app/build`, `android/.gradle` and `a/b/c/d/e/bu`; they do not contain `e/f/build`, `too-deep` (beyond depth 5), `android/node_modules`, or `work/other` ("the link is never followed").
- The page text contains `3 selected`; pressing `d` shows `gradle --stop first`.
- With `gradle.stop_fails = Some("Gradle daemon is busy")`, after gate 1 and the gate 2 phrase `TRASH {n} UNDER /Users/alex/work/android ON mbp`, the text contains `gradle --stop failed`, `/Users/alex/work/android/.gradle` still exists ("nothing was removed"), and `daemon_running` stays true.
- Unit (cleanup.rs): `walk_candidates` over the fixture tree returns only `Projects/rs/target` for a `/target` selector and nothing for `index.js` ("node_modules is never entered").

**Captures:** `shots/h_hp11_cleanup` (cleanup page: four Gradle candidates with sizes, `depth 5 · no symlinks · no node_modules`, the `gradle --stop` prerequisite fact) · `shots/h_hp11_gate1` (review gate 1: Trash mode, prerequisite line, four `trash` commands) · base matrix `shots/h_parity_gradle_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP44 | Journey: `gradle.build`, `gradle.clean`, `gradle.test` exist with `gradle` in the fixture tools and `build.gradle.kts` present; wrapper rows `gradlew.*` are the additive expansion. The clean/build/test order and the missing-tool case are not asserted. | Real `gradle` PATH probe and build-file detection. |
| OP45 | Journey: argv `gradle clean @/Users/alex/work/android`, `Succeeded`, `android/build` removed (tool-native, not Trash). | Real Gradle clean. |
| OP46 | Partly: journey runs the wrapper `./gradlew build` (argv asserted) to `Failed` with the daemon still up; `gradle build` itself is not run, only its id is asserted. | Real Gradle build. |
| OP47 | Partly: only the id `gradle.test` is asserted; no test verb is run. | Real Gradle test. |
| OP48 | Journey: label `49.0 MiB`, exact candidates, depth bound, node_modules and symlink exclusion, `gradle --stop first`, a failing stop yields `gradle --stop failed` with `.gradle` intact and the daemon still running; gate 2 phrase `TRASH {n} UNDER /Users/alex/work/android ON mbp`; unit walker test; frames `h_hp11_cleanup`, `h_hp11_gate1`. Successful cleanup and dry run are not asserted. | Real `gradle --stop`, filesystem traversal and Trash. |

**Deferred remainder:**
- Real Gradle commands, process probes/stops, filesystem traversal and cleanup (Later).
- Not proven by tests: wrapper-only and missing-tool states, `gradle build`/`gradle test` outcomes, a successful recursive cleanup, dry run leaving the daemon alone, no-candidates result, mixed failure report, and the global cache insight.
- No capture of a dry run or of a completed Trash cleanup.

**Limits:**
- Daemon state and Gradle exits are fixture flags; no JVM, daemon registry or real directory walk is involved.
- Frames are being regenerated; the current `h_hp11_cleanup` text reads `4 selected` while the journey asserts `3 selected`, so treat frame text as indicative until the integrator records the review.
