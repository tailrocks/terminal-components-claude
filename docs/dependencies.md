# Dependencies

Policy (AGENTS.md): keep every dependency, the Rust toolchain, tuiscotti and
velnor-actions on their latest releases, and re-run this upgrade at least
monthly. A version held below latest needs a recorded blocker here.

## How to upgrade

1. Latest versions: `curl -s https://crates.io/api/v1/crates/<name>`
   (`max_stable_version`), the stable channel manifest
   (`https://static.rust-lang.org/dist/channel-rust-stable.toml`),
   `git ls-remote https://github.com/tailrocks/tuiscotti main` and
   `gh release list -R tailrocks/velnor-new`.
2. Raise the requirements in `Cargo.toml` (majors included), the channel in
   `rust-toolchain.toml`, the tuiscotti `rev =`, and the velnor manifest
   (`docs/ci.md`), then `cargo update`.
3. Find what is held: `cargo update --verbose --dry-run` lists every crate
   "behind latest"; `cargo update -p <crate> --precise <latest> --dry-run`
   names the requirement that holds it.
4. Gate as in AGENTS.md, plus the smoke tier and approved pilot
   (`docs/visual-testing/`) and `cargo xtask visual check-touched`; a
   rendering change from a dependency is re-approved in its own commit that
   names the dependency.

## Versions (2026-10-16)

Direct dependencies and tools. "Locked" is the `Cargo.lock` version.

| Item | Before | After | Latest | Blocker |
|---|---|---|---|---|
| Rust toolchain (`rust-toolchain.toml`) | 1.98.1 | 1.99.1 | 1.99.1 | none locally; CI runs 1.98.1, see below |
| `rust-version` (root, xtask) | 1.98 | 1.98 | 1.99 | kept at the CI toolchain, see below |
| `ratatui` (`0.30`, `crossterm_0_29`) | 0.30.2 | 0.30.2 | 0.30.2 | none |
| `crossterm` (through ratatui) | 0.29.0 | 0.29.0 | 0.29.0 | none |
| `clap` (`4.6`) | 4.6.6 | 4.6.7 | 4.6.7 | none |
| `unicode-width` (`0.2`) | 0.2.0 | 0.2.0 | 0.2.2 | tuiscotti `Cargo.toml:32` `=0.2.0` |
| `unicode-segmentation` (`1`) | 1.13.3 | 1.13.3 | 1.13.3 | none |
| `libtest-mimic` (dev, `0.8.2`) | 0.8.2 | 0.8.2 | 0.8.2 | none |
| `sha2` (dev) | `0.10` / 0.10.9 | `0.11` / 0.11.0 | 0.11.1 | tuiscotti `Cargo.toml:30` `=0.11.0` |
| `tuiscotti` (dev, git `rev`) | `a47c9aae` | `a47c9aae` | `a47c9aae` (`main`) | none |
| velnor-actions | 0.1.8 | 0.1.8 | 0.1.8 | none |
| cargo-nextest in CI | 0.9.148 | 0.9.148 | 0.9.148 | pinned by velnor |
| actionlint, shellcheck, zizmor, gh in CI | 1.7.12, 0.11.0, 1.30.1, 2.102.0 | same | n/a | pinned by velnor |

`xtask` has no dependencies. `cargo update` moved 42 transitive packages
(semver-compatible). Transitive crates still behind latest, and why:

| Crate | Locked | Latest | Held by |
|---|---|---|---|
| `insta` | 1.48.0 | 1.49.0 | tuiscotti `Cargo.toml:27` `=1.48.0` |
| `serde_json` | 1.0.151 | 1.0.154 | tuiscotti `Cargo.toml:29` `=1.0.151` |
| `ratatui` (second copy) | 0.29.0 | 0.30.2 | tuiscotti `Cargo.toml:31` `=0.29.0` |
| `crossterm` (second copy) | 0.28.1 | 0.29.0 | tuiscotti `Cargo.toml:22` `=0.28.1` |
| `generic-array` | 0.14.7 | 0.14.9 | `crypto-common 0.1.7` pins `=0.14.7` (optional `termwiz` path of ratatui, not built) |
| `psm` | 0.1.24 | 0.1.32 | `pest 2.9.4` pins `=0.1.24` (same optional `termwiz` path, not built) |

The tuiscotti pins are tracked upstream in
`docs/upstream/tuiscotti-feedback.md` P2. Only `unicode-width` reaches the
shipped binaries: the lockfile holds one 0.2.x, so the library measures text
with the 0.2.0 tables. We never vendor or patch upstream crates to get
around a pin.

## Rust in CI

velnor-actions 0.1.8 hardcodes Rust 1.98.1
(`crates/velnor-actions-mise/src/catalog.rs:50`, `RUST_VERSION`) and sets
`RUSTUP_TOOLCHAIN: 1.98.1` on every job, so CI ignores
`rust-toolchain.toml` and a repository `mise.toml` (checked with
`velnor-actions generate --output-dir`). Local work uses 1.99.1;
`rust-version` stays `1.98` so CI builds, Cargo's MSRV-aware resolver keeps
the lockfile buildable on 1.98, and clippy's `incompatible_msrv` flags std
APIs newer than CI. Mirror CI before pushing with
`RUSTUP_TOOLCHAIN=1.98.1 cargo clippy --workspace --all-targets -- -D warnings`.
Raise `rust-version` when a velnor release pins a newer Rust.

## Visual impact

None. The upgrade changed no rendering: `tests/showcase_baseline.txt`, the
smoke tier and the approved tuiscotti pilot pass unchanged, and no snapshot
was re-approved.
