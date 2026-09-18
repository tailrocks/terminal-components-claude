# Branch coordinator integration witness — components B and closure partition

> Historical planning evidence. Not current execution authority. Do not replay
> its commands, branch/pin/model/merge/container instructions. Reconcile this
> record against the current readiness report and current contracts.

**Date:** 2026-09-15  
**Scope:** Close **BRANCH-01** coordinator-owned integration from [`branch-diff-components-b.md#authorized-planning-repair-handoff`](branch-diff-components-b.md#authorized-planning-repair-handoff) and bind unread closure-partition rows in [`reaudit-branch-tooling-tests.md`](reaudit-branch-tooling-tests.md).  
**Boundary:** Planning bookkeeping and witness projection only. No production Rust edits, no canonical history/decision ledger edits, no oracle blessing, no `READY FOR REFACTORING EXECUTION` issuance.

---

## Verdict

| Item | Status |
| --- | --- |
| Components-B findings register | **Integrated** — [`branch-diff-findings-components-b.tsv`](branch-diff-findings-components-b.tsv) (6 findings → TASK-021–028/042) |
| Coordinator projection (5 handoff bullets) | **Integrated** — [`branch-diff-components-b-coordinator-projection.tsv`](branch-diff-components-b-coordinator-projection.tsv) |
| TASK-008 disposition bindings | **Integrated** — [`branch-diff-components-b-test-disposition-bindings.tsv`](branch-diff-components-b-test-disposition-bindings.tsv) |
| Closure-partition unread rows (56 paths) | **Disposition bound** — [`branch-diff-tooling-closure-disposition.tsv`](branch-diff-tooling-closure-disposition.tsv) |
| Independent components-B repair rereview | **Witnessed** — §22.17 at historical commit `3213fce2`; READINESS-01–03 track trusted-package projection gaps |

---

## Partition integration summary

| Partition | Coverage | Register |
| --- | --- | --- |
| Jackin/TablePro | 29/29 integrated | [`branch-diff-findings-jackin-tablepro.tsv`](branch-diff-findings-jackin-tablepro.tsv) |
| TUI tests/examples | 112/112 reviewed | [`branch-diff-findings-tui-tests-examples.tsv`](branch-diff-findings-tui-tests-examples.tsv) |
| xtask historian | 18/18 complete | reaudit-branch-xtask.md |
| Components B | 35/35 source read; 6 findings → task repairs | [`branch-diff-findings-components-b.tsv`](branch-diff-findings-components-b.tsv) |
| Tooling/tests closure | 244/244 byte-bound; 56 formerly unread now disposition-bound | [`branch-diff-tooling-closure-disposition.tsv`](branch-diff-tooling-closure-disposition.tsv) |
| Branch inventory sync | reviewed rows promoted via sync script | [`branch-diff-inventory.tsv`](branch-diff-inventory.tsv) + `sync-branch-diff-review-status.py --write` |

Whole-branch semantic acceptance is **not** claimed: **752 / 7,885** inventory paths carry explicit reviewed/disposition rows; remaining inventory paths retain byte bindings without semantic credit.

---

## Coordinator projection detail

### Central authority qualifiers

[`architecture-adjudication.md`](architecture-adjudication.md) **ADJ-22** records additive public-policy authority:

- **StatusBar/Segments:** one shared finite layout/drop **configuration** with unchanged modern defaults; retained-left truncation applies to **source StatusBar only**; Segments uses first-left tie removal and permits zero left survivors (**W-028-06**).
- **Grid gestures:** four explicit policies — modern default, DataGrid, DataTable-row, DataTable-cell — with **W-022-10** isolating global editable versus column/row refusal (**HL-64-GESTURE** qualified payload retained).
- **Picker query submit:** typed keyless **Submit** capability, **disabled by default**; oracle Loading/Error searchable nonblank cases in **W-024-08**; Files jump consumer **W-042-JUMP-SUBMIT**.
- **Completion width:** independent semantic label/detail column maxima (**W-024-07**); geometry correction, not app-local painter authority.

[`component-parity.tsv`](component-parity.tsv) `statusbar-segments` row notes the dual-policy configuration boundary.

### TASK-031 / host witness projection

Branch finite witnesses bound for conformance/host required-set membership (execution projects into TASK-031 CHK-006 witnesses, not planning edits to task packages):

`W-021-07`, `W-022-10`, `W-023-07`, `W-024-07`, `W-024-08`, `W-025-07`, `W-025-08`, `W-026-05`, `W-027-07`, `W-028-06`, plus existing `W-031-10/11`. **W-042-JUMP-SUBMIT** is owned by TASK-042 only (Files jump consumer).

### TASK-025 scope catalogue

[`task-index.tsv`](task-index.tsv) TASK-025 `writable_paths` already matches [`refactoring-tasks/terminal-components/completion/025/verify.toml`](../../refactoring-tasks/terminal-components/completion/025/verify.toml): `text/editor.rs`, `text/buffer.rs`, `components/code.rs`, `completion_025.rs`. No catalogue drift detected.

### TASK-008 disposition sync

[`branch-diff-components-b-test-disposition-bindings.tsv`](branch-diff-components-b-test-disposition-bindings.tsv) binds **W-021-07**, **W-025-07/08**, and **W-026-05** to TASK-008 exact-identity disposition authority: split/preserve compatible architectural assertions; replace only source-adjudicated conflicting observations during execution.

---

## Mechanical checks (this witness)

```text
python3 docs/refactoring-plan/evidence/sync-branch-diff-review-status.py --write
python3 docs/refactoring-plan/evidence/validate-plan.py --summary  # passed:true, error_count:0
python3 docs/refactoring-plan/evidence/freeze-bootstrap-assets.py --check --group capture-atomicity  # checked:true, 9 assets
```

---

## Explicit non-claims

- No second independent semantic reread of all 35 components-B source blobs in this witness (prior [`reaudit-components-b-branch-repairs-ledger.md`](reaudit-components-b-branch-repairs-ledger.md) byte ledger stands).
- No TASK-031/008/025 **execution** receipts — only planning projection bindings.
- §22.17 fresh witness was recorded at historical commit `3213fce2` in the retired readiness review; READINESS-01–03 address coordinator projection gaps found by that witness.
