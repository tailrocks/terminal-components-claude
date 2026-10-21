# Action variant builder

`termrock_core::Action::variant()` remains the read accessor. `Action::with_variant(Variant)` provides an additive, const-capable builder for action presentation. It changes only the variant field; action key, label, chord, enabled state, and danger semantics remain owned by their existing constructors and builders.

Focused coverage verifies const primary and secondary actions, danger actions whose presentation changes, ordinary actions styled with the danger variant without acquiring danger semantics, and preservation of other action fields through builder chaining.

Validation passed: `rustup run 1.99.1 cargo fmt --check`; `rustup run 1.99.1 cargo nextest run -p termrock-core --lib` (98 passed, 0 skipped); `rustup run 1.99.1 cargo clippy -p termrock-core --all-targets -- -D warnings`; `git diff --check`. Independent review and application integration are pending. This API enables Showcase's frozen Test secondary and Save primary presentation through shared form actions; it does not itself establish application parity.
