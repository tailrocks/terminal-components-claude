# Historical paired Holla journey evidence

This archive records one historical `HELP-HOLLA-004` run from actual, separately built Holla executables. Package review is pending. Product readiness is not established, and visual parity remains blocked.

## Historical subjects and suite

- Candidate source commit: `0b7545ff7e3fff40254fd7a84a80c2c8a7872f3f`; Holla executable SHA-256: `a00f19664acd52ddfcb16c153d785f385bae3a68dd1c076e25f0ea44d2d5b897`.
- Reference source commit: `4b473a98a8641a9dae8dbf9c31c0c94b15465496`; Holla executable SHA-256: `d9dd6eb44daa25851e8461e39641c5494cc4ce206445cd044e85f5c517eac7c9`.
- Historical shared `termrock-e2e` tree: `c962085b9c9e6b80ddba31d8c82e01c8d2e97179`; suite digest: `b67efe786fb0c0f64db2aca62c572b700f2a5247d7a2258deab1313bdb581a5d`.
- The exact shared test binary was `shared_journeys-75a38fc8a0088043` (SHA-256 `e762809681c8047dd141a9e647275c9bde490036531d1e0ed786197514641863`), built from `tests/shared_journeys.rs`.
- The integration snapshot at commit `706fd4fdb0473cb9348039d0ce35182a089d0223` has `crates/termrock-e2e` tree `6c45dc171e689988957dae363e01a8afbacb0828`, which differs from the historical test tree. This archive does not claim that the current suite source was exercised.

## Run outcome

The exact ignored test `holla_help_overlay_004` launched the real candidate and reference binaries and produced all 80 recorded capture artifacts across four checkpoints for each role. Receipt checks show:

- 56 `PASS`, including 32 interaction checks (16 per role).
- 8 visual checks `BLOCKED` (four per role) because `expected_generation` was `null`.
- 16 `NOT_APPLICABLE`; zero check records were `FAIL`.

The suite test itself exited 100 after 111.20 seconds at its acceptance-readiness assertion because visual comparison was blocked. The test’s nonzero result is preserved. No expected-generation admission was run, and product readiness is not established.

The caller trust record, preflight policy, all run requests and raw captures, the prior R2 Nextest CLI startup failure, Nextest/Cargo metadata, independent trust/preflight reviews, and Reference’s actual-result review are included. Trust acceptance means the caller-supplied trust record matched the pinned identities; it does not approve snapshots or unblock visual checks. `ARCHIVE-MANIFEST.json` is a draft pending focused package review.

Candidate/reference application binaries and the shared test binary are not bundled. Their exact digests and build/test provenance are recorded in the paired receipt and provenance files. `ARCHIVE-MANIFEST.json` lists every included file’s archived path, byte size, SHA-256, and source path where copied from an external capture.
