# VIS-01 historical status attempt records (R2)

This package contains two derived typed result records and a source-facts history fragment. It does not replace or modify the raw run receipts. Those remain in the R9 archive at docs/implementation/visibility/evidence/reports/current-attempts-archive-20261010-r2/raw/status-history/.

Both attempts ran against base publication 882d5162ab6c838cc03ef35ee65df6b53ffe50f0 (tree bef0248d3df09bb832e1c219aca310725f0ba6d6) with distinct source overlays. R5 used freeze b60026ab91d39961027c435a274108d3f837671b7394724c2f3f9cbac8d29fc1, manifest d1e0f7323350dfa27807d79d337ab0f39c0f8641bf03d86977abc5a6a622b212, and patch d3651b036a9201ebb4a1bc76577a771dd14c7926b44222eb7d5dbce889bc966c; R6 used freeze 6356d5a3e37ca5705d70ba6d1c0d8203e65121777c383c391a1fb92039003317, manifest 868091a9cc2556406f2f4d99f954dbbaa5c6d49bb432e510a9a0914c51524d43, and patch a1c16313df97770a1bb4220302ec5daece5ee80f1633a2d495deb1c72ce65732. Both rows are HISTORICAL.

- R5: 53 selected, 0 started, 0 passed, 0 failed, 53 incomplete; Cargo child exited 101. The test gate did not run. The receipt does not record a wrapper exit; wrapper is INCOMPLETE. Target-artifact validation did not verify required artifacts.
- R6: 53 selected and started, 49 passed, 4 failed, 0 incomplete; Cargo child exited 100. All Nextest events were collected. The receipt does not record a wrapper exit; wrapper is INCOMPLETE. Target-artifact validation did not verify required artifacts.

For both rows, product qualification is NOT_APPLICABLE, product capture and admission are NOT_RUN, and no product-phase result is claimed. Process evidence records that each direct child was reaped, with no signal, timeout, output cap, forced close, or spawn error. Process-group cleanup was not independently verified, so cleanup is UNVERIFIED.

Review limits are preserved: R5 has a static runner/preflight review only, not an independent actual-result review. The archived R6 diagnosis covers two of its four failed assertions only and does not reassess the other two. No R7 result is represented.

The JSON rows are projections from the published archive at commit 6b00cb1c5cd10ab2e82014c306a603e92ea5e694 / tree f10bd01e2d1b38b8e5f50f397fa070a348149cfe; they are not new execution receipts and do not qualify product readiness.
