# Review receipt: commit FIX-usage-overview

```yaml
schema: termrock-review-record/v1
status: approved
work_item_id: FIX-usage-overview
review_stage: commit_binding
base_commit: 1d81a803438c2943485eda2bd9673a4baf7c1ea7
tree_verified: 247a513ea8c880de09b265536d6bfa914f3f81f9
changed_paths:
  - crates/jackin-preview-host-ui/src/usage.rs
  - crates/jackin-preview-app/src/app.rs
  - crates/jackin-preview-app/src/app/historical_accounts_settings_usage.rs
  - crates/jackin-preview-app/tests/usage_overview.rs
author_identity: Alexey Zhokhov <alexey@zhokhov.com>
reviewer_identity: independent reviewer
independence_evidence: Independent reviewer approved REVIEW2 before integration and authored no line of the change.
verdict: approved
committed_sha: ceaaa3bd156b6b2fae09d746cf0a9c6742714fd7
parent_verified: 1d81a803438c2943485eda2bd9673a4baf7c1ea7
sign_off: "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>"
checks:
  - command: main-tree nextest, termrock-conformance visual_baseline jackin_usage_overview_120x40_truecolor
    outcome: passed
    evidence: 1 passed, 800 skipped
  - command: jackin-preview-app usage_overview
    outcome: passed
    evidence: 2 tests passed
```

Not a merge. `crates/jackin-preview-host-ui/src/usage.rs` has no `paint_str`. Overview no longer calls `draw_historical_usage_overview_120_40`. Usage detail historical return remains. This receipt does not claim a push succeeded. This receipt does not claim the refactor is finished.
