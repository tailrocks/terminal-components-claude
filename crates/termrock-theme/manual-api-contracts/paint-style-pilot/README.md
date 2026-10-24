# PaintStyle manual API check

This directory preserves a **manual public-consumer compile check** for `PaintStyle`. It is not a Nextest test, Rustdoc test, automated CI coverage, or runtime-rendering proof. The positive and negative Rust files are linked below; they are retained source inputs, not executable Markdown examples.

## Source intent and current API

The original compile-fail intent is in [`resolve.rs` at immutable commit `8ec895f9ea9cf78e5484d80f96b7189d5b716f00`](https://github.com/tailrocks/terminal-components-claude/blob/8ec895f9ea9cf78e5484d80f96b7189d5b716f00/crates/termrock-theme/src/resolve.rs#L21-L32), original blob `90effd2831171d9a38f471a52869490f23339e86`. The original source bytes remain retained at that immutable Git commit; this fixture does not recreate or replace them. It says direct color mutation must not leave semantic metadata describing the old color. That old example imported `termrock_theme::theme::PaintStyle`; `theme` is crate-private, so an import error could have made the old compile-fail appear to pass for the wrong reason. The retained probe imports `termrock_theme::PaintStyle`, the public export [`PaintStyle`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-theme/src/lib.rs#L35), instead.

The current API at checkout commit `7b2eb51c925a105f210c853e81b8ad440dad210f` is [`resolve.rs` (blob `bf8c8c67bb5319838ee0db5a5e008fcafd9272da`)](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-theme/src/resolve.rs#L12-L89), also available as [the local source](../../src/resolve.rs#L12-L89). `PaintStyle` owns a private `Style`, provides a public `bg(Color)` builder, and exposes immutable `Deref`; it does not expose `DerefMut` ([implementation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-theme/src/resolve.rs#L166-L174)). The positive probe uses the public import and legal builder. The negative changes only the raw assignment shown in its linked source.

## Run the isolated fixture

From the repository root, enter this exact nested fixture directory with the command below. Its own workspace table and relative `../..` theme dependency are deliberate. The root workspace does not include this fixture, so ordinary root tests do not execute either bin. Before each command, check available disk space under the current repository policy and use the normal guarded `mise`/MBX provider. Keep each process group within 210 seconds. Do not use `--manifest-path`, `--config`, a compiler wrapper, `cargo test`, direct `rustc`/Rustdoc, or a new dependency or harness.

```sh
cd crates/termrock-theme/manual-api-contracts/paint-style-pilot
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo check --locked --message-format=json --bin paint_style_positive
# Run the negative only after the positive exits successfully.
/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo check --locked --message-format=json --bin paint_style_negative
```

The positive must succeed before attempting the negative. Preserve complete JSON stdout, stderr, numeric exit status, actual provider identity, and source/manifest/lock hashes with the run's external evidence. Cargo check type-checks the positive `assert_eq!`; it does not execute that assertion.

The negative is valid only when the intended assignment in the correct package/bin is rejected at `src/bin/paint_style_negative.rs:7`. The observed diagnostic was E0594, `cannot assign to data in dereference of PaintStyle`, with the primary span covering `style.bg = Some(Color::Blue);`. Check the actual error message and span; a future compiler may report a different code. A guard/provider/registry/timeout/parse/import error, wrong package or target, or any unrelated compiler error is not proof of this contract. Retain the failure output; do not force a different exit result or weaken the assertion.

## Recorded fixture and tool state

These hashes identify the source used for the reviewed pair; compare them before relying on the recorded result. A changed file, lock, or provider requires source-bound review and fresh evidence.

| File | SHA256 |
| --- | --- |
| [`Cargo.toml`](Cargo.toml) | `7ebb0dba42ee0cd4f2bf125c5786eb70c5d10dee6589f03b3418a63f992f1c88` |
| [`Cargo.lock`](Cargo.lock) | `6162b1d703e2253c9b125de05d2f55f016d415245f42a35419c3f7a975aeb96c` |
| [`paint_style_positive.rs`](src/bin/paint_style_positive.rs) | `15f2a4b9616963102b2f5c7069105d5ae9bc4f45fdfcdf02cd4221a229684f59` |
| [`paint_style_negative.rs`](src/bin/paint_style_negative.rs) | `3beab69ce620f51d28afa5a253ab698409e8564e597927523636f58cbc09b482` |

At the reviewed run on 2026-10-23 UTC, the locked positive exited 0 and the locked negative produced the intended line-7 E0594 rejection. The selected MBX was 1.27.0 while the unchanged project declaration was 1.26.0; Rust was 1.99.1. This records the observed provider, not execution under the declared MBX pin or proof of pin reproducibility. Do not change the pins or override provider configuration to match this historical observation.

The fixture lock was reviewed as an 83-package graph. It resolves fixture-only `smallvec` 1.16.4 while the retained root graph uses 1.16.3; this does not authorize a root dependency upgrade. Do not regenerate the fixture lock for routine checks. If `--locked` reports drift, preserve the evidence and request source-bound review instead of regenerating or bypassing the lock.

## Scope boundary

This manual pair covers only the `PaintStyle` privacy/provenance contract and does not restore the original Rustdoc execution path. The other eight protected negative API intents remain KEEP/REVIEW; this fixture makes no claim about them. No root workspace membership, doctest setting, inventory/scanner exception, guard rule, tool pin, hook, production dependency, or production source change is required or authorized by this probe.
