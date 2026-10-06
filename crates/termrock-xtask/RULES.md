# Ownership enforcement check (FIX-008)

`cargo run -p termrock-xtask -- check-ownership` enforces the
consumer/author boundary from `docs/verification/ownership-and-parity.md`
VER-006: preview applications compose reusable Termrock components and do
not paint screens themselves.

## Rules

| ID | Scope | What it flags |
| --- | --- | --- |
| OWN-01 | preview | raw `Buffer`/`Cell` imports (ratatui paths, alias-resolved) |
| OWN-02 | preview | cell-writing paint calls (`Ui` methods on Ui receivers, buffer methods on any receiver) |
| OWN-03 | preview | `Ui::raw`/`peek` raw-buffer escapes (receiver-resolved) |
| OWN-04 | preview | call sites of local paint-wrapper fns (same crate) |
| OWN-05 | preview | paint macros: definitions plus `name!` call sites |
| OWN-06 | preview | `include_str!`/`include_bytes!` stored-frame includes |
| OWN-07 | preview | screen-shaped string literals (3+ lines, box drawing on 2+) |
| OWN-08 | preview | `cfg!(test/debug_assertions)`, `option_env!`, `env!` render gates |
| OWN-09 | preview | `paused`-keyed behavior (review: input/layers stay live) |
| OWN-10 | preview | scenario-id switches (`match scenario`, `==`/`!=`) |
| OWN-11 | preview | width/height comparisons selecting behavior |
| OWN-12 | library | library code depending on a preview crate |
| OWN-13 | library | production library references to `baselines/` frame stores |
| OWN-14 | production | `.expect(`/`expect!` outside the allowed place |

Rule ids are stable: never reuse or renumber. `#[cfg(test)]` regions
(including `any(test, ...)` modules) are production-excluded everywhere.
Comments and string literals never match, except OWN-07/OWN-13, which
classify literal contents. Generated files are scanned, never skipped;
their findings carry the generated flag.

Preview scope is every workspace member outside `crates/termrock*`.
OWN-14 and OWN-13 additionally exclude the `termrock-test-support` and
`termrock-xtask` tooling crates, which fail fast and handle baseline
paths by design.

## Exceptions

`exceptions/v1.json` (schema `termrock-ownership-exceptions/v1`) admits
the grandfathered G1 corpus with ratchet semantics: each `(file, rule)`
entry pins the observed count as a maximum. Counts may shrink as
FIX-002…FIX-005 land, never grow. An entry matching nothing is stale
and fails; an unknown schema version fails closed.

## CI execution

The check runs as the `termrock-xtask` nextest lane
(`tests/ownership.rs`), which the generated CI already executes for
every crate. No bespoke workflow step exists: `.github/workflows/ci.yml`
is pinned-generator output that must not be hand-edited, and the Velnor
generator schema (`.velnor/config.toml`, schema 1) offers no supported
input for custom steps — the same gap FIX-019 tracks for the review
template. A dedicated CI step awaits upstream Velnor support; until
then, this lane is the enforcement point.
