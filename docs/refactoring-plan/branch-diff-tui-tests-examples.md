# Whole branch-delta audit — TUI tests and examples

Status: IN PROGRESS. Only ledger rows with explicit full read coverage receive semantic credit. No build/test execution during the concurrent timing measurement.

Scope agreed with closure: all112 changed tooling-tests paths under crates/tui/tests/ and crates/tui/examples/. Closure owns the complement, including crates/tui-testing and root tests/examples. All112 are main additions, absent at Holla. Exact ls-tree bindings verified for main7b27732a8c3c131760ec3438f641cb3c11343a42 and Holla2e2401393c47360741ebd321679de08982dca50a. Metadata totals33949 main lines, not a semantic-read claim. Immutable UI oracle remains02f5294bfdbf38004cc49130d0aff1d01f31434c; added main tests are architectural evidence, not oracle output.

Companion ledger records full path/blob/line/SHA, actual read coverage, disposition and future owner/proof. No production, canonical task, history or baseline edits. Every unchanged or conflicting assertion remains subject to exact TASK-007/008 source-test identity/disposition; do not delete whole tests because one expectation is wrong.

## Early findings requiring source-qualified task proof

- TTE-01: examples/09_composed_dialog.rs:57–87 and10_nested_overlay.rs:59–116 configure action arrays only during update; draw reconstructs Dialog without actions. Empty main and no interaction tests hide missing action rows and unequal phase props. TASK-065 must execute phase-equivalent example trajectories through TASK-023 public components, not equate compile success with working composition.
- TTE-02: examples/11_small_app.rs:38–40,67–72,92–96 keys roster entries by text but permits duplicate names. Add Alice twice, then remove keyed Alice: both entries share identity and removal deletes both. Example owner must allocate durable unique record keys; TASK-020 shared keyed identity must not be weakened to accommodate duplicates. TASK-065 owns example integration proof.
- TTE-03: examples/12_author_component.rs:174–218 does not reconcile cursor/selected or bounds-check pointer indexes against replacement labels. Cursor2 with three labels, rebuild same Id with one label, then Select emits index2. Style/reference-only tests cannot detect this. Preserve author-only imports and actual painting while requiring bounded current-source target validation and state replacement trajectories in TASK-065/031; runtime registration alone does not validate mutable source applicability.

- TTE-04: examples/13_connection_form.rs:244–276 configures submit(SAVE) but directly calls save(true) for SAVE_CONNECT. Pinned main Form::update:1310–1314 invokes submit_form only for the configured submit key; other keys emit ordinary Action. Idle invalid NAME/PostgreSQL DB followed by Save&connect bypasses total validation. An independent FocusOut may commit one child; this is not proof of total submit validation. Keep generic non-submit action semantics; the example must explicitly route its second submitting action through an accepted validated path.

Current example write ownership is TASK-068 only (verify.toml includes crates/tui/examples); TASK-065's writable set is xtask/source fixtures only and it precedes068. Thus the example corrections above need an earlier explicit owner/scope or reconciled ordering before strengthened065 phase/identity checks can close. References to065 in the findings mean enforcement/proof ownership, not existing authority to edit examples. No canonical repair made here.

These findings are source inspection, not executed candidate failures. Full source read and classification continue; no claim of complete112-path review yet.
