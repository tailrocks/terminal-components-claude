# Current refactor audit — UI/TUI restoration required

Date: 2026-09-05
Repository tip audited: `54a7aa1`
Historical source: `d5e7075` (`cc14dd6` UI/source state)

## Result

The architecture migration is substantially present. Historical UI/TUI parity
is not present or proven. The current work must restore the old experience
through the new architecture; it must not continue redesigning the products.

## Proven current state

- Workspace packages exist in `Cargo.toml` for `crates/tui`,
  `crates/tui-testing`, `apps/showcase`, `apps/tablepro`,
  `apps/jackin-preview`, and `xtask`.
- All three applications use the `junie-tui` public facade under `apps/*`.
- Shared runtime, focus, hit testing, layers, themes, components, testing,
  security hardening, and package boundaries are implemented.
- Current component/conformance, package, and focused application evidence is
  substantial. It does not prove historical product rendering.

## Historical contract

`baseline/before/MANIFEST.md` records 499 real-terminal captures made from the
known-good local source at `d5e7075`. The archive covers all three applications,
four sizes, color modes, themes, menus, dialogs, forms, grids, editors,
scrolling, mouse, resize, and route journeys. The archive contains exact ANSI,
plain-text, cursor, HTML, and PNG evidence.

The current `shots/capture-matrix.tsv` has 96 cells and stale provenance. The
current package baselines are post-migration self-baselines. Current visual
tests do not read `baseline/before/**`, so they can pass while output differs
from the accepted UI.

Direct evidence:

- Historical Showcase overview:
  `baseline/before/showcase_overview_default_120x40.txt`.
- Current Showcase overview:
  `shots/showcase_junie_truecolor_120x40.txt`.
- Historical TablePro Connections:
  `baseline/before/tablepro_connections_default_120x40.txt`.
- Current TablePro capture shows a different results-grid surface:
  `shots/tablepro_junie_truecolor_120x40.txt`.
- Historical Jackin manager/Capsule frames are in `baseline/before/` and the
  historical local `shots/` copy; current app captures are structurally
  different.

## Root cause

`18afddd` added the correct new foundations. `7784719` removed the historical
renderer before executable parity existed. Later migration rewrites changed
product rendering and interaction contracts:

- Showcase: `4e07ea1`; current shell at `apps/showcase/src/app.rs:627`.
- TablePro: `5042a40`; current update/draw at
  `apps/tablepro/src/app.rs:1016` and `:1190`.
- Jackin: `444a8f4`; current route/update and draw paths at
  `apps/jackin-preview/src/app.rs:2544` and `:3257`.
- Facade/evidence enforcement then cemented the new output without a
  historical comparison: `1378c31`.

The common architectural failure was treating rendering as replaceable. The
new runtime has no parity adapter preserving historical geometry, paint order,
focus order, hit regions, cursor placement, or interaction transitions.

## Fresh gate snapshot

Measured during this audit, after the documentation edits:

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

Follow [`GOAL.md`](../GOAL.md): build a dual-run parity oracle first, restore
shared visual contracts, restore Showcase/TablePro/Jackin route by route, add
non-vacuous historical-reference tests, classify approved additions and bug
fixes, refresh provenance, and run independent visual review before any
baseline blessing.

Do not treat this report, `REFACTORING_STATE.md`, stale captures, or green
self-baseline tests as completion proof. Fresh source, commands, captures, and
review decide completion.
