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
- Generated actionlint command: passed with the two ignores Velnor generates for `background` and `wait-all`. GitHub's [workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) documents both keywords and states that `wait-all` fails when a background step fails unless `continue-on-error` is set. Default actionlint 1.7.12 does not recognize these GitHub-supported keys; unfiltered validation reports 211 `background` parser messages and 43 missing-`run` messages for `wait-all` steps.
- All 43 current missing-`run` matches are the generated `Wait for parallel checks` steps. None has `if` or `continue-on-error`, so GitHub's documented failure propagation remains active. The generated missing-`run` ignore matches the whole actionlint diagnostic text and could suppress a future unrelated missing-`run` error; retain this as a known limitation until actionlint supports the GitHub syntax, then remove the ignore.
- With the pinned `shellcheck` 0.11.0 available, actionlint debug output confirmed 243 generated `run:` scripts were checked in Bash mode; no shellcheck diagnostics were reported. Actionlint's upstream [ShellCheck integration](https://github.com/rhysd/actionlint/blob/v1.7.12/docs/checks.md#shellcheck-integration-for-run) chooses the shell from workflow configuration and invokes ShellCheck for `bash` or `sh` scripts.
- GitHub's branch-protection endpoint reports main as unprotected. Repository delivery still follows the requested narrow CI-only branch/PR process; this audit does not authorize direct pushes to main.

## PR #39 run and unresolved gates

- PR: [#39](https://github.com/tailrocks/terminal-components-claude/pull/39), head `b293232dc17606b9cd99a5ab008b0c6dd980fd0b`.
- Run [`39423610357`](https://github.com/tailrocks/terminal-components-claude/actions/runs/39423610357) completed `Plan` (4m47s) and `Actionlint` (3m19s) successfully. At the last status query, `Rust / holla` had passed (4m26s) and the other 42 generated Rust jobs were pending; the run had no conclusion. No pending job is counted as passing.
- The PR `DCO` status check is failing with the CNCF DCO app as its provider ([check](https://github.com/cncf/dco2)). The app summary states that all commits are incorrectly signed off and did not pass; the three commits in this branch stack (`e9088f3`, `5ef0e21`, `b293232`) lack `Signed-off-by` trailers. This is recorded as `ACTION_REQUIRED`; it is not the repository's required merge status context, and this audit does not sign or rewrite commits.
- Main source commit `a97e351df736b576904bc450619ca347ea64d65b` has no root `.config/nextest.toml`, so the complete-tree plan correctly selects Nextest's default profile. The fixed parity/design commits `8681124d2cbbc75ef8ed998b5918883039a022c5`, `8533344520cfdf12734c1c149fc89ec946b3ea46`, and design review commit `f4867e9b5f8a08a3bb61ce80d644198a29f51984` all track the root config at blob `e74c1b213d28a5f864b1ffed690ba3ebcc67cb31`, with `ci` plus `smoke`, `full`, `narrow`, and `capture` profiles. Complete-input regeneration must include that file for those branches. The main plan's default profile is not evidence of a sparse-checkout omission.
- The generated plan still contains 39 `doctest/default` tasks whose command is `cargo test --doc`, despite the repository's hard rule to use `cargo nextest`. This is present in the pre-upgrade main task plan too. Pinned Velnor 0.1.29 unconditionally emits doctest tasks and provides no config switch to disable or replace them; excluding crates would also remove their other checks. This remains an upstream generator blocker and is not counted as a passing Rust gate.

The generated workflow's Rust checks have been planned, not fully executed remotely by this change. A successful Plan or Actionlint job does not validate Rust, and queued, skipped, or missing jobs are not a pass.

## Plan-job timeout follow-up

- PR head `d1ecbfdfbe117ffad061f6db896c0a3c98b3c11d` run [`39424952743`](https://github.com/tailrocks/terminal-components-claude/actions/runs/39424952743) cancelled. Actionlint passed, Plan was cancelled, all 43 Rust jobs were skipped, and Required failed. The Plan check annotation [`118635929718`](https://github.com/tailrocks/terminal-components-claude/check-runs/118635929718) says `The job has exceeded the maximum execution time of 5m0s`; its second annotation says the operation was canceled. The Plan job ran from 12:49:17 to 12:54:26 UTC. Checkout consumed 4m16s, Prepare pinned tools 13s, Check generated files 17s, and the Plan command was canceled after 7s.
- Pinned generator source [commit `e7b2f088715379c22be473fab896a43d50cc16fc`](https://github.com/tailrocks/velnor-new/tree/e7b2f088715379c22be473fab896a43d50cc16fc): `crates/velnor-actions-contract/src/config/workflow.rs` declares only `workflow.crate_timeout_minutes` and explicitly says control jobs (including Plan) keep the built-in default; the config uses `deny_unknown_fields`. `crates/velnor-actions-contract/src/workflow/timeout.rs` defines `JobTimeout::PLAN` as `Self::DEFAULT`. `crates/velnor-actions-orchestrator/src/workflow_jobs.rs` assigns `JobTimeout::PLAN` to the generated Plan job. Therefore Velnor 0.1.29 has no supported config-only Plan timeout override. No unsupported config key or generated workflow edit was introduced; this requires an upstream Velnor capability or a runner-side checkout-time fix.
- The PR's protected status context is `Required`; DCO is separately visible as `ACTION_REQUIRED` and is not treated as a required merge gate in this audit.

## Supported crate-job budget adjustment

- In response to the 23/43 crate jobs that exceeded their five-minute GitHub job budget on run `39429081515`, `.velnor/config.toml` now sets `workflow.crate_timeout_minutes = 15`. This is the pinned Velnor 0.1.29 config field (bounded 1..=360) and applies to generated Rust crate jobs. It does not change Plan or other control jobs; Plan remains at the built-in five minutes.
- `mise exec -- velnor-actions generate --output-dir /private/tmp/velnor-main-crate-timeout-preview` produced a preview identical to the checked-in `.github` after `mise exec -- velnor-actions generate`. The generated task manifest remains byte-identical to the previous commit (`85a2d313bc92c59c5880e98b6d3711c694c647447210aa517c678ea7efbc3d3b`); no tasks or crate discovery were removed. Only `.velnor/config.toml` and generated `.github/workflows/ci.yml` changed.
- Pinned Velnor plan (`velnor-actions 0.1.29`) passes and reports 46 discovered workspace crates, 43 execution jobs, and 15 minutes per crate job. The generated workflow has 43 crate jobs with `timeout-minutes: 15`; the four remaining five-minute timeouts are control jobs. The generated Actionlint command passed, and `git diff --check` passed.
- This is configuration-level validation only. The new remote run must execute before treating the longer budget as effective or the required CI gate as green. The Plan five-minute timeout and the existing generated doctest command issue remain unresolved and are not implied fixed by this change.
