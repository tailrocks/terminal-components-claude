# VIS-04 claim-pin harness controls: actual v2 evidence

Prepared in external staging; publication and archive review are recorded separately.

Evidence state: frozen record of actual run evidence.

## Scope and result

This archive records the Rust Nextest harness-control lane for the deferred helper, target termrock-visibility-tests::deferred. It contains the preserved v1 failure and the v2 rerun. It does not claim execution or qualification of the separate 23 product assertions.

- v1 run 2026-10-09T05-06-26-032Z-7d60e16b: 29 selected, 27 passed and 2 failed; Nextest exit 100. The failures were claim_pin_rejects_obsolete_or_mismatched_identity_before_tool_setup and claim_registry_rejects_non_integer_revisions_before_tool_setup. Its raw stdout, stderr, and receipt are preserved unchanged.
- v2 run 2026-10-09T05-13-58-526Z-adb32ee0: PASSED_29, 29 passed, zero skipped, exit 0, one matching Nextest run ID, no timeout/forced close/parse errors. Its raw stdout, stderr, and receipt are preserved unchanged.
- The v2 test-only follow-up treats a missing synthetic trace file as an empty trace and continues to fail on other I/O errors. The production claim-pin behavior is otherwise the same candidate. This follows the v1 failures and is pinned by the included diff and source snapshot.
- Independent actual-result review: READY; exact review JSON is included. Static implementation review: READY; exact review JSON is included.

The separate S1d product23 execution remains 22 PASS / 1 FAIL (BD-21, w13_filter_wide_trail_cells_clear). It is outside this archive's control lane and is not changed or rerun here.

## Inputs and verification

The source snapshot contains the 14 files listed in the v2 source manifest. Each copied source file was rehashed against that manifest before staging. The run receipt binds the source-manifest, runplan, runner, launch script, preflight, and 29-name selection hashes. The v2 raw event stream and stderr are also covered by the actual-review receipt and by MANIFEST.json.

Test toolchain: Rust 1.98.1, cargo-nextest 0.9.146, build jobs 2, test threads 1, offline mode. No Python test suite was run. The Rust fixture tests invoke the production helper CLI.

Published source commit: ed626a6b919fc91c303d1dcdafb11b5a2f200754. Its tools/visibility/deferred.py and crates/termrock-visibility-tests/tests/deferred.rs blobs match the actual v2-tested SHA-256 values recorded in the source manifest. This archive does not edit source or queue data.

## Artifact inventory

MANIFEST.json lists each staged file (except itself) with its exact byte count, SHA-256, and original path. The manifest's own SHA-256 is reported separately with the independent review handoff. Original receipt and log bytes are copied without transformation; no receipt or test output was generated for this archive.
