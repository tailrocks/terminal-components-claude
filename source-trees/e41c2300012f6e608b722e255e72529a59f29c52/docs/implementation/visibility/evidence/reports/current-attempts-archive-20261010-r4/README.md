# Current attempt evidence archive R4

This append adds raw evidence for the VIS-01 R8 status test, the PR17 provider refresh, the DCO clone-helper observation, and the measured RUN-03 pair phase record. R3 remains unchanged. No Cargo cache, build target, source checkout, or compiled helper is included. This archive adds no normalized attempt-history rows and makes no qualification or admission claim.

## VIS-01 status R8

The exact selected status integration case 'status_renders_previewed_v1_and_v2_identically_and_rejects_invalid_v2' passed once; 52 other status tests were filtered. The runner and actual-result reviews bind this to the R8 source overlay. This is not a 53-case suite pass and does not establish product qualification.

## PR17 provider observation

The raw GET response set records candidate head e0bcfa26ec174241f09de17d04f3f45379be3452, workflow run 38003496873 as failed, zero jobs, zero artifacts, an Actions check-suite response with no check runs, and a logs endpoint HTTP 404. The captured responses do not expose the failure cause. The workflow is 537,470 bytes, above the documented 500 KB limit; this archive does not claim that size caused the failure. The reference DCO state is action_required with 14 unsigned commits; candidate DCO is success in this observation.

## DCO clone-helper observation

The compiler exited 0, then the clone helper exited 2 with 81 bytes on stderr. The original review is retained but superseded by the correction. The correction identifies the pinned helper's expected directory ctime as 2,000 ns above the unchanged source inventory and current read-only lstat. The generic identity-changed message therefore came from a stale literal; it does not establish temporal drift. No clone, Git initialization, network probe, or application build occurred.

## RUN-03 pair evidence

The exact measured pair-evidence.json is included as a raw source artifact for the paired candidate/reference attempt. It remains separate from the single-case and provider observations above.

## Limits

The archive preserves evidence and provenance only. It does not alter earlier archive bytes, infer a provider startup cause, promote a partial Rust result to suite qualification, or state that a product was admitted.
