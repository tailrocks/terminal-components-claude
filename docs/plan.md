# Application acceptance plan

This document records acceptance work and its evidence boundary. A listed behavior is a requirement, not proof of implementation or readiness. Link a concrete current-source/test result before marking a requirement satisfied. Keep architecture ownership, product behavior, visual evidence, and operational proof distinct. Completed checkboxes on a design branch do not establish implementation in the modular workspace.

## Shared architecture and acceptance

Shared crates own reusable interaction, rendering, state, identity, layout, theme, and component behavior. Applications own domain workflows and composition. Verify crate boundaries and public APIs with independent real consumers; a manifest, API declaration, or passing build alone does not establish a complete boundary.

Compare the design reference across colors, layout, focus, keyboard and pointer behavior, scrolling, overlays, motion, deterministic scenarios, supported terminal sizes, and color levels. Preserve Showcase page coverage and reachability, including page gaps, vertical and horizontal scrolling, small-screen access, and responsive application behavior. Preserve current useful shared capabilities while recovering missing behavior.

Keep visual/conformance and PTY journey coverage traceable to current code and reviewed design comparisons. Map tests, captures, performance measurements, fixtures, and deferred black-box obligations separately. Component API and ownership assertions are not PTY output evidence. Preserve existing captures and fixtures; a passing test alone does not prove visual parity, and baseline changes require separate reviewed authorization.

## TablePro

The application domain owns SQL/catalog/filter semantics. Keep Grid sort requests keyed and shared; keep result-model ordering in the application domain. Compare values by their declared type, including numeric values, and preserve the acceptance rule that NULL values sort last in both directions. Track this separately from implementation evidence: source reviewed at `35303f52f218b74dea9b864e19e4b9f10d06d95e` uses numeric `total_cmp`, but reverses the full comparator for descending order, placing NULL first. That does not satisfy NULLS-last. Add or retain explicit ascending/descending NULL-order assertions before recording this requirement as met.

Preserve row, selection, cursor, column and edit identity across sorting and filtering whenever the same uniquely identified row remains. Exercise typed edit errors and per-app policy, read-only result grids where required, connection/query/filter/history/structure/plan/result/picker/dialog journeys, and ensure masked secrets never enter output, logs, or copy. Verify deterministic timing, status expiry, focus, overlays, and capture behavior. Keep simulated transaction state separate from claims of external database effects.

Current source includes domain sort/key tests, a real header-sort/undo identity test, and a filter-Apply/edit/save identity journey. These are named-case evidence only, not closure of all sorting, filtering, policy, or application journeys; a fresh run and exact revision binding are needed for a current pass claim.

## Jackin Preview

Verify host chrome, routes, dialogs, forms, Capsule and Manager surfaces. Preserve pure current-status projection with priority, tone, stable action identity and transient state. Keep product time owned by the Jackin route/world cadence; runtime repaint deadlines schedule updates but do not replace the route interval that advances the virtual world. Verify Bootstrap/Event/Tick/Settle ordering, earliest repaint deadline preservation across early input/draw and animation/modal changes, paused/reduced motion, and dimming across supported color levels.

Keep typed RunId construction and total derived display/container tokens. Verify launch-ID collision avoidance, secret masking and save/key validation, deterministic motion and timestamps, rain/handoff, failure/running frames, and outro content.

Current timing tests cover named early-deadline, unchanged-time, pause, idle/animation cadence, pending-job wake, dismissed-animation and modal-continuity cases. Current RunId tests cover named deterministic-ID, stable-label, short-form and container-UID cases. These tests and source definitions do not close untested status, color-level, masking, lifecycle, combined-deadline or visual cases, and their presence is not a fresh pass receipt.

## Holla: preserve requirements pending individual disposition

Current [Holla requirements](requirements.md#holla) describe deterministic simulation without real host side effects. That scope does not retire historical UI, interaction, data, timing, discovery, navigation, or scenario requirements. Preserve the original Holla integration plans and pinned artifacts for source-qualified, per-item review. Do not infer completion, obsolescence, or removal authority merely from their age or from the simulation boundary.

The historical candidate's blanket retirement statement is not adopted as a current decision. Historical migration steps and host-adapter proposals likewise are not automatically current APIs or supported operational capabilities. Useful behavior must be retained or explicitly adapted; unresolved differences remain REVIEW. Any future real host adapter follows the separate safety, capability, privacy and evidence requirements in the current requirements document.

## Traceability

Use [requirements](requirements.md), [architecture](architecture.md), [product](product.md), [decisions](decisions.md), and the immutable [design-work index](plans/design-work-index.md) without copying their full checklists. Its 533 source documents and 926 unchecked items describe the inspected `d9e569bf6fd33d90266db597c458442c729340ad` inventory, not a current implementation or completion audit. Canonical rendered comparisons must name their actual source revision separately.

The recovered planning source is [the original plan](https://github.com/tailrocks/terminal-components-claude/blob/b0edda802880473f5541dc71d18d5cfe5ecd991c/docs/plan.md). Preserve original detailed [TablePro](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/docs/plans/slice6-tablepro.md) and [Jackin](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/docs/plans/slice7-jackin.md) inventories, plus the original [Holla integration plan](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/docs/plans/main-holla-integration.md), [Holla integration task](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/docs/plans/main-holla-integration-task.md), and [Holla baseline](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/docs/audit/main-holla/baseline.md). Re-check current APIs and behavior before reusing historical migration steps. These immutable links preserve detailed sources; they neither retire useful acceptance nor establish current readiness.
