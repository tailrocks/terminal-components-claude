# Component-only interface contract

## ARC-001 — Required result

A preview screen is a composition of reusable Termrock components.
Its default output matches the frozen visual baseline.
Its interaction follows the same component path in tests and normal execution.

All of the following conditions are required:

```text
visual parity AND interaction parity AND component ownership
AND public API usability AND supported customization
```

Do not trade one condition for another.

## ARC-002 — Preview responsibility

The preview owns application data and simulated operations.
It selects the route and supplies the displayed records.
It stores durable component state between frames.
It handles typed component actions as application intents.
It selects documented layouts and semantic variants.

Examples include a workspace record, a query result, and a simulated activity.
Examples also include a chosen tab, a field draft, and a pending confirmation target.
A real product can replace the fixture service without replacing the view components.

The preview does not own generic focus, pointer capture, field editing, layer placement, or scroll geometry.
The preview does not draw borders, input carets, selection backgrounds, or widget hit regions.

Mock data is not mock interaction.
A preview must preserve the baseline's failure, cancellation, disabled, and recovery paths.

## ARC-003 — Library responsibility

A component owns its measurement, drawing, and generic interaction contract.
The shared runtime owns focus, hit testing, capture, layers, cursor arbitration, and supplied time.
Shared text and collection code owns Unicode geometry, editing, stable keys, and reconciliation.
A semantic theme owns default glyphs, colors, and metrics.

Keep one implementation for each shared mechanism.
The crate split does not justify duplicate mechanisms.
A crate with a Termrock name does not automatically contain reusable code.

## ARC-004 — Composition boundary

Use this data flow:

```text
fixture or service -> domain data -> public component props
                                  -> caller-owned component state
normalized input -> component update -> typed action -> application reducer
current data/state -> component measure/draw -> shared runtime -> terminal
```

A component must not receive a preview `World`, route enum, scenario ID, or expected frame.
Map domain records to narrow borrowed component models.
Do not clone a complete product world into component state.

Application screen functions can compose other application screen functions.
Each leaf visible control still belongs to a reusable component.
A wrapper around direct cell painting does not satisfy this boundary.

## ARC-005 — Simple public API

The ordinary application interface has four operations:

1. Construct a component from an ID and data.
2. Select documented options with builders.
3. Call update and handle its typed result.
4. Call measure and draw with current immutable inputs.

Do not require application authors to assemble runtime registries or internal contexts.
Do not expose a general mutable buffer through the ordinary consumer path.
Keep advanced component-author APIs separate.

Retain useful existing signatures when they meet the contract.
Do not redesign every API to match an invented uniform syntax.
Record current and proposed signatures separately in the API catalog.

## ARC-006 — Drawing permission

| Location | Permitted | Prohibited |
| --- | --- | --- |
| Preview route and shell | Component composition, state, typed actions, layout choices. | Raw cells, pre-rendered screens, manual generic input engines. |
| Domain and simulation crates | Plain records, deterministic effects, terminal process content. | Complete UI frames, component geometry, fixture-specific painters. |
| Stock component implementation | Cell rendering inside its component contract. | Product route names, expected-frame data, alternate snapshot answers. |
| Component-author extension | A reusable declared component or constrained visual part. | An application screen disguised as a primitive. |
| Test support | Isolated reference/candidate capture and observation. | Returning expected artifacts as actual output. |

This policy does not ban all geometry constants.
A one-row button height can be a component metric.
A responsive breakpoint can select a documented composition.
A table of captured terminal sizes cannot select stored answers.

## ARC-007 — Content is not an exported screen

Plain fixture labels and log messages are valid data.
Use component props to display them.
Do not concatenate padding, borders, selected rows, and buttons into a screen-sized string.

`TerminalView` receives terminal-process cells for its content area.
It must not receive a stored picture of Manager, Accounts, TablePro, or Holla.
The surrounding tabs, borders, menus, status, and input handling remain component compositions.

Use a reusable text-effect component for required decorative motion that no existing primitive can express.
Specify that component before implementation.
Do not add an unrestricted full-screen replay primitive.

## ARC-008 — Customization contract

The default recipe must match the visual baseline.
Other themes and variants have separate explicit tests.

Expose customization in this order:

| Level | Purpose |
| --- | --- |
| Theme | Shared semantic colors, glyphs, metrics, and state rules. |
| Variant | A named visual recipe for the same component. |
| Instance patch | A local semantic appearance change. |
| Part patch | A change to one declared visual part. |
| Content slot | Structured content inside a reserved area. |
| Author extension | A separately reviewed reusable component with its own contract. |

A slot receives immutable data and a bounded content interface.
A slot cannot alter the action target, focus ring, pointer capture, or disabled barrier.
A custom label cannot create another hidden button.
A patch must affect actual component output, not an unrelated wrapper.

Document allowed parts and their precedence.
Reject unknown parts instead of silently ignoring them.
Test default and customized output with changed data and geometry.

## ARC-009 — One path across execution modes

A fixture may select data, availability, and operation results.
A clock may select the current animation sample.
A motion policy may select a documented animation treatment.
None may select an alternate component tree just to match a stored frame.

Keep modal input, focus restoration, and pointer behavior active in paused mode.
Use the same components for keyboard and pointer operation.
Derive display state from real application/component state.
Do not create a display-only boolean that imitates an unperformed transition.

## ARC-010 — Two forms of proof

Visual proof asks whether the visible result matches the reference.
Ownership proof asks which implementation produced that result.

Require both proofs for every migrated component and preview surface.
Test component ownership through real call paths and controlled component mutations.
A change to the component painter must affect every relevant preview consumer.
A disabled component action must prevent the relevant application action.

Do not accept a draw call whose output is later covered by a historical painter.
Do not count an imported symbol as a migrated consumer.

## ARC-011 — Missing capability procedure

If a screen cannot use current components, identify the missing capability.
Check existing APIs and contracts first.
Specify the smallest generic extension with a real consumer.
Identify its owning crate and lower-level dependencies.
Show use with a second independent data shape or a generic conformance fixture.
Do not demand an artificial second product application.

Record implementation work as pending during this documentation task.
Never add the missing feature as an app-local renderer.

## ARC-012 — Existing violations

These requirements do not assert that the current code satisfies them.
Track known violations in the implementation checklist and repair list.
Do not weaken the contract to preserve a historical workaround.
Do not claim that documentation changes remove the workaround.
