# TablePro digest and perf drift: root-cause evidence package

Date: 2026-10-22. Branch: `parity/termrock-visual-baseline`, tip at
investigation start `5b6d6cc1ecb8a5f437ee993b93d93a43b02d523c`. Lane:
root-cause only (fixes limited to product-source perf repairs; no
expected-output file was touched).

Status: **reference defect record + closure case. No decision is final
until an independent review records its verdict on this document.** Both
observed results are preserved verbatim; no baseline file was edited.

## Failures investigated

All four reproduced at `5b6d6cc1e` in worktree
`/private/tmp/termrock-tablepro-digest-wt` (`cargo nextest run --locked`):

| Test | Observed at tip | Expected |
|---|---|---|
| `tablepro-ui::visual tablepro_visual_baseline` | FAIL, first row `workbench-default 120 40 junie truecolor`: baseline `691b9e8e2f4422a7`, got `02a3a9386f82f3db` | digest match |
| `tablepro-ui::perf frame_tablepro_grid_500x14_120x40` | FAIL, 112 allocs | < 100 allocs |
| `tablepro-ui::perf frame_tablepro_query_editor_2k_lines` | FAIL, 6045 allocs | < 40 allocs |
| `tablepro-ui::app_tests query_editor_keeps_multiline_unicode_and_cursor_through_resize` | FAIL at `app_tests.rs:574`: left `"V"`, right `"a"` | completion row cell `"a"` |

The harness aborts at the first mismatching digest row
(`crates/termrock-test-support/src/digest.rs:284`), so one run exposes one
row; rows after the first mismatch are unverified by that run.

## Finding 1: `tablepro_visual_baseline` digest red

Classification: **(b) deliberate recorded change — reference defect,
adjudication required.**

First bad commit: `4d6ad401249e05a4dcf9f856a82147e54008490d` —
`fix(jackin): match the 120-col capsule command palette`. Spot checks:

| Commit tested | Digest test | First failing row |
|---|---|---|
| `40c2663fe` (git bisect good) | PASS | — |
| `afb602936` (parent side of the jackin capsule lane) | PASS | — |
| `4d6ad4012` | FAIL, exit 100 | `quick-switcher 120 40 junie truecolor`: baseline `b206bc551015acfe`, got `b90250e64f4bfb1e` |
| `4593c762b` | FAIL, exit 100 | same row, same pair |
| `91ccd0138` | FAIL, exit 100 | `table-grid 120 40 junie truecolor`: baseline `31ddabeae1abc90b`, got `0822e8cf69e3084a` |
| tip `5b6d6cc1e` | FAIL, exit 100 | `workbench-default 120 40 junie truecolor`: baseline `691b9e8e2f4422a7`, got `02a3a9386f82f3db` |

`afb602936..4d6ad4012` contains exactly one commit (`4d6ad4012`), so it is
the first commit on the tip ancestry that flips the test. Its shared-code
content, all recorded in the migration matrix as deliberate jackin
matches:

- `crates/termrock-theme/src/builtin/mod.rs`: picker `Part::ICON` and
  `Part::META` drop `BOLD` and gain a `FOCUSED` state painting
  `Role::Fg(Primary)`. This moves the tablepro quick-switcher row
  (`b206bc551015acfe` → `b90250e64f4bfb1e`).
- `crates/termrock-navigation/src/item.rs`: an empty item glyph now
  paints one space cell instead of leaving the cell untouched.
- `crates/termrock-runtime/src/rowui.rs`: a disabled `CursorStyle::Fill`
  row keeps `Role::DisabledFg` on its label padding instead of the
  primary fill ink.

The same lane's earlier commit `afb602936` already moved Holla digests and
is recorded in
`docs/audits/2026-10-22-holla-reference-regeneration-case.md`; the
tablepro baseline was not re-approved by that lane either. Later commits
on the branch (`4593c762b` theme lib ink, and the theme declaration
provenance work `cfb70e8eb`/`5b4d96658`/`849d26dad`/`b4b1a3346`) move
further rows (`table-grid`, then `workbench-default`), which is why the
first failing row differs per commit.

An intermediate bisect step (`b12ced50e`) reported "bad" only because its
`crates/termrock/Cargo.toml` is unparseable (a duplicate nameless
`[[test]]` entry, fixed 18 seconds later by `d4b6021f2`); the workspace
cannot build at that commit, so it is not a digest flip. Noted so nobody
re-chases it.

### Recommended resolution (pending independent review)

The shared picker/row ink changes are deliberate and recorded; the
tablepro reference rows are stale relative to them, not the product being
wrong. Recommend: adjudicate the recorded shared ink changes, then
re-approve `crates/tablepro-ui/tests/baselines/tablepro.txt` through the
normal bless path in one reviewed step, capturing every row pair in the
adjudication record. Until then the digest stays red on purpose; nothing
was blessed in this lane.

## Finding 2: `frame_tablepro_grid_500x14_120x40` (112 vs 100 allocs)

Classification: **(a) accidental regression — fixed in product source.**

First bad commit (git bisect, good `29dddba0d`, bad tip):
`2c5838634` — `fix(tablepro): match narrow query results frames`. That
commit added `workbench_header_pieces` (four `format!`/`to_owned`
strings plus two `Vec`s) and `header_overflow` (two more `Vec<bool>`),
computed on every workbench frame at every width — the narrow-collapse
widths were measured by building the strings even though the overflow
path never paints them. The 120x40 grid bench gained the ~12 allocations
that pushed 112 over the 100 budget.

Fix shipped in this lane (behavior-identical, widths computed from
borrowed pieces; `HeaderKeep` makes the overflow flags stack resident):

- `crates/tablepro-ui/src/app.rs`: `workbench_header_pieces` →
  allocation-free `workbench_header_widths` returning
  `([u16; 6], [u16; 2])`; `header_overflow` now operates on
  `HeaderKeep` (fixed `[bool; 16]`) with the same drop priorities and
  the same ellipsis arithmetic.
- `crates/tablepro-ui/src/app/explorer_collapsed.rs`: the
  `header_drops_the_size_badge_before_help` assertions read the
  `HeaderKeep` flags instead of comparing a `Vec<bool>`.

Result at tip with the fix: `frame_tablepro_grid_500x14_120x40` PASS
(nextest exit 0, no `exceeded` line), `frame_tablepro_query_editor_2k_lines`
PASS, and all 76 `termrock-editors` tests pass. The failing gate before
the fix is the regression evidence (112/6045-alloc failures above).

## Finding 3: `frame_tablepro_query_editor_2k_lines` (6045 vs 40 allocs)

Classification: **(a) accidental regression — fixed in product source.**

First bad commit (spot bisect, good `fc7e932cb`, bad `09aa3a18a`, and
everything from `09aa3a18a` to tip bad):
`09aa3a18a` — `fix(tablepro): match the query gutter under overlay
cards`. To tint the cursor block's line number it added
`self.current_block(st)` to `CodeEditor::draw`. `current_block` calls
`blocks`, which runs the segmenter over the entire document and sorts a
fresh `Vec<Range>` — for the 2,000-line query that is ~60 heap
allocations per draw (6030 allocs per 100-iteration bench), against a 40
budget.

Fix shipped in this lane (`crates/termrock-editors/src/code.rs`):
`HighlightCache` now carries `blocks`/`blocks_generation`/`blocks_valid`
and the draw path rebuilds the block list only when `edit_generation`
moves, falling back to the uncached `current_block` for the
`u64::MAX` never-cache generation. `current_block` keeps its exact
signature and semantics via the new `current_block_in` helper.

Result at tip with the fix: both perf tests PASS (above).

## Finding 4: `query_editor_keeps_multiline_unicode_and_cursor_through_resize`

Classification: **(b) deliberate recorded change — test/reference
defect, adjudication required.**

First bad commit: `707b12c7621df67879361a1d7d55873764ded9ed` —
`fix(tablepro): match the completion menu and SQL ink`.

| Commit tested | Resize test |
|---|---|
| `4d6ad4012` | PASS |
| `30cfaab7c` | PASS |
| `707b12c76` | FAIL, `app_tests.rs:574`: left `"V"`, right `"a"` |
| tip `5b6d6cc1e` | FAIL, same assertion, same pair |
| `312b8f38a` (a parallel tip ancestor without `707b12c76`) | PASS |

`707b12c76` deliberately re-anchors the completion popup on the replaced
token (`popup_anchor` in `crates/termrock-overlays/src/completion.rs`),
paints the baseline selection row, and adds the Accept/Close footer; the
migration matrix entry records "The menu anchors on the replaced token,
uses the card and the highlight row, and the footer shows Accept and
Close." The behavioral app test still pins the pre-anchor popup cell
geometry (`app_tests.rs:574` expects `"a"` where the re-anchored,
card-framed row renders `"V"`), so the test was left red when the
deliberate change landed — on `707b12c76` itself, not just at tip.

### Recommended resolution (pending independent review)

The re-anchor is deliberate and recorded; the app test is the stale side.
Recommend: update
`query_editor_keeps_multiline_unicode_and_cursor_through_resize` to the
recorded completion layout (popup anchored on the replaced token, card
frame) as a test-only change reviewed next to the `707b12c76` record, or
revert the re-anchor if review rejects it. No expected-output file is
involved in this failure.

## Reproduction commands

```
mise exec -- cargo nextest run --locked -p tablepro-ui --test visual \
  -E 'test(=tablepro_visual_baseline)'
cargo nextest run --locked -p tablepro-ui --test perf \
  -E 'test(=frame_tablepro_grid_500x14_120x40) or test(=frame_tablepro_query_editor_2k_lines)'
cargo nextest run --locked -p tablepro-ui --test app_tests \
  -E 'test(=query_editor_keeps_multiline_unicode_and_cursor_through_resize)'
```

## Verification log index (this lane)

`/tmp/bisect-digest-run.log`, `/tmp/dig-{4593,4d6a,afb6}.log`,
`/tmp/perf-{afb6,2c58,29dd,707b,30cf-parent,fc7e,ed5b,8a9a,2f61,09aa,52e1}.log`,
`/tmp/bisect-grid-run.log`, `/tmp/grid-312b.log`,
`/tmp/resize-{afb6,4d6a,707b,312b,30cf}.log`, `/tmp/fix-perf.log`.
