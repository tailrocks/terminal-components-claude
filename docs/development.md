# Development

## Workspace commands

Install the checked-out workspace's pinned developer tools with `mise install`, then use its declared Rust toolchain. Use `cargo nextest` for tests, as required by the design-reference and WIP instructions. Before running a gate, inspect `AGENTS.md`, `Cargo.toml`, `rust-toolchain.toml`, and CI configuration because package layout and profiles differ.

| Workspace | Location | Test command policy |
|---|---|---|
| `main` | Workspace packages, with default members for facade/apps/conformance | `cargo run -p showcase`, `cargo run -p tablepro`, `cargo run -p holla`, `cargo run -p jackin-preview`; `cargo check --workspace --all-targets`; `cargo nextest run --workspace`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`. |
| `design/interaction-polish` and six `wip/*` branches | `junie-tui` package with `xtask` workspace member | `cargo run -p junie-tui --bin showcase` (substitute `tablepro`, `holla`, or `jackin-preview`); `cargo check --workspace --all-targets`; `cargo nextest run --workspace`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`. Follow branch-specific instructions for visual and repository-specific gates. |

The commands above are starting points verified against the package manifests. Format, check, lint, and test the affected workspace using its configured commands. Run focused tests first, then required integration and visual gates. Record exact command, branch, revision, and result. A passing local command is evidence only for that command and revision; old visual and nextest results are not current verification.

## Rust comments

Keep Rust comments only for a safety invariant, legally required notice, or non-obvious behavior constraint that cannot be recovered from code or tests and is necessary to maintain correctness. Do not replace removed prose with `#[doc = "..."]`. Preserve structural compiler attributes such as `#[doc(hidden)]` and actual runtime metadata such as `#[arg(help = "...")]`. Existing Clap derive documentation consumed as CLI help metadata is retained where removing it would change user-visible help; that narrow use does not make general API documentation in source the policy. Frozen Rust fixtures whose exact bytes are validation inputs remain unchanged. Put contracts, API guidance, examples, and design rationale in maintained Markdown.

## Safety evidence

Any unsafe operation requires its applicable invariant to be understandable at the operation and its callers. `pre_exec` runs in a post-fork child context and must use only operations permitted there; the PTY helper's `setpgid` setup and parent-side race handling require review. Nextest process isolation does not prove that a test process has no threads or environment readers. Zero-initializing `termios` before `tcgetattr` does not prove libc initializes padding, nor by itself justify reading every byte. Do not claim these concerns are resolved without a specific review and evidence.

## Documentation replacement limits

The previous Markdown tree was consumed by hash/freeze/task validators, generated component documentation and freshness checks, visibility scripts expecting status/queue files, and qualified-capture README pins. Those consumers have not been migrated as part of this documentation rewrite. Treat affected gates as incompatible until their formats, generators, and proof protocols are intentionally updated. Do not regenerate hashes or change validator inputs merely to make a gate appear green.

Removing source prose can expose documentation-lint failures in unchanged Clippy configuration, and comment deletion can leave formatting differences. These checks remain useful diagnostics, but this documentation-only change does not alter lint settings or executable Rust tokens to silence them. Resolve the policy or formatting contract in a separately scoped change; do not restore redundant source documentation merely to conceal the mismatch.
