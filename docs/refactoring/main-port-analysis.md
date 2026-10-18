# Structural migration direction

## Status

The workspace refactor is incomplete. This page describes how to evaluate migration work; it does not certify the state of either branch or preserve an old port checklist as active status.

## Authority

The design/interaction-polish branch defines exact product behavior. The main workspace is the structural direction for packages and reusable APIs. A candidate change must satisfy both authorities; cherry-picking a main change or passing its local tests is not sufficient evidence of product equivalence.

## Current evidence

This product-reference checkout uses a root `junie-tui` package and places its four applications under `src/bin/`. The target workspace separates the component library, testing support, `xtask`, and four application packages. See [architecture.md](../architecture.md) and [component architecture](../architecture/component-architecture.md) for maintained contracts.

## Migration requirements

- Port reusable state, rendering, event, identity, focus, scrolling, layer, theme, and collection contracts as a coherent architecture.
- Keep application-specific domain models and workflows in their app packages.
- Preserve existing public behavior and migrate public APIs deliberately; do not leave parallel legacy implementations after a replacement is qualified.
- Test structural boundaries and app integrations, then compare all required observable behavior with the immutable design reference.
- Re-evaluate every port against this branch's later product work; historical commit selection alone cannot establish compatibility.

Detailed contracts and owners remain in [shared-components-plan.md](../architecture/shared-components-plan.md), [remaining-work.md](../remaining-work.md), the task graph, and the sealed task catalog.
