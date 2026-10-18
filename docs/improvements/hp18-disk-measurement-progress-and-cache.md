# HP18 — Disk measurement, progress and cache

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build truthful fixture observations for allocated/apparent bytes, hardlinks, symlinks, partial/error states, bounded updates, cancellation and cache freshness. Use virtual storage/clock for schema/TTL/merge/restart cases.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live filesystem scanner, OS file metadata/dataless behavior, physical measurements and durable cache I/O.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve trustworthy disk bytes, progressive scans, cancellation, error states and durable cached hints. Timed fixture GB counts lack scanner semantics.

**Source evidence and mandatory scope:** [matrix HP18](../parity/holla-parity-matrix.md#hp18--disk-measurement-progress-and-cache) — `LD004`, `LD005`, `LD006`, `LD010`, `LD011`, `LD007`, `LD008`, `LD009`, `LD017`, `LD018`, `LD019`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model file/directory roots, allocated versus apparent bytes, hardlink identity, symlink leaves, hidden entries, skip subtrees and inaccessible/dataless outcomes. Stream bounded updates after first paint without moving active targets. Cache age is a hint: root/depth-two v3 sizes, seven-day/mtime validation, pre-scan snapshot, changed-path omission, merge/atomic save; partial and deep-unverified data must remain labeled.

**Architecture / reusable components:** Introduce a filesystem observation adapter and immutable identity-bearing events used by simulation and later live scanner. Bound/coalesce producer queues and cancel/join workers on owner removal/rescan. Separate measured bytes, apparent size, estimate, observation freshness and physical free space. Reuse TreeView/progress/Props/StatusBar.

**Required deterministic fixture:** `parity-disk-scan` — sparse files, hardlinks, symlink dirs, hidden/skip roots, permission/missing/I/O/dataless errors, rapidly changing trees, bounded burst, cancel/rescan, old/new/missing cache nodes, stale TTL/mtime, two writers and corrupt cache.

**Acceptance / automated verification:** Assert recursive totals/counts and dedup saturating arithmetic, no symlink traversal, live partial/error counts, bounded queue, cancellation acknowledgment, no stale generation replacement. Assert exact cache schema/TTL behavior, no disk/history I/O before first paint, changed-path exclusion and valid concurrent merge. Do not label incomplete or depth-limited freshness exact. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture cached-first/live/partial/error/cancelled states, allocated/apparent differences and cache replacement preserving selection.

**Dependencies:** HP19/HP20/HP22/HP23; F05/F11–F15/F23.

## Evidence

**Slice status:** current simulated slice complete for the journeyed variants; gap: bounded burst delivery, rapidly changing trees, two writers and a corrupt cache are proven only at unit level, and file roots, skip prefixes and macOS path exclusions are not journeyed · Later operational clauses open.

**Scenario and journey:** `parity-disk-scan` (fixture fn `disk_scan` in src/bin/holla/domain/parity.rs) · `hp18_disk_scan_streams_measures_exactly_and_keeps_cached_hints_as_hints` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/sim/fs.rs: scan_deduplicates_hardlinks_and_never_follows_links`, `src/bin/holla/domain/cleanup.rs: size_cache_validates_ttl_mtime_schema_and_merges`, `src/bin/holla/app_tests_proofs.rs: idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree`.

**What the journey asserts:**
- "Analyze disk usage" opens `Disk › Usage` with `scanning ·` in the text; `scan.ticks == 48`.
- Cache at load: `hint("/Users/alex/work/data/media", mtime, now)` is `Some` ("a valid entry is a hint"); the `/expired` entry is absent ("expired entries are dropped at load"); `/many` with a stale mtime gives `None`; `/gone` gives `None` or the path does not exist.
- After 60 ticks the text contains `scan complete`; `media.allocated == 12 * BLOCK + BLOCK` ("the hardlink counts once, plus the directory block"), `media.entries == 3`, a child with `link == true` ending `/link` is a leaf.
- `sparse.img` has `(apparent, allocated) == (50 * BLOCK, 3 * BLOCK)`; `.hidden` has `4 * BLOCK + BLOCK` ("hidden entries are scanned").
- `locked.error.is_some()` ("an unreadable folder is an error, not a zero"); `cloud.error.is_some()` with empty children ("dataless is never materialised"); the text contains `unreadable`.
- After the scan the cache has an entry for `/Users/alex/work/data/media` and `persisted.sizes` contains `"v":3`.
- `r` shows `Rescanning` and `scan.generation == 2`; `x` after 5 ticks shows `partial · scan cancelled` with `scan.cancelled == true`; 60 ticks later the text still contains `partial`, and the props show `Freshness` with `partial · scan cancelled`.
- `fs.rs: scan_deduplicates_hardlinks_and_never_follows_links`: allocated `BLOCK * (10 + 1 + 1)`, `entries == 4` ("both hardlink names and the symlink are entries"), link children empty, hidden excluded by default, sparse `(100 * BLOCK, 2 * BLOCK)`, skip root subtracts exactly `a.allocated`, `max_depth` keeps totals, `errors() == 1` with `allocated > 0`.
- `cleanup.rs: size_cache_validates_ttl_mtime_schema_and_merges`: depth two kept and depth three not, `mtime + 1` invalidates, `CACHE_TTL_SECS + 1` invalidates, round trip equal, `schema 2` and `nope` give `load_error`, a touched path is omitted after the scan, two writers merge by recency and prune the expired entry.
- `app_tests_proofs.rs: idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree`: a complete scan never rebuilds on idle ticks; a sort change rebuilds exactly once.

**Captures:** `shots/h_hp18_scanning` (`scanning · 32 of 73 entries`, rows `… · scanning`, `Freshness  live · 44%`) · `shots/h_hp18_complete` (`2 unreadable · grant Full Disk Access to include them · scan complete`, rows `dataless` and `permission denied`, `Apparent … · sparse`, `hardlinks counted once`, `Freshness  live · complete`) · `shots/h_hp18_cancelled` (`partial · scan cancelled at 27%`, `Freshness  partial · scan cancelled`) · base matrix `shots/h_parity_disk-scan_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| LD004 | `hp18_…`: directory branches with recursive `allocated` including the directory block (`12 * BLOCK + BLOCK`), `entries == 3`; `scan_deduplicates_…`: totals propagate through `a`. A file root is not asserted. | Live filesystem walker. |
| LD005 | `hp18_…`: sparse `(50 * BLOCK, 3 * BLOCK)` apparent versus allocated; hardlink counted once with both names as entries; `scan_deduplicates_…`: `entries == 4`. Saturating arithmetic is not asserted. | `st_blocks`/`st_size` and `(dev, ino)` identity on a real filesystem. |
| LD006 | `hp18_…`: `.hidden` scanned (`4 * BLOCK + BLOCK`), `link` is a leaf, never followed; `scan_deduplicates_…`: hidden suppressed by default, skip root subtracts exact bytes. Skip prefixes are not journeyed. | Real symlink and hidden-entry policy. |
| LD010 | `hp18_…`: `cloud` is an error node with no children ("dataless is never materialised"); `h_hp18_complete` labels it `dataless`. | macOS dataless materialisation policy, `SF_DATALESS`, QoS. |
| LD011 | Not represented: no test asserts `/System/Volumes/Data` or Mobile Documents skipping or worker counts. | Platform worker counts and mount/path exclusions. |
| LD007 | `hp18_…`: `scanning ·` on first paint, `scan complete` after the 48-tick scan (asserted at 60 ticks); `idle_ticks_rebuild_nothing_…`: no rebuilds on idle. Bounded 4096-event queue and coalescing are not asserted. | Coordinator/worker threads and bounded event channel. |
| LD008 | `hp18_…`: `r` gives `Rescanning` and `generation == 2`; `x` gives `partial · scan cancelled`, `cancelled == true`, partial tree retained after 60 ticks. Selection reset and worker join are not asserted. | Worker cancel/join and Spotlight abort. |
| LD009 | `hp18_…`: `locked.error` and `cloud.error` set, text `unreadable`; `scan_deduplicates_…`: `errors() == 1` with partial accounting; `h_hp18_complete` shows `permission denied`, `dataless`, `grant Full Disk Access`. Missing and other I/O classes are not asserted. | Real errno classification. |
| LD017 | `hp18_…`: `"v":3` persisted, expired entry dropped at load, stale mtime `None`; `size_cache_validates_…`: depth two only, exact mtime, seven-day TTL, `schema 2` and `nope` rejected. Cache file path is not modeled. | `sizes.json` path resolution and file I/O. |
| LD018 | `size_cache_validates_…`: touched path omitted after the scan, two writers merge by recency, expired pruned; `hp18_…`: the finished scan adds the `media` entry to `size_cache.entries`. Atomic rename and stderr surfacing are not modeled. | Lock, temp file plus rename, writer error reporting. |
| LD019 | `hp18_…`: a valid entry is a hint until the live number arrives; `Freshness` prop labels `live · 44%`, `live · complete`, `partial · scan cancelled`; src/bin/holla/screens/disk.rs words cached rows `hint only` and `a hint, not a measurement`. Path-reconciled replacement preserving selection and disappearing stale-only rows are not asserted. | none |

**Deferred remainder:**
- Live filesystem scanner, OS file metadata/dataless behavior, physical measurements and durable cache I/O (Later).
- Not proven by tests: bounded queue; no disk/history I/O before first paint; no stale generation replacement (only `generation == 2` after rescan); cache replacement preserving selection; allocated/apparent difference as a capture of its own; saturating byte sums.
- Captures named in the contract but absent from tools/holla_parity_flows.sh: cached-first frame on its own, allocated/apparent difference frame, cache replacement preserving selection.

**Limits:**
- `BLOCK` sizes, mtimes and the 48-tick scan are fixture values on a virtual clock; real block accounting, `(dev, ino)` identity and scan timing cannot be proven here.
- The dataless and permission errors are fixture flags, not OS results.
