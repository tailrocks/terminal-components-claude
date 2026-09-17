# Runner context-index receipt binding — READINESS-08

TASK-070 owns the immutable `tc-proof-context-index/v1` qualification corpus (`runner-bootstrap-index.py`, VF-03). TASK-071 does **not** re-run that suite on `--group 071`; index isolation is inherited through an explicit regression check.

## Machine authority

| Field | Value |
| --- | --- |
| Index owner | **TASK-070** |
| Transitive check | **CHK-008** in `verify.toml` |
| Driver argv | `--group 070` (runs `runner-bootstrap-index.py`) |
| Covers | **AC-006** — preserve accepted group70 with its independent corpus |

## Preconditions

- **P-001** on this package already requires an accepted **TASK-070** receipt and integrated ancestry.
- The verifier subagent publishes `$RUN_DIR/context-index.json` from accepted predecessor receipts; TASK-071 must not substitute a candidate-authored index.

## Acceptance rule

Passing TASK-071 requires both:

1. All `--group 071` checks (CHK-004/005/006/007) — accounting and stage-transition qualification.
2. **CHK-008** — transitive TASK-070 index suite at `--group 070`.

A production dispatcher that validates only per-operation child hashes without full index membership fails CHK-008 even if group 071 cases pass.
