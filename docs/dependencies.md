# Dependency pins

## Velnor Actions

PR #38 currently pins Velnor Actions `velnor-actions` 0.1.36 in `mise.toml` and `mise.lock`. GitHub published the release at `2026-10-21T09:04:54Z` from `tailrocks/velnor-new` commit `f4078347c4316ac0b41b894dca6ee9c2b0decc11`. The official `.velnor/release-manifest.json` SHA-256 is `b5374c49b700b09a77f2e99bedf74d7314bd59ff18b3383ac5ffa7b6045799b8`; the macOS ARM64 generator SHA-256 is `88151c24f09713b1844f5a64e1c58aaafbd9844eb03b78657984a332cb6d64c8`.

The current pin and generated-workflow audit is [Velnor Actions 0.1.36](audits/2026-10-21-velnor-0.1.36-upgrade.md). It also records the `gh` 2.102.1 pin update and the remaining Rust checksum warning. The [0.1.32 audit](audits/2026-10-21-velnor-0.1.32-upgrade.md) and earlier audits remain historical. Other dependency pins were not audited by these changes.
