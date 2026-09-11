# Independent streaming-grapheme feasibility review

The coordinator read the complete original 448-line prototype, then all additions in the final 540-line fixture, its README and changed TASK-013/015 contracts. This is planning-only source and feasibility review, not acceptance of a production painter or full application parity.

Final persisted `lib.rs` SHA-256: `b40fbabcf0f6965669041d5e39015cf8720520c36a1e5b4c77fa76fe2a3f7ca3`. `cmp` confirmed tested disposable source and Cargo.lock equal persisted files. The read-only dependency checkout is exactly main `7b27732a8c3c131760ec3438f641cb3c11343a42`, with empty `git status --short`. No TC source was changed.

## Independent executions

Both executions passed all seven release tests, serially, with zero ignored/filtered tests:

1. `rtk proxy cargo test --manifest-path /tmp/stream-projection.ghBlfQ/Cargo.toml --offline --locked --release -- --nocapture --test-threads=1`.
2. The same locked command through `/Users/donbeave/.rustup/toolchains/1.98.0-aarch64-apple-darwin/bin/cargo`, with explicit matching `RUSTC` and empty `RUSTC_WRAPPER`/`RUSTC_WORKSPACE_WRAPPER`. This compiled and ran a separately named executable. Existing dependency caches were retained; this is not a hermetic whole-toolchain certificate.

The debug-strip step warned about missing `libLLVM.dylib`; Cargo nevertheless linked and executed the release tests. This is not relabeled a clean toolchain run. The expected formatter panic was caught by its test, not an application panic.

## Findings and repairs

The initial retained-capacity bound was ambiguous across frames: after an 8,193-byte grapheme, a short ASCII draw legitimately retains 16,128 bytes. The repaired contract bounds capacity by the largest cluster encountered over that scratch owner's lifetime; live pending text remains bounded by the current cluster plus one scalar. New long/short/long and fresh-owner tests distinguish those bounds and verify cleared contents.

The original prototype logged cold allocations without enforcing the proposed ceilings. The final fixture asserts cold ceilings of five events for 256 combining marks and nine for 4,096 marks. It separately observes zero warm scratch allocations, fresh-owner cold allocations, actual output/style allocations, and first-frame cells after 20/8/20 allocation changes within one Scene. Scratch is test-owned here; production FrameState retention remains a required implementation witness.

Tests compare all UTF-8 fragment boundaries for the finite Unicode corpus against whole-input UnicodeSegmentation, exercise public List/RowUi cells, count one original Display call, preserve formatter effects after clipping, and clear on returned error/unwind. They do not prove every Unicode sequence, production byte/style provenance, secret redaction, tabs/control behavior, complete wide-shadow correctness or application parity. Those remain TASK-013/015 gates.

The author's compiled quadratic UTF-8-revalidation mutant was read in its log and mutation description; the coordinator did not rerun that mutant. Seven positive tests and the byte-work assertion were independently rerun. Cursor-call and UTF-8-validation counters do not claim to measure every internal instruction of the Unicode library.

## Disposition

Storage and consumer feasibility support one private FrameState capacity owner and one streaming shared painter. No public scratch argument, complete-row materialization, second formatter invocation or arbitrary cluster cap is needed. Preserve all existing borrowed-input allocation gates. Narrow the universal formatted-output zero-allocation claim to its proved domain; keep long-cell output allocation visible. ADJ-14 records this bounded compatibility decision, not a waiver of production correctness or performance proof.
