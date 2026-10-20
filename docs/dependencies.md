# Dependency pins

## Velnor Actions

PR #38 pins the latest released generator, `velnor-actions` 0.1.30, in `mise.toml` and `mise.lock`. The release was published 2026-10-20 from `tailrocks/velnor-new` commit `a6b1d7873151dc62711844158490e8687d191943`. Its official `.velnor/release-manifest.json` SHA-256 is `81f24072a16f35f30845fd373b114f7c615e9119ee434fe39dab83de84fc56ea`. The macOS ARM64 generator SHA-256 is `7cb74f0a82edbc9545d5d2170387a21dab33c84e8674610fd1e46dad2ca2ad3c`.

The migration and generated-workflow audit is in [the 0.1.30 audit](audits/2026-10-21-velnor-0.1.30-upgrade.md). Other dependency pins were not audited by that change.
