# Mole-informed disk and cleanup patterns

This reference distills useful disk-analysis and cleanup ideas from Mole into
Holla's product concept. It does not require copying Mole's interface or visual
design.

Research snapshot: 2026-09-03. Mole is macOS-focused. Its commands and cleanup
families demonstrate workflows; Holla must generalize them for Linux, remote
hosts, mounted filesystems, and the detected technology stack.

## Concrete Mole workflow examples

```sh
mo analyze [path]
mo analyze --json [path]
mo clean --dry-run
mo clean
mo clean --whitelist
mo purge
mo purge --dry-run
mo purge --paths
mo installer
mo history
mo history --json
mo status
mo status --json
mo status --watch --interval 2s
```

These examples provide conceptual evidence for:

- progressive path analysis and drill-down;
- known cleanup-family discovery with item and size totals;
- dry-run before destructive cleanup;
- persistent path protection;
- project-artifact discovery grouped by project, size, and age;
- removable-installer discovery;
- cleanup audit history;
- CPU, memory, disk, network, power, and process snapshots.

Holla should absorb these workflow ideas into its broader context, hierarchy,
plan, and activity model. It should not reproduce Mole's commands or interface
as Holla's required interaction language.

## Relevant workflow ideas

### Progressive analysis

Begin with filesystem and high-level location information, then stream deeper
size results. Let the user navigate from overview to large directories, files,
or rebuildable artifacts without waiting for a complete scan.

### Cleanup families

Group candidates by meaningful ownership:

- system data;
- user essentials;
- application caches;
- browsers and cloud tools;
- developer-tool caches;
- virtualization and container data;
- application leftovers;
- backups and firmware;
- large files;
- project artifacts.

Holla should adapt categories to the current operating system and installed
stack rather than assume macOS-only paths.

### Project artifacts

Recognize rebuildable outputs including:

- Node `node_modules` and distribution output;
- Rust and Maven target directories;
- Gradle and general build directories;
- project-local Gradle state;
- task-runner caches;
- coverage output;
- Python environments and caches.

Recognition by name is only the start. Confirm project ownership, manager,
activity, symlink behavior, configuration, and actual regeneration path.

## Required hierarchy

Disk understanding should preserve this semantic hierarchy:

```text
Host
├── Filesystems and mounted volumes
├── Current working directory
│   ├── Large directories
│   ├── Large files
│   └── Rebuildable artifacts
├── Child projects
│   ├── frontend: node_modules
│   ├── backend: Cargo target directory
│   └── android: Gradle outputs
└── System cleanup families
    ├── Package caches
    ├── Logs
    ├── Temporary data
    └── Application caches
```

Tree meaning is required. Visual representation remains open.

## Candidate facts

Every candidate should explain:

- resolved path;
- filesystem, scope, and owning project;
- category and item count;
- measured size;
- last known activity;
- why it is considered rebuildable or removable;
- whether a relevant process is active;
- required privilege;
- deletion method: Trash, permanent, or tool-managed;
- confidence, uncertainty, and skip reason;
- dependencies and conflicts.

## Freshness-aware defaults

Recent artifacts and artifacts whose activity cannot be verified should begin
unselected. Old, confidently rebuildable artifacts may begin selected.

Selection is a recommendation, not execution permission. The user can exclude a
whole project, category, or individual target before confirmation.

## Dry-run and protection

- Use the same eligibility rules for preview and execution.
- Support a persistent protected-path list.
- Refuse protected, ambiguous, busy, live, or unverifiable targets.
- State when missing privilege makes discovery partial.
- Revalidate targets immediately before deletion.
- Never delete parent directories merely because all displayed children were
  selected.

## Recovery and audit

Distinguish:

- move to Trash or another recoverable location;
- permanent deletion;
- cleanup through the owning tool;
- cleanup that can be regenerated automatically;
- cleanup that requires a later networked reinstall.

Keep history of targets, method, reclaimed space, skips, failures, and time.
History is an audit trail unless actual restoration is supported.

## Editable cleanup plan

Workflow:

1. resolve requested scope;
2. measure filesystem capacity and current usage;
3. discover candidates progressively;
4. group by filesystem, project, and cleanup family;
5. build dependency graph;
6. preselect only stale, confidently rebuildable candidates;
7. allow project, category, and path exclusions;
8. recalculate size and dependent work;
9. review exact targets and recovery method;
10. confirm according to risk;
11. execute independent branches in parallel;
12. stream per-step progress and output;
13. report removed, trashed, skipped, failed, and reclaimed totals;
14. preserve audit history.

Example:

```text
Clean developer artifacts under /work
├── frontend
│   ├── node_modules — 8.4 GB — inactive 31 days — selected
│   └── dist — 420 MB — active today — unselected
├── backend
│   └── target — 12.7 GB — inactive 18 days — selected
└── android
    ├── build — 3.1 GB — inactive 22 days — selected
    └── .gradle — 900 MB — activity unknown — unselected
```

Independent project branches may run concurrently. Shared caches, active build
systems, overlapping paths, and common output directories require ordering or
exclusion.

Broad permanent cleanup uses Holla's two-gate confirmation. For **Remove all
build artifacts under `/work`**, final confirmation must bind to `/work` and the
resolved plan rather than accept generic `yes`.

## Holla extensions beyond reference patterns

Holla should improve the concept through:

- full category and individual-target exclusion;
- cross-platform and remote-host awareness;
- dependency graphs rather than flat cleanup lists;
- parallel independent branches;
- stack-specific cleanup through Cargo, Gradle, npm, pnpm, Yarn, Bun, Docker,
  and system package tools;
- persistent program insights and per-step output;
- integration with current, parent, children, and system scope.
