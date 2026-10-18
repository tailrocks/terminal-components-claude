# HP09 — Docker and Compose outcomes

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Complete each standalone and compound action against the simulated daemon/project inventory. Prove dependency barriers, drift gates, exact IDs/argv and distinct no-op/failure/success effects.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Docker daemon access, real Compose/log streams and destructive container/image/volume operations.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Compose up/down/finite logs, container stop-and-remove, full cleanup and builder prune. Full cleanup graph alone does not cover standalone variants.

**Source evidence and mandatory scope:** [matrix HP09](../parity/holla-parity-matrix.md#hp09--docker-and-compose-outcomes) — `OP31`, `OP32`, `OP33`, `OP34`, `OP35`, `OP36`, `OP37`, `OP38`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Separate Compose project from host container resources. Include all four manifest names, docker executable/daemon/plugin states and finite logs --tail 200 independently of follow logs. Legacy docker.stop-all stops and removes captured running/stopped IDs: offer an explicit Stop and remove all plan retaining that outcome, alongside new stop-only/remove-only alternatives. Full cleanup includes all images, network/system/volume prune; builder prune remains independently callable. Explain named-volume and buildx expansion separately.

**Architecture / reusable components:** Resolve immutable Docker identities into typed argv and dependency barriers. Remove shell substitution, swallowed image errors and preview/execution drift; model effects for every standalone variant. Do not run remove after failed stop or use force implicitly. Use existing resource snapshots, plans, facts and activities.

**Required deterministic fixture:** `parity-docker` — empty daemon, stopped+running containers, daemon/query failure, IDs changed after review, failed stop/remove/image stage, finite versus follow logs, Compose down without volumes, standalone builder prune, complete cleanup.

**Acceptance / automated verification:** Assert exact target classes/argv and scoped counts, successful no-op versus discovery failure, per-stage effects and mixed results. Compare every standalone action with its own world mutation. Require reauthorization after target drift, truthful permanent deletion, failure visibility and dependency blocking while independent prune branches remain independent. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture each standalone variant review/result, finite log completion, daemon failure, stop failure barrier and full cleanup final inventory.

**Dependencies:** HP01/HP14/HP15/HP17/HP21/HP22/HP23; F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-docker` (fixture fn `docker` in src/bin/holla/domain/parity.rs; the daemon-down and empty-daemon legs use `parity-discovery` and `first-use`) · `hp09_docker_and_compose_outcomes_land_in_the_world_and_fail_where_the_daemon_does` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/outcomes.rs: docker_and_cargo_outcomes_depend_on_state`, `src/bin/holla/domain/catalog.rs: docker_cleanup_is_first_for_its_query_and_two_gated`. · row proofs in src/bin/holla/app_tests_rows.rs: `op34_op41_compose_logs_and_brew_start_carry_the_legacy_argv`.

**What the journey asserts:**
- Five containers run at start; `Restart acme-db-1` has argv `docker restart acme-db-1 @/Users/alex/work/stack`, settles `Succeeded`, and `acme-db-1` is running with health `healthy` when a health value is present.
- `Stop acme-redis-1` has argv `docker stop acme-redis-1 @/Users/alex/work/stack`, settles `Failed`, and `acme-redis-1` is still running.
- `Stop all containers` has argv `docker stop acme-db-1 acme-redis-1 acme-api-1 acme-worker-1 acme-scheduler-1 @/Users/alex/work/stack`, settles `Failed`, output contains `cannot stop container`, and at least 4 containers stay running.
- `Stop and remove all containers` shows `gate 1 of 2`, then `Type REMOVE ALL CONTAINERS ON mbp to confirm`; the test cancels with Esc.
- `Stop the Compose project` has argv `docker compose down @/Users/alex/work/stack`, settles `Succeeded`, no container keeps project `acme`, and the unmanaged `pgadmin` stays.
- `Start the Compose project` has argv `docker compose up -d @/Users/alex/work/stack`, settles `Succeeded`, and the `acme` containers are exactly `["acme-db-1","acme-api-1","acme-worker-1"]`, all running.
- With the daemon down (`parity-discovery`): `docker.stop_all` has argv `docker ps -q @/Users/alex/work/probe`, freshness `Freshness::Unavailable` containing `Cannot connect`, running it starts nothing (tab stays `Here`) and the page shows `unavailable · Cannot connect to the Docker daemon`.
- With a live empty daemon (`first-use`, guarded by `docker.daemon.is_ok()`): `Stop all containers` settles `Succeeded` with output containing `0 containers`.
- Unit (outcomes.rs): `docker stop` exits 0, exits 1 with `fail_stage = "stop"`, and prints `Cannot connect` when the daemon is down. Unit (catalog.rs): `docker.cleanup` phrase `I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox`, commands include `docker system prune --force`; `docker.remove_all` is two commands ("stop then rm") with no `--force`; `Stop all containers` and `Prune builder cache` are listed for the query `docker clean`.

**Captures:** `shots/h_hp09_stop_all_failed` (activity: stop-all failed, exit 1, daemon error in output) · `shots/h_hp09_remove_gate1` (review page for stop and remove all: two-command sequence, `rm runs only after stop succeeds`) · `shots/h_hp09_remove_gate2` (the typed-phrase dialog `REMOVE ALL CONTAINERS ON mbp`) · base matrix `shots/h_parity_docker_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP31 | Fixture sets `compose.yaml` under `/Users/alex/work/stack`; the journey drives `compose.up`/`compose.down` from it. The four manifest names and the docker-executable gate are not asserted by a test. | Docker executable and real manifest probes. |
| OP32 | Journey: argv `docker compose up -d @/Users/alex/work/stack`, `Succeeded`, `acme` containers exactly `["acme-db-1","acme-api-1","acme-worker-1"]` all running. | Real Compose execution. |
| OP33 | Journey: argv `docker compose down @/Users/alex/work/stack`, `Succeeded`, no `acme` container remains, `pgadmin` stays (no volumes flag in argv). | Real Compose execution. |
| OP34 | `op34_op41_compose_logs_and_brew_start_carry_the_legacy_argv`: `compose.logs` argv is `docker compose logs --tail 200` in the project directory (a finite snapshot; the follow variant is a separate item). | Real finite log stream. |
| OP35 | Partly: journey proves the daemon-down state is `Unavailable` with `Cannot connect`, that the capture argv is `docker ps -q`, and that nothing starts; the `docker system df` preview injection is not asserted. | Real `docker system df` and daemon probes. |
| OP36 | Journey: stop-all argv over the five captured IDs, `Failed` with `cannot stop container`, containers stay running; remove-all `gate 1 of 2` then `Type REMOVE ALL CONTAINERS ON mbp to confirm`; empty daemon gives `Succeeded` with `0 containers`; unit: two commands, no `--force`; frames `h_hp09_stop_all_failed`, `h_hp09_remove_gate1`, `h_hp09_remove_gate2`. Remove after a successful stop is not run. | Destructive container removal against a real daemon. |
| OP37 | Partly: unit asserts `docker.cleanup` phrase and `docker system prune --force`; the plan is built by `plan_docker_cleanup` in the fixture. The journey does not run the full cleanup, so per-stage effects and the final inventory are not asserted. | Real image/network/volume/system prune. |
| OP38 | Partly: `Prune builder cache` is listed for `docker clean` (catalog unit test) and `docker.builder_prune` runs `docker builder prune -f` in catalog.rs; no test runs it or checks its effect. | Real builder prune. |

**Deferred remainder:**
- Docker daemon access, real Compose/log streams and destructive container/image/volume operations (Later).
- Not proven by tests: finite logs (OP34), standalone builder prune effect (OP38), full cleanup per-stage effects and final inventory (OP37), IDs changed after review (drift reauthorization), a failed remove or image stage, remove after a successful stop, Compose down with volumes wording, and `docker system df` accounting.
- No capture of finite log completion, daemon failure, or the full cleanup final inventory.

**Limits:**
- All daemon behaviour is the simulated `DockerState`; no socket, plugin or process is touched.
- Frames are being regenerated; their text is indicative until the integrator records the review.
