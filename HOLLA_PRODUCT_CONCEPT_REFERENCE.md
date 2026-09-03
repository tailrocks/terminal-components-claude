# Holla: Context-Adaptive Action Launcher for the Terminal

> Concept brief for exploring and designing a new terminal product.

## Purpose

Use this document to brainstorm, prototype, and evaluate a terminal-native
product focused on one question:

> **What can I do here, on this host, right now?**

This is not a specification for copying another product. It describes user
needs, product ideas, experience principles, example workflows, and boundaries.
The product designer should explore the strongest terminal-native expression of
these ideas.

This concept starts from product needs alone. Design choices should follow the
desired experience, not constrain it in advance.

## How to interpret this document

Everything in this document is conceptual context. It defines what users need to
understand, which capabilities may be useful, which decisions must be supported,
and how workflows relate. It does **not** define visual design.

Nothing here prescribes:

- screen layout or pane structure;
- colors, borders, typography, spacing, density, or decoration;
- component shapes or placement;
- whether information uses lists, dialogs, overlays, pages, panels, or another
  representation;
- exact wording, icons, key bindings, or mouse gestures;
- compact, full-screen, inline, or multi-view presentation.

Names such as **Root**, **Suggested here**, **Preview**, **Action Panel**, and
**Gate** identify product ideas and user decisions only. They are not required
UI components. Text examples describe information and sequence, not appearance.

Any agent using this reference should independently brainstorm multiple ways to
represent the concept, compare them against user needs, then choose the clearest
coherent direction. Do not reproduce example formatting as a design.

---

## 1. Vision

Holla turns the user's current terminal context into a small, relevant, and
actionable set of choices.

The user invokes one launcher from any shell location. Holla understands where
the user is, what kind of project surrounds that location, which workflows are
available, what state matters now, and which actions are likely to help.

The current working directory is the primary contextual object. Project,
workspace, host, session, installed tools, live state, and personal habits add
meaning around it.

### Product promise

> **Open Holla anywhere and immediately understand what you can do here.**

### Product mantra

> **This folder. This host. Right now.**

### Design mantra

> **Context before catalog. Intent before syntax. Explain before execute.**

---

## 2. Problem

### 2.1 Terminal work assumes command recall

The shell expects users to know:

- which tool solves a problem;
- whether it is installed on this machine;
- exact commands and flags;
- correct working directory;
- project conventions;
- operation scope and consequences;
- whether a stronger specialized terminal tool exists.

Completion helps after choosing a command. History helps after remembering part
of a previous command. Documentation helps after identifying the right tool.
None starts from the user's likely intent.

### 2.2 Useful workflows are fragmented

Developer workflows live across aliases, shell functions, project tasks,
personal scripts, package manifests, task runners, saved commands, history, and
specialized TUIs.

This becomes a powerful but poorly indexed personal operating system. Holla
should make these capabilities discoverable without forcing them into one
configuration format.

### 2.3 Location contains intent

The current path often reveals what matters:

- a Rust project suggests build, test, lint, run, and artifact review;
- a Node project suggests package scripts and its selected package manager;
- a Git worktree suggests status, diff, synchronization, and branch actions;
- a workspace suggests operations across several projects;
- a Compose project suggests services, logs, health, start, and stop actions;
- large generated directories suggest storage investigation;
- a remote server suggests service, process, log, and health actions.

Holla should transform location into an actionable surface.

### 2.4 Availability is not relevance

Holla must distinguish:

- **Capability:** something available in the environment;
- **Action:** a concrete operation that can be performed;
- **Recommendation:** an action or resource relevant now.

Examples:

- Pull is relevant when a branch is behind.
- Review changes is relevant when a worktree is dirty.
- Tests are relevant after project-file changes or repeated use here.
- Service logs are relevant when a local service is unhealthy.
- Artifact cleanup is relevant when generated data is large or stale.
- Workspace-wide status is relevant only in a recognized workspace.

### 2.5 Easier execution increases safety responsibility

Terminal actions can modify files, discard work, remove data, change services,
affect several projects, run on the wrong host, or expose secrets.

When Holla makes an action easier to invoke, it must also make scope and
consequences easier to understand.

Safety is an execution discipline, not a content filter. Destructive actions
remain first-class, searchable, learnable, and eligible for recommendation when
they match user intent. Their risk changes preview and confirmation behavior; it
does not make them unavailable or inherently less relevant.

---

## 3. Primary job to be done

> **When I am in a terminal at any path, help me understand the most useful and
> relevant actions available here and now, then let me perform one with very few
> keystrokes without recalling exact syntax.**

Supporting jobs:

- remind me what this project supports;
- recover a workflow used here before;
- reveal useful installed tools;
- find a nearby file, folder, project, or service;
- act across a recognized project collection;
- inspect system and disk state;
- turn repeated commands into stable shortcuts;
- show exact target, behavior, provenance, and risk before execution;
- behave appropriately across local, remote, and container environments;
- hand off to a specialist tool when it offers a better focused experience.

---

## 4. Product model

Holla is a contextual action launcher, developer command surface, terminal
shortcut layer, and here-first workflow navigator.

It is not merely fuzzy search over commands. Search is only one interaction
primitive. Product value comes from combining context, ranking, explanation,
clear scope, predictable behavior, and coherent execution.

### Core experience objects

- **Context:** path, project, workspace, host, session, platform, and live state.
- **Capability:** an available tool, task system, service, workflow, or
  integration.
- **Resource:** a project, task, service, container, process, file, folder,
  cleanup candidate, or remote target.
- **Action:** inspect, open, run, build, test, synchronize, stop, clean,
  navigate, copy, or configure.
- **Recommendation:** an action or resource elevated by current relevance, with
  a reason.
- **Flow:** a focused experience richer than one command, such as disk
  exploration, task output, or multi-project status.
- **Plan:** a reviewable set of steps, dependencies, optional branches, and
  execution policies that together accomplish a larger intent.
- **Step:** one inspectable unit within a plan, with command, scope, inputs,
  dependencies, status, output, and retry behavior.

---

## 5. Context model

### 5.1 The current path is primary

This is the highest-priority context rule: Holla always starts from the exact
working directory where the command was launched. Initial recommendations,
search ranking, inferred intent, and default execution scope begin there.

Starting here must not trap the user here. From the same experience, the user
must be able to explore and act on:

- the exact current folder;
- detected ancestors and their project or workspace roots;
- detected child projects and services below the current scope;
- host-wide tools, resources, processes, and services.

Current-folder priority is a default relevance bias, not a hard filter. Explicit
queries such as `docker`, `children`, `workspace tasks`, or `parent setup` may
immediately elevate another scope. Urgent state from another scope may also be
recommended when its origin and target are clear.

If an action targets a parent, child, home directory, or entire machine, that
scope must be explicit. Holla never silently changes effective working directory
or execution target.

Path and host belong in persistent interface context, not hidden preview data.

### 5.2 Context rings

Holla expands outward only as needed.

#### Here

- files and manifests in the current folder;
- exact-folder tasks and state;
- current Git state;
- nearby generated data;
- actions previously used here.

#### Project

- nearest meaningful project root;
- project tasks, services, documentation, and workflows;
- project-wide state;
- trusted project actions.

#### Workspace and neighborhood

- parent workspace or monorepo;
- bounded child projects;
- sibling services;
- recognized collections of independent projects;
- workspace-wide operations.

#### Host

- installed tools and package managers;
- services, containers, processes, and ports;
- system resources and disk pressure;
- platform-specific actions.

#### Personal and team workflow

- aliases, pins, and saved paths;
- remote targets;
- personal automation;
- approved team workflows;
- recent global actions.

Global capabilities remain available but should not displace strong local
recommendations.

### 5.3 Scope directions

The context model has four navigable directions.

#### Current

Exact `pwd`. This is the default source of recommendations and actions.

#### Parent and ancestors

Walk conceptually upward from `pwd` to detected project, workspace, and
monorepository roots. Ancestor-defined tasks, services, environments, and setup
actions may be relevant inside a descendant folder.

Example: Holla starts in `monorepo/apps/frontend`. Frontend actions rank first,
but the parent root may contribute **Start development ecosystem**, **Start
required containers**, or **Run workspace setup** because those workflows are
defined for the whole monorepository.

Each ancestor suggestion must identify the defining scope and the directory in
which it will execute.

#### Children and descendants

A child is a detected project, package, service, or task-bearing folder below
the active scope. Child discovery must be bounded and structure-aware, not an
unlimited recursive scan.

Example: Holla starts at a monorepository root. It discovers child projects such
as `apps/frontend`, `services/api`, and `tools/worker`. If a child contains its
own `mise.toml`, Holla can expose that child's tasks, including a task that starts
its development process. Selecting it executes in that child's directory, not
the monorepository root.

Children may contribute:

- tasks from `mise.toml` or another project task definition;
- package scripts;
- build, test, lint, and run actions;
- service and container actions;
- logs and health;
- navigation into the child context.

#### System

Host-wide capabilities independent of one project: Docker status, container
search, all-container operations, system processes, ports, disk state, package
tools, and services.

System scope remains easy to enter from any folder. Its actions must state that
they affect the host rather than only the current project.

### 5.4 Seamless scope navigation

Users should move among Current, Parent, Children, and System without closing or
relaunching Holla. Search may cover all scopes or narrow to one. Returning to
Current must always be immediate.

Every result communicates:

- where it was discovered;
- where it will execute;
- whether it belongs to current, ancestor, descendant, or system scope;
- why a noncurrent result is relevant now.

These are scope concepts, not prescribed tabs, panes, menus, or layout.

### 5.5 Context contract

1. Always communicate current path.
2. Make host unmistakable, especially remotely or in sensitive environments.
3. Rank local actions above global actions by default.
4. State scope for every nonlocal action.
5. Make transitions to project, workspace, home, or system scope explicit.
6. Mark stale, cached, partial, or still-loading information.
7. Explain the strongest signal behind each recommendation.
8. Never treat recommendation as authorization.
9. Keep selection stable while new results arrive.
10. Provide one consistent route back to the root experience.
11. Make Current, Parent, Children, and System scopes directly explorable.
12. Preserve each action's effective working directory across search, preview,
    execution, and history.

---

## 6. Root experience

### 6.1 One entry point

A short command opens the same root experience everywhere. Git, tasks, files,
disk, services, personal actions, and tool handoffs remain discoverable from
that surface.

The user should not choose a category before expressing intent.

### 6.2 Useful before typing

The empty state is a recommendation surface, not an exhaustive command catalog.

The initial experience should make these concepts available. This list expresses
product priority, not layout or display order:

1. **Suggested here** — small ranked set based on current state;
2. **Recent here** — actions recently used in this context;
3. **Explore** — stable entry points such as Tasks, Git, Files, Disk, Services,
   and System;
4. **Discovery status** — subtle feedback for capabilities still loading.

### 6.3 Search across domains

A query searches actions and resources together.

For example, `logs` may surface a project task, Compose service logs, a system
service, a nearby log file, or trusted personal automation. Every result must
show type and scope without requiring domain selection first.

Search should match intent, labels, descriptions, keywords, resource names,
aliases, and relevant state. Queries such as `why disk full`, `tests`, `sync
projects`, `service logs`, or `open config` should work without exact syntax.

### 6.4 Actions and resources

Root results may represent:

- a direct action, such as **Run tests**;
- a resource, such as the current Git project;
- a recommendation, such as **Review 4 modified files**;
- a domain flow, such as **Analyze disk usage**;
- a specialist handoff, such as **Open system monitor**.

A resource should have one coherent identity. Its best current action is
primary; related operations remain accessible without becoming unrelated
duplicates.

### 6.5 Primary action and contextual alternatives

A direct interaction performs the obvious primary action. A separate interaction
exposes alternatives:

- inspect or explain;
- alternate operations;
- change scope;
- preview or copy command;
- insert command into shell;
- run now;
- pin, alias, hide, or reset ranking;
- choose a preferred specialist tool.

### 6.6 Discovery and muscle memory

Fuzzy search supports discovery. Exact aliases and visible command paths support
deterministic repetition.

Examples:

- `du` opens disk usage;
- `gp` selects Git pull;
- `test` selects the preferred test action here;
- `d`, then `u`, opens Disk, then Usage.

Exact aliases must not drift because learned ranking changed.

### 6.7 Structured arguments

Parameterized actions should collect values through structured fields. Show
field name, expected value, default, validation, and secret sensitivity. Keep the
resulting command or operation previewable.

### 6.8 Execution and return loop

Depending on action and preference, Holla may run, request confirmation, insert
an editable command, open a focused flow, hand off to another TUI, or continue
work in background.

A simple action may execute as one step. A compound intent such as **Upgrade
everything on this system** must become a reviewable plan rather than one opaque
command or a fixed linear script.

After completion, show outcome, duration, affected scope, failures, and useful
follow-up actions.

---

## 7. Conceptual experience states

This section defines information and decisions required in important states. It
does not define screens, layout, components, styling, or navigation mechanics.

### Initial state

Before the user enters a query, the experience should communicate:

- current path and host;
- a few relevant recommendations and why they matter;
- recent actions for this context;
- ways to explore broader domains;
- whether more context is still being discovered.

How these concepts are grouped, ordered, or displayed is open design work.

### Active selection state

When an action or resource is selected, the user should understand its identity,
type, scope, primary behavior, alternatives, recommendation reason, and current
focus. The representation is not prescribed.

### Preview questions

Preview should answer:

- What will happen?
- What path, project, service, or host is targeted?
- Why is this recommended?
- What will change?
- Is information live, cached, or partial?
- Does execution require confirmation or trust?

### Destructive confirmation state

For **Delete everything inside this folder**, the user must understand the
resolved path, host, item count, estimated size, inclusion of hidden and nested
content, recovery mode, and irreversibility before continuing.

That review is the first decision. A separate second decision requires the
target-bound phrase `DELETE EVERYTHING IN /work/scratch`. Execution remains
unavailable until it matches exactly.

This defines required knowledge and intent. It does not prescribe a dialog,
button, input placement, visual hierarchy, or other presentation.

---

## 8. Core experience domains

### 8.0 Non-negotiable technology focus

Holla must be designed around the user's real technology stack, not a generic
catalog of shell commands. These are primary product domains:

1. mise installation, active tools, missing tools, tasks, hierarchy, upgrades,
   and trust;
2. Git operations here or across discovered children, including status, pull,
   push, primary-branch switching, and GitHub cloning from personal or
   organization accounts;
3. Docker inspection, logs, Compose context, resource usage, lifecycle, and full
   host cleanup;
4. `btm` as preferred deep system-monitoring handoff;
5. progressive disk analysis and cleanup using Mole-informed concepts;
6. CPU, memory, disk, network, pressure, and process insights;
7. PostgreSQL activity through native critical metrics and `pg_activity`;
8. common Rust workflows through Cargo and nextest;
9. SSH destination discovery and connection from user SSH configuration;
10. Rust build-artifact cleanup;
11. Gradle output and cache cleanup;
12. Node dependency and package-manager cache cleanup.

These domains should dominate discovery, recommendation quality, hierarchy,
plan orchestration, activity multiplexing, and realistic concept scenarios.
Generic extensibility remains useful but must not dilute this focus.

Detailed local concept examples live in:

- [Technology-stack workflows](reference/technology-stack-workflows.md)
- [Mole-informed disk and cleanup patterns](reference/mole-disk-cleanup-patterns.md)

### 8.1 Current project

Discover build, run, test, lint, format, clean, project tasks, local services,
documentation, common paths, and trusted project workflows. Users should not
need to inspect manifests to learn what the project supports.

#### mise

Treat mise as a first-class context system. Explain effective local, ancestor,
monorepository, and global configuration; active and missing tools; available
updates; discovered tasks; task working directory; task dependencies; and trust
state.

From a monorepository root, expose namespaced child tasks. From inside a child,
expose local tasks first while retaining applicable parent tools, environment,
and ecosystem tasks. Trust must identify exact configuration and propagation
scope before running tasks or installing tools.

#### Rust

Recognize Cargo workspaces and members as one hierarchy. Focus on common actions:
check, build, run, test, nextest, format, Clippy, and cleanup. Preserve current
package, workspace, selected package, binary, test, feature, toolchain, and
profile scope without overwhelming common defaults.

### 8.2 Git

Use live worktree, branch, upstream, conflict, and remote state to influence
recommendations and primary actions. Deep history, staging, rebasing, or
conflict work may open a specialist Git TUI.

Core Git intents include:

- status here or across all discovered child Git worktrees;
- pull here or across selected children;
- push here or across selected children;
- switch selected projects to their actual primary branches;
- inspect blocked, dirty, detached, divergent, or missing-upstream states;
- clone a repository owned by the active GitHub account or one of its
  organizations.

Recursive discovery must understand `.git` directories and files, deduplicate
worktrees, and distinguish submodules from independent child projects. Bulk
plans allow exclusions, bounded parallelism, per-project output, and isolated
failure. Never hard-code `main` or `master`; resolve each project's primary
branch.

### 8.3 Project neighborhoods

Recognize intentional workspaces without assuming every parent folder is one.
Support collection status, safe synchronization, filtering, partial failure,
clear per-project results, and explicit multi-project scope.

### 8.4 Disk and cleanup

Separate observation from deletion. Show largest contributors first, allow
drill-down, recognize generated artifacts, explain age and regeneration, estimate
recoverable space, prefer recoverable deletion, and report each result.

Never recommend deletion only because a path is large.

Adopt these Mole-informed product patterns without copying its visual design:

- progressive results before full scanning completes;
- hierarchy by filesystem, current path, project, child project, and cleanup
  family;
- candidate size, item count, activity age, rebuildability, owning project,
  active-process state, privilege, deletion mode, and confidence;
- dry-run using the same eligibility rules as execution;
- persistent protected paths or whitelist;
- recent or unverifiable artifacts initially unselected;
- project, category, and individual-target exclusions;
- Trash, permanent, and tool-managed cleanup distinguished;
- auditable cleanup history and reclaimed-space reporting.

### 8.5 Docker and containers

Docker workflows include ordinary inspection and deliberately destructive
maintenance. Search must expose both.

When the user types `docker`, `containers`, or `docker cleanup`, relevant results
may include:

- list or inspect containers;
- view logs;
- start, stop, or restart project containers;
- stop all containers;
- stop and remove all containers;
- remove all images;
- prune unused networks;
- prune unused volumes;
- prune builder cache;
- perform complete Docker cleanup across containers, images, networks, volumes,
  system data, and builder cache.

Frequently used Docker actions should rise for Docker-related queries and on
hosts where the user commonly runs them. Exact aliases should make destructive
maintenance as fast to reach as any other intentional workflow.

For broad cleanup, show current Docker disk accounting, exact affected resource
classes, command sequence, host, and recoverability before confirmation. Do not
hide the action, replace it with a weaker operation, or assume destructive means
unwanted.

### 8.6 Services and processes

Surface start, stop, restart, health, logs, ports, and resource pressure based on
actual state. Keep project-local and host-wide scope distinct. Prefer specialist
monitoring tools for deep continuous inspection.

Host insight should cover total and per-core CPU, load, memory and swap, disk
capacity and I/O, network rates, process hierarchy, and Linux CPU/memory/I/O
pressure when available. Provide a lightweight contextual snapshot, then offer
`btm` as the preferred persistent deep-monitoring activity when installed.

### 8.7 PostgreSQL activity

Discover PostgreSQL contexts from standard connection configuration, project
environment, local sockets, and container context without exposing secrets.

Prioritize:

- active, waiting, blocked, and idle-in-transaction sessions;
- query and transaction duration;
- blocker dependency trees;
- connection saturation;
- database read, hit, temporary-file, deadlock, and I/O trends;
- expensive workload dimensions when statement statistics are available;
- vacuum and analyze health;
- replication state, retained WAL, and lag indicators.

Offer `pg_activity` as a persistent specialist activity. Query cancellation and
backend termination remain distinct actions; revalidate PID identity and query
before mutation.

### 8.8 Files and navigation

Nearby and project files should rank before broad filesystem matches. Actions
may open, preview, reveal, change directory, copy path, or pass a resource to
another workflow. A focused browser can handle deep navigation.

### 8.9 Personal and team workflows

Each custom workflow needs a clear name, description, stable identity, declared
scope, preview, arguments, risk class, provenance, trust state, and optional
alias or pin.

Do not execute arbitrary shell startup files merely to discover aliases.

### 8.10 SSH

Discover literal destinations from `$HOME/.ssh/config` and included files.
Treat wildcard and negated host rules as policies, not enumerable destinations.
For a selected alias, resolve effective OpenSSH behavior before connection and
explain destination, user, port, jump chain, identity filenames, forwarding,
host-key policy, and existing connection multiplexing without exposing secrets.

Execute the configured alias rather than rebuilding a simplified command that
could discard proxy, match, forwarding, canonicalization, or multiplexing rules.
Preserve native passphrase and host-key prompts. Changed host keys hard-stop.

### 8.11 Stack-aware cleanup

Cleanup must understand ownership and regeneration rather than delete folders by
name alone.

- **Rust:** resolve actual Cargo target directories, deduplicate shared targets,
  preview cleanup, and serialize projects sharing output.
- **Gradle:** prefer each project wrapper and `clean` task so custom build
  directories are respected. Treat project `.gradle` data and user-global caches
  as separate deeper cleanup.
- **Node:** identify workspace ownership, manager, lockfile, install strategy,
  Plug'n'Play or node-modules mode, symlinks, and checked-in caches. Separate
  dependency removal, locked restoration, and global cache cleanup.

Independent projects may clean in parallel. Shared workspaces, caches, and
output directories require dependency-aware ordering.

### 8.12 Remote environments

Make hostname, environment role, current path, remote state, and relevant
privilege visible. Use platform-appropriate actions and stronger confirmation
for production or sensitive hosts.

### 8.13 Specialist handoffs

Holla coordinates tools rather than recreating every expert workflow. Explain
why a tool fits, show its target, preserve context, honor user preference, and
return cleanly when possible.

### 8.14 Program insights and activity multiplexing

Holla must support more than launching a command and forgetting it. Long-lived
tasks, development servers, logs, monitors, and interactive programs become
activities that remain accessible after launch.

For every activity, retain:

- name and originating action;
- current, parent, child, or system scope;
- effective working directory and host;
- running, waiting, succeeded, failed, stopped, or detached state;
- live output and relevant program insights;
- start time, duration, exit status, and restart or stop actions;
- whether input can be attached safely.

Users should be able to start several activities and switch among them without
losing output or context. They may return to discovery, launch another child
task, inspect a system process, then return to the first activity.

This is a tab-capable, multiplexer-like activity model. Here, a tab means a
persistent labeled activity or insight context that the user can leave and
return to without losing state. It does not require a particular tab bar, pane
arrangement, or visual form. Required behavior is persistent named activities,
program insights, fast switching, clear state, and retained scope.

### 8.15 Plan-based workflows and dependency graphs

Some intents contain several commands with dependencies. Holla must represent
them as an execution graph, not pretend they are always one command or one
strictly sequential list.

The overall experience follows a guided plan lifecycle:

1. discover applicable work;
2. build the dependency graph;
3. review commands, effects, and available updates;
4. include or exclude optional work;
5. confirm the recalculated plan;
6. execute while following aggregate and per-step progress;
7. resolve failures or retry affected branches;
8. review the final outcome.

This is wizard-like guidance through decisions and stages. It does not force
graph nodes to execute sequentially when independent branches can run in
parallel.

A plan step may be:

- required or optional;
- ready, waiting, running, succeeded, failed, skipped, blocked, cancelled, or
  excluded;
- dependent on one or several earlier steps;
- independent and eligible to run in parallel;
- a prerequisite shared by several branches;
- a final verification that waits for several branches to converge.

Before execution, the user can inspect:

- overall intent and target host;
- every step and exact behavior;
- dependency relationships;
- which branches may run concurrently;
- required privileges and confirmation level;
- optional steps and default selections;
- expected effects and known uncertainty.

The user may exclude optional steps. Excluding a prerequisite must also disable
or invalidate dependent steps; Holla must explain that consequence and recompute
the plan. It must not silently run a dependent step without its prerequisite.

Parallel execution is allowed only when independence is known. Steps that share
exclusive resources, package-manager locks, mutable files, ports, services, or
other conflicting state must remain ordered even when they look unrelated.

During execution, the user can understand:

- completed, active, ready, waiting, blocked, failed, and excluded steps;
- overall progress and progress within each step;
- live output for any active or completed step;
- which dependency will unlock next;
- which branches are running concurrently;
- why a step is waiting or blocked.

Each step retains its own output and program insights as an activity. The user
can switch between aggregate plan status and individual step activity without
losing output.

Failure handling must preserve graph meaning:

- dependent steps become blocked when a prerequisite fails;
- independent branches may continue when policy allows;
- retrying a failed step must not repeat already successful unrelated work;
- the user may retry one step, retry its affected branch, skip an optional
  failure, or cancel remaining work;
- final summary distinguishes success, failure, exclusion, skip, and work never
  started.

The graph relationship is required product information. Its visual rendering as
a tree, graph, outline, tabs, timeline, or combination remains open design work.

### 8.16 System maintenance and upgrades

System-level discovery may recommend compound actions such as **Upgrade
everything on this system** when relevant package managers and tool managers are
available.

For a Debian host with global mise tools, the plan may contain:

- preflight inspection of operating system, privileges, package-manager locks,
  network availability, free disk space, and pending reboot state;
- refresh Debian package information;
- discover and present available Debian package upgrades;
- let the user include or exclude eligible package updates where supported;
- apply selected Debian upgrades using standard system methods;
- inspect globally managed mise tools and available versions;
- upgrade selected global mise tools;
- perform optional package cleanup;
- verify package state, tool versions, services, and reboot requirements.

Logical dependency example:

```text
Preflight
  -> Debian metadata -> Review Debian upgrades -> Apply Debian upgrades
  -> Inspect global mise tools -> Upgrade selected mise tools

Apply Debian upgrades + Upgrade selected mise tools
  -> Final verification

Apply Debian upgrades
  -> Optional package cleanup
```

This notation documents dependencies and possible parallel branches. It is not a
prescribed visual design.

The Debian and mise discovery branches may run in parallel after preflight when
they do not compete for the same resources. Applying changes follows the reviewed
graph and any required ordering.

The plan review must allow optional branch exclusion. For example, the user may
upgrade Debian packages but exclude mise upgrades and cleanup. Required
preflight and verification steps remain linked to whichever branches stay
enabled.

---

## 9. Personalization and ranking

Usage memory should distinguish global, host, project, exact path, query choice,
and current session. “Often used here” matters more than “often used somewhere.”

Suggested ranking order:

1. exact alias;
2. explicit pin;
3. live urgency;
4. local context match;
5. textual intent match;
6. contextual frequency and recency;
7. global frequency and recency;
8. stable default order.

Risk treatment never decreases because an action is frequent. Risk also must not
erase or arbitrarily demote a strong intent match.

Recommendation reasons may include:

- `branch is 3 commits behind`;
- `4 modified files`;
- `used 6 times in this project`;
- `defined by project task runner`;
- `service is unhealthy`;
- `12 GB generated artifacts`;
- `available on this host`.

Users must be able to pin, alias, hide, demote, restore, reset ranking, inspect
reasons, clear history, and disable personalization.

Default to local, minimal retention. Never retain secret arguments, credentials,
or sensitive output for ranking.

---

## 10. Safety, trust, and confidence

### Risk classes

- **Read-only:** inspection without intended mutation;
- **Mutating:** bounded state change;
- **Destructive:** data removal, discarded work, or difficult recovery;
- **Privileged or sensitive:** elevated access or protected environment.

Recommendation confidence is separate from risk. High relevance never weakens
safety treatment.

### Confirmation levels

- **Read-only:** run directly unless arguments require review.
- **Bounded mutation:** preview exact scope; require one explicit confirmation
  when the effect is surprising, sensitive, or difficult to reverse.
- **Destructive:** never execute from ordinary primary selection. Begin a
  dedicated review, then require a separate final confirmation.
- **Broad destructive:** use two gates; the second requires a target-bound typed
  phrase. This includes deleting everything below a folder, removing all containers or
  images, pruning all volumes, resetting a workspace, or affecting many targets.
- **Privileged or production:** apply the destructive flow whenever impact is
  broad or recovery is uncertain, even if the underlying command is normally
  considered mutating.

### Two-gate destructive confirmation

#### Gate 1: review the resolved plan

Show:

- action stated in plain language;
- absolute path, host, environment, and effective working directory;
- exact resources and resource classes affected;
- item count and estimated size when available;
- whether hidden files, nested folders, symlinks, mounts, volumes, images, or
  stopped resources are included;
- full command sequence or truthful operation summary;
- recoverable versus permanent behavior;
- known exclusions, uncertainty, privilege, provenance, and trust state.

The user must explicitly choose to review and continue. The interaction that
originally selected the action cannot count as this confirmation.

#### Gate 2: prove intent

Require a typed phrase containing both operation and resolved target. Examples:

- `DELETE EVERYTHING IN /work/scratch`
- `REMOVE ALL DOCKER DATA ON devbox`
- `RESTART PAYMENTS ON prod-eu-1`

For exceptionally broad actions, prefix the phrase with `I UNDERSTAND:`. A
generic `y`, `yes`, or `I know what I am doing` is insufficient because it does
not identify what the user is authorizing.

Execution remains unavailable until the phrase matches exactly.

### Confirmation invariants

- A clear cancellation action exits either gate.
- Aliases, frequency, automation, and remembered choices never bypass a gate.
- Confirmation is valid for one resolved plan, target, host, and invocation.
- Never remember approval for future destructive actions.
- Re-resolve the plan immediately before execution. If target or affected set
  changed materially, invalidate confirmation and show the new plan.
- Cancellation remains the default outcome unless the user explicitly
  continues.
- Do not use countdowns as a substitute for informed confirmation.
- Prefer recoverable operations, but clearly allow permanent operations after
  the required confirmation.
- Report each completed, skipped, and failed stage after execution.

Project-provided workflows require review before first use and after their
definition changes. Trust one exact definition, not an entire folder forever.

Never auto-execute destructive recommendations. Recommendation may assign an
action highest relevance; execution still begins at Gate 1.

Preview must truthfully describe actual execution, including conditional or
multi-step behavior.

---

## 11. Extensibility and composition

Holla should be useful without setup. Configuration enhances basic discovery;
it does not unlock it.

Users may add personal actions, team workflows, aliases, pins, saved locations,
remote hosts, preferred tools, and safety policies. Added capabilities inherit
the same search, preview, scope, trust, argument, and execution conventions.

Every action should have stable human-usable identity for direct invocation and
composition. Interactive and non-interactive entry points must preserve the same
scope and safety rules.

Own experiences where context, recommendation, or safety creates unique value.
Hand off where another terminal tool offers a stronger specialist experience.

---

## 12. Example journeys

### Rust project

From a nested project folder, Holla recognizes the project and suggests reviewing
modified files, running tests, building, opening project tasks, and inspecting
large generated artifacts. The user searches `test`, previews scope and behavior,
runs it, watches output, then chooses a useful follow-up.

### Disk usage

The user opens Disk, then Usage. Holla offers current folder, project, home,
volume, or custom path. A largest-first view appears quickly and deepens as data
arrives. The user reviews generated artifacts, selects recoverable deletion,
confirms exact targets, and receives per-item results.

### Project collection

In a recognized workspace, Holla suggests status across projects, reviewing
dirty projects, synchronizing safe projects, and opening a project by name.
Multi-project changes list every target and return per-project results.

### Monorepository root and child tasks

The user launches Holla from a monorepository root. Current-root recommendations
remain primary. Holla also discovers `apps/frontend`, `services/api`, and other
bounded children. Each child exposes tasks from its own `mise.toml` or equivalent
task definition.

The user starts the frontend development task. It runs with
`apps/frontend` as its effective working directory. The user then returns to
discovery, starts the API task from another child, and switches between both
running activities and their program insights.

### Nested child with parent ecosystem actions

The user launches Holla inside `apps/frontend`. Frontend-specific actions rank
first. Holla also explains that the monorepository root defines ecosystem setup
and required containers. Those parent actions remain easy to discover and run in
the root scope without losing the frontend context.

From the same session, the user enters System scope to inspect Docker status,
find a specific container, or choose a host-wide stop-all or cleanup action.
Every action retains its own scope and confirmation behavior.

### mise trust and child-task execution

From the monorepository root, the user searches for frontend tests. Holla finds
a namespaced task in the child's `mise.toml`, explains its child working
directory and dependency graph, then shows that the configuration is not yet
trusted.

The user reviews the exact configuration, commands, environment effect, and
trust scope. After explicit trust, Holla re-resolves the task and starts it in
the child directory. Dependencies and live output remain available as plan
activities.

### Git operations across child projects

The user asks for status across all child Git projects. Holla discovers and
deduplicates worktrees, distinguishes submodules, and inspects every project in
parallel. The user then creates a pull plan, excludes one dirty project, and
runs fast-forward-only pulls for eligible projects.

A later request to `checkout main` is understood as **switch each selected
project to its resolved primary branch**; projects whose primary branch is
`master` or another name use that actual branch. Dirty or ambiguous projects are
blocked without discarding work.

For cloning, Holla offers projects from the active GitHub account and selected
organizations. The user reviews account, owner, protocol, destination, primary
branch, and fork behavior before cloning.

### Follow logs from multiple containers

The user asks to follow logs from `api`, `worker`, and `scheduler`. If they belong
to one Compose project, Holla uses project-aware multi-service logs. Otherwise it
coordinates separate container streams. Output retains container identity, and
each stream remains independently inspectable within one combined activity.

### Diagnose host resource pressure

Holla notices sustained CPU or memory pressure and recommends **Show system
resources**. It provides enough context to identify the pressure domain, then
offers `btm` as a persistent deep-monitoring activity. The user can return to
Holla without losing the monitor or its host context.

### Diagnose PostgreSQL blocking

The user asks who is blocking a database. Holla identifies a blocker dependency
tree, query and transaction ages, wait state, database, user, client, and affected
sessions. It offers `pg_activity` for live inspection.

If the user chooses mutation, cancel-current-query is offered before
terminate-backend. Holla revalidates server, PID, and query identity immediately
before confirmation.

### Connect through SSH configuration

The user searches a literal alias discovered from `$HOME/.ssh/config`. Holla
resolves included configuration and explains effective destination, port, user,
jump chain, identity filenames, forwarding, host-key behavior, and existing
connection multiplexing. It then launches the original alias, preserving native
authentication and fingerprint checks.

### Clean Rust, Gradle, and Node artifacts

The user asks to clean build artifacts under a workspace. Holla groups Cargo
target directories, Gradle builds, and Node dependency trees beneath their
owning projects. It deduplicates shared targets and workspace roots, measures
reclaimable space, and leaves recent or uncertain data unselected.

Independent projects become parallel branches. Shared Cargo targets, tasks
within one Gradle build hierarchy, tasks within one Node workspace, and shared
caches remain ordered; independent roots may run in parallel. The user excludes
one project, reviews regeneration methods, confirms exact paths, then receives
reclaimed, skipped, blocked, and failed results per branch.

### Remote server

Remote host and environment role remain prominent. Recommendations prioritize
service health, logs, system pressure, disk state, and installed monitoring
tools. Sensitive changes require explicit target review.

### Repeated workflow

A frequently selected deployment workflow rises within its project. Holla does
not invent a hidden shortcut. It offers an explicit alias; once assigned, that
alias behaves deterministically.

### Docker cleanup

The user types `docker clean`. Holla shows **Clean Docker completely** near the
top because the query is an exact intent match and the workflow is frequently
used on this host. Less broad actions such as **Stop all containers** and **Prune
builder cache** remain adjacent alternatives.

Selecting complete cleanup shows Docker disk usage and the full sequence:
containers will stop and be removed, images will be removed, then networks,
system data, volumes, and builder cache will be pruned. The user confirms and
watches each stage complete.

Complete cleanup uses two gates. First, the user reviews Docker accounting,
host, affected resource classes, and full sequence. Second, the user types
`REMOVE ALL DOCKER DATA ON <host>`. Destructive scope affects execution
treatment, not discoverability or ranking eligibility.

### Upgrade everything on a Debian host

The user asks to upgrade everything on the current system. Holla discovers
Debian package management and globally managed mise tools, then produces a plan
instead of executing immediately.

The user reviews available Debian packages, global mise updates, prerequisites,
optional cleanup, and final verification. They exclude one optional cleanup step
and one mise tool from the plan. Holla recalculates dependencies before asking
for confirmation.

Execution begins with preflight. Debian package discovery and mise inspection run
in parallel when safe. Each step exposes progress and live output. Applying
Debian updates waits for package discovery and review; selected mise updates use
their independent branch. Final verification waits for enabled upgrade branches.

The user switches among overall plan status, Debian output, mise output, and
program insights through the persistent activity model. If one branch fails,
dependent verification reflects that failure while an independent branch may
finish. The final result explains exactly what changed, failed, was excluded, or
never started.

---

## 13. Product principles

1. Here first.
2. Intent before syntax.
3. One root experience.
4. Useful before typing.
5. Search across domains.
6. Resources plus actions.
7. Primary action plus discoverable alternatives.
8. Adaptive but predictable.
9. Explain every recommendation.
10. Keep scope visible.
11. Feel immediate.
12. Make safety structural.
13. Coordinate instead of recreating.
14. Zero configuration, optional mastery.
15. Keyboard-first, not shortcut-secret.
16. Local-first and privacy-conscious.

---

## 14. Product boundaries

Holla should not become:

- an exhaustive command encyclopedia;
- a shell replacement;
- a default AI command generator;
- an autonomous cleanup daemon;
- a replacement for every specialist TUI;
- a global menu that merely displays the current path;
- an unbounded filesystem crawler;
- a hidden automation engine;
- a macOS-only mental model;
- a product requiring configuration before usefulness;
- a system that silently expands scope;
- a launcher whose learned ranking defeats muscle memory.

AI may assist explanation or intent classification only when provenance,
preview, scope, and user control remain intact. Deterministic local behavior is
the default.

---

## 15. Concept exploration brief

Use this concept to create and compare possible product representations before
settling on one.

Explore conceptually:

- how context can remain understandable without overwhelming the experience;
- how users distinguish recommendations, search matches, and navigation;
- how resources expose primary behavior and contextual alternatives;
- how keyboard discovery works without memorizing shortcuts;
- how live discovery avoids disrupting active use;
- how different representations can share one mental model;
- how complex flows deepen progressively from Root Search;
- how risk, confidence, trust, and scope remain distinct;
- how completion returns useful next actions;
- how local and remote sessions feel consistent but unmistakable.
- how users move between current, parent, child, and system scope;
- how several running activities and program insights remain accessible;
- whether tab-like, multiplexer-like, or another model best expresses those
  relationships.
- how dependency graphs, optional branches, parallel work, and convergence are
  made understandable without overwhelming simple actions;
- how plan editing and exclusion communicate downstream consequences;
- how aggregate progress and per-step live output stay connected.

No representation is supplied or preferred by this document. Create alternatives
from the needs, workflows, and constraints; challenge every example; prefer the
clearest terminal-native model that satisfies the product principles.

The result should be one coherent product concept, not disconnected menus or a
gallery of commands.

---

## 16. Experience quality bar

Users must always understand:

- current path, host, and environment;
- active project or workspace;
- selected resource or action;
- result type and scope;
- keyboard focus;
- whether discovery is still running;
- why a recommendation appears;
- what the primary action will do;
- what will change;
- whether confirmation or trust is required;
- whether work succeeded, failed, or remains active.

Prefer progressive disclosure, clear information hierarchy, stable selection,
and small result sets. Exact visual treatment remains open. Do not compensate
for weak ranking by exposing more categories at once.

---

## 17. Success criteria

Holla succeeds when:

- opening without typing reveals a plausible next action;
- common actions take few predictable keystrokes;
- exact folder, project, and host scope is obvious;
- project workflows are discoverable without inspecting configuration;
- repeated workflows become explicit shortcuts;
- global capabilities do not crowd out local work;
- destructive operations are understandable and deliberate;
- local and remote use share one mental model;
- specialist tools feel integrated;
- users trust recommendations enough to invoke Holla habitually.

Do not optimize for number of commands. Optimize for surfacing the right action
with less cognitive effort.

---

## 18. Open product questions

1. How should exact folder context balance against nearest project root?
2. How should Holla distinguish a monorepo from unrelated neighboring projects?
3. Which signals qualify an item for Suggested Here?
4. How much live state can be discovered without slowing launch?
5. How should streaming results reorder before and after navigation begins?
6. Which command paths feel natural without conflicting with text input?
7. Should actions run, insert, or remember a preferred execution mode?
8. How can shell aliases be discovered without executing startup files?
9. What identity follows a project when it moves or is cloned elsewhere?
10. Which memory belongs to path, project, host, or user?
11. Which focused experiences should Holla own versus hand off?
12. How should sensitive hosts declare their role?
13. What happens when a desired specialist tool is unavailable?
14. How should long-running work return to Root Search?
15. How should multi-target partial failure be explained and recovered?
16. How should narrow terminals, reduced color, accessibility, and remote latency
    affect presentation?
17. How deep should child discovery go before requiring explicit exploration?
18. How should ancestor tasks declare that they apply to descendant contexts?
19. How should concurrent activities share input, output, and terminal control?
20. When should a running program detach, remain interactive, or open in an
    external terminal multiplexer?
21. Which evidence is sufficient to declare two mutating steps independent?
22. How should optional-step exclusion alter confirmation and final reporting?
23. How should a large execution graph collapse detail while preserving blocked
    dependencies and parallel progress?

---

## 19. Prompt handoff

When giving this document to another agent, ask it to:

- treat Holla as a new product concept;
- brainstorm several terminal-native interaction models;
- compare their clarity against this document's needs and principles;
- choose and justify the strongest coherent direction;
- turn that direction into a runnable product concept or prototype;
- use realistic project, Git, task, disk, service, and remote states;
- demonstrate empty, loading, partial, success, failure, and dangerous states;
- keep keyboard operation first-class;
- avoid researching or copying a named launcher unless explicitly requested;
- use local supporting references only when more pattern detail is needed;
- treat all examples as semantic requirements, never visual specifications;
- invent and validate the visual representation independently;
- stop at the product boundaries above.

Target judgment:

> **Yes, this is how a context-aware action launcher should feel when designed
> natively for the terminal.**

---

## 20. Optional local references

This document is self-contained. These local notes offer deeper conceptual
pattern analysis without requiring external research. They also do not prescribe
visual design:

- [Universal launcher patterns](reference/universal-launcher-patterns.md)
- [Terminal workflow patterns](reference/terminal-workflow-patterns.md)
- [Context-adaptive product principles](reference/context-adaptive-product-principles.md)
- [Technology-stack workflows](reference/technology-stack-workflows.md)
- [Mole-informed disk and cleanup patterns](reference/mole-disk-cleanup-patterns.md)

### Final interpretation rule

These references and every example above supply product context only. They are
not wireframes, visual references, component specifications, or interaction
templates. The future designer owns how the ideas are represented and should
derive that representation through independent exploration.
