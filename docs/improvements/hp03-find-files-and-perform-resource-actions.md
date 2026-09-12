# HP03 — Find files and perform resource actions

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Use an in-memory file index with progressive results, exclusions and exact native-path identities. Model Open/Reveal/Copy/Analyze intents and success/failure, including target/cwd/argv and clipboard destination.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real home indexing, OS open/reveal, OSC52/system clipboard transport and worker joins against live providers.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve home file/folder discovery and Open, Reveal, Copy path, Analyze actions. Current file rows are fixture resources; Reveal incorrectly copies cwd.

**Source evidence and mandatory scope:** [matrix HP03](../parity/holla-parity-matrix.md#hp03--find-files-and-perform-resource-actions) — `I-F01`, `I-F02`, `I-F03`, `I-F04`, `I-F05`, `I-F06`, `I-F07`, `I-F08`, `I-F09`, `I-F10`, `I-F11`, `I-F12`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide a Files scope in Here with mixed files/directories, indexed/partial counts and exact resource identity. Rank exact filename/stem, filename substring and fuzzy matches with deterministic path ties; expose bounded results (legacy 100) honestly. Keep an explicit blank-query state. Alternatives must operate on the selected path, not current cwd. Analyze directories directly and files via parent; preview resulting scope.

**Architecture / reusable components:** Use a cancellable file-index adapter; retain ignore/hidden policy, no symlink following and cloud/root exclusions. OS handoff and clipboard adapters receive validated path identity and typed intent. Reuse Picker, menu, Props and existing disk page routing.

**Required deterministic fixture:** `parity-files` — mixed home results; indexing in progress; .ignore/hidden/iCloud exclusions; same basename; Unicode byte-to-grapheme match; open failure; reveal on macOS/Linux; clipboard size/write failure; file versus folder Analyze.

**Acceptance / automated verification:** Assert bounded ranking, exact selected path through refresh/alternatives and cancellation joins pending index work. Assert macOS open/open -R and Linux xdg-open/parent fallback argv, failed handoff visibility, OSC52 payload/cap/flush behavior, and simulated Analyze root. Clipboard result must name actual destination without unsupported success claims. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture progressive results, resource alternatives, failed opener/clipboard and Analyze handoff at narrow width.

**Dependencies:** HP01/HP02/HP18/HP19/HP23; F09/F10/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-files` (fixture fn `files` in src/bin/holla/domain/parity.rs) · `hp03_find_files_indexes_home_truthfully_and_resource_actions_are_exact` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/sim/fs.rs: home_index_and_find_follow_the_legacy_policy`, `src/bin/holla/sim/fs.rs: canonical_parent_reports_link_ancestors_and_keeps_the_leaf`. · row proofs in src/bin/holla/app_tests_rows.rs: `i_f10_i_f12_file_actions_reveal_through_open_r_and_analyze_the_folder`.

**What the journey asserts:**
- "Find files under home" opens "Files ›"; "readme" lists "README.md" with parents "~/work/notes" and "~/work/app".
- "node_modules" and "scratch" never appear (both in `~/.ignore`).
- "café" finds "café menu.txt"; "東京" renders "東", "京" and ".md".
- "hidden-config" and "cloud-only" each show "0 results" (hidden entry; `Library/Mobile Documents` root excluded).
- "main.rs" lists "work/app/src/main.rs" and never "link-to-app/src" (symlink listed as itself, not traversed).
- Enter on the "todo" hit opens a modal actions menu; choosing the third row states the copy outcome ("OSC 52", "clipboard" or "Copied") and `world.clipboard` equals `/Users/alex/work/notes/todo.md` (30 bytes, under the fixture `osc52_limit` of 64; over the limit `clipboard` must be `None`).
- Open runs argv `open /Users/alex/work/notes/todo.md @/Users/alex/work/notes`.

**Captures:** `shots/h_hp03_find` (results for "readme"), `shots/h_hp03_find_unicode` (results for "café"), `shots/h_hp03_actions` (actions menu on a hit: Open, Reveal, Copy full path, Analyze disk usage); base matrix `shots/h_parity_files_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| I-F01 | hp03: "Find files under home" reaches "Files ›" in a fixture with no project. Tool-independence of the entry is not asserted. | none |
| I-F02 | hp03: mixed hits under home ("~/work/notes", "~/work/app"); `fs.rs: home_index_and_find_follow_the_legacy_policy` asserts `find(&idx, "   ")` is empty. | real home walk |
| I-F03 | Partly: hp03 asserts the "0 results" count text; the fixture `index` source answers at tick 30 but no assertion covers the indexing-in-progress state. | 50 ms refresh, worker |
| I-F04 | `fs.rs: home_index_and_find_follow_the_legacy_policy`: stem match `hits[0].score == 0`, substring `score >= 10`, 300 files cut to `FIND_MAX_RESULTS`. Order is not asserted through the UI. | none |
| I-F05 | hp03: name and parent both rendered ("README.md", "~/work/notes"); "café menu.txt" and wide "東"/"京" cells. Byte-to-grapheme highlight translation is not asserted. | none |
| I-F06 | hp03: "node_modules", "scratch" absent; hidden and cloud-only "0 results"; symlink not traversed; `fs.rs: home_index_and_find_follow_the_legacy_policy` asserts the same policy plus `Library/Caches` kept. | real FFF ignore parsing |
| I-F07 | Not asserted: no cancellation or worker-join test. Unproven. | cancel and join the index worker |
| I-F08 | hp03: Enter on a hit opens the actions menu (`modals` non-empty); Ctrl+U clears the query. Esc ladder is not asserted. | none |
| I-F09 | hp03: macOS argv `open /Users/alex/work/notes/todo.md @/Users/alex/work/notes`. Linux `xdg-open` and opener failure are not asserted. | real OS handoff |
| I-F10 | `i_f10_i_f12_file_actions_reveal_through_open_r_and_analyze_the_folder`: "Reveal in the file manager" on `index.html` starts `open -R /Users/alex/work/site/index.html` and succeeds; Linux says "No opener on devbox" (`hp23_…`). | real reveal |
| I-F11 | hp03: copy outcome stated; `clipboard == Some(path)` under the 64-byte fixture limit, `None` over it. Terminal flush is not modeled. | OSC52 terminal write and flush |
| I-F12 | `i_f10_i_f12_file_actions_reveal_through_open_r_and_analyze_the_folder`: "Analyze disk usage" on the `assets` row opens `Disk usage · ~/work/site/assets` nested under the browser (`Here › Files › site › Disk › Usage`) with `world.scan` rooted there. | none |

**Deferred remainder:**
- Real home indexing, OS open/reveal, OSC52 and system clipboard transport, worker joins (Later).
- Cancellation joining pending index work is not tested.
- `open -R`, `xdg-open`, parent fallback and failed handoff visibility are not asserted.
- Analyze root for file versus directory is not asserted.
- Bounded ranking is asserted at fs level only, not through the rendered list.

**Limits:**
- Clipboard is a world field; no terminal receives OSC 52 bytes.
- The index is a fixture tree; no real filesystem or cloud root is read.
