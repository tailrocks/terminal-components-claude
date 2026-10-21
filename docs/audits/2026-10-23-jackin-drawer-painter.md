# Jackin detail drawer: fixed RUNS patch table replaced with the shared split painter

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline`, base tip
`ef23c3303c` (origin tip at time of work).

## What was replaced

The last kept-on-purpose fixed RUNS patch table (2026-10-22 jackin
bypass-removal handoff, "deliberately kept"): `paint` in
`crates/jackin-preview-app/src/app/manager_detail_drawer_view.rs` still
stamped a pixel-exact fixed cell table (box seams, title ink, hint
segments, spacer runs) over `paint_pinned_split` after the live split
painted.

## Replacement

- `termrock::feedback::paint_manager_detail_drawer(ui, area, frame)` in
  `crates/termrock-feedback/src/manager_split.rs`: paints
  `paint_manager_split` (the saved split) and then derives the drawer
  presentation from the same live frame data — rule-grey box top/sides/
  bottom recomputed through `box_top` (the running-count badge keeps its
  split paint), the title `Current directory · {workspace_name}` in
  primary bold with the accent gutter beside it, the list selection fill
  kept without the focus bar, and the drawer hint row (`↑↓ Move · Tab
  Actions · Esc Back`). Colors come from the theme ink (junie tokens
  match the patch constants exactly: rule `#3a3a3a`, secondary
  `#b3b3b3`, primary `#ffffff`, muted `#8a8a8a`, card `#111111`,
  select `#173f1c`).
- The app drawer view keeps only the `approved` gate (which scenario,
  focus, theme and size may show the drawer) and calls
  `App::paint_pinned_detail_drawer`, which shares
  `paint_pinned_split_inner` with the plain split path. The patch table,
  its `PaintStyle`/`Modifier` imports and the duplicated `legacy`
  theme check are gone. No fixed cell table remains in
  `crates/jackin-preview-app/src`.

## Byte-neutrality proof (paired build, same shared base)

Paired capture harness (temporary test, deleted after the proof):
`App::for_scenario(scenario, Motion::Paused)`, junie and paper themes,
truecolor and mono, sizes 120x40 and 160x50, all 16 scenarios, each case
rendered twice — resting and after Tab (Tab moves focus to Accounts and
fires the drawer gate). Base build at bare `ef23c3303c`, candidate build
with only this change; per-case FNV frame digests compared.

- base vs candidate: 128/128 identical
  (`/tmp/drawer-capture-base2.txt` vs `/tmp/drawer-capture-cand3.txt`;
  resting frames 64/64, post-Tab frames 64/64).
- The drawer-gated cases (`returning`, junie, truecolor, both sizes,
  post-Tab) are proven to exercise the replaced path: the first candidate
  attempt diverged on exactly those two digests (the shared painter's box
  top repainted the running-count badge cells the patch table had left),
  and the fix restored byte identity — the gate fires and the shared
  painter reproduces the patch output cell for cell.
- The 64 stored `jackin.txt` baseline digests are unchanged (visual
  baseline test failure set identical before and after, see below).

## Gate on the exact working tree

- `cargo nextest run --locked -p termrock-feedback`: 135/136; the one
  failure is `meter_semantic_tokens::low_and_stale_tokens_match_reference
_roles_but_custom_colors_remain_authoritative`, reproduced at bare origin
  tip `ef23c3303c` with the change reverted — pre-existing, upstream
  (same theme-token drift family as the jackin digest red), not this
  lane's.
- `cargo nextest run --locked -p jackin-preview-app`: 188/189; the one
  failure is `jackin_visual_baseline`, reproduced at bare origin tip
  `ef23c3303c` with the change reverted (`digest of 'returning 100 30
  junie truecolor' changed: baseline 678f015972a21ee9, got
  2359288f161828ad`) — pre-existing, upstream, not this lane's. (The
  2026-10-22 handoff recorded the first-failing digest as `returning 120
  40 junie mono`; at `ef23c3303c` the suite stops at the 100x30
  truecolor case first. Same adjudicated class: no expected output
  re-pinned, no PERF_BLESS/BLESS used.)
- `cargo clippy --locked -p termrock-feedback -p
  jackin-preview-app --all-targets`: clean.
- `cargo fmt --check`: clean for the changed crates; workspace check
  still flags a pre-existing `crates/tablepro-ui/src/app.rs:4706` diff
  at bare origin tip (tablepro stream, upstream).

## Regression coverage

`detail_drawer_repaints_from_live_world_data`
(`crates/jackin-preview-app/tests/manager_identity.rs`) drives the real
app: Tab into the drawer, asserts the live title and drawer hints, then
shifts the daemon age in the world and asserts the painted frame
follows. `tab_from_the_manager_paints_the_detail_drawer_hints` already
covers the hint swap.

## Limits

The drawer presentation stays scoped to the approved gate (Returning,
paused, 120+ columns, junie truecolor, focus on Accounts) exactly as
before; wider adoption (other themes/sizes) is a visual change and needs
its own decision.
