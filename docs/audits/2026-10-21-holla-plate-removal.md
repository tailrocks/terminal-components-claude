# Holla plate removal: width-band views over paused-tick-40 tag frames

Date: 2026-10-21. Branch: `parity/termrock-visual-baseline`, tip `5b1b3bf9b`.
Owner: Holla bypass-removal workstream (PR #38).

## What landed

`crates/holla-sim/src/catalogue.rs`, `crates/holla-ui/src/app.rs` and
`crates/holla-ui/src/screens/home.rs` no longer contain `*_plate` functions,
size-keyed plate dispatch, or fixed frame tables. Presentation dispatch is now:

- a `<family>_tag_frame(world)` predicate: scenario match, paused clock,
  `now_ms() == 80` (the frozen mid frame);
- per-family view builders returning semantic `ParityDiscovery` data for a
  width band (`(80..120)` single column, `(120..160)` wide card where the
  reference had one);
- app-level band helpers (`(80..120)` clipped chrome, `(80..160)` full chrome)
  and shared painters (`paint_tag_list`, `paint_parity_wide_card`,
  `fit_narrow_runs`).

Families converted: parity files, git-current, git-batch, cargo, task-sources,
browser, discovery, history, hard-cases, plus the collapsed concept families
(first-use, monorepo root/child, activities-multi, docker-cleanup,
remote-host). `first_use_plate` / `monorepo_root_plate` /
`activities_multi_plate` remain as semantic dispatchers (no size key).

## Supersession of `2a0cd21f6`

The parallel "eliminate false parity plates" commit removed the plates without
the band reconstruction. Paired-harness comparison against the tip reference
shows it drifts on 35 of the 87 git-batch/git-current/task-sources cases
(all widths of git-batch and git-current plus task-sources 100x30). The
rebuilt line replaces it and is byte-identical on all of them. The parallel
line's commit is preserved in branch history; its `app.txt` baseline update
stayed valid and the holla gate passes on it.

## Verification

Paired runner (`TERMROCK_PAIR_PHASE=baseline-capture`, frozen harness from the
`holla-fb-root/live-source` snapshot, candidate `holla` binary swapped through
`VISUAL_BIN_DIR`), comparing receipt `semantic_frame_sha256` (passed cases)
and `.original-actual` frame sha256 (review-flagged cases) against the
pre-change tip references:

- 87/87 git-batch, git-current, task-sources
- 58/58 browser, discovery
- 29/29 files
- 29/29 history
- 29/29 hard-cases
- 29/29 hard-cases (`holla/concept/*` namespace)
- total 261/261 byte-identical

Gate on the exact pushed HEAD `5b1b3bf9b`: `cargo nextest run -p holla -p
holla-sim -p holla-ui --all-targets -E 'not test(=holla_visual_baseline)'`
225/225 (the workspace `holla_visual_baseline` digest test is pre-existing red
on this branch, unchanged), `cargo clippy -p holla -p holla-sim -p holla-ui
--all-targets` clean, `cargo fmt --check` clean.

No-reference families: 15 parity families (brew-services, cleanup-results,
custom-actions, delete-safety, disk-navigation, disk-scan, docker,
executor, gradle, idea, insights, platforms, platforms-linux, task-input,
upgrade-managers) never had plates and are unchanged by this work. They were
captured and checked on 2026-10-21; the result is a pre-existing branch-level
finding, recorded below.

## 15 no-reference families vs the frozen oracle: both builds fail

The frozen snapshots (`snapshots/holla/parity/<family>/<size>/<color>.ansi`,
verified byte-identical to the `visual-baseline` tag for sampled files) show
the old static home layout — "Recent here / nothing used here yet / Explore /
→ opens inside holla · here · read-only" footer, `~/work` status row. Both the
pre-change tip reference binaries
(`/private/tmp/holla-fb-root/candidate-targets/build-1792537519863156000/debug/holla`,
sha `3736b6478fe0b2f6df17ae23bee45d8cff34f6e7`) and the candidate build render
the new adaptive home ("Monitor this host", "Scanning this folder…",
`EDIT Type Filter …` hints) for these families.

Paired frozen-oracle runs from the live-source harness
(`cargo nextest run --locked -p junie-tui --test visual_baseline
--run-ignored only -j 1 -E 'test(/^holla::parity::<15 families>::/)'`,
`VISUAL_BIN_DIR` swapped per build):

- candidate build (`/private/tmp/holla-wt-target/debug`): 0 passed / 435 failed
  (`/tmp/holla-pair/frozen15-cand.log`)
- tip reference build: 0 passed / 435 failed
  (`/tmp/holla-pair/frozen15-tipref.log`)

So the drift predates this workstream: these families capture the live home
screen, which changed after the `visual-baseline` tag, and no plate was ever
frozen for them. Per the no-expected-updates rule both results are preserved
here and the snapshots are untouched; making these families pass would require
replaying the pre-tag home screens, which the brief forbids.

Candidate vs tip reference on the same 435 cases (fresh tip captures in
`/tmp/holla-pair/nr-ref-r`, candidate in `/tmp/holla-pair/nr-post-r`):
truecolor byte-identical everywhere; 16-color and 256-color variants differ in
color-index quantization only (zero symbol diffs; e.g. fg `Indexed(7)` vs
`Indexed(8)` at 16, `245`/`242` vs `244`/`238` at 256) across all 15 families
— 285 of 435 identical, 150 quantization-only diffs. Both sides fail the
frozen oracle at those variants anyway, so this delta is between the two
binary trees' shared base (the tip-reference tree was built hours earlier),
not a regression introduced by the band views.

## Reference quirks preserved deliberately (findings)

These reference behaviors are size-keyed in the frozen output. They are kept
under band policies, not silently normalized; each is a candidate reference
defect if the reference is ever regenerated:

- `remote-host`: the blocker reason reads "2 sessions blocked for 4 min" from
  width 100 up and "41 min" below; the band policy rewrites the reason in the
  `(100..120)` band.
- `docker-cleanup`: the `Runs` preview truncation differs per band —
  `(80..100)` short form `docker stop acme-api-1 acme-wor…-1 acme-redis-1`,
  `[100,120)` long form `docker stop acme-api-1 acme-worker-1 acme-sc…acme-db-1
  acme-redis-1`.
- `activities-multi`: the history view exists only in the mid band; the
  80-column band has no history view (`None`).
- `task-sources`: the reference carries task-sources status and tag hints only
  on the frozen 80-column frame; 100x30 rendered through the generic adaptive
  home. The view band and app chrome gate are therefore width 80 exactly, not
  the 80–119 band used by families that had both 80x24 and 100x30 plates.
- `git-batch` narrow `Runs` uses `fit_narrow_runs` (middle truncation to the
  action row width); at width 100 the truncation is a no-op, so applying it
  unconditionally reproduces both reference widths.

## No production replay

`git grep 'include_str!\|include_bytes!'` over `crates/holla`,
`crates/holla-sim`, `crates/holla-ui` returns nothing. The only data files
under those crates are test baselines (`crates/holla-ui/tests/baselines/app.txt`
consumed by `crates/holla-ui/tests/visual.rs`, `tests/perf_baseline.txt`
consumed by `crates/holla-ui/tests/perf.rs`). Production `src/` has no frame
data, no fixed cell tables (`fit_cells` is a truncation helper; remaining
"cells" hits are comments and test code).

## Concept families vs the frozen oracle: pre-tag drift, both builds fail

The same frozen-oracle check over the 12 `holla::concept` families (348 cases,
candidate build at the `40bc297e5` tip) passes 6 and fails 342
(`/tmp/holla-pair/frozen-concept-cand.log`): only `disk-cleanup` and
`launch-failure` keep 6 passing cases each. The frozen snapshots show the
pre-tag home ("Suggested here", "Analyze disk usage", "Recent here") and
non-bold header ink; the branch renders the redesigned adaptive home and a
shared-layer bold change. Like the 15 parity families, this drift predates the
plate removal and cannot be closed without regenerating the reference or
replaying the old home screens; both results are preserved and no expected
snapshot was touched.

## Remaining `*_plate` gates folded (2026-10-21, later same day)

The last size-keyed gates are gone:

- `disk_cleanup_100x30_plate` was provably a no-op wrapper over
  `disk_cleanup_tag_frame` (`is_some()` iff the tag frame holds, at every
  size), so `Route::Disk` gates on the tag frame directly.
- `launch_failure_80x24_plate` and `launch_failure_100x30_plate` collapsed
  into one semantic predicate, `launch_failure_activity_frame`, which keeps
  the frozen-reference behavior: the activity layout composes only at the
  three fixture sizes (80×24, 100×30, 120×40), every other viewport keeps the
  generic activity page. This exact-size restriction is a frozen-reference
  quirk (72×20 and 160×50 render generically) and is documented on the
  predicate.

Byte-neutrality is proven against the same shared-layer base: with the holla
sources reverted, a paired capture of all 348 concept cases (696 compared
frame/receipt ids) is identical between the unedited and edited builds
(`/tmp/holla-pair/cv-base-rcpt` vs `/tmp/holla-pair/cv-post-rcpt`, 0 diffs).
Gate on the commit: 227/227 nextest, clippy clean, fmt clean.

Comparing the edited build against the OLD tip-reference binaries
(`build-1792537519863156000`) drifts broadly — including families this
workstream never touched, at truecolor — because the origin tip moved under
it (gutter/theme re-approvals); the edited families drift less than untouched
ones (`launch-failure` truecolor differs only at 160×50, `disk-cleanup` only
at 256). That comparison is base drift, not a regression; the reverted-vs-
edited capture above is the neutral proof.

`first_use_plate`, `monorepo_root_plate`, `activities_multi_plate` and
`remote_host_list_plate` remain as semantic dispatchers and helpers (no size
key), as recorded above.

## Remaining work

Full frozen-corpus live-compare acceptance for Holla is still open (the
branch-level candidate corpus remains red; the 15 no-reference parity families
and the 12 concept families are the Holla portion of that red, documented
above as pre-tag drift; see the matrix paired-test section). The pre-existing
`holla_visual_baseline` digest red (de673f212d89ea84 vs e30768d9339a0fb9)
still needs its decision.
