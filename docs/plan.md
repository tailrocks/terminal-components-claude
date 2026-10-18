# Roadmap and active-work index

## Readiness

The `wip/plan-s16` worktree retains a large branch-specific implementation backlog. The split-crate architecture on `main` is unfinished and not ready; this worktree does not establish that every feature in the exact `design/interaction-polish` target is implemented. No task is closed by this index alone.

`main` controls crate structure and reusable API direction. `design/interaction-polish` controls UI, interaction, color, and behavior. Current source and task-level acceptance evidence determine implementation status; historical reviews do not close work.

## Active workstreams

The detailed ledger is [remaining work](remaining-work.md), including WIP-specific IDs, dependencies, recorded scoped closures, and acceptance checks. This map points to each fuller contract:

| Workstream | Specification | Closure evidence |
|---|---|---|
| Component APIs and crate migration | [Shared component plan](architecture/shared-components-plan.md), task packages | Current crate ownership, API tests, integration gates, and task-specific proof receipts |
| Interaction and rendering | [Design contract](design/DESIGN.md), [improvements](improvements/) | Named behavior tests and matched visual review |
| Cross-component defects | [Improvement plan](improvements/PLAN.md), [shared contracts](improvements/shared-contracts.md) | Each item’s acceptance criteria and regression tests |
| Design-review findings | [Design review index](audits/design-review-2026-10/README.md) | Finding-by-finding disposition backed by current source or an explicit decision |
| App parity and coverage | [Parity records](parity/), source-first app plans | Same scenario and dimensions against the design target, plus app journey assertions |
| Showcase reachability | Showcase navigation and coverage audits | Reachability/coverage tests and current visual evidence |
| Visual suite and approvals | [Visual testing guide](visual-testing/README.md), [suite plan](visual-testing/plan.md) | Documented tiers and current approval-store integrity |
| Terminal behavior and safety | [Safety rationale](../SAFETY.md), task contracts | Focused tests for terminal ownership, input, restoration, and safety contracts |
| External tools and dependencies | [Upstream records](upstream/), [dependencies](dependencies.md), [CI](ci.md) | Recorded disposition and green applicable CI job |

Task packages are scoped contracts, not a general status report. Their `trusted/` trees are immutable proof inputs. `docs/refactoring-plan/evidence/` contains protocols and executable proof assets; preserve these when updating this index.

## Updating and closing work

Keep unchecked requirements, task IDs, and dependency edges in their detailed owners. A “Done” note closes only the cited scope and revision; it does not establish that `main` contains or verifies the behavior. For each closure, inspect the current implementation, run focused tests and relevant repository gates, then record the exact revision, command, result, and visual scope. Do not infer status from successful compilation alone.
