# Velnor named testing configuration — 2026-10-21

## Scope and fixed inputs

This CI-only increment starts from PR #38 candidate commit
`df30dc013aa6d78df6f9eb58fc67f3f3d61be37f`. The frozen application source
remains `4169783ae883788186d161af09ba9c481bd84e5c`, the tag object remains
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and Tuiscotti remains pinned to
`a1ea27f757d75d07b05499276cd932e9f8e067ad`.

GitHub release metadata identifies Velnor Actions `v0.1.29` as the latest
stable semver release (published 2026-10-20), source commit
`e7b2f088715379c22be473fab896a43d50cc16fc`. The newer `runner-*` releases
are not versioned stable generator releases. This increment used the
installed `0.1.29` executable by absolute path; `PATH` resolves an older
`0.1.12` and was not used. The installed Apple ARM64 binary SHA-256
`7c4e0efeab89e05928bbd9a375d7b594fd9236caef2eac7b4ad241b3a92dd1d3`
matches the checksum asset in the `v0.1.29` release. The repository's
`mise.toml` and lock already pin `0.1.29`; neither tool pin nor lock was
changed.

## Configuration and generated argv

The approved `velnor-actions config migrate --to 2` output supplies the
`[[stacks.rust.configurations]]` schema. The existing `default` configuration
remains `features = ["default"]`. A separate `testing` configuration requests
`features = ["default", "testing"]`; it is not made the default.

All 259 pre-existing `default` task IDs remain present and their generated
`argv` arrays are unchanged from the starting task manifest. The feature gate
adds a parallel configuration without changing default compilation or tests.

Velnor derives features against each package's declared manifest features. The
generated task manifest passes `default,testing` only to packages declaring
`testing`: `termrock`, `termrock-core`, `termrock-runtime`, `termrock-theme`,
and `termrock-viewport` (the latter's generated feature list is `testing`).
It does not pass `testing` to other package manifests. For packages with no
matching feature, Velnor's plan explicitly reports that it is using that
package's default features; these are visible plan warnings, not silent argv
fallbacks. Since Velnor 0.1.29 has no package-scoped configuration selector,
the named configuration creates parallel default-equivalent package tasks.
This is generator behavior and is recorded here rather than hidden by a
hand-written task or generated-file edit.

The generated `termrock` testing task passes:

```text
cargo nextest run --profile ci --locked --offline \
  --manifest-path crates/termrock/Cargo.toml --package termrock \
  --no-tests fail --no-default-features --features default,testing
```

`nextest list --list-type binaries-only` with the same feature arguments found
the facade's `ex01_button`, `ex02_text_input`, `slots`, and
`split_block_consumer` binaries. The exact generated test command then ran 13
tests across six binaries: 13 passed, 0 skipped. An additional focused run of
the four requested binaries ran seven tests: seven passed, 0 skipped.

## Generation and validation

- Velnor `0.1.29 config migrate --to 2 --write`: succeeded; output follows the
  tool's schema.
- Velnor `0.1.29 plan`: succeeded; detected 47 workspace packages and emitted
  a separate `testing` configuration. The plan records per-package feature
  fallbacks for packages without `testing`.
- Velnor `0.1.29 generate`: succeeded. Only generated workflow and task
  execution manifest changed beneath `.github`; no generated file was edited
  by hand.
- A fresh `generate --output-dir` preview compared recursively equal to the
  checked-in `.github` tree (`diff -qr` exit 0).
- Generated actionlint invocation: passed.
- Zizmor 1.30.1 on `.github/workflows/ci.yml`: no findings (offline mode;
  YAML-anchor advisory only).
- The generator's plan still contains `cargo test --doc` invocations. The
  repository policy requires `cargo nextest` for Rust tests and forbids
  `cargo test`; no doctest command was run. Velnor 0.1.29 schema has no
  doctest-disable or per-task command setting. This is a remaining upstream
  policy blocker; generation is not represented as full CI acceptance.

This record demonstrates enabling the public facade's feature-gated consumer
tests. It does not establish full-workspace CI success or application parity.
