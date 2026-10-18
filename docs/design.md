# Product and interaction authority

The exact visual and behavioral target is the design/interaction-polish product line, represented here by the detailed [`design/DESIGN.md`](design/DESIGN.md), approved oracle captures, and the active parity contracts. This branch's source, tests, and snapshots are evidence about this checkout; they do not supersede the target or prove complete parity.

## Contract coverage

The design contract governs color tokens and fallback, spacing, typography, component states, focus and hover, keyboard and mouse behavior, editing, scrolling, overlays, dialogs, responsive layout, and all four applications. Feature-specific decisions live alongside the relevant specs in `docs/design/`, `docs/showcase/`, `docs/product/`, and the remaining-work ledger.

## Resolving conflicts

1. Follow explicit user-approved design decisions and the current design/interaction-polish source.
2. Use approved immutable captures and their scenarios to verify observable output.
3. Treat this branch's implementation and tests as evidence of what it does, not as authority to change the design.
4. Treat older reports and historical screenshots as provenance only when current authority has superseded them.

No implementation or test result may silently redefine a token, interaction, scenario, or expected capture. Any accepted product change must update the design contract and its evidence deliberately.
