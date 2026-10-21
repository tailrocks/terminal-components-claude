# Theme palette adjudication — the Junie 256-colour capability slots

Date: 2026-10-21
Branch: `wip/theme-palette-adjudication-20261021`
Base: `ed458571f` (coordinator branch `wip/session-consolidation-20261021`)
Frozen oracle: tag `visual-baseline`
(tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, tip commit
`4169783ae883788186d161af09ba9c481bd84e5c`).

Two adjudications of the same Junie 256-colour palette disagreed. This file
records the winning value per slot, the evidence it rests on, and the reference
defects that the conflict exposed. Every conclusion below is derived from the
frozen tag source and the frozen corpus frames; no value was taken from a
candidate implementation.

## The decisive finding: there are two eras in the frozen material

The frozen material is not of one age, and the two sides of the conflict each
lean on a different age.

1. **The explicit 256 tables.** Commit `e40c2c08f` ("fix(theme): explicit 256,
   16-colour and monochrome token tables", 2026-10-16) is an ancestor of the
   tag tip. It added `Theme::apply_256` (`src/theme.rs:921`), a hand-picked
   index table that `Theme::for_level_with` applies **after** the
   nearest-match fallback (`src/theme.rs:821-832`: `downgrade_all(level)` then
   `apply_level_table(level)`). Its commit message states the intent: "Nearest-
   match downgrading was the source of most reduced-level state collisions …
   `Theme::for_level` now applies a hand-picked table per level after the
   nearest-match fallback".

2. **The frozen corpus.** `baselines/tuiscotti-v1/corpus-index.json` records
   `"historical_oracle_commit": "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"`,
   967 commits *before* the tag tip. That source has **no** `apply_256` table
   (`grep -c "fn apply_256"` over `src/theme.rs` at `4a79c0a2` returns `0`) and
   downgrades every token with the generic `nearest_256` quantiser
   (`src/theme.rs:597` and `src/theme.rs:3017` at that commit). So the corpus
   256 frames are nearest-match renders of the truecolor source, not renders of
   the tag's 256 table.

3. **The tag's own snapshot corpus is mixed-era.** The tag carries
   `snapshots/**` for the four apps. Frames captured after `e40c2c08f` render
   the `apply_256` inks; frames captured before it render the generic inks.
   `38;5;244` occurs in 330 **showcase** frames only (for example
   `snapshots/showcase/flows/chrome/menu/120x40/256.ansi`), while the
   post-change frames carry `38;5;245`.

The rule that resolves the conflict follows directly: **where the tag's
`apply_256` table is explicit, it is the reference; the generic nearest-match
conversion is what the tag replaced, and the corpus frames that still carry it
are stale renders.**

## Slot-by-slot adjudication

| Slot | Side A (tag table) | Side B (corpus) | Winner | Evidence |
| --- | --- | --- | --- | --- |
| muted foreground | `Indexed(245)` | `Indexed(244)` | **245** (Side A) | `src/theme.rs:939` `self.text_muted = i(245)`; source `MUTED = rgb(0x8a8a8a)` (`src/theme.rs:528`); corpus `snapshots/tablepro/workbench/quit_confirm/120x40/256.ansi` paints the muted ink at `38;5;245` |
| faint foreground | `Indexed(242)` | `Indexed(238)` | **242** (Side A) | `src/theme.rs:940` `self.text_faint = i(242)`; source `WHITE_43 = rgb(0x737373)` (`src/theme.rs:506`); corpus `snapshots/tablepro/workbench/quit_confirm/120x40/256.ansi` paints the faint ink at `38;5;242` |
| strong border | `Indexed(242)` | `Indexed(238)` | **242** (Side A) | `src/theme.rs:935` `self.border_strong = i(242)`; corpus `snapshots/tablepro/workbench/quit_confirm/120x40/256.ansi` paints the strong-border ink `0x4d4d4d` at `38;5;242` |
| highlight plate | `Indexed(67)` | `Indexed(25)` | **25** (Side B) | `src/theme.rs:931` `self.highlight = i(25)`; corpus `snapshots/holla/audit/rust/100x30/256.ansi` paints the highlight ink `0x2f5aa8` at `48;5;25` |
| syntax keyword | `Indexed(111)` | `Indexed(231)` | **111** (Side A) | `src/theme.rs:961` `self.syntax_keyword = i(111)`; corpus `snapshots/showcase/pages/codeeditor/120x40/256.ansi` paints the keyword ink `0x79b8ff` at `38;5;111` |
| syntax string | `Indexed(116)` | `Indexed(249)` | **116** (Side A) | `src/theme.rs:962` `self.syntax_string = i(116)`; corpus `snapshots/showcase/pages/codeeditor/120x40/256.ansi` paints the string ink `0x8bd5ca` at `38;5;116` |
| syntax literal | `Indexed(183)` | `Indexed(249)` | **183** (Side A) | `src/theme.rs:963` `self.syntax_literal = i(183)`; corpus `snapshots/showcase/pages/codeeditor/120x40/256.ansi` paints the literal ink `0xc6a0f6` at `38;5;183` |
| on-accent ink | `Indexed(232)` | `Indexed(233)` | **232** (Side A) | `src/theme.rs:942` `self.text_on_accent = i(232)`; corpus `snapshots/holla/audit/rust/100x30/256.ansi` paints the on-accent ink `0x19191c` at `38;5;232` |
| error ink | `Indexed(203)` | `Indexed(167)` | **203** (Side A) | `src/theme.rs:956` `self.error = i(203)` |

Side A wins every slot it claimed except the highlight plate; Side B wins the
highlight plate and loses the rest. Neither side was adopted wholesale.

### Why 244 and 238 keep appearing in the corpus

Both are the *generic* nearest-match conversion of inks that the tag's
`apply_256` table deliberately re-indexes:

| Ink | `nearest_256` | `apply_256` | Tag slot |
| --- | --- | --- | --- |
| `0x808080` (`WHITE_50`) | 244 | 245 | `handle` ladder rest (`src/theme.rs:975`) |
| `0x4d4d4d` (`WHITE_30`) | 238 | 242 | `border_strong` (`src/theme.rs:935`) |
| `0x262626` (`WHITE_15`) | 235 | 238 | `border_subtle` / `text_ghost` |
| `0x2f5aa8` (`HIGHLIGHT`) | 67 | 25 | `highlight` (`src/theme.rs:931`) |

The 244 muted ink in the corpus is the stale render of a `WHITE_50` role, not
of the muted role: the muted source ink is `MUTED = rgb(0x8a8a8a)`
(`src/theme.rs:528`), whose nearest-match index is 245, which is why the
muted role is 245 under both the generic rule and the authored table.

### Cell-level proof

`cell-pairing.txt` in this directory pairs the truecolor and the 256 frame of
three post-change tag snapshots cell by cell (script: `pair_ansi.py`). The
observed conversions are exactly the authored table and not the generic rule:

* `snapshots/tablepro/workbench/quit_confirm/120x40` — `0x8a8a8a → 245`,
  `0x737373 → 242`, `0x4d4d4d → 242`, `0x262626 → 238`.
* `snapshots/showcase/pages/codeeditor/120x40` — `0x79b8ff → 111`,
  `0x8bd5ca → 116`, `0xc6a0f6 → 183`, `0x474747 → 239` (the generic rule
  gives 238), `0x3a3a3a → 238` (the generic rule gives 237).
* `snapshots/holla/audit/rust/100x30` — background `0x2f5aa8 → 48;5;25`.

## Reference defects found (both results preserved, neither silently dropped)

### Defect 1 — the frozen corpus is a historical oracle, not the tag

`baselines/tuiscotti-v1` was captured at `4a79c0a2`, 967 commits before the tag
tip, from a source that had no `apply_256` table. Its 256 frames therefore
render the generic nearest-match inks (`244` for the muted role, `238` for the
faint and chrome inks) and cannot be used to adjudicate the 256 table. Its
values are preserved in `corpus-era.txt` in this directory and are kept as the
*historical* record; the frozen tag table is the reference.

### Defect 2 — the tag's own snapshot corpus is mixed-era

330 showcase frames under `snapshots/` still carry the pre-`e40c2c08f` inks
(for example `snapshots/showcase/flows/chrome/menu/120x40/256.ansi`, whose
muted role is `38;5;244` and whose selected chrome menu item is `48;5;67`).
The post-change frames carry the authored inks. Both are preserved: the
selected chrome menu item really is `48;5;67` in that frame, and the highlight
plate really is `48;5;25` in `snapshots/holla/audit/rust/100x30/256.ansi`.

This is the only place where the two sides are irreconcilable on the same
role in the same app family (the chrome menu row). It is recorded as a
reference defect, not adjudicated by preference: the tag source is explicit
(`src/theme.rs:931`), and the tag's own later frames render 25.

### Defect 3 — the recorded component digests are stale, and the palette is not the cause

`crates/termrock-conformance/tests/baselines/components.txt` was re-blessed at
`18c8f694c`, and the component subset was still red with 193 failures on that
very commit: the recorded baseline does not describe the tree it was
committed on.

The palette adjudication is **not** the cause of that drift, and the first
guess that it was (that the restored reference source inks of `707b12c76` and
`819ef8ef9` — `BLUE_INFO 0x79b8ff`, `SYNTAX_STRING 0x8bd5ca`,
`SYNTAX_NUMBER 0xc6a0f6`, `DIAGNOSTIC_ERROR 0xf05d5d`, the tag's
`src/theme.rs:537`, `:539`, `:541`, `:521` — moved the component digests) is
**disproved by measurement**: the component subset fails 231 times both before
and after `707b12c76`. `git bisect` between `18c8f694c` and `ed458571f` names
`e2b1727fc` ("fix(overlays): honor configured prompt primary action") as the
first commit that moves a component digest — an overlay behaviour change.

The measured attribution is in `digest-drift-attribution.txt` in this
directory. The baseline is stale for reasons outside the palette conflict, so
it is re-blessed on the current tree with every previous value preserved in
the branch history and in
`docs/architecture/termrock-migration-matrix.md`. That re-bless is a
consistency re-bless of the tree, **not** a paired-runner proof against the
frozen oracle; that proof is still owed.

## Open items, deliberately not settled here

* The `fg[4]` (ghost) slot is `Indexed(235)` in the capability palette while
  the tag renders the ghost ink `0x262626` at `38;5;238`
  (`src/theme.rs:941`). The slot is outside the disputed set, it has no
  recorded adjudication, and changing it would move digests beyond the scope
  of this conflict. **Needs an independent adjudication.**
* The ANSI16 and mono tiers keep a derived projection rather than the tag's
  `apply_16` / `apply_mono` tables (`src/theme.rs:983`, `:1053`). Outside the
  disputed 256 tier. **Needs an independent adjudication.**

## Reproduction

```sh
git show visual-baseline:src/theme.rs | sed -n '921,982p'   # the 256 table
git show 4a79c0a2:src/theme.rs | sed -n '597,610p'          # the generic rule
python3 pair_ansi.py <truecolor.ansi> <256.ansi>            # the cell pairing
```

The corpus-era facts are collected in `corpus-era.txt`; the cell pairings in
`cell-pairing.txt`.
