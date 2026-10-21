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

## Limits

These checks establish the shared row path and current Holla UI test contract.
They do not establish complete frozen-baseline parity. Paired frozen-binary
captures, full route inventory, and the other Home presentation paths remain
required before Holla can be accepted.
