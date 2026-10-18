# Verified tui-snap repair

Final tool commit **e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2**, tree `e87f3fe3ae49d8066afe8ab7b319f319c2a1dd0f`, clean isolated `tool-sources/tui-snap-repaired` on `codex/qualification-repairs`. Original source `5036cf87e621e6beb66deffe3224abdbefc955cb` and original qualification captures remain unchanged.

Three signed commits: `7d5d62c` implementation; `02102e9` independently reviewed reference-derived fixture migration; `e45d3fa` physical-cursor boundary correction. Each has DCO and Codex coauthor trailers.

## Final verification

`FINAL.json` binds exact source/tree, binary, commands, counts, log/artifact hashes and both independent reviews. Stable resolves to Rust1.98.1 (48a229cea), isolated from the application workspace and its Rust1.88 promise.

- Locked all-target/all-feature suite: **49 passed**, zero failures/filters;49 identities listed in final-test-identities.json.
- Locked backend-free suite: **39 tests +1 doctest passed**, zero failures/filters.
- Explicit all-feature doctests: **1 passed**.
- Locked all-target/all-feature build, format check and strict Clippy: pass.
- Both complete test suites execute all24 tool visual fixtures, without approval bypasses. Visual tests took132.96s and135.42s in final logs. Earlier120s orchestration timeout is retained and not counted as a pass; final bounded900s executions completed.
- Original hand-authored384-cell ANSI oracle rerun with byte-identical fixture: zero differences, zero missing canonical fields. Five injected cell/style/coordinate mutations still detected. Cursor, DIM153 resolution, literal paste, zero-based click, drag endpoints, resize and style-only settling pass. PNG inspected; HIDDEN glyph absent, DIM F visible, last-column C correct.

## Root fixes

DIM arithmetic now divides before narrowing; exhaustive256x256 channel-pair test proves it. Schema3 records hidden/blink and rejects unsupported schema2 imports. PTY/raw/buffer adapters preserve fields and continuation styles; hidden glyphs are omitted from PNG/SVG paint. The original symbols remain canonical data, so concealment is not secret redaction.

A narrowly patched, fully licensed vt1000.16.2 is vendored: independent bold/DIM with correct reset/serialization, hidden/blink/strike attributes, DECAWM on/off and serialization, proper right-edge overwrite, suppression of nonfitting wide glyphs, and continuation attributes. Archive URL/SHA256 and all original file hashes are committed under vendor/UPSTREAM.json. No registry source edits or replacement emulator.

Physical cursor read now separates internal pending-wrap columnN from visible columnN-1 without modifying parser state. Independent tests cover widths1/2/8/80, repeated reads, combining marks, subsequent wrap/overwrite, resize, and public raw-replay rejection of zero dimensions.

Existing paste keeps termlens terminal emulation and sanitization. New paste_literal preserves LF, requires bracketed-paste mode and rejects delimiter injection. Tests cover both success and refusal. PTY example lookup now respects external target directories without a source-tree symlink.

## Independent review and fixture migration

`independent-review/REVIEW.md` accepts the implementation and24 migrations, bound to image index SHA256 `2a7cb723fc6323b7df1c8dda66467fe4ec6296ce5582c1b6ca3a899fb9e7364b`. Reviewer independently reconstructed every expected frame from original approved bytes and audited unchanged fixture source. Only schema3/known-false hidden+blink and60 continuation-style corrections across6 Glyph fixtures change canonical data. No actual capture seeded an expectation. Six Dialog image pairs were visually inspected: exactly928 changed pixels per pair, confined to the DIM No label;18 image pairs remain byte-identical. Only reviewed file hashes were applied; applied-review-verification.json records them.

`cursor-independent-review/REVIEW.md` independently accepts e45d3fa and passes3 targeted external suites. All24 approved files remain byte-identical to reviewed migration. Reviewer reports and hash inventories are bound by FINAL.json.

## Reproduce and acquire cleanly

`rtk proxy python3 tool-repair/run_repaired.py` rebuilds exact pinned repair, reruns the original fixture oracle and full tool gates. `run_final_gates.py` separately reproduces the complete matrix with streamed failure logs,900s per-command bounds and explicit running/complete status. No approval command is invoked.

`tuisnap-repair.bundle` contains all3 commits and requires immutable base5036cf8. `tuisnap-repair.patch` is the complete mail patch. Hashes are in source-artifacts.json; original reviewed vt100.patch is preserved, with final delta separately in vt100-final.patch. Clean acquisition was executed from a newly initialized repository: fetch pinned base, import bundle, checkout final commit, verify clean tree and locked metadata resolving exactlyone vt100 from that checkout's vendor directory. See clean-acquisition.json and clean-metadata.json. External consuming workspaces must repeat the vt100 path patch at their workspace root; Cargo ignores dependency-local patch tables.

## Remaining explicit limits

Raw ANSI replay still cannot observe cursor appearance; use qualified PTY capture for shape/blink assertions. Blink presence is preserved but slow/rapid rates and phase remain combined/frozen. Raster output is a pinned approximation, not terminal pixel identity. Tui-test's original blink/width/cursor-appearance limitations remain documented in tool-qualification/REPORT.md. macOS ECHO/ICANON restoration passes; exact termios deltaPENDIN is retained in evidence.

This repair qualifies synthetic tool contracts. It grants no application snapshot approval and proves no application parity, secret masking, simulations, CLI or user journey. All application sources and snapshots remain untouched by this task.
