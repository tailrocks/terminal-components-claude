# Documentation index

All project documentation lives under `docs/`. The repository root keeps only `README.md`.
For where every retired file landed, see [traceability.md](traceability.md).

## Start here: the refactoring

The current mission is to finish the refactoring on top of the `holla` branch, porting
what `main` actually completed, without breaking UI/UX or visual output. Reading order:

1. [refactoring/goal.md](refactoring/goal.md) — the consolidated current goal, with per-section provenance.
2. [refactoring/plan.md](refactoring/plan.md) — the consolidated plan: waves, pins, task-catalog mechanics.
3. [refactoring/task-status-audit.md](refactoring/task-status-audit.md) — all 73 execution tasks re-checked against plans and main, with per-task verdicts.
4. [refactoring/main-port-analysis.md](refactoring/main-port-analysis.md) — what main completed, what is partial, port candidates and strategy.
5. [refactoring/state.md](refactoring/state.md) — honest current status across both branches.
6. [refactoring/goal-evolution.md](refactoring/goal-evolution.md) — how the goal changed across history (6 REFACTORING_GOAL versions + REFACTORING_STATE timeline).

## Visual baseline (regression guard)

- [baseline/tuisnap-coverage.md](baseline/tuisnap-coverage.md) — the full capturable-surface inventory, the baseline matrix, legacy supersession map, and regeneration procedure.
- `tools/tuisnap_baseline.sh` — regenerates the whole baseline into `shots/tuisnap/` (tuisnap store: frame.json + ansi/txt/png/html per capture, `report.html` index).
- Re-verify after changes: `tuisnap report --store shots/tuisnap` (or per-capture `tuisnap check`).

## Directories

| Path | Contents |
| --- | --- |
| [architecture/](architecture/) | Component architecture: current holla layout, target model, structural delta. |
| [design/](design/) | The living behavioral/visual contract (DESIGN.md). |
| [refactoring/](refactoring/) | Goal, plan, state, audits and analyses for the current refactoring (see above). |
| [refactoring-plan/](refactoring-plan/) | Planning machinery: proof contract, task index, traceability TSVs, wave breakdowns. |
| [baseline/](baseline/) | Snapshot baseline coverage and procedures. |
| [product/](product/) | Product vision behind the Holla app: concept, goals, notes, reference patterns. |
| [improvements/](improvements/) | Active improvements tracker (PLAN.md) + per-task improvement files + historical audit annex. |
| [parity/](parity/) | Holla parity evidence, matrix, verification, preview inventory. |
| [audits/](audits/) | TUI audits: inventory, interactions, editor, research, verification. |
| [plan-verification/](plan-verification/) | Verification records for earlier planning rounds. |
| [sources/](sources/) | Verbatim retired planning files, incl. `sources/main/` snapshots of main-branch docs. |

## Related locations outside docs/

- `refactoring-tasks/` — the sealed execution catalog (73 task-format packages 001–073 + catalog READMEs). Read-only task contracts; execution order comes from each `task.toml`.
- `shots/` — snapshot corpus: `shots/tuisnap/` is the current baseline; legacy categories and their supersession are mapped in [baseline/tuisnap-coverage.md](baseline/tuisnap-coverage.md).
- `tools/` — capture and audit harnesses (legacy tmux-based `capture.sh` family + `tuisnap_baseline.sh`).
