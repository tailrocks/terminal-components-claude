# HP07 — Native project-task adapters

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Represent package.json, Just, Make, Taskfile and mise task sources using fixture manifests or fixture tool output; show runner, provenance, limits, args, errors and child cwd. Pure parsing of fixture input is allowed.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live manifest discovery, runner probes, external task listing and actual project-task execution.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve package.json, Just, Make, Taskfile and mise tasks. Current pnpm-looking tasks are mise fixtures, not native adapters.

**Source evidence and mandatory scope:** [matrix HP07](../parity/holla-parity-matrix.md#hp07--native-project-task-adapters) — `OP20`, `OP21`, `OP22`, `OP23`, `OP24`, `OP25`, `OP26`, `OP27`, `OP28`, `OP29`, `OP30`, `OP58`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Each task resource states defining file, runner, effective cwd, description, provenance and exact argv. Preserve package lock precedence pnpm/yarn/bun/npm, sorted script names and string-only values; Just summary discovery; conservative Make declaration order; Taskfile JSON discovery; uncapped ordered mise descriptions. Preserve visible first-30 limits where legacy has them, or provide an explicit more-results control. Missing runners/discovery errors must be truthful rather than invented task success.

**Architecture / reusable components:** Create bounded typed source adapters with no implicit shell expansion. Task execution still delegates recipe interpretation to its tool; trusted custom config does not automatically trust a project task file. Review newly introduced trust separately while preserving intentional task execution. Reuse existing task picker, argument form, trust page and activities.

**Required deterministic fixture:** `parity-task-sources` — each filename variant/parser, invalid/missing manifests, runner absent, duplicate tasks, 31 entries, package lock conflicts, whitespace/Unicode/metacharacters, description-only mise output, failed discovery, child versus ancestor cwd.

**Acceptance / automated verification:** Assert exact IDs, order/cap/source diagnostics and argv for every OP20–OP30 variant. Assert Make never executes during discovery and rejects unsupported target syntax without claiming a full Make parser. Assert failed mise discovery cannot consume misleading stdout as authoritative. Test original cwd and explicit trust cancellation; launch-time args are not runtime stdin. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture one native adapter per source, capped results, parse/unavailable state, provenance review and running task in child cwd.

**Dependencies:** HP01/HP14/HP15/HP17; F01/F08/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-task-sources` (fixture fn `task_sources` in src/bin/holla/domain/parity.rs) · `hp07_task_adapters_keep_exact_names_caps_and_diagnostics` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/manifest.rs: node_scripts_sort_keep_strings_and_pick_the_runner`, `src/bin/holla/domain/manifest.rs: just_make_taskfile_and_mise_follow_their_legacy_rules`, `src/bin/holla/domain/manifest.rs: json_parser_handles_nesting_escapes_and_errors`, `src/bin/holla/domain/catalog.rs: rust_dirty_suggests_review_tests_and_pull_with_reasons` (only for the `mise.task.test` id).

**What the journey asserts:**
- The script named `it's; rm` is one argument: item `node.script.it's; rm` has argv `yarn run 'it'\''s; rm' @/Users/alex/work/poly`; `weird key` displays as `yarn run 'weird key'` and `ünï` as `yarn run ünï`.
- `node_scripts` on the fixture package.json with `yarn.lock` and `bun.lockb` present gives `d.runner == "yarn"` ("yarn.lock wins over bun.lockb"), `d.total == 35`, `d.tasks.len() == 30`, and the two names sorting past the cap (`weird key`, `ünï`) are absent.
- Exactly 30 `node.script.` items are listed and one carries a `cap_note` containing `30 of 35`.
- Ids `just.recipe.build`, `just.recipe.test`, `just.recipe.lint`, `make.target.all`, `make.target.build`, `make.target.deploy-prod` and `taskfile.discovery` exist; no id starts with `make.target.%`, contains `VAR`, or starts with `taskfile.task.`.
- The `taskfile.discovery` item summary contains `yaml: line 3`.
- Running `yarn s01` produces argv `yarn run s01 @/Users/alex/work/poly`, settles `ActivityState::Failed` with a non-zero `exit`; `yarn dev` produces `yarn run dev @/Users/alex/work/poly` and settles `ActivityState::Succeeded`.
- Unit (manifest.rs): sorted names `["build","dev","test"]` with the non-string `weird` dropped; runner `pnpm` beats `yarn`; `node_runner(&["bun.lockb"]) == "bun"`, `node_runner(&[]) == "npm"`; `{not json` gives a diagnostic containing `unreadable`, `{"scripts":{}}` one containing `empty`; `d.title("Node scripts") == "Node scripts (30 of 31)"`.
- Unit (manifest.rs): Just summary `build test  build\ndeploy` gives `["build","deploy","test"]` with argv `["just","build"]`; a failed summary gives a diagnostic containing `failed`; `make_targets` returns `["all","build","clean"]` and rejects `[".PHONY","%.o","$(X)","src/dir"]` with a diagnostic containing `unsupported`; Taskfile JSON with a duplicate and an empty name gives `["build","lint"]`, argv `["task","lint"]`, empty descriptions; `mise_tasks` keeps output order (`zzz` last), description `compile everything`, fallback `Run mise task`, argv `["mise","run","build"]`, and `Err("mise: not trusted")` yields no tasks.

**Captures:** `shots/h_hp07_scripts` (query `yarn`: 31 results, runner and `package.json` provenance in the detail pane) · `shots/h_hp07_diagnostic` (query `Taskfile`: the discovery diagnostic row with the `task --list --json` failure) · base matrix `shots/h_parity_task_sources_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP20 | `node_scripts_sort_keep_strings_and_pick_the_runner`: names `["build","dev","test"]`, non-string value dropped, `unreadable`/`empty` diagnostics, `None` manifest gives no tasks; journey: 30 `node.script.` items from the fixture manifest. | Reading the real cwd package.json. |
| OP21 | Unit: runner `pnpm` over `yarn`; `bun.lockb` gives `bun`; no lockfile gives `npm`; journey: `d.runner == "yarn"` with `bun.lockb` also present. | Probing real lockfiles on disk. |
| OP22 | Journey: argv `yarn run 'it'\''s; rm'`, `yarn run s01`/`yarn run dev` at `/Users/alex/work/poly`, cap note `30 of 35`, exit code carried into `Failed`; unit: id `node.script.build`, argv `["pnpm","run","build"]`, title `Node scripts (30 of 31)`; frame `h_hp07_scripts`. | Real package-manager process execution. |
| OP23 | Unit: `just --summary` text deduped and sorted to `["build","deploy","test"]`; `Err` gives a `failed` diagnostic; empty output gives no tasks; journey: `just.recipe.build/test/lint` from `.justfile` plus summary `build test  lint build`. | `just` PATH probe and the real summary call. |
| OP24 | Unit: argv `["just","build"]`; journey: the three `just.recipe.` ids exist. The 30-recipe cap is not asserted for Just (only for Node scripts). | Recipe execution. |
| OP25 | Unit: `make_targets` accepts `all build: dep`, `clean:`, `build:`, rejects `.PHONY`, `%.o`, `$(X)`, `src/dir`, diagnostic `unsupported`, missing file diagnostic `exact name`; discovery takes file text only (`make_discovery("/p", Some(mk))`); journey: no `make.target.%` and no `VAR` id. | `make` PATH probe and reading the real `Makefile`. |
| OP26 | Unit: argv `["make","all"]`; journey: `make.target.all/build/deploy-prod` exist in source order. The 30-target cap is not asserted for Make. | Target execution. |
| OP27 | Unit: dup/empty names collapse to `["build","lint"]`, descriptions empty, `{"tasks":"no"}` and `[` give diagnostics; journey: `taskfile.discovery` summary contains `yaml: line 3` and no `taskfile.task.` id; frame `h_hp07_diagnostic`. The prototype shows a diagnostic row where legacy hid the group. | `task` PATH probe and the real `task --list --json`. |
| OP28 | Unit: argv `["task","lint"]` only; no Taskfile task is executed in the journey (the fixture listing fails by design). | Task execution. |
| OP29 | Unit: output order kept, `compile everything` description, `Run mise task` fallback, failed discovery yields no tasks; the `parity-task-sources` fixture removes `mise`, so the journey has no mise row. | Real `mise tasks ls --no-header`. |
| OP30 | Unit: argv `["mise","run","build"]`; `rust_dirty_suggests_review_tests_and_pull_with_reasons` finds id `mise.task.test`. Ordering ahead of Git/Gradle/Compose/IDEA rows is not asserted. | Task execution. |
| OP58 | Journey: after `run(&mut h, "yarn s01")` the active tab is an activity with exact argv `yarn run s01 @/Users/alex/work/poly`; frame `h_hp07_scripts` shows `Gate runs directly`. No assertion names a trust page or trust flag. | none (UI row); trust separation for project task files is reviewed with the Later adapters. |

**Deferred remainder:**
- Live manifest discovery, runner probes, external task listing and actual project-task execution (Later).
- Not proven by tests: child versus ancestor cwd (the fixture cwd is the project root), runner absent, explicit trust cancellation, launch-time args versus runtime stdin, the 30 cap for Just/Make/Taskfile groups, and ordering of mise rows before folder actions.
- No capture of a running task in a child cwd or of the provenance review page.

**Limits:**
- Outcomes are the simulated executor's; no real yarn/just/make/task/mise process, PATH or filesystem is touched.
- Frames are being regenerated; their text is indicative until the integrator records the review.
