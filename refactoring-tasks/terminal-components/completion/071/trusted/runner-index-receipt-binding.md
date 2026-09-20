# Runner context-index receipt binding — READINESS-08

Native `tools/refactor-proof/runner` owns the immutable `tc-proof-context-index/v1` qualification corpus (`runner-bootstrap-index.py`, VF-03). TASK-070 stays retired fail-closed and cannot supply a producer receipt. TASK-071 does **not** re-run that suite on `--group 071`; index isolation is inherited through an explicit regression check against the native runner.

## Machine authority

| Field | Value |
| --- | --- |
| Index owner | native `tools/refactor-proof/runner` (built by `scripts/campaign-build-proof.sh`) |
| Transitive check | **CHK-008** in `verify.toml` |
| Driver argv | `--group 070` (runs `runner-bootstrap-index.py`) |
| Covers | **AC-006** — preserve native group70 runner corpus with independent fixtures |

## Preconditions

- **P-001** on this package requires native proof qualification from `scripts/campaign-build-proof.sh`, [`path-contract.md`](../../../../../docs/refactoring-plan/path-contract.md), [`proof-contract.md`](../../../../../docs/refactoring-plan/proof-contract.md), and `tools/refactor-proof/{src,runner,bin}`; it does not require a TASK-070 receipt.
- The verifier subagent publishes `$RUN_DIR/context-index.json` from native predecessor inputs; TASK-071 must not substitute a candidate-authored index.

## Acceptance rule

Passing TASK-071 requires both:

1. All `--group 071` checks (CHK-004/005/006/007) — accounting and stage-transition qualification.
2. **CHK-008** — native group70 index suite at `--group 070`.

A production dispatcher that validates only per-operation child hashes without full index membership fails CHK-008 even if group 071 cases pass.
