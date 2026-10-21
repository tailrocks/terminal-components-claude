# FormDialog live visibility sizing

Status: implemented on `wip/forms-dynamic-dialog-resize-20261021`; not yet independently reviewed or integrated into PR #38.

## Finding

`FormDialog::update` measured visible form rows and constructed the `Dialog` before calling `Form::update`. A field update may mutate caller-owned view data that changes `FormData::visible`. On a tunnel checkbox transition from hidden to visible, the current event therefore updated the form but resized the modal using the previous field inventory. Drawing then rendered the newly visible chooser in the prior layer bounds. The next pointer input resolved against that stale geometry and the resize/recenter could move the chooser between mouse Down and Up.

This matches the independent real Showcase red reproduction for `dialog_kinds_add_connection_uses_live_form_and_stacked_picker` (1 failed, 0 passed, 48 filtered; log `/Users/donbeave/Projects/termrock-review-targets/form-reveal-f4db-20261021/nextest.log`). No API addition, app-specific workaround, extra settling input, or frozen modal geometry is involved.

## Change

`FormDialog::update` now updates the child form first, then measures the resulting live visible fields and constructs the dialog for that same event. Existing first-refusal Escape behavior, child Chose precedence over dialog submit, and dialog dismissal handling are preserved.

The new real-runtime regression toggles a conditional chooser from hidden to visible by pointer, commits the frame after each input, verifies the intrinsic dialog expands immediately after release, then checks chooser geometry remains stable through Down/Tick/Up and exactly one `Chose` action is returned. It also toggles visibility off and on by keyboard and verifies the layer shrinks and re-expands.

## Evidence

- Pre-fix focused regression: failed because same-event visibility did not expand the intrinsic layer. Log: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/form-dialog-intrinsic-red.log`; SHA-256 `b692bd25696ef3b20d1367df224b0682bfb70a5b827eb5edbc160bd68c6721cd`.
- Post-fix focused regression: passed (`cargo nextest run -p termrock-forms --lib intrinsic_visibility_changes_resize_before_next_pointer_input --no-fail-fast`).
- Forms package full suite: passed, 82/82 (`cargo nextest run -p termrock-forms --all-targets --no-fail-fast`).
- The independent app red remains to be rerun against a combined app+library worktree with exact source bindings before this change is integrated.
- Strict package Clippy (`cargo clippy -p termrock-forms --all-targets -- -D warnings`) passed; workspace formatting (`cargo fmt --check`) passed. Exact-reference review, independent review, and remote verification remain pending.

## Scope and references

No frozen reference revision changed. The frozen source establishes same-event conditional expansion in `src/bin/showcase/pages/dialog_kinds.rs` (lines 289–300) and the legacy form-dialog implementation in `src/widgets/form_dialog.rs` (lines 793–806). PR #26 and the `visual-baseline` tag remain read-only.

## Combined Showcase check

A separate diagnostic worktree merged app restoration checkpoint `f4db11270438c9e3a4059561af27bb60e82e346b` with this Forms WIP at `4399a54e293376ff3b318d38459a208db321bb6e` (combined merge commit `19e626de813fa74872ba8eaf13d5df7f6b548dc`; this source is not a delivery branch). `cargo nextest run -p showcase-ui --test app_tests -E 'test(dialog_kinds_)' --no-fail-fast` reports 1 pass / 2 fail. The chooser case shows the modal and chooser hit rectangle stable through Down/Tick and a nested `SSH key` picker in the rendered frame after Up; the hit rectangle then disappears under the overlay. Its current test expects `Choose SSH key`, but frozen source `src/bin/showcase/pages/dialog_kinds.rs` lines 272–275 constructs `Picker::new(KEYS, "SSH key")`; this is an incorrect test expectation, not a reference product title. The separate cancel test remains red for missing visible `Form · Cancel`, which frozen lines 329–336 explicitly logs, so that route result remains incomplete. Combined log `/Users/donbeave/Projects/termrock-cli-evidence-20261021/form-dialog-combined-showcase.log`, SHA-256 `3f1c292b10c64ae3e475d875d7d6aae1a773f113c501707e151189f3ce11c508`. Do not weaken or omit either app test; the test owner must correct the frozen-title assertion with independent review and separately fix/verify the cancel-result flow.
