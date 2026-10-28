# Independent review: modal backdrop adjudication + frame regeneration — 2026-10-21

Reviewer: independent subagent, read-only, no authorship overlap with the adjudication or regeneration work. Task rule applied: "If frozen artifacts conflict with repeatable reference output, record a reference defect. Preserve both results. Require independent review of the conflict."

## Verdict

**The adjudication + regeneration chain STANDS. No tier needs re-adjudication. Zero MAJOR findings. Five MINOR findings (dispositions below).**

## What was verified (all exit 0 unless noted)

Against `ca867254a` (regeneration), `45c66c749` (corpus-check tool), `0dfd18ec9` (docs), pre-regen base `2ca2c7ad3`:

1. **Artifact scope clean** — regeneration touches only `baselines/tuiscotti-v1/` (33 tiers × 9 artifacts + corpus-index + admission-record), `baselines/regen-sidecars/`, `snapshots/`, `docs/`; store diff confined to the 11 claimed scenarios. Tool commit touches only `tools/frozen_baseline.py` + its tests. No source smuggled.
2. **Sidecar preservation byte-identical** — 10/10 spot-checked sidecar artifacts sha256-identical to `git show 2ca2c7ad3:<original>`.
3. **Receipts 33/33** — every before-hash matches blobs at `2ca2c7ad3`, every after-hash at `ca867254a`. Truecolor/256 untouched anywhere in the diff. corpus-index/admission-record chain consistent.
4. **DIM self-consistency** — regenerated dialogs 16-tier now carries the DIM runs (0 → 46); picker snapshots untouched and byte-identical before/after (matching the audit's "never defective" claim).
5. **corpus-check detects the defect class** — pre-regen sidecars disagree with snapshots (`dim=4577` picker class); post-regen tiers agree. Store-wide claim corroborated by sampling (22/28 tier-pairs disagree ≈ 79% vs claimed ~1170/1510 ≈ 77.5%).
6. **Gap wording honest** — 78 red vs 75 control attributed to the still-open app/theme candidate parity gap; no closure claimed.

## MINOR findings and dispositions

1. **Conflict marker committed in `0dfd18ec9`** (matrix line 1777, diff3 `|||||||` line) — already removed by a later branch commit; no action.
2. **Dangling doc reference** — `ca867254a` message cites the regeneration audit added only in `0dfd18ec9`; commit-ordering nit, content intact; no action.
3. **Fail-open cross-store pass** (`frozen_baseline.py`: a missing snapshot dump silently skipped, so deleted snapshot coverage would report a vacuous pass with `cross_checks=0`) — **FIXED in this batch**: `corpus_modifier_parity` now raises `RunnerError` when a snapshots root is supplied but zero scenarios matched; test `test_cross_store_pass_refuses_vacuous_coverage` replaces the vacuous-pass assertion; runner suite 119/119 OK.
4. **Test-count claim mismatch** (audit said 85; suite defines 119) — **FIXED in this batch**: audit wording corrected to 119.
5. **Partial circularity for dialogs 16/none/nocolor** — regeneration overrides BOTH frozen stores in favor of freshly rebuilt tag-binary behavior, and the dialogs snapshot was regenerated in the same change the cross-store check now compares against. **Recorded, accepted with reason**: the choice rests on tag-source archaeology (DIM branch absent at `4a79c0a2d`, introduced by ancestor `d720a1c2e`), was settled by independent SGR re-measurement before replacement, and the superseded bytes are fully preserved in sidecars — conforming to the both-results-preserved rule. Downstream consumers must know dialogs 16/none/nocolor expected frames match rebuilt tag binaries, not the tag's recorded streams. The picker direction rests on untouched legacy snapshots (stronger). No re-adjudication warranted.

## Residual open work (unchanged)

The ~78 red tier comparisons remain the app/theme streams' open candidate parity gap for the dimmed backdrop; the regenerated frames are the correct oracle for that work.
