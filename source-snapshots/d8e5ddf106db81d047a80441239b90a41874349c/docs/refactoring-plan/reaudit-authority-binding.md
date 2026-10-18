# Independent canonical-authority suffix review

Reviewed 2026-09-11. Scope is the new `bind_authority` projection in `evidence/sync-history-stages.py` and the matching historical-suffix check in `evidence/validate-plan.py`. This is not a whole-plan or historical semantic-authority approval.

## Result

No counterexample found within the claimed suffix-drift boundary. Fourteen independent cases/sequences passed with normal Python and `python3 -B -O`. The actual synchronizer entry point was also exercised against a complete in-memory snapshot of the current relevant tables and package payloads, including all authored Holla remaps. No live synchronization write or shared-script edit occurred.

The tested binding at `sync-history-stages.py:28–34` removes the prior standardized authority suffix, preserves the authored semantic prefix, and derives one current status/provenance suffix from the canonical source. Both initial historical edges and newly constructed remap edges use it (`:49`, `:55`). The tested validator at `validate-plan.py:214–220` requires the exact current suffix and exactly one standardized authority marker. Existing exact-row and protected-edge duplicate checks remain active.

## Executed cases

- Binding repairs an old stale status, wrong source path/line and duplicated standardized suffix; adds a missing suffix; preserves a fresh canonical suffix. Every result preserves the intended authored prefix, leaves the input edge unchanged and is idempotent on a second bind.
- The actual `Audit.traceability` method accepts a minimal complete fresh-canonical source/task/payload fixture with no errors. It rejects stale status, wrong provenance, duplicate suffix, absent suffix and trailing unbound text specifically as `Historical authority/provenance drift`, rather than merely encountering unrelated fixture errors.
- Exact duplicate traceability rows and exact duplicate embedded protected edges are rejected by their corresponding duplicate diagnostics.
- The actual synchronizer main function processed 620 sources, 1,094 historical edges and 16 authored remap instructions in an isolated memory filesystem. The first pass changed 29 virtual files. The immediately repeated pass changed zero, and every resulting historical edge had the current canonical suffix exactly once.
- After that remap had already been applied, the fixture changed HP01's canonical authority and source ledger location. The next pass updated three virtual files; all three HP01 edges received the new status/provenance. Repeating again changed zero files and did not duplicate or reapply the remap.

The independent driver is `/tmp/authority-binding-audit.7Zdy9X/audit.py`. Both commands exited zero:

```sh
rtk proxy python3 -B /tmp/authority-binding-audit.7Zdy9X/audit.py
rtk proxy python3 -B -O /tmp/authority-binding-audit.7Zdy9X/audit.py
```

The driver loads the actual functions without invoking their command-line entry points automatically. For full synchronizer tests it redirects `Path.open/read_text/write_text/glob` under the nonexistent `/__authority_binding_memory_only__` prefix to a Python dictionary. Attempts to write anywhere else raise an exception. The reported `written: true` therefore describes writes to that dictionary, never changes to the shared repository. Required assertions use explicit exceptions and stay active under `-O`.

## Limits kept explicit

The suffix establishes which current canonical status and source location an edge names. It does not prove that the authored semantic prefix correctly interprets the source, that the canonical row itself is authoritative, or that an underlying source ledger projects to the canonical row. A semantically wrong authored sentence followed by a correct canonical suffix remains outside this check's claim. Source-ledger projection and whole-plan semantic reconciliation need their separately assigned checks. “Duplicate” here means repeated standardized suffixes or identical trace/protected rows; this review does not claim detection of every semantically redundant edge written with different prose.

Active task/history authors may change planning data after the snapshot. The live catalog was intentionally not synchronized during this independent test, and a live validation PASS is not inferred from the virtual normalization result. The coordinator must complete its owned synchronization and final validation after authors finish.

## Tested source identities

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `docs/refactoring-plan/evidence/sync-history-stages.py` | 114 | `0ea396d6be1b96faed5b1c27b00530e45445041904df7ce7de13696bc4e28955` |
| `docs/refactoring-plan/evidence/validate-plan.py` | 329 | `998bef6e514f8adf5bf3435e56f1e34cba48a72cf29f2136a03b6673e887c832` |

Both normal and optimized runs reported the same script fingerprints. No production source, baseline, canonical AGENTS file, Git ref or task-format checkout was modified.
## Independent source-ledger projection extension recheck

Reviewed the subsequent `Audit.inventory` guard at `evidence/validate-plan.py:61–77`, snapshot 343 lines, SHA-256 `e8d5aa54d3c2a6eb5172ae85865deffb5bd7bdced4da3ddc89c92d12f2eadf87`. This extends the earlier suffix-only claim; it does not retroactively broaden that earlier result.

Temporary `/tmp/authority-binding-audit.7Zdy9X/projection.py` invokes the actual inventory method against copied in-memory tables. Normal and `-O` execution each pass 33 cases. No shared script, live source ledger or generated payload was written. Checks cover the source-consistent positive; mutations of all eleven fields separately in each of the two same-schema source projections (including authority, requirement, current status, remaining work, tests and task mapping); missing canonical rows; missing, wrong-file and wrong-line provenance; and heterogeneous transformed-field acceptance with independent wrong-source rejection. Missing canonical IDs fail the existing exact join, while other same-schema fields fail the new exact field diagnostic.

The real current inventory correctly rejects only `HM46:historical_revision` and `HM47:historical_revision`. The positive fixture changes only those two canonical field values to their actual source-ledger values in memory. Historian owns their live repair. The retained heterogeneous EARLY-AMEND-001 transformation passes without invented column coercion; its source-row identity remains enforced. This is evidence for the explicitly bounded projection policy, not proof that every heterogeneous adjudication is semantically correct. No new guard counterexample found within this scope.
