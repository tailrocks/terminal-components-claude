# Owner decision package: the danger-ink token reference (`#f05d5d` frozen vs `#e44545` corpus)

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline` (base tip
`1c17d8f262`; drafted against `740374edb2`, and the tip moved to
`1c17d8f262` while this package was written — see deciding commit 11).
Status: **OPEN — one owner decision plus four sub-decisions requested.**
This document consolidates the scattered danger-ink record into a single
decision package; it states no new measurements without a command listed
in the verification appendix.

## Background (all cited, nothing new)

| Reference | TrueColor danger | 256 tier | danger_soft | Source |
|---|---|---|---|---|
| Tag `visual-baseline` (`4169783ae`) frozen source | `#f05d5d` | authored `error = i(203)`, `error_strong = i(210)` (`src/theme.rs:943`, `:944`); `i(203)` renders 1145 fg cells | `#e8a0a0` (`RED_SOFT`, `src/theme.rs:531`, mapped to `error_soft` at `:747`; authored 256 `error_soft = i(174)`) | tag `src/theme.rs` |
| Candidate builtin (tip `1c17d8f262`, after restore `740374edb2`) | `#f05d5d` (`crates/termrock-theme/src/builtin/junie.rs:38`) | authored `Indexed(203)` (`palettes.rs:99`), deliberately off the reference formula, which yields `Indexed(210)` on `#f05d5d` | `#d98a8a` (`RED_SOFT`, `junie.rs`; test pin `crates/termrock-theme/src/builtin/mod.rs`) | parity builtin |
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
11. `1c17d8f262` (2026-10-22, current tip, "match baseline 256-color
    indexes") — re-authors the 256 danger family from `Indexed(167)` to
    `Indexed(203)` (danger, `syntax.invalid`, `meter.high`,
    `meter.series[2]`, `syntax.diagnostic_error`; `palettes.rs:99`, 127,
    133, 140, 148) and pins the matching expected overrides in
    `capability_palettes.rs:85-91`, with the in-code comment "apply_256
    error is 203 (#ff5f5f). Nearest of #f05d5d is 210." This answers the
    corpus-pin half of sub-decision 2 in the tree; what still needs the
    owner is ratifying the deliberate authored-203-over-derived-210
    divergence from the reference formula. It does not touch the
    truecolor token, the corpus, or the held digests.

## 1. Decision statement requested from the owner

Approve the tag-source values as the danger-token oracle — truecolor
danger `#f05d5d` (already restored to the builtin by `740374edb2`), the
256-tier danger family as authored by `1c17d8f262` (`Indexed(203)`, the
frozen `apply_256` `error` slot, kept deliberately over the derived
`210`), and the tag's `danger_soft` decision under sub-decision 4.1
below — and authorize regeneration of the stores that pin the pre-tag
values: the held `jackin_visual_baseline` digest in
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
reverting `740374edb2` and `1c17d8f262`'s danger family and recording
the frozen-source divergence as a permanent, owned exception to D-009.

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

### 2.2 Pins of the 256 tier tied to danger

Authored candidate table `crates/termrock-theme/src/palettes.rs`
(`junie_256`). At `740374edb2` all five slots were `Indexed(167)`
(lines 98, 124, 133, 136, 144); the current tip `1c17d8f262` re-authors
them to `Indexed(203)`:

- `:99` `danger: Color::Indexed(203)`
- `:127` `syntax.invalid: Color::Indexed(203)`
- `:133` `syntax.diagnostic_error: Color::Indexed(203)` (was `210`)
- `:140` `meter.high: Color::Indexed(203)`
- `:148` `meter.series[2]: Color::Indexed(203)`

Adjacent context pins already restored to frozen values in the same
table: `highlight_danger_bg = i(88)`, `success = i(71)`. The assertion
side is `crates/termrock-theme/tests/capability_palettes.rs`
(`all_73_junie_slots_match_reference_capability_output`): it derives the
256 tier from the truecolor tokens with the pinned `reference256`
formula, then pins the authored overrides (lines 85-91 at the tip:
`diagnostic_error`, `danger`, `syntax.invalid`, `meter.high`,
`meter.series[2]` all `Indexed(203)`; line 100:
`highlight_danger_bg = i(88)`). With `740374edb2`'s `#f05d5d` the
formula yields `Indexed(210)` (`cube = 16 + 36*5 + 6*2 + 2`), so the
tip's authored 203 is a deliberate divergence from the reference
formula, recorded in the code comment and in `1c17d8f262`'s message
("apply_256 error is 203 (#ff5f5f). Nearest of #f05d5d is 210."); by
construction the test is green at the tip. Historical copies of the 167
state exist under `docs/audits/evidence/` (e.g.
`2026-10-21-theme-integrity/parent-nextest-known-red.log`) and
`docs/audits/2026-10-21-junie-token-contract-map.json`; they are
records, not gates.

Role: in-tree as of `1c17d8f262`, but it is product source decided by a
coordinator commit — the owner ratifies or reverts it here. Regeneration
cost if ratified: none (no expected output moves). If the owner instead
picks the derived 210, the same five authored lines plus the five
override lines change and nothing else.

### 2.3 The `standalone_empty_cells` fingerprint pin

`crates/termrock-theme/tests/empty_style_inheritance.rs:248`
(`standalone_empty_cells_match_pre_inheritance_reference`) folds every
cell of 32 junie renders into a FNV-style fingerprint pinned at line 289
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

- Goes green: the 73-slot capability check is already green at the tip by
  construction (authored 203 with matching overrides, §2.2); the owner
  ratification keeps it green. `jackin_visual_baseline`
  (`returning 120 40 junie mono` re-pins to `a8e727d2cc920213`);
  `jackin::audit::capsule::120x40::truecolor` frozen phase (candidate
  paints `#f05d5d` against the frozen store); the danger-ink cells inside
  `components.txt` and the corpus gate — after their respective re-pins.
- Stays red until its own decision: the non-danger component-digest
  families in D-010 (231-failure set includes muted/syntax drift already
  fixed in source), the ghost-slot corpus conflict (ladder audit
  conflict #1), and the live-page blockers in the UI/UX review.
- Files that change: `jackin.txt` (1–2 lines),
  `components.txt` (danger-scoped lines),
  `empty_style_inheritance.rs` (1 integer), the corpus store (custody
  re-capture, 1 170 files change bytes), and — under sub-decision 4.1 —
  `junie.rs`/`builtin/mod.rs` (`danger_soft`). `palettes.rs` and
  `capability_palettes.rs` change only if the owner overturns
  `1c17d8f262`'s authored 203 for the derived 210.
- Consequences: WCAG AA holds for danger on card (`0610904df`: 4.5:1 on
  canvas/surface/card/field vs 4.4:1 before); D-009 stays consistent —
  the frozen source is the oracle for every token family, with no
  exceptions; the ladder audit's conflict #2 closes by corpus re-approval
  exactly as that audit reserved.

### Option B — corpus-wins (reject; pin `#e44545`/167/`#d98a8a`)

- `740374edb2` reverts (`junie.rs` back to `0xe4_45_45`, the
  `builtin/mod.rs` test back to `0xe44545`) and `1c17d8f262`'s danger
  family re-authors to `Indexed(167)` (five `palettes.rs` lines plus the
  five `capability_palettes.rs` overrides). The 73-slot check goes green
  immediately (derived 167 = authored 167).
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
   (`junie.rs:43`, test-pinned `builtin/mod.rs:1224`, authored 256
   `Indexed(181)` at `palettes.rs:100`) vs tag `0xe8a0a0`
   (`4169783ae` `src/theme.rs:531`, authored 256 `error_soft = i(174)`,
   16-tier `LightRed`); the showcase overview page already shows
   `0xe8a0a0` (`overview.rs:381`) and the 256 swatch shows `i(174)`
   (`overview.rs:315`). Independent of the main token: tag-wins does not
   force it, but the showcase page evidence points at the tag value.
2. **The 256-tier danger index.** `167` (the old corpus pin, formula
   applied to the pre-tag ink) vs `210` (the pinned reference formula
   applied to the restored `0xf05d5d`, equal to the tag's authored
   `error_strong`) vs `203` (the tag's authored `apply_256` `error`
   slot). The tip already authors 203 with matching expected overrides
   (`1c17d8f262`); the owner ratifies that, or overturns it for the
   derived 210 (five authored lines plus five override lines) — 167 has
   no in-tree defender left. Authored-203 matches the frozen table
   literally but diverges from the reference formula the 73-slot test
   enforces everywhere else; derived-210 keeps the formula but puts
   danger on `error_strong`'s index instead of `error`'s.
3. **Jackin literal de-duplication onto `Role::Danger`.** Already done by
   `c4a7277da` (`app.rs:10324`); no `f05d5d`/`e44545` literals remain in
   the jackin crates (verified). The residual ruling is only whether
   `c4a7277da` stands and its re-blessed `a8e727d2cc920213` becomes the
   `jackin.txt` pin when the held digest re-pins.
4. **`standalone_empty_cells` fingerprint regeneration.** Re-pin
   `312_677_554_172_290_677` (`empty_style_inheritance.rs:289`) to the
   post-decision value; pre-existing red, re-pins once after the palette
   settles, not per-token.

## Verification appendix (every number above was produced by one of these)

- `git grep -c 'e44545' origin/parity/termrock-visual-baseline -- baselines/ crates/*/tests/baselines/` → 715 files / 1641 hits, all under `baselines/tuiscotti-v1/`; per-app and per-extension rollups via awk over that output.
- `git grep -l '228;69;69' … -- baselines/` → 468 files; set arithmetic vs the hex list → union 1170, intersection 13.
- `git show 740374edb2:crates/termrock-theme/src/palettes.rs | grep -n 'Indexed(167)'` → lines 98, 124, 136, 144; at the tip `git grep -n 'Indexed(203)' -- crates/termrock-theme/src/palettes.rs` → lines 99, 127, 133, 140, 148 and no `Indexed(167)` remains.
- `git grep -c 'Indexed(210)' … -- crates` at `740374edb2` → one each in `crates/showcase-demos/src/pages/overview.rs`, `crates/termrock-theme/src/palettes.rs`, `crates/termrock-theme/tests/capability_palettes.rs`; at the tip `1c17d8f262` only `overview.rs` keeps a `210` (the other two moved to 203).
- `git show 4169783ae:src/theme.rs` → `:521` RED, `:523` RED_STRONG, `:531` RED_SOFT, `:747` error_soft, `:943-944` `error = i(203)` / `error_strong = i(210)`, `:946` `error_soft = i(174)`.
- `git show origin/parity/termrock-visual-baseline:crates/termrock-theme/src/builtin/junie.rs` at the tip → line 39 `RED = 0xf0_5d_5d`, line 43 `RED_SOFT = 0xd9_8a_8a`.
- `git show 740374edb2` → the restore diff and its recorded reds.
- `grep -c '38;2;240;93;93' snapshots/jackin/audit/capsule/120x40/truecolor.ansi` → 3; `grep -rc '240;93;93' snapshots/` → 215 files, 581 hits; `grep -rl '228;69;69' snapshots/` → 137 files (showcase flows 86 / audit 18 / fade 17 / resize 8 / hover 8).
- `git show origin/parity/termrock-visual-baseline:crates/jackin-preview-app/tests/baselines/jackin.txt` line 62; `git show origin/parity/termrock-visual-baseline:crates/termrock-theme/tests/empty_style_inheritance.rs` lines 248/289.
- `reference256(0xf0,0x5d,0x5d)` evaluated by hand from `capability_palettes.rs:6-31` → `Indexed(210)`; `reference256(0xe4,0x45,0x45)` → `Indexed(167)`.
- `git grep -rn 'f05d5d|e44545' -- crates/jackin-preview-app crates/jackin-ui` → no hits (literals gone).
