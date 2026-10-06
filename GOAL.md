/goal
# Repair the Termrock specifications and agent rules

## 1. Active task

This file carries the active documentation-only task.
Repair the documentation that governs the remaining refactor.
Define reusable Termrock components as the only way to build preview interfaces.
Require independent review for each work item and each commit.
Make the visual, interaction, API, and ownership requirements explicit.

This task changes documentation only.
Do not fix, revert, rename, or refactor source code.
Do not regenerate or approve snapshots.
Record code defects as pending implementation work.

The implementation mission is preserved verbatim, but inactive, at
[docs/implementation/implementation-goal.md](docs/implementation/implementation-goal.md).
Do not follow it until this task restores it through its recorded restoration procedure.

## 2. Authorities

Work only on the existing `termrock-implementation` branch.
Do not merge `main` or reset the branch.
Do not move the `visual-baseline` tag or release.

| Reference | Purpose |
| --- | --- |
| Frozen `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) | Visual and observable interaction authority. |
| `origin/visual-baseline` branch | Compare with the frozen tag. Test reference only. |
| `termrock-refactor` | Intended architecture and preparation requirements. |
| `termrock-implementation` | Current APIs, implementation defects, documentation gaps. |
| `main` | Structural reference only. Never a visual authority. |

Research identities and the branch-versus-tag comparison live in the
[research record](docs/implementation/findings-record.md).

## 3. Write scope

The canonical path allowlist lives in [AGENTS.md](AGENTS.md).
Change only approved document classes.
Keep scripts, Rust files, baseline files, and test fixtures read-only.
Do not change verifier commands, workflow YAML, manifests, locks, or settings.
Do not stage or commit the `termrock-doc-guardrails/` input pack.
Preserve unrelated dirty files.

## 4. Canonical owners

| Contract | Owner |
| --- | --- |
| Agent entry rules | Root `AGENTS.md`. |
| Component-only composition | [docs/architecture/component-composition.md](docs/architecture/component-composition.md). |
| Public signatures and ownership | [docs/api/public-api.md](docs/api/public-api.md) and [types.md](docs/api/types.md). |
| Consumer examples | [docs/api/consumer-recipes.md](docs/api/consumer-recipes.md) and its example catalog. |
| Per-work and per-commit review | [docs/process/per-commit-review.md](docs/process/per-commit-review.md). |
| Visual and ownership verification | [docs/verification/ownership-and-parity.md](docs/verification/ownership-and-parity.md). |
| Individual component contracts | [docs/components/](docs/components/README.md) files. |
| Research record and findings | [docs/implementation/findings-record.md](docs/implementation/findings-record.md). |
| Current implementation status | `checklist.json`. |

Merge guardrail requirements into these owners.
Do not install a second competing documentation system.

## 5. Central rule

> A preview screen is a composition of reusable Termrock components.
> The component owns its pixels, geometry, interaction, and state rules.
> The preview supplies data, layout choices, component state, and typed application actions.

This rule applies to Showcase, Jackin Preview, Holla, and TablePro.
These previews show how a real interface uses the library.
They are not separate product redevelopment projects.

Acceptance requires all of these outcomes together:

```text
visual parity
AND observable interaction parity
AND reusable component ownership
AND usable public APIs
AND customization through supported contracts
AND valid review evidence
```

## 6. Review and stop line

Apply the [per-commit review procedure](docs/process/per-commit-review.md) to this task.
Assign a reviewer who did not author each change.
Freeze each staged change, record its exact subject, and review every line.
Bind each review receipt to its commit SHA outside that commit.
Review the complete work item again after integration.

Accept this task only when all documentation checklist rows have current evidence.
Verify the task diff against the allowlist.
Verify that source, tests, build inputs, workflows, and baseline bytes are unchanged.
Commit and push the reviewed document changes on `termrock-implementation`.
Do not merge the pull request.
Do not begin code repairs after the documentation gate passes.
Do not call the Termrock implementation complete.
