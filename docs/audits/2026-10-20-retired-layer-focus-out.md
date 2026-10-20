# Retired layer focus-out audit

## Finding

`Runtime::apply_staged_focus` can settle a focus transition after an input path has already closed the layer that owned the previous control. The last published focus ring still identifies that control, but the application's update phase may have stopped visiting it because the layer is closed. Enqueuing `FocusOut` then produces `Diagnostic::UndeliveredIntent` for a control that cannot receive the event.

## Change

Before enqueuing `FocusOut`, the runtime now checks the old control's published `LayerId`. It skips the event only when that entry belongs to a non-page layer whose exact `LayerId` is absent from the live layer stack. This policy is shared by immediate staged transitions and transitions deferred through `settle`; the latter retain the originating `LayerId` across publication, before the prior focus ring is replaced. Page blur and blur from a still-live layer remain delivered. Layer IDs advance on every open, so reopening the same layer owner does not make an old entry appear live. Focus-in, restoration order, and close events remain unchanged.

The runtime regressions model an app that drains controls only while their layer is open. They cover page blur on open and reopen, one focus-out between controls in a live layer, outside-click closure without a stale `UndeliveredIntent`, reopening the same layer owner under a new `LayerId`, and publication followed by settle when a non-restoring modal closes itself. The latter test failed before the settle-path guard with `UndeliveredIntent { owner: retired-focus-control }` and passed after.

Two existing `focus_restoration` assertions expected `FocusOut` to be delivered to a layer control after that layer had closed. They now assert that the still-live page blur remains delivered and that the closed modal or child receives no stale focus-out. Restoration and diagnostics assertions remain.

The real TablePro deletion dialog regression now checks for `UndeliveredIntent` immediately after Escape and directly after outside `MouseDown`, before `MouseUp`. It preserves cancel behavior, restored focus, frame/cursor identity, and reopen checks. It failed before the runtime fix with `UndeliveredIntent { owner: tablepro.delete-connection ▸ #0 }` and passed after.

An independent review found the same retired-layer case can be deferred through publication when a modal uses `.restore_focus(false)`. `PendingFocus` now retains the old ring entry's exact `LayerId` across publication, and `settle` applies the same liveness policy as immediate staged focus. A regression opens a real non-restoring modal, closes it from its focused control, draws the replacement page, and checkpoints after settle. It failed without the settle guard with `UndeliveredIntent { owner: retired-focus-control }` and passes with it.

## Validation

After the settle-path regression was added, `cargo nextest run --locked -p termrock-runtime` passed 315 tests with 3 skipped. The focused TablePro test previously passed 1 test with no skips. Scoped `rustfmt --edition 2024 --check crates/termrock-runtime/src/runtime.rs` passed. Runtime-only strict clippy passed on the prior code shape; the latest settle-path change is covered by the full runtime nextest run. Combined strict clippy for runtime and TablePro failed on `clippy::large_enum_variant` at `crates/tablepro-domain/src/tabs.rs:315` (`Tab::Query(QueryTab)`, 1936 bytes vs `TableTab`, 1472 bytes); this is outside the assigned files and unrelated to this focus change. Independent exact-commit review is pending. Workspace `cargo fmt --check` reports existing formatting differences outside the assigned files.
