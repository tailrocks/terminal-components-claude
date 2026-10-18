# Terminal Components

Rust terminal UI component library and deterministic demonstration applications.

## Status

The modular refactor has started but does not yet meet the intended behavior. It remains incomplete and not ready. `main` supplies the intended crate structure and reusable-API direction. `design/interaction-polish` supplies the accepted visual, color, UI/UX, and behavior reference, plus planned work to preserve.

## Authority

Start with [documentation index](docs/README.md). It links the canonical architecture, design, status, and plan documents. Older prompts, ledgers, reviews, and audits are historical evidence, not current readiness claims.

## Current evidence

The workspace manifest lists split library and application crates. `cargo check --workspace --offline` passed after comment removal at the current task baseline; this proves compilation only. It does not prove correct behavior, design parity, or readiness.

## Remaining work

Complete the comparison and documentation reconciliation across `main`, `design/interaction-polish`, `wip/reach-pages-a-2`, `wip/rapps-3-7-14`, `wip/reach-pages-a`, `wip/plan-s16`, `wip/s14-followups`, and `wip/v1-showcase`. Preserve the design branch's plans while implementing its intended experience through reusable APIs and split crates.

## Verification

Use current-revision checks and reviewed comparisons against the design reference. Do not treat successful builds, tests, or self-generated captures alone as proof of full completion.
