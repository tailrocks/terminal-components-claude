# Contributor handoff rules

Root [`AGENTS.md`](../../AGENTS.md) is the repository instruction authority. This page summarizes the minimum handoff practices for contributors working on a bounded task.

## Before work

- Read `AGENTS.md`, the active [goal](../GOAL.md), and the exact task or feature specification.
- Confirm the worktree, branch, and clean/dirty state. Do not overwrite another contributor's edits.
- Keep changes within the assigned files and preserve public behavior unless the accepted task explicitly changes it.

## Verification

- Use the repository's current verification commands and toolchain. Run `cargo nextest`, not `cargo test`.
- Verify visual changes against the immutable approved evidence. Never bless output merely to make a gate pass.
- Report command, exact revision, and result. A historical pass or a passing unit suite does not establish full parity.

## Handoff

Record unfinished work in [the handoff index](README.md) and its requirement in [remaining-work.md](../remaining-work.md). Include the exact commit, paths changed, verification performed, unresolved criteria, and next action. Keep credentials, scratch work, and machine-specific paths out of committed handoff docs. Remove the handoff when the maintained requirement is independently actionable without it.
