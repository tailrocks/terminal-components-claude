# Refactoring plan and task contracts

This plan preserves the task graph, acceptance machinery, and implementation requirements for the refactor. It is a live planning specification, not a progress diary. Pin values and pass counts below are included only where they remain required task inputs; verify all current refs and evidence before execution.

## 1. Goal and readiness boundary

Complete the reusable component and app-workspace refactor while preserving the exact design/interaction-polish product behavior. The design branch is the product reference; main provides the structural direction. A workspace build or self-generated baseline does not establish parity.

Execution is gated by the task graph and proof contract. Do not treat any historical completion statement, report, or task package status field as a current acceptance receipt. The production refactor can begin only after the required oracle, inventory, test-disposition, runner, accounting, and architecture evidence is independently accepted. Current readiness must be established from the exact candidate revision.

## 2. Authorities and identity handling

- Product behavior and appearance: [design authority](../design.md), the design/interaction-polish source, and approved immutable captures.
- Structural architecture: [architecture.md](../architecture.md) and the detailed component contracts in [shared-components-plan.md](../architecture/shared-components-plan.md).
- Work allocation and acceptance: the task graph, traceability inputs, and sealed task package files.
- Current implementation facts: source and tests on the exact candidate revision.

Historical oracle hashes, branch tips, tool revisions, and commit counts in earlier planning records are provenance, not moving aliases. Resolve current required identities from protected task inputs before execution; do not repin expected evidence from candidate output.

## 3. Historical project context

Earlier main-era goals described an in-place component refactor and worktree-consolidation campaign. They explain how the workspace direction arose, but their slice order, completion claims, model routing, branch status, and self-baseline results do not define current readiness. The detailed historical source records remain under `docs/sources/` where task inputs require them. The accepted behavior target and current package boundaries are stated in the maintained architecture and design documents.

## 4. Current execution waves (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §15)

| Wave | Tasks | Exit condition |
| --- | --- | --- |
| Qualified proof | 001, 070–072 | Independently accepted comparator, host, runner, accounting and architecture products |
| Immutable preparation | 002–008 | Four oracle namespaces, complete component baseline, exact inventory and approved test dispositions sealed |
| Attribution foundation | 073 | Truthful resolution/registry observations; no premature whole-family closure |
| Reusable system | 009–031 | Shared runtime/theme/layout/text/components restored and generated conformance complete |
| Applications | 032–064 | Four independently staged application chains, each ending in complete oracle closure |
| System closure | 065–069 | Ownership, exact test relocation, real performance, public API/docs and merge readiness |

Dependency-defined waves; exact earliest layers are in the derived graph. Task numbers are identifiers, not execution order — TASK-070–073 are early foundations despite their appended numbers, and TASK-031 closes complete conformance only after the reusable repairs it checks (removing circular acceptance).

## 5. Dependency DAG facts (`task-graph.md` / `task-graph.json`)

73 tasks; maximum dependency depth 35; 24 equally deepest paths. One exact longest path: `TASK-001 → 070 → 071 → 072 → 002 → 006 → 008 → 073 → 009 → 010 → 014 → 015 → 018 → 019 → 023 → 024 → 027 → 028 → 030 → 031 → 040 → 041 → 042 → 043 → 044 → 045 → 046 → 047 → 048 → 049 → 050 → 065 → 066 → 068 → 069` — proof qualification, complete oracle preparation, attribution, reusable runtime/scroll/field/layer/editor work, the Holla chain and final closure drive the critical path. This is an unweighted structural result, not a duration estimate.

Parallelization: independent oracle app capture/inventory after shared contracts exist; disjoint reusable modules concurrently; four application chains independently after shared prerequisites. Hard edges serialize every measured source overlap (e.g. TASK-065 before TASK-066 for `xtask/src/main.rs`). Graph derivation fails on unsupported wildcard scope and records new unordered overlaps for explicit resolution. Separate worktrees/target dirs per worker; joins require integrated-tree re-verification, not just individually green siblings (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §18).

## 6. Task catalog structure (the execution queue)

**Location:** `refactoring-tasks/terminal-components/completion/` — 73 sealed task-format packages (`001/`–`073/`) plus the group `README.md`. Project `terminal-components`, group `completion`. Companion text sometimes says "74 packages"; the directory holds 73 task packages + 1 README = 74 entries, and `task-index.tsv` has 73 data rows — flagged, not an omission.

**Format:** task packages declare their schemas and dependencies in `task.toml` and `verify.toml`; these files, not Markdown ordering, define machine-readable scope. A status field or prior report is not acceptance. A usable predecessor requires the specified independent verification evidence and integrated ancestry.

**Package anatomy** (verified on `001/`):
- `README.md` — the contract: goal, context, preconditions, in/out of scope, MUST/MUST-NOT/non-regression requirements, typed acceptance blocks (constrained Gherkin per non-gate `AC-*`, `Type: gate` for gates), requirement→acceptance mapping, fixed decisions, checklist.
- `AGENTS.md` — the canonical execution protocol: `/task` read-only, coordination in `/progress/progress.md` (strict versioned event grammar), work in `/work`, completion gate `taskfmt verify` exiting 0 with final line `DONE`.
- `task.toml` — `task-meta/v1`: status and hard dependencies (the DAG source).
- `verify.toml` — machine authority: checks invoked as fixed `/proof/bin/tc-proof ...` argv with requirement/acceptance references and expected exit 0; writable-path scope.
- `trusted/` — protected payloads outside executor writable scope: `obligations.md`, `source-obligations.tsv` (joins exact historical source clauses to requirement/acceptance/check IDs); `001/trusted/proof-bootstrap/` additionally holds the planner-frozen comparator/host qualification drivers, vectors and protocols.

**Catalog-level rules** (`refactoring-tasks/README.md`, `completion/README.md`): the host freezes the catalog and trusted inputs outside candidate authority before execution; use the pinned **standalone** `taskfmt verify` (explicit root/task-dir/immutable base) — the automated dispatcher/promotion lifecycle hardcodes `main` and must not be used; no task authorizes a push, main merge, automatic tui-snap PR merge, oracle repinning or approval of changed product output; a final passing task establishes readiness only. **`refactoring-tasks/` is the execution queue**; dispatch only after the top-level plan's readiness status is accepted.

**Wave letters in `task-index.tsv`:** B = 12 (001–008, 070–073: harness, four oracle producers, component baseline, inventory, test disposition, runner, accounting, architecture, conformance foundation), C = 23 (009–031 reusable system), S = 8 (032–039 Showcase), H = 11 (040–050 Holla), J = 7 (051–057 Jackin), T = 7 (058–064 TablePro), X = 5 (065–069 closure, ending in TASK-069 X-FINAL merge readiness).

## 7. Proof contract mechanics (`docs/refactoring-plan/proof-contract.md`)

- **Trusted host layout** outside every executor checkout: `campaign/{bootstrap,catalog,harness,bundles,receipts,runs,ledger}`, SHA-256 content-addressed via canonical manifests; receipts bind product digest, producer task, accepted source tree, bootstrap/harness/catalog digests, oracle SHA, tool pins, dependency receipts, gate evidence and reviewer decision. The host owns receipt creation/verification; no `latest` file or environment override participates in resolution.
- **Command interface (future B-HARNESS deliverables, not existing commands):** `tc-proof preflight|required|oracle|capture|compare|account-tests|architecture|close`, each driven by an immutable per-check context (`contexts/CHK-NNN.json` bound by a frozen `context-index.json`); plus host-only `tc-proof-host install|prepare|freeze|verify|seal|integrate`. `seal` is baseline-producer-only; `integrate` updates only the named local integration ref with an expected-parent check — never pushes, never merges to main.
- **Isolation/freeze/verify/integrate:** untrusted executor, separate verification workers (no host Docker socket/credentials, pinned image/toolchain digests, no network); freeze reconstructs the candidate tree from the recorded parent plus allowed changed files in a fresh host-owned object database; verify runs the pinned standalone `taskfmt` and independent integrity postchecks; integration commits exactly the verified tree onto the recorded parent (`git update-ref REF NEW EXPECTED_PARENT`); parallel siblings require a fresh join tree and re-run union gates.
- **Oracle derivation and exact comparison:** the reference runner imports the oracle Git bundle and checks the resolved commit against the fixed oracle SHA; only reviewed observation/time/input-orchestration adapters; double capture with exact repeat equality; schema-3 frames validated (`Frame::validate`/`from_json`) before `diff_cells`, cursor compared alongside cells; exact semantic-object equality (focus/edit owner, caret/selection, selected identity, scroll bounds/offsets, overlay owner, effects, navigation outcome) with no masks/tolerances; PNGs via `Store::check(..., 1.0)?.ensure_matched()`; candidate output never populates `approved/`; `tuisnap accept` is never invoked.
- **Intermediate stage gates:** whole-workspace build/MSRV/fmt/clint/backend-free/architecture gates green; task-owned and all previously closed oracle scenarios pass (monotonic closed-set ledger); complete required test/scenario inventory executed as a no-fail-fast diagnostic sweep; accounting accepts only the exact future-owner identities admitted by the immutable stage map; each app closure empties that app's unresolved set; X-FINAL empties the global set.
- **Qualification machinery:** the comparator fixture pack (72 cases + 69 fresh positive recoveries) and the separate host freeze/isolation/ref-update suite qualify the harness black-box; TASK-071 implements host-selected `preparation`/`production` accounting modes; TASK-008's test migration is governed by an independently applied, byte-exact assertion-span patch manifest producing a frozen post-approved-test-migration expectation register.
- **Readiness blockers before any execution:** freeze bootstrap drivers/fixtures; resolve the final tui-snap PR/revision; freeze source-level required IDs, finite axes and source-only expansion algorithms; map every protocol operation to its bounded producer; materialize and independently validate every flat numeric trace and oracle namespace; accept exact test dispositions/stage maps; seal the complete baseline receipt.

## 8. Verification architecture

Each required lane must compare candidate behavior to its independently selected oracle: direct production-view capture and real executable/PTY interaction where the contract requires it. Compare complete rendered cells and cursor state together with semantic observations such as focus, target identity, editing, selection, scroll position, overlays, effects, and navigation outcome. Include required no-op checkpoints so missing transitions cannot pass by producing no evidence.

Only explicitly qualified reference adapters may control observation or time. Expected output, scenario membership, actions, and source identities must be protected from candidate authority. Visual diffs aid review; machine acceptance follows the exact comparator and immutable evidence rules in [proof-contract.md](../refactoring-plan/proof-contract.md). Historical tool test counts and PR identities are not current verification evidence.

## 9. Integration strategy

Before execution, select and freeze the structural starting revision and all required source, tool, and oracle identities in protected task inputs. Build an isolated integration line from the accepted structural workspace. Preserve its reusable APIs and safety contracts while adapting the design branch's application composition, deterministic domain fixtures, and behavior through those APIs. Do not merge both branch histories wholesale, revive a superseded API, or cherry-pick old ports without qualification.

Use the task-format's accepted isolated verification workflow. Parallel work is not accepted until integrated on a common tree and the union of required checks is rerun. Publication and merge remain separate actions requiring the applicable authorization and final-tree verification.

## 10. Evidence matrices and traceability mechanics

- `task-index.tsv` — 73 rows: `task_id, key, title, dependencies, wave, writable_paths, purpose`; the canonical allocation of outcomes to tasks.
- `traceability.tsv` — ~3,236 data rows: `source_namespace, source_id, task_id, requirement_id, acceptance_id, check_id, role, disposition`; joins historical clauses, architecture contracts, component families and app scenarios to tasks with roles such as `baseline`, `frame-contribution` and `primary`, enabling requirement→task, task→requirement, component→task, scenario→verification and decision→implementation queries. Rejected/deferred decisions remain as explicit forbidden-path/scope dispositions; namespaced references distinguish `HIST:A01` from `ARCH:A01`.
- Supporting matrices: `historical-obligations-canonical.tsv` (620 retained clauses across nine ledgers), `architecture-matrix.tsv` (32 cross-cutting contracts), `component-parity.tsv` (54 families covering all 31 oracle widget modules and all 37 main component modules), `application-parity.tsv` (361 scenario contracts: 76 Showcase, 135 Holla, 70 Jackin, 80 TablePro) with per-app inventories and scenario TSVs, `decision-ledger.tsv` (ADJ bindings to implementation owners), `history-revision-index.tsv` (281 parent-relative document edges). Shell/app-flow contribution tables bind partial checkpoint ownership without ever closing a parent scenario early.

## 11. Completion risks and open gates

- Required source, task, test, scenario, and architecture inventories must be complete and independently checked.
- Scenario expansion must be deterministic and oracle-derived; broad labels such as “all states” do not enumerate acceptance coverage.
- Every app and component closure must retain exact source-qualified outcomes and protected expected evidence.
- Main's self-generated baselines can differ from the product oracle; conflicts require explicit dispositions and cannot be blessed from candidate output.
- Time-control proofs and real PTY proofs answer different questions; wall-clock measurements cannot prove deadline boundaries.
- Dynamic identity, capture, query attribution, callbacks, and alternative paint paths need nonvacuous adversarial tests where the contract uses them.
- The task graph gives dependency order, not durations; no schedule estimate is implied.

## 12. Decisions that resolve historical conflicts

1. Product authority follows the design/interaction-polish contract and protected captures; older baseline namespaces remain historical inputs only when explicitly named by a task.
2. The structural refactor follows the workspace direction in the maintained architecture plan; this does not certify current product parity.
3. Preserve exact task IDs and dependencies, but derive execution order from the graph rather than task number or historical slice order.
4. Candidate-generated output never changes oracle evidence. Any intentional product change requires an explicit design decision and a separately qualified reference update.
