# Diagnostic Cargo index-resolution attempt (R3)

This is one Root-only, review-gated Cargo command. The exact operation is:

```text
/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo update --workspace --dry-run --locked --manifest-path /private/tmp/termrock-vis11-diagnostic-candidate-e0bc-20261010/Cargo.toml
```

R2 metadata-only execution exited successfully but returned 45 packages with `resolve=null`; the reqwest sparse-index entry remained missing and the cache had zero changed paths. R3 preserves that receipt and reuses the same current isolated Cargo home. It does not copy or mutate the immutable donor. Success would not prove that a hypothetical non-dry-run update would satisfy the lockfile writer check.

Cargo's published command documentation describes `--workspace` scope, `--dry-run` write suppression, and `--locked` behavior. The implementation source narrows the combined semantics: `update_lockfile` resolves from the previous lock, then the dry-run branch warns `not updating lockfile due to dry run` and skips `write_pkg_lockfile`; the `--locked` writer check therefore is not claimed to validate a hypothetical updated graph on this path. The runner enforces that the actual Cargo.lock bytes remain unchanged. The local Cargo executable is independently pinned by SHA-256 in the plan. References: https://doc.rust-lang.org/cargo/commands/cargo-update.html; https://doc.rust-lang.org/stable/nightly-rustc/src/cargo/ops/cargo_update.rs.html.

The runner requires the exact dry-run warning, unchanged candidate/index/lock/config, and the reqwest index file. It walks the complete private Cargo home before and after. It permits only sparse index changes and bounded updates to the three already-present Cargo bookkeeping files; all other path deltas fail closed, including crate archives, registry sources, and the cached Git checkout. The Cargo-home root is checked by physical lstat identity/mode before and after.

The command uses an empty inherited environment, private HOME/TMP/XDG/target, the existing isolated CARGO_HOME, explicit `CARGO_NET_OFFLINE=false` for sparse-index resolution, and a pinned Git wrapper that blocks Git network subcommands. It has a 600-second deadline and 8 MiB combined output cap.

This is diagnostic preparation only. No build, test, Nextest, Velnor CLI, generation, release, or qualification is permitted or implied. A successful resolver result does not prove a build or full cache/archive closure.

R3 preflight attempt 01 is preserved under `evidence/preflight-attempt-01/`. It failed before Cargo with a copied checksum-literal mismatch; no Cargo child was spawned and no cache/source/output mutation occurred. Only a later current preflight can authorize Root review.
