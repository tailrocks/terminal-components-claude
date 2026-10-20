# Retired layer focus-out audit

## Finding

`Runtime::apply_staged_focus` can settle a focus transition after an input path has already closed the layer that owned the previous control. The last published focus ring still identifies that control, but the application's update phase may have stopped visiting it because the layer is closed. Enqueuing `FocusOut` then produces `Diagnostic::UndeliveredIntent` for a control that cannot receive the event.

## Change

Before enqueuing `FocusOut`, the runtime now checks the old control's published `LayerId`. It skips the event only when that entry belongs to a non-page layer whose exact `LayerId` is absent from the live layer stack. Page blur and blur from a still-live layer remain delivered. Layer IDs advance on every open, so reopening the same layer owner does not make an old entry appear live. Focus-in, restoration order, and close events remain unchanged.

The runtime regression models an app that drains controls only while their layer is open. It covers page blur on open and reopen, one focus-out between controls in a live layer, outside-click closure without a stale `UndeliveredIntent`, and reopening the same layer owner under a new `LayerId`. It failed before the runtime fix with `UndeliveredIntent { owner: retired-focus-next }` and passed after.

Two existing `focus_restoration` assertions expected `FocusOut` to be delivered to a layer control after that layer had closed. They now assert that the still-live page blur remains delivered and that the closed modal or child receives no stale focus-out. Restoration and diagnostics assertions remain.

The real TablePro deletion dialog regression now checks for `UndeliveredIntent` immediately after Escape and directly after outside `MouseDown`, before `MouseUp`. It preserves cancel behavior, restored focus, frame/cursor identity, and reopen checks. It failed before the runtime fix with `UndeliveredIntent { owner: tablepro.delete-connection ▸ #0 }` and passed after.

## Validation

After the dialog stack merge, `cargo nextest run --locked -p termrock-runtime` passed 314 tests with 3 skipped. The three focused runtime and restoration cases passed. `cargo nextest run --locked -p tablepro-ui --test delete_dialog_dismissal` passed 1 test with no skips. Scoped `rustfmt --edition 2024 --check` passed for all three changed Rust files. Runtime-only strict clippy passed. Combined strict clippy for runtime and TablePro failed on `clippy::large_enum_variant` at `crates/tablepro-domain/src/tabs.rs:315` (`Tab::Query(QueryTab)`, 1936 bytes vs `TableTab`, 1472 bytes); this is outside the assigned files and unrelated to this focus change. Independent exact-commit review is pending. Workspace `cargo fmt --check` reports existing formatting differences outside the assigned files.
