# HP10 — Homebrew service lifecycle

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model schemas, cache freshness, availability, limits, target identity and start/stop/restart results using fixture service data and in-memory cache state.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real brew services commands and durable service-cache I/O.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Homebrew service discovery, cache and start/stop/restart. Existing remote systemd resources are a different provider.

**Source evidence and mandatory scope:** [matrix HP10](../parity/holla-parity-matrix.md#hp10--homebrew-service-lifecycle) — `OP39`, `OP40`, `OP41`, `OP42`, `OP43`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose service resources on hosts with brew, including Linuxbrew. Support both array and services-array JSON forms, valid sorted unique names and all three verbs regardless initial status. Preserve 30-action/10-service bound visibly or allow explicit further discovery. Show cache age; refresh stale names before target-changing operations.

**Architecture / reusable components:** Versioned service-cache adapter preserves v1, 300-second TTL, tolerant corruption and atomic replacement. Bind action identity to service/host/verb and re-observe status after mutation. Reuse resource Picker/Props, alternatives and Activity.

**Required deterministic fixture:** `parity-brew-services` — both schemas, malformed rows, no brew, empty/failing command, fresh/stale/wrong-version cache, 11 services, started/stopped/error state and each verb.

**Acceptance / automated verification:** Assert cache hit avoids probe, expiry boundary/corruption refresh, cache write failure remains nonfatal, deterministic IDs/order/count, exact brew services argv and observed lifecycle outcome. Missing service after review must not retarget another row. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS/Linuxbrew service alternatives, stale-cache refresh, failed stop and successful restart.

**Dependencies:** HP01/HP14/HP17/HP23; F02/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-brew-services` (fixture fn `brew_services` in src/bin/holla/domain/parity.rs) · `hp10_brew_services_verbs_are_exact_capped_and_fail_with_brew_s_reason` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/catalog.rs: brew_json_accepts_both_schemas_and_rejects_malformed_rows`. · row proofs in src/bin/holla/app_tests_rows.rs: `op34_op41_compose_logs_and_brew_start_carry_the_legacy_argv`.

**What the journey asserts:**
- Exactly 30 `brew.service.` action ids exist for the 11 fixture services (the two malformed rows `{"status":"none"}` and `{"name":""}` produce none).
- No action id contains `stale-one` ("the stale cache name is never an action").
- The `brew.services` item label contains `30 of 33`.
- `Restart svc01` has argv `brew services restart svc01 @/Users/alex/work`, settles `Succeeded`, and `svc01` status is `started`.
- `Stop svc02` settles `Failed`, output contains `Bootstrap failed: 5: Input/output error`, and `svc02` status stays `error`.
- The `Homebrew services` page text contains `fails on this host` (the cache write failure) and `svc10`.
- Unit (catalog.rs): top-level array `[redis, " ", {status}, postgresql@17, redis]` parses to `["postgresql@17","redis"]`; `{"services":[b,a]}` parses to `["a","b"]`; `{}` and `nope` are errors; `[]` is empty.

**Captures:** `shots/h_hp10_services` (the services page: source, cache age line, 11 services with three verbs, cache-write failure notice) · `shots/h_hp10_stop_failed` (activity: `brew services stop svc02` failed, exit 1, brew's bootstrap error) · base matrix `shots/h_parity_brew_services_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP39 | Unit: both schemas accepted, blank and missing names dropped, dedup and sort, `{}`/`nope` rejected; journey: 30 actions from the fixture's `services` array with two malformed rows. | Real `brew` executable and `brew services list --json`. |
| OP40 | Partly: journey proves the stale v1 cache (`fetched_at` 400 s old) is refreshed so `stale-one` never becomes an action, and the failed cache write is stated (`fails on this host`); frame `h_hp10_services`. Not asserted: cache hit avoids the probe, the inclusive 300 s boundary, wrong-version and corrupt cache refresh. | Durable service-cache I/O (atomic replace, TTL, corruption). |
| OP41 | `op34_op41_compose_logs_and_brew_start_carry_the_legacy_argv`: `brew.service.svc01.start` argv is `brew services start svc01`; `hp10_…` asserts the 30 offered verbs. | Real `brew services start`. |
| OP42 | Journey: `Stop svc02` fails with `Bootstrap failed: 5: Input/output error` and status stays `error`; frame `h_hp10_stop_failed`. | Real `brew services stop`. |
| OP43 | Journey: argv `brew services restart svc01 @/Users/alex/work`, `Succeeded`, status `started`; 30 actions of 33 with label `30 of 33`; frame `h_hp10_services`. Linuxbrew is not asserted (fixture `linux: false`). | Real `brew services restart`; Linuxbrew host. |

**Deferred remainder:**
- Real brew services commands and durable service-cache I/O (Later).
- Not proven by tests: cache hit avoiding a probe, the 300 s expiry boundary, wrong-version/corrupt cache refresh, no-brew host, empty or failing list command, a missing service after review not retargeting another row, and a Linuxbrew host.
- No capture of a Linuxbrew alternative, a stale-cache refresh transition, or a successful restart.

**Limits:**
- Service status and brew errors are fixture data; no launchctl, systemd or brew process is involved.
- Frames are being regenerated; their text is indicative until the integrator records the review.
