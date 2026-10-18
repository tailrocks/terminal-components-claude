# VIS04 S1d product 23 assertion run

## Result

The frozen S1d candidate-direct assertion lane **FAILED**: 23 selected, 22 passed, and one failed. The failed row is BD-21 / W13-05, test w13_filter_wide_trail_cells_clear. It was registered ignored and was explicitly run with --run-ignored all; its failure was not inverted or suppressed.

Nextest run ID: b88b36d1-5809-4cc9-b9fd-306970f65be7  
Wrapper run ID: 2026-10-09T03-04-14-975Z-10bae096  
Raw receipt SHA-256: c6fec50571f45a875c375bf5668e4f1b8754eb9ac87ec9aa98c95590c79e1929  
Ledger SHA-256: 485142854f3569909c0d8980de58a4e2f2f863eaf522982a027638d9496853ff  
Independent actual review SHA-256: 8d397a4d2fb9e866e197792bf4eafd4c09e30fb66d9709b8b22523e83242eaf4

The preserved stderr records the failure at crates/termrock-conformance/tests/control_states.rs:6137:5: W13-05: no earlier-paint glyph may leak into 旧日本, got [" ", " ", " ", "旧", "界", "日", "y", "本", " ", " "]. The leaked y remains raw failure evidence; no product repair was made.

## Source and selection

Product source commit: 1d797d41c8141fcbdc3f69d7f11eb8875ab54712. The frozen source archive has SHA-256 a042bfc9e3a861660709fc345937c4ef2cecbebe6bb08c0646fafcdab7643edb and contains 481 files. The exact source manifest is preserved under provenance/source-manifest.json.

The selection is package termrock-conformance, binary control_states, with 23 canonical names: 22 normal registrations and ignored BD-21. The listed target contains 110 tests; 87 were filtered out. The run used locked, offline Nextest with --run-ignored all, two build jobs, one test thread, zero retries, a 600-second deadline, and a 16 MiB combined log cap. The frozen run plan, runner, wrapper, static/runtime preflights, names, and stable case map are in provenance/.

The three contract reference files under contract-source/ are separate copies from the VIS04 harness packet. They were not overlaid on the S1d product source.

The product source's registry status fields remain NOT_RUN metadata in the frozen snapshot. The actual execution result for this packet is FAILED as reported above; no registry mutation was made.

## Evidence and limits

runs/2026-10-09T03-04-14-975Z-10bae096/ contains byte-preserved list/run logs, the raw wrapper receipt, the separate external ledger record, and the independent actual-result review. provenance/reviews/ contains the independent static runner review. SHA256SUMS.json lists hashes for the files copied into this directory; it excludes itself.

This is direct candidate assertion evidence only. It does not establish paired PTY behavior, reference parity, screenshot parity, or product completion.

## Portability

This evidence directory is not a self-contained replay bundle. The 14 MB source archive and extracted source tree remain at /private/tmp/termrock-vis04-product23-s1d-plan-20261009-v3.1QGtWW/; the built Cargo target, reported at about 1.9 GB, also remains external. The private Cargo cache is /private/tmp/termrock-vis06-cargo-home-vis06-holla-20261009-44968e82-6442-42c6-a5b4-8aaac2bc64fa. The recorded toolchain is Rust 1.98.1, cargo-nextest 0.9.146, and Node 24.20.0 on macOS ARM64. The provenance copies contain absolute paths to that host packet and must not be invoked from this evidence directory.

The prelaunch run plan retains its original prepared_not_launched mode and is included byte-for-byte because the raw receipt binds its hash. The receipt is authoritative for the later execution result.
