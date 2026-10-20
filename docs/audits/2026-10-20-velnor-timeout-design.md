# Velnor Rust job timeout on the design branch — 2026-10-20

## Decision

Set `workflow.crate_timeout_minutes = 15` on `design/interaction-polish`. This
uses the supported Velnor Actions 0.1.29 setting for Rust capacity-phase jobs;
it does not alter the fixed five-minute Plan job. The change raises both
generated Rust jobs on this branch from five to fifteen minutes. It does not
change the suites workflow, the Rust tool pin, Tuiscotti, application sources,
or the frozen product reference.

## Remote evidence

- Design head before this increment: `59af1c4f760387be455bc79d44be75f4416a63f3`.
- Run [39435382085](https://github.com/tailrocks/terminal-components-claude/actions/runs/39435382085): Actionlint and Plan passed. Rust job `junie-tui` (`118672733202`) has the explicit annotation “The job has exceeded the maximum execution time of 5m0s”; later operation-canceled annotations are downstream. This confirms its Rust job used the five-minute limit.
- In the same run, Rust job `xtask` (`118672733171`) failed its Format step. Its annotation reports exit code 1. This remains a separate existing format failure; the timeout change does not fix or suppress it. The Required verdict and Required check failed, and this run does not pass CI.

## Upstream contract and pin

Velnor Actions remains pinned to 0.1.29, source commit
[`e7b2f088715379c22be473fab896a43d50cc16fc`](https://github.com/tailrocks/velnor-new/commit/e7b2f088715379c22be473fab896a43d50cc16fc).
Its [`WorkflowConfig`](https://github.com/tailrocks/velnor-new/blob/e7b2f088715379c22be473fab896a43d50cc16fc/crates/velnor-actions-contract/src/config/workflow.rs)
supports `crate_timeout_minutes`; the pinned [`JobTimeout` contract](https://github.com/tailrocks/velnor-new/blob/e7b2f088715379c22be473fab896a43d50cc16fc/crates/velnor-actions-contract/src/workflow/timeout.rs)
keeps Plan at its default five-minute timeout while allowing Rust crate jobs to
use the configured value.

## Reproduction on the exact source

The Velnor 0.1.29 generator ran against the complete design checkout. It
changed only the generated `.github/workflows/ci.yml` under `.github`; the task
execution manifest remained byte-identical.

- `mise exec -- velnor-actions --version`: `velnor-actions 0.1.29`.
- `mise exec -- velnor-actions generate`: passed.
- `mise exec -- velnor-actions generate --output-dir /private/tmp/termrock-design-timeout-preview-20261020`: the generated `.github` tree matched the checked-in tree, including path types and symlink targets.
- `mise exec -- velnor-actions plan`: passed; two Rust workspace crates and two execution jobs.
- Pinned Actionlint 1.7.12 with the same two generated-step compatibility filters used by CI: passed.
- Generated workflow counts: two Rust jobs at 15 minutes; four control jobs remain at five minutes.
- `git diff --check`: passed. Task manifest, Mise pins, Cargo manifests/lock, and tests are unchanged.

The new budget is configuration evidence until a remote run completes. The
existing Format failure remains unresolved and must be tracked independently.
