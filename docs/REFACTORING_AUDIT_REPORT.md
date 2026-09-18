# Historical refactor audit — UI/TUI restoration required

> Historical report from 2026-09-05. It is retained for provenance only. Do
> not use its readiness, branch, tool-version, or execution claims as current
> authority. Current execution is governed by [`AGENTS.md`](../AGENTS.md) and the
> [`current execution-readiness report`](refactoring-plan/execution-readiness-report.md);
> `GOAL.md` is product/architecture intent only.

Date: 2026-09-05
Repository tip audited: `54a7aa1`
Historical source: `d5e7075` (`cc14dd6` UI/source state)

## Result

The architecture migration was substantially present at the audited snapshot.
Historical UI/TUI parity was not present or proven. The work described by this
report was required to restore the old experience through the new architecture;
it was not authorization to continue redesigning the products.

## State recorded at audited tip 54a7aa1

- At the audited tip, workspace packages existed in `Cargo.toml` for `crates/tui`,
  `crates/tui-testing`, `apps/showcase`, `apps/tablepro`,
  `apps/jackin-preview`, and `xtask`.
- The three historical applications used the `junie-tui` public facade under
  `apps/*`.
- Shared runtime, focus, hit testing, layers, themes, components, testing,
  security hardening, and package boundaries are implemented.
- Component/conformance, package, and focused application evidence was
  substantial. It does not prove historical product rendering.

The later campaign tree adds `apps/holla` and its Holla-specific evidence. That
later state is not evidence from the `54a7aa1` snapshot and is governed by the
current readiness report.

## Historical contract

`baseline/before/MANIFEST.md` records 499 real-terminal captures made from the
known-good local source at `d5e7075`. The archive covers all three applications,
four sizes, color modes, themes, menus, dialogs, forms, grids, editors,
scrolling, mouse, resize, and route journeys. The archive contains exact ANSI,
plain-text, cursor, HTML, and PNG evidence.

At the 2026-09-05 snapshot, the then-current `shots/capture-matrix.tsv` had 96
cells and stale provenance. The current matrix has 112 cells (including Holla)
and its provenance is separately qualified. The
current package baselines are post-migration self-baselines. Current visual
tests do not read `baseline/before/**`, so they can pass while output differs
from the accepted UI.

Direct evidence:

- Historical Showcase overview:
  `baseline/before/showcase_overview_default_120x40.txt`.
- Historical capture at the audited tip:
  `54a7aa1:shots/showcase_junie_truecolor_120x40.txt`.
  The current grouped diagnostic store is `shots/showcase_junie_truecolor_120x40/`
  (`ansi`, `cursor`, `html`, `png`, and `txt`); it is not the frozen oracle.
- Historical TablePro Connections:
  `baseline/before/tablepro_connections_default_120x40.txt`.
- Historical TablePro capture showing a different results-grid surface:
  `54a7aa1:shots/tablepro_junie_truecolor_120x40.txt`.
  The current grouped diagnostic store is `shots/tablepro_junie_truecolor_120x40/`
  (`ansi`, `cursor`, `html`, `png`, and `txt`); it is not the frozen oracle.
- Historical Jackin manager/Capsule frames are in `baseline/before/` and the
  historical local `shots/` copy; current app captures are structurally
  different.

## Root cause

`18afddd` added the correct new foundations. `7784719` removed the historical
renderer before executable parity existed. Later migration rewrites changed
product rendering and interaction contracts:

- Showcase: `4e07ea1`; current shell at `apps/showcase/src/app.rs:627`.
- TablePro: `5042a40`; the then-current update/draw implementation in
  `apps/tablepro/src/app.rs`.
- Jackin: `444a8f4`; the then-current route/update and draw implementation in
  `apps/jackin-preview/src/app.rs`.
- Facade/evidence enforcement then cemented the new output without a
  historical comparison: `1378c31`.

The common architectural failure was treating rendering as replaceable. The
new runtime has no parity adapter preserving historical geometry, paint order,
focus order, hit regions, cursor placement, or interaction transitions.

## Historical gate snapshot at 54a7aa1

Measured during this historical audit, after the documentation edits. These
commands and results are historical evidence; do not replay the `cargo test`
commands, because current policy requires `cargo nextest`.

- `rtk proxy git diff --check`: pass.
- `rtk cargo run -p xtask -- doc-check`: pass; 76 Rust blocks and 865 resolved
  references.
- `rtk cargo test --workspace --all-targets --all-features`: fail; 63 tests
  passed and 3 Jackin journey tests failed:
  `detach_reconnect_and_final_exit_plays_one_outro`,
  `complete_flow_keyboard_first`, and `complete_jackin_flow_keyboard_first`.
- `rtk cargo test -p showcase --test visual`: pass; this is still a
  post-migration self-baseline, not historical parity proof.
- `rtk cargo run -p xtask -- boundary`: fail closed for
  `baseline_moves_are_classified` without an explicit comparison base, and
  fails `props_are_built_once` on seven Jackin constructors in
  `apps/jackin-preview/src/app.rs`.

These results are evidence for the continuation goal, not a completion claim.

## Required continuation

Use current [`AGENTS.md`](../AGENTS.md), the
[`execution-readiness report`](refactoring-plan/execution-readiness-report.md),
and current contracts for execution. The product intent in [`GOAL.md`](../GOAL.md)
requires a dual-run parity oracle first, restoration of shared visual contracts,
and restoration of Showcase/TablePro/Jackin/Holla route by route, with
non-vacuous historical-reference tests, classified additions and bug fixes,
refreshed provenance, and independent visual review before any candidate
baseline update is considered. The frozen visual oracle is never modified.

Do not treat this report, `REFACTORING_STATE.md`, stale captures, or green
self-baseline tests as completion proof. Fresh source, commands, captures, and
review decide completion; `REFACTORING_STATE.md` is historical provenance only.
