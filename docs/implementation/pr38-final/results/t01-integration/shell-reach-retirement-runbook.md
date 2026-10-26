# Shell reach retirement record

## Fixed inputs

- Candidate: `542349e2f70ec9883d743594f906ac6c4e96ba5a`.
- Product source: `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
- T37 cleanup input: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
- T37 source unit: `7368d4fc5a71e46cae5ee92963bcf3a6943953e9`.

## Decision

The new decision ID is `T01-HISTORY-SHELL-REACH-RETIRE-01`.

The independent reviewer accepted retirement of the exact shell reach harness and listed public paths. The review does not use T37-PROBE-01 or T37-PROBE-02 as authority.

The decision does not cover other shell APIs, production behavior, or a replacement harness. Historical behavior obligations remain proposed and unverified.

The exact scope, source identities, public paths, owner tasks, and limits are in `shell-reach-retirement-decision.json`.

## Source checks

A read-only Python audit loaded the frozen Git blobs with `git show`. It checked all four historical source hashes and their equality at 1d6 and c817. It also checked the public line ranges and the source declarations. Result: PASS.

The audit checked the candidate shell library hash. It found no `reach.rs`, `pub mod reach`, `reach_tests!`, or `reach::` source entry. Result: PASS.

The candidate has no current Rust source references under `crates/**/*.rs` for `termrock_shell::reach`, `termrock-shell::reach`, `reach_tests!`, `REACH_DUMP`, or `shell::reach`. `git grep` exited 1 because it found no matches. Immutable T00 evidence retains the old catalog rows.

The accepted T37 source unit is an ancestor of candidate 542. `git merge-base --is-ancestor 7368d4fc5a71e46cae5ee92963bcf3a6943953e9 542349e2f70ec9883d743594f906ac6c4e96ba5a` exited 0.

The T00 tracked-tree, artifact-index, and test-inventory hashes match the candidate bytes. Their values are in the decision JSON.

## Commands and results

| Command | Result |
| --- | --- |
| Inline `python3 -` source, line, and artifact hash audit | Exit 0; source bindings match. |
| `git cat-file -e 1d6ee92adf6ccca38eeee64b1371cba1497e8fcd:crates/termrock-shell/src/reach.rs` | Exit 0; historical file exists. |
| `git cat-file -e 542349e2f70ec9883d743594f906ac6c4e96ba5a:crates/termrock-shell/src/reach.rs` | Exit 128; candidate file is absent. |
| Candidate Rust-source `git grep` for reach paths, macro, and dump variable | Exit 1; no current Rust-source matches. |
| `git diff --check -- docs/implementation/pr38-final/results/t01-integration/shell-reach-retirement-decision.json` | Exit 0. |

No Cargo command, legacy harness, test, fixture, doctest, or product behavior case ran. Selected behavior cases remain zero. Product behavior remains `NOT VERIFIED`.

## Review record

- Reviewer: `/root/t01_inventory_review`.
- Requested selection: `gpt-6.1-sol`, high effort, no fork.
- Backend attestation: `NOT PROVIDED`.
- Decision: `ACCEPT_RETIREMENT_SCOPED`.
- Receipt consistency review of this persisted artifact: pending.
- Artifact SHA-256: `4ea12ecd0b6dcea45a7acf0bc79313356929e5322518236aaebc68674e9d720b`.
- Output commit: pending; the canonical run-state will bind it after commit.
