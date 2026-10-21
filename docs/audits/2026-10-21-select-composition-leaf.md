# Select composition leaf

PR #38 source: `6825b07e86bab75872866c6ed49e6877084db9bb`.

Added `composition::select` as a borrowed adapter over `Select::measure` and
`Select::draw`. The existing select remains the sole owner of field and popup
rendering, focus, hit regions, cursor placement, and interaction state. The
application retains the `SelectState`, item data, `update` call, and typed
`SelectAction` handling. The adapter contributes only the layout placement
already provided by the composition resolver.

The new external-consumer test exercises live Unicode option data, read-only
normal and zero-size measurement, resolved focus, keyboard open/move/choose,
pointer open, and narrow/zero-size drawing. It does not add screen-specific
configuration, fixed geometry, reference frames, or application code.

Validation: `cargo nextest run -p termrock --features testing --test
select_composition_consumer` passes both tests. Full Showcase Options/States
composition and frozen CLI parity remain unverified and incomplete.
