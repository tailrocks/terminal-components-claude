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
