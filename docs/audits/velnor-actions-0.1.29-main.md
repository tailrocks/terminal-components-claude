# Velnor Actions 0.1.29 main-branch workflow upgrade

This audit covers the main-only workflow upgrade from `a97e351df736b576904bc450619ca347ea64d65b` to `e9088f32db8314e5e319b346c736fc133c7fa68f` on `ci/velnor-0.1.29-main-20261020`.

## Pinned release

- Stable release: [Velnor Actions v0.1.29](https://github.com/tailrocks/velnor-new/releases/tag/v0.1.29), published 2026-10-20.
- Release source/tag commit: `e7b2f088715379c22be473fab896a43d50cc16fc`.
- Current upstream `main` at inspection: `5d3f15953aee66532196c05d949a4c985ad1b555`; it is newer than the release and was not used.
- Linux x86_64 asset SHA-256: `ffb8c82a10ae55344225cb77f8a3f68c2ccd42068cff5959254bfed5d7780b74`.
- macOS ARM64 asset SHA-256: `7c4e0efeab89e05928bbd9a375d7b594fd9236caef2eac7b4ad241b3a92dd1d3`.
- macOS x86_64 asset SHA-256: `d60f758b1c7a063f8dda72c81973cddb2d1369536e1931996fe74b418a20e1e6`.

The release manifest was fetched from the [v0.1.29 release asset](https://github.com/tailrocks/velnor-new/releases/download/v0.1.29/release-manifest.json). `mise lock github:tailrocks/velnor-new@0.1.29` refreshed the configured lock entries. Velnor's `verify-release-manifest` passed with the downloaded Linux x86_64 and macOS ARM64 binaries.

## Generated configuration

`mise.toml` and `mise.lock` pin 0.1.29. `.velnor/release-manifest.json` contains the release source and three published targets. `.velnor/config.toml` opts into `[agents] claude_md = "symlink"`, supported by the release's `ClaudeMdMode::Symlink` contract. Velnor generated `.github/CLAUDE.md` as the relative symlink `AGENTS.md`.

The exact preview and repository `.github` trees compared with no differences after generation. No generated file was edited by hand.

## Validation

- `mise exec -- velnor-actions --version`: `velnor-actions 0.1.29`.
- `mise exec -- velnor-actions generate`: passed.
- `mise exec -- velnor-actions plan`: passed; selected the Rust stack and detected 43 Rust crate jobs.
- `git diff --check`: passed.
- Generated actionlint command: passed with the two narrow ignores Velnor generates for its `background` and `wait-all` step extensions. Default `actionlint -color` returns 1 on these extensions (254 parser messages); this does not reflect a GitHub Actions syntax failure because Velnor's runtime consumes those step keys.
- With the pinned `shellcheck` 0.11.0 available, actionlint debug output confirmed 243 generated `run:` scripts were checked in Bash mode; no shellcheck diagnostics were reported.
- GitHub's branch-protection endpoint reports main as unprotected. Repository delivery still follows the requested narrow CI-only branch/PR process; this audit does not authorize direct pushes to main.

The generated workflow's Rust checks have been planned, not executed remotely by this change. CI must provide the actual validation result; skipped jobs are not a pass.
