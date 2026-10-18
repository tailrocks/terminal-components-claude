# Execution Model and Engineering Rules

## Active Visibility Prerequisite

Before each task:

- Read [WORK_QUEUE.md](WORK_QUEUE.md) and the accepted record in [tasks.json](docs/implementation/visibility/tasks.json).
- If [STATUS.md](STATUS.md) exists, read it. If it is missing, the VIS-01 owner must use the queue and accepted record to publish an honest initial STATUS.md as the first visibility work. Read it before starting the next task.
- Before STATUS.md exists, use the accepted record and its source evidence to identify candidate and reference commits. Compare them with current branch heads.
- After STATUS.md exists, compare its recorded source commits with current branch heads. Review changes to measured sources since their recorded commits.
- Check evidence freshness before you rely on a result. Record the source commit used for each result separately from the commit that publishes the report.
- Read assigned contracts and scoped AGENTS.md files before editing.
- Confirm that the task remains open and that its priority, claim token, owner, reviewer, base commit, and allowed paths match the accepted record.
- If any field differs, ask the integrator to resolve it before editing.
- If automatic instruction discovery does not load an applicable scoped AGENTS.md, the coordinator must give its full path to the owner and reviewer.
- Keep broad product refactoring on hold until the visibility acceptance checks pass. Continue accepted visibility work.

These instructions and records are not enforced automatically. `STATUS.md` reports current state. The accepted task records define scope. `WORK_QUEUE.md` lists task priorities. The public API, component ownership, checklist, review, and verification requirements remain in force. Never move, delete, recreate, or retarget the frozen `visual-baseline` tag or release. Do not reset or rebase `termrock-implementation`, merge `main` into it, force-push it, or rewrite its history. Product-source changes belong only on `termrock-implementation`. The only branch exception is test, report, and CI updates on `origin/visual-baseline`; do not change product source or the frozen tag or release there. Use `origin/visual-baseline` as a test and scenario reference only. Do not use captures from that branch as expected visual output. `main` is structural evidence only, never expected visual output. Run Rust tests with `cargo nextest`; do not use `cargo test`.

## Read First

After reading STATUS.md if it exists, WORK_QUEUE.md, and the accepted task record, read the active task in [GOAL.md](GOAL.md).
Read the [component composition contract](docs/architecture/component-composition.md).
Read the [per-commit review procedure](docs/process/per-commit-review.md).
Read the [ownership and parity contract](docs/verification/ownership-and-parity.md).
Read the canonical component contract before changing that component.
Follow the [document map](docs/process/document-map.md) when two documents overlap.

## Project Boundary and Authorities

Keep product changes on `termrock-implementation` for the current task. The only reference-branch exception is test, report, and CI updates on `origin/visual-baseline`.
Do not reset the implementation branch or create another implementation branch.
Do not create another Termrock repository.
Do not merge `main` or rebase onto it.

Termrock is the reusable component library.
Showcase, Jackin Preview, Holla, and TablePro are preview UI applications.
They demonstrate the library with fixture data and fixed interaction flows.
They are not separate product redevelopment projects.

The frozen visual authority is the `visual-baseline` tag at
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
Never move, delete, recreate, or retarget that tag or its release.
Use `termrock-refactor` as planning history.
Use `origin/visual-baseline` only for test, report, and CI updates and as a test and scenario reference. Do not use captures from it as expected visual output.
Use `main` only as structural evidence, never as expected visual output.

## Component Ownership

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

## Verification and Truth

Visual parity and component ownership are separate mandatory gates.
A matching image cannot excuse a bypassed component.
An imported component name cannot prove who painted the screen.
A large crate inventory cannot prove correct component boundaries.

Use `cargo nextest` to execute Rust tests. Do not use `cargo test`.
Build, format, lint, and document checks remain separate tool operations.
Select required ignored visual tests explicitly.
Never run an approval writer as an ordinary verification test.

Use synthetic test data. Do not capture real credentials.
Keep expected snapshots read-only during candidate verification.
Do not change expected output, tests, or thresholds to hide a regression.
Report not-run, failed, skipped, and passed checks separately.

## Invariant Rules

- **Strict Commit Sign-off Mandate**: Commits must ALWAYS be with `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`. Only use `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` for commits (using `git commit -s`). NEVER commit anything else than `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`. Any commit lacking this exact sign-off or containing any other sign-off is strictly forbidden.
- **Cargo Nextest Invariant**: All Rust test and validation commands must use `cargo nextest`; never use `cargo test`.
- **Visual Baseline Tag Invariant**: Never move, delete, retarget, force-push, or recreate the `visual-baseline` git tag or its GitHub release. It is the frozen pre-refactor visual oracle (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
- **CLAUDE.md Symlink Invariant**: `CLAUDE.md` must always be a symlink to the corresponding `AGENTS.md` in the same directory. Do not write a separate `CLAUDE.md` body.
