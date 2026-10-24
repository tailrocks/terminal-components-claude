# Oracle adjudication — the theme palette and the component digest baseline

Status: **adjudicated, CLEARED with a bounded scope.**
Date: 2026-10-22
Adjudicator: independent subagent (read-only; no product code changed)
Branch state at adjudication: `parity/termrock-visual-baseline` = `aa1c15a59b`

## 0. Scope and method

I was asked to decide, without touching product code:

1. which source is the oracle for the product palette (the frozen source, the
   approved frames, or the converted corpus);
2. the correct value of every disputed token;
3. whether `b4b1a3346` or `ec783f6c5` is the correct product state;
4. what actually caused the 231 component digest failures;
5. whether the reported 1060-row change to `components.txt` was real, and
   whether the current file is byte-identical to `032e9faa3`;
6. whether the two remaining `picker_chain` failures are a palette question.

Everything below is measured, not inferred. The only method is decoding the
approved frames and running the conformance gate against controlled trees.

### Commands used

```
git worktree add <dir> --detach <commit>          # read-only trees
python3 tools/frozen_baseline.py --repo . --root target/frozen-baseline prepare
CARGO_TARGET_DIR=<private> cargo nextest run -p termrock-conformance \
    --all-targets -E 'test(/render::components/)'
```

The frozen source was prepared with the repository's own pinned tool, which
verifies the tag identity before extracting:

```json
{
  "tag": "visual-baseline",
  "tag_object": "2718a6b8127a522a3d351d0b4bdb7542fc8da542",
  "source_commit": "4169783ae883788186d161af09ba9c481bd84e5c",
  "source_tree": "ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297",
  "tuiscotti_rev": "a1ea27f757d75d07b05499276cd932e9f8e067ad",
  "source_sha256": "f74646a92505bd9baff29c89398af4d31a4812dd7d51acc84d755be61a383850",
  "snapshots_sha256": "6b547179584d2fb708993fb5ccd415a89423547263836271105b52be0294e1a8"
}
```

## 1. Which source is the oracle

**The frozen source at `4169783ae` is the oracle, together with the frames it
produced. The converted corpus is not the oracle, and it is not even a
measurement of the frozen source.**

`baselines/tuiscotti-v1/` records its own provenance:

```json
"acquisition_method": "legacy_replayed_conversion",
"acquisition_note": "Acquisition was replayed ANSI from legacy snapshots rather than live interactive sessions.",
"historical_oracle_commit": "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b",
"historical_oracle_tag": "visual-baseline",
"generated_at": "2026-10-03T22:09:44Z"
```

`4a79c0a2` is **not** the `visual-baseline` tag. The tag points at
`4169783ae` (tag object `2718a6b8`). `4a79c0a2` is the commit the
`visual-baseline-old` tag points at — the *previous* position of the visual
baseline, which the repository froze separately on 2026-10-18 as "Frozen
pre-refactor visual representation of this project (legacy baseline)".

So the corpus measures the **legacy pre-refactor** product, not the frozen
source. Two independent facts confirm this, and both are decisive:

* The corpus is a *replay conversion* of legacy snapshot ANSI. It never
  rendered the product at `4169783ae`.
* The legacy commit's own `src/theme.rs` carries the ramp the corpus
  measures. At `4a79c0a2` (the corpus's oracle) the theme reads
  `text_muted: WHITE_50` and `text_faint: WHITE_30`, i.e. `0x808080` and
  `0x4d4d4d` — exactly the values the corpus reports and exactly the values
  `b4b1a3346` restored.

The corpus therefore *cannot* settle the palette of `4169783ae`: it is a
faithful measurement of a **different commit**. `b4b1a3346` used it as if it
measured the frozen source. That is the root error.

I verified the corpus is genuinely a different product by decoding it:

| Corpus (`baselines/tuiscotti-v1/`, 120x40 truecolor, 246 frames) | Count |
| --- | --- |
| `#8a8a8a` | **0** |
| `#808080` | 12 834 |
| `#737373` | **0** |
| `#4d4d4d` | 6 164 |
| `#79b8ff` | **0** |
| `┆` (U+2506) | **0** |
| `│` (U+2502) | 6 953 |

And the approved frames in the frozen source's own snapshot store decode to
the opposite (see §2). The two disagree, and the corpus is the one built
from the wrong commit.

## 2. The disputed values, measured

Approved frames were decoded directly from
`target/frozen-baseline/source/snapshots/`. Several screens, all three colour
tiers, plus a full-store census.

### Frame provenance — the frames are of two vintages

The frozen source's snapshot store contains frames from **two eras**. This
matters, because it is exactly the trap `b4b1a3346` fell into.

| Frame approval date | Frames at 120x40 truecolor | Ramp they carry |
| --- | --- | --- |
| 2026-09-13 / 09-14 (pre-retune) | 66 | `0x808080` / `0x4d4d4d`, `│` track |
| 2026-10-16 → 10-18 (post-retune) | 241 | `0x8a8a8a` / `0x737373`, `┆` track |

The retune is `0610904df0` ("feat(theme): colour carries meaning", 2026-10-16),
which set `text_muted: MUTED` (0x8a8a8a) and `text_faint: WHITE_43`
(0x737373); the dotted track is `ef2f816406` ("feat(scrollbar): dotted track on
both axes", 2026-10-16), which changed the track glyph `│` → `┆`.

So the 66 September frames are **stale captures of the pre-retune product**,
and the corpus was converted from exactly that generation. The corpus
measured the 66 stale frames and generalised them to the whole product. That
is why it reports 0x808080/0x4d4d4d/│ and zero 0x8a8a8a: it never saw the
retuned product.

Every frame re-approved after the retune (2026-10-16 onward, 241 of them)
carries the frozen source's values.

### Decoded values from the approved frames

Census over **all 307** frames at 120x40 truecolor in the frozen source
(ink × attribute, cell counts):

| Ink | Cells | Attribute |
| --- | --- | --- |
| `#ffffff` | 942 851 | plain |
| `#8a8a8a` | 206 437 | plain |
| `#b3b3b3` | 83 174 | plain |
| `#808080` | 42 480 | plain |
| `#4d4d4d` | 37 433 | plain |
| `#3a3a3a` | 34 869 | plain |
| `#737373` | 33 603 | plain |
| `#79b8ff` | (see below) | **bold** |
| `#c6a0f6` | 1 154 | **italic** |

The `#79b8ff` keyword and `#c6a0f6` literal appear with **bold** and
**italic** modifiers respectively — that is the frozen source's syntax
palette (`BLUE_INFO` keyword, `SYNTAX_LITERAL` number), not the corpus's flat
white/grey syntax.

Per-role decode, `snapshots/showcase/pages/codeeditor/120x40/truecolor.ansi`
(run-level grouping of identical ink+attribute runs):

| Role | Observed ink | Attribute |
| --- | --- | --- |
| keyword (`pub`, `async`, `fn`, `let`, `mut`, `for`, `return`) | `#79b8ff` | bold |
| string (`"attempt {attempt}`) | `#8bd5ca` | plain |
| literal / number (`200`, `2`, `true`) | `#c6a0f6` | italic |
| operator / punct (`(`, `:`, `&`, `)`, `->`, `<`, `,`, `=`, `;`, `::`) | `#8a8a8a` | plain |
| ident (`fetch`, `url`, `str`, `Body`, `delay`, `client`) | `#ffffff` | plain |
| comment | `#737373` | plain |
| gutter line numbers | `#8a8a8a` | plain |

Reduced tiers, same frame, same roles:

| Tier | keyword | string | literal | operator | comment |
| --- | --- | --- | --- | --- | --- |
| 256 (`38;5;n`) | **111** | **116** | **183** | **245** | **242** |
| 16 (`38;5;n`) | **12** | **6** | **13** | **7** | **8** |

### The scroll track glyph, distinguished from the divider `│`

This was the specific trap. `│` (U+2502) is a **divider/border** glyph in the
approved frames and carries border inks; `┆` (U+2506) is the **scroll track**
and carries the track ink.

Frame census over all frames, glyph × foreground ink:

| Glyph | Ink | Cells | Meaning |
| --- | --- | --- | --- |
| `│` | `#4d4d4d` | 24 641 | border (divider) |
| `│` | `#262626` | 12 885 | border (divider) |
| `│` | `#3a3a3a` | 7 794 | border (divider) |
| **`┆`** | **`#474747`** | **3 271** | **scroll track** |

`#474747` is `WHITE_28` in the frozen source, and the frozen source sets
`scroll_track: WHITE_28` (`src/theme.rs:750`) with the glyph
`scroll_track: "┆"` (`src/theme.rs:2429`). The approved frames carry the
scroll track as `┆` in `#474747` — an exact match on both the glyph and the
ink.

The corpus's "12 600 occurrences of U+2502" is a count of **divider** glyphs
misread as track glyphs. Its zero U+2506 count simply means it never rendered
the post-retune product, whose track is `┆`.

Frames approved before the retune (2026-09-13/14) show the opposite:
**66 of 66 contain `│`, and 0 contain `┆`** — consistent with the corpus
being that generation.

### The table

| Disputed value | Frozen source | Approved frames | Converted corpus | Correct | Evidence |
| --- | --- | --- | --- | --- | --- |
| `text_muted` / operator ink | `MUTED` = `0x8a8a8a` (`src/theme.rs:528,753`) | `#8a8a8a`, 206 437 cells; operator/punct runs read `#8a8a8a` | `0x808080` (12 834), `0x8a8a8a` **0** | **`0x8a8a8a`** | frames + source agree; corpus measured `4a79c0a2` |
| `text_faint` / comment ink | `WHITE_43` = `0x737373` (`src/theme.rs:506,754`) | `#737373`, 33 603 cells; comment runs read `#737373` | `0x4d4d4d` (6 164), `0x737373` **0** | **`0x737373`** | frames + source agree |
| `text_primary` | `WHITE` = `0xffffff` | `#ffffff`, 942 851 cells | `0xffffff` | `0xffffff` | no dispute |
| `text_secondary` | `WHITE_70` = `0xb3b3b3` | `#b3b3b3`, 83 174 cells | `0xb3b3b3` | `0xb3b3b3` | no dispute |
| `text_ghost` | `WHITE_15` = `0x262626` | `#262626` | `0x262626` | `0x262626` | no dispute |
| syntax keyword | `BLUE_INFO` = `0x79b8ff` (`src/theme.rs:537,772`) | `#79b8ff` **bold** | `0x79b8ff` **0** | **`0x79b8ff` bold** | frames + source agree |
| syntax string | `SYNTAX_STRING` = `0x8bd5ca` (`src/theme.rs:539,773`) | `#8bd5ca` | `0x8bd5ca` **0** | **`0x8bd5ca`** | frames + source agree |
| syntax literal / number | `SYNTAX_LITERAL` = `0xc6a0f6` (`src/theme.rs:541,774`) | `#c6a0f6` **italic**, 1 154 cells | `0xc6a0f6` **0** | **`0xc6a0f6` italic** | frames + source agree |
| syntax operator / punct | `text_muted` = `0x8a8a8a` | operator runs read `#8a8a8a` | `0x808080` | **`0x8a8a8a`** | frames + source agree |
| 256 tier keyword / string / literal | `i(111)` / `i(116)` / `i(183)` (`src/theme.rs:961-963`) | `38;5;111`, `38;5;116`, `38;5;183` | 244 / 249 | **`111` / `116` / `183`** | frames + source agree |
| 256 tier operator / faint | `i(245)` / `i(242)` | `38;5;245`, `38;5;242` | 244 / 238 | **`245` / `242`** | frames + source agree |
| 16 tier keyword / string / literal | `LightBlue` / `Cyan` / `LightMagenta` (`src/theme.rs:1031-1033`) | `38;5;12`, `38;5;6`, `38;5;13` | — | **`LightBlue` / `Cyan` / `LightMagenta`** | frames + source agree |
| scroll track glyph | `┆` (U+2506) (`src/theme.rs:2429`) | `┆` in `#474747`, 3 271 cells | `│` 6 953, `┆` **0** | **`┆` (U+2506)** | frames + source agree; `│` is the divider |
| scroll track ink | `WHITE_28` = `0x474747` (`src/theme.rs:511,750`) | `┆` cells carry `#474747` | — | **`0x474747`** | frames + source agree |

## 3. `b4b1a3346` versus `ec783f6c5`

**`ec783f6c5` must stand. `b4b1a3346` must be reverted.**

`ec783f6c5` sets exactly the values the frozen source and the approved frames
carry: `0x8a8a8a` / `0x737373`, keyword `#79b8ff` bold, string `#8bd5ca`,
literal `#c6a0f6` italic, 256 tier `111`/`116`/`183`/`245`/`242`, 16 tier
`LightBlue`/`Cyan`/`LightMagenta`, track `┆`. Every one of those is confirmed
by decoding the frames above.

`b4b1a3346` set `0x808080` / `0x4d4d4d` and track `│`, and its commit
message cites the corpus counts as proof. Those counts are true of the
corpus; they are false of the frozen source, because the corpus measured
`4a79c0a2`.

There is a second, independent reason `b4b1a3346` is wrong, visible in the
frozen source's own history: the ramp `0x808080`/`0x4d4d4d` and the `│`
track are the **pre-retune** values. `0610904df0` (2026-10-16) retuned the
ramp to `MUTED`/`WHITE_43`; `ef2f816406` (2026-10-16) dotted the track.
`b4b1a3346` reverted a deliberate, later, still-approved change back to an
older product state, on the evidence of a corpus built from that older state.

### Additional tokens the frozen source fixes, still not aligned

The crate at `ec783f6c5` still diverges from the frozen source on three
non-ramp tokens. These are outside the disputed set, but they are part of
the same oracle and they matter for the `picker_chain` keys (§6):

| Token | Frozen source `4169783ae` | Crate at `ec783f6c5` |
| --- | --- | --- |
| `error` / `RED` | `0xf05d5d` (`src/theme.rs:521`) | `0xe44545` |
| `success` | `SUCCESS` = `0x3fb950` (`src/theme.rs:513`) | `GREEN` = `0x48e054` |
| `info` | `BLUE_INFO` = `0x79b8ff` (`src/theme.rs:537`) | `PURPLE` = `0x8787ff` |

The branch tip has since moved `success` to `SUCCESS_GREEN` (`0x3fb950`) and
`RED` to `0xf05d5d` (`740374edb2`, `d191acb4a1`, `72c280a6ad`,
`c3e2816db0`), but `info` is still `PURPLE` at the tip — the frozen source
says `BLUE_INFO`. This is a real, still-open gap against the oracle.

## 4. What actually caused the 231 failures

**Two distinct causes, not one. And the baseline was never stale.**

Measured, each in a clean worktree with its own `CARGO_TARGET_DIR`, each
with the `components.txt` blob explicitly verified in the working tree
before the run:

| Tree | Theme ramp | `components.txt` | Result |
| --- | --- | --- | --- |
| `032e9faa3` (as landed) | corpus | `942219d9` | **320 passed, 0 failed** |
| `b4b1a3346` | corpus | `942219d9` | **178 passed, 142 failed** |
| `4d437794fa` | corpus | `942219d9` | 178 passed, 142 failed |
| `4d437794fa` | corpus | `20dcaa6b` | 133 passed, 187 failed |
| `ec783f6c5` | frozen | `942219d9` | 89 passed, 231 failed |
| `ec783f6c5` | frozen | `20dcaa6b` | **318 passed, 2 failed** |
| `4d7a358d3c` | frozen | `942219d9` | 89 passed, 231 failed |
| `4d7a358d3c` + `b4b1a3346` theme | corpus | `942219d9` | 178 passed, 142 failed |
| tip `aa1c15a59b` (shipped) | frozen | `942219d9` (shipped) | **73 passed, 247 failed** |
| tip `aa1c15a59b` | frozen | `20dcaa6b` | 250 passed, 70 failed |

The key-set comparison between the two ramps on the **same** baseline
(`942219d9`, corpus ramp = 142 failures, frozen ramp = 231 failures):

| Key set | Count |
| --- | --- |
| fails with the frozen ramp only (palette-sensitive) | 132 |
| fails with the corpus ramp only | 43 |
| fails with **both** (palette-independent) | **99** |

So the 231 failures are the sum of:

1. **132 keys** that moved because the ramp changed — the palette change is
   genuinely palette-sensitive for them, and `ec783f6c5`'s values are the
   ones the frozen source and the frames carry.
2. **99 keys** that fail under **either** ramp. These are palette-independent:
   they were already failing on the corpus-ramp tree `4d437794fa` before
   `ec783f6c5` touched anything. `37f55475f7` is the commit that addressed
   them (it changed `crates/termrock-overlays/src/completion.rs`, `help.rs`,
   `crates/termrock-controls/src/button.rs`, `crates/termrock-fields/src/input.rs`,
   `crates/termrock-navigation/src/filter_list.rs`,
   `crates/termrock-runtime/src/scroll_region.rs`,
   `crates/termrock-theme/src/downgrade.rs`, `recipe.rs`, `resolve.rs`) and
   simultaneously re-blessed 1060 rows.

**The baseline `942219d9` was never stale.** It is exactly right for
`032e9faa3`, where the gate is 320/320 green. The failures appeared when the
product changed underneath it.

### The critical correction about `20dcaa6b`

`37f55475f7` did not touch `crates/termrock-theme/src/builtin/junie.rs` at
all. Its tree still reads the **corpus** ramp
(`fg: [WHITE, WHITE_70, WHITE_50, WHITE_30, WHITE_15]`, `keyword: WHITE`,
`track: "│"`).

Yet:

* `37f55475f7`'s tree + its own committed baseline `20dcaa6b` → **133
  passed / 187 failed**. The commit was **red against its own baseline**.
* `ec783f6c5`'s tree + the same `20dcaa6b` → **318 passed / 2 failed**.

The re-bless therefore cannot have been taken on the tree that was committed
at `37f55475f7`. It was taken on a working tree that carried the
frozen-source palette (the `ec783f6c5` theme) uncommitted, and the theme
change was committed separately afterwards. So `20dcaa6b` **encodes the
frozen-source palette**, and `ec783f6c5` scores 318/2 against it because the
two agree.

**This inverts the claim in the circulated reference decision.** That document
says "No expected value was updated, no baseline was re-blessed, and no
`BLESS=1` was used. The digests in `components.txt` are byte-identical to
`032e9faa3`". The last clause is true only of `4d7a358d3c` and of the tip;
it is **false of `ec783f6c5`**, whose tree carries `20dcaa6b` — a re-bless
with 1060 changed rows, taken against an uncommitted tree. `ec783f6c5`
scores 318/2 *because* that re-bless encoded its own palette; with the
untouched `942219d9` the same tree scores 89/231.

The two circulating claims — "231 failures" and "318 passed / 2 failed" —
are therefore **not two measurements of the same thing**, and comparing them
to decide the palette question is invalid. The controlled comparison is
§4's table: the same baseline against the same product code with only the
palette swapped.

### The delivery branch is red today

At the branch tip as shipped — frozen ramp, baseline `942219d9` — the gate
reports **73 passed / 247 failed** (measured at `aa1c15a59b` and again at the
then-current tip `6b47e20eb`). The component render conformance gate is **not
green on the delivery branch**. Restoring `942219d9`
(`4d7a358d3c`, "preserve expectations pending reference review") is what
turned it red: the same tree with `20dcaa6b` scores 250/70, and `ec783f6c5`
with `20dcaa6b` scores 318/2.

## 5. The 1060-row change

**It was real, and it has been reverted.**

* `37f55475f7` — "fix(conformance): achieve 100% control-state and baseline
  parity" — changed `components.txt` by **1060 insertions / 1060 deletions**
  (numstat confirmed: `1060 1060`). Blob `20dcaa6b`.
* `4d7a358d3c` — "fix(conformance): preserve expectations pending reference
  review" — changed it back by **1060 / 1060**, restoring blob `3343d127`
  (sha256 `942219d9ad335ac892ad9658a85a57972fa6cf18afa33220af3b0705cdee474f`),
  which is byte-identical to `032e9faa3`'s blob.

**The current file at `parity/termrock-visual-baseline` (`aa1c15a59b`) is
byte-identical to `032e9faa3`:**

```
sha256 942219d9ad335ac892ad9658a85a57972fa6cf18afa33220af3b0705cdee474f
```

at `032e9faa3`, at `4d7a358d3c`, and at the branch tip. Confirmed by
`git show <commit>:... | shasum -a 256` at all three points.

So a reviewer reporting "a 1060-row change" was right that such a change
existed and was committed; it was subsequently reverted. Both blobs are
preserved: `3343d127` (restored, current) and `20dcaa6b` (the re-bless).

## 6. The two remaining `picker_chain` failures

**Not a palette question.** They are not the muted/faint ramp, not the track
glyph, and not the error ink.

Measured on `ec783f6c5` with the re-blessed baseline `20dcaa6b`, by
reverting one token at a time and re-running the `picker_chain` subset:

| Experiment | `picker_chain` result | Failing digests |
| --- | --- | --- |
| `ec783f6c5` as-is | 27 passed, 2 failed | `disabled` → `d6917718cbac7d35`, `pressed` → `1355ef28f8af466f` |
| `diagnostic_error` `0xf05d5d` → `RED` (`0xe44545`) | 27 passed, 2 failed | **identical digests** |
| fg ramp → corpus (`WHITE_50`/`WHITE_30`) | 21 passed, **8 failed** | the 2 originals **plus 6 more** |
| track glyph `┆` → `│` | 27 passed, 2 failed | **identical digests** |
| `RED` `0xe44545` → `0xf05d5d` | 27 passed, 2 failed | `disabled` changed to `6e284403ce5a03a5`; `pressed` **identical** |

Reading:

* Reverting the fg ramp **adds** six failures and fixes none — the ramp
  revert is wrong for these keys, confirming `ec783f6c5`'s ramp is right.
* The `disabled` key *is* sensitive to the error ink (`RED`), but
  `0xf05d5d` does not restore it either: it produces `6e284403ce5a03a5`,
  still not the baseline `e6f639489e0ae895`. So the frozen-source error ink
  alone does not explain it.
* The `pressed` key is **insensitive** to the error ink, the ramp and the
  track glyph — it is insensitive to every palette token tested.

These are the two keys whose digests depend on product behaviour, not on the
palette: `picker_chain::pressed` renders the loading spinner state
(`Status::Busy` → `EmptyState::Loading`, "⠋ Loading") and `picker_chain::disabled`
renders the error state (`Status::Error` → `! Unable to load` / `Press r to retry`).
Both are states the conformance baseline at `032e9faa3` did not bless in the
same product state, and `37f55475f7` changed the product in the same window
(`crates/termrock-overlays/src/completion.rs`, `help.rs`,
`crates/termrock-controls/src/button.rs`, `crates/termrock-fields/src/input.rs`,
`crates/termrock-navigation/src/filter_list.rs`,
`crates/termrock-runtime/src/scroll_region.rs`,
`crates/termrock-theme/src/downgrade.rs`, `recipe.rs`, `resolve.rs`).

They are deterministic: three consecutive runs produced identical digests
(`d6917718cbac7d35`, `1355ef28f8af466f`). So this is a **product/reference
conflict on two states, not flakiness and not a palette defect** — exactly
what the migration record already says ("open work and not a digest-baseline
question").

The branch tip still shows them:
`picker_chain::disabled` baseline `e6f639489e0ae895` got `6e284403ce5a03a5`;
`picker_chain::pressed` baseline `9ac6326c84e80dfe` got `1355ef28f8af466f`.

## 7. Verdict

1. **Oracle.** The frozen source `4169783ae` and the frames it produced are
   the oracle. `baselines/tuiscotti-v1/` is **not**: its own
   `corpus-index.json` names `4a79c0a2` — the `visual-baseline-old` position —
   and its `acquisition_method` is `legacy_replayed_conversion`. It measured
   the pre-2026-10-16 product.

2. **Palette.** `ec783f6c5`'s values are correct and are corroborated by
   decoding the approved frames on every disputed token, in all three colour
   tiers. `b4b1a3346` must be reverted: its ramp `0x808080`/`0x4d4d4d` and
   track `│` are the pre-retune values, and its supporting evidence
   (the corpus) measured a different commit.

3. **The 231 failures.** Not one cause. On the same baseline and the same
   product code, 132 keys are palette-sensitive and 99 fail under either
   palette (a product change, not a palette change). The baseline
   `942219d9` was **never stale** — it is exactly right for `032e9faa3`
   (320/320 green there). The stale artefact was the re-bless `20dcaa6b`,
   taken against an uncommitted tree, and the *removal* of that re-bless
   (`4d7a358d3c`) is what left the delivery branch red.

4. **The 1060 rows.** Real at `37f55475f7` (1060/1060), reverted at
   `4d7a358d3c` (1060/1060). The current file is byte-identical to
   `032e9faa3` (sha256 `942219d9…`). Both blobs preserved.

5. **The two `picker_chain` failures.** Not a palette question, not
   flakiness (deterministic across runs), and not fixed by the error ink,
   the ramp or the track glyph. A product/reference conflict on two states.

6. **The delivery branch is red on this gate.** At the tip the component
   render gate is 73 passed / 247 failed (measured at `aa1c15a59b` and at
   the then-current tip `6b47e20eb`). Restoring `942219d9` without
   re-blessing is what broke it. This must be fixed before the branch
   lands; the fix is to re-bless against the frozen-source palette (which
   `37f55475f7`'s own `20dcaa6b` already did, and which `ec783f6c5`
   matches at 318/2), not to change the palette back.

## 8. What must be recorded as a reference defect

Three defects, all in reference material or reference bookkeeping rather
than in the product:

**R-1 — the converted corpus is a different product, presented as the
oracle.** `baselines/tuiscotti-v1/` is a `legacy_replayed_conversion` of
`4a79c0a2`, the `visual-baseline-old` commit, but its `corpus-index.json`
labels that commit `historical_oracle_tag: "visual-baseline"`. The current
`visual-baseline` tag points at `4169783ae`. Any measurement taken from the
corpus is a measurement of the pre-2026-10-16 product and cannot be quoted
as the frozen source's rendering. The corpus must not be cited as the oracle
for `4169783ae` again.

**R-2 — the frozen source's snapshot store mixes two product vintages.**
66 of the 307 frames at 120x40 truecolor (and their siblings at other sizes)
are pre-retune captures, re-approved on 2026-09-13/14 and never refreshed
after `0610904df0` / `ef2f816406` on 2026-10-16. They carry `0x808080` /
`0x4d4d4d` and the `│` track, and they contradict the other 241 frames and
the source they ship with. A reader decoding "the approved frames" without
checking approval dates gets two different palettes from one tag. The
corpus was converted from exactly this stale subset, which is how R-1 and
R-2 combined to produce `b4b1a3346`.

**R-3 — the re-bless at `37f55475f7` was taken against a tree that was
never committed.** The commit's own tree still carries the corpus ramp in
`crates/termrock-theme/src/builtin/junie.rs`, and the commit is **red
against its own baseline** (133 passed / 187 failed). The blessed blob
`20dcaa6b` matches the *frozen*-ramp tree `ec783f6c5` (318/2), not the tree
it was committed in. A baseline blessed from an uncommitted working tree
cannot be reproduced from the repository, and the commit message "achieve
100% control-state and baseline parity" is not true of the commit as
landed. Any re-bless must be taken from a committed tree and verified
green in that tree before it is committed.

All three results are preserved: the frozen-source palette (frames, source,
`20dcaa6b`), the corpus palette (corpus, `4a79c0a2`, the 66 stale frames),
and the original expectation blob `942219d9` (green at `032e9faa3`).

## 9. Bounded scope

**CLEARED** — with this bounded scope:

* The palette adjudication above (muted, faint, syntax keyword/string/literal/
  operator, 256 tier, 16 tier, scroll track glyph and ink) is settled by the
  frozen source and the approved frames.
* `ec783f6c5` stands as the correct palette. `b4b1a3346` does not stand and
  is to be reverted (it is already reverted in the delivery branch's theme
  state at `aa1c15a59b`: `fg: [WHITE, WHITE_70, TEXT_MUTED, TEXT_FAINT,
  WHITE_15]`, `keyword: BLUE_INFO`, `track: "┆"`).
* The 1060-row change is settled: it was real and it has been reverted; the
  current file is byte-identical to `032e9faa3`.
* The two `picker_chain` failures are **HELD** as an open product/reference
  conflict on two states, out of scope for the palette question, and must not
  be closed by re-blessing or by another palette change.

**Not settled by this adjudication, and explicitly out of scope:**

* Which expectation values the two `picker_chain` keys should carry. That is
  a product-behaviour decision against the frozen reference for the spinner
  and error states of the picker chain, and it needs its own reference
  decision — not a palette change and not a silent re-bless.
* Whether the stale frames in R-2 should be re-approved or removed. That is
  an owner decision about the reference store, not a palette question.
* Whether `baselines/tuiscotti-v1/` should be re-converted against
  `4169783ae`. If it is to remain a reference, it must be re-converted, and
  the conversion would then agree with the frozen source rather than
  contradicting it.
* The `info` token, still `PURPLE` (`0x8787ff`) at the tip where the
  frozen source says `BLUE_INFO` (`0x79b8ff`), and `success`/`error`,
  moved after `ec783f6c5` but not adjudicated here.

## 10. Commands run, and their results

| Command | Result |
| --- | --- |
| `git worktree add <dir> --detach <commit>` | read-only trees at `4169783ae`, `032e9faa3`, `b4b1a3346`, `37f55475f7`, `4d437794fa`, `ec783f6c5`, `4d7a358d3c`, tip |
| `python3 tools/frozen_baseline.py --repo . --root target/frozen-baseline prepare` | tag identity verified; sha256 `f74646a92…`, snapshots `6b547179…` |
| `cargo nextest run -p termrock-conformance --all-targets -E 'test(/render::components/)'` @ `032e9faa3` + `942219d9` | **320 passed, 0 failed** |
| same @ `b4b1a3346` + `942219d9` | **178 passed, 142 failed** |
| same @ `ec783f6c5` + `20dcaa6b` | **318 passed, 2 failed** |
| same @ `ec783f6c5` + `942219d9` | 89 passed, 231 failed |
| same @ `4d437794fa` + `942219d9` | 178 passed, 142 failed |
| same @ `4d437794fa` + `20dcaa6b` | 133 passed, 187 failed |
| same @ `4d7a358d3c` + `942219d9` | 89 passed, 231 failed |
| same @ `4d7a358d3c` + `b4b1a3346` theme + `942219d9` | 178 passed, 142 failed |
| same @ `37f55475f7` + its own `20dcaa6b` | 133 passed, 187 failed (red against its own baseline) |
| same @ `ec783f6c5` pristine (no edits) + its own `20dcaa6b` | **318 passed, 2 failed** (reproducibility check) |
| same @ tip `aa1c15a59b` + `942219d9` (as shipped) | **73 passed, 247 failed** |
| same @ tip `6b47e20eb` + `942219d9` (as shipped) | **73 passed, 247 failed** |
| same @ tip `aa1c15a59b` + `20dcaa6b` | 250 passed, 70 failed |
| same @ tip `0583e77152` + `20dcaa6b` | 250 passed, 70 failed |
| `cargo nextest run … -E 'test(/picker_chain/)'` @ `ec783f6c5` + `20dcaa6b`, `diagnostic_error`→RED | 27 passed, 2 failed; digests unchanged |
| same, fg ramp→corpus | 21 passed, 8 failed; the 2 originals unchanged |
| same, track `┆`→`│` | 27 passed, 2 failed; digests unchanged |
| same, `RED`→`0xf05d5d` | 27 passed, 2 failed; `disabled` digest changed, `pressed` unchanged |
| same, run ×3 | identical digests each run (deterministic) |
| `git show <c>:…/components.txt \| shasum -a 256` @ `032e9faa3`, `4d7a358d3c`, tip | all `942219d9ad335ac8…` (byte-identical) |
| `git show --numstat` @ `37f55475f7`, `4d7a358d3c` | both `1060 1060` |
| frame decode (ANSI SGR parser over the frozen snapshot store) | see §2 tables |
| corpus decode (`baselines/tuiscotti-v1/`, 120x40 truecolor) | `0x8a8a8a` 0, `0x808080` 12 834, `┆` 0, `│` 6 953 |
| frame approval-date census (`git log -1 --format=%ad` per frame) | 66 frames at 2026-09-13/14, 241 at 2026-10-16…18 |

## 11. Corrections to earlier reporting in this thread

The circulated figures do not reproduce as stated, and I record the measured
values instead:

* "With `b4b1a3346` in place the gate reported 89 passed / 231 failed."
  Measured at `b4b1a3346` with its own (unchanged) baseline `942219d9`:
  **178 passed / 142 failed**. The 89/231 figure is what the tree at
  `ec783f6c5` — and `4d7a358d3c` — gives with the *original* baseline
  `942219d9`. The two claims were compared across different trees and
  different baselines, which is what made the palette question look settled.
* "With `ec783f6c5` in place the gate reports 318 passed / 2 failed."
  Reproduces against the re-blessed baseline `20dcaa6b`, which `ec783f6c5`
  carries. The same tree with the original baseline `942219d9` gives
  89/231. So 318/2 is not evidence that the palette was wrong; it is
  evidence that `20dcaa6b` was blessed against the frozen palette — and
  `37f55475f7`, which committed that blob, did **not** contain the frozen
  palette and was red (133/187) against it.
* "The digests in `components.txt` are byte-identical to `032e9faa3`."
  True of `4d7a358d3c` and of the tip. **False of `ec783f6c5`**, whose tree
  carries `20dcaa6b` (1060 rows changed).
* "The 231 failures were caused by the wrong palette." Partly. 132 of the
  231 are palette-sensitive; 99 fail under either palette and are a product
  change.

Both results are preserved: the corpus-palette result (`b4b1a3346`, the
142-key residue, the 66 stale frames) and the frozen-source-palette result
(`ec783f6c5`, the 241 re-approved frames, `20dcaa6b`).

## 12. Note on method

Two of my own early runs were contaminated by a staged baseline file left in
a reused worktree, and I discarded them. Every figure quoted above comes
from a **freshly created worktree at a single commit, with the working-tree
file's sha256 verified against the intended blob before the run**, and with
a private `CARGO_TARGET_DIR` per worktree. The 142/132/99 key-set
decomposition in §4 was computed from the failing-key lists of two such
runs on the *same* baseline (`942219d9`) differing only in the theme
palette.
