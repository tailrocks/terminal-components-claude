# Reference decision requested — component render digests against the frozen oracle

Status: **OPEN. Requesting an independent reference decision.**
Branch: `parity/termrock-visual-baseline`. Reviewer: coordinator.

## The question

`crates/termrock-conformance/tests/baselines/components.txt` holds the
expected digest of every component scene. It was last written at
`032e9faa3` ("restore: land the termrock crate split with all source work
(#24)"). Since then the shared theme changed, and the gate now reports
**320 tests run: 89 passed, 231 failed**.

Every failure has the same shape:

```
digest of `render::components::brand::focused 120 40 junie truecolor` changed:
baseline 9d2d64860f2f8023, got 754d865a10dc13ed; classify the change against
§20.10, capture it, then BLESS=1
```

## What the oracle says

The oracle for the product is the frozen source and the approved frames it
produced, not the repository's converted corpus.

`baselines/tuiscotti-v1/` is **not** the oracle. Its own
`corpus-index.json` records `acquisition_method: legacy_replayed_conversion`
and a corpus commit of `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, and it
disagrees with both the frozen source and the approved frames. A previous
coordinator decision (`b4b1a3346`) used that corpus to "restore" a ramp
that the frozen source does not have; that commit is being reverted here
and this document records why.

The frozen source at `4169783ae` states the palette directly:

| Token | Frozen source | Line |
| --- | --- | --- |
| `text_primary` | `WHITE` (0xffffff) | `src/theme.rs:751` |
| `text_secondary` | `WHITE_70` (0xb3b3b3) | `src/theme.rs:752` |
| `text_muted` | `MUTED` (0x8a8a8a) | `src/theme.rs:753` |
| `text_faint` | `WHITE_43` (0x737373) | `src/theme.rs:754` |
| `text_ghost` | `WHITE_15` (0x262626) | `src/theme.rs:755` |
| `syntax_keyword` | `BLUE_INFO` (0x79b8ff) | `src/theme.rs:772` |
| `syntax_string` | `SYNTAX_STRING` (0x8bd5ca) | `src/theme.rs:773` |
| `syntax_literal` | `SYNTAX_LITERAL` (0xc6a0f6) | `src/theme.rs:774` |
| 256 tier | keyword `i(111)`, string `i(116)`, literal `i(183)` | `src/theme.rs:961-963` |
| 16 tier | keyword `LightBlue`, string `Cyan`, literal `LightMagenta` | `src/theme.rs:1031-1033` |
| scroll track glyph | `┆` (U+2506) | `src/theme.rs:2429` |

The approved frames, decoded from the frozen source's own snapshot store,
agree:

| Measurement | Approved frame |
| --- | --- |
| `showcase/pages/codeeditor/120x40/truecolor.ansi`, keyword ink | 14 cells of `38;2;121;184;255` |
| same frame, operator and punctuation ink | `38;2;138;138;138` (0x8a8a8a) |
| same frame, faint ink | `38;2;115;115;115` (0x737373) |
| same frame, 256 tier | `38;5;111` keyword, `38;5;245` operator, `38;5;242` faint, `38;5;183` literal, `38;5;116` string |
| same frame, 16 tier | `38;5;12` keyword, `38;5;13` literal, `38;5;7` operator, `38;5;8` faint |
| `tablepro/query/explain/120x40/truecolor.ansi` | 2 keyword cells of `38;2;121;184;255` |

The independent `all_73_junie_slots_match_reference_capability_output` check,
which derives the reduced tiers from the truecolor tokens with the pinned
reference formula, passes again with these values and fails with the
corpus-derived ones.

## The decision

The component digests therefore encode a palette that the frozen source and
the approved frames contradict. Two readings are possible and the
difference is a product decision, not a mechanical one:

1. **The component baselines are stale.** They were captured before the
   palette was aligned with the frozen source, and they must be re-derived
   from the oracle-aligned rendering. The re-derivation must be recorded as
   a reference decision, with the old bytes preserved, and it needs
   independent review before it lands.
2. **The palette is wrong.** If the component scenes are the product
   contract and the frozen source's token table is not what those scenes
   render, then the theme must keep the captured values and the frozen
   source's `text_muted` / `text_faint` / syntax tokens apply to other
   roles only.

This coordinator's reading is (1), on the evidence above, and the palette
has been aligned with the frozen source in commit `493cc57d6`. The
re-derivation of the expected digests has deliberately **not** been done
here: the task forbids updating expected output from the candidate, and
this repository's own record says the same. The gate therefore stays
intentionally red until an independent reference decision authorises the
re-derivation.

## What an independent reviewer must check

1. That the frozen source token table quoted above is the one the approved
   frames were produced with, for the scenes in `components.txt`.
2. That no component scene is expected to render a muted or faint ink that
   the frozen source never produced.
3. Whether the re-derivation may be done from the oracle-aligned rendering
   once that rendering is proven against the approved application frames,
   and how the old bytes must be preserved.
4. Whether the 231 failing digests are one drift or several, so that the
   re-derivation is scoped to what the oracle actually supports.
