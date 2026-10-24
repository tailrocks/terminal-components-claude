# CI planning audit — 2026-10-20

## Fixed source and generator identities

- PR #38 branch `parity/termrock-visual-baseline`: `0b8f4e32b24a137987307600b10d892bec0619c3` at audit start.
- Comparison branch `design/interaction-polish`: `f4867e9b5f8a08a3bb61ce80d644198a29f51984`.
- Frozen baseline tag object: `2718a6b8127a522a3d351d0b4bdb7542fc8da542`; source: `4169783ae883788186d161af09ba9c481bd84e5c`.
- `tuiscotti`: `a1ea27f757d75d07b05499276cd932e9f8e067ad` in root `Cargo.toml` and the standalone frame-support lock.
- `mise.toml`: Rust `1.99.1`, cargo-nextest `0.9.148`, Velnor Actions `0.1.16`; generator release commit `66719d14f87a893b4c8f52f76204fad19f124909`.

## Planning failure and correction

Velnor's recursive discovery found `tools/frozen-cli-frame-support/Cargo.toml`. It is a standalone Cargo workspace (`[workspace]` in that manifest), outside the root workspace, with its own `Cargo.lock`. Velnor's offline preparation then failed because that lock requires `cc 1.9.0`, absent from the root-lock source cache. The helper is invoked by the Python paired-capture runner; it is not a production workspace package. Exclude `tools/frozen-cli-frame-support/**` from package discovery. Its standalone Rust tests then have no generated CI job; run them explicitly with `cargo nextest run --locked --manifest-path tools/frozen-cli-frame-support/Cargo.toml`. Do not pretend a skipped package job tested them.

Evidence:

- PR run at `5d9f5b1772e733dad4940c84fb9a06b3baaae0ce`: [Plan job, run 39407964906](https://github.com/tailrocks/terminal-components-claude/actions/runs/39407964906/job/118578696878), `Check generated files` failed with `preparation_incomplete: tools/frozen-cli-frame-support/Cargo.toml: metadata_offline:nonzero_exit` and the locked `cc 1.9.0` resolution error. The subsequent `Plan` step skipped; generated Rust jobs skipped because they needed the failed plan; `Required` failed merging missing task reports.
- Design run at `f4867e9b5f8a08a3bb61ce80d644198a29f51984`: [Plan job, run 39407426147](https://github.com/tailrocks/terminal-components-claude/actions/runs/39407426147/job/118576646568), same standalone-manifest preparation failure under Velnor `0.1.12`.
- At audit time, PR head `0b8f4e32b24a137987307600b10d892bec0619c3` run [39410920370](https://github.com/tailrocks/terminal-components-claude/actions/runs/39410920370) was still in progress with only `Actionlint` and `Plan` jobs started. DCO check showed `ACTION_REQUIRED`; separate from CI planning and not counted as a Rust result.

After editing `.velnor/config.toml`, regenerate generated output only with the pinned tool: `mise exec -- velnor-actions generate`. Do not hand-edit `.github` files. Plan smoke must complete through `Check generated files` and `Plan` before treating CI as available.

## Coverage limits

After adding the exclusion, `mise exec -- velnor-actions generate` succeeded, generated `.github` matched the preview tree, and `mise exec -- velnor-actions plan` completed. The plan detected 47 workspace packages and scheduled 44 independent crate jobs. Regeneration adds five missing `visual-harness` package tasks to the stale workflow output. PR task selection narrows obligations using the event-time affected-work plan. Generated tasks use per-package `cargo nextest run` for ordinary tests, plus formatting, clippy, executable listing, and documentation. This is not `cargo nextest run --workspace --all-targets`; excluded package tests and skipped jobs are not green. The plan also emits 39 `cargo test --doc` tasks, which conflicts with the repository instruction to use nextest for all Rust test validation. This audit did not run `cargo test`; a separate policy correction remains before claiming the generated CI plan is fully compliant.

The PR config sets `run_ignored = "default"`. Its generated tree contains only `.github/workflows/ci.yml`; the comparison branch has manually dispatched visual suites in `.github/workflows/suites.yml`, but the PR config does not declare suites and the generated `suites.yml` is absent. Therefore normal CI does not run the ignored visual scenarios. The frozen paired runner and its Python tests are present in both branches with byte-identical source (`tools/frozen_baseline.py`, `tools/test_frozen_baseline.py`, `tools/frozen_cli_pair.patch`, and `tools/frozen-cli-frame-support`). However, no generated CI task invokes `python3 tools/frozen_baseline.py` or `tools/test_frozen_baseline.py`; code presence is not execution evidence.

The PR excludes `crates/termrock-e2e/**` and `crates/termrock-visibility-tests/**` from discovery, and also excludes `crates/showcase-data/**`, `crates/holla-catalog/**`, and `crates/holla-plan/**`. These paths have no independent Velnor package gate. No dedicated external consumer crate was found that compiles only against public Termrock APIs with unrelated domain data. The present workflow therefore does not prove the requested external-consumer architecture gate, direct baseline/candidate CLI comparison, or full workspace all-targets execution.

## Branch comparison

The frozen CLI runner/tooling sources listed above are identical between the recorded PR and design commits. CI plans are not identical: the design branch declares visual tiers in `.velnor/config.toml` and generates `suites.yml`; the PR branch removes those declarations and generated workflow. Its latest run had not completed at audit time. Recheck both branches after the config correction and keep any remaining coverage gaps explicit.

## Generated instruction symlink correction

Velnor Actions 0.1.16 supports `[agents] claude_md = "symlink"`. This config now requests that mode, and generated `.github/CLAUDE.md` is the relative symlink `AGENTS.md`, matching the repository's generated-file instruction. `mise exec -- velnor-actions generate` and `mise exec -- velnor-actions plan` succeeded. A second `generate --output-dir /private/tmp/termrock-ci-plan-generated-symlink-20261020` preview exactly matched the committed `.github` tree, including symlink type. The plan still includes 39 `cargo test --doc` tasks, lacks an external-consumer target, and does not invoke the paired Python CLI runner; those are unresolved blockers, not accepted coverage. The upstream capability evidence is Velnor Actions release commit `f5d6692642fa6774048963073dfb5cc33d039abd`, source `crates/velnor-actions-contract/src/config/agents.rs` and CLI test `crates/velnor-actions-cli/tests/impl_cli_suites_and_symlink.rs`.


## Remote validation after the bounded correction

PR commit `2de0d01255390d2a02b7775ee452268782802a38` (tree `1e048111`) triggered run `39415797723`. Actionlint passed, but `Check generated files` failed after generation: only `task-execution-manifest-v1.json` and `workflows/ci.yml` differed. Plan and downstream Rust validation did not run; missing plan-artifact upload was consequential. This remains a CI failure.

An independent Linux reproduction used the same source tree, Velnor 0.1.16 Linux binary SHA-256 `8e7bda8b76d0613224c55bc9a02b95ea3f214b00351bf85cce986204055863d8`, logged Mise flags, matching Cargo/rustup paths, and available Cargo caches. Generation exited successfully and both files matched. The actual differing runner-generated files were not retained, so the cause is not established. No speculative config correction was made. Local reproducibility does not turn the remote failure into passing evidence. The coordinator preserved the remote job log under `termrock-cli-evidence-20261020/ci-integration-gates/remote-plan.log`; required CI execution and the previously listed coverage/policy gaps remain unresolved.
