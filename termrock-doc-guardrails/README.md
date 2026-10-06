# Termrock documentation repair pack

## Purpose

Use this pack to repair project rules and specifications on `termrock-implementation`.
Do not use this task to repair application code.
The required result is a clear contract for later implementation work.

Read `GOAL.md` first. Read `FINDINGS.md` for the source review.
The files under `proposal/` are proposed repository documents.
Merge their requirements into the current canonical documents. Do not create competing specifications.

## Contents

| File | Purpose |
| --- | --- |
| [GOAL.md](GOAL.md) | The documentation-only execution prompt. |
| [FINDINGS.md](FINDINGS.md) | The five commit findings, branch comparison, and review limits. |
| [AGENTS proposal](proposal/AGENTS.md) | Mandatory ownership, scope, and review rules. |
| [Composition contract](proposal/docs/architecture/component-composition.md) | What the library owns and what previews own. |
| [API recipes](proposal/docs/api/consumer-recipes.md) | Current-source examples and proposed consumer patterns. |
| [Example catalog](proposal/docs/api/example-catalog.json) | The status and verification needs of every example. |
| [Commit review](proposal/docs/process/per-commit-review.md) | Review before work, before each commit, and at work completion. |
| [Verification contract](proposal/docs/verification/ownership-and-parity.md) | Visual, semantic, ownership, and generalization evidence. |
| [Documentation checklist](proposal/docs/implementation/documentation-repair-checklist.md) | Requirements for this documentation task. |
| [Future repair list](proposal/docs/implementation/code-remediation-backlog.md) | Code and enforcement tasks. None are executed by this goal. |
| [Component template](proposal/docs/templates/component-contract.md) | Required format for each component reference. |
| [Review template](proposal/docs/templates/change-review.md) | Evidence fields for each work item and commit. |
| [Document map](proposal/docs/process/document-map.md) | Canonical owners and existing files to update. |
| [Writing rules](proposal/docs/process/writing-style.md) | STE writing instructions and project terms. |
| [Pull request template](proposal/.github/pull_request_template.md) | Review evidence summary. This does not replace commit reviews. |

## Evidence status

The source review uses pinned GitHub files and commit changes.
The review does not include a complete runtime visual test of the repository.
The proposed documents are not installed in the repository.
No source code, Git reference, workflow, or approved snapshot was changed.
No independent subagent review was performed while preparing this pack.

`current_source_checked` means that the example follows inspected declarations.
It does not mean that the example was compiled.
`proposed_target` means that the example defines intended API behavior.
It does not claim that all named API elements exist.
Each proposed helper has an owner and a future verification requirement.

The documents use short technical instructions and a project glossary.
This pack does not claim a complete dictionary audit or formal ASD-STE100 certification.
