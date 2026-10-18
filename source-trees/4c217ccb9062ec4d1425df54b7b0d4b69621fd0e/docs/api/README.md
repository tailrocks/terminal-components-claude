# Termrock API

This directory is the canonical public API contract for the in-place Termrock refactor. Following Stage A preparation on `termrock-refactor`, the API is implemented progressively on the fixed implementation branch `termrock-implementation` from the frozen `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The signatures are design declarations until the P1 API freeze; they do not claim that the current Rust crate already exposes these names.

## Read in this order

1. [Public API contract](public-api.md) — ownership, phases, response flow, reconciliation, and component-facing rules.
2. [Shared types and models](types.md) — identity, geometry, runtime values, rows, model traits, and terminal-cell inputs.
3. [Authoring contract](authoring.md) — the constrained extension surface for generic component and part authors.
4. [Component catalog](../components/README.md) — the 45 component/API contracts and their behavior cases.

The API is coordinated with the [architecture overview](../architecture/overview.md), [runtime contract](../foundations/runtime.md), [input and actions](../foundations/input-actions.md), [layout](../foundations/layout.md), [layers](../foundations/layers.md), [theme](../foundations/theme.md), and [identity foundation](../foundations/identity.md). Those documents own their respective mechanisms; this directory owns how callers use them.

## Status and authority

This is a target contract, not an implementation report. The current source still contains legacy names such as `junie_tui`, `WidgetId`, `Outcome`, and `RenderCtx`. They are implementation evidence during the refactor, not future public compatibility requirements. No source or crate rename has happened in the documentation-only phase.

The API is for this repository and its future in-place Termrock library executed on `termrock-implementation`. No second repository, clean-room implementation, or application project is part of this contract. `showcase`, `tablepro`, `jackin-preview`, and `holla` adopt these public components incrementally at each phase; their frozen output is governed by [visual verification](../verification/visual-parity.md).

The pinned visual source is commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. It owns appearance and observable interaction. API choices may simplify ownership and composition, but they do not authorize a visual redesign.

## Surface map

| Question | Canonical section |
| --- | --- |
| Who owns durable state, props, and semantic effects? | [Ownership and phases](public-api.md#ownership-and-phases) |
| How does an event become a typed action? | [Responses and event flow](public-api.md#responses-and-event-flow) |
| How are collections and controlled values kept stable? | [Controlled values and reconciliation](public-api.md#controlled-values-and-reconciliation) |
| How is visual customization constrained? | [Parts, patches, and slots](public-api.md#parts-patches-and-slots) and [authoring](authoring.md) |
| What can a component author access? | [Author boundary](authoring.md#purpose-and-boundary) |
| How do secrets differ from ordinary text? | [Secret state](public-api.md#secret-state) and [secret types](types.md#secret-and-validation-types) |
| Is there a universal `Widget` or `show` API? | [Deliberate exclusions](public-api.md#deliberate-exclusions) |
| How does `Grid` support read-only and editing? | [Grid capability boundary](public-api.md#grid-capability-boundary) |
| What does `TerminalView` own? | [Terminal cell adapter](types.md#terminal-cell-adapter) |

## Proposal and freeze rule

Before P1 API freeze, external-consumer probes may adjust exact Rust spelling when the ownership contract remains intact. A spelling change must update this directory, the affected component contract, and the source/family mapping together. After P1 freeze, component work may not invent per-component response shapes, add hidden state ownership, widen runtime access, or silently add compatibility aliases.
