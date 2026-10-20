# NavList row-origin contract: content gap

## Decision

`NavList::content_gap(u16)` controls only the spacer after the existing fixed
gutter, marker and icon cells. The default remains one cell, preserving the
existing label origin at `row.x + 4`. Setting zero starts row content at
`row.x + 3`; the prefix cells keep their prior positions. Full-mode minimum
measurement follows the configured gap (`11 + gap`) while the default remains
12. Collapsed mode still omits row content and retains its existing minimum.

## Evidence

The independent public consumer in the frozen-source review packet
`holla-nav-list-row-contract-proof-42fb861.json` renders a non-`Display` model
through `NavList::row`. Current source reserves `x+0` gutter, `x+1` marker,
`x+2` icon and starts content at `x+4`; the frozen Finder row label starts at
`x+3`. An opt-in zero gap expresses that presentation without app coordinates
or a copied painter, while other consumers retain their default output.

`nav_list_row_policy` compares default and explicit-gap-one frame digests,
checks the `x+4` and `x+3` label cells, and exercises empty, narrow, zero-sized
and Unicode content. The change does not alter update behavior, IDs, hit
regions, selection, cursor movement or activation.

## Remaining presentation work

The separately requested `CursorStyle::Fill` policy is not implemented in this
increment. A real-runtime regression reproduced the existing focused NavList
gutter (`▎`) where Fill requires a full-row treatment without an additional
focus gutter. `RowUi::new_with_patches` currently accepts only CONTAINER and
LABEL patches, while NavList owns independent prefix part styles and slots.
That contract is under independent review before extending shared RowUi. Do
not treat this gap-only change as Holla parity or as a completed NavList row
presentation contract.
