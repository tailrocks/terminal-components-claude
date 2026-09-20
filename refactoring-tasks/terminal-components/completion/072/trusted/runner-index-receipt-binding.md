# Native TASK-071 qualification binding — READINESS-08

Native `tools/refactor-proof/runner` owns the immutable
`tc-proof-context-index/v1` qualification corpus (`runner-bootstrap-index.py`,
VF-03). TASK-072 inherits the accepted native qualification evidence through
its TASK-071 dependency; it does not require or accept a retired TASK-070
producer receipt. The `--group 070` argument below is the runner's historical
index-fixture selector, not TASK-070 dispatch. TASK-072 does **not** re-run the
index suite as part of `--group 072`; it checks the inherited boundary through
an explicit regression check.

## Machine authority

| Field | Value |
| --- | --- |
| Index owner | native `tools/refactor-proof/runner` |
| Qualification evidence | accepted TASK-071 native qualification receipt |
| Dependency source | `task.toml.dependencies` (`terminal-components/completion/071`) |
| Transitive check | **CHK-009** in `verify.toml` |
| Driver argv | `--group 070` (native runner index-fixture selector; runs `runner-bootstrap-index.py`) |
| Covers | **AC-007** — preserve native TASK-071 qualification evidence / index corpus |

## Preconditions

- **P-001** requires accepted native **TASK-071** qualification evidence and integrated source ancestry. The graph dependency is the authority; no TASK-070 receipt or lifecycle task is admitted.
- Host context-index bytes remain protected inputs; TASK-072 must not rewrite index members between operations.

## Acceptance rule

Passing TASK-072 requires:

1. All `--group 072` checks — architecture, style-timing, broker, and gate qualification.
2. Accepted TASK-071 native qualification evidence bound by `task.toml.dependencies` and the graph's ancestry receipt.
3. **CHK-008** at `--group 071` — TASK-071 accounting regression.
4. **CHK-009** at `--group 070` — native context-index regression inherited through TASK-071, not a TASK-070 suite or receipt.

Index membership regressions cannot be skipped by passing only the task-local `--group 072` driver invocations.
