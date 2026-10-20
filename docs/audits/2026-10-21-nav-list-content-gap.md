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

## Cursor presentation extension

`NavList::cursor_style` remains `Bar` by default. Opt-in `Fill` uses the
existing RowUi fill policy and suppresses only the inherited focused gutter;
real `FOCUSED` remains present for action routing. Explicit theme or part
glyph Set/Clear and the named slot continue to win. A focused regression
drives the real runtime, checks the inherited `▎` versus blank gutter, verifies
the row region is unchanged, and activates by Enter. Override checks cover
TrueColor, ANSI256, ANSI16 and Mono.

The additive `RowUi::new_with_author_style` and fixed leading/trailing-part
hooks reuse `author::PartStyle` and canonical `resolve_row_part`. NavList
limits those instance patches and slots to `NavList::PARTS`, keeping caller
owned metadata/cells unaffected. The default prefix remains gutter, marker,
icon and gap; badge width and reservation still precede the custom row.
CONTAINER is more specific than ROW when both provide explicit fill patches;
theme/overlay/instance precedence remains in the shared Ui resolver. Existing
`new`/`new_with_patches` constructors and their owned patch parameters remain
available.

## Remaining presentation work

These changes only add a configurable content gap and the shared Fill adapter.
They do not establish the complete frozen Holla presentation contract or
application parity. Default Bar still requires independent complete-cell and
region comparison against pinned `fa5891532c8d32d182f83e299436d467bdc0df20`;
the current local assertions are focused regressions, not that comparison.

The nearest navigation suites report one existing failure:
`item_row_columns::constructor_defaults_match_the_pre_policy_full_cell_fixture`
(`17020103994864682368` vs expected `12184706452759668872`). It reproduces
unchanged at the exact pre-Fill commit `fa5891532c8d32d182f83e299436d467bdc0df20`,
so it predates this increment and is not waived. The other 563 tests in the
runtime/navigation aggregate passed; focused NavList and RowUi regressions
also pass.

## Verification for Fill adapter increment

`cargo nextest run -p termrock-navigation --test nav_list_row_policy --test nav_list_cursor_style --no-fail-fast` passed 9/9. `cargo clippy -p termrock-runtime -p termrock-navigation --all-targets -- -D warnings`, scoped rustfmt check, and `git diff --check` pass. The unfiltered command `cargo nextest run -p termrock-runtime -p termrock-navigation --no-fail-fast --status-level fail --final-status-level fail` ran 564 cases: 563 passed, 1 failed, 3 skipped. The sole failure above is also red at `fa5891532c8d32d182f83e299436d467bdc0df20` under the same fixture test, with identical fingerprints, so it is a confirmed pre-existing failure rather than a waived result.
