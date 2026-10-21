# Steps custom-row intrinsic measurement

The independent review of `17841c8228c81250d8e457422b12f7c1233283ec`
found that the new `Display` bound on `composition::steps` broke custom row
consumers whose view data has no `Display` implementation. The public
non-Display consumer failed to compile with E0277, while the same consumer
compiled on `c0408f34fe86d03802c59f961c3781aa7215558a`.

The fix keeps the composition adapter generic. `Steps::measure_label_width`
accepts an optional pure projection of the custom row's label width in terminal
cells. Measurement never invokes the row painter or the application state
projection. If a custom row does not supply a width projection, measurement
uses the established 12-cell minimum; consumers whose custom labels need more
space provide the matching width projection. Borrowed metadata and ordinal
widths remain measured from the live items. Legacy `Steps::measure` is
unchanged.

The consumer regression uses non-Display `ImportStep` view data and a custom
row painter. It checks the original 43-cell label/metadata case, 44-cell
three-digit ordinal case, and an independently longer custom label whose
measured terminal-cell width matches the rendered label.

The follow-up review of `8ac27e4307cf545efdb24ea5db05b831a7b44669` found the
12-cell fallback also applied to the built-in `DefaultRow`, clipping a live
34-cell Display label despite a 100-cell root. Measurement now asks the
source-compatible `RowFn::intrinsic_label_width` hook after any explicit
`measure_label_width` projection. Custom row callbacks keep the default
`None` implementation, so non-Display data stays generic and retains the
12-cell fallback unless it supplies the explicit projection. `DefaultRow`
implements the hook by formatting its Display value and measuring shared
terminal-cell width; measurement still does not call the painter, step-state
projection, input handler or domain action. The public consumer regression
`public_default_display_steps_measure_the_actual_label_without_custom_projection`
checks the full label, metadata, and measured intrinsic width. It passes after
the fix; the same public test was red on the pre-fix source with width 48 and
the visible label clipped to `Archive hist` (independent review packet
`steps-intrinsic-review-8ac27e43.json`).

This change restores API compatibility and intrinsic measurement behavior. It
does not establish focus ownership, baseline application adoption, or full
visual/functional parity. The existing independently reported
`item_row_columns::constructor_defaults_match_the_pre_policy_full_cell_fixture`
navigation test failure remains unresolved and is not waived.
