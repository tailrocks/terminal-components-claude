# Project plan

## Status

Active, unfinished migration. The modular implementation is broken as a product and not ready. Design-branch work already implemented remains the target experience; it does not prove that `main` implements it.

## Authority

Follow [architecture](architecture.md) for crate structure and reusable APIs, [design](design.md) for UI/UX, colors and behavior, and [status](status.md) for measured readiness. Preserve full acceptance criteria in the [design requirement index](plans/design-work-index.md); its immutable links retain task contracts and 926 recorded unchecked items across 533 source documents. These counts are inventory evidence, not a new completion assessment.

## Current evidence

The crate split has started on `main`. Design and WIP branches retain the product specification and planned work. The [branch comparison](verification/branch-authority-comparison.md) records the exact inspected revisions. Source checklist closures apply to their original branch and scope; they must not be transferred to `main` as completed work.

## Remaining work

1. Repair and finish crate boundaries and public APIs. Keep reusable interaction, rendering, state, identity, layout and theme behavior in shared crates; keep domain workflows and composition in application crates. Reconcile the retained slice plans under [plans](plans/) before retiring any valid acceptance requirement.
2. Restore the complete design-branch product experience through those APIs. Match colors, layout, focus, keyboard and pointer behavior, scrolling, overlays, motion, and deterministic scenarios. Review stronger design-branch API ideas individually and place them within focused crates.
3. Preserve shared-component S/SC plans, API reviews and interaction findings. See the complete source index, including the shared-components plan and design-review audit family. A prior implementation on the design branch is a contract to preserve, not proof of modular parity.
4. Complete showcase page coverage and content reachability, including the P0 page gaps, vertical and horizontal scrolling, small-screen access, and responsive application follow-ups. Keep the full backlog in the indexed `docs/remaining-work.md` and associated audit/task sources.
5. Preserve Holla/shared-component F/H/HP/O improvement contracts, deferred production capabilities, application follow-ups, and their delivery-order constraints. The indexed improvement tracker and small task files retain all detailed criteria.
6. Complete visual-test and oracle work against the accepted design branch. Keep current capture integrity, interaction coverage, deterministic fixtures, and applicable CI/dependency/upstream obligations. Old snapshots cannot establish current product acceptance.
7. Finish this documentation/comment-only cleanup across all eight branches. Reconcile branch-specific backlog differences; remove obsolete authority and completion claims; verify links, requirement preservation, zero Rust comments, and unchanged non-comment code tokens.

## Verification

Close each requirement only with evidence covering its complete acceptance contract. Build/test success is necessary where applicable and insufficient for visual/behavioral parity. Record exact branch revisions, commands, compared states and terminal sizes. Do not count omissions, historical green gates, or self-generated candidate captures as proof of completion.
