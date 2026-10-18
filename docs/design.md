# Product design reference

## Status

`design/interaction-polish` is the accepted reference for visual appearance, UI/UX, colors, interaction behavior, and planned product work.

## Authority

Preserve the design branch's product intent when implementing through `main`'s modular crate architecture. The design branch's monolithic code structure is not the target. Reusable API ideas may be adopted after review and placement in the appropriate crate.

## Current evidence

The branch reference is named by the active project goal. Its actual captures, source behavior, and plans remain the evidence to compare against when the branch is available. Do not claim parity based only on prose, code compilation, or self-generated baselines.

## Remaining work

- Preserve all planned work recorded on `design/interaction-polish`.
- Carry its accepted UI/UX, visual, color, and behavior contracts into the split crates.
- Reconcile existing design specifications, plans, and captures; remove or correct conclusions that contradict the accepted reference.
- Record unresolved differences as open work rather than describing them as complete.

## Verification

Compare interactions and rendered captures against the design branch at matching states and terminal sizes. Record exact evidence for accepted changes. Do not bless differences as parity without review.
