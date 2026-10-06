# Work and commit review record

This is a template. It is not an approval.

```yaml
schema: termrock-review-record/v1
status: pending
work_item_id: null
review_stage: null # plan | precommit | commit_binding | work_completion
base_commit: null
proposed_tree: null
diff_sha256: null
changed_paths: []
contract_ids: []
component_owners: []
preview_consumers: []
author_identity: null
reviewer_identity: null
independence_evidence: null
findings: []
checks: [] # Each check records its command, inputs, outcome, and evidence.
artifact_digests: []
limitations: []
external_receipt: null
committed_sha: null # Set outside the reviewed commit after tree verification.
```

## Reviewer decisions

Use `changes_requested` for unresolved blocking findings.
Use `approved` only for the exact reviewed subject.
Use `not_run` for a check that did not execute.
Do not use a filled template as proof that a reviewer ran.

For documentation work, check scope, references, examples, contracts, and claims.
For implementation work, require runtime, ownership, visual, semantic, and test evidence too.
