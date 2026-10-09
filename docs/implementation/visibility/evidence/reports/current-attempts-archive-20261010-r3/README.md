# Current attempt evidence archive R3

This increment copies raw receipts, logs, snapshots, source-binding records, and independent reviews for four distinct attempts. The R2 archive and the separate R2 typed status-record package remain unchanged. No Cargo target, package cache, source checkout, or scratch directory is included.

## VIS-01 status R7

The actual-result review confirms 53 selected tests started, 52 passed, and one failed. Cargo/Nextest exited 100. Source and tool snapshots were stable, and the target artifact contract passed. The failed test was status_renders_previewed_v1_and_v2_identically_and_rejects_invalid_v2 at the historical claim-snapshot fixture. This is a failed status test run; it is not a product test or product qualification result.

## Velnor R9

One selected test ran and passed: impl_schema2_routing::schema2_workflows_match_expected_bytes. The receipt and independent review keep tool and product qualification NOT_QUALIFIED. This single test does not establish qualification.

## VIS-19 cursor R5

The child exited 0 and all nine selected Rust tests passed. The original runner receipt remains FAIL_OR_INCOMPLETE; Root observed wrapper exit 125. The reviewed reconstruction found the human summary 88 skipped equals the two JSON filtered counts, 68 and 20, with zero ignored. The original wrapper rejected that summary, so this evidence does not change the wrapper result or establish broader product qualification.

## DCO observer R8

The receipt records a rustc_compile attempt stopped by SIGTERM after an unexpected temporary .rcgu.o file appeared. Its exit code is null, and the captured stdout and stderr files are empty. The receipt says cleanup was confirmed after SIGTERM and no Git initialization, multi-index, or probe operation occurred. No independent actual-result review is included. This is an interrupted observer compile attempt, not evidence of a completed product build or a compiler diagnostic.

## Limits

This is raw evidence only. It adds no normalized execution-history rows and makes no product-readiness, tool-qualification, capture, or admission claim. The VIS-01 R7 failure, Velnor R9 selected pass, cursor R5 selected-pass/original-wrapper-incomplete result, and DCO observer interruption remain separate attempts.
