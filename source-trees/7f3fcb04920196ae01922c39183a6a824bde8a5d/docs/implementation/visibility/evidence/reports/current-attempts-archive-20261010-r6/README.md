# Current attempt evidence archive R6 (external proposal)

This proposal adds byte-checked copies under the accepted VIS-13 `reports/**` scope. Its base is candidate commit `331b6fd95835ca505a9417581aaa37df52504797` (tree `c2a205953e94357e92b0e007318873f9fe0f32b4`). The prior R5 archive and all earlier archive directories are unchanged. This proposal has not been committed or published.

## VIS-01 status target, 59 cases

The archived Nextest run selected the `termrock-visibility-tests::status` integration target. The pinned Rust Reader v2 report records 59 selected identities, 59 starts and 59 successful terminal events, with 59 passed and zero failed, ignored, or filtered. Independent reviews bind the raw capture, exact selection, Reader report, source, and execution provenance. The target tests the status/reporting infrastructure; it is not a product run or product-readiness qualification.

The test source freeze is based on commit `57d505f343bf35e5041471f4cbab476cb92d67da` / tree `965aef70b90ca0a8a4be2f869da839a3711245d8`, with the two reviewed status source paths in the recorded overlay. Its historical product comparison pair is candidate `1d797d41c8141fcbdc3f69d7f11eb8875ab54712` and reference `b274dd57f4dd078ade6e424d546d83efbd2e8526`. These historical measured inputs are separate from the current published branch tips.

## VIS-08 collector 26-case run

The R17 alias-fix attempt ran 26 selected Rust Reader unit tests and the recorded Reader report says all 26 passed. Those are collector/reader tests, not application behavior tests. The same attempt reprocessed a prior 142-test capture: that historical run has 141 passes and one failure (`env_remove_drops_one_var`), and its acceptance remains blocked. The archive keeps the 26-test result and the 142-test input/result separate.

The committed reader source at candidate `331b6fd95835ca505a9417581aaa37df52504797` has `src/nextest_result.rs` SHA-256 `b3191ad38cf707255381f4c6851eb4be0f23dad45db42cd68dcbc6dbef9ead03`, matching the source exercised in the R17 packet. This does not qualify product behavior.

## Current publication and CI observation

Root's post-push readback records candidate `331b6fd95835ca505a9417581aaa37df52504797`, reference `4b473a98a8641a9dae8dbf9c31c0c94b15465496`, tag object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, and peeled tag commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The readback states `visibility_complete=false` and `refactor_ready=false`.

The exact-head PR-17 poll records run `38016983578` as failed with zero jobs and zero artifacts; DCO check `114109357313` succeeded. GitHub denied log retrieval with HTTP 403 (“Must have admin rights to Repository”), so the failure cause is not exposed by this capture. The workflow blob is unchanged at `672078dc6c964a768c05b53f73de8bb99b90bffa` (537,470 bytes); its size is not established as the cause of this run. No diagnosis is inferred.

## Evidence limits

This archive contains raw attempts, their exact input/source pins, reviews, and current publication/CI observations. It adds no normalized execution rows or status-source edits. It does not establish product parity, product readiness, CI success, visibility completion, tool qualification, or release qualification.
