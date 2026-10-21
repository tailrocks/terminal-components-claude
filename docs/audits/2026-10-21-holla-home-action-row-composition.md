# Holla Home action row composition

## Scope

This increment migrates the live Home action rows in
`crates/holla-ui/src/screens/home.rs` from an application-owned paint callback
to `NavList::row` and `RowUi`. Finder presentation selectors, size-specific
plates, and scenario/time routing remain separate and unresolved.

## Change

`NavList` now owns row fill, focus presentation, item hit regions, clipping,
scrolling, and the row gutter. Home supplies live `Action` data through five
semantic cells: title, kind, scope, reason, and risk. Cross-row natural widths
come from the live row data; the existing Termrock column solver applies the
36% title cap, two-cell gaps, and shrink/drop behavior within each row's
allocated content width. The warning state remains visible for actions that
need trust or are blocked. Stable action IDs and domain dispatch remain
unchanged.

The action-row app painter no longer fills row backgrounds, writes `▎`, or
calculates cell coordinates. `CursorStyle::Fill` keeps the active row distinct
without adding a second focus gutter while the query owns keyboard focus.

## Regression coverage

`shared_action_row_keeps_live_semantic_fields_and_owner_focus` drives the real
Home fixture. It checks all five live fields, transfers focus to the row list,
verifies there is no app-owned `▎` in its region, and confirms navigation stays
diagnostic-free. Existing Home tests continue to exercise keyboard and pointer
activation, query filtering, trust presentation, scrolling/reveal, and menu
focus restoration. The menu lifecycle test now asserts the inactive row region
contains no `▎`.

Commands on Rust 1.99.1, with a worktree-exclusive target:

```text
CARGO_TARGET_DIR=/private/tmp/holla-home-row-target-20261021 rustup run 1.99.1 cargo nextest run --locked -p holla-ui --lib --no-fail-fast
142 passed, 0 failed, 0 skipped

CARGO_TARGET_DIR=/private/tmp/holla-home-row-target-20261021 rustup run 1.99.1 cargo clippy --locked -p holla-ui --all-targets -- -D warnings
passed

rustup run 1.99.1 cargo fmt --all --check
passed
```

The broader package command
`cargo nextest run --locked -p holla-ui --all-targets --no-fail-fast`
finished with 144 passed, 1 failed, and 0 skipped. The failure is
`holla-ui::visual::holla_visual_baseline`: `first-use 80 24 junie truecolor`
changed from `e30768d9339a0fb9` to `d737412561a7e6fa`. The protected expected
artifact was not changed. This is an unresolved cell-level conflict until the
paired frozen-reference capture is run and independently reviewed; this
increment does not claim visual acceptance.

## Limits

These checks establish the shared row path and current Holla UI interaction
contract. They do not establish complete frozen-baseline parity. The protected
Holla visual digest failure remains unresolved pending paired frozen-binary
capture and independent review. Full route inventory and the other Home
presentation paths also remain required before Holla can be accepted.

## Frozen Home/Finder comparison

The component mapping is established for the first-use and Docker-cleanup
scenarios: the frozen `App::for_scenario` starts in `FinderPage`, while the
current CLI's `tag_scenario_names_select_the_baseline_screen` maps those same
scenario names to `Screen::Finder`, whose route is Home. This makes the frozen
Finder frames a valid reference for these Home rows.

Using the pinned Tuiscotti visual test suite through
`tools/frozen_baseline.py compare`, the frozen replay passed for first-use at
80×24, 100×30, 120×40, and 160×50 (truecolor), and Docker-cleanup at 120×40.
Candidate comparison failed each case on exactly 16 cells, all on row 0: the
`File`, `Go`, and `Help` menu labels gained the bold attribute. No other cells
differed. In the 80×24 first-use frame, the selected marker is at x=2 and the
row title begins at x=4 in both frozen and candidate frames. The proposed
one-cell row-origin discrepancy is therefore not present in these actual
Home/Finder samples; adding `.content_gap(0)` would change the shared component
layout without evidence. The row-origin hypothesis is rejected for these
scenarios and sizes.

The frozen Finder source sizes a risk column only from `Destructive` or
`Privileged` items, and paints risk text only for those items. The first-use
frames contain no visible row risk text; the Docker-cleanup frame contains
`destructive` only beside the destructive actions. The candidate matches all
row cells in both compared journeys, so the hypothesized extra row-risk text
is not observed for these identical fixtures. The broader risk-policy claim
remains unresolved: `parity-delete-safety` is not an equivalent-data
comparison on this branch. Its frozen frame contains Cargo actions including
destructive rows, while the candidate frame contains only “Monitor this host”
and differs in 1,322 cells. That is a fixture/content parity gap outside this
row-composition increment, not evidence for changing the row painter.

The live-pair runner was also attempted via `prepare-live`, but correctly
refused this local build: its Holla binary hash is
`ea914394154c6ccb9f5c29f4459c8852d3f17c5a7b257b82b4fac833364ca112`, while
the fixed reference-decision record requires
`6dd87a4e21dff2c69bc2b102234d0f37a5ddcd5c3f2ecd42114a825b2037e01c`. The
binary-identity guard was not bypassed. Thus these comparisons replay the
frozen committed frames and prove their frozen phase passes, but do not claim
a fresh capture from a separately qualified reference binary. Preserve this
qualification gap and the existing expected artifacts.

The 16-cell menu-label bold difference is a plausible explanation for the
separate Holla visual digest red, but the CLI-frame comparison and the
`holla-ui::visual::holla_visual_baseline` harness are distinct paths. It is not
yet established that this menu-only delta accounts for digest change
`e30768d9339a0fb9` → `d737412561a7e6fa`; that original red remains unresolved.
