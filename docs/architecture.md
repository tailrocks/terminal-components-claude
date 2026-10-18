# Architecture

## Current implementation direction

Main is moving from a monolithic package toward a workspace of focused crates and thin applications. The present split includes core, layout, theme, text, rendering, collections, runtime, controls, fields, feedback, navigation, viewport, overlays, grid, editors, forms, terminal, and session responsibilities, with a `termrock` facade. This migration is partial: manifests show intended boundaries, while current modules and consumers must be inspected to determine what has actually moved and whether APIs are coherent.

The core owns IDs, events, intents, responses, actions, validation, and diagnostics. Runtime owns focus, hit testing, key maps, layers, UI dispatch, capture, and runtime coordination. Terminal interaction models such as selection and input tokens are distinct from session lifecycle. Session owns entering, leaving, and running the terminal session. Controls, fields, feedback, navigation, viewport, overlays, grid, editors, and forms own reusable component families. Applications own domain models, routes, demo data, and composition; they should not duplicate shared widget behavior.

The applications are Showcase, TablePro, Holla, and Jackin Preview. In the six WIPs and design reference they remain together in a monolithic `junie-tui` package; main has split their responsibilities into crates. Treat each as a consumer of library behavior; do not assume a boundary is complete because the corresponding crate exists.

## Migration gates

Complete the migration in layers: first establish crate ownership and dependency direction; then move a component or runtime contract with its tests and real consumers; then demonstrate it in Showcase and validate app journeys; finally qualify visual states, portability, performance, and public API compatibility. A boundary is accepted only when its owner crate does not rely on application internals, consumers use the shared implementation, and focused plus integration evidence passes. Product parity and architecture completion are independent gates; a completed crate move does not close behavior or visual work.

## Required ownership contracts

- State changes that can fail must validate before commit or leave the prior state intact.
- Selection, cursor, and focus identity must remain stable across sort, filter, resize, and data replacement where the item remains present.
- Disabled controls reject interaction; read-only controls remain navigable and expose their state distinctly.
- Text editing and clipboard operations must respect grapheme boundaries. Copy, paste, and modal routing need consistent shared ownership.
- Key and modifier handling must preserve unhandled events and route modal input before background content.
- Empty or unavailable content must render deterministically and remain accessible through keyboard focus and visible state.
- Shared behavior belongs in the component or shared layer, not duplicated across screens.
- Keep public API behavior compatible unless a deliberate migration includes consumer updates and verification.

These are target contracts, not assertions that all branches currently satisfy them.

## API direction and review

An API proposed for main or a WIP is a candidate until real consumers and tests validate its ownership, naming, and compatibility. Do not treat current main APIs as automatically correct simply because they sit in the intended crate. Review ergonomics, object/state identity, failure behavior, accessibility, and whether the boundary prevents screen-level duplication. A reusable API must support the required behavior without leaking app-specific assumptions; avoid adding generic traits or extension points without a consumer that demonstrates the need.
