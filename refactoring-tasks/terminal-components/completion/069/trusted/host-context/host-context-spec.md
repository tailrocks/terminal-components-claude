# TASK-069 frozen host context templates

Host `prepare`/`freeze` MUST materialize contexts from `trusted/host-context/*.template.json` plus run/task/tree bindings. Candidates cannot alter template bytes.

## Pinned identities

| Pin | SHA |
| --- | --- |
| architectural_main | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| product_oracle | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| snapshots_tree_digest | `3be0a0c035e0688c5efe713636193b922e71997d7d9ac55b9f0100e5272f29db` |

Recompute snapshots digest at arm time:

```sh
python3 docs/refactoring-plan/evidence/compute-snapshots-digest.py
```

## CHK-001 — merge-readiness (preflight)

- Require pinned architectural main as ancestor of tested tree
- Read-only remote `refs/heads/main` drift detection; fail until fresh integration
- Require accepted prerequisite receipt digests from host ledger

## CHK-005 — fidelity visual + test inventory

- Mandatory fidelity command (no `TUISNAP_FAST`):

```sh
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

- Companion: `cargo nextest run -E 'test(store_integrity)'`
- `snapshots/` and `shots/` tree digest must match pinned `snapshots_tree_digest`

## CHK-007 — machine close only

- Join passed CHK-001–CHK-006 operation records
- **Forbidden fields:** `human_review`, `adversarial_verdict`, `reviewer_attestation`
- Human adversarial review: [`coordinator-review-contract.md`](../coordinator-review-contract.md)
