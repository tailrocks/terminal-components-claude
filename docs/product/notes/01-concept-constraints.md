# 01 · Concept constraints every Holla design must satisfy

Source of truth: `docs/product/CONCEPT.md` (cited as CONCEPT §n), `references/`
(cited as launcher, workflow, principles, tech-stack §n, mole), and `GOAL.md`
(cited as GOAL §n). One line per constraint, imperative. No visuals prescribed.

## A. Context and scope model

- Start every launch from the exact working directory; it is the primary contextual object and the default source of recommendations, ranking, intent, and execution scope (CONCEPT §5.1).
- Treat current-folder priority as a relevance bias, never a hard filter; explicit queries (`docker`, `children`, `workspace tasks`, `parent setup`) may elevate another scope immediately (CONCEPT §5.1).
- Allow urgent state from another scope to be recommended only when its origin and target are both stated (CONCEPT §5.1).
- Expand outward through rings only as needed: Here → Project → Workspace/neighborhood → Host → Personal/team (CONCEPT §5.2, launcher "Holla interpretation").
- Keep global capabilities available but never let them displace strong local recommendations (CONCEPT §5.2, §17).
- Expose four navigable scope directions: Current, Parent/ancestors, Children/descendants, System (CONCEPT §5.3).
- Let ancestor roots contribute ecosystem tasks (e.g. **Start development ecosystem**, **Start required containers**, **Run workspace setup**) inside a descendant folder, each stating defining scope and execution directory (CONCEPT §5.3, principles "ancestors and children").
- Bound child discovery structurally (manifests, `.git`, `mise.toml`, Compose files); never run an unlimited recursive scan (CONCEPT §5.3, §14).
- Run a child task in the child's own directory, never the launch directory (CONCEPT §5.3, §12 monorepo journey).
- Make System scope reachable from any folder and label its actions as host-affecting, not project-affecting (CONCEPT §5.3).
- Move among Current/Parent/Children/System without closing or relaunching; return to Current instantly (CONCEPT §5.4, §5.5 #10-11).
- Show on every result: where discovered, where it executes, which scope class it belongs to, and why a noncurrent result is relevant now (CONCEPT §5.4).
- Never silently change effective working directory or execution target; mark every cross-scope transition explicitly (CONCEPT §5.1, §5.5 #5, §14).
- Keep path and host in persistent interface context, not buried in preview (CONCEPT §5.1, §5.5 #1-2).
- Mark stale, cached, partial, and still-loading information distinctly (CONCEPT §5.5 #6, §7 preview questions).
- Keep selection stable while streamed results arrive; preserve the selected item's identity after navigation begins (CONCEPT §5.5 #9, principles "stream without disruption").
- Preserve each action's effective working directory across search, preview, execution, and history (CONCEPT §5.5 #12).
- Recognize intentional workspaces (monorepo, Cargo workspace, mise monorepo roots) without assuming every parent folder is one (CONCEPT §8.3, §18 Q2).
- Treat mise as a first-class context system: effective config from folder through ancestors, monorepo root and configured child roots, local vs global tool ownership (tech-stack §1, CONCEPT §8.1).

## B. Root experience, search, ranking, aliases

- Provide one entry point from any path; never require a category choice before expressing intent (CONCEPT §6.1, launcher "One root surface").
- Make the empty state a recommendation surface carrying four concepts: Suggested here, Recent here, Explore (Tasks, Git, Files, Disk, Services, System), Discovery status (CONCEPT §6.2, §7 initial state).
- Ensure opening without typing already reveals a plausible next action with a visible reason (CONCEPT §17, GOAL §5).
- Search actions and resources together in one query; show type and scope on every result (CONCEPT §6.3, launcher "Mixed result types").
- Match intent, labels, descriptions, keywords, resource names, aliases, and relevant state; `why disk full`, `tests`, `sync projects`, `service logs`, `open config` must work (CONCEPT §6.3).
- Let a query like `logs` return a project task, Compose service logs, a system service, a nearby log file, and personal automation side by side (CONCEPT §6.3).
- Represent each resource once with one coherent identity; its best current action is primary, related operations are alternatives, never duplicates (CONCEPT §6.4-6.5, principles "Resources plus actions").
- Result kinds must include: direct action, resource, recommendation, domain flow, specialist handoff (CONCEPT §6.4).
- Provide a separate interaction for alternatives: inspect/explain, alternate operations, change scope, preview or copy command, insert into shell, run now, pin/alias/hide/reset, choose preferred specialist tool (CONCEPT §6.5).
- Keep navigation grammar constant: text filters, navigation moves focus, direct invocation runs primary, a distinct interaction opens alternatives, back returns to prior context (launcher "Consistent navigation").
- Distinguish capability, action, and recommendation; availability is not relevance (CONCEPT §2.4).
- Rank in this order: exact alias, explicit pin, live urgency, local context match, textual intent match, contextual frequency/recency, global frequency/recency, stable default order (CONCEPT §9).
- Exact aliases beat learned ranking and never drift; e.g. `du` → disk usage, `gp` → Git pull, `test` → preferred test action here, `d` then `u` → Disk then Usage (CONCEPT §6.6, launcher "Fuzzy discovery and deterministic shortcuts").
- Scope usage memory by global, host, project, exact path, query choice, and session; "used here" outranks "used somewhere" (CONCEPT §9, workflow "Contextual shell history").
- Attach a short reason tied to live state, context, or observed use to every recommendation; allow inspecting deeper reasoning (CONCEPT §5.5 #7, §9, principles "Explain relevance").
- Provide user ranking controls: pin, alias, hide, demote, restore, reset, inspect reasons, clear history, disable personalization (CONCEPT §9, launcher "User control over ranking").
- Offer an explicit alias for a repeated workflow; never invent a hidden shortcut (CONCEPT §12 repeated workflow).
- Never let risk demote a strong intent match, and never let frequency weaken risk treatment (CONCEPT §9, principles "Separate relevance, confidence, and risk").
- Retain no secret arguments, credentials, or sensitive output for ranking; keep memory local, minimal, inspectable, clearable, optional (CONCEPT §9, principles "Default to local control").
- Never compensate for weak ranking by exposing more categories at once; keep result sets small (CONCEPT §16).
- Collect parameters through structured fields showing name, expected value, default, validation, secret sensitivity, with the resulting command previewable (CONCEPT §6.7).
- Give contributed capabilities (personal scripts, team workflows) the same search, metadata, arguments, preview, and safety conventions as built-ins (launcher "Native treatment", CONCEPT §8.9, §11).
- Never execute shell startup files to discover aliases (CONCEPT §8.9).
- Give every action a stable human-usable identity for direct/non-interactive invocation with identical scope and safety rules (CONCEPT §11).
- Support execution modes: run, confirm, insert editable command, open focused flow, hand off to TUI, continue in background (CONCEPT §6.8).
- After completion show outcome, duration, affected scope, failures, and useful follow-ups (CONCEPT §6.8).

## C. Preview and explanation

- Answer in preview: what will happen, what path/project/service/host is targeted, why recommended, what will change, live/cached/partial status, whether confirmation or trust is required (CONCEPT §7).
- On active selection convey identity, type, scope, primary behavior, alternatives, recommendation reason, current focus (CONCEPT §7).
- Show exact executable names and flags in preview; never require them as input (principles "Intent before syntax").
- Show exact path, project, host, affected resources, generated command, and recoverability; never hide multi-step behavior behind a simplified preview (principles "Preview scope truthfully", CONCEPT §10).
- Describe conditional and multi-step behavior truthfully; preview equals actual execution (CONCEPT §10 last paragraph).
- Show provenance and trust state for contributed workflows; require review before first use and after definition change; trust one exact definition, not a folder (CONCEPT §8.9, §10, principles "Earn trust").
- Explain why a specialist handoff fits, its target, and preserved context (CONCEPT §8.13).
- Explain step working directory and dependency graph for a mise task before running (CONCEPT §12 mise trust journey).

## D. Safety

- Classify every action into a risk class: Read-only, Mutating, Destructive, Privileged/sensitive (CONCEPT §10).
- Keep recommendation confidence separate from risk; high relevance never weakens safety treatment (CONCEPT §10, principles).
- Keep destructive actions searchable, recommendable, aliasable, learnable; risk changes treatment, never availability or ranking eligibility (CONCEPT §2.5, §8.5, §12 docker cleanup, principles "Destructive actions remain first-class").
- Confirmation levels: Read-only runs directly unless arguments need review; Bounded mutation previews scope and asks one explicit confirmation when surprising/sensitive/hard to reverse; Destructive never runs from primary selection, opens dedicated review then separate final confirmation; Broad destructive uses two gates with typed target-bound phrase; Privileged/production applies destructive flow when impact broad or recovery uncertain (CONCEPT §10).
- Broad destructive includes: delete everything below a folder, remove all containers or images, prune all volumes, reset a workspace, affect many targets (CONCEPT §10).
- Gate 1 must show: plain-language action; absolute path, host, environment, effective cwd; exact resources and classes; item count and size; inclusion of hidden files, nested folders, symlinks, mounts, volumes, images, stopped resources; full command sequence; recoverable vs permanent; exclusions, uncertainty, privilege, provenance, trust state (CONCEPT §10).
- The interaction that selected the action cannot count as Gate 1 acceptance; user must explicitly choose to continue (CONCEPT §10).
- Gate 2 requires a typed phrase containing both operation and resolved target; generic `y`, `yes`, `I know what I am doing` are insufficient (CONCEPT §10, principles "Bind confirmation").
- Prefix Gate 2 phrase with `I UNDERSTAND:` for exceptionally broad actions (CONCEPT §10, GOAL §4 P3).
- Keep execution unavailable until the phrase matches exactly (CONCEPT §7, §10).
- Provide a clear cancellation from either gate; cancellation is the default outcome (CONCEPT §10 invariants).
- Never let aliases, frequency, automation, or remembered choices bypass a gate (CONCEPT §10, principles).
- Bind confirmation to one resolved plan, target, host, and invocation; never remember approval (CONCEPT §10).
- Re-resolve the plan immediately before execution; any material change in target or affected set invalidates confirmation and shows the new plan (CONCEPT §10, principles "Bind confirmation").
- Never use countdowns as informed confirmation (CONCEPT §10).
- Prefer recoverable operations but allow permanent ones after required confirmation (CONCEPT §10).
- Report each completed, skipped, and failed stage after execution (CONCEPT §10, workflow "Destructive maintenance").
- Never auto-execute destructive recommendations; execution starts at Gate 1 even at top relevance (CONCEPT §10).
- Destructive-folder review must state resolved path, host, item count, estimated size, hidden/nested inclusion, recovery mode, irreversibility (CONCEPT §7).
- Apply stronger confirmation on production or sensitive hosts; make host role visible (CONCEPT §8.12, §12 remote server).
- Show PID, identity, hierarchy, signal, and likely effect before signaling a process (tech-stack §4).
- Never place a password in commands, arguments, history, or persisted plans; never reveal tokens or key contents (tech-stack §2, §6, §8).

## E. Plans and dependency graphs

- Turn compound intents into a reviewable dependency graph, never one opaque command or fixed linear script (CONCEPT §6.8, §8.15, principles "Represent compound intent").
- Follow the plan lifecycle: discover → build graph → review commands/effects/updates → include/exclude optional work → confirm recalculated plan → execute with aggregate + per-step progress → resolve failures/retry → review outcome (CONCEPT §8.15).
- Step attributes: required or optional; dependent on one or several steps; independent/parallel-eligible; shared prerequisite; final convergence verification (CONCEPT §8.15).
- Step states: ready, waiting, running, succeeded, failed, skipped, blocked, cancelled, excluded (CONCEPT §8.15).
- Pre-execution inspection: intent and target host, each step's exact behavior, dependencies, concurrent branches, privileges and confirmation level, optional steps with defaults, expected effects and uncertainty (CONCEPT §8.15).
- Excluding a prerequisite must disable or invalidate dependents with an explained consequence and a recomputed plan; never run a dependent without its prerequisite (CONCEPT §8.15, principles).
- Required preflight and verification stay linked to whichever branches remain enabled (CONCEPT §8.16).
- Run steps in parallel only when independence is known; order steps sharing exclusive resources, package-manager locks, mutable files, ports, services, or conflicting state (CONCEPT §8.15, principles "Preserve graph meaning").
- During execution show completed/active/ready/waiting/blocked/failed/excluded, overall and per-step progress, live output per step, next unlocking dependency, concurrent branches, why a step waits (CONCEPT §8.15).
- Retain each step's output and insights as an activity; switch between aggregate and step without losing output (CONCEPT §8.15).
- Failure: dependents become blocked; independent branches may continue per policy; retry never repeats successful unrelated work; user may retry one step, retry branch, skip optional failure, or cancel remaining (CONCEPT §8.15).
- Final summary distinguishes success, failure, exclusion, skip, never started (CONCEPT §8.15, §12 upgrade journey).
- Upgrade-everything graph: Preflight → {Debian metadata → Review Debian upgrades → Apply Debian upgrades} ∥ {Inspect global mise tools → Upgrade selected mise tools}; Apply + Upgrade → Final verification; Apply → Optional package cleanup (CONCEPT §8.16).
- Preflight checks: OS, privileges, package-manager locks, network, free disk, pending reboot (CONCEPT §8.16).
- Present available upgrades before mutation; allow tool and package exclusions (CONCEPT §8.16, workflow "System-upgrade orchestration", tech-stack §1 upgrade plan).
- Feed mise task dependency, post-dependency, and wait relationships directly into the graph (tech-stack §1 hierarchy).

## F. Activities and multiplexing

- Turn long-lived tasks, dev servers, logs, monitors, interactive programs into named activities that persist after launch (CONCEPT §8.14).
- Retain per activity: name and originating action; scope class; effective cwd and host; state running/waiting/succeeded/failed/stopped/detached; live output and insights; start time, duration, exit status, restart/stop actions; whether input can attach safely (CONCEPT §8.14).
- Allow starting several activities, returning to discovery, launching another child task, inspecting a process, then returning without losing output or scope (CONCEPT §8.14, §12, GOAL §4 P4).
- Multi-container logs: one Compose project → project-aware multi-service logs; otherwise one prefixed stream per container, each independently inspectable in one combined activity (CONCEPT §12, tech-stack §3).
- `btm` and `pg_activity` open as persistent activities preserving host and launch context (CONCEPT §8.6-8.7, tech-stack §4, §6).
- Keep tab meaning semantic (persistent labeled activity), not a mandated tab bar (CONCEPT §8.14).

## G. Per-domain required facts (priority stack, CONCEPT §8.0)

### mise (tech-stack §1, CONCEPT §8.1)
- Show effective config chain, monorepo root and child roots, local vs global tool ownership.
- Show tools as active, installed, missing, outdated with current/requested/latest; distinguish within-range vs range-change upgrades; show lockfile and config effects.
- Show per task: name, definition file, working directory, dependencies, environment; namespaced child identity like `//projects/frontend:build`.
- Show trust state and trust scope (current/parent/descendant) before running or installing; require renewed trust on changed content; never rely on silent auto-trust.
- Commands: `mise use node@26 python@3.13`, `mise use --global node@26`, `mise install --dry-run`, `mise install --include-task-tools --dry-run`, `mise install --monorepo --include-task-tools --dry-run`, `mise outdated --json`, `mise outdated --local --json`, `mise upgrade --dry-run`, `mise upgrade --interactive`, `mise upgrade --exclude go`, `mise tasks ls --json`, `mise tasks ls --all --json`, `mise tasks info //projects/frontend:build --json`, `mise tasks deps --dot //projects/frontend:build`, `mise run --dry-run //projects/frontend:build`, `mise run //projects/frontend:build`, `mise trust --show`, `mise trust /exact/path/mise.toml`, `mise trust --untrust /exact/path/mise.toml`.

### Git and GitHub (tech-stack §2, CONCEPT §8.2)
- Per project retain: path and type; branch or detached; upstream and ahead/behind; staged/unstaged/untracked/conflicting; stash count; remotes and primary branch; action eligibility and block reason.
- Discover `.git` dirs and files, stop inside Git metadata, resolve real top-level paths, dedupe worktrees, separate submodules from independent projects.
- Pull default is fast-forward-only; divergence blocks for separate review; never auto-reset or auto-stash.
- Resolve each project's actual primary branch; never hard-code `main`/`master`; `checkout main`, `checkout master`, `switch to default`, `switch to primary branch` all mean "switch to resolved primary".
- Block automatic switching on dirty work, detached state, unresolved operations, missing branch, remote ambiguity; never discard changes.
- Push-all excludes force, deletion, mirror, tag overwrite; force-with-lease is a separate dangerous workflow with fresh remote-state review and target-bound confirmation.
- Bulk plans: selectable hierarchy, exclusions, bounded parallelism, per-project output, isolated failure.
- Clone review shows account, host, visibility, protocol, owner, primary branch, destination, fork/upstream behavior; never overwrite a destination with user data; never reveal tokens.
- Commands: `git -C <project> status --porcelain=v2 --branch --show-stash`, `git -C <project> pull --ff-only`, `git -C <project> push --dry-run`, `git -C <project> push`, `git -C <project> switch <primary-branch>`, `gh auth status --active`, `gh org list --limit 100`, `gh repo list <owner> --limit 100 --no-archived`, `gh repo clone OWNER/REPOSITORY [directory]`.

### Docker (tech-stack §3, CONCEPT §8.5)
- Show container name, image, state, ports, project ownership, writable-layer size, CPU, memory, net I/O, block I/O, process count.
- Expose Compose project scope separately from host scope; preserve exact Compose file, project directory, project name.
- Freeze exact container identities before review; no shell substitution as product contract.
- Stop-and-remove-all plan: discover running+stopped → review names/images/state/project/host → stop running gracefully → remove stopped → verify remaining; force removal is a distinct harsher action.
- Complete cleanup plan: inventory containers, images, custom networks, named+anonymous volumes, builders, caches, reclaimable space → freeze and review → stop running containers → remove containers → (container barrier) remove unused images ∥ networks ∥ volumes → clean builder and Buildx caches → rescan and report reclaimed/remaining.
- Named volumes get strongest treatment; state that generic system prune does not remove every named volume; show exact resource classes.
- Show Docker disk accounting (`docker system df`) before broad cleanup; Gate 2 phrase `REMOVE ALL DOCKER DATA ON <host>`.
- Commands: `docker ps`, `docker ps -a`, `docker ps --size`, `docker system df`, `docker system df -v`, `docker stats`, `docker stats --no-stream`, `docker stop web db`, `docker rm web db`, `docker logs --tail 200 api`, `docker logs -f --since 10m api`, `docker compose ps --all`, `docker compose logs -f --tail 200 api worker`, `docker compose stop`, `docker compose down`, `docker image prune -a`, `docker network prune`, `docker volume prune -a`, `docker builder prune -a`, `docker buildx prune -a`.

### btm and system snapshot (tech-stack §4, CONCEPT §8.6)
- Snapshot covers total/per-core CPU, load and history, used/available memory, swap use and pressure, filesystem capacity, per-device read/write and I/O pressure, per-interface rx/tx rates and totals, process search/sort/hierarchy/CPU/memory/state, temperatures and battery where supported, Linux CPU/memory/I/O pressure distinguishing partial from system-wide thrashing.
- Intents: Show system resources; What uses CPU?; What uses memory?; Show process tree; What causes disk I/O?; Show network activity; Open `btm`; Find and stop a process.
- Do not recreate deep monitoring; hand off to `btm` as persistent activity with host context preserved.
- Command: `btm`.

### Disk and Mole-informed cleanup (tech-stack §5, mole, CONCEPT §8.4)
- Intents: Why is this disk full?; Analyze this directory; Find largest files and folders; Show reclaimable space; Clean current project; Clean child projects; Remove Rust targets, Gradle outputs, or Node dependencies; Review system cleanup; Show cleanup history.
- Separate observation, review, and deletion; show largest-first fast, deepen progressively; never recommend deletion only because a path is large.
- Preserve hierarchy: Host → filesystems/mounts, current directory (large dirs, large files, rebuildable artifacts), child projects (frontend node_modules, backend Cargo target, android Gradle outputs), system cleanup families (package caches, logs, temp, app caches).
- Cleanup families: system data, user essentials, application caches, browsers and cloud tools, developer-tool caches, virtualization and container data, application leftovers, backups and firmware, large files, project artifacts; adapt to OS and installed stack.
- Project artifacts: `node_modules` and dist output, Rust and Maven `target`, Gradle and general build dirs, project-local Gradle state, task-runner caches, coverage output, Python envs and caches; confirm ownership, manager, activity, symlinks, config, regeneration path.
- Candidate facts: resolved path; filesystem, scope, owning project; category and item count; measured size; last activity; why rebuildable/removable; active process; required privilege; deletion method (Trash, permanent, tool-managed); confidence, uncertainty, skip reason; dependencies and conflicts.
- Freshness defaults: recent or unverifiable artifacts start unselected; old confidently rebuildable start selected; selection is recommendation, not permission.
- Dry-run uses identical eligibility rules to execution; persistent protected-path list; refuse protected, ambiguous, busy, live, unverifiable targets; state when missing privilege makes discovery partial; revalidate immediately before deletion; never delete a parent because all shown children were selected.
- Recovery modes: move to Trash/recoverable location; permanent deletion; cleanup through owning tool; auto-regenerable; requires later networked reinstall.
- Keep history of targets, method, reclaimed space, skips, failures, time; history is audit unless restoration is supported.
- Editable plan: resolve scope → measure capacity/usage → discover progressively → group by filesystem/project/family → build dependency graph → preselect stale confident → allow project/category/path exclusions → recalc size and dependents → review targets and recovery → confirm by risk → parallel independent branches → stream per-step → report removed/trashed/skipped/failed/reclaimed → preserve audit.
- **Remove all build artifacts under `/work`** binds Gate 2 to `/work` and the resolved plan (mole "Editable cleanup plan").
- Mole reference commands (concept evidence, not Holla's language): `mo analyze [path]`, `mo analyze --json [path]`, `mo clean --dry-run`, `mo clean`, `mo clean --whitelist`, `mo purge`, `mo purge --dry-run`, `mo purge --paths`, `mo installer`, `mo history`, `mo history --json`, `mo status`, `mo status --json`, `mo status --watch --interval 2s`.

### PostgreSQL and pg_activity (tech-stack §6, CONCEPT §8.7)
- Discover connection context from environment, service definitions, local sockets, project settings, containers; never display passwords.
- Insights: connections by state vs configured max; active query/transaction/backend age; idle-in-transaction; wait event and blocking dependency tree; commit/rollback/read/hit/temp-file/deadlock/I/O trends; per-statement calls, total and mean latency, rows, physical reads, spill, WAL; table live/dead estimates and vacuum/analyze history; replication state, WAL backlog, slots, write/flush/replay lag.
- Rank total load, mean latency, frequency, reads, spill, WAL independently; never one "worst query" score; label cumulative counters and lag accurately.
- Blocking tree shows blocker chain, query and transaction ages, wait state, database, user, client, affected sessions.
- Offer cancel-current-query before terminate-backend; before either re-read server, database, PID, user, application, query identity, duration, blockers; PID or query change invalidates confirmation.
- Commands: `pg_activity`, `pg_activity -h HOST -p PORT -U USER -d DATABASE`, `pg_activity --duration-mode 2 --min-duration 5`.

### Rust, Cargo, nextest (tech-stack §7, CONCEPT §8.1)
- Group workspace members, targets, root, actual target directory via Cargo metadata; not every descendant manifest is a project; target may not live at `<project>/target`.
- Prefer nextest when installed or configured; keep ordinary Cargo tests; retain per-test state, duration, retries, flaky status, failure output, profile, summary.
- Offer package, workspace, target, feature, profile, toolchain, locked, offline, program-args choices only when relevant.
- Commands: `cargo check`, `cargo check --workspace`, `cargo build`, `cargo build --release`, `cargo run`, `cargo run -p api`, `cargo run --bin server -- --port 8080`, `cargo test`, `cargo test parser`, `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo nextest run`, `cargo nextest run -p api`, `cargo nextest run --workspace`, `cargo nextest run --profile ci`.

### SSH (tech-stack §8, CONCEPT §8.10)
- Enumerate literal positive host aliases from `$HOME/.ssh/config` and includes recursively; wildcard, catch-all, negated entries are rules, not destinations.
- Resolve effective behavior via installed OpenSSH, not static parsing alone.
- Preview: entered alias and resolved `user@host:port`; jump chain or proxy command; identity filenames (no content); agent and identities-only; requested terminal or remote command; local/remote/dynamic forwarding; host-key policy; existing multiplexing state.
- Run the alias unchanged; preserve native password, passphrase, new-host fingerprint prompts; never disable host-key verification; changed host key hard-stops.
- Expose multiplexing status and explicit stop/exit; never silently create or destroy shared masters.
- Commands: `ssh production-db`, `ssh -J bastion app.internal`.

### Rust artifact cleanup (tech-stack §9, CONCEPT §8.11)
- Plan: discover independent workspaces → resolve actual target dirs → dedupe shared targets → measure each and total → allow workspace exclusion and release/doc/package choice → preview via Cargo → confirm exact targets → clean independent in parallel, serialize shared → report reclaimed and failures.
- Never delete every folder named `target` blindly.
- Commands: `cargo clean --dry-run --verbose`, `cargo clean`, `cargo clean --release`, `cargo clean --doc`, `cargo clean -p api`.

### Gradle cleanup (tech-stack §10, CONCEPT §8.11)
- Discover roots via settings files and wrappers; prefer project wrapper and `clean` tasks so custom build dirs are respected.
- One multi-project build is one hierarchy; independent nested builds are separate branches; show affected projects, task paths, size, rebuild cost.
- Separate ordinary build cleanup from deeper removal of project `.gradle`, user caches, wrapper distributions, daemon logs; never confuse generated `.gradle` with checked-in `gradle`; inspect or stop daemons before deleting active state.
- Commands: `./gradlew clean`, `./gradlew :service-a:clean`, `./gradlew --stop`.

### Node dependency cleanup (tech-stack §11, CONCEPT §8.11)
- Identify package manager, lockfile, workspace ownership, install mode (PnP vs node_modules), manager config, symlinks, checked-in caches before removal; never assume Yarn uses `node_modules`; never traverse nested trees or follow symlinks outside resolved location.
- Separate plans: remove dependencies only; remove and restore from lockfile; verify/prune manager cache; clear global cache as explicit deeper cleanup; restoration is a separate stage (network, lifecycle scripts).
- Group workspace children under controlling root; collapse overlapping targets; independent roots parallel, shared workspaces and caches ordered.
- Commands: `npm ci`, `npm cache verify`, `pnpm install --frozen-lockfile`, `pnpm store prune`, `yarn install --immutable`, `yarn cache clean`, `bun install --frozen-lockfile`, `bun pm cache rm`.

### Stack priority contract (tech-stack §12, CONCEPT §8.0)
- Learn presence and frequency of these tools per path, project, child, parent, host; recommend without requiring command recall.
- Give this stack deeper context, richer plans, better insights, more realistic journeys than generic integrations.

## H. Remote hosts

- Keep hostname, environment role, current path, remote state, and privilege visible and unmistakable (CONCEPT §5.5 #2, §8.12).
- Prioritize service health, logs, system pressure, disk state, installed monitoring tools on remote hosts (CONCEPT §12 remote server).
- Use platform-appropriate actions; apply stronger confirmation and explicit target review for production or sensitive hosts (CONCEPT §8.12, §10 privileged level).
- Keep local and remote sessions one mental model yet unmistakably distinct (CONCEPT §15, §17).
- Bind remote Gate 2 phrases to the host, e.g. `RESTART PAYMENTS ON prod-eu-1` (CONCEPT §10).
- Generalize disk/cleanup for Linux, remote hosts, mounted filesystems (mole snapshot note, mole extensions).

## I. Boundaries (CONCEPT §14, GOAL §7)

- Do not become an exhaustive command encyclopedia, a shell replacement, a default AI command generator, an autonomous cleanup daemon, a replacement for every specialist TUI, a global menu that only shows the path, an unbounded filesystem crawler, a hidden automation engine, a macOS-only mental model, a product needing configuration first, a system that silently expands scope, or a launcher whose learned ranking defeats muscle memory.
- Allow AI only for explanation or intent classification with provenance, preview, scope, and user control intact; deterministic local behavior is default.
- Coordinate specialists (`btm`, `pg_activity`, Git TUI, file browser) rather than recreating them (CONCEPT §8.13, principles "Coordinate specialists").
- In this repo: never spawn a real stack command; every external system is a deterministic in-memory simulation (GOAL §1).
- Be useful without setup; configuration enhances discovery, never unlocks it (CONCEPT §11).

## J. Quality bar

- Users always understand: path/host/environment, active project or workspace, selected resource or action, result type and scope, keyboard focus, discovery still running, why a recommendation appears, what primary action does, what will change, confirmation or trust required, whether work succeeded/failed/active (CONCEPT §16).
- Prefer progressive disclosure, clear hierarchy, stable selection, small result sets (CONCEPT §16, principles "Prefer progressive disclosure").
- Success: opening without typing reveals a plausible next action; common actions take few predictable keystrokes; scope obvious; workflows discoverable without reading config; repeated workflows become explicit shortcuts; global does not crowd local; destructive is understandable and deliberate; local and remote share one model; specialists feel integrated; users invoke habitually (CONCEPT §17).
- Optimize for surfacing the right action with less cognitive effort, not command count (CONCEPT §17).
- Reject a generic fuzzy-finder with a green theme; plans read as graphs not scripts; activities survive navigation (GOAL §5).
- Demonstrate empty, loading, partial, success, failure, and dangerous states in every scenario (CONCEPT §19, GOAL §4).
- Keep keyboard first-class without secret shortcuts (CONCEPT §13 #15, §19).

## K. Required scenarios and journeys (GOAL §1, §4)

- Scenarios: `first-use`, `rust-dirty`, `monorepo-root`, `monorepo-child`, `docker-cleanup`, `disk-cleanup`, `upgrade-plan`, `activities-multi`, `remote-host`, `launch-failure`, `hard-cases` (GOAL §4).
- Journeys (CONCEPT §12): Rust project; Disk usage; Project collection; Monorepo root and child tasks; Nested child with parent ecosystem; mise trust and child-task execution; Git across children incl. `checkout main` resolution and clone review; Follow logs from multiple containers; Diagnose host pressure with `btm` handoff; Diagnose PostgreSQL blocking with cancel-before-terminate; SSH connection; Clean Rust/Gradle/Node artifacts; Remote server; Repeated workflow alias; Docker cleanup two-gate; Upgrade everything on Debian.
- Fixture world must include: Rust workspace monorepo with mise tasks and children; git states dirty/behind/diverged/detached/worktrees/submodules; docker/compose state; disk candidates with facts; pg activity; ssh config; GitHub account/org/repo state; ranking memory (pins, aliases, per-path usage); activities (GOAL §4 P1).

## L. Fixture-worthy facts, phrases, states (verbatim from sources)

- Reasons: `branch is 3 commits behind`; `4 modified files`; `used 6 times in this project`; `defined by project task runner`; `service is unhealthy`; `12 GB generated artifacts`; `available on this host` (CONCEPT §9).
- Recommendation labels: **Run tests**; **Review 4 modified files**; **Analyze disk usage**; **Open system monitor**; **Start development ecosystem**; **Start required containers**; **Run workspace setup**; **Show system resources**; **Clean Docker completely**; **Stop all containers**; **Prune builder cache**; **Upgrade everything on this system**; **Delete everything inside this folder**; **Remove all build artifacts under `/work`** (CONCEPT §5.3, §6.4, §8.16, §12, mole).
- Gate 2 phrases: `DELETE EVERYTHING IN /work/scratch`; `REMOVE ALL DOCKER DATA ON devbox`; `RESTART PAYMENTS ON prod-eu-1`; prefix `I UNDERSTAND:` for broadest (CONCEPT §7, §10, workflow).
- Aliases: `du`, `gp`, `test`, `d` then `u` (CONCEPT §6.6).
- Queries: `logs`, `why disk full`, `tests`, `sync projects`, `service logs`, `open config`, `docker`, `containers`, `docker cleanup`, `docker clean`, `children`, `workspace tasks`, `parent setup`, `checkout main`, `checkout master`, `switch to default`, `switch to primary branch` (CONCEPT §5.1, §6.3, §8.5, §12, tech-stack §2).
- Paths: `monorepo/apps/frontend`, `apps/frontend`, `services/api`, `tools/worker`, `/work`, `/work/scratch`, `//projects/frontend:build` (CONCEPT §5.3, §12, tech-stack §1).
- Hosts: `devbox`, `prod-eu-1`; SSH aliases `production-db`, `bastion`, `app.internal` (CONCEPT §10, tech-stack §8).
- Containers/services: `web`, `db`, `api`, `worker`, `scheduler`, `payments` (tech-stack §3, CONCEPT §12).
- Disk plan example: `frontend/node_modules — 8.4 GB — inactive 31 days — selected`; `frontend/dist — 420 MB — active today — unselected`; `backend/target — 12.7 GB — inactive 18 days — selected`; `android/build — 3.1 GB — inactive 22 days — selected`; `android/.gradle — 900 MB — activity unknown — unselected` (mole).
- Git block reasons: dirty work; detached state; unresolved operation; missing branch; remote ambiguity; diverged (blocked for review) (tech-stack §2).
- Docker cleanup stage order for display: stop containers → remove containers → remove images → prune networks → prune system data → prune volumes → prune builder cache (CONCEPT §12 docker cleanup).
- Step states: ready, waiting, running, succeeded, failed, skipped, blocked, cancelled, excluded (CONCEPT §8.15).
- Activity states: running, waiting, succeeded, failed, stopped, detached (CONCEPT §8.14).
- Cleanup result classes: removed, trashed, skipped, failed, reclaimed; per-branch reclaimed, skipped, blocked, failed (mole, CONCEPT §12).
- Upgrade journey exclusions: one optional cleanup step and one mise tool excluded; one branch fails while independent branch finishes; verification reflects failure (CONCEPT §12).
- mise tool example versions: `node@26`, `python@3.13`; exclusion `--exclude go` (tech-stack §1).
