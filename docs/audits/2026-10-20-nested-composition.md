# Nested composition groups — 2026-10-20

Status: implementation slice on `parity/nested-composition-20261020`, based on
the reviewed composition/runtime commit `10ab21cdb33939ec5ead16b1923c8f91a4bf5cb8`.

`CompositionSlots` supports borrowed component leaves and nested row,
column, and wrapping groups. The layout crate computes intrinsic group sizes
from already measured children. The runtime collects leaf data into a finite
tree, resolves shared geometry, performs at most one width-sensitive leaf
remeasure pass, and draws each leaf once through the existing `Ui::with_area`
clip and registration path. No input handlers or domain effects run during
measurement. No second rendering or input router is introduced.

Follow-up adversarial review found and fixed two runtime defects: width-driven
reflow now receives the ancestor's available height instead of a provisional
child height, and composition depth is bounded at 128 with an explicit
`CompositionError` returned by the draw/compose call. Deep invalid trees are rejected before measurement and
destroyed iteratively, avoiding recursive overflow during teardown.

The public external-consumer integration tests use borrowed data and cover a
multi-level tree. Layout tests cover wide/narrow wrapping, empty data, zero
constraints, child min/max constraints, and large groups. Existing runtime
tests continue to cover shared bounds, cursor clipping, flex wrapping, and
resize recomputation. External-consumer regressions cover width reflow into a
3-row text layout and rejection/destruction of a 2,001-level tree.

This is only the nested-group foundation. Named content adapters, shells,
panels, overlays, and application migrations remain outstanding. Product
parity is not established by this audit.

## Geometry consolidation follow-up

At `8cecb53b26470f7de6edfe6bbc95b25711cf1570`, before structural edits,
`cargo nextest run -p termrock-runtime --test composition_consumer --test
composition_cursor_clip` passed all 8 tests. After consolidation, the same
command passed the same 8 tests, including resolved bounds, cursor clipping,
width-sensitive 3-row reflow, wide leaf counts, zero-size behavior, and
bounded-depth error/drop behavior. The final draw path now consumes retained
logical and visible bounds; it does not invoke the layout resolver.

## Selective consolidation onto the current PR #38 source

The nested-composition branch was merged with PR #38 tip
`ef1d6b37daca2ca895c6b35c3253ebe3e37f57d6`. Conflicts in fields, session,
binding support, layer slots, and existing audit files were resolved in favor
of that PR #38 source. The composition API was ported additively into a new
`ui/composition.rs` module and an author submodule. Existing `LayerSlot`,
`LayerSlotAuthor`, filtered binding publication, binding extension, prepared
frame handling, and component identity remain owned by the PR #38 versions.
The shared layout resolver is unchanged; the nested intrinsic-size contract
and its tests were added to it.

The first merged runtime run exposed an off-clip cursor request in the new
composition regression. `Ui::record_cursor` now drops cursor requests outside
its active clip. The regression failed before this fix and passes after it;
the cursor is not moved to a different visible cell.

Validation on the selective integration worktree:

- `cargo nextest run --locked -p termrock-layout -p termrock-runtime -p termrock`: 356 passed, 3 skipped.
- Runtime composition integration targets: 8 passed, 0 skipped.
- `cargo nextest run --locked -p termrock --features testing --test slots`: 1 passed, 0 skipped.
- `cargo check --locked -p termrock --example composition_consumer`: passed.
- Strict clippy for `termrock-layout`, `termrock-runtime`, and `termrock`, all targets: passed.
- Rustfmt check for changed Rust files and `git diff --check`: passed.
- Workspace `cargo fmt --all --check`: still fails on pre-existing formatting in untouched `crates/showcase-demos/src/pages/chrome.rs` and `crates/tablepro-ui/src/app.rs`.

The independent review of the final merged composition API remains required.
This library increment does not establish application visual or functional
parity; all four application acceptance gates remain open.
