# Binding-selection identity audit

Base: `0cf26d54ecf53adec8a925050882d6a65d736f02`.

An independent probe in `/private/tmp/termrock-binding-adversarial-0cf/tests/contracts.rs`
failed on that base in both cases: filtered static bindings retained the `Alpha` hint
after selecting `Beta`, and composite bindings retained `Beta` after switching back to
`Alpha`. The probe used the actual public `publish_bindings_filtered` / `extend_bindings`
APIs and `Harness` rendering.

The public `BindingTableId` remains unchanged and continues to identify its original
static or dynamic table. Runtime caches now also compare the exact selected
`BindingDescriptor` slice. Focused hints, conflict diagnostics, and dynamic-binding base
identity all use this structural comparison, so frame-local composite revision reuse and
equal-sized filtered subsets cannot reuse stale results. Comparisons allocate only when
the descriptor set changes; stable frames reuse the cached vectors. The external
integration test also exhaustively matches the existing public enum variants.

`crates/termrock-runtime/tests/binding_filtered_identity.rs` covers alternating static
subsets, repeated composite frame swaps, selected-set/remap conflict diagnostics, and
the public enum shape. The external pre-fix run was `0/2` passing with both stale-hint
assertions failing (`Alpha` remained after selecting `Beta`; `Beta` remained when the
composite frame returned to `Alpha`).

The cache keeps a reusable descriptor vector and compares it exactly before returning a
hit. This avoids hash-collision aliasing and allocates only when selected descriptors
change. The dynamic registry retains the selected base descriptor slice for the same
exact comparison. Key-map revision remains part of focused-hint identity, so a remapped
chord refreshes even when the selected descriptors themselves are unchanged.

This increment is limited to runtime binding identity and cache invalidation. It does not
claim application parity or full library acceptance.

Post-fix validation on the worker commit: `cargo nextest run -p termrock-runtime
--all-targets` passed 299 tests with 3 skipped; `cargo clippy -p termrock-runtime
--all-targets -- -D warnings`, rustfmt checks on the changed Rust files, and
`git diff --check` passed.

Review correction: the first worker commit accidentally retained a `Composite` public
variant introduced by its base, despite the original public contract having only
`Static` and `Dynamic`. This follow-up removes that variant, represents composed-table
publication with the existing dynamic identity, and changes the external consumer test
to exhaustively match exactly the original two variants. The previous three-variant
claim was incorrect and is superseded by this correction.
