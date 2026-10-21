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
- Strict package Clippy, formatting, exact-reference review, PR #38 integration, and remote verification remain pending.

## Scope and references

No frozen reference revision changed. The frozen source establishes same-event conditional expansion in `src/bin/showcase/pages/dialog_kinds.rs` (lines 289–300) and the legacy form-dialog implementation in `src/widgets/form_dialog.rs` (lines 793–806). PR #26 and the `visual-baseline` tag remain read-only.
