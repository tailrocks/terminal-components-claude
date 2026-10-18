# Development

## Setup

Use the Rust toolchain declared by `rust-toolchain.toml` and install the repository tools through `mise` as described in [dependencies.md](dependencies.md). The workspace includes the application package and `xtask`; use `cargo xtask` for repository-specific visual and architecture workflows.

## Run

The root [README](../README.md) lists the four binaries, supported scenario and motion options, and common keyboard interactions. The default binary is `showcase`; the others are `tablepro`, `jackin-preview`, and `holla`.

## Checks

Run focused tests for the changed area, then the repository's required nextest profile and visual qualification gates. The CI command set is documented in [ci.md](ci.md); visual capture, review, and acceptance are documented in [visual-testing/README.md](visual-testing/README.md) and [baseline/tuisnap-coverage.md](baseline/tuisnap-coverage.md). Do not accept candidate-generated output as oracle evidence.

## Source commentary policy

Rust comments are limited to details needed for a safety invariant, a required legal notice, or a behavior constraint that cannot be recovered from code or tests and is necessary to maintain the implementation. Put API contracts, examples, general rationale, and contributor guidance in Markdown. Do not replace removed Rustdoc prose with `#[doc = "..."]` attributes. Keep structural compiler attributes such as `#[doc(hidden)]` and actual runtime metadata such as `#[arg(help = "...")]`; they are not documentation substitutes. Keep a needed safety explanation beside the relevant operation and covered by a focused test where possible.

## Updating documentation

Keep product requirements in their feature specification and link them from [plan.md](plan.md). Mark acceptance only when the named implementation and verification evidence exist. Historical reports should be reduced to evidence that remains useful; they are not current status or authority.
