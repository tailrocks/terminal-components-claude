# Velnor crate timeout on parity branch — 2026-10-20

## Decision

The parity config now sets `[workflow].crate_timeout_minutes = 15`, matching the reviewed Velnor 0.1.29 CI-upgrade config on the main-delivery PR. Velnor 0.1.29 supports this setting; it controls generated Rust capacity-phase and OpenTofu root jobs, whose default is five minutes. The regenerated parity workflow has 44 Rust jobs at 15 minutes.

This is a preventative increase for crate jobs, not a fix for Plan. Velnor 0.1.29 hard-codes the Plan job timeout at five minutes and has no Plan timeout override. The latest observed parity Plan conclusion is `cancelled`, so the evidence does not establish whether the cancellation was an enforced timeout or another cancellation source.

## Remote evidence

- Parity head inspected: `a5766d24783c2b268e727497fe6861b6479c71cc` (`parity/termrock-visual-baseline`). Run [39429669130](https://github.com/tailrocks/terminal-components-claude/actions/runs/39429669130): Actionlint succeeded; Plan was cancelled after 5m09s while `Prepare pinned tools`/`Acquire Velnor` was active; all Rust jobs were skipped; plan artifact upload then failed because the canceled plan produced no file, and Required verdict failed. The job log says `The operation was canceled`; it does not say `timed_out`.
- A prior parity head `a848219fd858ed6352fcb403b2cc5d5953083e6f` produced run [39423500058](https://github.com/tailrocks/terminal-components-claude/actions/runs/39423500058). This was a distinct Plan failure: its Velnor 0.1.16 `Check generated files` step reported differences in `.github/velnor/task-execution-manifest-v1.json` and `.github/workflows/ci.yml`, then exited 1. Its missing plan artifact and skipped Rust jobs were downstream consequences. This is not evidence for a timeout.
- Main-delivery CI upgrade source `a6929aeba7479d0b3a746f69d412a1ab701b5242` already sets `crate_timeout_minutes = 15`. Run [39433665508](https://github.com/tailrocks/terminal-components-claude/actions/runs/39433665508) attempt 1 had Actionlint succeed and Plan cancel after checkout took 4m33s and the job reached Velnor acquisition at the five-minute window; 43 Rust jobs were skipped. At the latest checkpoint, attempt 2's Plan was in progress. The job conclusion and log do not prove a timeout cause; the run was labeled canceled.
- Design run [39435382085](https://github.com/tailrocks/terminal-components-claude/actions/runs/39435382085) had Plan and Actionlint pass, with two Rust jobs in progress at the latest status checkpoint. This records its status only and does not establish completed Rust validation.

## Upstream contract

The pinned generator is Velnor Actions 0.1.29, source commit [`e7b2f088715379c22be473fab896a43d50cc16fc`](https://github.com/tailrocks/velnor-new/commit/e7b2f088715379c22be473fab896a43d50cc16fc). At that commit, `crates/velnor-actions-contract/src/config/workflow.rs` defines `workflow.crate_timeout_minutes`, documents its five-minute default and Rust/OpenTofu scope, and validates 1..=360. `crates/velnor-actions-contract/src/workflow/timeout.rs` defines `JobTimeout::PLAN` as the built-in default; the Plan job has no corresponding config field. Thus the setting cannot change the Plan timeout.

## Reproduction on parity source

Source head before this increment: `a5766d24783c2b268e727497fe6861b6479c71cc`.

- `mise exec -- velnor-actions --version`: `velnor-actions 0.1.29`.
- `mise exec -- velnor-actions generate`: succeeded; changed only `.github/workflows/ci.yml` among generated files. The task execution manifest was unchanged.
- `mise exec -- velnor-actions generate --output-dir /private/tmp/termrock-cratetimeout-preview-20261020` followed by `diff -r --brief .github /private/tmp/termrock-cratetimeout-preview-20261020/.github`: exact tree match.
- `mise exec -- velnor-actions plan`: succeeded; 47 workspace crates, 44 Rust execution jobs, 259 task IDs.
- CI-equivalent Actionlint command passed: `mise exec -- actionlint -color -ignore '^unexpected key "background" for step to (run shell command|execute action)\. ' -ignore '^step must run script with "run" section or run action with "uses" section$'`. Plain `mise exec -- actionlint` exits 1 because Actionlint 1.7.12 does not understand Velnor's generated `background` extension; the CI command carries these two established ignores.
- Generated workflow check: 44 `timeout-minutes: 15` entries; Plan remains `timeout-minutes: 5`. `git diff --check` passed.

No Rust jobs ran in the observed remote parity run because Plan did not complete. The 15-minute Rust budget is therefore generated/configuration evidence only until a remote CI run completes the plan and affected crate jobs.
