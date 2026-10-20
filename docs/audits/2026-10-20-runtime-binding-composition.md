# Runtime binding composition audit (2026-10-20)

## Finding and decision

Picker query composition needs a text editor and a result list to publish controls under one real focus owner. Ordinary component binding publication replaces the previous owner table, so sequential child publication loses whichever table was published first. Changing `publish_bindings` globally would alter existing replacement behavior and could make duplicate owner registrations silently accumulate.

Added an explicit author API in `termrock_runtime::author`:

- `publish_bindings_filtered` retains replacement semantics while selecting typed actions from a static table.
- `extend_bindings` appends explicitly selected bindings to an already-published same-owner table for the current frame.
- `publish_bindings_filtered` publishes an explicitly selected subset while preserving normal replacement semantics, allowing a widget to omit commands without copying the static table.

Extension requires a base table, rejects duplicate action IDs and ambiguous default chords, and emits the existing duplicate-action or bubble-phase binding-conflict diagnostic. Runtime key-map remapping conflicts remain visible through existing component-conflict diagnostics. Both APIs use normal focus/active-layer registration gates; extension data is frame-local and cleared by the existing registry reset. The original `publish_bindings` contract remains unchanged.

Picker caller must explicitly filter actions whose chord ownership is ambiguous (notably Enter, Esc, Home, End, and conditional Backspace/Delete); the registry does not arbitrate these by priority.

## Evidence

- Runtime registry tests cover cross-type descriptor aggregation, explicit action filtering, remapped chord conflicts, frame reset, and rejection of duplicate actions/default chords.
- UI author adapter test verifies the public extension path retains both the base and extension actions under the same owner.
- `cargo nextest run -p termrock-runtime --lib`: 182 tests passed (run ID `282f4833-9d4b-4f06-8a5b-7e45fd5719a8`).
- `cargo clippy -p termrock-runtime --lib -- -D warnings`: passed after fixing a `collapsible_if` lint found on the first run.
- `rustfmt --edition 2024` on the three changed Rust files: passed.
- Workspace `cargo fmt --check` has unrelated existing formatting differences in `crates/showcase-demos/src/pages/chrome.rs`; only changed runtime files are formatted with rustfmt.

## Remaining integration

Picker is responsible for calling the extension API for its result-list action set and using filtered publication for query TextInput command exclusions. End-to-end typing, Backspace, Up/Down, remapping, and modal behavior remain covered by the Picker integration change.
