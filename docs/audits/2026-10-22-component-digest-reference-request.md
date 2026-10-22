# Reference decision requested — the component render digests

Status: **OPEN. The palette is adjudicated; the re-bless is not authorised.**
Branch: `parity/termrock-visual-baseline`. Requested by: coordinator.

## What is blocked

`cargo nextest run -p termrock-conformance --all-targets -E 'test(/render::components/)'`
reports **320 tests run, 73 passed, 247 failed** on the delivery tip.

Every failure has the same shape:

```
digest of `render::components::brand::default 120 40 junie truecolor` changed:
baseline 9d2d64860f2f8023, got 754d865a10dc13ed; classify the change against
§20.10, capture it, then BLESS=1
```

The expected digests live in
`crates/termrock-conformance/tests/baselines/components.txt`. That file is
byte-identical to `032e9faa3`: same blob id `3343d127cfe998069e28037b9abaa95a77f466a4`,
same SHA-256 `942219d9ad335ac892ad9658a85a57972fa6cf18afa33220af3b0705cdee474f`,
2563 lines. It has never been updated on the current branch.

## What is already adjudicated

The palette question is settled. The independent adjudication is in
[docs/audits/evidence/2026-10-22-oracle-adjudication/verdict.md](evidence/2026-10-22-oracle-adjudication/verdict.md),
verdict CLEARED, and it is landed on the delivery branch.

| Value | Adjudicated | Evidence in the approved frames |
| --- | --- | --- |
| muted and operator ink | `0x8a8a8a` | 206 437 cells; the operator runs read `#8a8a8a` |
| faint and comment ink | `0x737373` | 33 603 cells; the comment runs read `#737373` |
| keyword | `#79b8ff`, bold | the bold attribute is confirmed |
| string and literal | `#8bd5ca` and `#c6a0f6`, italic | 1 154 italic cells |
| 256 tier | 111, 116, 183, 245, 242 | the `38;5;` values match |
| 16 tier | LightBlue, Cyan, LightMagenta | `38;5;12`, `38;5;6`, `38;5;13` |
| scroll track | `┆` (U+2506) in `#474747` | `│` is the DIVIDER, in `#4d4d4d`, `#262626`, `#3a3a3a` |

So `ec783f6c5` stands and `b4b1a3346` does not, and
`baselines/tuiscotti-v1/` is not the oracle: its own `corpus-index.json`
names the `visual-baseline-old` position and records
`acquisition_method: legacy_replayed_conversion`.

## The three reference defects behind the conflict

- **R-1** the converted corpus measures a different product, so it cannot
  decide the palette. The corpus was converted from exactly the stale
  subset of frames.
- **R-2** the tag's frame store mixes two vintages. 66 frames at 120x40
  truecolor approved on 2026-09-13 and 14 still carry `0x808080`,
  `0x4d4d4d` and the `│` track; 241 frames re-approved on 2026-10-16 and
  later carry the frozen values. A single frame is not a single oracle.
- **R-3** the re-bless at `37f55475f7` was taken from an uncommitted tree
  and is red against its own baseline.

## The decision requested

The adjudicator's own remedy statement is: **re-bless against the frozen
palette, not to revert the palette.** That is a change to expected values
derived from the candidate, which the delivery rules forbid the
coordinator from making. The owner is asked to authorise exactly this, in
one step, with these conditions:

1. The re-bless is taken from a candidate whose palette is the adjudicated
   one, and only for the 132 palette-sensitive cases. The 99 cases that
   fail under either palette are product work and must be fixed in the
   product, not in the expectation.
2. The old bytes are preserved. The pre-re-bless blob is
   `3343d127cfe998069e28037b9abaa95a77f466a4` and the pre-re-bless file is
   in the history at `032e9faa3`; both stay.
3. The re-bless is classified against §20.10 and carries the adjudication
   reference above, so the record says WHICH oracle decided each value.
4. A second independent reviewer confirms that the re-blessed values agree
   with the approved frames, and that no value was written that the
   approved frames do not support. The review must decode the frames, not
   trust the candidate.
5. The two `render::components::picker_chain::pressed` and `::disabled`
   cases stay red and open: reverting the error ink, the ramp or the track
   glyph fixes neither, and the ramp revert adds six failures. They are a
   product and reference conflict, not a palette question.

## What the coordinator did NOT do

- No expected value was written.
- No `BLESS=1` was used.
- `crates/termrock-conformance/tests/baselines/components.txt` is
  byte-identical to `032e9faa3` and stays that way until the decision
  lands.
- The corpus-era palette landing, which would make the component gate
  green by moving the product away from the approved frames, was NOT
  merged. It is kept on `wip/diagnose-component-digest-drift-20261021` as
  the record of the rejected direction.
