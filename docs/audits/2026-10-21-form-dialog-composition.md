# Reusable FormDialog composition (2026-10-21)

## Decision and frozen-source evidence

The frozen `visual-baseline` source at `4169783ae883788186d161af09ba9c481bd84e5c` defines the product contract in `src/widgets/form_dialog.rs`: default Escape dismissal, first visible focusable field (Save fallback), one form-owned action row, validated Save, Cancel, optional custom actions, Unicode field values, and keep-open-on-save. This change retains that ownership: Termrock `Form` owns field state, validation, scrolling, action buttons and typed field/action results; `Dialog` owns modal chrome, layer scope and dismissal. `FormDialog` composes the two and adapts their typed results without owning field values or another action/scroll state.

The existing `Dialog::new` layout intentionally remains unchanged. The additive `Dialog::chrome` mode is for nested component composition: when it has no actions it reserves no synthetic footer row, and it does not register the body as a focusable control. A normal Dialog with configured actions keeps its existing action layout. This resolves the composed form's extra blank footer while leaving standalone Dialog defaults intact.

## API and behavior

- `FormDialog::layer` builds an ESC-only layer with first-visible-field focus and sizes its body from live `FormData` via the same `Form::measure_with_data` algorithm, parameterized by the frame's design tokens.
- `FormDialog::update` updates Form before Dialog so field-local Escape (editor/select) keeps first refusal. The runtime's next ESC dismisses the layer; outside-click dismissal remains disabled by default. Submit validation remains Form-owned. `KeepOpen` leaves successful domain acceptance to the caller.
- `FormDialog::draw` delegates chrome to Dialog and the resolved body slot to Form. Application geometry and cell painting are not exposed in the consumer example.
- State composes existing `FormState` and `DialogState`; data remains caller-owned. Results are typed (`Changed`, `Committed`, `Chose`, `Action`, `Invalid`, `Cancelled`).
- `form_dialog_consumer.rs` is a different-data consumer through the `termrock` facade and uses `LayerSlot` composition.

## Focused evidence

`form_dialog.rs` runtime tests cover validation preventing acceptance, outside-click refusal, field Escape before modal Escape, live note-row-driven dialog measurement with Unicode data, and ESC-only/initial-focus layer configuration. The Dialog runtime regression test confirms an empty chrome footer removes one row while ordinary empty-action Dialog geometry retains its prior extra row, and the composed BODY is decorative rather than a focusable control.

Before the WIP commit, `cargo nextest run -p termrock-forms --lib` passed 61/61; the focused Dialog chrome nextest passed 1/1; `cargo check -p termrock --example form_dialog_consumer`, scoped strict Clippy for `termrock-overlays`, `termrock-forms`, and `termrock` (all targets), `cargo fmt --all --check`, and `git diff --check` passed. A broader `termrock-overlays --lib` run passed 84/85 and exposed the unrelated MenuBar failure below.

## Status and limits

This is a reusable component increment, not application migration or exact product parity evidence. No application has adopted FormDialog yet. The separate consumer compile and focused Rust checks are recorded in the change's verification; the complete four-application reference comparison and application adoption remain outstanding and must be independently reviewed before acceptance. No frozen artifact was changed.

The full `termrock-overlays` lib run currently has an unrelated failure in `menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence` (`menu.rs:1746`, expected local patch background `Rgb(7, 8, 9)`, observed overlay background `Rgb(28, 28, 31)`). This change does not edit menu code; the focused Dialog regression passes. The failure remains unresolved and is not attributed as pre-existing without a baseline run.

Showcase/catalog coverage for the new public `Dialog::chrome` option is not included in this worktree; project instructions require that follow-up before the API is accepted. No app migration or paired visual comparison was run.

## Post-merge receipt

FormDialog implementation commit: `4f12c155c`. Integrated latest PR #38 remote tip `52e1572ca` with merge commit `62f500b4a`. On the merged tree, `cargo nextest run -p termrock-overlays -p termrock-forms --lib` reported 145/146 (the single MenuBar failure above); all 61 Form tests passed. The focused chrome test passed, the standalone `termrock` facade example compiled, strict scoped Clippy passed for overlays/forms/facade all targets, formatting and diff checks passed. This receipt is incomplete for acceptance because of the unrelated test failure and missing showcase/catalog coverage.

## Opt-in pointer close affordance

Added `Dialog::close_button(ActionKey)` as an opt-in typed cancel action. Its control is a shared `Button` with `Focusability::ClickOnly`, drawn and hit-tested on the same three-cell rectangle at the existing frame offset. It is shown only at frame width >=16 and height >=4; the default Dialog and chrome-only Dialog keep their former decorative mark and no close hit region. FormDialog passes its configured cancel key, then closes through the existing modal lifecycle so nested field Escape handling remains first and focus restoration stays with the overlay runtime.

Evidence: FormDialog runtime tests verify hover and pressed frame changes, pointer-only focus behavior, one `Cancelled` outcome on click, no repeated cancellation after close, and absence of the hit region below width 16. The chrome-only default regression asserts no close hit region. Focused command `cargo nextest run -p termrock-overlays -p termrock-forms chrome_mode_omits_empty_action_row_and_does_not_focus_the_body pointer_close --lib` passed 3/3. These tests do not replace showcase coverage or paired application parity verification.

## Verification update (supersedes the earlier post-merge test count)

After adding the opt-in close affordance, `cargo nextest run -p termrock-overlays -p termrock-forms --lib` ran 148 tests: 147 passed and the same single MenuBar local-patch precedence test failed (`menu.rs:1746`). The focused close/default-chrome group passed 3/3; FormDialog now has 63 tests. Scoped strict Clippy, the public facade example check, formatting, and diff checks passed on the updated source. The inherited MenuBar failure remains recorded as unresolved; the independent reviewer HOLD prevents importing its unrelated proposed fix.
