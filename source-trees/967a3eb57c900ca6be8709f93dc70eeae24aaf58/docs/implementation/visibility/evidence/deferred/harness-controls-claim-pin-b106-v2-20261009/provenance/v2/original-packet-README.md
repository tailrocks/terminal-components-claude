# VIS-04 caller-pinned claim controls — trace helper follow-up

## State

This is a fresh external successor to the v1 29-control packet. It contains a single test-harness fix proposal in `DeferredFixture::trace()` and keeps the exact same 29 Rust test names, production helper bytes, source inputs, Nextest selection, receipt schema, cache, and bounded supervisor. The follow-up proposal is not applied to the repository, and its Rust tests have **NOT RUN**.

The previous v1 full run is preserved at `/private/tmp/termrock-vis04-claim-pin-controls-b106-v1/runner/runs/2026-10-09T05-06-26-032Z-7d60e16b/`. It selected 29 tests and completed with 27 passed / 2 failed, Nextest exit 100, run ID `d45df2a3-ef7b-4485-856c-a24744a47a9e`, without timeout, signal, or forced close. Both failing tests panicked at `tests/deferred.rs:441:48` because `DeferredFixture::trace()` treated a missing `trace.jsonl` as an error. Invalid-claim CLI paths correctly stop before fake tool invocation, so the trace file is absent; this is equivalent to an empty trace. The receipt SHA-256 is `cc1d73677d1fa73937ae8c3c6128f5edb09178f4fb8ac5675f0425dbf574cc96`. Raw stdout SHA-256: `e26228854578dfa12afd8202cfcbc796f985f29149544e089806bc2c81c9a123`; raw stderr SHA-256: `a0b4ac201af7801ececaa6fcc8bf1f79eadbbbc1d9c398ac952bdb501a5d4895`.

## Proposed bounded correction

The only source change is in `crates/termrock-visibility-tests/tests/deferred.rs`: `DeferredFixture::trace()` returns an empty vector for `std::io::ErrorKind::NotFound`, while retaining a panic for every other read error. It does not alter production code, the zero-tool-probe assertion, or the requirement. The exact diff is `runner/provenance/trace-not-found.diff`.

The accessor is shared by the deferred control tests. The follow-up therefore selects and reruns all 29 exact controls, rather than combining prior successes with a partial rerun. The v1 27 passing results remain useful diagnosis but do not qualify this changed source snapshot.

## Frozen source and cache

The candidate helper is the exact reviewed caller-pin proposal (SHA-256 `903812b22d757e685be0b0469abe30b5141c877ec5e1bcd41ff4f56f1c58baee`). The source tree retains 14 files, including the committed b106 fixture (`a7ee4a89646439a67d2d4e016b759a2d68eecec7339310778e81acf60235adc4`) and locked package inputs. The test-only follow-up changes only the Rust controls file. The original implementation source review is copied under `runner/provenance/`; the new trace hunk and rerun plan are awaiting independent review.

Static preflight passed at `2026-10-09T05:09:29.789Z` with `PASS_STATIC_PREFLIGHT_NO_CARGO`. It checked all 14 source files, all 29 names, the b106 fixture, tool versions/help, and all 32 locked archive plus 32 sparse-index file hashes. The same-runner `--preflight-only` passed at `2026-10-09T05:09:34.631Z` with `PASS_RUNTIME_PREFLIGHT_NO_CARGO`, `cargo_spawned:false`, and 29 names selected. Its before/after cache closure hashes match: `21867ab3cfc05a00da02bc9e7f1cfd1533ef0be465e0a897b39e3ff893ee26b4`.

The harness run receipt schema is `termrock-vis04-deferred-harness-controls-run-receipt/v1`; the production helper's own receipt schema remains `termrock-deferred-run/v2`. The runplan pins Rust 1.98.1, cargo-nextest 0.9.146, `--locked --offline`, 2 build jobs, one test thread, 600-second deadline, TERM/KILL bounds, and 16 MiB combined logs. The runner rehashes the 64 archive/index files before and after the run. No Cargo test command, production CLI command, Python test driver, or repository mutation occurred during packet preparation.

## Review and next run

Review `runner/provenance/trace-not-found.diff`, source and plan hashes, the exact 29-name selection, bounded supervisor, and the distinction between an absent trace and other I/O errors. Do not treat this prepared packet or its no-Cargo preflights as test execution.

The exact normal command, for Root's separate Cargo slot after review, is:

```text
"/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node" "/private/tmp/termrock-vis04-claim-pin-controls-b106-v2-trace-empty-20261009/runner/run-controls.cjs"
```

The separate S1d product23 lane remains 22 PASS / 1 FAIL (BD-21 `w13_filter_wide_trail_cells_clear`). This control rerun does not rerun or qualify product assertions or PTY journeys.

The runtime preflight receipt is `runner/runs/2026-10-09T05-09-34-631Z-1ca26db6/receipt.json` (SHA-256 `cc1e78c01b0aa4b863792629950de6ffeeded627daa516d558a5749d5a40daf6`). It binds runplan SHA `62457ef7ad3fd85a46edf57a2f98151b2aed4932f237fc6e6dd713a83d38bed8`, source manifest SHA `884a7d1f5cbd0f397087b8f5a0fa4ada95279e77fbf481eace30ab2bc2d16a60`, runner SHA `319c97ef266f01a8695e69667a801eb9e391c1847944477ef8cddca745286cfd`, and the unchanged cache closure. These preflight receipts are setup evidence only.
