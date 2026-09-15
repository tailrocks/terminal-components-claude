# Runner context-index receipt binding — READINESS-08

TASK-070 owns the immutable `tc-proof-context-index/v1` qualification corpus (`runner-bootstrap-index.py`, VF-03). TASK-072 does **not** re-run that suite on `--group 072`; index isolation is inherited through an explicit regression check.

## Machine authority

| Field | Value |
| --- | --- |
| Index owner | **TASK-070** |
| Transitive check | **CHK-009** in `verify.toml` |
| Driver argv | `--group 070` (runs `runner-bootstrap-index.py`) |
| Covers | **AC-007** — preserve exact comparison / index corpus with independent fixtures |

## Preconditions

- **P-001** requires accepted **TASK-071** (which itself transitively requires TASK-070 index qualification via its CHK-008).
- Host context-index bytes remain protected inputs; TASK-072 must not rewrite index members between operations.

## Acceptance rule

Passing TASK-072 requires:

1. All `--group 072` checks — architecture, style-timing, broker, and gate qualification.
2. **CHK-008** at `--group 071` — transitive TASK-071 accounting regression.
3. **CHK-009** at `--group 070` — transitive TASK-070 index suite.

Index membership regressions cannot be skipped by passing only the task-local `--group 072` driver invocations.
