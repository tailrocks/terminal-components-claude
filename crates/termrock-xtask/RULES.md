# Ownership check

This page describes the source-level ownership check at [revision `7b2eb51`](https://github.com/tailrocks/terminal-components-claude/commit/7b2eb51c925a105f210c853e81b8ad440dad210f). It supports the consumer/author boundary in the [component contracts](../../docs/components-contracts.md). A finding is a review signal from bounded source recognizers; it is not a semantic proof by the Rust compiler.

The CLI interface is `termrock-xtask check-ownership [--root DIR] [--exceptions FILE] [--format human|json]`. The default format is `human`; the default exception file is `crates/termrock-xtask/exceptions/v1.json` under the selected workspace root. Run Cargo commands from the workspace root through the repository's required build guard and mise provider. This page records the interface and implementation, not a check result.

## Stable rule index

Rule IDs are stable. Keep their meanings attached to their IDs; do not reuse or renumber them. The inventory and summaries below follow [`rules.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/rules.rs#L30-L115).

| ID | Scope | Source pattern reported |
| --- | --- | --- |
| OWN-01 | Preview | Import of a raw ratatui buffer or cell type. |
| OWN-02 | Preview | Call to a cell-writing paint method. |
| OWN-03 | Preview | Escape to a raw buffer through `Ui::raw` or `peek`. |
| OWN-04 | Preview | Call to a local paint-wrapper function. |
| OWN-05 | Preview | Definition or invocation of a paint macro. |
| OWN-06 | Preview | Include of a stored frame or asset. |
| OWN-07 | Preview | Screen-shaped string literal. |
| OWN-08 | Preview | Rendering branch on test, debug, or environment input. |
| OWN-09 | Preview | Behavior selected from paused state. |
| OWN-10 | Preview | Behavior selected by scenario ID. |
| OWN-11 | Preview | Behavior selected by a width or height comparison. |
| OWN-12 | Library | Library dependency on a preview crate. |
| OWN-13 | Library production source | Reference to a stored expected frame. |
| OWN-14 | Production source | Use of `expect` outside the allowed place. |

## How files receive a scope

The check reads workspace member paths. `rules.rs` classifies `crates/termrock`, paths beginning `crates/termrock-`, and paths beginning `crates/termrock/` as library members. The preview set is the other parsed workspace members. Production classification excludes members whose final path component is `termrock-test-support` or `termrock-xtask`; it is not a general rule that every crate with a similar package name is tooling. See [`rules.rs` member classification](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/rules.rs#L142-L188).

The scanner removes comments and string contents from ordinary code lines, while retaining string spans for the literal-aware OWN-07 and OWN-13 checks. Generated-source markers set a flag that is carried into findings; they do not cause the file to be skipped. See [`scan.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/scan.rs#L105-L220), the [OWN-07 matcher](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/rules.rs#L368-L386), the [OWN-13 matcher](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/rules.rs#L491-L501), and the [generated marker handling](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/scan.rs#L74-L103) with finding construction in [`rules.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/rules.rs#L190-L205).

## Conditional compilation boundary

The module graph is a bounded recognizer, not a Rust parser. It collects reachable source paths and prunes a file as test-only only when the package traversal is complete and every known route proves that the file is test-only. If package reachability is uncertain, the traversal retains source for production scanning. A physical source reached by both a test and production route remains production. At the item level, uncertain syntax yields no test-only line ranges, so those lines remain in the production scan. See the [`module_graph.rs` contract and classification](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/module_graph.rs#L1-L5) ([package result](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/module_graph.rs#L448-L495)).

Do not read this as “all `cfg(test)` source is excluded.” For example, `any(test, feature = "extra")` can also be enabled in a non-test build, while a supported predicate such as `all(test, feature = "extra")` can prove a test-only route. The tests cover those distinctions, shared physical lines, external modules, and sources with both test and production parents: [`ownership.rs` scope tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/tests/ownership.rs#L336-L390), [predicate tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/tests/ownership.rs#L507-L590), and [external-route tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/tests/ownership.rs#L1120-L1165).

The recognizer handles a bounded set of local module and include routes. Lexical ambiguity, unknown syntax, shared physical lines, filesystem boundaries, or an unmodelled route remain limits of this source check, not a claim that it models all Rust compilation. Uncertainty must not authorize pruning; known routes still need to be collected. The source and tests above document representative cases, not exhaustive compiler equivalence.

## Exception ratchet and result

`exceptions/v1.json` is a count ratchet, not a general waiver list. Each `(path, rule)` entry sets a maximum for the observed finding count. New unexcepted findings and counts above their maximum fail reconciliation. Exceptions for missing files or zero findings are stale and fail. The schema version, rule IDs, and duplicate entries are validated before reconciliation. See [`exceptions.rs` validation and reconciliation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/exceptions.rs#L43-L138).

The report keeps findings separate from gate failures: findings are counted by path and rule, while `passed` reflects whether reconciliation has failures. The command supports human and JSON output and exits nonzero when the report does not pass; malformed usage is a separate command error. See [`main.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/main.rs#L10-L87) and [`ownership::Report`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-xtask/src/ownership/mod.rs#L13-L37).

At this source revision the generated workflow contains separate `rust-termrock-xtask` and `rust-termrock-xtask-testing` jobs. This is a reference to the pinned workflow definition, not evidence that a CI run passed. It does not add a custom workflow step or make the current workspace ratchet pass. See [the workflow jobs at `7b2eb51`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/.github/workflows/ci.yml#L6059-L6178).
