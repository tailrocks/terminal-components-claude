# Plan

## Product and architecture target

The modular implementation is incomplete and not ready. Repair the split architecture and reusable APIs on `main`, then deliver the complete product behavior recorded by `design/interaction-polish`. Do not transfer that branch's completed checkboxes into `main` as implementation evidence.

## Remaining work

1. Complete and repair crate boundaries and public APIs. Shared crates own reusable interaction, rendering, state, identity, layout, theme, and component behavior; app crates own domain workflows and composition. Verify the APIs with independent consumers.
2. Match the design reference across colors, layout, focus, keyboard and pointer behavior, scrolling, overlays, motion, deterministic scenarios, supported terminal sizes, and color modes.
3. Preserve showcase page coverage and reachability, including page gaps, vertical and horizontal scrolling, small-screen access, and responsive application behavior.
4. Verify TablePro behavior in current `crates/tablepro*`: app-owned SQL/domain ordering (including NULLs-last and numeric comparison); keyed Grid sort requests; stable cursor, selection, and edit identity during reordering; typed edit errors and per-app policy; read-only result grids where required; connection/query/filter/history/structure/plan/result/picker/dialog journeys; masked secrets excluded from output/logs/copy; deterministic timing, status expiry, focus, overlays, and capture behavior.
5. Verify Jackin behavior in current `crates/jackin-preview*`: host chrome, routes, dialogs, forms, Capsule and Manager surfaces; pure current status projection with priority, tone, stable actions and transient state; Bootstrap/Event/Tick/Settle ordering; earliest repaint deadline preservation; route-owned product time; dimming across color levels; total RunId and derived tokens; secret masking/save/key validation; deterministic motion, timestamps, rain/handoff, failure/running frames, and outro content.
6. Finish visual/conformance and PTY journey coverage with reviewed comparisons against the design branch. Map tests, captures, performance measurements, fixtures, and deferred black-box obligations to current code; keep component API and ownership assertions separate from PTY output evidence. A passing build/test alone does not prove visual parity. Do not update baselines without a separately authorized, reviewed change.

## Preserved acceptance detail

[Design work index](plans/design-work-index.md) links immutable full requirements from inspected design revision `d9e569bf6fd33d90266db597c458442c729340ad`, including shared-component plans, API/color/interaction reviews, showcase coverage, app follow-ups, and task checklists. It records 926 unchecked items across 533 source documents at that revision; those counts are inventory only. Current source and behavior still require verification.

Historical architecture decisions through §75 remain linked and individually dispositioned in [architecture](architecture.md). The old TablePro and Jackin slice plans are retired as workflows; their current behavior acceptance is retained above. Original detailed inventories remain at immutable [TablePro](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/plans/slice6-tablepro.md) and [Jackin](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/plans/slice7-jackin.md) sources. Old Holla integration plans and pinned Holla artifacts do not define current product requirements.
