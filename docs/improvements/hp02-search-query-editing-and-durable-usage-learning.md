# HP02 — Search, query editing and durable usage learning

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement query editing, ranking explanations, recents, decay/expiry, opt-out and store-error states with a virtual clock and in-memory serialized store. Simulate restart, corruption, migration and concurrent merge without reading or writing user history.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Durable usage-store I/O, real restart/migration and concurrent filesystem writer integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve fuzzy matching, query editing, recents, learned query choices and resilient persisted usage. Current usage is in memory; stored last-used timestamps do not establish decay.

**Source evidence and mandatory scope:** [matrix HP02](../parity/holla-parity-matrix.md#hp02--search-query-editing-and-durable-usage-learning) — `L08`, `L10`, `L15`, `L30`, `L09`, `I-H01`, `I-H02`, `I-H03`, `I-H04`, `I-H05`, `I-H06`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep exact user aliases strongest. Preserve matching across label/group/description/keywords and grapheme-safe emphasis; learned choices must still match and must not authorize execution. Explain the difference between an explicit pin, a remembered query and contextual frequency. Retain five-item recent projection and truthful invocation history, including failed invocations; headless runs remain outside implicit learning. Implement visible query selection/undo/redo/word deletion.

**Architecture / reusable components:** Use a versioned usage-store adapter with deterministic clock, atomic merge and opt-out. Preserve v1 legacy reads or explicitly migrate that store into the new model without losing choices; writes retain concurrent updates. Reuse Input/Picker and existing ranking explanations. Do not confuse activity logs with usage.

**Required deterministic fixture:** `parity-history` — threshold/ties, exact alias versus learned query, whitespace/case normalization, Unicode, 20-use bound, 10-day decay, 90-day expiry, clock skew, corrupt/versioned store, two writers, save error, HOLLA_NO_HISTORY=1.

**Acceptance / automated verification:** Assert no unmatched learned results; bounded automatic frecency, deterministic ties and positive-only recents. Assert restart continuity, stale query pruning, opt-out performs neither reads nor writes, and save errors do not change action outcome. Exercise query Ctrl-A, Ctrl-Z, redo, word-delete and selected replacement. Never claim ignored clipboard requests as supported copy. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Recent here, Why this result, changed ranking after restart, disabled-history status and Unicode match emphasis.

**Dependencies:** HP01/HP17; F08/F10/F23. Explicit pins remain new-product behavior, separate from automatic learning.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-history` (fixture fn `history` in src/bin/holla/domain/parity.rs) · `hp02_recents_learned_queries_query_editing_and_persistence_merge` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/usage.rs: frecency_decays_saturates_and_bounds_uses`, `src/bin/holla/domain/usage.rs: recent_is_bounded_positive_and_deterministic`, `src/bin/holla/domain/usage.rs: learned_queries_normalise_and_survive_restart`, `src/bin/holla/domain/usage.rs: corruption_versions_merge_and_opt_out`, `src/bin/holla/domain/ranking.rs: aliases_beat_everything_and_carry_a_tag`, `src/widgets/picker.rs: query_edits_are_grapheme_safe_and_paste_is_one_event`.

**What the journey asserts:**
- `recent()` order is `git.pull`, `cargo.build`, `cargo.test`; `cargo.check` (one use 80 days ago) is absent; the top score is `<= 1.0 + 20.0`; every score `> 0.05`; the frame shows "Recent here".
- Typing "pull": the remembered but unmatched `git.fetch` is not shown ("Fetch and prune" absent) and row 6 contains "Pull ".
- After `used("git.pull_rebase", ...)` and `learn_query("pull", "git.pull_rebase", ...)` row 6 contains "Pull with rebase" and the frame contains "remembered for “pull”".
- "ünï code" yields "No matches" and never "cargo clippy" (a remembered choice must still match).
- Ctrl+A shows "Query selected"; typing "x" replaces the selection; Ctrl+Z restores "ünï code"; Ctrl+Y redoes; Alt+Backspace and Ctrl+Backspace delete "bui" leaving "cargo "; Ctrl+U clears; `Input::Paste("cargo\ncheck")` lands as "cargo check".
- Enter runs `cargo check @/Users/alex/work/holla`; the saved store contains "system.resources" (the other writer's record); the reloaded `cargo.check` count is 2; `learned_for("gone")` is `None`; `learned_for("pull")` is `Some("git.pull_rebase")`; `learned_for("ünï  code")` is `None`.
- With `UsageStore::disabled()`: no "remembered" text, nothing containing "cargo.check" is persisted, no "history not saved" message.

**Captures:** `shots/h_hp02_recent` (Recent here on the empty query), `shots/h_hp02_query` (query "pull" with its ranked rows), `shots/h_hp02_query_selected` (Ctrl+A selection), `shots/h_hp02_query_undone` (replacement undone); base matrix `shots/h_parity_history_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| L08 | hp02: "pull" leads with "Pull "; "ünï code" gives "No matches"; `usage.rs: learned_queries_normalise_and_survive_restart` asserts `normalize_query("Ünï  code") == "ünï code"`. Match-source breadth is in `ranking.rs: intent_phrases_match_keywords_and_scope_tokens_filter`. | none |
| L10 | Not asserted: no test checks the context line or grapheme emphasis. `h_hp02_query` shows the result rows. Unproven in tests. | none |
| L15 | hp02: rows change immediately after `type_str`; `picker.rs: query_edits_are_grapheme_safe_and_paste_is_one_event` asserts backspace removes the whole cluster "👩‍💻". | none |
| L30 | hp02: "Query selected", replacement, Ctrl+Z, Ctrl+Y, Alt+Backspace, Ctrl+Backspace, Ctrl+U, one-event paste as listed above. Ignored clipboard requests are not asserted. | none |
| L09 | `ranking.rs: aliases_beat_everything_and_carry_a_tag` (alias "dc" leads with tag "alias dc"); `usage.rs: recent_is_bounded_positive_and_deterministic` ties resolve `tie0` before `tie1`. | none |
| I-H01 | hp02: recent order and threshold as above, "Recent here"; `usage.rs: recent_is_bounded_positive_and_deterministic` asserts `RECENT_MAX` rows, all `> RECENT_THRESHOLD`, "elsewhere" excluded. | none |
| I-H02 | hp02: remembered `git.pull_rebase` leads for "pull" with "remembered for “pull”"; unmatched `git.fetch` never shown. The 25% boost cap is not asserted numerically. | none |
| I-H03 | hp02: run saves `persisted.frecency`; reloaded count 2; `usage.rs: learned_queries_normalise_and_survive_restart` asserts `loaded_from == Some("frecency.json")`. Failed-invocation recording and headless bypass are not asserted. | real XDG cache path I/O; headless `run` |
| I-H04 | `usage.rs: frecency_decays_saturates_and_bounds_uses`: one use 0.5, ten-day-old use counts half, count capped at `MAX_USES`, future stamp clamped to `now`, 95-day record and its query pruned; `corruption_versions_merge_and_opt_out`: "{" gives "unreadable", `{"v":7}` gives "schema 7". | real restart |
| I-H05 | hp02: "system.resources" survives the merge; `usage.rs: corruption_versions_merge_and_opt_out`: merged stamps `[now - 5, now - 3]`, save error "EACCES" is `Err` and count stays 1. Async load after first paint is not asserted. | lock, temp rename, concurrent filesystem writer |
| I-H06 | hp02 opt-out block: no "remembered", nothing persisted, silent; `usage.rs: corruption_versions_merge_and_opt_out`: `save` error contains "HOLLA_NO_HISTORY". | env var read |

**Deferred remainder:**
- Durable usage-store I/O, real restart/migration and concurrent filesystem writers (Later).
- v1 legacy read or explicit migration is not exercised; only unknown-schema rejection is.
- Failed invocations remembered and headless runs excluded are not asserted.
- Clipboard requests being ignored is not asserted.
- Match emphasis on Unicode graphemes has no assertion.

**Limits:**
- The clock is the fixture epoch; decay is arithmetic on seeded stamps, not elapsed time.
- The store is an in-memory string; atomic rename and locking are not modeled.
