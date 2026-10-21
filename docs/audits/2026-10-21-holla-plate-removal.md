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
upgrade-managers) have no tip reference capture; they never had plates and are
unchanged by this work.

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

## Remaining work

Full frozen-corpus live-compare acceptance for Holla is still open (the
branch-level candidate corpus remains red for reasons outside Holla; see the
matrix paired-test section). The 15 no-reference parity families need tip
references before the same paired proof can cover them.
