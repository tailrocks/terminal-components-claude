# FormDialog chooser activation audit (2026-10-21)

## Decision

A focused chooser is a child form control. Enter and Space activation must produce `FormAction::Chose` / `FormDialogAction::Chose` before the enclosing form or dialog interprets the same key as Save. The default `EnterPolicy::SubmitsWhenIdle` remains unchanged for non-chooser fields, and successful Save/cancel behavior is unchanged.

`Form::update` previously tested for idle Enter submission before updating its fields, so a focused chooser never received Enter. `FormDialog::update` then updated the dialog even if the form had produced a child chooser action. The fix excludes chooser fields from the form's idle-submit request and gives a `Chose` response first refusal over the enclosing dialog for that input frame.

## Evidence

- `crates/termrock-forms/src/form.rs`: focused matrix regression verifies Enter remains Save for select, radio, chips, checkbox, toggle and other idle controls, while a chooser returns `Chose`; a separate assertion verifies Space activates a chooser.
- `crates/termrock-forms/src/form_dialog.rs`: a public-widget regression verifies Enter, Space and pointer activation all return `FormDialogAction::Chose` and leave the dialog open.
- Focused command: `cargo nextest run -p termrock-forms --lib --no-capture -E 'test(enter_submits_only_when_the_focused_control_is_not_editing)'` — passed.
- Focused command: `cargo nextest run -p termrock-forms --lib --no-capture -E 'test(chooser_enter_and_space_are_returned_before_dialog_submit)'` — passed.

## Application integration status

The Showcase Add connection page is a separate, incomplete restoration. The real `showcase-ui` harness displays the chooser control but clicking its current field ID does not return a chooser action, while the public standalone FormDialog regression does. The app-level regression intentionally asserts the Picker's item data and remains red until this app integration issue is resolved. No synthetic click handling, app repaint, or fake Picker content has been added. This audit does not claim Showcase screen parity or completion.

## Regression sensitivity and unrelated baseline

- Red-before proof: with the two Form chooser exclusions and FormDialog child-action guard temporarily removed, the new widget test failed at `Some(Enter) submitted the dialog`; the production implementation was restored afterward.
- `cargo nextest run -p termrock-forms --all-targets` — 78 passed, 0 skipped.
- `cargo clippy -p termrock-forms --all-targets -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `cargo nextest run -p termrock-overlays --all-targets` — existing unrelated failure `menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence` (expected `Rgb(7, 8, 9)`, got `Rgb(28, 28, 31)`); run stopped after 73/174 tests, with 72 passed. This change does not edit overlay code.
