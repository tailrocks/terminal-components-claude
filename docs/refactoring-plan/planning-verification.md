# Planning verification procedure

## Purpose and boundary

Use this procedure to verify planning artifacts after contract or evidence changes. It covers the task catalog, joins, graph, source mappings, trusted inputs, and preparation checks. It does not certify the Rust implementation, UI parity, visual approval, or completion of the refactor.

## Required checks

1. Run the canonical task-format lint against all task packages using the pinned formatter/version in [task-format.md](task-format.md). Preserve its complete diagnostics; an error-free summary is meaningful only when the inventory includes every package.
2. Run the read-only planner validation: `python3 -B docs/refactoring-plan/evidence/validate-plan.py --summary`. Inspect every diagnostic and the validator's exit status. `--inventory-only` is narrower and does not establish readiness.
3. Verify deterministic joins by running `python3 -B docs/refactoring-plan/evidence/assemble-plan.py` and inspect its `changed` result. Use `--write` only when the authored source records have been reviewed and the generated fields are intended to change.
4. Check the derived dependency graph, protected source/task payloads, exact traceability joins, scenario ownership, bootstrap asset membership, and the negative controls required by their task contracts.
5. Independently review any changed contracts or proof scripts against their original counterexamples. Do not accept self-tests as the only evidence for a repair.
6. Record the exact checkout revision, toolchain, command, output/exit status, files covered, and limitations. Re-run affected checks after edits; a previous receipt does not transfer to new bytes.

## Current evidence boundary

The [re-audit register](reaudit-findings.tsv) lists unresolved synchronization, independent-review, and qualification work. Those rows remain open until current checks and required independent review are recorded. The planning validator has reported errors in this checkout; no global planning pass is claimed here.

## Completion gate

A planning check closes only its named scope. The current execution-readiness decision is recorded in [planning acceptance](planning-acceptance.md). Product implementation and parity require their own source-level tests, visual evidence, and [final integration gate](../../refactoring-tasks/terminal-components/completion/069/README.md).
