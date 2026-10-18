# Planning acceptance gate

## Status

Open. This gate defines what must be demonstrated before the refactoring plan is accepted for execution. It does not report that the gate currently passes or that product implementation is complete.

## Authority

The requirement set is [PLANNING_GOAL.md §22](../sources/PLANNING_GOAL.md). The current source-first audit and unresolved findings are in [reaudit-plan.md](reaudit-plan.md) and [reaudit-findings.tsv](reaudit-findings.tsv). Task packages define their own implementation and proof contracts.

## Acceptance conditions

All conditions require evidence tied to the current source/task revisions. Historical reviews and previous-pass reports do not satisfy them.

| # | Required condition | Evidence to inspect |
|---|---|---|
| 1 | Pin the immutable UI/UX reference and preserve its source identity. | [History](history.md), protected oracle inputs, and capture provenance. |
| 2 | Explain branch ancestry, divergence, and which branch controls structure versus product behavior. | [History](history.md), [branch continuation proposal](branch-continuation-proposal.md), and current authority docs. |
| 3 | Reconstruct the historical source versions needed to identify accepted, changed, and superseded behavior. | [Revision index](history-revision-index.tsv), [canonical historical obligations](historical-obligations-canonical.tsv), and the open history audit. |
| 4 | Distinguish accepted decisions from rejected, deferred, or superseded proposals. | [Decision ledger](decision-ledger.tsv), [adjudications](architecture-adjudication.md), and source-qualified task contracts. |
| 5 | Inventory the current component/API architecture and separate measured preservation from unimplemented work. | [Architecture matrix](architecture-matrix.tsv), [component parity](component-parity.tsv), and [components audit](components.md). |
| 6 | Inventory all four applications against the immutable design reference. | [Showcase](showcase.md), [Holla](holla.md), [Jackin](jackin.md), and [TablePro](tablepro.md). |
| 7 | Bind important screens, states, inputs, and journeys to source, owners, and proof. | [Application parity](application-parity.tsv), scenario identities, and staged contributions. |
| 8 | Provide deterministic behavioral and visual regression checks with explicit capture provenance and comparison rules. | Task-owned tests and capture protocols; full application captures remain required outputs. |
| 9 | Record external tool dependencies and their exact scope without treating a tool review as project-harness acceptance. | [Tui-snap review](tuisnap-review.md), dependency records, and task contracts. |
| 10 | Map every architecture obligation and accepted decision to an implementation owner and proof. | [Architecture matrix](architecture-matrix.tsv), [component parity](component-parity.tsv), and [traceability](traceability-components.tsv). |
| 11 | Map every important parity obligation to an owner and automated or explicitly reviewed proof. | [Application parity](application-parity.tsv), contribution records, and app task plans. |
| 12 | Validate every task package against the pinned task-format contract. | [Task format](task-format.md) and canonical task lint. |
| 13 | Require each task to state its purpose, scope, dependencies, authority, acceptance conditions, commands, and evidence. | Canonical `README.md`, `task.toml`, verification config, task index, and task-format lint. |
| 14 | Prove the dependency graph is acyclic, complete, and ordered for safe integration. | [Task graph](task-graph.md), canonical dependencies, and graph validation. |
| 15 | Identify parallel work and serialize conflicting source or integration work. | Derived graph and explicit worktree/join contracts. |
| 16 | Define an integration gate requiring complete architecture, system, parity, ancestry, and verification evidence. | [TASK-069](../../refactoring-tasks/terminal-components/completion/069/README.md) and its trusted acceptance inputs. |
| 17 | Obtain independent review of material plan and repair changes. | Current review records with named scope and exact revisions; authors cannot approve their own repairs. |
| 18 | Disposition every material finding with source-backed repair or evidence-backed rejection, then independently recheck its reproducer. | [Review findings](review-findings.tsv), [re-audit findings](reaudit-findings.tsv), and task-owned checks. |
| 19 | Show that no important source requirement, protected clause, scenario, or proof is orphaned from the task graph. | [Traceability tables](traceability.tsv), canonical historical obligations, scenario mappings, and exact joins. |
| 20 | Ensure every important UI/UX behavior has a design-authoritative contract and a verification owner. | Design branch source, [application parity](application-parity.tsv), detailed app plans, and reviewed visual evidence. |

## Decision rule

The plan is acceptable only when every condition above is supported by current evidence and all open findings are resolved or explicitly retained with an owner, prerequisite, and closure proof. A successful compile or validator run alone is not execution readiness. Acceptance of the plan is separate from completion of the split-crate refactor and application parity.
