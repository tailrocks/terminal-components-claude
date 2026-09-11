# Independent final decomposition and dependency review

Verdict: changes required. The declared dependency graph is acyclic, but its preparation contracts still contain a receipt dependency cycle and an impossible assertion-replacement scope. Candidate observation ownership also needs a canonical contract. Schema lint cannot establish those properties.

This review covers planning only. No terminal-components implementation, reference capture, execution branch, or merge was performed. The reviewer did not author the catalog. The review read the complete planning goal and top-level plan, inspected the requirements and decisions across all 73 packages, their machine checks/dependencies/scopes, the producer and closure contracts, and relevant pinned production and task-format source. It is a decomposition review, not a fresh reconstruction of all 620 historical clauses.

## DAG-FINAL-01 — P1: preparation accounting requires its own future accepted receipts

Affected tasks: TASK-002–008 and TASK-071.

References: TASK-071 README R-002 and trusted/obligations.md; TASK-007 verify.toml:25,33,41 and README D-002; TASK-008 verify.toml:25,33,41; TASK-002–006 CHK-005; proof-contract.md:91 and its existing-tests/stage-policy section.

TASK-071 defines `account-tests` as consuming trusted inventory and disposition producer receipts. TASK-007 produces the inventory, and TASK-008 produces the disposition. Both producers invoke that same operation to pass their own gates. The host only accepts their product after independent checks and full gate evidence; an unaccepted product expressly fails receipt resolution. Therefore TASK-007 cannot satisfy the ordinary accounting input contract, and TASK-008 cannot supply its own accepted disposition.

There is a second concrete instance: oracle tasks TASK-002–006 run `account-tests`, while TASK-007 is parallel and TASK-008 depends on TASK-006. Adding ordinary receipt dependencies would create a real TASK-006/TASK-008 cycle. The current acyclic TOML graph omits the trust dependency rather than resolving it.

Repair: define and independently qualify a preparation accounting contract in TASK-071. Bind its exact source inventory and permitted producer-stage observations to planner/host authority, distinguish proposed products from accepted consumer receipts, and prohibit a proposal from authorizing its own omitted tests or allowed failures. Assign that contract explicitly to TASK-002–008. Preserve strict accepted-receipt consumption for production tasks. Add positive producer-before-receipt tests and negative self-approved-inventory/disposition tests. Merely adding a dependency or saying that the host reviews the output does not define the required pre-gate authority.

## DAG-FINAL-02 — P1: TASK-008 cannot replace known conflicting inline assertions

Affected tasks: TASK-008, TASK-040/041 and later application/component owners containing inline tests.

References: TASK-008 README:38–55 and verify.toml:3; docs/refactoring-plan/holla.md:22,35; pinned main `7b27732a8c3c131760ec3438f641cb3c11343a42:apps/holla/src/app.rs:1201,1253`; pinned main `apps/holla/src/scenario.rs:107–111`.

TASK-008 must replace every oracle-conflicting product assertion before repair tasks, while preserving its original bytes and useful structural assertions. Its writable paths include integration-test directories but no production source files containing inline tests.

The mismatch is already proven in the planning sources and pinned code. `home_hint_casing_preserves_lowercase_physical_shortcuts` asserts `Ctrl+S Scope`, `q Quit`, and the resulting quit dialog. `home_escape_clears_scope_before_canonical_query` asserts that Escape removes scope while retaining `git`. `names_round_trip` asserts exactly 11 worlds. These tests live under `apps/holla/src`, outside TASK-008's scope. Later repair tasks may edit those production files but explicitly cannot change test disposition or expected authority. Neither leaving contradictory assertions active nor suppressing them is an allowed completion.

Repair: inventory inline conflicts explicitly and give their replacement a concrete authorized application path. One option is narrowly scoped TASK-008 source edits, independently restricted to the exact reviewed test spans and preservation of all other source bytes. Another is a protected replacement patch product applied by a specified host step with explicit source/scope-base accounting. Preserve original blobs, replacement identities and all retained architecture assertions. Recheck every crate/app inline test location, not just the demonstrated Holla files.

## DAG-FINAL-03 — P1: candidate observation ownership contradicts the canonical repair contracts

Affected tasks: TASK-002–006, TASK-070, and production repair tasks requiring direct semantic capture.

References: decomposition-proposal.md:60,63; proof-contract.md:119,140,156; TASK-002 README R-001; TASK-070 README R-001/R-002; application README R-004, for example TASK-033:51.

The proof contract requires candidate capture with accepted observation adapters and a fixed mapping from real candidate state to oracle semantic identity. The baseline tasks explicitly implement reference adapters; TASK-070 implements orchestration. The decomposition proposal separately permits narrow implementation tasks to add candidate observations, but canonical application tasks forbid altering observation adapters. No canonical producer, product/acceptance step, or permitted source seam reconciles those instructions for candidate private state that changes during migration.

This matters beyond an absent filename: the candidate implementation must expose actual editing, focus, selection, target, overlay and domain effects without generating its own verdict. An executor currently has to infer whether observations belong in production files, completion tests, the protected adapter tree, or a reopened runner task, and how they become accepted before the same candidate's capture gate.

Repair: establish one canonical ownership rule for candidate observation seams and their independent qualification. Name the owning tasks/paths and state whether untrusted extraction code is compiled with the frozen candidate or accepted separately. Freeze schema/identity mappings and expected evidence independently; explicitly permit the required extraction-only changes without permitting candidate-authored assertions or oracle changes. Qualify wrong-state, constant-state, omitted-field, wrong-source and test-only-path counterexamples. Update the conflicting proposal and canonical prohibition together.

## DAG-FINAL-04 — P2: advertised taskfmt invocation is missing its required configuration

Affected references: proof-contract.md command examples and host `verify` invocation; task-format.md planning validation commands.

The pinned CLI `52d9f1eb7721f409bc47beb9fced7997b5c13ede` requires an experiment configuration even for standalone lint/verify. Running the documented `taskfmt lint /absolute/catalog/.../007` from this repository fails before lint with `cannot find the experiment manifest experiment.toml`. Repeating all package lints from the pinned task-format checkout passes 73/73. The existing host-bootstrap driver already supplies `--config` explicitly, and its protocol documents this requirement.

Repair: align the top-level command contract with the tested bootstrap command: explicitly supply the independently pinned configuration path/hash, or bind a precise configuration-bearing working directory. Carry that binding through the installed host environment. Do not let the executor repository supply the configuration implicitly.

## DAG-FINAL-05 — P2: authoritative synthesis still describes a superseded graph and clause count

Affected references: REFACTORING_COMPLETION_PLAN.md:156,160; TASK-007 README R-001 and trusted/obligations.md.

Independent graph calculation from current task.toml/verify.toml yields 73 tasks, maximum depth 33, 24 longest paths, and zero incomparable writable-path overlaps. TASK-066 has a hard dependency on TASK-065. The top-level plan still states depth 32, 72 deepest paths and a separate TASK-065/TASK-066 soft lock. TASK-007 also requires a `complete621-row historical union`, while the current canonical historical table and validator have 620 rows.

Repair: derive or synchronize these facts from the canonical graph and historical union after all concurrent corrections. Keep historical figures only where expressly labeled historical. Extend cross-artifact validation to reject stale authoritative counts and scheduling claims.

## Verification and review boundary

Read-only measurements:

- `rtk` 0.48.0 at `/opt/homebrew/bin/rtk`.
- Taskfmt source HEAD and CLI revision both `52d9f1eb7721f409bc47beb9fced7997b5c13ede`; fingerprint `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4`.
- Canonical package lint: 73/73 pass with the required configuration-bearing working directory. The initial run from the project directory failed configuration discovery, not package parsing.
- Independent dependency/scope calculation: 73 nodes; final depth 33; 24 longest paths; no incomparable scope overlap.
- The cross-validator run observed 1,075 source obligations, 2,703 traceability edges and five transient errors involving TASK-031's changing CHK-004 mapping. The coordinator had already identified that concurrent correction. Those transient errors are not a new finding and this review does not claim a final clean cross-validator run.

The known per-check context, host-trust, actual-Rust qualification, TASK-031 architecture-versus-oracle and cross-route staging corrections were in progress during review. They require their assigned reviewers' final evidence; this report neither repeats nor approves them. TASK-031 changed during the review, so there was no single frozen final catalog snapshot.

Recorded SHA-256 boundaries (file bytes unless otherwise stated):

| Input | SHA-256 |
| --- | --- |
| PLANNING_GOAL.md | `728c65a7a771ed2cd8ba15c78e685e889244a9fa339a18dc2dc2cde0f3d4cbe5` |
| REFACTORING_COMPLETION_PLAN.md | `91e0aa45b6ca74fd6fdaf9776776709dd5c9dde263894956c509b731cddc17ef` |
| task-index.tsv | `7efa40b0b5a2e0e6649b07d1f4a00939fb2cfc877e529b966c75f250218f7eb2` |
| task-graph.json | `49eaf7475fde8a87e2ab71f1d8945f45829baa580e6160ced79e0c339e02a802` |
| proof-contract.md | `451e6fb017f868198552a2abe49839272fb7cf4a5edd1ab8eea2ede63018ae74` |
| decomposition-proposal.md | `227b491ebebfeafdf89a0b315b3f85b2d6a5b0889f49c8ed5fd6e7a73fead8fa` |
| TASK-007 README.md | `7a56a55622c4212856c2c8a8e522dde8f85d79baaeffbb89500d81d6a6b1cbed` |
| TASK-007 verify.toml | `05ade0d28e69f119b42a2b8ddd4ef8ba2add10c8ac9d7c0cce6c219c0b585dce` |
| TASK-008 README.md | `f364ae491e03a7d61bdddd062312a4e0d783c96588a6f4f48bf12a3bd86bcc42` |
| TASK-008 verify.toml | `0dfa150dfcda639466bc4d40cfeadb4c780c4e634dec00e61cc946ed082c99e3` |
| TASK-071 README.md | `8d00a12dcdfaaab1234d53369cbb7e9e9a6d7d0dbc9f0920114d4262ccc4e8` |

At the first catalog measurement, 468 files under `refactoring-tasks/terminal-components/completion` hashed to `d0e62bea035a8f2667c7374f3ce0e9f95538b23a34fae2f0c23c06218206895d`: sort paths lexically, then hash each repository-relative UTF-8 path, NUL, file bytes, NUL. Later concurrent edits invalidate that aggregate as a final catalog identity.

Re-review must cover each finding's repaired contracts and immutable qualification fixtures, a fresh source-to-task accounting walk for preparation receipts and inline replacements, all canonical package lints, graph/overlap recomputation and a clean cross-validator run against one frozen catalog hash. Material task ownership, trust-stage or scope changes require fresh independent review before execution-ready status.
