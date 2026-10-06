# Review each work item and each commit

## REV-001 — Independent reviewer

The reviewer must not author the submitted change.
A worker can review another worker's change only if the reviewer remains independent for that subject.
Do not use a second persona in the same unverified execution as proof of independence.
A unit test named `review` is not a reviewer.

An approval covers one identified subject and one declared scope.
It does not approve an entire future refactor.

## REV-002 — Review before work

Give the reviewer the work item and source references.
State the intended component owner and preview consumers.
State the allowed paths and expected evidence.
Identify any API change before writing its implementation.

The reviewer checks whether the plan solves the problem in the correct layer.
Reject a plan that repairs pixels through a preview painter.
Record a small approved work plan before execution.

For documentation work, review the proposed authority map and document scope.
Do not require the reviewer to approve known-unfixed implementation code.

## REV-003 — Prepare each commit

Keep a commit focused on one coherent change.
Include related tests and usage for future implementation changes.
Do not combine unrelated app repairs under a narrow title.

Stop edits to the staged subject before review.
Record the parent commit and proposed Git tree.
Hash the exact binary staged diff.
Record changed paths and relevant external evidence.

Use these commands to identify the review subject:

```sh
git rev-parse HEAD
git write-tree
git diff --cached --binary --full-index
```

These commands identify the subject. They do not prove approval.
`git write-tree` may create a Git object. It does not change the staged source.
`git write-tree` may create a Git object. It does not change the staged source.
Hash the exact diff bytes with the available SHA-256 tool.
Do not hash a display that truncates long lines or omits binary content.
Do not let another worker change the shared index during review.

## REV-004 — Review each proposed commit

The reviewer checks every changed line and its relevant context.
The reviewer follows the path from application data to final output.
Generated code receives the same ownership review as handwritten code.
A large generated patch is not an exception.

The review must answer:

| Question | Required evidence |
| --- | --- |
| Does the change fit this work item? | Full changed-path list and contract IDs. |
| Is the component the correct owner? | Data, update, draw, and consumer call paths. |
| Is normal use simple? | A normal public API example, not internal setup. |
| Does customization preserve behavior? | Parts, clipping, state, and action ownership. |
| Does default output match the oracle? | Relevant exact comparison results. |
| Does interaction still work? | Real events, outcomes, and negative paths. |
| Does the result generalize? | Changed data, noncanonical geometry, and paused input. |
| Are the tests independent? | Expected-data provenance and candidate isolation. |
| Are claims accurate? | Actual commands, result IDs, hashes, and limitations. |

For documentation-only commits, substitute document checks for implementation checks.
Check scope, links, examples, authority consistency, and proposed-versus-existing status.
List runtime verification as not required for the document diff, or not run.
Do not turn that status into implementation approval.

## REV-005 — Findings and approval

Use `changes_requested` when a blocking requirement fails.
The author fixes the change.
The reviewer reviews the changed subject again.

Do not let a matching screenshot close an ownership finding.
Do not close a finding by weakening the requirement or removing a test.
Nonblocking findings need a named owner and an explicit disposition.

An approval records the exact parent, proposed tree, diff digest, review scope, and reviewer result.
If staged content changes, invalidate the approval.
If the base changes, review the new parent and integration context.

## REV-006 — Commit identity

Create the commit only after approval.
Verify the committed parent and tree against the approved subject.
If a hook changed the content, obtain another review before accepting or pushing the commit.
Do not bypass hooks to avoid a review.

Bind the actual commit SHA to the approval in an external durable receipt.
A review artifact or a linked pull request review record can hold this binding.
Do not force a review record to contain the hash of the commit that contains the same record.
Do not treat a commit trailer as proof without the referenced independent record.

This task does not install receipt storage or hooks.
Describe an existing verified mechanism when available.
Otherwise, record the technical mechanism as a pending enforcement task.

## REV-007 — Frequent commits

Request review when each coherent change is ready.
Use small changes to keep reviews frequent.
Commit and push after the exact-subject check passes.
Do not wait for a complete multi-week refactor.
Do not bypass review to produce frequent commits.

## REV-008 — Review completed work

After integration, review the work item's complete result.
Check cross-component behavior and all required evidence.
Use a fresh reviewer for final architectural acceptance.
Retain per-commit receipts; a final review does not replace them.

A normal GitHub approval applies to a review subject, not every intermediate commit automatically.
The project's procedure requires an explicit mapping for every commit in scope.

## REV-009 — Stop conditions

If no independent reviewer is available, do not claim approval.
Preserve the pending work and report the exact blocker.
Continue independent research that does not require accepting the blocked change.
Do not invent review identities or timestamps.

## REV-010 — What documents can enforce

These rules are mandatory instructions for agents.
They are not a technical security boundary by themselves.
Hooks, CI, and repository controls need separate implementation and verification.
Keep manual review requirements active while those controls remain pending.
Do not claim that a checker exists because this document describes it.
