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

This change restores API compatibility and intrinsic measurement behavior. It
does not establish focus ownership, baseline application adoption, or full
visual/functional parity. The existing independently reported
`item_row_columns::constructor_defaults_match_the_pre_policy_full_cell_fixture`
navigation test failure remains unresolved and is not waived.
