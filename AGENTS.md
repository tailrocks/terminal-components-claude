# Execution Model and Engineering Rules

## Canonical Two-Stage Execution Model

1. **Stage A: Preparation on `termrock-refactor`**
   - Focus: snapshot, interaction, verification, and CI preparation.
   - Scope: `src/**` is strictly read-only; no refactoring of production behavior.
   - Deliverables: executable requirements registry, live/reference captures across all formats, independent admission, CI workflows, and acceptance evidence.

2. **Stage B: Termrock Refactor on `termrock-implementation`**
   - Execution: the Rust Termrock library refactor is executed exclusively on `termrock-implementation`.
   - Branch origin: `termrock-implementation` is created only from the exact accepted Stage A commit after passing its acceptance gate.
   - Scope: canonical P1–P7 implementation sequence, bounded consumer-adoption checkpoints across all four applications (`showcase`, `tablepro`, `jackin-preview`, `holla`), and final parity verification. Never continue production refactoring on `termrock-refactor`.

## Read First

Read the active task in [GOAL.md](GOAL.md) before any other document.
Read the [component composition contract](docs/architecture/component-composition.md).
Read the [per-commit review procedure](docs/process/per-commit-review.md).
Read the [ownership and parity contract](docs/verification/ownership-and-parity.md).
Read the canonical component contract before changing that component.
Follow the [document map](docs/process/document-map.md) when two documents overlap.

## Project Boundary and Authorities

Work only on `termrock-implementation` for the current task.
Do not reset the branch or create another implementation branch.
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
Use `origin/visual-baseline` only as test and scenario reference, never as expected output.
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

## Current Documentation-Only Task

The current task may change only its approved documentation paths.
This section is the canonical home of that path allowlist:

```text
AGENTS.md
GOAL.md
README.md
SPECIFICATION.md
CRATES.md
CHECKLIST.md
checklist.json
crate-map.json
component-ownership.json
docs/**
refactoring-tasks/** documentation and non-executable planning records
existing or necessary scoped AGENTS.md and README.md files
.github/pull_request_template.md
CLAUDE.md symlinks adjacent to approved AGENTS.md files
```

Do not modify source, tests, workflows, dependencies, or baseline artifacts.
Do not stage or commit the `termrock-doc-guardrails/` input pack.
Write future enforcement requirements as pending work.
Do not claim that these instructions install hooks or CI checks.

## Invariant Rules

- **Strict Commit Sign-off Mandate**: Commits must ALWAYS be with `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`. Only use `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` for commits (using `git commit -s`). NEVER commit anything else than `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`. Any commit lacking this exact sign-off or containing any other sign-off is strictly forbidden.
- **Cargo Nextest Invariant**: All Rust test and validation commands must use `cargo nextest`; never use `cargo test`.
- **Visual Baseline Tag Invariant**: Never move, delete, retarget, force-push, or recreate the `visual-baseline` git tag or its GitHub release. It is the frozen pre-refactor visual oracle (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
- **CLAUDE.md Symlink Invariant**: `CLAUDE.md` must always be a symlink to the corresponding `AGENTS.md` in the same directory. Do not write a separate `CLAUDE.md` body.
