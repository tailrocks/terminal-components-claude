# Current application inventory

## Status

Legacy documentation retained pending reconciliation with current source and accepted product reference.

## Authority

Current project direction: [architecture](../docs/architecture.md), [design](../docs/design.md), [status](../docs/status.md), and [plan](../docs/plan.md). Historical claims here do not establish current implementation readiness.

## Current evidence

This document snapshot is Git blob `0dccdb9086eed1de3fe634bb8cc5b0d711d61b8d` at the starting `main` revision. Its historical outputs do not transfer to current behavior without fresh verification.

## Remaining work

Preserve actionable requirements; reconcile them with the canonical plan and current branch evidence before treating them as active.

## Verification

Verify any current claim against the current source and, for product behavior, the accepted `design/interaction-polish` reference.

---

`app-inventory.json` schema1 is consumed by the strict borrowed Rust types in
`xtask/src/app_inventory.rs`. Unknown/duplicate fields, missing keys, unsupported
capabilities and shortened required obligations fail validation. It owns current
package/bin/lib/source identities, perf targets, startup CLI capabilities and
Holla capture requirements. Independent code pins protect required coverage.

- `cargo run --locked -p xtask -- app-inventory --json` validates required real
  Cargo package/bin/lib/ungated perf target identities before emitting JSON.
- `app-inventory --plan` emits `planned-not-captured`, explicit target blockers,
  the pinned132 Holla cases and44 additional ANSI16 cases. It does not capture,
  compare reference output, validate a built binary or declare coverage passed.
- `app-perf` validates targets then invokes each inventory-owned package's exact
  perf target with `--locked --release -- --test-threads=1 --nocapture`. Both CI
  perf lanes use it; failures propagate through the declared pipefail shell.

Startup capture retains the original96 identities and adds16 Holla cases using
`reference-default`, paused frame4000, first-use. Holla CLI receives no invented
`--theme` option. Required Paper coverage is separately declared as a pure
production-view suite. Metadata labels distinguish default reference theme from
an explicit selectable theme. Missing Holla binary/perf targets remain failures,
not temporary exclusions.

Historical499 parity manifests, IDs, aliases and frozen artifacts are unchanged.
Holla reference132 is pinned to794b095; added ANSI16 cases have no fabricated
reference artifact. The schema declares obligations, not successful artifacts.

Next phase requires a reviewed normalized case/build manifest join: suite and
case identity, exact Cargo compiler-artifact executable/source/lock/toolchain/
features hash, effective capabilities, steps and clock, plus capture-stage and
artifact hashes. Phase1 keeps existing capture backend safeguards but does not
claim that arbitrary BIN overrides prove their source build. Full/reduced clock
and journey capture semantics also remain explicit follow-up work.
