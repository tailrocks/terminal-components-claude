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

`cargo nextest run -p termrock-navigation --test nav_list_row_policy --test nav_list_cursor_style --no-fail-fast` passed 9/9 before adding the pinned Bar frame test; the final focused set passes 10/10. `cargo clippy -p termrock-runtime -p termrock-navigation --all-targets -- -D warnings`, scoped rustfmt check, and `git diff --check` pass. The unfiltered command `cargo nextest run -p termrock-runtime -p termrock-navigation --no-fail-fast --status-level fail --final-status-level fail` ran 565 cases: 564 passed, 1 failed, 3 skipped. The sole failure above is also red at `fa5891532c8d32d182f83e299436d467bdc0df20` under the same fixture test, with identical fingerprints, so it is a confirmed pre-existing failure rather than a waived result.

## Default Bar preservation regression found and fixed

A direct `Harness` comparison of the same 24x4 default-Bar scene at the pinned pre-Fill source and this candidate initially differed (frame digests `4491769324398299356` vs `11337825009153179326`) despite identical registered area and row rectangles. Cell inspection isolated the difference to unpainted component-author `ICON` fallback cells: NavList's prior renderer filled an inherited part with the resolved part style even without a fallback glyph, while the adapter left the row-container paint visible. `RowUi::{leading_part,trailing_part}` now fill the part style before inherited fallback rendering; authored slots, glyph Set/Clear, and their precedence are unchanged. `nav_list_bar_fingerprint::pinned_pre_fill_bar_frame_and_regions_remain_unchanged` pins the complete snapshot digest and both registered rectangles from `fa5891532c8d32d182f83e299436d467bdc0df20`. It passes alongside the Fill and gap regressions (10/10).

## Existing RowUi Fill compatibility regression found and fixed

Independent review of `0333cde2c6176e2c9e4639ce7f222d70bb90bbcb` found that routing `RowUi::gutter()` through the new generic part hook changed blank-cell foreground/modifiers for the existing public `RowUi::new(...).with_cursor_style(Fill).gutter()` API in all 20 combinations of color profile and flags. The reviewer packet is `nav-list-0333cde2-independent-review.json` (SHA-256 `e1c8b420e9aca28a8162e7aa95abf8c63ac2a2d61ec7183d5ac7b7313494c3c4`). An unchanged public RowUi probe failed before repair at `TrueColor/empty` (candidate white foreground versus frozen black-on-black), while the new regression test enumerates all four profiles and five states against the pre-existing Fill gutter role/modifier contract. The shared resolver now selects those defaults through `style_defaults_with_explicit` metadata, then applies canonical style resolution with authored theme/overlay/instance patches above defaults. Set/Clear glyphs and named slots remain authoritative; actual `FOCUSED` flags are not altered. The regression now passes. Existing default-Bar and custom-part cross-version corpus evidence from that review packet remains intact; the ItemRow fixture failure recorded above remains separate and unresolved.

## Verification after restoring RowUi Fill defaults

Before the fix, the added 20-case RowUi test failed at `TrueColor/empty` with foreground white instead of the pre-existing black gutter style. After the fix, `cargo nextest run -p termrock-runtime existing_fill_gutter_preserves_legacy_blank_cell_style_for_every_state_and_profile --no-fail-fast` passed, and the NavList gap/Fill/Bar fingerprint set passed 10/10. Strict scoped Clippy for runtime, navigation, and the public `termrock` facade passed; scoped rustfmt and `git diff --check` passed. The all-target run `cargo nextest run -p termrock-runtime -p termrock-navigation --all-targets --features termrock-runtime/testing --no-fail-fast --status-level fail --final-status-level fail` ran 566 cases: 565 passed, 1 failed, 3 skipped. The sole failure remains the independently reproduced ItemRow fixture mismatch at the pre-Fill `fa589` source and is neither fixed nor waived here.
