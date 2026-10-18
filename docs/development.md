# Development and verification

## Prerequisites

Use the repository pins in `rust-toolchain.toml` and `mise.toml` (Rust 1.99.1 with `clippy` and `rustfmt`; Cargo MSRV declaration 1.99; `cargo-nextest` 0.9.148). The [dependency guide](dependencies.md) records the current pins and compatibility holds. Install the pinned tools and fetch the locked crate dependencies before working offline:

```sh
mise install
cargo fetch --locked
cargo nextest --version
mise exec -- mbx --help
```

`mbx` is installed as an explicit CLI; mise does not automatically wrap other commands with it.

## Run the applications

The root package is `junie-tui`; its library crate is `junie_tui`. Launch the showcase or a simulated preview with:

```sh
cargo run --release
cargo run --release --bin tablepro
cargo run --release --bin jackin-preview
cargo run --release --bin holla
```

These previews are deterministic and do not perform real external side effects.

## Code checks

Run the workspace checks after Rust changes:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run --all-targets
```

Use `cargo nextest`, not `cargo test`. For UI changes, run `cargo xtask visual check-touched` and the relevant tier from [the visual testing guide](visual-testing/README.md). Inspect actual, expected, and diff artifacts before changing an approval; use the guide's isolated-commit procedure.

## Generated component reference

`src/docs.rs` generates `docs/components/` and defines its freshness test. Check without rewriting generated output:

```sh
cargo nextest run --lib -E 'test(=docs::tests::docs_components_fresh)'
```

Regenerate deliberately with `cargo xtask docs components`. The generation command rewrites files; it is not a check mode. Never edit generated component pages by hand.

## Task contracts and visual tooling

`xtask` consumes `docs/visual-testing/README.md` and `docs/visual-testing/plan.md`; keep their commands, scenario rules, and approval protocol usable. Each task package in `refactoring-tasks/` defines its own checks in `verify.toml` and its evidence/acceptance contract in its README. The `trusted/` trees are pinned proof inputs; changing them requires an intentional task/proof migration.

This WIP is `wip/reach-pages-a`. Its successful build or test results apply only to the exercised revision and scope; they do not prove that the `main` crate split is ready or that this implementation matches all behavior on `design/interaction-polish`.
