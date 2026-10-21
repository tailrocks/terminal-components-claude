# Ad86 parent comparison for the 24 broad conformance failures

This bounded comparison checks whether the 24 failures recorded by the unfiltered 935-case candidate run also fail at the exact pre-integration ad86 commit. It does not claim the broad suite is green or determine the failures' root causes.

- Parent source: detached, clean worktree at `ad86aaca1b987ecb395b7ca9f8717dd79f62f784` (`/Users/donbeave/Projects/termrock-worktrees/ad86-parent-conformance-20261021`).
- Separate target: `/Users/donbeave/Projects/termrock-review-targets/ad86-parent-conformance-target-20261021`.
- Command: `cargo nextest run --locked -p termrock-conformance --test conformance -E '<the 24 exact test filters recorded in parent-24-probe.log>' --no-fail-fast --status-level skip`.
- Result: 24 run, 0 passed, 24 failed, 911 skipped. Each selected test fails on ad86.
- Candidate comparison: the preserved current 935-case log lists the same 24 distinct test names; normalized exact-name comparison produced 24/24 intersection, with no names unique to either list.
- Ad86 test binary SHA-256: `d6a35b3d4b8c5cba40c2eb56890faa661a60feaf85d60686d10d4cfbc8eeb222`.
- Current candidate full-run evidence SHA-256 (compressed log): `7f42c757efca2df4983c57b78a508a7d5609358f595a613068eefa90674d031e`.
- Source blob IDs at ad86: `Cargo.lock` `b223212149cac8dbedd88cfa5d715f42ea850f80`; `Cargo.toml` `fabd8dabbc995fa509039b4150728d4570cc65cf`; `crates/termrock-conformance/tests/conformance.rs` `4a73e6262f3459f11657971884becf0e73d49e53`; `crates/termrock-test-support/src/conformance/driver.rs` `c0b2ea1d4dfbfcbfae019f2703e5dee7ec6a5d96`; `crates/termrock-test-support/src/conformance/mod.rs` `b49436f8da93481abfba72c0494ad6cc22772b9e`.

The comparison establishes that all 24 named failures reproduce on ad86, before the later caret/Form integration commits. It does not prove their root cause, establish that every other candidate test is unaffected, or change the unresolved RED status of the full conformance suite.

Detailed selected-run output: [`parent-24-probe.log.gz`](parent-24-probe.log). Current candidate full-run output: [`broad-conformance-935-red.log.gz`](broad-conformance-935-red.log.gz).
