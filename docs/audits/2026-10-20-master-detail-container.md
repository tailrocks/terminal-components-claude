# Shared master/detail container

## Scope

`termrock-controls::MasterDetail` provides the first reusable responsive layout
and state contract for Jackin's manager screen. It uses `SplitPane` for its
split geometry and resize seam. Defaults follow the frozen manager layout:
32% master, 28-column master minimum, 40-column detail minimum, 2-column gap,
collapse below 100 columns, and a 6-row summary.

The container exposes master, detail, summary, and seam regions. Its state owns
the split percentage, narrow drawer visibility, and focus return target. When a
detail owner or declared child has focus, narrowing selects the drawer even if
the explicit drawer flag is false. Tab opens details and returns focus to the
detail pane at every terminal width. Escape, Left, and BackTab return focus
from detail to master at every width. Commands are published through the
focused child's binding table so child bindings remain available and take
precedence on collisions. The additive binding API keeps each command's
visibility, label, and priority while applying user remaps, so the displayed
hint matches the command that will run. Hidden child bindings still shadow
later visible hints for the same chord. Explicit open/close methods use the
shared `Cx` API.
The roster split at 150 columns belongs to a future reusable library block;
it must not become private geometry in the Jackin consumer.

## Boundary

This step implements layout, state, baseline seam offset, and all-width
Tab/close focus routing for the detail owner and explicitly registered detail
descendants. The narrow detail view replaces the content area; it is not a
floating layer or modal. Existing `SplitPane` seam alignment defaults remain
unchanged. Showcase coverage and Jackin consumer integration remain pending.

Showcase coverage exception: keep the capability out of the Showcase until an
existing Showcase page can demonstrate it without adding a screen absent from
the frozen baseline. Remove this exception when that page conversion lands.

## Verification

Focused `cargo nextest` coverage checks the 120x40 default regions, exact
collapse threshold, summary and drawer states, tiny heights, area offsets,
minimum widths, overrides, split percentage/seam geometry, wide and narrow
focus transitions, key remapping, and child-binding precedence. Split resize,
full library, Showcase coverage, Jackin consumer integration, and baseline
parity gates remain required before acceptance.
