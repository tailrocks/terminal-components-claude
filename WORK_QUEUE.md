# Work queue

Generated from [accepted task records](docs/implementation/visibility/tasks.json).
Queue revision: 26. Acceptance owner: `/root`.

Highest open priority: **P0**.

| Work | Requirements | Priority | State | Dependencies | Owner | Reviewer | Allowed paths |
| --- | --- | --- | --- | --- | --- | --- | --- |
| VIS-01 | VIS-A01, VIS-S01, VIS-S02, VIS-S03, VIS-S04, VIS-S05, VIS-S06, VIS-P02 | P0 | in_progress | — | /root/status_recovery_luna | /root/technical_review | tools/visibility/status.py, tools/visibility/tests/test_status.py, tools/visibility/source-facts.json, tools/visibility/README.md, STATUS.md, README.md, crates/termrock-visibility-tests/tests/status.rs |
| VIS-02 | VIS-T01, VIS-T02, VIS-T03, VIS-T05, VIS-T06, VIS-T07, VIS-T08, VIS-T10, VIS-V01 | P0 | in_progress | — | /root/suite_luna | /root/technical_review | crates/termrock-e2e/** |
| VIS-03 | VIS-A03, VIS-P05 | P0 | review | — | /root/showcase_owner | /root/technical_review | AGENTS.md, GOAL.md |
| VIS-04 | VIS-B01, VIS-B02, VIS-B05, VIS-I06 | P0 | claimed | — | /root/current_branch_comparison_luna | /root/holla_owner | tools/visibility/deferred.py, tools/visibility/tests/test_deferred.py, docs/implementation/visibility/evidence/deferred/**, crates/termrock-visibility-tests/tests/deferred.rs |
| VIS-05 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | verified | — | /root/coordination_luna | /root/reference_luna | tools/visibility/queue.py, tools/visibility/tests/test_queue.py |
| VIS-06 | VIS-T02, VIS-T03, VIS-V01 | P0 | in_progress | — | /root/tablepro_owner | /root/technical_review | tools/visibility/subjects.py, docs/implementation/visibility/evidence/subjects/**, crates/termrock-visibility-tests/tests/subjects.rs |
| VIS-07 | VIS-A01, VIS-V01 | P0 | in_progress | — | /root/language_review | /root/technical_review | docs/implementation/visibility/evidence/reviews/** |
| VIS-08 | VIS-V01, VIS-P02 | P0 | in_progress | — | /root/rust_test_infrastructure | /root/technical_review | crates/termrock-visibility-tests/Cargo.toml, crates/termrock-visibility-tests/Cargo.lock, crates/termrock-visibility-tests/src/**, crates/termrock-visibility-tests/README.md |
| VIS-09 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | in_progress | — | /root/rust_test_infrastructure | /root/technical_review | crates/termrock-visibility-tests/tests/queue.rs, tools/visibility/tests/test_queue.py |
| VIS-10 | VIS-P02, VIS-P03, VIS-P04, VIS-P05, VIS-V01 | P0 | in_progress | — | /root/registry_check_luna | /root/jackin_inventory_current_luna | tools/visibility/queue.py, crates/termrock-visibility-tests/tests/queue_v2.rs, docs/implementation/visibility/queue-v2.md |

## Claim details

### VIS-01

Priority reason: Publish source facts and show missing paired execution and the current CI failure.

Claim token: `visibility-status-recovery-20261008-01`; accepted at queue revision 20; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approval from /root/technical_review; exact staged review pending., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests., User requires Rust-only tests. Preserve 12 existing status assertion groups in the accepted Rust module before removing Python tests; the receipt-reader contract remains pending., Explicit owner release and independently reviewed handoff applied at revision 20; scope and base preserved; paired receipts and publication remain unqualified.

Last handoff: /root/status_luna → /root/status_recovery_luna by `/root`. Next: Before applying, /root independently repeats the q19 record/hash, seven-path clean-diff, accepted-writer overlap, and actual owner-release checks, then obtains narrow independent review of this exact payload. Apply queue-v1 reassign at expected revision 19. Preserve the existing accepted base cc3ce8f6ac149aaee406047c5180d1a658d6bb9c, seven allowed paths, evidence, and claim history. Continue only in that scope; keep product rows NOT_RUN without validated receipts. Do not add or run Python tests; use Rust tests through cargo nextest only. Any collector integration requires a separate accepted scope amendment. Do not create a new claim from current HEAD 186c287 while history correction is pending, because it would orphan the accepted base.

### VIS-02

Priority reason: A shared driver for real binaries is needed before visibility can be measured.

Claim token: `visibility-suite-20261008-02`; accepted at queue revision 2; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent narrow pilot plan approved by /root/technical_review; exact staged review pending.

### VIS-03

Priority reason: Direct agents to the visibility tasks and accepted work paths before further product refactoring.

Claim token: `visibility-instructions-20261008-03`; accepted at queue revision 3; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Narrow instruction plan approved by /root/technical_review; runtime discovery remains NOT_RUN.

### VIS-04

Priority reason: Run the deferred assertions and preserve their failure results.

Claim token: `visibility-deferred-recovery-20261008-04`; accepted at queue revision 24; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approved by /root/technical_review; build slot not granted yet., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests.

Last handoff: /root/deferred_execution → /root/current_branch_comparison_luna by `/root`. Next: After independent approval and root's final CAS checks, transfer the in-progress VIS-04 assignment to the proposed owner without changing base, allowed paths, or historical claim/evidence. The queue CLI will set the successor state to claimed. Continue from current local history; preserve the historical 23-fail result as red evidence and do not claim a current pass.

### VIS-05

Priority reason: Queue safety is needed for independent work. Its code does not depend on status publication.

Claim token: `visibility-queue-20261008-05`; accepted at queue revision 5; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`.

Evidence: Independent plan approved by /root/technical_review; atomic controls not yet implemented., Integrator promoted the independent P0 queue-code prerequisite before initial publication; no file overlap or status-publication dependency., Integrator assigned the independent focused review to /root/reference_luna; owner and scope unchanged. Commit e696c4e is pushed; 34 focused tests pass., Scoped queue controls committed and independently reviewed; repository/worktree extension and runtime instruction checks remain open.

### VIS-06

Priority reason: Resolve and build actual release binaries from pinned source before paired observations.

Claim token: `visibility-subjects-20261008-06`; accepted at queue revision 7; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `994706c6fb4a8d15fa0d5e937839fb07d0e02585`.

Evidence: Independent subject-launcher plan approved by /root/technical_review; exact schema bound before implementation; actual builds not yet run., Integrator amended the scope at queue revision 14 for the user requirement: all test code and execution use Rust and cargo nextest. Preserve prior assertions before removing Python tests.

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

Last handoff: /root/coordination_luna → /root/rust_test_infrastructure by `/root`. Next: Independent review exact payload before q21 CAS handoff; preserve base/scope/history. Transition and separately reviewed diagnostic implementation only afterward. Keep original ambient failing CAS test unchanged; add controlled Rust diagnostics under same scope. Never recreate Python tests or mutate Git/index/refs.

### VIS-10

Priority reason: Implement the repository-aware amendment prerequisite before reference and isolated tool claims.

Claim token: `visibility-registry-check-20261008-10`; accepted at queue revision 25; expires `2026-10-09T00:00:00Z`.

Branch/base: `termrock-implementation` / `85b51da2e9832dba642abf7d64d032f848cade0e`.

Evidence: Independent plan approval /private/tmp/termrock-vis10-queue-v2-plan-review.json SHA-256 bd9dbce82a27aa57608fde5d3870828765448daf53fc7a936e6cb405e3607aa0. Code prerequisite uses committed helper 8189; actual migration remains blocked until status v2 compatibility and separately qualified roots/preflight., Independent reviewed implementation scope accepted; live migration is not authorized by this transition., Integrator acceptance: reviewed VIS10 scope-amendment plan /private/tmp/termrock-vis10-scope-amendment-plan-20261009-r2.md SHA-256 509b6cefae8e062477bbd647d9d55b3afbff77e98ec2b44d44160deb4017264b; independent review /private/tmp/termrock-vis10-scope-amendment-plan-review-luna-r2.json SHA-256 e5fb2a8d9bd474cf3c087b3ccab34c5140476f6640eaeb4a82ae0e1d49ec9574 approves this exact bounded workplan. This plan supersedes the prior handoff next_action only for the next implementation sequence. Preserve the existing VIS10 base, owner, reviewer, token, expiry, allowed paths, historical handoff object, and claim history. Implement only this schema-v2 scope-amendment increment in the existing three paths. No live migration, publication, history rewrite, or out-of-scope writes are authorized.

Last handoff: /root/coordination_luna → /root/registry_check_luna by `/root`. Next: After independent handoff review and integrator CAS, implement only reviewed registry-check-v2 plan r3; preserve original branch/base/scope/history. No publication or history rewrite is authorized.

Expiry does not release an active claim. Reassignment requires a recorded safe handoff.
Task state is assignment state; it does not prove test results or requirement completion.
