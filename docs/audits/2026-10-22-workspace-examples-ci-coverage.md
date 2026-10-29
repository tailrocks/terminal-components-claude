# Workspace examples CI coverage

Scope: goal item "Add compiled examples to CI". Audits what the generated
Velnor workflow actually compiles and runs for the external public-API consumer
demo and for workspace `[[example]]` targets. No generated file was edited and
no config change was needed; this document records the verified state and the
one residual gap.

## External consumer demo: covered

`external-consumer-demo/` (standalone non-member crate, own `Cargo.lock`) is
discovered and gated by the generated workflow. `.github/workflows/ci.yml` has
two jobs, `rust-external-consumer-demo` and `rust-external-consumer-demo-testing`,
each running the full per-crate lane — build (`cargo nextest list --profile ci
--list-type binaries-only`, which compiles the test binaries), nextest run,
clippy (`--all-targets`), format, doc — and both join `required`. The cache-key
step hashes `external-consumer-demo/Cargo.lock` separately from the workspace
lock, so the standalone lockfile is a first-class CI input. Local validation on
`ef23c3303c`: `cargo nextest run` in `external-consumer-demo/` — 9 tests run,
9 passed. No CI job was missing for the consumer demo.

## Workspace example targets: compiled, not run

The workspace declares 12 example targets in 4 crates: `crates/termrock`
(8, one explicit `[[example]]`: `master_detail_composition_consumer`, the rest
autodiscovered), `crates/termrock-forms` (2), `crates/termrock-overlays` (1),
`crates/termrock-shell` (1).

- **Compiled**: yes, with full codegen. The per-crate "Build test executables"
  step runs exactly `cargo nextest list --profile ci --list-type
  binaries-only --locked --offline ...` (`.github/velnor/
  task-execution-manifest-v1.json`), and nextest's underlying cargo build
  compiles example targets like any `cargo test`-family build. Verified
  empirically with the pinned toolchain: deleting
  `target/debug/examples/dialog_primary_consumer` and re-running that exact
  command rebuilt the example binary. Clippy additionally type- and
  borrow-checks every example through `--all-targets -- -D warnings` on every
  crate lane.
- **Run**: no. Nextest never lists or executes example targets — regardless of
  `test = true`, `harness = false`, or `--all-targets` (checked against
  nextest's own behavior, not the generator). The documented local gate
  (`cargo nextest run --all-targets`) has the same property, so CI does not
  diverge from the repo gate here; examples are compile-checked everywhere and
  executed nowhere.

All current examples are headless render-once consumers (none touches crossterm
raw mode, alternate screen, or event reads), so execution would be safe if a
mechanism existed.

## Why no config change was made

`.velnor/config.toml` cannot express example execution, and none is needed for
compilation: the existing per-crate build step already compiles every example
with codegen, which is what the goal item asks CI to guarantee. Checked against
the pinned generator (0.1.32) and current upstream `main` of
`tailrocks/velnor-new`:

- `stacks.rust` exposes compile driver, test runner, run-ignored mode,
  configurations, suites, and per-crate build jobs only. There is no
  all-targets or examples flag; upstream `target_flags()` derives `--examples`
  solely for `cargo test` groups, and the Nextest argv branch never emits
  target flags.
- `[[workflow.tasks]]` `kind = "build"` is the only declared task kind allowed
  to compile, and its runner enum is macOS-only (`macos-26-arm64`,
  `xcode-27-arm64`) — native-lane machinery, not a fit for a Linux cargo lane.
- `kind = "verification"` tasks are compile-free by contract; smuggling a cargo
  run of examples into one would violate that contract.
- Even with all of the above, nextest itself would not run an example binary,
  so no generator-side change short of a new task kind could deliver execution.

## Residual gap (recorded, deliberate)

Examples are never executed by any CI or local gate. If execution is wanted,
the options are a product decision: (a) convert the demo files into integration
test targets (they stop being `examples/`), (b) keep `examples/` and add
mirroring `[[test]]` targets with shared modules, or (c) upstream a velnor-new
task kind that runs example binaries under a timeout. Until one is chosen, the
compile guarantee stands and the demos remain exercised indirectly by the
consumer-example crates (`termrock-consumer-example`, `external-consumer-demo`)
whose nextest lanes run equivalent composition code.

## Baseline at audit time

`velnor-actions generate` (pinned 0.1.32) reproduced the committed `.github`
tree byte-for-byte (`diff -r --brief` clean), so the findings above describe
exactly what CI runs.
