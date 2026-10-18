# Documentation

## Status

Current project documentation lives in this directory's five canonical files.

## Authority

Use [status](status.md) for implementation state, [architecture](architecture.md) for the intended crate/API structure, [design](design.md) for visual and interaction requirements, and [plan](plan.md) for remaining work. Long-form plans remain actionable while their acceptance criteria are open. Audits, reviews, prompts, ledgers, and checkpoint results provide evidence only for their stated scope and revision; none overrides these files or establishes current readiness by itself.

[Safety rationales](../SAFETY.md) preserve the justifications for unchanged unsafe operations outside Rust comments.

## Current evidence

The workspace is split into Rust crates and deterministic application crates. This proves structural progress only. It does not establish correctness, reusable API quality, full behavior, visual fidelity, or release readiness.

## Remaining work

Keep the canonical files aligned with verified source, tests, and the accepted design reference. Preserve detailed plans and evidence; label their scope and historical status clearly.

## Verification

Check claims against the current workspace manifest, source, focused tests, and rendered application captures. A historical report or self-generated baseline alone is insufficient.

## Canonical documents

- [Architecture](architecture.md)
- [Design](design.md)
- [Status](status.md)
- [Plan](plan.md)
