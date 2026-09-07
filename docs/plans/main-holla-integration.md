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
