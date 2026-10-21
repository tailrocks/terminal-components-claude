# Holla reference conflicts: owner-decision evidence package

Date: 2026-10-22. Branch: `parity/termrock-visual-baseline`, evidence tip
`bdf62e3be`. Author: docs-only evidence lane (no product source changes, no
expected-output changes).

Status: **reference defect record + closure case. No decision is final until
an independent review records its verdict on this document.** Both observed
results are preserved verbatim; no snapshot or baseline file was touched by
this lane.

This packages the two open reference-data conflicts left after the Holla
bypass/plate removal (`docs/audits/2026-10-21-holla-plate-removal.md`, tip
ancestry `8689fe270`):

1. `holla_visual_baseline` digest red: computed vs expected
   `e30768d9339a0fb9`.
2. 15 no-reference parity families and 12 concept families failing the frozen
   oracle on both the tip-reference and candidate builds (frozen snapshots
   show the pre-tag static home).

---

## Conflict 1: `holla_visual_baseline` digest red

### Scope of the test

- Test: `holla-ui::visual holla_visual_baseline`
  (`crates/holla-ui/tests/visual.rs`), baseline
  `crates/holla-ui/tests/baselines/app.txt`.
- Grid: 34 scenarios x 4 sizes (80x24, 100x30, 120x40, 160x50) x 2 themes
  (junie, paper) x 4 color levels (truecolor, 256, 16, mono) = 1088 digest
  rows.
- The harness aborts at the first mismatching row
  (`crates/termrock-test-support/src/digest.rs:284`), so each run exposes one
  row; rows after the first mismatch are unverified by that run.

### Reproduction (exact commands, this lane, worktree
`/private/tmp/termrock-holla-digest-doc`)

```
mise exec -- cargo nextest run --locked -p holla-ui --test visual \
  -E 'test(=holla_visual_baseline)'
```

| Commit tested | Result | First failing row | Expected | Computed | Log |
|---|---|---|---|---|---|
| `bdf62e3be` (branch tip) | FAIL, exit 100 | `first-use 80 24 junie truecolor` | `e30768d9339a0fb9` | `83a32521d5113b5d` | `/tmp/hd-digest-tip.log` |
| `bdf62e3be` (repeat run) | FAIL, exit 100 | same | same | `83a32521d5113b5d` | `/tmp/hd-digest-8689.log` |
| `8689fe270` (holla tip ancestry) | FAIL, exit 100 | same | same | `780fcdd3d9d8d83d` | `/tmp/hd-digest-8689f.log` |
| `40c2663fe` (last holla bless) | **PASS** (113 s) | — | — | — | `/tmp/hd-digest-40c.log` |
| `afb602936` (bisect first bad) | FAIL, exit 100 | same | same | `83a32521d5113b5d` | `/tmp/hd-digest-afb6.log` |
| `5b1b3bf9b` (plate removal) | FAIL, exit 100 | `first-use 80 24 junie 256` | `e9b99f47bf935621` | `5a336a181c655c52` | `/tmp/hd-digest-5b1b.log` |

All observed values are preserved in the table above and in the referenced
logs; none was blessed. The recorded pair from 2026-10-21
(`de673f212d89ea84` vs `e30768d9339a0fb9`, docs
`docs/audits/2026-10-21-holla-plate-removal.md`,
`docs/architecture/termrock-migration-matrix.md:2348`,
`docs/handoff/README.md:347`) is preserved as the originally recorded
observation.

Provenance note on `de673f212d89ea84`: it is byte-identical to the value this
same row (`first-use 80 24 junie truecolor`, line 125) carried in the
pre-`9a581e850` blessed baseline (evidence:
`/tmp/termrock-mono-preexisting-3600c0e-20261020/crates/holla-ui/tests/baselines/app.txt`).
`9a581e850` (fix(holla): align the discovery results header) rewrote home.rs
and re-blessed the row to `e30768d9339a0fb9`. A computed value equal to the
pre-`9a581e850` blessing is consistent with that observation having been
produced against a build/tree predating the `9a581e850` render change; the
exact producing build was not identified. Both observations stand as
recorded.

### Which commit first produced the drift (bisect, this lane)

```
git bisect start 8689fe270 40c2663fe        # good = 40c2663fe verified green today
git bisect run /tmp/hd-scripts/bisect_step.sh
```

Step log `/tmp/hd-bisect-steps.log`, full run `/tmp/hd-bisect-run.log`
(7 tested commits: f375b7409 bad, cf854b9e5 bad, e9375eb4f bad, 56b97a3c6
bad, a71351873 good, afb602936 bad, cb4849273 good):

**First bad commit: `afb602936` — "fix(jackin): match the 120-col capsule tab
menu".** Files:

- `crates/termrock-theme/src/builtin/mod.rs`: the shared `menu` recipe
  changes `Part::TITLE` to `Fg(Muted) + BOLD` and `Part::KEY` to
  `Fg(Secondary)` with an `ACTIVE` rule `HighlightFg`, un-bolded.
- `crates/termrock-viewport/src/viewport.rs` (+98 lines).

Mechanism: a jackin-scoped visual fix moved the shared theme menu recipe; the
holla home renders menu title/key ink through that recipe, so every holla
digest row touching that ink drifted. The computed value at `afb602936`
(`83a32521d5113b5d`) equals today's tip value for the same row: the drift is
base drift from a shared-layer theme change, not a holla regression.
`40c2663fe` (the last `app.txt` bless, itself a holla re-approval) is green;
the red is therefore exactly the window `40c2663fe..8689fe270`, closed at
`afb602936`.

At `5b1b3bf9b` the truecolor row still matched and the 256 tier was already
drifted (`e9b99f47bf935621` vs `5a336a181c655c52`) — corroborated by the
preserved earlier capture `/tmp/holla-digest-audit-fix1.txt` line 1. The
frozen-oracle tier drift (16/256 quantization between binary trees, recorded
in the plate-removal audit) and this in-tree digest drift are separate
phenomena; this audit covers the in-tree digest.

Reference defect recorded: the shared `menu` theme recipe change landed
through a jackin lane and silently changed holla's blessed digests; the holla
baseline was not re-approved in the same change. That is a process defect in
cross-crate visual changes, independent of which closure the owner picks.

---

## Conflict 2: frozen-oracle families vs the pre-tag static home

### Family evidence table

Frozen oracle: `junie-tui::visual_baseline` ignored tests over
`snapshots/holla/{parity,concept}/<family>/<size>/<color>.ansi`, run with the
pinned tuiscotti runner. Case grid per family: 29 registered cases —
40x12 and 60x18 at {nocolor, truecolor}; 72x20, 80x24, 100x30, 120x40,
160x50 at {16, 256, nocolor, none, truecolor}. Committed snapshots per
family: 25 files (5 sizes x 5 colors).

Prior paired runs (preserved, 2026-10-21): candidate build
`/tmp/holla-pair/frozen15-cand.log` (0/435), tip-reference build
`/tmp/holla-pair/frozen15-tipref.log` (0/435), concept candidate
`/tmp/holla-pair/frozen-concept-cand.log` (6/348). This lane re-verified the
case inventory from those logs and did not re-run the paired suites (the
runs are recent, preserved, and each exceeds a single-command budget).

15 no-reference parity families — every registered case fails on BOTH builds:

| Family | Cases | Candidate build | Tip-reference build |
|---|---|---|---|
| brew-services | 29 | 0 pass / 29 fail | 0 pass / 29 fail |
| cleanup-results | 29 | 0 / 29 | 0 / 29 |
| custom-actions | 29 | 0 / 29 | 0 / 29 |
| delete-safety | 29 | 0 / 29 | 0 / 29 |
| disk-navigation | 29 | 0 / 29 | 0 / 29 |
| disk-scan | 29 | 0 / 29 | 0 / 29 |
| docker | 29 | 0 / 29 | 0 / 29 |
| executor | 29 | 0 / 29 | 0 / 29 |
| gradle | 29 | 0 / 29 | 0 / 29 |
| idea | 29 | 0 / 29 | 0 / 29 |
| insights | 29 | 0 / 29 | 0 / 29 |
| platforms | 29 | 0 / 29 | 0 / 29 |
| platforms-linux | 29 | 0 / 29 | 0 / 29 |
| task-input | 29 | 0 / 29 | 0 / 29 |
| upgrade-managers | 29 | 0 / 29 | 0 / 29 |
| total | 435 | 0 / 435 | 0 / 435 |

12 concept families — 348 executions = 12 x 29, 6 pass / 342 fail on the
candidate build. Passing cases (unique IDs, verified from the preserved
log): `disk-cleanup::{80x24,100x30,120x40}::truecolor` and
`launch-failure::{80x24,100x30,120x40}::truecolor`. Unique failing case
names extracted from the log: 24 per family
(`/tmp/hd-ev/concept-cases.txt`); the residual per-family cases (5, or 2 for
the two pass-bearing families) appear in the run's 342-fail total but their
long names are truncated in nextest progress lines, so per-family fail
counts from name extraction are lower bounds. Total: 348 executions,
342 fail, 6 pass — verified.

### Frozen snapshot content verification (this lane, read-only)

Snapshots sampled for blob identity against the frozen `visual-baseline` tag
(`2718a6b81`; the tag itself is on a preserved line, not an ancestor of the
parity tip — snapshots are content-identical, history diverged):

- `snapshots/holla/parity/docker/80x24/truecolor.ansi` — SAME
- `snapshots/holla/parity/docker/120x40/truecolor.ansi` — SAME
- `snapshots/holla/concept/first-use/80x24/truecolor.ansi` — SAME
- `snapshots/holla/parity/brew-services/160x50/256.ansi` — SAME

Decoded with the pinned tuiscotti runner's recorded frame format (per-cell
SGR stream, escape-stripped read-only, script
`/tmp/hd-scripts/decode_ansi.py`, frames under `/tmp/hd-ev/`):

- docker 80x24 truecolor: static home — "Suggested here" follow-log
  suggestions, "Recent here / nothing used here yet", "Explore" strip
  (Tasks Git Files Disk Services System), footer
  `→ docker logs -f --since 10m acme-api-1 · here · read-only`, status row
  `~/work/stack · truecolor · 80×24`.
- docker 120x40 truecolor: same static home plus the paused preview card.
- first-use 80x24 truecolor: static home — "Suggested here" (Analyze disk
  usage / Open btm / Show system resources), "Recent here / nothing used
  here yet", "Explore", footer `→ 'holla disk analyze' · here · read-only`,
  status row `~/scratch · / ━━━ ─ 91% ▲ · truecolor · 80×24`.

Confirmed: the frozen oracle for all 27 families encodes the pre-tag static
home. The live source renders the redesigned adaptive home ("Monitor this
host", "Scanning this folder…", `EDIT Type Filter …` hints) on both the
candidate and tip-reference builds. Closing the gap inside the live source is
therefore replaying pre-tag screens — forbidden by the brief.

---

## Closure options

### Option A — replay the pre-tag screens (FORBIDDEN)

Re-introduce static-home rendering (plates or replay paths) so live output
matches the frozen snapshots and the pre-tag digest rows.

- Exact scope: restore the pre-tag home for 27 frozen families (15 parity +
  12 concept) plus the digest rows that reference it.
- Risk: violates the standing brief (no replaying pre-tag screens), would
  re-create the false-parity plate pattern the plate-removal workstream just
  deleted, and would freeze the product to an oracle that no longer
  describes the intended UI. Not recommended under any reading of the
  current rules; listed only for completeness of the decision record.

### Option B — reference regeneration (requires explicit owner decision)

Approve the redesigned adaptive home as the new reference for the affected
stores, regenerated in one deliberate pass.

Exact regeneration scope:

1. In-tree digest baseline: `crates/holla-ui/tests/baselines/app.txt` — all
   1088 rows (34 scenarios x 4 sizes x 2 themes x 4 color levels), regenerated
   with `BLESS=1` and reviewed like source, one commit, message classifying
   the change per §20.10 and citing this audit.
2. Frozen snapshots: the 27 families above — 27 x 25 = 675 `.ansi` files at
   72x20, 80x24, 100x30, 120x40, 160x50 x {16, 256, nocolor, none,
   truecolor}, captured through the frozen harness bless path
   (`tools/frozen_baseline.py` capture/bless phases, owner-run; this lane
   never runs bless). The two currently passing concept families
   (`disk-cleanup`, `launch-failure`) should be regenerated whole, not
   patchwise, so one candidate build backs one coherent oracle.
3. Single provenance: capture both stores from one candidate build at one
   pinned revision, recorded in the run receipts, to avoid re-importing the
   16/256 quantization delta observed between differently-aged binary trees
   (`/tmp/holla-pair/nr-ref-r` vs `nr-post-r`, recorded in the
   plate-removal audit).

Risks of Option B:

- Blesses base drift wholesale: the digest drift introduced at `afb602936`
  came from a shared theme `menu` recipe change authored for jackin. If that
  ink change is itself wrong for holla, regeneration would approve it
  silently. Mitigation: the independent review must first adjudicate the
  `menu` recipe change on its own merits (it is visible in every app's menu
  surfaces, not just jackin), then the regeneration proceeds.
- The tip keeps moving (gutter/theme re-approvals landed after
  `8689fe270`); a regeneration dragged across several tip moves repeats the
  drift it meant to close. Mitigation: pin the capture revision, land the
  regeneration in one batch, re-run both stores' gates on the exact pushed
  HEAD.
- Coverage loss: the 6 currently passing truecolor cases prove live-source
  equivalence for those frames; regeneration replaces them with new
  expectations for the same frames. Acceptable only because the underlying
  screens did not change (verified: those families' non-home frames match;
  the drift is the home screen), but the reviewer should confirm the
  captured frames for those 6 cases are cell-identical to the old snapshots
  before overwrite.
- Registered but snapshot-less cases (40x12, 60x18 at two color levels per
  family) have no committed `.ansi` today; regeneration must either produce
  them or explicitly keep them out of scope, recorded.

Risks of Option A: stands forbidden; would also contradict the recorded
owner policy that pre-tag replay is not a closure path.

### Recommendation

Option B — reference regeneration, in the scope above, executed only after:

1. an independent review records a verdict on this document (required; no
   decision here is final until then), including an explicit verdict on the
   `afb602936` shared `menu` theme recipe change;
2. the owner records the regeneration approval (which store, which families,
   which sizes/color levels) in docs/audits;
3. both stores are regenerated from one pinned candidate build and landed in
   one batch, with the pre-change values preserved in this audit and in the
   regeneration receipt.

## Preservation

- Digest observations: logs `/tmp/hd-digest-{tip,8689,8689f,40c,afb6,5b1b}.log`,
  bisect logs `/tmp/hd-bisect-steps.log`, `/tmp/hd-bisect-run.log`,
  script `/tmp/hd-scripts/bisect_step.sh`.
- Preserved prior evidence referenced, not duplicated:
  `/tmp/holla-pair/frozen15-cand.log`, `/tmp/holla-pair/frozen15-tipref.log`,
  `/tmp/holla-pair/frozen-concept-cand.log`,
  `/tmp/holla-digest-audit-fix1.txt`,
  `/tmp/termrock-mono-preexisting-3600c0e-20261020/crates/holla-ui/tests/baselines/app.txt`.
- Snapshot decodes: `/tmp/hd-ev/*.ansi` + `/tmp/hd-scripts/decode_ansi.py`.
- No expected-output file (baseline or snapshot) was modified by this lane.
