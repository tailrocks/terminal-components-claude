# The finder plate and the finder screen become shared blocks (2026-10-21)

This audit records the shared blocks that the Holla home screen and its shell
composition moved into `termrock-shell`. It does not claim application or
frozen-baseline parity beyond the frames the tests cover.

## Public surface and ownership

- `termrock_shell::finder` — the content model (`FinderRow`, `FinderLine`,
  `FinderPreview`, `FinderTone`, `FinderContent`), the band policy
  (`FinderBand`, `finder_band`, `FinderColumns`, `FinderPreviewTitle`,
  `FinderLayout`, `FinderPolicy`) and the block (`FinderPlate`).
  `FinderPlate::draw` picks the layout the policy names for the band the
  terminal is in and reports whether it painted.
- `termrock_shell::finder_screen` — the grid (`FinderGrid`, with `area`,
  `query_area`, `content_area`, `content_below`, `draw_query`, `draw_empty`,
  `draw_note`, `note_fits`), the lead (`FinderLead`), the scope band
  (`ScopeBand`, `BandChip`, `BandMarker`) and the action row (`ActionRow`,
  `paint_action_row`).

The blocks own the geometry, the ink and the size policy. The app owns the
words, the rows and the policy: `holla-ui/src/screens/home.rs` names the
layout each plate uses in each band and maps the catalogue rows onto the shared
content.

## The band policy

`finder_band` is the single source of truth for the finder bands: below 80
columns no plate fits, 80 to 119 columns is the single-column band, 120 to 159
columns is the split master/detail band and 160 columns and up is the plain
band. The app no longer compares a width: the plate reads the band itself.

The catalogue in `crates/holla-sim` still compares raw widths to choose the
*content* of a view. It cannot use `finder_band` yet: `holla-sim` has no
dependency on a Termrock crate. Until it has one, the thresholds are written
down twice — once in `termrock_shell::finder` for the layout and once in
`crates/holla-sim/src/catalogue.rs` for the content. The two must agree.

## Evidence

The frame-equality tests in `crates/holla-ui` render the pre-refactor painters,
kept verbatim in `screens::home::plate_legacy` and `app::band_legacy`, and the
shared blocks, and compare the cells, the styles and the cursor:

- `screens::home::plate_legacy::tests::the_shared_plate_paints_the_cells_of_the_app_owned_painters`
  covers 15 scenarios, 6 widths, 3 heights and 4 colour levels.
- `screens::home::plate_legacy::tests::the_shared_action_row_paints_the_cells_of_the_app_owned_painter`
  covers every action row of every scenario at two focus states and three
  widths.
- `app::band_legacy::tests::the_shared_scope_band_paints_the_cells_of_the_app_owned_painters`
  covers 5 frames and 4 colour levels.

`crates/holla-ui/tests/visual.rs` keeps the frozen visual baseline green, and
`app::independent_draw_review` keeps the whole-screen cells stable.

## Open

- `crates/holla-ui/src/screens/home.rs` still asks
  `world.scenario.boot_title()` whether the finder leads with the recent list.
  The decision belongs to the catalogue: it needs a
  `catalogue::home_lead(&World) -> HomeLead` that names the lead, so the
  screen stops reading the scenario.
- The frozen reference plates of the disk cleanup and the launch failure
  screens still live in `crates/holla-ui/src/screens/home.rs` and still choose
  their layout by comparing the terminal size. They are their own family and
  want a frozen-plate block.
- The per-plate status and footer captions of the shell chrome still live in
  `crates/holla-ui/src/app.rs` and still compare the terminal size.
