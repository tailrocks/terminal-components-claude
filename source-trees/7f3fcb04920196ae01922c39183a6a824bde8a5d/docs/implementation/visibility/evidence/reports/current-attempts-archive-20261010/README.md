# Visibility attempt evidence archive — 2026-10-10

This append-only archive preserves raw provider responses, original tool receipts, logs, reconstructions, and independent review files for the attempts listed in MANIFEST.json. It adds a separate directory under the visibility evidence reports tree.

The published source commit 882d5162ab6c838cc03ef35ee65df6b53ffe50f0 contains the prior status-source archive. Its manifest lists 70 raw members (6,171,844 bytes). All paths, byte counts, and SHA-256 values were checked against that commit’s Git objects; this increment leaves those paths untouched.

The manifest records each original source path, byte count, and SHA-256. Reconstructions are labeled and do not replace absent original receipts.

## Limits

- The CI poll reports zero jobs and artifacts. The failed-log CLI output says the log was not found; no raw HTTP response was captured. HTTP status is NOT_CAPTURED and diagnostic status is NOT_EXPOSED.
- RUN-03 R7 has no original final receipt at its source location. The available raw preflight, postflight, event, and stream files are preserved with a separate reconstruction.
- VIS-16 R3/R5 and VIS-17 R4 remain separately identified attempts.
- Tag R15 R7 is a later read-only verification, distinct from the earlier R15 capture attempt.
- This raw archive does not populate typed attempt-history rows and does not qualify a product, capture, or admission.
