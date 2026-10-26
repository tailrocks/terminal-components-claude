# T01 foundation API inventory

## Scope and inputs

This fragment is owned by the foundation API worker in `/Users/donbeave/Downloads/termrock-t01-foundation`, branch `pr38/T01-foundation`.
It covers ten foundation libraries and nine support, tool, and consumer packages.
It excludes the facade, component families, Showcase, and four applications.
The fragment assigns no W IDs.

The historical product source is commit `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
The current inventory input is commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`, tree `66507c4aae0555d65904b268d6c95e7f3e3d17d1`.
Current production definitions match accepted T37 integration commit `43b5a322e0da46ab7282b5b820831eb70c91f4ed`.
Historical workload definitions use source commit `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
The prior correction starts at commit `35089472a77cbccb6fa40f5dcd08edb7eca910f9`.
Its input fragment hash is `6e7d16140040ce17b7b90002242157dd62d6e2b7347f42e7adc3f4111efa4b09`.
This correction starts at commit `77a6e34f9f874e06832782a22e2e8a72e3ca6c54`.
Its input fragment hash is `252eff0e2a82253e2c789faf038523fa2bac7f68bdfd323c4687272ec0414153`.

The five metadata records were generated from checkout commit `516198caac5e5b5fa0883fa1f209928c2e86c8ba`, tree `a10cd30dfafec83f7668f529bb03e27f1e5b8b4d`.
Their exact artifact bytes are stored at commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`.
The five artifact paths are absent from the `516198caac5e5b5fa0883fa1f209928c2e86c8ba` tree.
The generator checks each absence with `git ls-tree` and each artifact hash against commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`.
Source, workspace, and crate records keep the artifact commit separate from the execution source commit.

The 19 crate rows use workspace, Cargo package ID, and manifest path as identity.
The publisher packages remain separate, although both use the same package name.

## Rebuild and structural checks

Run these commands from the worktree root:

```sh
python3 -m py_compile docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py
python3 docs/implementation/pr38-final/results/t01-foundation-api/build_fragment.py
python3 -m json.tool docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json > /dev/null
git diff --check
```

Each command exited `0`.
The generator reused five accepted T37 Cargo metadata records.
This T01 run did not invoke Cargo.
Its structural gate joined 6,488 public item binding keys to export rows.
Zero public item binding keys were missing.
The gate found 6,634 unique export IDs and 4,468 unique item IDs.
It resolved all 62 retirement members; 59 rows retire and three preserve compatibility.
No retirement member remains unresolved.

The SHA-256 audit rehashed 206 current source files and three historical blobs from Git objects.
It checked 11,190 item, export, and provenance bindings.
It checked metadata artifact hashes and module import evidence.
All 57 module import evidence rows match their source path, line, and hash.
The audit exited `0`.
An earlier inline audit exited `1` because it compared a file SHA with a commit ID.
The corrected source audit exited `0`.

The relative import audit passed for two direct imports and one grouped import.
It binds `slot.rs:5`, `command_scope.rs:6`, and `paint.rs:5` to their source hashes.
The imports resolve `Ui` and `Target` under `termrock_runtime::ui`.
The source scan found zero repeated `super` imports.
A source-unbound case confirms that `super::super::Ui` removes two module parents.
The fragment keeps each lexical `Ui` implementation target beside its canonical owner.

The fragment records `functional_cases_selected: 0` and `behavior_evidence: NOT VERIFIED`.
No runtime behavior is claimed.
No Cargo build, functional test, doctest, fixture, deleted harness, or behavior lane ran.

## Inventory

| Record | Count |
| --- | ---: |
| Manifest-unique production crates | 19 |
| Workspace identities | 5 |
| Source files and evidence records | 209 |
| Public item records | 4,468 |
| Unique export and index records | 6,634 |
| Direct export records | 2,161 |
| Trait implementation contracts and members in item records | 166 / 179 |
| Trait contract and member export records | 365 / 437 |
| Re-export index rows | 3,561 |
| Public source modules | 109 |
| Source-backed requirements | 826 |
| Historical API dispositions | 497 |
| Blocking open requirements | 40 |
| Historical performance records | 3 |
| Associated constant items | 136 |
| Retirement member rows | 62 |
| Resolved module-import inherent items / trait implementations | 34 / 7 |
| Unresolved trait receiver records, classified by source evidence | 46 |
| Unresolved public receiver-owner joins | 0 |

The 46 remaining trait receiver records stay in the fragment.
They include 35 local types, five external types, three generic parameters, two built-in receivers, and one macro placeholder.
No local or generic receiver counts as a missing public owner.

The publisher inventories keep declaration, trait implementation, and export counts separate:

| Manifest | Public definitions | Trait contracts / members | Direct exports | Trait contract / member exports | Public modules |
| --- | ---: | ---: | ---: | ---: | ---: |
| `crates/termrock-visibility-publisher/Cargo.toml` | 274 | 14 / 10 | 52 | 9 / 6 | 3 |
| `tools/termrock-visibility-publisher/Cargo.toml` | 186 | 10 / 7 | 52 | 9 / 6 | 3 |

Each item has an `api_record_id`.
Each export has an `export_id` and an `export_key` with source and definition ownership.
Definition ownership and re-export ownership remain separate.
Alternative declarations retain source path, line, hash, signature, and cfg provenance.
`cfg_provenance_records` stores deduplicated cfg records referenced by ID.

Inherent implementation items stay under their owner type during module glob expansion.
Associated constants use the value namespace.
The inventory records `NavKeys::DEFAULT` under `NavKeys` at both current public type paths.
It records no top-level `termrock_core::DEFAULT` export from that associated constant.

Source-bounded private imports resolve all 22 inherent `Ui` methods to `termrock_runtime::ui::Ui`.
Each method records its import source path, line, and hash.
Each method has root, `author`, and `ui` public paths.
The `NestedLayerSlotAuthor for Ui<'_>` implementation binds to the same canonical `Ui` type.
`RowFn<T> for termrock_collections::DefaultRow` and `LabelOn for Chord` also join exact workspace type owners.

The historical `termrock_runtime::ui::composition` row resolves through its cleanup attachment.
Its current declaration is `crates/termrock-runtime/src/ui/mod.rs:2`.
The inventory records the public module path and its re-export path.
Historical `Ui::compose` record `487a176f` binds to `termrock_runtime::ui::Ui::compose` at `composition.rs:379`.

Re-exports from external packages are source-bound and have no invented local owner.
Twenty paths resolve to `ratatui-core 0.1.2` with lock checksum, requested features, target condition, and metadata evidence.
External definitions and enabled dependency feature closure were not inventoried.

The corrected foundation bindings include `ScrollRegion` at `crates/termrock-runtime/src/scroll_region.rs:44`.
They also include `AppShell` at `crates/termrock-shell/src/shell.rs:26` and `TextReport` at `crates/termrock-shell/src/report.rs:68`.
The component fragment owns their W IDs and facade links.
`PropsList` belongs to `termrock-controls`, outside this fragment.

T37-FOUNDATION-TESTING-03 remains bound to historical input commit `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
The generator records each runtime stub member, each re-export path, and `Cx::record` separately.
Their current input is commit `542349e2f70ec9883d743594f906ac6c4e96ba5a`.
`Runtime::draw_scene` is a separate `KEEP` row under `T37-BUILD-COMPAT-01`.
The current `runtime.rs:2327` method remains feature-gated.
No current API item is created for retired testing helpers.

## Limits and review state

The public binding check uses source and metadata records.
A missing current export key fails the generator until its migration inventory changes.
The check does not prove compiler reachability, cfg behavior, dependency availability, or runtime behavior.
Three public API macro invocations use source templates; compiler macro expansion did not run.

For 433 historical T37 source records, the source record supplies no commit.
The fragment keeps each source hash and evidence record.
It does not infer a source commit or line commit.
T33 performance measurements remain `NOT RECORDED`.

The fragment hash is `79c634ea8e20c6cb62a0c5463f6bfe05442ce15c9e4bb55d59cc6dc612485d73`.
The generator hash is `9761f25d3e118a888e397dda56edaf66bee3ead00986f1a1cce201b35ce73ec5`.
The independent Sol-high correction review remains `PENDING`.
This fragment is not self-accepted.
