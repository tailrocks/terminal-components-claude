# VIS-04 deferred harness controls, b106 v7

## Result

The bounded Rust/Nextest harness-control lane **PASSED**: 27 selected tests passed, none were skipped, and the runner exited 0. Nextest run ID: `e2844649-cddf-4afa-9b9c-4eff48edf3ae`. Wrapper run ID: `2026-10-09T04-10-28-951Z-2fccc5cd`.

This result qualifies the deferred runner's Rust harness controls only. It does not qualify the product's 23 deferred assertions, the production `deferred.py run` command, paired PTY behavior, or product completion. The separate S1d product assertion run is archived at [product23-s1d-20261009](../product23-s1d-20261009/README.md) and recorded 22 PASS / 1 FAIL (BD-21, `w13_filter_wide_trail_cells_clear`). The harness control receipt's registry metadata remains NOT_RUN for that control-lane scope; it does not change or restate the product23 result.

The 27 selected identities are in `provenance/test-names.json`. The receipt and independent actual-result review reconcile all 27 exact identities to one start/ok pair each, one ordered suite start/success pair, one Nextest run ID, and 27 pass / 0 skip. No expected-failure inversion was configured. The actual review confirms exit 0, no timeout, forced close, parse errors, or log errors, and both raw logs within the recorded cap.

## Frozen inputs and review

The source manifest pins 14 files. It identifies candidate product commit `1d797d41c8141fcbdc3f69d7f11eb8875ab54712`, fixture base `b106b2426cfce016381c58b3d380e762a454c7a6`, and the original source worktree observation at `766ae1e925e32b4b28aa9100bb3fde5279aa223b`. The copied `source/` tree preserves the bytes listed in that manifest; the mixed-stage worktree fixture was excluded in favor of the committed b106 fixture.

The raw receipt is `runs/2026-10-09T04-10-28-951Z-2fccc5cd/receipt.json` (SHA-256 `62e96370d3bb97bd83df72171ffc9d5654405fc0e522786a2583923e8c7e8980`). Its byte-preserved Nextest JSONL is `nextest.stdout.jsonl` (SHA-256 `ef00d1a85c62809ea0822b35bc7c592d6952b4f81eda0c6e0b572f4be898b5b2`); stderr is `nextest.stderr.log` (SHA-256 `98db926a1ec58014c87b707bcfe49a6dd54ee8b0b7764b9b136dffbb3892147f`).

The independent static review is in `reviews/static-review.json` (SHA-256 `6f87efa80d42dab14f1c736b96faf5f5c397ba09dddc7165c6041dc16bb33b80`). The independent actual-result review is in `reviews/actual-review.json` (SHA-256 `2ea0fd32c9f541425fcf4045e5eb0501fb7d670f33f5069753b9dadea57abcd4`). Both reviews are lane-scoped; the actual review explicitly retains the separate product23 result.

`provenance/` retains the source manifest, run plan, names, runner and launch scripts, both preflight records, cache manifest, prelaunch audit, package inventory, and original review request. `SHA256SUMS.json` lists the relative paths, byte lengths, and SHA-256 values for all archived files except itself.

## Portability limits

The archived source snapshot and raw evidence are locally verifiable, but this is not an offline replay bundle. The reviewed run used absolute macOS paths, Rust 1.98.1, cargo-nextest 0.9.146, Node 24.20.0, an external 32-package Cargo archive/index cache, and a compiled target directory. The cache manifest records the external cache paths and digests; the crate archives, extracted registry sources, and compiled target are not copied here. The archived launch script retains host-specific paths and must not be invoked from this evidence directory.

The raw run was driven by the frozen external Node supervisor in `provenance/run-controls.cjs`; this archive does not claim an end-to-end `tools/visibility/deferred.py run` collector receipt. The production helper's selection, parsing, classification, and receipt behavior is covered by the 27 Rust control tests, while actual product assertions are separately represented by the direct 23-test S1d run linked above.
