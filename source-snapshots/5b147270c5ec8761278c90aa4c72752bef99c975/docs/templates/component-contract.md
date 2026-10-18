# <Component name>

Status: <current / proposed / partly implemented>.
Owner: <one Termrock crate>.
Visual authority: <tag commit and source paths>.

## Purpose and exclusions

State the reusable need.
State what the component must not own.

## Public API

Show current source-checked signatures separately from proposed changes.
Document construction, state, update, measure, draw, and typed actions.
Name every application-owned helper in the example.

## Ordinary use

Show data, state, update handling, layout, and drawing.
Include a real preview consumer and an independent data fixture.
Do not use raw cells in the consumer.

## Ownership

List the owner of each visual part and interaction rule.
List shared runtime/text/scroll/theme dependencies.

## Customization

Document variants, patches, slots, precedence, clipping, and forbidden overrides.
Show an ordinary example and one advanced example.

## Behavior

Document keyboard, pointer, focus, disabled/read-only, editing, validation, and source-change rules where applicable.
Preserve intentional differences from related components.

## Visual matrix

List applicable states, geometry boundaries, color modes, and time checkpoints.
Give reasons for non-applicable states.

## Verification

Link exact reference/candidate case IDs, semantic assertions, and ownership mutations.
List tests that detect copied frames and bypassed interaction.

## Rejected use

Show one small forbidden example with its rule ID.
Explain the component-owned replacement.

## Known gaps

Link pending implementation work.
Do not mark the component complete while required evidence is absent.
