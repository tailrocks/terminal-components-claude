# Query completion resize test geometry: already-recorded resolution

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline`. Scope:
`query_editor_keeps_multiline_unicode_and_cursor_through_resize`
(`crates/tablepro-ui/tests/app_tests.rs`), the fourth finding of
`docs/audits/2026-10-22-tablepro-digest-drift.md`.

## Independent-review disposition

The independent review of the 2026-10-22 digest-drift audit returned
**AUDIT STANDS** and confirmed both dispositions. Disposition B asked for
a test-only geometry update to the recorded completion layout (matrix
`707b12c76`: the menu anchors on the replaced token, uses the card and
the highlight row, and the footer shows Accept and Close; frozen oracle
`tablepro::query::completion 120x40 truecolor` at 0 cells).

## Finding: the update is already landed and green

`3baa9615314f217d6622e8aa715ed596147edbd9` ("refactor(tablepro):
eliminate size-specific filter branch and sync visual baseline") already
updated exactly the stale geometry assertions of this test, deriving the
offsets from the card-framed, token-anchored popup:

- the second-row glyph cell moved from `popup_area.x + 2` to
  `popup_area.x + 4` and now asserts `"V"` (the view-kind glyph),
- the matched-grapheme cell moved from `popup_area.x + 9` to
  `popup_area.x + 11` (still `"c"`, still BOLD),
- the unselected row's plain modifier assertion follows the same offset.

All other assertions of the test are untouched.

## Red-before / green-after proof (reproduced 2026-10-23)

- At the audit-time tip `5b6d6cc1e` (pre-`3baa961531`), the test fails at
  `app_tests.rs:574:5` with exactly the geometry pair:
  `assertion 'left == right' failed: left: "V", right: "a"` — the
  failure is the geometry assertion and nothing else
  (`/tmp/attr/resize-red-5b6d.log`).
- At `3baa961531` itself and at the current origin tip, the test passes
  (`cargo nextest run --locked -p tablepro-ui --test app_tests -E
  'test(=query_editor_keeps_multiline_unicode_and_cursor_through_resize)'`:
  1 passed, exit 0).

No further change is required; this record closes Disposition B as
already implemented by `3baa961531`, with the red-before pair matching
the audit's recorded failure verbatim.
