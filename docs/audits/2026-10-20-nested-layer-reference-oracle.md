# Nested layer reference oracle correction

Reference: frozen source commit `4169783ae883788186d161af09ba9c481bd84e5c`.

`nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells`
previously asserted that ordinary `X` text became a space and that raw-painted
`R` received a black background. The test itself did not establish either
expectation from the frozen reference.

The frozen `Theme::backdrop` implementation at `src/theme.rs:1243-1310` keeps
glyphs and maps a non-canvas background to `surface_overlay`. Frozen Junie
tokens at `src/theme.rs:734-755` define Ghost as `#262626`, Overlay as
`#27272a`, muted as `#8a8a8a`, and primary as white. Applying that code to the
scenario's explicit styles yields these full cells:

| Cell | Symbol | Foreground | Background | Underline | Modifier |
|---|---|---|---|---|---|
| lower ordinary paint | `X` | `#262626` | `#27272a` | default | none |
| lower raw paint | `R` | `#262626` | `#27272a` | default | none |
| dimmed unwritten page | `P` | `#8a8a8a` | `#000000` | default | none |
| top modal paint | `U` | `#ffffff` | terminal default | default | none |

The regression asserts those cell properties for both layer orders and both
paint paths, plus absence of a cursor. It makes the former compositor behavior
fail: before correction, focused `cargo nextest` failed at the `X` assertion
with actual `"X"` and the obsolete expected `" "`. No production code,
snapshot, stored approval, or aggregate fingerprint changed.

Independent corroboration: reviewer `section_review` diagnosed this stale
expectation from the frozen `ctx.rs:658-688` and `theme.rs:1243-1302` behavior.
