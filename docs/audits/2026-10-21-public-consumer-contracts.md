# Public facade consumer contracts

Source base: `168d85a3f71acbaf0cce33efd4fe8a515b08db16`.

`crates/termrock/tests/public_component_contracts.rs` is an integration target
compiled as a downstream-style consumer of the public `termrock` facade. It
imports no application crate, component implementation crate, or
`termrock::author` API. Ratatui supplies the root terminal area and buffer.

The three checks exercise these public contracts with consumer-owned data:

- `Button::focusability(Focusability::ClickOnly)` excludes keyboard focus
  while the same rendered button still activates from pointer release.
- `PropsList::sheet()` composes in a column with borrowed Unicode rows,
  wrapping, and a keyed row region.
- `Picker::selection_policy(SelectionPolicy::FirstEnabledOnFilterChange)`
  routes `BackTab` into a second scope, skips its disabled first item, and
  retains a stable key when caller data is reordered. Empty data clears the
  cursor safely.

The Velnor 0.1.32 task manifest discovers package test targets dynamically:
the `termrock/nextest/testing` task runs `cargo nextest run` for the package
with the `testing` feature and `--no-tests fail`. The explicit Cargo test
target is therefore included by that existing task. No generated workflow or
Velnor configuration change is needed.

Validation on this source base:

```text
cargo nextest run -p termrock --test public_component_contracts --features testing
3 passed, 0 failed, 0 skipped
cargo fmt --check --all
passed
cargo clippy -p termrock --test public_component_contracts --features testing --no-deps -- -D warnings
passed
```

This fixture proves public API compilation and selected shared-component
contracts only. It does not prove that the four applications adopt these
contracts, that Showcase has a capability-coverage gate, or that any frozen
application frame or flow matches. Those acceptance checks remain separate.
