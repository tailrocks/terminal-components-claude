# Selection page ANSI-replay bypass removal audit

## What was removed

`crates/showcase-demos/src/pages/selectable/` (`frames.rs`,
`delete_confirm.ansi`, `bulk_mixed.ansi`) replayed frozen full-screen ANSI
frames over the real render at 120x40 and drove state through an
`Oracle` enum. The bypass is gone. The Selection page now renders through
the shared components:

- Delete confirmation via `termrock-overlays` `Dialog`
  (`Dialog::new(DELETE_DIALOG)` in `Model::with_dialog`), opened with
  `cx.open_layer(DELETE_DIALOG, dialog.layer(cx))` and rendered with
  `ui.layer(DELETE_DIALOG, ...)` from `Model::draw_dialog`, called by the
  shell after the page paints (`crates/showcase-ui/src/app.rs`).
- Bulk-mixed selection rendering lives in the page paint path
  (`paint_member_row`, `paint_header`, `paint_field`, `Summary::collect`);
  the page owns demo data and selection state only.
- `selectable.stamp(ui)` is removed from `crates/showcase-ui/src/app.rs`;
  the shell now calls `Model::draw_dialog` and `Model::hints`.

Proof there is no replay left: `grep -rn "include_str\|include_bytes\|\.ansi"
crates/showcase-demos/src crates/showcase-ui/src` returns nothing.

## Regression tests

- `crates/showcase-ui/tests/selectable_flow.rs` (new, 10 tests): drives the
  real `App` through the Harness — table render, footer hint switching,
  selection marks, editor summary ("Edit N members", `— mixed —`), dialog
  open/cancel/Esc/confirm, empty-selection `d` no-op.
- `crates/showcase-demos/src/pages/selectable.rs` unit tests: key handling
  (Ctrl+A, Shift-extend, Esc, disabled rows), `Summary` mixed detection,
  hints precedence.
- `crates/visual-harness/tests/visual_baseline/overlay_sites.rs`: SITES row
  added for the new dialog site
  (`crates/showcase-demos/src/pages/selectable.rs::with_dialog` →
  `showcase/flows/selectable/delete_confirm`).

## Visual evidence (tuiscotti PTY, frozen snapshots, never updated)

`cargo nextest run -p visual-harness --run-ignored only -E 'test(/selectable/)'`:
267 of 340 pass. Every flow frame the component code owns is pixel-identical
at truecolor for all captured sizes (40x12, 60x18, 72x20, 80x24, 100x30,
120x40, 160x50) — including both dialog-open frames and the bulk-mixed
editor (chip geometry `(inner.width-2)/2`, leading self-coloured pad cell,
secondary `▾`, mixed markers). Verified by cell-level SGR diffs of
`target/tuiscotti/actual` against `snapshots/showcase`.

Pre-change baseline at the branch tip (with the bypass): 70 of 141
flows/pages selectable tests passed. Post-change: the identical failure set
plus the two masked tests below; the dismissed/context/discard/elements
groups (199 tests) all pass.

## Unmasked failures (recorded, not fixed — outside file ownership)

The removed fixture pasted a stale full-screen frame at 120x40, masking
shell drift in the two open-dialog captures. After removal these two tests
fail only on cells owned by `crates/showcase-ui/src/app.rs` (nav/sidebar/
footer rendering), not by the Selection page:

1. `showcase::flows::selectable::bulk_mixed::120x40::truecolor`
2. `showcase::flows::selectable::delete_confirm::120x40::truecolor`

Exact remaining diffs (expected = frozen capture, actual = current shell):

- Sidebar section header row 2: expected `Components` in secondary
  `#b3b3b3` regular; actual white + bold. The frozen captures predate the
  current nav section-header emphasis.
- Sidebar rule row 3: expected `#3a3a3a`, actual `theme.border_subtle`
  `#262626` — the same drift that already fails
  `showcase::pages::dialogs`, `pages::overview` and `pages::selectable`
  at every size (pre-existing known-red; `tag_rule`/
  `border_subtle` vs frozen faint `#4d4d4d`→`#3a3a3a`).
- Nav selected row 11-12: expected bold text with the accent-tint fill
  starting one column wider; actual renders unbolded with the fill one
  column narrower.
- Footer row 39: the frozen renderer painted the overflowing hint
  truncated with an ellipsis (`Esc Clear selection …`); the current
  `paint_footer`/`paint_hint` drops the hint and paints `Tab Next`
  instead.

None of these cells are produced by the Selection page component code;
fixing them means changing shared shell/nav/footer painting and then
re-approving frozen captures on purpose, which is outside this workstream's
ownership and its "never update expected snapshots" constraint.

Non-truecolor tiers (16/256/none/nocolor) of the two dialog-open flows were
already red before the removal (the fixture replayed truecolor SGR into
every tier) and remain red; the lower-tier diff pattern (underlay expected
dimmed with a `dim` modifier, actual undimmed) points at tier-dependent
backdrop mapping in the shared runtime/theme downgrade path and needs its
own investigation by the runtime owners.

## Gate

- `cargo nextest run -p showcase-demos -p showcase-ui --all-targets`:
  failure set is name-identical to the pre-change branch tip (18
  app_tests, 12 elapsed_contract, 4 page_headings, 2 perf, 2 visual — all
  pre-existing); the 10 new selectable_flow tests pass.
- `cargo clippy -p showcase-demos -p showcase-ui --all-targets`: only the
  three pre-existing warnings in `grid.rs`/`settings.rs` remain; clippy
  failures outside these crates are the recorded known-red set
  (`crates/tablepro-ui/src/app.rs:9284`).
- `cargo fmt --check`: clean.
