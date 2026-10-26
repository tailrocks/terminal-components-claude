# T01 foundation API inventory runbook

## Input and ownership

Run from `/Users/donbeave/Downloads/termrock-t01-foundation` on branch `pr38/T01-foundation`.

The fixed input is commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`, tree `66507c4aae0555d65904b268d6c95e7f3e3d17d1`. The product files match accepted T37 integration commit `43b5a322e0da46ab7282b5b820831eb70c91f4ed`.

This fragment inventories ten foundational libraries and nine support, tool, and consumer packages. It excludes the facade, component-family packages, and four applications. The component task assigns W IDs.

## Rebuild

Run:

```sh
python3 docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py
```

The command exited `0`. It consumed five accepted T37 Cargo metadata records. This T01 run did not invoke Cargo.

Generator SHA-256: `4aac2168b80c001393dc09633a4df93f6be177cb31c94a7ca4d8cd56b0f3d4de`. No tests, doctests, builds, or harnesses ran.

The generator also confirmed the fixed input is an ancestor of `HEAD` and that product manifests and source match T37 integration. Both Git checks exited `0`.

## Structural and source audit

The inline Python audit exited `0`. It parsed the fragment, checked every source SHA and line count, and checked item and export source bindings, owner tasks, evidence states, disposition fields, and counts. It read immutable T00 workload files with `git show`; those files were not run.

The audit found 208 source records, with zero missing sources, hash mismatches, or line-count mismatches. It checked five metadata workspaces, four standalone crates, and corrected definition lines: `ScrollRegion` at runtime `scroll_region.rs:44`, `TextReport` at shell `report.rs:68`, and `AppShell` at shell `shell.rs:26`.

The structural export rule is metadata and source inventory only: a current public export absent from the inventory blocks T01 structural acceptance until its migration row is updated. This rule is not a functional test.

## Output and counts

The output is `foundation-api.fragment.json`, SHA-256 `f69c3928344dfd29989c7e1011ccbfe937ca22954f685068202278382ceaf4a1`.

| Record | Count |
| --- | ---: |
| Production crates | 19 |
| Public export rows | 2,826 |
| Public API item rows | 4,147 |
| Source-backed requirements | 788 |
| Historical API dispositions | 497 |
| Blocking open requirements | 40 |
| Historical performance records | 3 |

All 40 open requirements have named owner tasks and proposed cases. Requirement dispositions are 715 `KEEP`, 11 `ADAPT`, 22 `RETIRE`, and 40 `UNRESOLVED`. Each `RETIRE` record carries an approved T37 review decision. Historical compatibility records remain separate from current API rows.

## Evidence limits

All product behavior evidence is `NOT VERIFIED`; zero functional cases were selected. The inventory does not prove Rust compiler reachability, feature behavior, dependency availability, or runtime behavior. T33 performance measurements are `NOT RECORDED`.

Independent Sol high review is `PENDING`. This fragment is not accepted until that review and the parent aggregation are complete.
