# Dependency and toolchain policy

## Source of truth

`Cargo.toml`, all workspace and standalone `Cargo.lock` files, `rust-toolchain.toml`, `mise.toml`, `mise.lock`, and `.velnor/release-manifest.json` define the versions used by this checkout. This page records the update policy and notable compatibility constraints; it is not a replacement lockfile. Refresh the locks and run the affected package, workspace, CI, and visual checks before describing an upgrade as validated.

Keep direct dependencies and compatible transitive dependencies on their latest stable releases. Consider major releases too: update the requirement, make any needed source/API migration, and run the relevant checks. A compatibility hold must name the exact upstream requirement and affected dependency path here. Do not pin an older release just to avoid a migration. Keep exact lockfiles for repeatable builds. Pin Git dependencies to a reviewed commit, and move the pin to the latest reviewed upstream commit after validating its API and impact; do not rely on a floating branch name for builds.

## Current workspace pins

The committed manifests and locks currently select the following shared versions. `Cargo.toml` and `Cargo.lock` remain authoritative if a package has a different target- or feature-specific version.

| Component | Selected version |
|---|---:|
| Rust toolchain | 1.99.1 |
| Cargo `rust-version` (MSRV declaration) | 1.99 |
| Ratatui | 0.30.2 |
| Crossterm backend | 0.29.0 |
| Clap | 4.6.7 |
| Unicode width | 0.2.2 |
| Unicode segmentation | 1.13.3 |
| Unix libc | 0.2.191 |
| `sha2` test dependency | 0.11.1 |
| Tuiscotti Git dependency and CLI | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |

The repo-local tool pins in `mise.toml` are Rust 1.99.1, cargo-nextest 0.9.148, actionlint 1.7.12, shellcheck 0.11.0, zizmor 1.31.1, GitHub CLI 2.102.0, Velnor Actions 0.1.14, and mr-boxington 1.24.1. Velnor's per-target release artifacts are hash-pinned in `.velnor/release-manifest.json`; `mise.lock` records install checksums for supported platforms. Use `mise install` to install the declared tools. The `mbx` executable is not an automatic command wrapper; invoke it explicitly through mise, for example `mise exec -- mbx --help`.


For Velnor Actions, “latest” means the newest official `vX.Y.Z` generator release with its matching binary artifacts and release manifest published. The repository’s unfiltered latest release can refer to the separate Velnor host binary. Verify the generator release and manifest pair before updating `mise.toml` and `.velnor/release-manifest.json`.

## Upstream constraints and compatibility

The lock can include older transitive releases when upstream packages require an exact version. Current examples are `generic-array 0.14.7` (latest compatible 0.14.9), required exactly by `crypto-common 0.1.7`, and `psm 0.1.24` (latest compatible 0.1.32), required exactly by `pest 2.9.4`. Both arise in Ratatui’s optional `termwiz` dependency path and are not built by the default feature set. Do not add duplicate direct dependencies or patch upstream packages to force a lockfile-only bump. Revisit the holds when the owning upstream requirement changes, and test the feature path before claiming that the newer release is usable.

Crossterm 0.29 can stop reading queued terminal input after one bounded read on edge-triggered readiness. The WIP runtime currently re-arms the terminal when unread bytes remain; keep the Linux PTY flood regression test. Remove the workaround only after the selected Crossterm version reads through `WouldBlock` and the regression passes without it. This is an upstream compatibility constraint, not a reason to hold unrelated dependencies below latest.

Unicode-width upgrades can change terminal cell measurements. Version 0.2.2 changed the width of 493 code points compared with 0.2.0; the earlier source scan found none in its measured text, but that observation applies only to that source revision. Re-run current text and visual checks when changing the width crate. Do not infer current visual acceptance from the historical scan.

## Upgrade and verification

From the repository root:

```sh
mise install
cargo update
cargo tree -d
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run --all-targets
```

For a targeted dependency, inspect `cargo tree -i <crate>` and the upstream manifest to identify a hold. Run the relevant visual tier and `cargo xtask visual check-touched` when rendering, text measurement, input handling, or terminal behavior may change. Review actual/expected/diff output; a dependency update does not authorize changing a visual approval. See [CI](ci.md) for generated workflow and tool-pin details, [development](development.md) for local commands, and [the visual-testing plan](visual-testing/plan.md) for acceptance.

These rules do not make a branch ready. Each build, test, or visual result applies only to the exact revision and exercised scope; remaining project requirements stay open until their own acceptance evidence is recorded.
