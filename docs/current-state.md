# Current state

The refactor is incomplete and not ready for product use; the main modular state is known to be broken and incomplete relative to the intended architecture and product behavior. Main has workspace members for core, layout, theme, text, render, collections, runtime, controls, fields, feedback, navigation, viewport, overlays, grid, editors, forms, terminal, and session, then facade, Showcase data/demo/UI/app crates, Holla domain/catalog/plan/sim/UI/app crates, Jackin domain/sim/presentation/host-UI/capsule-UI/app crates, TablePro domain/SQL/demo/UI/app crates, test/conformance and tooling crates. A crate's existence does not prove its ownership boundary is complete or its API is ready.

The design reference and all six WIP workspaces use the monolithic `junie-tui` package plus `xtask`; each defines four application binaries. They are structurally distinct snapshots that diverge from the design reference in different ways; they are not branches that can be substituted for one another:

- `wip/v1-showcase` includes a Holla activity-timing correction that accounts for paused ticks and waits through a stopped-job prompt. Its component/theme, Showcase, visual, review, upstream, and CI work remains separate from that scoped fix.
- `wip/s14-followups` connects copy status to the platform's actual OSC 52 support and limits, avoiding an unconditional success message; Jackin has a corresponding application path. Its Showcase reachability and component work remains unfinished.
- `wip/plan-s16` contains collection API work for list item metadata and row marks, tree metadata/cells, and column markers, in addition to Showcase and component requirements.
- `wip/rapps-3-7-14` adds metadata folding when a metadata column is absent and hover reveal for dropped columns in app-specific tables, alongside responsive application work.
- `wip/reach-pages-a` contains natural-size measurement and scroll/terminal behavior, plus measured layouts for text utilities, query workbench, connections, and finder pages.
- `wip/reach-pages-a-2` contains natural-size measurement, a scroll-view horizontal-bar retry, panel metadata hover reveal, and measured/reveal behavior for step navigation; it does not contain all of the reach-pages-a measured pages.

These are scoped source differences, not parity or readiness claims. Each branch's remaining criteria and branch-local verification still apply; a fix in one does not close the equivalent behavior on another.

## Demonstration coverage gap

Showcase declares 22 page identities on `main`, compared with 54 on `design/interaction-polish`. Main lacks the design reference's distinct routes for `Diff`, `Selectable`, `CellEditors`, `StatusHints`, `BreadcrumbsRails`, `HelpKeys`, `QuitLeave`, `FocusNavigation`, `ChecksRadios`, `Toolbars`, `Columns`, `Tabs`, `Menus`, `DialogKinds`, `PopupsTooltips`, `FileBrowser`, `EmptyError`, `Meters`, `Notices`, `SplitsStacks`, `MasterDetail`, `PaneTree`, `TextUtilities`, `QueryWorkbench`, `Connections`, `Finder`, `DiskCleanup`, `Activity`, `Workspaces`, `AccountsUsage`, `LaunchCockpit`, and `ReadOnlyReport`. These identities and page factories are declared in `crates/showcase-ui/src/app.rs` on main and `src/bin/showcase/app.rs` on the design reference. This establishes missing route-level demonstration coverage; it does not establish that the underlying controls or other applications lack equivalent capabilities. Restoring this coverage while preserving the design experience is part of the future migration, not this documentation replacement.

## Documentation and proof tooling

The legacy Markdown tree has been replaced by this shared documentation set. `AGENTS.md` and `CLAUDE.md` are preserved byte-for-byte as required; some may still point to retired paths such as `docs/GOAL.md` or `docs/ci.md`. The shared `README.md` is the navigation entrypoint for this documentation. Existing automation has not been migrated and may still consume retired paths or exact historical text:

- Plan, hash, bootstrap, task-format, and validation tooling in the WIP and design workspaces may expect legacy task Markdown.
- Design source generates component documentation, and an `xtask` checks that generated output for freshness.
- Main visibility tooling expects `STATUS.md` and `WORK_QUEUE.md`.
- Qualified-capture configuration pins its README.

Those tools may fail or report missing evidence after the documentation replacement. No such result establishes the correctness or completeness of the new documentation, and the new documents do not certify old hash-bound proof inputs. Updating generators, validators, pins, and their proof protocol is required before those workflows can validate this documentation system. Until then, report the exact incompatible gate and do not claim all gates pass.

Non-Markdown source, test, configuration, generated, legal, and byte-bound fixture files remain governed by their own repository contracts. They are not a second maintained documentation tree, and their continued presence does not make the retired Markdown current. Their branch-specific consumers and exact-content constraints must be inspected before a future migration changes them.
