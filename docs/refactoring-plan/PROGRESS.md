# Refactoring completion planning progress

## Current status — completion reopened

The user requested a renewed source-first review of the whole plan and every task line. The previous catalog and qualification passed their recorded checks, but new independent review has identified scope and contract gaps. Execution readiness is therefore reopened. See [the current re-audit register](reaudit-plan.md). Prior review closures and the original artifact manifest remain historical evidence; they do not certify later edits or prove source completeness.

No terminal-components production refactoring, campaign task execution, integration branch creation, merge or publication has been performed under this planning goal. The new proof harness remains a future implementation deliverable; preparation self-tests are not evidence that it already exists or passes.

The additional whole-main-versus-Holla review has an exact 7,885-path inventory and a [working continuation proposal](branch-continuation-proposal.md). Foundation, component and Holla source partitions are fully read; other partitions and canonical repairs remain active. The [artifact report](branch-diff-artifacts.md) covers all 7,019 generated paths, including 17 cross-representation time-state discrepancies, without claiming manual raster review or oracle acceptance.

## Phase 0 planning checkpoint (2026-09-15)

Phase 0 cross-artifact validators are green in the working tree: `python3 docs/refactoring-plan/evidence/validate-plan.py --summary` passes with zero errors; `assemble-plan.py` and `derive-task-graph.py` report no stale projection; DEC ADJ-14 owners bind TASK-013/015/068 in traceability; all 73 AGENTS protocols match the canonical hash; bootstrap assets are refrozen (**211** rows; all **seven** `freeze-bootstrap-assets.py --check` groups pass, including capture-atomicity). CI visual smoke (`--profile ci`) passes 7550/7550 (~87 s). **All 49 of 49** [re-audit findings](reaudit-findings.tsv) are closed at the planning boundary (2026-09-15): bootstrap deferrals at planning-bootstrap qualification; **BRANCH-01** closed via [coordinator integration witness](reaudit-branch-coordinator-integration.md) (components-B ADJ-22, TASK-031/025/008 projection, closure partition 244/244 disposition-bound). **§22.17** fresh witness complete ([`review-readiness-final.md`](review-readiness-final.md) @ `3213fce2`); **READINESS-01–05** repairs and `READY FOR REFACTORING EXECUTION` remain open. TASK-001 harness (`tc-proof-host`) remains future work. See [execution-readiness-assessment.md](execution-readiness-assessment.md), [pre-execution-preparation-plan.md](pre-execution-preparation-plan.md) (Wave 1 repair registry), and [campaign-arming-readiness.md](campaign-arming-readiness.md) (**do not arm `/goal` until Wave 1 green**).

## Pinned authority

Recorded source inspection on 2026-09-11:

| Authority | Commit |
| --- | --- |
| Immutable UI/UX oracle pin (then peeled `holla-fable-2026-09-10`; live tag `visual-baseline` is `4a79c0a2`) | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| Annotated tag object recorded 2026-09-11 | `a643909d9a782adaf0aa1e3357710a5ed3f24443` |
| Architectural main inspected | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| Planning branch tip (`visual-baseline`; verify with `git rev-parse HEAD`) | `84742bf6` (2026-09-15; verify with `git rev-parse HEAD`) |
| Current-format source and installed CLI inspected | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |

These are recorded campaign pins, not a claim that mutable remote branches have been freshly rechecked on every status update. The oracle governs behavior; main governs retained architecture. [History](history.md), [authority reconciliation](history-ledger-reconciliation.md) and [task-format inspection](task-format.md) explain their different roles.

## Investigation inventory and current re-audit

- Historical reconstruction now records 281 exact parent-relative edges across thirteen source paths, independently enumerated with no missing/extra/duplicate keys. The canonical union preserves 620 HIST obligations. The renewed source-clause/task-proof audit is still in progress; prior fifteen-linked-input reports do not themselves prove a fresh reread: [revision index](history-revision-index.tsv), [source continuation](history-source-continuation.md), [canonical obligations](historical-obligations-canonical.tsv).
- Current architecture and reusable-system inventories contain 32 ARCH obligations and 54 COMP families: [architecture matrix](architecture-matrix.tsv), [component parity](component-parity.tsv), [architecture audit](architecture.md), [component audit](components.md). Fourteen explicit decisions now appear in [architecture adjudication](architecture-adjudication.md); their changed contracts have separate current review obligations.
- Four independent audits and the joined matrix retain 361 base APP scenarios: [Showcase](showcase.md), [Holla](holla.md), [Jackin](jackin.md), [TablePro](tablepro.md), [application parity](application-parity.tsv). The current re-audit repaired source-inconsistent actions, focus, tick, geometry and paste assertions while preserving parent IDs. TablePro cell selection/click repair and full current integration proof remain in progress.
- Important corrected facts remain explicit: Holla exists as a migrated app, not a missing binary; main has 11 worlds versus 34 oracle worlds. Dynamic-disabled capture cancellation already exists on main and its twelve focused tests pass; remaining overlap proof is distinct. Passing historical/app logic tests is not complete new-oracle parity.

The current synchronized assembly contains **1,170** source IDs and **3,243** trace edges (`validate-plan.py --summary`, 2026-09-15). New decisions, source-qualified owners and named contributions continue to land. Final projection, graph and complete source/payload joins remain pending active-author handoffs; a self-consistent join alone cannot establish source completeness.

## Catalog and execution graph

The [task index](task-index.tsv) contains 73 bounded canonical packages under [completion](../../refactoring-tasks/terminal-components/completion/README.md). Current taskfmt project lint passes all 73 when invoked with pinned `--config …/experiment.toml --projects-root …/refactoring-tasks`; canonical schemas and AGENTS provenance are preserved. Packages specify requirements, typed acceptance, check argv, dependencies and writable/forbidden scope. Lint establishes format consistency, not trustworthy execution or semantic completeness.

The [derived graph](task-graph.md) currently has maximum depth 35, 24 equally deepest paths and zero unordered writable-scope overlaps. TASK-066 depends on TASK-065; no stale soft lock replaces that dependency. [Machine graph](task-graph.json) records actual predecessor data. Real dependency receipts, preparation accounting and candidate-observer authority require separate proof because an acyclic TOML graph alone cannot prove them.

[Proof contract](proof-contract.md), [executor adaptation](campaign-executor-protocol.md) and [DAG repair record](dag-final-repairs.md) define the selected standalone taskfmt/host workflow. The future campaign branch starts from pinned architectural main before proof/preparation tasks; production repair waits for accepted complete baseline/disposition products. This does not authorize branch creation during planning.

## Verification and external tool evidence

The identified reusable tui-snap gaps were closed by merged [PR #1](https://github.com/donbeave/tui-snap/pull/1) (reviewed head `883d03f19d890bbbf27468798db78b04e85297ac`). The campaign pins `0a2e490802b7b048cd96349c6af860f8a3a05c3d` on `donbeave/tui-snap` main, which includes that merge plus tiered grouped checks and PTY pacing used by visual-baseline smoke. [Verification investigation](verification.md) distinguishes real direct/PTY/tool tests from future full-oracle captures.

The prior comparator, host, runner and actual-Rust preparation under [evidence](evidence/) has recorded independent review in [executed verification](planning-verification.md). [Bootstrap assets](bootstrap-assets.tsv) now record 202 frozen rows across six groups (proof, runner, flow, architecture, style-timing, broker); all pass `freeze-bootstrap-assets.py --check`. Source-policy, actual style-time qualification and corrected flow inputs still require independent review beyond the artifact-integrity gate. Current qualified-source and timing work is linked through [the finding register](reaudit-findings.tsv).

Numeric oracle traces, immutable complete capture bundles and accepted production-runner receipts are future prerequisite outputs. Their hashes must be produced and independently accepted, never fabricated during planning.

## Independent review and completion

The prior reviewers covered [architecture](review-architecture-final.md), [UI/UX parity](review-parity-final.md), [task decomposition](review-dag-final.md), [verification](review-verification-final.md) and [integration safety](review-integration-final.md). All 18 findings in [that historical register](review-findings.tsv) have recorded closure. The current [source-first register](reaudit-findings.tsv) contains additional findings and is not closed by those earlier reviews.

Latest history synchronization covers all 620 canonical clauses and 1,108 historical edges. The independent source-clause reading is complete; new whole-branch findings and owner/proof changes still require integration. Prior all-73 lint/idempotency results and newer focused package lints are recorded separately, not claimed as a current final aggregate pass. The complete artifact-integrity gate (`validate-plan.py --summary`) **passes** in the current working tree (2026-09-15); that does not close independent witness reviews, whole-branch integration or production harness readiness.

The previous planning pass recorded all twenty acceptance conditions as met. The expanded re-audit must establish them again against current artifacts and source evidence before this goal can complete. Future production harness implementation, oracle capture/sealing, refactoring, integration and merge approval remain separate gated work; none was executed during planning.
