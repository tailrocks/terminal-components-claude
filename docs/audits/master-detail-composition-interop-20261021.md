# MasterDetail composition interop (2026-10-21)

## Scope and decision

This increment adds a borrowed `composition::master_detail` adapter. It allows
the existing public `MasterDetail` block to be placed in a `Group` inside an
`AppShell` body. The adapter delegates measurement and drawing to the block's
existing methods and borrows its existing `MasterDetailState`; it adds no
interaction handler, state, renderer, or layout policy. Invalid blocks are
rejected when the adapter is created.

The external consumer test composes different domain data through public
Termrock APIs. It compares the adapter with direct placement across offsets,
widths around the configured breakpoint, narrow/zero widths, a resize, and
truecolor/256/16-color/mono profiles. It also checks regions, focus, cursor,
stable-key keyboard activation, and retained component state.

## Evidence

- Base source before this increment: `4f0836ec6cf67fdf72318af6f2433b7f4834c3cd`.
- `crates/termrock/src/composition.rs` adds the adapter.
- `crates/termrock/src/blocks/master_detail.rs` validates required child groups
  and the existing composition depth limit.
- `crates/termrock/tests/manager_composition_consumer.rs` is a separately
  compiled integration consumer; it imports only the public `termrock` facade.
- `crates/termrock/Cargo.toml` registers that test explicitly with `testing`.

## Limits / remaining work

This is only a composition adapter contract. It does not migrate Jackin's
Manager screen, remove application painters, or demonstrate product parity.
The test compares composed output to the same block placed directly; it is
not a comparison against the frozen baseline. Full workspace and application
acceptance checks remain outstanding. The composition trait cannot return
draw/measure errors, so after validating configuration the adapter follows
the existing Group convention and uses the block's established error fallback.

No reference revision changed.
