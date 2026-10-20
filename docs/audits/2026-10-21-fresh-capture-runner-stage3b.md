# Fresh baseline capture runner Stage 3B

Status: bounded baseline-only advancement implemented; independent review and one live chunk smoke remain pending.

Implementation branch base: `fa57d1a6d1984bd3b484b0de38488a73150d3b2b` (`wip/baseline-cohort-admission-20261021`).

Stage 3B adds `fresh-run-advance` to `tools/frozen_baseline.py`. Each invocation selects the next ordered chunk from the externally anchored fresh plan and can run only `baseline-capture` or `baseline-replay`. It has no phase argument, candidate binary input, candidate comparison, finalizer, or reference-approval output. Reaching the planned ordinary-check phase fails explicitly without advancing it. Reports always keep `complete`, candidate comparison eligibility, checkpoint approval, and product acceptance false.

Each invocation revalidates the pinned baseline source, runner inputs, runtime environment and tools, external plan digest, authenticated manifest, and sealed baseline executable hashes. A nonblocking per-run file lock serializes advances. The command enforces one 225-second wall-time budget across verification, execution, and post-run checks.

The runner reuses the existing pinned Tuiscotti test harness and `run_live_chunk` validators. A chunk must produce successful nextest events for every selected scenario, at least one ordered checkpoint per scenario, and authenticated launch and close receipts. Capture writes into a unique untrusted staging directory. Successful expected frames are sealed with a digest over paths, file types, permission modes, and file bytes; committed chunk trees receive the same treatment. Publication refuses existing destinations. Replay uses the exact captured expected store and checks its digest before and after execution. A versioned HMAC journal binds each ordered chunk to the source, tools, environment, binary hashes, checkpoint receipt inventory, process receipt inventory, expected frame digest, and committed artifact digest. `fresh-run-verify` rechecks every committed artifact against those receipts.

Failed attempts remain in staging without a trusted receipt. Extra, missing, reordered, duplicate, skipped, empty, stale, or unbound journal records fail closed. The private run-authentication key remains in the existing owned mode-0700 custody directory as a mode-0600 file. It is not added to child environments, logs, or Git. The existing per-process receipt key stays within the Tuiscotti harness environment and is removed before application spawn by the pinned harness adapter.

The planned complete visual inventory remains 29,069 cases; the 20 ordinary checks remain registered. This increment does not implement ordinary-check advancement, candidate comparison, finalization, or product acceptance. The single large initial plan chunk is attempted as declared by the plan; it is never split or silently omitted to satisfy the timeout.

Validation: `python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py`; `python3 -m unittest tools.test_frozen_baseline` (70 passed); `git diff --check`. These are runner tests, not application parity evidence. No live capture or replay has been claimed yet. The fixed baseline, tag object, design reference, and Tuiscotti revisions are unchanged.
