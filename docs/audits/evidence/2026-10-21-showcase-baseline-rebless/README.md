# Showcase visual baseline re-bless — frozen-reference evidence

Branch: `wip/showcase-baseline-rebless-20261021`, base `7bdf030be`
(`wip/session-consolidation-20261021`).

Scope: re-record `crates/showcase-ui/tests/baselines/showcase.txt` and the
eleven stale `crates/showcase-ui/tests/page_parity.rs` expectations against
the coordinator head, after classifying every changed digest against the
frozen tag `visual-baseline`.

## 1. The verdict

The recorded Showcase baseline was a **mixed-era artifact**, and neither the
frozen tag's inks nor the reverted tip's inks reproduce it. It is re-recorded
here against the tree that matches the tag.

The three frozen slots that the parity tip's commit `b4b1a3346` changed are
settled by the tag's own source and the tag's own rendered captures:

| slot | tag value | tag source | tag's own rendered capture |
|---|---|---|---|
| muted ink | `0x8a8a8a` | `visual-baseline:src/theme.rs:528`, `:753` | `38;2;138;138;138` × 25 in `snapshots/showcase/pages/overview/100x30/truecolor.ansi`; `38;5;245` × 27 in `…/256.ansi` |
| faint ink | `0x737373` | `visual-baseline:src/theme.rs:506`, `:754` | `38;2;115;115;115` × 22 in `…/truecolor.ansi`; `38;5;242` × 22 in `…/256.ansi` |
| scrollbar track | `┆` (U+2506) | `visual-baseline:src/theme.rs:2429` | the `┆` track column of `…/256.ansi` |

The tag's `apply_16` (`text_muted = Gray`, `text_faint = Gray`,
`text_ghost = DarkGray`) and `apply_mono` (`text_muted = Gray`,
`text_faint = Gray`) agree with the tag's `…/16.ansi`, which paints indices
`15` and `7` only.

`b4b1a3346` set the muted ink to `0x808080` and the faint ink to `0x4d4d4d`
and the track to `│` (U+2502). None of the three appears in the tag's source
or in the tag's own rendered captures. The coordinator head `7bdf030be` reverts
that commit, so the head carries the tag's values.

Evidence files in this directory:

* [`tag-source-lines.md`](tag-source-lines.md) — the seven verified tag source
  lines.
* [`tag-capture-histograms.md`](tag-capture-histograms.md) — the foreground
  histograms measured over the tag's own rendered captures.
* [`tag-overview-100x30-256.ansi`](tag-overview-100x30-256.ansi),
  [`tag-overview-100x30-truecolor.ansi`](tag-overview-100x30-truecolor.ansi),
  [`tag-overview-100x30-16.ansi`](tag-overview-100x30-16.ansi) — the tag's own
  captures, copied verbatim from the tag tree.
* [`classification.md`](classification.md) — the per-entry classification and
  the old-to-new digest table for every one of the 418 changed entries.

## 2. Why the recorded baseline was stale

The recorded baseline file was last written by `e830877cd` ("test: restore
held reference expectations after incoming re-blessing"). Measured against the
three trees that could have produced it, the file is a mixture:

* 680 of the 704 live keys are reproduced **exactly** by the tree at
  `43016c3cb` ("fix(showcase): wire list scroll registration, narrow progress
  percentage, and update baseline digests");
* 24 keys — the `Code editor` entries at `80x24`, `120x40` and `160x50` — are
  reproduced exactly by the tree at `fb6ed4f83` ("feat(showcase): purge editor
  baseline replay and restore live component rendering");
* 32 keys named `Editor …` are **dead**: the page title is now `Code editor`,
  so the `Editor …` keys are never looked up by the test. They are left in
  place by the bless (the bless merges unknown keys back in) and are not
  touched here.

At the era that recorded the file, the **256 tier was not the tag's 256 tier**.
`crates/termrock-theme/src/palettes.rs` at `e830877cd` still carried:

```rust
// Frozen corpus muted ink (baselines/tuiscotti-v1): 0x808080 fg
// renders 244 in every tier frame (1_011_55+ cells). The generic
// cube rule on the current 0x8a8a8a source would give 245.
Color::Indexed(244),
// Frozen corpus faint/chrome ink: 0x4d4d4d fg renders 238
// (cancel-confirm 120x40 borders and every dim shell strip).
Color::Indexed(238),
```

The tag's own rendered capture settles the question in the other direction:
`38;5;245` × 27 and `38;5;242` × 22 appear in
`visual-baseline:snapshots/showcase/pages/overview/100x30/256.ansi`, and
`38;5;244` appears **nowhere** in it. The coordinator branch had already
adjudicated the same slots against the tag (see
[the palette adjudication record](../../../audits/evidence/2026-10-21-theme-palette-adjudication/adjudication.md)
and `docs/architecture/termrock-migration-matrix.md`, "The Junie 256-colour
palette adjudication, settled against the frozen tag"), and moved the 256
table to `Indexed(245)` / `Indexed(242)` / `Indexed(232)` / `Indexed(25)` /
`Indexed(111)` / `Indexed(116)` / `Indexed(183)` / `Indexed(203)`.

The recorded baseline was therefore stale in *both* directions and matched
neither ink set:

* with the tag-aligned inks (the coordinator head `7bdf030be`),
  `Overview 80 24 junie 256` renders `355a2597648cc88e`, while the recorded
  baseline held `e9c247852562611b`;
* with the tip's reverted inks (`0x808080` / `0x4d4d4d`),
  `Overview 80 24 junie truecolor` renders `507ef7d92e16be19`, while the
  recorded baseline held `80780cbbac55284d` — the tag's own value.

## 3. The classification of every changed entry

The full per-entry table is in [`classification.md`](classification.md): 418 of
the 704 live entries changed, each with the era whose tree reproduces the
recorded digest, whether the frame's **text** moved or only its **colours**,
and the cell count that moved to or from each foreground value.

Every changed entry was measured against a before-frame that reproduces the
recorded digest exactly, so the classification is a measurement, not an
inference. Summary:

| group | changed entries | kind | measured cause |
|---|---|---|---|
| `Code editor` (16 / 256 / mono / truecolor) | 32 | content | the editor's own painting changed: the code surface moved off the card plane `0x111111` onto the canvas `0x000000`, the syntax runs moved off the muted ink `0x8a8a8a` onto the secondary ink `0xb3b3b3`, and the bold (`mod 4`) and italic (`mod 1`) modifiers moved onto the syntax runs. Source: the editor component work in `crates/termrock-editors/src/code.rs` and `crates/termrock-text/src/editor.rs`, re-approved upstream at `c487a61ac` |
| `Overview` (16 / 256 / mono) | 24 | content | the Overview token table now prints **each tier's own** value instead of the truecolour hex, and the tier palette is the tag's: `0x8a8a8a` → `Indexed(245)`, `0x737373` → `Indexed(242)`, `0xb3b3b3` → `Indexed(249)`, `0x474747` → `Indexed(239)`, `0x3a3a3a` → `Indexed(238)`, `0xffffff` → `Indexed(231)`, `0x48e054` → `Indexed(78)` |
| `Task runner` (16 / 256 / mono / truecolor) | 32 | content | the parity tip's live painting, `493f9054f` ("refactor(showcase): paint the task runner and terminal pages live") |
| `Terminal` (16 / 256 / mono / truecolor) | 32 | content | the same commit `493f9054f`: the terminal body moved off white and the secondary ink onto the muted ink `0x8a8a8a` and the faint ink `0x737373`, and the background moved off `0xffffff` onto the card plane `0x111111` |
| `Chrome` (16 / 256 / mono) | 24 | colour | the 256 / 16 / mono palette slots moved to the tag's tables |
| `Chrome` (truecolor) | 4 | colour | the parity tip's chrome rewrite `ef35a7f6b` repainted the card: white-on-black became the paper surface `0xfbfaf8`, the paper accent moved from the junie green `0x48e054` to the paper blue `0x3b5bdb`, and three bold runs lost the modifier |
| `Buttons` (junie, `120x40` and `160x50`, all tiers) | 6 | colour | the eight accent cells moved from `0x2b8632` to `0x2e9a38` (`Indexed(238)` → `Indexed(65)` at 256, `Green` → `LightGreen` at 16). `0x2e9a38` is the tag's `GREEN_60` / `accent_pressed` (`visual-baseline:src/theme.rs:516`, `:759`) |
| `Chips & selects`, `Data grid`, `Dialogs`, `Lists`, `Panels`, `Pickers`, `Progress`, `Scrolling`, `Sidebars`, `Tables`, `Trees` (16 / 256 / mono) | 264 | colour | the 256 / 16 / mono palette slots moved to the tag's tables |

Two honest caveats on the measured causes:

* the `Buttons` accent plane is a **rendered** value, not the theme token: the
  head's `crates/termrock-theme/src/builtin/junie.rs:28` still carries
  `GREEN_60 = 0x002b_8632`, while the rendered cell is the tag's `0x2e9a38`.
  That token drift is outside the three slots this change covers and is not
  touched here;
* the `Chrome` truecolour rows are *colour* moves, not content moves: the
  symbols are identical and only the card surface, the accent and three
  bold modifiers moved.

### The decisive per-cell proof for the reduced tiers

The Overview page's token table is a live readout of the tier palette. At the
head it prints the tier's own value; at the recorded era it printed the
truecolour hex in every tier. The head's readout reproduces the tag's
`apply_256`, `apply_16` and `apply_mono` tables **slot for slot**:

| token | head, 256 tier | tag `apply_256` | head, 16 tier | tag `apply_16` | head, mono tier | tag `apply_mono` |
|---|---|---|---|---|---|---|
| `canvas` | `256: 16` | `i(16)` | `Black` | `Black` | `Black` | `Black` |
| `surface` | `256: 232` | `i(232)` | `Black` | `Black` | `Black` | `Black` |
| `surface.elevated` | `256: 233` | `i(233)` | `Black` | `Black` | `Black` | `Black` |
| `surface.overlay` | `256: 235` | `i(235)` | `DarkGray` | `DarkGray` | `DarkGray` | `DarkGray` |
| `field` | `256: 234` | `i(234)` | `Black` | `Black` | `Black` | `Black` |
| `field.hover` | `256: 237` | `i(237)` | `DarkGray` | `DarkGray` | `Black` | `Black` |
| `popover` | `256: 237` | `i(237)` | `DarkGray` | `DarkGray` | `DarkGray` | `DarkGray` |
| `highlight` | `256: 25` | `i(25)` | `Blue` | `Blue` | `DarkGray` | `DarkGray` |
| `highlight.danger` | `256: 88` | `i(88)` | `Red` | `Red` | `DarkGray` | `DarkGray` |
| `selection` | `256: 17` | `i(17)` | `Black` | `Black` | `DarkGray` | `DarkGray` |
| `border.subtle` | `256: 238` | `i(238)` | `DarkGray` | `DarkGray` | `DarkGray` | `DarkGray` |
| `border.strong` | `256: 242` | `i(242)` | `Gray` | `Gray` | `Gray` | `Gray` |
| `scroll.track` | `256: 239` | `i(239)` | `DarkGray` | `DarkGray` | `DarkGray` | `DarkGray` |
| `text.primary` | `256: 231` | `i(231)` | `White` | `White` | `White` | `White` |
| `text.secondary` | `256: 249` | `i(249)` | `Gray` | `Gray` | `Gray` | `Gray` |
| `text.muted` | `256: 245` | `i(245)` | `Gray` | `Gray` | `Gray` | `Gray` |
| `text.faint` | `256: 242` | `i(242)` | `Gray` | `Gray` | `Gray` | `Gray` |
| `text.ghost` | `256: 238` | `i(238)` | `DarkGray` | `DarkGray` | `DarkGray` | `DarkGray` |
| `text.on_accent` | `256: 232` | `i(232)` | `Black` | `Black` | `Black` | `Black` |

Every one of the 19 slots agrees with the tag in all three reduced tiers.

### The truecolour tier

The truecolour source inks never moved between the recorded era and the head:
`crates/termrock-theme/src/builtin/junie.rs` carries `TEXT_MUTED =
Color::from_u32(0x8a_8a_8a)` and `TEXT_FAINT = Color::from_u32(0x73_73_73)`
at the head and at `e830877cd`. Consequently the 30 changed truecolour
entries are **content** moves (the `Code editor`, `Task runner`, `Terminal`
pages and four `Chrome` frames), not ink moves. The truecolour foreground
inventory of `Overview 100 30 junie truecolor` is byte-for-byte the same set
at the recorded era and at the head, and it matches the tag's capture:
`Rgb(255,255,255)`, `Rgb(138,138,138)`, `Rgb(115,115,115)`, `Rgb(71,71,71)`,
`Rgb(179,179,179)`, `Rgb(17,17,17)`, `Rgb(72,224,84)`, `Rgb(58,58,58)`.

### The scrollbar track glyph

`crates/termrock-theme/src/builtin/junie.rs` carries `scroll_track: "┆"`,
`begin: "┆"`, `end: "┆"` at the head, at the recorded era and at the held era.
The tag's `src/theme.rs:2429` is `scroll_track: "┆"`. The glyph is therefore
not a cause of any changed digest and was not re-blessed for its own sake.

## 4. The `page_parity` re-record

`crates/showcase-ui/tests/page_parity.rs` pins 66 frames. Eleven of them
moved on the coordinator head; the other 55 are unchanged and were not
touched. The old values are preserved in the test's header comment.

See `docs/architecture/termrock-migration-matrix.md`, section "The Showcase
visual baseline re-bless and the `page_parity` re-record (2026-10-21)", for
the per-page reason and the old-to-new table.

## 5. What was deliberately not changed

* The 32 dead `Editor …` keys in the baseline file. The bless merges unknown
  keys back in, so they survive; they are never looked up by the test.
* The `showcase-ui` test assertions, tolerances and the `EXPECTED` table
  shape. No test was weakened, skipped or deleted.
* The 55 `page_parity` expectations that still match.
* The `visual-baseline` tag. It was read only.
* The three failing `showcase-ui` editor `app_tests` cases. They fail on the
  branch before this change and are not part of this scope; the failing-name
  set before and after is listed in the matrix section.

### One consequence worth recording

`crates/showcase-ui/tests/baselines/showcase.txt` was one of the seven
*held expectation* blobs listed in
`docs/audits/evidence/2026-10-21-picker-contrast-integration/held-expectations.json`,
byte-identical to `52e1572caaf7d9effd0265bbfdf6edcd0e57533b`
(blob `6de3fa5a291c4427a360eabab99b39f2c2b95909`). Re-recording it changes
that blob, so the *preservation* property recorded for that one path no
longer holds. The manifest is a record of two historical commits and is left
untouched; the coordinator owns the decision to refresh it.
