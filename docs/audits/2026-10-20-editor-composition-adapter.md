# Code editor composition adapters

Added borrowed `CodeEditor` and `CompletingCodeEditor` leaves to the public composition facade. Both adapters delegate intrinsic measurement and rendering to their existing widgets; composition retains placement only. `CompletingCodeEditor::measure` delegates to its wrapped editor, so completion overlays add no measurement effects or alternate sizing rules.

A public consumer regression test composes a Unicode editor and an empty completing editor in a shared row, checks resolved bounds, focuses the editor through its rendered hit region, types a wide Unicode character, and verifies cursor bounds. The app update owns widget updates; composition measurement and drawing invoke no input handlers or domain actions.

Validation: `cargo nextest run --locked -p termrock --test composition_consumer --features testing --no-fail-fast` passed 5/5, including the Unicode edit, empty completing editor, nested flexible allocation, and resize checks. `cargo clippy --locked -p termrock --test composition_consumer --features testing -- -D warnings`, targeted `rustfmt --check`, and `git diff --check` passed. No baseline artifacts, visual approvals, app code, runtime input routing, or layer behavior changed.
