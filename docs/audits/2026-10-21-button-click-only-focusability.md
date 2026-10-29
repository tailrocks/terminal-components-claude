# Button click-only focusability

`Button` now accepts the shared `Focusability` policy. Its default remains `Focusable`. A `ClickOnly` button keeps its rendered hit target and pointer activation while it leaves the keyboard ring, publishes no keyboard activation bindings, ignores stale binding intents, skips autofocus, and suppresses focus-only style flags. This lets a caller render a real pointer control without creating a keyboard stop.

Regression coverage changes a button from `Focusable` to `ClickOnly` after it owns keyboard focus. The test checks ring removal, focus-marker removal, Enter/Space suppression, press without activation, and pointer release activation. Existing Button contract coverage also verifies unchanged default focus behavior.

Validation on the author branch:

- `cargo nextest run -p termrock-controls --test button_contract`: 12 passed, 0 skipped.
- `cargo clippy -p termrock-controls --all-targets -- -D warnings`: passed.
- `cargo fmt --check` and `git diff --check`: passed.
- Behavioral negative control: temporarily hard-coded Button registration to `Focusable`; the new regression failed at the ring-removal assertion. Restored the source and reran the full focused test binary successfully.

This increment proves the shared control contract only. It does not claim Dialog chrome adoption or application parity.
