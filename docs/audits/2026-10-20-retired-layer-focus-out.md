# Retired layer focus-out audit

## Finding

`Runtime::apply_staged_focus` can settle a focus transition after an input path has already closed the layer that owned the previous control. The last published focus ring still identifies that control, but the application's update phase may have stopped visiting it because the layer is closed. Enqueuing `FocusOut` then produces `Diagnostic::UndeliveredIntent` for a control that cannot receive the event.

## Change

Before enqueuing `FocusOut`, the runtime now checks the old control's published `LayerId`. It skips the event only when that entry belongs to a non-page layer whose exact `LayerId` is absent from the live layer stack. Page blur and blur from a still-live layer remain delivered. Layer IDs advance on every open, so reopening the same layer owner does not make an old entry appear live. Focus-in, restoration order, and close events remain unchanged.

The regression models an app that drains controls only while their layer is open. It covers page blur on open and reopen, one focus-out between controls in a live layer, outside-click closure without a stale `UndeliveredIntent`, and reopening the same layer owner under a new `LayerId`. The test failed before the runtime fix with `UndeliveredIntent { owner: retired-focus-next }`; it passed after the fix.

Two existing `focus_restoration` assertions expected `FocusOut` to be delivered to a layer control after that layer had closed. They now assert that the still-live page blur remains delivered and that the closed modal or child receives no stale focus-out. Restoration and diagnostics assertions remain.

## Validation

`cargo nextest run --locked -p termrock-runtime -E 'test(/removed_disabled_and_reparented_openers_use_published_eligibility/) | test(/nested_close_restores_only_inside_surviving_modal/) | test(/focus_out_is_delivered_once_while_live_and_skipped_after_layer_close/)'` passed all 3 selected tests after the fix. The first test run reproduced the stale-intent diagnostic with the runtime fix temporarily removed.

Full `termrock-runtime` nextest run had 310 passes, 2 failures, and 3 skips before updating the two now-obsolete assertions. Full rerun and independent review are pending. Workspace `cargo fmt --check` also reports existing formatting differences outside the assigned runtime files; scoped formatting and lint results are pending.
