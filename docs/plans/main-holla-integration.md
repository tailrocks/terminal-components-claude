# Main-based Holla integration execution plan

## Contract and authority

The user task supplied on 2026-09-08 is the execution authority. Its complete
text is archived in `docs/plans/main-holla-integration-task.md`. Main remains
the implementation base; Holla is the product reference. All four binaries
(showcase, tablepro, jackin-preview, holla) must satisfy both the product and
architecture contracts before integration into main.

This explicitly supersedes the Holla-first/tree-replacement directions in
`HOLLA_REFACTOR_RECOVERY_PLAN.md` and
`holla-project/notes/main-vs-holla-refactor-analysis.md`, historical three-app
scope, historical machine paths, and Claude/Fable/Opus routing restrictions.
Their findings and surviving technical obligations remain investigation inputs.
No architecture replacement or legacy deletion precedes the complete history
and contract review. A minimal compiler repair and reference characterization
may proceed concurrently. No candidate output becomes expected evidence by fiat.

## Immutable sources and preserved work

- MAIN_BASE: `c12cad8728755cd2d03eefdd8e02891143fca86d`.
- HOLLA_REFERENCE: `794b095c196562d38f1b6f7ce379c128af2a023d`.
- `git fetch origin` and explicit Holla fetch completed; `ls-remote --heads`
  confirmed both match the task's inspected revisions on 2026-09-08.
- Candidate branch: `codex/main-holla-integration`, descended from MAIN_BASE.
- Original dirty checkout is untouched. Its tracked patch and complete status
  were saved externally; useful work must be reviewed before selective salvage.
- Reference worktree is detached and immutable. Build and capture artifacts
  remain outside both source trees, in separate namespaces.
- Local execution paths are recorded in the external evidence directory
  `../terminal-components-integration-evidence`; they are not portable defaults.

## Ownership and dependency order

Only the integrator writes shared files, manifests, public exports, the ledger,
and this plan. Every future builder receives a dedicated branch/worktree,
explicit owned paths, base SHA, dependency contracts, and acceptance commands.
Current scouts are read-only against source; each owns its external report folder.

| Task | Requirements | Owner | Dependency | Current state |
|---|---|---|---|---|
| T01 Pin sources, isolate, preserve dirty work | §1–2 | integrator | none | verified |
| T02 Exhaustive history/obligation map | §3 | history_audit | T01 | two-document history independently verified; Holla/DESIGN semantic review complete |
| T03 Baseline health/artifact inventory | §4, §8–9 | baseline_gates + integrator | T01 | investigating |
| T04 Missing dialog chrome repair | §5 | integrator | baseline reproduction | implemented/reviewed/scoped verified fa99577; Panel follow-up ec7c965 |
| T05 Runtime/time/identity/layout foundation | §5 | runtime_audit, then builder | T02, T03 | auditing |
| T06 Theme/components/customization/public core | §6 | components_contract, then builder | T02, T05 | auditing |
| T07 Showcase 22 pages and shell | §7 Showcase | showcase_contract, then builder | T05–6 | auditing |
| T08 TablePro complete workflows | §7 TablePro | tablepro_jackin_contract, then builder | T05–6 | auditing |
| T09 Jackin full journeys/CLI/simulation | §7 Jackin | tablepro_jackin_contract, then builder | T05–6 | auditing |
| T10 Holla disposition and migration | §7 Holla | holla_disposition, then builder | T02, T05–6 | auditing |
| T11 Pure production-view/behavior/process evidence | §8 | verifier + app builders | T03, shared contracts | pending |
| T12 Mutation/provenance/CI/performance gates | §9 | verifier + integrator | T03, T11 | pending |
| T13 Independent review and corrections | §2, §8, §10 | independent reviewers | committed slices | pending |
| T14 PR, resolved merge, exact-merge verification | §10 | integrator | all acceptance gates | pending |

Each slice follows: characterize, failing reproducer, shared fix, caller migration,
independent view/behavior verification, committed review, integration. Reports
must distinguish implemented, reviewed, verified, and integrated. No gate or
missing artifact is silently skipped. Pending tasks retain the full task scope.

## Evidence ledger

| Task | Source/reproducer | Command/result | Artifacts | Review/candidate |
|---|---|---|---|---|
| T01 | Git refs/status/worktrees/remotes/toolchain | refs unchanged; original main dirty; isolated candidate/reference created | external initial/status.txt, initial/uncommitted.patch | base c12cad8 |
| T04 | 8831a62 extracts dialog chrome without adding helper | `cargo +1.88.0 check --locked --workspace --all-targets --all-features`: E0432 unresolved super::overlay_chrome | external initial/main-check.log | baseline c12cad8; repair pending |

External audit reports must be brought into versioned documentation after their
scope and evidence are checked. Logs and captures retain exact source, executable,
command, environment, clock, viewport and artifact hash identity. Historical
499 recipes and pinned-Holla acceptance are separate namespaces.

## Completion audit

The archived task is the checklist authority, including every explicit command,
scenario, matrix, mutation, artifact, review and final integration requirement.
Green library tests, text-only screenshots, plausible source, and old reports do
not prove the requested outcome. Every requirement needs current authoritative
evidence on the resolved candidate and then exact merged commit. The full goal
remains active until all requirements are proven; main has not been merged.

## Latest evidence and next slice

See REFACTORING_STATE.md, final checkpoint 2026-09-08, for exact committed
repairs, measured failures, current ownership and outstanding work. Full goal
remains unchanged. Initial source audits are copied under docs/audit/main-holla;
raw logs, captures, patches and hash manifests remain separate external evidence.
The sidebar real-process reproducer now confirms the predicted wrong hit target.
Shared compact NavList geometry is integrated in 3710431 (documentation 1f869d8).
Deferred focus traversal is integrated in bcaa200. Integrator independently ran
the public compact-renderer tests (2 passed) and focus traversal tests (7 passed)
on the candidate. Sidebar caller focus hookup and shared hover tokens remain
pending; no full sidebar parity is claimed. Tool qualification found fidelity
losses and correctly fails its strict gate; isolated tool repairs are underway.

The user additionally requires every coherent change to be committed and pushed.
The integration branch is published as origin/codex/main-holla-integration through
bcaa200. Continue signed commits with the Codex co-author trailer and push each
integrated slice. Publishing this work branch does not waive the acceptance gates
required before merging main. Preserve the original dirty checkout.

## Checkpoint: shared boundaries and CLI slices

- Integrated and pushed `89090d6`: sidebar EnterContent uses deferred runtime
  traversal; keyed hover no longer changes other rows, headings or gaps.
  Integrator executed all ten sidebar tests: nine pass; the exact hover-token
  mismatch remains red until the theme slice.
- Integrated and pushed `4e84b74`: core owns keyboard vocabulary; backend is
  optional. This supersedes historical R-14's unconditional backend aliases.
  Integrator independently executed the isolated core/testing dependency graph
  and executable gate: both pass without workspace feature unification.
- TablePro CLI candidate `967f7e7` is committed and pushed separately, awaiting
  independent review. Stable tests: 19 library, 35 application, 3 real-process
  tests pass. Rust 1.88 process tests: 3 pass. Strict Clippy still reports the
  seven existing application painter errors; no CLI-specific error remains.
  Reference help exit 0 versus old candidate 1 and error exit 2 versus 1 were
  reproduced against distinct binaries before repair. Argument-value redaction
  is a narrow security correction. Explicit color versus runtime auto-detection
  remains a shared session-API dependency, not a claimed end-to-end pass.
- Holla pure domain forward-port and bounded seek corrections are committed on
  a separate branch; validated DAG/one-shot target-bound approval work follows.
  No Holla executable or migrated-screen completeness is claimed.
- Runtime explicit initialization slice is in an isolated worktree. Painting
  must never initialize the app; pure Scene focus-state equivalence needs its
  own proof. Publication guard, input compatibility and real time remain next.
- Tool repair `7d5d62c` has a passing independent synthetic cell oracle; tool
  fixture schema migration is under independent source/cell/image review.
  Application expected captures remain unchanged and unapproved.

Current isolated ownership: runtime_audit (runtime/session/testing lifecycle),
showcase_contract (Jackin CLI), holla_disposition (Junie theme and background
provenance), baseline_gates (external capture tools), history_audit (independent
tool migration review), holla_domain (Holla model/simulation),
tablepro_cli_review (read-only CLI review). Root integrates and pushes reviewed
slices. The original dirty checkout and pinned reference remain untouched.

The user reinforced parallel execution: always delegate independent bounded work
to subagents. Keep one writer per owned path, separate worktrees and build targets,
and independent review of committed candidates. Root retains integration ownership.

## Checkpoint: initialization and Holla workspace integration

Integrated/pushed slices: `493eacc` explicit initialization (root 12 focused tests
pass); `52c9eed` independently reviewed Jackin CLI (root 3 parser + 5 process
tests pass); Holla pure domain `2d1989a`, bounded seek `6be420e`, target-bound
one-shot approval `00075c6`, Cargo library wiring `9f57ed6`, policy memory
`0a00bd0`, and reviewed Unicode/usage corrections `217c43b`. Root Holla tests:
35 pass stable/MSRV before memory, 45 pass stable after memory. Workspace wiring
exposes inherited visibility/dead-code lint failures (197 warnings after private
module wiring); owner is narrowing internal visibility without blanket suppression.
No Holla executable or screens exist yet; this is partial migration, not completion.

Theme commit `73a5ee9` was not integrated: root review found additional ambient
style-provenance failures, independently reproduced by its owner. A typed
`PaintStyle` carrier is approved as the structural replacement; raw Style cannot
recover semantic roles from equal RGB values. Theme owner has theme/ui/author/
collection and component files; runtime owner must defer component test edits
until this ownership is released. Showcase/Jackin painter migration runs separately.
TablePro guarded quit owner also owns its query state and pending-row accounting;
cloned active result versus stored tab result remains a separately tracked cause
of data loss, not waived by counting both in the guard.

Runtime publication Slice B runs separately: exclusive painted-frame guard,
commit after successful presentation, owned pending input, explicit settlement,
and one input per compatible frame. Absolute monotonic time and explicit CLI
color policy remain next shared dependencies. Tool repair has completed full
feature/default-free visual matrices; a newly reproduced last-column physical
cursor defect has a separate narrow engine correction under independent review.
The exact tool commit will be pinned only after that review and final qualification.

## Checkpoint: reviewed quit guard and capture regeneration

Integrated/pushed `01908b3` TablePro guarded quit: independent 10 MSRV tests and
six production PTY journeys; root 10 quit + 3 CLI tests pass. Followup `dbb3c66`
is independently accepted and integrated/pushed as `9378db8`, correcting Ctrl+C
precedence when focused Grid consumes Copy; root all 11 quit tests pass.
Canonical tab-owned editor/results state is being implemented separately
to remove reproduced cross-tab edit loss. Guard acceptance does not prove that gap closed.

`040a272` resolves historical source ambiguity through identical source/build
Git objects. `tools/historical-render` now regenerates all 499 HTML/PNG pairs from
immutable archived ANSI/cursor inputs and exact historical renderer source.
Pinned release fonts match installed fonts byte for byte; all 998 output hashes
reproduce, eight corruption/stale-output mutations reject. Original captures are
untouched. Tooling committed/pushed as `c051c8f`; clean detached checkout at that
commit downloaded the pinned font archive and reproduced all 998 exact hashes,
leaving its source clean. Independent review and baseline installation remain.

Qualified capture tool `e45d3fa` completed 49 full-feature and 39 backend-free tests,
both 24-fixture visual matrices, docs/build/fmt/Clippy and independent review.
Tool owner is packaging clean acquisition under tools/qualified-capture in isolation.
Carrier `a19049a` remains unintegrated: independent review reproduced lost modal
layer role metadata; owner is fixing it. App typed-painter adaptations proceed
with separate snapshots; no baseline approval. Runtime publication component
caller migration now owns released component test sections.

Holla accounting `53ba057` review found duplicate tool target and inconsistent
disk-capacity effects; `15bd595` corrections close initial repros, expanded tests
found relative-root overlap and percentage overflow. Final `e7942b5` closes those
findings with 64 independent MSRV tests; root followup is checking equivalent
Docker/Debian aggregate/identity invariants and pre-validation display sums before
integration. Domain owner has the question; no broader accounting acceptance assumed.
Holla owner now also owns apps/holla manifest/lib/CLI/app/screens for real public-API
migration. No Holla executable acceptance is claimed. Monotonic scheduling remains
a shared dependency, not replaced with event counts.

## Checkpoint: acquisition integration and checked simulation effects

Integrated/pushed capture acquisition `16e4e7a` (worker `263be4f`): root source
review, 43 payload hashes, two integrity tests/four corruptions, fresh dual-engine
journey/oracle rerun pass. Root review is in capture-tool-acquisition-review.md.
CI `1318e85` now requires locked Cargo resolution and explicit Bash pipeline
failure handling; full locked metadata resolves seven workspace packages/120
nodes. A matching-output producer failure still exits 42. `f9da03e` repairs the
existing CI guard test's command matching for the added Cargo flag; focused test
passes. Exact required test inventory and MSRV compile-fail coverage remain open.

Integrated/pushed Holla accounting train `e77f754`, `f2eeda8`, `d99ebf9`,
`75f966e`, `c79217f`, `7e054ee` (worker through `196b4d2`). Independent final
network/accounting review passes 64 tests; root actual package MSRV run passes
62. Checked getters reject invalid pre-plan presentation inventories; report
accounting validates before effects/consumption, including count-only networks.
Whole-app/lint acceptance is not inferred. App writer moved to fresh branch
codex/holla-app-migration based on `7e054ee`, with reviewed shared prerequisites;
pure CLI parser `d0e3b36` is not yet integrated and no executable exists yet.

Shared publication `6b496f7` independently passes 22 MSRV tests/seven doctests;
component production prefixes and prior identity mappings verify. Strict timing
still has metric-work mismatch and genuine timing gates to resolve; Scene
interaction snapshots, absolute monotonic scheduling, stationary-pointer rehit,
terminal mode cleanup and always-armed query/cursor policy remain separate work.
Layer carrier repair `db53dd5` and EMPTY inheritance `2d2ebc4` independently
accepted; no baseline blessing. App carrier `f0f6e6a` review found invisible Mono
Jackin text, blocking that slice until author default/Mono recipe resolution lands.
Separate Showcase lint `283593d` passes strict app all-target checks and preserves
240 complete frames; final combined app acceptance remains pending.

Historical 998-file installation is independently approved, limited to exact
regenerated hashes. Gate owner is implementing absent-only, hash-bound additions
and the narrow normative amendment before installation integration. Original
captured artifacts remain immutable. User again reinforced parallel subagents;
all writer ownership stays disjoint and committed candidates receive review.

## Checkpoint: historical archive restored

Integrated `2ebe74c` (worker `2633fe4`): 998 reviewed regenerated HTML/PNG files,
provenance/license outside the frozen archive, and §75 absent-only hash-bound
addition guard. Root verified all 998 hashes and Git's exact 998 additions with
zero original artifact modifications/deletions. Root full xtask 98/98 tests,
bless guard against `7159ff6`, and parity dry-run all pass: 499 recipes and 7,014
parsed input events. Current replay evidence remains required and unapproved.
Nested Python bytecode caches are ignored by `588423d`, keeping tool execution
from contaminating source provenance with generated caches.

Shared author style defaults `fc745ee` is committed and independently under
review. TablePro painter `00a243d` independently exposes the same Mono binding
defect as Jackin: Connect text becomes black on black; owner is applying the
shared recipe seam. Its other 14 changed modal frames affect only the dimmed
page, not dialog cells, and locked captures verify. No affected app paint slice
is accepted until the committed Mono corrections pass review. Test inventory
tool `8b97760` is committed/pushed for root review; all 3,211 historical source
obligations retain explicit pending mappings, with many-to-one relocation
allowed and current identities independent of compiler executable hashes.

## Checkpoint: reviewed shared style and publication integration

Integrated through `a3f157f`: hover planes, typed paint provenance, layer
composition, owning EMPTY inheritance, author StyleDefaults, Showcase/Jackin
and TablePro painter migrations, all reviewed Mono corrections, Showcase lint
cleanup and explicit successful-output publication. The root facade now exports
StyleDefaults. Independent final app reviews accepted Jackin `b436003` and
TablePro `979bee4`; these supersede their earlier Mono rejections. No candidate
baseline was blessed. All commits retain signoff and Codex attribution.

Root stable verification: 746 library tests pass; focused author defaults 6,
EMPTY inheritance 8, publication 10 and theme states 16 pass. Evidence is
`shared-integration-focused.log` in the external evidence directory. Full
architecture integration reports 37 pass and 8 failures, retained as open work:
four checks reject the undeclared fourth application root; named-test coverage
lacks `local_override_page_shows_three_distinct_buttons`; constructor scanner
and legacy scanner report app/shared violations needing source adjudication;
rustdoc-json nightly invocation fails through the Cargo wrapper. These are not
waived. Full application smoke stops at two Jackin preview failures:
`first_use_flow_enters_the_manager` leaves an enter-control UndeliveredIntent;
`every_named_scenario_renders_a_deterministic_frame` fails for launch-running.
The external `shared-integration-apps.log` preserves results. Independent owner
is investigating lifecycle/timing causes; later packages were not run by this
fail-fast command, so no all-application pass is claimed.

Scene snapshot `d19ff3d`, Grid cell renderer `079b910`, and test inventory feature
correction `8ceb281` are independently under review in parallel. Runtime owner
is repairing benchmark measurement boundaries without weakening thresholds;
component owner is implementing an authoritative List full-row hook. Holla
production composition and TablePro canonical tab ownership remain active.
Pinned Showcase production comparisons now cover 352 cases per side and expose
remaining product differences; matching main-derived captures is not fidelity
acceptance. Main is unmerged and completion gates remain open.

Grid renderer `079b910` independently accepted and integrated as `96bfb45`;
root six external renderer tests pass. Scene remains excluded: independent
review reproduced stale model cache reuse with both synthetic and production
TextViewport models. Runtime owner is binding cache lifetime to a borrowed
model projection session before resubmission.

Separate app runs exposed Showcase prompt submission (28/29 journeys pass)
and TablePro connection transition/form closure (33/35 pass), in addition to
the Jackin failures above. App owners are reconciling unconditional component
lifecycle updates with route/modal command gating. External logs are
`shared-integration-other-apps.log` and `shared-integration-tablepro.log`.

Four-app boundary inventory `9f38e7f` now includes Holla in package, facade,
dependency and binary checks. Five focused inventory tests pass and facade
scanning passes; binary identity correctly rejects the missing Holla binary.
Capture inventory is separate remaining work, not silently approved here.
Exact test inventory `8b97760` plus feature fix `8ceb281` independently accepted
and integrated as `551afe8`/`bc7d23c`; historical mappings remain pending.

Latest root checks: all 99 xtask tests and all 10 inventory-tool tests pass.
Rust 1.88 passes 746 library tests plus 46 focused style/publication/Grid tests
on `1855573`. Showcase prompt test corrected for initial published focus,
retaining validation/edit-state assertions and exact outcome; all 29 app
journeys pass (`showcase-fidelity/root-integrated-app-tests.log`). Shared text
segmentation is reused by the painter in `10f220f`; all 16 paint/theme tests
pass without changing width or clipping semantics.

Reviewed Holla CLI and target/operation-bound PostgreSQL simulation review are
integrated through `5f8ac62`; all 69 package tests pass on Rust 1.88
(`holla-domain/root-cli-pg-tests.log`), with no production Holla binary claim. Shared typed
acknowledgement testing found committed/trimmed text could disagree with the
visible draft. A component owner is fixing exact visible equality without
exposing secrets, preserving reference Enter-only-arms behavior. List's
full-row hook remains under final nested-clipping review; Scene remains blocked
on model-bound cache ownership. Main integration remains gated on all open
product, architecture, evidence and verification obligations.
