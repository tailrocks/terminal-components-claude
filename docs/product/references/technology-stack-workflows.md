# Technology-stack workflows

This reference defines Holla's priority technology concepts and realistic
command examples. It does not prescribe visual design, layout, components, or
exact interaction mechanics.

Commands are examples of intended capability. Before execution, Holla should
resolve behavior from installed tool versions, effective configuration, current
scope, and live targets.

## 1. mise

### Required understanding

Holla should explain:

- effective configuration from current folder through ancestors;
- monorepository roots and configured child roots;
- local versus global tool ownership;
- active, installed, missing, and outdated tools;
- task name, definition file, working directory, dependencies, and environment;
- trust state and the scope affected by trusting a configuration.

There is no need to present mise as one opaque command. Compose a useful status
from its configuration, tool, task, outdated, diagnostic, and trust information.

### Common actions

```sh
mise use node@26 python@3.13
mise use --global node@26
mise install --dry-run
mise install --include-task-tools --dry-run
mise install --monorepo --include-task-tools --dry-run
mise outdated --json
mise outdated --local --json
mise upgrade --dry-run
mise upgrade --interactive
mise upgrade --exclude go
mise tasks ls --json
mise tasks ls --all --json
mise tasks info //projects/frontend:build --json
mise tasks deps --dot //projects/frontend:build
mise run --dry-run //projects/frontend:build
mise run //projects/frontend:build
mise trust --show
mise trust /exact/path/mise.toml
mise trust --untrust /exact/path/mise.toml
```

### Hierarchy

Current and ancestor tasks rank first. Whole-monorepository discovery remains one
scope change away. Child tasks may use namespaced identities such as
`//projects/frontend:build`; the action must retain its child working directory.

Task dependency information should contribute directly to Holla's execution
graph. Independent tasks may run concurrently. Declared dependencies, post
dependencies, and wait relationships remain visible.

### Trust

Before task execution or tool installation, show exact configuration paths,
task definitions, commands, environment impact, and trust scope. Do not silently
rely on automatic trust. Broad trust covering current, parent, and descendant
configuration needs separate scope review. Changed content may require renewed
trust.

### Upgrade plan

Show current, requested, and latest versions; distinguish within-range upgrades
from version-range changes; show lockfile and configuration effects; allow tool
exclusions; then execute independent installs with retained output.

## 2. Git and GitHub

### Discovery

Current Git context ranks first. An explicit child scope discovers `.git`
directories and `.git` files, stops traversal inside Git metadata, resolves real
top-level paths, deduplicates worktrees, and distinguishes submodules from
independent projects.

For each project, retain:

- path and project type;
- branch or detached state;
- upstream and ahead/behind state;
- staged, unstaged, untracked, and conflicting changes;
- stash count;
- remotes and primary branch;
- action eligibility and block reason.

### Common actions

```sh
git -C <project> status --porcelain=v2 --branch --show-stash
git -C <project> pull --ff-only
git -C <project> push --dry-run
git -C <project> push
git -C <project> switch <primary-branch>
gh auth status --active
gh org list --limit 100
gh repo list <owner> --limit 100 --no-archived
gh repo clone OWNER/REPOSITORY [directory]
```

### Bulk plans

Support status, pull, push, and primary-branch switching here or across selected
children. Show a selectable project hierarchy and retain output per project.
Independent projects may run with bounded parallelism; one failure does not hide
or automatically cancel unrelated results.

Pull should use an explicit integration policy. A conservative default accepts
fast-forward-only updates and blocks divergence for separate review. Never
auto-reset or auto-stash.

Do not hard-code `main` or `master`. Resolve the project's actual primary branch.
Dirty work, detached state, unresolved operations, missing branches, and remote
ambiguity should block automatic switching rather than discard changes.

Treat queries such as `checkout main`, `checkout master`, `switch to default`,
and `switch to primary branch` as intent to resolve and switch to each selected
project's actual primary branch. The query is a familiar phrase, not permission
to hard-code that branch name.

Generic push-all excludes force pushes, deletion, mirror behavior, and tag
overwrites. Force-with-lease, when explicitly requested, is a separate dangerous
workflow with fresh remote-state review and target-bound confirmation.

### GitHub cloning

Use the active GitHub account and its organizations as discovery scopes. Let the
user search personal or organization projects, then show account, host,
visibility, protocol, owner, primary branch, destination, and fork/upstream
behavior before cloning. Never reveal authentication tokens. Never overwrite an
existing destination containing user data.

## 3. Docker

### Inspection and common actions

```sh
docker ps
docker ps -a
docker ps --size
docker system df
docker system df -v
docker stats
docker stats --no-stream
docker stop web db
docker rm web db
docker logs --tail 200 api
docker logs -f --since 10m api
docker compose ps --all
docker compose logs -f --tail 200 api worker
docker compose stop
docker compose down
docker image prune -a
docker network prune
docker volume prune -a
docker builder prune -a
docker buildx prune -a
```

Show container name, image, state, ports, project ownership, writable-layer size,
CPU, memory, network I/O, block I/O, and process count when available.

One-container Docker logs and multi-service Compose logs differ. For arbitrary
multiple containers, Holla should orchestrate one stream per container, prefix
output clearly, and retain each stream as an activity.

### Scope

If current or ancestor context contains a Compose definition, expose project
scope separately from host-wide Docker scope. Preserve the exact Compose file,
project directory, and project name so actions never target a different parent
configuration accidentally.

### Stop and remove all

Resolve and freeze exact container identities before review. Avoid shell
substitution as the product contract.

Plan:

1. discover running and stopped containers;
2. review names, images, state, project, and host;
3. stop selected running containers gracefully;
4. remove selected stopped containers;
5. verify remaining containers.

Force removal is a distinct harsher action, not an automatic fallback.

### Complete cleanup

“Complete Docker cleanup” means all user-removable Docker state, not daemon
uninstallation or factory reset.

Plan:

1. inventory containers, images, custom networks, named and anonymous volumes,
   builders, caches, and reclaimable space;
2. freeze and review targets;
3. stop running containers;
4. remove containers;
5. after the container barrier, remove unused images, networks, and volumes in
   independent branches;
6. clean builder and selected Buildx caches;
7. rescan and report reclaimed and remaining state.

Named volumes require explicit strongest treatment because they commonly contain
durable data. A generic system prune does not remove every named volume. Show
exact resource classes rather than promising an inaccurate single-command reset.

## 4. `btm` and system resources

Holla should offer quick system snapshots and a persistent `btm` handoff.

Key insights:

- total and per-core CPU;
- load and CPU history;
- used and available memory;
- swap use and pressure;
- filesystem capacity;
- per-device read/write throughput and I/O pressure;
- per-interface network receive/transmit rates and totals;
- process search, sorting, hierarchy, CPU, memory, and state;
- temperatures and battery where supported;
- Linux CPU, memory, and I/O pressure, distinguishing partial pressure from
  system-wide thrashing.

Common intents:

- Show system resources.
- What uses CPU?
- What uses memory?
- Show process tree.
- What causes disk I/O?
- Show network activity.
- Open `btm`.
- Find and stop a process.

```sh
btm
```

Holla should not recreate all deep monitoring. Open `btm` as a persistent
activity while preserving host and launch context. Process signaling must show
PID, identity, hierarchy, signal, and likely effect before mutation.

## 5. Disk usage and cleanup

Core intents:

- Why is this disk full?
- Analyze this directory.
- Find largest files and folders.
- Show reclaimable space.
- Clean current project.
- Clean child projects.
- Remove Rust targets, Gradle outputs, or Node dependencies.
- Review system cleanup.
- Show cleanup history.

Use progressive discovery, hierarchy, plan editing, freshness-aware defaults,
protected paths, target revalidation, clear recovery mode, and auditable results.
See `mole-disk-cleanup-patterns.md` for detailed concepts.

## 6. PostgreSQL and `pg_activity`

### Discovery

Recognize standard PostgreSQL connection context from environment, service
definitions, local sockets, project settings, and containers. Never place a
password in displayed commands, arguments, history, or persisted plans.

### Critical insights

- connections by state and saturation versus configured maximum;
- active query, transaction, and backend age;
- idle-in-transaction sessions;
- wait event and blocking dependency hierarchy;
- commit, rollback, read, hit, temporary-file, deadlock, and I/O trends;
- query calls, total and mean latency, rows, physical reads, temporary spill,
  and WAL generation when statement statistics exist;
- table live/dead estimates, analyze/vacuum history, and maintenance progress;
- replication state, WAL backlog, slots, and write/flush/replay lag.

Do not collapse workload into one universal “worst query” score. Rank total load,
mean latency, frequency, reads, spill, and WAL independently. Label cumulative
counters and replication lag accurately.

### Common actions

```sh
pg_activity
pg_activity -h HOST -p PORT -U USER -d DATABASE
pg_activity --duration-mode 2 --min-duration 5
```

Offer `pg_activity` for rich real-time inspection. Native Holla insights should
identify anomalies and explain why this handoff is relevant.

Cancel query before offering backend termination. Before either, re-read server,
database, PID, user, application, query identity, duration, and blockers. A PID
or query change invalidates confirmation.

## 7. Rust and nextest

### Project understanding

Use Cargo workspace metadata to group members, targets, workspace root, and
actual target directory. Do not treat every descendant manifest as an independent
project or assume output always lives at `<project>/target`.

### Common actions

```sh
cargo check
cargo check --workspace
cargo build
cargo build --release
cargo run
cargo run -p api
cargo run --bin server -- --port 8080
cargo test
cargo test parser
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo nextest run
cargo nextest run -p api
cargo nextest run --workspace
cargo nextest run --profile ci
```

Prefer nextest when installed or configured while keeping ordinary Cargo tests.
Retain per-test state, duration, retries, flaky status, failure output, selected
profile, and final summary.

Offer package, workspace, target, feature, profile, toolchain, locked, offline,
and program-argument choices only when relevant. Keep common actions simple.

## 8. SSH

### Destination discovery

Recursively inspect `$HOME/.ssh/config` and its includes. Enumerate literal
positive host aliases. Wildcard, catch-all, and negated entries are matching
rules, not finite destination lists.

For a selected alias, use installed OpenSSH to resolve effective behavior. Static
parsing finds candidates but cannot correctly reproduce all host, match,
canonicalization, proxy, identity, and system-default behavior.

### Connection preview

Explain:

- entered alias and resolved `user@host:port`;
- jump chain or arbitrary proxy command;
- selected identity filenames without key content;
- agent and identities-only behavior;
- requested terminal or remote command;
- local, remote, and dynamic forwarding;
- host-key policy;
- existing connection-multiplexing state.

Run the configured alias unchanged whenever possible:

```sh
ssh production-db
ssh -J bastion app.internal
```

Rebuilding a simplified destination command may discard important configuration.
Preserve native password, passphrase, and new-host fingerprint prompts. Never
disable host-key verification. A changed host key must hard-stop.

Respect existing connection multiplexing. Expose connection status and explicit
stop or exit actions without silently enabling or destroying shared masters.

## 9. Rust artifact cleanup

Plan:

1. discover independent Cargo workspaces;
2. resolve actual target directories;
3. deduplicate shared targets;
4. measure each target and total reclaimable size;
5. let the user exclude workspaces or choose release, documentation, or package
   cleanup;
6. preview through Cargo where available;
7. confirm exact targets;
8. clean independent target directories in parallel while serializing shared
   targets;
9. report reclaimed space and failures.

```sh
cargo clean --dry-run --verbose
cargo clean
cargo clean --release
cargo clean --doc
cargo clean -p api
```

Never delete every folder named `target` blindly.

## 10. Gradle cleanup

Discover roots through settings files and wrappers. Prefer the project wrapper
and Gradle's `clean` tasks so custom build directories are respected.

Treat one multi-project build as one hierarchy. Treat independent nested builds
as separate branches. Show affected projects, task paths, size, and rebuild cost.

Separate ordinary build cleanup from deeper removal of project `.gradle` data,
user caches, wrapper distributions, and daemon logs. Never confuse generated
`.gradle` with checked-in `gradle` content. Inspect or stop relevant daemons
before deleting active state.

```sh
./gradlew clean
./gradlew :service-a:clean
./gradlew --stop
```

## 11. Node dependency cleanup

Identify package manager, lockfile, workspace ownership, installation mode,
manager configuration, symlinks, and checked-in caches before removal.

Do not assume Yarn uses `node_modules`. Do not traverse nested dependency trees
or follow a `node_modules` symlink outside its resolved location.

Offer separate plans for:

- remove dependencies only;
- remove and restore exactly from lockfile;
- verify or prune package-manager cache;
- clear global cache as an explicit deeper cleanup.

Restoration is a separate stage because it may use network access and run
lifecycle scripts.

Common locked restoration and cache actions:

```sh
npm ci
npm cache verify
pnpm install --frozen-lockfile
pnpm store prune
yarn install --immutable
yarn cache clean
bun install --frozen-lockfile
bun pm cache rm
```

Group workspace children beneath their controlling root. Collapse overlapping
targets. Independent package roots may clean in parallel; shared workspaces and
caches require ordered execution.

## 12. Stack priority contract

Holla should learn which of these tools are present and frequently used per
current path, project, child project, parent, and host. It should recommend
relevant actions without requiring users to remember exact commands.

This stack is not an optional demonstration set. It defines the primary product
value and should receive deeper context understanding, richer plans, better
insights, and more realistic journeys than generic integrations.
