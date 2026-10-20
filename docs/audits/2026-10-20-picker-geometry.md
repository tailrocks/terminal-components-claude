# Picker geometry audit — 2026-10-20

The frozen searchable Picker computes its height as `2 + 1 + 2 + rows + 2`:
two border rows, a title, a two-row query/gap region, the visible row capacity,
and two trailing content rows. The shared Picker had used `rows + 5`, which
made the one-item TablePro picker 64×6 instead of 64×8. Its list viewport also
included a cell that the frozen layout reserves below the visible row.

The runtime intentionally defines `ScreenAlign::Center` as upper-third placement
and `ScreenAlign::GeometricCenter` as the geometric center. The frozen Picker
defaults to centered placement, so the shared Picker now defaults to
`GeometricCenter`; explicitly supplied alignment values pass through unchanged.
Other overlay anchors retain their runtime semantics.

Focused runtime-backed tests check the one-item 64×8 frame at `(28, 16)` in a
120×40 viewport, title and row positions, resize to 80×24, width capping at
40×12, empty/many-row measurement, and theme/explicit row limits. The Picker
continues to use its live `FilterList` renderer; no captured frame or
application-specific geometry path is involved.
