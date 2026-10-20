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
