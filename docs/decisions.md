# Decisions

## Shared authority

The design reference governs exact product behavior, color, layout, keyboard and pointer interaction, and visual acceptance. Main governs the intended modular architecture. These authorities answer different questions: main's crate boundaries do not override the design, and design's current implementation does not prove that main or a WIP has completed the architecture or product.

## Compatibility and migration

Preserve public APIs and documented runtime behavior unless a deliberate architecture decision authorizes a breaking change. Retire old paths through an intentional migration with consumer updates and verifiable acceptance. Do not use historical status reports, old staffing plans, or frozen baseline captures as current readiness evidence.

## Evidence limits

Build success demonstrates compilation for the selected workspace and configuration. Tests demonstrate only their exercised cases. Captures demonstrate only their named states and dimensions. Historical source comments and safety rationale are evidence of prior reasoning, not fresh safety certification. The former Markdown proof inputs and generated-document contracts need an explicit protocol migration before their validators can certify this replacement tree.

## Scope discipline

Future proposals remain deferred until a consumer and acceptance contract are identified. Upstream work remains an external dependency until released, pinned where appropriate, and verified in this repository. Do not silently broaden supported platforms, host operations, or interaction abstractions without isolated evidence.
