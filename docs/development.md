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

## Documentation compatibility

On main, visibility source validation does not require a checked-in prose snapshot: use `python3 tools/visibility/status.py --validate` and `python3 tools/visibility/queue.py validate`. Generated reports are operational artifacts under ignored `target/visibility/STATUS.md` and `target/visibility/WORK_QUEUE.md`. Explicit generation (`status.py --write` or `queue.py repair-render`) precedes output checking (`status.py --check` or `queue.py check`). Output checking still rejects missing, stale, or tampered reports. An explicit output override is confined to the visibility artifact directory; it cannot replace maintained documentation or authoritative inputs. No tracked status or work-queue documentation is recreated.

Main's capture and deferred-evidence packaging records retired documentation pins explicitly while continuing to require every operational payload. Historical source, archive, receipt, measurement, and binary identities remain historical; changing packaging is not fresh qualification. Integrity tests reject operational deletion, corruption, unauthorized retirement, and invalid provenance. Disposable integration fixtures generate their report views from authoritative inputs rather than copying deleted documentation.

Main's minimal-comment migration changes three documentation-specific lint settings: `missing_docs`, `clippy::missing_errors_doc`, and `clippy::missing_panics_doc` are allowed. Safety lints and the remaining lint policy stay unchanged. Formatting and behavior-preserving style fixes must still pass the workspace gates. This policy does not authorize restoring redundant prose to conceal a lint mismatch.

The design reference and WIP workspaces can still contain older generated-component documentation, task/hash/bootstrap validators, and report consumers expecting retired Markdown or historical text. Inspect the checked-out implementation before invoking those workflows. Their remaining protocol migrations must preserve operational evidence and rejection behavior; do not regenerate measured hashes or restore legacy guidance merely to make a gate green. A passing main gate does not certify another branch's tooling.
