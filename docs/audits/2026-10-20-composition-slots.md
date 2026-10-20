# Composition slot adapter increment

This increment adds a geometry-private `CompositionSlots` adapter over the
reviewed `termrock-layout` composition plans. It adds row, column, and wrap
groups of borrowed component leaves. The framework supplies the root `Ui`
area; ordinary composition callers receive no `Rect`, buffer, or painter API.

The component-author boundary is `author::CompositionLeaf`, extending the
existing pure `Measure` contract and accepting logical bounds for drawing.
`Ui::compose` resolves a group from live leaf measurements, then calls each
visible leaf once with the logical rectangle while `Ui::with_area` and the
existing registration methods share the plan's visible clip. Width-sensitive
leaves receive one bounded second measurement when their resolved width differs
from their intrinsic width. The second measurement is pure and uses tight width
constraints; no input routing or application action runs during either pass.

Coverage includes runtime tests for row, column, wrap and resize recomputation,
single draw calls, stable control identity, shared hit-region clipping, and a
width-dependent text leaf compressed by row allocation. A separate integration
consumer compiles only against public Termrock APIs with different live data.

An independent runtime probe found a cursor escaping a clipped real `TextInput`
leaf. `Ui::record_cursor` now suppresses cursor requests outside its active clip.
It does not relocate an offscreen insertion point to a different character or
cell. The real-input integration regression failed before this fix with visible
input `Rect { x: 6, y: 0, width: 4, height: 1 }` and cursor
`Position { x: 12, y: 0 }`; it now requires no visible cursor rather than a false
cursor at the last visible cell. The leaf must be measured/scroll-adjusted to
make its actual caret visible; that remains a component-level follow-up.
Focused runtime coverage includes out-of-clip and empty-clip requests.

The current API intentionally composes one flat group per call. Nested groups,
grids, named slots, shell and master-detail blocks, and application migrations
remain incomplete and must build on this adapter without exposing application
geometry.

Source base was reviewed commit `801e6c94892703bde5309e3daa31f37c3a727910`;
layout dependencies are `8f7fc45a0`, `a39869753`, and `eba2423164e679a2b5cdfd842aad060bea0dff6d`.
The base includes the reviewed Velnor-independent PR source through TablePro
help overlay commit `a5766d247` and the shared style helper from `708420d47`.

Focused checks:

```text
mise exec -- cargo nextest run -p termrock-runtime --lib ui::slot
mise exec -- cargo nextest run -p termrock-runtime --test composition_consumer
mise exec -- cargo nextest run -p termrock-runtime --test composition_cursor_clip
mise exec -- cargo nextest run -p termrock-runtime --all-targets
mise exec -- cargo clippy -p termrock-runtime --all-targets -- -D warnings
rustfmt --edition 2024 --check crates/termrock-runtime/src/author.rs crates/termrock-runtime/src/ui/mod.rs crates/termrock-runtime/src/ui/slot.rs crates/termrock-runtime/tests/composition_consumer.rs
```

Workspace `cargo fmt --check` remains red on pre-existing fixed-cell data in
`crates/showcase-demos/src/pages/chrome.rs`; this increment did not edit that
file. No application parity or full architecture acceptance is claimed.

Cursor-fix validation: `ui::slot` passed 6/6, `composition_cursor_clip` passed
1/1, and the full runtime target suite passed 306 tests with 3 pre-existing
skips. Package clippy, changed-file rustfmt, and `git diff --check` passed.
