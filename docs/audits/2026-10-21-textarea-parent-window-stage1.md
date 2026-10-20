# TextArea parent-window API, stage 1

Base commit: f5858b184954b0153f24dd9e97cdde5ab908c792

This increment adds an opt-in VerticalWindow descriptor to the runtime's
existing prepared layout facts. It maps a component's visible physical rows to
source rows in its logical body; it does not transform global coordinates,
allocate an off-screen buffer, or own scroll state. Existing layout facts keep
their prior behavior because vertical_window defaults to None.

VerticalWindow::new validates non-empty visible geometry and the visible
source-row range. VerticalWindow::full represents the existing unclipped,
zero-offset case, including zero-sized areas. Pure mapping helpers reject rows
outside the visible range and use checked arithmetic for physical coordinates.
LayoutFacts::with_vertical_window lets component authors publish the
descriptor through the existing Ui::report_layout / Cx::layout lifecycle.

Focused evidence on this commit:

- rustfmt --edition 2024 --check crates/termrock-runtime/src/author/window.rs crates/termrock-runtime/src/author.rs crates/termrock-runtime/src/ui/cx.rs passed.
- cargo nextest run -p termrock-runtime --lib passed: 226 tests, 0 skipped.
- cargo nextest run -p termrock-runtime --all-targets --features testing passed; nextest reported 348 tests and 3 skipped.
- cargo clippy -p termrock-runtime --all-targets -- --deny warnings passed.
- cargo clippy -p termrock-runtime --all-targets --features testing -- --deny warnings passed.

Index recovery note: an unpublished local commit attempt (`f761e8092`,
subsequently reset before any push) accidentally included absent paths from the
initial sparse checkout as deletions. That commit was discarded. The corrected
change contains only the four paths listed above; `git diff --cached --check`
and the staged path audit passed. A fresh sparse specification restored a clean
tracked worktree before rebasing this WIP on the current PR branch. No remote
reference was changed by the discarded attempt.

This is only the typed prepared-geometry prerequisite. It does not implement
window-aware ScrollRegion painting/input, TextArea clipping, Form parent reveal,
context-menu routing, or frozen application parity. Those remain separate
required increments and acceptance checks.

Independent review follow-up for c79dc196378d4eabf1549bc3c334d8507c97fade:
the public probe `zero_width_full_window_exposes_no_rows` failed because a
zero-width, positive-height full window still exposed rows. All mapping helpers
now treat either zero width or zero height as empty. The unchanged public probe
passes (2 passed, 0 skipped), and native tests explicitly cover both empty
shapes. `cargo nextest run -p termrock-runtime --all-targets --features testing`
passes (348 passed, 3 skipped); strict testing-feature clippy and rustfmt pass.
