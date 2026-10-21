# Stage one owned-process lifecycle

Status: implementation in `wip/canonical-stage1-owned-20261021`; independent review pending. This record does not authorize running the real `prepare` command, reading the frozen key, preparing a new baseline, or capturing frames.

The opt-in `prepare --owned-lifecycle` path adds one monotonic work budget, a fixed cleanup reserve, cancellation-safe signal recording, and an anchored broker for the stage-one fixed `git rev-parse` and `git archive` commands. The legacy default path and other runner stages keep their existing behavior. The owned path bounds work to 200 seconds and reserves 15 seconds for cleanup, below the 220-second supervisor ceiling. Extraction and digest loops check the same work deadline.

The broker remains the unreaped process-group leader until the parent verifies that no live members remain except the anchor, then accepts a one-byte release. The broker catches termination signals while its executed Git child receives default signal dispositions. Parent-pipe EOF triggers broker-owned termination of the group. Group membership is observed with `ps` only while the anchor remains live; the parent sends no group signal after releasing or reaping the anchor. A `SIGKILL` cleanup path is always reported as unverified failure and cannot produce a successful prepare result. Failure logs remain in the reported temporary directory.

The group contract covers the fixed, noninteractive Git commands and same-session children. It does not claim containment of a child that creates a new session, a process stuck in an uninterruptible kernel operation, or arbitrary future tool wrappers. Those cases require separate ownership edges and review.

Focused tests use synthetic Python commands only. They cover byte-preserving archive-style stdout, captured streams, ordinary group cleanup, cancellation, default child `SIGTERM`, and a `SIGTERM`-resistant command failing closed. No real frozen source preparation, key access, capture, replay, candidate comparison, or reference update is part of this increment.

## Verification

`python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py` and `git diff --check` pass. `python3 -m unittest tools.test_frozen_baseline` passes all 91 tests. One existing fresh-run timeout test failed once in an earlier full-suite run and passed when rerun alone and in the final full-suite run; no source change was made to that test.

Exact-source independent review and WIP publication remain pending. Passing synthetic tests are implementation evidence only; they are not evidence of frozen product parity or permission to use the owned path on the real baseline.
