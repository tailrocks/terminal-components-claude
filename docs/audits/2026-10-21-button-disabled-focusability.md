# Button Disabled focusability policy

The independent review of `6509b91e721057b9d5ec4f5c998ded752f29066` found that `Focusability::Disabled` removed a Button from the focus ring but did not prevent `Button::update` from accepting an Enter binding or pointer click already routed from the previous frame. The unchanged external probes reproduced both outcomes: each reported one activation where zero was expected. `Focusable` remains the Button default. `ClickOnly` continues to reject keyboard activation while accepting pointer activation.

Button now resolves one effective disabled state from `.disabled(true)` or `Focusability::Disabled`. Update uses it to reject keyboard and pointer activation. Autofocus skips Disabled policy. Draw registers the control as disabled, publishes no activation bindings, and removes hover, pressed, focused, and focus-visible state flags before resolving its parts. The rendered hit area remains available so pointer routing can finish without activation.

Regression checks use the real Button and Harness. The application changes its policy inside `App::update` after observing the already-routed Enter binding or pointer click, before `Button::update` consumes that intent. A separate probe checks Disabled autofocus. A ClickOnly probe verifies that the same in-flight update still permits pointer activation and rejects keyboard activation.

Validation on the author worktree:

- Before the implementation change, both new Disabled activation probes failed: 1 activation was observed instead of 0.
- `cargo nextest run -p termrock-controls --test button_contract --no-fail-fast`: 16 passed, 0 skipped.
- The independent unchanged external suite (`native_contract.rs` and `policy_edges.rs`, copied byte-for-byte for execution): 19 passed, 0 skipped. Its two prior failures now pass.
- `cargo nextest run -p termrock-controls --all-targets --no-fail-fast`: 149 passed, 0 skipped.
- `cargo clippy -p termrock-controls --all-targets --offline -- -D warnings`: passed.
- `rustfmt --edition 2024 --check` on the changed Rust files and `git diff --check`: passed.

This closes the Button policy finding only. It does not establish Showcase catalog coverage, Dialog adoption, full application parity, or completion of the larger Termrock migration. Independent review of this correction remains required.
