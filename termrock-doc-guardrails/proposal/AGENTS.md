# Termrock agent rules

## Read first

Read the current task scope in `GOAL.md`.
Read [component composition](docs/architecture/component-composition.md).
Read [commit review](docs/process/per-commit-review.md).
Read [verification](docs/verification/ownership-and-parity.md).
Use the canonical component contract before changing that component.

## Project boundary

Work only on `termrock-implementation` for this task.
Do not reset the branch or create another implementation branch.
Do not create another Termrock repository.

Termrock is the reusable component library.
Showcase, Jackin Preview, Holla, and TablePro demonstrate that library.
The previews must retain their baseline appearance and observable interactions.
They are not separate product redesign projects.

The frozen visual authority is `visual-baseline` at
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
Never move, delete, recreate, or retarget that tag or its release.
Use `termrock-refactor` as planning history.
Use `main` only as structural evidence, never as expected visual output.

## Component ownership

Build preview interfaces only from reusable Termrock components and public layout helpers.
Supply domain data, component state, and typed application actions from preview code.
Keep generic rendering and interaction in the owning library component.

Do not paint a widget or screen directly in preview code.
Do not call low-level painting through `termrock::author`, aliases, wrappers, or generated files.
Do not render a component and then paint over its result.
Do not move an app-specific painter into a Termrock crate to disguise it.

Do not use stored frames, padded screen strings, or fixture-specific painters as application output.
Do not select a different renderer for a test size or scenario ID.
Paused mode may control time. It must not bypass input, layers, or component behavior.

If the library lacks a required capability, extend its component contract first.
Do not solve that gap with a local painter.
Advanced customization must retain the stock component's geometry and interaction ownership.

## Reviews

Use an independent reviewer before each work item starts.
Use an independent reviewer for every proposed commit, including documentation and generated changes.
The reviewer must not author the submitted change.

Freeze the staged diff before review.
Record the parent commit, proposed tree, and diff digest.
Review all changed lines and relevant callers.
Resolve blocking findings before committing.
Any changed staged content requires another review.

After the commit, verify that its tree and parent match the approved subject.
Bind the review to the actual commit SHA in an external receipt or review record.
Do not claim approval from a checklist, test name, or self-written status.
Review the whole work item after integration.

Keep commits small and frequent.
Do not bypass review to push faster.
Do not claim an unavailable reviewer ran.

## Verification and truth

Visual parity and component ownership are separate mandatory gates.
A matching image cannot excuse a bypassed component.
An imported component name cannot prove who painted the screen.

Use `cargo nextest` to execute Rust tests. Do not use `cargo test`.
Build, format, lint, and document checks remain separate tool operations.
Select required ignored visual tests explicitly.
Never run an approval writer as an ordinary verification test.

Use synthetic test data. Do not capture real credentials.
Keep expected snapshots read-only during candidate verification.
Do not change expected output, tests, or thresholds to hide a regression.
Report not-run, failed, skipped, and passed checks separately.

## Current documentation-only task

The current task may change only its approved documentation paths.
Do not modify source, tests, workflows, dependencies, or baseline artifacts.
Write future enforcement requirements as pending work.
Do not claim that these instructions install hooks or CI checks.

## Repository conventions

Use this exact commit sign-off:
`Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`.
Preserve unrelated work and global Git settings.
Keep each `CLAUDE.md` as a symlink to its adjacent `AGENTS.md`.
Do not write a separate `CLAUDE.md` body.
