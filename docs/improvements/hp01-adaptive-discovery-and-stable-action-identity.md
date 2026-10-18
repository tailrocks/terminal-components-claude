# HP01 — Adaptive discovery and stable action identity

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model every mapped provider availability, provenance, collision and out-of-order result state in memory; connect finder, alternatives and preview with stable identity and first-paint behavior.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real executable/daemon probes and live discovery adapters.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve available actions, instant discovery, stable identity, grouping and keyboard invocation. The preview has timed sources and a finder, but lacks the complete legacy probe/registry contract.

**Source evidence and mandatory scope:** [matrix HP01](../parity/holla-parity-matrix.md#hp01--adaptive-discovery-and-stable-action-identity) — `L01`, `L07`, `L02`, `L03`, `L04`, `L05`, `L14`, `L06`, `L16`, `L17`, `L18`, `L19`, `L25`, `L26`, `L27`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model executable missing/present, local markers, daemon failure and unknown state separately. Keep Current/Project/Host provenance visible; unavailable tools cannot become runnable through aliases. Retain built-in Find/Disk/Cleanup routes. Merge provider contributions deterministically by canonical action ID; report collisions and malformed-source warnings without retargeting existing selection. First render must precede slow discovery/history work.

**Architecture / reusable components:** Give each source a generation and each row an identity independent of index, label or arrival order. Reuse Picker, Input, Panel, EmptyState, StatusBar and FocusRing; do not add a provider widget.

**Required deterministic fixture:** `parity-discovery` — empty PATH; each legacy provider individually available; marker-only/tool-only; .git file; unavailable daemon; out-of-order/failed sources; conflicting IDs; query typed before completion.

**Acceptance / automated verification:** Assert registry and visible action sets, source provenance, deterministic order and warning recovery. Assert first paint before a blocked source completes, preserved selection through inserts/removals, query reset semantics, Enter/separator eligibility, Home/End/PageUp/PageDown and preview focus/long-command scrolling. Measure first-paint latency with a stated environment; do not invent a universal 100 ms guarantee. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture discovering, no tools, one failed source, collision detail, long preview and narrow return-focus states.

**Dependencies:** F02/F03/F05/F08/F23; HP07/HP10/HP13/HP17 supply provider-specific fixtures.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-discovery` (fixture fn `discovery` in src/bin/holla/domain/parity.rs) · `hp01_discovery_is_adaptive_stable_and_reports_failed_sources_and_config_warnings` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/custom.rs: diagnostics_are_indexed_and_valid_siblings_survive`, `src/bin/holla/domain/parity.rs: every_parity_world_builds_a_catalogue_with_unique_ids`, `src/widgets/picker.rs: refresh_keeps_identity_and_query_reset_selects_first`. · row proofs in src/bin/holla/app_tests_rows.rs: `l07_built_in_find_browse_disk_and_cleanup_routes_survive_an_empty_toolset`, `l16_l19_navigation_keys_land_on_rows_and_the_focused_preview_scrolls`.

**What the journey asserts:**
- The first frame (Motion::Full, before any tick) contains "discovering" and the item set has no `git.pull` (the git source answers at tick 40): first paint precedes the slow source.
- After 14 ticks `failed_sources()` names "docker"; after 45 more ticks all of `git.pull`, `git.push`, `mise.task.test`, `cargo.build`, `node.script.dev`, `just.recipe.build`, `make.target.build`, `taskfile.task.lint`, `brew.service.redis.restart`, `gradle.build`, `idea.clean`, `docker.daemon`, `deploy`, `custom.config` are present.
- `docker.daemon` carries `Freshness::Unavailable(_)`.
- The project config's colliding `git.pull` never retargets the built-in: argv is `git -C /Users/alex/work/probe pull --ff-only @/Users/alex/work/probe`; `deploy` is `tools/deploy.sh @/Users/alex/work/probe`; the malformed id `broken` is not an item.
- `custom_project.diagnostics` contains "git.pull" and a diagnostic with `index == Some(2)`.
- The rendered frame contains "config warning" and "docker, github unavailable".
- `github.clone` resolves to `gh auth login @/Users/alex/work/probe` while gh is not logged in.
- Opening "Custom action configuration" renders "action[0]" or "action[2]".

**Captures:** `shots/h_hp01_discovering` (sources still answering, footer counts sources), `shots/h_hp01_discovered` (docker and github reported unavailable, config warnings), `shots/h_hp01_config_page` (configuration page with indexed diagnostics); base matrix `shots/h_parity_discovery_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| L01 | hp01: fixture `w.tools` lists every legacy tool; `docker.daemon` is `Freshness::Unavailable`; `github.clone` argv `gh auth login` (unavailable tool leads to login, not a clone). Runner-independent node scripts: `node.script.dev` present. | real PATH/executable probes |
| L07 | `l07_built_in_find_browse_disk_and_cleanup_routes_survive_an_empty_toolset`: with `world.tools` cleared, `find.files`, `browse.files`, `disk.usage`, `disk.overview` and `cleanup.review-all` stay while the tool-gated `cargo.build` leaves. | real PATH probes |
| L02 | hp01: first frame contains "discovering" and lacks `git.pull`; `h_hp01_discovering` shows the source count in the footer. Latency is not measured. | first-paint latency in a stated environment |
| L03 | hp01: sources answer at ticks 2, 3, 5, 6, 8, 12, 20, 30, 40 and the full id list is present afterwards; `parity.rs: every_parity_world_builds_a_catalogue_with_unique_ids`. Order permutation is not asserted. | real threads |
| L04 | hp01: colliding project `git.pull` keeps the built-in argv `git -C /Users/alex/work/probe pull --ff-only`; `custom.rs: diagnostics_are_indexed_and_valid_siblings_survive` asserts "action[3]" "duplicate". | none |
| L05 | hp01: "config warning", "docker, github unavailable", diagnostic `index == Some(2)`, config page shows "action[0]"/"action[2]". | none |
| L14 | `picker.rs: refresh_keeps_identity_and_query_reset_selects_first`: cursor follows key "k2" after a removal above, moves to first eligible "k3" when its row vanishes. Not asserted through hp01 itself. | none |
| L06 | Redesigned: hp02 asserts "Recent here" on the empty query; `h_hp01_discovering` shows the grouped empty-query layout. | none |
| L16 | `l16_l19_navigation_keys_land_on_rows_and_the_focused_preview_scrolls`: Home, End, PageUp, PageDown and forty Up/Down steps always land on an item or explore row (`cursor_on_row`), never a heading or blank; Enter runs the row in every journey (`open` helper). | none |
| L17 | `app_tests_flows.rs: esc_ladder_and_quit_rules`: Esc clears the query, then the scope, then a page, then asks before quitting; the `open` helper relies on the same ladder. | none |
| L18 | Preview shows Does, Runs, In, Host, Scope, Why, Changes, Risk, Data, Gate (`h_hp05_pull_blocked`); a long argv is truncated in the panel, shown whole in the review sequence (`h_hp21_gate1`) and copied whole with `y` (`finder.rs: preview_key`). Partial: the panel itself has no multiline full-command view. | none |
| L19 | `l16_l19_navigation_keys_land_on_rows_and_the_focused_preview_scrolls`: Tab focuses the preview (`focus.is(finder::PREVIEW)`), Down scrolls it at 120x24 while the list cursor stays, typing returns the keyboard to the list, and a query change resets the preview scroll to 0 (`rebuild` resets on a new query or a different previewed item). | none |
| L25 | not applicable (boundary row) | none |
| L26 | not applicable (boundary row) | none |
| L27 | hp01: every provider contribution present by id (git, mise, cargo, node, just, make, taskfile, brew, gradle, idea, docker, custom). Module order is not preserved by design. | none |

**Deferred remainder:**
- Real executable/daemon probes and live discovery adapters (Later).
- First-paint latency with a stated environment is not measured; only ordering (frame before the tick-40 source) is proven.
- Preserved selection through inserts/removals is proven at widget level only, not through the App.
- Home/End/PageUp/PageDown, separator eligibility, preview focus and long-command scrolling have no assertion.
- L07 built-in routes with no tools have no assertion.

**Limits:**
- Timing is simulated ticks; no wall-clock or real terminal is involved.
- Unavailability is fixture state, not a PATH or socket probe on this host.
