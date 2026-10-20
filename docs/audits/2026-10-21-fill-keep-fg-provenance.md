# `fill_keep_fg` semantic foreground provenance

The independently reviewed `1ceed23` consumer probe showed that `Ui::fill_keep_fg`
produced the same pre-dim cells as a background-only `Ui::fill`, but its
`mark_area(area, None)` discarded the retained foreground role. A following
`dim_layer(1)` therefore resolved that cell as unknown (`RGB 38`) while the
equivalent background-only fill retained the semantic primary foreground
(`RGB 179`). The external probe failed on the old implementation and passes
after `fill_keep_fg` delegates to `Ui::fill(PaintStyle::new().bg(background))`.

The regression covers page and modal-layer targets, four color profiles,
clipping, Reset/black/custom backgrounds, retained modifiers, and a later dim
pass. No screen-specific painting or caller correction is involved.

`recolor_rgb` and `recolor_fg` still use `mark_area(area, None)`. Their semantic
role behavior after changing raw color values is a separate unresolved contract;
this change does not alter either method. No application adoption or product
parity claim follows from this runtime fix.
