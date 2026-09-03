# Complete Rust TUI Component-System Refactor

Implement every requirement in @REFACTORING_GOAL.md. Deliver the finished repository, not an analysis, proposal, partial migration, compatibility layer, or unchecked baseline update.

## Mandatory execution model

- Run the primary coordinator and every implementation worker as `claude-fable-5-1` with `high` effort.
- Use fresh, read-only `opus-analyst` agents running `claude-opus-5` with `high` effort for every exploratory repository audit, external/current-documentation research, architecture decision, alternative comparison, root-cause diagnosis, public-API or test-design critique, domain-boundary decision, security analysis, performance interpretation, visual judgment, and independent verification.
- Use `fable-builder` agents for implementation only. Fable owns all repository mutations, migrations, commands, tests, captures, documentation, cleanup, correction loops, integration, and final reporting.
- Never use generic, inheriting, built-in Explore, or built-in Plan agents for this goal. Never override configured agent models or effort.
- If Fable encounters an unresolved architectural or public-invariant question, pause that slice, obtain fresh Opus adjudication, record it, then continue with Fable.
- Treat unavailable required models, model substitution, or model/effort mismatch as a blocker.

## Execution contract

Follow the eight vertical slices in @REFACTORING_GOAL.md. Stabilize shared foundations before parallel component migration. Parallel workers must have explicit, disjoint file ownership. Maintain `REFACTORING_STATE.md` with current slice, model assignments, accepted decisions, ownership, evidence, unresolved findings, and next action so work survives compaction or resume.

## Completion condition

Continue until every item in section 29 of @REFACTORING_GOAL.md is proven. Before claiming completion, surface current evidence from repository artifacts, `REFACTORING_STATE.md`, and final command and review results that:

1. Every component has a documented final disposition and no legacy API, compatibility facade, duplicate implementation, material TODO, stub, or unimplemented normal path remains.
2. Accepted architecture exists, matches implementation, and passed fresh Opus architecture and public-API review.
3. `showcase`, `tablepro`, and `jackin-preview` consume supported public APIs and retain required product behavior.
4. Junie defaults, distinct non-Junie theme, capability downgrades, scoped/local/per-instance overrides, keyboard, mouse, focus, scrolling, dragging, resize, nested overlays, and secret redaction were tested and reviewed.
5. The strongest applicable equivalents of the quality-gate commands in section 26 exit successfully after final corrections.
6. Fresh Opus visual and adversarial final verification finds no unresolved material issue.
7. `REFACTORING_STATE.md` reports no remaining work, and the final report contains every item required by section 30.

At every turn end, report completed evidence, unresolved criteria, and active blockers. Do not let the goal evaluator infer completion from claims unsupported by surfaced command results and review findings.
