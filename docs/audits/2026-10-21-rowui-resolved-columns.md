# RowUi resolved-column composition — 2026-10-21

This bounded runtime change lets a row consume `termrock_layout::ResolvedColumns`
without translating it back into width-only `Track`s. `ResolvedColumns` already
holds each column's offset, width, and visibility. Reconstructing positions from
widths plus a gap inserts space for dropped interior columns and loses the
difference between a dropped column and an intentionally visible zero-width
column.

`RowUi::columns_resolved_with_insets` places those resolved cells in the
remaining row area, reserves caller-supplied leading and trailing insets inside
that component boundary, and clips returned rectangles to the content area.
Hidden cells return no rectangle; visible zero-width cells retain their
resolved position. `ColumnsUi::rect` is the same geometry for drawing and for
consumer hit-region registration. Existing `columns` and `columns_with_gap`
behavior remains width-and-gap based.

The reference contract is visible in the fixed baseline source
`4169783ae883788186d161af09ba9c481bd84e5c`,
`src/bin/holla/screens/finder.rs`: line 853 removes four cells for the row
content; lines 863–872 resolve the cells and two-cell gaps; line 941 begins
content three cells after the row start. The current PR #38 Holla row-composition
WIP is still held on visual parity. Its independent review clears shared row
ownership and input mechanics only, and explicitly holds the generic renderer's
cell-by-cell parity because sampled screens bypass that renderer. The packet is
retained at
[`holla-home-row-review.json`](evidence/2026-10-21-rowui-resolved-columns/holla-home-row-review.json),
SHA-256 `e62fb52056092b60210d6cb468b16b7c597ada145d66f6d9b4700ee6e20e7932`.

Before the adapter, a focused probe exercised the width-only conversion using a
dropped second column and a visible zero-width third column. It failed on
Nextest run `84bb22a0-d0da-4513-bc9a-6391747ee97e`: the final cell began at
x=7 while the resolved layout placed it at x=6. The corrected regressions use
the resolved adapter directly. They verify actual row rendering, hidden versus
zero-width slots, a wide Unicode grapheme, narrow clipping, and control hit
bounds derived from the exact returned rectangles.

After normally merging PR #38 through `45f8d8ecefc81fcc25f87b7e7aa036363085c21b`,
the validated candidate was `fb01738e763b3078a6b5677a0a50d449a96ac3cb`.
`cargo nextest run -p termrock-runtime --all-targets` passed 421 tests with
three existing skips (run `1255fb72-5802-4540-8338-8eeac96d8148`); the focused
RowUi suite passed 17/17 (247 unrelated tests skipped; run
`bc591c55-ddfa-4c79-8a10-dd17ca183797`). Strict
`cargo clippy -p termrock-runtime --all-targets -- -D warnings`,
`cargo fmt --all --check`, and `git diff --check` passed. No Holla app source,
expected output, frozen artifact, or reference revision changed. This API is
not yet adopted by Holla or demonstrated in Showcase, and does not clear the
held Holla visual conflict or application acceptance.
