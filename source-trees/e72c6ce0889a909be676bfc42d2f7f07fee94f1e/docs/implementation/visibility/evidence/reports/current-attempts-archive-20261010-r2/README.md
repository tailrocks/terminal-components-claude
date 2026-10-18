# Visibility current-attempt evidence archive — 2026-10-10, increment 2

This is a separate append-only archive directory under the accepted reports tree. It leaves the earlier archive at commit 5b147270c5ec8761278c90aa4c72752bef99c975 untouched. The files listed in MANIFEST.json are byte-checked copies of the external sources in COPY-MAP.json.

## Attempts

- VIS-01 attempt-history R5: Cargo/Nextest exited 101 before any test started. The run expected 53 tests; 0 started and all 53 are incomplete. This is a compile/setup failure, not 53 test failures.
- VIS-01 attempt-history R6: 53 tests started; 49 passed and 4 failed. An independent diagnosis review covers two of the four failures only. The archived receipt and raw streams preserve all outcomes.
- VIS-18 cache-source audit R12-r4: the selected test passed and the Cargo/Nextest child exited 0. The outer wrapper exited 125 because its output contract failed. The audit report says PASS, but historical R11 cache content is NOT_VERIFIED, qualification is blocked, and capture/admission were not run.
- The linked R11 product build is separate: its builder receipt records build PASS. A separate R11 helper test failed, and the independent review keeps cache qualification blocked. The R12-r4 result does not turn that helper failure into a build failure.
- Velnor marker-correction R5: two tests were selected; one passed and one failed. The gate is NOT_QUALIFIED. Only the combined command log was retained, not separate stdout and stderr.
- Velnor R8 source-tag guard: one test was selected and failed; Nextest exited 100 and the wrapper exited 1. The gate is not qualified.
- This increment does not include the older generator-preview CLI R8, which is a distinct attempt.

## Evidence limits

Raw receipts, logs, snapshots, input manifests, and review files are listed individually in MANIFEST.json. Reconstructions are labeled and do not replace original receipts. Static reviews are identified as static. R5 and Velnor R5/R8 have no prior independent actual-result review receipt; R6 has a diagnosis review for two failures only. The VIS-18 R12-r4 and linked R11 actual-result reviews are included. No normalized typed attempt records are added, and no product, test, capture, or admission qualification is claimed.
