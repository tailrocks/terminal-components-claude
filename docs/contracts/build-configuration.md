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
The current Velnor Actions repository configuration is
[`.velnor/config.toml`](../../.velnor/config.toml). The retained
[`velnor-workflow.toml`](../../.github-gen/velnor-workflow.toml) is a historical
file and the source of the R-43 note below; it is not the current Velnor Actions
input. [`mise.toml`](../../mise.toml) pins Velnor Actions 0.1.48. The
generated-file ownership and regeneration rules are in
[`.github/AGENTS.md`](../../.github/AGENTS.md). Change declared inputs and use
the pinned generator. Do not hand-edit managed workflow output. The current
generated task manifest contains 90 formatting tasks across its default and
testing configurations. The removed `rustfmt.toml` comment identified that
file's presence as Velnor's `explicit_fmt` trigger. The generation check confirms
the output; it does not isolate the trigger logic.

At fixed input `383ad59b44baf95232005551b4d70f136bef3fea`, comments in the
retained historical file labeled it `Generator inputs for velnor-workflow`
with a D19 guard and marked recovery ledger R-43 as blocked. They reported that
the generator emitted one reusable unit workflow expanded per crate; with 44
members, the template exceeded GitHub's 5 MiB ceiling (99 callers = 8.24 MiB).
They said to regenerate when the generator chunks units or the workspace
splits lanes. Repository search found no separate active R-43 ledger entry.
These statements preserve the source-pinned rationale and do not establish
current generator behavior, size, or status. T37's generation evidence records
the current output separately.

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
