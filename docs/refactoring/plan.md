# Consolidated refactoring plan — as it stands on holla

**Provenance:** Consolidated on 2026-09-12 from `REFACTORING_COMPLETION_PLAN.md` and all of `docs/refactoring-plan/` (notably `proof-contract.md`, `task-index.tsv`, `traceability.tsv`, `task-graph.md`, `reaudit-plan.md`, `branch-continuation-proposal.md`, `verification.md`), `refactoring-tasks/README.md`, `refactoring-tasks/terminal-components/README.md`, `refactoring-tasks/terminal-components/completion/README.md` and a sampled task package (`completion/001/`) (holla working tree); historical phases from `main:GOAL.md`, `main:GOAL2.md`, `main:REFACTORING_GOAL.md` (via `git show`); planning requirements from `PLANNING_GOAL.md`. The governing goal statement is `docs/refactoring/goal.md`.

## 1. Current status (read first)

**Completion is reopened; execution readiness is NOT currently accepted.** The 73-task catalog passed its recorded checks, but the source-first, line-by-line re-audit reopened on 2026-09-11 identified scope and contract gaps (`REFACTORING_COMPLETION_PLAN.md` header; `docs/refactoring-plan/reaudit-plan.md`). The previous completion audit and verification record are historical evidence, not acceptance. The 151-row frozen asset manifest (`bootstrap-assets.tsv`) is stale and must be resealed only after independent review. The production refactor and the production verification harness have **not** been implemented or executed.

Active boundary: no terminal-components production refactor, task execution, integration branch, commit, merge or publication is authorized. The only permitted external implementation is the tui-snap dependency, which is done but unmerged: PR #1 head `883d03f19d890bbbf27468798db78b04e85297ac`, a hard prerequisite of reference qualification.

The re-audit is organized as seven task partitions (001–008/070–072, 009–020, 021–031/073, 032–039, 040–050, 051–064, 065–069) plus cross-cutting history and a coordinator, each with exact coverage and independent recheck rules (`reaudit-plan.md`). Findings and dispositions live in `reaudit-findings.tsv`; material work remains open until repaired bytes, all source/task joins, executable qualifiers and the final manifest are independently checked. A parallel whole-branch review (`branch-continuation-proposal.md`, exact inventory 7,885 changed paths) maps required continuation areas to existing task owners — it routes changes into the existing task graph, not a competing rewrite.

## 2. Resolved identities (pins, measured 2026-09-11)

| Authority | Identity |
| --- | --- |
| Immutable UI oracle | `02f5294bfdbf38004cc49130d0aff1d01f31434c` (tag object `a643909d9a782adaf0aa1e3357710a5ed3f24443`) |
| Architectural main | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| Planning checkout Holla | `2e2401393c47360741ebd321679de08982dca50a` |
| Main/Holla merge base | `cc14dd6beae526884aabdf897e309be837b4f504` |
| task-format main + installed CLI | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |
| tui-snap reviewed PR #1 head | `883d03f19d890bbbf27468798db78b04e85297ac` (tree `dadbaa70facc317cfabb52f0374c1f3cdceb46a1`) |

Topology: main and holla have 774 and 38 commits outside the merge base respectively. Main is a virtual workspace (`junie-tui`, `junie-tui-testing`, `xtask`, four app packages) and already contains the prior integration merged as PR #1; the oracle retains the old root library and `src/bin` applications (`REFACTORING_COMPLETION_PLAN.md` §3–4).

## 3. Historical phases (main era — evidence, not dependency authority)

- **`main:REFACTORING_GOAL.md`:** the original in-place architectural refactor into a reusable, themeable component system (shadcn/ui-analogous qualities translated to idiomatic Rust/Ratatui), executed as numbered slices; it was interrupted around Slice 4 (`HANDOFF_SLICE4_WAVE1.md`). Produced the workspace and `junie-tui` foundations.
- **`main:GOAL.md` phases 0–5 (parity-first restoration, partially done):** Phase 0 safe preflight and freezing `baseline/before/**`; Phase 1 build the parity oracle (map 499 historical recipes to current replay, fail-closed comparator over `.txt`/`.ansi`/`.cursor` plus `.png`/`.html` review); Phase 2 restore shared visual contracts (tokens, glyphs, panels, focus/hit/cursor/scroll/layers); Phase 3 restore each application (Showcase shell + 22 pages, TablePro connection/workbench, Jackin route-by-route); Phase 4 tests and evidence (narrow parity tests, immutable archive, reviewed baseline updates); Phase 5 required verification (full workspace gates, real binaries, comparator, `bless-guard`).
- **`main:GOAL2.md` (completed):** consolidation of all worktrees into `codex/main-holla-integration` with an explicit per-worktree disposition ledger; merged as PR #1 at `7b27732a`.

The current plan uses historical Slice/Wave terminology as evidence only; the initial 69-outcome decomposition proposal is historical evidence, not dependency authority (`REFACTORING_COMPLETION_PLAN.md` §14).

## 4. Current execution waves (`REFACTORING_COMPLETION_PLAN.md` §15)

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

Parallelization: independent oracle app capture/inventory after shared contracts exist; disjoint reusable modules concurrently; four application chains independently after shared prerequisites. Hard edges serialize every measured source overlap (e.g. TASK-065 before TASK-066 for `xtask/src/main.rs`). Graph derivation fails on unsupported wildcard scope and records new unordered overlaps for explicit resolution. Separate worktrees/target dirs per worker; joins require integrated-tree re-verification, not just individually green siblings (`REFACTORING_COMPLETION_PLAN.md` §18).

## 6. Task catalog structure (the execution queue)

**Location:** `refactoring-tasks/terminal-components/completion/` — 73 sealed task-format packages (`001/`–`073/`) plus the group `README.md`. Project `terminal-components`, group `completion`. Companion text sometimes says "74 packages"; the directory holds 73 task packages + 1 README = 74 entries, and `task-index.tsv` has 73 data rows — flagged, not an omission.

**Format:** canonical task-format revision `52d9f1e`; schemas `task/v5` (contract README), `verify/v2` (`verify.toml`), `task-meta/v1` (`task.toml`). Dependencies live in each `task.toml`, never in Markdown ordering. All packages are `status = "pending"`; an accepted host receipt and integrated ancestry — not a mutable status field — establish a usable predecessor.

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

## 8. Verification architecture (`verification.md`, `REFACTORING_COMPLETION_PLAN.md` §11–12)

Two lanes, each compared against its own oracle lane: direct production-view frame capture, and real PTY executable interaction with unmodified CLI binaries. Compare complete canonical cells plus cursor, then semantic observations that distinguish identical-looking states (focus, target identity, editing, selection, scroll position, overlays, effects), including required no-op checkpoints. Reference adapters may translate observation and controlled time only (the isolated oracle clock adapter is the concrete qualified case: 76 existing Showcase/TablePro binary tests plus four deadline probes, original duration literals unchanged). PNG/HTML diffs aid human review; exact machine equality is the gate. The whole proof chain is protected: oracle SHA, adapters, scenario membership, numeric actions, expected frames/state, comparator, hashes, tool/profile/font and environment. Repaired tui-snap at `883d03f` passed 51 all-target/all-feature tests, 41 no-default-feature tests, doctest, fmt, Clippy `-D warnings`, downstream consumer, reproducible migration and a 384-cell PTY fixture, plus all 256 modifier combinations across serialization modes.

## 9. Integration strategy (`REFACTORING_COMPLETION_PLAN.md` §13)

At the start of future execution: create the isolated integration branch from pinned main `7b27732a` before proof/baseline preparation; accepted preparation commits form the same auditable ancestry chain as later work. Production refactoring remains prohibited until complete oracle and test-disposition receipts are sealed. Preserve the workspace, reusable API, runtime and accepted safety/identity protections; adapt oracle app composition, deterministic domain fixtures and behavior through those APIs. Do not merge the two full branches, recreate the legacy API, or cherry-pick stale ports wholesale. Use canonical standalone `taskfmt verify` on isolated worktrees; the pinned dispatcher/promotion workflow that hardcodes main must not target the original repository. Final publication/merge is a separately authorized action; the resulting merge tree must itself be verified.

## 10. Evidence matrices and traceability mechanics

- `task-index.tsv` — 73 rows: `task_id, key, title, dependencies, wave, writable_paths, purpose`; the canonical allocation of outcomes to tasks.
- `traceability.tsv` — ~3,236 data rows: `source_namespace, source_id, task_id, requirement_id, acceptance_id, check_id, role, disposition`; joins historical clauses, architecture contracts, component families and app scenarios to tasks with roles such as `baseline`, `frame-contribution` and `primary`, enabling requirement→task, task→requirement, component→task, scenario→verification and decision→implementation queries. Rejected/deferred decisions remain as explicit forbidden-path/scope dispositions; namespaced references distinguish `HIST:A01` from `ARCH:A01`.
- Supporting matrices: `historical-obligations-canonical.tsv` (620 retained clauses across nine ledgers), `architecture-matrix.tsv` (32 cross-cutting contracts), `component-parity.tsv` (54 families covering all 31 oracle widget modules and all 37 main component modules), `application-parity.tsv` (361 scenario contracts: 76 Showcase, 135 Holla, 70 Jackin, 80 TablePro) with per-app inventories and scenario TSVs, `decision-ledger.tsv` (ADJ bindings to implementation owners), `history-revision-index.tsv` (281 parent-relative document edges). Shell/app-flow contribution tables bind partial checkpoint ownership without ever closing a parent scenario early.

## 11. Known risks and open items

- Re-audit findings (`reaudit-findings.tsv`) with repairs and independent rechecks still open; `bootstrap-assets.tsv` stale pending reseal.
- tui-snap PR #1 open and unmerged; its exact reviewed head is a hard prerequisite of reference qualification.
- Parameterized scenarios require complete deterministic oracle-only expansion frozen before candidate replay; broad prose ("all states") is insufficient.
- Main's green self-baselines can conflict with the oracle; every conflict needs an explicit TASK-008 disposition preserving useful structural assertions.
- Exact clock proof and unmodified PTY proof have distinct boundaries; wall time never establishes deadline boundaries.
- Dynamic identity, capture, query attribution, callbacks and alternative paint paths require nonvacuous adversarial tests.
- The critical path is structural; no duration estimates exist or should be fabricated.

## 12. Contradictions flagged

1. Catalog size: "74 packages" (companion text) vs 73 task packages + group README in `completion/` and 73 data rows in `task-index.tsv` — see §6.
2. Historical phase baselines (`main:GOAL.md` Phase 1: 499 `baseline/before` recipes, three apps) vs the current four-app `holla-fable-2026-09-10` oracle namespaces — the current oracle supersedes; historical captures stay historical evidence (see goal.md §9.1).
3. `main:GOAL.md` Phase 4 allows reviewed baseline updates; the proof contract forbids any candidate-side expected-evidence authority — the proof contract governs (see goal.md §9.2).
4. Execution base framing ("on top of holla") vs the chosen strategy (branch from pinned main `7b27732a`) — reconciled in goal.md §6: planning and behavior from holla, architecture from main, end state replaces main.
