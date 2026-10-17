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
| Rust toolchain (`rust-toolchain.toml`) | 1.98.1 | 1.99.1 | 1.99.1 | none; CI uses it since velnor v0.1.9 (see below) |
| `rust-version` (root, xtask) | 1.98 | 1.99 | 1.99 | none; raised with velnor v0.1.9 (2026-10-17) |
| `ratatui` (`0.30`, `crossterm_0_29`) | 0.30.2 | 0.30.2 | 0.30.2 | none |
| `crossterm` (through ratatui) | 0.29.0 | 0.29.0 | 0.29.0 | none |
| `clap` (`4.6`) | 4.6.6 | 4.6.7 | 4.6.7 | none |
| `unicode-width` (`0.2`) | 0.2.0 | 0.2.2 | 0.2.2 | none; tuiscotti held it at `=0.2.0` until PR 11 (raised 2026-10-17, no width change in our text, below) |
| `unicode-segmentation` (`1`) | 1.13.3 | 1.13.3 | 1.13.3 | none |
| `libc` (`0.2`, Unix only) | 0.2.190 | 0.2.190 | 0.2.190 | none; added for the runtime's terminal input re-arm (below) |
| `libtest-mimic` (dev, `0.8.2`) | 0.8.2 | 0.8.2 | 0.8.2 | none |
| `sha2` (dev) | `0.10` / 0.10.9 | `0.11` / 0.11.1 | 0.11.1 | none; tuiscotti held it at `=0.11.0` until PR 11 (raised 2026-10-17) |
| `tuiscotti` (dev, git `rev`) | `3382e57d` | `a1ea27f7` | `a1ea27f7` (`main`, 2026-10-17) | none; `3382e57d..a1ea27f7` is two CI-only commits (#13, #14: velnor-actions 0.1.10 and 0.1.12), no library or API change |
| velnor-actions | 0.1.10 | 0.1.12 | 0.1.12 (`f1b044a`, 2026-10-17) | none; adopted 2026-10-17 (`docs/ci.md`); 0.1.11/0.1.12 change only the mr-boxington preflight and add `[workflow] mise_quiet`, our generated output differs only in version strings and digests |
| cargo-nextest in CI | 0.9.148 | 0.9.148 | 0.9.148 (2026-10-17) | none; `[tools]` can override it |
| actionlint, shellcheck, zizmor, gh in CI | 1.7.12, 0.11.0, 1.30.1, 2.102.0 | same | 1.7.12, 0.11.0, 1.31.1, 2.102.0 | zizmor 1.30.1 is still velnor 0.1.12's catalog pin; `[tools] zizmor = "1.31.1"` would raise it, held: 0.1.12 still only shape-checks `[tools]` versions (`zizmor = "99.99.99"` generates, checked 2026-10-17), so the pin needs a velnor fix or a CI run to prove it installs (`docs/remaining-work.md` U3). The other three are on latest (2026-10-17) |

`xtask` has no dependencies. `cargo update` moved 42 transitive packages
(semver-compatible). Transitive crates still behind latest, and why:

| Crate | Locked | Latest | Held by |
|---|---|---|---|
| `generic-array` | 0.14.7 | 0.14.9 | `crypto-common 0.1.7` pins `=0.14.7` (optional `termwiz` path of ratatui, not built) |
| `psm` | 0.1.24 | 0.1.32 | `pest 2.9.4` pins `=0.1.24` (same optional `termwiz` path, not built) |

`crossterm` 0.29 reads the terminal on an edge-triggered readiness (mio)
and stops after the first read that yields an event, so input queued past
one 1 KiB read stays in the kernel until more arrives (on Linux the
`holla_pty` flood never saw its quit). The runtime does not patch it:
`runtime::poll_input` checks the terminal with a level-triggered `poll` and,
when bytes are queued that crossterm has not seen, re-applies the
terminal's own attributes (`tcsetattr`, `TCSANOW`), which wakes the reader.
Drop the re-arm once crossterm reads until `WouldBlock`.

tuiscotti held `unicode-width`, `insta`, `serde_json`, `sha2`, `ratatui`
and `crossterm` at exact versions until PR 11 (feedback P2,
`docs/upstream/tuiscotti-feedback.md`). Since the `3382e57d` pin (PR 11) it uses
ranges, so the lockfile has one `ratatui` (0.30.2), one `crossterm`
(0.29.0) and one `unicode-width`. `cargo update` on 2026-10-17 then moved
`unicode-width` 0.2.0 → 0.2.2, `sha2` 0.11.0 → 0.11.1, `insta` 1.48.0 →
1.49.0 and `serde_json` 1.0.151 → 1.0.154.

Only `unicode-width` reaches the shipped binaries. 0.2.2 changes the width
of 493 code points against 0.2.0. None of them occurs in `src/`, `tests/`
or `snapshots/`, and no line there measures differently (checked by
comparing both versions' `width` and `width_cjk` over every code point and
every line), so the bump changes no rendered width. We never vendor or
patch upstream crates to get around a pin.

## Tools via mise

`mise.toml` (committed, repository root) pins every developer tool to an
exact version: Rust (with clippy and rustfmt), cargo-nextest, actionlint,
shellcheck, zizmor, gh, velnor-actions (`github:tailrocks/velnor-new`, the
release in `.velnor/release-manifest.json`) and the tuiscotti CLI
(`cargo:` git backend at the `Cargo.toml` tuiscotti `rev`).

- Install: `mise trust` once per checkout, then `mise install`. Inside the
  repository mise exports `RUSTUP_TOOLCHAIN` from the `rust` pin, which
  overrides any global `rust = "latest"`, so plain `cargo` builds with the
  project toolchain.
- `rust` is listed last on purpose: mise puts tool bin directories on
  `PATH` in config order, and Rust's is `~/.cargo/bin`, where an older
  `cargo install`ed cargo-nextest, velnor-actions or tuiscotti would
  otherwise shadow the pinned ones.
- Upgrade: bump the pins to the latest releases in the monthly upgrade
  above (`mise latest <tool>`; tuiscotti follows its `Cargo.toml` rev,
  velnor-actions its manifest). Keep `mise.toml` rust ==
  `rust-toolchain.toml` channel == CI; the xtask test
  `mise_pins_match_the_repository_pins` fails when Rust, tuiscotti or
  velnor-actions drift. The other tools match the versions CI installs
  (`docs/ci.md`); zizmor stays at CI's 1.30.1 (1.31.1 is out) until the
  velnor `[tools]` raise above lands.
- `mise.lock` (committed) locks every pin with `linux-x64` and
  `macos-arm64` checksums for velnor's install audit (`docs/ci.md` "Tool
  lock"). After any pin change run `mise lock --platform
  linux-x64,macos-arm64` and commit the lock with the pin; the xtask test
  `mise_lock_covers_every_mise_pin` fails when a pin is missing from the
  lock, locked at another version, or lacks the CI platform.
- A `minimum_release_age` in someone's mise settings hides young releases
  from `latest` (`mise ls-remote rust` warns "newer rust releases hidden"),
  but not from an exact pin: on 2026-10-17 the `1.99.1` pin resolved while
  `mise latest rust` still said 1.99.0.

## Rust in CI

Since velnor-actions 0.1.9, CI installs the toolchain from
`rust-toolchain.toml` (`rust@1.99.1` with clippy and rustfmt) and exports
no `RUSTUP_TOOLCHAIN`, so local and CI builds use the same compiler and
`rust-version` is `1.99` in both crates. 0.1.8 hardcoded Rust 1.98.1 and
forced it with `RUSTUP_TOOLCHAIN`; that and the second clippy run under
1.98.1 are gone. Keep the channel an exact `X.Y.Z`: velnor 0.1.9 refuses
`stable` or `1.99` (`invalid_rust_toolchain`). When the channel moves,
raise `rust-version` with it and regenerate CI (`docs/ci.md`).

## Visual impact

None. The upgrade changed no rendering: `tests/showcase_baseline.txt`, the
smoke tier and the approved tuiscotti pilot pass unchanged, and no snapshot
was re-approved. The 2026-10-17 `cargo update` after tuiscotti PR 11 changed
no rendering either: `cargo xtask visual check-touched` ran every
re-approved group of all four apps (6,200 captures) green.
