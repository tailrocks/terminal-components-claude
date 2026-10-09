# Work queue

Generated from [accepted task records](docs/implementation/visibility/tasks.json).
Queue revision: 48. Acceptance owner: `/root`.

Highest open priority: **P0**.

| Work | Requirements | Priority | State | Dependencies | Owner | Reviewer | Allowed paths |
| --- | --- | --- | --- | --- | --- | --- | --- |
| VIS-01 | VIS-A01, VIS-S01, VIS-S02, VIS-S03, VIS-S04, VIS-S05, VIS-S06, VIS-P02 | P0 | claimed | — | /root/tag_capture_boundary_luna | /root/ci_parity_review_luna | tools/visibility/status.py, tools/visibility/tests/test_status.py, tools/visibility/source-facts.json, tools/visibility/README.md, STATUS.md, README.md, crates/termrock-visibility-tests/tests/status.rs |
| VIS-02 | VIS-T01, VIS-T02, VIS-T03, VIS-T05, VIS-T06, VIS-T07, VIS-T08, VIS-T10, VIS-V01 | P0 | claimed | — | /root/rust_test_infrastructure | /root/controls_wrapper_review_luna | crates/termrock-e2e/** |
| VIS-03 | VIS-A03, VIS-P05 | P0 | review | — | /root/showcase_owner | /root/technical_review | AGENTS.md, GOAL.md |
| VIS-04 | VIS-B01, VIS-B02, VIS-B05, VIS-I06 | P0 | claimed | — | /root | /root/jackin_inventory_current_luna | tools/visibility/deferred.py, tools/visibility/tests/test_deferred.py, docs/implementation/visibility/evidence/deferred/**, crates/termrock-visibility-tests/tests/deferred.rs |
| VIS-05 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | verified | — | /root/coordination_luna | /root/reference_luna | tools/visibility/queue.py, tools/visibility/tests/test_queue.py |
| VIS-06 | VIS-T02, VIS-T03, VIS-V01 | P0 | claimed | — | /root/rust_test_execution | /root/subjects_runner_review_luna | tools/visibility/subjects.py, docs/implementation/visibility/evidence/subjects/**, crates/termrock-visibility-tests/tests/subjects.rs |
| VIS-07 | VIS-A01, VIS-V01 | P0 | in_progress | — | /root/language_review | /root/technical_review | docs/implementation/visibility/evidence/reviews/** |
| VIS-08 | VIS-V01, VIS-P02 | P0 | in_progress | — | /root/rust_test_infrastructure | /root/technical_review | crates/termrock-visibility-tests/Cargo.toml, crates/termrock-visibility-tests/Cargo.lock, crates/termrock-visibility-tests/src/**, crates/termrock-visibility-tests/README.md |
| VIS-09 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | in_progress | — | /root/rust_test_infrastructure | /root/technical_review | crates/termrock-visibility-tests/tests/queue.rs, tools/visibility/tests/test_queue.py |
| VIS-10 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | in_progress | — | /root/coordination_luna | /root/jackin_inventory_current_luna | tools/visibility/queue.py, crates/termrock-visibility-tests/tests/queue_v2.rs, docs/implementation/visibility/queue-v2.md |
| VIS-11 | VIS-I01, VIS-I02, VIS-I03, VIS-I04, VIS-I05, VIS-I06, VIS-I07, VIS-I08, VIS-I09, VIS-I10 | P0 | in_progress | — | /root/current_branch_comparison_luna | /root/velnor_residual_review_luna | .github/workflows/**, .velnor/**, docs/implementation/visibility/evidence/ci/**, docs/implementation/visibility/evidence/tools/** |
| VIS-12 | VIS-I05, VIS-P06, VIS-P07 | P0 | review | — | /root/coordination_luna | /root/technical_review | docs/implementation/visibility/evidence/dco/** |
| VIS-13 | VIS-S06, VIS-P02 | P0 | claimed | — | /root/tag_capture_boundary_luna | /root/ci_parity_review_luna | docs/implementation/visibility/evidence/reports/** |
| VIS-14 | VIS-I07, VIS-I08, VIS-I09 | P0 | claimed | — | /root/jackin_inventory_current_luna | /root/observer_code_review_luna | Cargo.toml, Cargo.lock, crates/termrock-visibility-publisher/Cargo.toml, crates/termrock-visibility-publisher/src/lib.rs, crates/termrock-visibility-publisher/src/api.rs, crates/termrock-visibility-publisher/src/observer.rs, crates/termrock-visibility-publisher/tests/impl_observer_ingest.rs |

## Claim details

### VIS-01

Priority reason: Publish source facts and show missing paired execution and the current CI failure.

Claim token: `visibility-status-root-to-tagcapture-q35-20261009-01`; accepted at queue revision 36; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approval from /root/technical_review; exact staged review pending., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests., User requires Rust-only tests. Preserve 12 existing status assertion groups in the accepted Rust module before removing Python tests; the receipt-reader contract remains pending., Explicit owner release and independently reviewed handoff applied at revision 20; scope and base preserved; paired receipts and publication remain unqualified.

Last handoff: /root → /root/tag_capture_boundary_luna by `/root`. Recorded next action at handoff: After exact-payload independent review and Root final preconditions, apply only the VIS-01 handoff CAS at its expected queue revision. Preserve base, accepted paths, existing evidence, and claim_history. No source write authority transfers before CAS. Stop and rebind later steps on any drift.

### VIS-02

Priority reason: A shared driver for real binaries is needed before visibility can be measured.

Claim token: `visibility-e2e-root-to-rustinfra-q35-20261009-01`; accepted at queue revision 37; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent narrow pilot plan approved by /root/technical_review; exact staged review pending., User authorized /root/rust_test_infrastructure to install and execute the identical shared Termrock E2E package on refs/heads/visual-baseline at b274dd57f4dd078ade6e424d546d83efbd2e8526, limited to crates/termrock-e2e/**; keep the frozen visual-baseline tag and release unchanged., amended branch scopes at queue revision 47: previous []; replacement [{"allowed_paths":["crates/termrock-e2e/**"],"base_sha":"b274dd57f4dd078ade6e424d546d83efbd2e8526","branch":"visual-baseline"}]; observed refs/heads/visual-baseline b274dd57f4dd078ade6e424d546d83efbd2e8526; accepted plan SHA-256 d81bcb0c87ebebbaee02bc8a1fa314e50068963f24e8846889e6a18a6b5a171c; independent review SHA-256 d9578accb3177f3ba34bc382f5ddf1bfef40e242e0d0591e2980791fb68ac8f4

Additional branch scopes:
- `visual-baseline` / `b274dd57f4dd078ade6e424d546d83efbd2e8526`: crates/termrock-e2e/**

Last handoff: /root → /root/rust_test_infrastructure by `/root`. Recorded next action at handoff: After exact-payload independent review and Root final preconditions, apply only the VIS-02 handoff CAS at its expected queue revision. Preserve base, accepted paths, existing evidence, and claim_history. No source write authority transfers before CAS. Stop and rebind later steps on any drift.

### VIS-03

Priority reason: Direct agents to the visibility tasks and accepted work paths before further product refactoring.

Claim token: `visibility-instructions-20261008-03`; accepted at queue revision 3; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Narrow instruction plan approved by /root/technical_review; runtime discovery remains NOT_RUN.

### VIS-04

Priority reason: Run the deferred assertions and preserve their failure results.

Claim token: `visibility-deferred-root-q33-20261009-01`; accepted at queue revision 34; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approved by /root/technical_review; build slot not granted yet., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests.

Last handoff: /root/current_branch_comparison_luna → /root by `/root/current_branch_comparison_luna`. Recorded next action at handoff: Before any CAS, Root re-reads q33 revision, tasks.json and WORK_QUEUE.md hashes, VIS-04 current claim token/scope, exact scoped file preimages, branch/HEAD, active path conflicts, and owner-release hash. If unchanged, apply only the queue.py handoff CAS at expected revision 33. Preserve the exact accepted base, four allowed path strings, requirement and priority fields, existing evidence, existing claim history, and all other task records. No source/build/test claim is implied.

### VIS-05

Priority reason: Queue safety is needed for independent work. Its code does not depend on status publication.

Claim token: `visibility-queue-20261008-05`; accepted at queue revision 5; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approved by /root/technical_review; atomic controls not yet implemented., Integrator promoted the independent P0 queue-code prerequisite before initial publication; no file overlap or status-publication dependency., Integrator assigned the independent focused review to /root/reference_luna; owner and scope unchanged. Commit e696c4e is pushed; 34 focused tests pass., Scoped queue controls committed and independently reviewed; repository/worktree extension and runtime instruction checks remain open.

### VIS-06

Priority reason: Resolve and build actual release binaries from pinned source before paired observations.

Claim token: `visibility-subjects-root-to-rustinfra-q38-20261009-01`; accepted at queue revision 39; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `994706c6fb4a8d15fa0d5e937839fb07d0e02585`.

Evidence: Independent subject-launcher plan approved by /root/technical_review; exact schema bound before implementation; actual builds not yet run., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests.

Last handoff: /root/tablepro_owner → /root/rust_test_execution by `/root`. Recorded next action at handoff: After independent review of this exact R2 packet and Root final q38/source precondition recheck, run only: GIT_OPTIONAL_LOCKS=0 GIT_NO_LAZY_FETCH=1 GIT_NO_REPLACE_OBJECTS=1 python3 /Users/donbeave/Projects/tailrocks/terminal-components-claude/tools/visibility/queue.py handoff VIS-06 --expected-revision 38 --record /private/tmp/termrock-vis06-handoff-q38-r2-20261009-nCvN4E/handoff-record.json. The CLI successful CAS should create q39, append the prior claim snapshot plus this handoff, and set the new owner/reviewer/token/expiry; the old in_progress claim becomes claimed because dependencies are empty. Do not write source files before that CAS; stop and rebind on any revision/hash/preimage drift.

### VIS-07

Priority reason: Keep actual independent review and commit bindings in repository evidence.

Claim token: `visibility-review-archive-20261008-07`; accepted at queue revision 10; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `e696c4e6ce6ce28cecb9ed10bbbc827b5c777fc2`.

Evidence: Independent archive-only plan approved by /root/technical_review; raw review audit records confer no trust or admission., Archive exact existing external review bytes and manifest hashes; no new review decisions or qualification index.

### VIS-08

Priority reason: Use Rust tests to preserve and verify the reporting, queue, deferred, and binary-build requirements.

Claim token: `visibility-rust-test-infrastructure-20261008-08`; accepted at queue revision 12; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `effccf68d68a8ecc702bc7935b820e9c7b38b408`.

Evidence: Independent Rust test infrastructure plan approved by /root/technical_review. Synthetic tool protocols do not prove product results., Implement Rust-only shared test helpers and a compiled synthetic protocol fixture; no UI dependencies or Python test scripts.

### VIS-09

Priority reason: Preserve all queue assertions in Rust tests before further queue changes.

Claim token: `visibility-rust-queue-diagnostic-20261008-09`; accepted at queue revision 22; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `effccf68d68a8ecc702bc7935b820e9c7b38b408`.

Evidence: Independent Rust-only black-box test architecture approved by /root/technical_review. Preserve all 34 queue assertion groups before removing the Python test file., Implement Rust assertions against the actual queue CLI, with disposable Git and filesystem fixtures; no production test backdoors., Explicit VIS09 owner release and independently reviewed handoff applied at revision 22; post-CAS Rust nextest actual-record check c6a1cc34-8dd5-4646-873b-40e106899fbb passed 1 case with 33 filtered. Independent diagnostic plan v3 approved; preserve ambient failing CAS unchanged and add separate Rust controls. No lock-cause or visibility-completion claim.

Last handoff: /root/coordination_luna → /root/rust_test_infrastructure by `/root`. Recorded next action at handoff: Independent review exact payload before q21 CAS handoff; preserve base/scope/history. Transition and separately reviewed diagnostic implementation only afterward. Keep original ambient failing CAS test unchanged; add controlled Rust diagnostics under same scope. Never recreate Python tests or mutate Git/index/refs.

### VIS-10

Priority reason: Implement the repository-aware amendment prerequisite before reference and isolated tool claims.

Claim token: `visibility-queuev2-root-to-coordination-q35-20261009-01`; accepted at queue revision 38; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `85b51da2e9832dba642abf7d64d032f848cade0e`.

Evidence: Independent plan approval /private/tmp/termrock-vis10-queue-v2-plan-review.json SHA-256 bd9dbce82a27aa57608fde5d3870828765448daf53fc7a936e6cb405e3607aa0. Code prerequisite uses committed helper 8189; actual migration remains blocked until status v2 compatibility and separately qualified roots/preflight., Independent reviewed implementation scope accepted; live migration is not authorized by this transition., Integrator acceptance: reviewed VIS10 scope-amendment plan /private/tmp/termrock-vis10-scope-amendment-plan-20261009-r2.md SHA-256 509b6cefae8e062477bbd647d9d55b3afbff77e98ec2b44d44160deb4017264b; independent review /private/tmp/termrock-vis10-scope-amendment-plan-review-luna-r2.json SHA-256 e5fb2a8d9bd474cf3c087b3ccab34c5140476f6640eaeb4a82ae0e1d49ec9574 approves this exact bounded workplan. This plan supersedes the prior handoff next_action only for the next implementation sequence. Preserve the existing VIS10 base, owner, reviewer, token, expiry, allowed paths, historical handoff object, and claim history. Implement only this schema-v2 scope-amendment increment in the existing three paths. No live migration, publication, history rewrite, or out-of-scope writes are authorized., Integrator acceptance: reviewed v1 branch-scope amendment plan /private/tmp/termrock-vis02-branch-scope-amendment-plan-20261009-r3.md SHA-256 d81bcb0c87ebebbaee02bc8a1fa314e50068963f24e8846889e6a18a6b5a171c; independent review /private/tmp/termrock-vis02-branch-scope-amendment-plan-r3-review-luna.json SHA-256 d9578accb3177f3ba34bc382f5ddf1bfef40e242e0d0591e2980791fb68ac8f4. This exact bounded extension supersedes the prior schema-v2-only next increment. Keep VIS10 owner /root/coordination_luna, reviewer /root/jackin_inventory_current_luna, token, expiry, base, allowed paths, handoff, and claim history. Delegate only external implementation preparation to /root/reference_claim_luna in tools/visibility/queue.py, crates/termrock-visibility-tests/tests/queue_v2.rs, and docs/implementation/visibility/queue-v2.md; Root integrates. No live v2 migration, tag/release mutation, reference writes, or readiness promotion. Implement and verify the bounded v1 branch-scope CAS with Rust/Nextest tests before granting VIS02 reference scope., Root observed the unchanged origin SSH reference probe exceed its 10-second bound without stdout or stderr (operational diagnostic session 62932, SIGKILL after deadline). Current fixed reference remains b274dd57f4dd078ade6e424d546d83efbd2e8526. Record the transport block while preserving owner, token, original plan, and allowed paths; no tag, remote configuration, or release change., Root accepts transport addendum /private/tmp/termrock-vis02-branch-scope-transport-addendum-r1-20261009.md SHA-256 4949d127f98b3e9125d50e63e895dc4486cc074b0d87d6faba628b1d7839646f; focused review /private/tmp/termrock-vis02-v1-scope-transport-delta-independent-review-luna-r1.json SHA-256 e8b17392d437740520a8fa6a09ddba445259f73c1284e44474b59edeea91eaa7. Resume the existing VIS-10 scope and delegated /root/reference_claim_luna external preparation only: add process-scoped credential.helper= and exact canonical SSH-to-same-repository HTTPS insteadOf mapping to the bounded origin probe. Parent diagnostic with this mapping returned exact b274dd57f4dd078ade6e424d546d83efbd2e8526 and empty stderr, exit 0. Preserve original VIS-02 primary/base/path authority and immutable tag/release; do not change repository or global Git settings. Twelve focused Rust tests remain NOT_RUN, and no VIS-02 reference grant or readiness is accepted.

Last handoff: /root → /root/coordination_luna by `/root`. Recorded next action at handoff: After exact-payload independent review and Root final preconditions, apply only the VIS-10 handoff CAS at its expected queue revision. Preserve base, accepted paths, existing evidence, and claim_history. No source write authority transfers before CAS. Stop and rebind later steps on any drift.

### VIS-11

Priority reason: The user explicitly requested repair of PR #17 CI/CD; preserve required visibility and product gates.

Claim token: `visibility-pr17-ci-repair-comparison-q39-20261009-01`; accepted at queue revision 40; expires `2026-10-09T08:00:00Z`.

Branch/base: `termrock-implementation` / `766ae1e925e32b4b28aa9100bb3fde5279aa223b`.

Evidence: User explicitly authorized PR #17 CI/CD repair, including parallel subagents. Root remains the branch integration owner., Required evidence: live provider failure details; reviewed generator source/tool identity; generated workflows without manual YAML changes; workflow admission and actual branch/PR runs; failure artifact publication and separate visibility/readiness gates., Local base 766ae is the preserved checkout anchor, not the current remote product/test identity. Workflow generation must use independently pinned current remote source inputs and preserve newer work., Root accepts the independently reviewed R6 legacy-260 subset comparator plan /private/tmp/termrock-vis11-r6-260-subset-comparator-plan-r1-20261009.md SHA-256 0e3a5ef6e93f00445ccc93002419b73c9de850df50ecc06c399253c71b83a395; review /private/tmp/termrock-vis11-r6-260-subset-comparator-plan-review-luna-r1-20261009.json SHA-256 c950b11372947c6d581045badb75b8be3fddddcebe0267a1e9afd5be2cb5e3ce. Existing owner /root/current_branch_comparison_luna may implement only external successor /private/tmp/termrock-vis11-live-parity-r6-260-subset-20261009/src/lib.rs, src/main.rs, and README.md. Preserve all 260 baseline tasks and 44 product plus 4 infrastructure jobs as required subset; additions require reviewed full job/task/selector lane digests, with no total cap. Require missing infrastructure job, missing receipt, and missing expected-pin negative Rust regressions. No workflow parity, test execution, release, or readiness is accepted by this plan., Independent source review found the R6 comparator actively resolves prepare() from the old ca3ef Cargo dependency while its receipt may select a newer Velnor source. Preserve all existing claims and required legacy tasks; no comparator parity or tool qualification is accepted., Root accepts external comparator Cargo binding addendum /private/tmp/termrock-vis11-r6-comparator-cargo-source-binding-plan-addendum-r1-20261009.md SHA-256 6dd6dfeb2d9ba4ddfcda8dcec704b51b74a674f8ef0fca2d9a5a5baa9ddc1210 and independent review /private/tmp/termrock-vis11-r6-comparator-cargo-source-binding-plan-review-luna-r1-20261009/review.json SHA-256 4a8abba3707b9e9c4203b3a2914189558ccf0f981c4a01e23a656ac2dc55c742. Existing owner /root/current_branch_comparison_luna may add Cargo.toml to the three previously accepted external R6 paths only. Bind the comparator executable digest, actual Cargo-resolved generator package path and compilation-relevant source closure, reviewed Velnor producer artifacts, and exact invocation receipt digest through the measured Root runner. Freeze selected immutable generator inputs before build. No source build, comparator execution, generated workflow parity, release, or readiness is accepted by this plan.

Last handoff: /root/status_recovery_luna → /root/current_branch_comparison_luna by `/root/status_recovery_luna`. Recorded next action at handoff: Do not apply this proposal until /root/current_branch_comparison_luna confirms availability, /root/velnor_residual_review_luna completes independent review of this exact payload, and /root repeats the q39 tasks/view/queue.py hash, exact VIS-11 claim, allowed-path clean diff, active-writer overlap, and recipient/reviewer availability checks. Root alone applies the canonical queue-v1 reassign CAS at expected q39, preserving the existing branch, base, requirements, priority, all four allowed paths, existing evidence, and claim history; only owner, reviewer, claim token, expiry, handoff history, and the queue revision change. After CAS, the recipient must coordinate the required VIS-04 deferred `--claim-pin` caller update before relying on that contract. Current CI remains unqualified until a compact generated workflow is independently source/tool bound and passes workflow admission and actual branch/PR runs. Preserve the required 260 task identities and 44 product plus 4 infrastructure jobs. The R5 260-row ID/environment/dependency/output mapping remains unaccepted and full parity unestablished. Do not hand-edit generated YAML or expand to `.github/workflows/actionlint.yaml` without a separate accepted scope amendment.

### VIS-12

Priority reason: The user explicitly authorized limited DCO commit repairs and guarded force push for PR #17.

Claim token: `visibility-pr17-dco-repair-20261009-0136Z`; accepted at queue revision 31; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `766ae1e925e32b4b28aa9100bb3fde5279aa223b`.

Evidence: User explicitly authorizes conflict resolution and force push only to fix offending past DCO commits, without replacing or dropping unrelated history., Git operation scope: repair only confirmed offending remote PR commit messages; preserve every file tree, original authorship, topology, and commit order; rebuild descendants only as required by changed parent identities. Root alone integrates and pushes refs/heads/termrock-implementation with an exact-head lease. Preserve immutable visual tag/release and local HEAD/index., Required evidence: live DCO offending commit IDs; exact old/new mapping; all tree and metadata preservation checks; independent repair review; recoverable old head; exact remote lease; refreshed PR DCO result., The allowed file paths are evidence publication scope. The Git metadata operation is separately authorized above; this record does not confer a general history rewrite permission., PR #17 limited DCO repair was published at remote head 6819529c2d24d871ba1c251988d8544cfaa82c51 from old head 28058beecf63acee3b65bab847e507fe8e1c539d. The old and new tree is 3a58036c595688898fc989ef6453e6d9d512f230; the R2 map records 83 recreated commit objects and four message edits. GitHub DCO check 113639531062 completed success. The all-history audit still records 16 older unsigned commits; product visibility remains unaccepted and refactor readiness remains not ready. Evidence packet: docs/implementation/visibility/evidence/dco/20261009-pr17-repair/., Independent post-push actual review /private/tmp/termrock-pr17-dco-postpush-actual-review-technical.json (SHA-256 e931261240829fa67a814c102ce35b6499bf8c3f0ff90dd631d4319d064a9369) verified the 83 actual projected objects and 82 trees against the map, and bound the supplied remote/tag and DCO-success captures. Current VIS-12 allowed-path subject digest is 29b789040a1107e1ab0e465af4a3d590d3c56237919a546bd8b616eb69661ad1. Verification remains pending because the accepted task source has no review_subject_sha256 field.

### VIS-13

Priority reason: Provide the repository-portable, hash-checked current-observation archive needed by VIS-01 status evidence; retain current failure and NOT_RUN states without promoting readiness.

Claim token: `visibility-status-archive-q34-20261009-01`; accepted at queue revision 35; expires `2026-10-09T06:00:00Z`.

Branch/base: `termrock-implementation` / `766ae1e925e32b4b28aa9100bb3fde5279aa223b`.

Evidence: VIS-01 current-observation portability R3 bundle proposes a 50-raw-file archive totaling 3,993,515 bytes, manifest SHA-256 95c44bc19d03e1d17252e7bdf3f146060057f11e2fed34bcbf161c26286f3429; this is not yet a repository archive or qualification., VIS-13 is a repository-artifact prerequisite for VIS-01. Its output target is docs/implementation/visibility/evidence/reports/status-source-archive-20261009/**, within this claim scope. VIS-01 must later bind its renderer/facts to this target; the external R3 patch currently names a different target under evidence/status-source-archive-20261009., Applicable existing requirements: VIS-S06 (current receipts rather than historical claims) and VIS-P02 (canonical queue and accepted claims).

### VIS-14

Priority reason: Provide bounded read-only Rust ingestion that resolves main-CI and observer identities, distinguishes failures/missing or cancelled work, and validates observer artifacts as inert data. This is an independently useful prerequisite to trusted report projection; it does not publish STATUS or write Git refs.

Claim token: `visibility-vis14-t1-readonly-ingest-q47-20261009-1007z`; accepted at queue revision 48; expires `2026-10-09T13:00:00Z`.

Branch/base: `termrock-implementation` / `766ae1e925e32b4b28aa9100bb3fde5279aa223b`.

Evidence: Scope is exactly T1_READ_ONLY_INGEST in the accepted publisher split plan; the seven allowed paths above are the seven plan paths with descriptive '(new)' and '(workspace membership only)' annotations removed., Accepted plan: /private/tmp/termrock-vis11-reusable-publisher-split-r2-20261009/plan.json (SHA-256 d038e674baa6d69c22b05ad22a836183c262bb3605a0353ac44b0ac5c34e7b3f)., Independent plan review READY: /private/tmp/termrock-vis11-reusable-publisher-split-r2-review-observer-20261009.json (SHA-256 3af09e15f6ee96d02730557253461ddceb3b40b0cac2d0ec785202f81857fb22)., T1 is read-only: it has no Git Data write endpoint, no STATUS/report projection, and does not check out, extract, or execute candidate source or artifact content., The empty dependency list permits bounded ingestion work to start independently. V1 and initial T1 may run in parallel on their disjoint accepted path sets; the caller-identity test and T1 closure remain pending until V2 supplies the exact reviewed caller bytes. Do not mark VIS-14 verified or close T1 before binding those bytes., Root Cargo.toml is limited to workspace membership and root Cargo.lock to the locked T1 dependencies, exactly as scoped by the reviewed plan. A nested standalone workspace would require a separate crate Cargo.lock outside the seven paths, so it is not selected under this exact claim. No source edit or Cargo command may occur before Root accepts this claim., Queue overlap source: q47 tasks snapshot /private/tmp/termrock-vis02-q47-publication-proposal-fd9165-20261009/after/q47-tasks.json (SHA-256 178551e7546be481f33149834711ea18e5fb4ee40af4e8a9fb6d03dc03847d97) and WORK_QUEUE snapshot /private/tmp/termrock-vis02-q47-publication-proposal-fd9165-20261009/after/q47-WORK_QUEUE.md (SHA-256 59885ad044a1a92a5e21a00d825c0719929e90f7d133a2735583551d65f00489); actual q47 Root publication record /private/tmp/termrock-q47-root-publication-20261009.json (SHA-256 7fa526b1e53b4de0e2df2bf87e5a27d8e546710abb20d41ad2bb5122f38eca15) records commit and remote readback fa95b5acb6ad95150c8d6e7ce1f7e80a58a7f522. The one-file source-pin proposal /private/tmp/termrock-vis02-q48-pin-proposals-20261009/q47-pin-r1/proposal.json (SHA-256 d8333a9dbc6e146d34dfb5a6b160b29832ae8e5d3e193d9cbfa0f6fbfad701d2) changed only crates/termrock-e2e/src/lib.rs; its actual publication record /private/tmp/termrock-vis02-pin-root-publication-20261009.json (SHA-256 0e53bfc0c08b2e217a4ae47551689add33d4667bf9cb838cbcb65f93107c7c6e) records commit and remote readback 6a6ba79616b948b38b3c23317860290c96edb631, so it does not overlap these seven paths. At q47 the snapshot contains 13 tasks and no VIS-14; no existing scope includes root Cargo.toml, root Cargo.lock, or the new publisher crate. Root must re-read the live queue and recheck overlap immediately before acceptance. These external snapshots are evidence, not accepted queue authority., Root's queue acceptance requires this claim's base_sha to match the actual local checkout HEAD 766ae1e925e32b4b28aa9100bb3fde5279aa223b. That is the acceptance checkout anchor, not a claim that it is the published source-facts tip. The source plan's published termrock pin is commit 6a6ba79616b948b38b3c23317860290c96edb631, recorded at /private/tmp/termrock-vis02-pin-root-publication-20261009.json (SHA-256 0e53bfc0c08b2e217a4ae47551689add33d4667bf9cb838cbcb65f93107c7c6e), a child of fa95b5acb6ad95150c8d6e7ce1f7e80a58a7f522. Comparing only the seven allowed paths at local base 766 and published source pin 6a: Cargo.toml is blob b8b7f80d574ff4388fd638a046b410aad1bcd582 at both; Cargo.lock is blob 2d50e18232ef0541d7e5f85262ad6cbeb7c9c031 at both; the five new publisher crate paths are absent at both. This limited path comparison shows the scoped preimages match; it does not claim the full trees match. If Root accepts this claim, implementation must start from the actual published 6a source tree and recheck these seven scoped preimages before editing.

Expiry does not release an active claim. Reassignment requires a recorded safe handoff.
Task state is assignment state; it does not prove test results or requirement completion.
