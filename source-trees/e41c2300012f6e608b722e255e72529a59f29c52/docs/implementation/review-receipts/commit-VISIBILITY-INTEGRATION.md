# Review receipt: VISIBILITY-INTEGRATION

```yaml
schema: termrock-review-record/v1
receipt_id: VISIBILITY-INTEGRATION
receipt_status: recorded
generated_at: "2026-10-10T11:52:32+07:00"
branch_verified: termrock-implementation
head_verified: 09a0133e1dbda08d6483f8c0d1696e20d6cc9851
work_queue_revision_verified: 71
highest_open_priority_verified: P0
purpose: Bind four independently reviewed commits to their exact Git identities and reviewer outcomes.
allowed_write_path: docs/implementation/review-receipts/commit-VISIBILITY-INTEGRATION.md
active_visibility_prerequisite: open
receipt_does_not_close_visibility_prerequisite: true
identity_commands:
  - git cat-file -t COMMIT
  - git show -s --format="%H %T %P" COMMIT
  - git diff PARENT COMMIT --binary | sha256sum
  - git diff PARENT COMMIT --binary --full-index | sha256sum
review_source: coordinator_supplied_review_results
review_evidence_limitation: Reviewer names and subagent IDs were supplied to the receipt writer; repository evidence available here does not independently recompute reviewer isolation or non-authorship.
diff_digest_definitions:
  diff_sha256: SHA-256 of git diff PARENT COMMIT --binary
  diff_binary_sha256: SHA-256 of git diff PARENT COMMIT --binary --full-index
reviews:
  - order: 1
    committed_sha: 1d81a803438c2943485eda2bd9673a4baf7c1ea7
    subject: "chore(visibility): integrate shared binary test controls"
    parent_verified: f34cbe0be5362ad46b28b60743891b6d2f162e2e
    tree_verified: e336d8de12b7c49a51629b18a68759cc2ab167a3
    diff_sha256: aea5daab0045e4efa202d7b9c6c9ffd14952bec73724630cf579fef0683401aa
    diff_binary_sha256: 92a94c576577981f10ddd149e7c4c700bf6afed24dc5c8d0b64816e06efe290b
    changed_path_count: 811
    author_identity: Alexey Zhokhov <alexey@zhokhov.com>
    committer_identity: Alexey Zhokhov <alexey@zhokhov.com>
    sign_off_verified: "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
    reviewer: Peirce
    reviewer_subagent_id: 01a12342-5ffa-7092-957a-131a8853ec90
    verdict: approved_with_cas_conditions
    approval_conditions:
      kind: CAS
      detail: The reviewer approved with CAS conditions; expanded condition text was not supplied to this receipt writer.
    historical_recorded_checks:
      - command: cargo nextest e2e
        claimed_result: 114 passed, 9 skipped
        receipt_recomputation: not_run
        reason: The current descendant tree changed checked sources; no exact historical execution snapshot was created for this receipt.
      - command: cargo nextest visibility
        claimed_result: 191 passed, 3 skipped
        receipt_recomputation: not_run
        reason: The current descendant tree changed checked sources; no exact historical execution snapshot was created for this receipt.
      - command: cargo nextest publisher
        claimed_result: 45 passed
        receipt_recomputation: not_run
        reason: The current descendant tree changed checked sources; no exact historical execution snapshot was created for this receipt.
      - command: cargo nextest EX-01
        claimed_result: 1 passed
        receipt_recomputation: not_run
        reason: Later descendants replaced or changed the EX-01 test subject.
  - order: 2
    committed_sha: c597b3ba63fdcf2a2cfc41a1e12399cd02be75eb
    subject: "chore(visibility): reconcile implementation ancestry"
    parents_verified:
      - 60340e56ee3551d8210cdb58f9189cfee605da2c
      - edbe7c86e0025929bc3c481a136f90dd9027271a
    tree_verified: 0544461692f96d0a240f59b29ddbea47655b6a01
    diff_sha256: null
    diff_binary_sha256: null
    diff_digest_status: no merge diff digest was supplied or claimed; this receipt binds the two parents and exact result tree instead
    audit_only_first_parent_diff_binary_sha256: faa57ef5f61a0f504df8e362276f9d23ab25060c6c2c50e133ea6259a44ffe4c
    first_parent_changed_path_count: 293
    author_identity: Alexey Zhokhov <alexey@zhokhov.com>
    committer_identity: Alexey Zhokhov <alexey@zhokhov.com>
    sign_off_verified: "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
    reviewer: Lovelace
    reviewer_subagent_id: 01a1237b-c560-7632-9ba6-c9e4fe6d33ac
    verdict: approved_exact_cas_merge
    approval_scope: exact CAS merge
    ancestry_checks:
      first_parent_is_ancestor: passed
      second_parent_is_ancestor: passed
      commit_is_ancestor_of_current_termrock_implementation: passed
    historical_recorded_checks:
      - command: cargo nextest status contract
        claimed_result: 53 passed, 0 failed
        receipt_recomputation: not_run
        reason: A later commit changed the status contract sources; no exact historical execution snapshot was created for this receipt.
  - order: 3
    committed_sha: 2e2ec73b9c586fbf3b8b5c479f9e19cbb242128a
    subject: "fix(visibility): scope branch grants out of v2 migration"
    parent_verified: 72b9baaa6db774f577ab907e89b0b46fa3d588b3
    tree_verified: 7ab82d1d80cab4702f8b3222716e205752148fb4
    diff_sha256: bd86ea4133c718cda11a2e523ff27937ecaf3b22ec0c8bffec62c5d6c3859adc
    diff_binary_sha256: 263159b321a86a559a26daff99b8b0bc21481bee28f7ea34ba60d0984c82395f
    changed_path_count: 3
    changed_paths:
      - crates/termrock-visibility-tests/tests/queue_v2.rs
      - docs/implementation/visibility/queue-v2.md
      - tools/visibility/queue.py
    author_identity: Alexey Zhokhov <alexey@zhokhov.com>
    committer_identity: Alexey Zhokhov <alexey@zhokhov.com>
    sign_off_verified: "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
    reviewer: Beauvoir
    reviewer_subagent_id: 01a123f5-6671-76b0-85a8-42f81f2a0abb
    verdict: ready
    recomputation_basis: Current termrock-visibility-tests sources match this commit through HEAD 09a0133e1dbda08d6483f8c0d1696e20d6cc9851 for the checked suites.
    recomputed_checks:
      - command: cd crates/termrock-visibility-tests && cargo nextest run --test queue_v2
        outcome: passed
        evidence: 85 tests run, 85 passed, 0 failed, 0 skipped
        nextest_run_id: aa556cad-b092-47a5-ac25-cc8350a5ce3c
      - command: cd crates/termrock-visibility-tests && cargo nextest run --test status
        outcome: passed_with_leak_advisory
        evidence: 59 tests run, 59 passed, 0 failed, 0 skipped; Nextest labeled one passing process LEAK
        nextest_run_id: d2e01028-869b-4c6e-8ec9-0eeee2cc6d6d
  - order: 4
    committed_sha: 09a0133e1dbda08d6483f8c0d1696e20d6cc9851
    subject: "test: add external termrock consumer recipes"
    parent_verified: 2e2ec73b9c586fbf3b8b5c479f9e19cbb242128a
    tree_verified: 6d5b3dba5bc84c4a6d0d07748e78d579cf597de9
    diff_sha256: 276844baeb4a31dcaaba013e84015df59e417c87988f694f54d70fd15b7f4741
    diff_binary_sha256: 736cb9be3eccda895283277a53a17f119c5e2b7d936f9ee92ae2be0175b8a394
    changed_path_count: 2
    changed_paths:
      - crates/termrock/tests/ex01_button.rs
      - crates/termrock/tests/ex02_text_input.rs
    author_identity: Alexey Zhokhov <alexey@zhokhov.com>
    committer_identity: Alexey Zhokhov <alexey@zhokhov.com>
    sign_off_verified: "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
    reviewer: Gibbs
    reviewer_subagent_id: 01a12418-f033-7a92-a353-210dbc92c351
    verdict: ready
    recomputation_basis: This commit is HEAD and its tree was current when the nextest check ran.
    recomputed_checks:
      - command: cargo nextest run -p termrock --test ex01_button --test ex02_text_input
        outcome: passed
        evidence: 2 tests run, 2 passed, 0 failed, 0 skipped
        nextest_run_id: 16eef443-d0b6-4449-abe7-1fa7d9d2fc26
    historical_recorded_checks:
      - command: cargo fmt
        claimed_result: passed
        receipt_recomputation: not_run
        reason: The requested recompute scope was read-only Git and cargo nextest.
      - command: cargo check
        claimed_result: passed
        receipt_recomputation: not_run
        reason: The requested recompute scope was read-only Git and cargo nextest.
frozen_visual_authority_check:
  tag: visual-baseline
  tag_object_verified: 1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5
  tag_object_type_verified: tag
  commit_verified: 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
  commit_type_verified: commit
  target_commit_verified: 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
scope_limit: This file records only the four exact commits listed above.
generic_historical_receipts: Receipts for generic historical commits remain untracked and are not approved by this file.
```

This receipt binds supplied review outcomes and recomputed Git identities. It is not a new independent review.
