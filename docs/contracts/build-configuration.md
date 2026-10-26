# Build configuration

This guide records the maintainer context for the workspace lint, formatting,
workflow-generation, Nextest, and consumer dependency settings. The manifests
and generator inputs remain the source of truth.

## Workspace lints

The root `Cargo.toml` defines the workspace Rust and Clippy lint levels. The
maintainer rationale for these lint categories is preserved in the
[development guide's workspace lint rationale](../development.md#workspace-lint-rationale).
T37 keeps the parsed lint settings unchanged. This guide does not authorize a
lint waiver or a change to a lint level.

## Formatting and generated workflows

[`rustfmt.toml`](../../rustfmt.toml) selects `style_edition = "2024"`. The
workspace's Velnor input is
[`velnor-workflow.toml`](../../.github-gen/velnor-workflow.toml), and
[`mise.toml`](../../mise.toml) pins Velnor Actions 0.1.48. The generated-file
ownership and regeneration rules are in [`.github/AGENTS.md`](../../.github/AGENTS.md).
Change declared inputs and use the pinned generator. Do not hand-edit managed
workflow output. The current generated task manifest contains 90 formatting
tasks across its default and testing configurations. The removed comment named
this file as Velnor's `explicit_fmt` trigger. This generation check confirms the
output; it does not isolate the trigger logic.

At fixed input `383ad59b44baf95232005551b4d70f136bef3fea`, the removed Velnor
input comment recorded a D19 guard and an R-43 blocked status. It reported one
reusable unit workflow per crate, 99 callers, and an 8.24 MiB template against
a 5 MiB limit. It named generator chunking or split workflow lanes as possible
ways to address the reported size. Repository search found no separate active
R-43 ledger entry. This preserves the source-pinned report; it does not establish
the current status of that report. T37's generation check records the current
generator output separately.

The removed `rustfmt.toml` comment compared 565 legacy diffs with 223 diffs
under style edition 2024 when that setting was introduced. Those counts are
historical and are not current formatting results.

## Nextest profile

The standalone consumer profile keeps the same `fail-fast` and `slow-timeout`
values as the root CI profile. T37 preserves this runner configuration for
later reviewed targets. The current generated output has no eligible Nextest
run task, so behavior remains `NOT VERIFIED`; the retained profile does not
prove product behavior.

## Consumer dependency boundary

[`termrock-consumer-example`](../../crates/termrock-consumer-example/Cargo.toml)
and [`external-consumer-demo`](../../external-consumer-demo/Cargo.toml) each
declare `termrock` as their only direct Termrock normal dependency. The facade
declares production leaf-crate dependencies, so the consumer boundary does not
mean that no `termrock-*` crate appears transitively. The existing architecture
rules keep application and test-tool crates out of production normal and build
dependency closure; [T06](../implementation/pr38-final/TASKS.md#t06) and
[T30](../implementation/pr38-final/TASKS.md#t30) own those checks.

The external demo has an empty `[workspace]` table, its own lockfile, and a path
dependency on the facade. This gives it a separate Cargo workspace and
resolution. The facade-only normal dependency follows [API rule 14](../implementation/pr38-final/API-RULES.md#14-keep-examples-executable-and-source-comments-functional).
The production leaf and application boundary follows the
[architecture crate-boundary rules](../implementation/pr38-final/ARCHITECTURE.md#crate-boundary-rules).
The external path consumer is not a registry-package or no-checkout-path
proof; [T34](../implementation/pr38-final/TASKS.md#t34) owns that work.
