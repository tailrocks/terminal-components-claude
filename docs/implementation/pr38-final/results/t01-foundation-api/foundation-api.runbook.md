# T01 foundation API inventory

## Scope and inputs

This fragment is owned by the foundation API worker in `/Users/donbeave/Downloads/termrock-t01-foundation`, branch `pr38/T01-foundation`. It inventories ten foundational libraries and nine non-application support, tool, and consumer packages. The facade, component-family packages, and four application families remain in their assigned fragments. This worker assigned no W IDs.

The fixed source input is commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`, tree `66507c4aae0555d65904b268d6c95e7f3e3d17d1`. Product manifests and source match accepted T37 integration commit `43b5a322e0da46ab7282b5b820831eb70c91f4ed`; metadata evidence comes from commit `516198caac5e5b5fa0883fa1f209928c2e86c8ba`.

The five metadata records carry distinct `workspace_id` values. The 19 crate rows are keyed by workspace, Cargo package ID, and manifest path, so `crates/termrock-visibility-publisher` and `tools/termrock-visibility-publisher` remain separate despite sharing a package name.

## Rebuild and structural checks

From the worktree root, run:

```sh
python3 -m py_compile docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py
python3 docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py
```

Both commands exited `0`. The generator reuses five accepted T37 Cargo metadata records; this T01 run did not invoke Cargo. Its built-in structural gate passed: 6,266 public item binding keys joined to export rows, zero were missing, all 6,504 export IDs and 4,453 item IDs were unique, 22 `RETIRE` requirements had reviewed absence evidence, and all 20 third-party export rows had no owned definition package.

An independent source audit exited `0`. It rehashed and line-counted all 209 source records, using Git objects for the two immutable historical workload files that are absent from the current tree. It checked source path, line, and SHA bindings for items, exports, and multi-source wildcard provenance; workspace/package identities; unique export keys; cfg provenance references; and the three runtime stub retirement rows.

The fragment reports `functional_cases_selected: 0` and `behavior_evidence: NOT VERIFIED`. No Cargo build, functional test, doctest, fixture, deleted harness, or behavior lane ran.

## Inventory

| Record | Count |
| --- | ---: |
| Manifest-unique production crates | 19 |
| Workspace identities | 5 |
| Source files and evidence records | 209 |
| Public item records | 4,453 |
| Unique export/index records | 6,504 |
| Direct export records | 2,139 |
| Trait implementation contracts / members in item records | 159 / 171 |
| Trait implementation contract / member export records | 348 / 416 |
| Re-export index rows | 3,491 |
| Public source modules | 109 |
| Source-backed requirements | 788 |
| Historical API dispositions | 497 |
| Blocking open requirements | 40 |
| Historical performance records | 3 |

The publisher inventories separate declaration, trait-implementation, and export categories:

| Manifest | Public definitions | Trait contracts / members | Direct exports | Trait contract / member export rows | Public modules |
| --- | ---: | ---: | ---: | ---: | ---: |
| `crates/termrock-visibility-publisher/Cargo.toml` | 274 | 14 / 10 | 52 | 9 / 6 | 3 |
| `tools/termrock-visibility-publisher/Cargo.toml` | 186 | 10 / 7 | 52 | 9 / 6 | 3 |

Each item has an `api_record_id`; each public export has an `export_id` and an `export_key` containing workspace, source package, canonical public path, definition package, definition owner, and Rust namespace. Definition ownership and re-export ownership remain separate. Alternative source declarations for the same key retain their exact source path, line, hash, signatures, and cfg provenance. `cfg_provenance_records` stores deduplicated cfg source records referenced by ID.

Re-exports from external packages are source-bound but do not receive invented local owners. Twenty re-export paths resolve to `ratatui-core 0.1.2`, including the Cargo.lock checksum, requested features, target condition, and metadata/manifest evidence. External type definitions and the enabled dependency feature closure were not inventoried.

The corrected foundational definition bindings include `ScrollRegion` at `crates/termrock-runtime/src/scroll_region.rs:44`, `AppShell` at `crates/termrock-shell/src/shell.rs:26`, and `TextReport` at `crates/termrock-shell/src/report.rs:68`. The component fragment owns their W IDs and facade links. `PropsList` is owned by `termrock-controls`, outside this fragment.

T37’s three `runtime-stub-harness` retirement rows carry decision `T37-FOUNDATION-TESTING-03`, exact source identity at `c817b99e9e43abbbf3519d56b2f5ee86924243d8` (`runtime.rs:3301` or `lib.rs:55`), and explicit absence at the fixed T01 input. The separate `Cx::record` retirement row also uses decision 03. No current API item is created for retired testing helpers.

## Limits and review state

The current public-binding check is structural metadata and source reconciliation. A missing current export key fails the generator until its migration inventory is updated. It does not establish compiler reachability, cfg behavior, external dependency availability, or runtime behavior. Three public API macro invocations are represented from source templates; compiler macro expansion was not run.

For 433 historical T37 source records, the source record supplies a file hash but no commit. The fragment preserves that hash and its exact T37 evidence record, marks the commit as not supplied, and does not infer one. T33 performance measurements remain `NOT RECORDED`.

The correction artifact is `foundation-api.fragment.json` (SHA-256 `3ba2da120c3053527e76f606e13da134f5f9b04f929702a9b4675e70a8c1138e`). The generator is `build_fragment.py` (SHA-256 `37e21945a3db69f1392fa24f850080d9dbd2d6e94eec186a49a85f935241e9d8`). The required independent Sol-high correction review remains `PENDING`; this fragment is not self-accepted.
