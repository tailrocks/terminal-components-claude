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

## Independent review (exact source commit `9c07189c8f38a4e29bda4060f2dfd5efc8ceda73`)

The separate reviewer cleared only the shared Form/FormDialog chooser activation delta. The review found no API signature changes or rendering/measurement side effects. Its independent public consumer passed 10/10 candidate checks; the same consumer on the parent produced 2 red chooser regressions and 8 passes. Workspace Form/overlay checks completed 251/252 with 0 skips; the sole failure was the known menu overlay color assertion above. Clippy and format passed. This review explicitly does not approve the Showcase flow or claim product parity.

Evidence is committed at [`evidence/form-chooser-20261021/review.json`](evidence/form-chooser-20261021/review.json) (SHA-256 `8ea0eaf7f6d736c95fc3821033377968a995ff299f0e6c79d03b6347a828c906`) and [`evidence/form-chooser-20261021/review-notes.md`](evidence/form-chooser-20261021/review-notes.md).

| Contract | Retain / replace | Evidence and remaining scope |
|---|---|---|
| Form idle Enter submit | Retain for ordinary fields; exclude only chooser fields so the chooser receives activation first. | Form matrix regression preserves Save for other idle fields and asserts chooser Enter returns `Chose`. |
| FormDialog child actions | Replace same-frame dialog submit after a chooser `Chose` with child-first response handling. | Widget test covers Enter, Space, pointer; dialog remains open. |
| Public API | Retain typed `FormAction::Chose(Id)` / `FormDialogAction::Chose(Id)` and existing FormData ownership. | Independent consumer passed; no signature change. |
| Showcase nested Picker | Still unverified / failing in the actual Showcase harness. | App test currently observes no chooser action from pointer input; do not claim this route or app parity is complete. |
| Menu overlay color | Known unrelated failure; unchanged and not waived. | Full reviewer run: 251/252; 0 skips. |
