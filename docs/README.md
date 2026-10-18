# Documentation map

## Start here

- [Architecture](architecture.md): current package shape and the split-crate migration target.
- [Design](design.md): product authority and the detailed visual/interaction contracts.
- [Development](development.md): install, run, checks, visual testing, and generated-doc workflows.
- [Roadmap](plan.md): active workstreams and closure expectations.
- [Remaining work](remaining-work.md): branch-specific task IDs, open requirements, dependencies, and scoped closures.
- [Documentation maintenance](documentation-scope.md): update active specifications and preserve generated or pinned inputs.

## Worktree and readiness

This is the `wip/reach-pages-a-2` worktree. The code remains in one `junie-tui` package and is not the split-crate target. `main` gives the correct structural direction, but its refactor remains incomplete and not ready. `design/interaction-polish` is the exact UI/UX, color, interaction, and behavior target; this WIP contains useful reference material and scoped implementation work, and does not imply that every target feature is present.

The root manifest defines package `junie-tui`, library `junie_tui`, four binaries (`showcase`, `tablepro`, `jackin-preview`, `holla`), and the `xtask` workspace member. `src/docs.rs` generates and checks `docs/components/`; `xtask` reads the visual README and plan. Keep those executable documents and generated pages intact.

## Evidence and preservation

Treat task-level evidence as revision- and scope-specific. A historical report, branch name, or successful compile does not establish parity or readiness. Preserve captures, legal notices, trusted task inputs, validator contracts, and tool-consumed documentation. See [documentation maintenance](documentation-scope.md) for the update rules.
