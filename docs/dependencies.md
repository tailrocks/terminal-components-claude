# Dependency and toolchain policy

## Source of truth

Manifests and their lockfiles define the selected dependencies. This includes the root workspace and separate lockfiles for the PTY suite, visibility CLI tests, and visibility publisher. `rust-toolchain.toml`, `mise.toml`, `mise.lock`, and `.velnor/release-manifest.json` define the toolchain and developer/CI tools. Keep this page as the update policy and index; do not duplicate every transitive lock entry here.

Keep direct dependencies and compatible transitive dependencies on their latest stable releases. Consider major releases too: update the declared requirement, make any needed source/API migration, and run the checks for every affected package and feature. Record an older transitive version only when the exact upstream requirement prevents moving it, with the package and feature path. Do not pin older versions merely to avoid migration work. Commit lockfiles so normal builds remain reproducible. Pin Git dependencies to a reviewed source commit and update to the latest reviewed upstream commit when it moves; do not depend on a floating branch name for builds.

## Current selected versions

These selected pins are recorded by the manifests and lockfiles; the lockfiles remain authoritative where multiple versions are present.

| Component | Selected version |
|---|---:|
| Rust toolchain | 1.99.1 |
| Cargo `rust-version` declaration | 1.99 |
| Ratatui | 0.30.2 |
| Crossterm backend | 0.29.0 |
| Clap | 4.6.7 |
| Unicode width | 0.2.2 |
| Unicode segmentation | 1.13.3 |
| SHA-2 | 0.11.1 |
| Base64 | 0.23.1 |
| JSON Schema (PTY suite) | 0.59.0 |
| Unix libc | 0.2.191 |
| `num_threads` | 0.1.8 |
| Tuiscotti Git source | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |

The repo-local tool pins are cargo-nextest 0.9.148, actionlint 1.7.12, shellcheck 0.11.0, zizmor 1.31.1, GitHub CLI 2.102.0, Velnor Actions 0.1.13, and mr-boxington 1.24.1. Rust is pinned to 1.99.1 with clippy and rustfmt; the repository's declared MSRV is 1.99. `mise.toml` declares mise 2026.10.13 as its minimum. Velnor's target binaries are hash-pinned by `.velnor/release-manifest.json`; `mise.lock` locks the mise tools. Install the declared tools with `mise install`.

The `mise.toml` pin makes the `mbx` executable available, while intentionally disabling mr-boxington's automatic command wrapping. Invoke it explicitly, for example `mise exec -- mbx --help`.


For Velnor Actions, “latest” means the newest official `vX.Y.Z` generator release with its matching binary artifacts and release manifest published. The repository’s unfiltered latest release can refer to the separate Velnor host binary. Verify the generator release and manifest pair before updating `mise.toml` and `.velnor/release-manifest.json`.

## Upstream constraints

The root lock can contain older optional upstream dependencies that default features do not compile. Current examples are `generic-array 0.14.7` (latest compatible 0.14.9), required exactly by `crypto-common 0.1.7`, and `psm 0.1.24` (latest compatible 0.1.32), required exactly by `pest 2.9.4`; both occur in Ratatui’s optional `termwiz` path. Keep those lock entries while the exact constraints remain. Do not add duplicate direct dependencies or patch upstream packages solely to force these transitive versions. Revisit them when the owning upstream requirement changes and validate the optional feature path before reporting an upgrade.

The `base64 0.22.1` lock entry is retained for an upstream `termwiz` consumer while the current first-party package requirements use 0.23.1. Multiple locked versions are valid when independent semver requirements require them; inspect `cargo tree -d` before trying to unify them.

## Frozen historical tooling

Two auxiliary tool inputs intentionally retain older versions for exact historical reproduction. `tools/historical-render/pins.json` requires Pillow 11.3.0 and binds 499 generated HTML/PNG outputs to archived source and capture hashes. `tools/qualified-capture/pins.json` binds its independent harness to Rust 1.98.1, exact tool commits, and payload hashes. These are immutable historical qualification inputs, not the current project toolchain. Changing a pin requires a separate requalification with new evidence; preserve the original attestations and hashes.

## Upgrade and verification

From the repository root, install pinned tools, update Cargo locks, inspect duplicate/held dependencies, and run the affected validation:

```sh
mise install
cargo update
cargo tree -d
cargo fmt --all -- --check
cargo check --workspace --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
```

Run checks for separate lockfile packages when their dependencies change. When UI rendering, text measurement, capture, or terminal behavior can change, inspect current actual/expected/diff outputs and run the relevant visual checks. A dependency update does not authorize changing an approval or a historical baseline. The [development guide](development.md) has setup and run commands; the [design reference](design.md) and [plan](plan.md) define product acceptance. Successful dependency installation or compilation alone does not establish product readiness.
