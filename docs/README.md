# Documentation guide

This checkout is the visual and interaction reference branch. The product target is the exact behavior and presentation recorded by the [design contract](design/DESIGN.md) and its approved captures. This branch's implementation and remaining-work ledger describe this checkout only; they do not claim that the separate main workspace has completed the target.

| Document | Purpose |
|---|---|
| [architecture.md](architecture.md) | Current monolithic package and the intended workspace boundary. |
| [design.md](design.md) | Product authority and visual/interaction contracts. |
| [development.md](development.md) | Setup, run, verification, and documentation maintenance. |
| [../SAFETY.md](../SAFETY.md) | Historical unsafe-operation rationales and their current proof limitations. |
| [plan.md](plan.md) | Active refactor plan, acceptance gates, and task navigation. |
| [remaining-work.md](remaining-work.md) | Unfinished product, component, parity, and verification requirements. |
| [design/DESIGN.md](design/DESIGN.md) | Detailed design-system source. |
| [architecture/shared-components-plan.md](architecture/shared-components-plan.md) | Detailed component API and state contracts for the intended architecture. |
| [refactoring-plan/](refactoring-plan/README.md) | Task graph, proof contracts, traceability, and dated planning evidence. |
| [sources/](sources/README.md) | Historical source records and their non-authoritative status. |
| [refactoring-tasks/](../refactoring-tasks/) | Sealed task packages and their exact acceptance contracts. |
| [baseline/tuisnap-coverage.md](baseline/tuisnap-coverage.md) | Captured surface inventory and baseline verification procedure. |

Historical reports are evidence only. Current behavior comes from this checkout's source and tests; product acceptance comes from the design authority and immutable approved evidence. A historical pass or implementation note does not close current work.
