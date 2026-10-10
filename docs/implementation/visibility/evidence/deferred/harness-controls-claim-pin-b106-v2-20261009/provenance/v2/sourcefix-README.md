# VIS-04 caller-pinned claim fix proposal

## State

This is an external two-file source proposal. It has not been applied to the repository. The design received independent `READY_FOR_BOUNDED_IMPLEMENTATION` review at `design-review.json` (SHA-256 `ff07e9a2e8d2d1be5e3dd2f1ebcbdf03952039d3f7b36e54e25fced9e3f6bf99`). No Cargo, Nextest, Python test suite, production CLI command, or product assertion was run for this patch.

## Change

The helper requires an explicit `--claim-pin PATH`. It rejects missing, duplicate, extra, malformed, or mistyped pin fields; reads the canonical task record; and compares its owner, token, work ID, branch, active state, accepted revision, and exact allowed paths against the explicit expectation and existing VIS-04 scope rules. It rejects Boolean/non-integer current or accepted queue revisions. The receipt records current queue revision and accepted queue revision separately, and records the SHA-256 of the caller pin.

The Rust fixture now models queue revision 38 with accepted revision 34 and passes the pin to the production CLI. The existing positive receipt control checks both revisions and pin digest. Two new Rust tests reject obsolete/wrong owner/wrong token/wrong work ID/wrong accepted revision/extra or duplicate pin fields before tool version/setup probes, and reject Boolean current or accepted queue revisions before those probes. The patched deferred Rust target contains 29 controls.

## Validation and limits

- The patch applies cleanly to the pinned base files: `git apply --check` exited 0.
- A read-only Python AST parse of the modified helper succeeded; this was not a Python test or CLI invocation.
- `rustfmt --check` reports formatting differences already present throughout the base Rust file. No whole-file reformat was applied; the changed Rust hunks have no rustfmt differences.
- No Rust tests were run. The earlier 27/27 result is historical for the old source hashes, not evidence for this patch.
- The separate S1d product lane remains its archived 22 PASS / 1 FAIL result (BD-21); this claim fix does not change or rerun product behavior.

`base/` and `source/` preserve both file versions. `sourcefix.diff` contains only `tools/visibility/deferred.py` and `crates/termrock-visibility-tests/tests/deferred.rs`. `test-names.json` lists the 29-control selection for a future reviewed Rust/Nextest gate. The cache, build target, and runner are not part of this patch proposal.
