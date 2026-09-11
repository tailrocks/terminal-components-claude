# Planning verification record

This verifies completed planning/preparation artifacts, not execution of the refactoring campaign or acceptance of application parity. Independent qualification, final source/task joins and all 151 frozen bootstrap assets pass. The complete artifact identity is recorded in [planning-artifacts.tsv](planning-artifacts.tsv).

## Reconfirmed authorities

Read-only remote checks on 2026-09-11 reconfirmed:

- Terminal-components main `7b27732a8c3c131760ec3438f641cb3c11343a42`, Holla `2e2401393c47360741ebd321679de08982dca50a`, annotated tag object `a643909d9a782adaf0aa1e3357710a5ed3f24443`, and peeled UI oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`.
- Task-format remote main and installed `taskfmt 0.2.0` both identify `52d9f1eb7721f409bc47beb9fced7997b5c13ede`.
- Tui-snap upstream main remains `5036cf87e621e6beb66deffe3224abdbefc955cb`. Its [required PR](https://github.com/donbeave/tui-snap/pull/1) remains open, non-draft and mergeable at reviewed head `883d03f19d890bbbf27468798db78b04e85297ac`; no merge was performed.

## Executed checks

| Check | Observed result |
| --- | --- |
| Canonical `taskfmt project lint terminal-components` with explicit configuration and catalog root | All 73 packages pass with zero errors/warnings. TASK-072 also passes after adding AC-009/CHK-011. |
| `derive-task-graph.py` | 73 tasks, maximum depth 33, 24 longest dependency paths, zero unordered writable-scope overlaps. Stored JSON/Markdown and authoritative depth/count/serialization claims agree. |
| `sync-history-stages.py` and `sync-derived-obligations.py`, read-only modes | No remaining changes: 620 historical sources/1,094 edges and 84 derived-flow sources/336 embedded edges. |
| `assemble-plan.py` | Current union has 3,159 edges over 1,159 source IDs. Separately named Holla/shell contributions number 35; their required parent/task/proof bindings are checked. |
| `validate-plan.py --summary` | PASS with zero errors: all inventories, graph, canonical AGENTS, typed mappings, exact protected source payloads and all 151 frozen asset hashes agree. Actual-Rust CHK-011 exists and is bound to AC-009/R-001. |
| Frozen TASK-001 comparator, `python3 -B -O ... --self-test` | 72 cases, 888 report-schema/aggregation rejection probes, nine lying runners rejected. |
| Frozen TASK-001 host, optimized self-test and `--observer-test` | Eight self-tests and eight real Darwin observer gates pass after relocation. These include actual build/test/taskfmt execution, pre-prepare substitution denial, old-child public-evidence mutation, immutable private proof, forbidden reads/writes and replay rejection. |
| Frozen TASK-070 runner, `python3 -B -O ... --self-test` | 58 base cases, 34 actual base observer cases, 232 malformed-result rejections, 14 context-index cases, 58 extension cases, five-stage accounting sequence and zero-worker liar rejection pass after relocation. |
| Actual-Rust independent frozen-layout qualification | 41 standalone Rust cases and 25 actual-TC subjects pass. Real external transport accepts two positive/recovery runs and rejects always-pass, zero-observer and forged-result subjects; both passing and failing relocated executables retain verified source/tool/output binding. |
| `freeze-bootstrap-assets.py --check` | All four groups match 151 source/destination/hash records: 56 proof, 33 runner, 48 flow and 14 architecture assets. The Rust reviewer additionally compared the actual TASK-072 destination bytes and runner siblings independently. |
| `git diff --exit-code` | No tracked terminal-components source changes. All deliverables remain new planning artifacts. |
| New task catalog whitespace check | No whitespace diagnostics. Original evidence patch context and retained measured-source EOF whitespace are not rewritten to change historical evidence identities. |

The canonical lint configuration used here is `/Users/donbeave/Projects/donbeave/task-format/experiment.toml`; future execution uses the separately frozen `/proof/bootstrap/experiment.toml`. These paths are explicit, not ambient-CWD defaults.

The actual-TC archive is deterministic `gzip -n` output. Compressed SHA-256 is `ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119`; decompression independently reproduces the original Git archive SHA-256 `732e40d9cf82e98e3d6375aaeecee349e8cb774abc5094dbbff145555201633a`. Compression changes neither source membership nor bytes. The redundant uncompressed temporary artifact and generated Python caches were moved to `/tmp/tc-planning-recovery.WZzROo`, not deleted.

## Independent review boundary

Five original fresh reviews and subsequent independent repair reviews are linked in [review-findings.tsv](review-findings.tsv), [contract rereview](review-contract-repairs.md), [parity rereview](review-parity-repairs.md), [verifier rereview](review-verifier-repairs.md), [bootstrap rereview](bootstrap-review.md), and [actual-Rust qualification review](review-rust-qualification.md). All 18 tracked findings have independent closure. Additional defects discovered during rereview were repaired and their original reproducers rejected before acceptance. Author self-tests do not substitute for these reviews.

No future application baseline hash, production comparator/host/runner acceptance, terminal-components task completion, integration branch, main merge or push is claimed. Those are separately gated future operations.
