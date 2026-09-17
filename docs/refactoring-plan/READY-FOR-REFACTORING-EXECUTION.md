# Historical READY FOR REFACTORING EXECUTION issuance

> **Superseded.** This issuance is historical evidence only. The current
> campaign state is **NO-GO**; do not arm or execute from this document. Read
> [`execution-readiness-report.md`](execution-readiness-report.md).

**Issued:** 2026-09-16 (Phase 0 catalog repairs follow on `refactor/holla-parity`)  
**Branch:** `refactor/holla-parity`  
**Catalog commit:** `dcd687a7` on branch `refactor/holla-parity` (verify with `git rev-parse HEAD`; post-repair tip may advance)  
**Parent planning tip:** `7b27732a8c3c131760ec3438f641cb3c11343a42` (architectural main)  
**Campaign worktree:** `.worktrees/campaign` on branch `refactor/holla-parity` (init via `scripts/campaign-init.sh`; historical `task-001-bootstrap` @ `3ed51570` absorbed)  
**Planning branch `visual-baseline`:** `98fdd8d5` — READINESS-08 tip (Wave 2 on `prep-wave1-verify`; verify with `git rev-parse refs/heads/visual-baseline`)  
**Product oracle:** `02f5294bfdbf38004cc49130d0aff1d01f31434c`  
**Architectural main:** `7b27732a8c3c131760ec3438f641cb3c11343a42`  
**Task-format authority:** `52d9f1eb7721f409bc47beb9fced7997b5c13ede`  
**Frozen tag `visual-baseline` (peeled):** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` — unmoved (verify with `git rev-parse refs/tags/visual-baseline^{commit}`)

---

## Issuance statement

Independent planning witnesses, Wave 1 catalog repairs, Phase 0 partition C/D repairs, and mechanical validators establish that the **frozen task catalog on `refactor/holla-parity`** satisfies Phase 0 planning readiness for campaign **specification and dispatch planning**.

**This issuance authorizes:**

- Arming the campaign `/goal` from [`campaign-execution-prompt.md`](campaign-execution-prompt.md) on the exact catalog SHA recorded below
- Creating `.worktrees/campaign` on `refactor/holla-parity` for **TASK-001** bootstrap (first production deliverable)
- Coordinator-led campaign execution per DAG, host protocol, and iteration guide

**This issuance does NOT authorize:**

- Merging or pushing to `main`
- Modifying `visual-baseline` tag or `snapshots/` / `shots/`
- Skipping per-task host verification or independent verifier protocol
- Treating absent `tc-proof-host` as satisfied (TASK-001 deliverable)
- Skipping coordinator adversarial review before final campaign success (see TASK-069 `coordinator-review-contract.md`)

---

## Wave 1 closure evidence

| Item | Status |
| --- | --- |
| READINESS-01 — TASK-031 branch projection | **Closed** |
| READINESS-02 — W-042 removed from TASK-031 | **Closed** |
| READINESS-03 — TASK-008 disposition trusted | **Closed** |
| READINESS-04 — CAMPAIGN_AGENTS overlay | **Closed** |
| WITNESS-01 — accounting context templates | **Closed** — 73 templates + bindings TSV |
| READINESS-05 — TASK-069 review decoupled from CHK-007 | **Closed** |
| READINESS-06 — host-context templates (visual/merge/close) | **Closed** |
| READINESS-08 — transitive TASK-070 index receipt (071/072) | **Closed** |
| IW-02 — catalog argv smoke gate | **Closed** — `catalog_argv_smoke()` validates 479 `/proof/bin/tc-proof` argv references |
| IW-03 — TASK-001 operator bootstrap checklist | **Closed** — `tc-proof-operator-bootstrap-checklist/v1` in campaign-executor-protocol |
| INT-03 — host isolation fixture corpus | **Deferred** — production host qualification is TASK-001+ deliverable |
| §22.17 witness | **Complete** — [`review-readiness-final.md`](review-readiness-final.md) + spot-check below |
| Re-audit register | **49/49 closed** |
| Mechanical gates | **Green** — validate-plan 0 errors, 73/73 lint, bootstrap freeze 211 assets |

---

## Partition spot-check (post Wave 1)

| Partition | Verdict | Evidence |
| --- | --- | --- |
| **C — Architecture / ADJ-22** | **PASS** | `branch-host-projection.tsv` + CHK-006.template.json + producer path lint; TASK-008 trusted disposition; ADJ-22 four-policy Grid prose |
| **D — Verification / bootstrap** | **PASS** | CAMPAIGN_AGENTS ×73 machine-bound in validate-plan; accounting templates machine-bound; bootstrap self-tests unchanged green |
| **E — Integration / TASK-069** | **PASS** | `coordinator-review-contract.md`; CHK-007 forbids human fields; CHK-005 fidelity argv frozen |

---

## Remaining execution prerequisites (post-issuance, not blockers to this document)

| Prerequisite | Owner |
| --- | --- |
| `tc-proof-host` implementation + qualification | `.worktrees/campaign` on `refactor/holla-parity` — Phase 3 **complete** (historical tip `3ed51570`) (CHK-004 141/141, host matrix 63/63, nextest 17/17, context-check subprocess dispatch, Mach-O harness bin sync, architecture exemption); review fixes @ `05b20ad4`/`cd910c7b`/`3ed51570`; IW-03 operator checklist required before production receipt — see [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) |
| Operator explicit authorization to arm `/goal` | Operator |
| Record exact catalog SHA at arm time | Coordinator |
| Wave 2 hardening | **Closed** — IW-02/03 closed; INT-03 deferred to TASK-001+ production host ([`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md)) |

---

## Arm procedure

1. `git checkout refactor/holla-parity && git rev-parse HEAD` — record SHA in campaign ledger
2. Copy `/goal` body from `campaign-execution-prompt.md` only
3. `./scripts/campaign-init.sh` → `.worktrees/campaign` on `refactor/holla-parity` (from `7b27732a…` / absorbed bootstrap)
4. Dispatch TASK-001 per campaign executor protocol (manual bootstrap exception for first host)

---

## Signatures

| Role | Record |
| --- | --- |
| Planning witness | [`review-readiness-final.md`](review-readiness-final.md) @ `3213fce2`; Wave 1 spot-check this document |
| Wave 1 probe | [`prep-wave1-verification-report.md`](prep-wave1-verification-report.md) |
| Mechanical validator | `validate-plan.py --summary` error_count 0 @ catalog commit `dcd687a7` (Phase 0 repairs) |
| TASK-001 planning | [`task-001-operator-evidence-draft.md`](task-001-operator-evidence-draft.md); [`task-001-qualification-report.md`](task-001-qualification-report.md); [`task-001-verify-container.md`](task-001-verify-container.md); [`task-001-progress-completion-guide.md`](task-001-progress-completion-guide.md); [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md); [`task-001-ci-requirements.md`](task-001-ci-requirements.md); PR [#4](https://github.com/donbeave/terminal-components-claude/pull/4) |
