- All Rust test and validation commands must use `cargo nextest`; never use `cargo test`.
- Never use containers, Docker, Podman, images, mounts, firmlinks, or a
  container runtime for this refactoring. Use isolated subagents only.
- Use only the latest standalone taskfmt from
  `/Users/donbeave/Projects/taskfmt/task-format` for per-task `lint` and
  `verify`; never use taskfmt lifecycle or container-start commands.
- Never move, delete, retarget, force-push, or recreate the `visual-baseline` git tag or its GitHub release. It is the frozen pre-refactor visual oracle.
- `CLAUDE.md` must always be a symlink to the corresponding `AGENTS.md` in the same directory. Do not write a separate `CLAUDE.md` body.
