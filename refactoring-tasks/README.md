# Refactoring execution catalog

This catalog implements the planning deliverable described by the current refactoring plan. It is not authorization to execute the terminal-components refactor. Check [`docs/refactoring-plan/execution-readiness-report.md`](../docs/refactoring-plan/execution-readiness-report.md) before dispatch.

The project is `terminal-components`; its group is `completion`. Canonical packages use taskfmt `0.2.0` at revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`: `task/v5`, `verify/v2`, and `task-meta/v1`. Dependencies live in each task.toml, not Markdown ordering.

Read the [proof contract](../docs/refactoring-plan/proof-contract.md), [task index](../docs/refactoring-plan/task-index.tsv), and [bidirectional traceability](../docs/refactoring-plan/traceability.tsv). The host freezes the catalog and trusted inputs outside candidate authority before future execution. Package status remains `pending`; an accepted host receipt and integrated ancestry, not a mutable status field, establish a usable predecessor.

Use only the latest standalone taskfmt verifier from
`/Users/donbeave/Projects/taskfmt/task-format`. Its automated
dispatcher/promotion lifecycle hardcodes main and must not be used for this
campaign. No task authorizes a push, main merge, automatic tui-snap PR merge,
oracle repinning or approval of changed product output.

**Visual validation:** the frozen regression oracle is the grouped store
`snapshots/<app>/…/<cols>x<rows>/<color>.{ansi,txt,png,html}` on the immutable
`visual-baseline` tag. The current branch does not contain that store or its
PTY suite, so this remains a preparation requirement, not a runnable current
gate. Once imported read-only, affected tasks must use `cargo nextest` for the
visual suite. Candidates never write `snapshots/` or run `tuisnap accept`.
The deleted `shots/` tree is not a gate.
