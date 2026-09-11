# Independent final integration-plan review

Reviewer scope: planning only; branch topology, protected catalog/source separation, dependency ancestry, frozen-tree integration, standalone taskfmt, historical assertion authority and final readiness. No production edits, task execution, ref updates, push or merge. Reviewed 2026-09-11. Findings below exclude already assigned host-observer substitution/ledger-authority and Rust architecture qualification defects.

## INT-01 — High: integration-branch creation follows tasks that already require that branch

Status: accepted by coordinator; repair pending independent reread.

Sources: `REFACTORING_COMPLETION_PLAN.md` §13 originally says create the integration branch from pinned main **after** the complete oracle contract is frozen; `history.md` recommended integration repeats that order. TASK-070 README P-001 requires TASK-001's actual integrated source ancestry; TASK-002 P-002 and the baseline DAG require the integrated 001→070→071→072 chain. `proof-contract.md:125–148` prepares and advances the campaign integration parent/ref for those tasks.

Reproduction: start future execution at pinned main with no integration branch. Obey §13 by withholding branch creation until TASK-006 seals all oracle namespaces. TASK-070 cannot start without integrated TASK-001, and baseline tasks cannot start without integrated proof predecessors. No separate preparation branch/handoff is defined. Creating a fresh pinned-main branch after completing those source producers would also omit their required ancestry.

Repair: create the future campaign integration branch at pinned architectural main before TASK-001; accumulate proof/preparation commits on that same ancestry. Delay production component/application refactoring until complete baseline and disposition receipts exist. This grants no branch creation during the current planning goal. Coordinator explicitly accepted this repair.

## INT-02 — Medium: canonical worker protocol conflicts with campaign verification authority

Status: open for coordinator disposition.

Sources: every package's canonical `AGENTS.md:32–33` instructs the executor to run ordered checks, then `taskfmt verify --progress ""`, append completion, and run bare full verification. `proof-contract.md:23` prohibits empty progress and requires explicit base; `proof-contract.md:138–146` puts taskfmt/dispatch under host authority and candidate execution in subordinate workers. The mandatory package protocol does not distinguish an advisory executor check from an authoritative host gate or explain how its worker obtains a host verification result.

Reproduction: follow TASK-069 AGENTS step 7 literally. Either run the prohibited progress-disabled gate under executor authority, or stop because accepted host command/context authority is unavailable inside the candidate worker. Following only the external campaign procedure instead leaves the mandatory local sequence unsatisfied. This is a protocol contradiction, not an assertion that taskfmt itself lacks environment fallback: inspected pinned `harness/src/gate.rs:361–384` does support `TASKFMT_BASE`; an earlier review hypothesis to the contrary was rejected.

Repair: add an explicit campaign adaptation, preserving canonical schemas and documenting template provenance. Define worker progress and advisory checks, the point at which the worker requests host freeze/verification, the immutable base/context arguments, and how the worker reports the host-authentic result. State which template command sequence is superseded and prohibit progress-disabled runs from authorizing integration. Keep full final progress validation mandatory.

## INT-03 — High: freeze qualification never tests distrust of the executor index

Status: open; needs fixture correction and independent rereview.

Sources: `proof-contract.md:144` requires filesystem reconstruction, relevant untracked source inclusion and distrust of the executor index. `host-bootstrap-driver.py:388–396` instead runs `git add --all` before every successful submitted-host freeze and derives `expected_tree` with the candidate's `git write-tree`. `host-bootstrap-vectors.json` has hidden-index rejection but no passing case with divergent staged/unstaged bytes or an allowed untracked addition. TASK-001 `trusted/obligations.md` claims qualified mutable-source/index behavior.

Reproduction by corpus inspection: substitute the freeze tree-selection algorithm with `git write-tree` against the executor index, retaining the required explicit negative checks. Every existing successful fixture supplies a fully staged candidate, so this forbidden algorithm chooses exactly the driver's expected tree. An actual executor can leave a changed tracked file unstaged or add a new source file without staging; the same implementation then verifies/integrates stale bytes or silently omits required source. The suite cannot distinguish the forbidden implementation from filesystem reconstruction. No real production host exists yet, so this finding does not claim an existing host passed such a mutant.

Repair: add a positive fixture containing a tracked file whose staged content differs from final filesystem bytes, an ordinary unstaged modification/deletion, and a required allowed untracked new source. Build the expected tree independently from the protected parent and validated filesystem map, without staging the submitted candidate. Require checks and final integrated commit to contain those exact bytes. Add a negative/lying freeze implementation or equivalent unit witness proving index-only construction fails the corrected corpus.

Related uncovered boundary: the host fixture explicitly makes scope base equal integration parent (`host-bootstrap-protocol.md:79`), while production permits protected verification overlays (`proof-contract.md:142`). Add a successful case where parent and overlay scope base differ, an overlay-tampering rejection, and exact final-tree assertions. Specify whether immutable overlay files enter the integrated tree; never test one tree and silently strip its overlay before integration. The standalone missing-overlay failure is useful but does not exercise successful host overlay reconstruction/integration.

## Inspected protections with no additional finding

- Immutable oracle and architectural main resolve locally to their exact declared SHAs. Merge base and 774/38 divergence match. Main workspace preservation and selective behavior adaptation avoid losing already integrated architectural work.
- External catalog and receipts are separated from writable source; taskfmt run/dispatch/promote are explicitly prohibited because they hardcode main. Actual predecessor ancestry, not status alone, is required.
- Integration requires exact verified tree, sole recorded parent, DCO/coauthor trailers and expected-parent update-ref. Sibling joins require fresh combined-tree gates. Final TASK-069 detects remote-main drift read-only, invalidates changed source/parent/merge trees, and authorizes no merge/push.
- TASK-007/008 preserve source-qualified historical assertions and independently accepted oracle conflicts; TASK-066 explicitly owns test relocation; TASK-068 requires reviewed final API baseline. Their obligations do not authorize candidate baseline blessing.

## Review fingerprints

Initial review snapshot (files can change during coordinator repairs):

| Input | Identity |
| --- | --- |
| Oracle commit | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| Architectural main | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| Planning HEAD | `2e2401393c47360741ebd321679de08982dca50a` |
| Merge base | `cc14dd6beae526884aabdf897e309be837b4f504` |
| proof-contract.md SHA-256 | `451e6fb017f868198552a2abe49839272fb7cf4a5edd1ab8eea2ede63018ae74` |
| host-bootstrap-driver.py SHA-256 | `f8e982b128eb40d2d8e1f600dbac4f4a306fa5b146f97c9d2e444e658d607da1` |
| host-bootstrap-vectors.json SHA-256 | `028041c5449e737ece29460122bd6134168e790399c52facc0abcdad9b65dd7b` |
| task-graph.json SHA-256 | `49eaf7475fde8a87e2ab71f1d8945f45829baa580e6160ced79e0c339e02a802` |
| traceability.tsv SHA-256 | `c1ba0f0c4e1ceaed1e6b7882d7039ac492e1271023097ae6800c2064edbda73f` |
| TASK-069 AGENTS.md SHA-256 | `a742721ad41eec30a3821eadd9d570e2b78e893baef29ac8edff972e78206245` |

Material fixes require rereview before these findings can close. This report is not campaign execution approval.
