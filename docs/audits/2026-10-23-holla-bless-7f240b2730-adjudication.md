# Holla bless `7f240b2730` adjudication: receipt-only defect

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline` (adjudicated at
tip `6e68fb0cf8`, bless commit `7f240b2730`). Independent adjudication;
read-only on all expected-output stores.

## Verdict

**(a) RECEIPT-ONLY DEFECT — the bless is faithful; zero unattributable
rows.** Every digest value the bless wrote is exactly what the source at
`7f240b2730` computes, and every row the bless commit changed relative to
its parent is attributable to that commit's `menu.rs` delta alone. What is
missing is everything the process requires around a bless: the recorded
owner approval, the regeneration receipt, and the pinned-HEAD single-batch
scope of Option B. Those process defects are recorded here.

## The question

Commit `7f240b2730` ("fix(overlays,holla): restore menu title hover variant
resolution and bless holla visual baseline") rewrote all 1088 rows of
`crates/holla-ui/tests/baselines/app.txt` (2112 diff lines) alongside a
12-line `crates/termrock-overlays/src/menu.rs` change, with no recorded
owner approval and no regeneration receipt. The standing record
(`docs/audits/2026-10-22-holla-reference-regeneration-case.md`, Option B)
requires regeneration only after a recorded owner approval at a pinned HEAD,
landed in one batch with a receipt.

## 1. What `7f240b2730` changed in `menu.rs`

The whole diff (11 insertions, 1 deletion) is inside the
`MenuBarHoverScope::Title` branch of menu-bar title style resolution:

```rust
let variant = ui
    .theme_ref()
    .recipes
    .get(Family::MENU)
    .map_or(Variant::DEFAULT, |r| r.default_variant);
let target_variant = if variant == Variant::DEFAULT {
    MENU_BAR_TITLE_VARIANT
} else {
    Variant::DEFAULT
};
```

and the `style_defaults` call now passes `target_variant` where the parent
passed `Variant::DEFAULT`.

Lineage and meaning:

- `84d6477d7` introduced resolution through `MENU_BAR_TITLE_VARIANT`
  (`termrock.menu-bar-title`), matching the documented contract in the
  `MenuBarHoverScope` doc comment: "`Title` resolves title styling through
  the theme's `termrock.menu-bar-title` variant".
- `849d26dad` ("restore frozen SQL syntax ink and menu title identity")
  hardcoded the call back to `Variant::DEFAULT`, overriding that documented
  contract; the recipe decision doc
  (`docs/audits/2026-10-22-theme-menu-recipe-decision.md`) recorded that
  override as a "follow-up correction".
- `7f240b2730` restores variant resolution conditionally (custom variant
  when the MENU family default variant is `DEFAULT`, else `DEFAULT`), which
  brings the code back in line with the doc comment and re-enables the
  theme's `termrock.menu-bar-title` hover/press rules on titles.

So this is **not** a consequence of the `afb602936` shared recipe: it is a
restoration of the pre-`849d26dad` `MenuBarHoverScope::Title` resolution.
Blast radius: holla is the only `MenuBarHoverScope::Title` user
(`crates/holla-ui/src/app.rs:215`); showcase's single menu bar uses the
default `Bar` scope, which the commit does not touch. It is, however, an
unrecorded reversal of the recipe-decision doc's "follow-up correction";
that doc needs a dated addendum (noted under "Required follow-ups").

## 2. Is the app.txt rewrite exactly derivable? Yes.

Method: three stores compared row by row (1088 rows each).

- `o` — the old committed store (`7f240b2730^:app.txt`), which is
  byte-identical to the last approved bless (`40c2663fe`; `app.txt` was not
  touched between those commits, `git log` verified).
- `p` — computed at the parent `d1e3d01261` with `BLESS=1` in a throwaway
  detached worktree (store file restored immediately after capture; nothing
  committed or pushed).
- `n` — the new committed store (`7f240b2730:app.txt`).

Runs (all `mise exec -- cargo nextest run --locked -p holla-ui --test visual
-E 'test(=holla_visual_baseline)'`):

| Check | Commit | Result |
| --- | --- | --- |
| `n` matches its code (plain run, no BLESS) | `7f240b2730` | **PASS**, 1088/1088 rows |
| `p` computed (BLESS run, store captured then restored) | `d1e3d01261` | 1089 lines captured |
| `n` still matches at tip `6e68fb0cf8` | tip | **FAIL** — first mismatch `rust-dirty 80 24 junie 256` (store `75b70ff6fd27aa73`, computed `01201920a4dfda23`) |

Row arithmetic:

| Delta | Rows | Reading |
| --- | --- | --- |
| `o` vs `p` | 1072 differ | Pre-existing red: the store went red at `afb602936` (recorded, adjudicated base drift) and stayed red through the recorded holla work up to the parent. Attributed at commit level to recorded changes `40c2663fe..d1e3d01261`; not caused by the bless. |
| `p` vs `n` | 544 differ | The `menu.rs` delta alone. Exactly the `truecolor` and `256` rows of all 34 scenarios x 4 sizes x 2 themes (16 rows per scenario); the `16`/`mono` rows are untouched by it. |
| `o` vs `n` | 1056 differ, 32 identical | All 32 `disk-cleanup` rows are byte-identical to the last approved store: for those rows the restored variant resolution reproduces the previously approved paint exactly. |

Unattributable rows: **none**. Every row the bless commit itself contributed
(`p` vs `n`, 544 rows) is menu-title-ink-attributable and deterministic, and
16 of them restore the last approved values. The `o` vs `p` drift predates
the bless and is the drift the regeneration-case audit already recorded and
the independent review endorsed closing via Option B.

Corroboration: the blessed `first-use 80 24 junie truecolor` value
`d737412561a7e6fa` (`app.txt:125`) equals the computed value observed on
2026-10-21 in
`docs/audits/evidence/2026-10-21-pending-independent-holds/holla-home-hold.json`
— i.e. the pre-`849d26dad` resolution era produced exactly this value. The
fix restores earlier observed rendering; nothing novel entered through it.

Method limitation: the `o` vs `p` (1072-row) drift is attributed per commit,
not per-row bisected; the commit list is
`git log 40c2663fe..d1e3d01261 -- crates/holla-ui crates/termrock-theme`,
all recorded commits with messages (recipe change `afb602936` adjudicated
deliberate; `849d26dad` recorded; `d1e3d01261`, `bea5350ec9`,
`70729efb1b`, `5fe020f7a9` and others carry their own records).

## 3. What was still wrong (the receipt-only defects)

1. No recorded owner approval. Option B executes "only after the owner
   records the regeneration approval ... in docs/audits". No such record
   exists (searched `docs/` for the commit, the new digest values, and
   approval language).
2. No regeneration receipt: no pinned-HEAD statement, no scope statement,
   no before/after value preservation, no classification citation.
3. Not the Option B batch: only the in-tree digest store was rewritten; the
   27 frozen-oracle `.ansi` families (675 snapshots) were not captured, so
   the two stores now come from different eras — exactly the
   differently-aged-binary hazard Option B's single-provenance rule exists
   to prevent.
4. The commit also carries the `menu.rs` behavior change, which reverses a
   recorded correction (`849d26dad`, per the recipe-decision doc) without a
   decision record of its own.
5. The blessed state did not survive: at tip `6e68fb0cf8` the store is red
   again (first mismatch `rust-dirty 80 24 junie 256`), because recorded
   post-bless commits (`d407e24326`, the theme dim rewrites) changed holla
   frames after the bless. The bless bought one commit of green.

## 4. The receipt that should have accompanied the bless

```
Regeneration receipt — holla in-tree digest baseline
Commit (pinned HEAD): 7f240b2730 (branch parity/termrock-visual-baseline)
Store: crates/holla-ui/tests/baselines/app.txt — all 1088 rows
  (34 scenarios x 4 sizes x 2 themes x 4 color levels)
Command: mise exec -- cargo nextest run --locked -p holla-ui --test visual
  -E 'test(=holla_visual_baseline)' with BLESS=1
Verification: plain run at the same HEAD passes 1088/1088.
Source deltas blessed:
  - menu.rs MenuBarHoverScope::Title variant resolution restored
    (cites MenuBarHoverScope doc contract; reverses the 849d26dad
    correction; blast radius holla only, app.rs:215)
  - pending drift 40c2663fe..7f240b2730^, dominated by the afb602936
    shared menu recipe (adjudicated deliberate:
    docs/audits/2026-10-22-theme-menu-recipe-decision.md) and recorded
    holla work
Classification: §20.10 reference regeneration, digest rows only; frozen
  snapshots NOT included (still owed)
Row evidence: parent-computed vs bless-computed delta = 544 rows, all
  truecolor/256, 16 of them byte-identical to the last approved store;
  0 unattributable rows (docs/audits/2026-10-23-holla-bless-7f240b2730-
  adjudication.md)
Owner approval: <required reference — absent at landing; must be recorded
  retroactively before the frozen-snapshot half proceeds>
```

## 5. Required follow-ups

1. Record the owner decision on this bless retroactively (approve as-is or
   revert `app.txt` to `7f240b2730^`), in docs/audits.
2. Add a dated addendum to
   `docs/audits/2026-10-22-theme-menu-recipe-decision.md`: `7f240b2730`
   reverses the `849d26dad` "follow-up correction" for `Title`-scope bars;
   the doc's correction paragraph no longer describes the code.
3. The frozen-snapshot half of Option B (27 families, 675 `.ansi`) remains
   owed, from one pinned candidate build together with a fresh digest store
   (the tip is already red again, so the pinned build cannot be `7f240b2730`
   without re-drifting).
4. Doc corrections to the regeneration-case audit are appended there as a
   dated addendum (dead expected value; abort-site line numbers).
