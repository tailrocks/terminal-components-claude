# HP20 — Complete cleanup insight taxonomy and guards

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Represent all 18 categories and every mapped policy variant with fixture paths/ages/process observations. Enforce shared eligibility from tree, insight and custom entry; test inherited protections and unknown states.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live artifact discovery, process probes and filesystem sizing.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve all 18 reachable cleanup categories, project artifact classifier, streaming sizing, age policy and process guards. Family enums and a few candidates are not full coverage.

**Source evidence and mandatory scope:** [matrix HP20](../parity/holla-parity-matrix.md#hp20--complete-cleanup-insight-taxonomy-and-guards) — `LD022`, `LD023`, `LD024`, `LD025`, `LD026`, `LD027`, `LD029`, `LD030`, `LD033`, `LD034`, `LD035`, `LD037`, `LD039`, `LD028`, `LD031`, `LD032`, `LD038`, `LD040`, `LD041`, `LD042`, `LD043`, `LD044`, `LD045`, `LD046`, `LD047`, `LD036`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Implement every LD022–LD047 root/tool/platform/age contract. Category resources lead to candidate detail or one cleanup plan. Preserve rebuildable/old-only/review-first distinctions; review-first starts unchecked; too-recent/unknown-age gated items remain visible but ineligible. Keep Xcode/Simulator guards, restricted pnpm root resolution and explicit Gradle stop prerequisite. Global review and category-specific entry remain searchable.

**Architecture / reusable components:** Centralize policy with category/path identity, inherited strictest descendant restrictions, freshness and process observation; the same policy applies from tree, insight or custom cleanup entry. Failed process probes mean unknown, not not-running. Share scanner sizing with bounded concurrency three; propagate partial/inaccessible diagnostics. Reuse grouped rows, TreeView, Props, selection and plans.

**Required deterministic fixture:** `parity-insights` — one positive/negative fixture per 18 category; macOS/Linux; 7/30/90-day boundaries, future/unknown age, no home, rejected pnpm store, all 12 artifact names × indicators/decoys/depth-six/link/nested case, active/unknown process, partial sizing.

**Acceptance / automated verification:** Assert exact roots/detection and category actions, hidden mac-only categories on Linux, largest-first candidates, age/default/manual eligibility and skip recheck before execution. Recent ineligible items cannot become selected via Space/select-all or a different entry point. Assert safe empty versus unavailable/partial and no direct Docker disk-image candidate. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture category overview/detail, review-first warning, age-disabled row, unavailable process guard, partial scan and all category families across platform captures.

**Dependencies:** HP11/HP12/HP18/HP19/HP21/HP22/HP23; F02/F05/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-insights` (`insights` in src/bin/holla/domain/parity.rs) · `hp20_insight_categories_sizes_eligibility_and_guards_are_truthful` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: eligibility_follows_age_safety_and_guards`, `src/bin/holla/domain/cleanup.rs: detection_respects_platform_tools_and_roots`, `src/bin/holla/domain/cleanup.rs: artifacts_need_an_indicator_and_never_nest`, `src/bin/holla/domain/cleanup.rs: execution_is_truthful_across_modes_dedup_and_failures` (process skip record), `src/bin/holla/sim/fs.rs: scan_deduplicates_hardlinks_and_never_follows_links`. · row proofs in src/bin/holla/app_tests_rows.rs: `ld022_to_ld047_the_category_table_matches_the_legacy_contract`.

**What the journey asserts:**
- Every `xcode.derived-data` candidate is `Eligibility::Ineligible` with a reason containing "Xcode is running" (fixture has `Proc` "Xcode").
- `xcode.device-support`: the `iOS` root (91 days) is `Eligibility::Preselected`, the `watchOS` root (89 days) is `Ineligible`.
- `pnpm.store` candidate path is `/Users/alex/Library/pnpm/store/v3` (from `pnpm store path`) with bytes `2200 * 1024 * 1024 + 2 * BLOCK`.
- `project.artifacts` contains `web/node_modules`, `rs/target`, `py/.venv`, `kt/build`, `ios/Pods`, `next/.next`; no path contains `target/node_modules`; no `/plain/build`; nothing under `/linked/` or `/Users/alex/work/`; `unreadable` ends with `/locked`.
- Page: "18 categories"; "Xcode DerivedData" with "810.0 MiB" and "Xcode is running" on the category; End shows "Project artifacts" with "2.5 GiB"; clicking that row in `cleanup::LIST` shows "Unreadable" and "locked".
- Unit `eligibility_follows_age_safety_and_guards`: `Running` gives `Ineligible("Xcode is running")`, `Unknown("pgrep failed")` gives a reason containing "unknown", age 89 `Ineligible` and 90 `Preselected`, `None` age contains "age unknown", `xcode.archives` is `Selectable`, `user.caches` 31 days `Selectable` and 29 `Ineligible`, future mtime is `None`, `CATEGORIES.len() == 18` with 18 unique ids.
- Unit `detection_respects_platform_tools_and_roots`: `xcode.derived-data` hidden on `Os::Debian`, `xcode.archives` undetected without root, `npm.cache` undetected without `npm`, `pnpm_store_root` accepts `.local/share/pnpm/store/v3`, rejects `/elsewhere/store` ("outside") and `Err` ("failed") to `Library/pnpm/store`.
- Unit `artifacts_need_an_indicator_and_never_nest`: `find_artifacts` returns exactly `rs/target` and `web/node_modules`; `walk_candidates` never enters `node_modules`.

**Captures:** `shots/h_hp20_categories` ("18 categories", "Xcode DerivedData" "810.0 MiB", "Xcode is running"), `shots/h_hp20_derived_data` (derived data detail with the guard), `shots/h_hp20_artifacts` ("Project artifacts" "2.5 GiB", "Unreadable"); base matrix `shots/h_parity_insights_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| LD022 | journey: all candidates `Ineligible` "Xcode is running", page "810.0 MiB"; unit detection hidden on `Os::Debian`. | live `pgrep`, real DerivedData sizing |
| LD023 | journey: `iOS` 91 d `Preselected`, `watchOS` 89 d `Ineligible`; unit 89/90 boundary. | real mtime |
| LD024 | unit: `xcode.archives` `Selectable` (never preselected), undetected without root; fixture has `Archives/2026/app.xcarchive`. | real sizing |
| LD025 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`simulator.caches`: rebuildable, any age, macOS, `Library/Developer/CoreSimulator/Caches`, guard `Simulator`); fixture `CoreSimulator/Caches/dyld/x` 900 MiB. | live Simulator probe |
| LD026 | unit: `brew.cache` detected on macOS with `brew` on PATH; fixture `Library/Caches/Homebrew`. | real sizing |
| LD027 | unit: `npm.cache` undetected without `npm`; fixture `.npm/_cacache` and `.npm/_logs` with `npm` in tools. | real sizing |
| LD029 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`yarn.cache`: rebuildable, any age, macOS, `.yarn/cache` and `Library/Caches/Yarn`); fixture `.yarn/cache/x.zip` with `yarn` in tools. | real sizing |
| LD030 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`bun.cache`: rebuildable, any age, all platforms, `.bun/install/cache`); fixture `.bun/install/cache/x` with `bun` in tools. | real sizing |
| LD033 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`maven.repository`: review first, any age, all platforms, `.m2/repository`); fixture `.m2/repository/org/x.jar` (200 d). | real sizing |
| LD034 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`pip.cache`: rebuildable, any age, macOS, `Library/Caches/pip`, detected by `pip3` or the directory); fixture `Library/Caches/pip/http/x`. | real sizing |
| LD035 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`uv.cache`: rebuildable, any age, all platforms, `.cache/uv`); fixture `.cache/uv/x` with `uv` in tools; the `$XDG_CACHE_HOME` override is not modeled. | real XDG resolution |
| LD037 | unit: `user.caches` detected on macOS, 31 d `Selectable`, 29 d `Ineligible`; validation refuses `Library/Caches` itself (HP21 unit). | real sizing |
| LD039 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`ide.jetbrains-logs`: rebuildable, 7 d, macOS, `Library/Logs/JetBrains`); fixture `Library/Logs/JetBrains/idea.log` (9 d). | real sizing |
| LD028 | journey: path `/Users/alex/Library/pnpm/store/v3`, bytes `2200 MiB + 2 * BLOCK`; unit `pnpm_store_root` accept/outside/failed. | live `pnpm store path` |
| LD031 | unit: `cargo.registry-cache` detected on `Os::Debian`; fixture `.cargo/registry/cache` (29 d) and `.cargo/git` (40 d). | real sizing |
| LD032 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`gradle.caches`: safe if old, 30 d, all platforms, `.gradle/caches`, `.gradle/daemon`, `.gradle/wrapper/dists`); the `gradle --stop` prerequisite before a daemon cleanup is asserted by `hp11_…` (a failing stop cancels the cleanup). | live `gradle --stop` |
| LD038 | `ld022_to_ld047_the_category_table_matches_the_legacy_contract`: id, safety, minimum age, platform, roots and guard asserted for every one of the 18 categories (`user.logs`: rebuildable, 7 d, macOS, children of `Library/Logs`, the root protected); fixture `old.log` (8 d) and `fresh.log` (6 d); `eligibility_follows_age_safety_and_guards` covers the age rule, the 7 d split of this fixture is not individually asserted. | real sizing |
| LD040 | journey: six artifact paths present, `/plain/build` absent, symlinked `linked` never followed, depth-six `deep/a/b/c/d/e/f` in fixture; unit `find_artifacts` exact list. | real traversal |
| LD041 | journey: `target/node_modules` never doubles; unit `is_artifact` false for `plain/build`, decoy `src/build` excluded. | none |
| LD042 | `cleanup.review-all` and `cleanup.<id>` items in catalog.rs; `hp23_*` asserts no `cleanup.xcode*` id on Linux; journey opens `Review cleanup candidates`. | none |
| LD043 | page "18 categories", "810.0 MiB", "2.5 GiB" per category; largest-first and three-way concurrency not asserted. | shared scanner concurrency |
| LD044 | journey: `unreadable` ends with `/locked`, symlinks never followed; unit future mtime is `None` and `None` age is "age unknown". | real inaccessible roots |
| LD045 | journey: `Preselected` versus `Ineligible` by age; unit review-first `Selectable`; dimmed rendering not asserted textually. | none |
| LD046 | journey: End then click on `cleanup::LIST` opens detail ("Unreadable", "locked"); Space/select-all on ineligible rows not asserted in `hp20_*`. | none |
| LD047 | journey: "Xcode is running" reason on every DerivedData candidate; unit `Unknown("pgrep failed")` is ineligible "unknown"; execution log line contains "Xcode is running" for the guarded item. | live `pgrep -x` and `ps -axo ucomm=` |
| LD036 | not applicable (boundary row) | none |

**Deferred remainder:**
- Live artifact discovery, process probes and filesystem sizing (Later section).
- Acceptance clauses not proven by the tests: per-category assertions for Simulator, yarn, bun, Maven, pip, uv, JetBrains logs, user logs; Linux hidden mac-only categories on the page (only `detected()` unit and `hp23_*` item ids); Space and select-all refusing too-recent rows; skip recheck before execution from the insight entry (HP21 covers execution); partial sizing diagnostics beyond the `unreadable` list; no direct Docker candidate is a boundary row.

**Limits:**
- Process observation is a fixture `Proc` list, not a real `pgrep`; ages are fixture mtimes against the fixture clock.
- Sizes come from the simulated `Fs`; no real cache directories are read.
