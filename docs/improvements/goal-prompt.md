# Holla prototype implementation goal

Use the following text as the goal prompt. This file defines future execution;
creating it does not start implementation.

```text
Complete only the Current implementation phase in IMPROVEMENTS_PLAN.md.
Read docs/improvements/scope.md first, then the linked task files and relevant
parts of docs/improvements/shared-contracts.md. Treat the current scope as the
execution boundary when original full-roadmap task wording is broader.

Deliver a usable, keyboard-complete Holla design prototype on the existing
Junie system. Complete H00, the selected shared-component/coverage slices,
and the simulated UI slices of HP01–HP15 and HP17–HP23. Preserve the existing
Here/context, resource/action, preview, scope, activity and plan model. Use
holla-project/CONCEPT.md for product intent, DESIGN.md for visual/interaction
rules and holla-project/notes/04-design-note.md for the existing design.

All product providers and external effects stay deterministic and in memory.
Implement real interactions and exact simulated target/cwd/argv, state changes,
errors, cancellation, trust, cleanup eligibility and final effects. Do not count
a label, generic success animation or fixture-only data as a completed flow.
Model restart/cache/history/process/filesystem/platform failures with fixtures;
do not connect real commands, services, scanners, persistence, OS clipboard,
open/reveal or destructive filesystem actions. HP16 production/headless CLI
work, including new simulated CLI parity, remains deferred.

Extend existing library components only where these Holla flows require it.
Find the owning invariant before fixing a bug; remove the enabling condition
at that boundary. Include required shared API caller migrations, targeted
showcase examples and affected-consumer regressions. Keep Holla domain policy
in Holla. No speculative widget/theme/container/collection/worker frameworks.
Do not redesign TablePro, Jackin or unrelated showcase pages.

Start with current-source inventory and baseline under H00. Follow the scoped
delivery order, resolve shared contracts before dependent UI work, and finish
reviewable vertical slices. Do not wait for whole tasks in a circular legacy
dependency list; implement their shared contracts and integrate the selected
slices. If deferred work is proven necessary for a current Holla invariant,
record the exact consumer/failing proof and promote only the required slice.
Never use deferral to excuse a known-wrong current flow or introduced regression.

For each slice, implement and retain its relevant semantic regressions, named
Holla fixtures and exact target/effect assertions. Verify keyboard, mouse,
paste, focus, resize, Unicode/copy, errors and cancellation where applicable.
Inspect affected PNG/terminal captures and retain text/cursor/ANSI evidence
with source/binary/scenario/size/color/tool/font provenance. Preserve actual
NO_COLOR proof separately from selecting Mono. Normal repository builds/tests,
capture output and isolated preview/test PTYs are allowed verification work.

Update the task checklists/Evidence sections and the Current tracker checkboxes
as proof passes. Record the row-level HP evidence and every deferred clause.
Do not mark a full task or capability Covered when only its current slice is
complete. Keep all Later/conditional work visible and unchecked; historical
passing suites and unavailable platform checks are not current proof.

Run rtk cargo test --bin holla plus relevant primitive/consumer tests during
implementation. After integration run rtk cargo fmt --check,
rtk cargo clippy --all-targets -- -D warnings, rtk cargo test,
rtk cargo doc --no-deps and rtk git diff --check. Update only intentional,
inspected affected visual baselines. Persist until every Current checkbox has
its required implementation, semantic proof and visual review, with no known
introduced regression. Report concrete changes, validation, remaining Later
work and any proven tool/platform limits without claiming those limits passed.
```
