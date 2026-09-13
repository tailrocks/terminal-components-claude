# Refactoring execution catalog

This catalog implements the planning deliverable in [REFACTORING_COMPLETION_PLAN.md](../REFACTORING_COMPLETION_PLAN.md). It is not authorization to execute the terminal-components refactor during the planning goal. Check the top-level plan's readiness status before dispatch.

The project is `terminal-components`; its group is `completion`. Canonical packages use task-format revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`: `task/v5`, `verify/v2`, and `task-meta/v1`. Dependencies live in each task.toml, not Markdown ordering.

Read the [proof contract](../docs/refactoring-plan/proof-contract.md), [task index](../docs/refactoring-plan/task-index.tsv), and [bidirectional traceability](../docs/refactoring-plan/traceability.tsv). The host freezes the catalog and trusted inputs outside candidate authority before future execution. Package status remains `pending`; an accepted host receipt and integrated ancestry, not a mutable status field, establish a usable predecessor.

Use the pinned standalone taskfmt verifier. Its automated dispatcher/promotion lifecycle hardcodes main and must not be used for this campaign. No task authorizes a push, main merge, automatic tui-snap PR merge, oracle repinning or approval of changed product output.

**Visual validation:** the live regression oracle is the committed grouped store `snapshots/<app>/…/<cols>x<rows>/<color>.{ansi,txt,png,html}` (see [visual-validation.md](visual-validation.md) and `docs/baseline/snapshots-v2.md`). After any product edit, `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` must match. Candidates never write `snapshots/` or run `tuisnap accept`. The deleted `shots/` tree is not a gate.
