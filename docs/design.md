# Product design target

`design/interaction-polish` is the product authority for appearance, colors, UI/UX, keyboard and pointer behavior, and planned product work. `main` is the structural authority for the split Rust crates and reusable APIs. The modular implementation remains incomplete and not ready until it satisfies both.

Preserve all design-branch work and acceptance criteria. Implement accepted behavior through the split architecture: reusable controls, layout, state, identity, theme, rendering and interaction belong in shared crates; application workflows and domain policy belong in app crates. Review design-branch API ideas individually before adopting them.

Verify parity with exact source revisions, matching states, terminal sizes, supported color modes, and reviewed rendered comparisons. Compilation, tests, prose, and candidate baselines alone do not establish parity. Keep unresolved differences in [the plan](plan.md) and retain full design-branch acceptance detail through [the design work index](plans/design-work-index.md).
