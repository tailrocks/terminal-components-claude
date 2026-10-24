# Text input initial focus selection

## Decision

Fresh editable text input sessions need a reusable initial selection policy.
The frozen Showcase DialogKinds flow uses a populated input whose first typed
characters replace its initial value. In the frozen source,
`src/bin/showcase/pages/dialog_kinds.rs` builds the Add connection form with a
populated Name field, and
`src/bin/showcase/app_tests_dialog_kinds.rs::the_form_previews_discloses_tests_and_saves`
types `billing` and expects the preview to contain `billing` instead of
`orders-prod`.

`termrock-fields` now exposes `FocusSelect::{CaretStart, CaretEnd, SelectAll}`
and `TextInput::focus_select`. The compatibility default remains `CaretEnd`.
Configured selection applies only when an editable input starts a fresh draft
from `FocusVia::Keyboard`, `FocusVia::Programmatic`, or `FocusVia::Restore`.
`FocusVia::Pointer` leaves caret placement and selection to the pointer gesture.
An already active draft retains its caret and selection through `BlurPolicy::Keep`
and focus restoration. Disabled and read-only inputs do not start an edit draft.

`TextInputState::select_all` applies to the private editor for plain and secret
drafts. Public secret text remains unavailable (`draft_text() == None`), and
debug output remains redacted.

## Evidence

Focused real-runtime tests are in
`crates/termrock-fields/tests/keyboard_editor.rs`; they exercise Tab and
programmatic focus, first-key replacement, pointer placement, retained draft
selection, Unicode values, and disabled/read-only controls. Secret selection
and redaction are tested in `crates/termrock-fields/src/input.rs`.

Validation on this change:

- `rustup run 1.99.1 cargo fmt` — passed.
- `CARGO_TARGET_DIR=/private/tmp/termrock-target-text-input-focus-select-20261021 rustup run 1.99.1 cargo nextest run -p termrock-fields --all-targets --no-fail-fast` — 97 passed, 0 failed, 0 skipped after adding the restore-focus case.
- `CARGO_TARGET_DIR=/private/tmp/termrock-target-text-input-focus-select-20261021 rustup run 1.99.1 cargo clippy -p termrock-fields --all-targets -- -D warnings` — passed.
- `rustup run 1.99.1 cargo fmt --check` and `git diff --check` — passed.

The first test run exposed fixture range assumptions (`billing-orders-production`
is 25 bytes) and confirmed this is a new API contract, not an existing behavior
regression. The final tests use the actual value length. A real runtime frame
assertion verifies selected text receives its theme's selected style. No expected
visual artifact was changed. The frozen Showcase application flow still needs to adopt
the public option and pass its paired reference comparison before product parity
can be claimed.
