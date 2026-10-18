# Component architecture and migration requirements

This page describes the current implementation in this WIP, the split-workspace direction visible in `main`, and the contracts the migration must satisfy. `main` remains incomplete, broken, and not ready. The exact intended UI, interaction, color, layout, and behavior reference is `design/interaction-polish`; the monolithic code in this WIP is not the target architecture. A workspace manifest or successful compilation does not establish feature parity or readiness.

## Current WIP structure

The root manifest is a Cargo package named `junie-tui` with library crate `junie_tui`, workspace members `.` and `xtask`, and four binaries: `showcase`, `tablepro`, `jackin-preview`, and `holla`. They are implemented under `src/bin/` in the same package. The library exports `catalog`, `cli`, `clock`, `core`, `docs`, `motion`, `runtime`, `shell`, `theme`, `ui`, and `widgets`. Shared code currently lives in `src/core`, `src/shell`, `src/ui`, and `src/widgets`; app-specific domain, simulation, screens, and workflows remain inside the corresponding `src/bin/{showcase,tablepro,jackin_preview,holla}` trees.

This is a monolithic source/package boundary: the applications can reach library visibility available within their package, and the manifest does not enforce app-family crate boundaries. Treat implementation details as current-state evidence only. Do not carry forward historical module counts, line counts, or defect tallies as present measurements.

## Split-workspace direction in `main`

The `main` checkout uses a virtual workspace with 46 declared workspace members, plus three package manifests outside workspace membership and the root manifest (50 Cargo manifests total). The members group into 19 foundation/library packages (`termrock-*`), 22 application-family packages (Showcase, Jackin, Holla, and TablePro), and five test/proof/tooling packages. The exact current membership is recorded in the root `Cargo.toml`; package ownership and dependencies must be checked there and in each member manifest rather than inferred from the former `crates/tui`, `apps/*`, or `crates/tui-testing` proposal.

The split is the intended structural direction, not a completion claim. Current code, task-specific evidence, and the remaining-work ledger decide what has been implemented. Do not describe the split as landed, complete, ready, or behaviorally equivalent without current evidence for the specific claim.

## Required component and state contracts

The following are migration requirements. Names such as `Cx`, `Ui`, `Response`, and `ItemKey` describe the intended API shape; verify the current crate API before citing any as implemented.

### State and phase boundaries

- Durable interaction state belongs to caller-owned state types. It must not contain frame geometry, terminal colors/styles, borrowed application data, or other draw-only values. Keep state inspectable and testable; stateless components need no artificial state object.
- Per-frame props borrow application data and are constructed for the current frame. Props do not borrow durable state.
- Separate semantic updates from painting. Update receives input/context and mutable durable state, returns typed response/action information, and is testable without a terminal buffer. Drawing receives immutable durable state and paints/registers layout facts. Drawing must not commit values, dismiss layers, mutate domain data, or otherwise change semantics.
- Geometry and hit/focus registrations describe the last completed frame. They must not be recomputed by scanning a mutable app tree during event handling. Draw-time layout facts flow to the runtime through a defined reporting interface.

### Responses, identity, focus, and layers

- Use a common response shape with independent propagation/consumption, invalidation, state flags, and an optional component-specific action. Actions targeting collection entries carry stable item identity, not a row index that silently changes meaning after sort, insertion, or filtering.
- Reconciliation of externally controlled values is explicit and tested. A component must not silently retarget a retained selection/cursor when its item disappears or moves.
- Focus order follows visible reading order unless the UI contract specifies otherwise. Focus traps, modal barriers, pointer capture, wheel routing, hover suppression, press/release activation, click-outside dismissal, cursor ownership, and press feedback have one documented runtime policy and tests for boundary cases.
- A runtime-owned layer stack owns overlay ordering, hit testing, modality, dismissal, and focus policy. App content can be drawn into a layer without requiring `'static` storage. Modal and popover semantics remain distinct; underlying content is inert only where the design requires it.
- Timed feedback distinguishes wall-clock UI feedback from deterministic simulation time where app scenarios depend on simulated time.

### Layout, surfaces, themes, and composition

- Reusable layout primitives report the space they consume, handle empty/tiny rectangles safely, and support responsive arrangements without component-specific geometry leaking into durable state.
- Containers provide inherited surface context. Components resolve semantic roles against the active theme, surface, and terminal capability; avoid passing raw background colors through every component API or inferring semantics from RGB equality.
- Theme data covers semantic color roles and the design system's spacing, sizes, glyphs, borders, motion, density, and meter thresholds. Theme patches operate on semantic roles with explicit inherit/set/clear behavior. Variant, state, fallback, global, scoped, and instance customization precedence must be deterministic and tested.
- Capability downgrade is generic across true color, 256-color, 16-color, and monochrome output, with authored semantic palettes where needed. Junie and paper remain distinct built-in themes. Their values and appearance follow `design/interaction-polish` and its approved evidence.
- Compose components with explicit slots/closures and reusable collection APIs. Collections accept stable key extraction, caller-owned row/cell rendering and decoration, empty states, and a shared scroll-region policy. Keep behavior uniform without forcing every component through an unsuitable universal widget trait.

### Library and application boundary

- Foundation crates own reusable rendering, interaction, layout, text, theme, collection, and component behavior. App-family crates own product vocabulary, domain models, persistence-shaped simulation data, workflow orchestration, and app-specific screens.
- The grid component may provide a reusable model/editor protocol, but SQL, database constraints, pending changes, undo policy, and TablePro workflow semantics stay in TablePro-owned crates. No TablePro or Jackin product vocabulary belongs in general-purpose component APIs.
- Public APIs are deliberate and documented; internal modules remain inaccessible across package boundaries. App packages depend on the reusable public surface instead of reaching into implementation modules.

## Migration sequence

1. Use the active task ledger in `docs/remaining-work.md` and scoped task packages to identify each obligation, dependency, source owner, and acceptance gate. Historical checkpoint reports do not close tasks.
2. Assign each shared API and domain type to the actual `main` workspace package that owns it. Confirm package paths and dependency edges from manifests; record any ownership decision before moving code.
3. Migrate foundation contracts first: response/identity, state ownership, focus and input routing, layout, text, theme/surface, and layer behavior. Preserve the design contract and make the boundaries testable before moving app code.
4. Migrate reusable components and collections to the update/draw, stable-identity, inherited-surface, and typed-action contracts. Add focused behavior and architecture tests at each boundary.
5. Move each application's domain, simulation, and screens into its existing app-family package set. Keep application workflows intact and route reusable presentation through the library's public API.
6. Verify the integrated tree with relevant unit, integration, conformance, architecture, visual, terminal-safety, and performance gates. Compare identical scenarios and dimensions against `design/interaction-polish`; report exact scope and evidence for each closure.

## Verification

For current ownership and boundaries, inspect the WIP root `Cargo.toml` and `src/` tree, then inspect `main`'s root and member manifests. For design behavior and approval scope, follow `design/interaction-polish` and [the visual testing guide](../visual-testing/README.md). For migration status and per-task closure conditions, follow [the remaining-work ledger](../remaining-work.md) and its linked task contracts. Successful compilation verifies compilation only.
