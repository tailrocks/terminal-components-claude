# Current attempt evidence archive R5 (prepared proposal)

This external staging tree adds raw evidence under the accepted VIS-13 reports/** scope. The prior R2, R3, and R4 archives are unchanged. The proposal parent is 00ba7397bd935797ca3954cd270e3d649a0b6185 (tree c46659dac09cf09755143a01860f6162fbb3dfb5). This is not yet committed or published.

## VIS-08 Nextest reader R4

The independent actual-result review records 23/23 selected reader unit tests passing, plus separate locked/offline CLI build and two synthetic v1/v2 CLI contract exercises passing. These synthetic fixtures are not product runs. Scope is the reader package and its test/CLI contracts only; product parity and product qualification remain unestablished. The standalone source package and five CLI fixture inputs are included; target output and Cargo caches are excluded. Root's 00ba7397 commit and post-push records are included.

## Cargo metadata R2

The command exited 0 and produced valid metadata JSON for 45 packages/workspace members, but resolve was absent, the reqwest sparse index remained absent, and the Cargo home had zero changes. Independent actual-result review classifies the requested index-fill postcondition as incomplete and not qualified. This is not a build, test, CLI, or generator qualification. The R2 static packet review is separate from the actual-result review.

## Cargo metadata R3

A separate dry-run index-resolution command exited 0. Independent actual-result review verified the reqwest sparse index was populated with 151 added cache paths (152 changes including .global-cache); the candidate source, lockfile, and target remained unchanged. The command is a Cargo update dry-run, so it does not validate a hypothetical updated graph through the lockfile writer. Qualification remains NOT_QUALIFIED; this does not establish complete cache closure, build/test/generator qualification, or release qualification.

## PR-17 f6 REST observation

The capture preserves the bounded raw GET responses, manifest, and summary. It reports PR head f6d26bf2f0e5ba5594b9c80c5e512dce4213e545; exact-head Actions run 38007979466 completed with failure and exposed zero jobs/artifacts, and the logs endpoint returned HTTP 404. The provider/startup cause is NOT EXPOSED. The 537,470-byte workflow exceeds the documented 500 KB trigger limit, but the capture does not prove this was the run's exact cause. The DCO check completed successfully. No current status or qualification is inferred from this historical capture.

## Velnor 48-case R5 attempt

The independent actual-result review confirms 48 selected cases expected, 47 started, 46 raw PASS rows, and one real test failure. The aggregate's 14 gate-counted passes come only from exact successful groups 1, 2, 5, and 6; this is not the raw PASS-row count. Group 3 failed impl_ci_observer::runtime_distinguishes_missing_pending_ambiguous_and_http_errors. The review says deadline exhaustion is likely, but the logs do not prove the exact cause; it is not called a flake or production defect. Group 4 ran 10 of 11 expected selectors and exited 0. The configured selector omitted branch_rejections::; source nesting points to generator_release::source_gate::tests::branch_rejections::stable_dispatch_rejects_wrong_event_ref_sha_and_ci_receipt, but this run has no list-only output confirming that full Nextest ID. Group 4 therefore remains incomplete. Wrapper exit was 1 and tool qualification remains NOT_QUALIFIED. The source snapshot and immutable cache inputs were unchanged; the separately recorded .global-cache file changed. No source tree, target, Cargo home, or runtime binary cache is included.

## Limits

This archive preserves evidence categories separately. It adds no status/source changes and no normalized execution history. Nothing here establishes product parity, tool qualification, release qualification, CI readiness, or admission. The Velnor actual-result review is included and bound to the raw aggregate and per-command evidence.
