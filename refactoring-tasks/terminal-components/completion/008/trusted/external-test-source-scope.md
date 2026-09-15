# External test-module source scope (TASK-008 disposition targets)

Source pin: `7b27732a8c3c131760ec3438f641cb3c11343a42`. Complements `/task/trusted/inline-test-source-scope.md`, which covers `src/` inline tests only. TASK-008 may change paths listed here only within independently approved exact test assertion spans bound by `/task/trusted/branch-test-disposition-bindings.tsv`. No production mutation.

TASK-007 must reconcile this seed with parser discovery and actual compiled listings. Paths marked `future-task` are authorized write targets for span patches once the owning completion test module exists.

## Exact external test paths (components-B disposition)

- `crates/tui/tests/viewport.rs` — W-021-07
- `crates/tui/tests/completion_021.rs` — W-021-07 (future-task; TASK-021 writable scope)
- `crates/tui/tests/keyboard_editor.rs` — W-025-07, W-025-08
- `crates/tui/tests/completion_025.rs` — W-025-07, W-025-08 (future-task; TASK-025 writable scope)
- `crates/tui/tests/completion_026.rs` — W-026-05 (future-task; TASK-026 writable scope)
- `crates/tui/tests/*diff*` — W-026-05; TASK-007 resolves exact basename matches at dispatch (pinned-main diff review tests under `crates/tui/tests/` whose path contains `diff`)

Inline component tests referenced as `M:…` in the disposition TSV remain governed by `/task/trusted/inline-test-source-scope.md`.
