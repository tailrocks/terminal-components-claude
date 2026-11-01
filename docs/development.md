# Development

Install pinned tools with `mise install` (Rust 1.99.1 and the tool
pins in `mise.toml`), then work from the workspace root.

## Commands

| Task | Command |
| --- | --- |
| Run an app | `cargo run --bin showcase` (also `tablepro`, `holla`, `jackin`) |
| Functional tests (required gate) | `cargo nextest run --locked --test functional --profile functional --no-tests fail` |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` |
| Format check | `cargo fmt --check` |

All test commands use `cargo nextest`, never `cargo test`. Run the
functional suite, clippy and fmt before reporting work as done; see
[functional testing](functional-testing.md) for the test system and
[CI](ci.md) for how the gates run remotely.

## Conventions

- Keep Rust comments to safety invariants, legally required notices,
  or non-obvious behaviour constraints; contracts and rationale live
  in these docs.
- Any `unsafe` operation must have its invariant understandable at
  the operation and its callers; do not claim a safety concern is
  resolved without a specific review and evidence.
- `CLAUDE.md` is a symlink to `AGENTS.md`; edit `AGENTS.md` only.
