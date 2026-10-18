# Product design and interaction contract

## Product authority

Use `design/interaction-polish` as the exact authority for visual language, color, layout, focus, keyboard and pointer interaction, states, and user-visible behavior. Use `main` for structural direction; its split-crate refactor remains incomplete and not ready. The `wip/reach-pages-a` implementation and captures are branch-specific evidence and do not imply that every design-target feature is present.

## Detailed specifications

The full design contract is [DESIGN.md](design/DESIGN.md). Focused source findings are in [design review](audits/design-review-2026-10/); prioritized reusable changes are in [improvements](improvements/PLAN.md); app capability status and evidence are under [parity](parity/). Check each row and its scoped evidence instead of inferring blanket completion.

The visual harness and commands are in [visual testing](visual-testing/README.md). Captures and approval stores are revision-scoped data. The `visual-baseline` tag is immutable historical evidence, not the current design authority.

## Remaining acceptance work

Keep every open design, behavior, accessibility, visual, parity, API, app-coverage, and verification requirement in [remaining work](remaining-work.md), improvements, audits, parity plans, and task contracts. Keep scenario coverage aligned with actual states, overlay open and dismissal paths, keyboard/pointer behavior, terminal sizes, and color modes. A source change closes an item only with its stated current acceptance evidence.

## Visual verification

Compare matched scenarios, terminal sizes, and color levels against `design/interaction-polish`; inspect rendered output, not only cell deltas. For implementation changes run the corresponding app/component tests and `cargo xtask visual check-touched`. Record the exact revision, scenario set, color level, and disposition for every approval change.
