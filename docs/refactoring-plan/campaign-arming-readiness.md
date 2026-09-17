# Historical campaign arming readiness — prompt analysis (2026-09-15)

> **Superseded.** Do not arm `/goal` from this document. Current readiness is
> **NO-GO**; use [`execution-readiness-report.md`](execution-readiness-report.md).

**Purpose:** Prepare for arming `/goal` from [`campaign-execution-prompt.md`](campaign-execution-prompt.md) **without** starting production work.  
**Catalog tip (last known):** `84742bf6` on `visual-baseline` (tag peeled `4a79c0a2` unmoved; branch **9 commits ahead**).  
**Method:** Subagent audit + local command benchmarks (re-run 2026-09-15) + doc cross-check.

---

## Executive verdict

| Question | Answer |
| --- | --- |
| **Store coordinator prompt on branch?** | **Yes** — canonical body in `campaign-execution-prompt.md` (with iteration amendments) |
| **Arm `/goal` and start campaign now?** | **No** |
| **Issue `READY FOR REFACTORING EXECUTION` now?** | **No** |
| **Verification commands block edit-loop iteration?** | **No** — tiered gates documented; full matrix only at acceptance boundaries |
| **Safe to continue Phase 0 / local iteration?** | **Yes** — mechanical validators green |

---

## Readiness blockers (must clear before arming)

1. ~~**§22.17** witness~~ — **Done** @ `3213fce2` → [`review-readiness-final.md`](review-readiness-final.md). **READINESS-01–05** repairs still open.
2. **Explicit issuance** — obtain `READY FOR REFACTORING EXECUTION` on exact catalog SHA (not inferred from green validators).
3. **Catalog freeze** — commit + record tree hash at arm time (`git rev-parse HEAD` on planning branch).
4. **TASK-001 bootstrap** — `tools/refactor-proof/bin/tc-proof-host` absent; required after Phase 0, on `.worktrees/campaign` / `refactor/holla-parity` (not tag branch production edits).

Re-audit register: **49/49 closed** at planning boundary ([`reaudit-findings.tsv`](reaudit-findings.tsv)). §22: **20/20 proven** ([`planning-acceptance.md`](planning-acceptance.md)); READINESS repairs open ([`review-readiness-final.md`](review-readiness-final.md)).

---

## Coordinator paste vs stored prompt

The coordinator-supplied `/goal` body matches the stored prompt on mission, Phase 0, topology, verifier protocol, and DoD. **Material differences** (stored version is authoritative):

| Topic | Coordinator paste | Stored canonical prompt |
| --- | --- | --- |
| **Identities** | Single “frozen visual-baseline tree/tag” @ `4a79c0a2` | Tag **vs** planning branch tip (verify at arm time); record catalog SHA |
| **Visual gates** | “Then the mandatory full visual gate” after every production edit | **Tiered:** edit loop = targeted `TUISNAP_FAST=1` filter; acceptance = fidelity full matrix ([`campaign-iteration-guide.md`](campaign-iteration-guide.md) §1/§4) |
| **IMPLEMENTER / repair loop** | Local tests advisory | + explicit FAST filters during repair; fidelity matrix before re-freeze |
| **DoD / final report** | “Full visual-baseline result” | Must record command, env, and tier (smoke / fast full / fidelity) |

**Do not arm from the unamended paste** — it would force ~45–60 min fidelity matrix on every edit and contradict measured iteration practice.

---

## Verification command benchmarks (2026-09-15)

All Rust validation uses **`cargo nextest`**, never `cargo test`.

| Tier | Command | Wall time | Result | Use when |
| --- | --- | ---: | --- | --- |
| **Edit loop** | `validate-plan.py --summary` | ~0.6 s | PASS | After planning/doc edits |
| **Edit loop** | `taskfmt project lint terminal-components --config …/experiment.toml --projects-root …/refactoring-tasks` | ~0.3 s | PASS 73/73 | After task package edits |
| **Edit loop** | `cargo nextest run -E 'test(store_integrity)'` | ~2.7 s | PASS | Snapshot store sanity |
| **Edit loop** | `TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(<filter>)'` | **~17 s** wall (25 combos, `showcase_pages_overview`; nextest ~16 s)* | PASS | After rendering/interaction edits — pick smallest `<filter>` |
| **Smoke** | `cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline)'` | ~113 s | PASS 7550/7550 | PR/CI; not task acceptance |
| **Acceptance** | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` (no FAST) | ~45–60 min | Required at boundaries | Task acceptance, app closures, TASK-069 |
| **Blocked** | `taskfmt verify` / `tc-proof-host` | — | **Binary absent** | Campaign task execution only |

\*Wall time may include post-run `mbx` GC noise; trust nextest summary line for test duration.

**Iteration is not blocked** by command duration if teams follow tiers in [`campaign-iteration-guide.md`](campaign-iteration-guide.md). The only **hard blocker** for campaign *execution* is absent proof host (TASK-001 deliverable).

---

## Pre-arm checklist

### A. Phase 0 (before `/goal`)

- [x] Fresh §22.17 independent witness → [`review-readiness-final.md`](review-readiness-final.md) @ `3213fce2`
- [x] Move §22.17 to **Proven** in `planning-acceptance.md` (20/20)
- [ ] Repair **READINESS-01–05** (see witness doc) and spot-check partitions C/D/E
- [ ] Issue **`READY FOR REFACTORING EXECUTION`** on frozen catalog bytes
- [ ] Record catalog SHA; refresh prompt header if tip moved
- [ ] Operator explicit authorization to arm campaign

### B. Campaign bootstrap (after Phase 0, still not full campaign)

- [ ] `.worktrees/campaign` on `refactor/holla-parity` (`scripts/campaign-init.sh`)
- [ ] TASK-001 → qualified `tc-proof-host`
- [ ] TASK-070/071/072 bootstrap chain per DAG

### C. Explicit non-starters (now)

- Do **not** arm campaign `/goal` until checklist A complete
- Do **not** dispatch TASK-001 production edits on `visual-baseline` tag branch
- Do **not** move/retarget `visual-baseline` tag
- Do **not** treat green mechanical validators as execution authorization

---

## Recommended next step (this preparation goal)

1. **Commit** this readiness doc + any prompt header updates (if not already on tip).
2. Execute **Wave 1** repairs per [`pre-execution-preparation-plan.md`](pre-execution-preparation-plan.md) (READINESS-01–06, WITNESS-01, CAMPAIGN_AGENTS).
3. **When green:** arm `/goal` copying body from `campaign-execution-prompt.md` only; record SHA in campaign ledger.

**This session does not arm the campaign or dispatch TASK-001.**
