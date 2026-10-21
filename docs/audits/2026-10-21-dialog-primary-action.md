# Dialog prompt primary action audit

## Finding

`Dialog::primary(key)` styled the configured key, but prompt Enter used the
first enabled non-cancel action instead. Selecting a real primary action fixes
that mismatch, but treating every `TextAction::Committed` as Enter also captures
the text field's existing commit-on-blur behavior. A pointer activation or Tab
therefore lost the value committed by the field and could submit the primary
action before the intended control handled the event.

## Resolution

Prompt Enter submission now requires the input's configured `TextCmd::Commit`
binding and no simultaneous `FocusOut` intent. The prompt keeps the shared
`TextInput` commit-on-blur policy, so Tab and pointer focus changes preserve
the draft without submitting. The configured primary key is resolved only
inside the effective action slice and must be enabled; invalid or missing
keys fail closed. Other focused actions, cancel, and pointer actions continue
to return their own keys.

## Evidence

- `crates/termrock-overlays/tests/dialog_primary.rs` drives the public dialog
  through the runtime harness. Its nine cases cover configured/reordered
  primary actions, fail-closed states, explicit cancel primary, focused
  alternatives, chords, pointer actions, Unicode draft preservation on Tab
  and pointer activation, dismissal, and repeated open/close.
- Before this resolution, the two added value-preservation tests failed with
  the Unicode draft reduced to an empty string. Afterward all nine focused
  tests and all 165 `termrock-overlays` tests passed.
- `crates/termrock-overlays/examples/dialog_primary_consumer.rs` compiles a
  consumer using only the public `Dialog`, `Action`, and `ActionKey` APIs.
- `cargo clippy --locked -p termrock-overlays --all-targets --no-deps -- -D warnings`,
  `cargo fmt --all -- --check`, and the public-consumer example check passed.

This is a shared component correction; no app-specific workaround or visual
reference artifact is involved.
