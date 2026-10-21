# Owned `prepare-live` stage 2

This implementation adds an opt-in, cancellation-aware lifecycle to `prepare-live`. It implements only the reviewed stage 2 source-preparation operation. It does not invoke stage 2 against the operational evidence root, create a plan, read a private key, run visual scenarios, or approve frames.

Architecture input: `stage3d-stage2-owned-lifecycle-architecture-00f967-20261021.json`, SHA-256 `cfacde36eda321a673bee36bcde80f76de0e3e7d706a1eb18e7c446966dcba96`.

The opt-in CLI is `prepare-live --owned-lifecycle --owned-budget N --fresh-cohort frozen-20261021-serial-nonincr-r1-r2 --baseline-bin-dir <sealed-binaries>`. `--owned-budget` is bounded by the existing 1..200 second policy plus the fixed 15 second cleanup reserve. Legacy `prepare-live` parsing and execution remain on the existing path. Owned API calls require the explicit fixed cohort and binary directory too.

One `PrepareExecution` is passed through source verification, cohort and binary hashing, checked source copy, snapshot verification and read-only traversal, the exact pinned patch, tool observations, and complete Nextest inventory. File hashing, copy operations, and directory walks check the same work deadline. Tool probes keep their 12 second per-probe ceiling, capped by that deadline. Optional probe failures retain their legacy `None` result for ordinary spawn/nonzero/local-timeout outcomes; cancellation, global expiry, ownership errors, and unverified cleanup propagate as failures. `cargo nextest list --locked ... --message-format json` remains the inventory source; the owned list environment removes `VISUAL_TOOL`. The custom harness trial bodies and application binaries are not invoked in list mode.

The existing seven allowed source changes, fixed snapshot bytes, extension manifest, and complete 29,069 visual plus 20 ordinary inventory requirements remain enforced. `live-source.json` is written only after those checks pass. On an error, any copied source, build target, or temporary manifest remains as incomplete evidence; no automatic cleanup, resume, reset, or manifest repair is performed.

Validation on the isolated writer worktree: `python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py`, `git diff --check`, and the full Python runner guard suite (103 tests) pass. Synthetic tests exercise shared owner propagation, checked copy and immutable snapshots, the exact seven-path change set, failure without a final manifest, Nextest list discovery with `VISUAL_TOOL` absent and no trial body, the 12 second probe cap, optional nonzero/cooperative timeout behavior, and propagation of global deadline/cancellation/unverified-cleanup failures.

Remaining gate: independent review of the exact commit is required before operational stage 2. Product acceptance remains incomplete. No candidate or frozen baseline output was changed.
