# Fresh baseline capture runner Stage 3B

Status: bounded baseline-only advancement implemented; independent review and one live chunk smoke remain pending.

Implementation branch base: `fa57d1a6d1984bd3b484b0de38488a73150d3b2b` (`wip/baseline-cohort-admission-20261021`).

Stage 3B adds `fresh-run-advance` to `tools/frozen_baseline.py`. Each invocation selects the next ordered chunk from the externally anchored fresh plan and can run only `baseline-capture` or `baseline-replay`. It has no phase argument, candidate binary input, candidate comparison, finalizer, or reference-approval output. Reaching the planned ordinary-check phase fails explicitly without advancing it. Reports always keep `complete`, candidate comparison eligibility, checkpoint approval, and product acceptance false.

Each invocation revalidates the pinned baseline source, runner inputs, runtime environment and tools, external plan digest, authenticated manifest, and sealed baseline executable hashes. A nonblocking per-run file lock serializes advances. The command enforces one 225-second wall-time budget across verification, execution, and post-run checks.

The runner reuses the existing pinned Tuiscotti test harness and `run_live_chunk` validators. A chunk must produce successful nextest events for every selected scenario, at least one ordered checkpoint per scenario, and authenticated launch and close receipts. Capture writes into a unique untrusted staging directory. Successful expected frames are sealed with a digest over paths, file types, permission modes, and file bytes; committed chunk trees receive the same treatment. Publication refuses existing destinations. Replay uses the exact captured expected store and checks its digest before and after execution. A versioned HMAC journal binds each ordered chunk to the source, tools, environment, binary hashes, checkpoint receipt inventory, process receipt inventory, expected frame digest, and committed artifact digest. `fresh-run-verify` rechecks every committed artifact against those receipts.

Failed attempts remain in staging without a trusted receipt. Extra, missing, reordered, duplicate, skipped, empty, stale, or unbound journal records fail closed. The private run-authentication key remains in the existing owned mode-0700 custody directory as a mode-0600 file. It is not added to child environments, logs, or Git. The existing per-process receipt key stays within the Tuiscotti harness environment and is removed before application spawn by the pinned harness adapter.

The planned complete visual inventory remains 29,069 cases; the 20 ordinary checks remain registered. This increment does not implement ordinary-check advancement, candidate comparison, finalization, or product acceptance. The single large initial plan chunk is attempted as declared by the plan; it is never split or silently omitted to satisfy the timeout.

Validation: `python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py`; `python3 -m unittest tools.test_frozen_baseline` (70 passed); `git diff --check`. These are runner tests, not application parity evidence. No live capture or replay has been claimed yet. The fixed baseline, tag object, design reference, and Tuiscotti revisions are unchanged.

## Live smoke update (capture only; no replay claim)

A single real Tuiscotti CLI capture chunk was executed after this implementation, on the externally anchored plan and sealed baseline cohort. The plan was not edited, subsetted, reordered, or replaced for this run. Its declared schedule remains 910 capture chunks followed by 910 replay chunks; the smoke advanced only ordinal 0 of baseline capture. This run is evidence for the runner path, not full inventory completion or application parity.

- Source implementation: `cacb040598c3a398f0cc9d50cc531d46de8b44d1`.
- Frozen baseline source: `4169783ae883788186d161af09ba9c481bd84e5c`; tag object: `2718a6b8127a522a3d351d0b4bdb7542fc8da542`.
- Tuiscotti pin: `a1ea27f757d75d07b05499276cd932e9f8e067ad`.
- Anchored plan ID: `6e181372dda965ca819a3c593e9f549b`; plan creation digest: `5bb3130e0999cb552a6aeb82184fb63071307adfc519134ae8975ca6a5432470`; plan self-digest: `23943fc0bf01edd1e2af0a890190bbb7aeff6ed2bcd5057f9d18f3f513f49d2e`.
- Run ID: `1f73e7b76f514379ceb2ad49a5653912`; run-manifest SHA-256: `dc47aebe5e78f59221f30500ec236a8a097fe92dd495b0dd68dc8556c06cec79`.
- Step journal SHA-256: `056c36b15b9940cc40d03e420b234ec5c5a6da165dab8fa9213ed7c2ac2981d4`.
- Actual runner report SHA-256: `4325e14edc423a955ae5684a13d7a750182e9ef0094b998d77039e8f69ca11d5`; status `passed`; elapsed 12.71 seconds.
- The actual CLI flow launched 32 planned Holla audit cases. It recorded 32 successful test events, 32 ordered checkpoints, and 32 process receipts with launch/close validation. Checkpoint inventory digest: `53994b758334697841072de52d64a41ddb8d5eb32c3e3e0debcde1f14aaeb3b8`; process receipt inventory digest: `c80571a891c6a751fe337eb22fa10e01a4afcb03352069070c89221a6741741d`.
- Captured artifact-tree digest: `92d9d37cf41830bed1db5ab24ba04e36b50e966b834397e0c1194ca28725012b`; committed chunk digest: `2a5507fab7537c876fc50095ad9b5511bc5c327963476825fd8ce300bdf53d09`; immutable expected-frame tree digest: `5029c75622cd3a8cbadd6ac0ea51406ff67f4cbaa432a660842853e1654d491f`.

The run verifier re-read the journal and committed artifact hashes successfully. The receipt still records `capture_complete=false`, `replay_complete=false`, `complete=false`, candidate comparison ineligible, checkpoint reference approval `none`, and product acceptance false. No replay was run: the plan correctly requires all 910 capture chunks before any replay. No expected output was derived from a candidate. Full capture/replay, ordinary checks, candidate comparisons, and application acceptance remain incomplete. The external plan and raw evidence remain outside Git; no private authentication material or custody location is recorded here.

The prior immutable plan file was preserved byte-for-byte under the external evidence root before creating the anchored smoke plan; its path was renamed there to make room for the new canonical plan filename. Its original bytes and digest were not altered. The external smoke evidence is subject to independent parent review before use as acceptance evidence.
