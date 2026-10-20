# Shared master/detail container

## Scope

`termrock-controls::MasterDetail` provides the first reusable responsive layout
and state contract for Jackin's manager screen. It uses `SplitPane` for its
split geometry and resize seam. Defaults follow the frozen manager layout:
32% master, 28-column master minimum, 40-column detail minimum, 2-column gap,
collapse below 100 columns, and a 6-row summary.

The container exposes master, detail, summary, and seam regions. Its state owns
the split percentage, narrow drawer visibility, and focus return target. In
narrow master mode, Tab opens details; Escape returns focus to the remembered
master or detail child. Explicit open/close methods also use the shared `Cx` API.
Jackin's roster split at 150 columns remains consumer composition.

## Boundary

This step implements layout, state, seam rendering in split mode, and narrow
Tab/Escape focus routing for the detail owner and explicitly registered detail
descendants.
It does not yet open a floating runtime layer for the narrow detail view or
register the capability in the Showcase coverage catalog. Existing runtime
layer ownership needs a follow-up integration before the Jackin manager can
consume this component.

Showcase coverage exception: keep the capability out of the Showcase until an
existing Showcase page can demonstrate it without adding a screen absent from
the frozen baseline. Remove this exception when that page conversion lands.

## Verification

Focused `cargo nextest` coverage checks the 120x40 default regions, exact
collapse threshold, summary and drawer states, tiny heights, area offsets,
minimum widths, overrides, and split percentage/seam geometry. Interaction
coverage for split resize, full library, Showcase coverage,
floating-layer integration, and baseline parity gates remain required before
consumer integration or acceptance.
