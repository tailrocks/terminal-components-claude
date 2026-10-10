# Review receipt: commit EX-01

```yaml
schema: termrock-review-record/v1
status: approved
work_item_id: EX-01
review_stage: commit_binding
base_commit: df857eb6c62cc895d83af0311243ac9323d8d913
tree_verified: 9ab87d51a48b95ec60630827538e9b5bd2a70abe
changed_paths:
  - crates/termrock/tests/ex01_button.rs
blob: acfe5ba5adb78359227e38ce3e803fbc414f2f94
author_identity: Alexey Zhokhov <alexey@zhokhov.com>
reviewer_identity: independent reviewer
independence_evidence: Independent reviewer approved the scratch test before the commit and authored no line of it.
verdict: approved
committed_sha: eded9540db4a617eef980de3e1048899509e074d
parent_verified: df857eb6c62cc895d83af0311243ac9323d8d913
checks:
  - command: cargo nextest run -p termrock --test ex01_button
    outcome: passed
    evidence: ex01_disabled_save_without_input_paints_label
```
