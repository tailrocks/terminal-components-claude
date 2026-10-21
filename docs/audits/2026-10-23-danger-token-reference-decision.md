# Owner decision package: the danger-ink token reference (`#f05d5d` frozen vs `#e44545` corpus)

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline` (base tip
`740374edb2`). Status: **OPEN — one owner decision plus four sub-decisions
requested.** This document consolidates the scattered danger-ink record
into a single decision package; it states no new measurements without a
command listed in the verification appendix.

## Background (all cited, nothing new)

| Reference | TrueColor danger | 256 tier | danger_soft | Source |
|---|---|---|---|---|
| Tag `visual-baseline` (`4169783ae`) frozen source | `#f05d5d` | authored `error = i(203)`, `error_strong = i(210)` (`src/theme.rs:943`, `:944`); `i(203)` renders 1145 fg cells | `#e8a0a0` (`RED_SOFT`, `src/theme.rs:531`, mapped to `error_soft` at `:747`; authored 256 `error_soft = i(174)`) | tag `src/theme.rs` |
| Candidate builtin (after restore `740374edb2`) | `#f05d5d` (`crates/termrock-theme/src/builtin/junie.rs:36`) | derived `Indexed(210)` by the pinned reference formula; authored pin still `Indexed(167)` (`crates/termrock-theme/src/palettes.rs:98`) | `#d98a8a` (`RED_SOFT`, `junie.rs:39`; test pin `crates/termrock-theme/src/builtin/mod.rs:1221`) | parity builtin |
| Corpus `baselines/tuiscotti-v1` (gate oracle via `DEFAULT_BASELINE_STORE`, `crates/termrock-conformance/tests/visual_baseline/support.rs:832`) | `#e44545` | `Indexed(167)` (20 000 fg / 486 bg cells; index 203 appears nowhere) | renders the candidate builtin tokens | `docs/audits/2026-10-21-junie-palette-ladder.md` conflict #2 |

Deciding commits, in order (verified in history):

1. `4b857a079` (2026-09-02) — Junie lab introduces `RED = rgb(0xe44545)`.
2. `032e9faa3` (2026-10-15) — crate split lands `junie.rs` with
   `RED = 0xe44545`; parity never adopts the retune below.
3. `0610904df` (2026-10-16, tag ancestry) — deliberate, message-recorded
   contrast retune `#e44545` → `#f05d5d` ("error text … clear[s] 4.5:1 on
   canvas, surface, card and field"), plus `RED_STRONG #ff8a8a` and
   `RED_SOFT #e8a0a0`. **This commit decides the frozen value.**
4. `docs/audits/2026-10-21-junie-palette-ladder.md` — records conflict #2
   (tag 203/210 vs corpus 167) and keeps the corpus index "while it is the
   gate oracle"; superseded as a *decision basis* by D-009.
5. `4d437794fa` + `ec783f6c50` + D-009 (`docs/architecture/
   termrock-migration-record.md:402`) — the oracle is the frozen source at
   `4169783ae` and the frames it produced; `baselines/tuiscotti-v1` is a
   legacy replayed conversion, not the oracle.
6. `c4a7277da` (2026-10-21) — jackin de-duplicates its private
   `Color::Rgb(0xf0, 0x5d, 0x5d)` literal onto `Role::Danger`
   (`crates/jackin-preview-app/src/app.rs:10324`, `Tone::Error` arm) and
   adds the mono `ink()` fallback; re-blesses
   `returning 120 40 junie mono` `78a2b61980bc2587` → `a8e727d2cc920213`.
7. `e830877cd` — restores the held digests, leaving
   `jackin_visual_baseline` red pending adjudication.
8. `docs/audits/2026-10-22-jackin-digest-drift-adjudication.md` — verdict:
   reference defect between two preserved references; routes the truecolor
   token, the 256 index and the held digests into one reference decision.
9. `740374edb2` (2026-10-22, current base tip) — restores candidate danger
   to `0xf05d5d` under D-009/D-010, leaving three recorded reds and
   deferring the corpus-derived stores to this decision.
10. `docs/audits/2026-10-22-component-digest-reference-decision.md`
    (D-010, `termrock-migration-record.md:420`) — the broader component
    digest escalation; the danger token is one family inside it.

## 1. Decision statement requested from the owner

Approve the tag-source values as the danger-token oracle — truecolor
danger `#f05d5d` (already restored to the builtin by `740374edb2`),
`error_strong`/derived 256 index `210`, and the tag's `danger_soft`
decision under sub-decision 4.1 below — and authorize regeneration of the
stores that pin the pre-tag values: the 256-tier authored pins in
`crates/termrock-theme/src/palettes.rs` (167 → the decided index), the
held `jackin_visual_baseline` digest in
`crates/jackin-preview-app/tests/baselines/jackin.txt` (BLESS of the
candidate digests `c4a7277da` produced), the danger-affected digests in
`crates/termrock-conformance/tests/baselines/components.txt` (scoped to
the danger family; the rest of D-010 stays open), the
`standalone_empty_cells` fingerprint, and a corpus re-approval of
`baselines/tuiscotti-v1` executed only through the cohort/custody
machinery — with the old bytes preserved on both sides, every re-pin
classified and independently reviewed before it lands, and no in-place
expected-output edit — or reject this and instead pin the corpus values
(`#e44545` / `Indexed(167)` / `#d98a8a`) as the product contract,
reverting `740374edb2` and recording the frozen-source divergence as a
permanent, owned exception to D-009.

## 2. Complete inventory

### 2.1 Stores carrying the old truecolor ink `#e44545`

**`baselines/tuiscotti-v1/` (corpus store, 7 550 captures = 302 screens
x 25 per `docs/architecture/termrock-migration-record.md:578`).** Total
files carrying the old ink in any encoding: **1 170** (715 files with hex
`e44545`, 1 641 hits; 468 files with SGR decimal `228;69;69`; 13 files
carry both). By extension (hex set): 520 `.html`, 65 `.ansi`, 65
`.ascii`, 65 `.txt`. By application (hex set, files/hits):

| Application | Files | Hits |
|---|---|---|
| showcase | 409 | 801 |
| holla | 183 | 524 |
| jackin | 81 | 249 |
| tablepro | 42 | 67 |

Role: read-only gate oracle today (`DEFAULT_BASELINE_STORE`), but **not**
the oracle per D-009. Candidate-derivable: yes, but only through the
cohort/custody re-approval path. Regeneration cost: a corpus re-capture
from ordinal zero via `tools/frozen_baseline.py` (`fresh-plan` →
`fresh-run-init` → `fresh-run-advance`, `corpus-check`), never `BLESS`
and never an in-place edit; `corpus-index.json` `manifest_hashes` are
recomputed as part of it (current oracle commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`). The ladder audit sizes the
rendering delta at ~20 000 gated danger fg cells + 486 bg cells. This is
the expensive item: a full 302 x 25 capture pass, partitioned.

**`crates/*/tests/baselines/` (and all other crate source trees): zero
literal `#e44545` pins** — the only two `e44545` occurrences outside the
corpus are comments (`crates/tablepro-ui/src/app.rs:275`,
`crates/termrock-theme/src/builtin/junie.rs:34`). The old ink reaches
the digest stores through rendering, not literals:

- `crates/jackin-preview-app/tests/baselines/jackin.txt:62` — held digest
  `returning 120 40 junie mono 78a2b61980bc2587`; candidate
  `a8e727d2cc920213` already reproduces in-process. Re-pin via `BLESS=1`
  after §20.10 classification (mechanism:
  `crates/termrock-test-support/src/digest.rs:277`). Cost: one line plus
  the paper 120x40 keys `c4a7277da` blessed and `e830877cd` reverted.
- `crates/termrock-conformance/tests/baselines/components.txt` — 231
  failing digests (89 passed / 231 failed per D-010). Danger is one
  family inside a multi-drift (text/syntax already restored by
  `493cc57d6`/`ec783f6c50`); re-pin via `BLESS=1` scoped per D-010 item 4
  after its independent decision. Not a danger-only cost.

**Frozen store `snapshots/` (tag side, read-only oracle).** 215 snapshot
files carry the frozen ink `38;2;240;93;93` (`#f05d5d`), 581 cell hits —
e.g. `snapshots/jackin/audit/capsule/120x40/truecolor.ansi`, 3 fg cells —
and 137 snapshot files still carry the old ink `38;2;228;69;69`, all in
showcase areas (flows 86, audit 18, fade 17, resize 8, hover 8). These
are the oracle under D-009: never edited under either option; the 137
old-ink files must be classified during the regeneration planning (they
record the pre-restore candidate, so tag-wins makes them stale evidence
inside a frozen store — resolved through the cohort/custody capture, not
by editing cells). Under tag-wins the danger side needs no regeneration;
the `jackin::audit::capsule::120x40::truecolor` frozen-phase red (19
danger-ink-class cells, `#f05d5d` expected vs `#e44545` actual) resolves
from the candidate side once the candidate renders `#f05d5d`.

### 2.2 Pins of the 256 tier `Indexed(167)` tied to danger

Authored candidate table `crates/termrock-theme/src/palettes.rs`
(`junie_256`), all four slots derive from the `RED` source:

- `:98` `danger: Color::Indexed(167)`
- `:124` `syntax.invalid: Color::Indexed(167)`
- `:136` `meter.high: Color::Indexed(167)`
- `:144` `meter.series[2]: Color::Indexed(167)`

Adjacent context pins already restored to frozen values in the same
table: `highlight_danger_bg = i(88)`, `success = i(71)`,
`syntax.diagnostic_error = i(210)`. The assertion side is
`crates/termrock-theme/tests/capability_palettes.rs`
(`all_73_junie_slots_match_reference_capability_output`): it derives the
256 tier from the truecolor tokens with the pinned `reference256`
formula, and pins only the frozen authored overrides (line 80:
`diagnostic_error = i(210)`, line 89: `highlight_danger_bg = i(88)`).
With `740374edb2`'s `#f05d5d` the formula yields `Indexed(210)`
(`cube = 16 + 36*5 + 6*2 + 2`), so the test is red against the authored
167. Historical evidence copies of the 167 state exist under
`docs/audits/evidence/` (e.g. `2026-10-21-theme-integrity/
parent-nextest-known-red.log`) and
`docs/audits/2026-10-21-junie-token-contract-map.json`; they are records,
not gates.

Role: candidate-derivable. Regeneration cost: a small authored-table edit
(plus, if the owner picks an authored index instead of the derived one, a
matching override line in `capability_palettes.rs`), then the 73-slot
test goes green without touching any expected output. This is product
source, so it lands only after the owner decision and review.

### 2.3 The `standalone_empty_cells` fingerprint pin

`crates/termrock-theme/tests/empty_style_inheritance.rs:248`
(`standalone_empty_cells_match_pre_inheritance_reference`) folds every
cell of 32 junie renders into a FNV-style fingerprint pinned at line 305
(`312_677_554_172_290_677`). Pre-existing red — the ladder audit's
verification section records it failing on its base commit too — so it is
not caused by `740374edb2`, but any palette decision changes the expected
fingerprint and it must re-pin in the same motion. Regeneration: run the
test, take the actual from the assert failure, pin it with owner
authorization. Cost: one integer.

### 2.4 Stores that do not move

- Tag `visual-baseline` and its store: frozen, untouchable (repo rule).
- `baselines/tuiscotti-v1/corpus-index.json` application counts: update
  only as part of the custody re-capture.
- `docs/audits/evidence/**`: historical record, preserved.

## 3. The two options

### Option A — tag-wins (approve the statement in §1)

- Goes green: the 73-slot capability check (authored 256 aligned with the
  derived/frozen index); `jackin_visual_baseline`
  (`returning 120 40 junie mono` re-pins to `a8e727d2cc920213`);
  `jackin::audit::capsule::120x40::truecolor` frozen phase (candidate
  paints `#f05d5d` against the frozen store); the danger-ink cells inside
  `components.txt` and the corpus gate — after their respective re-pins.
- Stays red until its own decision: the non-danger component-digest
  families in D-010 (231-failure set includes muted/syntax drift already
  fixed in source), the ghost-slot corpus conflict (ladder audit
  conflict #1), and the live-page blockers in the UI/UX review.
- Files that change: `palettes.rs` (4 lines),
  `capability_palettes.rs` (only if an authored index is chosen),
  `jackin.txt` (1–2 lines), `components.txt` (danger-scoped lines),
  `empty_style_inheritance.rs` (1 integer), the corpus store (custody
  re-capture, 1 170 files change bytes), and — under sub-decision 4.1 —
  `junie.rs`/`builtin/mod.rs` (`danger_soft`).
- Consequences: WCAG AA holds for danger on card (`0610904df`: 4.5:1 on
  canvas/surface/card/field vs 4.4:1 before); D-009 stays consistent —
  the frozen source is the oracle for every token family, with no
  exceptions; the ladder audit's conflict #2 closes by corpus re-approval
  exactly as that audit reserved.

### Option B — corpus-wins (reject; pin `#e44545`/167/`#d98a8a`)

- `740374edb2` reverts (`junie.rs` back to `0xe4_45_45`, the
  `builtin/mod.rs:1220` test back to `0xe44545`). The 73-slot check goes
  green immediately (derived 167 = authored 167).
- `jackin_visual_baseline` stays red: that digest diverged at
  `c4a7277da` (mono `ink()` fallback + `Tone::Error` arm), not at the
  restore; corpus-wins does not fix it. The capsule frozen-phase red
  stays red too, and now permanently: the frozen store paints `#f05d5d`
  and the candidate would paint `#e44545` against it by decision.
- WCAG consequence: the product keeps a danger ink the tag record calls
  4.4:1 on a card — sub-AA — and the showcase overview page
  (`crates/showcase-demos/src/pages/overview.rs:379-383`) already paints
  the tag values (`error f05d5d`, `error.soft e8a0a0`,
  `error.strong ff8a8a`) as doctrine copy, so the live page and the theme
  would disagree by decision.
- D-009 consequence: D-009 declares the frozen source the oracle and
  `493cc57d6`/`ec783f6c50` already restored text and syntax to it; danger
  would become the single token family where the converted corpus
  overrides the oracle — a permanent recorded exception, plus the frozen
  snapshot store (215 files of `#f05d5d` cells) contradicts the product
  at every danger cell with no closure path that does not edit frozen
  artifacts. The ladder audit's conflict #2 stays open forever instead of
  closing by re-approval.

## 4. Open sub-decisions (rule on each)

1. **danger_soft restore.** Candidate `0xd98a8a`
   (`junie.rs:39`, test-pinned `builtin/mod.rs:1221`, authored 256
   `Indexed(181)` at `palettes.rs:99`) vs tag `0xe8a0a0`
   (`4169783ae` `src/theme.rs:531`, authored 256 `error_soft = i(174)`,
   16-tier `LightRed`); the showcase overview page already shows
   `0xe8a0a0` (`overview.rs:381`) and the 256 swatch shows `i(174)`
   (`overview.rs:315`). Independent of the main token: tag-wins does not
   force it, but the showcase page evidence points at the tag value.
2. **The 256-tier danger index.** `167` (corpus pin, formula applied to
   the pre-tag ink) vs `210` (the pinned reference formula applied to the
   restored `0xf05d5d`, equal to the tag's authored `error_strong`) vs
   `203` (the tag's authored `apply_256` `error` slot). Derived-210 makes
   the 73-slot test green with no authored override; authored-203 matches
   the frozen table literally but needs an override line (and diverges
   from the reference formula the 73-slot test enforces elsewhere).
3. **Jackin literal de-duplication onto `Role::Danger`.** Already done by
   `c4a7277da` (`app.rs:10324`); no `f05d5d`/`e44545` literals remain in
   the jackin crates (verified). The residual ruling is only whether
   `c4a7277da` stands and its re-blessed `a8e727d2cc920213` becomes the
   `jackin.txt` pin when the held digest re-pins.
4. **`standalone_empty_cells` fingerprint regeneration.** Re-pin
   `312_677_554_172_290_677` (`empty_style_inheritance.rs:305`) to the
   post-decision value; pre-existing red, re-pins once after the palette
   settles, not per-token.

## Verification appendix (every number above was produced by one of these)

- `git grep -c 'e44545' origin/parity/termrock-visual-baseline -- baselines/ crates/*/tests/baselines/` → 715 files / 1641 hits, all under `baselines/tuiscotti-v1/`; per-app and per-extension rollups via awk over that output.
- `git grep -l '228;69;69' … -- baselines/` → 468 files; set arithmetic vs the hex list → union 1170, intersection 13.
- `git grep -n 'Indexed(167)' -- crates/termrock-theme/src/palettes.rs` → lines 98, 124, 136, 144.
- `git grep -n 'Indexed(210)' …` → `crates/showcase-demos/src/pages/overview.rs`, `crates/termrock-theme/src/palettes.rs`, `crates/termrock-theme/tests/capability_palettes.rs` (one each).
- `git show 4169783ae:src/theme.rs` → `:521` RED, `:523` RED_STRONG, `:531` RED_SOFT, `:747` error_soft, `:943-944` `error = i(203)` / `error_strong = i(210)`, `:946` `error_soft = i(174)`.
- `git show origin/parity/termrock-visual-baseline:crates/termrock-theme/src/builtin/junie.rs` → line 36 `RED = 0xf0_5d_5d`, line 39 `RED_SOFT = 0xd9_8a_8a`.
- `git show 740374edb2` → the restore diff and its recorded reds.
- `grep -c '38;2;240;93;93' snapshots/jackin/audit/capsule/120x40/truecolor.ansi` → 3; `grep -rc '240;93;93' snapshots/` → 215 files, 581 hits; `grep -rl '228;69;69' snapshots/` → 137 files (showcase flows 86 / audit 18 / fade 17 / resize 8 / hover 8).
- `git show origin/parity/termrock-visual-baseline:crates/jackin-preview-app/tests/baselines/jackin.txt` line 62; `git show origin/parity/termrock-visual-baseline:crates/termrock-theme/tests/empty_style_inheritance.rs` lines 248/305.
- `reference256(0xf0,0x5d,0x5d)` evaluated by hand from `capability_palettes.rs:6-31` → `Indexed(210)`; `reference256(0xe4,0x45,0x45)` → `Indexed(167)`.
- `git grep -rn 'f05d5d|e44545' -- crates/jackin-preview-app crates/jackin-ui` → no hits (literals gone).
