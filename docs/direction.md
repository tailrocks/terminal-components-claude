# Direction

Termrock is a component library first. Its reusable controls and shared runtime own interaction and state; the demonstration applications prove that those contracts compose in realistic screens. The target combines the modular ownership described in [architecture](architecture.md) with the exact visual and interaction behavior described in [product](product.md).

This repository is the design reference and the product authority; its current structure is recorded in [current state](current-state.md). The modular workspace in [architecture](architecture.md) is the future refactoring state, not proof that the migration is complete. Nothing should be represented as equivalent to the design target merely because names, APIs, or selected screens match.

Preserve observable behavior and public APIs unless a reviewed architecture decision authorizes a change. Build toward shared ownership, validated state transitions, deterministic rendering, and evidence for real user journeys. Keep optional or speculative proposals deferred until a concrete consumer and acceptance need justify them.
