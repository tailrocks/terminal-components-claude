# Empty `termrock-shell` doctest target (2026-10-21)

At PR #38 source `b521748360a2a056d95c79090707ded595f53a6a`, `termrock-shell` has one library target and no runnable Rustdoc examples. Its implicit `doctest = true` setting produced empty generated doctest jobs for both `default` and `testing` feature configurations. This source change makes that empty-target fact explicit with `[lib] doctest = false`.

The existing fail-closed `termrock-xtask` documentation inventory remains enabled. A focused regression appends a runnable Rustdoc example to the real `termrock-shell/src/lib.rs` input and confirms that the inventory rejects it. The workspace inventory also continues scanning tracked Rust and Markdown sources, Cargo target inputs, and included documentation. This keeps future examples covered by the executable nextest inventory instead of silently dropping them.

Validation on the bounded source copy:

- `cargo nextest run --locked --offline --manifest-path crates/termrock-xtask/Cargo.toml --package termrock-xtask --test doc_inventory --no-tests fail`: 8 passed, 0 skipped.
- `cargo nextest run --locked --offline --manifest-path crates/termrock-shell/Cargo.toml --package termrock-shell --all-targets --no-tests fail`: 48 passed, 0 skipped.
- `cargo clippy --locked --offline --manifest-path Cargo.toml -p termrock-shell -p termrock-xtask --all-targets -- -D warnings`: passed.
- `cargo fmt --package termrock-shell --package termrock-xtask -- --check`: passed.

No generated workflow or task manifest was edited. **Superseded:** the Velnor 0.1.31 preview note was replaced by the independently validated 0.1.32 upgrade recorded in the current CI migration audit. That preview reported no empty `termrock-shell` doctest jobs. This does not establish hosted CI or application parity.

## Follow-up: default Rustdoc link (2026-10-21)

At source `ffa723642decc48fc62c11fd165af0f63ad46501`, the strict default-feature shell documentation build failed on a crate-root intra-doc link to `reach`, which is cfg-gated behind the `testing` feature. The module is intentionally absent from default builds. Changed only the prose to name the module as code and state its feature gate; no public API, feature, or Cargo configuration changed.

- Before fix: `RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps --manifest-path Cargo.toml -p termrock-shell` failed with unresolved `reach` link (Rustdoc E101).
- After fix: same command passed.
- After fix: same command with `--features testing` passed.
- `cargo fmt -p termrock-shell -- --check` and `git diff --check`: passed.

This is a source-level strict Rustdoc correction only. It does not claim hosted CI, full application parity, or completion of the overall migration.
