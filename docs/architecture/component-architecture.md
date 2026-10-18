# Component architecture: current state and target

## Current state in this checkout

The root `junie-tui` package builds the library and four applications. `src/lib.rs` exposes modules for core input/state, rendering support, widgets, shell composition, and theming. Widgets generally combine props, durable state, event methods, drawing, and cached geometry. App binaries compose those widgets and own application-specific navigation and workflows. These facts describe the current source tree, not a completed component architecture.

The applications and library are built from this branch's source. The design/interaction-polish branch and its approved evidence define the exact product target. The current implementation can contain gaps; a matching compile or passing local test suite does not establish visual or interaction parity.

## Required component contracts

The following are target requirements. Their detailed API proposals and acceptance conditions are maintained in [shared-components-plan.md](shared-components-plan.md) and the task packages; implementation must be verified before any item is called complete.

1. Keep durable interaction state caller-owned and independent of terminal geometry and rendering styles. Keep borrowed presentation props frame-local.
2. Separate semantic update from drawing. Event/update paths produce typed actions and responses; drawing observes state and reports layout or hit-region facts without changing application meaning.
3. Use stable identities for controls and collection items. Reconciliation must preserve identity through insertion, filtering, sorting, and replacement according to each component's contract.
4. Give focus, pointer capture, hover/press feedback, overlays, scroll routing, and cursor ownership one explicit runtime policy rather than duplicating policy in each app.
5. Resolve semantic theme roles and surfaces centrally. Components must not infer semantic meaning from raw color equality or depend on caller-threaded background colors.
6. Share collection mechanics while preserving each component's public behavior: keyboard and pointer selection, editing, scrolling, empty/disabled states, row decoration, and caller-owned data.
7. Keep domain models and operations in their app packages. The reusable library must not absorb TablePro, Holla, or Jackin workflow semantics.
8. Keep public facades curated and enforce package boundaries in build checks. App-facing and component-author APIs have distinct responsibilities.

## Structural migration sequence

1. Establish workspace/package boundaries and freeze the public API surface.
2. Port reusable primitives and components while retaining this branch's exact product contract.
3. Port app composition and domain behavior through the shared APIs; do not transplant a branch's self-baseline as parity proof.
4. Qualify behavior and rendering against the immutable visual reference at every required state and terminal size.
5. Remove superseded paths only after callers, tests, and executable boundary checks demonstrate the replacement.

The active requirements and task dependencies are indexed in [the refactoring plan](../plan.md). This sequence does not declare any migration step complete.
