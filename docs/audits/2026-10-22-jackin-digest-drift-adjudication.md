# Jackin digest drift adjudication: danger ink `#f05d5d` (frozen) vs `#e44545` (candidate)

Date: 2026-10-22. Branch: `parity/termrock-visual-baseline`, base tip `bdf62e3be`.
Scope: the two recorded-but-open Jackin findings — the
`jackin_visual_baseline` digest red on `returning 120 40 junie mono`
(baseline `78a2b61980bc2587`, candidate `a8e727d2cc920213`) and the frozen
self-drift on `jackin::audit::capsule::120x40::truecolor` (19 danger-ink
cells, `#f05d5d` expected vs `#e44545` actual).

Verdict: **reference defect between two preserved references, both sides
grounded in recorded decisions. Not an accidental regression. No product
source is fixed and no expected output is edited.** Resolution path is
adjudication (this document), a frozen-reference regeneration proposal
through the cohort/custody machinery, and independent review.

## Method

Read-only. `git log -S` / `git grep` archaeology across `visual-baseline`
(`4169783ae883788186d161af09ba9c481bd84e5c`) and parity ancestry; two
`git bisect run` passes over `e4cd9d1b..e830877cd` with a temporary
uncommitted probe that rendered exactly `returning 120 40 junie mono` in
process and compared its digest against the two known values; reruns of
`cargo nextest run --locked -p jackin-preview-app -E
'test(jackin_visual_baseline)'` at the tip and at candidate commits. No
`BLESS`, no snapshot or baseline edits, no frozen store writes.

## The token conflict

| Reference | TrueColor danger/error ink | ANSI-256 | Source |
|---|---|---|---|
| Tag `visual-baseline` (`4169783ae`) store, incl. `snapshots/jackin/audit/capsule/120x40/truecolor.ansi` | `#f05d5d` | 203 (`error = i(203)`) | `src/theme.rs:521` `RED = rgb(0xf05d5d)` |
| Candidate `crates/termrock-theme` builtin + corpus `baselines/tuiscotti-v1` | `#e44545` | 167 | `crates/termrock-theme/src/builtin/junie.rs:31` `RED = 0xe4_45_45` |

The candidate (and the corpus, which is a refactor-line replay whose
truecolor tokens are the current builtin) carries the **pre-tag** value
`#e44545`. The frozen store carries the tag's retuned value `#f05d5d`.

Deciding commits, in order:

1. `4b857a079` (2026-09-02, common ancestry): the Junie design-system lab
   introduces `RED = rgb(0xe44545)`.
2. `032e9faa3` (2026-10-15 22:33, parity): the termrock crate split lands
   `junie.rs` with `RED = 0xe44545`. The split snapshot predates the retune
   below by roughly a day; the parity line never adopted it.
3. `0610904df` (2026-10-16 20:04, tag ancestry, verified ancestor of
   `visual-baseline`): deliberate, message-recorded contrast retune —
   `RED` `#e44545` → `#f05d5d` ("the old #e44545 was 4.4:1 on a card",
   i.e. sub-WCAG-AA) plus `RED_STRONG #ff8a8a`. **This commit decides the
   frozen value.**
4. `2026-10-21-junie-palette-ladder.md` (parity): the conflict is recorded
   as reference defect #2 — tag renders 203 from `0xf05d5d`, corpus renders
   167 from `0xe44545`; "the corpus index (167) is kept; switching to 203
   would turn ~20k gated cells red", both references preserved, and the tag
   ladder is recorded for a future corpus re-approval. Candidate truecolor
   danger stays `#e44545` by this recorded decision.
5. `c4a7277da` (2026-10-21 20:32, "fix(jackin): ensure status text contrast
   in mono mode"): Jackin de-duplicates its private
   `Color::Rgb(0xf0, 0x5d, 0x5d)` literal — a copy of the tag-era ink —
   onto the shared `Role::Danger` token, and adds a Mono `ink()` fallback
   that flips `fg == bg` pairs to White/Black (mono status text was
   invisible: a genuine WCAG contrast defect). The baseline is re-blessed
   in the same commit: `returning 120 40 junie mono`
   `78a2b61980bc2587` → `a8e727d2cc920213` (paper 120x40 keys likewise).
6. `e830877cd` (2026-10-21 20:38, "test: restore held reference
   expectations after incoming re-blessing"): restores the held frozen
   digests, deliberately leaving `jackin_visual_baseline` red pending this
   adjudication. The red recorded in `docs/handoff/README.md` (2026-10-22
   section) and reproduced at bare origin tip `384b818b2` follows from
   this; `c4a7277da` is in bare-tip ancestry, so "zero jackin commits"
   reproduction is consistent.

Bisect evidence: probe digest equals the frozen `78a2b61980bc2587` at
`795badb0`; equals the candidate `a8e727d2cc920213` at `c4a7277da`; the
commit's own diff rewrites exactly that baseline line and contains both
mechanisms (the Mono `ink()` fallback changes the mono cells of the paused
returning manager; `capsule_span_role`'s `Tone::Error` arm changes the
error-tone ink at every color level). An earlier convergence attempt
landed on `e4cd9d1b` only because an unrelated key
(`accounts-mixed 120 40 junie truecolor`) reddened first; the probe-based
pass is the deciding one.

## Finding 2 mechanics

`jackin::audit::capsule::120x40::truecolor` compares a capsule-audit render
against the tag-derived store, whose snapshot paints `#f05d5d`
(3 fg cells in the committed tag file; the recorded run reports 19
danger-ink-class cells differing). The candidate paints error-tone spans
through `Role::Danger` = `#e44545` — the same token conflict as finding 1,
surfacing through the paired frozen-runner gate instead of the in-process
digest gate. The candidate CLI exposes only the eight user scenarios, so
this case stays candidate-unverifiable until the paired scenario support is
regenerated; the frozen phase's own red is the store-vs-render expression
of the same defect. It is the same class as the recorded concept-family
frozen drift.

## Adjudication

- Candidate source is **not** accidentally wrong: `#e44545` is the parity
  builtin token, kept by the recorded corpus-wins decision (ladder audit,
  conflict #2), and `c4a7277da` removes a duplicated private color literal
  and a real mono contrast defect — both required by the component
  architecture rules.
- The frozen store is **not** silently stale either: `#f05d5d` is the
  deliberate, message-recorded WCAG retune `0610904df`, and the tag is the
  frozen visual oracle.
- Therefore both results are preserved and the conflict moves to the
  frozen-artifact conflict rule: neither side of the store may be edited
  to make a gate green.

## Recommended resolution

1. Adjudication: this document records the defect, both digests, and the
   deciding commits.
2. Regeneration proposal: one proposal covering the affected frozen
   references — the `jackin_visual_baseline` held digests (at minimum
   `returning 120 40 junie mono` `78a2b61980bc2587` →
   `a8e727d2cc920213` and the paper 120x40 keys re-blessed by
   `c4a7277da`/reverted by `e830877cd`), the tag-derived frozen store
   snapshots carrying `#f05d5d` danger ink (including
   `snapshots/jackin/audit/capsule/…`), and the corpus ladder question
   (167 vs 203) already reserved for corpus re-approval in the ladder
   audit. The proposal must go through the cohort/custody machinery
   (`tools/frozen_baseline.py` plan, capture from ordinal zero, custody
   key), never an in-place expected-output edit.
3. Independent review of the proposal before any digest advances.
4. Until then: the digest red and the capsule frozen-drift case stay red
   and are classified pre-existing; no `BLESS`, no snapshot edits, no
   palette switch to `#f05d5d` (it would turn the corpus's ~20k gated
   cells red and reopen the ladder decision recorded on 2026-10-21).

## Verification

- Read-only gates at the base tip: `cargo nextest run --locked -p
  jackin-preview-app -p jackin-ui` shows only the recorded pre-existing
  reds (this digest line among them); `cargo fmt --check` clean. No test
  was blessed and no expected file changed by this adjudication.
- Temporary bisect probe removed; worktree clean.
