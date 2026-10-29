# Reference decision — component render digests against the frozen oracle

Status: **RESOLVED. No reference decision was needed.**
Branch: `parity/termrock-visual-baseline`. Reviewer: coordinator.

## What was asked

`crates/termrock-conformance/tests/baselines/components.txt` holds the
expected digest of every component scene. It was last written at
`032e9faa3` ("restore: land the termrock crate split with all source work
(#24)"). After a theme change the gate reported **320 tests run: 89 passed,
231 failed**, every failure of the shape:

```
digest of `render::components::brand::focused 120 40 junie truecolor` changed:
baseline 9d2d64860f2f8023, got 754d865a10dc13ed; classify the change against
§20.10, capture it, then BLESS=1
```

Two readings were possible: the digests were stale, or the palette was
wrong. This document records how that was settled.

## What the oracle says

The oracle for the product is the frozen source and the approved frames it
produced, not the repository's converted corpus.

`baselines/tuiscotti-v1/` is **not** the oracle. Its own
`corpus-index.json` records `acquisition_method: legacy_replayed_conversion`
and a corpus commit of `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, and it
disagrees with both the frozen source and the approved frames. A
coordinator decision (`b4b1a3346`) used that corpus to "restore" a muted
and faint ramp of 0x808080 and 0x4d4d4d. The frozen source says otherwise:

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

## The resolution

The palette was wrong, not the digests. With the theme aligned to the
frozen source, commit `ec783f6c5`, the component render gate reports:

```
320 tests run: 318 passed, 2 failed
```

The two remaining failures are `render::components::picker_chain::pressed`
and `render::components::picker_chain::disabled`, which are open work and
not a digest-baseline question. The independent
`all_73_junie_slots_match_reference_capability_output` check, which derives
the reduced tiers from the truecolor tokens with the pinned reference
formula, passes again with these values and fails with the
corpus-derived ones.

**No expected value was updated, no baseline was re-blessed, and no
`BLESS=1` was used.** The digests in `components.txt` are byte-identical to
`032e9faa3`; the product was corrected to match them.

## What an independent reviewer must still check

1. That the frozen source token table quoted above is the one the approved
   frames were produced with, for the scenes in `components.txt`.
2. That the 231 recovered digests are recovered for the right reason, and
   not because a scene happens to agree under both palettes.
3. The two remaining `picker_chain` failures.

