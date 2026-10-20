# Design branch Velnor Actions 0.1.29 upgrade

This audit records the workflow generator upgrade on `design/interaction-polish`.
It is scoped to Velnor inputs, the Mise lock, generated `.github` files, and
this record. It does not change application code, the fixed product reference,
the frozen visual baseline, or Tuiscotti.

## Source and pin

- Branch source before this change: `8681124d2cbbc75ef8ed998b5918883039a022c5`.
- Velnor Actions: `0.1.29`, source commit
  `e7b2f088715379c22be473fab896a43d50cc16fc`.
- Official release manifest: `.velnor/release-manifest.json`, downloaded from
  the `v0.1.29` release. Linux x86_64 SHA-256 is
  `ffb8c82a10ae55344225cb77f8a3f68c2ccd42068cff5959254bfed5d7780b74`, macOS
  ARM64 is `7c4e0efeab89e05928bbd9a375d7b594fd9236caef2eac7b4ad241b3a92dd1d3`,
  and macOS x86_64 is
  `d60f758b1c7a063f8dda72c81973cddb2d1369536e1931996fe74b418a20e1e6`.
- `mise.toml` and `mise.lock` pin the same release. Mise installed the pinned
  Linux and macOS ARM64 artifacts and completed lock-time provenance checks.

## Generator inputs and output

Velnor 0.1.29 discovered the standalone Rust workspace under
`tools/frozen-cli-frame-support`, which has no declared `smoke` profile. The
design workflow suites target the root workspace profiles, so
`.velnor/config.toml` now excludes that helper workspace alongside archived
documentation. The generated plan then selects the root `ci` Nextest profile
and includes only the root workspace and `xtask`.

The existing `[agents] claude_md = "symlink"` setting generated a relative
`.github/CLAUDE.md` symlink to `.github/AGENTS.md`; the checked-in symlink type
remains intact. `mise exec -- velnor-actions generate` produced the checked-in
workflow and helper changes. A second generation to `/tmp` compared equal to
the checked-in `.github` tree, including file types and symlinks.

## Verification

- `mise exec -- velnor-actions --version`: `0.1.29`.
- `mise exec -- velnor-actions generate`: passed.
- `mise exec -- velnor-actions plan`: passed; two Rust workspaces, root CI
  Nextest profile selected. The plan still reports the existing
  `tool_install_unverified` warning for `rust@1.99.1(no_checksums)` because
  local planning cannot install on every CI platform.
- Pinned Actionlint `1.7.12` passed against both generated workflows with the
  two exact native-step concurrency ignore patterns Velnor emits in CI.
- A clean generated preview matched the checked-in `.github` tree.
- Independent reproduction at exact commit `417c5e884a96eab5f8bc8be305bc3219c18a14b3`
  initially failed with `unknown_nextest_profile:smoke` when the preview output
  directory did not exist. Repeating from the same checkout with a newly
  created empty output directory passed and matched all 22 `.github` entries,
  including file bytes and symlink targets. The verifier confirmed the same
  Velnor binary SHA-256, tracked 3,050-byte Nextest config, root plus `xtask`
  discovery, and no environment overrides. Preserve both transcripts at
  `/tmp/velnor-design-preview-failure-20261020.log` and
  `/tmp/velnor-design-preview-retry-20261020.log`.

No CI run is claimed by this audit. The product reference branch, frozen
baseline tag and commit, and Tuiscotti pin remain unchanged.
